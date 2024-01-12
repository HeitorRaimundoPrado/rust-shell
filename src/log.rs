use crate::config;

fn write_to_log_file(cfg: &config::Config, msg: &str) {
    let bytes_written = unsafe {
        let ret = libc::write(cfg.log_file, format!("{}\n", msg).as_ptr() as *const _, msg.len() + 1);
        if ret == -1 {
            panic!("Error writting to log file");
        }
    };

    return bytes_written
}

pub fn debug(cfg: &config::Config, msg: &str) {
    if matches!(cfg.log_level, config::LogLevel::Debug) {
        write_to_log_file(cfg, msg);
    }
}
