//! Manager Agent orchestration engine for `mag`.

pub mod diagnostics;
pub mod evaluator;
pub mod orchestrator;

pub use diagnostics::{EnvDoctor, JpCargoAnalyzer};
pub use evaluator::{EvaluationVerdict, ResultEvaluator};
pub use orchestrator::{Orchestrator, OrchestratorError};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_orchestrator_lifecycle() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.sqlite");

        let orch = Orchestrator::new(dir.path(), &db_path).unwrap();
        let tasks = orch.decompose_requirement("Create hello library in Rust").unwrap();
        assert_eq!(tasks.len(), 5);

        let success = orch.run_orchestration_loop(Some(dir.path()), 10).unwrap();
        assert!(success);
    }
}
