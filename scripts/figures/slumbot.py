#!/usr/bin/env python3
"""Slumbot benchmark figures, drawn from the raw hand log.

Emits light/dark SVG pairs for the public README:

  competition-convergence-{light,dark}.svg   running bb/100 vs hands played
  competition-cube-{light,dark}.svg          the (depth, world, dirac) corner cube

Source of truth is `players ⋈ users ⋈ hands` — one row per hand per hero, with
the variant carried by `users.username` (`bot:<variant>`). Chip scale is the
engine's SB=1/BB=2, so bb/100 = 50 × mean(pnl).

    DB_URL=postgresql://... python3 scripts/figures/slumbot.py OUTDIR
"""

import math
import os
import subprocess
import sys

PSQL = "/opt/homebrew/opt/libpq/bin/psql"
WINDOW = ("2026-09-04 16:00", "2026-09-05 17:30")
STRIDE = 100

QUERY = """
WITH x AS (
  SELECT replace(u.username,'bot:','') AS variant, p.pnl::bigint AS pnl,
         row_number() OVER (PARTITION BY u.username
           ORDER BY (('x'||substr(replace(h.id::text,'-',''),1,12))::bit(48)::bigint)) AS n
  FROM players p JOIN users u ON u.id=p.user_id JOIN hands h ON h.id=p.hand_id
  WHERE u.username LIKE 'bot:%%' AND u.username<>'bot:slumbot'
    AND (('x'||substr(replace(h.id::text,'-',''),1,12))::bit(48)::bigint)/1000.0
        BETWEEN extract(epoch from timestamp '%s') AND extract(epoch from timestamp '%s')),
c AS (
  SELECT variant, n, max(n) OVER (PARTITION BY variant) AS mx,
         sum(pnl)     OVER (PARTITION BY variant ORDER BY n) AS cum,
         sum(pnl*pnl) OVER (PARTITION BY variant ORDER BY n) AS cum2
  FROM x)
SELECT variant, n, cum, cum2 FROM c WHERE n %% %d = 0 OR n = mx ORDER BY variant, n;
"""

# ── the variant cube ────────────────────────────────────────────────────────
# corner → (depth, world, dirac); `fish` is the uniform-random control, off-cube.
CORNERS = {
    "base": (0, 0, 0),
    "depth": (1, 0, 0),
    "world": (0, 1, 0),
    "depth+world": (1, 1, 0),
    "dirac": (0, 0, 1),
    "depth+dirac": (1, 0, 1),
    "world+dirac": (0, 1, 1),
    "depth+world+dirac": (1, 1, 1),
}
LEAD = "world+dirac"
REF = "base"
CONTROL = "fish"

# ── themes ──────────────────────────────────────────────────────────────────
# Three categorical slots — aqua = search with dirac, orange = without, blue =
# the uniform-random control. Identity *inside* a group is carried by direct
# labels, never by a lightness ramp: the hue means "which group", nothing else.
# Validated all-pairs in both modes (dataviz `validate_palette.js`); the light
# aqua sits under 3:1 on the surface, which the direct labels relieve.
THEMES = {
    "light": dict(
        surface="#fcfcfb", grid="#e8e7e3", axis="#d6d5d0",
        primary="#0b0b0b", secondary="#52514e", muted="#8a8983",
        on="#1baf7a", off="#eb6834", control="#2a78d6", faint=0.45,
    ),
    "dark": dict(
        surface="#1a1a19", grid="#2b2b29", axis="#3a3a37",
        primary="#ffffff", secondary="#c3c2b7", muted="#8a8983",
        on="#199e70", off="#d95926", control="#3987e5", faint=0.6,
    ),
}
FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif"


class Series:
    """One variant's running bb/100, sampled every STRIDE hands."""

    def __init__(self, name, points):
        self.name = name
        self.pts = points
        self.n = [n for n, _, _ in points]
        self.bb = [50.0 * c / n for n, c, _ in points]
        self.hands = self.n[-1]
        self.final = self.bb[-1]
        n, c, c2 = points[-1]
        self.conf = 1.96 * 50.0 * math.sqrt((c2 - c * c / n) / (n - 1)) / math.sqrt(n)

    def walk(self, lo, count=340):
        """Log-spaced samples of (hands, bb/100, ci) from `lo` on — keeps the
        emitted path small without visibly smoothing the running mean."""
        keep = {min(range(len(self.n)), key=lambda i: abs(math.log10(max(self.n[i], 1)) - g))
                for g in (math.log10(lo) + i * (math.log10(self.hands) - math.log10(lo)) / count
                          for i in range(count + 1))}
        out = []
        for i in sorted(keep):
            n, c, c2 = self.pts[i]
            if n < lo:
                continue
            sd = math.sqrt(max(c2 - c * c / n, 0) / (n - 1))
            out.append((n, self.bb[i], 1.96 * 50.0 * sd / math.sqrt(n)))
        return out

    @property
    def dirac(self):
        return CORNERS.get(self.name, (0, 0, 0))[2] == 1

    def hue(self, t):
        if self.name == CONTROL:
            return t["control"]
        return t["on"] if self.dirac else t["off"]

    def lead(self):
        return self.name in (LEAD, REF)


