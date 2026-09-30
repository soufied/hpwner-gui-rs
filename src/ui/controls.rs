use crate::ui::components::{draw_focus_ring, ui_scale, Ctx};
use crate::ui::icons::{self, Icon};
use crate::ui::motion;
use crate::ui::theme::{
    blend, with_alpha, CONTROL_HEIGHT, ICON_SIZE, RADIUS_MEDIUM, RADIUS_PILL, RADIUS_SMALL,
    SPACE_12, SPACE_4, SPACE_8,
};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, Key, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui,
    Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Validation {
    None,
    Valid,
    Invalid,
}

pub struct InputOptions<'a> {
    pub placeholder: &'a str,
    pub hint: Option<&'a str>,
    pub validation: Validation,
    pub validation_message: Option<&'a str>,
    pub password: bool,
    pub monospace: bool,
    pub tooltip: &'a str,
    pub max_width: f32,
}

impl<'a> InputOptions<'a> {
    pub fn new(placeholder: &'a str, tooltip: &'a str) -> Self {
        Self {
            placeholder,
            hint: None,
            validation: Validation::None,
            validation_message: None,
            password: false,
            monospace: false,
            tooltip,
            max_width: FIELD_WIDTH_MEDIUM,
        }
    }

    pub fn validation(mut self, validation: Validation, message: Option<&'a str>) -> Self {
        self.validation = validation;
        self.validation_message = message;
        self
    }

    pub fn monospace(mut self) -> Self {
        self.monospace = true;
        self
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }
}

pub const FIELD_WIDTH_SHORT: f32 = 220.0;
pub const FIELD_WIDTH_MEDIUM: f32 = 260.0;
pub const FIELD_WIDTH_LONG: f32 = 480.0;

pub fn text_input(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    value: &mut String,
    options: InputOptions,
) -> Response {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let width = ui.available_width().min(options.max_width);
    let clear_size = 22.0;

    let focus_t = motion::animate_bool(
        ui.ctx(),
        id.with("focus"),
        ui.memory(|m| m.has_focus(id)),
        cx.theme.motion.fast,
        cx.reduce_motion,
    );

    let tone = match options.validation {
        Validation::None => None,
        Validation::Valid => Some(p.success),
        Validation::Invalid => Some(p.danger),
    };
    let border_color = match tone {
        Some(t) => t,
        None => blend(p.border, p.accent, focus_t),
    };

    let mut clear_clicked = false;
    let frame = egui::Frame::none()
        .fill(blend(p.surface_variant, p.surface, 0.5 - 0.5 * focus_t))
        .stroke(Stroke::new(1.0_f32 + focus_t * 0.5, border_color))
        .rounding(Rounding::same(RADIUS_MEDIUM))
        .inner_margin(egui::Margin::symmetric(SPACE_12, 0.0));

    let mut edit_response: Option<Response> = None;
    let frame_out = frame.show(ui, |ui| {
        ui.set_width(width - SPACE_12 * 2.0 - 2.0);
        ui.set_height(CONTROL_HEIGHT);
        ui.horizontal_centered(|ui| {
            let font = if options.monospace {
                cx.theme.mono((cx.theme.type_scale.body - 0.5) * ui_scale(ui))
            } else {
                cx.theme.font(cx.theme.type_scale.body * ui_scale(ui))
            };
            let show_clear = !value.is_empty();
            let reserved = if show_clear { clear_size + SPACE_8 } else { 0.0 };
            let edit = egui::TextEdit::singleline(value)
                .id(id)
                .hint_text(
                    egui::RichText::new(options.placeholder)
                        .font(font.clone())
                        .color(p.text_muted),
                )
                .font(font)
                .password(options.password)
                .frame(false)
                .desired_width((ui.available_width() - reserved).max(40.0))
                .margin(Vec2::new(0.0, 4.0));
            let response = ui.add(edit);
            edit_response = Some(response);

            if show_clear {
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::splat(clear_size), Sense::click());
                let resp = resp.on_hover_text("Clear field");
                resp.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, "Clear field")
                });
                let hovered = resp.hovered();
                if hovered {
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(RADIUS_PILL),
                        with_alpha(p.text_muted, 50),
                    );
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), Vec2::splat(12.0)),
                    Icon::Close,
                    if hovered { p.text_primary } else { p.text_muted },
                    1.6,
                );
                if resp.has_focus() {
                    draw_focus_ring(ui, rect, RADIUS_PILL, cx.theme);
                }
                if resp.clicked() {
                    clear_clicked = true;
                }
            }
        });
    });

    if clear_clicked {
        value.clear();
        ui.memory_mut(|m| m.request_focus(id));
    }

    let mut response = edit_response.unwrap_or(frame_out.response);
    response = response.on_hover_text(options.tooltip);

    let mut caption: Option<(String, Color32)> = None;
    if let Some(message) = options.validation_message {
        if options.validation != Validation::None {
            let tone = tone.unwrap_or(p.text_muted);
            let glyph = if options.validation == Validation::Valid {
                "Looks good: "
            } else {
                "Check this: "
            };
            caption = Some((format!("{}{}", glyph, message), tone));
        }
    } else if let Some(hint) = options.hint {
        caption = Some((hint.to_string(), p.text_muted));
    }
    if let Some((text, color)) = caption {
        ui.add_space(SPACE_4);
        ui.label(
            egui::RichText::new(text)
                .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                .color(color),
        );
    }
    response
}

