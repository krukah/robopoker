#!/usr/bin/env python3
"""Slumbot benchmark figures, drawn from the raw hand log.

Emits light/dark SVG pairs for the public README:

  competition-convergence-{light,dark}.svg   running bb/100 vs hands played
  competition-cube-{light,dark}.svg          the (depth, world, dirac) corner cube

Source of truth is `players ⋈ users ⋈ hands` — one row per hand per hero, with
the variant carried by `users.username` (`bot:<variant>`). Chip scale is the
engine's SB=1/BB=2, so bb/100 = 50 × mean(pnl).

    DB_URL=postgresql://... python3 scripts/figures/slumbot.py OUTDIR

Rows are cached (SERIES_CACHE, default $TMPDIR/slumbot-series.csv) so that
redrawing — new glyphs, a different axis — costs nothing and does not need the
database's temporary ingress reopened.
"""

import bisect
import math
import os
import pathlib
import subprocess
import sys
import tempfile

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
AXES = (
    ("depth", "depth-limited subgame"),
    ("world", "safe multi-world subgame"),
    ("dirac", "zero-temperature argmax"),
)
SLOT = 15
LEAD = "world+dirac"
REF = "base"
CONTROL = "fish"  # measured as a sanity floor, never plotted

# ── themes ──────────────────────────────────────────────────────────────────
# Three categorical slots — aqua = search with dirac, orange = without, blue =
# the uniform-random control. Identity *inside* a group is carried by direct
# labels, never by a lightness ramp: the hue means "which group", nothing else.
# Validated all-pairs in both modes (dataviz `validate_palette.js`); the light
# aqua sits under 3:1 on the surface, which the direct labels relieve.
THEMES = {
    "light": dict(
        surface="#fcfcfb", panel="#f2f1ed", grid="#e8e7e3", axis="#c9c8c2",
        primary="#0b0b0b", secondary="#52514e", muted="#8a8983",
        on="#1baf7a", off="#eb6834", faint=0.45,
    ),
    "dark": dict(
        surface="#1a1a19", panel="#232322", grid="#2b2b29", axis="#46453f",
        primary="#ffffff", secondary="#c3c2b7", muted="#8a8983",
        on="#199e70", off="#d95926", faint=0.6,
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

    def tail(self, start, near, count=320):
        """Samples log-spaced in hands *remaining* — the resolution a
        right-aligned plot needs, where log-spacing in hands played would
        collapse the whole endgame into one straight segment."""
        out, seen = [], set()
        for i in range(count + 1):
            r = 10 ** (math.log10(start) - i * (math.log10(start) - math.log10(near)) / count)
            j = min(bisect.bisect_left(self.n, self.hands - r), len(self.n) - 1)
            if j in seen:
                continue
            seen.add(j)
            n, c, c2 = self.pts[j]
            sd = math.sqrt(max(c2 - c * c / n, 0) / (n - 1))
            out.append((self.hands - n, self.bb[j], 1.96 * 50.0 * sd / math.sqrt(n)))
        return sorted(out, reverse=True)

    @property
    def dirac(self):
        return CORNERS.get(self.name, (0, 0, 0))[2] == 1

    def hue(self, t):
        return t["on"] if self.dirac else t["off"]

    def lead(self):
        return self.name in (LEAD, REF)


def pull():
    """Fold the window query into one Series per variant, from the database if
    it can be reached and from the cache if it cannot."""
    cache = pathlib.Path(os.environ.get("SERIES_CACHE", tempfile.gettempdir()) ) / "slumbot-series.csv"
    try:
        out = subprocess.run(
            [PSQL, os.environ["DB_URL"], "-At", "-F,", "-c", QUERY % (WINDOW[0], WINDOW[1], STRIDE)],
            capture_output=True, text=True, check=True, timeout=180,
        ).stdout
        cache.write_text(out)
    except (KeyError, subprocess.SubprocessError) as e:
        if not cache.exists():
            raise
        print(f"{type(e).__name__}: falling back to {cache}", file=sys.stderr)
        out = cache.read_text()
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

    def rect(self, x, y, w, h, fill, rx=0, stroke=None, opacity=1):
        k = f' stroke="{stroke}"' if stroke else ""
        self.body.append(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}"{k} opacity="{opacity}"/>')

    def line(self, x1, y1, x2, y2, stroke, width=1, opacity=1):
        self.body.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="{stroke}" stroke-width="{width}" opacity="{opacity}"/>')

    def path(self, pts, stroke, width, opacity=1):
        d = "M" + " L".join(f"{x:.1f},{y:.1f}" for x, y in pts)
        self.body.append(f'<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{width}" stroke-linejoin="round" stroke-linecap="round" opacity="{opacity}"/>')

    def dot(self, x, y, r, fill, ring=None):
        if ring:
            self.body.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r + 2:.1f}" fill="{ring}"/>')
        self.body.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="{fill}"/>')

    def text(self, x, y, s, fill, size=12, anchor="start", weight=400, mono=False):
        f = "ui-monospace, SFMono-Regular, Menlo, monospace" if mono else FONT
        s = s.replace("&", "&amp;").replace("<", "&lt;")
        self.body.append(f'<text x="{x:.1f}" y="{y:.1f}" fill="{fill}" font-family="{f}" font-size="{size}" '
                         f'font-weight="{weight}" text-anchor="{anchor}">{s}</text>')

    def glyph(self, kind, x, y, color, on=True):
        """One degree of freedom, drawn as the thing it does.

        depth and world are a mirrored pair, and the mirror is where the bar
        sits: depth branches down onto a floor, world hangs down from a lid.

        depth  a tree splitting downward, stopped by a rule beneath it — the
               search descends only so far, then hands the rest to a leaf value
        world  a bar over three equal branches — one commitment held across
               every world, so the opponent cannot pick the one left exposed
        dirac  a symmetric distribution with a dot on its tallest bar — the
               argmax lifted out of the policy instead of sampled from it
        """
        c = color if on else self.t["axis"]
        o = 1 if on else 0.55
        g = [f'<g transform="translate({x:.1f},{y:.1f})" opacity="{o}">']
        k = f'fill="none" stroke="{c}" stroke-linecap="round" stroke-linejoin="round"'
        if kind == "depth":
            g.append(f'<path d="M0,-6.4 L0,-2.6 M0,-2.6 L-4.1,1.2 M0,-2.6 L4.1,1.2" {k} stroke-width="1.5"/>'
                     f'<path d="M-5.8,4.4 L5.8,4.4" {k} stroke-width="1.5"/>')
        elif kind == "world":
            g.append(f'<path d="M-5.8,-4.6 L5.8,-4.6" {k} stroke-width="1.5"/>'
                     f'<path d="M-4.1,-4.4 L-4.1,4.6 M0,-4.4 L0,4.6 M4.1,-4.4 L4.1,4.6" '
                     f'{k} stroke-width="1.5"/>')
        elif kind == "dirac":
            for bx, h in ((-4.1, 3.6), (0, 9.2), (4.1, 3.6)):  # same lanes as world
                g.append(f'<path d="M{bx},4.6 L{bx},{4.6 - h:.1f}" {k} stroke-width="1.5"/>')
            g.append(f'<circle cx="0" cy="-6.4" r="1.6" fill="{c}"/>')
        self.body.append("".join(g) + "</g>")

    def slots(self, x, y, corner, hue):
        """The three DOF in fixed order, so any two pills line up column by column."""
        for i, ((kind, _), on) in enumerate(zip(AXES, corner)):
            self.glyph(kind, x + 7 + i * SLOT, y, hue, bool(on))

    def pill(self, x, y, corner, hue, value, lead=False, anchor="start"):
        """Identity and result in one small panel: the DOF slots say which
        variant this is — spelling the name out beside them says it twice.

        Every measurement is padded to the same width (see `stat`), so every
        pill comes out the same size and both rows can sit centred: the slots
        stack into one column down the figure, the ± lines up beneath them.
        """
        t = self.t
        w, h = max(3 * SLOT, 6.4 * len(value)) + 20, 38
        x = x if anchor == "start" else x - w
        self.rect(x, y - h / 2, w, h, t["panel"], rx=9)
        self.rect(x, y - h / 2, w, h, hue, rx=9, opacity=0.10)
        self.slots(x + w / 2 - 1.5 * SLOT, y - 8, corner, hue)
        self.text(x + w / 2, y + 13, value, t["primary"] if lead else t["muted"], 11,
                  weight=600 if lead else 400, mono=True, anchor="middle")
        return w

    def key(self, x, y, arrows=None):
        """The glyphs are not self-evident, so the key says outright what each
        one means. It gives no short name: `depth` / `world` / `dirac` are
        labels for the feature, and the feature is right there in words. Nor
        does it gloss solid-versus-ghosted — a filled mark reading as present
        needs no caption."""
        t = self.t
        for i, (kind, gloss) in enumerate(AXES):
            self.glyph(kind, x + 7, y - 4, t["secondary"])
            if arrows:
                self.text(x + 20, y, arrows[i], t["muted"], 11.5)
            self.text(x + (37 if arrows else 20), y, gloss, t["secondary"], 11.5)
            x += (56 if arrows else 39) + 6.3 * len(gloss)

    def render(self):
        return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {self.w} {self.h}" '
                f'width="{self.w}" height="{self.h}" font-family="{FONT}">\n'
                + "\n".join(self.body) + "\n</svg>\n")


