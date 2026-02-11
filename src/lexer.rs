use crate::token::{KeywordKind, Token, lookup_keyword};

const TWO_WORD_KEYWORDS: &[(KeywordKind, &str, KeywordKind)] = &[
    (KeywordKind::Order, "BY", KeywordKind::OrderBy),
    (KeywordKind::Group, "BY", KeywordKind::GroupBy),
    (KeywordKind::Left, "JOIN", KeywordKind::LeftJoin),
    (KeywordKind::Right, "JOIN", KeywordKind::RightJoin),
    (KeywordKind::Inner, "JOIN", KeywordKind::InnerJoin),
    (KeywordKind::Outer, "JOIN", KeywordKind::OuterJoin),
    (KeywordKind::Cross, "JOIN", KeywordKind::CrossJoin),
    (KeywordKind::Union, "ALL", KeywordKind::UnionAll),
    (KeywordKind::Primary, "KEY", KeywordKind::PrimaryKey),
    (KeywordKind::Foreign, "KEY", KeywordKind::ForeignKey),
    (KeywordKind::Rows, "BETWEEN", KeywordKind::RowsBetween),
    (KeywordKind::Range, "BETWEEN", KeywordKind::RangeBetween),
];

const THREE_CHAR_OPS: &[&[u8]] = &[b"->>"];
const TWO_CHAR_OPS: &[&[u8]] = &[b"<>", b"!=", b"<=", b">=", b"||", b"::", b"->"];

