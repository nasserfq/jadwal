//! Arabic text: shaped with harfrust in the embedded Noto Sans Arabic face (craft-fonts), laid
//! out in visual order with the Unicode bidirectional algorithm, and written as glyph ids
//! (`Identity-H`) with a `ToUnicode` map so the text can be searched and copied.
//!
//! Only Arabic-script characters use the face; the spaces, digits, Latin and punctuation around
//! them keep the standard fonts. Without craft-fonts at build time this module draws nothing and
//! Arabic falls back to the non-embedded CID font as before.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use read_fonts::types::GlyphId;
use read_fonts::{FontRef, TableProvider};

include!(concat!(env!("OUT_DIR"), "/arabic_font.rs"));

/// PostScript name the face is embedded under.
pub(crate) const BASE_FONT: &str = "NotoSansArabic";

/// True for the characters drawn with the Arabic face.
pub(crate) fn is_arabic(c: char) -> bool {
    matches!(c as u32, 0x0600..=0x06FF | 0x0750..=0x077F | 0x0870..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
}

/// The parsed face: its bytes, metrics and shaping tables.
pub(crate) struct Face {
    pub bytes: &'static [u8],
    upem: f32,
    pub ascent: i32,
    pub descent: i32,
    pub bbox: [i32; 4],
    shaper: harfrust::ShaperData,
}

/// The embedded face, or `None` in a build without craft-fonts (or with an unreadable file).
pub(crate) fn face() -> Option<&'static Face> {
    static FACE: OnceLock<Option<Face>> = OnceLock::new();
    FACE.get_or_init(|| {
        let bytes = ARABIC_FONT_BYTES?;
        let font = FontRef::new(bytes).ok()?;
        let head = font.head().ok()?;
        let hhea = font.hhea().ok()?;
        let upem = f32::from(head.units_per_em().max(1));
        let scale = |v: i16| (f32::from(v) * 1000.0 / upem).round() as i32;
        Some(Face {
            bytes,
            upem,
            ascent: scale(hhea.ascender().to_i16()),
            descent: scale(hhea.descender().to_i16()),
            bbox: [scale(head.x_min()), scale(head.y_min()), scale(head.x_max()), scale(head.y_max())],
            shaper: harfrust::ShaperData::new(&font),
        })
    })
    .as_ref()
}

/// Whether Arabic text in `text` is drawn with the embedded face.
pub(crate) fn applies(text: &str) -> bool {
    text.chars().any(is_arabic) && face().is_some()
}

/// One shaped glyph.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Glyph {
    pub gid: u16,
    /// Shaped advance in 1/1000 em.
    pub advance: f32,
    /// The face's own advance for this glyph (its `/W` width), in 1/1000 em.
    pub width: f32,
    /// Where the glyph is drawn from the pen, in 1/1000 em (a dot or vowel mark on its letter).
    pub offset: [f32; 2],
    /// The characters this glyph stands for (empty for a cluster's later glyphs).
    pub text: String,
}

/// Longest run shaped at once; longer text is cut (a cell's text, not a document).
const MAX_RUN_CHARS: usize = 32_768;

/// `text` shaped in the Arabic face, in visual (left-to-right) order.
pub(crate) fn shape(text: &str, rtl: bool) -> Vec<Glyph> {
    let Some(face) = face() else { return Vec::new() };
    let Ok(font) = FontRef::new(face.bytes) else { return Vec::new() };
    let text: &str = match text.char_indices().nth(MAX_RUN_CHARS) {
        Some((i, _)) => text.get(..i).unwrap_or(text),
        None => text,
    };
    let shaper = face.shaper.shaper(&font).build();
    let mut buffer = harfrust::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(if rtl { harfrust::Direction::RightToLeft } else { harfrust::Direction::LeftToRight });
    buffer.guess_segment_properties();
    let shaped = shaper.shape(buffer, harfrust::ShapeOptions::new());
    let hmtx = font.hmtx().ok();
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();
    // Each cluster's text goes to the first glyph met for it, so ToUnicode maps it once.
    let mut clusters: Vec<u32> = infos.iter().map(|i| i.cluster).collect();
    clusters.sort_unstable();
    clusters.dedup();
    let cluster_text = |start: u32| -> String {
        let at = clusters.partition_point(|c| *c <= start);
        let end = clusters.get(at).map_or(text.len(), |c| *c as usize);
        text.get(start as usize..end).unwrap_or_default().to_string()
    };
    let mut seen: Vec<u32> = Vec::new();
    let mut out = Vec::with_capacity(infos.len());
    for (info, pos) in infos.iter().zip(positions) {
        let gid = u16::try_from(info.glyph_id).unwrap_or(0);
        let own = hmtx.as_ref().and_then(|h| h.advance(GlyphId::new(u32::from(gid)))).unwrap_or(0);
        let text = if seen.contains(&info.cluster) {
            String::new()
        } else {
            seen.push(info.cluster);
            cluster_text(info.cluster)
        };
        let k = 1000.0 / face.upem;
        out.push(Glyph {
            gid,
            advance: pos.x_advance as f32 * k,
            width: f32::from(own) * k,
            offset: [pos.x_offset as f32 * k, pos.y_offset as f32 * k],
            text,
        });
    }
    out
}

