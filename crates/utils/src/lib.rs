pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_line_endings_converts_windows_and_old_mac_endings() {
        let input = "a\r\nb\rc\n";

        assert_eq!(normalize_line_endings(input), "a\nb\nc\n");
    }
}
