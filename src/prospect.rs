use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prospect {
    pub id: String,
    pub nom: String,
    pub email: String,
    pub genre: String,
    pub specialite: String,
    pub couleur: String,
}

impl Prospect {
    pub fn link(&self, base_url: &str) -> String {
        format!(
            "{}/?email={}&nom={}&genre={}&specialite={}&couleur={}",
            base_url.trim_end_matches('/'),
            urlencoding::encode(&self.email),
            urlencoding::encode(&self.nom),
            self.genre,
            urlencoding::encode(&self.specialite),
            self.couleur,
        )
    }

    pub fn load_all(path: &str) -> anyhow::Result<Vec<Self>> {
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }
}
