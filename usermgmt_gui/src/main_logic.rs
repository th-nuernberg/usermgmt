//! Contains the high level control flow of this application.
//! The folder named after this module contains all files which make up the high level control flow
//! You should start here for adding new features.
//!
//! This application has a global state, contained in the [`UsermgmtWindow`]
//! Every frame the GUI is drawn based on this global state.
//! Global state can be change by either
//!
//! - GUI elements like a button
//! - reacting to returned results from an IO background task.
//!
//! IO background tasks are started by certain interactions with GUI elements like a button.
//! The start of an IO background task starts an OS thread.
//! After a while an IO background task returns with a result.
//! At every frame the main loop checks if an IO background task has just completed.
//! An IO background task is what accomplishes the management of user within LDAP and Slurm in the cluster.

/// Go here If you want to add a new selectable view.
mod current_selected_view;
/// Queries for pending Io background task.
/// Go here if you want to put some work into a background OS thread without blocking the GUI.
mod query_io_tasks;
/// Go here if you want to add a new value which influences the look of the GUI and is contained
/// in the settings files.  
mod settings;
/// The place where the drawing is chosen for the selected view.
/// Go here If you add the drawing for a selectable new view.
mod top_level_drawing;
/// Main window with the global state.
/// Go here if you want to change the initializing of the global state.
/// Go here if you want to change the order of the high level control flow
mod usermgmt_window;

pub use current_selected_view::CurrentSelectedView;
pub use settings::{Init, Settings};
pub use usermgmt_window::UsermgmtWindow;
