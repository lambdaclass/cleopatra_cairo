use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::memory_errors::MemoryError;
use crate::vm::vm_memory::memory::Memory;

pub struct MemorySegmentManager {
    pub num_segments: usize,
    pub segment_used_sizes: Option<Vec<usize>>,
}

impl MemorySegmentManager {
    ///Adds a new segment and returns its starting location as a RelocatableValue.
    ///If size is not None the segment is finalized with the given size. (size will be always none for initialization)
    pub fn add(&mut self, memory: &mut Memory, _size: Option<usize>) -> Relocatable {
        let segment_index = self.num_segments;
        self.num_segments += 1;
        memory.data.push(Vec::new());
        Relocatable::from((segment_index, 0))
    }
    ///Writes data into the memory at address ptr and returns the first address after the data.
    pub fn load_data(
        &mut self,
        memory: &mut Memory,
        ptr: &MaybeRelocatable,
        data: Vec<MaybeRelocatable>,
    ) -> Result<MaybeRelocatable, MemoryError> {
        for (num, value) in data.iter().enumerate() {
            memory.insert(&ptr.add_usize_mod(num, None), value)?;
        }
        Ok(ptr.add_usize_mod(data.len(), None))
    }

    pub fn new() -> MemorySegmentManager {
        MemorySegmentManager {
            num_segments: 0,
            segment_used_sizes: None,
        }
    }

    ///Calculates the size (number of non-none elements) of each memory segment
    pub fn compute_effective_sizes(&mut self, memory: &Memory) {
        if self.segment_used_sizes != None {
            return;
        }
        let mut segment_used_sizes = Vec::new();
        for segment in memory.data.iter() {
            segment_used_sizes.push(segment.len());
        }
        self.segment_used_sizes = Some(segment_used_sizes);
    }

    ///Returns a vector that contains the first relocated address of each memory segment
    pub fn relocate_segments(&self) -> Result<Vec<usize>, MemoryError> {
        let first_addr = 1;
        let mut relocation_table = vec![first_addr];
        match &self.segment_used_sizes {
            Some(segment_used_sizes) => {
                for (i, size) in segment_used_sizes.iter().enumerate() {
                    relocation_table.push(relocation_table[i] + size);
                }
            }
            None => return Err(MemoryError::EffectiveSizesNotCalled),
        }
        //The last value corresponds to the total amount of elements across all segments, which isnt needed for relocation.
        relocation_table.pop();
        Ok(relocation_table)
    }
}

impl Default for MemorySegmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{bigint, relocatable};
    use num_bigint::BigInt;
    use num_traits::FromPrimitive;

    use super::*;

    #[test]
    fn add_segment_no_size() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        let base = segments.add(&mut memory, None);
        assert_eq!(base, relocatable!(0, 0));
        assert_eq!(segments.num_segments, 1);
    }

    #[test]
    fn add_segment_no_size_test_two_segments() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        let mut _base = segments.add(&mut memory, None);
        _base = segments.add(&mut memory, None);
        assert_eq!(_base, Relocatable::from((1, 0)),);
        assert_eq!(segments.num_segments, 2);
    }

    #[test]
    fn load_data_empty() {
        let data = Vec::new();
        let ptr = MaybeRelocatable::from((0, 3));
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        let current_ptr = segments.load_data(&mut memory, &ptr, data).unwrap();
        assert_eq!(current_ptr, MaybeRelocatable::from((0, 3)));
    }

    #[test]
    fn load_data_one_element() {
        let data = vec![MaybeRelocatable::from(bigint!(4))];
        let ptr = MaybeRelocatable::from((0, 0));
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        let current_ptr = segments.load_data(&mut memory, &ptr, data).unwrap();
        assert_eq!(current_ptr, MaybeRelocatable::from((0, 1)));
        assert_eq!(
            memory.get(&ptr).unwrap(),
            Some(&MaybeRelocatable::from(bigint!(4)))
        );
    }

    #[test]
    fn load_data_three_elements() {
        let data = vec![
            MaybeRelocatable::from(bigint!(4)),
            MaybeRelocatable::from(bigint!(5)),
            MaybeRelocatable::from(bigint!(6)),
        ];
        let ptr = MaybeRelocatable::from((0, 0));
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        let current_ptr = segments.load_data(&mut memory, &ptr, data).unwrap();
        assert_eq!(current_ptr, MaybeRelocatable::from((0, 3)));

        assert_eq!(
            memory.get(&ptr).unwrap(),
            Some(&MaybeRelocatable::from(bigint!(4)))
        );
        assert_eq!(
            memory.get(&MaybeRelocatable::from((0, 1))).unwrap(),
            Some(&MaybeRelocatable::from(bigint!(5)))
        );
        assert_eq!(
            memory.get(&MaybeRelocatable::from((0, 2))).unwrap(),
            Some(&MaybeRelocatable::from(bigint!(6)))
        );
    }
    #[test]
    fn compute_effective_sizes_for_one_segment_memory() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        memory
            .insert(
                &MaybeRelocatable::from((0, 0)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 1)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        segments.compute_effective_sizes(&memory);
        assert_eq!(Some(vec![3]), segments.segment_used_sizes);
    }

    #[test]
    fn compute_effective_sizes_for_one_segment_memory_with_gap() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        memory
            .insert(
                &MaybeRelocatable::from((0, 6)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        segments.compute_effective_sizes(&memory);
        assert_eq!(Some(vec![7]), segments.segment_used_sizes);
    }

    #[test]
    fn compute_effective_sizes_for_one_segment_memory_with_gaps() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        memory
            .insert(
                &MaybeRelocatable::from((0, 3)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 4)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 7)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 9)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        segments.compute_effective_sizes(&memory);
        assert_eq!(Some(vec![10]), segments.segment_used_sizes);
    }

    #[test]
    fn compute_effective_sizes_for_three_segment_memory() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        segments.add(&mut memory, None);
        segments.add(&mut memory, None);
        memory
            .insert(
                &MaybeRelocatable::from((0, 0)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 1)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((1, 0)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((1, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 0)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 1)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();

        segments.compute_effective_sizes(&memory);
        assert_eq!(Some(vec![3, 3, 3]), segments.segment_used_sizes);
    }

    #[test]
    fn compute_effective_sizes_for_three_segment_memory_with_gaps() {
        let mut segments = MemorySegmentManager::new();
        let mut memory = Memory::new();
        segments.add(&mut memory, None);
        segments.add(&mut memory, None);
        segments.add(&mut memory, None);
        memory
            .insert(
                &MaybeRelocatable::from((0, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 5)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((0, 7)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((1, 1)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 2)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 4)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((2, 7)),
                &MaybeRelocatable::from(bigint!(1)),
            )
            .unwrap();
        segments.compute_effective_sizes(&memory);
        assert_eq!(Some(vec![8, 2, 8]), segments.segment_used_sizes);
    }

    #[test]
    fn relocate_segments_one_segment() {
        let mut segments = MemorySegmentManager::new();
        segments.segment_used_sizes = Some(vec![3]);
        assert_eq!(
            segments
                .relocate_segments()
                .expect("Couldn't relocate after compute effective sizes"),
            vec![1]
        )
    }

    #[test]
    fn relocate_segments_five_segment() {
        let mut segments = MemorySegmentManager::new();
        segments.segment_used_sizes = Some(vec![3, 3, 56, 78, 8]);
        assert_eq!(
            segments
                .relocate_segments()
                .expect("Couldn't relocate after compute effective sizes"),
            vec![1, 4, 7, 63, 141]
        )
    }
}
