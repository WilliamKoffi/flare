use crate::message::Message;
use crate::prospect::Prospect;

pub struct Template {
    subject: String,
    body: String,
}

impl Template {
    pub fn load() -> anyhow::Result<Self> {
        let subject = std::env::var("MAIL_SUBJECT")?;
        let body_path = std::env::var("MAIL_BODY_PATH")?;
        let body = std::fs::read_to_string(&body_path)?;

        Ok(Self { subject, body })
    }

    pub fn render(&self, prospect: &Prospect, base: &str) -> Message {
        Message::new(
            prospect.email.clone(),
            interpolate(&self.subject, prospect, base),
            interpolate(&self.body, prospect, base),
        )
    }
}

fn interpolate(value: &str, prospect: &Prospect, base: &str) -> String {
    value
        .replace("{{name}}", &prospect.name)
        .replace("{{link}}", &prospect.link(base))
}

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::prospect::Prospect;

    #[test]
    fn renders_prospect() {
        let template = Template {
            subject: "Présentation web — Maître {{name}}".into(),
            body: "Bonjour {{name}} — {{link}}".into(),
        };
        let prospect = Prospect {
            id: "1".into(),
            name: "Aya".into(),
            email: "aya@example.com".into(),
            gender: "F".into(),
            color: "B8860B".into(),
        };

        let message = template.render(&prospect, "https://example.com");

        assert_eq!(message.recipient(), "aya@example.com");
        assert_eq!(message.subject(), "Présentation web — Maître Aya");
        assert!(message.body().contains("Bonjour Aya"));
        assert!(message.body().contains("https://example.com/?name=Aya"));
    }
}
