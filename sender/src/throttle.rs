use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub struct Throttle {
    min_secs: u64,
    max_secs: u64,
}

impl Throttle {
    pub fn new(min_secs: u64, max_secs: u64) -> anyhow::Result<Self> {
        anyhow::ensure!(min_secs <= max_secs, "min_delay must be <= max_delay");
        Ok(Self { min_secs, max_secs })
    }

    pub async fn wait(&self) {
        let secs = rand::thread_rng().gen_range(self.min_secs..=self.max_secs);
        println!("  ⏳ Pause de {}s avant le prochain envoi...", secs);
        sleep(Duration::from_secs(secs)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::Throttle;

    #[test]
    fn rejects_inverted_range() {
        assert!(Throttle::new(10, 5).is_err());
        assert!(Throttle::new(5, 10).is_ok());
    }
}
