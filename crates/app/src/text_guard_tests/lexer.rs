//! A small reader of Rust source: the string literals of a file with the code in front of each,
//! and the code alone (no comments, no string contents).

/// A string literal: where it is, what it says, and the (up to 80) characters of code before it
/// (comments dropped, earlier literals replaced by `\u{1}`).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Literal {
    pub(super) line: usize,
    pub(super) text: String,
    pub(super) before: String,
}

/// Everything the reader finds in one file.
pub(super) struct Scan {
    pub(super) literals: Vec<Literal>,
    /// The code with comments removed and every literal replaced by `\u{1}`.
    pub(super) code: String,
}

/// Reads `src`. A `'` starts a char literal only when it closes (`'x'`, `'\n'`); otherwise it is a
/// lifetime. Block comments nest.
pub(super) fn scan(src: &str) -> Scan {
    let chars: Vec<char> = src.chars().collect();
    let mut literals = Vec::new();
    let mut code = String::new();
    let (mut i, mut line) = (0, 1);
    let tail = |code: &str| -> String {
        let mut t: Vec<char> = code.chars().rev().take(80).collect();
        t.reverse();
        t.into_iter().collect()
    };
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
        } else if c == '"' || (c == 'r' && is_raw_start(&chars, i)) {
            let start_line = line;
            let before = tail(&code);
            let mut text = String::new();
            if c == 'r' {
                let mut hashes = 0;
                i += 1;
                while chars[i] == '#' {
                    hashes += 1;
                    i += 1;
                }
                i += 1; // the opening quote
                while i < chars.len() {
                    if chars[i] == '"' && (0..hashes).all(|k| chars.get(i + 1 + k) == Some(&'#')) {
                        i += 1 + hashes;
                        break;
                    }
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    text.push(chars[i]);
                    i += 1;
                }
            } else {
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' {
                        text.push(chars[i]);
                        i += 1;
                    }
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    text.push(chars[i]);
                    i += 1;
                }
                i += 1;
            }
            literals.push(Literal {
                line: start_line,
                text,
                before,
            });
            code.push('\u{1}');
        } else if c == '\'' {
            // `'x'` or `'\n'`: a char literal; anything else is a lifetime.
            let close = if next == Some('\\') {
                (i + 3..chars.len().min(i + 12)).find(|&k| chars[k] == '\'')
            } else if chars.get(i + 2) == Some(&'\'') {
                Some(i + 2)
            } else {
                None
            };
            match close {
                Some(k) => {
                    code.push('\u{1}');
                    i = k + 1;
                }
                None => {
                    code.push(c);
                    i += 1;
                }
            }
        } else {
            if c == '\n' {
                line += 1;
            }
            code.push(c);
            i += 1;
        }
    }
    Scan { literals, code }
}

/// `r"`, `r#"`, `r##"` … but not an identifier that ends in `r` (`for"` cannot occur; `bar#` is not
/// a raw string).
fn is_raw_start(chars: &[char], i: usize) -> bool {
    if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') && chars[i - 1] != 'b' {
        return false;
    }
    let mut k = i + 1;
    while chars.get(k) == Some(&'#') {
        k += 1;
    }
    chars.get(k) == Some(&'"')
}

#[test]
fn the_reader_tells_code_comments_and_literals_apart() {
    let src = r##"
// text("in a comment")
/* nested /* "in a block" */ still */
let a = text("Hello"); // "after"
let b = r#"raw "quoted" text"#;
let c = '"';
fn f<'a>(x: &'a str) -> &'a str { "x\"y" }
let d = "// not a comment";
"##;
    let s = scan(src);
    let texts: Vec<&str> = s.literals.iter().map(|l| l.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "Hello",
            r#"raw "quoted" text"#,
            r#"x\"y"#,
            "// not a comment"
        ]
    );
    assert_eq!(s.literals[0].line, 4);
    assert!(s.literals[0].before.trim_end().ends_with("text("));
    assert!(!s.code.contains("Hello") && !s.code.contains("comment"));
    assert!(
        s.code.contains("fn f<'a>"),
        "a lifetime is code: {}",
        s.code
    );
}
