use crate::prospect::Prospect;

pub struct Template(String);

impl Template {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        Ok(Self(std::fs::read_to_string(path)?))
    }

    pub fn render(&self, prospect: &Prospect, base: &str) -> String {
        self.0
            .replace("{{name}}", &prospect.name)
            .replace("{{link}}", &prospect.link(base))
    }
}

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::prospect::Prospect;

    #[test]
    fn renders_prospect() {
        let template = Template("Bonjour {{name}} — {{link}}".into());
        let prospect = Prospect {
            id: "1".into(),
            name: "Aya".into(),
            email: "aya@example.com".into(),
            gender: "F".into(),
            color: "B8860B".into(),
        };

        let message = template.render(&prospect, "https://example.com");

        assert!(message.contains("Bonjour Aya"));
        assert!(message.contains("https://example.com/?name=Aya"));
    }
}
