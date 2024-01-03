use crate::config;

pub fn load_args(cfg: &mut config::Config, argv: Vec<String>) {
    let help_msg = "
    
    ";
    
    for (i, _val) in  argv.iter().enumerate() {
        if argv[i] == "--log-level" {
            if argv.len() < i+1 {
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
