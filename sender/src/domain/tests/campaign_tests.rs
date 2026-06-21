use super::TestMail;

#[test]
fn test_message_is_independent_from_campaign_content() {
    let test = TestMail {
        recipient: "score@example.com".into(),
        slots: 1,
    };

    let message = test.message();

    assert_eq!(message.recipient(), "score@example.com");
    assert!(message.subject().starts_with("Test message "));
    assert!(message.body().starts_with("This is a simple test message."));
    assert!(!message.body().contains("lawyer"));
}
