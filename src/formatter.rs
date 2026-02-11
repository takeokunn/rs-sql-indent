use crate::config::{FormatOptions, FormatStyle};
use crate::token::{KeywordKind, Token};

/// Tracks which clause context we are currently inside.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ClauseContext {
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

fn is_single_value_clause(kw: KeywordKind) -> bool {
    matches!(kw, KeywordKind::Limit | KeywordKind::Offset)
}

fn clause_context_from_keyword(kw: KeywordKind) -> ClauseContext {
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

fn needs_space_before(token: &Token<'_>, prev: Option<&Token<'_>>) -> bool {
    let Some(prev_token) = prev else {
        return false;
    };

    // No space before :: -> ->>
    if let Token::Operator(op) = token
        && (*op == "::" || *op == "->" || *op == "->>")
    {
        return false;
    }
    // No space after :: -> ->>
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

struct FormatterBase<'a> {
    tokens: &'a [Token<'a>],
    options: &'a FormatOptions,
    paren_depth: usize,
    is_subquery_paren: Vec<bool>,
    inline_paren_depth: usize,
    clause_context: ClauseContext,
    is_first_token: bool,
    prev_was_ddl_starter: bool,
    output: String,
}

impl<'a> FormatterBase<'a> {
    fn new(tokens: &'a [Token<'a>], options: &'a FormatOptions) -> Self {
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

    fn is_inline(&self) -> bool {
        self.inline_paren_depth > 0
    }

    fn keyword_str(&self, kw: KeywordKind) -> String {
        if self.options.uppercase {
            kw.as_str().to_string()
        } else {
            kw.as_str().to_lowercase()
        }
    }
}

trait SqlFormatter<'a> {
    fn base(&self) -> &FormatterBase<'a>;
    fn base_mut(&mut self) -> &mut FormatterBase<'a>;

    // Hook methods that each formatter implements differently
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

    // Hook for comment handling — StandardFormatter overrides to clear pending state
    fn on_comment(&mut self) {}

    // Hook for dot handling — StandardFormatter overrides to clear after_comma_newline
    fn on_dot(&mut self) {}

    // Hook for final output processing
    fn finalize_output(&mut self) -> String {
        let output = &mut self.base_mut().output;
        let trimmed = output.trim_end().len();
        output.truncate(trimmed);
        std::mem::take(output)
    }

    // The unified main loop — default implementation
    fn format(&mut self) -> String {
        // Copy the token slice reference to a local variable to avoid
        // holding an immutable borrow of `self` across the mutable calls below.
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

struct StandardFormatter<'a> {
    base: FormatterBase<'a>,
    indent_depth: usize,
    /// Set to true after a clause starter / order modifier / join keyword
    /// to indicate the next content token should go on a new indented line.
    needs_indent_newline: bool,
    /// For single-value clauses (LIMIT, OFFSET), the next value stays on the same line.
    needs_space_only: bool,
    /// True after a comma wrote a newline+indent, so the next token should NOT add extra space.
    after_comma_newline: bool,
}

impl<'a> StandardFormatter<'a> {
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
            self.base.output.push_str("  ");
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

    /// If currently inside inline parentheses, emit the keyword with appropriate
    /// spacing and return `true`. Otherwise return `false` so the caller can
    /// apply normal (non-inline) formatting.
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

    /// Emit a token that follows a clause starter (newline+indent or space).
    /// Returns true if it handled the output, false if the caller should use normal logic.
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

    /// Returns the paren depth at which DDL column definitions start.
    fn ddl_base_paren_depth(&self) -> usize {
        self.base
            .is_subquery_paren
            .iter()
            .filter(|&&is_sub| is_sub)
            .count()
    }
}

impl<'a> SqlFormatter<'a> for StandardFormatter<'a> {
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

struct RiverFormatter<'a> {
    base: FormatterBase<'a>,
    base_col: usize,
    base_stack: Vec<(usize, ClauseContext)>,
    between_depth: usize,
    in_cte_header: bool,
    after_leading_comma: bool,
}

impl<'a> RiverFormatter<'a> {
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

