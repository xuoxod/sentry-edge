pub struct LogSanitizer;

impl LogSanitizer {
    pub fn sanitize(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '\r' => ' ',
                '\n' => ' ',
                '\t' => ' ',
                '\0'..='\x1f' | '\x7f' => '?',
                _ => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    pub fn validate_no_crlf(input: &str) -> bool {
        !input.contains('\r') && !input.contains('\n') && !input.contains('\0')
    }
}
