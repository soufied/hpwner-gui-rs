use crate::ui::icons::{self, Icon};
use crate::ui::motion;
use crate::ui::theme::{
    blend, with_alpha, Theme, CONTROL_HEIGHT, ICON_SIZE, RADIUS_LARGE, RADIUS_MEDIUM,
    RADIUS_PILL, RADIUS_SMALL, SPACE_12, SPACE_16, SPACE_4, SPACE_8,
};
use eframe::egui::{
    self, Align2, Color32, CursorIcon, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub theme: &'a Theme,
    pub reduce_motion: bool,
}

pub fn draw_focus_ring(ui: &Ui, rect: Rect, rounding: f32, theme: &Theme) {
    ui.painter().rect_stroke(
        rect.expand(2.5),
        Rounding::same(rounding + 2.5),
        Stroke::new(2.0_f32, theme.palette.accent),
    );
}

fn button_colors(
    theme: &Theme,
    kind: ButtonKind,
    hover_t: f32,
    press_t: f32,
    enabled: bool,
) -> (Color32, Color32, Stroke) {
    let p = &theme.palette;
    let (rest_bg, hover_bg, press_bg, fg, border) = match kind {
        ButtonKind::Primary => (
            p.accent,
            p.accent_hover,
            p.accent_pressed,
            p.on_accent,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Secondary => (
            p.surface_variant,
            blend(p.surface_variant, p.accent, 0.14),
            blend(p.surface_variant, p.accent, 0.26),
            p.text_primary,
            p.border,
        ),
        ButtonKind::Ghost => (
            Color32::TRANSPARENT,
            blend(p.surface, p.accent, 0.12),
            blend(p.surface, p.accent, 0.22),
            p.text_primary,
            Color32::TRANSPARENT,
        ),
        ButtonKind::Danger => {
            let fg = if theme.dark {
                Color32::from_rgb(20, 6, 8)
            } else {
                Color32::from_rgb(255, 255, 255)
            };
            (
                p.danger,
                blend(p.danger, Color32::WHITE, if theme.dark { 0.14 } else { 0.0 }),
                blend(p.danger, Color32::BLACK, 0.2),
                fg,
                Color32::TRANSPARENT,
            )
        }
    };

    if !enabled {
        let bg = match kind {
            ButtonKind::Ghost => Color32::TRANSPARENT,
            _ => blend(p.surface_variant, p.background, 0.4),
        };
        return (
            bg,
            with_alpha(p.text_muted, 190),
            Stroke::new(1.0_f32, with_alpha(p.border, 120)),
        );
    }

    let mut bg = blend(rest_bg, hover_bg, hover_t);
    bg = blend(bg, press_bg, press_t);
    if kind == ButtonKind::Ghost {
        let alpha = (hover_t * 255.0) as u8;
        bg = with_alpha(blend(p.surface, p.accent, 0.12 + 0.10 * press_t), alpha);
    }
    (bg, fg, Stroke::new(1.0_f32, border))
}

pub fn button(
    ui: &mut Ui,
    cx: Ctx,
    kind: ButtonKind,
    label: &str,
    icon: Option<Icon>,
    enabled: bool,
    tooltip: &str,
) -> Response {
    let font = cx.theme.font(ui.style().text_styles[&egui::TextStyle::Button].size);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE);
    let icon_extra = if icon.is_some() { ICON_SIZE + SPACE_8 } else { 0.0 };
    let width = galley.size().x + icon_extra + SPACE_16 * 2.0;
    let height = CONTROL_HEIGHT.max(galley.size().y + SPACE_12);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
    let response = response.on_hover_text(tooltip);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, label)
    });

    if ui.is_rect_visible(rect) {
        let id = response.id;
        let hover_t = motion::animate_bool(
            ui.ctx(),
            id.with("hover"),
            response.hovered() && enabled,
            cx.theme.motion.fast,
            cx.reduce_motion,
        );
        let press_t = motion::animate_bool(
            ui.ctx(),
            id.with("press"),
            response.is_pointer_button_down_on() && enabled,
            cx.theme.motion.fast * 0.6,
            cx.reduce_motion,
        );
        let (bg, fg, stroke) = button_colors(cx.theme, kind, hover_t, press_t, enabled);
        let lift = if enabled { hover_t * 1.0 - press_t * 1.0 } else { 0.0 };
        let draw_rect = rect.translate(Vec2::new(0.0, -lift));

        if enabled && kind == ButtonKind::Primary && hover_t > 0.0 {
            let shadow = cx.theme.elevation_shadow(2);
            ui.painter().add(egui::epaint::RectShape::filled(
                draw_rect.translate(shadow.offset),
                Rounding::same(RADIUS_MEDIUM),
                with_alpha(cx.theme.palette.accent, (46.0 * hover_t) as u8),
            ));
        }

        ui.painter()
            .rect(draw_rect, Rounding::same(RADIUS_MEDIUM), bg, stroke);

        let mut cursor_x = draw_rect.min.x + SPACE_16;
        if let Some(ic) = icon {
            let icon_rect = Rect::from_min_size(
                Pos2::new(cursor_x, draw_rect.center().y - ICON_SIZE * 0.5),
                Vec2::splat(ICON_SIZE),
            );
            icons::paint(ui.painter(), icon_rect, ic, fg, 1.6);
            cursor_x += ICON_SIZE + SPACE_8;
        }
        ui.painter().text(
            Pos2::new(cursor_x, draw_rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font,
            fg,
        );

        if response.has_focus() {
            draw_focus_ring(ui, draw_rect, RADIUS_MEDIUM, cx.theme);
        }
    }

    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    response
}

