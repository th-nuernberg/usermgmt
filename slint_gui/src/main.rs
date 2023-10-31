use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::Duration,
};

use log::{error, info};
use slint::{ComponentHandle, Timer, TimerMode};

use crate::{io_resources::UnsyncSharedResources, pending_jobs::PendingJobs};

slint::include_modules!();

mod io_resources;
mod pending_jobs;
mod task;

const TICK_FOR_JOB_CHECK: u64 = 200;

fn main() {
    env_logger::init();
    let jobs = Rc::new(PendingJobs::default());
    let io_resources = UnsyncSharedResources::default();
    jobs.load_config
        .spawn_task(|| usermgmt_lib::config::load_config());
    let app = AppWindow::new().expect("Could not initialize the gui front end of the app.");

    let rust_backend = app.global::<RustBackend>();
    rust_backend.on_listing_ldap_users({
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
                app.global::<AppState>()
                    .set_is_listing_ldap_user(LoadStatus::Loading);
            }
        }
    });
    rust_backend.on_loading_configuration({
        let jobs = jobs.clone();
        let app = app.as_weak().unwrap();
        move || {
            let conf_job = &jobs.load_config;
            let app_state = app.global::<AppState>();
            info!("Callback {} is called.", stringify!(on_listing_ldap_users));
            if !conf_job.is_thread_running() {
                app_state.set_configuration(LoadStatus::Loading);
                app_state.set_configuration_status_msg("Configuration is being loaded.".into());
                conf_job.spawn_task(|| usermgmt_lib::config::load_config());
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
            let io_resources = io_resources.clone();
            move || {
                if let Some(result) = all_jobs.listing_ldap_users.query_task() {
                    report_changed_load_status(
                        &app,
                        &result,
                        |app, status| app.set_is_listing_ldap_user(status),
                        |_, _| (),
                        || "Ldap users to be list was returned.",
                    );
                }
                if let Some(result) = all_jobs.load_config.query_task() {
                    report_changed_load_status(
                        &app,
                        &result,
                        |app, status| app.set_configuration(status),
                        |app, msg| app.set_configuration_status_msg(msg.into()),
                        || "Application configuation was loaded.",
                    );
                    if let Ok(loaded) = result.as_ref() {
                        app.global::<AppState>()
                            .set_conf_path(loaded.path.to_string_lossy().to_string().into());
                    }

                    io_resources.borrow_mut().configuration = Some(result);
                }
            }
        },
    );
    app.run().expect("Could not start the gui main window.");
}

fn report_changed_load_status<T, E, S>(
    app: &AppWindow,
    result: &Result<T, E>,
    on_change: impl Fn(&AppState, LoadStatus),
    on_change_msg: impl Fn(&AppState, &str),
    success_msg: impl FnOnce() -> S,
) where
    T: Debug,
    E: Display,
    S: AsRef<str>,
{
    let new_status = result_to_load_status(result);
    let global = app.global::<AppState>();
    match result.as_ref() {
        Ok(_) => {
            let msg = success_msg();
            on_change_msg(&global, msg.as_ref());
            info!("Success in fetching an io resource:\n{}", msg.as_ref())
        }
        Err(to_log) => error!("Error in fetching an io resource:\n{}", to_log),
    }
    on_change(&global, new_status);
}

fn result_to_load_status<T, E>(result: &Result<T, E>) -> LoadStatus {
    if result.is_err() {
        LoadStatus::Failure
    } else {
        LoadStatus::Succees
    }
}
