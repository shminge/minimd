use crate::inline_tokeniser::{tokenise, Tokens};

pub enum Inline {
    Text(String),
    Italic(Vec<Inline>),
    Bold(Vec<Inline>),
    Wikilink { url: String, title: Vec<Inline> },
    Hyperlink { url: String, title: Vec<Inline> },
    Image {url: String, alt_text: String },
}

pub fn parse_inline(content: &str) -> Vec<Inline> {
    let tokens = tokenise(content);
    parse_inline_tokens(&tokens)
}

fn parse_inline_tokens(tokens: &[Tokens]) -> Vec<Inline> {
    let mut result: Vec<Inline> = Vec::new();
    let mut pos: usize = 0;

    while pos < tokens.len() {
        match &tokens[pos] {
            // Backslash consumes itself and specifies the next character as literal text
            Tokens::BACKSLASH => {
                if let Some(t) = tokens.get(pos + 1) {
                    result.add_text(&t.to_string());
                    pos += 2
                } else {
                    // if we read a backslash with no following character, we throw it away (ending the parse
                    pos += 1
                }
            }
            Tokens::TEXT(s) => {
                result.add_text(s);
                pos += 1
            }
            Tokens::ASTERISK if tokens.get(pos + 1) == Some(&Tokens::ASTERISK) => {
                // We're trying to match a bold section
                if let Some((offset, content_tokens)) =
                    find_tokens(&[Tokens::ASTERISK, Tokens::ASTERISK], &tokens[pos+2..]) {
                    let bold = Inline::Bold(parse_inline_tokens(content_tokens));
                    pos += offset + 2;
                    result.push(bold);
                } else {
                    pos += 2;
                    result.add_text("**");
                }
            }
            Tokens::ASTERISK => {
                if let Some((offset, content_tokens)) =
                    find_tokens(&[Tokens::ASTERISK], &tokens[pos+1..]) {
                    let italics = Inline::Italic(parse_inline_tokens(content_tokens));
                    pos += offset + 1;
                    result.push(italics);
                } else {
                    pos += 1;
                    result.add_text("*");
                }
            }
            Tokens::EXCLAMATION => {
                if let Some((advance, url, alt_tokens)) = match_link(&tokens[pos+1..]) {
                    let alt_text = alt_tokens.iter().map(|t| t.to_string()).collect();
                    result.push(Inline::Image { url, alt_text });
                    pos += advance + 1;
                } else {
                    pos += 1;
                    result.add_text("!");
                }
            }
            Tokens::LBRACKET if tokens.get(pos + 1) == Some(&Tokens::LBRACKET) => {
                if let Some((offset, content_tokens)) =
                    find_tokens(&[Tokens::RBRACKET, Tokens::RBRACKET], &tokens[pos+2..]) {
                    pos += offset + 2;

                    if let Some(idx) = content_tokens.iter().position(|t| *t == Tokens::PIPE) {
                        let (before, rest) = content_tokens.split_at(idx);
                        let after = &rest[1..];
                        result.push(Inline::Wikilink
                        {
                            url: before.iter().map(|t| t.to_string()).collect(),
                            title: parse_inline_tokens(after),
                        });
                    } else {
                        let content: String = content_tokens.iter().map(|t| t.to_string()).collect();
                        result.push(Inline::Wikilink {
                            url: content.clone(),
                            title: Vec::from([Inline::Text(content)]),
                        })
                    }
                } else {
                    pos += 2;
                    result.add_text("[[");
                }
            }

            Tokens::LBRACKET => {
                if let Some((advance, url, title)) = match_link(&tokens[pos..]) {
                    result.push(Inline::Hyperlink { url,  title: parse_inline_tokens(title) });
                    pos += advance;
                } else {
                    pos += 1;
                    result.add_text("[");
                }
            }

            t => {
                result.add_text(&t.to_string());
                pos += 1
            }

        }
    }



    result
}

/// Search for a particular token string to end the delimiter
fn find_tokens<'a>(
    target: &[Tokens],
    tokens: &'a [Tokens],
) -> Option<(usize, &'a [Tokens])> {
    let mut pos = 0;
    while pos + target.len() <= tokens.len() {
        if tokens[pos..pos + target.len()] == *target {
            return Some((pos + target.len(), &tokens[..pos]));
        }
        pos += 1;
    }
    None
}

