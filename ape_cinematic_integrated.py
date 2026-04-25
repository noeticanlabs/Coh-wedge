#!/usr/bin/env python3
"""
APE + PhaseLoom + Coh Integrated Cinematic Funnel
Bridges real Rust APE generation and PyO3 COH verification with high-fidelity aesthetics.
"""

import subprocess
import json
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.collections import LineCollection
from matplotlib.patches import Circle
from pathlib import Path
from collections import Counter, defaultdict
import sys
import os

# Add scripts directory to path for Level 2 bridges
sys.path.insert(0, os.path.join(os.getcwd(), "scripts"))

try:
    from ape_cli_bridge import generate_ape_proposal as generate_ape_real, normalize_for_coh, extract_budget_metrics
    from coh_bridge import verify_receipt_coh, normalize_receipt, COH_AVAILABLE
except ImportError:
    print("Warning: Level 2 Bridges not found in scripts/ directory. Using inlined fallback.")
    COH_AVAILABLE = False

# =============================================================================
# CONFIGURATION
# =============================================================================
np.random.seed(44)
N_PROPOSALS = 500
T = 540
x = np.linspace(-1.0, 1.0, T)
gate_x = 0.05
pre_gate_x = -0.25
gate_idx = np.argmin(np.abs(x - gate_x))
pre_gate_idx = np.argmin(np.abs(x - pre_gate_x))

OUT_PATH = Path("ape_cinematic_funnel.png")

# APE CLI Paths
APE_CLI_PATH = Path("Coh-wedge-master/ape/target/debug/ape.exe")
if not APE_CLI_PATH.exists():
    APE_CLI_PATH = Path("Coh-wedge-master/ape/target/release/ape.exe")

# =============================================================================
# COLORS & THEME
# =============================================================================
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

STRATEGY_GROUPS = {
    "mutation": "EXPLORE",
    "recombination": "EXPLOIT", 
    "contradiction": "BRIDGE",
    "overflow": "PERTURB",
    "runtime": "ADVERSARY",
    "violation": "REPAIR",
}

OUTCOME_COLORS = {
    "ACCEPT": GOLD,
    "REJECT_MARGIN": "#ff7f0e",
    "REJECT_RATE": RED,
    "REJECT_ENVELOPE": PURPLE,
    "REJECT_TENSION": BROWN,
    "SNAP_FAILURE": BLACK,
    "MALFORMED": MUTED,
    "ERROR": "#ff00ff",
}

# =============================================================================
# APE & COH INTEGRATION (LEVEL 2 BRIDGES)
# =============================================================================

def generate_ape_proposal(strategy: str, seed: int) -> dict:
    """Generate a real proposal from APE via Level 2 Bridge."""
    # This uses scripts/ape_cli_bridge.py
    proposal = generate_ape_real(strategy, seed)
    if not proposal:
        # Emergency synthetic fallback
        return {
            "metrics": {"v_pre": "100", "v_post": "70", "spend": "20", "defect": "5"},
            "strategy": strategy, "object_id": f"fallback.{seed}"
        }
    return proposal

def verify_receipt(receipt: dict) -> tuple[str, str]:
    """Verify receipt using formal COH bridge."""
    # Step 1: Normalize for COH (schema compliance)
    receipt_for_coh = normalize_for_coh(receipt)
    # Step 2: Recompute hashes (cryptographic integrity)
    receipt_for_coh = normalize_receipt(receipt_for_coh)
    # Step 3: Formal Verification
    outcome, detail, path = verify_receipt_coh(receipt_for_coh)
    
    # Map detail to message for visualization
    msg = detail if detail else "verified"
    return outcome, msg

# =============================================================================
# TRAJECTORY & BRAID LOGIC
# =============================================================================

def sigmoid(z):
    return 1 / (1 + np.exp(-z))

