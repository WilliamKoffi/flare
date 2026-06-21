# Prospect Mailer

Prospect Mailer is a small Rust command-line application for sending personalized
outreach emails through the Gmail API. It reads prospects from JSON and mail
settings from a runtime `.env` file, renders a Markdown message as plain text, adds
a personalized website URL, throttles
deliveries, and records successful sends to avoid contacting the same prospect
twice.

The repository also contains an optional Playwright scraper for collecting raw
lawyer directory records. Its output must currently be transformed into the
mailer's prospect format before it can be used.

## Features

- Runtime-configurable email subject and Markdown body
- Personalized links built from prospect data
- Gmail API delivery with OAuth 2.0
- Dry-run mode for reviewing messages without sending them
- Random delay between successful sends
- Per-run sending cap
- Persistent ledger that prevents duplicate sends

## Requirements

- Rust toolchain with Cargo
- A Google Cloud project with the Gmail API enabled
- OAuth desktop application credentials
- A Gmail account authorized to send messages
- Optional scraper: Node.js or Bun, plus Playwright and Chromium

## Project Structure

```text
.
├── .env               # Private runtime config, ignored by Git
├── src/
│   ├── main.rs       # CLI and sending workflow
│   ├── prospect.rs   # Prospect loading and personalized URL generation
│   ├── mailbox.rs    # Gmail OAuth authentication and message delivery
│   ├── ledger.rs     # Successful-send persistence
│   └── throttle.rs   # Random delay between sends
├── scraper/main.ts   # Optional lawyer-directory scraper
├── scrapper/
│   └── directory.json    # Generated raw directory records
└── storage/
    ├── prospects.json    # Mailer input
    ├── credentials.json  # Gmail OAuth client credentials
    ├── token.json        # Generated OAuth token
    ├── sent.json         # Generated delivery ledger
    └── body.md           # Private message body, ignored by Git
```

## Gmail Setup

1. Create or select a project in Google Cloud Console.
2. Enable the Gmail API.
3. Configure the OAuth consent screen.
4. Create OAuth credentials for a desktop application.
5. Download the credentials and save them as `storage/credentials.json`.

On the first real send, the application opens the OAuth authorization flow and
stores the resulting token in `storage/token.json`. Runtime files in `storage/`
are ignored by Git.

The value passed to `--from` should match the Gmail account authorized during
this flow.

## Prospect Format

The mailer expects a JSON array:

```json
[
  {
    "id": "kouassi-aya-2026-05",
    "name": "Aya Kouassi",
    "email": "aya@example.com",
    "gender": "F",
    "color": "B8860B"
  }
]
```

Fields:

- `id`: stable unique identifier used by the send ledger
- `name`: recipient name
- `email`: recipient email address
- `gender`: value added to the personalized URL
- `color`: color value added to the personalized URL

The generated URL has this form. Recipient email addresses are deliberately
excluded to keep them out of browser history, analytics, logs, and referrer
headers:

```text
<base-url>/?name=...&gender=...&color=...
```

## Mail Configuration

Generate the private runtime files:

```bash
cargo run -- --init
```

The command creates `.env`, `storage/body.md`, `storage/prospects.json`, and
`storage/sent.json` when they do not exist. Existing files are never
overwritten. Google OAuth credentials must still be downloaded to
`storage/credentials.json`; `storage/token.json` is generated during the first
OAuth authorization.

Edit `.env` to change the subject or body path:

```dotenv
MAIL_SUBJECT="Une idée de vitrine web pour votre cabinet — Maître {{name}}"
MAIL_BODY_PATH="storage/body.md"
```

Edit `storage/body.md` to change the body. Both private files are ignored by
Git and are loaded at runtime, so they can be changed after compiling the
application. The subject and body support these placeholders:

```text
{{name}}
{{link}}
```

Messages are sent as UTF-8 plain text, not rendered HTML.

## Usage

Build the application:

```bash
cargo build
```

Always review generated messages with a dry run first:

```bash
cargo run -- \
  --base-url "https://example.com/presentation" \
  --from "sender@example.com" \
  --dry-run
```

Send emails using the default files and safety limits:

```bash
cargo run --release -- \
  --base-url "https://example.com/presentation" \
  --from "sender@example.com"
```

Available options:

```text
--init                    Generate missing local runtime files
--prospects <PATH>      Prospect JSON file (default: storage/prospects.json)
--ledger <PATH>         Successful-send ledger (default: storage/sent.json)
--env-file <PATH>       Runtime environment file (default: .env)
--base-url <URL>        Required personalized website base URL
--from <EMAIL>          Required sender address
--min-delay <SECONDS>   Minimum delay after a send (default: 600)
--max-delay <SECONDS>   Maximum delay after a send (default: 1800)
--daily-cap <COUNT>     Maximum successful sends per UTC day (default: 15)
--dry-run               Print messages without authenticating or sending
```

The minimum delay must not be greater than the maximum delay.

## Delivery Ledger

After Gmail accepts a message, the prospect ID and UTC timestamp are written to
`storage/sent.json`. Future runs skip IDs already present in that file.

The `--daily-cap` option includes successful sends already recorded for the
current UTC calendar day. Dry runs apply the same cap without modifying the
ledger.

Use a separate ledger path for testing if you do not want to affect the normal
send history:

```bash
cargo run -- \
  --base-url "https://example.com/presentation" \
  --from "sender@example.com" \
  --ledger "storage/test-sent.json"
```

## Optional Scraper

`scraper/main.ts` uses Playwright to browse the configured lawyer directory,
follow pagination, and write records to `scrapper/directory.json` by default:

```bash
cd scraper
npm install
npm run browsers
npm run scrape
```

Pass a different output path as the first argument:

```bash
npm run scrape -- ../scrapper/another-file.json
```

The scraper produces raw records with fields such as `name`, `profile`, `firm`,
`website`, `email`, `phone`, and `avatar`. This is not the same schema consumed
by the Rust mailer. A transformation step is required to create stable IDs and
provide `name`, `gender`, and `color`.

The scraper currently launches Chromium in visible mode and targets selectors
specific to the configured directory. Website changes may require selector
updates.

## Safety Notes

- Keep `storage/credentials.json`, `storage/token.json`, prospects, and send
  ledgers private.
- Use `--dry-run` before every campaign.
- Use stable prospect IDs; changing an ID can bypass duplicate protection.
- Confirm recipient consent and comply with applicable privacy, anti-spam, and
  professional solicitation rules.
- Start with conservative caps and delays.

## Verification

Run the Rust checks with:

```bash
cargo test
```

The project currently compiles successfully but does not contain automated unit
tests.
