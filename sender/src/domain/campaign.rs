//! Campaign domain entity. Owns the outreach state and drives the send loop.

use crate::domain::message::Message;
use crate::domain::prospect::{Prospect, Prospects};
use crate::domain::template::Template;
use crate::domain::warmup::Warmup;
use crate::infrastructure::client::Client;
use crate::infrastructure::ledger::Ledger;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub struct Campaign {
    prospects: Prospects,
    ledger: Ledger,
    template: Template,
    client: Option<Client>,
    capacity: usize,
    minimum: u64,
    maximum: u64,
    test: Option<TestMail>,
    warmup: Option<Warmup>,
}

impl Campaign {
    pub fn new(
        prospects: Prospects,
        ledger: Ledger,
        template: Template,
        client: Option<Client>,
        capacity: usize,
        minimum: u64,
        maximum: u64,
        test: Option<TestMail>,
        warmup: Option<Warmup>,
    ) -> Self {
        Self {
            prospects,
            ledger,
            template,
            client,
            capacity,
            minimum,
            maximum,
            test,
            warmup,
        }
    }

    /// AFFORDANCE: A campaign can be launched.
    pub async fn launch(mut self) -> anyhow::Result<()> {
        anyhow::ensure!(self.minimum <= self.maximum, "minimum must be <= maximum");

        let capacity = match &self.warmup {
            Some(schedule) => {
                let day = self
                    .ledger
                    .earliest()
                    .map(|start| {
                        let today = chrono::Utc::now().date_naive();
                        (today - start).num_days().max(0) as usize + 1
                    })
                    .unwrap_or(1);
                let scheduled = schedule.resolve(day);
                let effective = scheduled.min(self.capacity);
                println!(
                    "📈 Warmup day {day}: {effective} emails (schedule: {scheduled}, ceiling: {})",
                    self.capacity
                );
                effective
            }
            None => self.capacity,
        };

        let mut count = self.ledger.today();
        let mut sent = false;

        if let Some(t) = &self.test {
            let date = chrono::Utc::now().date_naive();

            for slot in 1..=t.slots {
                let id = format!("test:{date}:{slot}");
                if self.ledger.sent(&id) {
                    continue;
                }

                if count >= capacity {
                    println!("🛑 Quota journalier atteint ({}).", capacity);
                    return Ok(());
                }

                let message = t.message();

                match &self.client {
                    None => preview("TEST", &message),
                    Some(c) => {
                        if sent {
                            pause(self.minimum, self.maximum).await;
                        }

                        message.deliver(c).await?;
                        self.ledger.record(&id)?;
                        println!(
                            "✓ Email test {}/{} envoyé à {}",
                            slot,
                            t.slots,
                            message.recipient()
                        );
                        sent = true;
                    }
                }

                count += 1;
            }
        }

        for prospect in self.prospects.iter() {
            if self.ledger.sent(&prospect.id) {
                println!("⏭  {} — déjà envoyé", prospect.name);
                continue;
            }

            if count >= capacity {
                println!("🛑 Quota journalier atteint ({}).", capacity);
                break;
            }

            let message = prospect.draft(&self.template);

            match &self.client {
                None => {
                    detail("DRY RUN", prospect, &message);
                }
                Some(c) => {
                    if sent {
                        pause(self.minimum, self.maximum).await;
                    }

                    message.deliver(c).await?;
                    self.ledger.record(&prospect.id)?;
                    println!("✓ Envoyé à {} <{}>", prospect.name, message.recipient());
                    sent = true;
                }
            }

            count += 1;
        }

        Ok(())
    }
}

async fn pause(minimum: u64, maximum: u64) {
    let seconds = rand::thread_rng().gen_range(minimum..=maximum);
    println!("  ⏳ Pause de {}s avant le prochain envoi...", seconds);
    sleep(Duration::from_secs(seconds)).await;
}

fn detail(label: &str, prospect: &Prospect, message: &Message) {
    println!(
        "\n--- {label} : {} <{}> ---",
        prospect.name,
        message.recipient()
    );
    println!("Sujet : {}", message.subject());
    println!("{}", message.body());
    println!("--- fin ---\n");
}

fn preview(label: &str, message: &Message) {
    println!("\n--- {label} : <{}> ---", message.recipient());
    println!("Sujet : {}", message.subject());
    println!("{}", message.body());
    println!("--- fin ---\n");
}

#[derive(Debug)]
pub struct TestMail {
    recipient: String,
    slots: usize,
}

impl TestMail {
    pub fn load() -> anyhow::Result<Option<Self>> {
        let recipient = std::env::var("TEST_EMAIL")
            .unwrap_or_default()
            .trim()
            .to_string();
        let slots = std::env::var("TEST_EMAIL_SLOTS")
            .unwrap_or_else(|_| "0".into())
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("TEST_EMAIL_SLOTS must be a non-negative integer"))?;

        if slots == 0 {
            return Ok(None);
        }

        anyhow::ensure!(
            !recipient.is_empty(),
            "TEST_EMAIL is required when TEST_EMAIL_SLOTS is greater than 0"
        );
        crate::domain::prospect::validate_email(&recipient)?;

        Ok(Some(Self { recipient, slots }))
    }

    fn message(&self) -> Message {
        let reference = format!("{:08X}", rand::thread_rng().gen::<u32>());
        Message::new(
            self.recipient.clone(),
            format!("Test message {reference}"),
            format!("This is a simple test message.\n\nReference: {reference}"),
        )
    }
}

#[cfg(test)]
#[path = "tests/campaign_tests.rs"]
mod tests;
