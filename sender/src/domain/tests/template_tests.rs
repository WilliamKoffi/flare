use super::Template;
use crate::domain::prospect::Prospect;

#[test]
fn interpolates_prospect() {
    let template = Template {
        subject: "Présentation web — Maître {{name}}".into(),
        body: "Bonjour {{name}} — {{link}}".into(),
    };
    let prospect = Prospect {
        id: "1".into(),
        name: "Aya".into(),
        email: "aya@example.com".into(),
        gender: "F".into(),
        phone: String::new(),
        image: String::new(),
        firm: String::new(),
        url: "https://example.com/v1_encrypted".into(),
    };

    let (subject, body) = template.interpolate(&prospect, "https://example.com/v1_encrypted");

    assert_eq!(subject, "Présentation web — Maître Aya");
    assert!(body.contains("Bonjour Aya"));
    assert!(body.contains("https://example.com/v1_encrypted"));
}
