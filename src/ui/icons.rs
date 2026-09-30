use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Unlock,
    Lock,
    Download,
    Convert,
    Settings,
    Search,
    Sun,
    Moon,
    Command,
    Menu,
    Close,
    Copy,
    Save,
    Trash,
    Check,
    Warning,
    Error,
    Info,
    Folder,
    ChevronDown,
    ChevronRight,
    ChevronUp,
    Play,
    Expand,
    Rows,
    Reset,
    Sparkle,
}

fn p(rect: Rect, x: f32, y: f32) -> Pos2 {
    Pos2::new(
        rect.min.x + rect.width() * x,
        rect.min.y + rect.height() * y,
    )
}

fn line(painter: &Painter, rect: Rect, a: (f32, f32), b: (f32, f32), stroke: Stroke) {
    painter.line_segment([p(rect, a.0, a.1), p(rect, b.0, b.1)], stroke);
}

fn poly(painter: &Painter, rect: Rect, pts: &[(f32, f32)], stroke: Stroke) {
    let points: Vec<Pos2> = pts.iter().map(|(x, y)| p(rect, *x, *y)).collect();
    painter.add(Shape::line(points, stroke));
}

fn closed(painter: &Painter, rect: Rect, pts: &[(f32, f32)], stroke: Stroke) {
    let points: Vec<Pos2> = pts.iter().map(|(x, y)| p(rect, *x, *y)).collect();
    painter.add(Shape::closed_line(points, stroke));
}

fn filled(painter: &Painter, rect: Rect, pts: &[(f32, f32)], color: Color32) {
    let points: Vec<Pos2> = pts.iter().map(|(x, y)| p(rect, *x, *y)).collect();
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn arc(painter: &Painter, center: Pos2, radius: f32, from: f32, to: f32, stroke: Stroke) {
    let steps = 24;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = from + (to - from) * (i as f32 / steps as f32);
        points.push(Pos2::new(
            center.x + radius * t.cos(),
            center.y + radius * t.sin(),
        ));
    }
    painter.add(Shape::line(points, stroke));
}

