use std::sync::Arc;

use ssh2::PublicKey;

#[derive(Debug, Clone)]
pub struct SshPublicKeySuggestion {
    comment: Arc<str>,
}

impl From<&PublicKey> for SshPublicKeySuggestion {
    fn from(value: &PublicKey) -> Self {
        Self::new(value.comment())
    }
}

impl SshPublicKeySuggestion {
    pub fn new(comments: &str) -> Self {
        Self {
            comment: comments.into(),
        }
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }
}
