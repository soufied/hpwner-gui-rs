use crate::backend::{self, CommandRequest};
use crate::settings::Settings;
use crate::ui::components::{
    button, field_label, hint_text, ui_scale, ButtonKind, Ctx,
};
use crate::ui::controls::{
    dropdown, multiline_input, segmented, slider, text_input, toggle_switch, InputOptions,
    Validation, FIELD_WIDTH_LONG, FIELD_WIDTH_MEDIUM, FIELD_WIDTH_SHORT,
};
use crate::ui::icons::{self, Icon};
use crate::ui::overlays::{
    confirm_body, modal, modal_kind_title, section_for_shortcut, shortcuts_body, show_palette,
    show_toasts, PaletteState, ToastStack,
};
use crate::ui::shell::{
    form_card, output_panel, page_header, page_margin, page_transition, sidebar, sidebar_width,
    spacing_for, status_bar, OutputView, AUTO_COLLAPSE_WIDTH, CONTENT_MAX_WIDTH,
};
use crate::ui::state::{ModalKind, PaletteAction, RunState, Section, ToastKind, ViewMode};
use crate::ui::theme::{Accent, Theme, SPACE_12, SPACE_16, SPACE_8};
use eframe::egui::{self, Key, Modifiers, Sense, Stroke, Vec2};
use std::fs;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EncryptFamily {
    Happ,
    V2RayTun,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum UaPreset {
    Happ,
    Incy,
    V2RayTun,
    Custom,
}

impl UaPreset {
    pub fn as_cli_value(&self, custom: &str) -> Option<String> {
        match self {
            Self::Happ => Some("happ".to_string()),
            Self::Incy => Some("incy".to_string()),
            Self::V2RayTun => Some("v2".to_string()),
            Self::Custom => {
                let trimmed = custom.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConvertInputMode {
    Paste,
    File,
    Url,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConvertMode {
    Uri,
    SingBox,
    Base64,
    Raw,
    SingBoxUri,
}

impl ConvertMode {
    pub fn to_cli_args(&self) -> Vec<String> {
        match self {
            Self::Uri => vec!["uri".to_string()],
            Self::SingBox => vec!["sb".to_string()],
            Self::Base64 => vec!["b64".to_string()],
            Self::Raw => vec!["raw".to_string()],
            Self::SingBoxUri => vec!["sb".to_string(), "uri".to_string()],
        }
    }
}

type WorkerResult = Result<String, String>;

pub struct HpwnrApp {
    section: Section,
    settings: Settings,
    nav_generation: u64,

    decrypt_input: String,

    encrypt_url: String,
    encrypt_family: EncryptFamily,
    happ_format: String,
    v2_format: String,
    v2_key: String,

    fetch_url: String,
    fetch_hwid: String,
    fetch_ua_preset: UaPreset,
    fetch_ua_custom: String,

    convert_input_mode: ConvertInputMode,
    convert_paste_text: String,
    convert_file_path: String,
    convert_url: String,
    convert_hwid: String,
    convert_ua_preset: UaPreset,
    convert_ua_custom: String,
    convert_mode: ConvertMode,

    output_text: String,
    status_message: String,
    status_is_error: bool,
    run_state: RunState,
    run_started: f64,
    last_duration: Option<f64>,

    worker_rx: Option<Receiver<WorkerResult>>,

    palette: PaletteState,
    toasts: ToastStack,
    modal: ModalKind,
    applied_theme_key: Option<(bool, usize, i32)>,
}

impl HpwnrApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        let section = Section::from_key(&settings.last_section);
        let app = Self {
            section,
            settings,
            nav_generation: 0,

            decrypt_input: String::new(),

            encrypt_url: String::new(),
            encrypt_family: EncryptFamily::Happ,
            happ_format: "crypt5".to_string(),
            v2_format: "v2r".to_string(),
            v2_key: "crypt4".to_string(),

            fetch_url: String::new(),
            fetch_hwid: String::new(),
            fetch_ua_preset: UaPreset::Happ,
            fetch_ua_custom: String::new(),

            convert_input_mode: ConvertInputMode::Paste,
            convert_paste_text: String::new(),
            convert_file_path: String::new(),
            convert_url: String::new(),
            convert_hwid: String::new(),
            convert_ua_preset: UaPreset::Happ,
            convert_ua_custom: String::new(),
            convert_mode: ConvertMode::SingBox,

            output_text: String::new(),
            status_message: String::new(),
            status_is_error: false,
            run_state: RunState::Idle,
            run_started: 0.0,
            last_duration: None,

            worker_rx: None,

            palette: PaletteState::new(),
            toasts: ToastStack::new(),
            modal: ModalKind::None,
            applied_theme_key: None,
        };
        app.current_theme().apply(&cc.egui_ctx, app.settings.font_scale);
        app
    }

    fn current_theme(&self) -> Theme {
        Theme::with_mode(
            self.settings.dark_mode,
            Accent::from_index(self.settings.accent_index),
        )
    }

    fn theme_key(&self) -> (bool, usize, i32) {
        (
            self.settings.dark_mode,
            self.settings.accent_index,
            (self.settings.font_scale * 100.0).round() as i32,
        )
    }

    fn view_mode(&self) -> ViewMode {
        if self.settings.compact_view {
            ViewMode::Compact
        } else {
            ViewMode::Comfortable
        }
    }

    fn is_running(&self) -> bool {
        self.worker_rx.is_some()
    }

    fn navigate(&mut self, section: Section) {
        if self.section != section {
            self.section = section;
            self.nav_generation += 1;
            self.settings.last_section = section.key().to_string();
        }
    }

    fn toast(&mut self, now: f64, kind: ToastKind, message: impl Into<String>) {
        self.toasts.push(now, kind, message);
    }

    fn build_request(&self) -> Result<CommandRequest, String> {
        match self.section {
            Section::Decrypt => {
                let link = self.decrypt_input.trim();
                if link.is_empty() {
                    return Err("Enter an encrypted link first.".to_string());
                }
                Ok(CommandRequest::Decrypt {
                    link: link.to_string(),
                })
            }
            Section::Encrypt => {
                let url = self.encrypt_url.trim();
                if url.is_empty() {
                    return Err("Enter a subscription URL first.".to_string());
                }
                Ok(match self.encrypt_family {
                    EncryptFamily::Happ => CommandRequest::EncryptHapp {
                        url: url.to_string(),
                        format: self.happ_format.clone(),
                    },
                    EncryptFamily::V2RayTun => CommandRequest::EncryptV2 {
                        url: url.to_string(),
                        format: self.v2_format.clone(),
                        key: if self.v2_key.is_empty() {
                            None
                        } else {
                            Some(self.v2_key.clone())
                        },
                    },
                })
            }
            Section::Fetch => {
                let url = self.fetch_url.trim();
                if url.is_empty() {
                    return Err("Enter a subscription URL first.".to_string());
                }
                let hwid = if self.fetch_hwid.trim().is_empty() {
                    None
                } else {
                    Some(self.fetch_hwid.clone())
                };
                Ok(CommandRequest::Fetch {
                    url: url.to_string(),
                    hwid,
                    ua: self.fetch_ua_preset.as_cli_value(&self.fetch_ua_custom),
                })
            }
            Section::Convert => {
                let modes = self.convert_mode.to_cli_args();
                match self.convert_input_mode {
                    ConvertInputMode::Paste => {
                        let content = self.convert_paste_text.trim();
                        if content.is_empty() {
                            return Err("Paste some subscription content first.".to_string());
                        }
                        Ok(CommandRequest::ConvertStdin {
                            content: content.to_string(),
                            modes,
                        })
                    }
                    ConvertInputMode::File => {
                        let path = self.convert_file_path.trim();
                        if path.is_empty() {
                            return Err("Choose a subscription file first.".to_string());
                        }
                        Ok(CommandRequest::ConvertFile {
                            path: path.to_string(),
                            modes,
                        })
                    }
                    ConvertInputMode::Url => {
                        let url = self.convert_url.trim();
                        if url.is_empty() {
                            return Err("Enter a subscription URL first.".to_string());
                        }
                        let hwid = if self.convert_hwid.trim().is_empty() {
                            None
                        } else {
                            Some(self.convert_hwid.clone())
                        };
                        Ok(CommandRequest::ConvertUrl {
                            url: url.to_string(),
                            hwid,
                            ua: self.convert_ua_preset.as_cli_value(&self.convert_ua_custom),
                            modes,
                        })
                    }
                }
            }
            Section::Settings => Err("There is nothing to run on this page.".to_string()),
        }
    }

    fn can_run(&self) -> bool {
        self.section != Section::Settings && self.build_request().is_ok()
    }

    fn run_current(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.is_running() {
            self.toast(now, ToastKind::Info, "An operation is already running.");
            return;
        }
        match self.build_request() {
            Ok(req) => self.run_command(ctx, req),
            Err(msg) => {
                if self.section != Section::Settings {
                    self.toast(now, ToastKind::Warning, msg);
                }
            }
        }
    }

    fn run_command(&mut self, ctx: &egui::Context, req: CommandRequest) {
        let (tx, rx) = channel();
        let exe = self.settings.exe_path.clone();
        let repaint = ctx.clone();

        self.status_message = "Processing request...".to_string();
        self.status_is_error = false;
        self.run_state = RunState::Running;
        self.run_started = ctx.input(|i| i.time);
        self.worker_rx = Some(rx);

        thread::spawn(move || {
            let res = backend::execute(&exe, req);
            let _ = tx.send(res);
            repaint.request_repaint();
        });
    }

    fn check_worker(&mut self, ctx: &egui::Context) {
        let result = match self.worker_rx {
            Some(ref rx) => match rx.try_recv() {
                Ok(r) => Some(r),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(Err("The background task stopped unexpectedly.".to_string()))
                }
            },
            None => None,
        };
        let Some(result) = result else { return };

        let now = ctx.input(|i| i.time);
        let elapsed = (now - self.run_started).max(0.0);
        self.worker_rx = None;
        self.last_duration = Some(elapsed);
        match result {
            Ok(output) => {
                self.output_text = output;
                self.status_message = "Operation completed successfully.".to_string();
                self.status_is_error = false;
                self.run_state = RunState::Succeeded;
                self.toast(now, ToastKind::Success, format!("Done in {:.2}s", elapsed));
            }
            Err(err) => {
                let first_line = err.lines().next().unwrap_or("Operation failed").to_string();
                self.status_message = err;
                self.status_is_error = true;
                self.run_state = RunState::Failed;
                self.toast(now, ToastKind::Error, first_line);
            }
        }
    }

    fn copy_output(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.output_text.is_empty() {
            self.toast(now, ToastKind::Warning, "There is no output to copy.");
            return;
        }
        ctx.copy_text(self.output_text.clone());
        self.toast(now, ToastKind::Success, "Output copied to clipboard.");
    }

    fn save_output(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if self.output_text.is_empty() {
            self.toast(now, ToastKind::Warning, "There is no output to save.");
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("hpwnr-output.txt")
            .save_file()
        {
            match fs::write(&path, &self.output_text) {
                Ok(_) => {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    self.toast(now, ToastKind::Success, format!("Saved to {}", name));
                }
                Err(e) => {
                    self.toast(now, ToastKind::Error, format!("Failed to save file: {}", e));
                }
            }
        }
    }

    fn clear_output(&mut self, ctx: &egui::Context) {
        if self.is_running() {
            return;
        }
        let now = ctx.input(|i| i.time);
        if self.output_text.is_empty() && self.status_message.is_empty() {
            return;
        }
        self.output_text.clear();
        self.status_message.clear();
        self.status_is_error = false;
        self.run_state = RunState::Idle;
        self.last_duration = None;
        self.toast(now, ToastKind::Info, "Output cleared.");
    }

    fn save_settings(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        match self.settings.save() {
            Ok(_) => self.toast(now, ToastKind::Success, "Preferences saved."),
            Err(e) => self.toast(
                now,
                ToastKind::Error,
                format!("Failed to save settings: {}", e),
            ),
        }
    }

    fn reset_settings(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        self.settings.reset();
        self.section = Section::from_key(&self.settings.last_section);
        self.nav_generation += 1;
        self.toast(now, ToastKind::Info, "Settings reset to defaults.");
    }

    fn handle_action(&mut self, ctx: &egui::Context, action: PaletteAction) {
        match action {
            PaletteAction::Go(section) => self.navigate(section),
            PaletteAction::RunCurrent => self.run_current(ctx),
            PaletteAction::CopyOutput => self.copy_output(ctx),
            PaletteAction::SaveOutput => self.save_output(ctx),
            PaletteAction::ClearOutput => self.clear_output(ctx),
            PaletteAction::ToggleTheme => self.settings.dark_mode = !self.settings.dark_mode,
            PaletteAction::ToggleSidebar => {
                self.settings.sidebar_collapsed = !self.settings.sidebar_collapsed
            }
            PaletteAction::OpenResult => {
                if self.output_text.is_empty() {
                    let now = ctx.input(|i| i.time);
                    self.toast(now, ToastKind::Warning, "There is no result to open yet.");
                } else {
                    self.modal = ModalKind::ResultDetail;
                }
            }
            PaletteAction::ShowShortcuts => self.modal = ModalKind::Shortcuts,
            PaletteAction::ToggleViewMode => {
                self.settings.compact_view = !self.settings.compact_view
            }
            PaletteAction::CycleAccent => {
                self.settings.accent_index =
                    (self.settings.accent_index + 1) % Accent::ALL.len();
            }
        }
    }

    fn read_shortcuts(&mut self, ctx: &egui::Context) -> Vec<PaletteAction> {
        let mut actions = Vec::new();

        let open_palette = ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::K));
        if open_palette {
            if self.palette.open {
                self.palette.close();
            } else {
                self.modal = ModalKind::None;
                self.palette.open();
            }
            return actions;
        }
        if self.palette.open || self.modal != ModalKind::None {
            return actions;
        }

        if let Some(section) = section_for_shortcut(ctx) {
            actions.push(PaletteAction::Go(section));
        }
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::COMMAND, Key::Enter) {
                actions.push(PaletteAction::RunCurrent);
            }
            if i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::C) {
                actions.push(PaletteAction::CopyOutput);
            }
            if i.consume_key(Modifiers::COMMAND, Key::S) {
                actions.push(PaletteAction::SaveOutput);
            }
            if i.consume_key(Modifiers::COMMAND, Key::L) {
                actions.push(PaletteAction::ClearOutput);
            }
            if i.consume_key(Modifiers::COMMAND, Key::R) {
                actions.push(PaletteAction::OpenResult);
            }
            if i.consume_key(Modifiers::COMMAND, Key::T) {
                actions.push(PaletteAction::ToggleTheme);
            }
            if i.consume_key(Modifiers::COMMAND, Key::B) {
                actions.push(PaletteAction::ToggleSidebar);
            }
            if i.consume_key(Modifiers::NONE, Key::F1) {
                actions.push(PaletteAction::ShowShortcuts);
            }
        });
        actions
    }

    fn page_decrypt(&mut self, ui: &mut egui::Ui, cx: Ctx) {
        form_card(ui, cx, "decrypt_card", "Encrypted link", |ui| {
            let (validation, message) = link_validation(&self.decrypt_input);
            text_input(
                ui,
                cx,
                "decrypt_input",
                &mut self.decrypt_input,
                InputOptions::new(
                    "Paste a Happ or V2RayTun link",
                    "The encrypted subscription link to decrypt",
                )
                .monospace()
                .validation(validation, message)
                .hint("Happ and V2RayTun encrypted links are supported.")
                .max_width(FIELD_WIDTH_LONG),
            );
        });
    }

    fn page_encrypt(&mut self, ui: &mut egui::Ui, cx: Ctx) {
        let gap = spacing_for(self.view_mode());

        form_card(ui, cx, "encrypt_url_card", "HTTPS subscription URL", |ui| {
            let (validation, message) = https_validation(&self.encrypt_url);
            text_input(
                ui,
                cx,
                "encrypt_url",
                &mut self.encrypt_url,
                InputOptions::new(
                    "https://example.com/sub/...",
                    "The subscription URL to encrypt",
                )
                .monospace()
                .validation(validation, message)
                .hint("Only HTTPS subscription URLs can be wrapped.")
                .max_width(FIELD_WIDTH_LONG),
            );
        });
        ui.add_space(gap);

        form_card(ui, cx, "encrypt_format_card", "Output format", |ui| {
            field_label(ui, cx, "Family");
            segmented(
                ui,
                cx,
                "encrypt_family",
                &mut self.encrypt_family,
                &[
                    (EncryptFamily::Happ, "Happ", None),
                    (EncryptFamily::V2RayTun, "V2RayTun", None),
                ],
            );
            ui.add_space(SPACE_8);

            match self.encrypt_family {
                EncryptFamily::Happ => {
                    field_label(ui, cx, "Format");
                    let options = string_options(&[
                        "crypt",
                        "crypt2",
                        "crypt3",
                        "crypt4",
                        "crypt5",
                        "crypt5old",
                        "crypt5legacy",
                    ]);
                    dropdown(
                        ui,
                        cx,
                        "happ_format",
                        &mut self.happ_format,
                        &options,
                        220.0,
                        "Choose the Happ encryption format",
                    );
                }
                EncryptFamily::V2RayTun => {
                    field_label(ui, cx, "Format");
                    let formats = string_options(&["v2", "v2r", "v2ray", "v2raytun"]);
                    dropdown(
                        ui,
                        cx,
                        "v2_format",
                        &mut self.v2_format,
                        &formats,
                        220.0,
                        "Choose the V2RayTun format",
                    );
                    ui.add_space(SPACE_8);
                    field_label(ui, cx, "Key");
                    let keys: Vec<(String, String)> = ["", "crypt3", "crypt4", "key3"]
                        .iter()
                        .map(|k| {
                            let label = if k.is_empty() { "Default" } else { *k };
                            (k.to_string(), label.to_string())
                        })
                        .collect();
                    dropdown(
                        ui,
                        cx,
                        "v2_key",
                        &mut self.v2_key,
                        &keys,
                        220.0,
                        "Choose the V2RayTun key",
                    );
                }
            }
        });
    }

    fn page_fetch(&mut self, ui: &mut egui::Ui, cx: Ctx) {
        let gap = spacing_for(self.view_mode());

        form_card(ui, cx, "fetch_url_card", "Subscription URL", |ui| {
            let (validation, message) = https_validation(&self.fetch_url);
            text_input(
                ui,
                cx,
                "fetch_url",
                &mut self.fetch_url,
                InputOptions::new(
                    "https://example.com/sub/...",
                    "The endpoint to download the subscription from",
                )
                .monospace()
                .validation(validation, message)
                .max_width(FIELD_WIDTH_LONG),
            );
        });
        ui.add_space(gap);

        form_card(ui, cx, "fetch_opts_card", "Request options", |ui| {
            request_options(
                ui,
                cx,
                "fetch",
                &mut self.fetch_hwid,
                &mut self.fetch_ua_preset,
                &mut self.fetch_ua_custom,
            );
        });
    }

    fn page_convert(&mut self, ui: &mut egui::Ui, cx: Ctx) {
        let gap = spacing_for(self.view_mode());
        let compact = self.view_mode() == ViewMode::Compact;

        form_card(ui, cx, "convert_source_card", "Source", |ui| {
            segmented(
                ui,
                cx,
                "convert_input_mode",
                &mut self.convert_input_mode,
                &[
                    (ConvertInputMode::Paste, "Paste text", None),
                    (ConvertInputMode::File, "Local file", Some(Icon::Folder)),
                    (ConvertInputMode::Url, "URL", Some(Icon::Download)),
                ],
            );
            ui.add_space(SPACE_12);

            match self.convert_input_mode {
                ConvertInputMode::Paste => {
                    multiline_input(
                        ui,
                        cx,
                        "convert_paste",
                        &mut self.convert_paste_text,
                        "Paste subscription content here...",
                        if compact { 1 } else { 2 },
                        "Raw subscription content to convert",
                    );
                }
                ConvertInputMode::File => {
                    text_input(
                        ui,
                        cx,
                        "convert_file",
                        &mut self.convert_file_path,
                        InputOptions::new("Path to a subscription file", "Local file to convert")
                            .monospace()
                            .max_width(FIELD_WIDTH_LONG),
                    );
                    ui.add_space(SPACE_8);
                    if button(
                        ui,
                        cx,
                        ButtonKind::Secondary,
                        "Browse...",
                        Some(Icon::Folder),
                        true,
                        "Pick a file from disk",
                    )
                    .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.convert_file_path = path.display().to_string();
                        }
                    }
                }
                ConvertInputMode::Url => {
                    let (validation, message) = https_validation(&self.convert_url);
                    text_input(
                        ui,
                        cx,
                        "convert_url",
                        &mut self.convert_url,
                        InputOptions::new(
                            "https://example.com/sub/...",
                            "The endpoint to download and convert",
                        )
                        .monospace()
                        .validation(validation, message)
                        .max_width(FIELD_WIDTH_LONG),
                    );
                    ui.add_space(SPACE_12);
                    request_options(
                        ui,
                        cx,
                        "convert",
                        &mut self.convert_hwid,
                        &mut self.convert_ua_preset,
                        &mut self.convert_ua_custom,
                    );
                }
            }
        });
        ui.add_space(gap);

        form_card(ui, cx, "convert_target_card", "Conversion target", |ui| {
            let options = vec![
                (ConvertMode::Uri, "URI links".to_string()),
                (ConvertMode::SingBox, "sing-box".to_string()),
                (ConvertMode::Base64, "Base64".to_string()),
                (ConvertMode::Raw, "Raw response".to_string()),
                (ConvertMode::SingBoxUri, "sing-box + URI".to_string()),
            ];
            dropdown(
                ui,
                cx,
                "convert_mode",
                &mut self.convert_mode,
                &options,
                240.0,
                "What to convert the subscription into",
            );
        });
    }

    fn page_settings(&mut self, ui: &mut egui::Ui, cx: Ctx) {
        let gap = spacing_for(self.view_mode());
        let p = cx.theme.palette;

        form_card(ui, cx, "settings_backend_card", "hpwnr executable", |ui| {
            let path = self.settings.exe_path.trim().to_string();
            let has_separator = path.contains('/') || path.contains('\\');
            let (validation, message) = if path.is_empty() {
                (Validation::Invalid, Some("The path cannot be empty."))
            } else if has_separator {
                if std::path::Path::new(&path).is_file() {
                    (Validation::Valid, Some("File found."))
                } else {
                    (Validation::Invalid, Some("No file exists at this path."))
                }
            } else {
                (Validation::None, None)
            };
            text_input(
                ui,
                cx,
                "settings_exe",
                &mut self.settings.exe_path,
                InputOptions::new("hpwnr", "Path to the hpwnr command-line tool")
                    .monospace()
                    .validation(validation, message)
                    .hint("A bare name such as \"hpwnr\" is looked up on your PATH.")
                    .max_width(FIELD_WIDTH_LONG),
            );
            ui.add_space(SPACE_8);
            if button(
                ui,
                cx,
                ButtonKind::Secondary,
                "Browse...",
                Some(Icon::Folder),
                true,
                "Pick the hpwnr executable",
            )
            .clicked()
            {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.settings.exe_path = path.display().to_string();
                }
            }
        });
        ui.add_space(gap);

        form_card(ui, cx, "settings_appearance_card", "Appearance", |ui| {
            toggle_switch(
                ui,
                cx,
                "set_dark",
                &mut self.settings.dark_mode,
                "Dark theme",
                Some("Easier on the eyes in low light."),
                true,
            );
            ui.add_space(SPACE_12);

            field_label(ui, cx, "Accent color");
            ui.horizontal_wrapped(|ui| {
                for accent in Accent::ALL {
                    let selected = self.settings.accent_index == accent.index();
                    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
                    let response = response.on_hover_text(accent.label());
                    let base = accent.base(self.settings.dark_mode);
                    ui.painter().circle_filled(rect.center(), 12.0, base);
                    if selected {
                        ui.painter().circle_stroke(
                            rect.center(),
                            15.0,
                            Stroke::new(2.0_f32, p.text_primary),
                        );
                        icons::paint(
                            ui.painter(),
                            egui::Rect::from_center_size(rect.center(), Vec2::splat(14.0)),
                            Icon::Check,
                            crate::ui::theme::readable_on(base),
                            1.8,
                        );
                    } else if response.hovered() {
                        ui.painter().circle_stroke(
                            rect.center(),
                            14.0,
                            Stroke::new(1.5_f32, p.text_muted),
                        );
                    }
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if response.clicked() {
                        self.settings.accent_index = accent.index();
                    }
                }
            });
            ui.add_space(SPACE_12);

            slider(
                ui,
                cx,
                "set_font_scale",
                &mut self.settings.font_scale,
                0.8..=1.6,
                0.05,
                "Text size",
                |v| format!("{}%", (v * 100.0).round() as i32),
            );
        });
        ui.add_space(gap);

        form_card(ui, cx, "settings_behavior_card", "Layout and motion", |ui| {
            toggle_switch(
                ui,
                cx,
                "set_compact",
                &mut self.settings.compact_view,
                "Compact spacing",
                Some("Tighter cards and smaller result panel."),
                true,
            );
            ui.add_space(SPACE_8);
            toggle_switch(
                ui,
                cx,
                "set_sidebar",
                &mut self.settings.sidebar_collapsed,
                "Collapse sidebar",
                Some("Show only icons in the navigation."),
                true,
            );
            ui.add_space(SPACE_8);
            toggle_switch(
                ui,
                cx,
                "set_motion",
                &mut self.settings.reduce_motion,
                "Reduce motion",
                Some("Skip transitions and animated highlights."),
                true,
            );
        });
        ui.add_space(gap);

        ui.horizontal(|ui| {
            if button(
                ui,
                cx,
                ButtonKind::Primary,
                "Save preferences",
                Some(Icon::Save),
                true,
                "Write these settings to disk",
            )
            .clicked()
            {
                let ctx = ui.ctx().clone();
                self.save_settings(&ctx);
            }
            if button(
                ui,
                cx,
                ButtonKind::Danger,
                "Reset to defaults",
                Some(Icon::Reset),
                true,
                "Restore every setting to its default",
            )
            .clicked()
            {
                self.modal = ModalKind::ConfirmReset;
            }
        });
        ui.add_space(SPACE_8);
        hint_text(
            ui,
            cx,
            "Changes apply immediately. Save to keep them next time.",
        );
    }
}

