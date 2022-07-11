use std::any::Any;

use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;
use crate::vm::runners::builtin_runner::BuiltinRunner;
use crate::vm::vm_memory::memory::Memory;
use crate::vm::vm_memory::memory_segments::MemorySegmentManager;
use num_bigint::{BigInt, Sign};
use starknet_crypto::{pedersen_hash, FieldElement};

pub struct HashBuiltinRunner {
    pub base: Option<Relocatable>,
    included: bool,
    _ratio: usize,
    cells_per_instance: usize,
    _n_input_cells: usize,
    _stop_ptr: Option<Relocatable>,
    verified_addresses: Vec<MaybeRelocatable>,
}

impl HashBuiltinRunner {
    pub fn new(included: bool, ratio: usize) -> Self {
        Self {
            base: None,
            included,
            _ratio: ratio,
            cells_per_instance: 3,
            _n_input_cells: 2,
            _stop_ptr: None,
            verified_addresses: Vec::new(),
        }
    }
}

impl BuiltinRunner for HashBuiltinRunner {
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
            if relocatable.offset % self.cells_per_instance != 2
                || self.verified_addresses.contains(address)
            {
                return Ok(None);
            };
            if let (
                Ok(Some(MaybeRelocatable::Int(num_a))),
                Ok(Some(MaybeRelocatable::Int(num_b))),
            ) = (
                memory.get(&MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: relocatable.segment_index,
                    offset: relocatable.offset - 1,
                })),
                memory.get(&MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: relocatable.segment_index,
                    offset: relocatable.offset - 2,
                })),
            ) {
                self.verified_addresses.push(address.clone());

                //Convert MaybeRelocatable to FieldElement
                let a_string = num_a.to_str_radix(10);
                let b_string = num_b.to_str_radix(10);
                let (y, x) = match (
                    FieldElement::from_dec_str(&a_string),
                    FieldElement::from_dec_str(&b_string),
                ) {
                    (Ok(field_element_a), Ok(field_element_b)) => {
                        (field_element_a, field_element_b)
                    }
                    _ => return Err(RunnerError::FailedStringConversion),
                };
                //Compute pedersen Hash
                let fe_result = pedersen_hash(&x, &y);
                //Convert result from FieldElement to MaybeRelocatable
                let r_byte_slice = fe_result.to_bytes_be();
                let result = BigInt::from_bytes_be(Sign::Plus, &r_byte_slice);
                return Ok(Some(MaybeRelocatable::from(result)));
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
    use crate::vm::vm_memory::memory::Memory;
    use crate::{bigint, bigint_str};
    use num_traits::FromPrimitive;

    #[test]
    fn get_initial_stack_for_pedersen_not_included() {
        let builtin = HashBuiltinRunner::new(false, 8);
        let initial_stack = builtin.initial_stack();
        assert_eq!(initial_stack, Ok(Vec::new()));
    }

    #[test]
    fn get_initial_stack_for_pedersen_with_error() {
        let builtin = HashBuiltinRunner::new(true, 8);
        assert_eq!(builtin.initial_stack(), Err(RunnerError::UninitializedBase));
    }

    #[test]
    fn deduce_memory_cell_pedersen_for_preset_memory_valid() {
        let mut memory = Memory::new();
        let mut builtin = HashBuiltinRunner::new(true, 8);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 3)),
                &MaybeRelocatable::Int(bigint!(32)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::Int(bigint!(72)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 5)), &memory);
        assert_eq!(
            result,
            Ok(Some(MaybeRelocatable::from(bigint_str!(
                b"3270867057177188607814717243084834301278723532952411121381966378910183338911"
            ))))
        );
        assert_eq!(
            builtin.verified_addresses,
            vec![MaybeRelocatable::from((0, 5))]
        );
    }

    #[test]
    fn deduce_memory_cell_pedersen_for_preset_memory_incorrect_offset() {
        let mut memory = Memory::new();
        let mut builtin = HashBuiltinRunner::new(true, 8);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::Int(bigint!(32)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(72)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 6)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 6)), &memory);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn deduce_memory_cell_pedersen_for_preset_memory_already_computed() {
        let mut memory = Memory::new();
        let mut builtin = HashBuiltinRunner::new(true, 8);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 3)),
                &MaybeRelocatable::Int(bigint!(32)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::Int(bigint!(72)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::Int(bigint!(0)),
            )
            .unwrap();
        builtin.verified_addresses = vec![MaybeRelocatable::from((0, 5))];
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((0, 5)), &memory);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn deduce_memory_cell_pedersen_for_no_relocatable_address() {
        let memory = Memory::new();
        let mut builtin = HashBuiltinRunner::new(true, 8);
        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from(bigint!(5)), &memory);
        assert_eq!(result, Err(RunnerError::NonRelocatableAddress));
    }

    #[test]
    fn deduce_memory_cell_pedersen_for_preset_memory_no_values_to_hash() {
        let mut memory = Memory::new();
        let mut builtin = HashBuiltinRunner::new(true, 8);
        memory.data.push(Vec::new());
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::Int(bigint!(72)),
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
