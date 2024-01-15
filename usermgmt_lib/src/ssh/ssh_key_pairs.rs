use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SshKeyPair {
    pub_key: PathBuf,
    private_key: PathBuf,
}

impl SshKeyPair {
    pub fn from_one_path(mut pub_key: PathBuf) -> Self {
        let private_key = pub_key.clone();
        const PUB: &str = "pub";
        if let Some(os_string) = pub_key.extension() {
            if os_string != PUB {
                pub_key.set_extension(PUB);
            }
        } else {
            pub_key.set_extension(PUB);
        }

        Self {
            pub_key,
            private_key,
        }
    }
    pub fn pub_key(&self) -> &Path {
        &self.pub_key
    }
    pub fn private_key(&self) -> &Path {
        &self.private_key
    }
}
