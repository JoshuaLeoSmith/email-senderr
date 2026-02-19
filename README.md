# 📧 email-senderr

A desktop application for sending personalized bulk emails with templates, per-recipient arguments, and file attachments — built entirely in Rust.

Stop copy-pasting the same email over and over. Create a template once, add your recipients with personalized fields, and send to everyone in one click.

---

## Features

- **Template Management** — Create, edit, and delete reusable email templates. Templates are automatically persisted to disk (`templates.json`).
- **Placeholder Syntax** — Use `{placeholder}` tokens in the subject and body that get replaced with unique values for each recipient (e.g., `{name}`, `{company}`, `{role}`).
- **Rich Text Formatting** — Format your email body with **bold**, *italic*, and <u>underline</u> using a built-in toolbar or keyboard shortcuts (`Ctrl+B`, `Ctrl+I`, `Ctrl+U`). You can also type HTML tags directly. Emails are sent as HTML with a plain-text fallback for maximum compatibility.
- **File Attachments** — Attach one or more files to any template via a native file picker dialog. All recipients receive the same attachments.
- **Recipient List** — Each template has its own list of recipients. Every recipient has an email address and a set of key-value arguments that map to placeholders.
- **Inline Editing** — Edit recipient emails and argument values directly in the recipients grid.
- **Email Preview** — Preview exactly what a specific recipient will see (rendered subject, body, and attachment list) before sending.
- **Single Send** — Send to one recipient at a time using the per-row **Send** button.
- **Bulk Send** — Send to all recipients at once with a single click. A confirmation dialog ensures you don't send accidentally.
- **Live Status Log** — A color-coded log at the bottom of the window shows real-time send progress, successes (✓), and failures (✗).
- **Anti-Spam Measures** — Proper `From` / `Reply-To` / `Message-ID` / `Date` headers, STARTTLS encryption, and configurable throttle delay between sends.
- **Native GUI** — Cross-platform desktop UI powered by [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe). No browser, no Electron.

---

## Screenshots

*Launch the app and create your first template:*

```
┌─────────────────┬───────────────────────────────────────────────────┐
│  Templates      │  📧 Bulk Email Sender                            │
│─────────────────│                                                   │
│  [___________]  │  Select a template from the left panel or create  │
│  [+ New]        │  a new one.                                       │
│                 │                                                   │
│  ▸ Outreach v1  │  Use {placeholder} syntax in subject/body for     │
│  ▸ Follow-up    │  per-recipient personalization.                   │
│                 │                                                   │
├─────────────────┴───────────────────────────────────────────────────┤
│  Status Log                                                         │
│  ✓ [1] Sent to joshua@example.com                                  │
│  ✓ [2] Sent to daniel@example.com                                  │
│  — Bulk send complete.                                              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024 / rustc 1.85+)
- A Gmail account with an [App Password](https://myaccount.google.com/apppasswords) (or another SMTP provider)

### Installation

```bash
git clone https://github.com/JoshuaLeoSmith/email-senderr.git
cd email-senderr
cargo build --release
```

The compiled binary will be at `target/release/email-senderr` (or `email-senderr.exe` on Windows).

### Configuration

Edit `src/Settings.toml` with your SMTP credentials before running:

```toml
host = "smtp.gmail.com"
username = "your-email@gmail.com"
password = "your-app-password"
from_name = "Your Name"
send_delay_ms = 2000
```

| Field            | Description                                                                                         |
|------------------|-----------------------------------------------------------------------------------------------------|
| `host`           | SMTP server hostname (e.g., `smtp.gmail.com`)                                                       |
| `username`       | The email address used to authenticate and appear in the `From` header                              |
| `password`       | SMTP password — for Gmail, use an [App Password](https://myaccount.google.com/apppasswords)         |
| `from_name`      | Display name that appears in the `From` field (e.g., `Joshua Smith`)                                |
| `send_delay_ms`  | Milliseconds to wait between each email during bulk send (helps avoid spam filters; default `2000`) |

> **Gmail users:** You must enable 2-Step Verification on your Google account, then generate an App Password at [https://myaccount.google.com/apppasswords](https://myaccount.google.com/apppasswords). Use that 16-character password in the `password` field — not your regular Gmail password.

### Running

```bash
cargo run
```

Or run the compiled release binary directly:

```bash
./target/release/email-senderr
```

---

## Usage

### 1. Create a Template

1. Type a name in the text field at the top of the left panel and click **+ New**.
2. Fill in the **Subject** and **Body**. Use `{placeholder}` syntax for any values that should vary per recipient.
3. Use the formatting toolbar above the body editor to apply **Bold**, *Italic*, or <u>Underline</u> formatting. You can also use keyboard shortcuts: `Ctrl+B`, `Ctrl+I`, `Ctrl+U`.
4. You can type HTML tags directly in the body (e.g., `<b>bold text</b>`, `<a href="https://example.com">link</a>`).

**Example body:**

```
Hello <b>{name}</b>,

