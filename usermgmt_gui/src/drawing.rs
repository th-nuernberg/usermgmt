pub mod draw_add_state;
pub mod draw_delete_state;
pub mod draw_listing_of_users;
pub mod modify_state;

pub mod about;
pub mod configuration;
pub mod draw_utils;

pub trait ProduceIoStatusMessages<T> {
    fn msg_init(&mut self) -> String;
    fn msg_loading(&mut self) -> String;
    fn msg_success(&mut self, resource: &T) -> String;
    fn msg_error(&mut self) -> String;
}

impl<T, I, L, S, E> ProduceIoStatusMessages<T> for (I, L, S, E)
where
    I: FnMut() -> String,
    L: FnMut() -> String,
    S: FnMut(&T) -> String,
    E: FnMut() -> String,
{
    fn msg_init(&mut self) -> String {
        self.0()
    }

    fn msg_loading(&mut self) -> String {
        self.1()
    }

    fn msg_success(&mut self, resource: &T) -> String {
        self.2(resource)
    }

    fn msg_error(&mut self) -> String {
        self.3()
    }
}
