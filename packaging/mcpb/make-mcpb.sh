#!/usr/bin/env bash
# Claude desktop extension (.mcpb) for جدول on macOS: double-clicking it opens Claude's own
# «Install» window, which adds جدول's MCP server to Claude without editing any file.
#
# The extension holds only its description and the جدول logo: it runs the MCP server inside the
# installed app (/Applications/جدول.app/Contents/MacOS/jadwal-cli), so the server updates with the
# app and the file stays a few kilobytes. On other systems, and for other AI apps, جدول's own
# Automate › Connect to AI Apps window sets it up.
#
# usage: packaging/mcpb/make-mcpb.sh OUT_DIR VERSION
set -euo pipefail
OUT="${1:?out dir}"
VERSION="${2:?version}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cp "$ROOT/assets/app-icon/gridcraft-macos-512.png" "$WORK/icon.png"
cat > "$WORK/manifest.json" <<JSON
{
  "manifest_version": "0.3",
  "name": "jadwal",
  "display_name": "جدول (Jadwal)",
  "version": "$VERSION",
  "description": "أنشئ جداول Excel واملأها ونسّقها بكلامك في جدول، برنامج الجداول العربي.",
  "long_description": "جدول برنامج جداول بيانات عربي مبني على GridCraft. بعد تثبيت هذه الإضافة ينشئ Claude المصنفات ويكتب القيم والصيغ وينسّق الخلايا ويضيف المخططات ويحفظها بصيغة Excel أو PDF عندما تطلب منه ذلك. يحتاج جدول مثبتًا في مجلد التطبيقات.\n\nJadwal is an Arabic spreadsheet based on GridCraft. With this extension Claude can build workbooks, enter values and formulas, format cells, add charts and save as Excel or PDF. Needs Jadwal installed in Applications.",
  "author": { "name": "Nasser Alqahtani", "url": "https://github.com/nasserfq/jadwal" },
  "homepage": "https://github.com/nasserfq/jadwal",
  "repository": { "type": "git", "url": "https://github.com/nasserfq/jadwal.git" },
  "license": "MIT OR Apache-2.0",
  "icon": "icon.png",
  "server": {
    "type": "binary",
    "entry_point": "/Applications/جدول.app/Contents/MacOS/jadwal-cli",
    "mcp_config": {
      "command": "/Applications/جدول.app/Contents/MacOS/jadwal-cli",
      "args": ["mcp"]
    }
  },
  "keywords": ["spreadsheet", "excel", "xlsx", "arabic", "جدول"],
  "compatibility": { "platforms": ["darwin"] }
}
JSON
python3 -m json.tool "$WORK/manifest.json" >/dev/null
mkdir -p "$OUT"
rm -f "$OUT/jadwal-claude-extension-macos.mcpb"
(cd "$WORK" && zip -q -X "jadwal-claude-extension-macos.mcpb" manifest.json icon.png)
mv "$WORK/jadwal-claude-extension-macos.mcpb" "$OUT/"
echo "$OUT/jadwal-claude-extension-macos.mcpb"
