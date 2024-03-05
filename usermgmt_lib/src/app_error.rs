use crate::prelude::*;
use anyhow::anyhow;
use std::process::Output;

/// Converts parameter `output` from spawned os process into a result
/// for easy error propagation
pub fn output_to_result(output: Output) -> AppResult<Output> {
    let status = output.status;
    if status.success() {
        Ok(output)
    } else {
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!(
            "Exit status: {}.\nstderr: {}\nstdout: {}",
            status,
            err,
            out
        ))
    }
}
