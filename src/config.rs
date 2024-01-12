use std::collections::HashMap;
use std::fs::File;
use crate::builtins;
use crate::main_loop;
use crate::symbol_table;
use crate::keywords;

use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use std::io;

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Critical
}

pub struct Config {
    pub rsh_builtins: HashMap<String, fn(&Vec<&String>, &mut Config) -> Result<(i32, i32, i32), String>>,
    pub variables: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    pub keywords: HashMap<String, fn(&Vec<&String>, &mut Config) -> Result<(i32, i32), String>>,
    pub log_level: LogLevel,
    pub log_file: RawFd,
    pub stdin_to_execute: RawFd
}

pub fn load_config() -> Result<Config, String> {
    let mut loc_config = Config {
        rsh_builtins: HashMap::new(),
        variables: HashMap::new(),
        functions: HashMap::new(),
        keywords: HashMap::new(),
        log_level: LogLevel::Critical,
        log_file: io::stderr().as_raw_fd(),
        stdin_to_execute: -1,
    };
    

    keywords::load_keywords(&mut loc_config);
    builtins::load_builtins(&mut loc_config);
    symbol_table::load_variables(&mut loc_config);

    let rshconfig_path = Path::new(loc_config.variables.get("HOME").unwrap()).join(Path::new(".rshconfig"));
    let rshconfig_file = File::open(rshconfig_path).unwrap();
    
    loc_config.stdin_to_execute = rshconfig_file.as_raw_fd();
    
    main_loop(&mut loc_config);
    
    Ok(loc_config)
}