def fmt(v):
    return f"{v:.1f}".replace("-", "\u2212")


def stat(v):
    """`−13.1 ± 14.0`, padded with figure spaces to a fixed twelve columns —
    what makes every pill one width and the ± align down the column."""
    return f"{fmt(v.final):\u2007>5} ± {f'{v.conf:.1f}':\u2007>4}"


def convergence(data, name):
    """Running bb/100, every series aligned on its own last hand.

    The x-axis is hands *remaining*, so the estimates all land on the right
    edge at the value Table 1 reports and a longer run simply reaches further
    left — rather than the short runs stopping dead in the middle of the plot.
    """
    t = THEMES[name]
    W, H = 900, 520
    L, R, T, B = 62, 128, 108, 56
    x0, x1, y0, y1 = L, W - R, T, H - B
    near, far, step = 1_000, 480_000, 15
    curve = {v.name: v.tail(v.hands - 20_000, near) for v in drawn(data).values()}
    flat = [b for c in curve.values() for _, b, _ in c]
    top = step * math.ceil(max(flat) / step)
    bot = step * math.floor(min(flat) / step)
    span = math.log10(far) - math.log10(near)
    fx = lambda r: x1 - (x1 - x0) * (math.log10(max(r, near)) - math.log10(near)) / span
    fy = lambda b: y1 - (y1 - y0) * (b - bot) / (top - bot)
    s = Svg(W, H, t)
    s.rect(0, 0, W, H, t["surface"], rx=10)
    s.text(L, 36, "bb/100 against Slumbot", t["primary"], 17, weight=600)
    s.text(L, 57, "running mean, aligned on the last hand of each run · 2026-09-04 · 1.9 M hands", t["muted"], 12)
    s.key(L - 7, 84)
    for b in range(int(bot), int(top) + 1, step):
        s.line(x0, fy(b), x1, fy(b), t["grid"], 1)
        s.text(x0 - 10, fy(b) + 4, fmt(float(b)).rstrip("0").rstrip("."), t["muted"], 11, anchor="end", mono=True)
    for r in (300_000, 100_000, 30_000, 10_000, 3_000, 1_000):
        s.line(fx(r), y0, fx(r), y1, t["grid"], 1)
        s.text(fx(r), y1 + 20, f"{r // 1000} K", t["muted"], 11, anchor="middle", mono=True)
    s.text((x0 + x1) / 2, y1 + 42, "hands remaining  →  end of run", t["secondary"], 12, anchor="middle")
    s.line(x0, fy(0), x1, fy(0), t["axis"], 1)
    s.text(x0 + 8, fy(0) + 15, "break-even", t["muted"], 11)
    for v in sorted(drawn(data).values(), key=lambda v: v.lead()):
        pts = [(fx(r), fy(b)) for r, b, _ in curve[v.name]]
        s.path(pts, v.hue(t), 2.4 if v.lead() else 1.4, 1 if v.lead() else t["faint"])
        s.dot(*pts[0], 2.6, t["surface"], ring=v.hue(t))  # where this run enters
    # one pill per variant, pushed apart to clear each other and then, if the
    # column ran past the axis, lifted back inside it as a block
    order = sorted(((fy(v.final), v) for v in drawn(data).values()), key=lambda p: p[0])
    rows, gap = [], 42
    for end, _ in order:
        rows.append(max(end, rows[-1] + gap) if rows else end)
    lift = max(0, rows[-1] + gap / 2 - y1)
    for (end, v), y in zip(order, rows):
        y = max(y - lift, y0 + gap / 2)
        s.line(x1, end, x1 + 14, y, v.hue(t), 0.9, opacity=0.3)
        s.dot(x1, end, 3, v.hue(t), ring=t["surface"])
        s.pill(x1 + 14, y, CORNERS[v.name], v.hue(t), stat(v), v.lead())
    return s.render()


