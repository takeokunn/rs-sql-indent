mod aligned;
mod basic;
mod dataops;
mod streamline;

use crate::config::{FormatOptions, FormatStyle};
use crate::token::{KeywordKind, Token};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ClauseContext {
    None,
    Select,
    From,
    Where,
    Set,
    Values,
    Having,
    GroupBy,
    OrderBy,
    Join,
    Ddl,
    Cte,
    Other,
}

pub(crate) fn is_single_value_clause(kw: KeywordKind) -> bool {
    matches!(kw, KeywordKind::Limit | KeywordKind::Offset)
}

pub(crate) fn clause_context_from_keyword(kw: KeywordKind) -> ClauseContext {
    match kw {
        KeywordKind::Select => ClauseContext::Select,
        KeywordKind::From => ClauseContext::From,
        KeywordKind::Where => ClauseContext::Where,
        KeywordKind::Set => ClauseContext::Set,
        KeywordKind::Values => ClauseContext::Values,
        KeywordKind::Having => ClauseContext::Having,
        _ => ClauseContext::Other,
    }
}

pub(crate) fn needs_space_before(token: &Token<'_>, prev: Option<&Token<'_>>) -> bool {
    let Some(prev_token) = prev else {
        return false;
    };

    if let Token::Operator(op) = token
        && (*op == "::" || *op == "->" || *op == "->>")
    {
        return false;
    }
    if let Token::Operator(op) = prev_token
        && (*op == "::" || *op == "->" || *op == "->>")
    {
        return false;
    }

    !matches!(
        (prev_token, token),
        (Token::OpenParen, _)
            | (_, Token::CloseParen)
            | (Token::Dot, _)
            | (_, Token::Dot)
            | (_, Token::Comma)
            | (_, Token::Semicolon)
    )
}

pub(crate) struct FormatterBase<'a> {
    pub(crate) tokens: &'a [Token<'a>],
    pub(crate) options: &'a FormatOptions,
    pub(crate) paren_depth: usize,
    pub(crate) is_subquery_paren: Vec<bool>,
    pub(crate) inline_paren_depth: usize,
    pub(crate) clause_context: ClauseContext,
    pub(crate) is_first_token: bool,
    pub(crate) prev_was_ddl_starter: bool,
    pub(crate) output: String,
}

impl<'a> FormatterBase<'a> {
    pub(crate) fn new(tokens: &'a [Token<'a>], options: &'a FormatOptions) -> Self {
        Self {
            tokens,
            options,
            paren_depth: 0,
            is_subquery_paren: Vec::new(),
            inline_paren_depth: 0,
            clause_context: ClauseContext::None,
            is_first_token: true,
            prev_was_ddl_starter: false,
            output: String::new(),
        }
    }

    pub(crate) fn is_inline(&self) -> bool {
        self.inline_paren_depth > 0
    }

    pub(crate) fn keyword_str(&self, kw: KeywordKind) -> String {
        if self.options.uppercase {
            kw.as_str().to_string()
        } else {
            kw.as_str().to_lowercase()
        }
    }
}

pub(crate) trait SqlFormatter<'a> {
    fn base(&self) -> &FormatterBase<'a>;
    fn base_mut(&mut self) -> &mut FormatterBase<'a>;

    fn format_keyword(&mut self, kw: KeywordKind, prev_token: Option<&Token<'a>>);
    fn format_comma(&mut self);
    fn format_open_paren(
        &mut self,
        filtered: &[&Token<'a>],
        idx: usize,
        prev_token: Option<&Token<'a>>,
    );
    fn format_close_paren(&mut self);
    fn format_semicolon(&mut self);
    fn format_value(&mut self, text: &str, prev_token: Option<&Token<'a>>, token: &Token<'a>);

    fn on_comment(&mut self) {}
    fn on_dot(&mut self) {}

    fn finalize_output(&mut self) -> String {
        let output = &mut self.base_mut().output;
        let trimmed = output.trim_end().len();
        output.truncate(trimmed);
        std::mem::take(output)
    }

    fn format(&mut self) -> String {
        let tokens: &'a [Token<'a>] = self.base().tokens;

        let filtered: Vec<&Token<'a>> = tokens
            .iter()
            .filter(|t| !matches!(t, Token::Whitespace(_)))
            .collect();

        let mut i = 0;
        let mut prev_token: Option<&Token<'a>> = None;

        while i < filtered.len() {
            let token = filtered[i];

            match token {
                Token::Keyword(kw) => {
                    if matches!(prev_token, Some(Token::Dot)) {
                        let name = kw.as_str().to_lowercase();
                        self.format_value(&name, prev_token, token);
                    } else {
                        self.format_keyword(*kw, prev_token);
                    }
                }
                Token::Comma => {
                    self.format_comma();
                }
                Token::OpenParen => {
                    self.format_open_paren(&filtered, i, prev_token);
                }
                Token::CloseParen => {
                    self.format_close_paren();
                }
                Token::Semicolon => {
                    self.format_semicolon();
                }
                Token::LineComment(text) => {
                    if !self.base().is_first_token {
                        self.base_mut().output.push(' ');
                    }
                    self.base_mut().output.push_str("--");
                    self.base_mut().output.push_str(text);
                    self.base_mut().is_first_token = false;
                    self.on_comment();
                }
                Token::BlockComment(text) => {
                    if !self.base().is_first_token && needs_space_before(token, prev_token) {
                        self.base_mut().output.push(' ');
                    }
                    self.base_mut().output.push_str("/*");
                    self.base_mut().output.push_str(text);
                    self.base_mut().output.push_str("*/");
                    self.base_mut().is_first_token = false;
                    self.on_comment();
                }
                Token::Dot => {
                    self.base_mut().output.push('.');
                    self.base_mut().is_first_token = false;
                    self.on_dot();
                }
                Token::Identifier(name) => {
                    self.format_value(name, prev_token, token);
                }
                Token::QuotedIdentifier(name) => {
                    let quoted = format!("\"{}\"", name);
                    self.format_value(&quoted, prev_token, token);
                }
                Token::StringLiteral(val) => {
                    let literal = format!("'{}'", val);
                    self.format_value(&literal, prev_token, token);
                }
                Token::NumberLiteral(val) => {
                    self.format_value(val, prev_token, token);
                }
                Token::Operator(op) => {
                    self.format_value(op, prev_token, token);
                }
                Token::TemplateVariable(content) => {
                    let text = format!("{{{{{}}}}}", content);
                    self.format_value(&text, prev_token, token);
                }
                Token::Whitespace(_) => {}
            }

            prev_token = Some(token);
            i += 1;
        }

        self.finalize_output()
    }
}

pub fn format_tokens(tokens: &[Token<'_>], options: &FormatOptions) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    match options.style {
        FormatStyle::Basic => basic::format(tokens, options),
        FormatStyle::Streamline => streamline::format(tokens, options),
        FormatStyle::Aligned => aligned::format(tokens, options),
        FormatStyle::Dataops => dataops::format(tokens, options),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tokens() {
        let result = format_tokens(&[], &FormatOptions::default());
        assert_eq!(result, "");
    }
}
