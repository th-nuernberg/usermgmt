use std::path::Path;

use crate::AppResult;
use flexi_logger::{
    Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming, WriteMode,
};

const FILE_NAME: &str = "usermgmt_ouput.log";
const MAX_SIZE_MEGA_BYTES: u64 = 10 * 1024 * 1024;
const NUMBER_OF_FILES: usize = 10;

/// Set writing logging to terminal stderr and to logging file.
///
/// # Notes
///
/// - It tries to log within the data folder for the application first.
/// - If this fails, then it tries to log within the same folder of executable.
/// - If logging to logging file is not possible, logs are only written to stderr.
/// - Keep returned logger handler in a variable alive to keep the file logging working.
///  
/// # Error
///
/// - If initializing of logger fails even for terminal, stderr output.
///
pub fn set_up_logging(app_name: &str) -> AppResult<LoggerHandle> {
    let handler = if is_debug() {
        let project_path_logging_file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(app_name);
        set_up_file_logger(&project_path_logging_file, app_name)
    } else {
        match dirs::data_dir() {
            Some(logger_folder_path) => {
                let folder_path = logger_folder_path.join(app_name);
                if let Ok(()) = std::fs::create_dir_all(folder_path) {
                    set_up_file_logger(&logger_folder_path, app_name)
                } else {
                    get_terminal_logger(app_name)
                }
            }
            None => {
                if let Ok(exec_path) = std::env::current_exe() {
                    let exec_path = exec_path.parent().ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not get the directory of the executable for logging there"
                        )
                    })?;
                    set_up_file_logger(exec_path, app_name)
                } else {
                    get_terminal_logger(app_name)
                }
            }
        }
    }?
    .start()?;

    return Ok(handler);

    fn set_up_file_logger(folder_path: &Path, app_name: &str) -> AppResult<Logger> {
        let fs_specs = FileSpec::default()
            .directory(folder_path)
            .basename(FILE_NAME)
            .suppress_timestamp();
        let logger = get_terminal_logger(app_name)?
            .format_for_files(flexi_logger::detailed_format)
            .log_to_file(fs_specs)
            .write_mode(WriteMode::Async)
            .rotate(
                Criterion::Size(MAX_SIZE_MEGA_BYTES),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(NUMBER_OF_FILES),
            )
            .append()
            .duplicate_to_stderr(Duplicate::All);
        Ok(logger)
    }
    fn get_terminal_logger(app_name: &str) -> AppResult<Logger> {
        let is_debug = is_debug();
        let lib_crate_name = env!("CARGO_PKG_NAME");
        let logger_str = std::env::var("RUST_LOG").ok().unwrap_or_else(|| {
            if is_debug {
                format!("{}=debug, {}=debug", app_name, lib_crate_name)
            } else {
                format!("{}=info, {}=info", app_name, lib_crate_name)
            }
        });
        let logger = Logger::try_with_str(logger_str)?
            .format_for_stderr(flexi_logger::colored_default_format);
        Ok(logger)
    }

    fn is_debug() -> bool {
        cfg!(debug_assertions)
    }
}
