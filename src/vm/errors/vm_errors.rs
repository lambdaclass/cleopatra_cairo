use crate::vm::errors::memory_errors::MemoryError;
use num_bigint::BigInt;
use std::fmt;

use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;

#[derive(Debug, PartialEq)]
pub enum VirtualMachineError {
    InvalidInstructionEncoding,
    InvalidOp1Reg(i64),
    ImmShouldBe1,
    UnknownOp0,
    InvalidApUpdate(i64),
    InvalidPcUpdate(i64),
    UnconstrainedResAdd,
    UnconstrainedResJump,
    UnconstrainedResJumpRel,
    UnconstrainedResAssertEq,
    DiffAssertValues(BigInt, BigInt),
    CantWriteReturnPc(BigInt, BigInt),
    CantWriteReturnFp(BigInt, BigInt),
    NoDst,
    PureValue,
    InvalidRes(i64),
    InvalidOpcode(i64),
    RelocatableAdd,
    OffsetExeeded(BigInt),
    NotImplemented,
    DiffIndexSub,
    InconsistentAutoDeduction(String, MaybeRelocatable, Option<MaybeRelocatable>),
    RunnerError(RunnerError),
    InvalidHintEncoding(MaybeRelocatable),
    MemoryError(MemoryError),
    NoRangeCheckBuiltin,
    IncorrectIds(Vec<String>, Vec<String>),
    MemoryGet(MaybeRelocatable),
    ExpectedInteger(MaybeRelocatable),
    FailedToGetIds,
    NonLeFelt(BigInt, BigInt),
    FailedToGetReference(BigInt),
    ValueOutOfRange(BigInt),
    UnknownHint(String),
    ValueOutsideValidRange(BigInt),
    SplitIntNotZero,
    SplitIntLimbOutOfRange(BigInt),
    DiffTypeComparison(MaybeRelocatable, MaybeRelocatable),
    AssertNotEqualFail(MaybeRelocatable, MaybeRelocatable),
    DiffIndexComp(Relocatable, Relocatable),
    ValueOutside250BitRange(BigInt),
    SqrtNegative(BigInt),
    FailedToGetSqrt(BigInt),
    AssertNotZero(BigInt, BigInt),
}

