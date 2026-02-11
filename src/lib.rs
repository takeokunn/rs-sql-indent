use sqlformat::{format, FormatOptions, QueryParams};
pub use sqlformat::Indent;

pub fn format_sql(input: &str, indent: Indent, uppercase: bool, lines_between_queries: u8) -> String {
    let options = FormatOptions {
        indent,
        uppercase,
        lines_between_queries,
    };
    format(input, &QueryParams::None, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_select() {
        let result = format_sql("select id, name from users", Indent::Spaces(2), true, 1);
        assert!(result.contains("SELECT"));
        assert!(result.contains("FROM"));
        assert!(result.contains("  id"));
    }

    #[test]
    fn test_select_with_where() {
        let result = format_sql("select id from users where id = 1", Indent::Spaces(2), true, 1);
        assert!(result.contains("SELECT"));
        assert!(result.contains("FROM"));
        assert!(result.contains("WHERE"));
    }

    #[test]
    fn test_empty_string() {
        let result = format_sql("", Indent::Spaces(2), true, 1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_multi_query() {
        let result = format_sql("select 1; select 2", Indent::Spaces(2), true, 1);
        assert!(result.contains("SELECT"));
        assert!(result.matches("SELECT").count() == 2);
    }

    #[test]
    fn test_with_join() {
        let result = format_sql("select u.id, o.total from users u join orders o on u.id = o.user_id", Indent::Spaces(2), true, 1);
        assert!(result.contains("SELECT"));
        assert!(result.contains("FROM"));
        assert!(result.contains("JOIN"));
        assert!(result.contains("ON"));
    }
}
