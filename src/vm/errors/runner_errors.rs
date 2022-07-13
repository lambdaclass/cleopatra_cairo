use crate::types::relocatable::MaybeRelocatable;
use num_bigint::BigInt;
use std::fmt;

use super::memory_errors::MemoryError;

#[derive(Debug, PartialEq)]
pub enum RunnerError {
    NoExecBase,
    NoExecBaseForEntrypoint,
    NoProgBase,
    MissingMain,
    UninitializedBase,
    WriteFail,
    NoPC,
    NoAP,
    NoFP,
    MemoryValidationError(MemoryError),
    MemoryInitializationError(MemoryError),
    NonRelocatableAddress,
    FailedStringConversion,
    ExpectedInteger(MaybeRelocatable),
    MemoryGet(MaybeRelocatable),
    FailedMemoryGet(MemoryError),
    EcOpBuiltinScalarLimit(BigInt),
    FailedToParseIdsNameFromPath(String),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerError::NoExecBase => {
                write!(f, "Can't initialize state without an execution base")
            }
            RunnerError::NoProgBase => write!(f, "Can't without a program base"),
            RunnerError::NoExecBaseForEntrypoint => write!(
                f,
                "Can't initialize the function entrypoint without an execution base"
            ),
            RunnerError::MissingMain => write!(f, "Missing main()"),
            RunnerError::UninitializedBase => write!(f, "Uninitialized self.base"),
            RunnerError::WriteFail => write!(f, "Failed to write program output"),
            RunnerError::NoPC => write!(f, "Found None PC during VM initialization"),
            RunnerError::NoAP => write!(f, "Found None AP during VM initialization"),
            RunnerError::NoFP => write!(f, "Found None FP during VM initialization"),
            RunnerError::MemoryValidationError(error) => {
                write!(f, "Memory validation failed during VM initialization.")?;
                error.fmt(f)
            }
            RunnerError::MemoryInitializationError(error) => {
                write!(f, "Memory loading failed during state initialization.")?;
                error.fmt(f)
            }
            RunnerError::NonRelocatableAddress => write!(f, "Memory addresses must be relocatable"),
            RunnerError::FailedStringConversion => {
                write!(f, "Failed to convert string to FieldElement")
            }

            RunnerError::ExpectedInteger(addr) => {
                write!(f, "Expected integer at address {:?}", addr)
            }

            RunnerError::MemoryGet(addr) => {
                write!(f, "Failed to retrieve value from address {:?}", addr)
            }
            RunnerError::FailedMemoryGet(error) => {
                write!(f, "Failed to fetch memory address.")?;
                error.fmt(f)
            }

            RunnerError::EcOpBuiltinScalarLimit(scalar) => {
                write!(f, "EcOpBuiltin: m should be at most {}", scalar)
            }
            RunnerError::FailedToParseIdsNameFromPath(path) => {
                write!(
                    f,
                    "Failed to get variable name from path: {:?}, when parsing reference ids",
                    path
                )
            }
        }
    }
}
