use crate::ui::components::{button, icon_button, kbd, ui_scale, ButtonKind, Ctx};
use crate::ui::icons::{self, Icon};
use crate::ui::motion;
use crate::ui::state::{
    all_commands, fuzzy_score, ModalKind, PaletteAction, PaletteCommand, Section, Toast, ToastKind,
};
use crate::ui::theme::{
    blend, with_alpha, Theme, ICON_SIZE, RADIUS_LARGE, RADIUS_MEDIUM, RADIUS_PILL, RADIUS_SMALL,
    SPACE_12, SPACE_16, SPACE_24, SPACE_4, SPACE_8,
};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, Id, Key, Order, Pos2, Rect, Response, Rounding, Sense,
    Stroke, Ui, Vec2,
};

pub struct PaletteState {
    pub open: bool,
    pub query: String,
    pub cursor: usize,
    pub just_opened: bool,
}

impl PaletteState {
    pub fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            cursor: 0,
            just_opened: false,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.cursor = 0;
        self.just_opened = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.cursor = 0;
    }
}

pub struct ToastStack {
    pub items: Vec<Toast>,
    next_id: u64,
}

impl ToastStack {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    pub fn push(&mut self, now: f64, kind: ToastKind, message: impl Into<String>) {
        let lifetime = match kind {
            ToastKind::Error => 7.0,
            ToastKind::Warning => 5.5,
            _ => 3.6,
        };
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(Toast {
            id,
            kind,
            message: message.into(),
            created: now,
            lifetime,
        });
        if self.items.len() > 4 {
            self.items.remove(0);
        }
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    pub fn prune(&mut self, now: f64) {
        self.items.retain(|t| now - t.created < t.lifetime + 0.35);
    }
}

pub fn toast_tone(theme: &Theme, kind: ToastKind) -> Color32 {
    let p = &theme.palette;
    match kind {
        ToastKind::Success => p.success,
        ToastKind::Warning => p.warning,
        ToastKind::Error => p.danger,
        ToastKind::Info => p.info,
    }
}

pub fn show_toasts(ctx: &egui::Context, cx: Ctx, stack: &mut ToastStack) {
    let now = ctx.input(|i| i.time);
    stack.prune(now);
    if stack.items.is_empty() {
        return;
    }

    let screen = ctx.screen_rect();
    let toast_width = 340.0f32.min((screen.width() - SPACE_24 * 2.0).max(220.0));
    let mut dismissed: Vec<u64> = Vec::new();
    let mut y_cursor = screen.max.y - 88.0;
    let items: Vec<Toast> = stack.items.iter().rev().cloned().collect();

    let mut any_animating = false;

    for toast in items {
        let age = (now - toast.created) as f32;
        let remaining = toast.lifetime as f32 - age;
        let enter = if cx.reduce_motion {
            1.0
        } else {
            motion::ease_out_cubic(age / cx.theme.motion.slow)
        };
        let leave = if cx.reduce_motion {
            1.0
        } else {
            1.0 - motion::ease_out_cubic((0.3 - remaining).max(0.0) / 0.3)
        };
        let visibility = (enter * leave).clamp(0.0, 1.0);
        if visibility <= 0.001 {
            continue;
        }
        if enter < 1.0 || remaining < 0.3 {
            any_animating = true;
        }

        let p = cx.theme.palette;
        let tone = toast_tone(cx.theme, toast.kind);
        let slide = (1.0 - enter) * 36.0 + (1.0 - leave) * 24.0;
        let known_height: Option<f32> =
            ctx.memory(|m| m.data.get_temp(Id::new(("toast_height", toast.id))));
        let height = known_height.unwrap_or(72.0);
        let top_left = Pos2::new(
            screen.max.x - toast_width - SPACE_24 + slide,
            y_cursor - height,
        );

        let output = egui::Area::new(Id::new(("toast", toast.id)))
            .order(Order::Tooltip)
            .fixed_pos(top_left)
            .show(ctx, |ui| {
                ui.set_opacity(visibility);
                egui::Frame::none()
                    .fill(blend(p.surface, tone, 0.08))
                    .stroke(Stroke::new(1.0_f32, tone))
                    .rounding(Rounding::same(RADIUS_MEDIUM))
                    .shadow(cx.theme.elevation_shadow(3))
                    .inner_margin(egui::Margin::symmetric(SPACE_12, SPACE_12))
                    .show(ui, |ui| {
                        ui.set_width(toast_width - SPACE_12 * 2.0 - 2.0);
                        ui.horizontal(|ui| {
                            let (icon_rect, _) = ui
                                .allocate_exact_size(Vec2::splat(ICON_SIZE + 4.0), Sense::hover());
                            ui.painter().circle_filled(
                                icon_rect.center(),
                                (ICON_SIZE + 4.0) * 0.5,
                                with_alpha(tone, 40),
                            );
                            icons::paint(
                                ui.painter(),
                                Rect::from_center_size(
                                    icon_rect.center(),
                                    Vec2::splat(ICON_SIZE - 4.0),
                                ),
                                toast.kind.icon(),
                                tone,
                                1.7,
                            );
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(toast.kind.label())
                                        .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                                        .strong()
                                        .color(tone),
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&toast.message)
                                            .font(
                                                cx.theme
                                                    .font(cx.theme.type_scale.label * ui_scale(ui)),
                                            )
                                            .color(p.text_primary),
                                    )
                                    .wrap(true),
                                );
                            });
                        });
                        let progress = (1.0 - age / toast.lifetime as f32).clamp(0.0, 1.0);
                        ui.add_space(SPACE_8);
                        let (bar, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 3.0),
                            Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            bar,
                            Rounding::same(RADIUS_PILL),
                            with_alpha(tone, 40),
                        );
                        let mut filled = bar;
                        filled.set_width(bar.width() * progress);
                        ui.painter()
                            .rect_filled(filled, Rounding::same(RADIUS_PILL), tone);
                    });
            });

        let measured = output.response.rect.height();
        ctx.memory_mut(|m| {
            m.data
                .insert_temp(Id::new(("toast_height", toast.id)), measured)
        });

        let response = output.response.interact(Sense::click());
        if response.hovered() {
            ctx.set_cursor_icon(CursorIcon::PointingHand);
        }
        if response.clicked() {
            dismissed.push(toast.id);
        }
        y_cursor -= measured + SPACE_8;
    }

    if any_animating {
        ctx.request_repaint();
    } else {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }

    for id in dismissed {
        stack.dismiss(id);
    }
}

