use crate::config::SmtpConfig;
use crate::email::{send_bulk, send_single, SendProgress};
use crate::template::{self, EmailTemplate, Recipient};
use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};

pub struct EmailApp {
    config: SmtpConfig,
    templates: Vec<EmailTemplate>,
    selected_template: Option<usize>,

    // Editing state for new recipient
    new_recipient_email: String,
    new_recipient_args: HashMap<String, String>,

    // New template name
    new_template_name: String,

    // Sending state
    progress_rx: Option<Receiver<SendProgress>>,
    is_sending: bool,
    status_log: Vec<String>,

    // Confirmation dialog
    show_confirm_dialog: bool,

    // Preview state
    preview_recipient_idx: Option<usize>,
}

impl EmailApp {
    pub fn new(config: SmtpConfig, templates: Vec<EmailTemplate>) -> Self {
        Self {
            config,
            templates,
            selected_template: None,
            new_recipient_email: String::new(),
            new_recipient_args: HashMap::new(),
            new_template_name: String::new(),
            progress_rx: None,
            is_sending: false,
            status_log: Vec::new(),
            show_confirm_dialog: false,
            preview_recipient_idx: None,
        }
    }

    fn save_templates(&self) {
        template::save_templates(&self.templates);
    }

    fn poll_progress(&mut self) {
        if let Some(rx) = &self.progress_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    SendProgress::Sent { index, email } => {
                        self.status_log
                            .push(format!("✓ [{}] Sent to {}", index + 1, email));
                    }
                    SendProgress::Failed { index, email, error } => {
                        self.status_log
                            .push(format!("✗ [{}] Failed to send to {}: {}", index + 1, email, error));
                    }
                    SendProgress::Done => {
                        self.status_log.push("— Bulk send complete.".to_string());
                        self.is_sending = false;
                        self.progress_rx = None;
                        return;
                    }
                }
            }
        }
    }
}

