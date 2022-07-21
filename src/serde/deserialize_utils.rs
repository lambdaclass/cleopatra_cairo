#[allow(dead_code)]
use crate::serde::deserialize_program::ValueAddress;
use crate::types::instruction::Register;
use nom::{
    branch::alt,
    bytes::{complete::take_until, streaming::tag},
    combinator::{map_res, opt},
    sequence::{delimited, tuple},
    IResult,
};
use num_bigint::{BigInt, ParseBigIntError};
use num_integer::Integer;
use parse_hyperlinks::take_until_unbalanced;
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

fn outer_brackets(input: &str) -> IResult<&str, bool> {
    opt(delimited(
        tag("["),
        take_until_unbalanced('[', ']'),
        tag("]"),
    ))(input)
    .map(|(reminder_input, consumed_input_res)| {
        if let Some(consumed_input) = consumed_input_res {
            (consumed_input, true)
        } else {
            (reminder_input, false)
        }
    })
}

fn take_cast(input: &str) -> IResult<&str, &str> {
    tuple((
        tag("cast"),
        delimited(tag("("), take_until_unbalanced('(', ')'), tag(")")),
    ))(input)
    .map(|(consumed_input, (_, inside_parenthesis))| (inside_parenthesis, consumed_input))
}

fn take_cast_first_arg(input: &str) -> IResult<&str, &str> {
    let (next_input, _) = take_cast(input)?;

    take_until(",")(next_input)
        .map(|(consumed_input, cast_first_arg)| (cast_first_arg, consumed_input))
}

fn register(input: &str) -> IResult<&str, Register> {
    alt((tag("ap"), tag("fp")))(input).map(|(consumed_input, res)| match res {
        "ap" => (consumed_input, Register::AP),
        "fp" => (consumed_input, Register::FP),
        _ => unreachable!(),
    })
}

// Examples:
// " + (-1)"
// " + 2"
fn offset(input: &str) -> IResult<&str, i32> {
    tuple((
        alt((tag(" + "), tag(" - "))),
        opt(delimited(tag("("), take_until(")"), tag(")"))),
    ))(input)
    .map(|(consumed_input, res)| {
        if let Some(offset) = res.1 {
            // FIXME handle error correctly
            let num = offset.parse::<i32>().unwrap();
            (consumed_input, num)
        } else {
            // FIXME handle error correctly
            let num = consumed_input.parse::<i32>().unwrap();
            (consumed_input, num)
        }
    })
}
// fp + 2
// ap + (-1)
fn register_and_offset(input: &str) -> IResult<&str, (Register, i32)> {
    tuple((register, offset))(input)
}

fn inside_brackets(input: &str) -> IResult<&str, (Register, i32)> {
    map_res(
        delimited(tag("["), take_until("]"), tag("]")),
        register_and_offset,
    )(input)
    .map(|(consumed_input, res)| {
        let (_, (register, offset)) = res;
        (consumed_input, (register, offset))
    })
}

// The final parser
fn parse_value(input: &str) -> IResult<&str, (bool, Option<(Register, i32)>, Option<i32>)> {
    tuple((
        outer_brackets,
        take_cast_first_arg,
        opt(alt((inside_brackets, register_and_offset))),
        opt(offset),
    ))(input)
    .map(|(consumed_input, res)| (consumed_input, (res.0, res.2, res.3)))
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

        // FIXME this match arm is handled like this just to avoid unnecesary deserialization errors.
        // For the moment, the ValueAddress structs returned in ths arm are not used in hints, so they are not important.
        // issue: https://github.com/lambdaclass/cleopatra_cairo/issues/280

        // _ => Err(ReferenceParseError::InvalidStringError(String::from(value))),
        _ => Ok(ValueAddress {
            register: Some(Register::FP),
            offset1: 0,
            offset2: 0,
            immediate: None,
            dereference: true,
            inner_dereference: false,
        }),
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
        inner_dereference: false,
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

    if splitted_value_str[0].contains(&"([") {
        deref.inner_dereference = true;
        return Ok(deref);
    }

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
    deref.inner_dereference = true;

    Ok(deref)
}

pub fn parse_reference(value: &str) -> Result<ValueAddress, ReferenceParseError> {
    let splitted: Vec<_> = value.split(" + ").collect();

    match splitted.len() {
        1 => parse_reference_no_offsets(&splitted),
        2 => parse_reference_with_one_offset(&splitted),
        3 => parse_reference_with_two_offsets(&splitted),

        // FIXME this match arm is handled like this just to avoid unnecesary deserialization errors.
        // For the moment, the ValueAddress structs returned in ths arm are not used in hints, so they are not important.
        // issue: https://github.com/lambdaclass/cleopatra_cairo/issues/280

        // _ => Err(ReferenceParseError::InvalidStringError(String::from(value))),
        _ => Ok(ValueAddress {
            register: Some(Register::FP),
            offset1: 0,
            offset2: 0,
            immediate: None,
            dereference: false,
            inner_dereference: false,
        }),
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
        inner_dereference: false,
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
            inner_dereference: false,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn parse_dereference_with_one_offset_and_inner_dereference_test() {
        let value_string: &str = "[cast([fp + (-3)], felt*)]";
        let splitted_value: Vec<&str> = value_string.split(" + ").collect();

        let parsed_value = parse_dereference_with_one_offset(&splitted_value).unwrap();

        let value_address = ValueAddress {
            register: Some(Register::FP),
            offset1: -3,
            offset2: 0,
            immediate: None,
            dereference: true,
            inner_dereference: true,
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
            inner_dereference: true,
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
            inner_dereference: false,
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
            inner_dereference: false,
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
            inner_dereference: false,
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
            inner_dereference: false,
        };

        assert_eq!(value_address, parsed_value);
    }

    #[test]
    fn outer_brackets_test() {}

    #[test]
    fn parse_value_test() {
        let value_1 = "[cast([fp + (-1)] + 2, felt*)]";
        let parsed_1 = parse_value(value_1);
        assert_eq!(
            parsed_1,
            Ok(("2", (true, Some((Register::FP, -1_i32)), Some(2_i32))))
        );

        let value_2 = "cast(ap + 2, felt*)";
        let parsed_2 = parse_value(value_2);
        assert_eq!(parsed_2, Ok(("2", (false, Some((Register::AP, 2)), None))));
    }
}