pub const MULTILINE_WIDTH_MAX: f32 = 560.0;

pub fn multiline_input(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    value: &mut String,
    placeholder: &str,
    rows: usize,
    tooltip: &str,
) -> Response {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let width = ui.available_width().min(MULTILINE_WIDTH_MAX);
    let focus_t = motion::animate_bool(
        ui.ctx(),
        id.with("focus"),
        ui.memory(|m| m.has_focus(id)),
        cx.theme.motion.fast,
        cx.reduce_motion,
    );
    let frame = egui::Frame::none()
        .fill(blend(p.surface_variant, p.surface, 0.5 - 0.5 * focus_t))
        .stroke(Stroke::new(1.0_f32 + focus_t * 0.5, blend(p.border, p.accent, focus_t)))
        .rounding(Rounding::same(RADIUS_MEDIUM))
        .inner_margin(egui::Margin::symmetric(SPACE_12, SPACE_4));
    let mut response: Option<Response> = None;
    let out = frame.show(ui, |ui| {
        ui.set_width(width - SPACE_12 * 2.0 - 2.0);
        let font = cx.theme.mono((cx.theme.type_scale.body - 0.5) * ui_scale(ui));
        let edit = egui::TextEdit::multiline(value)
            .id(id)
            .hint_text(
                egui::RichText::new(placeholder)
                    .font(font.clone())
                    .color(p.text_muted),
            )
            .font(font)
            .frame(false)
            .desired_rows(rows)
            .desired_width(f32::INFINITY);
        let max_height = (rows.max(1) as f32 + 1.0) * ui.text_style_height(&egui::TextStyle::Monospace);
        egui::ScrollArea::vertical()
            .id_source(id.with("scroll"))
            .max_height(max_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                response = Some(ui.add(edit));
            });
    });
    response
        .unwrap_or(out.response)
        .on_hover_text(tooltip)
}

pub fn toggle_switch(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    on: &mut bool,
    label: &str,
    description: Option<&str>,
    enabled: bool,
) -> Response {
    let p = &cx.theme.palette;
    let track = Vec2::new(44.0, 24.0);
    let mut changed = false;

    let row = ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)))
                    .color(if enabled { p.text_primary } else { p.text_muted }),
            );
            if let Some(desc) = description {
                ui.label(
                    egui::RichText::new(desc)
                        .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
                        .color(p.text_muted),
                );
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let sense = if enabled { Sense::click() } else { Sense::hover() };
            let id = ui.make_persistent_id(&id_source);
            let (rect, response) = ui.allocate_exact_size(track, sense);
            let response = response.on_hover_text(label);
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, label)
            });
            if response.clicked() {
                *on = !*on;
                changed = true;
            }
            let t = motion::animate_bool(
                ui.ctx(),
                id.with("toggle"),
                *on,
                cx.theme.motion.normal,
                cx.reduce_motion,
            );
            let hover_t = motion::animate_bool(
                ui.ctx(),
                id.with("toggle_hover"),
                response.hovered() && enabled,
                cx.theme.motion.fast,
                cx.reduce_motion,
            );
            let off_color = blend(p.surface_variant, p.border, 0.5 + 0.2 * hover_t);
            let track_color = blend(off_color, p.accent, t);
            let track_color = if enabled {
                track_color
            } else {
                with_alpha(track_color, 110)
            };
            ui.painter().rect(
                rect,
                Rounding::same(RADIUS_PILL),
                track_color,
                Stroke::new(1.0_f32, blend(p.border, p.accent, t)),
            );
            let knob_radius = track.y * 0.5 - 3.0 + hover_t * 0.6;
            let travel = track.x - track.y;
            let knob_x = rect.min.x + track.y * 0.5 + travel * t;
            let knob_center = Pos2::new(knob_x, rect.center().y);
            let knob_fill = blend(p.text_secondary, p.on_accent, t);
            ui.painter().circle_filled(knob_center, knob_radius, knob_fill);
            if t > 0.5 {
                icons::paint(
                    ui.painter(),
                    Rect::from_center_size(knob_center, Vec2::splat(knob_radius * 1.3)),
                    Icon::Check,
                    p.accent,
                    1.5,
                );
            }
            if response.has_focus() {
                draw_focus_ring(ui, rect, RADIUS_PILL, cx.theme);
            }
            if enabled && response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if enabled && response.has_focus() {
                let space = ui.input(|i| i.key_pressed(Key::Space) || i.key_pressed(Key::Enter));
                if space {
                    *on = !*on;
                    changed = true;
                }
            }
        });
    });
    let mut response = row.response;
    if changed {
        response.mark_changed();
    }
    response
}

