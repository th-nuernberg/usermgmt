use crate::prelude::*;

use super::{SshKeyPair, SshPublicKeySuggestion};

pub trait SshCredentials: Clone {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
    fn ssh_paths_pair_key(&self) -> Option<&SshKeyPair>;
    fn auth_agent_resolve(&self, _many_keys: Vec<SshPublicKeySuggestion>) -> AppResult<usize> {
        bail!("No resolving for serveral keys implemented");
    }
}
