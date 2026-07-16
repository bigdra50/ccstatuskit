//! The `format` template: `"$model $git\n${custom.unilyze}"`.
//!
//! Single-pass parser. `$name` / `${name}` reference builtin modules,
//! `${custom.x}` references a `[custom.x]` entry (the dot requires braces).
//! `\$` and `\\` escape, a bare `$` before a non-identifier stays literal,
//! and each newline starts a new statusline row. Rows with no tokens are
//! dropped so TOML triple-quoted strings don't produce blank lines.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Text(String),
    Module(ModuleRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleRef {
    Builtin(String),
    Custom(String),
}

pub type Row = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    UnclosedBrace,
    EmptyModuleName,
    UnknownNamespace(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::UnclosedBrace => write!(f, "unclosed ${{ in format template"),
            FormatError::EmptyModuleName => write!(f, "empty module name in format template"),
            FormatError::UnknownNamespace(ns) => {
                write!(
                    f,
                    "unknown namespace `{ns}` in format template (only `custom` is supported)"
                )
            }
        }
    }
}

impl std::error::Error for FormatError {}

pub fn parse_format(template: &str) -> Result<Vec<Row>, FormatError> {
    let mut rows: Vec<Row> = Vec::new();
    let mut row: Row = Vec::new();
    let mut literal = String::new();
    let mut chars = template.chars().peekable();

    fn flush_literal(literal: &mut String, row: &mut Row) {
        if !literal.is_empty() {
            row.push(Token::Text(std::mem::take(literal)));
        }
    }

    fn flush_row(row: &mut Row, rows: &mut Vec<Row>) {
        let finished = std::mem::take(row);
        if !finished.is_empty() {
            rows.push(finished);
        }
    }

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(escaped @ ('$' | '\\')) => literal.push(escaped),
                Some(other) => {
                    literal.push('\\');
                    literal.push(other);
                }
                None => literal.push('\\'),
            },
            '\n' => {
                flush_literal(&mut literal, &mut row);
                flush_row(&mut row, &mut rows);
            }
            '$' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => name.push(c),
                            None => return Err(FormatError::UnclosedBrace),
                        }
                    }
                    flush_literal(&mut literal, &mut row);
                    row.push(Token::Module(parse_braced_name(&name)?));
                } else {
                    let mut name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            name.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if name.is_empty() {
                        // Bare `$` before a non-identifier stays literal.
                        literal.push('$');
                    } else {
                        flush_literal(&mut literal, &mut row);
                        row.push(Token::Module(ModuleRef::Builtin(name)));
                    }
                }
            }
            _ => literal.push(c),
        }
    }
    flush_literal(&mut literal, &mut row);
    flush_row(&mut row, &mut rows);
    Ok(rows)
}

fn parse_braced_name(name: &str) -> Result<ModuleRef, FormatError> {
    if name.is_empty() {
        return Err(FormatError::EmptyModuleName);
    }
    match name.split_once('.') {
        None => Ok(ModuleRef::Builtin(name.to_string())),
        Some(("custom", rest)) => {
            if rest.is_empty() {
                Err(FormatError::EmptyModuleName)
            } else {
                Ok(ModuleRef::Custom(rest.to_string()))
            }
        }
        Some((ns, _)) => Err(FormatError::UnknownNamespace(ns.to_string())),
    }
}
