"""
Fuzzing the Coh-wedge Python bridge using Hypothesis.
Ensures the PyO3 boundary is robust against malformed or random data.
"""

import sys
import os
import json
from hypothesis import given, strategies as st, settings, Verbosity

# Add current dir to path to ensure we can import coh
sys.path.append(os.getcwd())
try:
    import coh
except ImportError:
    print("Error: 'coh' module not found. Build the project with 'maturin develop' or equivalent.")
    sys.exit(1)

# --- Strategies ---

# Random leaf/fragment strings
json_atoms = st.one_of(
    st.text(),
    st.integers(),
    st.floats(allow_nan=False, allow_infinity=False),
    st.booleans(),
    st.none()
)

# Recursive JSON-like objects
json_objects = st.recursive(
    json_atoms,
    lambda children: st.one_of(
        st.lists(children),
        st.dictionaries(st.text(), children)
    ),
    max_leaf_size=5
)

# Specifically malformed decimal strings
malformed_decimals = st.one_of(
    st.text(),
    st.from_regex(r"^-?\d+\.\d+e[+-]\d+$"),
    st.from_regex(r"^0x[0-9a-fA-F]+$"),
    st.just("Infinity"),
    st.just("NaN"),
    st.just("")
)

# --- Tests ---

@settings(max_examples=250, deadline=None)
@given(val=json_objects)
def test_fuzz_verify_random_dict(val):
    """Feed random JSON objects to verify(). Should not crash."""
    try:
        coh.verify(val)
    except (coh.CohMalformedError, coh.CohVerificationError, TypeError):
        pass
    except Exception as e:
        print(f"FAILED on dict fuzz: {type(e).__name__}: {e}")
        raise

@settings(max_examples=250, deadline=None)
@given(text=st.text())
def test_fuzz_verify_random_string(text):
    """Feed random strings to verify(). Should not crash."""
    try:
        coh.verify(text)
    except (coh.CohMalformedError, coh.CohVerificationError, TypeError):
        pass
    except Exception as e:
        print(f"FAILED on string fuzz: {type(e).__name__}: {e}")
        raise

@settings(max_examples=250, deadline=None)
@given(val=st.lists(json_objects, max_size=10))
def test_fuzz_verify_chain_random_list(val):
    """Feed random lists to verify_chain(). Should not crash."""
    try:
        coh.verify_chain(val)
    except (coh.CohMalformedError, coh.CohVerificationError, TypeError):
        pass
    except Exception as e:
        print(f"FAILED on chain fuzz: {type(e).__name__}: {e}")
        raise

@settings(max_examples=250, deadline=None)
@given(val=json_objects)
def test_fuzz_normalize(val):
    """Feed random JSON objects to normalize(). Should not crash."""
    try:
        coh.normalize(val)
    except (coh.CohMalformedError, coh.CohVerificationError, TypeError):
        pass
    except Exception as e:
        print(f"FAILED on normalize fuzz: {type(e).__name__}: {e}")
        raise

if __name__ == "__main__":
    print("Running Python Bridge Fuzz Pass (Hypothesis)...")
    
    # We run manually since pytest might not be set up
    successful = 0
    total = 0
    
    # Simple manual runner for the fuzzy tests
    for test_fn in [test_fuzz_verify_random_dict, test_fuzz_verify_random_string, test_fuzz_verify_chain_random_list, test_fuzz_normalize]:
        print(f" - Executing {test_fn.__name__}...")
        try:
            test_fn()
            print(f"   [PASS] {test_fn.__name__}")
            successful += 1
        except Exception as e:
            print(f"   [FAIL] {test_fn.__name__}: {e}")
        total += 1
        
    if successful == total:
        print(f"\nSUCCESS: All {total} fuzzer entrypoints held up under random pressure.")
    else:
        print(f"\nFAILURE: Only {successful}/{total} fuzzers passed.")
        sys.exit(1)
