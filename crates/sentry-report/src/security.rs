//! Sentry Report Security & Sanitization Utilities
//! Defenses against CSV Injection (Formula Injection) and HTML XSS injection.

/// Escape string for safe embedding into HTML documents.
pub fn escape_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(c),
        }
    }
    output
}

/// Sanitize a string cell for CSV to prevent formula execution in spreadsheet software (CSV Injection / CWE-1236).
/// Prepends single quote if field starts with `=`, `+`, `-`, `@`, `\t`, or `\r`.
pub fn sanitize_csv_cell(cell: &str) -> String {
    let trimmed = cell.trim();
    let dangerous_prefix = trimmed.starts_with('=')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('@')
        || trimmed.starts_with('\t')
        || trimmed.starts_with('\r');

    let sanitized = if dangerous_prefix {
        format!("'{}", trimmed)
    } else {
        trimmed.to_string()
    };

    // If cell contains commas, quotes, or newlines, quote it and escape internal quotes
    if sanitized.contains(',') || sanitized.contains('"') || sanitized.contains('\n') || sanitized.contains('\r') {
        format!("\"{}\"", sanitized.replace('"', "\"\""))
    } else {
        sanitized
    }
}
