
use crate::ui::components::{
    button, card, chip, field_label, hint_text, icon_button, kbd, separator, skeleton_block,
    status_chip, ui_scale, ButtonKind, Ctx,
};
use crate::ui::icons::{self, Icon};
use crate::ui::motion;
use crate::ui::state::{RunState, Section, ViewMode};
use crate::ui::theme::{
    blend, with_alpha, ICON_SIZE, RADIUS_MEDIUM, SPACE_12, SPACE_16, SPACE_24,
    SPACE_4, SPACE_8,
};
use eframe::egui::{self, Align2, Color32, CursorIcon, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

pub const SIDEBAR_EXPANDED: f32 = 216.0;
pub const SIDEBAR_COLLAPSED: f32 = 64.0;
pub const AUTO_COLLAPSE_WIDTH: f32 = 720.0;

#[derive(Default)]
pub struct SidebarOutcome {
    pub navigate: Option<Section>,
    pub toggle_collapse: bool,
    pub open_palette: bool,
}

pub fn sidebar_width(ctx: &egui::Context, cx: Ctx, collapsed: bool) -> f32 {
    let t = motion::animate_bool(
        ctx,
        egui::Id::new("sidebar_collapse"),
        collapsed,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );
    motion::lerp(SIDEBAR_EXPANDED, SIDEBAR_COLLAPSED, t)
}

pub fn sidebar(
    ui: &mut Ui,
    cx: Ctx,
    current: Section,
    collapsed: bool,
    running: bool,
) -> SidebarOutcome {
    let p = &cx.theme.palette;
    let mut out = SidebarOutcome::default();
    let compact = collapsed;

    ui.add_space(SPACE_12);

    ui.horizontal(|ui| {
        ui.add_space(SPACE_8);
        let (mark, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        ui.painter().rect(
            mark,
            Rounding::same(RADIUS_MEDIUM),
            p.accent,
            Stroke::NONE,
        );
        icons::paint(
            ui.painter(),
            Rect::from_center_size(mark.center(), Vec2::splat(18.0)),
            Icon::Lock,
            p.on_accent,
            1.8,
        );
        if !compact {
            ui.add_space(SPACE_4);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(
                    egui::RichText::new("hpwnr")
                        .font(cx.theme.font(cx.theme.type_scale.title * ui_scale(ui)))
                        .strong()
                        .color(p.text_primary),
                );
                ui.label(
                    egui::RichText::new("GUI")
                        .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                        .color(p.text_muted),
                );
            });
        }
    });

    ui.add_space(SPACE_16);

    if palette_launcher(ui, cx, compact).clicked() {
        out.open_palette = true;
    }

    ui.add_space(SPACE_12);
    if !compact {
        ui.horizontal(|ui| {
            ui.add_space(SPACE_12);
            ui.label(
                egui::RichText::new("WORKSPACE")
                    .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                    .strong()
                    .color(p.text_muted),
            );
        });
        ui.add_space(SPACE_4);
    }

    for section in Section::ALL {
        if section == Section::Settings {
            continue;
        }
        if nav_item(ui, cx, section, current == section, compact, running).clicked() {
            out.navigate = Some(section);
        }
    }

    let footer_height = 36.0 * 2.0 + SPACE_8 * 3.0;
    let spare = (ui.available_height() - footer_height).max(SPACE_8);
    ui.add_space(spare);

    separator(ui, cx);
    ui.add_space(SPACE_8);
    if nav_item(
        ui,
        cx,
        Section::Settings,
        current == Section::Settings,
        compact,
        false,
    )
    .clicked()
    {
        out.navigate = Some(Section::Settings);
    }

    let (label, icon) = if compact {
        ("Expand sidebar", Icon::ChevronRight)
    } else {
        ("Collapse sidebar", Icon::Menu)
    };
    ui.horizontal(|ui| {
        ui.add_space(SPACE_8);
        if icon_button(ui, cx, icon, label, false, true).clicked() {
            out.toggle_collapse = true;
        }
        if !compact {
            hint_text(ui, cx, "Ctrl+B");
        }
    });
    ui.add_space(SPACE_8);

    out
}

