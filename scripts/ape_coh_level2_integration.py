#!/usr/bin/env python3
"""
APE + COH Level 2 Integration Wrapper

This module shows how to integrate your Level 2 visualization script with real APE 
proposal generation and COH verification while preserving all the visualization.

Usage:
    python ape_coh_level2_integration.py
    
Or import in your existing script:
    from ape_coh_level2_integration import integrate_proposal_flow
    
    receipt, outcome, trajectory = integrate_proposal_flow(strategy="mutation", seed=42, idx=0)
"""

import numpy as np
from pathlib import Path
from typing import Optional, Tuple, Dict, Any

# Import bridges
from ape_cli_bridge import generate_ape_proposal, extract_budget_metrics, is_ape_available, normalize_for_coh
from coh_bridge import verify_receipt_coh, normalize_receipt, COH_AVAILABLE


# =============================================================================
# NUMERIC DOMAIN FREEZE
# =============================================================================
np.random.seed(52)
FLOAT_DTYPE = np.float64


# =============================================================================
# STRATEGY MAPPING
# =============================================================================
STRATEGY_TO_GROUP = {
    "mutation": "EXPLORE",
    "recombination": "EXPLOIT",
    "contradiction": "BRIDGE",
    "overflow": "PERTURB",
    "runtime": "ADVERSARY",
    "violation": "REPAIR",
}

GROUP_SHAPES = {
    "EXPLORE": dict(amp=0.18, f1=(3.0, 8.0), f2=(10.0, 20.0), noise=0.012),
    "EXPLOIT": dict(amp=0.07, f1=(2.0, 4.5), f2=(5.0, 11.0), noise=0.004),
    "BRIDGE": dict(amp=0.14, f1=(2.0, 6.0), f2=(7.0, 15.0), noise=0.008),
    "PERTURB": dict(amp=0.24, f1=(5.0, 11.0), f2=(14.0, 28.0), noise=0.016),
    "ADVERSARY": dict(amp=0.34, f1=(6.0, 14.0), f2=(18.0, 34.0), noise=0.022),
    "REPAIR": dict(amp=0.05, f1=(1.5, 3.0), f2=(4.0, 8.0), noise=0.003),
}


# =============================================================================
# INTEGRATION FLOW
# =============================================================================
def integrate_proposal_flow(
    strategy: str,
    seed: int,
    idx: int,
) -> Tuple[Optional[Dict[str, Any]], str, Optional[np.ndarray]]:
    """
    Complete integration: APE → COH → Trajectory
    
    Args:
        strategy: APE strategy (mutation, recombination, etc.)
        seed: Random seed for APE generation
        idx: Proposal index for trajectory
    
    Returns:
        Tuple of (receipt, outcome, trajectory)
    """
    group = STRATEGY_TO_GROUP.get(strategy, "EXPLORE")
    
    # Step 1: Generate real proposal
    receipt = generate_ape_proposal(strategy, seed)
    
    if receipt is None:
        return None, "GENERATION_FAIL", None
    
    # Step 2: Normalize for COH (strip extra fields)
    receipt_for_coh = normalize_for_coh(receipt)
    
    # Step 3: Recompute cryptographic hashes
    receipt_for_coh = normalize_receipt(receipt_for_coh)
    
    # Step 4: Verify with COH
    outcome, detail, path = verify_receipt_coh(receipt_for_coh)
    
    # Step 3: Compute trajectory using real budget data
    metrics = extract_budget_metrics(receipt)
    trajectory = compute_trajectory(group, metrics, idx)
    
    # Store verification data in receipt
    receipt["_verification"] = {
        "outcome": outcome,
        "detail": detail,
        "path": path,
    }
    
    return receipt, outcome, trajectory


