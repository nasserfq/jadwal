<p align="center">
  <img src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.gridcraft.png" width="128" height="128" alt="جدول">
</p>

<h1 align="center">جدول</h1>

<p align="center">
  <b>برنامج جداول بيانات عربي، مجاني ومفتوح المصدر</b><br>
  An Arabic spreadsheet, free and open source
</p>

<p align="center">
  <a href="https://github.com/nasserfq/jadwal/releases/latest"><b>⬇︎ تنزيل · Download</b></a>
  &nbsp;·&nbsp; macOS &nbsp;·&nbsp; Windows &nbsp;·&nbsp; Linux
</p>

---

<div dir="rtl">

## عن جدول

جدول برنامج جداول بيانات مصمم للعربية أولًا: الواجهة عربية ومن اليمين إلى اليسار، والأوراق الجديدة تبدأ من اليمين إلى اليسار، فيكون العمود A على اليمين.

- **واجهة عربية بالكامل** من اليمين إلى اليسار، مع خيار الإنجليزية ولغات أخرى.
- **أوراق من اليمين إلى اليسار** افتراضيًا.
- **ملفات Excel** ‏(‎.xlsx‎) وCSV فتحًا وحفظًا، مع الصيغ والمخططات والجداول والتنسيق الشرطي.

## التثبيت

| النظام | الملف | الخطوات |
|---|---|---|
| macOS ‏(Apple Silicon وIntel، الإصدار 11 فأحدث) | `jadwal-…-macos-universal.dmg` | افتح الملف واسحب جدول إلى «التطبيقات». موقّع وموثَّق من Apple. |
| Windows | `jadwal-…-windows-x64-portable.zip` | فك الضغط وشغّل `Jadwal.exe`. |
| Linux | `jadwal-…-linux-x86_64.tar.gz` | فك الضغط وشغّل `jadwal`. |

التحديث تلقائي: عند صدور إصدار جديد يعرض جدول «تحديث وإعادة تشغيل»، وتبقى إعداداتك وملفاتك كما هي.

## ما أضافه ناصر القحطاني إلى GridCraft

- ترجمة الواجهة كاملة إلى العربية، والعربية لغة افتراضية.
- الواجهة من اليمين إلى اليسار في كل النوافذ والقوائم ومربعات الحوار.
- الأوراق الجديدة من اليمين إلى اليسار افتراضيًا.
- الاسم والشعار ونافذة التثبيت العربية، وتطبيق Mac موقّع وموثَّق لأجهزة Apple Silicon وIntel، وتنزيلات Windows وLinux، والتحديث التلقائي من إصدارات جدول.

## شكر

جدول مبني على [GridCraft](https://github.com/storytold/gridcraft)، مع الشكر لفريق ArtCraft على عملهم الأصلي. جدول مشروع مستقل، وليس من إنتاج فريق ArtCraft ولا برعايته.

</div>

---

## About

Jadwal (جدول) is an Arabic-first spreadsheet by Nasser Alqahtani: an Arabic, right-to-left
interface (English and other languages available), right-to-left sheets by default, and Excel
`.xlsx` and CSV files with formulas, charts, tables and conditional formatting.

Nasser's additions to GridCraft: the Arabic interface translation as the default, the right-to-left
interface in every window, menu and dialog, right-to-left new sheets, the name, logo and Arabic
installer window, the signed universal Mac app and Windows and Linux downloads, and updates from
جدول's own releases.

It is based on [GridCraft](https://github.com/storytold/gridcraft) by the ArtCraft team. Jadwal is
an independent project, not made, sponsored or endorsed by the ArtCraft team.

## Build from source

Install [Rust](https://rustup.rs), then build with the open fonts (Noto Sans Arabic among them) from
[storytold/craft-fonts](https://github.com/storytold/craft-fonts):

```sh
git clone https://github.com/storytold/craft-fonts ../craft-fonts
CRAFT_FONTS_DIR="$(cd ../craft-fonts && pwd)" cargo run --release -p gridcraft
```

Without `CRAFT_FONTS_DIR` the Arabic interface uses an Arabic font installed on the system, if any.

Releases: pushing to the `release` branch builds, signs and publishes the downloads
(`.github/workflows/release.yml`).

## License

MIT OR Apache-2.0, like GridCraft: see [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE),
[NOTICE](NOTICE) and [ATTRIBUTION.md](ATTRIBUTION.md).