def cube(data, name):
    """The 2×2×2 configuration cube: depth × world × dirac, dirac drawn wide."""
    t = THEMES[name]
    W, H = 900, 400
    ox, oy = 230, 344
    D, O, K = (100, -70), (0, -136), (340, 0)
    at = lambda d, w, k: (ox + d * D[0] + w * O[0] + k * K[0], oy + d * D[1] + w * O[1] + k * K[1])
    s = Svg(W, H, t)
    s.rect(0, 0, W, H, t["surface"], rx=10)
    s.text(48, 36, "the search cube", t["primary"], 17, weight=600)
    s.text(48, 57, "every corner played live against Slumbot · bb/100 ± 95% CI", t["muted"], 12)
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
        mx = (a[0] + b[0]) / 2
        s.rect(mx - 28, a[1] - 12, 56, 24, t["panel"], rx=12)
        s.text(mx, a[1] + 4, f"+{gain:.1f}", t["secondary"], 12.5, anchor="middle", weight=600, mono=True)
    rail = (ox - 22, ox + D[0] + K[0] + 22)  # pills flank the cube, never cover it
    for corner, (d, w, k) in CORNERS.items():
        x, y = at(d, w, k)
        v, lead = data[corner], corner == LEAD
        s.line(rail[k], y, x, y, v.hue(t), 0.9, opacity=0.3)
        s.dot(x, y, 7 if lead else 5, v.hue(t), ring=t["surface"])
        s.pill(rail[k], y, (d, w, k), v.hue(t), stat(v), lead=lead, anchor="start" if k else "end")
    s.key(41, 84, arrows=("↗", "↑", "→"))
    return s.render()


def drawn(data):
    """The eight cube corners. `fish` is a sanity floor, not a search variant —
    it is measured and tabled, but plotting it only stretches the y-axis."""
    return {k: v for k, v in data.items() if k in CORNERS}


def key_of(d, w, k):
    return next(n for n, c in CORNERS.items() if c == (d, w, k))


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "."
    data = pull()
    for name in THEMES:
        for stem, fig in (("convergence", convergence), ("cube", cube)):
            path = f"{out}/competition-{stem}-{name}.svg"
            open(path, "w").write(fig(data, name))
            print(path)


if __name__ == "__main__":
    main()
