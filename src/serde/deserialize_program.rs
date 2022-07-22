use crate::serde::deserialize_utils;
use crate::types::instruction::Register;
use crate::types::{
    errors::program_errors::ProgramError, program::Program, relocatable::MaybeRelocatable,
};
use num_bigint::{BigInt, Sign};
use num_traits::abs;
use serde::{de, de::MapAccess, de::SeqAccess, Deserialize, Deserializer};
use std::{collections::HashMap, fmt, fs::File, io::BufReader, path::Path};

#[derive(Deserialize, Debug)]
pub struct ProgramJson {
    #[serde(deserialize_with = "deserialize_bigint_hex")]
    pub prime: BigInt,
    pub builtins: Vec<String>,
    #[serde(deserialize_with = "deserialize_array_of_bigint_hex")]
    pub data: Vec<MaybeRelocatable>,
    pub identifiers: HashMap<String, Identifier>,
    pub hints: HashMap<usize, Vec<HintParams>>,
    pub reference_manager: ReferenceManager,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HintParams {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub accessible_scopes: Vec<String>,
    pub flow_tracking_data: FlowTrackingData,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct FlowTrackingData {
    pub ap_tracking: ApTracking,
    #[serde(deserialize_with = "deserialize_map_to_string_and_bigint_hashmap")]
    pub reference_ids: HashMap<String, BigInt>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ApTracking {
    pub group: usize,
    pub offset: usize,
}

impl ApTracking {
    pub fn new() -> ApTracking {
        ApTracking {
            group: 0,
            offset: 0,
        }
    }
}

impl Default for ApTracking {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Debug)]
pub struct Identifier {
    pub pc: Option<usize>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ReferenceManager {
    pub references: Vec<Reference>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Reference {
    pub ap_tracking_data: ApTracking,
    pub pc: Option<usize>,
    #[serde(deserialize_with = "deserialize_value_address")]
    #[serde(rename(deserialize = "value"))]
    pub value_address: ValueAddress,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ValueAddress {
    pub register: Option<Register>,
    pub offset1: i32,
    pub offset2: i32,
    pub immediate: Option<BigInt>,
    pub dereference: bool,
    pub inner_dereference: bool,
}

struct BigIntVisitor;

impl<'de> de::Visitor<'de> for BigIntVisitor {
    type Value = BigInt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Could not deserialize hexadecimal string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Strip the '0x' prefix from the encoded hex string
        if let Some(no_prefix_hex) = value.strip_prefix("0x") {
            // Add padding if necessary
            let no_prefix_hex = deserialize_utils::maybe_add_padding(no_prefix_hex.to_string());
            let decoded_result: Result<Vec<u8>, hex::FromHexError> = hex::decode(&no_prefix_hex);

            match decoded_result {
                Ok(decoded_hex) => Ok(BigInt::from_bytes_be(Sign::Plus, &decoded_hex)),
                Err(e) => Err(e).map_err(de::Error::custom),
            }
        } else {
            Err(String::from("hex prefix error")).map_err(de::Error::custom)
        }
    }
}

struct MaybeRelocatableVisitor;

impl<'de> de::Visitor<'de> for MaybeRelocatableVisitor {
    type Value = Vec<MaybeRelocatable>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Could not deserialize array of hexadecimal")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut data: Vec<MaybeRelocatable> = vec![];

        while let Some(value) = seq.next_element::<String>()? {
            if let Some(no_prefix_hex) = value.strip_prefix("0x") {
                // Add padding if necessary
                let no_prefix_hex = deserialize_utils::maybe_add_padding(no_prefix_hex.to_string());
                let decoded_result: Result<Vec<u8>, hex::FromHexError> =
                    hex::decode(&no_prefix_hex);

                match decoded_result {
                    Ok(decoded_hex) => data.push(MaybeRelocatable::Int(BigInt::from_bytes_be(
                        Sign::Plus,
                        &decoded_hex,
                    ))),
                    Err(e) => return Err(e).map_err(de::Error::custom),
                };
            } else {
                return Err(String::from("hex prefix error")).map_err(de::Error::custom);
            };
        }
        Ok(data)
    }
}

struct ReferenceIdsVisitor;

impl<'de> de::Visitor<'de> for ReferenceIdsVisitor {
    type Value = HashMap<String, BigInt>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with string keys and integer values")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut data: HashMap<String, BigInt> = HashMap::new();

        while let Some((key, value)) = map.next_entry::<String, i64>()? {
            if value >= 0 {
                data.insert(key, BigInt::from_bytes_le(Sign::Plus, &value.to_le_bytes()));
            } else {
                data.insert(
                    key,
                    BigInt::from_bytes_le(Sign::Minus, &abs(value).to_le_bytes()),
                );
            }
        }

        Ok(data)
    }
}

struct ValueAddressVisitor;

impl<'de> de::Visitor<'de> for ValueAddressVisitor {
    type Value = ValueAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing the address in memory of a variable")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let res = match value.chars().next() {
            Some('[') => deserialize_utils::parse_dereference(value).map_err(de::Error::custom)?,
            Some('c') => deserialize_utils::parse_reference(value).map_err(de::Error::custom)?,
            _c => return Err("Expected '[' or 'c' as first char").map_err(de::Error::custom),
        };
        //let (_, res) = deserialize_utils::parse_value(value).map_err(de::Error::custom)?;

        Ok(res)
    }
}