impl eframe::App for EmailApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_progress();

        // Request repaint while sending so we see progress updates
        if self.is_sending {
            ctx.request_repaint();
        }

        // --- Confirmation Dialog ---
        if self.show_confirm_dialog {
            egui::Window::new("Confirm Bulk Send")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(idx) = self.selected_template {
                        let count = self.templates[idx].recipients.len();
                        ui.label(format!(
                            "You are about to send emails to {} recipient(s).",
                            count
                        ));
                        ui.label("Are you sure you want to proceed?");
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("  Cancel  ").clicked() {
                                self.show_confirm_dialog = false;
                            }
                            if ui
                                .button(
                                    egui::RichText::new("  Send All  ")
                                        .color(egui::Color32::WHITE),
                                )
                                .clicked()
                            {
                                self.show_confirm_dialog = false;
                                // Start bulk send
                                let (tx, rx) = mpsc::channel();
                                let template = self.templates[idx].clone();
                                let config = self.config.clone();
                                self.progress_rx = Some(rx);
                                self.is_sending = true;
                                self.status_log
                                    .push(format!("— Starting bulk send for '{}'...", template.name));
                                send_bulk(config, template, tx);
                            }
                        });
                    }
                });
        }

        // --- Left Panel: Template List ---
        egui::SidePanel::left("template_list")
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Templates");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_template_name);
                    if ui.button("+ New").clicked() && !self.new_template_name.is_empty() {
                        let t = EmailTemplate::new(self.new_template_name.clone());
                        self.templates.push(t);
                        self.selected_template = Some(self.templates.len() - 1);
                        self.new_template_name.clear();
                        self.save_templates();
                    }
                });

                ui.separator();

                let mut to_delete: Option<usize> = None;
                for (i, t) in self.templates.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let label = if Some(i) == self.selected_template {
                            egui::RichText::new(&t.name).strong()
                        } else {
                            egui::RichText::new(&t.name)
                        };
                        if ui.selectable_label(Some(i) == self.selected_template, label).clicked() {
                            self.selected_template = Some(i);
                            self.new_recipient_email.clear();
                            self.new_recipient_args.clear();
                            self.preview_recipient_idx = None;
                        }
                        if ui.small_button("🗑").clicked() {
                            to_delete = Some(i);
                        }
                    });
                }

                if let Some(del) = to_delete {
                    self.templates.remove(del);
                    if self.selected_template == Some(del) {
                        self.selected_template = None;
                    } else if let Some(sel) = self.selected_template {
                        if sel > del {
                            self.selected_template = Some(sel - 1);
                        }
                    }
                    self.save_templates();
                }
            });

        // --- Bottom Panel: Status Log ---
        egui::TopBottomPanel::bottom("status_log")
            .min_height(120.0)
            .show(ctx, |ui| {
                ui.heading("Status Log");
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &self.status_log {
                            if line.starts_with('✓') {
                                ui.colored_label(egui::Color32::from_rgb(80, 200, 80), line);
                            } else if line.starts_with('✗') {
                                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), line);
                            } else {
                                ui.label(line);
                            }
                        }
                    });
            });

        // --- Central Panel: Template Editor ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(idx) = self.selected_template {
                // We need to work with a clone to avoid borrow issues, then copy back
                let mut template = self.templates[idx].clone();
                let mut changed = false;

                ui.heading(format!("Editing: {}", template.name));
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- Template Name ---
                    ui.horizontal(|ui| {
                        ui.label("Template Name:");
                        if ui.text_edit_singleline(&mut template.name).changed() {
                            changed = true;
                        }
                    });

                    ui.add_space(5.0);

                    // --- Subject ---
                    ui.horizontal(|ui| {
                        ui.label("Subject:");
                        if ui.text_edit_singleline(&mut template.subject).changed() {
                            changed = true;
                        }
                    });

                    ui.add_space(5.0);

                    // --- Body ---
                    ui.label("Body (use {placeholder} for per-recipient variables):");
                    ui.label("Tip: Use the toolbar below to format text, or type HTML tags directly (e.g. <b>bold</b>).");
                    ui.add_space(3.0);

                    // --- Formatting Toolbar ---
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new(" B ").strong().size(14.0))
                            .on_hover_text("Bold (wraps selection in <b></b>)")
                            .clicked()
                        {
                            wrap_body_selection(&mut template.body, "<b>", "</b>");
                            changed = true;
                        }
                        if ui.button(egui::RichText::new(" I ").italics().size(14.0))
                            .on_hover_text("Italic (wraps selection in <i></i>)")
                            .clicked()
                        {
                            wrap_body_selection(&mut template.body, "<i>", "</i>");
                            changed = true;
                        }
                        if ui.button(egui::RichText::new(" U ").underline().size(14.0))
                            .on_hover_text("Underline (wraps selection in <u></u>)")
                            .clicked()
                        {
                            wrap_body_selection(&mut template.body, "<u>", "</u>");
                            changed = true;
                        }
                        ui.separator();
                        if ui.button("Link")
                            .on_hover_text("Insert a hyperlink at cursor")
                            .clicked()
                        {
                            template.body.push_str("<a href=\"URL\">link text</a>");
                            changed = true;
                        }
                        if ui.button("• List Item")
                            .on_hover_text("Insert a bullet point")
                            .clicked()
                        {
                            template.body.push_str("\n• ");
                            changed = true;
                        }
                    });

                    ui.add_space(3.0);

                    let body_edit = ui
                        .add(
                            egui::TextEdit::multiline(&mut template.body)
                                .desired_width(f32::INFINITY)
                                .desired_rows(8)
                                .hint_text("Hello {name},\n\nI wanted to reach out about..."),
                        );
                    if body_edit.changed() {
                        changed = true;
                    }

                    // Handle keyboard shortcuts for formatting (Ctrl+B, Ctrl+I, Ctrl+U)
                    if body_edit.has_focus() {
                        let modifiers = ui.input(|i| i.modifiers);
                        if modifiers.ctrl || modifiers.command {
                            if ui.input(|i| i.key_pressed(egui::Key::B)) {
                                wrap_body_selection(&mut template.body, "<b>", "</b>");
                                changed = true;
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::I)) {
                                wrap_body_selection(&mut template.body, "<i>", "</i>");
                                changed = true;
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::U)) {
                                wrap_body_selection(&mut template.body, "<u>", "</u>");
                                changed = true;
                            }
                        }
                    }

                    // Show detected placeholders
                    let placeholders = template.extract_placeholders();
                    if !placeholders.is_empty() {
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.label("Detected placeholders:");
                            for p in &placeholders {
                                ui.code(format!("{{{}}}", p));
                            }
                        });
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // --- Attachments ---
                    ui.heading("Attachments");
                    let mut attachment_to_remove: Option<usize> = None;
                    for (ai, path) in template.attachment_paths.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "📎 {}",
                                path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path.to_string_lossy().to_string())
                            ));
                            if ui.small_button("Remove").clicked() {
                                attachment_to_remove = Some(ai);
                            }
                        });
                    }
                    if let Some(rm) = attachment_to_remove {
                        template.attachment_paths.remove(rm);
                        changed = true;
                    }

                    if ui.button("📁 Add Attachment(s)").clicked() {
                        if let Some(files) = rfd::FileDialog::new().pick_files() {
                            for f in files {
                                template.attachment_paths.push(f);
                            }
                            changed = true;
                        }
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // --- Recipients ---
                    ui.heading("Recipients");
                    ui.add_space(5.0);

                    let placeholders = template.extract_placeholders();

                    // Table of current recipients
                    let mut recipient_to_remove: Option<usize> = None;
                    let mut send_single_idx: Option<usize> = None;

                    if !template.recipients.is_empty() {
                        egui::Grid::new("recipients_grid")
                            .striped(true)
                            .min_col_width(100.0)
                            .show(ui, |ui| {
                                // Header
                                ui.strong("#");
                                ui.strong("Email");
                                for p in &placeholders {
                                    ui.strong(p);
                                }
                                ui.strong("Actions");
                                ui.end_row();

                                for (ri, recipient) in template.recipients.iter_mut().enumerate() {
                                    ui.label(format!("{}", ri + 1));

                                    if ui
                                        .add(egui::TextEdit::singleline(&mut recipient.email).desired_width(200.0))
                                        .changed()
                                    {
                                        changed = true;
                                    }

                                    for p in &placeholders {
                                        let val = recipient
                                            .args
                                            .entry(p.clone())
                                            .or_default();
                                        if ui
                                            .add(egui::TextEdit::singleline(val).desired_width(120.0))
                                            .changed()
                                        {
                                            changed = true;
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        if ui.small_button("Send").clicked() {
                                            send_single_idx = Some(ri);
                                        }
                                        if ui.small_button("🗑").clicked() {
                                            recipient_to_remove = Some(ri);
                                        }
                                        if ui.small_button("👁").on_hover_text("Preview").clicked() {
                                            self.preview_recipient_idx = Some(ri);
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                    } else {
                        ui.label("No recipients yet. Add one below.");
                    }

                    if let Some(rm) = recipient_to_remove {
                        template.recipients.remove(rm);
                        changed = true;
                    }

                    // Handle single send
                    if let Some(si) = send_single_idx {
                        let recipient = template.recipients[si].clone();
                        let config = self.config.clone();
                        let tmpl = template.clone();
                        match send_single(&config, &tmpl, &recipient) {
                            Ok(()) => {
                                self.status_log
                                    .push(format!("✓ Sent to {}", recipient.email));
                            }
                            Err(e) => {
                                self.status_log
                                    .push(format!("✗ Failed to send to {}: {}", recipient.email, e));
                            }
                        }
                    }

                    ui.add_space(5.0);

                    // Add new recipient
                    ui.group(|ui| {
                        ui.label("Add Recipient:");
                        ui.horizontal(|ui| {
                            ui.label("Email:");
                            ui.text_edit_singleline(&mut self.new_recipient_email);
                        });
                        for p in &placeholders {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", p));
                                self.new_recipient_args.entry(p.clone()).or_default();
                                if let Some(val) = self.new_recipient_args.get_mut(p) {
                                    ui.text_edit_singleline(val);
                                }
                            });
                        }
                        if ui.button("+ Add Recipient").clicked()
                            && !self.new_recipient_email.is_empty()
                        {
                            let recipient = Recipient {
                                email: self.new_recipient_email.clone(),
                                args: self.new_recipient_args.clone(),
                            };
                            template.recipients.push(recipient);
                            self.new_recipient_email.clear();
                            self.new_recipient_args.clear();
                            changed = true;
                        }
                    });

                    // --- Preview ---
                    if let Some(pi) = self.preview_recipient_idx {
                        if pi < template.recipients.len() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.heading("📨 Preview");
                            let r = &template.recipients[pi];
                            ui.label(format!("To: {}", r.email));
                            ui.label(format!("Subject: {}", template.render_subject(r)));
                            ui.add_space(5.0);
                            ui.group(|ui| {
                                let rendered = template.render_body(r);
                                render_html_preview(ui, &rendered);
                            });
                            ui.add_space(3.0);
                            ui.colored_label(
                                egui::Color32::from_rgb(150, 150, 150),
                                "(This is an approximate preview. The actual email may render slightly differently in Gmail.)"
                            );
                            if !template.attachment_paths.is_empty() {
                                ui.label(format!(
                                    "Attachments: {}",
                                    template
                                        .attachment_paths
                                        .iter()
                                        .filter_map(|p| p.file_name())
                                        .map(|n| n.to_string_lossy().to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ));
                            }
                        }
                    }

                    ui.add_space(15.0);
                    ui.separator();

                    // --- Send Buttons ---
                    ui.horizontal(|ui| {
                        let can_send = !template.recipients.is_empty() && !self.is_sending;

                        if ui
                            .add_enabled(
                                can_send,
                                egui::Button::new(
                                    egui::RichText::new("🚀 Send to All Recipients")
                                        .size(16.0),
                                ),
                            )
                            .clicked()
                        {
                            self.show_confirm_dialog = true;
                        }

                        if self.is_sending {
                            ui.spinner();
                            ui.label("Sending...");
                        }
                    });

                    if ui.button("Clear Log").clicked() {
                        self.status_log.clear();
                    }
                });

                // Write back changes
                if changed {
                    self.templates[idx] = template;
                    self.save_templates();
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("📧 Bulk Email Sender");
                    ui.add_space(20.0);
                    ui.label("Select a template from the left panel or create a new one.");
                    ui.add_space(10.0);
                    ui.label("Use {placeholder} syntax in subject/body for per-recipient personalization.");
                });
            }
        });
    }
}