fn palette_launcher(ui: &mut Ui, cx: Ctx, compact: bool) -> egui::Response {
    let p = &cx.theme.palette;
    let width = ui.available_width() - SPACE_16;
    let size = if compact {
        Vec2::splat(40.0)
    } else {
        Vec2::new(width.max(40.0), 38.0)
    };
    let row = ui.horizontal(|ui| {
        ui.add_space(if compact { (ui.available_width() - 40.0) * 0.5 } else { SPACE_8 });
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let response = response.on_hover_text("Search commands (Ctrl+K)");
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, "Search commands")
        });
        let hover_t = motion::animate_bool(
            ui.ctx(),
            response.id.with("hover"),
            response.hovered(),
            cx.theme.motion.fast,
            cx.reduce_motion,
        );
        ui.painter().rect(
            rect,
            Rounding::same(RADIUS_MEDIUM),
            blend(p.surface_variant, p.accent, 0.10 * hover_t),
            Stroke::new(1.0_f32, blend(p.border, p.accent, 0.6 * hover_t)),
        );
        let icon_rect = Rect::from_min_size(
            Pos2::new(
                if compact { rect.center().x - 9.0 } else { rect.min.x + SPACE_12 },
                rect.center().y - 9.0,
            ),
            Vec2::splat(18.0),
        );
        icons::paint(ui.painter(), icon_rect, Icon::Search, p.text_secondary, 1.6);
        if !compact {
            ui.painter().text(
                Pos2::new(icon_rect.max.x + SPACE_8, rect.center().y),
                Align2::LEFT_CENTER,
                "Search",
                cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)),
                p.text_secondary,
            );
            let badge = "Ctrl K";
            ui.painter().text(
                Pos2::new(rect.max.x - SPACE_12, rect.center().y),
                Align2::RIGHT_CENTER,
                badge,
                cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)),
                p.text_muted,
            );
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        response
    });
    row.inner
}

fn nav_item(
    ui: &mut Ui,
    cx: Ctx,
    section: Section,
    selected: bool,
    compact: bool,
    busy: bool,
) -> egui::Response {
    let p = &cx.theme.palette;
    let full = ui.available_width() - SPACE_16;
    let height = 40.0;
    let size = if compact {
        Vec2::splat(height)
    } else {
        Vec2::new(full.max(40.0), height)
    };

    let inner = ui.horizontal(|ui| {
        ui.add_space(if compact { (ui.available_width() - height) * 0.5 } else { SPACE_8 });
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let tip = format!("{} ({})", section.title(), section.shortcut());
        let response = response.on_hover_text(tip);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, selected, section.title())
        });

        let id = response.id;
        let hover_t = motion::animate_bool(
            ui.ctx(),
            id.with("hover"),
            response.hovered(),
            cx.theme.motion.fast,
            cx.reduce_motion,
        );
        let sel_t = motion::animate_bool(
            ui.ctx(),
            id.with("sel"),
            selected,
            cx.theme.motion.normal,
            cx.reduce_motion,
        );

        let bg = blend(
            blend(p.background, p.accent, 0.10 * hover_t),
            p.accent_soft,
            sel_t,
        );
        ui.painter()
            .rect_filled(rect, Rounding::same(RADIUS_MEDIUM), bg);

        if sel_t > 0.01 {
            let bar_h = 18.0 * sel_t;
            let bar = Rect::from_center_size(
                Pos2::new(rect.min.x + 2.0, rect.center().y),
                Vec2::new(3.0, bar_h),
            );
            ui.painter()
                .rect_filled(bar, Rounding::same(2.0), p.accent);
        }

        let fg = blend(
            blend(p.text_secondary, p.text_primary, hover_t),
            p.accent,
            sel_t,
        );
        let icon_x = if compact { rect.center().x - ICON_SIZE * 0.5 } else { rect.min.x + SPACE_12 };
        let icon_rect = Rect::from_min_size(
            Pos2::new(icon_x, rect.center().y - ICON_SIZE * 0.5),
            Vec2::splat(ICON_SIZE),
        );
        icons::paint(ui.painter(), icon_rect, section.icon(), fg, 1.7);

        if !compact {
            ui.painter().text(
                Pos2::new(icon_rect.max.x + SPACE_12, rect.center().y),
                Align2::LEFT_CENTER,
                section.title(),
                cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)),
                fg,
            );
        }

        if busy && selected {
            let pulse = motion::pulse(ui.ctx(), 1.1, cx.reduce_motion);
            let center = Pos2::new(rect.max.x - SPACE_12, rect.center().y);
            ui.painter().circle_filled(
                center,
                3.0 + pulse,
                with_alpha(p.accent, (120.0 + 100.0 * pulse) as u8),
            );
        }

        if response.has_focus() {
            crate::ui::components::draw_focus_ring(ui, rect, RADIUS_MEDIUM, cx.theme);
        }
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        response
    });
    inner.inner
}

