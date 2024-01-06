use std::env;
use std::os::unix::io::AsRawFd;
use crate::config::Config;
use std::io;

pub fn cd_builtin(argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32, i32), String> {
    let stdout = io::stdout().as_raw_fd();
    let help_msg = String::from("Usage:\n\ncd [new directory]\n");
    if argv.len() != 1 {
        println!("{}", help_msg);
        return Ok((1, 1, stdout));
    }
    
    env::set_current_dir(&argv[0]).expect("Failed to change directory");
    Ok((1, 0, stdout))
}

pub fn help_builtin(_argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32, i32), String> {
    let stdout = io::stdout().as_raw_fd();
    println!("Builtins:\n\nhelp - prints this help message\ncd - changes directory\nexit - exits the program with specified return code\n");
    Ok((1, 0, stdout))
}

pub fn export_builtin(argv: &Vec<&String>, config: &mut Config) -> Result<(i32, i32, i32), String> {
    let help_msg = String::from("Usage:\n\nexport [variable]=[value]\n");
    let stdout = io::stdout().as_raw_fd();
    if argv.len() != 1 {
        println!("{}", help_msg);
        return Ok((1, 1, stdout));
    }

    let split_args:Vec<String> = argv[0].split("=").map(|word| word.to_string()).collect();

    if split_args.len() != 2 {
        println!("{}", help_msg);
        return Ok((1, 1, stdout));
    }

    config.variables.insert(split_args[0].clone(), split_args[1].clone());
    Ok((1, 0, stdout))
}

pub fn exit_builtin(argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32, i32), String> {
    let stdout = io::stdout().as_raw_fd();
    let help_msg = "Usage:\n\nexit [status code]\n";
    if argv.len() > 1 {
        println!("{}", help_msg);
        return Ok((1, 1, stdout));
    }
    
    let mut status_code = 0;

    if argv.len() > 0 {
        let parse_argv = argv[0].parse::<i32>();
        match parse_argv {
            Ok(parse_argv) => {
                status_code = parse_argv;
            },
            
            Err(_e) => {
                println!("{}", help_msg);
                return Ok((1, 1, stdout));
            }
        }
    }
    
    return Ok((0, status_code, stdout));
}

pub fn load_builtins(cfg: &mut Config) {
    cfg.rsh_builtins.insert(String::from("help"), help_builtin);
    cfg.rsh_builtins.insert(String::from("cd"), cd_builtin);
    cfg.rsh_builtins.insert(String::from("exit"), exit_builtin);
    cfg.rsh_builtins.insert(String::from("export"), export_builtin);
}
