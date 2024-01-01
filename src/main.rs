use std::process::Command;
use std::io::{self, Write};
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::os::unix::process;
use std::os::unix::process::CommandExt;

#[derive(Debug)]
struct GeneralError;

impl Error for GeneralError {}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}

fn cd_builtin(argv: &Vec<String>) -> Result<(i32, i32), GeneralError> {
    Ok((1, 0))
}

fn help_builtin(argv: &Vec<String>) -> Result<(i32, i32), GeneralError> {
    Ok((1, 0))
}

fn exit_builtin(argv: &Vec<String>) -> Result<(i32, i32), GeneralError> {
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
    rsh_builtins: HashMap<String, fn(&Vec<String>) -> Result<(i32, i32), GeneralError>>
}


fn load_config() -> Result<Config, GeneralError> {
    let mut loc_config = Config {
        ps1: String::from("> "),
        rsh_builtins: HashMap::new()
    };
    
    loc_config.rsh_builtins.insert(String::from("help"), help_builtin);
    loc_config.rsh_builtins.insert(String::from("cd"), cd_builtin);
    loc_config.rsh_builtins.insert(String::from("exit"), exit_builtin);
    
    Ok(loc_config)
}

fn parse_command(s: &String) -> Vec<String> {
    s.split(" ").map(|word| word.to_string()).collect()
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
        
        let parsed_command = parse_command(&line);
        
        if config.rsh_builtins.get(&parsed_command[0]) != None {
            (should_continue, status) = config.rsh_builtins.get(&parsed_command[0]).unwrap()(&parsed_command[1..].to_vec()).unwrap();
        }

        else {
            let mut command = Command::new(&parsed_command[0]);
            command.args(&parsed_command[1..]);
            let mut child = command.spawn().expect("Failed to spawn subshell");
            status = child.wait().expect("Failed to wait child process to finish").code().unwrap();
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
