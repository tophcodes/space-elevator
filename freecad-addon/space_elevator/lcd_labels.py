"""Build an SVG showing labels for the first N buttons in a row.

Hard-coded layout for v1: 8 cells, equal width across 640px, 150px tall, white
text on dark background. Future iterations can do grids, per-workbench layouts,
icons, etc.
"""

from html import escape

_W = 640
_H = 150
_COLS = 8
_CELL_W = _W // _COLS
_FONT_SIZE = 22
_BG = "#001428"
_FG = "#f0f0f0"
_BORDER = "#274869"


def render(labels):
    """labels: iterable of strings; the first 8 entries are drawn."""
    labels = list(labels)[:_COLS] + [""] * max(0, _COLS - len(labels))
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {_W} {_H}">',
        f'<rect width="{_W}" height="{_H}" fill="{_BG}"/>',
    ]
    for i, lbl in enumerate(labels):
        x = i * _CELL_W
        parts.append(
            f'<rect x="{x}" y="0" width="{_CELL_W}" height="{_H}" fill="none" stroke="{_BORDER}" stroke-width="1"/>'
        )
        cx = x + _CELL_W // 2
        parts.append(
            f'<text x="{cx}" y="{_H//2 + _FONT_SIZE//3}" font-size="{_FONT_SIZE}" '
            f'font-family="sans-serif" fill="{_FG}" text-anchor="middle">{escape(lbl)}</text>'
        )
    parts.append("</svg>")
    return "".join(parts)
