//! Windows only: embed version info (VERSIONINFO) into the command-line tool (shipped as
//! `jadwal-cli.exe`), so Explorer's Details tab names جدول and its version, as it does for the
//! app (`apps/gridcraft/build.rs`).
//!
//! On every other target this does nothing. A missing resource compiler is a warning, unless
//! `GRIDCRAFT_REQUIRE_WINRES=1` turns it into an error.

#[path = "../gridcraft/src/windows_version.rs"]
mod windows_version;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../gridcraft/src/windows_version.rs");
    println!("cargo:rerun-if-env-changed=GRIDCRAFT_REQUIRE_WINRES");
    println!("cargo:rerun-if-env-changed=JADWAL_VERSION");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let version = windows_version::version(env!("CARGO_PKG_VERSION"));
    let number = windows_version::version_number(&version);
    let mut res = winresource::WindowsResource::new();
    res.set("ProductName", windows_version::PRODUCT_NAME)
        .set("FileDescription", "جدول command-line tool and MCP server")
        .set("CompanyName", windows_version::COMPANY_NAME)
        .set("LegalCopyright", windows_version::COPYRIGHT)
        .set("OriginalFilename", "jadwal-cli.exe")
        .set("InternalName", "jadwal-cli")
        .set("FileVersion", &version)
        .set("ProductVersion", &version)
        .set_version_info(winresource::VersionInfo::FILEVERSION, number)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, number);
    if let Err(e) = res.compile() {
        if std::env::var_os("GRIDCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=jadwal-cli.exe built without version resources: {e}");
    }
}
