use std::path::Path;

use cleopatra_cairo::{
    types::program::Program,
    vm::{runners::cairo_runner::CairoRunner, trace::trace_entry::RelocatedTraceEntry},
};

#[test]
fn struct_integration_test() {
    let program = Program::new(Path::new("cairo_programs/struct.json"), "main")
        .expect("Failed to deserialize program");
    let mut cairo_runner = CairoRunner::new(&program);
    cairo_runner.initialize_segments(None);
    let end = cairo_runner.initialize_main_entrypoint().unwrap();

    assert!(cairo_runner.initialize_vm() == Ok(()), "Execution failed");
    assert!(cairo_runner.run_until_pc(end) == Ok(()), "Execution failed");
    assert!(cairo_runner.relocate() == Ok(()), "Execution failed");
    let relocated_entry = RelocatedTraceEntry {
        pc: 1,
        ap: 4,
        fp: 4,
    };

    assert_eq!(cairo_runner.relocated_trace[0], relocated_entry);
}
