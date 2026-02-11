use crate::config::FormatOptions;
use crate::token::{KeywordKind, Token};

use super::{
    ClauseContext, FormatterBase, SqlFormatter, clause_context_from_keyword,
    is_single_value_clause, needs_space_before,
};

struct DataopsFormatter<'a> {
    base: FormatterBase<'a>,
    indent_depth: usize,
    needs_indent_newline: bool,
    needs_space_only: bool,
    after_comma_newline: bool,
}

impl<'a> DataopsFormatter<'a> {
    fn new(tokens: &'a [Token<'a>], options: &'a FormatOptions) -> Self {
        Self {
            base: FormatterBase::new(tokens, options),
            indent_depth: 0,
            needs_indent_newline: false,
            needs_space_only: false,
            after_comma_newline: false,
        }
    }

    fn write_indent(&mut self, depth: usize) {
        for _ in 0..depth {
            self.base.output.push_str("    ");
        }
    }

    fn write_newline_at(&mut self, depth: usize) {
        self.base.output.push('\n');
        self.write_indent(depth);
    }

    fn base_indent(&self) -> usize {
        self.base
            .is_subquery_paren
            .iter()
            .filter(|&&is_sub| is_sub)
            .count()
    }

    fn clear_pending_state(&mut self) {
        self.needs_indent_newline = false;
        self.needs_space_only = false;
        self.after_comma_newline = false;
    }

    fn try_emit_inline(
        &mut self,
        kw: KeywordKind,
        kw_str: &str,
        prev_token: Option<&Token<'_>>,
    ) -> bool {
        if !self.base.is_inline() {
            return false;
        }
        if needs_space_before(&Token::Keyword(kw), prev_token) {
            self.base.output.push(' ');
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        true
    }

    fn emit_clause_content(&mut self, text: &str) -> bool {
        if self.needs_indent_newline {
            self.needs_indent_newline = false;
            self.write_newline_at(self.indent_depth);
            self.base.output.push_str(text);
            return true;
        }
        if self.needs_space_only {
            self.needs_space_only = false;
            self.base.output.push(' ');
            self.base.output.push_str(text);
            return true;
        }
        false
    }

    // ── Keyword dispatcher ──────────────────────────────────────────────

    fn do_format_keyword(&mut self, kw: KeywordKind, prev_token: Option<&Token<'_>>) {
        let kw_str = self.base.keyword_str(kw);

        if kw.is_ddl_starter() {
            self.format_ddl_keyword(kw, &kw_str);
        } else if kw.is_clause_starter() {
            self.format_clause_starter(kw, &kw_str, prev_token);
        } else if kw.is_join_keyword() {
            self.format_join_keyword(&kw_str, prev_token);
        } else if kw.is_order_modifier() {
            self.format_order_modifier(kw, &kw_str, prev_token);
        } else if kw == KeywordKind::On || kw == KeywordKind::And || kw == KeywordKind::Or {
            self.format_sub_clause_keyword(kw, &kw_str, prev_token);
        } else {
            self.format_other_keyword(kw, &kw_str, prev_token);
        }
    }

    // ── Keyword category handlers ───────────────────────────────────────

    fn format_ddl_keyword(&mut self, _kw: KeywordKind, kw_str: &str) {
        self.clear_pending_state();

        if !self.base.is_first_token {
            self.write_newline_at(self.base_indent());
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        self.base.prev_was_ddl_starter = true;
        self.base.clause_context = ClauseContext::Ddl;
        self.indent_depth = self.base_indent() + 1;
    }

    fn format_clause_starter(
        &mut self,
        kw: KeywordKind,
        kw_str: &str,
        prev_token: Option<&Token<'_>>,
    ) {
        if self.try_emit_inline(kw, kw_str, prev_token) {
            self.clear_pending_state();
            return;
        }

        self.clear_pending_state();

        let base = self.base_indent();

        if !self.base.is_first_token {
            self.write_newline_at(base);
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        self.base.prev_was_ddl_starter = false;
        self.base.clause_context = clause_context_from_keyword(kw);
        self.indent_depth = base + 1;

        if is_single_value_clause(kw) {
            self.needs_space_only = true;
        } else {
            self.needs_indent_newline = true;
        }
    }

    fn format_join_keyword(&mut self, kw_str: &str, prev_token: Option<&Token<'_>>) {
        if self.try_emit_inline(KeywordKind::Join, kw_str, prev_token) {
            return;
        }

        self.clear_pending_state();

        let base = self.base_indent();

        if !self.base.is_first_token {
            self.write_newline_at(base);
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        self.base.clause_context = ClauseContext::Join;
        self.indent_depth = base + 1;
        self.base.prev_was_ddl_starter = false;
        self.needs_space_only = true;
    }

    fn format_order_modifier(
        &mut self,
        kw: KeywordKind,
        kw_str: &str,
        prev_token: Option<&Token<'_>>,
    ) {
        if self.try_emit_inline(kw, kw_str, prev_token) {
            return;
        }

        self.clear_pending_state();

        let base = self.base_indent();

        if !self.base.is_first_token {
            self.write_newline_at(base);
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        self.indent_depth = base + 1;
        self.base.prev_was_ddl_starter = false;

        if kw == KeywordKind::GroupBy {
            self.base.clause_context = ClauseContext::GroupBy;
        } else if kw == KeywordKind::OrderBy {
            self.base.clause_context = ClauseContext::OrderBy;
        }

        self.needs_indent_newline = true;
    }

    fn format_sub_clause_keyword(
        &mut self,
        kw: KeywordKind,
        kw_str: &str,
        prev_token: Option<&Token<'_>>,
    ) {
        if self.try_emit_inline(kw, kw_str, prev_token) {
            return;
        }

        self.clear_pending_state();

        let base = self.base_indent();
        self.write_newline_at(base + 1);
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
        self.indent_depth = base + 1;
    }

    fn format_other_keyword(
        &mut self,
        kw: KeywordKind,
        kw_str: &str,
        prev_token: Option<&Token<'_>>,
    ) {
        if self.try_emit_inline(kw, kw_str, prev_token) {
            return;
        }

        if self.base.prev_was_ddl_starter {
            self.base.output.push(' ');
            self.base.output.push_str(kw_str);
            self.base.is_first_token = false;
            self.base.prev_was_ddl_starter = false;
            self.needs_indent_newline = false;
            self.needs_space_only = false;
            return;
        }

        if self.emit_clause_content(kw_str) {
            self.base.is_first_token = false;
            return;
        }

        if self.after_comma_newline {
            self.after_comma_newline = false;
            self.base.output.push_str(kw_str);
            self.base.is_first_token = false;
            return;
        }

        if needs_space_before(&Token::Keyword(kw), prev_token) {
            self.base.output.push(' ');
        }
        self.base.output.push_str(kw_str);
        self.base.is_first_token = false;
    }

    // ── Non-keyword formatters ──────────────────────────────────────────

    fn do_format_comma(&mut self) {
        self.clear_pending_state();

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
                self.write_newline_at(self.indent_depth);
                self.base.output.push_str(", ");
                self.base.is_first_token = false;
                self.after_comma_newline = true;
            }
            _ => {
                self.base.output.push(',');
                self.base.is_first_token = false;
            }
        }
    }

    fn do_format_open_paren(
        &mut self,
        filtered: &[&Token<'_>],
        idx: usize,
        prev_token: Option<&Token<'_>>,
    ) {
        let next = filtered.get(idx + 1).copied();
        let is_subquery = matches!(next, Some(Token::Keyword(kw)) if kw.is_clause_starter());

        if self.needs_indent_newline {
            self.needs_indent_newline = false;
            self.write_newline_at(self.indent_depth);
        }
        if self.needs_space_only {
            self.needs_space_only = false;
        }
        self.after_comma_newline = false;

        if is_subquery {
            self.base.paren_depth += 1;
            self.base.is_subquery_paren.push(true);
            self.indent_depth = self.base_indent();

            if needs_space_before(&Token::OpenParen, prev_token) {
                self.base.output.push(' ');
            }
            self.base.output.push('(');
            self.base.is_first_token = false;
        } else if self.base.clause_context == ClauseContext::Ddl
            && self.base.paren_depth == self.ddl_base_paren_depth()
        {
            self.base.paren_depth += 1;
            self.base.is_subquery_paren.push(false);

            if needs_space_before(&Token::OpenParen, prev_token) {
                self.base.output.push(' ');
            }
            self.base.output.push('(');
            self.write_newline_at(self.indent_depth);
            self.base.is_first_token = false;
        } else {
            self.base.paren_depth += 1;
            self.base.is_subquery_paren.push(false);
            self.base.inline_paren_depth += 1;

            match prev_token {
                Some(Token::Identifier(_)) => {}
                _ => {
                    if needs_space_before(&Token::OpenParen, prev_token) {
                        self.base.output.push(' ');
                    }
                }
            }
            self.base.output.push('(');
            self.base.is_first_token = false;
        }
    }

    fn do_format_close_paren(&mut self) {
        self.clear_pending_state();

        if self.base.paren_depth == 0 {
            self.base.output.push(')');
            self.base.is_first_token = false;
            return;
        }

        let subquery_base = self.base_indent();
        let was_subquery = self.base.is_subquery_paren.pop().unwrap_or(false);
        self.base.paren_depth -= 1;

        if was_subquery {
            let outer_base = self.base_indent();
            self.indent_depth = outer_base;
            self.write_newline_at(subquery_base);
            self.base.output.push(')');
        } else if self.base.inline_paren_depth > 0 {
            self.base.inline_paren_depth -= 1;
            self.base.output.push(')');
        } else {
            let base = self.base_indent();
            self.write_newline_at(base);
            self.base.output.push(')');
            self.indent_depth = base;
        }

        self.base.is_first_token = false;
    }

    fn do_format_semicolon(&mut self) {
        self.clear_pending_state();

        self.base.output.push(';');

        self.base.output.push('\n');
        self.base.output.push('\n');

        self.indent_depth = 0;
        self.base.clause_context = ClauseContext::None;
        self.base.prev_was_ddl_starter = false;
        self.base.is_first_token = true;
    }

    fn do_format_value(&mut self, text: &str, prev_token: Option<&Token<'_>>, token: &Token<'_>) {
        if self.base.is_inline() {
            self.clear_pending_state();
            if needs_space_before(token, prev_token) {
                self.base.output.push(' ');
            }
            self.base.output.push_str(text);
            self.base.is_first_token = false;
            return;
        }

        if self.base.prev_was_ddl_starter {
            self.base.output.push(' ');
            self.base.output.push_str(text);
            self.base.is_first_token = false;
            self.base.prev_was_ddl_starter = false;
            self.clear_pending_state();
            return;
        }

        if self.emit_clause_content(text) {
            self.base.is_first_token = false;
            self.after_comma_newline = false;
            return;
        }

        if self.after_comma_newline {
            self.after_comma_newline = false;
            self.base.output.push_str(text);
            self.base.is_first_token = false;
            return;
        }

        if needs_space_before(token, prev_token) {
            self.base.output.push(' ');
        }
        self.base.output.push_str(text);
        self.base.is_first_token = false;
    }

    fn ddl_base_paren_depth(&self) -> usize {
        self.base
            .is_subquery_paren
            .iter()
            .filter(|&&is_sub| is_sub)
            .count()
    }
}

impl<'a> SqlFormatter<'a> for DataopsFormatter<'a> {
    fn base(&self) -> &FormatterBase<'a> {
        &self.base
    }

    fn base_mut(&mut self) -> &mut FormatterBase<'a> {
        &mut self.base
    }

    fn on_comment(&mut self) {
        self.clear_pending_state();
    }

    fn on_dot(&mut self) {
        self.after_comma_newline = false;
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
    let mut formatter = DataopsFormatter::new(tokens, options);
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
                style: FormatStyle::Dataops,
                ..FormatOptions::default()
            },
        )
    }

    // ── Differentiating: leading comma ──

    #[test]
    fn test_basic_select() {
        let result = fmt("select velocity, color from rockets");
        assert_eq!(
            result,
            "SELECT\n    velocity\n    , color\nFROM\n    rockets"
        );
    }

    #[test]
    fn test_multiple_columns() {
        let result = fmt("select a, b, c from t");
        assert_eq!(result, "SELECT\n    a\n    , b\n    , c\nFROM\n    t");
    }

    #[test]
    fn test_create_table() {
        let result = fmt(
            "create table users (id int primary key, name varchar(255) not null, email varchar(255) unique)",
        );
        assert_eq!(
            result,
            "CREATE TABLE users (\n    id int PRIMARY KEY\n    , name varchar(255) NOT NULL\n    , email varchar(255) UNIQUE\n)"
        );
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