/// Matches a link of the form `[.*?](.*?)` into `Option<Url,Title>`
fn match_link(tokens: &[Tokens]) -> Option<(usize, String, &[Tokens])> {
    if tokens.first() != Some(&Tokens::LBRACKET) {
        return None;
    }

    let after_bracket = &tokens[1..];

    let (offset, title_tokens) =
        find_tokens(&[Tokens::RBRACKET, Tokens::LPAREN], after_bracket)?;

    let after_rparen = &after_bracket[offset..];

    let (offset2, url_tokens) =
        find_tokens(&[Tokens::RPAREN], after_rparen)?;

    let url: String = url_tokens.iter().map(|t| t.to_string()).collect();
    let total = 1 + offset + offset2; // [ + title + ]( + url + )

    Some((total, url, title_tokens))
}


trait TextAdd {
    fn add_text(&mut self, text: &str);
}

impl TextAdd for Vec<Inline> {
    fn add_text(&mut self, text: &str) {
        if let Some(Inline::Text(s)) = self.last_mut() {
            s.push_str(text);
        } else {
            self.push(Inline::Text(text.to_string()));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper: flatten Inline tree to a simple string for easy assertions ──

    fn inline_to_string(inlines: &[Inline]) -> String {
        inlines.iter().map(|i| match i {
            Inline::Text(s) => s.clone(),
            Inline::Bold(inner) => format!("<b>{}</b>", inline_to_string(inner)),
            Inline::Italic(inner) => format!("<i>{}</i>", inline_to_string(inner)),
            Inline::Hyperlink { url, title } =>
                format!("<a href=\"{}\">{}</a>", url, inline_to_string(title)),
            Inline::Wikilink { url, title } =>
                format!("<wiki url=\"{}\">{}</wiki>", url, inline_to_string(title)),
            Inline::Image { url, alt_text } =>
                format!("<img src=\"{}\" alt=\"{}\"/>", url, alt_text),
        }).collect()
    }

    // ═══════════════════════════════════════
    //  Plain text
    // ═══════════════════════════════════════

    #[test]
    fn plain_text() {
        let result = parse_inline("hello world");
        assert_eq!(inline_to_string(&result), "hello world");
    }

    #[test]
    fn empty_input() {
        let result = parse_inline("");
        assert!(result.is_empty());
    }

    #[test]
    fn only_whitespace() {
        let result = parse_inline("   ");
        assert_eq!(inline_to_string(&result), "   ");
    }

    // ═══════════════════════════════════════
    //  Bold
    // ═══════════════════════════════════════

    #[test]
    fn simple_bold() {
        let result = parse_inline("**hello**");
        assert_eq!(inline_to_string(&result), "<b>hello</b>");
    }

    #[test]
    fn bold_with_surrounding_text() {
        let result = parse_inline("before **bold** after");
        assert_eq!(inline_to_string(&result), "before <b>bold</b> after");
    }

    #[test]
    fn unmatched_bold_opener() {
        let result = parse_inline("**hello");
        assert_eq!(inline_to_string(&result), "**hello");
    }

    #[test]
    fn empty_bold() {
        let result = parse_inline("****");
        assert_eq!(inline_to_string(&result), "<b></b>");
    }

    #[test]
    fn multiple_bold_sections() {
        let result = parse_inline("**a** and **b**");
        assert_eq!(inline_to_string(&result), "<b>a</b> and <b>b</b>");
    }

    // ═══════════════════════════════════════
    //  Italic
    // ═══════════════════════════════════════

    #[test]
    fn simple_italic() {
        let result = parse_inline("*hello*");
        assert_eq!(inline_to_string(&result), "<i>hello</i>");
    }

    #[test]
    fn italic_with_surrounding_text() {
        let result = parse_inline("before *italic* after");
        assert_eq!(inline_to_string(&result), "before <i>italic</i> after");
    }

    #[test]
    fn unmatched_italic_opener() {
        let result = parse_inline("*hello");
        assert_eq!(inline_to_string(&result), "*hello");
    }

    #[test]
    fn empty_italic() {
        let result = parse_inline("**");
        // Two asterisks with nothing to close bold — falls through to unmatched bold
        assert_eq!(inline_to_string(&result), "**");
    }

    #[test]
    fn multiple_italic_sections() {
        let result = parse_inline("*a* and *b*");
        assert_eq!(inline_to_string(&result), "<i>a</i> and <i>b</i>");
    }

    // ═══════════════════════════════════════
    //  Nesting: bold inside italic and vice versa
    // ═══════════════════════════════════════

    // #[test]
    // fn bold_inside_italic() {
    //     let result = parse_inline("*hello **world***");
    //     assert_eq!(inline_to_string(&result), "<i>hello <b>world</b></i>");
    // }
    //
    // #[test]
    // fn italic_inside_bold() {
    //     let result = parse_inline("**hello *world***");
    //     assert_eq!(inline_to_string(&result), "<b>hello <i>world</i></b>");
    // }

    // ═══════════════════════════════════════
    //  Backslash escaping
    // ═══════════════════════════════════════

    #[test]
    fn escape_asterisk() {
        let result = parse_inline("\\*not italic\\*");
        assert_eq!(inline_to_string(&result), "*not italic*");
    }

    #[test]
    fn escape_bracket() {
        let result = parse_inline("\\[not a link\\]");
        assert_eq!(inline_to_string(&result), "[not a link]");
    }

    #[test]
    fn escape_backslash() {
        let result = parse_inline("\\\\hello");
        assert_eq!(inline_to_string(&result), "\\hello");
    }

    #[test]
    fn trailing_backslash() {
        // Backslash at end of input with nothing after it — gets discarded
        let result = parse_inline("hello\\");
        assert_eq!(inline_to_string(&result), "hello");
    }

    #[test]
    fn escape_inside_bold() {
        let result = parse_inline("**hello \\* world**");
        assert_eq!(inline_to_string(&result), "<b>hello * world</b>");
    }

    // ═══════════════════════════════════════
    //  Hyperlinks [title](url)
    // ═══════════════════════════════════════

    #[test]
    fn simple_hyperlink() {
        let result = parse_inline("[click here](https://example.com)");
        assert_eq!(
            inline_to_string(&result),
            "<a href=\"https://example.com\">click here</a>"
        );
    }

    #[test]
    fn hyperlink_with_surrounding_text() {
        let result = parse_inline("go to [site](https://x.com) now");
        assert_eq!(
            inline_to_string(&result),
            "go to <a href=\"https://x.com\">site</a> now"
        );
    }

    #[test]
    fn unmatched_bracket_no_link() {
        let result = parse_inline("[just a bracket");
        assert_eq!(inline_to_string(&result), "[just a bracket");
    }

    #[test]
    fn bracket_without_paren() {
        let result = parse_inline("[text] no paren");
        assert_eq!(inline_to_string(&result), "[text] no paren");
    }

    #[test]
    fn hyperlink_with_bold_title() {
        let result = parse_inline("[**bold link**](https://example.com)");
        assert_eq!(
            inline_to_string(&result),
            "<a href=\"https://example.com\"><b>bold link</b></a>"
        );
    }

    // ═══════════════════════════════════════
    //  Wikilinks [[url]] and [[url|title]]
    // ═══════════════════════════════════════

    #[test]
    fn simple_wikilink() {
        let result = parse_inline("[[some page]]");
        assert_eq!(
            inline_to_string(&result),
            "<wiki url=\"some page\">some page</wiki>"
        );
    }

    #[test]
    fn wikilink_with_title() {
        let result = parse_inline("[[some page|display text]]");
        assert_eq!(
            inline_to_string(&result),
            "<wiki url=\"some page\">display text</wiki>"
        );
    }

    #[test]
    fn wikilink_with_surrounding_text() {
        let result = parse_inline("see [[page]] for info");
        assert_eq!(
            inline_to_string(&result),
            "see <wiki url=\"page\">page</wiki> for info"
        );
    }

    #[test]
    fn unmatched_double_bracket() {
        let result = parse_inline("[[no closing");
        assert_eq!(inline_to_string(&result), "[[no closing");
    }

    #[test]
    fn wikilink_with_formatted_title() {
        let result = parse_inline("[[page|**bold title**]]");
        assert_eq!(
            inline_to_string(&result),
            "<wiki url=\"page\"><b>bold title</b></wiki>"
        );
    }

    // ═══════════════════════════════════════
    //  Images ![alt](url)
    // ═══════════════════════════════════════

    #[test]
    fn simple_image() {
        let result = parse_inline("![alt text](https://img.com/pic.png)");
        assert_eq!(
            inline_to_string(&result),
            "<img src=\"https://img.com/pic.png\" alt=\"alt text\"/>"
        );
    }

    #[test]
    fn image_with_surrounding_text() {
        let result = parse_inline("before ![pic](url.png) after");
        assert_eq!(
            inline_to_string(&result),
            "before <img src=\"url.png\" alt=\"pic\"/> after"
        );
    }

    #[test]
    fn unmatched_exclamation() {
        let result = parse_inline("!not an image");
        assert_eq!(inline_to_string(&result), "!not an image");
    }

    #[test]
    fn exclamation_without_valid_link() {
        let result = parse_inline("![alt but no paren");
        assert_eq!(inline_to_string(&result), "![alt but no paren");
    }

    // ═══════════════════════════════════════
    //  Special characters as plain text
    // ═══════════════════════════════════════

    #[test]
    fn lone_rbracket() {
        let result = parse_inline("a ] b");
        assert_eq!(inline_to_string(&result), "a ] b");
    }

    #[test]
    fn lone_rparen() {
        let result = parse_inline("a ) b");
        assert_eq!(inline_to_string(&result), "a ) b");
    }

    #[test]
    fn lone_colon() {
        let result = parse_inline("key: value");
        assert_eq!(inline_to_string(&result), "key: value");
    }

    #[test]
    fn lone_lparen() {
        let result = parse_inline("a ( b");
        assert_eq!(inline_to_string(&result), "a ( b");
    }

    // ═══════════════════════════════════════
    //  Mixed / complex
    // ═══════════════════════════════════════

    #[test]
    fn bold_and_link_together() {
        let result = parse_inline("**bold** and [link](url)");
        assert_eq!(
            inline_to_string(&result),
            "<b>bold</b> and <a href=\"url\">link</a>"
        );
    }

    #[test]
    fn italic_and_wikilink() {
        let result = parse_inline("*italic* then [[page]]");
        assert_eq!(
            inline_to_string(&result),
            "<i>italic</i> then <wiki url=\"page\">page</wiki>"
        );
    }

    #[test]
    fn everything_together() {
        let result = parse_inline("**bold** *italic* [link](url) [[wiki]] ![img](pic.png)");
        assert_eq!(
            inline_to_string(&result),
            "<b>bold</b> <i>italic</i> <a href=\"url\">link</a> <wiki url=\"wiki\">wiki</wiki> <img src=\"pic.png\" alt=\"img\"/>"
        );
    }

    // ═══════════════════════════════════════
    //  find_tokens unit tests
    // ═══════════════════════════════════════

    #[test]
    fn find_tokens_at_start() {
        let tokens = tokenise("**hello");
        let result = find_tokens(&[Tokens::ASTERISK, Tokens::ASTERISK], &tokens);
        assert!(result.is_some());
        let (offset, before) = result.unwrap();
        assert!(before.is_empty());
        assert_eq!(offset, 2);
    }

    #[test]
    fn find_tokens_not_found() {
        let tokens = tokenise("hello world");
        let result = find_tokens(&[Tokens::ASTERISK], &tokens);
        assert!(result.is_none());
    }

    #[test]
    fn find_tokens_at_end() {
        let tokens = tokenise("hello**");
        let result = find_tokens(&[Tokens::ASTERISK, Tokens::ASTERISK], &tokens);
        assert!(result.is_some());
        let (_, before) = result.unwrap();
        assert_eq!(before.len(), 1); // TEXT("hello")
    }

    // ═══════════════════════════════════════
    //  match_link unit tests
    // ═══════════════════════════════════════

    #[test]
    fn match_link_valid() {
        let tokens = tokenise("[title](url)");
        let result = match_link(&tokens);
        assert!(result.is_some());
        let (advance, url, _title_tokens) = result.unwrap();
        assert_eq!(url, "url");
        assert_eq!(advance, tokens.len());
    }

    #[test]
    fn match_link_no_opening_bracket() {
        let tokens = tokenise("not a link");
        let result = match_link(&tokens);
        assert!(result.is_none());
    }

    #[test]
    fn match_link_no_closing_paren() {
        let tokens = tokenise("[title](url");
        let result = match_link(&tokens);
        assert!(result.is_none());
    }

    #[test]
    fn match_link_no_paren_after_bracket() {
        let tokens = tokenise("[title] nope");
        let result = match_link(&tokens);
        assert!(result.is_none());
    }

    // ═══════════════════════════════════════
    //  TextAdd trait tests
    // ═══════════════════════════════════════

    #[test]
    fn add_text_to_empty_vec() {
        let mut v: Vec<Inline> = Vec::new();
        v.add_text("hello");
        assert_eq!(inline_to_string(&v), "hello");
    }

    #[test]
    fn add_text_merges_adjacent() {
        let mut v: Vec<Inline> = Vec::new();
        v.add_text("hello ");
        v.add_text("world");
        assert_eq!(inline_to_string(&v), "hello world");
        assert_eq!(v.len(), 1); // should be merged into one Text node
    }

    #[test]
    fn add_text_after_non_text() {
        let mut v: Vec<Inline> = Vec::new();
        v.push(Inline::Bold(vec![Inline::Text("b".into())]));
        v.add_text("after");
        assert_eq!(v.len(), 2);
    }
}
