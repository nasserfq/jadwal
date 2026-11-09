//! Shared by `apps/gridcraft/build.rs` and `apps/gridcraft-cli/build.rs`: the Windows file details
//! (Explorer > Properties > Details) for جدول's executables. They name جدول and its maker, credit
//! GridCraft, and carry the release version (`JADWAL_VERSION`, set by the release workflow), not
//! the GridCraft crate version.

/// Product name shown in the Details tab, the taskbar and Task Manager.
pub const PRODUCT_NAME: &str = "جدول (Jadwal)";
pub const COMPANY_NAME: &str = "Nasser Alqahtani";
pub const COPYRIGHT: &str = "Copyright (c) Nasser Alqahtani. Based on GridCraft, copyright (c) the GridCraft authors. MIT OR Apache-2.0.";

/// The version the executables report: `JADWAL_VERSION` when set, else the crate version.
pub fn version(package: &str) -> String {
    std::env::var("JADWAL_VERSION").ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| package.to_string())
}

/// `MAJOR << 48 | MINOR << 32 | PATCH << 16`, the numeric form of `version`.
pub fn version_number(version: &str) -> u64 {
    let mut parts = version.split(['.', '-', '+']).map(|p| p.parse::<u16>().unwrap_or(0));
    let mut number = 0u64;
    for shift in [48, 32, 16] {
        number |= u64::from(parts.next().unwrap_or(0)) << shift;
    }
    number
}
