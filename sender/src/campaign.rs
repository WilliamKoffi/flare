use crate::ledger::Ledger;
use crate::mailbox::Mailbox;
use crate::prospect::Prospects;
use crate::template::Template;
use crate::throttle::Throttle;

enum Mode<'a> {
    Preview,
    Delivery(&'a Mailbox),
}

pub struct Campaign<'a> {
    prospects: &'a Prospects,
    ledger: &'a mut Ledger,
    template: &'a Template,
    throttle: &'a Throttle,
    mode: Mode<'a>,
    base: &'a str,
    cap: usize,
}

impl<'a> Campaign<'a> {
    pub fn preview(
        prospects: &'a Prospects,
        ledger: &'a mut Ledger,
        template: &'a Template,
        throttle: &'a Throttle,
        base: &'a str,
        cap: usize,
    ) -> Self {
        Self {
            prospects,
            ledger,
            template,
            throttle,
            mode: Mode::Preview,
            base,
            cap,
        }
    }

    pub fn delivery(
        prospects: &'a Prospects,
        ledger: &'a mut Ledger,
        template: &'a Template,
        throttle: &'a Throttle,
        mailbox: &'a Mailbox,
        base: &'a str,
        cap: usize,
    ) -> Self {
        Self {
            prospects,
            ledger,
            template,
            throttle,
            mode: Mode::Delivery(mailbox),
            base,
            cap,
        }
    }

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

            let message = self.template.render(prospect, self.base);

            match self.mode {
                Mode::Preview => {
                    println!(
                        "\n--- DRY RUN : {} <{}> ---",
                        prospect.name,
                        message.recipient()
                    );
                    println!("Sujet : {}", message.subject());
                    println!("{}", message.body());
                    println!("--- fin ---\n");
                }
                Mode::Delivery(mailbox) => {
                    if delivered {
                        self.throttle.wait().await;
                    }

                    mailbox.send(&message).await?;
                    self.ledger.record(&prospect.id)?;
                    println!("✓ Envoyé à {} <{}>", prospect.name, message.recipient());
                    delivered = true;
                }
            }

            count += 1;
        }

        Ok(())
    }
}