struct Lexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.pos).copied()?;
        self.pos += 1;
        Some(byte)
    }

    fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        let b = self.peek()?;

        match b {
            // Whitespace
            b if b.is_ascii_whitespace() => Some(self.lex_whitespace()),

            // Line comment: --
            b'-' if self.peek_at(1) == Some(b'-') => Some(self.lex_line_comment()),

            // Block comment: /*
            b'/' if self.peek_at(1) == Some(b'*') => Some(self.lex_block_comment()),

            // String literal: 'single-quoted'
            b'\'' => Some(self.lex_string_literal()),

            // Quoted identifier: "double-quoted"
            b'"' => Some(self.lex_quoted_identifier()),

            // Number literal starting with digit
            b'0'..=b'9' => Some(self.lex_number()),

            // Number literal starting with dot followed by digit
            b'.' if matches!(self.peek_at(1), Some(b'0'..=b'9')) => Some(self.lex_number()),

            // Punctuation (single-char tokens)
            b',' | b';' | b'.' | b'(' | b')' => {
                self.advance();
                Some(match b {
                    b',' => Token::Comma,
                    b';' => Token::Semicolon,
                    b'.' => Token::Dot,
                    b'(' => Token::OpenParen,
                    _ => Token::CloseParen,
                })
            }

            // Template variable: {{...}}
            b'{' if self.peek_at(1) == Some(b'{') => Some(self.lex_template_variable()),

            // Lone braces as operators
            b'{' | b'}' => {
                let start = self.pos;
                self.advance();
                Some(Token::Operator(self.slice(start, self.pos)))
            }

            // Operators (multi-char checked first)
            b'<' | b'>' | b'!' | b'=' | b'|' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'^'
            | b'~' | b':' => Some(self.lex_operator()),

            // Keywords and identifiers
            b if b.is_ascii_alphabetic() || b == b'_' => Some(self.lex_word()),

            // Unknown character: emit as single-char operator
            _ => {
                let start = self.pos;
                self.advance();
                Some(Token::Operator(self.slice(start, self.pos)))
            }
        }
    }

    fn lex_whitespace(&mut self) -> Token<'a> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        Token::Whitespace(self.slice(start, self.pos))
    }

    fn lex_line_comment(&mut self) -> Token<'a> {
        // Skip the '--'
        self.advance();
        self.advance();
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.advance();
        }
        Token::LineComment(self.slice(start, self.pos))
    }

    fn lex_block_comment(&mut self) -> Token<'a> {
        // Skip '/*'
        self.advance();
        self.advance();
        let start = self.pos;
        loop {
            match self.peek() {
                None => break, // unclosed: consume to end
                Some(b'*') if self.peek_at(1) == Some(b'/') => {
                    let end = self.pos;
                    self.advance(); // skip '*'
                    self.advance(); // skip '/'
                    return Token::BlockComment(self.slice(start, end));
                }
                _ => {
                    self.advance();
                }
            }
        }
        Token::BlockComment(self.slice(start, self.pos))
    }

    fn lex_string_literal(&mut self) -> Token<'a> {
        // Skip opening quote
        self.advance();
        let start = self.pos;
        loop {
            match self.peek() {
                None => break, // unclosed: consume to end
                Some(b'\'') => {
                    if self.peek_at(1) == Some(b'\'') {
                        // Escaped quote: include both in raw content
                        self.advance();
                        self.advance();
                    } else {
                        let end = self.pos;
                        self.advance(); // skip closing quote
                        return Token::StringLiteral(self.slice(start, end));
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
        Token::StringLiteral(self.slice(start, self.pos))
    }

    fn lex_quoted_identifier(&mut self) -> Token<'a> {
        // Skip opening quote
        self.advance();
        let start = self.pos;
        loop {
            match self.peek() {
                None => break, // unclosed: consume to end
                Some(b'"') => {
                    let end = self.pos;
                    self.advance(); // skip closing quote
                    return Token::QuotedIdentifier(self.slice(start, end));
                }
                _ => {
                    self.advance();
                }
            }
        }
        Token::QuotedIdentifier(self.slice(start, self.pos))
    }

    fn lex_number(&mut self) -> Token<'a> {
        let start = self.pos;
        // Integer part (may be empty if starting with '.')
        while let Some(b'0'..=b'9') = self.peek() {
            self.advance();
        }
        // Decimal point followed by digits
        if self.peek() == Some(b'.') && matches!(self.peek_at(1), Some(b'0'..=b'9')) {
            self.advance(); // consume '.'
            while let Some(b'0'..=b'9') = self.peek() {
                self.advance();
            }
        }
        Token::NumberLiteral(self.slice(start, self.pos))
    }

    fn lex_operator(&mut self) -> Token<'a> {
        let remaining = &self.bytes[self.pos..];

        // Three-char operators (longest match first)
        if remaining.len() >= 3 {
            for &op in THREE_CHAR_OPS {
                if remaining.starts_with(op) {
                    let start = self.pos;
                    self.pos += 3;
                    return Token::Operator(self.slice(start, self.pos));
                }
            }
        }

        // Two-char operators
        if remaining.len() >= 2 {
            for &op in TWO_CHAR_OPS {
                if remaining.starts_with(op) {
                    let start = self.pos;
                    self.pos += 2;
                    return Token::Operator(self.slice(start, self.pos));
                }
            }
        }

        // Single-char operator
        let start = self.pos;
        self.advance();
        Token::Operator(self.slice(start, self.pos))
    }

    fn lex_template_variable(&mut self) -> Token<'a> {
        let brace_start = self.pos;
        // Skip opening '{{'
        self.advance();
        self.advance();
        let content_start = self.pos;
        loop {
            match self.peek() {
                None => {
                    // Unclosed: emit first '{' as operator
                    self.pos = brace_start + 1;
                    return Token::Operator(self.slice(brace_start, self.pos));
                }
                Some(b'}') if self.peek_at(1) == Some(b'}') => {
                    let end = self.pos;
                    self.advance();
                    self.advance();
                    return Token::TemplateVariable(self.slice(content_start, end));
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn lex_word(&mut self) -> Token<'a> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.advance();
            } else {
                break;
            }
        }
        let word = self.slice(start, self.pos);

        match lookup_keyword(word) {
            Some(kind) => self.try_combine_keyword(kind),
            None => Token::Identifier(word),
        }
    }

    /// Peek ahead past optional whitespace for a word.
    /// Returns (word_str, word_end_pos) if found.
    fn peek_word_after_whitespace(&self, from: usize) -> Option<(&'a str, usize)> {
        let mut p = from;
        // Skip whitespace
        while p < self.bytes.len() && self.bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p >= self.bytes.len() {
            return None;
        }
        let b = self.bytes[p];
        if !(b.is_ascii_alphabetic() || b == b'_') {
            return None;
        }
        let word_start = p;
        while p < self.bytes.len()
            && (self.bytes[p].is_ascii_alphanumeric() || self.bytes[p] == b'_')
        {
            p += 1;
        }
        Some((self.slice(word_start, p), p))
    }

    fn try_combine_keyword(&mut self, kind: KeywordKind) -> Token<'a> {
        for &(ref first, expected, combined) in TWO_WORD_KEYWORDS {
            if kind == *first {
                return self.try_two_word(kind, expected, combined);
            }
        }
        match kind {
            KeywordKind::Full => self.try_keyword_combination(
                KeywordKind::Full,
                "JOIN",
                KeywordKind::FullJoin,
                "OUTER",
                "JOIN",
                KeywordKind::FullJoin,
            ),
            KeywordKind::If => self.try_keyword_combination(
                KeywordKind::If,
                "EXISTS",
                KeywordKind::IfExists,
                "NOT",
                "EXISTS",
                KeywordKind::IfNotExists,
            ),
            _ => Token::Keyword(kind),
        }
    }

    /// Try to combine a two-word keyword. If the next word (case-insensitive) matches
    /// `expected`, consume whitespace + word and return the combined variant.
    fn try_two_word(
        &mut self,
        standalone: KeywordKind,
        expected: &str,
        combined: KeywordKind,
    ) -> Token<'a> {
        if let Some((word, word_end)) = self.peek_word_after_whitespace(self.pos)
            && word.eq_ignore_ascii_case(expected)
        {
            self.pos = word_end;
            return Token::Keyword(combined);
        }
        Token::Keyword(standalone)
    }

    /// Generic three-word keyword combination helper.
    ///
    /// Handles patterns like:
    ///   FULL JOIN         -> direct_combined (FullJoin)
    ///   FULL OUTER JOIN   -> full_combined (FullJoin)
    ///   IF EXISTS         -> direct_combined (IfExists)
    ///   IF NOT EXISTS     -> full_combined (IfNotExists)
    fn try_keyword_combination(
        &mut self,
        standalone: KeywordKind,
        direct_word: &str,
        direct_combined: KeywordKind,
        middle_word: &str,
        final_word: &str,
        full_combined: KeywordKind,
    ) -> Token<'a> {
        if let Some((word, word_end)) = self.peek_word_after_whitespace(self.pos) {
            if word.eq_ignore_ascii_case(direct_word) {
                self.pos = word_end;
                return Token::Keyword(direct_combined);
            }
            if word.eq_ignore_ascii_case(middle_word)
                && let Some((word2, word_end2)) = self.peek_word_after_whitespace(word_end)
                && word2.eq_ignore_ascii_case(final_word)
            {
                self.pos = word_end2;
                return Token::Keyword(full_combined);
            }
        }
        Token::Keyword(standalone)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

