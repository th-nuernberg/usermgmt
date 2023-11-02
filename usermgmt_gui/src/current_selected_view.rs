mod add_state;
mod configuration_state;
mod listing_state;
mod ssh_connection_state;

mod connection_state;
mod ldap_connection_state;
mod remove_state;

pub use add_state::AddState;
pub use configuration_state::ConfigurationState;
pub use connection_state::ConnectionState;
pub use ldap_connection_state::LdapConnectionState;
pub use listing_state::ListingState;
pub use remove_state::RemoveState;
pub use ssh_connection_state::SshConnectionState;

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
    pub fn create_str(&self) -> &'static str {
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
