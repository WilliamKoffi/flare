use super::{multipart, CreateResponse, Links, NONCE_LEN};
use crate::domain::prospect::Prospect;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::{aead, digest};

fn prospect() -> Prospect {
    Prospect {
        id: "1".into(),
        name: "Aya Kouassi".into(),
        email: "aya@example.com".into(),
        gender: "F".into(),
        phone: "0700000000".into(),
        image: String::new(),
        firm: "Cabinet Kouassi".into(),
        url: String::new(),
    }
}

fn links(token: &str) -> Links {
    let digest = digest::digest(&digest::SHA256, token.as_bytes());
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, digest.as_ref()).unwrap();
    Links {
        base: "https://example.com/presentation".into(),
        key: aead::LessSafeKey::new(key),
        shortener: None,
    }
}

#[tokio::test]
async fn encrypted_link_hides_and_recovers_personal_data() {
    let token = "a sufficiently long shared secret token";
    let link = links(token).for_prospect(&prospect()).await.unwrap();

    assert!(link.starts_with("https://example.com/presentation/v1_"));
    assert!(!link.contains("Aya"));
    assert!(!link.contains("aya@example.com"));
    assert!(!link.contains('?'));

    let encoded = link.rsplit_once("v1_").unwrap().1;
    let encrypted = URL_SAFE_NO_PAD.decode(encoded).unwrap();
    let (nonce, ciphertext) = encrypted.split_at(NONCE_LEN);
    let digest = digest::digest(&digest::SHA256, token.as_bytes());
    let key = aead::UnboundKey::new(&aead::AES_256_GCM, digest.as_ref()).unwrap();
    let key = aead::LessSafeKey::new(key);
    let mut ciphertext = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(
            aead::Nonce::try_assume_unique_for_key(nonce).unwrap(),
            aead::Aad::empty(),
            &mut ciphertext,
        )
        .unwrap();
    let payload: serde_json::Value = serde_json::from_slice(plaintext).unwrap();

    assert_eq!(payload["name"], "Aya Kouassi");
    assert_eq!(payload["email"], "aya@example.com");
    assert!(payload.get("image").is_none());
}

#[test]
fn creates_lnk_multipart_field() {
    let body = String::from_utf8(multipart("boundary", "https://example.com/v1_abc")).unwrap();

    assert!(body.contains("Content-Disposition: form-data; name=\"link\""));
    assert!(body.contains("https://example.com/v1_abc"));
    assert!(body.ends_with("--boundary--\r\n"));
}

#[test]
fn reads_short_link_from_lnk_response() {
    let response: CreateResponse = serde_json::from_str(
        r#"{
            "result": {
                "lnk": "https://lnk.ua/bMen0KNgd",
                "key": "bMen0KNgd",
                "qr": "https://lnk.ua/qr/bMen0KNgd"
            }
        }"#,
    )
    .unwrap();

    assert_eq!(response.result.lnk, "https://lnk.ua/bMen0KNgd");
}