fn string_options(values: &[&str]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|v| (v.to_string(), v.to_string()))
        .collect()
}

fn ua_options() -> Vec<(UaPreset, String)> {
    vec![
        (UaPreset::Happ, "Happ".to_string()),
        (UaPreset::Incy, "INCY".to_string()),
        (UaPreset::V2RayTun, "V2RayTun".to_string()),
        (UaPreset::Custom, "Custom".to_string()),
    ]
}

fn https_validation(value: &str) -> (Validation, Option<&'static str>) {
    let v = value.trim();
    if v.is_empty() {
        (Validation::None, None)
    } else if v.len() > "https://".len() && v.to_lowercase().starts_with("https://") {
        (Validation::Valid, Some("HTTPS URL."))
    } else {
        (Validation::Invalid, Some("The URL should start with https://"))
    }
}

fn link_validation(value: &str) -> (Validation, Option<&'static str>) {
    let v = value.trim();
    if v.is_empty() {
        (Validation::None, None)
    } else if v.contains("://") {
        (Validation::Valid, Some("Link format recognised."))
    } else {
        (
            Validation::Invalid,
            Some("Links normally look like scheme://..."),
        )
    }
}

fn request_options(
    ui: &mut egui::Ui,
    cx: Ctx,
    id: &str,
    hwid: &mut String,
    preset: &mut UaPreset,
    custom: &mut String,
) {
    field_label(ui, cx, "HWID (optional)");
    text_input(
        ui,
        cx,
        format!("{}_hwid", id),
        hwid,
        InputOptions::new("Device identifier", "Sent as the HWID header when set")
            .monospace()
            .max_width(FIELD_WIDTH_SHORT),
    );
    ui.add_space(SPACE_12);

    field_label(ui, cx, "User-Agent");
    dropdown(
        ui,
        cx,
        format!("{}_ua", id),
        preset,
        &ua_options(),
        220.0,
        "Pretend to be this client when requesting the subscription",
    );
    if *preset == UaPreset::Custom {
        ui.add_space(SPACE_8);
        text_input(
            ui,
            cx,
            format!("{}_ua_custom", id),
            custom,
            InputOptions::new("Custom User-Agent string", "Sent as the User-Agent header")
                .monospace()
                .hint("Leave empty to use the tool's default.")
                .max_width(FIELD_WIDTH_MEDIUM),
        );
    }
}

