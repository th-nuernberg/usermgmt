use crate::prelude::*;

use super::{SshKeyPair, SshPublicKeySuggestion};

/// Several functions in this trait return a result
/// to allow for implementer to propagate error in their environment.
/// Example: an implementer for a CLI-App deals with errors and user input within a terminal.
pub trait SshCredentials: Clone {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
    fn ssh_paths_pair_key(&self) -> Option<&SshKeyPair>;
    /// Determines which key from the parameter `_many_keys` to use for a ssh connection.
    fn auth_agent_resolve(&self, _many_keys: Vec<SshPublicKeySuggestion>) -> AppResult<usize> {
        Err(anyhow!("No resolving for several keys implemented"))
    }
}
