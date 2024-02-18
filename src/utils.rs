use crate::block_logic::BlockRequestError;
use serde_json::Value;
use std::fmt::Display;

// Convert an f64 value to a human readable format
// e.g 523294.0223 = "523.3K"
pub fn format_generic<T: Into<f64> + Display>(value: T) -> String {
    const K: f64 = 1_000.0;
    const M: f64 = 1_000_000.0;
    const B: f64 = 1_000_000_000.0;

    let value = value.into();

    if value < K {
        return format!("{}", value);
    } else if value < M {
        return format!("{:.1}K", value / K);
    } else if value < B {
        return format!("{:.1}M", value / M);
    } else {
        return format!("{:.1}B", value / B);
    }
}

// Prints one input string per line surrounded by a box
pub fn print_in_box(texts: Vec<String>) {
    let max_len = texts.iter().map(|s| s.len()).max().unwrap_or(0);
    let horizontal_line = format!("#{:-<width$}#", "", width = max_len);
    println!("{}", horizontal_line);

    for text in texts {
        let line = format!("#{:-<width$}#", text, width = max_len);
        println!("{}", line);
    }
    println!("{}", horizontal_line);
}

pub fn parse_hexa_value(input: &Value) -> Result<u64, BlockRequestError> {
    Ok(u64::from_str_radix(
        input
            .as_str()
            .ok_or(BlockRequestError::ConversionError(
                "Json block parameter to str",
            ))?
            .trim_start_matches("0x"),
        16,
    )
    .map_err(BlockRequestError::IntConversionError)?)
}
