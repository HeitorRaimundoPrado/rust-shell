use std::collections::HashMap;
use crate::builtins;
use crate::symbol_table;
use crate::keywords;
use std::os::unix::io::{AsRawFd, RawFd};
use std::io;

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Critical
}

pub struct Config {
    pub rsh_builtins: HashMap<String, fn(&Vec<&String>, &mut Config) -> Result<(i32, i32), String>>,
    pub variables: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    pub keywords: HashMap<String, fn(&Vec<&String>, &mut Config) -> Result<(i32, i32), String>>,
    pub log_level: LogLevel,
    pub log_file: RawFd,
}

pub fn load_config() -> Result<Config, String> {
    let mut loc_config = Config {
        rsh_builtins: HashMap::new(),
        variables: HashMap::new(),
        functions: HashMap::new(),
        keywords: HashMap::new(),
        log_level: LogLevel::Critical,
        log_file: io::stderr().as_raw_fd(),
    };
    

    keywords::load_keywords(&mut loc_config);
    builtins::load_builtins(&mut loc_config);
    symbol_table::load_variables(&mut loc_config);
    
    Ok(loc_config)
}