pub fn backdrop(ctx: &egui::Context, cx: Ctx, id: &str, openness: f32) -> Response {
    let screen = ctx.screen_rect();
    let p = &cx.theme.palette;
    let area = egui::Area::new(Id::new(("backdrop", id)))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(true);
    let out = area.show(ctx, |ui| {
        let (rect, response) = ui.allocate_exact_size(screen.size(), Sense::click_and_drag());
        let base = p.scrim;
        let alpha = (base.a() as f32 * openness) as u8;
        ui.painter().rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha),
        );
        let bands = 6;
        for i in 0..bands {
            let t = i as f32 / bands as f32;
            let inset = 40.0 * t;
            let band = rect.shrink(inset);
            ui.painter().rect_stroke(
                band,
                Rounding::same(RADIUS_LARGE + inset * 0.4),
                Stroke::new(
                    28.0_f32,
                    Color32::from_rgba_unmultiplied(
                        base.r(),
                        base.g(),
                        base.b(),
                        (10.0 * openness * (1.0 - t)) as u8,
                    ),
                ),
            );
        }
        response
    });
    out.inner
}

pub struct ModalFrame {
    pub close_requested: bool,
}

pub fn modal<R>(
    ctx: &egui::Context,
    cx: Ctx,
    id: &str,
    open: bool,
    width: f32,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> (ModalFrame, Option<R>) {
    let p = &cx.theme.palette;
    let openness = motion::animate_bool(
        ctx,
        Id::new(("modal_open", id)),
        open,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );
    let mut frame_out = ModalFrame {
        close_requested: false,
    };
    if openness <= 0.001 {
        return (frame_out, None);
    }

    let screen = ctx.screen_rect();
    let bg = backdrop(ctx, cx, id, openness);
    if open && bg.clicked() {
        frame_out.close_requested = true;
    }

    let scale = 0.94 + 0.06 * openness;
    let effective_width = (width * scale).min(screen.width() - SPACE_24 * 2.0).max(240.0);
    let center = screen.center() + Vec2::new(0.0, (1.0 - openness) * 12.0);
    let mut result: Option<R> = None;

    egui::Area::new(Id::new(("modal", id)))
        .order(Order::Tooltip)
        .fixed_pos(Pos2::new(center.x - effective_width * 0.5, screen.min.y + SPACE_24))
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, (1.0 - openness) * 12.0))
        .show(ctx, |ui| {
            ui.set_opacity(openness);
            egui::Frame::none()
                .fill(p.surface)
                .stroke(Stroke::new(1.0_f32, p.border))
                .rounding(Rounding::same(RADIUS_LARGE))
                .shadow(cx.theme.elevation_shadow(4))
                .inner_margin(egui::Margin::same(SPACE_24))
                .show(ui, |ui| {
                    ui.set_width(effective_width - SPACE_24 * 2.0 - 2.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(title)
                                .font(cx.theme.font(cx.theme.type_scale.title * ui_scale(ui)))
                                .strong()
                                .color(p.text_primary),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if icon_button(ui, cx, Icon::Close, "Close dialog", false, true)
                                .clicked()
                            {
                                frame_out.close_requested = true;
                            }
                        });
                    });
                    ui.add_space(SPACE_12);
                    result = Some(add_contents(ui));
                });
        });

    if open && ctx.input(|i| i.key_pressed(Key::Escape)) {
        frame_out.close_requested = true;
    }
    (frame_out, result)
}

