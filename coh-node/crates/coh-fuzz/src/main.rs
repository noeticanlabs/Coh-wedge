use coh_core::types::{SlabReceiptWire, Decision};
use coh_core::verify_slab::verify_slab_envelope;
use libafl::{
    corpus::{InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{ExitKind, InProcessExecutor},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    monitors::SimpleMonitor,
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::StdState,
};
use libafl_bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice};
use std::path::PathBuf;

/// The main entry point for the LibAFL fuzzer.
/// This fuzzer targets `coh_core::verify_slab_envelope`.
fn main() {
    // 1. Define the observer for coverage (using a dummy map for now as we aren't using a compiler plugin)
    // In a real scenario, we'd use libafl_cc for edge coverage.
    // For this setup, we'll use a basic crash-oriented feedback.
    static mut MAP: [u8; 65536] = [0; 65536];
    let observer = unsafe { StdMapObserver::new("edges", &mut MAP) };

    // 2. Define the feedback mechanism
    let mut feedback = MaxMapFeedback::new(&observer);
    let mut objective = CrashFeedback::new();

    // 3. Define the state
    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        InMemoryCorpus::new(),
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    // 4. Define the monitor and event manager
    let monitor = SimpleMonitor::new(|s| println!("{}", s));
    let mut mgr = SimpleEventManager::new(monitor);

    // 5. Define the scheduler
    let scheduler = QueueScheduler::new();

    // 6. Define the fuzzer
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // 7. Define the harness (the function to be fuzzed)
    let mut harness = |input: &BytesInput| {
        let target = input.target_bytes();
        let buf = target.as_slice();

        // Attempt to deserialize the input as SlabReceiptWire
        // LibAFL will mutate the bytes, and we see if verify_slab_envelope crashes.
        if let Ok(wire) = serde_json::from_slice::<SlabReceiptWire>(buf) {
            let _ = verify_slab_envelope(wire);
        }

        ExitKind::Ok
    };

    // 8. Define the executor
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .unwrap();

    // 9. Initial seed (optional but recommended)
    if state.must_load_initial_inputs() {
        state
            .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &[PathBuf::from("./corpus")])
            .unwrap_or_else(|_| {
                println!("No initial corpus found, starting from scratch.");
            });
    }

    // 10. Define the stages
    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    // 11. Run the fuzzer
    println!("Starting the LibAFL fuzzer for Coh-wedge...");
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}
