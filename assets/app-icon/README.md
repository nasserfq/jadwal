# جدول app icon

The word جدول (outlined Noto Kufi Arabic Black, SIL Open Font License 1.1) over a small table on
a paper page, on the blue tile of مكتب, the family of Nasser Alqahtani's Arabic apps. The table's
header row is blue and one cell is gold, the colour that marks the current app in the مكتب mark.
Made for جدول by Nasser Alqahtani with Claude; it replaces GridCraft's placeholder ledger icon.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| مكتب blue | `#3b5bdb` | The tile and the header row |
| Ink | `#0b0b0c` | The word, outlines and grid lines |
| Paper | `#efe9dc` | The page |
| Gold | `#f5b841` | The selected cell |

## Geometry

A 512 × 512 viewBox, rounded square `rx=112`. Windows and Linux use the full tile. The macOS icons
(`.icns`, `gridcraft-macos-512.png`) sit on Apple's grid: an 824/1024 body with a transparent
margin.

## Files

`gridcraft.svg` is the artwork (the file names keep GridCraft's so the build scripts stay as they
are). `packaging/icons.sh` renders the PNGs and the `.ico`; on Linux the `.icns` is written from
the same renders (PNG entries).
