macro_rules! define_keywords {
    (
        single {
            $( $single_variant:ident => $single_str:expr ),* $(,)?
        }
        multi {
            $( $multi_variant:ident => $multi_str:expr ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KeywordKind {
            $( $single_variant, )*
            $( $multi_variant, )*
        }

        impl KeywordKind {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( KeywordKind::$single_variant => $single_str, )*
                    $( KeywordKind::$multi_variant => $multi_str, )*
                }
            }
        }

        impl std::fmt::Display for KeywordKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for KeywordKind {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // Case-insensitive matching for single-word keywords only
                $( if s.eq_ignore_ascii_case($single_str) { return Ok(KeywordKind::$single_variant); } )*
                Err(())
            }
        }

        /// Lookup a single-word keyword (case-insensitive).
        /// Multi-word keywords (ORDER BY, LEFT JOIN, etc.) are NOT returned here.
        pub fn lookup_keyword(word: &str) -> Option<KeywordKind> {
            word.parse().ok()
        }
    };
}

define_keywords! {
    single {
        // DML keywords
        Select => "SELECT",
        From => "FROM",
        Where => "WHERE",
        And => "AND",
        Or => "OR",
        Not => "NOT",
        In => "IN",
        Between => "BETWEEN",
        Like => "LIKE",
        Is => "IS",
        Null => "NULL",
        As => "AS",
        On => "ON",
        Join => "JOIN",
        Having => "HAVING",
        Limit => "LIMIT",
        Offset => "OFFSET",
        Union => "UNION",
        Intersect => "INTERSECT",
        Except => "EXCEPT",
        Insert => "INSERT",
        Into => "INTO",
        Values => "VALUES",
        Update => "UPDATE",
        Set => "SET",
        Delete => "DELETE",
        Distinct => "DISTINCT",
        All => "ALL",
        Asc => "ASC",
        Desc => "DESC",
        Case => "CASE",
        When => "WHEN",
        Then => "THEN",
        Else => "ELSE",
        End => "END",
        Exists => "EXISTS",
        Any => "ANY",
        With => "WITH",
        Recursive => "RECURSIVE",
        Returning => "RETURNING",
        Using => "USING",
        Natural => "NATURAL",
        Fetch => "FETCH",
        For => "FOR",
        Window => "WINDOW",
        Over => "OVER",
        Partition => "PARTITION",
        Rows => "ROWS",
        Range => "RANGE",
        Unbounded => "UNBOUNDED",
        Preceding => "PRECEDING",
        Following => "FOLLOWING",
        Current => "CURRENT",
        Row => "ROW",

        // Standalone single-word variants (for lexer lookahead)
        Order => "ORDER",
        Group => "GROUP",
        Left => "LEFT",
        Right => "RIGHT",
        Inner => "INNER",
        Outer => "OUTER",
        Full => "FULL",
        Cross => "CROSS",

        // DDL keywords
        Create => "CREATE",
        Alter => "ALTER",
        Drop => "DROP",
        Table => "TABLE",
        Index => "INDEX",
        View => "VIEW",
        Column => "COLUMN",
        Add => "ADD",
        Primary => "PRIMARY",
        Key => "KEY",
        Foreign => "FOREIGN",
        References => "REFERENCES",
        Unique => "UNIQUE",
        Default => "DEFAULT",
        Check => "CHECK",
        Constraint => "CONSTRAINT",
        Cascade => "CASCADE",
        Restrict => "RESTRICT",
        If => "IF",
        Temporary => "TEMPORARY",
        Temp => "TEMP",
        Schema => "SCHEMA",
        Database => "DATABASE",
        Sequence => "SEQUENCE",
        Trigger => "TRIGGER",
        Function => "FUNCTION",
        Procedure => "PROCEDURE",
        Type => "TYPE",
        Enum => "ENUM",
        Grant => "GRANT",
        Revoke => "REVOKE",
        Truncate => "TRUNCATE",
        Rename => "RENAME",
        Replace => "REPLACE",
        Comment => "COMMENT",

        // Other
        True => "TRUE",
        False => "FALSE",
        Begin => "BEGIN",
        Commit => "COMMIT",
        Rollback => "ROLLBACK",
        Savepoint => "SAVEPOINT",
        Transaction => "TRANSACTION",
        Lock => "LOCK",
        Unlock => "UNLOCK",
    }
    multi {
        OrderBy => "ORDER BY",
        GroupBy => "GROUP BY",
        LeftJoin => "LEFT JOIN",
        RightJoin => "RIGHT JOIN",
        InnerJoin => "INNER JOIN",
        OuterJoin => "OUTER JOIN",
        FullJoin => "FULL JOIN",
        CrossJoin => "CROSS JOIN",
        UnionAll => "UNION ALL",
        PrimaryKey => "PRIMARY KEY",
        ForeignKey => "FOREIGN KEY",
        IfExists => "IF EXISTS",
        IfNotExists => "IF NOT EXISTS",
        RowsBetween => "ROWS BETWEEN",
        RangeBetween => "RANGE BETWEEN",
    }
}

