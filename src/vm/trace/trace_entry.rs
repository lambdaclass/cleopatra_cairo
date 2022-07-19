///A trace entry for every instruction that was executed.
///Holds the register values before the instruction was executed.
use crate::types::relocatable::Relocatable;
use crate::vm::errors::trace_errors::TraceError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub struct TraceEntry {
    pub pc: u64,
    pub ap: u64,
    pub fp: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RelocatedTraceEntry {
    pub ap: usize,
    pub fp: usize,
    pub pc: usize,
}

pub fn relocate_trace_register(
    value: &Relocatable,
    relocation_table: &Vec<usize>,
) -> Result<usize, TraceError> {
    if relocation_table.len() <= value.segment_index {
        return Err(TraceError::NoRelocationFound);
    }
    Ok(relocation_table[value.segment_index] + value.offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relocate_relocatable_value() {
        let value = Relocatable {
            segment_index: 2,
            offset: 7,
        };
        let relocation_table = vec![1, 2, 5];
        assert_eq!(
            relocate_trace_register(&value, &relocation_table).unwrap(),
            12
        );
    }

    #[test]
    fn relocate_relocatable_value_no_relocation() {
        let value = Relocatable {
            segment_index: 2,
            offset: 7,
        };
        let relocation_table = vec![1, 2];
        let error = relocate_trace_register(&value, &relocation_table);
        assert_eq!(error, Err(TraceError::NoRelocationFound));
        assert_eq!(
            error.unwrap_err().to_string(),
            "No relocation found for this segment"
        );
    }
}
