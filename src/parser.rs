use crate::tree;
use crate::config;
use regex::Regex;

pub enum TokenType {
    Word,
    Root,
    PipelineRedirect,
    OutputRedirect,
    OutputRedirectAppend,
    Subshell
}

pub enum WordType {
    NotWord,
    Builtin,
    Function,
    Keyword,
    General
}

pub struct Token {
    pub t_type: TokenType,
    pub w_type: WordType,
    pub value:  Box<String>,
}

fn classify_token(s: &String) -> Token {
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

    let re = Regex::new(r"\$?\((.*)\)").unwrap();

    if re.is_match(s) {
        tok.t_type = TokenType::Subshell;
        tok.value = Box::new(re.captures(s).unwrap().get(1).unwrap().as_str().to_string());
    }

    tok
}


fn split_command(s: &String, config: &config::Config) -> Vec<String> {
    let mut split_string: Vec<String> = s.split(" ").map(|word| word.to_string()).collect();
    let mut ret_vec: Vec<String> = Vec::new();
    
    for s in split_string.iter_mut() {
        if s.starts_with('$') {
            let var_name = &s[1..];
            let val = config.variables.get(var_name).unwrap();
            ret_vec.push(val.to_string());
        }
        
        let re = Regex::new(r#"[^'"]*|[^'"]*"#).unwrap();

        if re.is_match(s) {
            ret_vec.extend(s.split("|").map(|word| word.to_string()));
        }
    }

    ret_vec
}

pub fn build_ast (command: &String, cfg: &config::Config) ->   Box<tree::TreeNode<Box<Token>>> {
    let mut root = Box::new(tree::TreeNode {
         value: Box::new(Token {
            t_type: TokenType::Root,
            w_type: WordType::NotWord,
            value: Box::new(command.to_string())
        }),
        
        children: Vec::new(),
    });

    let parsed_command = split_command(command, cfg);

    for s in parsed_command {
        let tok = classify_token(&s);
        root.children.push(tree::TreeNode{
            value: Box::new(tok),
            children: Vec::new()
        });
    }
    
    root
}
