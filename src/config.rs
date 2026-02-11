use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum FormatStyle {
    #[default]
    Standard,
    River,
}

impl fmt::Display for FormatStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatStyle::Standard => write!(f, "standard"),
            FormatStyle::River => write!(f, "river"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub uppercase: bool,
    pub style: FormatStyle,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            uppercase: true,
            style: FormatStyle::Standard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format_options() {
        let opts = FormatOptions::default();
        assert!(opts.uppercase);
        assert_eq!(opts.style, FormatStyle::Standard);
    }

    #[test]
    fn test_format_options_equality() {
        let a = FormatOptions::default();
        let b = FormatOptions::default();
        assert_eq!(a, b);

        let c = FormatOptions {
            style: FormatStyle::River,
            ..FormatOptions::default()
        };
        assert_ne!(a, c);
    }

    #[test]
    fn test_format_style_display() {
        assert_eq!(FormatStyle::Standard.to_string(), "standard");
        assert_eq!(FormatStyle::River.to_string(), "river");
    }
}
