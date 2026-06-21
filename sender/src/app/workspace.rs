use anyhow::{ensure, Result};
use std::fs::{create_dir_all, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;

use crate::infrastructure::link;

const ENVIRONMENT: &str = concat!(
    "BASE_URL=\"https://example.com/presentation\"\n",
    "LINK_ENCRYPTION_TOKEN=\"replace-with-at-least-32-random-characters\"\n",
    "LNK_BEARER_TOKEN=\"\"\n",
    "SENDER_EMAIL=\"sender@example.com\"\n",
    "MAIL_SUBJECT=\"Hello {{name}}\"\n",
    "MAIL_BODY_PATH=\"storage/body.md\"\n",
    "TEST_EMAIL=\"\"\n",
    "TEST_EMAIL_SLOTS=0\n",
);

const BODY: &str = "Hello {{name}},\n\nI have prepared a brief overview tailored to your profile.\n\nYou can view it here:\n\n{{link}}\n\nThis is a free preview with no commitment. Feel free to reach out if you would like to discuss further.\n\nBest regards,\n[your contact details]\n";

pub(crate) fn setup() -> Result<()> {
    create_dir_all("storage")?;

    create(".env", &generate_environment()?)?;
    create("storage/body.md", BODY)?;
    create(
        "storage/prospects.toml",
        "[[prospect]]\nid = \"example\"\nname = \"Aya Kouassi\"\nemail = \"aya@example.com\"\ngender = \"F\"\nurl = \"\"\n",
    )?;
    create("storage/sent.json", "{}\n")?;
    create(
        "storage/warmup.toml",
        "[[tier]]\nday = 1\ncapacity = 2\n\n[[tier]]\nday = 4\ncapacity = 5\n\n[[tier]]\nday = 8\ncapacity = 10\n\n[[tier]]\nday = 15\ncapacity = 20\n",
    )?;

    println!(
        "Setup complete. Add prospects to storage/prospects.toml and download Google OAuth credentials to storage/credentials.json."
    );
    Ok(())
}

pub(crate) fn require(paths: &[&str]) -> Result<()> {
    let missing: Vec<_> = paths
        .iter()
        .filter(|path| !Path::new(path).is_file())
        .copied()
        .collect();

    ensure!(
        missing.is_empty(),
        "missing required file(s): {}\nRun `prospect-mailer workspace setup` to generate local runtime files.",
        missing.join(", ")
    );
    Ok(())
}

fn generate_environment() -> Result<String> {
    Ok(ENVIRONMENT.replace(
        "replace-with-at-least-32-random-characters",
        &link::generate_token()?,
    ))
}

fn create(path: &str, content: &str) -> Result<()> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            file.write_all(content.as_bytes())?;
            println!("created {path}");
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            println!("kept existing {path}");
        }
        Err(error) => return Err(error.into()),
    }
    Ok(())
}