/// Wraps text in the body with HTML tags. Since egui TextEdit doesn't expose
/// selection ranges directly, this function appends empty tags at the end
/// of the body so the user can type formatted content within them.
fn wrap_body_selection(body: &mut String, open_tag: &str, close_tag: &str) {
    // Append opening and closing tag so user can type between them
    body.push_str(open_tag);
    body.push_str(close_tag);
}

/// Renders a simple HTML preview in egui, supporting <b>, <i>, <u>, <a>, and <br> tags.
/// This provides an approximate visual preview of how the email will look.
fn render_html_preview(ui: &mut egui::Ui, html: &str) {
    // Parse the HTML into segments with formatting info
    let segments = parse_html_segments(html);

    // Use a wrapping layout to render segments inline
    ui.horizontal_wrapped(|ui| {
        for segment in &segments {
            if segment.is_newline {
                ui.end_row();
                continue;
            }
            let mut text = egui::RichText::new(&segment.text);
            if segment.bold {
                text = text.strong();
            }
            if segment.italic {
                text = text.italics();
            }
            if segment.underline {
                text = text.underline();
            }
            if segment.is_link {
                text = text.color(egui::Color32::from_rgb(66, 133, 244));
                text = text.underline();
            }
            ui.label(text);
        }
    });
}

