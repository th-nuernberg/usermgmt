#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurrentSelectedView {
    Nothing,
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
            CurrentSelectedView::Nothing => "Nothing",
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
        Self::Nothing
    }
}