        // Inside inline parens, emit inline
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
                // CTE and FROM subqueries: close paren at outer base level
                self.write_padding(old_base);
            } else {
                // Other subqueries (e.g. WHERE IN): close paren at inner base level
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

impl<'a> SqlFormatter<'a> for RiverFormatter<'a> {
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

pub fn format_tokens(tokens: &[Token<'_>], options: &FormatOptions) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    match options.style {
        FormatStyle::Standard => {
            let mut formatter = StandardFormatter::new(tokens, options);
            formatter.format()
        }
        FormatStyle::River => {
            let mut formatter = RiverFormatter::new(tokens, options);
            formatter.format()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn fmt(sql: &str) -> String {
        let tokens = tokenize(sql);
        format_tokens(&tokens, &FormatOptions::default())
    }

    fn fmt_lower(sql: &str) -> String {
        let tokens = tokenize(sql);
        format_tokens(
            &tokens,
            &FormatOptions {
                uppercase: false,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_empty_tokens() {
        let result = format_tokens(&[], &FormatOptions::default());
        assert_eq!(result, "");
    }

    #[test]
    fn test_basic_select_columns() {
        let result = fmt("select velocity, color from rockets");
        assert_eq!(result, "SELECT\n  velocity,\n  color\nFROM\n  rockets");
    }

    #[test]
    fn test_select_with_where() {
        let result = fmt("select radius from potions where radius = 1");
        assert_eq!(
            result,
            "SELECT\n  radius\nFROM\n  potions\nWHERE\n  radius = 1"
        );
    }

    #[test]
    fn test_select_star() {
        let result = fmt("select * from galaxies");
        assert_eq!(result, "SELECT\n  *\nFROM\n  galaxies");
    }

    #[test]
    fn test_select_with_join_and_on() {
        let result = fmt(
            "select * from crystals c join nebulae n on c.id = n.crystal_id where n.luminosity > 100",
        );
        assert_eq!(
            result,
            "SELECT\n  *\nFROM\n  crystals c\nJOIN nebulae n\n  ON c.id = n.crystal_id\nWHERE\n  n.luminosity > 100"
        );
    }

    #[test]
    fn test_select_with_group_by_order_by_having() {
        let result = fmt(
            "select tier, count(*) from dragons group by tier having count(*) > 5 order by tier",
        );
        assert_eq!(
            result,
            "SELECT\n  tier,\n  count(*)\nFROM\n  dragons\nGROUP BY\n  tier\nHAVING\n  count(*) > 5\nORDER BY\n  tier"
        );
    }

    #[test]
    fn test_multi_query_with_semicolons() {
        let result = fmt("select 1; select 2");
        assert_eq!(result, "SELECT\n  1;\n\nSELECT\n  2");
    }

    #[test]
    fn test_subquery_in_where() {
        let result =
            fmt("select signal from beacons where signal in (select beacon_id from anchors)");
        assert_eq!(
            result,
            "SELECT\n  signal\nFROM\n  beacons\nWHERE\n  signal IN (\n  SELECT\n    beacon_id\n  FROM\n    anchors\n  )"
        );
    }

    #[test]
    fn test_create_table() {
        let result = fmt(
            "create table users (id int primary key, name varchar(255) not null, email varchar(255) unique)",
        );
        assert_eq!(
            result,
            "CREATE TABLE users (\n  id int PRIMARY KEY,\n  name varchar(255) NOT NULL,\n  email varchar(255) UNIQUE\n)"
        );
    }

    #[test]
    fn test_uppercase_mode() {
        let result = fmt("select id from users where active = true");
        assert_eq!(
            result,
            "SELECT\n  id\nFROM\n  users\nWHERE\n  active = TRUE"
        );
    }

    #[test]
    fn test_lowercase_mode() {
        let result = fmt_lower("SELECT id FROM users WHERE active = TRUE");
        assert_eq!(
            result,
            "select\n  id\nfrom\n  users\nwhere\n  active = true"
        );
    }

    #[test]
    fn test_line_comment_preservation() {
        let result = fmt("select -- pick columns\nid from users");
        assert_eq!(result, "SELECT -- pick columns id\nFROM\n  users");
    }

    #[test]
    fn test_block_comment_preservation() {
        let result = fmt("select /* all cols */ * from users");
        assert_eq!(result, "SELECT /* all cols */ *\nFROM\n  users");
    }

    #[test]
    fn test_and_or_in_where() {
        let result =
            fmt("select id from users where age > 18 and status = 'active' or role = 'admin'");
        assert_eq!(
            result,
            "SELECT\n  id\nFROM\n  users\nWHERE\n  age > 18\n  AND status = 'active'\n  OR role = 'admin'"
        );
    }

    #[test]
    fn test_function_call_inline() {
        let result = fmt("select count(*) from users");
        assert_eq!(result, "SELECT\n  count(*)\nFROM\n  users");
    }

    #[test]
    fn test_qualified_column_names() {
        let result = fmt("select u.id, u.name from users u");
        assert_eq!(result, "SELECT\n  u.id,\n  u.name\nFROM\n  users u");
    }

    #[test]
    fn test_limit_offset_same_line() {
        let result = fmt("select id from users limit 10 offset 5");
        assert_eq!(result, "SELECT\n  id\nFROM\n  users\nLIMIT 10\nOFFSET 5");
    }

    #[test]
    fn test_left_join() {
        let result = fmt("select * from a left join b on a.id = b.a_id");
        assert_eq!(
            result,
            "SELECT\n  *\nFROM\n  a\nLEFT JOIN b\n  ON a.id = b.a_id"
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
            "INSERT\nINTO\n  users(id, name)\nVALUES\n   (1, 'alice')"
        );
    }

    #[test]
    fn test_update_set() {
        let result = fmt("update users set name = 'bob', age = 30 where id = 1");
        assert_eq!(
            result,
            "UPDATE\n  users\nSET\n  name = 'bob',\n  age = 30\nWHERE\n  id = 1"
        );
    }

    #[test]
    fn test_delete_from() {
        let result = fmt("delete from users where id = 1");
        assert_eq!(result, "DELETE\nFROM\n  users\nWHERE\n  id = 1");
    }

    #[test]
    fn test_union() {
        let result = fmt("select 1 union select 2");
        assert_eq!(result, "SELECT\n  1\nUNION\nSELECT\n  2");
    }

    #[test]
    fn test_string_literal_preserved() {
        let result = fmt("select 'hello world' from dual");
        assert_eq!(result, "SELECT\n  'hello world'\nFROM\n  dual");
    }

    #[test]
    fn test_number_literal() {
        let result = fmt("select 42, 3.14 from dual");
        assert_eq!(result, "SELECT\n  42,\n  3.14\nFROM\n  dual");
    }

    #[test]
    fn test_quoted_identifier() {
        let result = fmt("select \"My Column\" from users");
        assert_eq!(result, "SELECT\n  \"My Column\"\nFROM\n  users");
    }

    #[test]
    fn test_multiple_joins() {
        let result = fmt("select * from a join b on a.id = b.a_id join c on b.id = c.b_id");
        assert_eq!(
            result,
            "SELECT\n  *\nFROM\n  a\nJOIN b\n  ON a.id = b.a_id\nJOIN c\n  ON b.id = c.b_id"
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
        assert_eq!(result, "SELECT\n  DISTINCT id\nFROM\n  users");
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
            "SELECT\n  id,\n  row_number() OVER (PARTITION by dept ORDER BY salary DESC) AS rn\nFROM\n  employees"
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
            "ALTER TABLE users ADD COLUMN email varchar (\n  255\n)"
        );
    }

    #[test]
    fn test_cast_operator_no_space() {
        let result = fmt("select col::int from t");
        assert_eq!(result, "SELECT\n  col::int\nFROM\n  t");
    }

    #[test]
    fn test_json_arrow_no_space() {
        let result = fmt("select data->>'key' from t");
        assert_eq!(result, "SELECT\n  data->>'key'\nFROM\n  t");
    }

    #[test]
    fn test_template_variable_preserved() {
        let result = fmt("select * from t where id = {{user_id}}");
        assert_eq!(result, "SELECT\n  *\nFROM\n  t\nWHERE\n  id = {{user_id}}");
    }

    // ── River style tests ───────────────────────────────────────────────

    fn fmt_river(sql: &str) -> String {
        let tokens = tokenize(sql);
        format_tokens(
            &tokens,
            &FormatOptions {
                style: FormatStyle::River,
                ..FormatOptions::default()
            },
        )
    }

    #[test]
    fn test_river_basic_select() {
        let result = fmt_river("select id, name from users");
        assert_eq!(result, "SELECT id\n       , name\n  FROM users");
    }

    #[test]
    fn test_river_select_star() {
        let result = fmt_river("select * from users");
        assert_eq!(result, "SELECT *\n  FROM users");
    }

    #[test]
    fn test_river_select_where_and() {
        let result = fmt_river("select id from users where id = 1 and status = 'active'");
        assert_eq!(
            result,
            "SELECT id\n  FROM users\n WHERE id = 1\n   AND status = 'active'"
        );
    }

    #[test]
    fn test_river_left_join() {
        let result = fmt_river("select * from a left join b on a.id = b.a_id and b.active = true");
        assert_eq!(
            result,
            "SELECT *\n  FROM a\n  LEFT JOIN b\n    ON a.id = b.a_id\n   AND b.active = TRUE"
        );
    }

    #[test]
    fn test_river_order_by() {
        let result = fmt_river("select id from users order by name desc");
        assert_eq!(result, "SELECT id\n  FROM users\n ORDER BY name DESC");
    }

    #[test]
    fn test_river_group_by() {
        let result =
            fmt_river("select dept, count(*) from employees group by dept having count(*) > 5");
        assert_eq!(
            result,
            "SELECT dept\n       , count(*)\n  FROM employees\n GROUP BY dept\nHAVING count(*) > 5"
        );
    }

    #[test]
    fn test_river_limit_offset() {
        let result = fmt_river("select id from users limit 10 offset 5");
        assert_eq!(result, "SELECT id\n  FROM users\n LIMIT 10\nOFFSET 5");
    }

    #[test]
    fn test_river_leading_comma() {
        let result = fmt_river("select a, b, c from t");
        assert_eq!(result, "SELECT a\n       , b\n       , c\n  FROM t");
    }

    #[test]
    fn test_river_between_and() {
        let result = fmt_river("select id from t where x between 1 and 10");
        assert_eq!(result, "SELECT id\n  FROM t\n WHERE x BETWEEN 1 AND 10");
    }

    #[test]
    fn test_river_function_inline() {
        let result = fmt_river("select count(*) from users");
        assert_eq!(result, "SELECT count(*)\n  FROM users");
    }

    #[test]
    fn test_river_subquery() {
        let result = fmt_river("select id from users where id in (select user_id from orders)");
        assert_eq!(
            result,
            "SELECT id\n  FROM users\n WHERE id IN (\n  SELECT user_id\n    FROM orders\n  )"
        );
    }

    #[test]
    fn test_river_union() {
        let result = fmt_river("select 1 union select 2");
        assert_eq!(result, "SELECT 1\n\n UNION\n\nSELECT 2");
    }

    #[test]
    fn test_river_cte_basic() {
        let result = fmt_river(
            "with active as (select id from users where active = true) select * from active",
        );
        assert_eq!(
            result,
            r#"WITH active AS (
  SELECT id
    FROM users
   WHERE active = TRUE
)
SELECT *
  FROM active"#
        );
    }

    #[test]
    fn test_river_cte_multiple() {
        let result = fmt_river("with a as (select 1), b as (select 2) select * from a, b");
        assert_eq!(
            result,
            r#"WITH a AS (
  SELECT 1
)
, b AS (
  SELECT 2
)
SELECT *
  FROM a, b"#
        );
    }

    #[test]
    fn test_river_inline_comma_single_space() {
        let result = fmt_river("select * from t where id in ('a', 'b', 'c')");
        assert_eq!(result, "SELECT *\n  FROM t\n WHERE id IN ('a', 'b', 'c')");
    }

    #[test]
    fn test_inline_comma_single_space() {
        let result = fmt("select * from t where id in ('a', 'b', 'c')");
        assert_eq!(
            result,
            "SELECT\n  *\nFROM\n  t\nWHERE\n  id IN ('a', 'b', 'c')"
        );
    }

    #[test]
    fn test_river_keyword_after_dot() {
        let result =
            fmt_river("select es.sequence, t.order from events es join tasks t on es.id = t.id");
        assert!(
            result.contains("es.sequence"),
            "should not uppercase 'sequence' after dot"
        );
        assert!(
            result.contains("t.order"),
            "should not uppercase 'order' after dot"
        );
    }

    #[test]
    fn test_keyword_after_dot_standard() {
        let result = fmt("select es.sequence from events es");
        assert!(
            result.contains("es.sequence"),
            "should not uppercase 'sequence' after dot in standard mode"
        );
    }

    // ── Phase 4: Golden test ─────────────────────────────────────────────

    #[test]
    fn test_river_golden_full_query() {
        let sql = include_str!("../tests/fixtures/river__golden_full_query.sql");
        let result = fmt_river(sql);
        let expected = include_str!("../tests/fixtures/river__golden_full_query.expected");
        assert_eq!(result, expected);
    }

    // ── Phase 4: Standard mode regression tests ─────────────────────────

    #[test]
    fn test_standard_select_regression() {
        let result = fmt("select id, name, email from users where active = true order by name");
        assert_eq!(
            result,
            "SELECT\n  id,\n  name,\n  email\nFROM\n  users\nWHERE\n  active = TRUE\nORDER BY\n  name"
        );
    }

    #[test]
    fn test_standard_join_regression() {
        let result = fmt(
            "select u.name, o.total from users u inner join orders o on u.id = o.user_id where o.total > 100",
        );
        assert_eq!(
            result,
            "SELECT\n  u.name,\n  o.total\nFROM\n  users u\nINNER JOIN orders o\n  ON u.id = o.user_id\nWHERE\n  o.total > 100"
        );
    }

    #[test]
    fn test_standard_cte_regression() {
        let result = fmt("with cte as (select 1 as n) select n from cte");
        assert_eq!(
            result,
            r#"WITH
  cte AS (
  SELECT
    1 AS n
  )
SELECT
  n
FROM
  cte"#
        );
    }

    #[test]
    fn test_standard_subquery_regression() {
        let result = fmt("select * from users where id in (select user_id from orders)");
        assert_eq!(
            result,
            r#"SELECT
  *
FROM
  users
WHERE
  id IN (
  SELECT
    user_id
  FROM
    orders
  )"#
        );
    }

    // ── Phase 4: Edge case tests ────────────────────────────────────────

    #[test]
    fn test_river_deeply_nested_subquery() {
        let result = fmt_river("select * from (select * from (select 1))");
        assert_eq!(
            result,
            r#"SELECT *
  FROM (
  SELECT *
    FROM (
    SELECT 1
  )
)"#
        );
    }

    #[test]
    fn test_river_empty_cte_body() {
        let result = fmt_river("with a as (select 1) select * from a");
        assert_eq!(
            result,
            r#"WITH a AS (
  SELECT 1
)
SELECT *
  FROM a"#
        );
    }

    #[test]
    fn test_river_nested_between() {
        let result = fmt_river("select * from t where a between 1 and 10 and b between 20 and 30");
        assert_eq!(
            result,
            "SELECT *\n  FROM t\n WHERE a BETWEEN 1 AND 10\n   AND b BETWEEN 20 AND 30"
        );
    }

    #[test]
    fn test_river_template_variable() {
        let result = fmt_river("select * from t where id = {{user_id}}");
        assert_eq!(result, "SELECT *\n  FROM t\n WHERE id = {{user_id}}");
    }

    #[test]
    fn test_river_cast_operator() {
        let result = fmt_river("select created_at::date from events");
        assert_eq!(result, "SELECT created_at::date\n  FROM events");
    }

    #[test]
    fn test_river_json_operators() {
        let result = fmt_river("select data -> 'key', data ->> 'value' from t");
        assert_eq!(
            result,
            "SELECT data->'key'\n       , data->>'value'\n  FROM t"
        );
    }

    #[test]
    fn test_river_leading_comma_before_paren() {
        // Regression test: leading comma before ( should not produce double space
        let result = fmt_river("select 1, (2 + 3) from t");
        assert!(
            !result.contains(",  "),
            "should not have double space after comma: {result}"
        );
    }

    #[test]
    fn test_river_union_all() {
        let result = fmt_river("select 1 union all select 2");
        assert_eq!(result, "SELECT 1\n\n UNION ALL\n\nSELECT 2");
    }

    #[test]
    fn test_river_order_by_multiple() {
        let result = fmt_river("select * from t order by a desc, b asc, c");
        assert_eq!(
            result,
            "SELECT *\n  FROM t\n ORDER BY a DESC\n       , b ASC\n       , c"
        );
    }
}
