use crate::prelude::AppResult;
use anyhow::{anyhow, Context};

/// Allows to collect several errors messages which can be later combined into one error variant for
/// error propagation. If no error message is collected, it resolves to an Ok variant.
pub struct ResultAccumulator {
    errs: Vec<String>,
    base_err_msg: String,
}

impl ResultAccumulator {
    pub fn new(error_msg: String) -> Self {
        Self {
            errs: Default::default(),
            base_err_msg: error_msg,
        }
    }

    /// Collects the given error message as the parameter "err_msg" if the parameter "condition" is
    /// false
    pub fn add_err_if_false(&mut self, condition: bool, err_msg: String) {
        if !condition {
            self.errs.push(err_msg)
        }
    }

    /// Collects the given error message as the parameter "err_msg"
    pub fn add_err(&mut self, err_msg: String) {
        self.errs.push(err_msg)
    }
}

impl From<ResultAccumulator> for AppResult {
    fn from(value: ResultAccumulator) -> Self {
        if value.errs.is_empty() {
            return Ok(());
        }

        let all_errs = Err(anyhow!("{}", value.base_err_msg));

        all_errs.context(value.errs.join("\n"))
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn resolve_to_ok_for_no_errors() {
        let mut accumulator = ResultAccumulator::new("Should not resolve to an error.".to_owned());
        accumulator.add_err_if_false(true, "...".to_owned());
        let result = AppResult::from(accumulator);
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_to_errors() {
        let mut accumulator = ResultAccumulator::new("Should not resolve to an error.".to_owned());
        accumulator.add_err_if_false(false, "false".to_owned());
        accumulator.add_err_if_false(true, "true".to_owned());
        accumulator.add_err("added".to_owned());
        let result = AppResult::from(accumulator);
        insta::assert_debug_snapshot!(result.err().unwrap());
    }
}