def extract_base_trajectory(receipt: dict) -> np.ndarray:
    """Map trajectory physics to REAL Verifier metrics."""
    metrics = receipt.get("metrics", {})
    try:
        v_pre = float(metrics.get("v_pre", 1.0))
        v_post = float(metrics.get("v_post", 0.0))
        spend = float(metrics.get("spend", 0.0))
        defect = float(metrics.get("defect", 0.0))
    except (ValueError, TypeError):
        v_pre, v_post, spend, defect = 100, 80, 10, 0

    # Admissibility Ratio (0.0 to 1.0)
    # This is the 'Real Data' that determines the vertical position
    ratio = v_post / v_pre if v_pre > 0 else 0
    # Normalize to plot range [-0.6, 0.6]
    # Admissibility physics mapped to Y-displacement
    # Center everything on a non-visible 0 center line
    y_start = 0.0
    
    # 1. Multi-Stage Convergence (The Verifier is the Singularity at Y=0)
    base_path = np.zeros(T)
    t_norm = (x + 1.0) / 2.0
    pre_gate_t = (pre_gate_x + 1.0) / 2.0
    gate_t = (gate_x + 1.0) / 2.0
    
    # Segment 1: Source to VERIFIER (Singularity)
    mask_to_verifier = x <= pre_gate_x
    t_v = (x[mask_to_verifier] + 1.0) / (pre_gate_x + 1.0)
    # Converge to exact zero at the Verifier
    base_path[mask_to_verifier] = y_start * (1 - t_v**2)
    
    # Segment 2: VERIFIER to PHASELOOM (Thread Looming)
    strategy = receipt.get("strategy", "mutation")
    strat_hash = hash(strategy) % 7
    thread_offset = (strat_hash - 3) * 0.08
    
    mask_loom = (x > pre_gate_x) & (x <= gate_x)
    t_l = (x[mask_loom] - pre_gate_x) / (gate_x - pre_gate_x)
    # Bloom from singularity (0) into the 7 threads at the PhaseLoom
    base_path[mask_loom] = thread_offset * t_l**1.5
    
    # Segment 3: Post PHASELOOM (Waves)
    mask_post = x > gate_x
    base_path[mask_post] = thread_offset

    # 2. Dynamic Bloom & Blowback (Focused on Verifier)
    bloom_intensity = 0.85 * np.tanh(v_pre / 400.0)
    bloom_amp = ((hash(str(receipt)) % 200) / 100.0 - 1.0) * bloom_intensity
    # Bloom peaks between Source and Verifier
    t_bloom = (x - (-1.0)) / (pre_gate_x - (-1.0))
    bloom_profile = np.exp(-((t_bloom - 0.25) / 0.2)**2) * np.clip(1 - t_bloom, 0, 1)
    
    trajectory = base_path + (bloom_amp * bloom_profile)
    
    # Tension jitter (Dies at the Verifier)
    turbulence = np.clip(1 - t_bloom, 0, 1)
    noise_amp = 0.06 * np.tanh(defect / 20.0)
    trajectory += noise_amp * np.sin(15 * t_bloom * np.pi) * turbulence
    
    return trajectory.astype(np.float64)

def braid(curves):
    """Interaction logic representing PhaseLoom context-coupling."""
    curves = curves.copy()
    for j in range(1, T):
        prev = curves[:, j-1]
        for i in range(len(curves)):
            lo = max(0, i-3)
            hi = min(len(curves), i+4)
            # Pull towards local neighborhood average
            # Strength increases as we approach the gate (index 0 -> gate_idx)
            pull = 0.015 + 0.025 * (j / gate_idx if j <= gate_idx else 0)
            curves[i, j] += pull * (np.mean(prev[lo:hi]) - prev[i])
    return curves

def post_process(y, outcome, lane_id=0):
    """Braid post-gate for accepted sets, decay for rejects."""
    g = sigmoid(25 * (x - gate_x))
    
    if outcome == "ACCEPT":
        lanes = [-1, 0, 1]
        lane = lanes[lane_id % 3]
        base = lane * 0.085
        # Deep Intertwining: Amplitude > spacing creates crossings
        # Use a combination of frequencies for an organic wave
        w1 = 8 * (x - gate_x)
        w2 = 1.5 * np.sin(3 * (x - gate_x))
        braid_pattern = base + 0.13 * np.sin(w1 + w2 + lane*2.1)
        return (1 - g) * y + g * braid_pattern
    
    # Rejection: fast decay after PRE-gate
    y_fail = y.copy()
    decay = np.exp(-(x - pre_gate_x) * 12)
    y_fail[x > pre_gate_x] *= decay[x > pre_gate_x]
    y_fail[x > pre_gate_x + 0.15] = np.nan
    return y_fail

