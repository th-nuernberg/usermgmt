use crate::prelude::AppResult;

pub trait SshCredentials: Clone {
    fn username(&self) -> AppResult<&str>;
    fn password(&self) -> AppResult<&str>;
    fn auth_resolve(&self) -> bool;
}
