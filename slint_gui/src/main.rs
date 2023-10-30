use std::{rc::Rc, time::Duration};

use log::info;
use slint::{Timer, TimerMode};

use crate::pending_jobs::PendingJobs;

slint::include_modules!();
mod pending_jobs;
mod task;
const TICK_FOR_JOB_CHECK: u64 = 200;

fn main() {
    env_logger::init();
    let app = AppWindow::new().expect("Could not initialize the gui front end of the app.");
    let jobs = Rc::new(PendingJobs::default());
    app.global::<RustBackend>().on_on_listing_ldap_users({
        let jobs = jobs.clone();
        let app = app.as_weak().unwrap();
        move || {
            let listing_job = &jobs.listing_ldap_users;
            if !listing_job.is_thread_running() {
                listing_job.spawn_task(|| {
                    info!("Spawning thread");
                    std::thread::sleep(Duration::from_secs(2));
                    Ok("Success".to_lowercase())
                });
                app.global::<AppState>().set_is_listing_ldap_user(true);
            }
        }
    });

    info!("Starting the app");
    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        Duration::from_millis(TICK_FOR_JOB_CHECK),
        {
            let all_jobs = jobs.clone();
            let app = app.as_weak().unwrap();
            move || {
                if let Some(result) = all_jobs.listing_ldap_users.query_task() {
                    info!("Result: {:?}", result);
                    app.global::<AppState>().set_is_listing_ldap_user(false);
                }
            }
        },
    );
    app.run().expect("Could not start the gui main window.");
}
