#![deny(warnings)]
use clap::{Parser, ValueHint};
use cleopatra_cairo::cairo_run;
use cleopatra_cairo::vm::errors::cairo_run_errors::CairoRunError;
use cleopatra_cairo::vm::errors::runner_errors::RunnerError;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser, value_hint=ValueHint::FilePath)]
    filename: PathBuf,
    #[clap(long = "--trace_file", value_parser)]
    trace_file: Option<PathBuf>,
    #[structopt(long = "--print_output")]
    print_output: bool,
    #[structopt(long = "--entrypoint", default_value = "main")]
    entrypoint: String,
    trace: Option<PathBuf>,
    #[structopt(long = "--memory_file")]
    memory_file: Option<PathBuf>,
}

fn main() -> Result<(), CairoRunError> {
    let args = Args::parse();
    let mut cairo_runner = match cairo_run::cairo_run(&args.filename, &args.entrypoint) {
        Ok(runner) => runner,
        Err(error) => return Err(error),
    };

    if let Some(trace_path) = args.trace_file {
        match cairo_run::write_binary_trace(&cairo_runner.relocated_trace, &trace_path) {
            Ok(()) => (),
            Err(_e) => return Err(CairoRunError::Runner(RunnerError::WriteFail)),
        }
    }

    if args.print_output {
        cairo_run::write_output(&mut cairo_runner)?;
    }

    if let Some(memory_path) = args.memory_file {
        match cairo_run::write_binary_memory(&cairo_runner.relocated_memory, &memory_path) {
            Ok(()) => (),
            Err(_e) => return Err(CairoRunError::Runner(RunnerError::WriteFail)),
        }
    }

    Ok(())
}
