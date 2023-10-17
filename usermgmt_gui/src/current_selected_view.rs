mod configuration_state;
mod listing_state;

pub use configuration_state::ConfigurationState;
pub use listing_state::ListingState;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurrentSelectedView {
    SshConnection,
    LdapConnection,
    Configuration,
    Listing,
    Adding,
    Removing,
    Modifing,
}

impl CurrentSelectedView {
    pub fn to_str(&self) -> &'static str {
        match self {
            CurrentSelectedView::SshConnection => "Ssh connection",
            CurrentSelectedView::LdapConnection => "Ldap connection",
            CurrentSelectedView::Configuration => "Configuration",
            CurrentSelectedView::Listing => "Listing",
            CurrentSelectedView::Adding => "Adding",
            CurrentSelectedView::Removing => "Removing",
            CurrentSelectedView::Modifing => "Modify",
        }
    }
}

impl Default for CurrentSelectedView {
    fn default() -> Self {
        Self::Configuration
    }
}