pub fn tokenize(input: &str) -> Vec<Token<'_>> {
    Lexer::new(input).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_whitespace<'a>(tokens: &'a [Token<'a>]) -> Vec<&'a Token<'a>> {
        tokens
            .iter()
            .filter(|t| !matches!(t, Token::Whitespace(_)))
            .collect()
    }

    macro_rules! assert_tokens {
        ($input:expr, $($token:expr),+ $(,)?) => {
            assert_eq!(tokenize($input), vec![$($token),+]);
        };
    }

    #[test]
    fn test_basic_select() {
        let tokens = tokenize("SELECT mass, glow FROM comets");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::Identifier("mass"));
        assert_eq!(non_ws[2], &Token::Comma);
        assert_eq!(non_ws[3], &Token::Identifier("glow"));
        assert_eq!(non_ws[4], &Token::Keyword(KeywordKind::From));
        assert_eq!(non_ws[5], &Token::Identifier("comets"));
    }

    #[test]
    fn test_string_literal_simple() {
        assert_tokens!("'hello world'", Token::StringLiteral("hello world"));
    }

    #[test]
    fn test_string_literal_with_embedded_quotes() {
        assert_tokens!("'it''s a test'", Token::StringLiteral("it''s a test"));
    }

    #[test]
    fn test_quoted_identifier() {
        assert_tokens!("\"my column\"", Token::QuotedIdentifier("my column"));
    }

    #[test]
    fn test_number_integer() {
        assert_tokens!("42", Token::NumberLiteral("42"));
    }

    #[test]
    fn test_number_decimal() {
        assert_tokens!("3.14", Token::NumberLiteral("3.14"));
    }

    #[test]
    fn test_number_leading_dot() {
        assert_tokens!(".5", Token::NumberLiteral(".5"));
    }

    #[test]
    fn test_operators_multi_char() {
        let cases = vec![
            ("<>", Token::Operator("<>")),
            ("!=", Token::Operator("!=")),
            ("<=", Token::Operator("<=")),
            (">=", Token::Operator(">=")),
            ("||", Token::Operator("||")),
        ];
        for (input, expected) in cases {
            assert_tokens!(input, expected);
        }
    }

    #[test]
    fn test_operators_single_char() {
        let ops = "= < > + - * / % & | ^ ~ !";
        let tokens = tokenize(ops);
        let op_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::Operator(_)))
            .collect();
        assert_eq!(op_tokens.len(), 13);
        assert_eq!(op_tokens[0], &Token::Operator("="));
        assert_eq!(op_tokens[5], &Token::Operator("*"));
    }

    #[test]
    fn test_star_is_operator() {
        let tokens = tokenize("SELECT *");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[1], &Token::Operator("*"));
    }

    #[test]
    fn test_line_comment() {
        let tokens = tokenize("SELECT -- a comment\nfrequency");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::LineComment(" a comment"));
        assert_eq!(non_ws[2], &Token::Identifier("frequency"));
    }

    #[test]
    fn test_block_comment() {
        let tokens = tokenize("SELECT /* block */ altitude");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::BlockComment(" block "));
        assert_eq!(non_ws[2], &Token::Identifier("altitude"));
    }

    #[test]
    fn test_order_by() {
        let tokens = tokenize("ORDER BY velocity");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::OrderBy));
        assert_eq!(non_ws[1], &Token::Identifier("velocity"));
    }

    #[test]
    fn test_group_by() {
        let tokens = tokenize("GROUP BY flavor");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::GroupBy));
        assert_eq!(non_ws[1], &Token::Identifier("flavor"));
    }

    #[test]
    fn test_left_join() {
        let tokens = tokenize("LEFT JOIN potions");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::LeftJoin));
        assert_eq!(non_ws[1], &Token::Identifier("potions"));
    }

    #[test]
    fn test_right_join() {
        let tokens = tokenize("RIGHT JOIN t");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::RightJoin));
    }

    #[test]
    fn test_inner_join() {
        let tokens = tokenize("INNER JOIN t");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::InnerJoin));
    }

    #[test]
    fn test_cross_join() {
        let tokens = tokenize("CROSS JOIN t");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::CrossJoin));
    }

    #[test]
    fn test_full_join() {
        let tokens = tokenize("FULL JOIN t");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::FullJoin));
    }

    #[test]
    fn test_full_outer_join() {
        let tokens = tokenize("FULL OUTER JOIN t");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::FullJoin));
        assert_eq!(non_ws[1], &Token::Identifier("t"));
    }

    #[test]
    fn test_union_all() {
        assert_tokens!("UNION ALL", Token::Keyword(KeywordKind::UnionAll));
    }

    #[test]
    fn test_primary_key() {
        assert_tokens!("PRIMARY KEY", Token::Keyword(KeywordKind::PrimaryKey));
    }

    #[test]
    fn test_foreign_key() {
        assert_tokens!("FOREIGN KEY", Token::Keyword(KeywordKind::ForeignKey));
    }

    #[test]
    fn test_if_exists() {
        assert_tokens!("IF EXISTS", Token::Keyword(KeywordKind::IfExists));
    }

    #[test]
    fn test_if_not_exists() {
        assert_tokens!("IF NOT EXISTS", Token::Keyword(KeywordKind::IfNotExists));
    }

    #[test]
    fn test_rows_between() {
        assert_tokens!("ROWS BETWEEN", Token::Keyword(KeywordKind::RowsBetween));
    }

    #[test]
    fn test_range_between() {
        assert_tokens!("RANGE BETWEEN", Token::Keyword(KeywordKind::RangeBetween));
    }

    #[test]
    fn test_insert_into_stays_separate() {
        let tokens = tokenize("INSERT INTO users");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Insert));
        assert_eq!(non_ws[1], &Token::Keyword(KeywordKind::Into));
        assert_eq!(non_ws[2], &Token::Identifier("users"));
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_unclosed_string() {
        assert_tokens!("'unclosed", Token::StringLiteral("unclosed"));
    }

    #[test]
    fn test_unclosed_block_comment() {
        assert_tokens!("/* unclosed", Token::BlockComment(" unclosed"));
    }

    #[test]
    fn test_unclosed_quoted_identifier() {
        assert_tokens!("\"unclosed", Token::QuotedIdentifier("unclosed"));
    }

    #[test]
    fn test_mixed_tokens() {
        let tokens = tokenize(
            "SELECT u.id, COUNT(*) FROM users u WHERE u.age >= 18 GROUP BY u.id ORDER BY u.id ASC;",
        );
        let non_ws = strip_whitespace(&tokens);

        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::Identifier("u"));
        assert_eq!(non_ws[2], &Token::Dot);
        assert_eq!(non_ws[3], &Token::Identifier("id"));
        assert_eq!(non_ws[4], &Token::Comma);
        assert_eq!(non_ws[5], &Token::Identifier("COUNT"));
        assert_eq!(non_ws[6], &Token::OpenParen);
        assert_eq!(non_ws[7], &Token::Operator("*"));
        assert_eq!(non_ws[8], &Token::CloseParen);
        assert_eq!(non_ws[9], &Token::Keyword(KeywordKind::From));
        assert_eq!(non_ws[10], &Token::Identifier("users"));
        assert_eq!(non_ws[11], &Token::Identifier("u"));
        assert_eq!(non_ws[12], &Token::Keyword(KeywordKind::Where));
        assert_eq!(non_ws[13], &Token::Identifier("u"));
        assert_eq!(non_ws[14], &Token::Dot);
        assert_eq!(non_ws[15], &Token::Identifier("age"));
        assert_eq!(non_ws[16], &Token::Operator(">="));
        assert_eq!(non_ws[17], &Token::NumberLiteral("18"));
        assert_eq!(non_ws[18], &Token::Keyword(KeywordKind::GroupBy));
        assert_eq!(non_ws[19], &Token::Identifier("u"));
        assert_eq!(non_ws[20], &Token::Dot);
        assert_eq!(non_ws[21], &Token::Identifier("id"));
        assert_eq!(non_ws[22], &Token::Keyword(KeywordKind::OrderBy));
        assert_eq!(non_ws[23], &Token::Identifier("u"));
        assert_eq!(non_ws[24], &Token::Dot);
        assert_eq!(non_ws[25], &Token::Identifier("id"));
        assert_eq!(non_ws[26], &Token::Keyword(KeywordKind::Asc));
        assert_eq!(non_ws[27], &Token::Semicolon);
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let tokens = tokenize("select FROM Where");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::Keyword(KeywordKind::From));
        assert_eq!(non_ws[2], &Token::Keyword(KeywordKind::Where));
    }

    #[test]
    fn test_case_insensitive_multi_word() {
        assert_tokens!("order by", Token::Keyword(KeywordKind::OrderBy));
        assert_tokens!("left join", Token::Keyword(KeywordKind::LeftJoin));
    }

    #[test]
    fn test_standalone_keyword_no_combine() {
        // "ORDER" followed by non-"BY" keyword should stay standalone
        let tokens = tokenize("ORDER SELECT");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Order));
        assert_eq!(non_ws[1], &Token::Keyword(KeywordKind::Select));
    }

    #[test]
    fn test_whitespace_preserved() {
        let tokens = tokenize("SELECT  id");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Keyword(KeywordKind::Select));
        assert_eq!(tokens[1], Token::Whitespace("  "));
        assert_eq!(tokens[2], Token::Identifier("id"));
    }

    #[test]
    fn test_dot_number_vs_dot() {
        // ".5" is a number, standalone "." is a dot
        let tokens = tokenize("t.id + .5");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Identifier("t"));
        assert_eq!(non_ws[1], &Token::Dot);
        assert_eq!(non_ws[2], &Token::Identifier("id"));
        assert_eq!(non_ws[3], &Token::Operator("+"));
        assert_eq!(non_ws[4], &Token::NumberLiteral(".5"));
    }

    #[test]
    fn test_multi_word_with_extra_whitespace() {
        // Multi-word keyword combination should work with multiple spaces/tabs
        let tokens = tokenize("ORDER   BY id");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::OrderBy));
        assert_eq!(non_ws[1], &Token::Identifier("id"));
    }

    #[test]
    fn test_if_followed_by_non_exists() {
        // "IF" not followed by EXISTS or NOT EXISTS stays standalone
        let tokens = tokenize("IF something");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::If));
        assert_eq!(non_ws[1], &Token::Identifier("something"));
    }

    #[test]
    fn test_full_without_join() {
        // "FULL" not followed by JOIN or OUTER JOIN stays standalone
        let tokens = tokenize("FULL something");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Full));
        assert_eq!(non_ws[1], &Token::Identifier("something"));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let tokens = tokenize("user_name _temp __internal");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Identifier("user_name"));
        assert_eq!(non_ws[1], &Token::Identifier("_temp"));
        assert_eq!(non_ws[2], &Token::Identifier("__internal"));
    }

    #[test]
    fn test_line_comment_at_end_of_input() {
        let tokens = tokenize("SELECT -- end");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Keyword(KeywordKind::Select));
        assert_eq!(non_ws[1], &Token::LineComment(" end"));
    }

    #[test]
    fn test_consecutive_operators() {
        let tokens = tokenize("a+b-c");
        assert_eq!(tokens[0], Token::Identifier("a"));
        assert_eq!(tokens[1], Token::Operator("+"));
        assert_eq!(tokens[2], Token::Identifier("b"));
        assert_eq!(tokens[3], Token::Operator("-"));
        assert_eq!(tokens[4], Token::Identifier("c"));
    }

    #[test]
    fn test_lex_cast_operator() {
        let tokens = tokenize("col::int");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Identifier("col"));
        assert_eq!(non_ws[1], &Token::Operator("::"));
        assert_eq!(non_ws[2], &Token::Identifier("int"));
    }

    #[test]
    fn test_lex_json_arrow() {
        let tokens = tokenize("data->'key'");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Identifier("data"));
        assert_eq!(non_ws[1], &Token::Operator("->"));
        assert_eq!(non_ws[2], &Token::StringLiteral("key"));
    }

    #[test]
    fn test_lex_json_double_arrow() {
        let tokens = tokenize("data->>'key'");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Identifier("data"));
        assert_eq!(non_ws[1], &Token::Operator("->>"));
        assert_eq!(non_ws[2], &Token::StringLiteral("key"));
    }

    #[test]
    fn test_lex_template_variable() {
        assert_tokens!("{{tenant_id}}", Token::TemplateVariable("tenant_id"));
    }

    #[test]
    fn test_lex_template_variable_with_spaces() {
        assert_tokens!("{{ tenant_id }}", Token::TemplateVariable(" tenant_id "));
    }

    #[test]
    fn test_lex_unclosed_template() {
        let tokens = tokenize("{ not_template");
        let non_ws = strip_whitespace(&tokens);
        assert_eq!(non_ws[0], &Token::Operator("{"));
        assert_eq!(non_ws[1], &Token::Identifier("not_template"));
    }
}
