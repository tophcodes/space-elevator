/* =====================================================================
   SPACE ELEVATOR — SpaceMouse Enterprise LCD layout template
   ---------------------------------------------------------------------
   Panel:  640 x 150 px, 16-bit R5G6B5 colour.
   Layout: two 6-key clusters (left + right), each 3 cols x 2 rows,
           with a reserved centre gutter for a profile / mode label.
           => 12 tiles, mirroring the 12 Intelligent Function Keys.

   PUBLIC API
     renderLCD(config, themeName)  -> returns an <svg> string (640x150)
     THEMES                        -> { console, signal, paper }
     ICONS                         -> name -> fn(opts) glyph library

   The whole renderer is dependency-free. Drop this file straight into
   the driver and call renderLCD() with your live button assignments.
   ===================================================================== */

(function (global) {
  'use strict';

  var W = 640, H = 150;

  /* --- geometry helpers --------------------------------------------- */
  function arrow(cx, cy, ang, c, sz) {           // small filled arrowhead
    sz = sz || 4;
    var a = ang * Math.PI / 180;
    function p(d, o) {
      return [
        cx + Math.cos(a) * d - Math.sin(a) * o,
        cy + Math.sin(a) * d + Math.cos(a) * o
      ].map(function (n) { return n.toFixed(2); }).join(',');
    }
    return '<polygon points="' + p(0, 0) + ' ' + p(-sz, sz * 0.62) + ' ' +
           p(-sz, -sz * 0.62) + '" fill="' + c + '"/>';
  }
  function hexPts(cx, cy, r) {
    var o = [];
    for (var i = 0; i < 6; i++) {
      var a = (i * 60) * Math.PI / 180;
      o.push((cx + r * Math.cos(a)).toFixed(2) + ',' + (cy + r * Math.sin(a)).toFixed(2));
    }
    return o.join(' ');
  }

  /* --- ICON LIBRARY -------------------------------------------------- *
     Every glyph is authored in a 24x24 box and built from simple
     primitives only (lines, circles, rects, polygons, arcs).
     opts = { c1: stroke, c2: accent, sw: stroke-width, fill: shape fill }
   * ------------------------------------------------------------------ */
  var ICONS = {
    line: function (o) {
      return L(4, 20, 20, 4, o.c1, o.sw) + dot(4, 20, o.c2) + dot(20, 4, o.c2);
    },
    point: function (o) {
      return L(12, 4, 12, 20, o.c1, o.sw * 0.7) + L(4, 12, 20, 12, o.c1, o.sw * 0.7) +
             '<circle cx="12" cy="12" r="3.2" fill="' + o.c2 + '"/>';
    },
    rectangle: function (o) {
      return '<rect x="4" y="6.5" width="16" height="11" rx="1.4" fill="' + o.fill +
             '" stroke="' + o.c1 + '" stroke-width="' + o.sw + '"/>' +
             dot(4, 6.5, o.c2) + dot(20, 17.5, o.c2);
    },
    circle: function (o) {
      return '<circle cx="12" cy="12" r="8" fill="' + o.fill + '" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/><circle cx="12" cy="12" r="1.5" fill="' + o.c2 + '"/>';
    },
    arc: function (o) {
      return '<path d="M4 18.5 A 11 11 0 0 1 20 18.5" fill="none" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/>' + dot(4, 18.5, o.c2) + dot(20, 18.5, o.c2) +
             dot(12, 18.5, o.c2);
    },
    polygon: function (o) {
      return '<polygon points="' + hexPts(12, 12, 8.5) + '" fill="' + o.fill +
             '" stroke="' + o.c1 + '" stroke-width="' + o.sw + '" stroke-linejoin="round"/>';
    },
    slot: function (o) {
      return '<rect x="3" y="8" width="18" height="8" rx="4" fill="' + o.fill +
             '" stroke="' + o.c1 + '" stroke-width="' + o.sw + '"/>' +
             dot(7, 12, o.c2) + dot(17, 12, o.c2);
    },
    trim: function (o) {
      return L(3.5, 12, 20.5, 12, o.c1, o.sw) +
             L(9, 9, 15, 15, o.c2, o.sw) + L(15, 9, 9, 15, o.c2, o.sw);
    },
    construction: function (o) {
      return '<rect x="4" y="6.5" width="16" height="11" rx="1" fill="none" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '" stroke-dasharray="3 2.4"/>' +
             dot(4, 6.5, o.c2) + dot(20, 17.5, o.c2);
    },
    extend: function (o) {
      return L(3.5, 12, 14, 12, o.c1, o.sw) +
             '<path d="M14 12 L18.5 12" stroke="' + o.c2 + '" stroke-width="' + o.sw +
             '" stroke-dasharray="2 2"/>' + arrow(19.5, 12, 0, o.c2, 3.6) +
             L(21, 6.5, 21, 17.5, o.c1, o.sw);
    },
    coincident: function (o) {
      return '<circle cx="12" cy="12" r="6.2" fill="none" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/><circle cx="12" cy="12" r="2.6" fill="' + o.c2 + '"/>';
    },
    horizontal: function (o) {
      return L(3.5, 12, 20.5, 12, o.c1, o.sw) + dot(3.5, 12, o.c2) + dot(20.5, 12, o.c2);
    },
    vertical: function (o) {
      return L(12, 3.5, 12, 20.5, o.c1, o.sw) + dot(12, 3.5, o.c2) + dot(12, 20.5, o.c2);
    },
    parallel: function (o) {
      return L(6, 20, 13, 4, o.c1, o.sw) + L(12, 20, 19, 4, o.c1, o.sw);
    },
    perpendicular: function (o) {
      return L(6, 3.5, 6, 18, o.c1, o.sw) + L(6, 18, 20.5, 18, o.c1, o.sw) +
             '<rect x="6" y="13.5" width="4.5" height="4.5" fill="none" stroke="' + o.c2 +
             '" stroke-width="' + (o.sw * 0.8) + '"/>';
    },
    equal: function (o) {
      return L(5, 9.5, 19, 9.5, o.c1, o.sw) + L(5, 14.5, 19, 14.5, o.c1, o.sw) +
             dot(5, 9.5, o.c2) + dot(19, 14.5, o.c2);
    },
    tangent: function (o) {
      return '<circle cx="9.5" cy="14.5" r="6" fill="' + o.fill + '" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/>' + L(2.5, 8.5, 21.5, 8.5, o.c1, o.sw) +
             dot(9.5, 8.5, o.c2);
    },
    dimension: function (o) {
      return L(4, 6.5, 4, 18, o.c1, o.sw * 0.8) + L(20, 6.5, 20, 18, o.c1, o.sw * 0.8) +
             L(4, 13.5, 20, 13.5, o.c1, o.sw) +
             arrow(4.5, 13.5, 180, o.c2, 3.6) + arrow(19.5, 13.5, 0, o.c2, 3.6);
    },
    radius: function (o) {
      return '<circle cx="11" cy="13" r="8" fill="' + o.fill + '" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/>' + L(11, 13, 18.5, 8, o.c1, o.sw) +
             arrow(18.5, 8, -34, o.c2, 3.6) + dot(11, 13, o.c2);
    },
    angle: function (o) {
      return L(5, 20, 21, 20, o.c1, o.sw) + L(5, 20, 19, 7, o.c1, o.sw) +
             '<path d="M16 20 A 11 11 0 0 0 12.2 12.2" fill="none" stroke="' + o.c2 +
             '" stroke-width="' + (o.sw * 0.85) + '"/>';
    },
    fillet: function (o) {
      return '<path d="M6 4 L6 13 Q6 18 11 18 L20 18" fill="none" stroke="' + o.c1 +
             '" stroke-width="' + o.sw + '"/>' +
             '<path d="M6 18 L6 18 M6 18 L20 18" fill="none"/>' +
             '<path d="M6 13 L6 18 L11 18" fill="none" stroke="' + o.c2 +
             '" stroke-width="' + (o.sw * 0.8) + '" stroke-dasharray="1.6 1.8"/>';
    },
    fit: function (o) {
      var k = function (x, y, rx, ry) {
        return '<path d="M' + x + ' ' + (y + 5 * ry) + ' L' + x + ' ' + y + ' L' +
               (x + 5 * rx) + ' ' + y + '" fill="none" stroke="' + o.c1 +
               '" stroke-width="' + o.sw + '" stroke-linecap="round"/>';
      };
      return k(4, 4, 1, 1) + k(20, 4, -1, 1) + k(4, 20, 1, -1) + k(20, 20, -1, -1) +
             '<rect x="9.5" y="9.5" width="5" height="5" rx="0.8" fill="' + o.c2 + '"/>';
    }
  };

  function L(x1, y1, x2, y2, c, w) {
    return '<line x1="' + x1 + '" y1="' + y1 + '" x2="' + x2 + '" y2="' + y2 +
           '" stroke="' + c + '" stroke-width="' + w + '" stroke-linecap="round"/>';
  }
  function dot(x, y, c) { return '<circle cx="' + x + '" cy="' + y + '" r="2.1" fill="' + c + '"/>'; }

  /* --- CATEGORY PALETTES -------------------------------------------- *
     Hues share chroma/lightness; only the hue changes (oklch-derived),
     so the four families read as one system. Dark + light variants.
   * ------------------------------------------------------------------ */
  var CAT_DARK  = { draw: '#4DA3FF', modify: '#FFB454', constrain: '#46D08A', view: '#B98CFF', cut: '#FF6B6B' };
  var CAT_LIGHT = { draw: '#1F6FE0', modify: '#B5760C', constrain: '#0E9152', view: '#7A47CF', cut: '#D7453F' };

  /* --- THEMES -------------------------------------------------------- */
  var THEMES = {
    console: {
      name: 'Console',
      blurb: 'Near-black panel, hairline outline icons. Restrained, instrument-like.',
      bg: '#0C0D10', cat: CAT_DARK,
      tileFill: 'none', tileStroke: 'none', tileRadius: 0,
      label: '#C7CBD3', labelWeight: 500, labelSize: 13.5,
      icon: { sw: 1.7, fill: 'none', mono: false },     // mono:false => icon in category colour
      iconBaseColor: null,                              // null => use category colour
      gutterLine: '#22252D', profile: '#5E646F', mode: '#AEB4BF',
      active: { fill: 'cat', fg: '#0C0D10', radius: 9 }
    },
    signal: {
      name: 'Signal',
      blurb: 'Dark with solid, category-filled glyphs on soft tiles. Bold and scannable.',
      bg: '#0A0B0D', cat: CAT_DARK,
      tileFill: '#15171C', tileStroke: 'none', tileRadius: 9,
      label: '#E6E9EF', labelWeight: 600, labelSize: 13.5,
      icon: { sw: 2.3, fillAlpha: '33', mono: false },  // translucent category fill behind strokes
      iconBaseColor: null,
      gutterLine: '#20242C', profile: '#6A7180', mode: '#D6DBE4',
      active: { fill: 'cat', fg: '#0A0B0D', radius: 9 }
    },
    paper: {
      name: 'Paper',
      blurb: 'Warm off-white panel, two-tone icons (grey body + colour accent). Daylight-friendly.',
      bg: '#ECEAE4', cat: CAT_LIGHT,
      tileFill: 'none', tileStroke: 'none', tileRadius: 0,
      label: '#37332C', labelWeight: 600, labelSize: 13.5,
      icon: { sw: 1.8, fillAlpha: '1f', mono: true },   // mono base + category accent
      iconBaseColor: '#615C52',
      gutterLine: '#CFCAC0', profile: '#7A746A', mode: '#37332C',
      active: { fill: 'cat', fg: '#FCFBF8', radius: 9 }
    }
  };

  /* --- DEFAULT DEMO CONFIG ------------------------------------------ *
     Edit this block (or pass your own) to drive the layout.
     Each tile:  { label, icon, cat, active? }
       label  - string shown under the glyph
       icon   - key into ICONS
       cat    - 'draw' | 'modify' | 'constrain' | 'view'  (drives colour)
       active - true to render the "mode active" highlighted state
   * ------------------------------------------------------------------ */
  var DEMO = {
    profile: 'FreeCAD',
    mode: 'Sketch',
    left: [                                   // cluster 1  (3x2)  — DRAW
      { label: 'Line',       icon: 'line',         cat: 'draw' },
      { label: 'Rectangle',  icon: 'rectangle',    cat: 'draw' },
      { label: 'Circle',     icon: 'circle',       cat: 'draw' },
      { label: 'Arc',        icon: 'arc',          cat: 'draw' },
      { label: 'Trim',       icon: 'trim',         cat: 'modify' },
      { label: 'Construction', icon: 'construction', cat: 'modify', active: true }
    ],
    right: [                                  // cluster 2  (3x2)  — CONSTRAIN
      { label: 'Coincident', icon: 'coincident',   cat: 'constrain' },
      { label: 'Horizontal', icon: 'horizontal',   cat: 'constrain' },
      { label: 'Vertical',   icon: 'vertical',     cat: 'constrain' },
      { label: 'Parallel',   icon: 'parallel',     cat: 'constrain' },
      { label: 'Equal',      icon: 'equal',        cat: 'constrain' },
      { label: 'Dimension',  icon: 'dimension',    cat: 'view' }
    ]
  };

  /* --- RENDERER ------------------------------------------------------ */
  function esc(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
  function hex(c, a) { return c + a; }         // append alpha byte to #rrggbb

  function renderTile(it, x, y, w, h, t) {
    var cat = t.cat[it.cat] || t.label;
    var out = [];
    var pad = 4, inX = x + pad, inY = y + 5, inW = w - pad * 2, inH = h - 10;

    // tile background (Signal) or active highlight
    if (it.active) {
      out.push('<rect x="' + inX.toFixed(1) + '" y="' + inY.toFixed(1) + '" width="' + inW.toFixed(1) +
        '" height="' + inH.toFixed(1) + '" rx="' + t.active.radius + '" fill="' + cat + '"/>');
    } else if (t.tileFill !== 'none') {
      out.push('<rect x="' + inX.toFixed(1) + '" y="' + inY.toFixed(1) + '" width="' + inW.toFixed(1) +
        '" height="' + inH.toFixed(1) + '" rx="' + t.tileRadius + '" fill="' + t.tileFill + '"/>');
    }

    // icon colours by theme
    var fg = it.active ? t.active.fg : null;
    var c1, c2, fill;
    if (fg) {                                   // active: monochrome on accent
      c1 = fg; c2 = fg; fill = hex(fg, '30');
    } else if (t.icon.mono) {                    // two-tone: grey body + colour accent
      c1 = t.iconBaseColor; c2 = cat;
      fill = t.icon.fillAlpha ? hex(cat, t.icon.fillAlpha) : 'none';
    } else {                                      // category-coloured glyph
      c1 = cat; c2 = cat;
      fill = t.icon.fillAlpha ? hex(cat, t.icon.fillAlpha) : (t.icon.fill || 'none');
    }

    var iconSize = 27, s = iconSize / 24;
    var cxp = x + w / 2, iconCY = y + h * 0.36;
    var glyph = (ICONS[it.icon] || ICONS.point)({ c1: c1, c2: c2, sw: t.icon.sw, fill: fill });
    out.push('<g transform="translate(' + (cxp - 12 * s).toFixed(2) + ',' + (iconCY - 12 * s).toFixed(2) +
      ') scale(' + s.toFixed(3) + ')" stroke-linecap="round" stroke-linejoin="round">' + glyph + '</g>');

    // label
    var lblColor = it.active ? t.active.fg : t.label;
    out.push('<text x="' + cxp.toFixed(1) + '" y="' + (y + h * 0.80).toFixed(1) +
      '" text-anchor="middle" font-family="\'Source Sans 3\',\'DejaVu Sans\',\'Segoe UI\',sans-serif"' +
      ' font-size="' + t.labelSize + '" font-weight="' + t.labelWeight + '" letter-spacing="0.1"' +
      ' fill="' + lblColor + '">' + esc(it.label) + '</text>');

    return out.join('');
  }

  function cluster(items, x0, x1, t) {
    var top = 6, bottom = H - 6;
    var colW = (x1 - x0) / 3, rowH = (bottom - top) / 2, out = [];
    for (var i = 0; i < items.length && i < 6; i++) {
      var col = i % 3, row = Math.floor(i / 3);
      out.push(renderTile(items[i], x0 + col * colW, top + row * rowH, colW, rowH, t));
    }
    return out.join('');
  }

  function renderLCD(config, themeName) {
    config = config || DEMO;
    var t = THEMES[themeName] || THEMES.console;
    var gutterW = 40, cx = W / 2;
    var gX0 = cx - gutterW / 2, gX1 = cx + gutterW / 2;
    var leftX0 = 6, leftX1 = gX0;
    var rightX0 = gX1, rightX1 = W - 6;

    var svg = [];
    svg.push('<svg xmlns="http://www.w3.org/2000/svg" width="' + W + '" height="' + H +
      '" viewBox="0 0 ' + W + ' ' + H + '" shape-rendering="geometricPrecision">');
    svg.push('<rect width="' + W + '" height="' + H + '" fill="' + t.bg + '"/>');

    svg.push(cluster(config.left || [], leftX0, leftX1, t));
    svg.push(cluster(config.right || [], rightX0, rightX1, t));

    // centre gutter: hairline + rotated profile / mode label
    svg.push('<line x1="' + cx + '" y1="20" x2="' + cx + '" y2="' + (H - 20) +
      '" stroke="' + t.gutterLine + '" stroke-width="1"/>');
    svg.push('<g transform="translate(' + cx + ',' + (H / 2) + ') rotate(-90)"' +
      ' font-family="\'Source Sans 3\',\'DejaVu Sans\',sans-serif" text-anchor="middle">');
    svg.push('<text x="0" y="-4.5" font-size="9.5" font-weight="600" letter-spacing="2"' +
      ' fill="' + t.profile + '">' + esc((config.profile || '').toUpperCase()) + '</text>');
    svg.push('<text x="0" y="8.5" font-size="13" font-weight="700" letter-spacing="0.5"' +
      ' fill="' + t.mode + '">' + esc(config.mode || '') + '</text>');
    svg.push('</g>');

    svg.push('</svg>');
    return svg.join('');
  }

  /* --- exports ------------------------------------------------------- */
  var API = { renderLCD: renderLCD, THEMES: THEMES, ICONS: ICONS, DEMO: DEMO,
              CAT_DARK: CAT_DARK, CAT_LIGHT: CAT_LIGHT };
  if (typeof module !== 'undefined' && module.exports) module.exports = API;
  global.LCD = API;

})(typeof window !== 'undefined' ? window : this);
