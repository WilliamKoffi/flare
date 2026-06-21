use super::{export_emails, store_urls, Directory, Prospect, Prospects};

#[test]
fn personalization_contains_required_and_populated_optional_fields_only() {
    let prospect = Prospect {
        id: "1".into(),
        name: "Aya Kouassi".into(),
        email: "aya@example.com".into(),
        gender: "F".into(),
        phone: "0700000000".into(),
        image: String::new(),
        firm: "Cabinet Kouassi".into(),
        url: String::new(),
    };

    let payload = serde_json::to_value(prospect.personalization()).unwrap();

    assert_eq!(payload["email"], "aya@example.com");
    assert_eq!(payload["name"], "Aya Kouassi");
    assert_eq!(payload["gender"], "F");
    assert_eq!(payload["phone"], "0700000000");
    assert_eq!(payload["firm"], "Cabinet Kouassi");
    assert!(payload.get("image").is_none());
}

#[test]
fn directory_ignores_scraper_only_keys() {
    let raw = r#"
        [metadata]
        total = 1

        [[prospect]]
        id = "1"
        name = "Aya Kouassi"
        email = "aya@example.com"
        gender = "F"
        link = "https://directory.test/1"
        office = "https://directory.test/firm/1"
        valid = true
    "#;

    let directory: Directory = toml::from_str(raw).unwrap();

    assert_eq!(directory.prospect.len(), 1);
    assert!(directory.prospect[0].phone.is_empty());
}

#[test]
fn loads_lawyer_records() {
    let directory =
        std::env::temp_dir().join(format!("prospect-mailer-load-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("prospects.toml");
    std::fs::write(
        &input,
        r#"
            [metadata]
            total = 1

            [[lawyer]]
            id = "1"
            name = "Aya Kouassi"
            email = "aya@example.com"
            gender = "F"
            valid = true
        "#,
    )
    .unwrap();

    let prospects = Prospects::load(input.to_str().unwrap()).unwrap();

    assert_eq!(prospects.iter().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn validates_required_fields() {
    let mut prospect = Prospect {
        id: "1".into(),
        name: "Aya Kouassi".into(),
        email: "not-an-email".into(),
        gender: "F".into(),
        phone: String::new(),
        image: String::new(),
        firm: String::new(),
        url: String::new(),
    };

    assert!(prospect.validate(1).is_err());
    prospect.email = "aya@example.com".into();
    prospect.gender = "X".into();
    assert!(prospect.validate(1).is_err());
}

#[test]
fn stores_urls_without_losing_other_fields() {
    let directory =
        std::env::temp_dir().join(format!("prospect-mailer-url-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("prospects.toml");
    std::fs::write(
        &input,
        r#"
            [metadata]
            total = 2

            [[prospect]]
            id = "1"
            name = "Aya"
            custom = "kept"

            [[lawyer]]
            id = "2"
            name = "Noa"
            url = "https://old.example"
        "#,
    )
    .unwrap();

    store_urls(
        input.to_str().unwrap(),
        &[
            ("1".into(), "https://example.com/v1_one".into()),
            ("2".into(), "https://lnk.ua/two".into()),
        ],
    )
    .unwrap();

    let updated: toml::Value = toml::from_str(&std::fs::read_to_string(&input).unwrap()).unwrap();
    assert_eq!(updated["metadata"]["total"].as_integer(), Some(2));
    assert_eq!(updated["prospect"][0]["custom"].as_str(), Some("kept"));
    assert_eq!(
        updated["prospect"][0]["url"].as_str(),
        Some("https://example.com/v1_one")
    );
    assert_eq!(
        updated["lawyer"][0]["url"].as_str(),
        Some("https://lnk.ua/two")
    );

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn exports_sorted_unique_emails_from_scraper_records() {
    let directory =
        std::env::temp_dir().join(format!("prospect-mailer-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("prospects.toml");
    let output = directory.join("prospects.csv");
    std::fs::write(
        &input,
        r#"
            [[lawyer]]
            email = "zara@example.com"

            [[lawyer]]
            email = "aya@example.com"

            [[lawyer]]
            email = "zara@example.com"

            [[lawyer]]
            email = "lea@example.com; noa@example.com"
        "#,
    )
    .unwrap();

    export_emails(input.to_str().unwrap(), output.to_str().unwrap()).unwrap();

    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "email\naya@example.com\nlea@example.com\nnoa@example.com\nzara@example.com\n"
    );

    std::fs::remove_dir_all(directory).unwrap();
}
