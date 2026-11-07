//! Embeds the Arabic face for PDF text from the optional craft-fonts build input
//! (`CRAFT_FONTS_DIR`, the same checkout the interface uses). Without it Arabic text keeps the
//! non-embedded CID fallback; nothing is downloaded and no font is in this repository.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=CRAFT_FONTS_DIR");
    let face = std::env::var_os("CRAFT_FONTS_DIR")
        .map(|dir| PathBuf::from(dir).join("fonts/noto-sans-arabic/NotoSansArabic.ttf"))
        .filter(|path| path.is_file());
    let body = match &face {
        Some(path) => {
            println!("cargo:rerun-if-changed={}", path.display());
            format!("pub(crate) static ARABIC_FONT_BYTES: Option<&[u8]> = Some(include_bytes!({:?}));\n", path.display().to_string())
        }
        None => "pub(crate) static ARABIC_FONT_BYTES: Option<&[u8]> = None;\n".to_string(),
    };
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default()).join("arabic_font.rs");
    if let Err(e) = std::fs::write(&out, body) {
        println!("cargo:warning=could not write {}: {e}", out.display());
    }
}
