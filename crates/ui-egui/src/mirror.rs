//! The right-to-left interface for جدول: every frame is laid out as usual, then drawn mirrored
//! (ribbon, menus, panes, dialogs, sheet tabs and the status bar read from the right), with
//! pointer input mirrored back so clicks land where they are drawn. Text, pictures and icons'
//! textures are not flipped. The sheet area ([`keep`]) is moved as a block instead of being
//! mirrored: a right-to-left sheet already lays its columns out from the right by itself.
//!
//! Every window is mirrored, each with its own state: workbooks opened in further native windows
//! are egui viewports of their own. Dialogs and other windows ([`overlay`]), with the menus and
//! tooltips that open on them, are always mirrored as a whole, even over the sheet. Accessibility
//! bounds stay unmirrored; per-widget layout would fix that.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use egui::epaint::ClippedShape;
use egui::layers::ShapeIdx;
use egui::{CornerRadius, Event, LayerId, Order, Pos2, Rect, Shape, TextureId, ViewportId, pos2, vec2};

/// One window's mirroring.
#[derive(Clone, Default)]
struct Window {
    /// `min.x + max.x` of the window: x mirrors to `span − x`.
    span: f32,
    /// The document area drawn this frame, and the one drawn last frame (for input).
    keep: Option<Rect>,
    last_keep: Option<Rect>,
    /// Windows registered this frame with [`overlay`].
    overlays: Vec<LayerId>,
    /// Menus registered this frame with [`menu`].
    menus: Vec<LayerId>,
    /// Where last frame's floating layers were drawn, topmost first, and how, for input.
    last_layers: Vec<(Rect, Drawn)>,
}

/// How a floating layer is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Drawn {
    /// Mirrored with the whole interface.
    Mirrored,
    /// Moved with the document area.
    Moved,
    /// A menu opened on the sheet: moved with the document area and mirrored within its own
    /// rect (`min.x + max.x` in layout), so it stays at the click and reads from the right.
    Flipped(f32),
}

thread_local! {
    static WINDOWS: RefCell<HashMap<ViewportId, Window>> = RefCell::new(HashMap::new());
    /// Whether the last frame drew in a right-to-left language (input arrives before the frame
    /// says which language it draws in).
    static ACTIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn with<R>(id: ViewportId, f: impl FnOnce(&mut Window) -> R) -> R {
    WINDOWS.with_borrow_mut(|w| f(w.entry(id).or_default()))
}

/// Is the interface mirrored? Whenever the interface language is right to left (outside tests).
pub fn active() -> bool {
    !cfg!(test) && ACTIVE.get()
}

/// Mirrors every viewport's pointer input before egui reads it (register once per context).
#[derive(Default)]
pub struct MirrorInput;

impl egui::Plugin for MirrorInput {
    fn debug_name(&self) -> &'static str {
        "jadwal_mirror_input"
    }

    fn input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        self::input(input);
    }
}

/// The document area of `ctx`'s window this frame (rulers and pages): moved, not mirrored.
pub fn keep(ctx: &egui::Context, rect: Rect) {
    with(ctx.viewport_id(), |w| w.keep = Some(rect));
}

/// A dialog or other window shown this frame in `ctx`'s window: mirrored as a whole, with any
/// menu or tooltip that touches it, never moved with the document area under it.
pub fn overlay(ctx: &egui::Context, layer: LayerId) {
    with(ctx.viewport_id(), |w| w.overlays.push(layer));
}

/// A menu opened on the sheet (context, filter, fill and header menus) shown this frame: it
/// stays where it opened, and reads from the right.
pub fn menu(ctx: &egui::Context, layer: LayerId) {
    with(ctx.viewport_id(), |w| w.menus.push(layer));
}