pub struct PaletteOutcome {
    pub action: Option<PaletteAction>,
    pub closed: bool,
}

pub fn filtered_commands(query: &str) -> Vec<PaletteCommand> {
    let mut scored: Vec<(i32, PaletteCommand)> = all_commands()
        .into_iter()
        .filter_map(|c| {
            let title_score = fuzzy_score(query, c.title);
            let hint_score = fuzzy_score(query, c.hint).map(|s| s / 2);
            let best = match (title_score, hint_score) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            best.map(|s| (s, c))
        })
        .collect();
    if !query.trim().is_empty() {
        scored.sort_by(|a, b| b.0.cmp(&a.0));
    }
    scored.into_iter().map(|(_, c)| c).collect()
}

pub fn show_palette(
    ctx: &egui::Context,
    cx: Ctx,
    state: &mut PaletteState,
) -> PaletteOutcome {
    let p = &cx.theme.palette;
    let mut outcome = PaletteOutcome {
        action: None,
        closed: false,
    };

    let input_id = Id::new("palette_input");
    if state.just_opened {
        ctx.memory_mut(|m| m.request_focus(input_id));
    }

    let openness = motion::animate_bool(
        ctx,
        Id::new("palette_open"),
        state.open,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );
    if openness <= 0.001 {
        return outcome;
    }

    let screen = ctx.screen_rect();
    let bg = backdrop(ctx, cx, "palette", openness);

    let commands = filtered_commands(&state.query);
    if state.cursor >= commands.len() {
        state.cursor = commands.len().saturating_sub(1);
    }

    if state.open {
        let (down, up, enter, escape) = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Escape),
            )
        });
        if down && !commands.is_empty() {
            state.cursor = (state.cursor + 1) % commands.len();
        }
        if up && !commands.is_empty() {
            state.cursor = if state.cursor == 0 {
                commands.len() - 1
            } else {
                state.cursor - 1
            };
        }
        if enter {
            if let Some(cmd) = commands.get(state.cursor) {
                outcome.action = Some(cmd.id);
                outcome.closed = true;
            }
        }
        if escape {
            outcome.closed = true;
        }
    }

    let scale = 0.96 + 0.04 * openness;
    let width = (620.0 * scale).min(screen.width() - SPACE_24 * 2.0).max(260.0);
    let top = screen.min.y + (screen.height() * 0.14).max(48.0) + (1.0 - openness) * -10.0;
    let list_height = (screen.height() * 0.5).clamp(160.0, 420.0);

    let panel_output = egui::Area::new(Id::new("palette_area"))
        .order(Order::Tooltip)
        .fixed_pos(Pos2::new(screen.center().x - width * 0.5, top))
        .show(ctx, |ui| {
            ui.set_opacity(openness);
            egui::Frame::none()
                .fill(p.surface)
                .stroke(Stroke::new(1.0_f32, p.border))
                .rounding(Rounding::same(RADIUS_LARGE))
                .shadow(cx.theme.elevation_shadow(4))
                .inner_margin(egui::Margin::same(SPACE_8))
                .show(ui, |ui| {
                    ui.set_width(width - SPACE_8 * 2.0 - 2.0);

                    ui.horizontal(|ui| {
                        ui.add_space(SPACE_8);
                        let (icon_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(ICON_SIZE), Sense::hover());
                        icons::paint(ui.painter(), icon_rect, Icon::Command, p.accent, 1.7);
                        let font = cx.theme.font(cx.theme.type_scale.title * 0.9 * ui_scale(ui));
                        let edit = egui::TextEdit::singleline(&mut state.query)
                            .id(input_id)
                            .hint_text(
                                egui::RichText::new("Type a command or search")
                                    .font(font.clone())
                                    .color(p.text_muted),
                            )
                            .font(font)
                            .frame(false)
                            .desired_width(ui.available_width() - 64.0)
                            .margin(Vec2::new(0.0, 10.0));
                        let response = ui.add(edit);
                        if state.just_opened {
                            response.request_focus();
                            ctx.memory_mut(|m| m.request_focus(input_id));
                            state.just_opened = false;
                        }
                        if response.changed() {
                            state.cursor = 0;
                        }
                        kbd(ui, cx, "Esc");
                    });

                    ui.add_space(SPACE_4);
                    let (line, _) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
                    ui.painter()
                        .rect_filled(line, 0.0, with_alpha(p.border, 180));
                    ui.add_space(SPACE_4);

                    if commands.is_empty() {
                        ui.add_space(SPACE_24);
                        ui.vertical_centered(|ui| {
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                            icons::paint(
                                ui.painter(),
                                Rect::from_center_size(rect.center(), Vec2::splat(26.0)),
                                Icon::Search,
                                p.text_muted,
                                1.7,
                            );
                            ui.label(
                                egui::RichText::new("No matching commands")
                                    .font(cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)))
                                    .strong()
                                    .color(p.text_primary),
                            );
                            ui.label(
                                egui::RichText::new("Try a shorter or different search term.")
                                    .font(cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)))
                                    .color(p.text_secondary),
                            );
                        });
                        ui.add_space(SPACE_24);
                    } else {
                        let row_height = 48.0;
                        let cursor = state.cursor;
                        let scroll_to_cursor = ctx.input(|i| {
                            i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowUp)
                        });
                        egui::ScrollArea::vertical()
                            .max_height(list_height)
                            .auto_shrink([false, true])
                            .show_rows(ui, row_height, commands.len(), |ui, range| {
                                for index in range {
                                    let command = &commands[index];
                                    let selected = index == cursor;
                                    let (rect, response) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), row_height),
                                        Sense::click(),
                                    );
                                    response.widget_info(|| {
                                        egui::WidgetInfo::selected(
                                            egui::WidgetType::SelectableLabel,
                                            selected,
                                            command.title,
                                        )
                                    });
                                    let hover = response.hovered();
                                    if hover && ctx.input(|i| i.pointer.delta().length() > 0.0) {
                                        state.cursor = index;
                                    }
                                    if selected || hover {
                                        ui.painter().rect_filled(
                                            rect.shrink2(Vec2::new(0.0, 2.0)),
                                            Rounding::same(RADIUS_MEDIUM),
                                            if selected {
                                                p.accent_soft
                                            } else {
                                                blend(p.surface, p.accent, 0.06)
                                            },
                                        );
                                    }
                                    if selected {
                                        ui.painter().rect_filled(
                                            Rect::from_min_size(
                                                Pos2::new(rect.min.x, rect.min.y + 10.0),
                                                Vec2::new(3.0, rect.height() - 20.0),
                                            ),
                                            Rounding::same(RADIUS_PILL),
                                            p.accent,
                                        );
                                        if scroll_to_cursor {
                                            ui.scroll_to_rect(rect, None);
                                        }
                                    }
                                    icons::paint(
                                        ui.painter(),
                                        Rect::from_center_size(
                                            Pos2::new(rect.min.x + SPACE_16 + 9.0, rect.center().y),
                                            Vec2::splat(ICON_SIZE),
                                        ),
                                        command.icon,
                                        if selected { p.accent } else { p.text_secondary },
                                        1.7,
                                    );
                                    ui.painter().text(
                                        Pos2::new(rect.min.x + SPACE_16 + 36.0, rect.center().y - 9.0),
                                        Align2::LEFT_CENTER,
                                        command.title,
                                        cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)),
                                        p.text_primary,
                                    );
                                    ui.painter().text(
                                        Pos2::new(rect.min.x + SPACE_16 + 36.0, rect.center().y + 10.0),
                                        Align2::LEFT_CENTER,
                                        command.hint,
                                        cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)),
                                        p.text_muted,
                                    );
                                    if !command.shortcut.is_empty() {
                                        let font = cx
                                            .theme
                                            .font(cx.theme.type_scale.caption * ui_scale(ui));
                                        let galley = ui.painter().layout_no_wrap(
                                            command.shortcut.to_owned(),
                                            font.clone(),
                                            p.text_secondary,
                                        );
                                        let badge = Rect::from_min_size(
                                            Pos2::new(
                                                rect.max.x - SPACE_12 - galley.size().x - SPACE_8 * 1.5,
                                                rect.center().y - (galley.size().y + 6.0) * 0.5,
                                            ),
                                            Vec2::new(galley.size().x + SPACE_8 * 1.5, galley.size().y + 6.0),
                                        );
                                        ui.painter().rect(
                                            badge,
                                            Rounding::same(RADIUS_SMALL - 2.0),
                                            p.surface_variant,
                                            Stroke::new(1.0_f32, p.border),
                                        );
                                        ui.painter().text(
                                            badge.center(),
                                            Align2::CENTER_CENTER,
                                            command.shortcut,
                                            font,
                                            p.text_secondary,
                                        );
                                    }
                                    if hover {
                                        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                    }
                                    if response.clicked() {
                                        outcome.action = Some(command.id);
                                        outcome.closed = true;
                                    }
                                }
                            });
                    }

                    ui.add_space(SPACE_4);
                    let (line, _) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
                    ui.painter()
                        .rect_filled(line, 0.0, with_alpha(p.border, 180));
                    ui.add_space(SPACE_8);
                    ui.horizontal(|ui| {
                        ui.add_space(SPACE_8);
                        kbd(ui, cx, "Up");
                        kbd(ui, cx, "Down");
                        ui.label(
                            egui::RichText::new("navigate")
                                .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                                .color(p.text_muted),
                        );
                        ui.add_space(SPACE_8);
                        kbd(ui, cx, "Enter");
                        ui.label(
                            egui::RichText::new("run")
                                .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                                .color(p.text_muted),
                        );
                        ui.add_space(SPACE_8);
                        kbd(ui, cx, "Esc");
                        ui.label(
                            egui::RichText::new("close")
                                .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                                .color(p.text_muted),
                        );
                    });
                    ui.add_space(SPACE_4);
                });
        });

    let panel_rect = panel_output.response.rect;
    let clicked_outside = ctx.input(|i| {
        i.pointer.any_click()
            && i.pointer
                .interact_pos()
                .map(|pos| !panel_rect.contains(pos))
                .unwrap_or(false)
    });
    if state.open && clicked_outside {
        outcome.closed = true;
    }
    let _ = bg;

    outcome
}

