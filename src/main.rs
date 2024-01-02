mod builtins;
mod config;

use std::process::Command;
use std::io::{self, Write};

fn parse_command(s: &String, config: &config::Config) -> Vec<String> {
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

fn print_prompt1(cfg: &config::Config) {
    print!("{}", cfg.variables.get("PS1").unwrap());
    io::stdout().flush().unwrap();
}

fn print_prompt2(cfg: &config::Config) {
    print!("{}", cfg.variables.get("PS2").unwrap());
    io::stdout().flush().unwrap();
}

fn read_command(cfg: &config::Config) -> String {
    let mut line = String::new();
    loop {
        io::stdin().read_line(&mut line)
            .expect("Failed to read_line");

        line = line.trim().to_string();
        
        if line.chars().nth(line.len() - 1).unwrap() == '\\' {
            print_prompt2(&cfg);
            line.remove(line.len()- 1);
        }
        
        else {
            break;
        }
    } 

    line
}

fn execute_command(cfg: &mut config::Config, parsed_command: Vec<String>) -> Result<(i32, i32), String> {
    let mut should_continue: i32 = 1;
    let status: i32;
    
    if cfg.rsh_builtins.get(&parsed_command[0]) != None {
        (should_continue, status) = cfg.rsh_builtins.get(&parsed_command[0]).unwrap()(&parsed_command[1..].to_vec(), cfg).unwrap();
    }

    else {
        let mut command = Command::new(&parsed_command[0]);
        command.args(&parsed_command[1..]);
        let child = command.spawn();
        match child {
            Ok(_) => {},
            Err(ref e) => {
                println!("Error: {}\n", e);
            }
                
        };
        
        status = child.unwrap().wait().expect("Failed to wait child process to finish").code().unwrap();
        cfg.variables.insert(String::from("?"), status.to_string());
    }
    
    return Ok((should_continue, status));
}

fn main_loop(cfg: &mut config::Config) -> i32 {
    let mut should_continue = 1;
    let mut status = 0;
    
    while should_continue != 0 {
            
        print_prompt1(&*cfg);

        let line = read_command(&*cfg);
        
        if line == "" {
            continue;
        }
        
        let parsed_command = parse_command(&line, &cfg);
        
        (should_continue, status) = execute_command(cfg, parsed_command).unwrap();
    }

    return status;
}

fn main() -> Result<(), String> {
    let mut cfg = config::load_config().unwrap();
    
    println!("hello world!");
    let status = main_loop(&mut cfg);
    
    std::process::exit(status);
}
