use crate::vm::errors::memory_errors::MemoryError;
use num_bigint::BigInt;
use std::fmt;

use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;

use super::exec_scope_errors::ExecScopeError;

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
    OffsetExceeded(BigInt),
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
    ExpectedRelocatable(MaybeRelocatable),
    ExpectedRelocatableAtAddr(MaybeRelocatable),
    FailedToGetIds,
    NonLeFelt(BigInt, BigInt),
    OutOfValidRange(BigInt, BigInt),
    FailedToGetReference(BigInt),
    ValueOutOfRange(BigInt),
    ValueNotPositive(BigInt),
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
    MainScopeError(ExecScopeError),
    ScopeError,
    VariableNotInScopeError(String),
    CantCreateDictionaryOnTakenSegment(usize),
    NoDictTracker(usize),
    NoValueForKey(BigInt),
    AssertLtFelt(BigInt, BigInt),
    FindElemMaxSize(BigInt, BigInt),
    InvalidIndex(BigInt, MaybeRelocatable, MaybeRelocatable),
    KeyNotFound,
    NoneApTrackingData,
    InvalidTrackingGroup(usize, usize),
    InvalidApValue(MaybeRelocatable),
    NoInitialDict,
    NoLocalVariable(String),
    NoKeyInAccessIndices(BigInt),
    EmptyAccessIndices,
    EmptyCurrentAccessIndices,
    CurrentAccessIndicesNotEmpty,
    WrongPrevValue(BigInt, Option<BigInt>, BigInt),
    NumUsedAccessesAssertFail(BigInt, usize, BigInt),
    KeysNotEmpty,
    EmptyKeys,
    PtrDiffNotDivisibleByDictAccessSize,
    SquashDictMaxSizeExceeded(BigInt, BigInt),
    NAccessesTooBig(BigInt),
    BigintToUsizeFail,
    InvalidSetRange(MaybeRelocatable, MaybeRelocatable),
    MismatchedDictPtr(Relocatable, Relocatable),
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
            VirtualMachineError::OffsetExceeded(n) => write!(f, "Offset {} exeeds maximum offset value", n),
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
            VirtualMachineError::ExpectedRelocatableAtAddr(addr) => {
                write!(f, "Expected relocatable at address {:?}", addr)
            }
            VirtualMachineError::ExpectedRelocatable(mayberelocatable) => {
                write!(f, "Expected address to be a Relocatable, got {:?}", mayberelocatable)
            },
            VirtualMachineError::FailedToGetIds => {
                write!(f, "Failed to get ids from memory")
            },
            VirtualMachineError::NonLeFelt(a, b) => {
                write!(f, "Assertion failed, {}, is not less or equal to {}", a, b)
            },
            VirtualMachineError::OutOfValidRange(div, max) => {
                write!(f, "Div out of range: 0 < {} <= {}", div, max)
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
            VirtualMachineError::ValueNotPositive(value) => write!(f, "Value: {:?} should be positive", value),
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
            VirtualMachineError::MainScopeError(error) => {
                write!(f, "Got scope error {}", error)
            },
            VirtualMachineError::VariableNotInScopeError(var_name) => {
                write!(f, "Variable {} not in local scope", var_name)
            },
            VirtualMachineError::ScopeError => write!(f, "Failed to get scope variables"),
            VirtualMachineError::CantCreateDictionaryOnTakenSegment(index) => {
                write!(f, "DictManagerError: Tried to create tracker for a dictionary on segment: {:?} when there is already a tracker for a dictionary on this segment", index)
            },
            VirtualMachineError::NoDictTracker(index) => {
                write!(f, "Dict Error: No dict tracker found for segment {:?}", index)
            },
            VirtualMachineError::NoValueForKey(key) => {
                write!(f, "Dict Error: No value found for key: {:?}", key)},
            VirtualMachineError::AssertLtFelt(a, b) => {
                write!(f, "Assertion failed, a = {} % PRIME is not less than b = {} % PRIME", a, b)
            },
            VirtualMachineError::NoInitialDict => {
                write!(f, "Dict Error: Tried to create a dict whithout an initial dict")
            },
            VirtualMachineError::NoLocalVariable(name) => {
                write!(f, "Hint Exception: Couldnt find local variable '{:?}'", name)
            },
            VirtualMachineError::NoKeyInAccessIndices(key) => {
                write!(f, "squash_dict_inner fail: couldnt find key {:?} in accesses_indices", key)
            },
            VirtualMachineError::EmptyAccessIndices =>{
                write!(f, "squash_dict_inner fail: local accessed_indices is empty")
            },
            VirtualMachineError::EmptyCurrentAccessIndices =>{
                write!(f, "squash_dict_inner fail: local current_accessed_indices is empty")
            },
            VirtualMachineError::CurrentAccessIndicesNotEmpty =>{
                write!(f, "squash_dict_inner fail: local current_accessed_indices not empty, loop ended with remaining unaccounted elements")
            },
            VirtualMachineError::WrongPrevValue(prev, current, key) => {
                write!(f, "Dict Error: Got the wrong value for dict_update, expected value: {:?}, got: {:?} for key: {:?}", prev, current, key)
            },
            VirtualMachineError::NoneApTrackingData => {
                write!(f, "AP tracking data is None; could not apply correction to address")
            },
            VirtualMachineError::InvalidTrackingGroup(group1, group2) => {
                write!(f, "Tracking groups should be the same, got {} and {}", group1, group2)
            },
            VirtualMachineError::InvalidApValue(addr) => {
                write!(f, "Expected relocatable for ap, got {:?}", addr)
            },
            VirtualMachineError::NumUsedAccessesAssertFail(used, len, key) => {
                write!(f, "squash_dict_inner fail: Number of used accesses:{:?} doesnt match the lengh: {:?} of the access_indices at key: {:?}", used, len, key)
            },
            VirtualMachineError::KeysNotEmpty =>{
                write!(f, "squash_dict_inner fail: local keys is not empty")
            },
            VirtualMachineError::EmptyKeys =>{
                write!(f, "squash_dict_inner fail: No keys left but remaining_accesses > 0")
            },
            VirtualMachineError::PtrDiffNotDivisibleByDictAccessSize =>{
                write!(f, "squash_dict fail: Accesses array size must be divisible by DictAccess.SIZE")
            },
            VirtualMachineError::SquashDictMaxSizeExceeded(max_size, n_accesses) =>{
                write!(f, "squash_dict() can only be used with n_accesses<={:?}. ' \nGot: n_accesses={:?}", max_size, n_accesses)
            },
            VirtualMachineError::NAccessesTooBig(n_accesses) => {
                write!(f, "squash_dict fail: n_accesses: {:?} is too big to be converted into an iterator", n_accesses)
            },
            VirtualMachineError::BigintToUsizeFail => write!(f, "Couldn't convert BigInt to usize"),
            VirtualMachineError::InvalidSetRange(start, end) => write!(f, "Set starting point {:?} is bigger it's ending point {:?}", start, end),
            VirtualMachineError::FindElemMaxSize(find_elem_max_size, n_elms) => write!(f, "find_elem() can only be used with n_elms <= {:?}.\nGot: n_elms = {:?}", find_elem_max_size, n_elms),
            VirtualMachineError::InvalidIndex(find_element_index, key, found_key) => write!(f, "Invalid index found in find_element_index. Index: {:?}.\nExpected key: {:?}, found_key {:?}", find_element_index, key, found_key),
            VirtualMachineError::KeyNotFound => write!(f, "Found Key is None"),
            VirtualMachineError::MismatchedDictPtr(current_ptr, dict_ptr) => write!(f, "Wrong dict pointer supplied. Got {:?}, expected {:?}.", dict_ptr, current_ptr),
        }
    }
}
