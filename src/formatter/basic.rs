use crate::config::FormatOptions;
use crate::token::{KeywordKind, Token};

use super::{
    ClauseContext, FormatterBase, SqlFormatter, clause_context_from_keyword,
    is_single_value_clause, needs_space_before,
};

struct BasicFormatter<'a> {
    base: FormatterBase<'a>,
    indent_depth: usize,
    needs_indent_newline: bool,
    needs_space_only: bool,
    after_comma_newline: bool,
}

impl<'a> BasicFormatter<'a> {
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
                self.base.output.push(',');
                self.write_newline_at(self.indent_depth);
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

impl<'a> SqlFormatter<'a> for BasicFormatter<'a> {
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
    let mut formatter = BasicFormatter::new(tokens, options);
    formatter.format()
}

#[cfg(test)]
mod tests {
    use crate::config::FormatOptions;
    use crate::formatter::format_tokens;
    use crate::lexer::tokenize;

    fn fmt(sql: &str) -> String {
        let tokens = tokenize(sql);
        format_tokens(&tokens, &FormatOptions::default())
    }

    #[test]
    fn test_select_columns() {
        let result = fmt("select velocity, color from rockets");
        assert_eq!(
            result,
            "SELECT\n    velocity,\n    color\nFROM\n    rockets"
        );
    }

    #[test]
    fn test_select_with_where() {
        let result = fmt("select radius from potions where radius = 1");
        assert_eq!(
            result,
            "SELECT\n    radius\nFROM\n    potions\nWHERE\n    radius = 1"
        );
    }

    #[test]
    fn test_select_star() {
        let result = fmt("select * from galaxies");
        assert_eq!(result, "SELECT\n    *\nFROM\n    galaxies");
    }

    #[test]
    fn test_select_with_join_and_on() {
        let result = fmt(
            "select * from crystals c join nebulae n on c.id = n.crystal_id where n.luminosity > 100",
        );
        assert_eq!(
            result,
            "SELECT\n    *\nFROM\n    crystals c\nJOIN nebulae n\n    ON c.id = n.crystal_id\nWHERE\n    n.luminosity > 100"
        );
    }

    #[test]
    fn test_select_with_group_by_order_by_having() {
        let result = fmt(
            "select tier, count(*) from dragons group by tier having count(*) > 5 order by tier",
        );
        assert_eq!(
            result,
            "SELECT\n    tier,\n    count(*)\nFROM\n    dragons\nGROUP BY\n    tier\nHAVING\n    count(*) > 5\nORDER BY\n    tier"
        );
    }

    #[test]
    fn test_multi_query_with_semicolons() {
        let result = fmt("select 1; select 2");
        assert_eq!(result, "SELECT\n    1;\n\nSELECT\n    2");
    }

    #[test]
    fn test_subquery_in_where() {
        let result =
            fmt("select signal from beacons where signal in (select beacon_id from anchors)");
        assert_eq!(
            result,
            "SELECT\n    signal\nFROM\n    beacons\nWHERE\n    signal IN (\n    SELECT\n        beacon_id\n    FROM\n        anchors\n    )"
        );
    }

    #[test]
    fn test_create_table() {
        let result = fmt(
            "create table users (id int primary key, name varchar(255) not null, email varchar(255) unique)",
        );
        assert_eq!(
            result,
            "CREATE TABLE users (\n    id int PRIMARY KEY,\n    name varchar(255) NOT NULL,\n    email varchar(255) UNIQUE\n)"
        );
    }

    #[test]
    fn test_line_comment_preservation() {
        let result = fmt("select -- pick columns\nid from users");
        assert_eq!(result, "SELECT -- pick columns id\nFROM\n    users");
    }

    #[test]
    fn test_block_comment_preservation() {
        let result = fmt("select /* all cols */ * from users");
        assert_eq!(result, "SELECT /* all cols */ *\nFROM\n    users");
    }

    #[test]
    fn test_and_or_in_where() {
        let result =
            fmt("select id from users where age > 18 and status = 'active' or role = 'admin'");
        assert_eq!(
            result,
            "SELECT\n    id\nFROM\n    users\nWHERE\n    age > 18\n    AND status = 'active'\n    OR role = 'admin'"
        );
    }

    #[test]
    fn test_function_call_inline() {
        let result = fmt("select count(*) from users");
        assert_eq!(result, "SELECT\n    count(*)\nFROM\n    users");
    }

    #[test]
    fn test_qualified_column_names() {
        let result = fmt("select u.id, u.name from users u");
        assert_eq!(result, "SELECT\n    u.id,\n    u.name\nFROM\n    users u");
    }

    #[test]
    fn test_limit_offset_same_line() {
        let result = fmt("select id from users limit 10 offset 5");
        assert_eq!(
            result,
            "SELECT\n    id\nFROM\n    users\nLIMIT 10\nOFFSET 5"
        );
    }

    #[test]
    fn test_left_join() {
        let result = fmt("select * from a left join b on a.id = b.a_id");
        assert_eq!(
            result,
            "SELECT\n    *\nFROM\n    a\nLEFT JOIN b\n    ON a.id = b.a_id"
        );
    }

