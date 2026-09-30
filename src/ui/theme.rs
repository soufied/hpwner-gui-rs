use eframe::egui::epaint::Shadow;
use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, Vec2};
use std::collections::BTreeMap;

pub const SPACE_4: f32 = 4.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_24: f32 = 24.0;

pub const RADIUS_SMALL: f32 = 6.0;
pub const RADIUS_MEDIUM: f32 = 10.0;
pub const RADIUS_LARGE: f32 = 16.0;
pub const RADIUS_PILL: f32 = 999.0;

pub const CONTROL_HEIGHT: f32 = 26.0;
pub const ICON_SIZE: f32 = 18.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Cobalt,
    Teal,
    Violet,
    Rose,
    Amber,
    Emerald,
}

impl Accent {
    pub const ALL: [Accent; 6] = [
        Accent::Cobalt,
        Accent::Teal,
        Accent::Violet,
        Accent::Rose,
        Accent::Amber,
        Accent::Emerald,
    ];

    pub fn from_index(index: usize) -> Accent {
        Accent::ALL[index % Accent::ALL.len()]
    }

    pub fn index(self) -> usize {
        Accent::ALL
            .iter()
            .position(|a| *a == self)
            .unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            Accent::Cobalt => "Cobalt",
            Accent::Teal => "Teal",
            Accent::Violet => "Violet",
            Accent::Rose => "Rose",
            Accent::Amber => "Amber",
            Accent::Emerald => "Emerald",
        }
    }

    pub fn base(self, dark: bool) -> Color32 {
        match (self, dark) {
            (Accent::Cobalt, true) => Color32::from_rgb(110, 156, 255),
            (Accent::Cobalt, false) => Color32::from_rgb(38, 84, 214),
            (Accent::Teal, true) => Color32::from_rgb(74, 206, 200),
            (Accent::Teal, false) => Color32::from_rgb(8, 116, 122),
            (Accent::Violet, true) => Color32::from_rgb(178, 148, 255),
            (Accent::Violet, false) => Color32::from_rgb(103, 62, 210),
            (Accent::Rose, true) => Color32::from_rgb(255, 128, 160),
            (Accent::Rose, false) => Color32::from_rgb(190, 30, 84),
            (Accent::Amber, true) => Color32::from_rgb(255, 190, 84),
            (Accent::Amber, false) => Color32::from_rgb(158, 88, 0),
            (Accent::Emerald, true) => Color32::from_rgb(92, 212, 140),
            (Accent::Emerald, false) => Color32::from_rgb(10, 122, 70),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub surface_variant: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub scrim: Color32,
}

#[derive(Clone, Copy)]
pub struct TypeScale {
    pub headline: f32,
    pub title: f32,
    pub body: f32,
    pub label: f32,
    pub caption: f32,
}

#[derive(Clone, Copy)]
pub struct Motion {
    pub fast: f32,
    pub normal: f32,
    pub slow: f32,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub palette: Palette,
    pub type_scale: TypeScale,
    pub motion: Motion,
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 {
        (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

fn channel_luminance(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(color: Color32) -> f32 {
    0.2126 * channel_luminance(color.r())
        + 0.7152 * channel_luminance(color.g())
        + 0.0722 * channel_luminance(color.b())
}

pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn readable_on(background: Color32) -> Color32 {
    let dark_text = Color32::from_rgb(12, 16, 24);
    let light_text = Color32::from_rgb(250, 251, 253);
    if contrast_ratio(dark_text, background) >= contrast_ratio(light_text, background) {
        dark_text
    } else {
        light_text
    }
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    mix(a, b, t)
}

impl Theme {
    pub fn with_mode(dark: bool, accent: Accent) -> Theme {
        Theme::build(dark, accent)
    }

    fn build(dark: bool, accent_choice: Accent) -> Theme {
        let accent = accent_choice.base(dark);
        let palette = if dark {
            let background = Color32::from_rgb(14, 17, 22);
            let surface = Color32::from_rgb(22, 26, 33);
            let surface_variant = Color32::from_rgb(31, 36, 45);
            Palette {
                background,
                surface,
                surface_variant,
                border: Color32::from_rgb(52, 59, 72),
                text_primary: Color32::from_rgb(236, 240, 246),
                text_secondary: Color32::from_rgb(178, 187, 201),
                text_muted: Color32::from_rgb(140, 150, 166),
                success: Color32::from_rgb(88, 208, 138),
                warning: Color32::from_rgb(246, 187, 82),
                danger: Color32::from_rgb(255, 122, 122),
                info: Color32::from_rgb(118, 178, 255),
                accent,
                accent_hover: mix(accent, Color32::WHITE, 0.14),
                accent_pressed: mix(accent, Color32::BLACK, 0.16),
                accent_soft: mix(surface, accent, 0.18),
                on_accent: readable_on(accent),
                scrim: Color32::from_rgba_unmultiplied(4, 6, 10, 170),
            }
        } else {
            let background = Color32::from_rgb(246, 245, 242);
            let surface = Color32::from_rgb(255, 255, 255);
            let surface_variant = Color32::from_rgb(237, 236, 232);
            Palette {
                background,
                surface,
                surface_variant,
                border: Color32::from_rgb(196, 194, 187),
                text_primary: Color32::from_rgb(22, 25, 31),
                text_secondary: Color32::from_rgb(66, 72, 84),
                text_muted: Color32::from_rgb(98, 104, 116),
                success: Color32::from_rgb(14, 122, 66),
                warning: Color32::from_rgb(150, 86, 0),
                danger: Color32::from_rgb(190, 32, 46),
                info: Color32::from_rgb(28, 90, 200),
                accent,
                accent_hover: mix(accent, Color32::BLACK, 0.12),
                accent_pressed: mix(accent, Color32::BLACK, 0.26),
                accent_soft: mix(surface, accent, 0.12),
                on_accent: readable_on(accent),
                scrim: Color32::from_rgba_unmultiplied(18, 20, 26, 120),
            }
        };

        Theme {
            dark,
            palette,
            type_scale: TypeScale {
                headline: 26.0,
                title: 19.0,
                body: 14.5,
                label: 13.0,
                caption: 11.5,
            },
            motion: Motion {
                fast: 0.12,
                normal: 0.2,
                slow: 0.34,
            },
        }
    }

    pub fn elevation_shadow(&self, level: u8) -> Shadow {
        let level = level.min(4);
        if level == 0 {
            return Shadow::NONE;
        }
        let strength = if self.dark { 1.6 } else { 0.7 };
        let alpha = (14.0 * level as f32 * strength).min(120.0) as u8;
        Shadow {
            offset: Vec2::new(0.0, 1.0 + 2.0 * level as f32),
            blur: 4.0 + 6.0 * level as f32,
            spread: 0.0,
            color: Color32::from_rgba_unmultiplied(0, 0, 0, alpha),
        }
    }

    pub fn font(&self, size: f32) -> FontId {
        FontId::new(size, FontFamily::Proportional)
    }

    pub fn mono(&self, size: f32) -> FontId {
        FontId::new(size, FontFamily::Monospace)
    }

    pub fn apply(&self, ctx: &egui::Context, font_scale: f32) {
        let p = &self.palette;
        let scale = font_scale.clamp(0.8, 1.6);

        let mut style = (*ctx.style()).clone();

        let mut text_styles = BTreeMap::new();
        text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(self.type_scale.headline * scale, FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(self.type_scale.body * scale, FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(self.type_scale.label * scale, FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(self.type_scale.caption * scale, FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new((self.type_scale.body - 0.5) * scale, FontFamily::Monospace),
        );
        style.text_styles = text_styles;

        style.spacing.item_spacing = Vec2::new(SPACE_8, SPACE_8);
        style.spacing.button_padding = Vec2::new(SPACE_16, SPACE_8);
        style.spacing.interact_size = Vec2::new(40.0, CONTROL_HEIGHT);
        style.spacing.window_margin = egui::Margin::same(SPACE_16);
        style.spacing.menu_margin = egui::Margin::same(SPACE_8);
        style.spacing.indent = SPACE_16;
        style.spacing.icon_width = 18.0;
        style.spacing.icon_width_inner = 10.0;
        style.spacing.icon_spacing = SPACE_8;
        style.spacing.combo_width = 160.0;
        style.spacing.scroll = egui::style::ScrollStyle {
            floating: true,
            bar_width: 8.0,
            floating_width: 4.0,
            floating_allocated_width: 0.0,
            handle_min_length: 32.0,
            bar_inner_margin: 2.0,
            bar_outer_margin: 2.0,
            ..egui::style::ScrollStyle::floating()
        };

        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.dark_mode = self.dark;
        visuals.override_text_color = Some(p.text_primary);
        visuals.panel_fill = p.background;
        visuals.window_fill = p.surface;
        visuals.extreme_bg_color = p.surface;
        visuals.faint_bg_color = p.surface_variant;
        visuals.code_bg_color = p.surface_variant;
        visuals.window_stroke = Stroke::new(1.0_f32, p.border);
        visuals.window_rounding = Rounding::same(RADIUS_LARGE);
        visuals.window_shadow = self.elevation_shadow(4);
        visuals.popup_shadow = self.elevation_shadow(3);
        visuals.menu_rounding = Rounding::same(RADIUS_MEDIUM);
        visuals.hyperlink_color = p.accent;
        visuals.warn_fg_color = p.warning;
        visuals.error_fg_color = p.danger;
        visuals.selection.bg_fill = with_alpha(p.accent, 90);
        visuals.selection.stroke = Stroke::new(1.0_f32, p.accent);
        visuals.text_cursor = Stroke::new(2.0_f32, p.accent);
        visuals.striped = false;
        visuals.slider_trailing_fill = true;
        visuals.indent_has_left_vline = false;
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

        let radius = Rounding::same(RADIUS_SMALL + 2.0);

        visuals.widgets.noninteractive.bg_fill = p.surface;
        visuals.widgets.noninteractive.weak_bg_fill = p.surface;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text_secondary);
        visuals.widgets.noninteractive.rounding = radius;

        visuals.widgets.inactive.bg_fill = p.surface_variant;
        visuals.widgets.inactive.weak_bg_fill = p.surface_variant;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text_primary);
        visuals.widgets.inactive.rounding = radius;

        let hover_fill = mix(p.surface_variant, p.accent, 0.10);
        visuals.widgets.hovered.bg_fill = hover_fill;
        visuals.widgets.hovered.weak_bg_fill = hover_fill;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, mix(p.border, p.accent, 0.6));
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, p.text_primary);
        visuals.widgets.hovered.rounding = radius;
        visuals.widgets.hovered.expansion = 0.0;

        let active_fill = mix(p.surface_variant, p.accent, 0.22);
        visuals.widgets.active.bg_fill = active_fill;
        visuals.widgets.active.weak_bg_fill = active_fill;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, p.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0_f32, p.text_primary);
        visuals.widgets.active.rounding = radius;
        visuals.widgets.active.expansion = 0.0;

        visuals.widgets.open.bg_fill = p.accent_soft;
        visuals.widgets.open.weak_bg_fill = p.accent_soft;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.accent);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text_primary);
        visuals.widgets.open.rounding = radius;

        style.visuals = visuals;
        style.interaction.tooltip_delay = 0.35;
        style.interaction.selectable_labels = true;
        style.animation_time = 0.15;

        ctx.set_style(style);
    }
}

