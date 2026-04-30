use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;
use coh_npe::closure::LeanClosureStatus;

pub struct NoSorryScanner;

impl NoSorryScanner {
    /// Scan a Lean file for 'sorry', 'admit', or other proof gaps.
    pub fn scan_file(path: &Path) -> Result<LeanClosureStatus, std::io::Error> {
        let content = fs::read_to_string(path)?;
        
        if content.contains("admit") {
            return Ok(LeanClosureStatus::BuildPassedWithAdmit);
        }
        
        if content.contains("sorry") {
            return Ok(LeanClosureStatus::BuildPassedWithSorry);
        }
        
        // In a more advanced version, we would also run 'lake build' and check for
        // specific 'axiom' or 'untrusted' labels.
        Ok(LeanClosureStatus::ClosedNoSorry)
    }

    /// Scan the output of a lake build command for warnings about sorry/admit.
    pub fn scan_build_output(stdout: &str, stderr: &str) -> LeanClosureStatus {
        let combined = format!("{} {}", stdout, stderr);
        
        if combined.contains("error:") {
            LeanClosureStatus::BuildFailed
        } else if combined.contains("admit") {
            LeanClosureStatus::BuildPassedWithAdmit
        } else if combined.contains("sorry") {
            LeanClosureStatus::BuildPassedWithSorry
        } else {
            LeanClosureStatus::ClosedNoSorry
        }
    }

    /// Execute a Lean build with a hard timeout and output cap.
    pub fn build_with_guard(
        lake_path: &str,
        project_path: &Path,
        target: &str,
        timeout: Duration,
    ) -> Result<(LeanClosureStatus, String), String> {
        let mut child = Command::new(lake_path)
            .args(["build", target])
            .current_dir(project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn lake: {}", e))?;

        let status = match child.wait_timeout(timeout) {
            Ok(Some(status)) => status,
            Ok(None) => {
                child.kill().ok();
                return Ok((LeanClosureStatus::ToolchainError, "Build timed out".to_string()));
            }
            Err(e) => {
                child.kill().ok();
                return Err(format!("Error waiting for lake: {}", e));
            }
        };

        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let combined = format!("{}\n{}", stdout, stderr);
        let closure = if status.success() {
            Self::scan_build_output(&stdout, &stderr)
        } else {
            LeanClosureStatus::BuildFailed
        };

        Ok((closure, combined))
    }

    /// Find the Coh workspace root by looking for '.lake' or 'coh-node'.
    pub fn find_workspace_root() -> Result<std::path::PathBuf, String> {
        let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
        let mut path = current_dir.as_path();
        
        while let Some(parent) = path.parent() {
            if path.join("coh-node").exists() || path.join("coh-t-stack").exists() {
                return Ok(path.to_path_buf());
            }
            path = parent;
        }
        
        Err("Could not find workspace root".to_string())
    }

    /// Resolve a path relative to the workspace root.
    pub fn resolve_path(relative: &str) -> Result<std::path::PathBuf, String> {
        let root = Self::find_workspace_root()?;
        Ok(root.join(relative))
    }
}