    #[test]
    fn test_deterministic() {
        let sql = "select id, name from users where id = 1 order by name";
        let r1 = fmt(sql);
        let r2 = fmt(sql);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_no_trailing_whitespace_on_lines() {
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

    #[test]
    fn test_insert_into() {
        let result = fmt("insert into users (id, name) values (1, 'alice')");
        assert_eq!(
            result,
            "INSERT\nINTO\n    users(id, name)\nVALUES\n     (1, 'alice')"
        );
    }

    #[test]
    fn test_update_set() {
        let result = fmt("update users set name = 'bob', age = 30 where id = 1");
        assert_eq!(
            result,
            "UPDATE\n    users\nSET\n    name = 'bob',\n    age = 30\nWHERE\n    id = 1"
        );
    }

    #[test]
    fn test_delete_from() {
        let result = fmt("delete from users where id = 1");
        assert_eq!(result, "DELETE\nFROM\n    users\nWHERE\n    id = 1");
    }

    #[test]
    fn test_union() {
        let result = fmt("select 1 union select 2");
        assert_eq!(result, "SELECT\n    1\nUNION\nSELECT\n    2");
    }

    #[test]
    fn test_string_literal_preserved() {
        let result = fmt("select 'hello world' from dual");
        assert_eq!(result, "SELECT\n    'hello world'\nFROM\n    dual");
    }

    #[test]
    fn test_number_literal() {
        let result = fmt("select 42, 3.14 from dual");
        assert_eq!(result, "SELECT\n    42,\n    3.14\nFROM\n    dual");
    }

    #[test]
    fn test_quoted_identifier() {
        let result = fmt("select \"My Column\" from users");
        assert_eq!(result, "SELECT\n    \"My Column\"\nFROM\n    users");
    }

    #[test]
    fn test_multiple_joins() {
        let result = fmt("select * from a join b on a.id = b.a_id join c on b.id = c.b_id");
        assert_eq!(
            result,
            "SELECT\n    *\nFROM\n    a\nJOIN b\n    ON a.id = b.a_id\nJOIN c\n    ON b.id = c.b_id"
        );
    }

    #[test]
    fn test_drop_table() {
        let result = fmt("drop table users");
        assert_eq!(result, "DROP TABLE users");
    }

    #[test]
    fn test_select_distinct() {
        let result = fmt("select distinct id from users");
        assert_eq!(result, "SELECT\n    DISTINCT id\nFROM\n    users");
    }

    #[test]
    fn test_case_expression() {
        let result = fmt(
            "select case when status = 1 then 'active' when status = 2 then 'inactive' else 'unknown' end as label from users",
        );
        assert_eq!(
            result,
            r#"SELECT
    CASE WHEN status = 1 THEN 'active' WHEN status = 2 THEN 'inactive' ELSE 'unknown' END AS label
FROM
    users"#
        );
    }

    #[test]
    fn test_cte_with_as() {
        let result = fmt(
            "with active_users as (select id from users where active = true) select * from active_users",
        );
        assert_eq!(
            result,
            r#"WITH
    active_users AS (
    SELECT
        id
    FROM
        users
    WHERE
        active = TRUE
    )
SELECT
    *
FROM
    active_users"#
        );
    }

    #[test]
    fn test_window_function() {
        let result = fmt(
            "select id, row_number() over (partition by dept order by salary desc) as rn from employees",
        );
        assert_eq!(
            result,
            "SELECT\n    id,\n    row_number() OVER (PARTITION by dept ORDER BY salary DESC) AS rn\nFROM\n    employees"
        );
    }

    #[test]
    fn test_nested_subquery() {
        let result = fmt("select * from (select id from (select id from users) t1) t2");
        assert_eq!(
            result,
            r#"SELECT
    *
FROM
     (
    SELECT
        id
    FROM
         (
        SELECT
            id
        FROM
            users
        ) t1
    ) t2"#
        );
    }

    #[test]
    fn test_empty_statement() {
        let result = fmt("select 1;;select 2");
        assert_eq!(result.matches("SELECT").count(), 2);
    }

    #[test]
    fn test_alter_table() {
        let result = fmt("alter table users add column email varchar(255)");
        assert_eq!(
            result,
            "ALTER TABLE users ADD COLUMN email varchar (\n    255\n)"
        );
    }

    #[test]
    fn test_cast_operator_no_space() {
        let result = fmt("select col::int from t");
        assert_eq!(result, "SELECT\n    col::int\nFROM\n    t");
    }

    #[test]
    fn test_json_arrow_no_space() {
        let result = fmt("select data->>'key' from t");
        assert_eq!(result, "SELECT\n    data->>'key'\nFROM\n    t");
    }

    #[test]
    fn test_template_variable_preserved() {
        let result = fmt("select * from t where id = {{user_id}}");
        assert_eq!(
            result,
            "SELECT\n    *\nFROM\n    t\nWHERE\n    id = {{user_id}}"
        );
    }

    #[test]
    fn test_keyword_after_dot() {
        let result = fmt("select es.sequence from events es");
        assert!(
            result.contains("es.sequence"),
            "should not uppercase 'sequence' after dot in basic mode"
        );
    }

    #[test]
    fn test_inline_comma_single_space() {
        let result = fmt("select * from t where id in ('a', 'b', 'c')");
        assert_eq!(
            result,
            "SELECT\n    *\nFROM\n    t\nWHERE\n    id IN ('a', 'b', 'c')"
        );
    }
}
