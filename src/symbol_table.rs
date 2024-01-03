use crate::config;
use std::env;

pub fn set_env_var(key: &str, value: &str, cfg: &mut config::Config) {
    cfg.variables.insert(String::from(key), String::from(value));
    env::set_var(key, value);
}

pub fn load_variables(cfg: &mut config::Config) {
    for (key, value) in env::vars() {
        cfg.variables.insert(String::from(key), String::from(value));
    }
    
    set_env_var("?", "0", cfg);
    set_env_var("PS1", "$ ", cfg);
    set_env_var("PS2", "> ", cfg);
}
