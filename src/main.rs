mod builtins;
mod config;
mod symbol_table;
mod tree;
mod parser;
mod keywords;
mod args;
mod log;

use std::process::Command;
use std::io::{self, Write};
use std::env;

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

fn execute_command(cfg: &mut config::Config, parsed_command: &mut tree::TreeNode<Box<parser::Token>>) -> Result<(i32, i32), String> {
    log::debug(cfg, format!("\nexecuting {:?}\n", parsed_command).as_str());
    
    let mut should_continue: i32 = 1;
    let mut status: i32 = 0;
    
    // assume it is a compound command
    let compound_command = &mut parsed_command.children;

    for child in compound_command.iter() {
        log::debug(cfg, format!("{}, {:?}", child.value.value, child.value.t_type).as_str());
    }
    
    for child in compound_command.iter_mut() {
        (should_continue, status) = match child.value.t_type  {
            parser::TokenType::Subshell => execute_command(cfg, child).unwrap(),
            parser::TokenType::OutputRedirect => todo!(),
            parser::TokenType::PipelineRedirect => todo!(),
            parser::TokenType::OutputRedirectAppend => todo!(),
            parser::TokenType::Word => continue,
            parser::TokenType::Node => execute_command(cfg, child).unwrap()
                
        }
    }
    
    
    if matches!(compound_command[0].value.t_type, parser::TokenType::Word) {
        log::debug(cfg, format!("{:?}", compound_command.iter().map(|v| &*v.value.value).collect::<Vec<&String>>()).as_str());
        
        if matches!(compound_command[0].value.w_type, parser::WordType::Builtin) {
            let builtin = cfg.rsh_builtins.get(&*compound_command[0].value.value).unwrap();
            (should_continue, status) = builtin(&compound_command[1..].iter().map(|arg| &*arg.value.value).collect::<Vec<&String>>(), cfg).unwrap();
        }
        
        else {
            let mut command = Command::new(&*compound_command[0].value.value);
            command.args(&compound_command[1..].iter().map(|v| &*v.value.value).collect::<Vec<&String>>());

            
            
            log::debug(cfg, format!("{:?}", parsed_command.value.t_type).as_str());
            
            if matches!(parsed_command.value.t_type, parser::TokenType::Node) || matches!(parsed_command.value.t_type, parser::TokenType::Word) {
                let child = command.spawn();
                match child {
                    Ok(_) => {},
                    Err(ref e) => {
                        println!("Error: {}\n", e);
                        return Ok((should_continue, 1));
                    }
                        
                };
                
                status = child.unwrap().wait().expect("Failed to wait child process to finish").code().unwrap();
            }

            else if matches!(parsed_command.value.t_type, parser::TokenType::Subshell) {
                let child = command.output().expect("Failed to execute subshell command");
                parsed_command.value.value = Box::new(String::from_utf8_lossy(child.stdout.as_slice()).trim().to_string());
                
                println!("parsed_command.value.value = {}", parsed_command.value.value);
                
                status = child.status.code().unwrap();
            }
            
            symbol_table::set_env_var("?", &status.to_string(), cfg);
        }
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
        
        let mut parsed_command = parser::build_ast(&mut line, &cfg);
        
        (should_continue, status) = execute_command(cfg, &mut parsed_command).unwrap();
    }

    return status;
}

fn main() -> Result<(), String> {
    let mut cfg = config::load_config().unwrap();
    log::debug(&mut cfg, "Starting entry point");
    
    args::load_args(&mut cfg, env::args().collect());
    
    let status = main_loop(&mut cfg);
    
    std::process::exit(status);
}
