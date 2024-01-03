mod builtins;
mod config;
mod symbol_table;
mod tree;
mod parser;

use std::process::Command;
use std::io::{self, Write};

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

fn execute_command(cfg: &mut config::Config, parsed_command: &Box<tree::TreeNode<Box<parser::Token>>>) -> Result<(i32, i32), String> {
    let mut should_continue: i32 = 1;
    let status: i32;
    
    let simple_command: Vec<&String> = parsed_command.children.iter().map(|child| &*child.value.value).collect();
    if cfg.rsh_builtins.get(simple_command[0]) != None {
        let builtin = cfg.rsh_builtins.get(simple_command[0]).unwrap();
        (should_continue, status) = builtin(&simple_command[1..].to_vec(), cfg).unwrap();
    }

    else {
        let mut command = Command::new(&simple_command[0]);
        command.args(&simple_command[1..]);
        let child = command.spawn();
        match child {
            Ok(_) => {},
            Err(ref e) => {
                println!("Error: {}\n", e);
                return Ok((should_continue, 1));
            }
                
        };
        
        status = child.unwrap().wait().expect("Failed to wait child process to finish").code().unwrap();
        
        symbol_table::set_env_var("?", &status.to_string(), cfg);
    }
    
    return Ok((should_continue, status));
}

fn main_loop(cfg: &mut config::Config) -> i32 {
    let mut should_continue = 1;
    let mut status = 0;
    
    while should_continue != 0 {
            
        print_prompt1(&cfg);

        let mut line = read_command(&cfg);
        
        if line == "" {
            continue;
        }
        
        let parsed_command = parser::build_ast(&mut line, &cfg);
        
        (should_continue, status) = execute_command(cfg, &parsed_command).unwrap();
    }

    return status;
}

fn main() -> Result<(), String> {
    let mut cfg = config::load_config().unwrap();
    
    println!("hello world!");
    let status = main_loop(&mut cfg);
    
    std::process::exit(status);
}
