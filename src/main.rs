use std::process::Command;
use std::io::{self, Write};
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::os::unix::process;

#[derive(Debug)]
struct GeneralError;

impl Error for GeneralError {}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}

fn cd_builtin() -> Result<(i32, i32), GeneralError> {
    Ok((1, 0))
}

fn help_builtin() -> Result<(i32, i32), GeneralError> {
    Ok((1, 0))
}

fn exit_builtin() -> Result<(i32, i32), GeneralError> {
    println!("exit works");
    Ok((0, 0))
}

struct Config {
    ps1: String,
    rsh_builtins: HashMap<String, fn() -> Result<(i32, i32), GeneralError>>
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

fn parse_command(s: &String) -> String {
    s.split(" ")
}

fn main_loop(config: Config) {
    let mut should_continue = 1;
    let mut status = 0;
    
    while should_continue != 0 {
        let mut line = String::new();
            
        print!("{}", config.ps1);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line)
            .expect("Failed to read line");


        println!("{}", line);
        let line = String::from(line.trim_end());
        
        let parsed_command = parse_command(&line);
        
        if config.rsh_builtins.get(&parsed_command[0]) != None {
            (should_continue, status) = config.rsh_builtins.get(&line).unwrap()(&parsed_command[1..]).unwrap();
        }

        else {
            
        }
    }
}

fn main() {
    let cfg = load_config().unwrap();
    
    println!("hello world!");
    main_loop(cfg);

    
    let output = Command::new("ls").args(&["-l"]).output().unwrap();

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8(output.stdout).unwrap());
    println!("stdout: {}", String::from_utf8(output.stderr).unwrap());
}
