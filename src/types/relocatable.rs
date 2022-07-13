use crate::vm::errors::{memory_errors::MemoryError, vm_errors::VirtualMachineError};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Signed, ToPrimitive};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct Relocatable {
    pub segment_index: usize,
    pub offset: usize,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum MaybeRelocatable {
    RelocatableValue(Relocatable),
    Int(BigInt),
}

impl From<(usize, usize)> for Relocatable {
    fn from(index_offset: (usize, usize)) -> Self {
        Relocatable {
            segment_index: index_offset.0,
            offset: index_offset.1,
        }
    }
}

impl From<(usize, usize)> for MaybeRelocatable {
    fn from(index_offset: (usize, usize)) -> Self {
        MaybeRelocatable::RelocatableValue(Relocatable::from(index_offset))
    }
}

impl From<BigInt> for MaybeRelocatable {
    fn from(num: BigInt) -> Self {
        MaybeRelocatable::Int(num)
    }
}

impl MaybeRelocatable {
    ///Adds a bigint to self, then performs mod prime
    pub fn add_int_mod(
        &self,
        other: &BigInt,
        prime: &BigInt,
    ) -> Result<MaybeRelocatable, VirtualMachineError> {
        match *self {
            MaybeRelocatable::Int(ref value) => {
                Ok(MaybeRelocatable::Int((value + other).mod_floor(prime)))
            }
            MaybeRelocatable::RelocatableValue(ref rel) => {
                let mut big_offset = rel.offset + other;
                assert!(
                    !big_offset.is_negative(),
                    "Address offsets cant be negative"
                );
                big_offset = big_offset.mod_floor(prime);
                let new_offset = match big_offset.to_usize() {
                    Some(usize) => usize,
                    None => return Err(VirtualMachineError::OffsetExeeded(big_offset)),
                };
                Ok(MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: new_offset,
                }))
            }
        }
    }
    ///Adds a usize to self, then performs mod prime if prime is given
    pub fn add_usize_mod(&self, other: usize, prime: Option<BigInt>) -> MaybeRelocatable {
        match *self {
            MaybeRelocatable::Int(ref value) => {
                let mut num = value + other;
                if let Some(num_prime) = prime {
                    num = num.mod_floor(&num_prime);
                }
                MaybeRelocatable::Int(num)
            }
            MaybeRelocatable::RelocatableValue(ref rel) => {
                let new_offset = rel.offset + other;
                MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: new_offset,
                })
            }
        }
    }

    ///Adds a MaybeRelocatable to self, then performs mod prime
    /// Cant add two relocatable values
    pub fn add_mod(
        &self,
        other: &MaybeRelocatable,
        prime: &BigInt,
    ) -> Result<MaybeRelocatable, VirtualMachineError> {
        match (self, other) {
            (&MaybeRelocatable::Int(ref num_a_ref), MaybeRelocatable::Int(num_b)) => {
                let num_a = Clone::clone(num_a_ref);
                Ok(MaybeRelocatable::Int((num_a + num_b).mod_floor(prime)))
            }
            (&MaybeRelocatable::RelocatableValue(_), &MaybeRelocatable::RelocatableValue(_)) => {
                Err(VirtualMachineError::RelocatableAdd)
            }
            (&MaybeRelocatable::RelocatableValue(ref rel), &MaybeRelocatable::Int(ref num_ref))
            | (&MaybeRelocatable::Int(ref num_ref), &MaybeRelocatable::RelocatableValue(ref rel)) =>
            {
                let big_offset: BigInt = (num_ref + rel.offset).mod_floor(prime);
                let new_offset = match big_offset.to_usize() {
                    Some(usize) => usize,
                    None => return Err(VirtualMachineError::OffsetExeeded(big_offset)),
                };
                Ok(MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: new_offset,
                }))
            }
        }
    }
    ///Substracts two MaybeRelocatable values and returns the result as a MaybeRelocatable value.
    /// Only values of the same type may be substracted.
    /// Relocatable values can only be substracted if they belong to the same segment.
    pub fn sub(
        &self,
        other: &MaybeRelocatable,
        prime: &BigInt,
    ) -> Result<MaybeRelocatable, VirtualMachineError> {
        match (self, other) {
            (&MaybeRelocatable::Int(ref num_a), &MaybeRelocatable::Int(ref num_b)) => {
                Ok(MaybeRelocatable::Int((num_a - num_b).mod_floor(prime)))
            }
            (
                MaybeRelocatable::RelocatableValue(rel_a),
                MaybeRelocatable::RelocatableValue(rel_b),
            ) => {
                if rel_a.segment_index == rel_b.segment_index {
                    return Ok(MaybeRelocatable::RelocatableValue(Relocatable {
                        segment_index: rel_a.segment_index,
                        offset: rel_a.offset - rel_b.offset,
                    }));
                }
                Err(VirtualMachineError::DiffIndexSub)
            }
            _ => Err(VirtualMachineError::NotImplemented),
        }
    }

    /// Performs mod floor for a MaybeRelocatable::Int with BigInt
    pub fn mod_floor(&self, other: &BigInt) -> Result<MaybeRelocatable, VirtualMachineError> {
        match self {
            MaybeRelocatable::Int(value) => Ok(MaybeRelocatable::Int(value.mod_floor(other))),
            _ => Err(VirtualMachineError::NotImplemented),
        }
    }
}

