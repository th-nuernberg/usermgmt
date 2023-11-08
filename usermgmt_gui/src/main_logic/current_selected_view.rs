use strum::AsRefStr;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "title_case")]
/// Every variant represents a selectable view.
/// Adding a new variant will automatically draw the button for changing to this view.
/// Note: You still need to implement the drawing of this new view however.
/// The default implementation determines in which view the application starts.
pub enum CurrentSelectedView {
    Configuration,
    Listing,
    Adding,
    Removing,
    Modifying,
    About,
}

impl Default for CurrentSelectedView {
    fn default() -> Self {
        // Start with the configuration.
        // Reason: if there is now configuration then no management of LDAP and Slurm user
        // can not be done.
        Self::Configuration
    }
}
