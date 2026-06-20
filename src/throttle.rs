use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub struct Throttle {
    min_secs: u64,
    max_secs: u64,
}

impl Throttle {
    pub fn new(min_secs: u64, max_secs: u64) -> Self {
        Self { min_secs, max_secs }
    }

    pub async fn wait(&self) {
        let secs = rand::thread_rng().gen_range(self.min_secs..=self.max_secs);
        println!("  ⏳ Pause de {}s avant le prochain envoi...", secs);
        sleep(Duration::from_secs(secs)).await;
    }
}
