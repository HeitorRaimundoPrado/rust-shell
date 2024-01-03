use crate::tree;
use crate::config;
use regex::Regex;
use crate::log;

#[derive(Debug)]
pub enum TokenType {
    Word,
    Node,
    PipelineRedirect,
    OutputRedirect,
    OutputRedirectAppend,
    Subshell
}

#[derive(Debug)]
pub enum WordType {
    NotWord,
    Builtin,
    Function,
    Keyword,
    General
}

#[derive(Debug)]
pub struct Token {
    pub t_type: TokenType,
    pub w_type: WordType,
    pub value:  Box<String>,
}

fn classify_token(s: &String, cfg: &config::Config) -> Token {
    let mut tok = Token {
        t_type: TokenType::Word,
        w_type: WordType::NotWord,
        value: Box::new(s.to_string()),
    };
    
    match s.as_str() {
        "|" => tok.t_type = TokenType::PipelineRedirect,
        ">" => tok.t_type = TokenType::OutputRedirect,
        ">>" => tok.t_type = TokenType::OutputRedirectAppend,
        &_ => tok.t_type = TokenType::Word
    }

    let re = [
        // There are still nodes to parse
        Regex::new(r#"[|><'"$()\s]+"#).unwrap(),
    ];

    if re[0].is_match(s) {
        tok.t_type = TokenType::Node;
    }


    // Otherwise it's a word
     
    
    if matches!(tok.t_type, TokenType::Word) {
        match s.as_str() {
            val if cfg.rsh_builtins.get(val) != None  => tok.w_type = WordType::Builtin,
            val if cfg.functions.get(val) != None => tok.w_type = WordType::Function,
            val if cfg.keywords.get(val) != None => tok.w_type = WordType::Keyword,
            &_ => tok.w_type = WordType::General
        }
    }

    tok
}


fn split_simple_command(s: &String, cfg: &config::Config) -> Vec<String> {
    let mut split_string: Vec<String> = s.split(" ").map(|word| word.to_string()).collect();
    let mut ret_vec: Vec<String> = Vec::new();
    
    for s in split_string.iter_mut() {
        if s.starts_with('$') {
            let var_name = &s[1..];
            let val = cfg.variables.get(var_name);
            let val = match val {
                Some(v) => v,
                None => ""
            };
                
            ret_vec.push(val.to_string());
        }
       
        else {
            ret_vec.push(s.to_string())
        }

    }

    ret_vec
}

pub fn build_ast (command: &String, cfg: &config::Config) ->  Box<tree::TreeNode<Box<Token>>> {
    log::debug(cfg, format!("building ast for {}", command).as_str());
    
    let mut root = Box::new(tree::TreeNode {
         value: Box::new(classify_token(command, cfg)),
        children: Vec::new(),
    });

    
    let regexp = vec![
        // Pipeline
        Regex::new(r#"^(?P<left>.*)\|(?P<right>.*)$"#).unwrap(),
        
        // $() or () syntax
        Regex::new(r"^(?P<main>.*)\$\((?P<sub>.*)\)").unwrap(),
    ];
    

    if regexp[0].is_match(command) {
        let left = build_ast(&regexp[0].captures(command).unwrap().name("left").unwrap().as_str().trim().to_string(), cfg);
        let right = build_ast(&regexp[0].captures(command).unwrap().name("right").unwrap().as_str().trim().to_string(), cfg);
        root.children.push(*left);
        root.children.push(tree::TreeNode {
            value: Box::new(classify_token(&String::from("|"), cfg)),
            children: Vec::new()
        });
        root.children.push(*right);
    }

    else if regexp[1].is_match(command) {
        let captures = regexp[1].captures(command).unwrap();
        if captures.name("main") != None && captures.name("main").unwrap().as_str().trim() != "" {
            let main = build_ast(&captures.name("main").unwrap().as_str().trim().to_string(), cfg);
            root.children.extend(main.children);
        }
        
        let mut subcommand = build_ast(&captures.name("sub").unwrap().as_str().trim().to_string(), cfg);
        subcommand.value.t_type = TokenType::Subshell;
        
        root.children.push(*subcommand);
    }
    
    else {
        let parsed_command = split_simple_command(command, cfg);
        
        for s in parsed_command {
            let tok = classify_token(&s, cfg);
            root.children.push(tree::TreeNode{
                value: Box::new(tok),
                children: Vec::new()
            });
        }
    }
    
    root
}
