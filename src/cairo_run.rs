use crate::types::program::Program;
use crate::vm::errors::cairo_run_errors::CairoRunError;
use crate::vm::runners::cairo_runner::CairoRunner;
use crate::vm::trace::trace_entry::RelocatedTraceEntry;
use num_bigint::BigInt;
use std::fs::File;
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;

pub fn cairo_run(path: &Path, entrypoint: &str) -> Result<CairoRunner, CairoRunError> {
    let program = match Program::new(path, entrypoint) {
        Ok(program) => program,
        Err(error) => return Err(CairoRunError::Program(error)),
    };

    let mut cairo_runner = CairoRunner::new(&program);
    cairo_runner.initialize_segments(None);

    let end = match cairo_runner.initialize_main_entrypoint() {
        Ok(end) => end,
        Err(error) => return Err(CairoRunError::Runner(error)),
    };

    if let Err(error) = cairo_runner.initialize_vm() {
        return Err(CairoRunError::Runner(error));
    }

    if let Err(error) = cairo_runner.run_until_pc(end) {
        return Err(CairoRunError::VirtualMachine(error));
    }

    if let Err(error) = cairo_runner.vm.verify_auto_deductions() {
        return Err(CairoRunError::VirtualMachine(error));
    }

    if let Err(error) = cairo_runner.relocate() {
        return Err(CairoRunError::Trace(error));
    }

    Ok(cairo_runner)
}

pub fn write_output(cairo_runner: &mut CairoRunner) -> Result<(), CairoRunError> {
    if let Err(error) = cairo_runner.write_output(&mut io::stdout()) {
        return Err(CairoRunError::Runner(error));
    }
    Ok(())
}

/// Writes a trace as a binary file. Bincode encodes to little endian by default and each trace
/// entry is composed of 3 usize values that are padded to always reach 64 bit size.
pub fn write_binary_trace(
    relocated_trace: &[RelocatedTraceEntry],
    trace_file: &Path,
) -> io::Result<()> {
    let mut buffer = File::create(trace_file)?;

    for (i, entry) in relocated_trace.iter().enumerate() {
        if let Err(e) = bincode::serialize_into(&mut buffer, entry) {
            let error_string =
                format!("Failed to dump trace at position {i}, serialize error: {e}");
            return Err(Error::new(ErrorKind::Other, error_string));
        }
    }

    Ok(())
}

/*
   Writes a binary memory file with the relocated memory as input.
   The memory pairs (address, value) are encoded and concatenated in the file
   given by the path `memory_file`.

   * address -> 8-byte encoded
   * value -> 32-byte encoded
*/
pub fn write_binary_memory(
    relocated_memory: &[Option<BigInt>],
    memory_file: &Path,
) -> io::Result<()> {
    let mut buffer = File::create(memory_file)?;

    // initialize bytes vector that will be dumped to file
    let mut memory_bytes: Vec<u8> = Vec::new();

    // filters None's from the relocated memory
    let filtered_memory_iter = relocated_memory.iter().filter(|x| !x.is_none());
    for (i, memory_cell) in filtered_memory_iter.enumerate() {
        match memory_cell {
            None => continue,
            Some(unwrapped_memory_cell) => {
                encode_relocated_memory(&mut memory_bytes, i, unwrapped_memory_cell);
            }
        }
    }

    match buffer.write(&memory_bytes) {
        Err(e) => Err(e),
        Ok(_) => Ok(()),
    }
}

