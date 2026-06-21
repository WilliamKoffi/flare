use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Ledger {
    #[serde(skip)]
    path: String,
    #[serde(default)]
    entries: HashMap<String, DateTime<Utc>>,
}

impl Ledger {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        if std::path::Path::new(path).exists() {
            let raw = std::fs::read_to_string(path)?;
            let mut ledger: Ledger = serde_json::from_str(&raw)?;
            ledger.path = path.to_string();
            Ok(ledger)
        } else {
            Ok(Self {
                path: path.to_string(),
                entries: HashMap::new(),
            })
        }
    }

    pub fn sent(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn count(&self, date: NaiveDate) -> usize {
        self.entries
            .values()
            .filter(|sent| sent.date_naive() == date)
            .count()
    }

    pub fn today(&self) -> usize {
        self.count(Utc::now().date_naive())
    }

    pub fn earliest(&self) -> Option<NaiveDate> {
        self.entries.values().map(|sent| sent.date_naive()).min()
    }

    pub fn record(&mut self, id: &str) -> anyhow::Result<()> {
        self.entries.insert(id.to_string(), Utc::now());
        if let Some(parent) = std::path::Path::new(&self.path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/ledger_tests.rs"]
mod tests;
