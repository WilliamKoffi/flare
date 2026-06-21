use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use google_gmail1::{hyper, hyper_rustls};
use ring::{aead, digest, rand};
use serde::Deserialize;

use crate::domain::prospect::Prospect;

const LNK_CREATE_URL: &str = "https://lnk.ua/api/v1/link/create";
const NONCE_LEN: usize = 12;
const TOKEN_PLACEHOLDER: &str = "replace-with-at-least-32-random-characters";

pub fn generate_token() -> anyhow::Result<String> {
    let mut token = [0_u8; 32];
    rand::SecureRandom::fill(&rand::SystemRandom::new(), &mut token)
        .map_err(|_| anyhow::anyhow!("could not generate a link encryption token"))?;
    Ok(URL_SAFE_NO_PAD.encode(token))
}

pub struct Links {
    base: String,
    key: aead::LessSafeKey,
    shortener: Option<Shortener>,
}

impl Links {
    pub fn load(base: String, shorten: bool) -> anyhow::Result<Self> {
        anyhow::ensure!(
            base.starts_with("https://") || base.starts_with("http://"),
            "BASE_URL must start with http:// or https://"
        );

        let token = std::env::var("LINK_ENCRYPTION_TOKEN")
            .map_err(|_| anyhow::anyhow!("LINK_ENCRYPTION_TOKEN is missing"))?;
        anyhow::ensure!(
            token.len() >= 32 && token != TOKEN_PLACEHOLDER,
            "LINK_ENCRYPTION_TOKEN must be replaced with at least 32 random bytes"
        );

        let digest = digest::digest(&digest::SHA256, token.as_bytes());
        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, digest.as_ref())
            .map_err(|_| anyhow::anyhow!("could not initialize link encryption"))?;

        let bearer = std::env::var("LNK_BEARER_TOKEN").unwrap_or_default();
        let bearer = bearer.trim().to_string();
        anyhow::ensure!(
            !shorten || !bearer.is_empty(),
            "LNK_BEARER_TOKEN is required with --shorten"
        );

        Ok(Self {
            base,
            key: aead::LessSafeKey::new(unbound),
            shortener: if shorten {
                Some(Shortener::new(bearer)?)
            } else {
                None
            },
        })
    }

    pub async fn for_prospect(&self, prospect: &Prospect) -> anyhow::Result<String> {
        let encrypted = self.encrypt(prospect)?;
        let long = format!("{}/{}", self.base.trim_end_matches('/'), encrypted);

        match &self.shortener {
            Some(shortener) => shortener.shorten(&long).await,
            None => Ok(long),
        }
    }

    fn encrypt(&self, prospect: &Prospect) -> anyhow::Result<String> {
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        rand::SecureRandom::fill(&rand::SystemRandom::new(), &mut nonce_bytes)
            .map_err(|_| anyhow::anyhow!("could not generate an encryption nonce"))?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = serde_json::to_vec(&prospect.personalization())?;
        self.key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext)
            .map_err(|_| anyhow::anyhow!("could not encrypt prospect data"))?;

        let mut encoded = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        encoded.extend_from_slice(&nonce_bytes);
        encoded.extend_from_slice(&ciphertext);
        Ok(format!("v1_{}", URL_SAFE_NO_PAD.encode(encoded)))
    }
}

struct Shortener {
    bearer: String,
    client: hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl Shortener {
    fn new(bearer: String) -> anyhow::Result<Self> {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_only()
            .enable_http1()
            .build();

        Ok(Self {
            bearer,
            client: hyper::Client::builder().build(connector),
        })
    }

    async fn shorten(&self, link: &str) -> anyhow::Result<String> {
        let boundary = format!("flare-{:016x}", rand::SystemRandom::new().generate_u64()?);
        let body = multipart(&boundary, link);
        let request = hyper::Request::post(LNK_CREATE_URL)
            .header(hyper::header::ACCEPT, "application/json")
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", self.bearer),
            )
            .header(
                hyper::header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(hyper::Body::from(body))?;

        let response = self.client.request(request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        anyhow::ensure!(
            status.is_success(),
            "lnk.ua returned {status}: {}",
            String::from_utf8_lossy(&body)
        );

        let response: CreateResponse = serde_json::from_slice(&body)
            .map_err(|error| anyhow::anyhow!("invalid lnk.ua response: {error}"))?;
        anyhow::ensure!(
            response.result.lnk.starts_with("https://"),
            "lnk.ua returned an invalid short link"
        );
        Ok(response.result.lnk)
    }
}

trait RandomU64 {
    fn generate_u64(&self) -> anyhow::Result<u64>;
}

impl RandomU64 for rand::SystemRandom {
    fn generate_u64(&self) -> anyhow::Result<u64> {
        let mut bytes = [0_u8; 8];
        rand::SecureRandom::fill(self, &mut bytes)
            .map_err(|_| anyhow::anyhow!("could not generate multipart boundary"))?;
        Ok(u64::from_le_bytes(bytes))
    }
}

#[derive(Deserialize)]
struct CreateResponse {
    result: CreateResult,
}

#[derive(Deserialize)]
struct CreateResult {
    lnk: String,
}

fn multipart(boundary: &str, link: &str) -> Vec<u8> {
    format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"link\"\r\n\
         \r\n\
         {link}\r\n\
         --{boundary}--\r\n"
    )
    .into_bytes()
}

#[cfg(test)]
#[path = "tests/link_tests.rs"]
mod tests;
