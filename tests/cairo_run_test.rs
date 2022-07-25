use std::path::Path;

use cleopatra_cairo::cairo_run;

#[test]
fn cairo_run_test() {
    cairo_run::cairo_run(Path::new("cairo_programs/fibonacci.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_bitwise_output() {
    cairo_run::cairo_run(Path::new("cairo_programs/bitwise_output.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_bitwise_recursion() {
    cairo_run::cairo_run(Path::new("cairo_programs/bitwise_recursion.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_integration() {
    cairo_run::cairo_run(Path::new("cairo_programs/integration.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_integration_with_alloc_locals() {
    cairo_run::cairo_run(
        Path::new("cairo_programs/integration_with_alloc_locals.json"),
        false,
    )
    .expect("Couldn't run program");
}

#[test]
fn cairo_run_compare_arrays() {
    cairo_run::cairo_run(Path::new("cairo_programs/compare_arrays.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_compare_greater_array() {
    cairo_run::cairo_run(
        Path::new("cairo_programs/compare_greater_array.json"),
        false,
    )
    .expect("Couldn't run program");
}

#[test]
fn cairo_run_compare_lesser_array() {
    cairo_run::cairo_run(Path::new("cairo_programs/compare_lesser_array.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_assert_le_felt_hint() {
    cairo_run::cairo_run(Path::new("cairo_programs/assert_le_felt_hint.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_assert_250_bit_element_array() {
    cairo_run::cairo_run(
        Path::new("cairo_programs/assert_250_bit_element_array.json"),
        false,
    )
    .expect("Couldn't run program");
}

#[test]
fn cairo_abs_value() {
    cairo_run::cairo_run(Path::new("cairo_programs/abs_value_array.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_compare_different_arrays() {
    cairo_run::cairo_run(
        Path::new("cairo_programs/compare_different_arrays.json"),
        false,
    )
    .expect("Couldn't run program");
}

#[test]
fn cairo_run_assert_nn() {
    cairo_run::cairo_run(Path::new("cairo_programs/assert_nn.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_sqrt() {
    cairo_run::cairo_run(Path::new("cairo_programs/sqrt.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_assert_not_zero() {
    cairo_run::cairo_run(Path::new("cairo_programs/assert_not_zero.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_split_int() {
    cairo_run::cairo_run(Path::new("cairo_programs/split_int.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_split_int_big() {
    cairo_run::cairo_run(Path::new("cairo_programs/split_int_big.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_split_felt() {
    cairo_run::cairo_run(Path::new("cairo_programs/split_felt.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_is_le_felt() {
    cairo_run::cairo_run(Path::new("cairo_programs/math_cmp_is_le_felt.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_unsigned_div_rem() {
    cairo_run::cairo_run(Path::new("cairo_programs/unsigned_div_rem.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_signed_div_rem() {
    cairo_run::cairo_run(Path::new("cairo_programs/signed_div_rem.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_assert_lt_felt() {
    cairo_run::cairo_run(Path::new("cairo_programs/assert_lt_felt.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_memcpy() {
    cairo_run::cairo_run(Path::new("cairo_programs/memcpy_test.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_memset() {
    cairo_run::cairo_run(Path::new("cairo_programs/memset.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_pow() {
    cairo_run::cairo_run(Path::new("cairo_programs/pow.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_dict() {
    cairo_run::cairo_run(Path::new("cairo_programs/dict.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_dict_update() {
    cairo_run::cairo_run(Path::new("cairo_programs/dict_update.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_uint256() {
    cairo_run::cairo_run(Path::new("cairo_programs/uint256.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_find_element() {
    cairo_run::cairo_run(Path::new("cairo_programs/find_element.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_dict_write_bad() {
    assert!(cairo_run::cairo_run(
        Path::new("cairo_programs/bad_programs/bad_dict_new.json"),
        false
    )
    .is_err());
    let err = cairo_run::cairo_run(
        Path::new("cairo_programs/bad_programs/bad_dict_new.json"),
        false,
    )
    .err();
    assert_eq!(
        err.unwrap().to_string(),
        "VM failure: Dict Error: Tried to create a dict whithout an initial dict"
    );
}

#[test]
fn cairo_run_dict_update_bad() {
    assert!(cairo_run::cairo_run(
        Path::new("cairo_programs/bad_programs/bad_dict_update.json"),
        false
    )
    .is_err());
    let err = cairo_run::cairo_run(
        Path::new("cairo_programs/bad_programs/bad_dict_update.json"),
        false,
    )
    .err();
    assert_eq!(
        err.unwrap().to_string(),
        "VM failure: Dict Error: Got the wrong value for dict_update, expected value: 3, got: Some(5) for key: 2"
    );
}

#[test]
fn cairo_run_squash_dict() {
    cairo_run::cairo_run(Path::new("cairo_programs/squash_dict.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_dict_squash() {
    cairo_run::cairo_run(Path::new("cairo_programs/dict_squash.json"), false)
        .expect("Couldn't run program");
}

#[test]
fn cairo_run_set_add() {
    cairo_run::cairo_run(Path::new("cairo_programs/set_add.json"), false)
        .expect("Couldn't run program");
}
