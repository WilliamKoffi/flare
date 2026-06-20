use std::io::Cursor;

use google_gmail1::api::Message;
use google_gmail1::{hyper, hyper_rustls, Gmail};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub struct Mailbox {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    from: String,
}

impl Mailbox {
    pub async fn authenticate(from: String) -> anyhow::Result<Self> {
        let secret = yup_oauth2::read_application_secret("credentials.json").await?;
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("token.json")
                .build()
                .await?;

        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_or_http()
            .enable_http1()
            .build();

        let client = hyper::Client::builder().build(connector);
        let hub = Gmail::new(client, auth);

        Ok(Self { hub, from })
    }

    pub async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
        let raw_mail = format!(
            "From: {}\r\n\
             To: {}\r\n\
             Subject: {}\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: text/plain; charset=UTF-8\r\n\
             Content-Transfer-Encoding: 8bit\r\n\
             \r\n\
             {}",
            self.from, to, subject, body
        );

        let message = Message::default();
        let mime_type: mime::Mime = "message/rfc822".parse()?;
        let stream = Cursor::new(raw_mail.into_bytes());

        self.hub
            .users()
            .messages_send(message, "me")
            .upload(stream, mime_type)
            .await?;
        Ok(())
    }
}
