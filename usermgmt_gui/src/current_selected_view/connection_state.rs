use std::path::Path;

pub trait ConnectionState {
    fn username(&self) -> Option<&str>;
    fn password(&self) -> Option<&str>;
    fn ssh_key_pair(&self) -> Option<&Path>;

    fn all_fields_filled(&self) -> Option<(&str, &str)> {
        match (self.username(), self.password()) {
            (Some(username), Some(password)) => Some((username, password)),
            _ => None,
        }
    }

    fn username_maybe_password(&self) -> Option<(&str, Option<&str>)> {
        match (self.username(), self.password()) {
            (Some(username), Some(password)) => Some((username, Some(password))),
            (Some(username), None) => Some((username, None)),
            _ => None,
        }
    }

    fn are_fields_filled(&self) -> bool {
        self.all_fields_filled().is_some()
    }
}
