use c_mer::*;

#[path = "common.rs"]
mod common;
use common::*;

#[test]
fn test_activation_performance() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(1000, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);

    let (elapsed, _activated) = measure_time(|| memory.activate_fragments(&context));

    assert!(elapsed < 0.1);
}

#[test]
fn test_compilation_performance() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(500, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);

    let (elapsed, _eeg) = measure_time(|| compile_thought(&context, &mut memory));

    assert!(elapsed < 1.0);
}

#[test]
fn test_execution_performance() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(100, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);
    let eeg = compile_thought(&context, &mut memory);

    let (elapsed, _result) = measure_time(|| execute_eeg(&eeg, &mut memory));

    assert!(elapsed < 0.5);
}

#[test]
fn test_memory_scalability() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(10000, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    assert_eq!(memory.fragments.len(), 10000);

    let mut context = create_test_context("test", "test", 0.2);

    context.confidence_threshold = 0.0;

    let activated = memory.activate_fragments(&context);

    assert!(activated.len() >= 0);
    if activated.len() > 0 {
        assert!(activated.len() <= context.max_fragments);
    }
}

#[test]
fn test_compilation_scalability() {
    let mut memory = create_test_memory();
    let fragments = create_test_fragments(5000, "test");

    for fragment in fragments {
        memory.insert_fragment(fragment, Vec::new());
    }

    let context = create_test_context("test", "test", 0.2);

    let (elapsed, eeg) = measure_time(|| compile_thought(&context, &mut memory));

    assert!(eeg.nodes.len() > 0);
    assert!(elapsed < 2.0);
}
