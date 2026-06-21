use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prospect {
    pub id: String,
    pub name: String,
    pub email: String,
    pub gender: String,
    pub color: String,
}

impl Prospect {
    pub fn link(&self, base: &str) -> String {
        format!(
            "{}/?name={}&gender={}&color={}",
            base.trim_end_matches('/'),
            urlencoding::encode(&self.name),
            urlencoding::encode(&self.gender),
            urlencoding::encode(&self.color),
        )
    }
}

pub struct Prospects(Vec<Prospect>);

impl Prospects {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        Ok(Self(serde_json::from_str(&raw)?))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Prospect> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Prospect;

    #[test]
    fn link_excludes_email() {
        let prospect = Prospect {
            id: "1".into(),
            name: "Aya Kouassi".into(),
            email: "aya@example.com".into(),
            gender: "F".into(),
            color: "B8860B".into(),
        };

        let link = prospect.link("https://example.com/");

        assert!(!link.contains("email"));
        assert!(!link.contains("aya%40example.com"));
        assert!(link.starts_with("https://example.com/?name=Aya%20Kouassi"));
    }
}
