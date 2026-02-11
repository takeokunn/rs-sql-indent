pub mod config;
pub mod formatter;
pub mod lexer;
pub mod token;

pub use config::{FormatOptions, FormatStyle};

pub fn format_sql(input: &str, options: &FormatOptions) -> String {
    let tokens = lexer::tokenize(input);
    formatter::format_tokens(&tokens, options)
}
