//! NPE Proposal Lineage and Graph tracking
//!
//! Tracks the evolutionary history of proposals as they are mutated and refined.

use serde::{Deserialize, Serialize};
#[cfg(feature = "npe-graph")]
use crate::engine::{NpeProposal, ProposalStatus};

#[cfg(feature = "npe-graph")]
use petgraph::{graph::NodeIndex, stable_graph::StableGraph, Directed};

/// Edge type for NPE proposal graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpeEdge {
    /// Mutation type (e.g., "mutate", "simplify", "rewrite")
    pub mutation_type: String,
    /// Score delta (advisory)
    pub score_delta: f64,
    /// Verdict from verifier (if any)
    pub verdict: Option<String>,
}

/// NPE proposal graph using petgraph
#[cfg(feature = "npe-graph")]
pub struct NpeProposalGraph {
    /// The underlying graph
    graph: StableGraph<NpeProposal, NpeEdge, Directed>,
    /// Map from proposal ID to node index
    id_to_index: std::collections::HashMap<String, NodeIndex>,
    /// Root proposal ID (if any)
    root_id: Option<String>,
}

#[cfg(feature = "npe-graph")]
impl NpeProposalGraph {
    /// Create a new proposal graph
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
            id_to_index: std::collections::HashMap::new(),
            root_id: None,
        }
    }

    /// Add a proposal to the graph
    pub fn add_proposal(&mut self, proposal: NpeProposal, parent_id: Option<String>) -> NodeIndex {
        // Add node to graph
        let index = self.graph.add_node(proposal.clone());

        // Update ID map
        self.id_to_index.insert(proposal.id.clone(), index);

        // If there's a parent, add edge
        if let Some(pid) = parent_id {
            if let Some(parent_index) = self.id_to_index.get(&pid) {
                let edge = NpeEdge {
                    mutation_type: "mutate".to_string(),
                    score_delta: 0.0,
                    verdict: None,
                };
                self.graph.add_edge(*parent_index, index, edge);
            }
        }

        // If this is the first node, set as root
        if self.root_id.is_none() {
            self.root_id = Some(proposal.id.clone());
        }

        index
    }

    /// Add a proposal with explicit edge data
    pub fn add_proposal_with_edge(
        &mut self,
        proposal: NpeProposal,
        parent_id: Option<String>,
        edge: NpeEdge,
    ) -> NodeIndex {
        let index = self.graph.add_node(proposal.clone());
        self.id_to_index.insert(proposal.id.clone(), index);

        if let Some(pid) = parent_id {
            if let Some(parent_index) = self.id_to_index.get(&pid) {
                self.graph.add_edge(*parent_index, index, edge);
            }
        }

        if self.root_id.is_none() {
            self.root_id = Some(proposal.id.clone());
        }

        index
    }

    /// Get a proposal by ID
    pub fn get_proposal(&self, id: &str) -> Option<&NpeProposal> {
        self.id_to_index
            .get(id)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    /// Get all accepted proposals
    pub fn accepted_proposals(&self) -> Vec<&NpeProposal> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let p = self.graph.node_weight(idx)?;
                if p.status == ProposalStatus::Accepted {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all rejected proposals
    pub fn rejected_proposals(&self) -> Vec<&NpeProposal> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let p = self.graph.node_weight(idx)?;
                if matches!(p.status, ProposalStatus::Rejected(_)) {
                    Some(p)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get the root ID
    pub fn root_id(&self) -> Option<&str> {
        self.root_id.as_deref()
    }
}

#[cfg(feature = "npe-graph")]
impl Default for NpeProposalGraph {
    fn default() -> Self {
        Self::new()
    }
}
