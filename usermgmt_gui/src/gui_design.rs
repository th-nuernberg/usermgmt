pub mod text_design;

pub const WINDOW_START_HEIGHT: f32 = 640.0;
pub const WINDOW_START_WIDTH: f32 = 480.0;
pub const WHICH_GUI_VIEW_SIZE: f32 = 20.0;
pub const MAX_HEIGHT_LDAP_TABLE: f32 = 200.0;
pub const HEADER_HEIGHT_LDAP_TABLE: f32 = 20.0;
pub const LDAP_MULTI_FIELD_SEP: &str = "| ";

pub mod colors {
    use eframe::epaint::Color32;

    pub const ERROR_MSG: Color32 = Color32::RED;
    pub const INIT_MSG: Color32 = Color32::GRAY;
    pub const LOADING_MSG: Color32 = Color32::BLUE;
    pub const SUCCESS_MSG: Color32 = Color32::GREEN;
}
