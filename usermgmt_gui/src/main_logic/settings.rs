use crate::prelude::*;
use serde::Deserialize;

#[cfg(debug_assertions)]
mod development;
#[cfg(debug_assertions)]
pub use development::DebugSettingWatcher;

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub ldap_multi_field_sep: String,
    pub username_label: String,
}
