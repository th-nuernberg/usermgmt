use crate::prelude::*;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
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
            CurrentSelectedView::SshConnection => text_design::button::SSH_CONNECTION,
            CurrentSelectedView::LdapConnection => text_design::button::LDAP_CONNECTION,
            CurrentSelectedView::Configuration => text_design::button::CONFIGURATION,
            CurrentSelectedView::Listing => text_design::button::LISTING,
            CurrentSelectedView::Adding => text_design::button::ADDING,
            CurrentSelectedView::Removing => text_design::button::REMOVING,
            CurrentSelectedView::Modifing => text_design::button::MODIFING,
        }
    }
}

impl Default for CurrentSelectedView {
    fn default() -> Self {
        Self::Configuration
    }
}
