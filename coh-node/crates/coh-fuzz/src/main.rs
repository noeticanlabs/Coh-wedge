#[cfg(not(test))]
use {
    coh_core::types::SlabReceiptWire,
    coh_core::verify_slab::verify_slab_envelope,
    libafl::{
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
        Evaluator,
    },
    libafl_bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice},
    std::path::PathBuf,
};

#[cfg(not(test))]
fn main() {
    // 1. Define the observer for coverage
    // Using a local heap allocated map to avoid static mut alignment issues
    let mut map = vec![0u8; 65536];
    let map_ptr = map.as_mut_ptr();
    let map_slice = unsafe { std::slice::from_raw_parts_mut(map_ptr, 65536) };
    let observer = StdMapObserver::from_mut_slice("edges", map_slice.into());

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
        // Try multiple candidate paths so CI and local runs both work
        let candidates = [
            PathBuf::from("./corpus"),
            PathBuf::from("./crates/coh-fuzz/corpus"),
            PathBuf::from("../../crates/coh-fuzz/corpus"), // Try relative from project root if running from coh-node
        ];

        let mut loaded = false;
        for path in &candidates {
            if path.exists() {
                match state.load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &[path.clone()]) {
                    Ok(_) => {
                        println!("Loaded initial corpus from {}", path.display());
                        loaded = true;
                        break;
                    }
                    Err(e) => {
                        println!("Failed to load corpus from {}: {:?}", path.display(), e);
                    }
                }
            }
        }

        if !loaded {
            println!("No initial corpus found in expected locations, inserting dummy seed.");
            let dummy_input = BytesInput::new(vec![0u8; 16]);
            fuzzer
                .add_input(&mut state, &mut executor, &mut mgr, dummy_input)
                .expect("Failed to add dummy seed to corpus");
        }
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

#[cfg(test)]
fn main() {}