#[derive(Debug)]
struct HtmlSegment {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
    is_link: bool,
    is_newline: bool,
}

/// Simple HTML parser that extracts text segments with their formatting state.
fn parse_html_segments(html: &str) -> Vec<HtmlSegment> {
    let mut segments = Vec::new();
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut is_link = false;
    let mut current_text = String::new();
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            // Flush current text
            if !current_text.is_empty() {
                segments.push(HtmlSegment {
                    text: current_text.clone(),
                    bold,
                    italic,
                    underline,
                    is_link,
                    is_newline: false,
                });
                current_text.clear();
            }

            // Read the tag
            let mut tag = String::new();
            for tc in chars.by_ref() {
                if tc == '>' {
                    break;
                }
                tag.push(tc);
            }
            let tag_lower = tag.to_lowercase();

            match tag_lower.as_str() {
                "b" | "strong" => bold = true,
                "/b" | "/strong" => bold = false,
                "i" | "em" => italic = true,
                "/i" | "/em" => italic = false,
                "u" => underline = true,
                "/u" => underline = false,
                "/a" => is_link = false,
                "br" | "br/" | "br /" => {
                    segments.push(HtmlSegment {
                        text: String::new(),
                        bold: false,
                        italic: false,
                        underline: false,
                        is_link: false,
                        is_newline: true,
                    });
                }
                t if t.starts_with("a ") || t == "a" => {
                    is_link = true;
                }
                _ => {} // Ignore unknown tags
            }
        } else if c == '\n' {
            // Flush current text before newline
            if !current_text.is_empty() {
                segments.push(HtmlSegment {
                    text: current_text.clone(),
                    bold,
                    italic,
                    underline,
                    is_link,
                    is_newline: false,
                });
                current_text.clear();
            }
            segments.push(HtmlSegment {
                text: String::new(),
                bold: false,
                italic: false,
                underline: false,
                is_link: false,
                is_newline: true,
            });
        } else {
            current_text.push(c);
        }
    }

    // Flush remaining text
    if !current_text.is_empty() {
        segments.push(HtmlSegment {
            text: current_text,
            bold,
            italic,
            underline,
            is_link,
            is_newline: false,
        });
    }

    segments
}

