use crate::tree;
use crate::config;
use regex::Regex;
use crate::log;

#[derive(Debug, Clone)]
pub enum TokenType {
    Word,
    Node,
    PipelineRedirect,
    PipelineSendOuput,
    PipelineGetInput,
    OutputRedirect,
    OutputRedirectAppend,
    QuotedStr,
    Subshell
}

#[derive(Debug, Clone)]
pub enum WordType {
    NotWord,
    Builtin,
    Function,
    Keyword,
    General
}

#[derive(Debug, Clone)]
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

    if re[0].is_match(s) && matches!(tok.t_type, TokenType::Word) {
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

pub fn parse_quot_string(s: &str, cfg: &config::Config) -> Vec<tree::TreeNode<Box<Token>>> {
    let re = vec![
        Regex::new(r#"(?P<before>.*)\$\((?P<sub>.*)\)(?P<after>.*)"#).unwrap(),
    ];

    let mut children: Vec::<tree::TreeNode<Box<Token>>> = Vec::new();
    
    if re[0].is_match(s) {
        let captures = re[0].captures(s).unwrap();
        
        let bef_str = captures.name("before").unwrap().as_str();
        log::debug(cfg, format!("{}", bef_str).as_str());
        children.extend(parse_quot_string(bef_str, cfg));
        
        let sub = captures.name("sub").unwrap().as_str().to_string();
        let mut subtree = build_ast(&sub, cfg);
        subtree.value.t_type = TokenType::Subshell;
        children.push(*subtree);

        let aft_str = captures.name("after").unwrap().as_str();
        log::debug(cfg, format!("{}", aft_str).as_str());
        children.extend(parse_quot_string(aft_str, cfg));
        
    }
    
    vec![
        tree::TreeNode {
        value: Box::new(Token {
            t_type: TokenType::QuotedStr,
            w_type: WordType::NotWord,
            value: Box::new(s.to_string())
        }),
        
        children
        }
    ]
}

pub fn build_ast (command: &String, cfg: &config::Config)
    ->  Box<tree::TreeNode<Box<Token>>> {
    log::debug(cfg, format!("building ast for {}", command).as_str());
    
    let mut root = Box::new(tree::TreeNode {
         value: Box::new(classify_token(command, cfg)),
        children: Vec::new(),
    });

    
    let regexp = vec![
        // Pipeline
        Regex::new(r#"^(?P<left>.*)\|(?P<right>.*)$"#).unwrap(),
        
        // Single-quoted or double quoted string
        Regex::new(r#"^(?P<before>.*)["'](?P<str>.*)["'](?P<after>.*)$"#).unwrap(),
        
        // $() or () syntax
        Regex::new(r"^(?P<before>.*)\$\((?P<sub>.*)\)(?P<after>.*)$").unwrap(),

        // Output redirect (>)
        Regex::new(r"^(?P<before>.*)[^>]>[^>](?P<after>.*)$").unwrap(),

        // Output append (>>)
        Regex::new(r"^(?P<before>.*)>>(?P<after>.*)$").unwrap(),
    ];
    

    if regexp[0].is_match(command) {
        let mut left = build_ast(&regexp[0].captures(command).unwrap().name("left").unwrap().as_str().trim().to_string(), cfg);
        left.value.t_type = TokenType::PipelineSendOuput;
        
        let mut right = build_ast(&regexp[0].captures(command).unwrap().name("right").unwrap().as_str().trim().to_string(), cfg);
        right.value.t_type = TokenType::PipelineGetInput;
        
        root.children.push(tree::TreeNode {
            value: Box::new(classify_token(&String::from("|"), cfg)),
            children: Vec::from(vec![
                *left,
                *right
            ])
        });
    }

    else if regexp[1].is_match(command) {
        let captures = regexp[1].captures(command).unwrap();
       
        let bef_str = captures.name("before").unwrap().as_str().trim().to_string();
        let aft_str = captures.name("after").unwrap().as_str().trim().to_string();
        
        if bef_str != "" {
            root.children.extend(build_ast(&bef_str, cfg).children);
        }
        
        let quot_str = parse_quot_string(&captures.name("str").unwrap().as_str().trim().to_string(), cfg);
        root.children.push(quot_str.get(0).unwrap().clone());

        if aft_str != "" {
            root.children.extend(build_ast(&aft_str, cfg).children);
        }
    }
    
    else if regexp[2].is_match(command) {
        let captures = regexp[2].captures(command).unwrap();
        
        let bef_str = &captures.name("before").unwrap().as_str().trim().to_string();
        
        if bef_str != "" {
            root.children.extend(build_ast(bef_str, cfg).children);
        }
        
        let mut subcommand = build_ast(&captures.name("sub").unwrap().as_str().trim().to_string(), cfg);
        subcommand.value.t_type = TokenType::Subshell;
        root.children.push(*subcommand);

        let aft_str = &captures.name("after").unwrap().as_str().trim().to_string();
        if aft_str != "" {
            root.children.extend(build_ast(aft_str, cfg).children);
        }
    }

    else if regexp[3].is_match(command) {
        let captures = regexp[3].captures(command).unwrap();

        let bef_str = &captures.name("before").unwrap().as_str().trim().to_string();
        let aft_str = &captures.name("after").unwrap().as_str().trim().to_string();

        let mut out_redir_tree = Vec::new();
        
        if bef_str != "" {
            out_redir_tree.push(*build_ast(bef_str, cfg));
        }
        
        if aft_str != "" {
            out_redir_tree.push(*build_ast(aft_str, cfg));
        }
        
        root.children.push(tree::TreeNode {
            value: Box::new(classify_token(&String::from(">"), cfg)),
            children: out_redir_tree
        });
    }

    else if regexp[4].is_match(command) {
        let captures = regexp[4].captures(command).unwrap();

        let bef_str = &captures.name("before").unwrap().as_str().trim().to_string();
        let aft_str = &captures.name("after").unwrap().as_str().trim().to_string();

        let mut out_redir_tree = Vec::new();

        if bef_str != "" {
            out_redir_tree.push(*build_ast(bef_str, cfg));
        }

        if aft_str != "" {
            out_redir_tree.push(*build_ast(aft_str, cfg));
        }

        root.children.push(tree::TreeNode {
            value: Box::new(classify_token(&String::from(">>"), cfg)),
            children: out_redir_tree
        })
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