impl fmt::Display for VirtualMachineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VirtualMachineError::InvalidInstructionEncoding => {
                write!(f, "Instruction should be an int. Found:")
            }
            VirtualMachineError::InvalidOp1Reg(n) => write!(f, "Invalid op1_register value: {}", n),
            VirtualMachineError::ImmShouldBe1 => {
                write!(f, "In immediate mode, off2 should be 1")
            }
            VirtualMachineError::UnknownOp0 => {
                write!(f, "op0 must be known in double dereference")
            }
            VirtualMachineError::InvalidApUpdate(n) => write!(f, "Invalid ap_update value: {}", n),
            VirtualMachineError::InvalidPcUpdate(n) => write!(f, "Invalid pc_update value: {}", n),
            VirtualMachineError::UnconstrainedResAdd => {
                write!(f, "Res.UNCONSTRAINED cannot be used with ApUpdate.ADD")
            }
            VirtualMachineError::UnconstrainedResJump => {
                write!(f, "Res.UNCONSTRAINED cannot be used with PcUpdate.JUMP")
            }
            VirtualMachineError::UnconstrainedResJumpRel => {
                write!(f, "Res.UNCONSTRAINED cannot be used with PcUpdate.JUMP_REL")
            }
            VirtualMachineError::UnconstrainedResAssertEq => {
                write!(f, "Res.UNCONSTRAINED cannot be used with Opcode.ASSERT_EQ")
            }
            VirtualMachineError::DiffAssertValues(res, dst) => write!(f, "ASSERT_EQ instruction failed; res:{} != dst:{}", res, dst),
            VirtualMachineError::CantWriteReturnPc(op0, ret_pc) => write!(f, "Call failed to write return-pc (inconsistent op0): {} != {}. Did you forget to increment ap?", op0, ret_pc),
            VirtualMachineError::CantWriteReturnFp(dst, ret_fp) => write!(f, "Call failed to write return-fp (inconsistent dst): {} != {}. Did you forget to increment ap?", dst, ret_fp),
            VirtualMachineError::NoDst => write!(f,  "Couldn't get or load dst"),
            VirtualMachineError::InvalidRes(n) => write!(f, "Invalid res value: {}", n),
            VirtualMachineError::InvalidOpcode(n) => write!(f, "Invalid opcode value: {}", n),
            VirtualMachineError::RelocatableAdd => {
                write!(f, "Cannot add two relocatable values")
            }
            VirtualMachineError::OffsetExeeded(n) => write!(f, "Offset {} exeeds maximum offset value", n),
            VirtualMachineError::NotImplemented => write!(f, "This is not implemented"),
            VirtualMachineError::PureValue => Ok(()),
            VirtualMachineError::DiffIndexSub => write!(
                f,
                "Can only subtract two relocatable values of the same segment"
            ),
            VirtualMachineError::InconsistentAutoDeduction(builtin_name, expected_value, current_value) => {
                write!(f, "Inconsistent auto-deduction for builtin {}, expected {:?}, got {:?}", builtin_name, expected_value, current_value)
            },
            VirtualMachineError::RunnerError(runner_error) => runner_error.fmt(f),
            VirtualMachineError::InvalidHintEncoding(address) => write!(f, "Invalid hint encoding at pc: {:?}", address),
            VirtualMachineError::NoRangeCheckBuiltin => {
                write!(f, "Expected range_check builtin to be present")
            },
            VirtualMachineError::IncorrectIds(expected, existing) => {
                write!(f, "Expected ids to contain {:?}, got: {:?}", expected, existing)
            },
            VirtualMachineError::MemoryGet(addr) => {
                write!(f, "Failed to retrieve value from address {:?}", addr)
            },
            VirtualMachineError::ExpectedInteger(addr) => {
                write!(f, "Expected integer at address {:?}", addr)
            },
            VirtualMachineError::FailedToGetIds => {
                write!(f, "Failed to get ids from memory")
            },
            VirtualMachineError::NonLeFelt(a, b) => {
                write!(f, "Assertion failed, {}, is not less or equal to {}", a, b)
            },
            VirtualMachineError::FailedToGetReference(reference_id) => {
                write!(f, "Failed to get reference for id {}", reference_id)
            },
            VirtualMachineError::ValueOutOfRange(a) => {
                write!(f, "Assertion failed, 0 <= ids.a % PRIME < range_check_builtin.bound \n a = {:?} is out of range", a)
            },
            VirtualMachineError::UnknownHint(hint_code) => write!(f, "Unknown Hint: {:?}", hint_code),
            VirtualMachineError::MemoryError(memory_error) => memory_error.fmt(f),
            VirtualMachineError::ValueOutsideValidRange(value) => write!(f, "Value: {:?} is outside valid range", value),
            VirtualMachineError::SplitIntNotZero => write!(f,"split_int(): value is out of range"),
            VirtualMachineError::SplitIntLimbOutOfRange(limb) => write!(f, "split_int(): Limb {:?} is out of range.", limb),
            VirtualMachineError::DiffTypeComparison(a, b) => {
                write!(f, "Failed to compare {:?} and  {:?}, cant compare a relocatable to an integer value", a, b)
            },
            VirtualMachineError::AssertNotEqualFail(a, b) => {
                write!(f, "assert_not_equal failed: {:?} =  {:?}", a, b)
            },
            VirtualMachineError::DiffIndexComp(a, b) => {
                write!(f, "Failed to compare {:?} and  {:?}, cant compare two relocatable values of different segment indexes", a, b)
            },
            VirtualMachineError::ValueOutside250BitRange(value) => write!(f, "Value: {:?} is outside of the range [0, 2**250)", value),
            VirtualMachineError::SqrtNegative(value) => write!(f, "Can't calculate the square root of negative number: {:?})", value),
            VirtualMachineError::FailedToGetSqrt(value) => write!(f, "Failed to calculate the square root of: {:?})", value),
            VirtualMachineError::AssertNotZero(value, prime) => {
                write!(f, "Assertion failed, {} % {} is equal to 0", value, prime)
            },
        }
    }
}
