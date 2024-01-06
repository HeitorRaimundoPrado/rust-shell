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
use std::process::Stdio;
use std::fs::File;
use std::os::unix::io::{OwnedFd, AsRawFd, FromRawFd};

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
    io::stdin().read_line(&mut line)
        .expect("Failed to read_line");
    
    line = line.trim().to_string();
    
    log::debug(cfg, format!("line: {}", line).as_str());
    
    if line.is_empty() {
        return line;
    }
    
    while line.chars().nth(line.len() - 1).unwrap() == '\\' {
        line.remove(line.len() - 1);
        
        print_prompt2(&cfg);
        
        let mut ap_line = String::new();
        io::stdin().read_line(&mut ap_line).expect("Failed to read_line");
            
        line.push_str(ap_line.as_str());
        
        line.remove(line.len()- 1);
    }

    line
}

fn execute_builtin(argv : &mut Vec<tree::TreeNode<Box<parser::Token>>>, cfg: &mut config::Config) -> Result<(i32, i32, i32), String> {
    let builtin = cfg.rsh_builtins.get(&*argv[0].value.value).unwrap();
    Ok(builtin(&argv[1..].iter().map(|arg| &*arg.value.value).collect::<Vec<&String>>(), cfg).unwrap())
}

fn execute_pipeline(cfg: &mut config::Config, pipeline: &mut Vec<tree::TreeNode<Box<parser::Token>>>, stdin: i32, stdout: i32) -> Result<(i32, i32, i32), String> {
    log::debug(cfg, format!("executing pipe:\n {}\n\npiped into:\n\n{}\n", pipeline[0].value.value, pipeline[1].value.value).as_str());
    
    let (fd_read, fd_write) = nix::unistd::pipe().unwrap();
    let mut _should_continue = 1; 
    let should_continue;
    let status ;
    let mut _status = 0;
    let mut stdout_ = io::stdout().as_raw_fd();
    let stdout_ret;
    
    (_should_continue, _status, stdout_) = execute_command(cfg,&mut pipeline[0], parser::TokenType::PipelineRedirect, stdin, fd_write).unwrap();
    
    (should_continue, status, stdout_ret) = execute_command(cfg, &mut pipeline[1], parser::TokenType::PipelineRedirect, fd_read, stdout).unwrap();
    
    
    Ok((should_continue, status, stdout_ret))
}
    


