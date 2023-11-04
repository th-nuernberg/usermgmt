use strum::AsRefStr;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "title_case")]
pub enum CurrentSelectedView {
    SshConnection,
    LdapConnection,
    Configuration,
    Listing,
    Adding,
    Removing,
    Modifing,
}

impl Default for CurrentSelectedView {
    fn default() -> Self {
        Self::Configuration
    }
}
