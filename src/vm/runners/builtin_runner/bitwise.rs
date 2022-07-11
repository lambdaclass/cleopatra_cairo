use std::any::Any;

use crate::bigint;
use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;
use crate::vm::runners::builtin_runner::BuiltinRunner;
use crate::vm::vm_memory::memory::Memory;
use crate::vm::vm_memory::memory_segments::MemorySegmentManager;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

pub struct BitwiseBuiltinRunner {
    included: bool,
    _ratio: usize,
    pub base: Option<Relocatable>,
    cells_per_instance: usize,
    _n_input_cells: usize,
    total_n_bits: u32,
}

impl BitwiseBuiltinRunner {
    pub fn new(included: bool, ratio: usize) -> Self {
        Self {
            base: None,
            included,
            _ratio: ratio,
            cells_per_instance: 5,
            _n_input_cells: 2,
            total_n_bits: 251,
        }
    }
}

impl BuiltinRunner for BitwiseBuiltinRunner {
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

    fn add_validation_rule(&self, _memory: &mut Memory) {}

    fn deduce_memory_cell(
        &mut self,
        address: &MaybeRelocatable,
        memory: &Memory,
    ) -> Result<Option<MaybeRelocatable>, RunnerError> {
        if let &MaybeRelocatable::RelocatableValue(ref relocatable) = address {
            let index = relocatable.offset % self.cells_per_instance;
            if index == 0 || index == 1 {
                return Ok(None);
            }
            let x_addr =
                MaybeRelocatable::from((relocatable.segment_index, relocatable.offset - index));
            let y_addr = x_addr.add_usize_mod(1, None);
            if let (
                Ok(Some(MaybeRelocatable::Int(num_x))),
                Ok(Some(MaybeRelocatable::Int(num_y))),
            ) = (memory.get(&x_addr), memory.get(&y_addr))
            {
                assert!(
                    num_x < &bigint!(2).pow(self.total_n_bits),
                    "Expected integer at address {:?} to be smaller than 2^{}, Got {}",
                    x_addr,
                    self.total_n_bits,
                    num_x
                );
                assert!(
                    num_y < &bigint!(2).pow(self.total_n_bits),
                    "Expected integer at address {:?} to be smaller than 2^{}, Got {}",
                    y_addr,
                    self.total_n_bits,
                    num_y
                );
                let res = match index {
                    2 => Some(MaybeRelocatable::from(num_x & num_y)),
                    3 => Some(MaybeRelocatable::from(num_x ^ num_y)),
                    4 => Some(MaybeRelocatable::from(num_x | num_y)),
                    _ => None,
                };
                return Ok(res);
            }
            Ok(None)
        } else {
            Err(RunnerError::NonRelocatableAddress)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_initial_stack_for_bitwise_not_included() {
        let builtin = BitwiseBuiltinRunner::new(false, 8);
        let initial_stack = builtin.initial_stack();
        assert_eq!(initial_stack, Ok(Vec::new()));
    }

    #[test]
    fn get_initial_stack_for_bitwise_with_error() {
        let builtin = BitwiseBuiltinRunner::new(true, 8);
        assert_eq!(builtin.initial_stack(), Err(RunnerError::UninitializedBase));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_preset_memory_valid_and() {
        let mut memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(10)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 6)),
                &MaybeRelocatable::Int(bigint!(12)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 7)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 7)), &memory);
        assert_eq!(result, Ok(Some(MaybeRelocatable::from(bigint!(8)))));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_preset_memory_valid_xor() {
        let mut memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(10)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 6)),
                &MaybeRelocatable::Int(bigint!(12)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 8)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 8)), &memory);
        assert_eq!(result, Ok(Some(MaybeRelocatable::from(bigint!(6)))));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_preset_memory_valid_or() {
        let mut memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(10)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 6)),
                &MaybeRelocatable::Int(bigint!(12)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 9)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 9)), &memory);
        assert_eq!(result, Ok(Some(MaybeRelocatable::from(bigint!(14)))));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_preset_memory_no_values_to_operate() {
        let mut memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(12)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 7)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 5)), &memory);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_no_relocatable_address() {
        let memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from(bigint!(5)), &memory);
        assert_eq!(result, Err(RunnerError::NonRelocatableAddress));
    }

    #[test]
    fn deduce_memory_cell_bitwise_for_preset_memory_incorrect_offset() {
        let mut memory = Memory::new();
        let mut builtin = BitwiseBuiltinRunner::new(true, 256);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 3)),
                &MaybeRelocatable::Int(bigint!(10)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::Int(bigint!(12)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 5)), &memory);
        assert_eq!(result, Ok(None));
    }
}
