use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum FormatStyle {
    #[default]
    Basic,
    Streamline,
    Aligned,
    Dataops,
}

impl FormatStyle {
    pub fn from_name(name: &str) -> Self {
        match name {
            "basic" => FormatStyle::Basic,
            "streamline" => FormatStyle::Streamline,
            "aligned" => FormatStyle::Aligned,
            "dataops" => FormatStyle::Dataops,
            _ => FormatStyle::Basic,
        }
    }
}

impl fmt::Display for FormatStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatStyle::Basic => write!(f, "basic"),
            FormatStyle::Streamline => write!(f, "streamline"),
            FormatStyle::Aligned => write!(f, "aligned"),
            FormatStyle::Dataops => write!(f, "dataops"),
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
            style: FormatStyle::Basic,
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
        assert_eq!(opts.style, FormatStyle::Basic);
    }

    #[test]
    fn test_format_options_equality() {
        let a = FormatOptions::default();
        let b = FormatOptions::default();
        assert_eq!(a, b);

        let c = FormatOptions {
            style: FormatStyle::Aligned,
            ..FormatOptions::default()
        };
        assert_ne!(a, c);
    }

    #[test]
    fn test_format_style_display() {
        assert_eq!(FormatStyle::Basic.to_string(), "basic");
        assert_eq!(FormatStyle::Streamline.to_string(), "streamline");
        assert_eq!(FormatStyle::Aligned.to_string(), "aligned");
        assert_eq!(FormatStyle::Dataops.to_string(), "dataops");
    }

    #[test]
    fn test_format_style_from_name_valid() {
        assert_eq!(FormatStyle::from_name("basic"), FormatStyle::Basic);
        assert_eq!(
            FormatStyle::from_name("streamline"),
            FormatStyle::Streamline
        );
        assert_eq!(FormatStyle::from_name("aligned"), FormatStyle::Aligned);
        assert_eq!(FormatStyle::from_name("dataops"), FormatStyle::Dataops);
    }

    #[test]
    fn test_format_style_from_name_unknown_falls_back_to_basic() {
        assert_eq!(FormatStyle::from_name("unknown"), FormatStyle::Basic);
        assert_eq!(FormatStyle::from_name(""), FormatStyle::Basic);
        assert_eq!(FormatStyle::from_name("Basic"), FormatStyle::Basic);
        assert_eq!(FormatStyle::from_name("BASIC"), FormatStyle::Basic);
    }

    #[test]
    fn test_format_style_from_name_display_roundtrip() {
        for style in [
            FormatStyle::Basic,
            FormatStyle::Streamline,
            FormatStyle::Aligned,
            FormatStyle::Dataops,
        ] {
            assert_eq!(FormatStyle::from_name(&style.to_string()), style);
        }
    }
}
