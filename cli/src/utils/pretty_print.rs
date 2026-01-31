use colored::Colorize;

/// Helper function to pad a string to a specific width
pub fn pad_string(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// Struct to represent a column configuration for pretty printing
pub struct ColumnConfig {
    pub header: String,
    pub color: fn(&str) -> colored::ColoredString,
    pub width: usize,
}

/// Generic function to pretty print tabular data with formatted columns and colors
pub fn pretty_print_table(
    title: &str,
    version: &str,
    columns: Vec<ColumnConfig>,
    rows: Vec<Vec<String>>,
    empty_message: &str,
) {
    // Print the version information
    println!("{} {}", title.purple().bold(), version);
    println!();

    if rows.is_empty() {
        println!("{}", empty_message.yellow().italic());
        return;
    }

    let column_spacing = 3;

    // Print header
    let mut header_str = String::new();
    let mut divider_len = 0;

    for (i, col) in columns.iter().enumerate() {
        if i < columns.len() - 1 {
            header_str.push_str(&format!(
                "{:width$}",
                col.header.green().bold(),
                width = col.width + column_spacing
            ));
        } else {
            // Last column doesn't need padding
            header_str.push_str(&col.header.green().bold().to_string());
        }
        divider_len += col.width;
    }

    // Add spacing between columns to divider length
    divider_len += (columns.len() - 1) * column_spacing;
    // Add extra padding for better visual appearance
    divider_len += 30;

    println!("{header_str}");
    println!("{}", "-".repeat(divider_len));

    // Print each row with the specified colors
    for row in rows {
        let mut row_str = String::new();

        for (i, (value, col)) in row.iter().zip(columns.iter()).enumerate() {
            if i < columns.len() - 1 {
                let padded = pad_string(value, col.width);
                row_str.push_str(&format!(
                    "{:width$}",
                    (col.color)(&padded),
                    width = col.width + column_spacing
                ));
            } else {
                // Last column doesn't need padding
                row_str.push_str(&(col.color)(value).to_string());
            }
        }

        println!("{row_str}");
    }
}
