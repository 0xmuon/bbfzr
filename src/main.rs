extern crate libafl;
extern crate libafl_bolts;

use libafl::{
    bolts::{current_nanos, rands::StdRand},
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::RandBytesGenerator,
    inputs::{BytesInput, HasTargetBytes},
    monitors::SimpleMonitor,
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    stages::StdMutationalStage,
    state::{HasCorpus, HasMetadata, StdState},
};

use std::path::PathBuf;

fn harness(input: &BytesInput) -> ExitKind {
    let target = input.target_bytes();
    let buf = target.as_slice();
    if buf.len() > 0 && buf[0] == b'a' {
        if buf.len() > 1 && buf[1] == b'b' {
            if buf.len() > 2 && buf[2] == b'c' {
                panic!("Crash found with input: {:?}", buf);
            }
        }
    }
    ExitKind::Ok
}

fn main() {
    let monitor = SimpleMonitor::new(|s| println!("{}", s));
    let mut mgr = SimpleEventManager::new(monitor);
    let corpus_dir = PathBuf::from("./corpus");
    let mut corpus = InMemoryCorpus::new();
    let crashes = OnDiskCorpus::new("./crashes").unwrap();
    let generator = RandBytesGenerator::new(16); 
    let mutator = StdScheduledMutator::new(havoc_mutations());
    let observer = StdMapObserver::new("shared_mem", vec![0u8; 65536].as_mut_slice());
    let feedback = MaxMapFeedback::new(&observer);
    let objective = CrashFeedback::new();
    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        corpus,
        crashes,
        &mutator,
    )
    .unwrap();

   
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut state,
        &mut mgr,
    )
    .unwrap();

    let mut fuzzer = StdFuzzer::new(feedback, objective);

    
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .unwrap();
}
