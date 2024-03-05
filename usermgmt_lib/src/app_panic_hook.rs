use crate::prelude::*;
const SEP: &str = "==============================================";
/// After calling this, the end of every panic message is appended with a notice of how to report a
/// bug as an end user, a link to the issue page is included.
/// In a perfect world, panic messages should not be triggered.
/// If they occur however then the end user knows where to report this issue.
pub fn set_app_panic_hook() {
    use std::panic;

    panic::set_hook(Box::new(|panic_info| {
        // Reuse default panic by printing "panic_info"
        println!("{panic_info}\n{SEP}\n{}", constants::BUG_REPORT);
    }));
}
