use std::collections::HashMap;

pub enum BlockTokens {
    HEADING(String, u8),
    CODE(Vec<String>, Option<String>),
    CUSTOM(String, Option<HashMap<String, String>>, Option<String>),
    BULLET(Vec<String>),
    LIST(Vec<String>),
    PARAGRAPH(String),
}

enum State {
    Default,
    InCodeBlock { lang: Option<String>, content: Vec<String> },
    InBulletList { items: Vec<String> },
    InOrderedList { items: Vec<String> },
    InCustomBlock { name: String, props: HashMap<String, String>, content: Vec<String>, in_content: bool },
}

pub fn tokenise_blocks(input: &str) -> Vec<BlockTokens> {
    let mut tokens: Vec<BlockTokens> = Vec::new();
    let mut state = State::Default;

    for line in input.lines() {
        state = match state {
            State::Default => default_line_parse(line, &mut tokens),

            State::InCodeBlock { lang, mut content } => {
                if line == "```" {
                    tokens.push(BlockTokens::CODE(content, lang));
                    State::Default
                } else {
                    content.push(line.to_string());
                    State::InCodeBlock { lang, content }
                }
            }

            State::InBulletList { mut items } => {
                if let Some(pt) = line.strip_prefix("- ") {
                    items.push(pt.to_string());
                    State::InBulletList { items }
                } else {
                    tokens.push(BlockTokens::BULLET(items));
                    default_line_parse(line, &mut tokens)
                }
            }

            State::InOrderedList { mut items } => {
                if let Some(dot_pos) = line.find(". ") {
                    if dot_pos > 0 && line[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
                        items.push(line[dot_pos + 2..].to_string());
                        State::InOrderedList { items }
                    } else {
                        tokens.push(BlockTokens::LIST(items));
                        default_line_parse(line, &mut tokens)
                    }
                } else {
                    tokens.push(BlockTokens::LIST(items));
                    default_line_parse(line, &mut tokens)
                }
            }

            State::InCustomBlock { name, mut props, mut content, in_content } => {
                if line == ":::" {
                    tokens.push(BlockTokens::CUSTOM(
                        name,
                        if props.is_empty() { None } else { Some(props) },
                        if content.is_empty() { None } else { Some(content.join("\n")) },
                    ));
                    State::Default
                } else if in_content || line.is_empty() {
                    if !in_content && line.is_empty() {
                        State::InCustomBlock { name, props, content, in_content: true }
                    } else {
                        content.push(line.to_string());
                        State::InCustomBlock { name, props, content, in_content }
                    }
                } else if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_string();
                    let value = line[colon_pos + 1..].trim().to_string();
                    props.insert(key, value);
                    State::InCustomBlock { name, props, content, in_content }
                } else {
                    content.push(line.to_string());
                    State::InCustomBlock { name, props, content, in_content: true }
                }
            }
        };
    }

    // Flush any open state at end of input
    match state {
        State::InCodeBlock { lang, content } => {
            tokens.push(BlockTokens::CODE(content, lang));
        }
        State::InBulletList { items } => {
            tokens.push(BlockTokens::BULLET(items));
        }
        State::InOrderedList { items } => {
            tokens.push(BlockTokens::LIST(items));
        }
        State::InCustomBlock { name, props, content, .. } => {
            tokens.push(BlockTokens::CUSTOM(
                name,
                if props.is_empty() { None } else { Some(props) },
                if content.is_empty() { None } else { Some(content.join("\n")) },
            ));
        }
        State::Default => {}
    }

    tokens
}

fn default_line_parse(line: &str, blocks: &mut Vec<BlockTokens>) -> State {
    // Code fences
    if let Some(lang) = line.strip_prefix("```") {
        let lang = if lang.is_empty() { None } else { Some(lang.to_string()) };
        return State::InCodeBlock { lang, content: Vec::new() };
    }

    // Headings
    if line.starts_with('#') {
        let level = line.chars().take_while(|c| *c == '#').count();
        if level <= 6 && line.chars().nth(level) == Some(' ') {
            let title = line[level + 1..].to_string();
            blocks.push(BlockTokens::HEADING(title, level as u8));
            return State::Default;
        }
    }

    // Bullet lists
    if let Some(pt) = line.strip_prefix("- ") {
        return State::InBulletList { items: vec![pt.to_string()] };
    }

    // Custom blocks
    if let Some(component) = line.strip_prefix("::: ") {
        if component.ends_with(" :::") {
            let name = component.strip_suffix(" :::").unwrap();
            blocks.push(BlockTokens::CUSTOM(name.to_string(), None, None));
            return State::Default;
        }
        return State::InCustomBlock {
            name: component.to_string(),
            props: HashMap::new(),
            content: Vec::new(),
            in_content: false,
        };
    }

    // Ordered lists
    if let Some(dot_pos) = line.find(". ") {
        if dot_pos > 0 && line[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
            let text = line[dot_pos + 2..].to_string();
            return State::InOrderedList { items: vec![text] };
        }
    }

    // Fallback: paragraph
    blocks.push(BlockTokens::PARAGRAPH(line.to_string()));
    State::Default
}