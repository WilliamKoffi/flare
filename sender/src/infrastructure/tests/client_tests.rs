use super::header;

#[test]
fn rejects_header_injection() {
    assert!(header("person@example.com\r\nBcc: victim@example.com", "recipient").is_err());
    assert!(header("Safe value", "subject").is_ok());
}