def pull(url):
    """Run the window query and fold the rows into Series, one per variant."""
    out = subprocess.run(
        [PSQL, url, "-At", "-F,", "-c", QUERY % (WINDOW[0], WINDOW[1], STRIDE)],
        capture_output=True, text=True, check=True,
    ).stdout
    rows = {}
    for line in out.strip().splitlines():
        variant, n, cum, cum2 = line.split(",")
        rows.setdefault(variant, []).append((int(n), int(cum), int(cum2)))
    return {k: Series(k, v) for k, v in rows.items()}


class Svg:
    """Minimal SVG builder — every figure is a list of elements on one canvas."""

    def __init__(self, w, h, t):
        self.w, self.h, self.t = w, h, t
        self.body = []

    def rect(self, x, y, w, h, fill, rx=0):
        self.body.append(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}"/>')

    def line(self, x1, y1, x2, y2, stroke, width=1, dash=None, opacity=1):
        d = f' stroke-dasharray="{dash}"' if dash else ""
        o = f' opacity="{opacity}"' if opacity != 1 else ""
        self.body.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="{stroke}" stroke-width="{width}"{d}{o}/>')

    def path(self, pts, stroke, width, opacity=1):
        d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts)
        self.body.append(f'<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{width}" stroke-linejoin="round" stroke-linecap="round" opacity="{opacity}"/>')

    def dot(self, x, y, r, fill, ring=None):
        if ring:
            self.body.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r + 2:.1f}" fill="{ring}"/>')
        self.body.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="{fill}"/>')

    def text(self, x, y, s, fill, size=12, anchor="start", weight=400, mono=False, halo=False):
        f = "ui-monospace, SFMono-Regular, Menlo, monospace" if mono else FONT
        s = s.replace("&", "&amp;").replace("<", "&lt;")
        h = f' stroke="{self.t["surface"]}" stroke-width="3.5" paint-order="stroke"' if halo else ""
        self.body.append(f'<text x="{x:.1f}" y="{y:.1f}" fill="{fill}" font-family="{f}" font-size="{size}" font-weight="{weight}" text-anchor="{anchor}"{h}>{s}</text>')

    def render(self):
        return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {self.w} {self.h}" '
                f'width="{self.w}" height="{self.h}" font-family="{FONT}">\n'
                + "\n".join(self.body) + "\n</svg>\n")


def fmt(v):
    return f"{v:.1f}".replace("-", "−")


def convergence(data, name):
    """Running bb/100 against hands played — one line per variant, log x."""
    t = THEMES[name]
    W, H = 900, 520
    L, R, T, B = 62, 168, 96, 54
    x0, x1, y0, y1 = L, W - R, T, H - B
    lo, hi = 12_000, 500_000
    top, bot = 30.0, -150.0
    fx = lambda n: x0 + (x1 - x0) * (math.log10(n) - math.log10(lo)) / (math.log10(hi) - math.log10(lo))
    fy = lambda b: y1 - (y1 - y0) * (b - bot) / (top - bot)
    s = Svg(W, H, t)
    s.rect(0, 0, W, H, t["surface"], rx=10)
    s.text(L, 36, "bb/100 against Slumbot", t["primary"], 17, weight=600)
    s.text(L, 57, "running mean by hands played · 2026-09-04 run · 1.9 M hands", t["muted"], 12)
    lx = W - 62
    for label, key in (("uniform random", "control"), ("without dirac", "off"), ("with dirac", "on")):
        s.text(lx, 36, label, t["secondary"], 12, anchor="end")
        lx -= 6.9 * len(label) + 9
        s.dot(lx, 32, 4, t[key])
        lx -= 20
    for b in range(-150, 1, 30):
        s.line(x0, fy(b), x1, fy(b), t["grid"], 1)
        s.text(x0 - 10, fy(b) + 4, fmt(float(b)).rstrip("0").rstrip("."), t["muted"], 11, anchor="end", mono=True)
    for n in (20_000, 50_000, 100_000, 200_000, 500_000):
        s.line(fx(n), y0, fx(n), y1, t["grid"], 1)
        s.text(fx(n), y1 + 20, f"{n // 1000} K", t["muted"], 11, anchor="middle", mono=True)
    s.text((x0 + x1) / 2, y1 + 42, "hands played", t["secondary"], 12, anchor="middle")
    s.line(x0, fy(0), x1, fy(0), t["axis"], 1)
    s.text(x1 - 4, fy(0) - 8, "break-even", t["muted"], 11, anchor="end")
    # series, recessive first so the two protagonists sit on top
    for v in sorted(data.values(), key=lambda v: v.lead()):
        pts = [(fx(n), fy(max(min(b, top), bot))) for n, b, _ in v.walk(lo)]
        s.path(pts, v.hue(t), 2.4 if v.lead() else 1.4, 1 if v.lead() else t["faint"])
    # direct labels, pushed apart so none collide, each tied back to its line
    placed = []
    for y, v in sorted(((fy(v.final), v) for v in data.values()), key=lambda p: p[0]):
        end = y
        y = max(y, (placed[-1] + 30) if placed else y0 + 6)
        placed.append(y)
        s.line(fx(v.hands), end, x1 + 8, y - 4, v.hue(t), 0.9, opacity=0.25)
        if v.lead():  # precision, shown only on the two protagonists
            s.line(fx(v.hands), fy(max(v.final - v.conf, bot)), fx(v.hands), fy(min(v.final + v.conf, top)),
                   v.hue(t), 1.6, opacity=0.5)
        s.dot(fx(v.hands), end, 3, v.hue(t), ring=t["surface"])
        s.dot(x1 + 12, y - 4, 3.5, v.hue(t))
        s.text(x1 + 22, y, v.name, t["primary"] if v.lead() else t["secondary"], 12,
               weight=600 if v.lead() else 400)
        s.text(x1 + 22, y + 14, f"{fmt(v.final)} ± {v.conf:.1f}", t["muted"], 11, mono=True)
    return s.render()