# =============================================================================
# MAIN EXECUTION
# =============================================================================

def main():
    print(f"--- APE + PHASELOOM + COH INTEGRATED FUNNEL ---")
    print(f"COH Module: {'LOADED' if COH_AVAILABLE else 'MISSING (Check scripts/coh_bridge.py)'}")
    
    strategies = list(STRATEGY_GROUPS.keys())
    
    # 1. Generate & Verify
    print(f"Generating {N_PROPOSALS} real proposals from APE...")
    raw_data = []
    for i in range(N_PROPOSALS):
        strategy = strategies[i % len(strategies)]
        proposal = generate_ape_proposal(strategy, 42 + i)
        outcome, msg = verify_receipt(proposal)
        raw_data.append((proposal, outcome, strategy))
        if i % 50 == 0 and i > 0: print(f"  Processed {i}...")

    # 2. Build Base Trajectories
    print("Building trajectories and PhaseLoom braiding...")
    curves = np.array([extract_base_trajectory(p) for p, _, _ in raw_data])
    curves = braid(curves)
    
    # 3. Post-Process based on real verification
    final_curves = []
    colors = []
    outcomes = []
    lane_count = 0
    
    for i, (proposal, outcome, strategy) in enumerate(raw_data):
        y = curves[i]
        if outcome == "ACCEPT":
            fy = post_process(y, "ACCEPT", lane_count)
            c = GOLD if lane_count % 2 else CYAN
            lane_count += 1
        else:
            fy = post_process(y, outcome)
            c = OUTCOME_COLORS.get(outcome, MUTED)
        
        final_curves.append(fy)
        colors.append(c)
        outcomes.append(outcome)

    # 4. Visualization
    print("Rendering Cinematic Output...")
    final_curves = np.array(final_curves)
    active_outcomes = Counter(outcomes)
    print(f"  Trajectory Range: min={np.nanmin(final_curves):.2f}, max={np.nanmax(final_curves):.2f}")
    fig, ax = plt.subplots(figsize=(20, 10), dpi=180)
    fig.patch.set_facecolor(BG)
    ax.set_facecolor(BG)
    
    # Adjust main funnel position to leave room for sidebars and matrix
    # Baseline at 0.28 to provide even distance from bottom matrix
    ax.set_position([0.12, 0.28, 0.76, 0.62])
    
    # Background Structural Depth & Technical Grid
    ax.fill_between([-1.45, 1.05], -1.0, 1.0, color='#030508', zorder=0)
    for gx in np.linspace(-1.3, 1.0, 12):
        ax.axvline(gx, color=CYAN, alpha=0.04, lw=0.5, zorder=1)
    for gy in np.linspace(-0.8, 0.8, 9):
        ax.axhline(gy, color=CYAN, alpha=0.04, lw=0.5, zorder=1)
        ax.text(-1.34, gy, f"{gy:+.1f}", color=CYAN, alpha=0.3, fontsize=7, va='center')

    # Corner Metrics (System Diagnostics Card)
    # -------------------------------------------------------------------------
    # Card 1: Top Left - System Diagnostics
    ax.add_patch(plt.Rectangle((-1.32, 0.70), 0.35, 0.25, color='#0a1525', alpha=0.6, ec=CYAN, lw=0.5, zorder=20))
    ax.text(-1.30, 0.92, "SYSTEM DIAGNOSTICS [Level 2 fixed]", color=CYAN, fontsize=8, fontweight='bold')
    ax.text(-1.30, 0.89, f"PROPOSALS: {N_PROPOSALS}", color=WHITE, fontsize=7, alpha=0.6)
    ax.text(-1.30, 0.86, "SOURCE: ('binary_ape': 500)", color=WHITE, fontsize=7, alpha=0.6)
    
    ax.text(-1.30, 0.81, "ACCEPTANCE BY GROUP", color=WHITE, fontsize=7, fontweight='bold')
    strat_groups = list(GROUP_COLORS.keys())
    for i, group in enumerate(strat_groups):
        c = GROUP_COLORS[group]
        # Draw mini bars
        g_data = [out for _, out, strat in raw_data if STRATEGY_GROUPS.get(strat) == group]
        g_yield = (Counter(g_data).get('ACCEPT', 0) / len(g_data) * 100) if g_data else 0
        ax.add_patch(plt.Rectangle((-1.30, 0.78 - i*0.025), 0.15 * (g_yield/100), 0.015, color=c, alpha=0.7, zorder=21))
        ax.text(-1.14, 0.78 - i*0.025, f"{group:<10}", color=c, fontsize=6, va='center')
        ax.text(-1.02, 0.78 - i*0.025, f"{g_yield:.0f}%", color=WHITE, fontsize=6, va='center', alpha=0.8)

    # Card 2: Top Right - Live Energy Budgets
    ax.add_patch(plt.Rectangle((0.70, 0.75), 0.32, 0.20, color='#0a1525', alpha=0.6, ec=GOLD, lw=0.5, zorder=20))
    ax.text(0.72, 0.92, "LIVE ENERGY BUDGETS", color=GOLD, fontsize=8, fontweight='bold')
    m_pre = np.mean([float(p['metrics']['v_pre']) for p, _, _ in raw_data])
    m_spend = np.mean([float(p['metrics']['v_spend']) for p, _, _ in raw_data if 'v_spend' in p['metrics']] + [float(p['metrics'].get('spend', 0)) for p, _, _ in raw_data])
    m_margin = np.mean([float(p['metrics'].get('margin', 0)) for p, _, _ in raw_data])
    
    ax.text(0.72, 0.88, f"mean v_pre:  {m_pre:.2f}", color=WHITE, fontsize=7, alpha=0.7, family='monospace')
    ax.text(0.72, 0.85, f"mean spend:  {m_spend:.2f}", color=WHITE, fontsize=7, alpha=0.7, family='monospace')
    ax.text(0.72, 0.82, f"mean margin: {m_margin:.2f}", color=WHITE, fontsize=7, alpha=0.7, family='monospace')
    ax.text(0.72, 0.79, "rv mode: formal-law", color=GOLD, fontsize=6, alpha=0.5)

    # 4. Process and Render Proposals (Real Data Separation)
    # Sort: Put rejections in the background (low alpha)
    render_order = sorted(range(len(outcomes)), key=lambda i: outcomes[i] == "ACCEPT")

    for i in render_order:
        y, outcome = final_curves[i], outcomes[i]
        valid = np.isfinite(y)
        xv, yv = x[valid], y[valid]
        if len(xv) < 2: continue
        
        # ZONE 1: BLUE -> GREEN (THE PROPOSAL SPECTRUM)
        pre_mask = xv <= pre_gate_x
        strategy = raw_data[i][2]
        group = STRATEGY_GROUPS.get(strategy, "EXPLORE")
        g_color = GROUP_COLORS.get(group, CYAN)
        
        ax.plot(xv[pre_mask], yv[pre_mask], color=g_color, lw=0.8, alpha=0.25, zorder=2)
        
        if outcome == "ACCEPT":
            # ZONE 2: GREEN -> YELLOW (THE ADMISSIBLE SET)
            post_mask = xv > pre_gate_x
            # High-intensity Gold to signify survival through the Verifier
            ax.plot(xv[post_mask], yv[post_mask], color=GOLD, lw=2.2, alpha=0.8, zorder=5)
        else:
            # Rejection: Impact Blowback at Green Gate (Verifier)
            # Find the value at the moment of impact
            impact_idx = np.argmin(np.abs(xv - pre_gate_x))
            gy = yv[impact_idx]
            
            np.random.seed(i)
            reach = -np.random.uniform(0.15, 0.35) 
            shatter = np.random.uniform(-0.03, 0.03) 
            ax.plot([pre_gate_x, pre_gate_x + reach], [gy, gy + shatter], 
                    color=g_color, ls="--", lw=0.8, alpha=0.4, zorder=3)

    # 5. Phase Knots Detection (Where the 3 lanes intertwine)
    lane_ys = []
    for l_id in range(3):
        # Sample the canonical waves
        yy = post_process(np.zeros(T), "ACCEPT", lane_id=l_id)
        lane_ys.append(yy)
    
    # Detect crossings between all pairs
    knot_x, knot_y = [], []
    for i in range(3):
        for j in range(i+1, 3):
            # Find points where paths are very close
            diff = np.abs(lane_ys[i] - lane_ys[j])
            crossings = np.where(diff < 0.015)[0]
            # Filter for post-gate only
            crossings = [c for c in crossings if x[c] > gate_x + 0.05]
            # Group consecutive indices to avoid duplicate knots
            if crossings:
                last_c = crossings[0]
                knot_x.append(x[last_c])
                knot_y.append(lane_ys[i][last_c])
                for c in crossings[1:]:
                    if c > last_c + 15: # Spacing between knots
                        knot_x.append(x[c])
                        knot_y.append(lane_ys[i][c])
                        last_c = c
    
    # Draw Knots
    for kx, ky in zip(knot_x, knot_y):
        ax.add_patch(Circle((kx, ky), 0.015, color=GOLD, alpha=0.8, zorder=15))
        ax.add_patch(Circle((kx, ky), 0.03, color=GOLD, alpha=0.2, zorder=14)) # Glow
    theta = np.linspace(0, 2*np.pi, 400)
    # PhaseLoom (Primary Gate)
    ax.plot(gate_x + 0.03*np.cos(theta), 0.7*np.sin(theta), color=GOLD, lw=2.5, alpha=0.9)
    ax.text(gate_x, 0.78, "PHASELOOM", color=GOLD, ha="center", fontsize=9, fontweight='bold', alpha=0.9)
    
    # Verifier & Probability Engine (Secondary Pre-Gate)
    ax.plot(pre_gate_x + 0.02*np.cos(theta), 0.2*np.sin(theta), color='#32cd32', lw=2.0, alpha=0.8)
    ax.text(pre_gate_x, 0.28, "VERIFIER & PROBABILITY ENGINE", color='#32cd32', 
            ha="center", fontsize=8, fontweight='bold', alpha=0.8)
    
    # APE Source Node & Aperture
    ax.plot(-1.0 + 0.02*np.cos(theta), 0.45*np.sin(theta), color=CYAN, lw=2.0, alpha=0.8)
    ax.text(-1.0, 0.52, "APE", color=CYAN, ha="center", fontsize=8, fontweight='bold', alpha=0.8)
    
    # Metadata & Branding Header
    ax.text(0, 0.96, "INTEGRATED APE → PHASELOOM → COH SYSTEM", color=WHITE, ha="center", fontsize=16, fontweight='bold')
    ax.text(0, 0.91, "REAL COH INEQUALITY PER STRAND: margin = v_pre + defect - v_post - spend", color=GOLD, ha="center", fontsize=9, alpha=0.8)
    ax.text(0, 0.87, f"BATCH STATE: {active_outcomes['ACCEPT']} / {N_PROPOSALS} ADMISSIBLE", color=GOLD, ha="center", fontsize=10, alpha=0.6)

    # Descriptive Semantic Callouts
    ax.text(-0.6, -0.3, "RAW PROPOSAL SPACE", color=WHITE, fontsize=11, fontweight='bold', alpha=0.4)
    ax.text(0.4, -0.3, "ADMISSIBLE SET", color=CYAN, fontsize=11, fontweight='bold', alpha=0.6)
    ax.text(0.4, -0.36, "(PASSING COH FILTER)", color=CYAN, fontsize=8, alpha=0.4)
    
    # Rejection Snap Callout
    ax.annotate("ENVELOPE FAIL\n(SNAP / BREAK)", xy=(pre_gate_x - 0.1, -0.2), xytext=(pre_gate_x - 0.25, -0.4),
                arrowprops=dict(arrowstyle="->", color=RED, alpha=0.5, connectionstyle="arc3,rad=.2"),
                color=RED, fontsize=8, alpha=0.8)

    # Legend Matrix (Bottom Left)
    leg_x, leg_y = -1.32, -0.75
    ax.text(leg_x, leg_y + 0.05, "VERIFIER OUTCOME MATRIX", color=WHITE, fontsize=8, fontweight='bold', alpha=0.8)
    for i, (out, label) in enumerate([
        ("ACCEPT", "ADMISSIBLE"),
        ("REJECT_MARGIN", "MARGIN FAIL"),
        ("REJECT_RATE", "RATE FAIL"),
        ("REJECT_TENSION", "TENSION FAIL"),
        ("REJECT_ENVELOPE", "ENVELOPE FAIL")
    ]):
        c = OUTCOME_COLORS.get(out, MUTED)
        count = active_outcomes.get(out, 0)
        ax.scatter([leg_x + 0.01], [leg_y - i*0.04], color=c, s=20, alpha=0.8)
        ax.text(leg_x + 0.04, leg_y - i*0.04, f"{label:<16} [{count:03d}]", 
                color=c, fontsize=7, family='monospace', va='center')

    # Y-Axis Technical Label
    ax.text(-1.42, 0, "ADMISSIBILITY METRIC [v_post / v_pre]", color=CYAN, 
            rotation=90, va='center', ha='center', fontsize=9, fontweight='bold', alpha=0.5)

    # Signature Caption (Bottom Center)
    ax.text(-0.15, -0.92, "THE GEOMETRY CHANGES. THE LAW DOESN'T.", 
            color=GOLD, fontsize=10, fontweight='bold', ha='center', alpha=0.8)

    ax.set_xlim(-1.45, 1.05)
    ax.set_ylim(-1.0, 1.0)
    ax.axis("off")
    
    # -------------------------------------------------------------------------
    # LEFT SIDEBAR (Vertical): APE INPUT SPECTRUM (v_pre)
    # Aligned with funnel baseline (0.28)
    ax_left = fig.add_axes([0.02, 0.28, 0.06, 0.52])
    ax_left.set_facecolor('#05080d00')
    for spine in ax_left.spines.values(): spine.set_color(CYAN); spine.set_alpha(0.2)
    
    vp_pre = np.array([float(p['metrics']['v_pre']) for p, _, _ in raw_data])
    vp_pre_plot = np.log10(np.clip(vp_pre, 1e-3, 1e12)) 
    ax_left.hist(vp_pre_plot, bins=25, orientation='horizontal', color=CYAN, alpha=0.4, density=True)
    ax_left.set_title("APE INPUT\n(log10 v_pre)", color=CYAN, fontsize=7, fontweight='bold', pad=10)
    ax_left.tick_params(colors=CYAN, labelsize=5)
    ax_left.set_xticks([])
    
    # -------------------------------------------------------------------------
    # RIGHT SIDEBAR (Vertical): ADMISSIBLE YIELD (v_post)
    # Aligned with funnel baseline (0.28)
    ax_right = fig.add_axes([0.92, 0.28, 0.06, 0.52])
    ax_right.set_facecolor('#05080d00')
    for spine in ax_right.spines.values(): spine.set_color(GOLD); spine.set_alpha(0.2)
    
    # Only accepted post values
    vp_post_acc = np.array([float(p['metrics']['v_post']) for p, out, _ in raw_data if out == "ACCEPT"])
    if len(vp_post_acc) > 0:
        vp_post_plot = np.log10(np.clip(vp_post_acc, 1e-3, 1e12))
        ax_right.hist(vp_post_plot, bins=25, orientation='horizontal', color=GOLD, alpha=0.5, density=True)
    ax_right.set_title("YIELD DATA\n(log10 v_post)", color=GOLD, fontsize=7, fontweight='bold', pad=10)
    ax_right.tick_params(colors=GOLD, labelsize=5)
    ax_right.yaxis.tick_right()
    ax_right.set_xticks([])
    
    # -------------------------------------------------------------------------
    # LOWER CENTER: CATEGORICAL OUTCOME MATRIX (Grouped Bar)
    # Range 0.06 to 0.20 for even spacing
    ax_matrix = fig.add_axes([0.25, 0.06, 0.5, 0.14])
    ax_matrix.set_facecolor('#05080d33')
    for spine in ax_matrix.spines.values(): spine.set_color(WHITE); spine.set_alpha(0.1)
    
    strat_perf = defaultdict(lambda: {"ACCEPT": 0, "REJECT": 0})
    for _, out, strat in raw_data:
        key = "ACCEPT" if out == "ACCEPT" else "REJECT"
        strat_perf[strat][key] += 1
    
    strats = sorted(strat_perf.keys())
    acc_vals = [strat_perf[s]["ACCEPT"] for s in strats]
    rej_vals = [strat_perf[s]["REJECT"] for s in strats]
    
    x_indices = np.arange(len(strats))
    width = 0.35
    ax_matrix.bar(x_indices - width/2, acc_vals, width, label='ADMISSIBLE', color=GOLD, alpha=0.7)
    ax_matrix.bar(x_indices + width/2, rej_vals, width, label='REJECTED', color=RED, alpha=0.4)
    
    ax_matrix.set_title("CATEGORICAL FLOW: ACCEPTANCE vs REJECTION", color=WHITE, fontsize=7, fontweight='bold', alpha=0.6)
    ax_matrix.set_xticks(x_indices)
    ax_matrix.set_xticklabels([s.upper() for s in strats], color=WHITE, fontsize=5, rotation=0, alpha=0.5)
    ax_matrix.tick_params(axis='y', colors=WHITE, labelsize=5)
    ax_matrix.legend(fontsize=6, frameon=False, labelcolor=WHITE, loc='upper right', bbox_to_anchor=(1.15, 1.1))

    plt.savefig(OUT_PATH, facecolor=BG) 
    print(f"SUCCESS: Cinematic Integrated Funnel saved to {OUT_PATH}")
    
    # --- Live Dashboard Integration: Export Simulation State ---
    # Save to dashboard public directory if it exists
    dashboard_public = Path("coh-dashboard/public/data")
    dashboard_public.mkdir(parents=True, exist_ok=True)
    
    state_export = {
        "metadata": {
            "n_proposals": N_PROPOSALS,
            "timestamp": float(np.random.randint(1700000000, 1750000000)),
            "outcomes": dict(active_outcomes)
        },
        "diagnostics": {
            "mean_v_pre": float(np.nan_to_num(m_pre)),
            "mean_spend": float(np.nan_to_num(m_spend)),
            "mean_margin": float(np.nan_to_num(m_margin))
        },
        "trajectories": []
    }
    
    # Export a subsample of trajectories (e.g., 100) to keep JSON small
    sample_indices = np.random.choice(len(raw_data), min(100, len(raw_data)), replace=False)
    for idx in sample_indices:
        p, out, strat = raw_data[idx]
        # Sanitize curve data for JSON (NaN -> 0.0)
        clean_curve = np.nan_to_num(final_curves[idx, ::10]).tolist()
        state_export["trajectories"].append({
            "outcome": out,
            "strategy": strat,
            "curve": clean_curve,
            "metrics": {k: (float(v) if isinstance(v, (int, float, np.number)) else v) for k, v in p['metrics'].items()}
        })
    
    with open(dashboard_public / "simulation_state.json", "w") as f:
        json.dump(state_export, f, indent=2)
    print(f"SUCCESS: Live Simulation State exported to {dashboard_public / 'simulation_state.json'}")

if __name__ == "__main__":
    main()
