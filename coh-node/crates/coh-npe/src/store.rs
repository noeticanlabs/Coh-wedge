//! NPE Memory Manifold (Storage)
//!
//! Provides persistent storage for the NPE loop using SQLite.
//! Enabled via the `npe-store` feature.

#[cfg(feature = "npe-store")]
use {
    crate::engine::{NpeProposal, ProposalStatus},
    crate::lineage::NpeEdge,
    rusqlite::{params, Connection, Result as SqlResult},
    serde_json,
    std::path::Path,
};

#[cfg(feature = "npe-store")]
pub struct NpeStore {
    conn: Connection,
}

#[cfg(feature = "npe-store")]
impl NpeStore {
    /// Open or create a new NPE memory manifold database
    pub fn open<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let mut store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Open an in-memory database (useful for testing)
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let mut store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Initialize the database schema
    fn init_schema(&mut self) -> SqlResult<()> {
        let tx = self.conn.transaction()?;

        // Schema version tracking
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        // Check current version
        let current_version: i64 = tx
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            // Version 1: Initial schema

            // Proposals table
            tx.execute(
                "CREATE TABLE IF NOT EXISTS proposals (
                    id TEXT PRIMARY KEY,
                    content TEXT NOT NULL,
                    seed INTEGER NOT NULL,
                    score REAL NOT NULL,
                    content_hash TEXT NOT NULL,
                    depth INTEGER NOT NULL,
                    parent_id TEXT,
                    status_json TEXT NOT NULL
                )",
                [],
            )?;

            // Edges table (Proposal lineage)
            tx.execute(
                "CREATE TABLE IF NOT EXISTS edges (
                    source_id TEXT NOT NULL,
                    target_id TEXT NOT NULL,
                    mutation_type TEXT NOT NULL,
                    score_delta REAL NOT NULL,
                    verdict TEXT,
                    PRIMARY KEY (source_id, target_id),
                    FOREIGN KEY(source_id) REFERENCES proposals(id),
                    FOREIGN KEY(target_id) REFERENCES proposals(id)
                )",
                [],
            )?;

            // Index on content hash for equivalence lookup
            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_proposals_hash ON proposals(content_hash)",
                [],
            )?;

            // Index on status for querying accepted/rejected
            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_proposals_status ON proposals(status_json)",
                [],
            )?;

            tx.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
        }

        if current_version < 2 {
            // Version 2: Memory Ecology (tau and provenance)
            tx.execute(
                "ALTER TABLE proposals ADD COLUMN tau INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
            tx.execute(
                "ALTER TABLE proposals ADD COLUMN provenance TEXT NOT NULL DEFAULT 'SIM'",
                [],
            )?;
            tx.execute("UPDATE schema_version SET version = 2", [])?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Save a proposal to the database
    pub fn save_proposal(&self, proposal: &NpeProposal) -> SqlResult<()> {
        let status_json = serde_json::to_string(&proposal.status)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO proposals 
             (id, content, seed, score, content_hash, depth, parent_id, tau, provenance, status_json) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                proposal.id,
                proposal.content,
                proposal.seed as i64, 
                proposal.score,
                proposal.content_hash,
                proposal.depth,
                proposal.parent_id,
                proposal.tau as i64,
                proposal.provenance,
                status_json,
            ],
        )?;

        Ok(())
    }

    /// Load a proposal by ID
    pub fn load_proposal(&self, id: &str) -> SqlResult<Option<NpeProposal>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, seed, score, content_hash, depth, parent_id, tau, provenance, status_json 
             FROM proposals WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let status_json: String = row.get(9)?;
            let status: ProposalStatus = serde_json::from_str(&status_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let seed_i64: i64 = row.get(2)?;
            let tau_i64: i64 = row.get(7)?;

            Ok(Some(NpeProposal {
                id: row.get(0)?,
                content: row.get(1)?,
                seed: seed_i64 as u64,
                score: row.get(3)?,
                content_hash: row.get(4)?,
                depth: row.get(5)?,
                parent_id: row.get(6)?,
                tau: tau_i64 as u64,
                provenance: row.get(8)?,
                status,
            }))
        } else {
            Ok(None)
        }
    }

    /// Save a lineage edge
    pub fn save_edge(&self, source_id: &str, target_id: &str, edge: &NpeEdge) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO edges 
             (source_id, target_id, mutation_type, score_delta, verdict) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                source_id,
                target_id,
                edge.mutation_type,
                edge.score_delta,
                edge.verdict,
            ],
        )?;
        Ok(())
    }

    /// Get all proposals with a specific status
    pub fn get_proposals_by_status(&self, status_prefix: &str) -> SqlResult<Vec<NpeProposal>> {
        // Simple prefix match since we serialized an enum.
        // For 'Accepted', it's `"Accepted"`. For `Rejected(String)`, it starts with `{"Rejected":`.
        let mut stmt = self.conn.prepare(
            "SELECT id, content, seed, score, content_hash, depth, parent_id, tau, provenance, status_json 
             FROM proposals WHERE status_json LIKE ?1",
        )?;

        let like_pattern = format!("{}%", status_prefix);
        let proposal_iter = stmt.query_map(params![like_pattern], |row| {
            let status_json: String = row.get(9)?;
            let status: ProposalStatus = serde_json::from_str(&status_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let seed_i64: i64 = row.get(2)?;
            let tau_i64: i64 = row.get(7)?;

            Ok(NpeProposal {
                id: row.get(0)?,
                content: row.get(1)?,
                seed: seed_i64 as u64,
                score: row.get(3)?,
                content_hash: row.get(4)?,
                depth: row.get(5)?,
                parent_id: row.get(6)?,
                tau: tau_i64 as u64,
                provenance: row.get(8)?,
                status,
            })
        })?;

        let mut proposals = Vec::new();
        for p in proposal_iter {
            proposals.push(p?);
        }
        Ok(proposals)
    }
}

