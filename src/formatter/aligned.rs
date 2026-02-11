use crate::config::FormatOptions;
use crate::token::{KeywordKind, Token};

use super::{
    ClauseContext, FormatterBase, SqlFormatter, clause_context_from_keyword, needs_space_before,
};

struct AlignedFormatter<'a> {
    base: FormatterBase<'a>,
    base_col: usize,
    base_stack: Vec<(usize, ClauseContext)>,
    between_depth: usize,
    in_cte_header: bool,
    after_leading_comma: bool,
}

impl<'a> AlignedFormatter<'a> {
    fn new(tokens: &'a [Token<'a>], options: &'a FormatOptions) -> Self {
        Self {
            base: FormatterBase::new(tokens, options),
            base_col: 0,
            base_stack: Vec::new(),
            between_depth: 0,
            in_cte_header: false,
            after_leading_comma: false,
        }
    }

    fn keyword_padding(&self, kw: KeywordKind) -> usize {
        let kw_str = kw.as_str();
        let len = kw_str.len();
        if kw.is_join_keyword() {
            (self.base_col + 11).saturating_sub(len)
        } else if len > 6 {
            self.base_col + 1
        } else {
            (self.base_col + 6).saturating_sub(len)
        }
    }

    fn write_padding(&mut self, n: usize) {
        for _ in 0..n {
            self.base.output.push(' ');
        }
    }

    fn write_keyword_on_newline(&mut self, kw: KeywordKind) {
        let kw_str = self.base.keyword_str(kw);
        let padding = self.keyword_padding(kw);
        if !self.base.is_first_token {
            self.base.output.push('\n');
        }
        self.write_padding(padding);
        self.base.output.push_str(&kw_str);
        self.base.is_first_token = false;
    }

    fn write_leading_comma(&mut self) {
        self.base.output.push('\n');
        self.write_padding(self.base_col + 7);
        self.base.output.push_str(", ");
        self.after_leading_comma = true;
    }

    // ── Keyword dispatcher ──────────────────────────────────────────────

    fn do_format_keyword(&mut self, kw: KeywordKind, prev: Option<&Token<'_>>) {
        let kw_str = self.base.keyword_str(kw);

        if self.base.is_inline() {
            if needs_space_before(&Token::Keyword(kw), prev) {
                self.base.output.push(' ');
            }
            self.base.output.push_str(&kw_str);
            self.base.is_first_token = false;
            return;
        }

        if kw.is_ddl_starter() {
            self.format_ddl_keyword(kw);
        } else if kw == KeywordKind::With {
            self.format_with_keyword();
        } else if kw.is_clause_starter() {
            self.format_clause_starter(kw);
        } else if kw.is_join_keyword() {
            self.format_join_keyword(kw);
        } else if kw.is_order_modifier() {
            self.format_order_modifier(kw);
        } else if kw == KeywordKind::On || kw == KeywordKind::And || kw == KeywordKind::Or {
            self.format_sub_clause(kw, prev);
        } else {
            self.format_other_keyword(kw, &kw_str, prev);
        }
    }

    // ── Keyword category handlers ───────────────────────────────────────

    fn format_ddl_keyword(&mut self, kw: KeywordKind) {
        self.write_keyword_on_newline(kw);
        self.base.clause_context = ClauseContext::Ddl;
        self.base.prev_was_ddl_starter = true;
    }

    fn format_with_keyword(&mut self) {
        let kw_str = self.base.keyword_str(KeywordKind::With);
        if !self.base.is_first_token {
            self.base.output.push('\n');
        }
        self.base.output.push_str(&kw_str);
        self.base.is_first_token = false;
        self.base.clause_context = ClauseContext::Cte;
        self.in_cte_header = true;
    }

    fn format_clause_starter(&mut self, kw: KeywordKind) {
        if matches!(kw, KeywordKind::Union | KeywordKind::UnionAll) && !self.base.is_first_token {
            self.base.output.push('\n');
        }
        self.write_keyword_on_newline(kw);
        if matches!(kw, KeywordKind::Union | KeywordKind::UnionAll) {
            self.base.output.push('\n');
        }
        self.base.clause_context = clause_context_from_keyword(kw);
    }

    fn format_join_keyword(&mut self, kw: KeywordKind) {
        self.write_keyword_on_newline(kw);
        self.base.clause_context = ClauseContext::Join;
    }

    fn format_order_modifier(&mut self, kw: KeywordKind) {
        self.write_keyword_on_newline(kw);
        self.base.clause_context = match kw {
            KeywordKind::GroupBy => ClauseContext::GroupBy,
            KeywordKind::OrderBy => ClauseContext::OrderBy,
            _ => ClauseContext::Other,
        };
    }

    fn format_sub_clause(&mut self, kw: KeywordKind, prev: Option<&Token<'_>>) {
        if kw == KeywordKind::And && self.between_depth > 0 {
            self.between_depth -= 1;
            if needs_space_before(&Token::Keyword(kw), prev) {
                self.base.output.push(' ');
            }
            let kw_str = self.base.keyword_str(kw);
            self.base.output.push_str(&kw_str);
            self.base.is_first_token = false;
            return;
        }
        self.write_keyword_on_newline(kw);
    }

    fn format_other_keyword(&mut self, kw: KeywordKind, kw_str: &str, prev: Option<&Token<'_>>) {
        if kw == KeywordKind::Between {
            self.between_depth += 1;
        }
        if self.after_leading_comma {
            self.after_leading_comma = false;
            self.base.output.push_str(kw_str);
            self.base.is_first_token = false;
            return;
        }
        if needs_space_before(&Token::Keyword(kw), prev) {
            self.base.output.push(' ');
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
    }

    // ── Non-keyword formatters ──────────────────────────────────────────

    fn do_format_comma(&mut self) {
        if self.base.is_inline() {
            self.base.output.push(',');
            self.base.is_first_token = false;
            return;
        }

        match self.base.clause_context {
            ClauseContext::Select
            | ClauseContext::GroupBy
            | ClauseContext::OrderBy
            | ClauseContext::Set
            | ClauseContext::Ddl => {
                self.write_leading_comma();
            }
            ClauseContext::Cte => {
                self.base.output.push('\n');
                self.write_padding(self.base_col);
                self.base.output.push_str(", ");
                self.in_cte_header = true;
                self.after_leading_comma = true;
            }
            _ => {
                self.base.output.push(',');
            }
        }
        self.base.is_first_token = false;
    }

    fn do_format_open_paren(
        &mut self,
        filtered: &[&Token<'_>],
        idx: usize,
        prev: Option<&Token<'_>>,
    ) {
        let next = filtered.get(idx + 1).copied();
        let is_subquery = matches!(next, Some(Token::Keyword(kw)) if kw.is_clause_starter());

        if is_subquery {
            self.base.paren_depth += 1;
            self.base.is_subquery_paren.push(true);
            self.base_stack
                .push((self.base_col, self.base.clause_context));

            if self.in_cte_header {
                self.in_cte_header = false;
                self.base_col += 2;
            } else {
                self.base_col += 2;
            }

            if self.after_leading_comma {
                self.after_leading_comma = false;
            } else if needs_space_before(&Token::OpenParen, prev) {
                self.base.output.push(' ');
            }
            self.base.output.push('(');
            self.base.is_first_token = false;
        } else {
            self.base.paren_depth += 1;
            self.base.is_subquery_paren.push(false);
            self.base.inline_paren_depth += 1;
            if self.after_leading_comma {
                self.after_leading_comma = false;
            } else {
                match prev {
                    Some(Token::Identifier(_)) => {}
                    _ => {
                        if needs_space_before(&Token::OpenParen, prev) {
                            self.base.output.push(' ');
                        }
                    }
                }
            }
            self.base.output.push('(');
            self.base.is_first_token = false;
        }
    }

    fn do_format_close_paren(&mut self) {
        if self.base.paren_depth == 0 {
            self.base.output.push(')');
            self.base.is_first_token = false;
            return;
        }

        let was_subquery = self.base.is_subquery_paren.pop().unwrap_or(false);
        self.base.paren_depth -= 1;

        if was_subquery {
            let (old_base, old_context) = self.base_stack.pop().unwrap_or((0, ClauseContext::None));
            self.base.output.push('\n');
            if old_context == ClauseContext::Cte || old_context == ClauseContext::From {
                self.write_padding(old_base);
            } else {
                self.write_padding(old_base + 2);
            }
            self.base.output.push(')');
            self.base_col = old_base;
            self.base.clause_context = old_context;
        } else if self.base.inline_paren_depth > 0 {
            self.base.inline_paren_depth -= 1;
            self.base.output.push(')');
        } else {
            self.base.output.push(')');
        }

        self.base.is_first_token = false;
    }

    fn do_format_semicolon(&mut self) {
        self.base.output.push(';');
        self.base.output.push('\n');
        self.base.output.push('\n');
        self.base_col = 0;
        self.base.clause_context = ClauseContext::None;
        self.base.prev_was_ddl_starter = false;
        self.base.is_first_token = true;
    }

    fn do_format_value(&mut self, text: &str, prev: Option<&Token<'_>>, token: &Token<'_>) {
        if self.base.prev_was_ddl_starter {
            self.base.output.push(' ');
            self.base.output.push_str(text);
            self.base.prev_was_ddl_starter = false;
            self.base.is_first_token = false;
            return;
        }
        if self.after_leading_comma {
            self.after_leading_comma = false;
            self.base.output.push_str(text);
            self.base.is_first_token = false;
            return;
        }
        if needs_space_before(token, prev) {
            self.base.output.push(' ');
        }
        self.base.output.push_str(text);
        self.base.is_first_token = false;
    }
}

impl<'a> SqlFormatter<'a> for AlignedFormatter<'a> {
    fn base(&self) -> &FormatterBase<'a> {
        &self.base
    }

    fn base_mut(&mut self) -> &mut FormatterBase<'a> {
        &mut self.base
    }

    fn finalize_output(&mut self) -> String {
        let result: String = self.base.output.lines().collect::<Vec<_>>().join("\n");
        let trimmed = result.trim_end().to_string();
        self.base.output.clear();
        trimmed
    }

    fn format_keyword(&mut self, kw: KeywordKind, prev_token: Option<&Token<'a>>) {
        self.do_format_keyword(kw, prev_token);
    }

    fn format_comma(&mut self) {
        self.do_format_comma();
    }

    fn format_open_paren(
        &mut self,
        filtered: &[&Token<'a>],
        idx: usize,
        prev_token: Option<&Token<'a>>,
    ) {
        self.do_format_open_paren(filtered, idx, prev_token);
    }

    fn format_close_paren(&mut self) {
        self.do_format_close_paren();
    }

    fn format_semicolon(&mut self) {
        self.do_format_semicolon();
    }

    fn format_value(&mut self, text: &str, prev_token: Option<&Token<'a>>, token: &Token<'a>) {
        self.do_format_value(text, prev_token, token);
    }
}

