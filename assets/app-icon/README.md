# جدول app icon

The word جدول (outlined Noto Kufi Arabic Black, SIL Open Font License 1.1) over a small table on
a paper page, on a green tile (the GridCraft green of the ArtCraft apps). The table's
header row is green and one cell is gold.
Made for جدول by Nasser Alqahtani with Claude; it replaces GridCraft's placeholder ledger icon.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| GridCraft green | `#1f9d55` | The tile and the header row |
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
