use crate::config;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;

pub fn load_args(cfg: &mut config::Config, argv: Vec<String>) {
    let help_msg = "
    Usage: rust-shell [options] (script)

    Options:
        --log-level: One of
            debug
            info
            warn
            critical
    ";
    
    for (i, _val) in  argv.iter().enumerate() {
        if argv[i].ends_with(".sh") || argv[i].ends_with(".rsh") && cfg.stdin_to_execute == io::stdin().as_raw_fd() {
            cfg.stdin_to_execute = File::open(&argv[i]).unwrap().as_raw_fd();
            continue;
        }
        
        if argv[i] == "--log-level" {
            if argv.len() < i+2 {
                println!("{}", help_msg);
                break;
            }

            else {
                cfg.log_level = match argv[i+1].as_str() {
                    "debug" => config::LogLevel::Debug,
                    "info" => config::LogLevel::Info,
                    "warn" => config::LogLevel::Warn,
                    "crit" => config::LogLevel::Critical,
                    &_ => {
                        println!("{}", help_msg);
                        break;
                    }
                }
            }
            
        }
    }
}
