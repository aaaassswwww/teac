use pest_derive::Parser as DeriveParser;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Syntax(String),

    #[error("invalid integer literal `{literal}` at line {line}, column {column}")]
    InvalidNumber {
        literal: String,
        line: usize,
        column: usize,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("invalid float literal")]
    InvalidFloat {
        literal: String,
        line: usize,
        column: usize,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("unexpected parse tree structure in {0}")]
    Grammar(String),
}

#[derive(DeriveParser)]
#[grammar = "tealang.pest"]
pub(crate) struct TeaLangParser;

pub(crate) type ParseResult<T> = Result<T, Error>;
pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;

pub(crate) fn compact_snippet(snippet: &str) -> String {
    const MAX_CHARS: usize = 48;

    let compact = snippet.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized = if compact.is_empty() {
        snippet.trim().to_string()
    } else {
        compact
    };

    if normalized.is_empty() {
        return "<empty>".to_string();
    }

    let mut chars = normalized.chars();
    let preview: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}

pub(crate) fn grammar_error(context: &'static str, pair: &Pair<'_>) -> Error {
    let span = pair.as_span();
    let (line, column) = span.start_pos().line_col();
    let near = compact_snippet(span.as_str());

    Error::Grammar(format!(
        "{context} at line {line}, column {column}, near `{near}`"
    ))
}

pub(crate) fn grammar_error_static(context: &'static str) -> Error {
    Error::Grammar(context.to_string())
}

pub(crate) fn get_pos(pair: &Pair<'_>) -> usize {
    pair.as_span().start()
}

pub(crate) fn parse_num(pair: Pair) -> ParseResult<i32> {
    let literal = pair.as_str().to_string();
    let (line, column) = pair.as_span().start_pos().line_col();

    literal.parse().map_err(|source| Error::InvalidNumber {
        literal,
        line,
        column,
        source,
    })
}


pub(crate) fn parse_float(pair: Pair) -> ParseResult<f32> {
    let literal = pair.as_str().to_string();
    let (line, column) = pair.as_span().start_pos().line_col();

    literal.parse().map_err(|source| Error::InvalidFloat {
        literal,
        line,
        column,
        source,
    })
}