I'm reaching out from <b>{company}</b> regarding your interest in <i>{service}</i>.
I'd love to schedule a quick call to discuss how we can help.

Best regards,
Joshua
```

This template has three placeholders: `{name}`, `{company}`, and `{service}`.

### 2. Add Attachments (Optional)

Click **📁 Add Attachment(s)** to open a file picker. Select one or more files. They will be listed with a **Remove** button next to each.

### 3. Add Recipients

In the **Add Recipient** section, enter an email address and fill in values for each detected placeholder, then click **+ Add Recipient**.

| Email                  | name    | company       | service              |
|------------------------|---------|---------------|----------------------|
| joshua@example.com     | Joshua  | Acme Corp     | cloud consulting     |
| daniel@example.com     | Daniel  | Widgets Inc   | DevOps automation    |

Recipient details (email and argument values) can also be edited inline directly in the recipients grid.

### 4. Preview

Click the **👁** (eye) button on any recipient row to see a rendered preview of the email as that recipient would receive it — subject, body with all placeholders replaced, and the attachment list.

### 5. Send

- **Single send:** Click the **Send** button on an individual recipient row.
- **Bulk send:** Click **🚀 Send to All Recipients**. A confirmation dialog will appear showing the recipient count. Confirm to begin sending.

During bulk send, a spinner is displayed and the **Status Log** at the bottom updates in real time:

```
— Starting bulk send for 'Outreach v1'...
✓ [1] Sent to joshua@example.com
✓ [2] Sent to daniel@example.com
✗ [3] Failed to send to invalid@bad: relay access denied
— Bulk send complete.
```

---

## Project Structure

```
email-senderr/
├── Cargo.toml              # Dependencies and project metadata
├── LICENSE                  # MIT License
├── README.md               # This file
├── templates.json           # Auto-generated template persistence (created at runtime)
└── src/
    ├── main.rs              # Entry point — loads config & templates, launches GUI
    ├── app.rs               # egui application — UI layout, state management, user interactions
    ├── config.rs            # SMTP configuration loading from Settings.toml
    ├── email.rs             # Email building (lettre), SMTP transport, single/bulk send logic
    ├── template.rs          # Template & Recipient data models, placeholder rendering, JSON persistence
    └── Settings.toml        # SMTP credentials and send configuration
```

---

## Dependencies

| Crate                                                        | Purpose                                      |
|--------------------------------------------------------------|----------------------------------------------|
| [eframe](https://crates.io/crates/eframe) / [egui](https://crates.io/crates/egui) | Native desktop GUI framework       |
| [lettre](https://crates.io/crates/lettre)                    | SMTP email building and transport             |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | Serialization for template persistence |
| [config](https://crates.io/crates/config)                    | TOML configuration file loading               |
| [rfd](https://crates.io/crates/rfd)                          | Native file picker dialogs                    |
| [uuid](https://crates.io/crates/uuid)                        | Unique template IDs and Message-ID generation |

---

## Anti-Spam Best Practices

This application implements several measures to reduce the chance of emails being flagged as spam:

1. **Proper Headers** — Every email includes `From`, `Reply-To`, `Message-ID`, and `Date` headers as recommended by RFC 5322.
2. **Unique Message-ID** — Each email gets a globally unique `Message-ID` generated with UUID v4 + timestamp.
3. **HTML + Plain-Text (multipart/alternative)** — Every email includes both an HTML body and an auto-generated plain-text fallback, which is the format preferred by major email providers and reduces spam scoring.
4. **STARTTLS Encryption** — Connects to the SMTP server over an encrypted TLS connection.
5. **Send Throttling** — A configurable delay (`send_delay_ms`) is applied between each email during bulk sends to avoid triggering rate limits.
6. **Authenticated SMTP** — Uses proper credential-based authentication with the mail server.

> **Tip:** For best deliverability, keep your email content professional, avoid excessive links or images, and ensure your sending domain has proper SPF/DKIM/DMARC records configured.

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 JoshuaLeoSmith
