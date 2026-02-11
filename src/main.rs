use std::io::{self, Read};
use std::process;

use clap::Parser;
use rs_sql_indent::{FormatOptions, FormatStyle, format_sql};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Output keywords in lowercase
    #[arg(long)]
    lowercase: bool,

    /// Formatting style
    #[arg(long, value_enum, default_value_t = FormatStyle::Standard)]
    style: FormatStyle,
}

fn main() {
    let cli = Cli::parse();

    let options = FormatOptions {
        uppercase: !cli.lowercase,
        style: cli.style,
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

    let formatted = format_sql(&input, &options);
    println!("{}", formatted);
}
