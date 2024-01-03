use std::collections::HashMap;
use crate::builtins;
use crate::symbol_table;

pub struct Config {
    pub rsh_builtins: HashMap<String, fn(&Vec<&String>, &mut Config) -> Result<(i32, i32), String>>,
    pub variables: HashMap<String, String>,
}

pub fn load_config() -> Result<Config, String> {
    let mut loc_config = Config {
        rsh_builtins: HashMap::new(),
        variables: HashMap::new()
    };
    
    loc_config.rsh_builtins.insert(String::from("help"), builtins::help_builtin);
    loc_config.rsh_builtins.insert(String::from("cd"), builtins::cd_builtin);
    loc_config.rsh_builtins.insert(String::from("exit"), builtins::exit_builtin);
    loc_config.rsh_builtins.insert(String::from("export"), builtins::export_builtin);
    loc_config.rsh_builtins.insert(String::from("if"), builtins::if_builtin);

    symbol_table::load_variables(&mut loc_config);
    
    Ok(loc_config)
}