impl eframe::App for HpwnrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let key = self.theme_key();
        if self.applied_theme_key != Some(key) {
            self.current_theme().apply(ctx, self.settings.font_scale);
            self.applied_theme_key = Some(key);
        }

        self.check_worker(ctx);

        let theme = self.current_theme();
        let cx = Ctx {
            theme: &theme,
            reduce_motion: self.settings.reduce_motion,
        };
        let p = theme.palette;
        let screen = ctx.screen_rect();
        self.settings.window_width = screen.width();
        self.settings.window_height = screen.height();

        let mut actions = self.read_shortcuts(ctx);
        let blocking_overlay = self.palette.open || self.modal != ModalKind::None;

        let collapsed = self.settings.sidebar_collapsed || screen.width() < AUTO_COLLAPSE_WIDTH;
        let nav_width = sidebar_width(ctx, cx, collapsed);
        egui::SidePanel::left("navigation_sidebar")
            .resizable(false)
            .exact_width(nav_width)
            .frame(egui::Frame::none().fill(p.background))
            .show(ctx, |ui| {
                ui.set_enabled(!blocking_overlay);
                let out = sidebar(ui, cx, self.section, collapsed, self.is_running());
                if let Some(section) = out.navigate {
                    actions.push(PaletteAction::Go(section));
                }
                if out.toggle_collapse {
                    actions.push(PaletteAction::ToggleSidebar);
                }
                if out.open_palette {
                    self.palette.open();
                }
            });

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(30.0)
            .frame(egui::Frame::none().fill(p.surface))
            .show(ctx, |ui| {
                ui.set_enabled(!blocking_overlay);
                status_bar(ui, cx, &self.settings.exe_path, self.run_state, self.section);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.background))
            .show(ctx, |ui| {
                ui.set_enabled(!blocking_overlay);
                let avail = ui.available_width();
                let view_mode = self.view_mode();
                let margin = page_margin(avail, view_mode);
                let content_w = (avail - margin * 2.0).clamp(240.0, CONTENT_MAX_WIDTH);
                let left = ((avail - content_w) * 0.5).max(0.0);
                let gap = spacing_for(view_mode);
                let progress = page_transition(ctx, cx, self.nav_generation);

                egui::ScrollArea::vertical()
                    .id_source("page_scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.add_space(left);
                            ui.vertical(|ui| {
                                ui.set_width(content_w);
                                ui.add_space(margin * 0.75 + (1.0 - progress) * 10.0);

                                let show_run = self.section != Section::Settings;
                                let run_label = match self.section {
                                    Section::Decrypt => "Decrypt",
                                    Section::Encrypt => "Encrypt",
                                    Section::Fetch => "Fetch",
                                    Section::Convert => "Convert",
                                    Section::Settings => "Run",
                                };
                                let header = page_header(
                                    ui,
                                    cx,
                                    self.section,
                                    self.run_state,
                                    view_mode,
                                    show_run,
                                    self.can_run(),
                                    run_label,
                                );
                                if header.run {
                                    actions.push(PaletteAction::RunCurrent);
                                }
                                if header.toggle_theme {
                                    actions.push(PaletteAction::ToggleTheme);
                                }
                                if header.toggle_view_mode {
                                    actions.push(PaletteAction::ToggleViewMode);
                                }
                                ui.add_space(gap);

                                match self.section {
                                    Section::Decrypt => self.page_decrypt(ui, cx),
                                    Section::Encrypt => self.page_encrypt(ui, cx),
                                    Section::Fetch => self.page_fetch(ui, cx),
                                    Section::Convert => self.page_convert(ui, cx),
                                    Section::Settings => self.page_settings(ui, cx),
                                }

                                if self.section != Section::Settings {
                                    ui.add_space(gap);
                                    let out = output_panel(
                                        ui,
                                        cx,
                                        OutputView {
                                            text: &mut self.output_text,
                                            run_state: self.run_state,
                                            status: &self.status_message,
                                            status_is_error: self.status_is_error,
                                            view_mode,
                                            last_duration: self.last_duration,
                                        },
                                    );
                                    if out.copy {
                                        actions.push(PaletteAction::CopyOutput);
                                    }
                                    if out.save {
                                        actions.push(PaletteAction::SaveOutput);
                                    }
                                    if out.clear {
                                        actions.push(PaletteAction::ClearOutput);
                                    }
                                    if out.expand {
                                        actions.push(PaletteAction::OpenResult);
                                    }
                                }
                                ui.add_space(SPACE_16 + SPACE_12);
                            });
                        });
                    });
            });

        let palette_out = show_palette(ctx, cx, &mut self.palette);
        if let Some(action) = palette_out.action {
            self.palette.close();
            actions.push(action);
        } else if palette_out.closed {
            self.palette.close();
        }

        let (frame, confirm) = modal(
            ctx,
            cx,
            "confirm_reset",
            self.modal == ModalKind::ConfirmReset,
            440.0,
            modal_kind_title(ModalKind::ConfirmReset),
            |ui| {
                confirm_body(
                    ui,
                    cx,
                    "Every preference, including the executable path, theme and layout, goes back to its default value.",
                    "Reset settings",
                )
            },
        );
        if self.modal == ModalKind::ConfirmReset {
            let confirmed = confirm.as_ref().map(|c| c.confirmed).unwrap_or(false);
            let cancelled = confirm.as_ref().map(|c| c.cancelled).unwrap_or(false);
            if confirmed {
                self.modal = ModalKind::None;
                self.reset_settings(ctx);
            } else if cancelled || frame.close_requested {
                self.modal = ModalKind::None;
            }
        }

        let (frame, _) = modal(
            ctx,
            cx,
            "shortcuts",
            self.modal == ModalKind::Shortcuts,
            460.0,
            modal_kind_title(ModalKind::Shortcuts),
            |ui| shortcuts_body(ui, cx),
        );
        if self.modal == ModalKind::Shortcuts && frame.close_requested {
            self.modal = ModalKind::None;
        }

        let mut detail_copy = false;
        let mut detail_save = false;
        let result_text = self.output_text.clone();
        let (frame, _) = modal(
            ctx,
            cx,
            "result_detail",
            self.modal == ModalKind::ResultDetail,
            720.0,
            modal_kind_title(ModalKind::ResultDetail),
            |ui| {
                let mut view: &str = &result_text;
                egui::ScrollArea::vertical()
                    .id_source("result_detail_scroll")
                    .max_height((screen.height() - 240.0).max(160.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut view)
                                .font(theme.mono((theme.type_scale.body - 0.5) * ui_scale(ui)))
                                .desired_width(f32::INFINITY)
                                .desired_rows(14),
                        );
                    });
                ui.add_space(SPACE_12);
                ui.horizontal(|ui| {
                    if button(
                        ui,
                        cx,
                        ButtonKind::Primary,
                        "Copy",
                        Some(Icon::Copy),
                        true,
                        "Copy the full result",
                    )
                    .clicked()
                    {
                        detail_copy = true;
                    }
                    if button(
                        ui,
                        cx,
                        ButtonKind::Secondary,
                        "Save to file",
                        Some(Icon::Save),
                        true,
                        "Write the full result to disk",
                    )
                    .clicked()
                    {
                        detail_save = true;
                    }
                });
            },
        );
        if self.modal == ModalKind::ResultDetail && frame.close_requested {
            self.modal = ModalKind::None;
        }
        if detail_copy {
            actions.push(PaletteAction::CopyOutput);
        }
        if detail_save {
            actions.push(PaletteAction::SaveOutput);
        }

        show_toasts(ctx, cx, &mut self.toasts);

        for action in actions {
            self.handle_action(ctx, action);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.settings.save();
    }
}