def compute_trajectory(
    group: str,
    metrics: Dict[str, float],
    idx: int,
) -> np.ndarray:
    """
    Compute trajectory geometry using real budget data.
    
    This replaces your synthetic proposal_trajectory() with data-driven geometry.
    """
    x = np.linspace(-1.0, 1.0, 560)
    rng = np.random.default_rng(1000 + idx)
    
    # Extract real budget values
    v_pre = metrics.get("v_pre", 100)
    v_post = metrics.get("v_post", 70)
    spend = metrics.get("spend", 20)
    defect = metrics.get("defect", 5)
    margin = metrics.get("margin", 10)
    
    # Compute pressure and other ratios
    total_budget = max(v_pre + defect, 1.0)
    pressure = spend / total_budget
    defect_ratio = defect / total_budget
    margin_ratio = margin / total_budget
    
    # Get group shape parameters
    shape = GROUP_SHAPES.get(group, GROUP_SHAPES["EXPLORE"])
    
    # Scale amplitude by real budget health
    amp = shape["amp"] * (1.0 + 0.8 * pressure + 0.4 * max(-margin_ratio, 0))
    f1 = rng.uniform(*shape["f1"])
    f2 = rng.uniform(*shape["f2"])
    noise_scale = shape["noise"] * (1.0 + 0.9 * pressure)
    
    # Initial position from budget
    y0 = np.clip(
        0.9 * (pressure - 0.25) + 0.6 * defect_ratio - 0.9 * margin_ratio,
        -0.78, 0.78
    )
    
    # Generate base trajectory
    drift = np.clip(0.25 * (pressure - defect_ratio) - 0.4 * margin_ratio, -0.25, 0.25)
    phase1 = rng.uniform(0, 2 * np.pi)
    phase2 = rng.uniform(0, 2 * np.pi)
    
    raw = (
        y0
        + amp * np.sin(f1 * (x + 1.0) + phase1)
        + 0.45 * amp * np.sin(f2 * (x + 1.0) + phase2)
        + drift * (x + 1.0)
    )
    
    # Add noise
    noise = np.cumsum(rng.normal(0, noise_scale, 560))
    noise -= noise.mean()
    raw += 0.20 * noise
    
    return raw.astype(FLOAT_DTYPE)


# =============================================================================
# BATCH PROCESSING
# =============================================================================
def process_batch(
    strategies: list[str],
    n: int = 520,
    start_seed: int = 1000,
) -> list[Tuple[Dict[str, Any], str, np.ndarray]]:
    """
    Process N proposals through the full pipeline.
    
    Args:
        strategies: List of strategies to cycle through
        n: Total proposals
        start_seed: Starting seed
    
    Returns:
        List of (receipt, outcome, trajectory) tuples
    """
    results = []
    
    for i in range(n):
        strategy = strategies[i % len(strategies)]
        seed = start_seed + i
        
        result = integrate_proposal_flow(strategy, seed, i)
        results.append(result)
        
        if (i + 1) % 100 == 0:
            print(f"Processed {i + 1}/{n}")
    
    return results


# =============================================================================
# SUMMARY STATISTICS
# =============================================================================
def compute_summary_stats(results: list) -> Dict[str, Any]:
    """Compute summary statistics from results."""
    outcomes = [r[1] for r in results if r[0] is not None]
    
    from collections import Counter
    counts = Counter(outcomes)
    
    accepted = counts.get("ACCEPT", 0)
    total = len(outcomes)
    accept_rate = accepted / total if total > 0 else 0
    
    return {
        "total": total,
        "accepted": accepted,
        "rejected": total - accepted,
        "accept_rate": accept_rate,
        "counts": dict(counts),
    }


# =============================================================================
# SELF-TEST
# =============================================================================
if __name__ == "__main__":
    print("APE + COH Level 2 Integration")
    print("=" * 50)
    
    print(f"APE available: {is_ape_available()}")
    print(f"COH available: {COH_AVAILABLE}")
    print()
    
    # Test single proposal flow
    print("Testing single proposal flow:")
    receipt, outcome, trajectory = integrate_proposal_flow("mutation", seed=42, idx=0)
    
    if receipt:
        v_info = receipt.get("_verification", {})
        detail = v_info.get("detail")
        print(f"  Outcome: {outcome}")
        if detail: print(f"  Detail: {detail}")
        if trajectory is not None:
            print(f"  Trajectory shape: {trajectory.shape}")
            print(f"  Trajectory range: [{trajectory.min():.3f}, {trajectory.max():.3f}]")
        
        metrics = extract_budget_metrics(receipt)
        print(f"  Metrics: v_pre={metrics['v_pre']}, v_post={metrics['v_post']}, "
              f"spend={metrics['spend']}, margin={metrics['margin']:.2f}")
    else:
        print("  Generation failed - APE binary may not be built")
        print("  Falling back to synthetic mode")
    
    print()
    
    # Quick batch test (20 proposals)
    print("Running quick batch test (20 proposals):")
    strategies = list(STRATEGY_TO_GROUP.keys())
    results = process_batch(strategies, n=20, start_seed=1000)
    
    stats = compute_summary_stats(results)
    print(f"  Total: {stats['total']}")
    print(f"  Accepted: {stats['accepted']} ({stats['accept_rate']:.1%})")
    print(f"  Counts: {stats['counts']}")