///Turns a MaybeRelocatable into a BigInt value
/// If the value is an Int, it will extract the BigInt value from it
/// If the value is Relocatable, it will relocate it using the relocation_table
pub fn relocate_value(
    value: MaybeRelocatable,
    relocation_table: &Vec<usize>,
) -> Result<BigInt, MemoryError> {
    match value {
        MaybeRelocatable::Int(num) => Ok(num),
        MaybeRelocatable::RelocatableValue(relocatable) => {
            if relocation_table.len() <= relocatable.segment_index {
                return Err(MemoryError::Relocation);
            }
            match BigInt::from_usize(
                relocation_table[relocatable.segment_index] + relocatable.offset,
            ) {
                None => Err(MemoryError::Relocation),
                Some(relocated_value) => Ok(relocated_value),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint;
    use crate::bigint_str;
    use crate::relocatable;
    use num_bigint::BigInt;
    use num_bigint::Sign;
    use num_traits::FromPrimitive;

    #[test]
    fn add_bigint_to_int() {
        let addr = MaybeRelocatable::from(bigint!(7));
        let added_addr = addr.add_int_mod(&bigint!(2), &bigint!(17));
        assert_eq!(Ok(MaybeRelocatable::Int(bigint!(9))), added_addr);
    }

    #[test]
    fn add_usize_to_int() {
        let addr = MaybeRelocatable::from(bigint!(7));
        let added_addr = addr.add_usize_mod(2, Some(bigint!(17)));
        assert_eq!(MaybeRelocatable::Int(bigint!(9)), added_addr);
    }

    #[test]
    fn add_bigint_to_relocatable() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(7, 65));
        let added_addr = addr.add_int_mod(&bigint!(2), &bigint!(121));
        assert_eq!(Ok(MaybeRelocatable::from((7, 67))), added_addr);
    }

    #[test]
    fn add_int_mod_offset_exceeded() {
        let addr = MaybeRelocatable::from((0, 0));
        let error = addr.add_int_mod(
            &bigint_str!(b"18446744073709551616"),
            &bigint_str!(b"18446744073709551617"),
        );
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExeeded(bigint_str!(
                b"18446744073709551616"
            )))
        );
        assert_eq!(
            error.unwrap_err().to_string(),
            "Offset 18446744073709551616 exeeds maximum offset value"
        );
    }

    #[test]
    fn add_usize_to_relocatable() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(7, 65));
        let added_addr = addr.add_int_mod(&bigint!(2), &bigint!(121));
        assert_eq!(Ok(MaybeRelocatable::from((7, 67))), added_addr);
    }

    #[test]
    fn add_bigint_to_int_prime_mod() {
        let addr = MaybeRelocatable::Int(BigInt::new(
            Sign::Plus,
            vec![
                43680, 0, 0, 0, 0, 0, 0, 2013265920, 4294967289, 4294967295, 4294967295,
                4294967295, 4294967295, 4294967295, 4294967295, 1048575,
            ],
        ));
        let added_addr = addr.add_int_mod(
            &bigint!(1),
            &BigInt::new(
                Sign::Plus,
                vec![
                    4294967089, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295,
                    4294967295, 67108863,
                ],
            ),
        );
        assert_eq!(Ok(MaybeRelocatable::Int(bigint!(4))), added_addr);
    }

    #[test]
    fn add_bigint_to_relocatable_prime() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(1, 9));
        let added_addr = addr.add_int_mod(
            &BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            &BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(1, 9))),
            added_addr
        );
    }

    #[test]
    fn add_int_to_int() {
        let addr_a = &MaybeRelocatable::from(bigint!(7));
        let addr_b = &MaybeRelocatable::from(bigint!(17));
        let added_addr = addr_a.add_mod(addr_b, &bigint!(71));
        assert_eq!(Ok(MaybeRelocatable::from(bigint!(24))), added_addr);
    }

    #[test]
    fn add_int_to_int_prime() {
        let addr_a = &MaybeRelocatable::Int(BigInt::new(
            Sign::Plus,
            vec![1, 0, 0, 0, 0, 0, 17, 134217728],
        ));
        let addr_b = &MaybeRelocatable::from(bigint!(17));
        let added_addr = addr_a.add_mod(
            addr_b,
            &BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(Ok(MaybeRelocatable::from(bigint!(17))), added_addr);
    }

    #[test]
    fn add_relocatable_to_relocatable_should_fail() {
        let addr_a = &MaybeRelocatable::from((7, 5));
        let addr_b = &MaybeRelocatable::RelocatableValue(relocatable!(7, 10));
        let error = addr_a.add_mod(addr_b, &bigint!(17));
        assert_eq!(error, Err(VirtualMachineError::RelocatableAdd));
        assert_eq!(
            error.unwrap_err().to_string(),
            "Cannot add two relocatable values"
        );
    }

    #[test]
    fn add_int_to_relocatable() {
        let addr_a = &MaybeRelocatable::from((7, 7));
        let addr_b = &MaybeRelocatable::from(bigint!(10));
        let added_addr = addr_a.add_mod(addr_b, &bigint!(21));
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 17))),
            added_addr
        );
    }

    #[test]
    fn add_relocatable_to_int() {
        let addr_a = &MaybeRelocatable::from(bigint!(10));
        let addr_b = &MaybeRelocatable::RelocatableValue(relocatable!(7, 7));
        let added_addr = addr_a.add_mod(addr_b, &bigint!(21));
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 17))),
            added_addr
        );
    }

    #[test]
    fn add_int_to_relocatable_prime() {
        let addr_a = &MaybeRelocatable::from((7, 14));
        let addr_b = &MaybeRelocatable::Int(BigInt::new(
            Sign::Plus,
            vec![1, 0, 0, 0, 0, 0, 17, 134217728],
        ));
        let added_addr = addr_a.add_mod(
            addr_b,
            &BigInt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 14))),
            added_addr
        );
    }

    #[test]
    fn add_int_rel_int_offset_exceeded() {
        let addr = MaybeRelocatable::from((0, 0));
        let error = addr.add_mod(
            &MaybeRelocatable::from(bigint_str!(b"18446744073709551616")),
            &bigint_str!(b"18446744073709551617"),
        );
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExeeded(bigint_str!(
                b"18446744073709551616"
            )))
        );
    }

    #[test]
    fn add_int_int_rel_offset_exceeded() {
        let addr = MaybeRelocatable::Int(bigint_str!(b"18446744073709551616"));
        let relocatable = Relocatable {
            offset: 0,
            segment_index: 0,
        };
        let error = addr.add_mod(
            &MaybeRelocatable::RelocatableValue(relocatable),
            &bigint_str!(b"18446744073709551617"),
        );
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExeeded(bigint_str!(
                b"18446744073709551616"
            )))
        );
    }

    #[test]
    fn sub_int_from_int() {
        let addr_a = &MaybeRelocatable::from(bigint!(7));
        let addr_b = &MaybeRelocatable::from(bigint!(5));
        let sub_addr = addr_a.sub(addr_b, &bigint!(23));
        assert_eq!(Ok(MaybeRelocatable::from(bigint!(2))), sub_addr);
    }

    #[test]
    fn sub_relocatable_from_relocatable_same_offset() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from((7, 7));
        let sub_addr = addr_a.sub(addr_b, &bigint!(23));
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 10))),
            sub_addr
        );
    }

    #[test]
    fn sub_relocatable_from_relocatable_diff_offset() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from((8, 7));
        let error = addr_a.sub(addr_b, &bigint!(23));
        assert_eq!(error, Err(VirtualMachineError::DiffIndexSub));
        assert_eq!(
            error.unwrap_err().to_string(),
            "Can only subtract two relocatable values of the same segment"
        );
    }

    #[test]
    fn sub_int_addr_ref_from_relocatable_addr_ref() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from(bigint!(5));
        let error = addr_a.sub(addr_b, &bigint!(23));
        assert_eq!(error, Err(VirtualMachineError::NotImplemented));
        assert_eq!(error.unwrap_err().to_string(), "This is not implemented");
    }

    #[test]
    fn mod_floor_int() {
        let num = MaybeRelocatable::Int(bigint!(7));
        let div = bigint!(5);
        let expected_rem = MaybeRelocatable::Int(bigint!(2));
        assert_eq!(num.mod_floor(&div), Ok(expected_rem));
    }

    #[test]
    fn relocate_relocatable_value() {
        let value = MaybeRelocatable::from((2, 7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(relocate_value(value, &relocation_table), Ok(bigint!(12)));
    }

    #[test]
    fn relocate_int_value() {
        let value = MaybeRelocatable::from(bigint!(7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(relocate_value(value, &relocation_table), Ok(bigint!(7)));
    }

    #[test]
    fn relocate_relocatable_value_no_relocation() {
        let value = MaybeRelocatable::from((2, 7));
        let relocation_table = vec![1, 2];
        assert_eq!(
            relocate_value(value, &relocation_table),
            Err(MemoryError::Relocation)
        );
    }
}
