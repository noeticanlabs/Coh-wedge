import os
import glob
import re

examples_dir = "crates/coh-genesis/examples"
files = glob.glob(os.path.join(examples_dir, "*.rs"))

replacements = [
    (r"coh_genesis::mathlib_advisor::", r"coh_npe::tools::mathlib_advisor::"),
    (r"coh_genesis::mathlib_advisor", r"coh_npe::tools::mathlib_advisor"),
    (r"coh_genesis::lean_proof::", r"coh_npe::tools::lean_proof::"),
    (r"coh_genesis::lean_proof", r"coh_npe::tools::lean_proof"),
    (r"coh_genesis::code_patch::", r"coh_npe::tools::code_patch::"),
    (r"coh_genesis::code_patch", r"coh_npe::tools::code_patch"),
    (r"coh_genesis::failure_taxonomy::", r"coh_npe::failure_taxonomy::"),
    (r"coh_genesis::failure_taxonomy", r"coh_npe::failure_taxonomy"),
    (r"coh_genesis::phaseloom_lite::", r"coh_genesis::"), # most things were exported from coh_genesis root or coh_phaseloom
]

# Specifically for phaseloom_lite items:
# BoundaryReceiptSummary is in coh_npe::BoundaryReceiptSummary
# PhaseLoomConfig, PhaseLoomState, etc are in coh_phaseloom::
for file in files:
    with open(file, 'r', encoding='utf-8') as f:
        content = f.read()

    original = content
    for old, new in replacements:
        content = re.sub(old, new, content)

    # specific fixes for phaseloom_lite imports
    content = re.sub(r"coh_genesis::\{([^}]*PhaseLoomConfig[^}]*)\}", r"coh_phaseloom::{\1}", content)
    content = re.sub(r"coh_genesis::\{([^}]*PhaseLoomState[^}]*)\}", r"coh_phaseloom::{\1}", content)
    
    # BoundaryReceiptSummary
    content = re.sub(r"coh_genesis::\{([^}]*BoundaryReceiptSummary[^}]*)\}", r"coh_npe::{\1}", content)

    if content != original:
        print(f"Fixed {file}")
        with open(file, 'w', encoding='utf-8') as f:
            f.write(content)
