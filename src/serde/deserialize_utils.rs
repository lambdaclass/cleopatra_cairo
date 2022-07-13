use crate::serde::deserialize_program::ValueAddress;
use crate::types::instruction::Register;
use num_bigint::{BigInt, ParseBigIntError};
use num_integer::Integer;
use std::fmt;
use std::num::{IntErrorKind, ParseIntError};

#[derive(Debug, PartialEq)]
pub enum ReferenceParseError {
    IntError(ParseIntError),
    BigIntError(ParseBigIntError),
    InvalidStringError(String),
}

impl fmt::Display for ReferenceParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReferenceParseError::IntError(error) => {
                write!(f, "Int parsing error: ")?;
                error.fmt(f)
            }
            ReferenceParseError::BigIntError(error) => {
                write!(f, "BigInt parsing error: ")?;
                error.fmt(f)
            }
            ReferenceParseError::InvalidStringError(error) => {
                write!(f, "Invalid reference string error: ")?;
                error.fmt(f)
            }
        }
    }
}

// Checks if the hex string has an odd length.
// If that is the case, prepends '0' to it.
pub fn maybe_add_padding(mut hex: String) -> String {
    if hex.len().is_odd() {
        hex.insert(0, '0');
        return hex;
    }
    hex
}

fn parse_register(splitted_value_str: &[&str]) -> Option<Register> {
    let str_tmp: Vec<&str> = splitted_value_str[0].split(',').collect();

    let mut raw_reg_str = str_tmp[0].to_string();

    raw_reg_str.retain(|c| !r#"["#.contains(c));

    match raw_reg_str.split('(').collect::<Vec<_>>()[1] {
        "ap" => Some(Register::AP),
        "fp" => Some(Register::FP),
        _ => None,
    }
}

pub fn parse_dereference(value: &str) -> Result<ValueAddress, ReferenceParseError> {
    let splitted: Vec<&str> = value.split(" + ").collect();

    match splitted.len() {
        1 => parse_dereference_no_offsets(&splitted),
        2 => parse_dereference_with_one_offset(&splitted),
        3 => parse_dereference_with_two_offsets(&splitted),
        _ => Err(ReferenceParseError::InvalidStringError(String::from(value))),
    }
}
// parse string values of format `[cast(reg, *felt)]`
fn parse_dereference_no_offsets(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let register = parse_register(splitted_value_str);

    Ok(ValueAddress {
        register,
        offset1: 0,
        offset2: 0,
        immediate: None,
        dereference: true,
    })
}

// parse string values of format `[cast(reg + offset1, *felt)]`
fn parse_dereference_with_one_offset(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let mut deref = parse_dereference_no_offsets(splitted_value_str)?;

    let mut offset1_str = splitted_value_str[1].split(',').collect::<Vec<_>>()[0].to_string();
    offset1_str.retain(|c| !r#"()]"#.contains(c));

    let offset1: i32 = match offset1_str.parse() {
        Ok(offset1) => offset1,
        // for the moment, references with values that overflow i32 are not important, they are just dummy references.
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => 0,
            _ => return Err(ReferenceParseError::IntError(e)),
        },
    };
    deref.offset1 = offset1;

    Ok(deref)
}

// parse string values of format `[cast([reg + offset1] + offset2, *felt)]`
fn parse_dereference_with_two_offsets(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let mut deref = parse_dereference_with_one_offset(splitted_value_str)?;

    let mut offset2_str = splitted_value_str[2].split(',').collect::<Vec<_>>()[0].to_string();
    offset2_str.retain(|c| !r#"()"#.contains(c));

    let offset2: i32 = match offset2_str.parse() {
        Ok(offset2) => offset2,
        // for the moment, references with values that overflow i32 are not important, they are just dummy references.
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => 0,
            _ => return Err(ReferenceParseError::IntError(e)),
        },
    };
    deref.offset2 = offset2;

    Ok(deref)
}

pub fn parse_reference(value: &str) -> Result<ValueAddress, ReferenceParseError> {
    let splitted: Vec<_> = value.split(" + ").collect();

    match splitted.len() {
        1 => parse_reference_no_offsets(&splitted),
        2 => parse_reference_with_one_offset(&splitted),
        3 => parse_reference_with_two_offsets(&splitted),
        _ => Err(ReferenceParseError::InvalidStringError(String::from(value))),
    }
}

fn parse_reference_no_offsets(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let register = parse_register(splitted_value_str);

    Ok(ValueAddress {
        register,
        offset1: 0,
        offset2: 0,
        immediate: None,
        dereference: false,
    })
}

fn parse_reference_with_one_offset(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let mut refe = parse_reference_no_offsets(splitted_value_str)?;

    let mut offset1_str = splitted_value_str[1].split(',').collect::<Vec<_>>()[0].to_string();
    offset1_str.retain(|c| !r#"()]"#.contains(c));

    let offset1: i32 = match offset1_str.parse() {
        Ok(offset1) => offset1,
        // for the moment, references with values that overflow i32 are not important, they are just dummy references.
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => 0,
            _ => return Err(ReferenceParseError::IntError(e)),
        },
    };
    refe.offset1 = offset1;

    Ok(refe)
}

fn parse_reference_with_two_offsets(
    splitted_value_str: &[&str],
) -> Result<ValueAddress, ReferenceParseError> {
    let mut refe = parse_reference_with_one_offset(splitted_value_str)?;

    let mut immediate_str = splitted_value_str[2].split(',').collect::<Vec<_>>()[0].to_string();
    immediate_str.retain(|c| !r#"()"#.contains(c));

    let immediate: BigInt = match immediate_str.parse() {
        Ok(immediate) => immediate,
        Err(e) => return Err(ReferenceParseError::BigIntError(e)),
    };

    refe.immediate = Some(immediate);

    Ok(refe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint;
    use num_traits::FromPrimitive;

    #[test]
    fn parse_dereference_with_one_offset_test() {
        let value_string: &str = "[cast(fp + (-3), felt*)]";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_dereference_with_one_offset(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: -3,
            offset2: 0,
            immediate: None,
            dereference: true,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_dereference_with_two_offsets_test() {
        let value_string: &str = "[cast([fp + (-4)] + 1, felt*)]";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_dereference_with_two_offsets(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: -4,
            offset2: 1,
            immediate: None,
            dereference: true,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_dereference_no_offsets_test() {
        let value_string: &str = "[cast(fp, felt*)]";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_dereference_no_offsets(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: 0,
            offset2: 0,
            immediate: None,
            dereference: true,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_reference_with_one_offset_test() {
        let value_string: &str = "cast(fp + (-3), felt*)";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_reference_with_one_offset(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: -3,
            offset2: 0,
            immediate: None,
            dereference: false,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_reference_with_two_offsets_test() {
        let value_string: &str = "cast([fp + (-4)] + 1, felt*)";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_reference_with_two_offsets(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: -4,
            offset2: 0,
            immediate: Some(bigint!(1)),
            dereference: false,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_reference_no_offsets_test() {
        let value_string: &str = "cast(fp, felt*)";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_reference_no_offsets(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: 0,
            offset2: 0,
            immediate: None,
            dereference: false,
        };

        assert_eq!(value_address, parsed_value);
    }
}