/// Shows a modal mirrored as a whole (see [`overlay`]).
pub fn modal<R>(ctx: &egui::Context, modal: egui::Modal, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::ModalResponse<R> {
    let shown = modal.show(ctx, add);
    overlay(ctx, shown.response.layer_id);
    shown
}

fn mirror_rect(s: f32, r: Rect) -> Rect {
    Rect::from_min_max(pos2(s - r.max.x, r.min.y), pos2(s - r.min.x, r.max.y))
}

/// How far the document area moves: from where it is laid out to its mirrored place.
fn keep_dx(s: f32, keep: Rect) -> f32 {
    (s - keep.max.x) - keep.min.x
}

/// A point as the user sees it, back to where the layout put it.
fn to_layout(p: Pos2, s: f32, keep: Option<Rect>, layers: &[(Rect, Drawn)]) -> Pos2 {
    match (layers.iter().find(|(r, _)| r.contains(p)), keep) {
        (Some((_, Drawn::Mirrored)), _) => return pos2(s - p.x, p.y),
        (Some((_, Drawn::Moved)), Some(k)) => return pos2(p.x - keep_dx(s, k), p.y),
        (Some((_, Drawn::Flipped(a))), Some(k)) => return pos2(a - (p.x - keep_dx(s, k)), p.y),
        _ => {}
    }
    match keep {
        Some(k) if mirror_rect(s, k).contains(p) => pos2(p.x - keep_dx(s, k), p.y),
        _ => pos2(s - p.x, p.y),
    }
}

/// A rect in layout space, to where it is drawn.
fn to_screen(r: Rect, s: f32, keep: Option<Rect>) -> Rect {
    match keep {
        Some(k) if k.expand(1.0).contains_rect(r) => r.translate(vec2(keep_dx(s, k), 0.0)),
        _ => mirror_rect(s, r),
    }
}

/// Mirror a window's pointer input for this frame (call from an input hook, before egui reads it).
pub fn input(raw: &mut egui::RawInput) {
    if !active() {
        return;
    }
    let (s, keep, layers) = with(raw.viewport_id, |w| {
        if let Some(r) = raw.screen_rect {
            w.span = r.min.x + r.max.x;
        }
        (w.span, w.last_keep, w.last_layers.clone())
    });
    if s <= 0.0 {
        return;
    }
    for e in &mut raw.events {
        match e {
            Event::PointerMoved(p) => *p = to_layout(*p, s, keep, &layers),
            Event::PointerButton { pos, .. } | Event::Touch { pos, .. } => *pos = to_layout(*pos, s, keep, &layers),
            Event::MouseMoved(d) => d.x = -d.x,
            _ => {}
        }
    }
}

/// Mirror everything painted in `ctx`'s window this frame (call after the window's `ui`).
pub fn output(ctx: &egui::Context) {
    let s = ctx.input(|i| {
        let r = i.viewport_rect();
        r.min.x + r.max.x
    });
    let (keep, overlays, menus) = with(ctx.viewport_id(), |w| {
        let keep = w.keep.take();
        w.last_keep = keep;
        w.last_layers.clear();
        if s > 0.0 {
            w.span = s;
        }
        (keep, std::mem::take(&mut w.overlays), std::mem::take(&mut w.menus))
    });
    ACTIVE.set(crate::i18n::current(ctx).is_rtl());
    if !active() || s <= 0.0 {
        return;
    }
    let mut ids: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
    if !ids.contains(&LayerId::background()) {
        ids.push(LayerId::background());
    }
    // Each floating layer (menu, tooltip, dialog) is drawn whole one way, never split by what
    // lies under it: mirrored, or moved with the document area when it opened from the page.
    let floating = shown_layers(ctx, &ids);
    let modes = layer_modes(&floating, &overlays, keep);
    let drawn: Vec<Drawn> = floating
        .iter()
        .zip(&modes)
        .map(|(&(id, r), &whole)| match (whole, menus.contains(&id)) {
            (true, _) => Drawn::Mirrored,
            (false, true) => Drawn::Flipped(r.min.x + r.max.x),
            (false, false) => Drawn::Moved,
        })
        .collect();
    with(ctx.viewport_id(), |w| {
        w.last_layers = floating.iter().zip(&drawn).rev().map(|(&(_, r), &d)| (to_screen_as(r, s, keep, d == Drawn::Mirrored), d)).collect();
    });
    ctx.graphics_mut(|g| {
        for id in ids {
            let mode = floating.iter().position(|&(f, _)| f == id).and_then(|i| drawn.get(i).copied());
            if let Some(list) = g.get_mut(id) {
                for i in 0..list.next_idx().0 {
                    list.mutate_shape(ShapeIdx(i), |c| match (mode, keep) {
                        (Some(Drawn::Moved), Some(k)) => {
                            let d = vec2(keep_dx(s, k), 0.0);
                            c.clip_rect = c.clip_rect.translate(d);
                            c.shape.translate(d);
                        }
                        (Some(Drawn::Flipped(a)), Some(k)) => {
                            clipped(c, a, None);
                            let d = vec2(keep_dx(s, k), 0.0);
                            c.clip_rect = c.clip_rect.translate(d);
                            c.shape.translate(d);
                            c.transform_after_tessellation.translation.x += d.x - c.transform_after_tessellation.scaling * d.x;
                        }
                        (Some(_), _) => clipped(c, s, None),
                        (None, _) => clipped(c, s, keep),
                    });
                }
            }
        }
    });
    ctx.output_mut(|o| {
        if let Some(ime) = &mut o.ime {
            // A text field in a dialog is mirrored with its dialog.
            match floating.iter().zip(&drawn).rev().find(|((_, r), _)| r.contains_rect(ime.rect)) {
                Some((_, Drawn::Mirrored)) => {
                    ime.rect = to_screen(ime.rect, s, None);
                    ime.cursor_rect = to_screen(ime.cursor_rect, s, None);
                }
                Some((_, &Drawn::Flipped(a))) => {
                    let d = vec2(keep.map_or(0.0, |k| keep_dx(s, k)), 0.0);
                    ime.rect = mirror_rect(a, ime.rect).translate(d);
                    ime.cursor_rect = mirror_rect(a, ime.cursor_rect).translate(d);
                }
                _ => {
                    ime.rect = to_screen(ime.rect, s, keep);
                    ime.cursor_rect = to_screen(ime.cursor_rect, s, keep);
                }
            }
        }
    });
}

/// For each floating layer, bottom to top: mirrored whole (`true`), or moved with the document
/// area (`false`). Registered overlays are mirrored. Any other layer goes with what is just above
/// its top edge, where it was opened from: a ribbon button or a mirrored menu (mirrored), or the
/// page (moved), so a context menu stays at the click and a ribbon list under its button.
fn layer_modes(floating: &[(LayerId, Rect)], overlays: &[LayerId], keep: Option<Rect>) -> Vec<bool> {
    let mut modes: Vec<bool> = Vec::with_capacity(floating.len());
    for &(id, r) in floating {
        let probe = pos2(r.center().x, r.min.y - 6.0);
        let under = floating[..modes.len()].iter().zip(&modes).rev().find(|((_, u), _)| u.contains(probe)).map(|(_, &m)| m);
        let whole = overlays.contains(&id) || id.order == Order::Middle || under.unwrap_or_else(|| !keep.is_some_and(|k| k.contains(probe)));
        modes.push(whole);
    }
    modes
}

fn to_screen_as(r: Rect, s: f32, keep: Option<Rect>, whole: bool) -> Rect {
    match keep {
        Some(k) if !whole => r.translate(vec2(keep_dx(s, k), 0.0)),
        _ => mirror_rect(s, r),
    }
}

fn clipped(c: &mut ClippedShape, s: f32, keep: Option<Rect>) {
    if let Some(k) = keep
        && k.expand(1.0).contains_rect(c.clip_rect)
    {
        let d = vec2(keep_dx(s, k), 0.0);
        c.clip_rect = c.clip_rect.translate(d);
        c.shape.translate(d);
        return;
    }
    c.clip_rect = mirror_rect(s, c.clip_rect);
    flip(&mut c.shape, s);
    // A transform applied after tessellation (a popup scaling in) maps the mirrored shape to the
    // mirror of where it would have gone.
    let t = &mut c.transform_after_tessellation;
    t.translation.x = s - t.translation.x - t.scaling * s;
}

fn flip(shape: &mut Shape, s: f32) {
    let fx = |p: &mut Pos2| p.x = s - p.x;
    match shape {
        Shape::Noop => {}
        Shape::Vec(v) => v.iter_mut().for_each(|x| flip(x, s)),
        Shape::Circle(c) => fx(&mut c.center),
        Shape::Ellipse(e) => {
            fx(&mut e.center);
            e.angle = -e.angle;
        }
        Shape::LineSegment { points, .. } => points.iter_mut().for_each(fx),
        Shape::Path(p) => p.points.iter_mut().for_each(fx),
        Shape::Band(b) => {
            b.points.iter_mut().for_each(|p| p.x = s - p.x);
            b.points.reverse(); // kept sorted by x
        }
        Shape::Rect(r) => {
            r.rect = mirror_rect(s, r.rect);
            let c = r.corner_radius;
            r.corner_radius = CornerRadius { nw: c.ne, ne: c.nw, sw: c.se, se: c.sw };
            r.angle = -r.angle;
            if let Some(b) = &mut r.brush {
                // Swapping the u range draws a picture the right way round in its mirrored rect.
                let b = Arc::make_mut(b);
                b.uv = Rect::from_min_max(pos2(b.uv.max.x, b.uv.min.y), pos2(b.uv.min.x, b.uv.max.y));
            }
        }
        Shape::Text(t) => {
            // Moved, never flipped: its galley keeps its own (bidi) order.
            let g = t.galley.rect;
            let min_x = s - (t.pos.x + g.max.x);
            t.pos.x = min_x - g.min.x;
        }
        Shape::Mesh(m) => {
            let m = Arc::make_mut(m);
            m.vertices.iter_mut().for_each(|v| v.pos.x = s - v.pos.x);
            if m.texture_id != TextureId::default() {
                // A picture: flip its texture coordinates back within the mesh's u range.
                let (lo, hi) = m.vertices.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| (lo.min(v.uv.x), hi.max(v.uv.x)));
                m.vertices.iter_mut().for_each(|v| v.uv.x = lo + hi - v.uv.x);
            }
        }
        Shape::QuadraticBezier(b) => b.points.iter_mut().for_each(fx),
        Shape::CubicBezier(b) => b.points.iter_mut().for_each(fx),
        Shape::Callback(c) => c.rect = mirror_rect(s, c.rect),
    }
}

