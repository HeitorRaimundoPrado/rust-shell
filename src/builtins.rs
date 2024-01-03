use std::env;
use crate::config::Config;
use crate::builtins;

fn cd_builtin(argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32), String> {
    let help_msg = String::from("Usage:\n\ncd [new directory]\n");
    if argv.len() != 1 {
        println!("{}", help_msg);
        return Ok((1, 1));
    }
    
    env::set_current_dir(&argv[0]).expect("Failed to change directory");
    Ok((1, 0))
}

fn help_builtin(_argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32), String> {
    println!("Builtins:\n\nhelp - prints this help message\ncd - changes directory\nexit - exits the program with specified return code\n");
    Ok((1, 0))
}

fn export_builtin(argv: &Vec<&String>, config: &mut Config) -> Result<(i32, i32), String> {
    let help_msg = String::from("Usage:\n\nexport [variable]=[value]\n");
    if argv.len() != 1 {
        println!("{}", help_msg);
        return Ok((1, 1));
    }

    let split_args:Vec<String> = argv[0].split("=").map(|word| word.to_string()).collect();

    if split_args.len() != 2 {
        println!("{}", help_msg);
        return Ok((1, 1));
    }

    config.variables.insert(split_args[0].clone(), split_args[1].clone());
    Ok((1, 0))
}

fn exit_builtin(argv: &Vec<&String>, _config: &mut Config) -> Result<(i32, i32), String> {
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
            
            Err(_e) => {
                println!("{}", help_msg);
                return Ok((1, 1));
            }
        }
    }
    
    return Ok((0, status_code));
}

pub fn load_builtins(cfg: &mut Config) {
    cfg.rsh_builtins.insert(String::from("help"), builtins::help_builtin);
    cfg.rsh_builtins.insert(String::from("cd"), builtins::cd_builtin);
    cfg.rsh_builtins.insert(String::from("exit"), builtins::exit_builtin);
    cfg.rsh_builtins.insert(String::from("export"), builtins::export_builtin);
}
    
