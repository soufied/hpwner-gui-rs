use eframe::egui::{self, Id};
use std::time::Duration;

const FRAME_INTERVAL: Duration = Duration::from_millis(16);

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn animate_bool(
    ctx: &egui::Context,
    id: Id,
    target: bool,
    duration: f32,
    reduce_motion: bool,
) -> f32 {
    if reduce_motion || duration <= 0.0 {
        return if target { 1.0 } else { 0.0 };
    }
    ease_out_cubic(ctx.animate_bool_with_time(id, target, duration))
}

pub fn animate_value(
    ctx: &egui::Context,
    id: Id,
    target: f32,
    duration: f32,
    reduce_motion: bool,
) -> f32 {
    if reduce_motion || duration <= 0.0 {
        return target;
    }
    ctx.animate_value_with_time(id, target, duration)
}

pub fn pulse(ctx: &egui::Context, period: f32, reduce_motion: bool) -> f32 {
    if reduce_motion {
        return 1.0;
    }
    let time = ctx.input(|i| i.time) as f32;
    let phase = (time / period.max(0.1)) * std::f32::consts::TAU;
    ctx.request_repaint_after(FRAME_INTERVAL);
    0.5 + 0.5 * phase.sin()
}

pub fn spinner_angle(ctx: &egui::Context, reduce_motion: bool) -> f32 {
    let time = ctx.input(|i| i.time) as f32;
    if reduce_motion {
        ctx.request_repaint_after(Duration::from_millis(250));
        (time * 2.0).floor() * std::f32::consts::FRAC_PI_2
    } else {
        ctx.request_repaint_after(FRAME_INTERVAL);
        time * std::f32::consts::TAU * 0.9
    }
}

pub fn transition_progress(
    ctx: &egui::Context,
    channel: &'static str,
    generation: u64,
    duration: f32,
    reduce_motion: bool,
) -> f32 {
    if reduce_motion || duration <= 0.0 {
        return 1.0;
    }
    let id = Id::new(("transition", channel));
    let now = ctx.input(|i| i.time);
    let stored: Option<(u64, f64)> = ctx.memory(|m| m.data.get_temp(id));
    let started = match stored {
        Some((gen, start)) if gen == generation => start,
        _ => {
            ctx.memory_mut(|m| m.data.insert_temp(id, (generation, now)));
            now
        }
    };
    let raw = ((now - started) as f32 / duration).clamp(0.0, 1.0);
    if raw < 1.0 {
        ctx.request_repaint_after(FRAME_INTERVAL);
    }
    ease_out_cubic(raw)
}

