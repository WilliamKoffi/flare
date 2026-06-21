use std::io::Cursor;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use google_gmail1::api::Message as GmailMessage;
use google_gmail1::{hyper, hyper_rustls, Gmail};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use crate::domain::message::Message;

pub struct Client {
    hub: Gmail<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    from: String,
}

impl Client {
    pub async fn authenticate(from: String) -> anyhow::Result<Self> {
        header(&from, "sender")?;
        let secret = yup_oauth2::read_application_secret("storage/credentials.json").await?;
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("storage/token.json")
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

    pub async fn transmit(&self, message: &Message) -> anyhow::Result<()> {
        header(message.recipient(), "recipient")?;
        header(message.subject(), "subject")?;
        let subject = format!("=?UTF-8?B?{}?=", STANDARD.encode(message.subject()));
        let raw = format!(
            "From: {}\r\n\
             To: {}\r\n\
             Subject: {}\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: text/plain; charset=UTF-8\r\n\
             Content-Transfer-Encoding: 8bit\r\n\
             \r\n\
             {}",
            self.from,
            message.recipient(),
            subject,
            message.body()
        );

        let message = GmailMessage::default();
        let mime: mime::Mime = "message/rfc822".parse()?;
        let stream = Cursor::new(raw.into_bytes());

        self.hub
            .users()
            .messages_send(message, "me")
            .upload(stream, mime)
            .await?;
        Ok(())
    }
}

fn header(value: &str, field: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !value.contains(['\r', '\n']),
        "{field} contains a line break"
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests/client_tests.rs"]
mod tests;
