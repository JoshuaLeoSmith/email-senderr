use crate::config::SmtpConfig;
use crate::template::{EmailTemplate, Recipient};
use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum SendProgress {
    Sent { index: usize, email: String },
    Failed { index: usize, email: String, error: String },
    Done,
}

pub fn build_message(
    config: &SmtpConfig,
    template: &EmailTemplate,
    recipient: &Recipient,
) -> Result<Message, Box<dyn std::error::Error>> {
    let from = format!("{} <{}>", config.from_name, config.username);
    let rendered_subject = template.render_subject(recipient);
    let rendered_body = template.render_body(recipient);

    let mut builder = Message::builder()
        .from(from.parse()?)
        .reply_to(config.username.parse()?)
        .to(recipient.email.parse()?)
        .subject(rendered_subject);

    // Add Message-ID header for anti-spam
    let msg_id = format!(
        "<{}.{}@{}>",
        uuid::Uuid::new_v4(),
        chrono_timestamp(),
        config.host
    );
    builder = builder.message_id(Some(msg_id));

    // Convert plain newlines to <br> so line breaks are preserved in the email.
    // HTML tags from the formatting toolbar (bold, italic, underline) pass through as-is.
    let rendered_body_html = rendered_body.replace('\n', "<br>");

    // Build the HTML body with a wrapper for proper email rendering
    let html_body = format!(
        "<!DOCTYPE html>\
        <html><head><meta charset=\"UTF-8\"></head>\
        <body style=\"font-family: Arial, sans-serif; font-size: 14px; color: #222;\">{}</body></html>",
        rendered_body_html
    );

    // Build a plain-text fallback by stripping HTML tags
    let plain_body = strip_html_tags(&rendered_body);

    // Create an alternative part (plain + HTML) so email clients pick the best version
    let alternative = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(plain_body),
        )
        .singlepart(
            SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(html_body),
        );

    if template.attachment_paths.is_empty() {
        Ok(builder.multipart(alternative)?)
    } else {
        // Wrap alternative + attachments in a mixed multipart
        let mut multipart = MultiPart::mixed().multipart(alternative);

        for path in &template.attachment_paths {
            if let Ok(file_bytes) = std::fs::read(path) {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "attachment".to_string());

                let content_type = ContentType::parse("application/octet-stream")
                    .unwrap_or(ContentType::TEXT_PLAIN);

                let attachment =
                    Attachment::new(filename).body(file_bytes, content_type);

                multipart = multipart.singlepart(attachment);
            }
        }

        Ok(builder.multipart(multipart)?)
    }
}

pub fn create_transport(config: &SmtpConfig) -> Result<SmtpTransport, Box<dyn std::error::Error>> {
    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let transport = SmtpTransport::relay(&config.host)?
        // .port(config.port)
        .credentials(creds)
        .build();

    Ok(transport)
}

pub fn send_single(
    config: &SmtpConfig,
    template: &EmailTemplate,
    recipient: &Recipient,
) -> Result<(), String> {
    let transport = create_transport(config).map_err(|e| e.to_string())?;
    let message = build_message(config, template, recipient).map_err(|e| e.to_string())?;
    transport.send(&message).map_err(|e| format!("{:?}", e))?;
    Ok(())
}

pub fn send_bulk(
    config: SmtpConfig,
    template: EmailTemplate,
    progress_tx: Sender<SendProgress>,
) {
    std::thread::spawn(move || {
        let transport = match create_transport(&config) {
            Ok(t) => t,
            Err(e) => {
                let _ = progress_tx.send(SendProgress::Failed {
                    index: 0,
                    email: "N/A".to_string(),
                    error: format!("Failed to create transport: {}", e),
                });
                let _ = progress_tx.send(SendProgress::Done);
                return;
            }
        };

        let delay = std::time::Duration::from_millis(config.send_delay_ms);

        for (i, recipient) in template.recipients.iter().enumerate() {
            match build_message(&config, &template, recipient) {
                Ok(message) => match transport.send(&message) {
                    Ok(_) => {
                        let _ = progress_tx.send(SendProgress::Sent {
                            index: i,
                            email: recipient.email.clone(),
                        });
                    }
                    Err(e) => {
                        let _ = progress_tx.send(SendProgress::Failed {
                            index: i,
                            email: recipient.email.clone(),
                            error: format!("{:?}", e),
                        });
                    }
                },
                Err(e) => {
                    let _ = progress_tx.send(SendProgress::Failed {
                        index: i,
                        email: recipient.email.clone(),
                        error: e.to_string(),
                    });
                }
            }

            // Throttle to avoid spam filters
            if i < template.recipients.len() - 1 {
                std::thread::sleep(delay);
            }
        }

        let _ = progress_tx.send(SendProgress::Done);
    });
}

fn chrono_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Simple HTML tag stripper for generating a plain-text fallback.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut inside_tag = false;
    for c in html.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(c),
            _ => {}
        }
    }
    result
}