pub(crate) fn format<'a>(tokens: &'a [Token<'a>], options: &'a FormatOptions) -> String {
    let mut formatter = AlignedFormatter::new(tokens, options);
    formatter.format()
}

#[cfg(test)]
mod tests {
    use crate::config::{FormatOptions, FormatStyle};
    use crate::formatter::format_tokens;
    use crate::lexer::tokenize;

    fn fmt(sql: &str) -> String {
        let tokens = tokenize(sql);
        format_tokens(
            &tokens,
            &FormatOptions {
                style: FormatStyle::Aligned,
                ..FormatOptions::default()
            },
        )
    }

    // ── Differentiating: right-aligned keywords + leading comma ──

    #[test]
    fn test_basic_select() {
        let result = fmt("select id, name from users");
        assert_eq!(result, "SELECT id\n       , name\n  FROM users");
    }

    #[test]
    fn test_select_where_and() {
        let result = fmt("select id from users where id = 1 and status = 'active'");
        assert_eq!(
            result,
            "SELECT id\n  FROM users\n WHERE id = 1\n   AND status = 'active'"
        );
    }

    #[test]
    fn test_left_join() {
        let result = fmt("select * from a left join b on a.id = b.a_id and b.active = true");
        assert_eq!(
            result,
            "SELECT *\n  FROM a\n  LEFT JOIN b\n    ON a.id = b.a_id\n   AND b.active = TRUE"
        );
    }

    #[test]
    fn test_leading_comma() {
        let result = fmt("select a, b, c from t");
        assert_eq!(result, "SELECT a\n       , b\n       , c\n  FROM t");
    }

    // ── Quality Assurance ──

    #[test]
    fn test_deterministic() {
        let sql = "select id, name from users where id = 1 order by name";
        let r1 = fmt(sql);
        let r2 = fmt(sql);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_no_trailing_whitespace() {
        let result = fmt("select id, name from users");
        for line in result.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "trailing whitespace found: {:?}",
                line
            );
        }
    }

    #[test]
    fn test_no_trailing_newline() {
        let result = fmt("select 1");
        assert!(!result.ends_with('\n'));
    }

}
