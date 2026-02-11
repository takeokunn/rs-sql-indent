use wasm_bindgen::prelude::*;
use crate::config::{FormatOptions, FormatStyle};
use crate::format_sql;

#[wasm_bindgen]
pub fn format_sql_wasm(input: &str, uppercase: bool, style: &str) -> String {
    let format_style = match style {
        "river" => FormatStyle::River,
        _ => FormatStyle::Standard,
    };

    let options = FormatOptions {
        uppercase,
        style: format_style,
    };

    format_sql(input, &options)
}
