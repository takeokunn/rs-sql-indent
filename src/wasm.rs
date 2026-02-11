use crate::config::{FormatOptions, FormatStyle};
use crate::format_sql;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn format_sql_wasm(input: &str, uppercase: bool, style: &str) -> String {
    let options = FormatOptions {
        uppercase,
        style: FormatStyle::from_name(style),
    };

    format_sql(input, &options)
}
