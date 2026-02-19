use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub email: String,
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub attachment_paths: Vec<PathBuf>,
    pub recipients: Vec<Recipient>,
}

impl EmailTemplate {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            subject: String::new(),
            body: String::new(),
            attachment_paths: Vec::new(),
            recipients: Vec::new(),
        }
    }

    /// Replace all `{key}` placeholders with recipient arg values.
    pub fn render_text(&self, text: &str, recipient: &Recipient) -> String {
        let mut result = text.to_string();
        for (key, value) in &recipient.args {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    pub fn render_subject(&self, recipient: &Recipient) -> String {
        self.render_text(&self.subject, recipient)
    }

    pub fn render_body(&self, recipient: &Recipient) -> String {
        self.render_text(&self.body, recipient)
    }

    /// Extract all placeholder keys like `{name}` from body and subject.
    pub fn extract_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        let combined = format!("{} {}", self.subject, self.body);
        let mut chars = combined.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let mut key = String::new();
                for inner in chars.by_ref() {
                    if inner == '}' {
                        break;
                    }
                    key.push(inner);
                }
                if !key.is_empty() && !placeholders.contains(&key) {
                    placeholders.push(key);
                }
            }
        }
        placeholders
    }
}

const TEMPLATES_FILE: &str = "templates.json";

pub fn load_templates() -> Vec<EmailTemplate> {
    match std::fs::read_to_string(TEMPLATES_FILE) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_templates(templates: &[EmailTemplate]) {
    if let Ok(data) = serde_json::to_string_pretty(templates) {
        let _ = std::fs::write(TEMPLATES_FILE, data);
    }
}

