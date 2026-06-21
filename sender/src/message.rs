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
