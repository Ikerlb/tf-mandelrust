# /// script
# requires-python = ">=3.9"
# dependencies = ["numpy"]
# ///
"""Generates resources/src/targets.rs: points on the Mandelbrot boundary worth zooming into.

Minibrot targets are nuclei (centers) of small copies of the set, found with Newton's
method on z_p(c) = 0. Their max_zoom frames the copy, from its estimated size.
Misiurewicz targets solve z_{k+p}(c) = z_k(c): spiral and star centers that look the same
at any depth, so their max_zoom is where f64 precision runs out. Only ones with many arms
are kept; the rest are a lone filament on a blank background.

Run: uv run tools/gen_targets.py
"""
import math
from pathlib import Path

import numpy as np

SEED = 1
MINIBROTS_PER_DECADE = 5   # per factor of 10 in max_zoom, from 1e3 up to MAX_ZOOM
MISIUREWICZ = 8
MAX_ZOOM = 5e10            # f64 runs out of precision not far past this
ITERATIONS_PER_PERIOD = 40 # below this a minibrot renders as a lump instead of a cardioid
# Render time is ~3 ms per average iteration per pixel on a 2.6 GHz core, so 700 is ~2 s;
# Lambda at 1024 MB gets ~0.6 of a core, which keeps the slowest frame well under the
# 10 s timeout.
MAX_ITERATIONS_PER_PIXEL = 700
OUT = Path(__file__).resolve().parent.parent / "resources/src/targets.rs"

rng = np.random.default_rng(SEED)


def boundary_seeds(count):
    """Random points just outside the set, i.e. close to its boundary."""
    c = rng.uniform(-2, 0.5, count) + 1j * rng.uniform(-1.2, 1.2, count)
    z = np.zeros_like(c)
    n = np.full(c.shape, -1)
    for i in range(400):
        live = n < 0
        z[live] = z[live] ** 2 + c[live]
        n[live & (np.abs(z) > 2)] = i
    return c[n > 40]


def newton(c, f, steps=80):
    """Vectorised Newton's method; f(c) returns (value, derivative)."""
    with np.errstate(all="ignore"):
        for _ in range(steps):
            v, dv = f(c)
            c = c - v / dv
    return c


def orbit(c, n):
    """z_0..z_n of 0 under z^2 + c, and their derivatives with respect to c."""
    zs, dzs = [np.zeros_like(c)], [np.zeros_like(c)]
    with np.errstate(all="ignore"):
        for _ in range(n):
            dzs.append(2 * zs[-1] * dzs[-1] + 1)
            zs.append(zs[-1] ** 2 + c)
    return zs, dzs


def find_nuclei(p, seeds):
    def f(c):
        zs, dzs = orbit(c, p)
        return zs[p], dzs[p]

    c = newton(seeds, f)
    zs, _ = orbit(c, p)
    ok = np.isfinite(c) & (np.abs(zs[p]) < 1e-12)
    for q in range(1, p):
        ok &= np.abs(zs[q]) > 1e-9  # period is exactly p
    return c[ok]


def find_misiurewicz(k, p, seeds):
    def f(c):
        zs, dzs = orbit(c, k + p)
        return zs[k + p] - zs[k], dzs[k + p] - dzs[k]

    c = newton(seeds, f)
    zs, _ = orbit(c, k + p)
    ok = np.isfinite(c) & (np.abs(zs[k + p] - zs[k]) < 1e-12)
    ok &= np.abs(zs[k - 1 + p] - zs[k - 1]) > 1e-8  # preperiod is exactly k
    for q in range(1, p):
        ok &= np.abs(zs[k + q] - zs[k]) > 1e-8  # period is exactly p
    for j in range(1, k + p + 1):
        ok &= np.abs(zs[j]) > 1e-8  # 0 is not periodic: not a minibrot nucleus
    return c[ok]


def minibrot_size(c, p):
    """Size estimate of the minibrot with nucleus c and period p, relative to the whole set."""
    z, l, b = 0j, 1 + 0j, 1 + 0j
    for _ in range(1, p):
        z = z * z + c
        l = 2 * z * l
        b = b + 1 / l
    return abs(1 / (b * l * l))


def iteration_limit(zoom):
    """Same as render::iteration_limit."""
    return int(255 + 250 * math.log10(max(zoom, 1)))


def frame(center, zoom, w=160, h=90):
    """Pixel coordinates of a low-res, 16:9 version of the frame main.rs renders."""
    xspan, yspan = 1.3 / zoom, 1.3 * h / w / zoom
    xs = center.real - xspan + np.arange(w) * 2 * xspan / w
    ys = center.imag + yspan - np.arange(h) * 2 * yspan / h
    return xs[None, :] + 1j * ys[:, None]


