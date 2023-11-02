use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::Duration,
};

use log::{error, info};
use slint::{
    ComponentHandle, ModelRc, SharedString, StandardListViewItem, TableColumn, Timer, TimerMode,
    VecModel,
};
use usermgmt_lib::ldap::{LDAPConfig, LdapSearchResult, LdapSimpleCredential};

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
        .spawn_task(|| usermgmt_lib::config::load_config(None));
    let app = AppWindow::new().expect("Could not initialize the gui front end of the app.");

    let rust_backend = app.global::<RustBackend>();
    rust_backend.on_listing_ldap_users({
        let jobs = jobs.clone();
        let app = app.as_weak().unwrap();
        let io_resources = io_resources.clone();
        move || {
            log_callback_from_gui("on_listing_ldap_users");
            let listing_job = &jobs.listing_ldap_users;
            if !listing_job.is_thread_running() {
                let app_state = app.global::<AppState>();
                app_state.set_is_listing_ldap_user(LoadStatus::Loading);
                app_state.set_listing_ldap_msg("Fetching LDAP users".into());
                listing_job.spawn_task({
                    let config =
                        if let Some(Ok(ref must_be_here)) = io_resources.borrow().configuration {
                            must_be_here.config.clone()
                        } else {
                            unreachable!(
                                "App should not come here without successful loaded configuration"
                            );
                        };
                    let ldap_config = LDAPConfig::new_readonly(
                        &config,
                        LdapSimpleCredential::new(Default::default(), Default::default()),
                    )
                    .unwrap();
                    move || usermgmt_lib::ldap::list_ldap_users(ldap_config)
                });
            }
        }
    });

    rust_backend.on_loading_configuration({
        let jobs = jobs.clone();
        let app = app.as_weak().unwrap();
        move || {
            log_callback_from_gui("on_listing_ldap_users");
            let conf_job = &jobs.load_config;
            let app_state = app.global::<AppState>();
            if !conf_job.is_thread_running() {
                app_state.set_configuration(LoadStatus::Loading);
                app_state.set_configuration_status_msg("Configuration is being loaded.".into());
                conf_job.spawn_task(|| usermgmt_lib::config::load_config(None));
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
                        |app, msg| app.set_listing_ldap_msg(msg.into()),
                        || "Ldap users to be listed was returned.",
                    );
                    if let Ok(ref resource) = result {
                        let table = listed_ldap_to_slint_table(resource.clone());
                        app.global::<AppState>().set_table_ldap_users(table);
                    }

                    io_resources.borrow_mut().listed_ldap_users = Some(result)
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

fn listed_ldap_to_slint_table(to_conver: LdapSearchResult) -> TableForListing {
    let (header, _body) = to_conver.into();
    let slint_headers = {
        let iterator: Vec<TableColumn> = header
            .into_iter()
            .map(SharedString::from)
            .map(|shared_str| {
                let mut column = TableColumn::default();
                column.title = shared_str;
                column
            })
            .collect();
        to_modle_rc(iterator)
    };
    let columns = {
        let rows: Vec<ModelRc<StandardListViewItem>> = _body
            .into_iter()
            .map(cells_to_row)
            .map(to_modle_rc)
            .collect();
        to_modle_rc(rows)
    };

    return TableForListing {
        headers: slint_headers,
        columns,
    };

    fn to_modle_rc<T>(iterator: Vec<T>) -> ModelRc<T>
    where
        T: Clone + 'static,
    {
        let vec_model = VecModel::from(iterator);
        let as_rc = Rc::new(vec_model);
        ModelRc::from(as_rc.clone())
    }

    fn cells_to_row(cells: Vec<Vec<String>>) -> Vec<StandardListViewItem> {
        cells
            .into_iter()
            .map(cell_fields_to_shared)
            .map(|shared| {
                let mut list_item = StandardListViewItem::default();
                list_item.text = shared;
                list_item
            })
            .collect()
    }

    fn cell_fields_to_shared(cells: Vec<String>) -> SharedString {
        let joined: String = cells.join(" | ");
        SharedString::from(joined)
    }
}

fn log_callback_from_gui(call_back_name: &str) {
    info!("Callback {} is called.", call_back_name);
}
