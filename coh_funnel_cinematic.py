#!/usr/bin/env python3

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.collections import LineCollection
from matplotlib.patches import Circle, Polygon
from pathlib import Path
from collections import Counter, defaultdict

np.random.seed(44)

# ================= CONFIG =================
N = 520
T = 540

x = np.linspace(-1.0, 1.0, T)
gate_x = 0.05
gate_idx = np.argmin(np.abs(x - gate_x))

OUT_PATH = Path("coh_funnel_final.png")

# ================= COLORS =================
BG = "#05080d"
GOLD = "#d9a233"
CYAN = "#8ed8ff"
WHITE = "#eaf2ff"
RED = "#ff4e3f"
PURPLE = "#9467bd"
BROWN = "#8c564b"
BLACK = "#000000"
MUTED = "#6b7280"

GROUP_COLORS = {
    "EXPLORE": "#17becf",
    "EXPLOIT": "#2ca02c",
    "BRIDGE": "#1f77b4",
    "PERTURB": "#ff7f0e",
    "ADVERSARY": "#d62728",
    "REPAIR": "#9467bd",
}

GROUPS = list(GROUP_COLORS.keys())

OUTCOME_COLORS = {
    "ACCEPT": GOLD,
    "REJECT_MARGIN": "#ff7f0e",
    "REJECT_RATE": RED,
    "REJECT_ENVELOPE": PURPLE,
    "REJECT_TENSION": BROWN,
    "SNAP_FAILURE": BLACK,
    "MALFORMED": MUTED,
}

# ================= CORE =================

def sigmoid(z):
    return 1 / (1 + np.exp(-z))

def envelope(y):
    return np.sum(np.abs(np.diff(y))) * 0.3 + np.max(np.abs(y))*0.6

def tension(y):
    return np.mean(np.abs(np.diff(y))) + 0.4*np.mean(np.abs(np.diff(y,2)))

def snap(y):
    local = y[max(0, gate_idx-6):gate_idx+1]
    if len(local) < 4:
        return 0
    return max(
        np.max(np.abs(np.gradient(local))) / 3.2,
        np.max(np.abs(np.diff(local,2))) / 8.5
    )

# ================= TRAJECTORY =================

def make_traj(group):
    amp = {
        "EXPLORE": 0.22,
        "EXPLOIT": 0.08,
        "BRIDGE": 0.16,
        "PERTURB": 0.30,
        "ADVERSARY": 0.42,
        "REPAIR": 0.06,
    }[group]

    y0 = np.random.uniform(-0.7, 0.7)
    f1 = np.random.uniform(2, 8)
    f2 = np.random.uniform(8, 20)

    y = y0 + amp*np.sin(f1*(x+1)) + 0.4*amp*np.sin(f2*(x+1))
    y += 0.15*np.cumsum(np.random.randn(T))*0.02

    if group == "REPAIR":
        y *= (1 - 0.3*sigmoid(4*(x+0.2)))

    return y

# ================= BRAID =================

def braid(curves):
    curves = curves.copy()
    for j in range(1, T):
        prev = curves[:, j-1]
        for i in range(len(curves)):
            lo = max(0, i-3)
            hi = min(len(curves), i+4)
            curves[i, j] += 0.02*(np.mean(prev[lo:hi]) - prev[i])
    return curves

# ================= RV =================

def rv(y):
    e = envelope(y[:gate_idx+1])
    t = tension(y[:gate_idx+1])
    s = snap(y)
    r = abs(np.gradient(y)[gate_idx])

    if s > 1: return "SNAP_FAILURE"
    if t > 0.22: return "REJECT_TENSION"
    if r > 2.45: return "REJECT_RATE"
    if e > 1.85: return "REJECT_ENVELOPE"

    return "ACCEPT" if np.random.rand() < 0.4 else "REJECT_MARGIN"

# ================= POST =================

def post(y, ok, lane_id=0):
    g = sigmoid(20*(x-gate_x))

    if ok:
        lanes = [-1,0,1]
        lane = lanes[lane_id % 3]

        base = lane*0.085
        braid = base + 0.05*np.sin(18*(x-gate_x) + lane)

        return (1-g)*y + g*braid

    y2 = y.copy()
    decay = np.exp(-(x-gate_x)*10)
    y2[x>gate_x] *= decay[x>gate_x]
    y2[x>gate_x+0.2] = np.nan
    return y2

# ================= SIM =================

curves = []
groups = []

for i in range(N):
    g = GROUPS[i % len(GROUPS)]
    groups.append(g)
    curves.append(make_traj(g))

curves = np.array(curves)
curves = braid(curves)

results = []
final = []
colors = []

lane = 0

for y, g in zip(curves, groups):
    outcome = rv(y)

    if outcome == "ACCEPT":
        fy = post(y, True, lane)
        lane += 1
        c = GOLD if lane % 2 else CYAN
    else:
        fy = post(y, False)
        c = OUTCOME_COLORS[outcome]

    final.append(fy)
    results.append(outcome)
    colors.append(c)

counts = Counter(results)

# ================= PLOT =================

fig, ax = plt.subplots(figsize=(18,9), dpi=180)
fig.patch.set_facecolor(BG)
ax.set_facecolor(BG)

for y, g, o, c in zip(final, groups, results, colors):

    valid = np.isfinite(y)
    xv = x[valid]
    yv = y[valid]

    if len(xv) < 2:
        continue

    pre = xv <= gate_x
    postm = xv > gate_x

    # pre = group color
    if np.sum(pre) > 2:
        pts = np.array([xv[pre], yv[pre]]).T.reshape(-1,1,2)
        segs = np.concatenate([pts[:-1], pts[1:]], axis=1)
        ax.add_collection(LineCollection(segs, colors=GROUP_COLORS[g], linewidths=0.5, alpha=0.3))

    # post
    if o == "ACCEPT":
        ax.plot(xv[postm], yv[postm], color=c, lw=2, alpha=0.8)
    else:
        gy = y[gate_idx]
        sign = np.sign(gy) if abs(gy)>1e-6 else 1
        ax.plot([gate_x, gate_x+0.1], [gy, gy+0.2*sign], color=c, ls="--", lw=1)

# gate
theta = np.linspace(0,2*np.pi,400)
ax.plot(gate_x + 0.03*np.cos(theta), 0.6*np.sin(theta), color=GOLD, lw=2)

# text
ax.text(0,0.85,"APE → PHASELOOM → COH VERIFIER", color=WHITE, ha="center", fontsize=16)
ax.text(0,-0.95,"THE GEOMETRY CHANGES. THE LAW DOESN'T.", color=GOLD, ha="center", fontsize=14)

# limits
ax.set_xlim(-1.05,1.05)
ax.set_ylim(-1.0,0.9)
ax.axis("off")

plt.tight_layout()
plt.savefig(OUT_PATH, facecolor=BG)
# plt.show() # Commented out for headless execution

print("Outcome counts:", counts)
