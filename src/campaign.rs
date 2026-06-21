use crate::ledger::Ledger;
use crate::mailbox::Mailbox;
use crate::prospect::Prospects;
use crate::template::Template;
use crate::throttle::Throttle;

pub struct Campaign<'a> {
    pub prospects: &'a Prospects,
    pub ledger: &'a mut Ledger,
    pub template: &'a Template,
    pub throttle: &'a Throttle,
    pub mailbox: Option<&'a Mailbox>,
    pub base: &'a str,
    pub cap: usize,
    pub preview: bool,
}

impl Campaign<'_> {
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut count = self.ledger.today();
        let mut delivered = false;

        for prospect in self.prospects.iter() {
            if self.ledger.sent(&prospect.id) {
                println!("⏭  {} — déjà envoyé", prospect.name);
                continue;
            }

            if count >= self.cap {
                println!("🛑 Quota journalier atteint ({}).", self.cap);
                break;
            }

            let body = self.template.render(prospect, self.base);
            let subject = format!("Présentation web — Maître {}", prospect.name);

            if self.preview {
                println!("\n--- DRY RUN : {} <{}> ---", prospect.name, prospect.email);
                println!("Sujet : {}", subject);
                println!("{}", body);
                println!("--- fin ---\n");
            } else {
                if delivered {
                    self.throttle.wait().await;
                }

                let mailbox = self
                    .mailbox
                    .ok_or_else(|| anyhow::anyhow!("mailbox is required for delivery"))?;
                mailbox.send(&prospect.email, &subject, &body).await?;
                self.ledger.record(&prospect.id)?;
                println!("✓ Envoyé à {} <{}>", prospect.name, prospect.email);
                delivered = true;
            }

            count += 1;
        }

        Ok(())
    }
}