impl KeywordKind {
    pub fn is_clause_starter(&self) -> bool {
        matches!(
            self,
            KeywordKind::Select
                | KeywordKind::From
                | KeywordKind::Where
                | KeywordKind::Set
                | KeywordKind::Values
                | KeywordKind::Into
                | KeywordKind::Having
                | KeywordKind::Limit
                | KeywordKind::Offset
                | KeywordKind::Union
                | KeywordKind::UnionAll
                | KeywordKind::Intersect
                | KeywordKind::Except
                | KeywordKind::Returning
                | KeywordKind::Insert
                | KeywordKind::Update
                | KeywordKind::Delete
                | KeywordKind::With
                | KeywordKind::Fetch
        )
    }

    pub fn is_join_keyword(&self) -> bool {
        matches!(
            self,
            KeywordKind::Join
                | KeywordKind::LeftJoin
                | KeywordKind::RightJoin
                | KeywordKind::InnerJoin
                | KeywordKind::OuterJoin
                | KeywordKind::FullJoin
                | KeywordKind::CrossJoin
                | KeywordKind::Natural
        )
    }

    pub fn is_sub_clause(&self) -> bool {
        matches!(self, KeywordKind::On | KeywordKind::And | KeywordKind::Or)
    }

    pub fn is_order_modifier(&self) -> bool {
        matches!(self, KeywordKind::OrderBy | KeywordKind::GroupBy)
    }