/// The floating layers (menus, tooltips, dialogs) shown now, with where they were laid out.
/// egui remembers every area it has ever shown, so a dialog that has closed is left out: its old
/// rect would otherwise still catch clicks and mirror them across the window.
fn shown_layers(ctx: &egui::Context, ids: &[LayerId]) -> Vec<(LayerId, Rect)> {
    ctx.memory(|m| {
        ids.iter().filter(|id| id.order != Order::Background && m.areas().is_visible(id)).filter_map(|&id| Some((id, m.area_rect(id.id)?))).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closed_dialog_no_longer_catches_clicks() {
        let ctx = egui::Context::default();
        let frame = |show: bool| {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                if show {
                    egui::Window::new("dialog").show(ui.ctx(), |ui| ui.label("x"));
                }
            });
            out.textures_delta.clear();
        };
        frame(true);
        let ids: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
        assert!(shown_layers(&ctx, &ids).iter().any(|(l, _)| l.order == Order::Middle));
        frame(false);
        frame(false);
        let ids: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
        assert!(ids.iter().any(|l| l.order == Order::Middle), "egui still remembers the closed dialog");
        assert!(shown_layers(&ctx, &ids).is_empty());
    }

    #[test]
    fn points_round_trip_and_the_document_area_moves_as_a_block() {
        let s = 1000.0;
        let keep = Some(Rect::from_min_max(pos2(200.0, 50.0), pos2(900.0, 800.0)));
        // Outside the document area: mirrored.
        assert_eq!(to_layout(pos2(10.0, 5.0), s, keep, &[]), pos2(990.0, 5.0));
        // The area is drawn at 100..800: a point there maps back by the same shift.
        assert_eq!(to_layout(pos2(150.0, 100.0), s, keep, &[]), pos2(250.0, 100.0));
        let r = Rect::from_min_max(pos2(300.0, 60.0), pos2(310.0, 70.0));
        let drawn = to_screen(r, s, keep);
        assert_eq!(to_layout(drawn.center(), s, keep, &[]), r.center());
        // Over a dialog drawn above the document area: mirrored like the rest of the interface.
        let dialog = [(Rect::from_min_max(pos2(120.0, 90.0), pos2(400.0, 300.0)), Drawn::Mirrored)];
        assert_eq!(to_layout(pos2(150.0, 100.0), s, keep, &dialog), pos2(850.0, 100.0));
        // A context menu laid out at 300..400 is drawn at 200..300 (the area's shift), mirrored
        // within itself: its left edge on screen is its right edge in layout.
        let menu = [(Rect::from_min_max(pos2(200.0, 400.0), pos2(300.0, 500.0)), Drawn::Flipped(700.0))];
        assert_eq!(to_layout(pos2(201.0, 450.0), s, keep, &menu), pos2(399.0, 450.0));
    }

    #[test]
    fn floating_layers_follow_where_they_opened() {
        let keep = Some(Rect::from_min_max(pos2(0.0, 100.0), pos2(1000.0, 800.0)));
        let l = |n: &str, o, r: [f32; 4]| (LayerId::new(o, egui::Id::unique(n)), Rect::from_min_max(pos2(r[0], r[1]), pos2(r[2], r[3])));
        let dialog = l("dialog", Order::Middle, [300.0, 200.0, 700.0, 500.0]);
        let floating = [
            dialog,
            // Font size list under a ribbon button, hanging over the page.
            l("size", Order::Tooltip, [500.0, 102.0, 560.0, 400.0]),
            // Its list in a dialog, and a submenu of that list.
            l("style", Order::Foreground, [400.0, 300.0, 520.0, 600.0]),
            l("sub", Order::Foreground, [520.0, 320.0, 600.0, 400.0]),
            // A context menu opened on the page.
            l("context", Order::Foreground, [100.0, 600.0, 250.0, 750.0]),
        ];
        assert_eq!(layer_modes(&floating, &[dialog.0], keep), [true, true, true, true, false]);
    }

    #[test]
    fn text_moves_without_flipping() {
        let r = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 20.0));
        let m = mirror_rect(1000.0, r);
        assert_eq!((m.min.x, m.max.x), (900.0, 1000.0));
    }
}