pub fn paint(painter: &Painter, area: Rect, icon: Icon, color: Color32, weight: f32) {
    let side = area.width().min(area.height());
    let rect = Rect::from_center_size(area.center(), Vec2::splat(side));
    let stroke = Stroke::new(weight, color);

    match icon {
        Icon::Unlock => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.2, 0.46), p(rect, 0.8, 0.9)),
                2.0,
                stroke,
            );
            arc(
                painter,
                p(rect, 0.5, 0.32),
                rect.width() * 0.2,
                std::f32::consts::PI,
                std::f32::consts::TAU * 0.98,
                stroke,
            );
            line(painter, rect, (0.7, 0.32), (0.7, 0.2), stroke);
            painter.circle_filled(p(rect, 0.5, 0.68), rect.width() * 0.05, color);
        }
        Icon::Lock => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.2, 0.46), p(rect, 0.8, 0.9)),
                2.0,
                stroke,
            );
            arc(
                painter,
                p(rect, 0.5, 0.42),
                rect.width() * 0.2,
                std::f32::consts::PI,
                std::f32::consts::TAU,
                stroke,
            );
            painter.circle_filled(p(rect, 0.5, 0.68), rect.width() * 0.05, color);
        }
        Icon::Download => {
            line(painter, rect, (0.5, 0.12), (0.5, 0.64), stroke);
            poly(
                painter,
                rect,
                &[(0.3, 0.46), (0.5, 0.66), (0.7, 0.46)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.16, 0.66), (0.16, 0.88), (0.84, 0.88), (0.84, 0.66)],
                stroke,
            );
        }
        Icon::Convert => {
            poly(
                painter,
                rect,
                &[(0.14, 0.34), (0.82, 0.34)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.64, 0.16), (0.84, 0.34), (0.64, 0.52)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.86, 0.68), (0.18, 0.68)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.36, 0.5), (0.16, 0.68), (0.36, 0.86)],
                stroke,
            );
        }
        Icon::Settings => {
            let c = p(rect, 0.5, 0.5);
            painter.circle_stroke(c, rect.width() * 0.16, stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let inner = rect.width() * 0.28;
                let outer = rect.width() * 0.4;
                painter.line_segment(
                    [
                        Pos2::new(c.x + inner * a.cos(), c.y + inner * a.sin()),
                        Pos2::new(c.x + outer * a.cos(), c.y + outer * a.sin()),
                    ],
                    Stroke::new(weight * 1.4, color),
                );
            }
            painter.circle_stroke(c, rect.width() * 0.28, stroke);
        }
        Icon::Search => {
            painter.circle_stroke(p(rect, 0.44, 0.44), rect.width() * 0.26, stroke);
            line(painter, rect, (0.63, 0.63), (0.86, 0.86), stroke);
        }
        Icon::Sun => {
            let c = p(rect, 0.5, 0.5);
            painter.circle_stroke(c, rect.width() * 0.16, stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let inner = rect.width() * 0.26;
                let outer = rect.width() * 0.4;
                painter.line_segment(
                    [
                        Pos2::new(c.x + inner * a.cos(), c.y + inner * a.sin()),
                        Pos2::new(c.x + outer * a.cos(), c.y + outer * a.sin()),
                    ],
                    stroke,
                );
            }
        }
        Icon::Moon => {
            let c = p(rect, 0.5, 0.5);
            let r = rect.width() * 0.34;
            arc(
                painter,
                c,
                r,
                std::f32::consts::FRAC_PI_2 * 0.35,
                std::f32::consts::TAU - std::f32::consts::FRAC_PI_2 * 0.35,
                stroke,
            );
            arc(
                painter,
                p(rect, 0.62, 0.4),
                r * 0.78,
                std::f32::consts::FRAC_PI_2 * 0.6,
                std::f32::consts::PI * 1.25,
                stroke,
            );
        }
        Icon::Command => {
            let s = rect.width() * 0.14;
            let a = p(rect, 0.36, 0.36);
            let b = p(rect, 0.64, 0.36);
            let c = p(rect, 0.36, 0.64);
            let d = p(rect, 0.64, 0.64);
            painter.rect_stroke(Rect::from_two_pos(a, d), 0.0, stroke);
            painter.circle_stroke(Pos2::new(a.x - s * 0.2, a.y - s * 0.2), s, stroke);
            painter.circle_stroke(Pos2::new(b.x + s * 0.2, b.y - s * 0.2), s, stroke);
            painter.circle_stroke(Pos2::new(c.x - s * 0.2, c.y + s * 0.2), s, stroke);
            painter.circle_stroke(Pos2::new(d.x + s * 0.2, d.y + s * 0.2), s, stroke);
        }
        Icon::Menu => {
            line(painter, rect, (0.16, 0.28), (0.84, 0.28), stroke);
            line(painter, rect, (0.16, 0.5), (0.84, 0.5), stroke);
            line(painter, rect, (0.16, 0.72), (0.84, 0.72), stroke);
        }
        Icon::Close => {
            line(painter, rect, (0.24, 0.24), (0.76, 0.76), stroke);
            line(painter, rect, (0.76, 0.24), (0.24, 0.76), stroke);
        }
        Icon::Copy => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.34, 0.34), p(rect, 0.84, 0.86)),
                2.0,
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.24, 0.62), (0.16, 0.62), (0.16, 0.14), (0.62, 0.14), (0.62, 0.24)],
                stroke,
            );
        }
        Icon::Save => {
            closed(
                painter,
                rect,
                &[
                    (0.16, 0.16),
                    (0.72, 0.16),
                    (0.86, 0.3),
                    (0.86, 0.86),
                    (0.16, 0.86),
                ],
                stroke,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.32, 0.16), p(rect, 0.62, 0.36)),
                1.0,
                stroke,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.3, 0.56), p(rect, 0.7, 0.86)),
                1.0,
                stroke,
            );
        }
        Icon::Trash => {
            line(painter, rect, (0.14, 0.26), (0.86, 0.26), stroke);
            poly(
                painter,
                rect,
                &[(0.38, 0.26), (0.38, 0.14), (0.62, 0.14), (0.62, 0.26)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.24, 0.26), (0.3, 0.88), (0.7, 0.88), (0.76, 0.26)],
                stroke,
            );
            line(painter, rect, (0.42, 0.42), (0.44, 0.72), stroke);
            line(painter, rect, (0.58, 0.42), (0.56, 0.72), stroke);
        }
        Icon::Check => {
            poly(
                painter,
                rect,
                &[(0.2, 0.52), (0.42, 0.74), (0.82, 0.28)],
                Stroke::new(weight * 1.2, color),
            );
        }
        Icon::Warning => {
            closed(
                painter,
                rect,
                &[(0.5, 0.14), (0.9, 0.84), (0.1, 0.84)],
                stroke,
            );
            line(painter, rect, (0.5, 0.38), (0.5, 0.6), stroke);
            painter.circle_filled(p(rect, 0.5, 0.72), rect.width() * 0.04, color);
        }
        Icon::Error => {
            painter.circle_stroke(p(rect, 0.5, 0.5), rect.width() * 0.38, stroke);
            line(painter, rect, (0.36, 0.36), (0.64, 0.64), stroke);
            line(painter, rect, (0.64, 0.36), (0.36, 0.64), stroke);
        }
        Icon::Info => {
            painter.circle_stroke(p(rect, 0.5, 0.5), rect.width() * 0.38, stroke);
            line(painter, rect, (0.5, 0.46), (0.5, 0.7), stroke);
            painter.circle_filled(p(rect, 0.5, 0.31), rect.width() * 0.04, color);
        }
        Icon::Folder => {
            closed(
                painter,
                rect,
                &[
                    (0.12, 0.24),
                    (0.4, 0.24),
                    (0.48, 0.34),
                    (0.88, 0.34),
                    (0.88, 0.8),
                    (0.12, 0.8),
                ],
                stroke,
            );
        }
        Icon::ChevronDown => {
            poly(
                painter,
                rect,
                &[(0.22, 0.38), (0.5, 0.66), (0.78, 0.38)],
                stroke,
            );
        }
        Icon::ChevronRight => {
            poly(
                painter,
                rect,
                &[(0.38, 0.22), (0.66, 0.5), (0.38, 0.78)],
                stroke,
            );
        }
        Icon::ChevronUp => {
            poly(
                painter,
                rect,
                &[(0.22, 0.62), (0.5, 0.34), (0.78, 0.62)],
                stroke,
            );
        }
        Icon::Play => {
            filled(
                painter,
                rect,
                &[(0.3, 0.18), (0.82, 0.5), (0.3, 0.82)],
                color,
            );
        }
        Icon::Expand => {
            poly(
                painter,
                rect,
                &[(0.16, 0.4), (0.16, 0.16), (0.4, 0.16)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.6, 0.16), (0.84, 0.16), (0.84, 0.4)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.84, 0.6), (0.84, 0.84), (0.6, 0.84)],
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.4, 0.84), (0.16, 0.84), (0.16, 0.6)],
                stroke,
            );
        }
        Icon::Rows => {
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.16, 0.18), p(rect, 0.84, 0.42)),
                1.5,
                stroke,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(rect, 0.16, 0.58), p(rect, 0.84, 0.82)),
                1.5,
                stroke,
            );
        }
        Icon::Reset => {
            arc(
                painter,
                p(rect, 0.5, 0.5),
                rect.width() * 0.32,
                -std::f32::consts::FRAC_PI_2 * 0.2,
                std::f32::consts::TAU * 0.86,
                stroke,
            );
            poly(
                painter,
                rect,
                &[(0.82, 0.12), (0.82, 0.36), (0.58, 0.36)],
                stroke,
            );
        }
        Icon::Sparkle => {
            closed(
                painter,
                rect,
                &[
                    (0.5, 0.1),
                    (0.6, 0.4),
                    (0.9, 0.5),
                    (0.6, 0.6),
                    (0.5, 0.9),
                    (0.4, 0.6),
                    (0.1, 0.5),
                    (0.4, 0.4),
                ],
                stroke,
            );
        }
    }
}

pub fn allocate_icon(ui: &mut egui::Ui, icon: Icon, size: f32, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect.shrink(size * 0.06), icon, color, 1.6);
    }
    response
}
