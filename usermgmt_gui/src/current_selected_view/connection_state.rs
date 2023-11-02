pub trait ConnectionState {
    fn username(&self) -> Option<&str>;
    fn password(&self) -> Option<&str>;

    fn all_fields_filled(&self) -> Option<(&str, &str)> {
        match (self.username(), self.password()) {
            (Some(username), Some(password)) => Some((username, password)),
            _ => None,
        }
    }

    fn are_fields_filled(&self) -> bool {
        self.all_fields_filled().is_some()
    }
}
