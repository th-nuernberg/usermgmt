pub const WINDOW_TITLE: &str = "Usermgmt";
pub const MODE_MAINT_TITLE: &str = "On which system";
pub const MODE_LDAP: &str = "LDAP";
pub const MODE_SLURM: &str = "Slurm";
pub const MODE_DIRECTORY: &str = "Directory";

pub mod group {
    pub const SSH_CRED: &str = "SSH credentials";
    pub const LDAP_CRED: &str = "LDAP credentials";
    pub const DIR_CONF_PATH: &str = "Path to directory with the file called conf.toml";
    pub const GENERAL_STATUS: &str = "Status";
    pub const REQUIRED: &str = "Required";
    pub const OPTIONAL: &str = "Optional";
    pub const READONLY_LDAP_CRED: &str = "Ldap readonly credentials";
    pub const STATUS_LIST_SLURM: &str = "Status of listing Slurm users";
    pub const STATUS_LIST_LDAP: &str = "Status of listing LDAP users";
}

pub mod label {
    pub const USERNAME: &str = "Username";
    pub const PASSWORD: &str = "Password";
    pub const FIRSTNAME: &str = "Firstname";
    pub const LASTNAME: &str = "Lastname";
    pub const MAIL: &str = "Email";
    pub const QOS: &str = "Quality of Services";
    pub const DEFAULT_QOS: &str = "Default Quality of Services";
    pub const PUBLIC_KEY: &str = "Default Quality of Services";
    pub const GROUP: &str = "User group";
}

pub mod button {
    pub const ACTION_ADD: &str = "Add User";
    pub const ACTION_REMOVE: &str = "Remove User";
    pub const ACTION_MODIFY: &str = "Modify User";
    pub const LIST_LDAP_USERS: &str = "List LDAP users";
    pub const LIST_SLURM_USERS: &str = "List Slurm users";

    pub const LIST_REMOVE: &str = "Remove";

    pub const SSH_CONNECTION: &str = "Ssh connection";
    pub const LDAP_CONNECTION: &str = "Ldap connection";
    pub const CONFIGURATION: &str = "Configuration";
    pub const LISTING: &str = "Listing";
    pub const ADDING: &str = "Adding";
    pub const REMOVING: &str = "Removing";
    pub const MODIFING: &str = "Modify";
    pub const NEW_ITEM: &str = "Add new item";
}
pub mod error_messages {
    pub const FAILED_PARSING_SLURM: &str = "Could not parse slurm users to a table";
    pub const LDAP_CRED_MISSING: &str = "LDAP credentials are missing.";
    pub const SSH_CRED_MISSING: &str = "Ssh credentials are missing.";
}

pub mod create_msg {
    use std::fmt::{Debug, Display};

    pub fn error_status<T>(msg: &str, error_details: T) -> String
    where
        T: Display + Debug,
    {
        format!("{}. Details: \n{:?}", msg, error_details)
    }

    pub fn not_implemented_action(action_name: &str) -> String {
        format!("The action {} is not implemented yet", action_name)
    }
    pub fn listing_slurm_init() -> String {
        "No slurm user listed yet.".to_string()
    }
    pub fn listing_slurm_loading() -> String {
        "Fetching slurm users".to_string()
    }
    pub fn listing_slurm_success() -> String {
        "Fetched Slurm users".to_string()
    }
    pub fn listing_slurm_failure() -> String {
        "Failed to fetch Slurm users".to_string()
    }
    pub fn listing_ldap_init() -> String {
        "No LDAP user listed yet.".to_string()
    }
    pub fn listing_ldap_loading() -> String {
        "Fetching LDAP users".to_string()
    }
    pub fn listing_ldap_success() -> String {
        "Fetched LDAP users".to_string()
    }
    pub fn listing_ldap_failure() -> String {
        "Failed to fetch LDAP users:".to_string()
    }
    pub fn modify_init() -> String {
        "No user modified yet:".to_string()
    }
    pub fn modify_loading(user: &str) -> String {
        format!("Modifieng user ({}):", user)
    }
    pub fn modify_success(user: &str) -> String {
        format!("Modified user ({}) successfully:", user)
    }
    pub fn modify_failure(user: &str) -> String {
        format!("Failed to modify user ({}):", user)
    }
}