def escape_counts(center, zoom):
    """Escape counts with render.rs's iteration limit; -1 = in the set."""
    c = frame(center, zoom)
    z = np.zeros_like(c)
    n = np.full(c.shape, -1)
    live = np.ones(c.shape, bool)
    for i in range(iteration_limit(zoom)):
        z[live] = z[live] ** 2 + c[live]
        esc = live & (z.real ** 2 + z.imag ** 2 > 4)
        n[esc] = i
        live &= ~esc
        if not live.any():
            break
    return n


def affordable(n, zoom):
    return np.where(n < 0, iteration_limit(zoom), n).mean() <= MAX_ITERATIONS_PER_PIXEL


def frames_minibrot(c, zoom):
    """The minibrot sits in the middle of the final frame without filling it."""
    n = escape_counts(c, zoom)
    inside = (n < 0).mean()
    return n[n.shape[0] // 2, n.shape[1] // 2] < 0 and 0.01 < inside < 0.3 and affordable(n, zoom)


def busy(c, zoom):
    """Most of the frame lies within two pixels of the set, by distance estimate."""
    cs = frame(c, zoom)
    pixel = abs(cs[0, 1] - cs[0, 0])
    z, dz = np.zeros_like(cs), np.zeros_like(cs)
    distance = np.zeros(cs.shape)
    live = np.ones(cs.shape, bool)
    for _ in range(iteration_limit(zoom)):
        dz[live] = 2 * z[live] * dz[live] + 1
        z[live] = z[live] ** 2 + cs[live]
        esc = live & (np.abs(z) > 1e3)
        r = np.abs(z[esc])
        distance[esc] = r * np.log(r) / np.abs(dz[esc])
        live &= ~esc
        if not live.any():
            break
    return (distance < 2 * pixel).mean() > 0.6


def far_from(c, chosen):
    """Distance to the nearest chosen target, counting mirror images as the same place."""
    return min([abs(c - t) for t, *_ in chosen] + [abs(c - t.conjugate()) for t, *_ in chosen] + [9.0])


def pick_spread(candidates, count, chosen, accept):
    """Greedily takes the candidate farthest from everything chosen so far, if accept() passes."""
    picked = []
    candidates = list(candidates)
    while candidates and len(picked) < count:
        best = max(candidates, key=lambda t: far_from(t[0], chosen + picked))
        candidates.remove(best)
        if far_from(best[0], chosen + picked) > 1e-6 and accept(best):
            picked.append(best)
    return picked


def main():
    seeds = boundary_seeds(400_000)

    nuclei = []
    for p in range(8, 160):
        for c in find_nuclei(p, rng.choice(seeds, 300, replace=False)):
            zoom = 0.5 / minibrot_size(c, p)
            if 1e3 <= zoom <= MAX_ZOOM and p * ITERATIONS_PER_PERIOD <= iteration_limit(zoom):
                nuclei.append((c, "minibrot", p, zoom))

    chosen = []
    for decade in range(3, math.ceil(math.log10(MAX_ZOOM))):
        bucket = [t for t in nuclei if decade <= math.log10(t[3]) < decade + 1]
        picked = pick_spread(bucket, MINIBROTS_PER_DECADE, chosen, lambda t: frames_minibrot(t[0], t[3]))
        print(f"1e{decade}: {len(bucket)} candidates, {len(picked)} picked")
        chosen += picked

    spirals = []
    for k in range(2, 30, 2):
        for p in range(1, 7):
            for c in find_misiurewicz(k, p, rng.choice(seeds, 60, replace=False)):
                spirals.append((c, f"Misiurewicz M{k},{p}", p, MAX_ZOOM))
    picked = pick_spread(
        spirals, MISIUREWICZ, chosen, lambda t: busy(t[0], 1e6) and affordable(escape_counts(t[0], t[3]), t[3])
    )
    print(f"Misiurewicz: {len(spirals)} candidates, {len(picked)} picked")
    chosen += picked

    chosen.sort(key=lambda t: t[3])
    lines = [
        "// Generated by tools/gen_targets.py; regenerate rather than edit by hand.",
        "",
        "/// A point on the boundary of the Mandelbrot set to zoom into, and the zoom level",
        "/// past which there is nothing new to see (or f64 runs out of precision).",
        "pub struct Target {",
        "    pub re: f64,",
        "    pub im: f64,",
        "    pub max_zoom: f64,",
        "}",
        "",
        "pub const TARGETS: &[Target] = &[",
    ]
    for c, kind, p, zoom in chosen:
        label = f"{kind}, period {p}" if kind == "minibrot" else kind
        lines.append(f"    // {label}")
        lines.append(f"    Target {{ re: {float(c.real)!r}, im: {float(c.imag)!r}, max_zoom: {zoom:.3e} }},")
    lines.append("];")
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {len(chosen)} targets to {OUT}")


if __name__ == "__main__":
    main()
