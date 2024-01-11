mod builtins;
mod config;
mod symbol_table;
mod tree;
mod parser;
mod keywords;
mod args;
mod log;

use std::process::Command;
use std::io::{self, Write, Stdout};
use std::env;
use std::process::Stdio;
use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, FromRawFd};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::{cursor, terminal_size};

fn print_prompt1(cfg: &config::Config) {
    print!("{}", cfg.variables.get("PS1").unwrap());
    io::stdout().flush().unwrap();
}

fn print_prompt2(cfg: &config::Config) {
    print!("{}", cfg.variables.get("PS2").unwrap());
    io::stdout().flush().unwrap();
}

fn key_backspace(cfg: &config::Config, line: &mut String, cur_x: &mut u16, cur_y: &mut u16, ins_cur: &mut u32) {
    if *ins_cur == 0 {
        return;
    }
    
    if *ins_cur == line.len() as u32 {
        line.pop();
    }

    else {
        line.remove(*ins_cur as usize - 1);
    }
    
    let (terminal_cols, _terminal_lines) = terminal_size().unwrap();
   
    log::debug(cfg, format!("cur_x inside key_backspace: {}", *cur_x).as_str());
    if *cur_x == 1 {
        log::debug(cfg, "Hitting if condition inside key_backspace");
        *cur_x = terminal_cols;
        *cur_y -= 1;
    }

    else {
        *cur_x -= 1;
    }
    *ins_cur -= 1;
}

fn key_left_arrow(cfg: &config::Config, cur_x: &mut u16, cur_y: &mut u16, ins_cur: &mut u32) {
    log::debug(cfg, format!("(inside key_left_arrow) ins_cur: {}", ins_cur).as_str());

    let (terminal_cols, _terminal_lines) = terminal_size().unwrap();
    
    if *cur_x == 1 {
        *cur_x = terminal_cols + 1;
        *cur_y -= 1;
    }
    
    if *ins_cur > 0 {
        *ins_cur = *ins_cur - 1;
        *cur_x -= 1;
    }
}

fn key_right_arrow(cfg: &config::Config, line: &String, cur_x: &mut u16, cur_y: &mut u16, ins_cur: &mut u32) {
    log::debug(cfg, format!("(inside key_left_arrow) ins_cur: {}", ins_cur).as_str());
    
    if *ins_cur >= line.len() as u32 {
        return;
    }
    
    let (terminal_cols, _terminal_lines) = terminal_size().unwrap();
    
    if *cur_x == terminal_cols {
        *cur_x = 0;
        *cur_y += 1; 
    }
    
    *ins_cur += 1 ;
    *cur_x += 1;
}

fn read_raw(cfg: &config::Config) -> String {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    
    let mut line = String::new();
    
    // represents where in the string the next character should be inserted
    let mut insert_cur = 0;
    
    for c in io::stdin().keys() {
        
        let (mut cur_x, mut cur_y) = cursor::DetectCursorPos::cursor_pos(&mut stdout).unwrap();
        
        match c.unwrap() {
            termion::event::Key::Char('\n') => break,
            termion::event::Key::Char(ch) => {
                let (terminal_cols, _terminal_lines) = terminal_size().unwrap();
                
                if (insert_cur as usize) < line.len() {
                    line.insert(insert_cur as usize, ch);
                }
                
                else {
                    line.push(ch);
                }

                insert_cur += 1;
                
                if cur_x < terminal_cols {
                    cur_x += 1;
                }

                else {
                    cur_x = 0;
                    cur_y += 1;
                }
            }, 
            termion::event::Key::Up => todo!(),
            termion::event::Key::End => todo!(),
            termion::event::Key::Esc => continue,
            termion::event::Key::Home => todo!(),
            termion::event::Key::Right => key_right_arrow(cfg, &line, &mut cur_x, &mut cur_y, &mut insert_cur),
            termion::event::Key::Down => todo!(),
            termion::event::Key::Left => key_left_arrow(cfg, &mut cur_x, &mut cur_y, &mut insert_cur),
            termion::event::Key::Delete => todo!(),
            termion::event::Key::Null => panic!(),
            termion::event::Key::Backspace => key_backspace(cfg, &mut line, &mut cur_x, &mut cur_y, &mut insert_cur),
            termion::event::Key::PageUp => todo!(),
            termion::event::Key::PageDown => todo!(),
            termion::event::Key::BackTab => todo!(),
            termion::event::Key::Insert => todo!(),
            termion::event::Key::F(_) => unreachable!(),
            termion::event::Key::Alt(_) => todo!(),
            termion::event::Key::Ctrl(_) => todo!(),
            _ => unreachable!()
                
        }

        let (terminal_cols, terminal_lines) = terminal_size().unwrap();

        log::debug(cfg, format!("terminal_cols: {}\nterminal_lines: {}\n", terminal_cols, terminal_lines).as_str());
        log::debug(cfg, format!("line.len: {}\n", line.len()).as_str());
        
        
        let prompt_len = cfg.variables.get("PS1").unwrap().len();
        
        let mut cols_to_erase = ((insert_cur as usize + prompt_len ) / terminal_cols as usize > 0) as usize;
        if (insert_cur as usize + prompt_len) / terminal_cols as usize > 1 {
            cols_to_erase += (insert_cur as usize + prompt_len) / terminal_cols as usize - 1;
        }
        
        log::debug(cfg, format!("cur_y: {}\ncur_x: {}", cur_y, cur_x).as_str());
        
        // for i in 0..cols_to_erase {
        //     log::debug(cfg, format!("erasing line {} from bottom to top", i).as_str());
        //     write!(stdout, "{}", cursor::Goto(0, cur_y-i as u16)).unwrap();
        //     write!(stdout, "{}", termion::clear::AfterCursor).unwrap();
        // }
        
        
        cur_y -= cols_to_erase as u16;
        
        write!(stdout, "{}", cursor::Goto((prompt_len + 1) as u16, cur_y)).unwrap();
        write!(stdout, "{}", termion::clear::AfterCursor).unwrap();
        write!(stdout, "{}", line).unwrap();
        
        if cols_to_erase > 0 {
            cur_y += cols_to_erase as u16;
        }
        
        log::debug(cfg, format!("prompt_len: {}", prompt_len).as_str());
        log::debug(cfg, format!("if cond: {:?}", (line.len() + prompt_len) % terminal_cols as usize == 0).as_str());

        if cur_y - 1 as u16 == terminal_lines && (line.len() + prompt_len) % terminal_cols as usize == 0 {
            log::debug(cfg, "writting newline to stdout");
            write!(stdout, "\n").unwrap();
        }
        
        write!(stdout, "{}", cursor::Goto(cur_x, cur_y)).unwrap();
        stdout.flush().unwrap();
    }

    write!(stdout, "\r\n").unwrap();
    line
}

