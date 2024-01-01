use std::process::Command;
use std::io::{self, Write};
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::os::unix::process;
use std::os::unix::process::CommandExt;
use std::env;

fn cd_builtin(argv: &Vec<String>) -> Result<(i32, i32), String> {
    let help_msg = String::from("Usage:\n\ncd [new directory]\n");
    if argv.len() != 1 {
        println!("{}", help_msg);
    }
    
    env::set_current_dir(&argv[0]).expect("Failed to change directory");
    Ok((1, 0))
}

fn help_builtin(argv: &Vec<String>) -> Result<(i32, i32), String> {
    println!("Builtins:\n\nhelp - prints this help message\ncd - changes directory\nexit - exits the program with specified return code\n");
    Ok((1, 0))
}

fn exit_builtin(argv: &Vec<String>) -> Result<(i32, i32), String> {
    let help_msg = "Usage:\n\nexit [status code]\n";
    if argv.len() > 1 {
        println!("{}", help_msg);
        return Ok((1, 1));
    }
    
    let mut status_code = 0;

    if argv.len() > 0 {
        let parse_argv = argv[0].parse::<i32>();
        match parse_argv {
            Ok(parse_argv) => {
                status_code = parse_argv;
            },
            
            Err(e) => {
                println!("{}", help_msg);
                return Ok((1, 1));
            }
        }
    }
    
    return Ok((0, status_code));
}

struct Config {
    ps1: String,
    rsh_builtins: HashMap<String, fn(&Vec<String>) -> Result<(i32, i32), String>>,
    variables: HashMap<String, String>
}


fn load_config() -> Result<Config, String> {
    let mut loc_config = Config {
        ps1: String::from("> "),
        rsh_builtins: HashMap::new(),
        variables: HashMap::new()
    };
    
    loc_config.rsh_builtins.insert(String::from("help"), help_builtin);
    loc_config.rsh_builtins.insert(String::from("cd"), cd_builtin);
    loc_config.rsh_builtins.insert(String::from("exit"), exit_builtin);

    loc_config.variables.insert(String::from("?"), String::from("0"));
    
    
    Ok(loc_config)
}

fn parse_command(s: &String, config: &Config) -> Vec<String> {
    let mut split_string: Vec<String> = s.split(" ").map(|word| word.to_string()).collect();
    for s in split_string.iter_mut() {
        if s.starts_with('$') {
            let var_name = &s[1..];
            let val = config.variables.get(var_name).unwrap();
            *s = val.to_string();
        }
    }

    split_string
}

fn main_loop(config: Config) -> i32 {
    let mut should_continue = 1;
    let mut status = 0;
    
    while should_continue != 0 {
        let mut line = String::new();
            
        print!("{}", config.ps1);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line)
            .expect("Failed to read line");


        let line = String::from(line.trim_end());
        
        if line.trim() == "" {
            continue;
        }
        
        let parsed_command = parse_command(&line, &config);
        
        if config.rsh_builtins.get(&parsed_command[0]) != None {
            (should_continue, status) = config.rsh_builtins.get(&parsed_command[0]).unwrap()(&parsed_command[1..].to_vec()).unwrap();
        }

        else {
            let mut command = Command::new(&parsed_command[0]);
            command.args(&parsed_command[1..]);
            let mut child = command.spawn();
            match child {
                Ok(_) => {},
                Err(e) => {
                    println!("Error: {}\n", e);
                    continue;
                }
                    
            };
            
            status = child.unwrap().wait().expect("Failed to wait child process to finish").code().unwrap();
        }
    }

    return status;
}

fn main() -> Result<(), String> {
    let cfg = load_config().unwrap();
    
    println!("hello world!");
    let status = main_loop(cfg);
    
    std::process::exit(status);
}
