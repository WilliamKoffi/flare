use crate::infrastructure::client::Client;

pub struct Message {
    recipient: String,
    subject: String,
    body: String,
}

impl Message {
    pub fn new(recipient: String, subject: String, body: String) -> Self {
        Self {
            recipient,
            subject,
            body,
        }
    }

    /// AFFORDANCE: A message can be delivered via a transport client.
    pub async fn deliver(&self, client: &Client) -> anyhow::Result<()> {
        client.transmit(self).await
    }

    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}
