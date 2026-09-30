use crate::ui::icons::Icon;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Section {
    Decrypt,
    Encrypt,
    Fetch,
    Convert,
    Settings,
}

impl Section {
    pub const ALL: [Section; 5] = [
        Section::Decrypt,
        Section::Encrypt,
        Section::Fetch,
        Section::Convert,
        Section::Settings,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Section::Decrypt => "decrypt",
            Section::Encrypt => "encrypt",
            Section::Fetch => "fetch",
            Section::Convert => "convert",
            Section::Settings => "settings",
        }
    }

    pub fn from_key(key: &str) -> Section {
        match key {
            "encrypt" => Section::Encrypt,
            "fetch" => Section::Fetch,
            "convert" => Section::Convert,
            "settings" => Section::Settings,
            _ => Section::Decrypt,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Section::Decrypt => "Decrypt",
            Section::Encrypt => "Encrypt",
            Section::Fetch => "Fetch",
            Section::Convert => "Convert",
            Section::Settings => "Settings",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Section::Decrypt => "Reveal the subscription URL hidden inside a Happ or V2RayTun link.",
            Section::Encrypt => "Wrap an HTTPS subscription URL in a Happ or V2RayTun link.",
            Section::Fetch => "Download a subscription payload straight from its endpoint.",
            Section::Convert => "Turn a subscription into URIs, sing-box, Base64 or the raw response.",
            Section::Settings => "Point the app at hpwnr and tune how it looks.",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Section::Decrypt => Icon::Unlock,
            Section::Encrypt => Icon::Lock,
            Section::Fetch => Icon::Download,
            Section::Convert => Icon::Convert,
            Section::Settings => Icon::Settings,
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            Section::Decrypt => "Ctrl+1",
            Section::Encrypt => "Ctrl+2",
            Section::Fetch => "Ctrl+3",
            Section::Convert => "Ctrl+4",
            Section::Settings => "Ctrl+5",
        }
    }

    pub fn number_key(self) -> eframe::egui::Key {
        match self {
            Section::Decrypt => eframe::egui::Key::Num1,
            Section::Encrypt => eframe::egui::Key::Num2,
            Section::Fetch => eframe::egui::Key::Num3,
            Section::Convert => eframe::egui::Key::Num4,
            Section::Settings => eframe::egui::Key::Num5,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    Comfortable,
    Compact,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
    Info,
}

impl ToastKind {
    pub fn icon(self) -> Icon {
        match self {
            ToastKind::Success => Icon::Check,
            ToastKind::Warning => Icon::Warning,
            ToastKind::Error => Icon::Error,
            ToastKind::Info => Icon::Info,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ToastKind::Success => "Success",
            ToastKind::Warning => "Warning",
            ToastKind::Error => "Error",
            ToastKind::Info => "Info",
        }
    }
}

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
    pub created: f64,
    pub lifetime: f64,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RunState {
    Idle,
    Running,
    Succeeded,
    Failed,
}

impl RunState {
    pub fn label(self) -> &'static str {
        match self {
            RunState::Idle => "Ready",
            RunState::Running => "Running",
            RunState::Succeeded => "Completed",
            RunState::Failed => "Failed",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ModalKind {
    None,
    ConfirmReset,
    ResultDetail,
    Shortcuts,
}

#[derive(Clone)]
pub struct PaletteCommand {
    pub id: PaletteAction,
    pub title: &'static str,
    pub hint: &'static str,
    pub icon: Icon,
    pub shortcut: &'static str,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PaletteAction {
    Go(Section),
    RunCurrent,
    CopyOutput,
    SaveOutput,
    ClearOutput,
    ToggleTheme,
    ToggleSidebar,
    OpenResult,
    ShowShortcuts,
    ToggleViewMode,
    CycleAccent,
}

pub fn all_commands() -> Vec<PaletteCommand> {
    vec![
        PaletteCommand {
            id: PaletteAction::Go(Section::Decrypt),
            title: "Go to Decrypt",
            hint: "Reveal a hidden subscription URL",
            icon: Icon::Unlock,
            shortcut: "Ctrl+1",
        },
        PaletteCommand {
            id: PaletteAction::Go(Section::Encrypt),
            title: "Go to Encrypt",
            hint: "Create a Happ or V2RayTun link",
            icon: Icon::Lock,
            shortcut: "Ctrl+2",
        },
        PaletteCommand {
            id: PaletteAction::Go(Section::Fetch),
            title: "Go to Fetch",
            hint: "Download a subscription payload",
            icon: Icon::Download,
            shortcut: "Ctrl+3",
        },
        PaletteCommand {
            id: PaletteAction::Go(Section::Convert),
            title: "Go to Convert",
            hint: "Convert to URIs, sing-box or Base64",
            icon: Icon::Convert,
            shortcut: "Ctrl+4",
        },
        PaletteCommand {
            id: PaletteAction::Go(Section::Settings),
            title: "Go to Settings",
            hint: "Executable path and appearance",
            icon: Icon::Settings,
            shortcut: "Ctrl+5",
        },
        PaletteCommand {
            id: PaletteAction::RunCurrent,
            title: "Run current operation",
            hint: "Execute the action on this page",
            icon: Icon::Play,
            shortcut: "Ctrl+Enter",
        },
        PaletteCommand {
            id: PaletteAction::CopyOutput,
            title: "Copy output",
            hint: "Copy the result to the clipboard",
            icon: Icon::Copy,
            shortcut: "Ctrl+Shift+C",
        },
        PaletteCommand {
            id: PaletteAction::SaveOutput,
            title: "Save output to file",
            hint: "Write the result to disk",
            icon: Icon::Save,
            shortcut: "Ctrl+S",
        },
        PaletteCommand {
            id: PaletteAction::ClearOutput,
            title: "Clear output",
            hint: "Empty the result panel",
            icon: Icon::Trash,
            shortcut: "Ctrl+L",
        },
        PaletteCommand {
            id: PaletteAction::OpenResult,
            title: "Open result details",
            hint: "Inspect the full output",
            icon: Icon::Expand,
            shortcut: "Ctrl+R",
        },
        PaletteCommand {
            id: PaletteAction::ToggleTheme,
            title: "Toggle light or dark theme",
            hint: "Switch the appearance",
            icon: Icon::Sun,
            shortcut: "Ctrl+T",
        },
        PaletteCommand {
            id: PaletteAction::CycleAccent,
            title: "Cycle accent color",
            hint: "Try the next accent preset",
            icon: Icon::Sparkle,
            shortcut: "",
        },
        PaletteCommand {
            id: PaletteAction::ToggleSidebar,
            title: "Toggle sidebar",
            hint: "Collapse or expand navigation",
            icon: Icon::Menu,
            shortcut: "Ctrl+B",
        },
        PaletteCommand {
            id: PaletteAction::ToggleViewMode,
            title: "Toggle compact view",
            hint: "Tighten or loosen spacing",
            icon: Icon::Rows,
            shortcut: "",
        },
        PaletteCommand {
            id: PaletteAction::ShowShortcuts,
            title: "Show keyboard shortcuts",
            hint: "See every shortcut",
            icon: Icon::Command,
            shortcut: "F1",
        },
    ]
}

pub fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Some(0);
    }
    let target_lower = target.to_lowercase();
    let target_chars: Vec<char> = target_lower.chars().collect();
    let mut score: i32 = 0;
    let mut search_from = 0usize;
    let mut previous: Option<usize> = None;

    for qc in query.chars() {
        if qc.is_whitespace() {
            continue;
        }
        let mut found = None;
        for (offset, tc) in target_chars.iter().enumerate().skip(search_from) {
            if *tc == qc {
                found = Some(offset);
                break;
            }
        }
        let index = found?;
        score += 10;
        if let Some(prev) = previous {
            if index == prev + 1 {
                score += 15;
            }
        }
        if index == 0 {
            score += 12;
        } else if target_chars[index - 1] == ' ' {
            score += 8;
        }
        score -= (index as i32 - search_from as i32).min(6);
        previous = Some(index);
        search_from = index + 1;
    }

    if target_lower.contains(query.as_str()) {
        score += 40;
    }
    Some(score)
}