pub fn icon_button(
    ui: &mut Ui,
    cx: Ctx,
    icon: Icon,
    tooltip: &str,
    selected: bool,
    enabled: bool,
) -> Response {
    let size = Vec2::splat(CONTROL_HEIGHT);
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let response = response.on_hover_text(tooltip);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, tooltip));

    if ui.is_rect_visible(rect) {
        let p = &cx.theme.palette;
        let id = response.id;
        let hover_t = motion::animate_bool(
            ui.ctx(),
            id.with("hover"),
            response.hovered() && enabled,
            cx.theme.motion.fast,
            cx.reduce_motion,
        );
        let press_t = motion::animate_bool(
            ui.ctx(),
            id.with("press"),
            response.is_pointer_button_down_on() && enabled,
            cx.theme.motion.fast * 0.6,
            cx.reduce_motion,
        );
        let sel_t = motion::animate_bool(
            ui.ctx(),
            id.with("sel"),
            selected,
            cx.theme.motion.normal,
            cx.reduce_motion,
        );

        let base = blend(p.surface, p.accent, 0.12 * hover_t + 0.10 * press_t);
        let bg = blend(base, p.accent_soft, sel_t);
        let alpha = ((hover_t.max(sel_t)) * 255.0) as u8;
        ui.painter()
            .rect_filled(rect, Rounding::same(RADIUS_MEDIUM), with_alpha(bg, alpha));

        let fg = if !enabled {
            with_alpha(p.text_muted, 160)
        } else if selected {
            p.accent
        } else {
            blend(p.text_secondary, p.text_primary, hover_t)
        };
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ICON_SIZE));
        icons::paint(ui.painter(), icon_rect, icon, fg, 1.7);

        if selected {
            ui.painter().rect_stroke(
                rect,
                Rounding::same(RADIUS_MEDIUM),
                Stroke::new(1.0_f32, with_alpha(p.accent, (140.0 * sel_t) as u8)),
            );
        }
        if response.has_focus() {
            draw_focus_ring(ui, rect, RADIUS_MEDIUM, cx.theme);
        }
    }

    if enabled && response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    response
}

pub fn chip(ui: &mut Ui, cx: Ctx, label: &str, tone: Option<Color32>) -> Response {
    let p = &cx.theme.palette;
    let font = cx.theme.font(ui.style().text_styles[&egui::TextStyle::Small].size);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font.clone(), p.text_primary);
    let size = Vec2::new(galley.size().x + SPACE_12 * 2.0, galley.size().y + SPACE_8);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, label));
    if ui.is_rect_visible(rect) {
        let accent = tone.unwrap_or(p.text_secondary);
        let bg = blend(p.surface_variant, accent, 0.14);
        ui.painter().rect(
            rect,
            Rounding::same(RADIUS_PILL),
            bg,
            Stroke::new(1.0_f32, with_alpha(accent, 110)),
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            font,
            p.text_primary,
        );
    }
    response
}

pub fn status_chip(ui: &mut Ui, cx: Ctx, icon: Icon, label: &str, tone: Color32) -> Response {
    let p = &cx.theme.palette;
    let font = cx.theme.font(ui.style().text_styles[&egui::TextStyle::Small].size);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font.clone(), p.text_primary);
    let icon_size = 14.0;
    let size = Vec2::new(
        galley.size().x + icon_size + SPACE_8 * 2.0 + SPACE_4,
        galley.size().y + SPACE_8,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, label));
    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            Rounding::same(RADIUS_PILL),
            blend(p.surface, tone, 0.16),
            Stroke::new(1.0_f32, with_alpha(tone, 120)),
        );
        let icon_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + SPACE_8, rect.center().y - icon_size * 0.5),
            Vec2::splat(icon_size),
        );
        icons::paint(ui.painter(), icon_rect, icon, tone, 1.5);
        ui.painter().text(
            Pos2::new(icon_rect.max.x + SPACE_4, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font,
            p.text_primary,
        );
    }
    response
}

