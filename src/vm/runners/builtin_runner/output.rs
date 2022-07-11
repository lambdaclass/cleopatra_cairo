use std::any::Any;

use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;
use crate::vm::runners::builtin_runner::BuiltinRunner;
use crate::vm::vm_memory::memory::Memory;
use crate::vm::vm_memory::memory_segments::MemorySegmentManager;

pub struct OutputBuiltinRunner {
    included: bool,
    base: Option<Relocatable>,
    _stop_ptr: Option<Relocatable>,
}

impl OutputBuiltinRunner {
    pub fn new(included: bool) -> Self {
        Self {
            included,
            base: None,
            _stop_ptr: None,
        }
    }
}

impl BuiltinRunner for OutputBuiltinRunner {
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
    use crate::relocatable;
    use crate::vm::vm_memory::memory::Memory;

    #[test]
    fn initialize_segments_for_output() {
        let mut builtin = OutputBuiltinRunner::new(true);
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        builtin.initialize_segments(&mut segments, &mut memory);
        assert_eq!(builtin.base, Some(relocatable!(0, 0)));
    }

    #[test]
    fn get_initial_stack_for_output_included_with_base() {
        let mut builtin = OutputBuiltinRunner::new(true);
        builtin.base = Some(Relocatable {
            segment_index: 1,
            offset: 0,
        });
        let initial_stack = builtin.initial_stack().unwrap();
        assert_eq!(
            initial_stack[0].clone(),
            MaybeRelocatable::RelocatableValue(builtin.base().unwrap())
        );
        assert_eq!(initial_stack.len(), 1);
    }

    #[test]
    fn get_initial_stack_for_output_included_without_base() {
        let builtin = OutputBuiltinRunner::new(true);
        let error = builtin.initial_stack();
        assert_eq!(error, Err(RunnerError::UninitializedBase));
    }

    #[test]
    fn get_initial_stack_for_output_not_included() {
        let builtin = OutputBuiltinRunner::new(false);
        let initial_stack = builtin.initial_stack().unwrap();
        assert_eq!(initial_stack.len(), 0);
    }
}
