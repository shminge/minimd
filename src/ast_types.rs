

pub enum Inline {
    Text(String),
    Italic(Vec<Inline>),
    Bold(Vec<Inline>),
    Link { url: String, title: Vec<Inline> },
}