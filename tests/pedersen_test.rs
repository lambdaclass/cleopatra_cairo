use std::path::Path;

use cleopatra_cairo::{
    types::program::Program, vm::runners::cairo_runner::CairoRunner,
    vm::trace::trace_entry::RelocatedTraceEntry,
};

#[test]
fn pedersen_integration_test() {
    let program = Program::new(Path::new("cairo_programs/pedersen_test.json"), "main")
        .expect("Failed to deserialize program");
    let mut cairo_runner = CairoRunner::new(&program);
    cairo_runner.initialize_segments(None);
    let end = cairo_runner.initialize_main_entrypoint().unwrap();
    assert!(cairo_runner.initialize_vm() == Ok(()), "Execution failed");
    assert!(cairo_runner.run_until_pc(end) == Ok(()), "Execution failed");
    assert!(cairo_runner.relocate() == Ok(()), "Execution failed");

    let python_vm_relocated_trace: Vec<RelocatedTraceEntry> = vec![
        RelocatedTraceEntry {
            pc: 7,
            ap: 25,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 8,
            ap: 26,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 10,
            ap: 27,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 12,
            ap: 28,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 1,
            ap: 30,
            fp: 30,
        },
        RelocatedTraceEntry {
            pc: 2,
            ap: 30,
            fp: 30,
        },
        RelocatedTraceEntry {
            pc: 3,
            ap: 30,
            fp: 30,
        },
        RelocatedTraceEntry {
            pc: 5,
            ap: 31,
            fp: 30,
        },
        RelocatedTraceEntry {
            pc: 6,
            ap: 32,
            fp: 30,
        },
        RelocatedTraceEntry {
            pc: 14,
            ap: 32,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 15,
            ap: 32,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 17,
            ap: 33,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 18,
            ap: 34,
            fp: 25,
        },
        RelocatedTraceEntry {
            pc: 19,
            ap: 35,
            fp: 25,
        },
    ];
    assert_eq!(cairo_runner.relocated_trace, python_vm_relocated_trace);
}
