use crate::AppResult;
use flexi_logger::{
    Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming, WriteMode,
};

const FILE_NAME: &str = "usermgmt_ouput.log";
const MAX_SIZE_MEGA_BYTES: u64 = 10 * 1024 * 1024;
const NUMBER_OF_FILES: usize = 10;

pub fn set_up_logging(app_name: &str) -> AppResult<LoggerHandle> {
    let handler = match dirs::data_dir() {
        Some(logger_folder_path) => {
            let folder_path = logger_folder_path.join(app_name);
            std::fs::create_dir_all(&folder_path)?;
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
        None => get_terminal_logger(app_name),
    }?
    .start()?;
    return Ok(handler);

    fn get_terminal_logger(app_name: &str) -> AppResult<Logger> {
        let is_debug = cfg!(debug_assertions);
        let lib_crate_name = env!("CARGO_PKG_NAME");
        let logger_str = if is_debug {
            format!("{}=debug, {}=debug", app_name, lib_crate_name)
        } else {
            format!("{}=info, {}=info", app_name, lib_crate_name)
        };
        let logger = Logger::try_with_str(logger_str)?
            .format_for_stderr(flexi_logger::colored_default_format);
        Ok(logger)
    }
}