// encodes a given memory cell.
fn encode_relocated_memory(memory_bytes: &mut Vec<u8>, addr: usize, memory_cell: &BigInt) {
    // append memory address to bytes vector using a 8 bytes representation
    let mut addr_bytes = (addr as u64 + 1).to_le_bytes().to_vec();
    memory_bytes.append(&mut addr_bytes);

    // append memory value at address using a 32 bytes representation
    let mut value_bytes = memory_cell.to_signed_bytes_le();
    value_bytes.resize(32, 0);
    memory_bytes.append(&mut value_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn run_test_program(program_path: &Path) -> Result<CairoRunner, CairoRunError> {
        let program = match Program::new(program_path, "main") {
            Ok(program) => program,
            Err(e) => return Err(CairoRunError::Program(e)),
        };

        let mut cairo_runner = CairoRunner::new(&program);

        cairo_runner.initialize_segments(None);

        let end = match cairo_runner.initialize_main_entrypoint() {
            Ok(end) => end,
            Err(e) => return Err(CairoRunError::Runner(e)),
        };

        assert!(cairo_runner.initialize_vm().is_ok());

        assert!(cairo_runner.run_until_pc(end).is_ok());

        Ok(cairo_runner)
    }

    fn compare_files(file_path_1: &Path, file_path_2: &Path) -> io::Result<()> {
        let mut file_1 = File::open(file_path_1)?;
        let mut file_2 = File::open(file_path_2)?;

        let mut buffer_1 = Vec::new();
        let mut buffer_2 = Vec::new();

        file_1.read_to_end(&mut buffer_1)?;
        file_2.read_to_end(&mut buffer_2)?;

        assert_eq!(&buffer_1.len(), &buffer_2.len());

        for (buf_byte_1, buf_byte_2) in buffer_1.iter().zip(buffer_2.iter()) {
            assert_eq!(buf_byte_1, buf_byte_2);
        }
        Ok(())
    }

    #[test]
    fn cairo_run_with_no_data_program() {
        // a compiled program with no `data` key.
        // it should fail when the program is loaded.
        let no_data_program_path = Path::new("tests/support/no_data_program.json");

        assert!(cairo_run(no_data_program_path, "main").is_err());
    }

    #[test]
    fn cairo_run_with_no_main_program() {
        // a compiled program with no main scope
        // it should fail when trying to run initialize_main_entrypoint.
        let no_main_program_path = Path::new("tests/support/no_main_program.json");

        assert!(cairo_run(no_main_program_path, "main").is_err());
    }

    #[test]
    fn cairo_run_with_invalid_memory() {
        // the program invalid_memory.json has an invalid memory cell and errors when trying to
        // decode the instruction.
        let invalid_memory = Path::new("tests/support/invalid_memory.json");

        assert!(cairo_run(invalid_memory, "main").is_err());
    }

    #[test]
    fn write_binary_trace_file() {
        let program_path = Path::new("tests/support/struct.json");
        let expected_trace_path = Path::new("tests/support/struct.trace");
        let cleopatra_trace_path = Path::new("tests/support/struct_cleopatra.trace");

        // run test program until the end
        let cairo_runner_result = run_test_program(program_path);
        let mut cairo_runner = cairo_runner_result.unwrap();

        // relocate memory so we can dump it to file
        assert!(cairo_runner.relocate().is_ok());

        // write cleopatra vm trace file
        assert!(write_binary_trace(&cairo_runner.relocated_trace, cleopatra_trace_path).is_ok());

        // compare that the original cairo vm trace file and cleopatra vm trace files are equal
        assert!(compare_files(cleopatra_trace_path, expected_trace_path).is_ok());
    }

    #[test]
    fn write_binary_memory_file() {
        let program_path = Path::new("tests/support/struct.json");
        let expected_memory_path = Path::new("tests/support/struct.memory");
        let cleopatra_memory_path = Path::new("tests/support/struct_cleopatra.memory");

        // run test program until the end
        let cairo_runner_result = run_test_program(program_path);
        let mut cairo_runner = cairo_runner_result.unwrap();

        // relocate memory so we can dump it to file
        assert!(cairo_runner.relocate().is_ok());

        // write cleopatra vm memory file
        assert!(write_binary_memory(&cairo_runner.relocated_memory, cleopatra_memory_path).is_ok());

        // compare that the original cairo vm memory file and cleopatra vm memory files are equal
        assert!(compare_files(cleopatra_memory_path, expected_memory_path).is_ok());
    }
}