fn read_command(cfg: &config::Config) -> String {

    let mut line = read_raw(cfg);
    
    line = line.trim().to_string();
    
    log::debug(cfg, format!("line: {}", line).as_str());
    
    if line.is_empty() {
        return line;
    }
    
    while line.chars().nth(line.len() - 1).unwrap() == '\\' {
        line.remove(line.len() - 1);
        
        print_prompt2(&cfg);
        
        let ap_line = read_raw(cfg);
            
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

fn execute_out_redir(cfg: &mut config::Config, command: &mut tree::TreeNode<Box<parser::Token>>, stdin: i32) -> Result<(i32, i32, i32), String> {
    // file needs to live until stdout redirection
    // so that fd is still valid
   
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(command.children[1].value.value.as_str())
        .unwrap();

    let fd = file.as_raw_fd();
    execute_command(cfg, &mut command.children[0], parser::TokenType::Node, stdin, fd)
}

fn execute_out_app(cfg: &mut config::Config, command: &mut tree::TreeNode<Box<parser::Token>>, stdin: i32) -> Result<(i32, i32, i32), String> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(command.children[1].value.value.as_str())
        .unwrap();

    let fd = file.as_raw_fd();
    execute_command(cfg, &mut command.children[0], parser::TokenType::Node, stdin, fd)
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
            parser::TokenType::OutputRedirect => execute_out_redir(cfg, child, stdin).unwrap(),
            parser::TokenType::PipelineRedirect => execute_pipeline(cfg, &mut child.children, stdin, stdout).unwrap(),
            parser::TokenType::OutputRedirectAppend => execute_out_app(cfg, child, stdin).unwrap(),
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

            if stdout != io::stdout().as_raw_fd() && !matches!(parsed_command.value.t_type, parser::TokenType::Subshell){
                command = command.stdout(Stdio::from( unsafe {File::from_raw_fd(stdout) } ));
            }

            if stdin != io::stdin().as_raw_fd() && !matches!(parsed_command.value.t_type, parser::TokenType::Subshell) {
                command = command.stdin(Stdio::from(unsafe { File::from_raw_fd(stdin) } ));
            }

            
            
            log::debug(cfg, format!("{:?}", parsed_command.value.t_type).as_str());
            
            // parsed_command is either a simple Node, which is a vec of command and args
            // or it is just a single-word command which will be classified as a word
            

            
            if !matches!(t_type, parser::TokenType::QuotedStr) &&
                (matches!(parsed_command.value.t_type, parser::TokenType::Node) ||
                 matches!(parsed_command.value.t_type, parser::TokenType::Word) || 
                 matches!(parsed_command.value.t_type, parser::TokenType::PipelineGetInput) ||
                 matches!(parsed_command.value.t_type, parser::TokenType::PipelineSendOuput)) {
                    
                let mut child = command.spawn()
                    .expect("Failed to execute command ");
                
                status = child.wait().unwrap().code().unwrap();
            }

            else if matches!(parsed_command.value.t_type, parser::TokenType::Subshell) {

                
                let child_output = command.output().expect("Failed to retrieve output from command");
                parsed_command.value.value = Box::new(String::from_utf8_lossy(child_output.stdout.as_slice()).trim().to_string());
                
                log::debug(cfg, format!("parsed_command.value.value = {}", parsed_command.value.value).as_str());
                
                status = child_output.status.code().unwrap();
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