/// Advance of `text` in the Arabic face, in 1/1000 em (0 without the face).
pub(crate) fn width(text: &str, rtl: bool) -> f32 {
    shape(text, rtl).iter().map(|g| g.advance).sum()
}

/// A piece of a line in visual order.
#[derive(Debug, PartialEq)]
pub(crate) enum Piece {
    /// Shaped Arabic glyphs.
    Arabic(Vec<Glyph>),
    /// Other characters, already in visual order, for the standard fonts.
    Plain(String),
}

/// `text` split into pieces in visual (left-to-right) order: bidi runs, and within each run the
/// Arabic and the other characters.
pub(crate) fn pieces(text: &str) -> Vec<Piece> {
    let info = unicode_bidi::ParagraphBidiInfo::new(text, None);
    let (levels, runs) = info.visual_runs(0..text.len());
    let mut out = Vec::new();
    for run in runs {
        let rtl = levels.get(run.start).is_some_and(|l| l.is_rtl());
        let Some(slice) = text.get(run) else { continue };
        // Logical sub-runs of Arabic and of other characters.
        let mut subs: Vec<(bool, &str)> = Vec::new();
        let mut start = 0;
        let mut kind: Option<bool> = None;
        for (i, c) in slice.char_indices() {
            let arabic = is_arabic(c);
            if kind.is_some_and(|k| k != arabic) {
                subs.push((kind.unwrap_or(false), slice.get(start..i).unwrap_or_default()));
                start = i;
            }
            kind = Some(arabic);
        }
        if let Some(k) = kind {
            subs.push((k, slice.get(start..).unwrap_or_default()));
        }
        if rtl {
            subs.reverse();
        }
        for (arabic, sub) in subs {
            out.push(if arabic {
                Piece::Arabic(shape(sub, rtl))
            } else if rtl {
                // Neutral characters inside a right-to-left run read from the right.
                Piece::Plain(sub.chars().rev().map(mirror).collect())
            } else {
                Piece::Plain(sub.to_string())
            });
        }
    }
    out
}

/// The mirrored form of a paired bracket drawn in a right-to-left run.
fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}

/// The glyphs a document used, with the text each stands for (for `/W` and `ToUnicode`).
#[derive(Default)]
pub(crate) struct Used {
    pub glyphs: BTreeMap<u16, (f32, String)>,
}

impl Used {
    /// The glyphs of every page together.
    pub fn merged<'a>(pages: impl IntoIterator<Item = &'a Used>) -> Used {
        let mut all = Used::default();
        for page in pages {
            for (gid, (w, text)) in &page.glyphs {
                let entry = all.glyphs.entry(*gid).or_insert_with(|| (*w, String::new()));
                if entry.1.is_empty() {
                    entry.1.clone_from(text);
                }
            }
        }
        all
    }

    pub fn add(&mut self, glyphs: &[Glyph]) {
        for g in glyphs {
            let entry = self.glyphs.entry(g.gid).or_insert_with(|| (g.width, String::new()));
            if entry.1.is_empty() && !g.text.is_empty() {
                entry.1 = g.text.clone();
            }
        }
    }

    /// The CIDFont `/W` array.
    pub fn widths(&self) -> String {
        let mut s = String::from("[");
        for (gid, (w, _)) in &self.glyphs {
            s.push_str(&format!("{gid} [{}] ", w.round() as i64));
        }
        s.push(']');
        s
    }

    /// The `ToUnicode` CMap stream body.
    pub fn to_unicode(&self) -> String {
        let mut entries: Vec<String> = Vec::new();
        for (gid, (_, text)) in &self.glyphs {
            if text.is_empty() {
                continue;
            }
            let hex: String = text.encode_utf16().map(|u| format!("{u:04X}")).collect();
            entries.push(format!("<{gid:04X}> <{hex}>"));
        }
        let mut s = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        for chunk in entries.chunks(100) {
            s.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for e in chunk {
                s.push_str(e);
                s.push('\n');
            }
            s.push_str("endbfchar\n");
        }
        s.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        s
    }
}

/// `TJ` arrays drawing `glyphs` at `size` points with the shaped advances and offsets. A glyph
/// with an offset (the dots and vowel marks Noto Sans Arabic places on their letters) is drawn
/// on its own, raised with `Ts`, so the pen still moves by the shaped advances only.
pub(crate) fn tj(glyphs: &[Glyph], size: f32) -> String {
    let mut s = String::from("[");
    let num = |v: f32| -> i64 { if v.is_finite() { v.clamp(-1.0e6, 1.0e6).round() as i64 } else { 0 } };
    for g in glyphs {
        let [dx, dy] = g.offset;
        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            s.push_str(&format!("<{:04X}>", g.gid));
            // TJ numbers move the next glyph left by n/1000 em: the face's width minus the shaped advance.
            let adjust = num(g.width - g.advance);
            if adjust != 0 {
                s.push_str(&format!(" {adjust} "));
            }
            continue;
        }
        // Move right by dx, raise by dy, draw, then put the pen where the shaped advance says.
        let rise = crate::n(dy * size / 1000.0);
        s.push_str(&format!("] TJ\n{rise} Ts\n[{} <{:04X}> {}] TJ\n0 Ts\n[", -num(dx), g.gid, num(dx + g.width - g.advance)));
    }
    s.push_str("] TJ");
    s
}