#[derive(Default)]
pub struct HeaderOutcome {
    pub run: bool,
    pub toggle_theme: bool,
    pub toggle_view_mode: bool,
}

pub fn page_header(
    ui: &mut Ui,
    cx: Ctx,
    section: Section,
    run_state: RunState,
    view_mode: ViewMode,
    show_run: bool,
    can_run: bool,
    run_label: &str,
) -> HeaderOutcome {
    let p = &cx.theme.palette;
    let mut out = HeaderOutcome::default();

    ui.horizontal(|ui| {
        let (tile, _) = ui.allocate_exact_size(Vec2::splat(44.0), Sense::hover());
        ui.painter().rect(
            tile,
            Rounding::same(RADIUS_MEDIUM + 2.0),
            p.accent_soft,
            Stroke::new(1.0_f32, with_alpha(p.accent, 90)),
        );
        icons::paint(
            ui.painter(),
            Rect::from_center_size(tile.center(), Vec2::splat(22.0)),
            section.icon(),
            p.accent,
            1.8,
        );

        ui.add_space(SPACE_4);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                egui::RichText::new(section.title())
                    .font(cx.theme.font(cx.theme.type_scale.headline * ui_scale(ui)))
                    .strong()
                    .color(p.text_primary),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(section.subtitle())
                        .font(cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)))
                        .color(p.text_secondary),
                )
                .wrap(true),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if show_run {
                let enabled = can_run && run_state != RunState::Running;
                let tip = if run_state == RunState::Running {
                    "An operation is already running"
                } else if !can_run {
                    "Fill in the required fields first"
                } else {
                    "Run this operation (Ctrl+Enter)"
                };
                if button(ui, cx, ButtonKind::Primary, run_label, Some(Icon::Play), enabled, tip)
                    .clicked()
                {
                    out.run = true;
                }
            }
            let (icon, tip) = if cx.theme.dark {
                (Icon::Sun, "Switch to light theme (Ctrl+T)")
            } else {
                (Icon::Moon, "Switch to dark theme (Ctrl+T)")
            };
            if icon_button(ui, cx, icon, tip, false, true).clicked() {
                out.toggle_theme = true;
            }
            let compact = view_mode == ViewMode::Compact;
            let tip = if compact {
                "Switch to comfortable spacing"
            } else {
                "Switch to compact spacing"
            };
            if icon_button(ui, cx, Icon::Rows, tip, compact, true).clicked() {
                out.toggle_view_mode = true;
            }
        });
    });
    out
}

pub fn page_margin(width: f32, view_mode: ViewMode) -> f32 {
    let base = if view_mode == ViewMode::Compact { SPACE_16 } else { SPACE_24 };
    if width > 1100.0 {
        base + SPACE_16
    } else {
        base
    }
}

pub const CONTENT_MAX_WIDTH: f32 = 880.0;

pub fn page_transition(ctx: &egui::Context, cx: Ctx, generation: u64) -> f32 {
    motion::transition_progress(
        ctx,
        "page",
        generation,
        cx.theme.motion.slow,
        cx.reduce_motion,
    )
}

pub fn spacing_for(view_mode: ViewMode) -> f32 {
    match view_mode {
        ViewMode::Comfortable => SPACE_16,
        ViewMode::Compact => SPACE_8,
    }
}

#[derive(Default)]
pub struct OutputOutcome {
    pub copy: bool,
    pub save: bool,
    pub clear: bool,
    pub expand: bool,
}

pub struct OutputView<'a> {
    pub text: &'a mut String,
    pub run_state: RunState,
    pub status: &'a str,
    pub status_is_error: bool,
    pub view_mode: ViewMode,
    pub last_duration: Option<f64>,
}

