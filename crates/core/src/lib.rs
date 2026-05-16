pub fn hello_message() -> String {
    "byi is installed and ready.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_message_is_stable() {
        assert_eq!(hello_message(), "byi is installed and ready.");
    }
}
