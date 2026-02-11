use std::io::{self, Read};
use std::process;

use clap::Parser;
use rs_sql_indent::{Indent, format_sql};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(long, default_value_t = 2, conflicts_with = "indent_tabs")]
    indent_spaces: u8,

    #[arg(long, default_value_t = false)]
    indent_tabs: bool,

    /// Disable uppercasing SQL keywords
    #[arg(long)]
    no_uppercase: bool,

    #[arg(long, default_value_t = 1)]
    lines_between_queries: u8,
}

fn main() {
    let cli = Cli::parse();

    let indent = if cli.indent_tabs {
        Indent::Tabs
    } else {
        Indent::Spaces(cli.indent_spaces)
    };

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Error reading stdin: {}", e);
        process::exit(1);
    }

    if input.trim().is_empty() {
        eprintln!("Error: no SQL input provided");
        process::exit(1);
    }

    let formatted = format_sql(&input, indent, !cli.no_uppercase, cli.lines_between_queries);
    println!("{}", formatted);
}