pub fn deserialize_bigint_hex<'de, D: Deserializer<'de>>(d: D) -> Result<BigInt, D::Error> {
    d.deserialize_str(BigIntVisitor)
}

pub fn deserialize_array_of_bigint_hex<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Vec<MaybeRelocatable>, D::Error> {
    d.deserialize_seq(MaybeRelocatableVisitor)
}

pub fn deserialize_map_to_string_and_bigint_hashmap<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, BigInt>, D::Error> {
    d.deserialize_map(ReferenceIdsVisitor)
}

pub fn deserialize_value_address<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<ValueAddress, D::Error> {
    d.deserialize_str(ValueAddressVisitor)
}

pub fn deserialize_program_json(path: &Path) -> Result<ProgramJson, ProgramError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let program_json = serde_json::from_reader(&mut reader)?;

    Ok(program_json)
}

pub fn deserialize_program(path: &Path) -> Result<Program, ProgramError> {
    let program_json: ProgramJson = deserialize_program_json(path)?;
    Ok(Program {
        builtins: program_json.builtins,
        prime: program_json.prime,
        data: program_json.data,
        main: program_json.identifiers["__main__.main"].pc,
        hints: program_json.hints,
        reference_manager: program_json.reference_manager,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint;
    use num_traits::FromPrimitive;

    #[test]
    fn deserialize_bigint_from_string_json_gives_error() {
        let invalid_even_length_hex_json = r#"
            {
                "prime": "0bx000A"
            }"#;

        // ProgramJson result instance for the json with an even length encoded hex.
        let even_result: Result<ProgramJson, _> =
            serde_json::from_str(&invalid_even_length_hex_json);

        assert!(even_result.is_err());

        let invalid_odd_length_hex_json = r#"
            {
                "prime": "0bx00A"
            }"#;

        // ProgramJson result instance for the json with an odd length encoded hex.
        let odd_result: Result<ProgramJson, _> = serde_json::from_str(&invalid_odd_length_hex_json);

        assert!(odd_result.is_err());
    }

    #[test]
    fn deserialize_from_string_json() {
        let valid_json = r#"
            {
                "prime": "0x000A",
                "builtins": [],
                "data": [
                    "0x480680017fff8000",
                    "0x3e8",
                    "0x480680017fff8000",
                    "0x7d0",
                    "0x48307fff7ffe8000",
                    "0x208b7fff7fff7ffe"
                ],
                "identifiers": {
                    "__main__.main": {
                        "decorators": [],
                        "pc": 0,
                        "type": "function"
                    },
                    "__main__.main.Args": {
                        "full_name": "__main__.main.Args",
                        "members": {},
                        "size": 0,
                        "type": "struct"
                    },
                    "__main__.main.ImplicitArgs": {
                        "full_name": "__main__.main.ImplicitArgs",
                        "members": {},
                        "size": 0,
                        "type": "struct"
                    }
                },
                "hints": {
                    "0": [
                        {
                            "accessible_scopes": [
                                "starkware.cairo.common.alloc",
                                "starkware.cairo.common.alloc.alloc"
                            ],
                            "code": "memory[ap] = segments.add()",
                            "flow_tracking_data": {
                                "ap_tracking": {
                                    "group": 0,
                                    "offset": 0
                                },
                                "reference_ids": {
                                    "starkware.cairo.common.math.split_felt.high": 0,
                                    "starkware.cairo.common.math.split_felt.low": -14,
                                    "starkware.cairo.common.math.split_felt.range_check_ptr": 16,
                                    "starkware.cairo.common.math.split_felt.value": 12
                                }
                            }
                        }
                    ]
                },
                "reference_manager": {
                    "references": [
                        {
                            "ap_tracking_data": {
                                "group": 0,
                                "offset": 0
                            },
                            "pc": 0,
                            "value": "[cast(fp + (-4), felt*)]"
                        },
                        {
                            "ap_tracking_data": {
                                "group": 0,
                                "offset": 0
                            },
                            "pc": 0,
                            "value": "[cast(fp + (-3), felt*)]"
                        },
                        {
                            "ap_tracking_data": {
                                "group": 0,
                                "offset": 0
                            },
                            "pc": 0,
                            "value": "cast([fp + (-3)] + 2, felt)"
                        },
                        {
                            "ap_tracking_data": {
                                "group": 0,
                                "offset": 0
                            },
                            "pc": 0,
                            "value": "[cast(fp, felt*)]"
                        }
                    ]
                }
            }"#;

        // ProgramJson instance for the json with an even length encoded hex.
        let program_json: ProgramJson = serde_json::from_str(&valid_json).unwrap();

        let builtins: Vec<String> = Vec::new();

        let data: Vec<MaybeRelocatable> = vec![
            MaybeRelocatable::Int(BigInt::from_i64(5189976364521848832).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(1000).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(5189976364521848832).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(2000).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(5201798304953696256).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(2345108766317314046).unwrap()),
        ];

        let mut hints: HashMap<usize, Vec<HintParams>> = HashMap::new();
        hints.insert(
            0,
            vec![HintParams {
                code: vec![
                    109, 101, 109, 111, 114, 121, 91, 97, 112, 93, 32, 61, 32, 115, 101, 103, 109,
                    101, 110, 116, 115, 46, 97, 100, 100, 40, 41,
                ],
                accessible_scopes: vec![
                    String::from("starkware.cairo.common.alloc"),
                    String::from("starkware.cairo.common.alloc.alloc"),
                ],
                flow_tracking_data: FlowTrackingData {
                    ap_tracking: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    reference_ids: HashMap::from([
                        (
                            String::from("starkware.cairo.common.math.split_felt.high"),
                            bigint!(0),
                        ),
                        (
                            String::from("starkware.cairo.common.math.split_felt.low"),
                            bigint!(-14),
                        ),
                        (
                            String::from("starkware.cairo.common.math.split_felt.range_check_ptr"),
                            bigint!(16),
                        ),
                        (
                            String::from("starkware.cairo.common.math.split_felt.value"),
                            bigint!(12),
                        ),
                    ]),
                },
            }],
        );

        let reference_manager = ReferenceManager {
            references: vec![
                Reference {
                    ap_tracking_data: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    pc: Some(0),
                    value_address: ValueAddress {
                        register: Some(Register::FP),
                        offset1: -4,
                        offset2: 0,
                        immediate: None,
                        dereference: true,
                        inner_dereference: false,
                    },
                },
                Reference {
                    ap_tracking_data: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    pc: Some(0),
                    value_address: ValueAddress {
                        register: Some(Register::FP),
                        offset1: -3,
                        offset2: 0,
                        immediate: None,
                        dereference: true,
                        inner_dereference: false,
                    },
                },
                Reference {
                    ap_tracking_data: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    pc: Some(0),
                    value_address: ValueAddress {
                        register: Some(Register::FP),
                        offset1: -3,
                        offset2: 0,
                        immediate: Some(bigint!(2)),
                        dereference: false,
                        inner_dereference: false,
                    },
                },
                Reference {
                    ap_tracking_data: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    pc: Some(0),
                    value_address: ValueAddress {
                        register: Some(Register::FP),
                        offset1: 0,
                        offset2: 0,
                        immediate: None,
                        dereference: true,
                        inner_dereference: false,
                    },
                },
            ],
        };

        assert_eq!(program_json.prime, bigint!(10));
        assert_eq!(program_json.builtins, builtins);
        assert_eq!(program_json.data, data);
        assert_eq!(program_json.identifiers["__main__.main"].pc, Some(0));
        assert_eq!(program_json.hints, hints);
        assert_eq!(program_json.reference_manager, reference_manager);
    }

    #[test]
    fn deserialize_program_json_from_json_file_a() {
        // Open json file with (valid) even length encoded hex
        let file = File::open("cairo_programs/manually_compiled/valid_program_a.json").unwrap();
        let mut reader = BufReader::new(file);

        let program_json: ProgramJson = serde_json::from_reader(&mut reader).unwrap();
        let builtins: Vec<String> = Vec::new();

        assert_eq!(
            program_json.prime,
            BigInt::parse_bytes(
                b"3618502788666131213697322783095070105623107215331596699973092056135872020481",
                10
            )
            .unwrap()
        );
        assert_eq!(program_json.builtins, builtins);
        assert_eq!(program_json.data.len(), 6);
        assert_eq!(program_json.identifiers["__main__.main"].pc, Some(0));
    }

    #[test]
    fn deserialize_program_json_from_json_file_b() {
        // Open json file with (valid) odd length encoded hex
        let file = File::open("cairo_programs/manually_compiled/valid_program_b.json").unwrap();
        let mut reader = BufReader::new(file);

        let program_json: ProgramJson = serde_json::from_reader(&mut reader).unwrap();
        let builtins: Vec<String> = vec![String::from("output"), String::from("range_check")];

        assert_eq!(
            program_json.prime,
            BigInt::parse_bytes(
                b"3618502788666131213697322783095070105623107215331596699973092056135872020481",
                10
            )
            .unwrap()
        );
        assert_eq!(program_json.builtins, builtins);
        assert_eq!(program_json.data.len(), 24);
        assert_eq!(program_json.identifiers["__main__.main"].pc, Some(13));
    }

    #[test]
    fn deserialize_program_json_from_json_file_gives_error() {
        // Open json file with (invalid) even length encoded hex
        let even_length_file =
            File::open("cairo_programs/manually_compiled/invalid_even_length_hex.json").unwrap();
        let mut reader = BufReader::new(even_length_file);

        let even_result: Result<ProgramJson, _> = serde_json::from_reader(&mut reader);

        assert!(even_result.is_err());

        // Open json file with (invalid) odd length encoded hex
        let odd_length_file =
            File::open("cairo_programs/manually_compiled/invalid_odd_length_hex.json").unwrap();
        let mut reader = BufReader::new(odd_length_file);

        let odd_result: Result<ProgramJson, _> = serde_json::from_reader(&mut reader);

        assert!(odd_result.is_err());
    }

    #[test]
    fn deserialize_program_test() {
        let program: Program = deserialize_program(Path::new(
            "cairo_programs/manually_compiled/valid_program_a.json",
        ))
        .expect("Failed to deserialize program");

        let builtins: Vec<String> = Vec::new();
        let data: Vec<MaybeRelocatable> = vec![
            MaybeRelocatable::Int(BigInt::from_i64(5189976364521848832).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(1000).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(5189976364521848832).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(2000).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(5201798304953696256).unwrap()),
            MaybeRelocatable::Int(BigInt::from_i64(2345108766317314046).unwrap()),
        ];

        let mut hints: HashMap<usize, Vec<HintParams>> = HashMap::new();
        hints.insert(
            0,
            vec![HintParams {
                code: vec![
                    109, 101, 109, 111, 114, 121, 91, 97, 112, 93, 32, 61, 32, 115, 101, 103, 109,
                    101, 110, 116, 115, 46, 97, 100, 100, 40, 41,
                ],
                accessible_scopes: vec![
                    String::from("starkware.cairo.common.alloc"),
                    String::from("starkware.cairo.common.alloc.alloc"),
                ],
                flow_tracking_data: FlowTrackingData {
                    ap_tracking: ApTracking {
                        group: 0,
                        offset: 0,
                    },
                    reference_ids: HashMap::new(),
                },
            }],
        );
        hints.insert(
            46,
            vec![HintParams {
                code: vec![105, 109, 112, 111, 114, 116, 32, 109, 97, 116, 104],
                accessible_scopes: vec![String::from("__main__"), String::from("__main__.main")],
                flow_tracking_data: FlowTrackingData {
                    ap_tracking: ApTracking {
                        group: 5,
                        offset: 0,
                    },
                    reference_ids: HashMap::new(),
                },
            }],
        );

        assert_eq!(
            program.prime,
            BigInt::parse_bytes(
                b"3618502788666131213697322783095070105623107215331596699973092056135872020481",
                10
            )
            .unwrap()
        );
        assert_eq!(program.builtins, builtins);
        assert_eq!(program.data, data);
        assert_eq!(program.main, Some(0));
        assert_eq!(program.hints, hints);
    }
}