pub fn slider(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    label: &str,
    format: impl Fn(f32) -> String,
) -> Response {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let (lo, hi) = (*range.start(), *range.end());
    let width = ui.available_width();
    let height = 28.0;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .font(cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)))
                .color(p.text_primary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format(*value))
                    .font(cx.theme.mono(cx.theme.type_scale.label * ui_scale(ui)))
                    .color(p.text_secondary),
            );
        });
    });

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
    let response = response.on_hover_text(label);
    let mut changed = false;

    let pad = 10.0;
    let track_left = rect.min.x + pad;
    let track_right = rect.max.x - pad;

    if let Some(pos) = response.interact_pointer_pos() {
        if response.dragged() || response.clicked() || response.is_pointer_button_down_on() {
            let t = ((pos.x - track_left) / (track_right - track_left)).clamp(0.0, 1.0);
            let mut new_value = lo + (hi - lo) * t;
            if step > 0.0 {
                new_value = (new_value / step).round() * step;
            }
            new_value = new_value.clamp(lo, hi);
            if (new_value - *value).abs() > f32::EPSILON {
                *value = new_value;
                changed = true;
            }
        }
    }

    if response.has_focus() {
        let (left, right, home, end) = ui.input(|i| {
            (
                i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::End),
            )
        });
        let delta = if step > 0.0 { step } else { (hi - lo) / 20.0 };
        if left {
            *value = (*value - delta).clamp(lo, hi);
            changed = true;
        }
        if right {
            *value = (*value + delta).clamp(lo, hi);
            changed = true;
        }
        if home {
            *value = lo;
            changed = true;
        }
        if end {
            *value = hi;
            changed = true;
        }
    }

    let target_t = if hi > lo { ((*value - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 0.0 };
    let t = motion::animate_value(
        ui.ctx(),
        id.with("slider_value"),
        target_t,
        cx.theme.motion.fast,
        cx.reduce_motion,
    );
    let hover_t = motion::animate_bool(
        ui.ctx(),
        id.with("slider_hover"),
        response.hovered() || response.dragged(),
        cx.theme.motion.fast,
        cx.reduce_motion,
    );
    let press_t = motion::animate_bool(
        ui.ctx(),
        id.with("slider_press"),
        response.dragged(),
        cx.theme.motion.fast,
        cx.reduce_motion,
    );

    if ui.is_rect_visible(rect) {
        let cy = rect.center().y;
        let track_rect = Rect::from_min_max(
            Pos2::new(track_left, cy - 3.0),
            Pos2::new(track_right, cy + 3.0),
        );
        ui.painter().rect_filled(
            track_rect,
            Rounding::same(RADIUS_PILL),
            blend(p.surface_variant, p.border, 0.7),
        );
        let knob_x = track_left + (track_right - track_left) * t;
        let fill_rect = Rect::from_min_max(track_rect.min, Pos2::new(knob_x, track_rect.max.y));
        ui.painter()
            .rect_filled(fill_rect, Rounding::same(RADIUS_PILL), p.accent);
        let radius = 8.0 + hover_t * 1.5 + press_t * 0.5;
        if hover_t > 0.0 {
            ui.painter().circle_filled(
                Pos2::new(knob_x, cy),
                radius + 5.0 * hover_t,
                with_alpha(p.accent, (55.0 * hover_t) as u8),
            );
        }
        ui.painter()
            .circle_filled(Pos2::new(knob_x, cy), radius, p.accent);
        ui.painter().circle_stroke(
            Pos2::new(knob_x, cy),
            radius,
            Stroke::new(2.0_f32, p.surface),
        );
        if response.has_focus() {
            draw_focus_ring(ui, rect.shrink2(Vec2::new(0.0, 2.0)), RADIUS_SMALL, cx.theme);
        }
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    let mut response = response;
    if changed {
        response.mark_changed();
    }
    response
}

pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    current: &mut T,
    options: &[(T, &str, Option<Icon>)],
) -> bool {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let font = cx.theme.font(cx.theme.type_scale.label * ui_scale(ui));
    let mut changed = false;

    let segment_widths: Vec<f32> = options
        .iter()
        .map(|(_, label, icon)| {
            let galley = ui
                .painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), p.text_primary);
            galley.size().x + SPACE_12 * 2.0 + if icon.is_some() { ICON_SIZE * 0.85 + SPACE_4 } else { 0.0 }
        })
        .collect();
    let total: f32 = segment_widths.iter().sum::<f32>() + 8.0;
    let height = CONTROL_HEIGHT;
    let (outer, _) = ui.allocate_exact_size(Vec2::new(total, height), Sense::hover());

    ui.painter().rect(
        outer,
        Rounding::same(RADIUS_MEDIUM),
        p.surface_variant,
        Stroke::new(1.0_f32, p.border),
    );

    let current_index = options
        .iter()
        .position(|(v, _, _)| *v == *current)
        .unwrap_or(0);
    let mut x = outer.min.x + 4.0;
    let mut segment_rects: Vec<Rect> = Vec::with_capacity(options.len());
    for w in &segment_widths {
        segment_rects.push(Rect::from_min_size(
            Pos2::new(x, outer.min.y + 4.0),
            Vec2::new(*w, height - 8.0),
        ));
        x += *w;
    }

    let target_left = segment_rects[current_index].min.x;
    let target_width = segment_rects[current_index].width();
    let left = motion::animate_value(
        ui.ctx(),
        id.with("seg_left"),
        target_left,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );
    let width = motion::animate_value(
        ui.ctx(),
        id.with("seg_width"),
        target_width,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );
    let indicator = Rect::from_min_size(
        Pos2::new(left, outer.min.y + 4.0),
        Vec2::new(width, height - 8.0),
    );
    ui.painter().rect(
        indicator,
        Rounding::same(RADIUS_SMALL + 1.0),
        p.surface,
        Stroke::new(1.0_f32, with_alpha(p.accent, 160)),
    );

    for (index, (value, label, icon)) in options.iter().enumerate() {
        let rect = segment_rects[index];
        let resp = ui.interact(rect, id.with(("seg", index)), Sense::click());
        let resp = resp.on_hover_text(*label);
        resp.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::RadioButton, index == current_index, *label)
        });
        let selected = index == current_index;
        let fg = if selected {
            p.text_primary
        } else if resp.hovered() {
            p.text_primary
        } else {
            p.text_secondary
        };
        let mut cursor_x = rect.center().x
            - (segment_widths[index] - SPACE_12 * 2.0) * 0.5;
        if let Some(ic) = icon {
            let icon_size = ICON_SIZE * 0.85;
            let icon_rect = Rect::from_min_size(
                Pos2::new(cursor_x, rect.center().y - icon_size * 0.5),
                Vec2::splat(icon_size),
            );
            icons::paint(
                ui.painter(),
                icon_rect,
                *ic,
                if selected { p.accent } else { fg },
                1.6,
            );
            cursor_x += icon_size + SPACE_4;
        }
        ui.painter().text(
            Pos2::new(cursor_x, rect.center().y),
            Align2::LEFT_CENTER,
            *label,
            font.clone(),
            fg,
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if resp.has_focus() {
            draw_focus_ring(ui, rect, RADIUS_SMALL, cx.theme);
        }
        if resp.clicked() && *current != *value {
            *current = *value;
            changed = true;
        }
        if resp.has_focus() {
            let (left_key, right_key) = ui.input(|i| {
                (i.key_pressed(Key::ArrowLeft), i.key_pressed(Key::ArrowRight))
            });
            if left_key && index > 0 {
                *current = options[index - 1].0;
                ui.memory_mut(|m| m.request_focus(id.with(("seg", index - 1))));
                changed = true;
            } else if right_key && index + 1 < options.len() {
                *current = options[index + 1].0;
                ui.memory_mut(|m| m.request_focus(id.with(("seg", index + 1))));
                changed = true;
            }
        }
    }
    changed
}