pub fn output_panel(ui: &mut Ui, cx: Ctx, view: OutputView) -> OutputOutcome {
    let p = &cx.theme.palette;
    let mut out = OutputOutcome::default();
    let has_text = !view.text.is_empty();
    let running = view.run_state == RunState::Running;
    let rows = if view.view_mode == ViewMode::Compact { 3 } else { 4 };

    card(ui, cx, "output_card", false, false, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Result")
                    .font(cx.theme.font(cx.theme.type_scale.title * ui_scale(ui)))
                    .strong()
                    .color(p.text_primary),
            );
            let (tone, icon) = run_tone(cx, view.run_state);
            status_chip(ui, cx, icon, view.run_state.label(), tone);
            if has_text {
                let lines = view.text.lines().count();
                let chars = view.text.chars().count();
                chip(
                    ui,
                    cx,
                    &format!("{} lines, {} chars", lines, chars),
                    None,
                );
            }
            if let Some(secs) = view.last_duration {
                if !running {
                    chip(ui, cx, &format!("{:.2}s", secs), None);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, cx, Icon::Expand, "Open result details (Ctrl+R)", false, has_text)
                    .clicked()
                {
                    out.expand = true;
                }
                if icon_button(ui, cx, Icon::Trash, "Clear output (Ctrl+L)", false, has_text)
                    .clicked()
                {
                    out.clear = true;
                }
                if icon_button(ui, cx, Icon::Save, "Save output to a file (Ctrl+S)", false, has_text)
                    .clicked()
                {
                    out.save = true;
                }
                if icon_button(
                    ui,
                    cx,
                    Icon::Copy,
                    "Copy output (Ctrl+Shift+C)",
                    false,
                    has_text,
                )
                .clicked()
                {
                    out.copy = true;
                }
            });
        });

        if !view.status.is_empty() {
            ui.add_space(SPACE_4);
            let tone = if view.status_is_error { p.danger } else { p.text_secondary };
            ui.add(
                egui::Label::new(
                    egui::RichText::new(view.status)
                        .font(cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)))
                        .color(tone),
                )
                .wrap(true),
            );
        }

        ui.add_space(SPACE_8);

        if running && !has_text {
            skeleton_block(ui, cx, 5);
        } else if !has_text {
            let hint = "Results appear here. Run an operation, or press Ctrl+K to browse commands.";
            ui.add_space(SPACE_8);
            ui.vertical_centered(|ui| {
                icons::allocate_icon(ui, Icon::Sparkle, 26.0, p.text_muted);
                ui.add_space(SPACE_4);
                ui.label(
                    egui::RichText::new(hint)
                        .font(cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)))
                        .color(p.text_muted),
                );
            });
            ui.add_space(SPACE_8);
        } else {
            egui::ScrollArea::vertical()
                .id_source("output_scroll")
                .max_height(if view.view_mode == ViewMode::Compact { 120.0 } else { 160.0 })
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(view.text)
                            .font(cx.theme.mono((cx.theme.type_scale.body - 0.5) * ui_scale(ui)))
                            .desired_width(f32::INFINITY)
                            .desired_rows(rows)
                            .frame(false)
                            .text_color(p.text_primary),
                    );
                });
        }
    });

    out
}

pub fn run_tone(cx: Ctx, state: RunState) -> (Color32, Icon) {
    let p = &cx.theme.palette;
    match state {
        RunState::Idle => (p.text_secondary, Icon::Info),
        RunState::Running => (p.info, Icon::Play),
        RunState::Succeeded => (p.success, Icon::Check),
        RunState::Failed => (p.danger, Icon::Error),
    }
}

pub fn status_bar(ui: &mut Ui, cx: Ctx, exe_path: &str, run_state: RunState, section: Section) {
    let p = &cx.theme.palette;
    ui.horizontal_centered(|ui| {
        ui.add_space(SPACE_12);
        if run_state == RunState::Running {
            let angle = motion::spinner_angle(ui.ctx(), cx.reduce_motion);
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
            let c = rect.center();
            for i in 0..8 {
                let a = angle + i as f32 * std::f32::consts::TAU / 8.0;
                let alpha = (40 + i * 26).min(255) as u8;
                let from = c + Vec2::angled(a) * 3.5;
                let to = c + Vec2::angled(a) * 6.5;
                ui.painter().line_segment(
                    [from, to],
                    Stroke::new(1.6_f32, with_alpha(p.accent, alpha)),
                );
            }
            hint_text(ui, cx, "Running...");
        } else {
            let (tone, _) = run_tone(cx, run_state);
            let (dot, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
            ui.painter().circle_filled(dot.center(), 3.5, tone);
            hint_text(ui, cx, run_state.label());
        }

        ui.add_space(SPACE_12);
        hint_text(ui, cx, &format!("Backend: {}", exe_path));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(SPACE_12);
            kbd(ui, cx, "F1");
            hint_text(ui, cx, "Shortcuts");
            ui.add_space(SPACE_8);
            kbd(ui, cx, section.shortcut());
        });
    });
}

pub fn form_card<R>(
    ui: &mut Ui,
    cx: Ctx,
    id: &str,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let (_, inner) = card(ui, cx, id, false, false, |ui| {
        field_label(ui, cx, title);
        ui.add_space(SPACE_4);
        add_contents(ui)
    });
    inner
}

