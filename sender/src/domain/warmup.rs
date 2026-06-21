//! Warmup schedule. Ramps daily capacity based on campaign age.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Tier {
    day: usize,
    capacity: usize,
}

#[derive(Debug, Deserialize)]
pub struct Warmup {
    tier: Vec<Tier>,
}

impl Warmup {
    pub fn load(path: &str) -> anyhow::Result<Option<Self>> {
        if !std::path::Path::new(path).is_file() {
            return Ok(None);
        }

        let raw = std::fs::read_to_string(path)?;
        let mut warmup: Warmup = toml::from_str(&raw)
            .map_err(|error| anyhow::anyhow!("invalid warmup TOML in {path}: {error}"))?;

        anyhow::ensure!(
            !warmup.tier.is_empty(),
            "warmup schedule must have at least one tier"
        );
        warmup.tier.sort_by_key(|t| t.day);

        Ok(Some(warmup))
    }

    /// Returns the capacity for a given campaign day.
    /// Picks the last tier whose `day` is <= the current campaign day.
    pub fn resolve(&self, day: usize) -> usize {
        self.tier
            .iter()
            .rev()
            .find(|t| t.day <= day)
            .map(|t| t.capacity)
            .unwrap_or(self.tier[0].capacity)
    }
}

#[cfg(test)]
#[path = "tests/warmup_tests.rs"]
mod tests;
