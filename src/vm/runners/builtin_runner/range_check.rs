use std::any::Any;

use crate::bigint;
use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::memory_errors::MemoryError;
use crate::vm::errors::runner_errors::RunnerError;
use crate::vm::runners::builtin_runner::BuiltinRunner;
use crate::vm::vm_memory::memory::{Memory, ValidationRule};
use crate::vm::vm_memory::memory_segments::MemorySegmentManager;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

pub struct RangeCheckBuiltinRunner {
    included: bool,
    _ratio: BigInt,
    base: Option<Relocatable>,
    _stop_ptr: Option<Relocatable>,
    _cells_per_instance: i32,
    _n_input_cells: i32,
    _inner_rc_bound: BigInt,
    pub _bound: BigInt,
    _n_parts: u32,
}

impl RangeCheckBuiltinRunner {
    pub fn new(included: bool, ratio: BigInt, n_parts: u32) -> Self {
        let inner_rc_bound = bigint!(2_i32.pow(16));
        Self {
            included,
            _ratio: ratio,
            base: None,
            _stop_ptr: None,
            _cells_per_instance: 1,
            _n_input_cells: 1,
            _inner_rc_bound: inner_rc_bound.clone(),
            _bound: inner_rc_bound.pow(n_parts),
            _n_parts: n_parts,
        }
    }
}

impl BuiltinRunner for RangeCheckBuiltinRunner {
    fn initialize_segments(&mut self, segments: &mut MemorySegmentManager, memory: &mut Memory) {
        self.base = Some(segments.add(memory, None))
    }
    fn initial_stack(&self) -> Result<Vec<MaybeRelocatable>, RunnerError> {
        if self.included {
            if let Some(builtin_base) = &self.base {
                Ok(vec![MaybeRelocatable::RelocatableValue(
                    builtin_base.clone(),
                )])
            } else {
                Err(RunnerError::UninitializedBase)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn base(&self) -> Option<Relocatable> {
        self.base.clone()
    }

    fn add_validation_rule(&self, memory: &mut Memory) {
        let rule: ValidationRule = ValidationRule(Box::new(
            |memory: &Memory,
             address: &MaybeRelocatable|
             -> Result<MaybeRelocatable, MemoryError> {
                if let Some(MaybeRelocatable::Int(ref num)) = memory.get(address)? {
                    if bigint!(0) <= num.clone() && num.clone() < bigint!(2).pow(128) {
                        Ok(address.to_owned())
                    } else {
                        Err(MemoryError::NumOutOfBounds)
                    }
                } else {
                    Err(MemoryError::FoundNonInt)
                }
            },
        ));
        memory.add_validation_rule(self.base.as_ref().unwrap().segment_index, rule);
    }

    fn deduce_memory_cell(
        &mut self,
        _address: &MaybeRelocatable,
        _memory: &Memory,
    ) -> Result<Option<MaybeRelocatable>, RunnerError> {
        Ok(None)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::vm_memory::memory::Memory;
    use crate::{bigint, relocatable};

    #[test]
    fn initialize_segments_for_range_check() {
        let mut builtin = RangeCheckBuiltinRunner::new(true, bigint!(8), 8);
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        builtin.initialize_segments(&mut segments, &mut memory);
        assert_eq!(
            builtin.base,
            Some(Relocatable {
                segment_index: 0,
                offset: 0
            })
        );
    }

    #[test]
    fn get_initial_stack_for_range_check_included_with_base() {
        let mut builtin = RangeCheckBuiltinRunner::new(true, bigint!(8), 8);
        builtin.base = Some(relocatable!(1, 0));
        let initial_stack = builtin.initial_stack().unwrap();
        assert_eq!(
            initial_stack[0].clone(),
            MaybeRelocatable::RelocatableValue(builtin.base().unwrap())
        );
        assert_eq!(initial_stack.len(), 1);
    }

    #[test]
    fn get_initial_stack_for_range_check_included_without_base() {
        let builtin = RangeCheckBuiltinRunner::new(true, bigint!(8), 8);
        let error = builtin.initial_stack();
        assert_eq!(error, Err(RunnerError::UninitializedBase));
    }

    #[test]
    fn get_initial_stack_for_range_check_not_included() {
        let builtin = RangeCheckBuiltinRunner::new(false, bigint!(8), 8);
        let initial_stack = builtin.initial_stack().unwrap();
        assert_eq!(initial_stack.len(), 0);
    }
}