pub fn shortcuts_body(ui: &mut Ui, cx: Ctx) {
    let p = &cx.theme.palette;
    let rows: Vec<(&str, &str)> = all_commands()
        .iter()
        .filter(|c| !c.shortcut.is_empty())
        .map(|c| (c.title, c.shortcut))
        .collect();
    let extra: [(&str, &str); 2] = [
        ("Open command palette", "Ctrl+K"),
        ("Focus global search", "Ctrl+F"),
    ];
    egui::ScrollArea::vertical()
        .max_height(360.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (title, shortcut) in extra.iter().chain(rows.iter()) {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(*title)
                            .font(cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)))
                            .color(p.text_primary),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        kbd(ui, cx, shortcut);
                    });
                });
                ui.add_space(SPACE_4);
            }
        });
}

pub struct ConfirmOutcome {
    pub confirmed: bool,
    pub cancelled: bool,
}

pub fn confirm_body(
    ui: &mut Ui,
    cx: Ctx,
    message: &str,
    confirm_label: &str,
) -> ConfirmOutcome {
    let p = &cx.theme.palette;
    let mut out = ConfirmOutcome {
        confirmed: false,
        cancelled: false,
    };
    ui.add(
        egui::Label::new(
            egui::RichText::new(message)
                .font(cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)))
                .color(p.text_secondary),
        )
        .wrap(true),
    );
    ui.add_space(SPACE_16);
    ui.horizontal(|ui| {
        if button(
            ui,
            cx,
            ButtonKind::Danger,
            confirm_label,
            Some(Icon::Reset),
            true,
            confirm_label,
        )
        .clicked()
        {
            out.confirmed = true;
        }
        if button(
            ui,
            cx,
            ButtonKind::Secondary,
            "Cancel",
            None,
            true,
            "Keep current settings",
        )
        .clicked()
        {
            out.cancelled = true;
        }
    });
    out
}

pub fn modal_kind_title(kind: ModalKind) -> &'static str {
    match kind {
        ModalKind::None => "",
        ModalKind::ConfirmReset => "Reset all settings?",
        ModalKind::ResultDetail => "Result details",
        ModalKind::Shortcuts => "Keyboard shortcuts",
    }
}

pub fn section_for_shortcut(ctx: &egui::Context) -> Option<Section> {
    ctx.input(|i| {
        if !i.modifiers.command || i.modifiers.shift || i.modifiers.alt {
            return None;
        }
        Section::ALL
            .iter()
            .copied()
            .find(|s| i.key_pressed(s.number_key()))
    })
}

