use crate::config;

fn if_keyword(argv: &Vec<&String>, config: &mut config::Config) -> Result<(i32, i32), String> {
    Ok((1, 0))
}

pub fn load_keywords(cfg: &mut config::Config) {
    cfg.keywords.insert(String::from("if"), if_keyword);
}