def cube(data, name):
    """The 2×2×2 configuration cube: depth × world × dirac, dirac drawn wide."""
    t = THEMES[name]
    W, H = 900, 430
    ox, oy = 152, 316
    D, O, K = (104, -74), (0, -142), (352, 0)
    at = lambda d, w, k: (ox + d * D[0] + w * O[0] + k * K[0], oy + d * D[1] + w * O[1] + k * K[1])
    s = Svg(W, H, t)
    s.rect(0, 0, W, H, t["surface"], rx=10)
    s.text(48, 36, "the search cube", t["primary"], 17, weight=600)
    s.text(48, 57, "every corner played live against Slumbot · bb/100", t["muted"], 12)
    face = [(0, 0), (1, 0), (1, 1), (0, 1)]
    for k in (0, 1):
        hue = t["on"] if k else t["off"]
        poly = " ".join(f"{x:.1f},{y:.1f}" for x, y in (at(d, w, k) for d, w in face))
        s.body.append(f'<polygon points="{poly}" fill="{hue}" opacity="0.07"/>')
        for i in range(4):
            a, b = face[i], face[(i + 1) % 4]
            s.line(*at(*a, k), *at(*b, k), hue, 1.3, opacity=0.5)
    for d, w in face:  # the four dirac edges carry the story
        a, b = at(d, w, 0), at(d, w, 1)
        gain = data[key_of(d, w, 1)].final - data[key_of(d, w, 0)].final
        s.line(*a, *b, t["axis"], 1.6)
        mx, my = (a[0] + b[0]) / 2, a[1]
        s.rect(mx - 26, my - 12, 52, 22, t["surface"], rx=11)
        s.text(mx, my + 4, f"+{gain:.1f}", t["secondary"], 12.5, anchor="middle", weight=600, mono=True)
    for corner, (d, w, k) in CORNERS.items():
        x, y = at(d, w, k)
        v, lead = data[corner], corner == LEAD
        s.dot(x, y, 7 if lead else 5, v.hue(t), ring=t["surface"])
        anchor, dx = ("start", 15) if k else ("end", -15)
        s.text(x + dx, y - 2, corner, t["primary"] if lead else t["secondary"], 12.5,
               anchor=anchor, weight=600 if lead else 400, halo=True)
        s.text(x + dx, y + 15, fmt(v.final), t["primary"] if lead else t["muted"], 12.5,
               anchor=anchor, weight=600 if lead else 400, mono=True, halo=True)
    s.text(48, H - 26, "↗ depth    ↑ world    → dirac", t["muted"], 12, mono=True)
    s.text(W - 48, H - 26, f"off-cube: {CONTROL} (uniform random) {fmt(data[CONTROL].final)}",
           t["muted"], 12, anchor="end")
    return s.render()


def key_of(d, w, k):
    return next(n for n, c in CORNERS.items() if c == (d, w, k))


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "."
    data = pull(os.environ["DB_URL"])
    for name in THEMES:
        for stem, fig in (("convergence", convergence), ("cube", cube)):
            path = f"{out}/competition-{stem}-{name}.svg"
            open(path, "w").write(fig(data, name))
            print(path)


if __name__ == "__main__":
    main()
