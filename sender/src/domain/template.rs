use crate::domain::prospect::Prospect;

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

    pub fn interpolate(&self, prospect: &Prospect, link: &str) -> (String, String) {
        (
            substitute(&self.subject, prospect, link),
            substitute(&self.body, prospect, link),
        )
    }
}

fn substitute(value: &str, prospect: &Prospect, link: &str) -> String {
    value
        .replace("{{name}}", &prospect.name)
        .replace("{{link}}", link)
}

#[cfg(test)]
#[path = "tests/template_tests.rs"]
mod tests;