pub fn separator(ui: &mut Ui, cx: Ctx) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, with_alpha(cx.theme.palette.border, 160));
}

pub fn field_label(ui: &mut Ui, cx: Ctx, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(cx.theme.font(cx.theme.type_scale.label * ui_scale(ui)))
            .strong()
            .color(cx.theme.palette.text_secondary),
    );
}

pub fn hint_text(ui: &mut Ui, cx: Ctx, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui)))
            .color(cx.theme.palette.text_muted),
    );
}

pub fn ui_scale(ui: &Ui) -> f32 {
    let body = ui.style().text_styles[&egui::TextStyle::Body].size;
    (body / 14.5).clamp(0.8, 1.6)
}

pub fn card<R>(
    ui: &mut Ui,
    cx: Ctx,
    id_source: impl std::hash::Hash,
    interactive: bool,
    selected: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> (Response, R) {
    let p = &cx.theme.palette;
    let id = ui.make_persistent_id(id_source);
    let rect_key = id.with("card_rect");
    let previous_rect: Option<Rect> = ui.ctx().memory(|m| m.data.get_temp(rect_key));
    let pointer_inside = match (previous_rect, ui.ctx().pointer_hover_pos()) {
        (Some(rect), Some(pos)) => rect.contains(pos),
        _ => false,
    };

    let hover_t = if interactive {
        motion::animate_bool(
            ui.ctx(),
            id.with("card_hover"),
            pointer_inside,
            cx.theme.motion.normal,
            cx.reduce_motion,
        )
    } else {
        0.0
    };
    let sel_t = motion::animate_bool(
        ui.ctx(),
        id.with("card_sel"),
        selected,
        cx.theme.motion.normal,
        cx.reduce_motion,
    );

    let lift = hover_t * 2.0;
    let shadow_level = if hover_t > 0.5 { 2 } else { 1 };
    let shadow = cx.theme.elevation_shadow(shadow_level);

    let padding = SPACE_12;
    let outer_width = ui.available_width();
    let frame_fill = blend(p.surface, blend(p.surface, p.accent, 0.05), hover_t);
    let border = blend(blend(p.border, p.accent, 0.5 * hover_t), p.accent, sel_t);

    let inner = egui::Frame::none()
        .fill(frame_fill)
        .stroke(Stroke::new(1.0_f32 + sel_t * 0.5, border))
        .rounding(Rounding::same(RADIUS_LARGE))
        .shadow(egui::epaint::Shadow {
            offset: Vec2::new(shadow.offset.x, shadow.offset.y - lift),
            ..shadow
        })
        .inner_margin(egui::Margin::same(padding))
        .show(ui, |ui| {
            ui.set_width((outer_width - padding * 2.0 - 2.0).max(40.0));
            add_contents(ui)
        });

    ui.ctx()
        .memory_mut(|m| m.data.insert_temp(rect_key, inner.response.rect));

    (inner.response, inner.inner)
}

pub fn skeleton_line(ui: &mut Ui, cx: Ctx, width: f32, height: f32) {
    let p = &cx.theme.palette;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let shimmer = motion::pulse(ui.ctx(), 1.4, cx.reduce_motion);
    let base = blend(p.surface_variant, p.border, 0.25 + 0.35 * shimmer);
    ui.painter()
        .rect_filled(rect, Rounding::same(RADIUS_SMALL), base);
}

pub fn skeleton_block(ui: &mut Ui, cx: Ctx, lines: usize) {
    let available = ui.available_width();
    for i in 0..lines {
        let factor = match i % 4 {
            0 => 1.0,
            1 => 0.86,
            2 => 0.94,
            _ => 0.58,
        };
        skeleton_line(ui, cx, available * factor, 12.0);
        ui.add_space(SPACE_4);
    }
}

pub fn kbd(ui: &mut Ui, cx: Ctx, text: &str) {
    if text.is_empty() {
        return;
    }
    let p = &cx.theme.palette;
    let font = cx.theme.font(cx.theme.type_scale.caption * ui_scale(ui));
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font.clone(), p.text_secondary);
    let size = Vec2::new(galley.size().x + SPACE_8 * 1.5, galley.size().y + SPACE_4 * 1.5);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect(
        rect,
        Rounding::same(RADIUS_SMALL - 2.0),
        p.surface_variant,
        Stroke::new(1.0_f32, p.border),
    );
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, p.text_secondary);
}