    pub fn is_ddl_starter(&self) -> bool {
        matches!(
            self,
            KeywordKind::Create
                | KeywordKind::Alter
                | KeywordKind::Drop
                | KeywordKind::Truncate
                | KeywordKind::Grant
                | KeywordKind::Revoke
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    Keyword(KeywordKind),
    Identifier(&'a str),
    QuotedIdentifier(&'a str),
    StringLiteral(&'a str),
    NumberLiteral(&'a str),
    Operator(&'a str),
    Comma,
    Semicolon,
    Dot,
    OpenParen,
    CloseParen,
    LineComment(&'a str),
    BlockComment(&'a str),
    Whitespace(&'a str),
    TemplateVariable(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_keyword_uppercase() {
        assert_eq!(lookup_keyword("SELECT"), Some(KeywordKind::Select));
        assert_eq!(lookup_keyword("FROM"), Some(KeywordKind::From));
        assert_eq!(lookup_keyword("WHERE"), Some(KeywordKind::Where));
        assert_eq!(lookup_keyword("CREATE"), Some(KeywordKind::Create));
    }

    #[test]
    fn test_lookup_keyword_lowercase() {
        assert_eq!(lookup_keyword("select"), Some(KeywordKind::Select));
        assert_eq!(lookup_keyword("from"), Some(KeywordKind::From));
        assert_eq!(lookup_keyword("where"), Some(KeywordKind::Where));
        assert_eq!(lookup_keyword("insert"), Some(KeywordKind::Insert));
    }

    #[test]
    fn test_lookup_keyword_mixed_case() {
        assert_eq!(lookup_keyword("Select"), Some(KeywordKind::Select));
        assert_eq!(lookup_keyword("fRoM"), Some(KeywordKind::From));
        assert_eq!(lookup_keyword("WhErE"), Some(KeywordKind::Where));
        assert_eq!(lookup_keyword("DeLeTe"), Some(KeywordKind::Delete));
    }

    #[test]
    fn test_lookup_keyword_not_found() {
        assert_eq!(lookup_keyword("foobar"), None);
        assert_eq!(lookup_keyword(""), None);
        assert_eq!(lookup_keyword("SELECTX"), None);
    }

    #[test]
    fn test_lookup_keyword_standalone_variants() {
        assert_eq!(lookup_keyword("order"), Some(KeywordKind::Order));
        assert_eq!(lookup_keyword("group"), Some(KeywordKind::Group));
        assert_eq!(lookup_keyword("left"), Some(KeywordKind::Left));
        assert_eq!(lookup_keyword("right"), Some(KeywordKind::Right));
        assert_eq!(lookup_keyword("inner"), Some(KeywordKind::Inner));
        assert_eq!(lookup_keyword("outer"), Some(KeywordKind::Outer));
        assert_eq!(lookup_keyword("full"), Some(KeywordKind::Full));
        assert_eq!(lookup_keyword("cross"), Some(KeywordKind::Cross));
        assert_eq!(lookup_keyword("primary"), Some(KeywordKind::Primary));
        assert_eq!(lookup_keyword("foreign"), Some(KeywordKind::Foreign));
        assert_eq!(lookup_keyword("if"), Some(KeywordKind::If));
    }

    #[test]
    fn test_multi_word_not_in_lookup() {
        assert_eq!(lookup_keyword("ORDER BY"), None);
        assert_eq!(lookup_keyword("GROUP BY"), None);
        assert_eq!(lookup_keyword("LEFT JOIN"), None);
        assert_eq!(lookup_keyword("RIGHT JOIN"), None);
        assert_eq!(lookup_keyword("INNER JOIN"), None);
        assert_eq!(lookup_keyword("OUTER JOIN"), None);
        assert_eq!(lookup_keyword("FULL JOIN"), None);
        assert_eq!(lookup_keyword("CROSS JOIN"), None);
        assert_eq!(lookup_keyword("UNION ALL"), None);
        assert_eq!(lookup_keyword("PRIMARY KEY"), None);
        assert_eq!(lookup_keyword("FOREIGN KEY"), None);
        assert_eq!(lookup_keyword("IF EXISTS"), None);
        assert_eq!(lookup_keyword("IF NOT EXISTS"), None);
        assert_eq!(lookup_keyword("ROWS BETWEEN"), None);
        assert_eq!(lookup_keyword("RANGE BETWEEN"), None);
    }

    #[test]
    fn test_is_clause_starter() {
        assert!(KeywordKind::Select.is_clause_starter());
        assert!(KeywordKind::From.is_clause_starter());
        assert!(KeywordKind::Where.is_clause_starter());
        assert!(KeywordKind::Set.is_clause_starter());
        assert!(KeywordKind::Values.is_clause_starter());
        assert!(KeywordKind::Into.is_clause_starter());
        assert!(KeywordKind::Having.is_clause_starter());
        assert!(KeywordKind::Limit.is_clause_starter());
        assert!(KeywordKind::Offset.is_clause_starter());
        assert!(KeywordKind::Union.is_clause_starter());
        assert!(KeywordKind::UnionAll.is_clause_starter());
        assert!(KeywordKind::Intersect.is_clause_starter());
        assert!(KeywordKind::Except.is_clause_starter());
        assert!(KeywordKind::Returning.is_clause_starter());
        assert!(KeywordKind::Insert.is_clause_starter());
        assert!(KeywordKind::Update.is_clause_starter());
        assert!(KeywordKind::Delete.is_clause_starter());
        assert!(KeywordKind::With.is_clause_starter());
        assert!(KeywordKind::Fetch.is_clause_starter());

        assert!(!KeywordKind::And.is_clause_starter());
        assert!(!KeywordKind::Or.is_clause_starter());
        assert!(!KeywordKind::Join.is_clause_starter());
        assert!(!KeywordKind::On.is_clause_starter());
        assert!(!KeywordKind::Table.is_clause_starter());
    }

    #[test]
    fn test_is_join_keyword() {
        assert!(KeywordKind::Join.is_join_keyword());
        assert!(KeywordKind::LeftJoin.is_join_keyword());
        assert!(KeywordKind::RightJoin.is_join_keyword());
        assert!(KeywordKind::InnerJoin.is_join_keyword());
        assert!(KeywordKind::OuterJoin.is_join_keyword());
        assert!(KeywordKind::FullJoin.is_join_keyword());
        assert!(KeywordKind::CrossJoin.is_join_keyword());
        assert!(KeywordKind::Natural.is_join_keyword());

        assert!(!KeywordKind::Select.is_join_keyword());
        assert!(!KeywordKind::On.is_join_keyword());
        assert!(!KeywordKind::Left.is_join_keyword());
    }

    #[test]
    fn test_is_sub_clause() {
        assert!(KeywordKind::On.is_sub_clause());
        assert!(KeywordKind::And.is_sub_clause());
        assert!(KeywordKind::Or.is_sub_clause());

        assert!(!KeywordKind::Select.is_sub_clause());
        assert!(!KeywordKind::Where.is_sub_clause());
    }

    #[test]
    fn test_is_order_modifier() {
        assert!(KeywordKind::OrderBy.is_order_modifier());
        assert!(KeywordKind::GroupBy.is_order_modifier());

        assert!(!KeywordKind::Order.is_order_modifier());
        assert!(!KeywordKind::Group.is_order_modifier());
        assert!(!KeywordKind::Select.is_order_modifier());
    }

    #[test]
    fn test_is_ddl_starter() {
        assert!(KeywordKind::Create.is_ddl_starter());
        assert!(KeywordKind::Alter.is_ddl_starter());
        assert!(KeywordKind::Drop.is_ddl_starter());
        assert!(KeywordKind::Truncate.is_ddl_starter());
        assert!(KeywordKind::Grant.is_ddl_starter());
        assert!(KeywordKind::Revoke.is_ddl_starter());

        assert!(!KeywordKind::Table.is_ddl_starter());
        assert!(!KeywordKind::Select.is_ddl_starter());
    }

    #[test]
    fn test_as_str_single_word() {
        assert_eq!(KeywordKind::Select.as_str(), "SELECT");
        assert_eq!(KeywordKind::From.as_str(), "FROM");
        assert_eq!(KeywordKind::Where.as_str(), "WHERE");
        assert_eq!(KeywordKind::Create.as_str(), "CREATE");
        assert_eq!(KeywordKind::True.as_str(), "TRUE");
        assert_eq!(KeywordKind::Null.as_str(), "NULL");
    }

    #[test]
    fn test_as_str_multi_word() {
        assert_eq!(KeywordKind::OrderBy.as_str(), "ORDER BY");
        assert_eq!(KeywordKind::GroupBy.as_str(), "GROUP BY");
        assert_eq!(KeywordKind::LeftJoin.as_str(), "LEFT JOIN");
        assert_eq!(KeywordKind::RightJoin.as_str(), "RIGHT JOIN");
        assert_eq!(KeywordKind::InnerJoin.as_str(), "INNER JOIN");
        assert_eq!(KeywordKind::OuterJoin.as_str(), "OUTER JOIN");
        assert_eq!(KeywordKind::FullJoin.as_str(), "FULL JOIN");
        assert_eq!(KeywordKind::CrossJoin.as_str(), "CROSS JOIN");
        assert_eq!(KeywordKind::UnionAll.as_str(), "UNION ALL");
        assert_eq!(KeywordKind::PrimaryKey.as_str(), "PRIMARY KEY");
        assert_eq!(KeywordKind::ForeignKey.as_str(), "FOREIGN KEY");
        assert_eq!(KeywordKind::IfExists.as_str(), "IF EXISTS");
        assert_eq!(KeywordKind::IfNotExists.as_str(), "IF NOT EXISTS");
        assert_eq!(KeywordKind::RowsBetween.as_str(), "ROWS BETWEEN");
        assert_eq!(KeywordKind::RangeBetween.as_str(), "RANGE BETWEEN");
    }

    #[test]
    fn test_as_str_standalone_variants() {
        assert_eq!(KeywordKind::Order.as_str(), "ORDER");
        assert_eq!(KeywordKind::Group.as_str(), "GROUP");
        assert_eq!(KeywordKind::Left.as_str(), "LEFT");
        assert_eq!(KeywordKind::Right.as_str(), "RIGHT");
        assert_eq!(KeywordKind::Inner.as_str(), "INNER");
        assert_eq!(KeywordKind::Outer.as_str(), "OUTER");
        assert_eq!(KeywordKind::Full.as_str(), "FULL");
        assert_eq!(KeywordKind::Cross.as_str(), "CROSS");
    }
}