fn execute_command(cfg: &mut config::Config, parsed_command: &mut tree::TreeNode<Box<parser::Token>>, t_type: parser::TokenType, stdin: i32, stdout: i32) -> Result<(i32, i32, i32), String> {
    log::debug(cfg, format!("\nexecuting {:?}\n", parsed_command.children.iter().map(|child| &*child.value.value).collect::<Vec<&String>>()).as_str());
    log::debug(cfg, format!("\nstdin: {:?}\nstdout: {:?}\n", stdin, stdout).as_str());
    log::debug(cfg, format!("\nview into tree: {:#?}\n", parsed_command).as_str());
    
    let mut should_continue: i32 = 1;
    let mut status: i32 = 0;
    let mut child_stdout: i32 = io::stdout().as_raw_fd();
    
    // assume it is a compound command
    let compound_command = &mut parsed_command.children;

    for child in compound_command.iter() {
        log::debug(cfg, format!("{}, {:?}", child.value.value, child.value.t_type).as_str());
    }
    
    for child in compound_command.iter_mut() {
        (should_continue, status, child_stdout) = match child.value.t_type  {
            parser::TokenType::Subshell => execute_command(cfg, child, parser::TokenType::Subshell, stdin, stdout).unwrap(),
            parser::TokenType::OutputRedirect => todo!(),
            parser::TokenType::PipelineRedirect => execute_pipeline(cfg, &mut child.children, stdin, stdout).unwrap(),
            parser::TokenType::OutputRedirectAppend => todo!(),
            parser::TokenType::Word => continue,
            parser::TokenType::PipelineSendOuput => execute_command(cfg, child, parser::TokenType::PipelineSendOuput, stdin, stdout).unwrap(),
            parser::TokenType::PipelineGetInput => execute_command(cfg, child, parser::TokenType::PipelineGetInput, stdin, stdout).unwrap(),
            parser::TokenType::QuotedStr => execute_command(cfg, child, parser::TokenType::QuotedStr, stdin, stdout).unwrap(),
            parser::TokenType::Node => execute_command(cfg, child, parser::TokenType::Node, stdin, stdout).unwrap()
                
        }
    }
    
    
    if compound_command.len() < 1 {
        return Ok((1, 0, stdout));
    }
    
    if matches!(compound_command[0].value.t_type, parser::TokenType::Word) {
        log::debug(cfg, format!("{:?}", compound_command.iter().map(|v| &*v.value.value).collect::<Vec<&String>>()).as_str());
        
        if matches!(compound_command[0].value.w_type, parser::WordType::Builtin) {
            (should_continue, status, child_stdout) = execute_builtin(compound_command, cfg).unwrap();
        }
        
        else {
            let mut command = Command::new(&*compound_command[0].value.value);
            let mut command = command
                .args(&compound_command[1..].iter().map(|v| &*v.value.value).collect::<Vec<&String>>());

            if !(stdout == io::stdout().as_raw_fd()) {
                command = command.stdout(Stdio::from( unsafe {File::from_raw_fd(stdout) } ));
            }

            if !(stdin == io::stdin().as_raw_fd()) {
                command = command.stdin(Stdio::from(unsafe { File::from_raw_fd(stdin) } ));
            }

            
            
            log::debug(cfg, format!("{:?}", parsed_command.value.t_type).as_str());
            
            // parsed_command is either a simple Node, which is a vec of command and args
            // or it is just a single-word command which will be classified as a word
            let mut child = command.spawn()
                    .expect("Failed to execute command ");

            
            if !matches!(t_type, parser::TokenType::QuotedStr) && (matches!(parsed_command.value.t_type, parser::TokenType::Node) || matches!(parsed_command.value.t_type, parser::TokenType::Word)) {
                // match child {
                //     Ok(_) => {},
                //     Err(ref e) => {
                //         println!("Error: {}\n", e);
                //         return Ok((should_continue, 1, stdout));
                //     }
                //         
                // };
                
                // let mut child = child.unwrap();
                
                child.wait().unwrap();

                if let Some(stdout_obj) = child.stdout.take() {
                    child_stdout = stdout_obj.as_raw_fd();
                }

                if !matches!(t_type, parser::TokenType::PipelineRedirect) {
                    log::debug(cfg, format!("waiting for child: {:?}", child).as_str());
                }
            }

            else if matches!(parsed_command.value.t_type, parser::TokenType::Subshell) ||
                matches!(parsed_command.value.t_type, parser::TokenType::PipelineGetInput) ||
                matches!(parsed_command.value.t_type, parser::TokenType::PipelineSendOuput) {
                
                child.wait().unwrap();
                log::debug(cfg, format!("parsed_command.value.value = {}", parsed_command.value.value).as_str());
                
            }
            
            symbol_table::set_env_var("?", &status.to_string(), cfg);
        }
    }

    
    if matches!(t_type, parser::TokenType::QuotedStr) {
        parsed_command.value.value = Box::new(String::from(parsed_command.children.iter().map(|child| child.value.value.to_string()).collect::<Vec<String>>().join("")));
    }
    
    return Ok((should_continue, status, child_stdout));
}


fn main_loop(cfg: &mut config::Config) -> i32 {
    let mut should_continue = 1;
    let mut status = 0;
    let mut _stdout = io::stdout().as_raw_fd();
    
    while should_continue != 0 {
            
        print_prompt1(&cfg);

        let mut line = read_command(&cfg);
        
        if line == "" {
            continue;
        }
        
        let mut parsed_command = parser::build_ast(&mut line, &cfg);
        
        (should_continue, status, _stdout) = execute_command(cfg, &mut parsed_command, parser::TokenType::Node, io::stdin().as_raw_fd(), io::stdout().as_raw_fd()).unwrap();

        log::debug(cfg, format!("\nafter changes in tree:\n\n{:#?}\n", parsed_command).as_str());
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
