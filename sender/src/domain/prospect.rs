use crate::domain::message::Message;
use crate::domain::template::Template;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prospect {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub gender: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub firm: String,
    #[serde(default)]
    pub url: String,
}

impl Prospect {
    /// AFFORDANCE: A prospect can be drafted into a message.
    pub fn draft(&self, template: &Template) -> Message {
        let (subject, body) = template.interpolate(self, &self.url);
        Message::new(self.email.clone(), subject, body)
    }

    pub(crate) fn personalization(&self) -> Personalization<'_> {
        Personalization {
            email: &self.email,
            name: &self.name,
            gender: &self.gender,
            phone: &self.phone,
            image: &self.image,
            firm: &self.firm,
        }
    }

    fn validate(&self, index: usize) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.id.trim().is_empty(),
            "prospect #{index}: id is required"
        );
        anyhow::ensure!(
            !self.name.trim().is_empty(),
            "prospect #{index}: name is required"
        );
        validate_email(&self.email)
            .map_err(|error| anyhow::anyhow!("prospect #{index}: {error}"))?;
        anyhow::ensure!(
            matches!(self.gender.as_str(), "M" | "F"),
            "prospect #{index}: gender must be M or F"
        );
        Ok(())
    }
}

#[derive(Serialize)]
pub(crate) struct Personalization<'a> {
    email: &'a str,
    name: &'a str,
    gender: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    phone: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    image: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    firm: &'a str,
}

#[derive(Deserialize)]
struct Directory {
    #[serde(default)]
    prospect: Vec<Prospect>,
    #[serde(default)]
    lawyer: Vec<Prospect>,
}

pub struct Prospects(Vec<Prospect>);

impl Prospects {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let mut directory: Directory = toml::from_str(&raw)
            .map_err(|error| anyhow::anyhow!("invalid prospect TOML in {path}: {error}"))?;
        directory.prospect.append(&mut directory.lawyer);

        for (index, prospect) in directory.prospect.iter().enumerate() {
            prospect.validate(index + 1)?;
        }

        Ok(Self(directory.prospect))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Prospect> {
        self.0.iter()
    }

    pub fn require_urls(&self) -> anyhow::Result<()> {
        let missing: Vec<_> = self
            .0
            .iter()
            .filter(|prospect| prospect.url.trim().is_empty())
            .map(|prospect| prospect.id.as_str())
            .collect();

        anyhow::ensure!(
            missing.is_empty(),
            "missing prepared URL for prospect ID(s): {}. Run `prospect-mailer prepare-links` first",
            missing.join(", ")
        );
        Ok(())
    }
}

pub(crate) fn validate_email(email: &str) -> anyhow::Result<()> {
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| anyhow::anyhow!("email must be a valid email address"))?;
    anyhow::ensure!(
        !local.is_empty()
            && !domain.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !email.contains(char::is_whitespace)
            && !email.contains(['\r', '\n']),
        "email must be a valid email address"
    );
    Ok(())
}

pub fn export_emails(input: &str, output: &str) -> anyhow::Result<()> {
    let raw = std::fs::read_to_string(input)?;
    let directory: toml::Value = toml::from_str(&raw)
        .map_err(|error| anyhow::anyhow!("invalid prospect TOML in {input}: {error}"))?;
    let mut emails = BTreeSet::new();

    for collection in ["prospect", "lawyer"] {
        let Some(records) = directory.get(collection).and_then(toml::Value::as_array) else {
            continue;
        };

        for (index, record) in records.iter().enumerate() {
            let value = record
                .get("email")
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow::anyhow!("{collection} #{}: email is required", index + 1))?;

            for email in value
                .split(|character: char| {
                    character.is_whitespace() || matches!(character, ',' | ';')
                })
                .filter(|email| !email.is_empty())
            {
                validate_email(email)
                    .map_err(|error| anyhow::anyhow!("{collection} #{}: {error}", index + 1))?;
                emails.insert(email.to_owned());
            }
        }
    }

    anyhow::ensure!(
        !emails.is_empty(),
        "no [[prospect]] or [[lawyer]] email addresses found in {input}"
    );

    let mut csv = String::from("email\n");
    for email in &emails {
        csv.push_str(&csv_field(email));
        csv.push('\n');
    }
    std::fs::write(output, csv)?;

    Ok(())
}

pub fn store_urls(path: &str, urls: &[(String, String)]) -> anyhow::Result<()> {
    let raw = std::fs::read_to_string(path)?;
    let mut directory: toml::Value = toml::from_str(&raw)
        .map_err(|error| anyhow::anyhow!("invalid prospect TOML in {path}: {error}"))?;
    let by_id: std::collections::HashMap<_, _> = urls
        .iter()
        .map(|(id, url)| (id.as_str(), url.as_str()))
        .collect();
    let mut updated = 0;

    for collection in ["prospect", "lawyer"] {
        let Some(records) = directory
            .get_mut(collection)
            .and_then(toml::Value::as_array_mut)
        else {
            continue;
        };

        for record in records {
            let Some(table) = record.as_table_mut() else {
                continue;
            };
            let Some(id) = table.get("id").and_then(toml::Value::as_str) else {
                continue;
            };
            let Some(url) = by_id.get(id) else {
                continue;
            };
            table.insert("url".into(), toml::Value::String((*url).to_owned()));
            updated += 1;
        }
    }

    anyhow::ensure!(
        updated == urls.len(),
        "updated {updated} of {} prospect URLs; prospect IDs must be unique",
        urls.len()
    );
    let serialized = toml::to_string_pretty(&directory)?;
    let temporary = format!("{path}.tmp");
    std::fs::write(&temporary, serialized)?;
    std::fs::rename(&temporary, path)?;
    Ok(())
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
#[path = "tests/prospect_tests.rs"]
mod tests;
