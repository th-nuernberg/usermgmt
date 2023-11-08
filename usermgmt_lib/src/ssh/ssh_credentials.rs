use crate::prelude::*;

use super::SshPublicKeySuggestion;

pub trait SshCredentials: Clone {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
    fn auth_agent_resolve(&self, _many_keys: Vec<SshPublicKeySuggestion>) -> AppResult<usize> {
        bail!("No resolving for serveral keys implemented");
    }
}