#[cfg(test)]
#[cfg(feature = "npe-store")]
mod tests {
    use super::*;

    #[test]
    fn test_store_proposal() -> SqlResult<()> {
        let store = NpeStore::open_in_memory()?;

        let proposal = NpeProposal {
            id: "p1".to_string(),
            content: "test content".to_string(),
            seed: 42,
            score: 0.95,
            content_hash: "hash123".to_string(),
            depth: 1,
            parent_id: Some("root".to_string()),
            tau: 100,
            provenance: "EXT".to_string(),
            status: ProposalStatus::Accepted,
        };

        store.save_proposal(&proposal)?;

        let loaded = store.load_proposal("p1")?.unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.content, "test content");
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.score, 0.95);
        assert_eq!(loaded.tau, 100);
        assert_eq!(loaded.provenance, "EXT");
        assert_eq!(loaded.status, ProposalStatus::Accepted);

        // Test non-existent
        let missing = store.load_proposal("nope")?;
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_store_edge() -> SqlResult<()> {
        let store = NpeStore::open_in_memory()?;

        let p1 = NpeProposal {
            id: "p1".to_string(),
            content: "v1".to_string(),
            seed: 1,
            score: 0.5,
            content_hash: "h1".to_string(),
            depth: 0,
            parent_id: None,
            tau: 0,
            provenance: "SIM".to_string(),
            status: ProposalStatus::Generated,
        };

        let p2 = NpeProposal {
            id: "p2".to_string(),
            content: "v2".to_string(),
            seed: 1,
            score: 0.6,
            content_hash: "h2".to_string(),
            depth: 1,
            parent_id: Some("p1".to_string()),
            tau: 1,
            provenance: "DER".to_string(),
            status: ProposalStatus::Generated,
        };

        store.save_proposal(&p1)?;
        store.save_proposal(&p2)?;

        let edge = NpeEdge {
            mutation_type: "refine".to_string(),
            score_delta: 0.1,
            verdict: Some("OK".to_string()),
        };

        store.save_edge("p1", "p2", &edge)?;

        Ok(())
    }

    #[test]
    fn test_query_status() -> SqlResult<()> {
        let store = NpeStore::open_in_memory()?;

        let p1 = NpeProposal {
            id: "p1".to_string(),
            content: "v1".to_string(),
            seed: 1,
            score: 0.5,
            content_hash: "h1".to_string(),
            depth: 0,
            parent_id: None,
            tau: 0,
            provenance: "SIM".to_string(),
            status: ProposalStatus::Accepted,
        };

        let mut p2 = p1.clone();
        p2.id = "p2".to_string();
        p2.tau = 50;
        p2.provenance = "SIM".to_string();
        p2.status = ProposalStatus::Rejected("Too complex".to_string());

        store.save_proposal(&p1)?;
        store.save_proposal(&p2)?;

        let accepted = store.get_proposals_by_status("\"Accepted\"")?;
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].id, "p1");

        let rejected = store.get_proposals_by_status("{\"Rejected\"")?;
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].id, "p2");

        Ok(())
    }
}