pub fn dropdown<T: Clone + PartialEq>(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    current: &mut T,
    options: &[(T, String)],
    width: f32,
    tooltip: &str,
) -> bool {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let popup_id = id.with("popup");
    let font = cx.theme.font(cx.theme.type_scale.body * ui_scale(ui));
    let mut changed = false;

    let selected_label = options
        .iter()
        .find(|(v, _)| v == current)
        .map(|(_, l)| l.clone())
        .unwrap_or_default();

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, CONTROL_HEIGHT), Sense::click());
    let response = response.on_hover_text(tooltip);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, selected_label.clone())
    });

    let is_open = ui.memory(|m| m.is_popup_open(popup_id));
    let hover_t = motion::animate_bool(
        ui.ctx(),
        id.with("dd_hover"),
        response.hovered() || is_open,
        cx.theme.motion.fast,
        cx.reduce_motion,
    );

    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    if response.has_focus() {
        let open_keys = ui.input(|i| {
            i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)
        });
        if open_keys && !is_open {
            ui.memory_mut(|m| m.open_popup(popup_id));
        }
    }

    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            Rounding::same(RADIUS_MEDIUM),
            blend(p.surface_variant, p.accent, 0.10 * hover_t),
            Stroke::new(1.0_f32, blend(p.border, p.accent, hover_t)),
        );
        ui.painter().text(
            Pos2::new(rect.min.x + SPACE_12, rect.center().y),
            Align2::LEFT_CENTER,
            &selected_label,
            font,
            p.text_primary,
        );
        icons::paint(
            ui.painter(),
            Rect::from_center_size(
                Pos2::new(rect.max.x - SPACE_12 - 6.0, rect.center().y),
                Vec2::splat(14.0),
            ),
            if is_open { Icon::ChevronUp } else { Icon::ChevronDown },
            p.text_secondary,
            1.7,
        );
        if response.has_focus() {
            draw_focus_ring(ui, rect, RADIUS_MEDIUM, cx.theme);
        }
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    if ui.memory(|m| m.is_popup_open(popup_id)) {
        let area = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::new(rect.min.x, rect.max.y + 4.0))
            .constrain(true);
        let inner = area.show(ui.ctx(), |ui| {
            egui::Frame::none()
                .fill(p.surface)
                .stroke(Stroke::new(1.0_f32, p.border))
                .rounding(Rounding::same(RADIUS_MEDIUM))
                .shadow(cx.theme.elevation_shadow(3))
                .inner_margin(egui::Margin::same(SPACE_4))
                .show(ui, |ui| {
                    ui.set_min_width(width - SPACE_4 * 2.0);
                    for (value, label) in options {
                        let selected = value == current;
                        let (row_rect, row_resp) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width().max(width - SPACE_4 * 2.0), 32.0),
                            Sense::click(),
                        );
                        row_resp.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                selected,
                                label.clone(),
                            )
                        });
                        let row_hover = row_resp.hovered();
                        if selected || row_hover {
                            ui.painter().rect_filled(
                                row_rect,
                                Rounding::same(RADIUS_SMALL),
                                if selected { p.accent_soft } else { blend(p.surface, p.accent, 0.08) },
                            );
                        }
                        if selected {
                            icons::paint(
                                ui.painter(),
                                Rect::from_center_size(
                                    Pos2::new(row_rect.min.x + SPACE_12 + 2.0, row_rect.center().y),
                                    Vec2::splat(12.0),
                                ),
                                Icon::Check,
                                p.accent,
                                1.7,
                            );
                        }
                        ui.painter().text(
                            Pos2::new(row_rect.min.x + SPACE_12 + 20.0, row_rect.center().y),
                            Align2::LEFT_CENTER,
                            label,
                            cx.theme.font(cx.theme.type_scale.body * ui_scale(ui)),
                            p.text_primary,
                        );
                        if row_hover {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        if row_resp.clicked() {
                            if *current != *value {
                                *current = value.clone();
                                changed = true;
                            }
                            ui.memory_mut(|m| m.close_popup());
                        }
                    }
                });
        });
        let clicked_elsewhere = ui.input(|i| i.pointer.any_click())
            && !inner.response.rect.contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO))
            && !rect.contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO));
        if clicked_elsewhere || ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.memory_mut(|m| m.close_popup());
        }
    }
    changed
}

