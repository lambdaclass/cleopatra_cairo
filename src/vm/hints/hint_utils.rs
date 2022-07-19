use crate::bigint;
use crate::math_utils::as_int;
use crate::math_utils::isqrt;
use crate::serde::deserialize_program::ApTracking;
use crate::types::relocatable::Relocatable;
use crate::types::{instruction::Register, relocatable::MaybeRelocatable};
use crate::vm::{
    context::run_context::RunContext, errors::vm_errors::VirtualMachineError,
    hints::execute_hint::HintReference, runners::builtin_runner::RangeCheckBuiltinRunner,
    vm_core::VirtualMachine,
};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Signed, ToPrimitive, Zero};
use std::collections::HashMap;
use std::ops::{Neg, Shl, Shr};

fn apply_ap_tracking_correction(
    ap: &Relocatable,
    ref_ap_tracking: &ApTracking,
    hint_ap_tracking: &ApTracking,
) -> Result<MaybeRelocatable, VirtualMachineError> {
    // check that both groups are the same
    if ref_ap_tracking.group != hint_ap_tracking.group {
        return Err(VirtualMachineError::InvalidTrackingGroup(
            ref_ap_tracking.group,
            hint_ap_tracking.group,
        ));
    }
    let ap_diff = hint_ap_tracking.offset - ref_ap_tracking.offset;

    Ok(MaybeRelocatable::from((
        ap.segment_index(),
        ap.offset() - ap_diff,
    )))
}

///Computes the memory address indicated by the HintReference
pub fn compute_addr_from_reference(
    hint_reference: &HintReference,
    run_context: &RunContext,
    vm: &VirtualMachine,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<Option<MaybeRelocatable>, VirtualMachineError> {
    let base_addr = match hint_reference.register {
        Register::FP => run_context.fp.clone(),
        Register::AP => {
            if hint_ap_tracking.is_none() || hint_reference.ap_tracking_data.is_none() {
                return Err(VirtualMachineError::NoneApTrackingData);
            }

            if let MaybeRelocatable::RelocatableValue(ref relocatable) = run_context.ap {
                apply_ap_tracking_correction(
                    relocatable,
                    // it is safe to call these unrwaps here, since it has been checked
                    // they are not None's
                    // this could be refactored to use pattern match but it will be
                    // unnecesarily verbose
                    hint_reference.ap_tracking_data.as_ref().unwrap(),
                    hint_ap_tracking.unwrap(),
                )?
            } else {
                return Err(VirtualMachineError::InvalidApValue(run_context.ap.clone()));
            }
        }
    };

    if let MaybeRelocatable::RelocatableValue(relocatable) = base_addr {
        if hint_reference.offset1.is_negative()
            && relocatable.offset() < hint_reference.offset1.abs() as usize
        {
            return Ok(None);
        }
        if !hint_reference.inner_dereference {
            return Ok(Some(MaybeRelocatable::from((
                relocatable.segment_index(),
                (relocatable.offset() as i32 + hint_reference.offset1 + hint_reference.offset2)
                    as usize,
            ))));
        } else {
            let addr = MaybeRelocatable::from((
                relocatable.segment_index(),
                (relocatable.offset() as i32 + hint_reference.offset1) as usize,
            ));

            match vm.memory.get(&addr) {
                Ok(Some(&MaybeRelocatable::RelocatableValue(ref dereferenced_addr))) => {
                    return Ok(Some(MaybeRelocatable::from((
                        dereferenced_addr.segment_index(),
                        (dereferenced_addr.offset() as i32 + hint_reference.offset2) as usize,
                    ))))
                }

                _none_or_error => return Ok(None),
            }
        }
    }

    Ok(None)
}

///Computes the memory address given by the reference id
pub fn get_address_from_reference(
    reference_id: &BigInt,
    references: &HashMap<usize, HintReference>,
    run_context: &RunContext,
    vm: &VirtualMachine,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<Option<MaybeRelocatable>, VirtualMachineError> {
    if let Some(index) = reference_id.to_usize() {
        if index < references.len() {
            if let Some(hint_reference) = references.get(&index) {
                return compute_addr_from_reference(
                    hint_reference,
                    run_context,
                    vm,
                    hint_ap_tracking,
                );
            }
        }
    }
    Ok(None)
}

///Implements hint: memory[ap] = segments.add()
pub fn add_segment(vm: &mut VirtualMachine) -> Result<(), VirtualMachineError> {
    let new_segment_base =
        MaybeRelocatable::RelocatableValue(vm.segments.add(&mut vm.memory, None));
    match vm.memory.insert(&vm.run_context.ap, &new_segment_base) {
        Ok(_) => Ok(()),
        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
    }
}

//Implements hint: memory[ap] = 0 if 0 <= (ids.a % PRIME) < range_check_builtin.bound else 1
pub fn is_nn(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let a_ref = if let Some(a_ref) = ids.get(&String::from("a")) {
        a_ref
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let a_addr = if let Ok(Some(a_addr)) =
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking)
    {
        a_addr
    } else {
        return Err(VirtualMachineError::FailedToGetReference(a_ref.clone()));
    };

    //Check that the ids are in memory
    match vm.memory.get(&a_addr) {
        Ok(Some(maybe_rel_a)) => {
            //Check that the value at the ids address is an Int
            let a = if let MaybeRelocatable::Int(ref a) = maybe_rel_a {
                a
            } else {
                return Err(VirtualMachineError::ExpectedInteger(a_addr.clone()));
            };
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check") {
                    let range_check_builtin = if let Some(range_check_builtin) =
                        builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>()
                    {
                        range_check_builtin
                    } else {
                        return Err(VirtualMachineError::NoRangeCheckBuiltin);
                    };
                    //Main logic (assert a is not negative and within the expected range)
                    let mut value = bigint!(1);
                    if a.mod_floor(&vm.prime) >= bigint!(0)
                        && a.mod_floor(&vm.prime) < range_check_builtin._bound
                    {
                        value = bigint!(0);
                    }
                    return match vm
                        .memory
                        .insert(&vm.run_context.ap, &MaybeRelocatable::from(value))
                    {
                        Ok(_) => Ok(()),
                        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
                    };
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        Ok(None) => Err(VirtualMachineError::MemoryGet(a_addr.clone())),
        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
    }
}

//Implements hint: memory[ap] = 0 if 0 <= ((-ids.a - 1) % PRIME) < range_check_builtin.bound else 1
pub fn is_nn_out_of_range(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let a_ref = if let Some(a_ref) = ids.get(&String::from("a")) {
        a_ref
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let a_addr = if let Ok(Some(a_addr)) =
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking)
    {
        a_addr
    } else {
        return Err(VirtualMachineError::FailedToGetReference(a_ref.clone()));
    };
    //Check that the ids are in memory
    match vm.memory.get(&a_addr) {
        Ok(Some(maybe_rel_a)) => {
            //Check that the value at the ids address is an Int
            let a = if let MaybeRelocatable::Int(ref a) = maybe_rel_a {
                a
            } else {
                return Err(VirtualMachineError::ExpectedInteger(a_addr.clone()));
            };
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check") {
                    let range_check_builtin = if let Some(range_check_builtin) =
                        builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>()
                    {
                        range_check_builtin
                    } else {
                        return Err(VirtualMachineError::NoRangeCheckBuiltin);
                    };
                    //Main logic (assert a is not negative and within the expected range)
                    let value = if (-a.clone() - 1usize).mod_floor(&vm.prime)
                        < range_check_builtin._bound
                    {
                        bigint!(0)
                    } else {
                        bigint!(1)
                    };
                    return match vm
                        .memory
                        .insert(&vm.run_context.ap, &MaybeRelocatable::from(value))
                    {
                        Ok(_) => Ok(()),
                        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
                    };
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        Ok(None) => Err(VirtualMachineError::MemoryGet(a_addr.clone())),
        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
    }
}
//Implements hint:from starkware.cairo.common.math_utils import assert_integer
//        assert_integer(ids.a)
//        assert_integer(ids.b)
//        a = ids.a % PRIME
//        b = ids.b % PRIME
//        assert a <= b, f'a = {a} is not less than or equal to b = {b}.'

//        ids.small_inputs = int(
//            a < range_check_builtin.bound and (b - a) < range_check_builtin.bound)
pub fn assert_le_felt(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (a_ref, b_ref, small_inputs_ref) =
        if let (Some(a_ref), Some(b_ref), Some(small_inputs_ref)) = (
            ids.get(&String::from("a")),
            ids.get(&String::from("b")),
            ids.get(&String::from("small_inputs")),
        ) {
            (a_ref, b_ref, small_inputs_ref)
        } else {
            return Err(VirtualMachineError::IncorrectIds(
                vec![
                    String::from("a"),
                    String::from("b"),
                    String::from("small_inputs"),
                ],
                ids.into_keys().collect(),
            ));
        };
    //Check that each reference id corresponds to a value in the reference manager
    let (a_addr, b_addr, small_inputs_addr) = if let (
        Ok(Some(a_addr)),
        Ok(Some(b_addr)),
        Ok(Some(small_inputs_addr)),
    ) = (
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(b_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(
            small_inputs_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (a_addr, b_addr, small_inputs_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    //Check that the ids are in memory (except for small_inputs which is local, and should contain None)
    //small_inputs needs to be None, as we cant change it value otherwise
    match (
        vm.memory.get(&a_addr),
        vm.memory.get(&b_addr),
        vm.memory.get(&small_inputs_addr),
    ) {
        (Ok(Some(maybe_rel_a)), Ok(Some(maybe_rel_b)), Ok(None)) => {
            //Check that the values at the ids address are Int
            let a = if let &MaybeRelocatable::Int(ref a) = maybe_rel_a {
                a
            } else {
                return Err(VirtualMachineError::ExpectedInteger(a_addr.clone()));
            };
            let b = if let MaybeRelocatable::Int(ref b) = maybe_rel_b {
                b
            } else {
                return Err(VirtualMachineError::ExpectedInteger(b_addr.clone()));
            };
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check") {
                    match builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>() {
                        None => return Err(VirtualMachineError::NoRangeCheckBuiltin),
                        Some(builtin) => {
                            //Assert a <= b
                            if a.mod_floor(&vm.prime) > b.mod_floor(&vm.prime) {
                                return Err(VirtualMachineError::NonLeFelt(a.clone(), b.clone()));
                            }
                            //Calculate value of small_inputs
                            let value = if *a < builtin._bound && (a - b) < builtin._bound {
                                bigint!(1)
                            } else {
                                bigint!(0)
                            };
                            match vm
                                .memory
                                .insert(&small_inputs_addr, &MaybeRelocatable::from(value))
                            {
                                Ok(_) => return Ok(()),
                                Err(memory_error) => {
                                    return Err(VirtualMachineError::MemoryError(memory_error))
                                }
                            }
                        }
                    }
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint:from starkware.cairo.common.math_cmp import is_le_felt
//    memory[ap] = 0 if (ids.a % PRIME) <= (ids.b % PRIME) else 1
pub fn is_le_felt(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (a_ref, b_ref) = if let (Some(a_ref), Some(b_ref)) =
        (ids.get(&String::from("a")), ids.get(&String::from("b")))
    {
        (a_ref, b_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a"), String::from("b")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (a_addr, b_addr) = if let (Ok(Some(a_addr)), Ok(Some(b_addr))) = (
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(b_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
    ) {
        (a_addr, b_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    match (vm.memory.get(&a_addr), vm.memory.get(&b_addr)) {
        (Ok(Some(maybe_rel_a)), Ok(Some(maybe_rel_b))) => {
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check")
                    && builtin
                        .as_any()
                        .downcast_ref::<RangeCheckBuiltinRunner>()
                        .is_some()
                {
                    let mut value = bigint!(0);
                    let a_mod = match maybe_rel_a.mod_floor(&vm.prime) {
                        Ok(MaybeRelocatable::Int(n)) => n,
                        Ok(MaybeRelocatable::RelocatableValue(_)) => {
                            return Err(VirtualMachineError::ExpectedInteger(a_addr.clone()))
                        }
                        Err(e) => return Err(e),
                    };
                    let b_mod = match maybe_rel_b.mod_floor(&vm.prime) {
                        Ok(MaybeRelocatable::Int(n)) => n,
                        Ok(MaybeRelocatable::RelocatableValue(_)) => {
                            return Err(VirtualMachineError::ExpectedInteger(b_addr.clone()))
                        }
                        Err(e) => return Err(e),
                    };
                    if a_mod > b_mod {
                        value = bigint!(1);
                    }

                    return vm
                        .memory
                        .insert(&vm.run_context.ap, &MaybeRelocatable::from(value))
                        .map_err(VirtualMachineError::MemoryError);
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint: from starkware.cairo.lang.vm.relocatable import RelocatableValue
//        both_ints = isinstance(ids.a, int) and isinstance(ids.b, int)
//        both_relocatable = (
//            isinstance(ids.a, RelocatableValue) and isinstance(ids.b, RelocatableValue) and
//            ids.a.segment_index == ids.b.segment_index)
//        assert both_ints or both_relocatable, \
//            f'assert_not_equal failed: non-comparable values: {ids.a}, {ids.b}.'
//        assert (ids.a - ids.b) % PRIME != 0, f'assert_not_equal failed: {ids.a} = {ids.b}.'
pub fn assert_not_equal(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (a_ref, b_ref) = if let (Some(a_ref), Some(b_ref)) =
        (ids.get(&String::from("a")), ids.get(&String::from("b")))
    {
        (a_ref, b_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a"), String::from("b")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (a_addr, b_addr) = if let (Ok(Some(a_addr)), Ok(Some(b_addr))) = (
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(b_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
    ) {
        (a_addr, b_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    //Check that the ids are in memory
    match (vm.memory.get(&a_addr), vm.memory.get(&b_addr)) {
        (Ok(Some(maybe_rel_a)), Ok(Some(maybe_rel_b))) => match (maybe_rel_a, maybe_rel_b) {
            (MaybeRelocatable::Int(ref a), MaybeRelocatable::Int(ref b)) => {
                if (a - b).is_multiple_of(&vm.prime) {
                    return Err(VirtualMachineError::AssertNotEqualFail(
                        maybe_rel_a.clone(),
                        maybe_rel_b.clone(),
                    ));
                };
                Ok(())
            }
            (MaybeRelocatable::RelocatableValue(a), MaybeRelocatable::RelocatableValue(b)) => {
                if a.segment_index() != b.segment_index() {
                    return Err(VirtualMachineError::DiffIndexComp(a.clone(), b.clone()));
                };
                if a.offset() == b.offset() {
                    return Err(VirtualMachineError::AssertNotEqualFail(
                        maybe_rel_a.clone(),
                        maybe_rel_b.clone(),
                    ));
                };
                Ok(())
            }
            _ => Err(VirtualMachineError::DiffTypeComparison(
                maybe_rel_a.clone(),
                maybe_rel_b.clone(),
            )),
        },
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint:
// %{
//     from starkware.cairo.common.math_utils import assert_integer
//     assert_integer(ids.a)
//     assert 0 <= ids.a % PRIME < range_check_builtin.bound, f'a = {ids.a} is out of range.'
// %}
pub fn assert_nn(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for 'a' variable used by the hint
    let a_ref = if let Some(a_ref) = ids.get(&String::from("a")) {
        a_ref
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a")],
            ids.into_keys().collect(),
        ));
    };
    //Check that 'a' reference id corresponds to a value in the reference manager
    let a_addr = if let Ok(Some(a_addr)) =
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking)
    {
        a_addr
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };

    //Check that the 'a' id is in memory
    let maybe_rel_a = if let Ok(Some(maybe_rel_a)) = vm.memory.get(&a_addr) {
        maybe_rel_a
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };

    //assert_integer(ids.a)
    let a = if let &MaybeRelocatable::Int(ref a) = maybe_rel_a {
        a
    } else {
        return Err(VirtualMachineError::ExpectedInteger(a_addr.clone()));
    };

    for (name, builtin) in &vm.builtin_runners {
        //Check that range_check_builtin is present
        if name == &String::from("range_check") {
            let range_check_builtin = if let Some(range_check_builtin) =
                builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>()
            {
                range_check_builtin
            } else {
                return Err(VirtualMachineError::NoRangeCheckBuiltin);
            };
            // assert 0 <= ids.a % PRIME < range_check_builtin.bound
            // as prime > 0, a % prime will always be > 0
            if a.mod_floor(&vm.prime) < range_check_builtin._bound {
                return Ok(());
            } else {
                return Err(VirtualMachineError::ValueOutOfRange(a.clone()));
            }
        }
    }
    Err(VirtualMachineError::NoRangeCheckBuiltin)
}

//Implements hint:from starkware.cairo.common.math.cairo
// %{
// from starkware.cairo.common.math_utils import assert_integer
// assert_integer(ids.value)
// assert ids.value % PRIME != 0, f'assert_not_zero failed: {ids.value} = 0.'
// %}
pub fn assert_not_zero(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    let value_ref = if let Some(value_ref) = ids.get(&String::from("value")) {
        value_ref
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("value")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let value_addr = if let Ok(Some(value_addr)) = get_address_from_reference(
        value_ref,
        &vm.references,
        &vm.run_context,
        vm,
        hint_ap_tracking,
    ) {
        value_addr
    } else {
        return Err(VirtualMachineError::FailedToGetReference(value_ref.clone()));
    };
    match vm.memory.get(&value_addr) {
        Ok(Some(maybe_rel_value)) => {
            //Check that the value at the ids address is an Int
            if let &MaybeRelocatable::Int(ref value) = maybe_rel_value {
                if value.is_multiple_of(&vm.prime) {
                    Err(VirtualMachineError::AssertNotZero(
                        value.clone(),
                        vm.prime.clone(),
                    ))
                } else {
                    Ok(())
                }
            } else {
                Err(VirtualMachineError::ExpectedInteger(value_addr.clone()))
            }
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint: assert ids.value == 0, 'split_int(): value is out of range.'
pub fn split_int_assert_range(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let value_ref = if let Some(value_ref) = ids.get(&String::from("value")) {
        value_ref
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("value")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let value_addr = if let Ok(Some(value_addr)) = get_address_from_reference(
        value_ref,
        &vm.references,
        &vm.run_context,
        vm,
        hint_ap_tracking,
    ) {
        value_addr
    } else {
        return Err(VirtualMachineError::FailedToGetReference(value_ref.clone()));
    };
    //Check that the ids are in memory
    match vm.memory.get(&value_addr) {
        Ok(Some(maybe_rel_value)) => {
            //Check that the value at the ids address is an Int
            let value = if let MaybeRelocatable::Int(ref value) = maybe_rel_value {
                value
            } else {
                return Err(VirtualMachineError::ExpectedInteger(value_addr.clone()));
            };
            //Main logic (assert value == 0)
            if !value.is_zero() {
                return Err(VirtualMachineError::SplitIntNotZero);
            }
            Ok(())
        }
        Ok(None) => Err(VirtualMachineError::MemoryGet(value_addr.clone())),
        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
    }
}

//Implements hint: memory[ids.output] = res = (int(ids.value) % PRIME) % ids.base
//        assert res < ids.bound, f'split_int(): Limb {res} is out of range.'
pub fn split_int(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (output_ref, value_ref, base_ref, bound_ref) =
        if let (Some(output_ref), Some(value_ref), Some(base_ref), Some(bound_ref)) = (
            ids.get(&String::from("output")),
            ids.get(&String::from("value")),
            ids.get(&String::from("base")),
            ids.get(&String::from("bound")),
        ) {
            (output_ref, value_ref, base_ref, bound_ref)
        } else {
            return Err(VirtualMachineError::IncorrectIds(
                vec![
                    String::from("output"),
                    String::from("value"),
                    String::from("base"),
                    String::from("bound"),
                ],
                ids.into_keys().collect(),
            ));
        };
    //Check that the ids are in memory (except for small_inputs which is local, and should contain None)
    //small_inputs needs to be None, as we cant change it value otherwise
    let (output_addr, value_addr, base_addr, bound_addr) = if let (
        Ok(Some(output_addr)),
        Ok(Some(value_addr)),
        Ok(Some(base_addr)),
        Ok(Some(bound_addr)),
    ) = (
        get_address_from_reference(
            output_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            value_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            base_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            bound_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (output_addr, value_addr, base_addr, bound_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    //Check that the ids are in memory
    let (mr_output, mr_value, mr_base, mr_bound) =
        if let (Ok(Some(mr_output)), Ok(Some(mr_value)), Ok(Some(mr_base)), Ok(Some(mr_bound))) = (
            vm.memory.get(&output_addr),
            vm.memory.get(&value_addr),
            vm.memory.get(&base_addr),
            vm.memory.get(&bound_addr),
        ) {
            (mr_output, mr_value, mr_base, mr_bound)
        } else {
            return Err(VirtualMachineError::FailedToGetIds);
        };
    //Check that the type of the ids
    let (output, value, base, bound) = if let (
        MaybeRelocatable::RelocatableValue(output),
        MaybeRelocatable::Int(value),
        MaybeRelocatable::Int(base),
        MaybeRelocatable::Int(bound),
    ) = (mr_output, mr_value, mr_base, mr_bound)
    {
        (output, value, base, bound)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    //Main Logic
    let res = (value.mod_floor(&vm.prime)).mod_floor(base);
    if res > *bound {
        return Err(VirtualMachineError::SplitIntLimbOutOfRange(res));
    }
    let output_base = MaybeRelocatable::RelocatableValue(output.to_owned());
    vm.memory
        .insert(&output_base, &MaybeRelocatable::Int(res))
        .map_err(VirtualMachineError::MemoryError)
}

//from starkware.cairo.common.math_utils import is_positive
//ids.is_positive = 1 if is_positive(
//    value=ids.value, prime=PRIME, rc_bound=range_check_builtin.bound) else 0
pub fn is_positive(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (value_ref, is_positive_ref) = if let (Some(value_ref), Some(is_positive_ref)) = (
        ids.get(&String::from("value")),
        ids.get(&String::from("is_positive")),
    ) {
        (value_ref, is_positive_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("value"), String::from("is_positive")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (value_addr, is_positive_addr) = if let (Ok(Some(value_addr)), Ok(Some(is_positive_addr))) = (
        get_address_from_reference(
            value_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            is_positive_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (value_addr, is_positive_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };

    //Check that the ids are in memory
    match (vm.memory.get(&value_addr), vm.memory.get(&is_positive_addr)) {
        (Ok(Some(maybe_rel_value)), Ok(_)) => {
            //Check that the value at the ids address is an Int
            let value = if let MaybeRelocatable::Int(ref value) = maybe_rel_value {
                value
            } else {
                return Err(VirtualMachineError::ExpectedInteger(value_addr.clone()));
            };
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check") {
                    let range_check_builtin = if let Some(range_check_builtin) =
                        builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>()
                    {
                        range_check_builtin
                    } else {
                        return Err(VirtualMachineError::NoRangeCheckBuiltin);
                    };
                    //Main logic (assert a is positive)
                    let int_value = as_int(value, &vm.prime);
                    if int_value.abs() > range_check_builtin._bound {
                        return Err(VirtualMachineError::ValueOutsideValidRange(int_value));
                    }
                    let result = if int_value.is_positive() {
                        bigint!(1)
                    } else {
                        bigint!(0)
                    };
                    return vm
                        .memory
                        .insert(&is_positive_addr, &MaybeRelocatable::from(result))
                        .map_err(VirtualMachineError::MemoryError);
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        (Err(memory_error), _) | (_, Err(memory_error)) => {
            Err(VirtualMachineError::MemoryError(memory_error))
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint:
// %{
//     from starkware.cairo.common.math_utils import assert_integer
//     assert ids.MAX_HIGH < 2**128 and ids.MAX_LOW < 2**128
//     assert PRIME - 1 == ids.MAX_HIGH * 2**128 + ids.MAX_LOW
//     assert_integer(ids.value)
//     ids.low = ids.value & ((1 << 128) - 1)
//     ids.high = ids.value >> 128
// %}
pub fn split_felt(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for the variables used by the hint
    let (high_ref, low_ref, value_ref) = if let (Some(high_ref), Some(low_ref), Some(value_ref)) = (
        ids.get(&String::from("high")),
        ids.get(&String::from("low")),
        ids.get(&String::from("value")),
    ) {
        (high_ref, low_ref, value_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![
                String::from("high"),
                String::from("low"),
                String::from("value"),
            ],
            ids.into_keys().collect(),
        ));
    };

    // Get the addresses of the variables used in the hints
    let (high_addr, low_addr, value_addr) =
        if let (Ok(Some(high_addr)), Ok(Some(low_addr)), Ok(Some(value_addr))) = (
            get_address_from_reference(
                high_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
            get_address_from_reference(
                low_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
            get_address_from_reference(
                value_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
        ) {
            (high_addr, low_addr, value_addr)
        } else {
            return Err(VirtualMachineError::FailedToGetIds);
        };

    //Check that the 'value' variable is in memory
    match vm.memory.get(&value_addr) {
        Ok(Some(MaybeRelocatable::Int(ref value))) => {
            //Main logic
            //assert_integer(ids.value) (done by match)
            // ids.low = ids.value & ((1 << 128) - 1)
            // ids.high = ids.value >> 128
            let low: BigInt = value.clone() & ((bigint!(1).shl(128_u8)) - bigint!(1));
            let high: BigInt = value.shr(128_u8);
            match (
                vm.memory.insert(&low_addr, &MaybeRelocatable::from(low)),
                vm.memory.insert(&high_addr, &MaybeRelocatable::from(high)),
            ) {
                (Ok(_), Ok(_)) => Ok(()),
                (Err(error), _) | (_, Err(error)) => Err(VirtualMachineError::MemoryError(error)),
            }
        }
        Ok(Some(MaybeRelocatable::RelocatableValue(ref _value))) => {
            Err(VirtualMachineError::ExpectedInteger(value_addr.clone()))
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint: from starkware.python.math_utils import isqrt
//        value = ids.value % PRIME
//        assert value < 2 ** 250, f"value={value} is outside of the range [0, 2**250)."
//        assert 2 ** 250 < PRIME
//        ids.root = isqrt(value)
pub fn sqrt(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (value_ref, root_ref) = if let (Some(value_ref), Some(root_ref)) = (
        ids.get(&String::from("value")),
        ids.get(&String::from("root")),
    ) {
        (value_ref, root_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("value"), String::from("root")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (value_addr, root_addr) = if let (Ok(Some(value_addr)), Ok(Some(root_addr))) = (
        get_address_from_reference(
            value_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            root_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (value_addr, root_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    //Check that the ids are in memory
    match (vm.memory.get(&value_addr), vm.memory.get(&root_addr)) {
        (Ok(Some(maybe_rel_value)), Ok(_)) => {
            let value = if let MaybeRelocatable::Int(value) = maybe_rel_value {
                value
            } else {
                return Err(VirtualMachineError::ExpectedInteger(
                    maybe_rel_value.clone(),
                ));
            };
            let mod_value = value.mod_floor(&vm.prime);
            //This is equal to mod_value > bigint!(2).pow(250)
            if (&mod_value).shr(250_i32).is_positive() {
                return Err(VirtualMachineError::ValueOutside250BitRange(mod_value));
            }
            vm.memory
                .insert(&root_addr, &MaybeRelocatable::from(isqrt(&mod_value)?))
                .map_err(VirtualMachineError::MemoryError)
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

pub fn signed_div_rem(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (r_ref, biased_q_ref, range_check_ptr_ref, div_ref, value_ref, bound_ref) = if let (
        Some(r_ref),
        Some(biased_q_ref),
        Some(range_check_ptr_ref),
        Some(div_ref),
        Some(value_ref),
        Some(bound_ref),
    ) = (
        ids.get(&String::from("r")),
        ids.get(&String::from("biased_q")),
        ids.get(&String::from("range_check_ptr")),
        ids.get(&String::from("div")),
        ids.get(&String::from("value")),
        ids.get(&String::from("bound")),
    ) {
        (
            r_ref,
            biased_q_ref,
            range_check_ptr_ref,
            div_ref,
            value_ref,
            bound_ref,
        )
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![
                String::from("r"),
                String::from("biased_q"),
                String::from("range_check_ptr"),
                String::from("div"),
                String::from("value"),
                String::from("bound"),
            ],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (r_addr, biased_q_addr, range_check_ptr_addr, div_addr, value_addr, bound_addr) = if let (
        Ok(Some(r_addr)),
        Ok(Some(biased_q_addr)),
        Ok(Some(range_check_ptr_addr)),
        Ok(Some(div_addr)),
        Ok(Some(value_addr)),
        Ok(Some(bound_addr)),
    ) = (
        get_address_from_reference(r_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(
            biased_q_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            range_check_ptr_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            div_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            value_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            bound_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (
            r_addr,
            biased_q_addr,
            range_check_ptr_addr,
            div_addr,
            value_addr,
            bound_addr,
        )
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    match (
        vm.memory.get(&r_addr),
        vm.memory.get(&biased_q_addr),
        vm.memory.get(&range_check_ptr_addr),
        vm.memory.get(&div_addr),
        vm.memory.get(&value_addr),
        vm.memory.get(&bound_addr),
    ) {
        (
            Ok(_),
            Ok(_),
            Ok(_),
            Ok(Some(maybe_rel_div)),
            Ok(Some(maybe_rel_value)),
            Ok(Some(maybe_rel_bound)),
        ) => {
            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                if name == &String::from("range_check") {
                    match builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>() {
                        Some(builtin) => {
                            // Main logic
                            let div = if let MaybeRelocatable::Int(ref div) = maybe_rel_div {
                                div
                            } else {
                                return Err(VirtualMachineError::ExpectedInteger(div_addr.clone()));
                            };

                            if !div.is_positive() || div > &(&vm.prime / &builtin._bound) {
                                return Err(VirtualMachineError::OutOfValidRange(
                                    div.clone(),
                                    &vm.prime / &builtin._bound,
                                ));
                            }

                            let bound = if let MaybeRelocatable::Int(ref bound) = maybe_rel_bound {
                                bound
                            } else {
                                return Err(VirtualMachineError::ExpectedInteger(
                                    bound_addr.clone(),
                                ));
                            };

                            // Divide by 2
                            if bound > &(&builtin._bound).shr(1_i32) {
                                return Err(VirtualMachineError::OutOfValidRange(
                                    bound.clone(),
                                    (&builtin._bound).shr(1_i32),
                                ));
                            }

                            let value = if let MaybeRelocatable::Int(ref value) = maybe_rel_value {
                                value
                            } else {
                                return Err(VirtualMachineError::ExpectedInteger(
                                    value_addr.clone(),
                                ));
                            };

                            let int_value = &as_int(value, &vm.prime);

                            let (q, r) = int_value.div_mod_floor(div);

                            if bound.neg() > q || &q >= bound {
                                return Err(VirtualMachineError::OutOfValidRange(q, bound.clone()));
                            }

                            let biased_q = MaybeRelocatable::Int(q + bound);

                            return match (
                                vm.memory
                                    .insert(&r_addr, &MaybeRelocatable::Int(r))
                                    .map_err(VirtualMachineError::MemoryError),
                                vm.memory
                                    .insert(&biased_q_addr, &biased_q)
                                    .map_err(VirtualMachineError::MemoryError),
                            ) {
                                (Ok(_), Ok(_)) => Ok(()),
                                (Err(e), _) | (_, Err(e)) => Err(e),
                            };
                        }
                        None => {
                            return Err(VirtualMachineError::NoRangeCheckBuiltin);
                        }
                    }
                };
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

/*
Implements hint:

from starkware.cairo.common.math_utils import assert_integer
assert_integer(ids.div)
assert 0 < ids.div <= PRIME // range_check_builtin.bound, \
    f'div={hex(ids.div)} is out of the valid range.'
ids.q, ids.r = divmod(ids.value, ids.div)
*/
pub fn unsigned_div_rem(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (r_ref, q_ref, div_ref, value_ref) =
        if let (Some(r_ref), Some(q_ref), Some(div_ref), Some(value_ref)) = (
            ids.get(&String::from("r")),
            ids.get(&String::from("q")),
            ids.get(&String::from("div")),
            ids.get(&String::from("value")),
        ) {
            (r_ref, q_ref, div_ref, value_ref)
        } else {
            return Err(VirtualMachineError::IncorrectIds(
                vec![
                    String::from("r"),
                    String::from("q"),
                    String::from("div"),
                    String::from("value"),
                ],
                ids.into_keys().collect(),
            ));
        };
    //Check that each reference id corresponds to a value in the reference manager
    let (r_addr, q_addr, div_addr, value_addr) = if let (
        Ok(Some(r_addr)),
        Ok(Some(q_addr)),
        Ok(Some(div_addr)),
        Ok(Some(value_addr)),
    ) = (
        get_address_from_reference(r_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(q_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(
            div_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
        get_address_from_reference(
            value_ref,
            &vm.references,
            &vm.run_context,
            vm,
            hint_ap_tracking,
        ),
    ) {
        (r_addr, q_addr, div_addr, value_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };
    match (
        vm.memory.get(&r_addr),
        vm.memory.get(&q_addr),
        vm.memory.get(&div_addr),
        vm.memory.get(&value_addr),
    ) {
        (Ok(_), Ok(_), Ok(Some(maybe_rel_div)), Ok(Some(maybe_rel_value))) => {
            let div = if let MaybeRelocatable::Int(ref div) = maybe_rel_div {
                div
            } else {
                return Err(VirtualMachineError::ExpectedInteger(div_addr.clone()));
            };
            let value = maybe_rel_value;

            for (name, builtin) in &vm.builtin_runners {
                //Check that range_check_builtin is present
                let builtin = match builtin.as_any().downcast_ref::<RangeCheckBuiltinRunner>() {
                    Some(b) => b,
                    None => return Err(VirtualMachineError::NoRangeCheckBuiltin),
                };

                if name == &String::from("range_check") {
                    // Main logic
                    if !div.is_positive() || div > &(&vm.prime / &builtin._bound) {
                        return Err(VirtualMachineError::OutOfValidRange(
                            div.clone(),
                            &vm.prime / &builtin._bound,
                        ));
                    }

                    let (q, r) = match value.divmod(&MaybeRelocatable::from(div.clone())) {
                        Ok((q, r)) => (q, r),
                        Err(e) => return Err(e),
                    };

                    return match (
                        vm.memory
                            .insert(&r_addr, &r)
                            .map_err(VirtualMachineError::MemoryError),
                        vm.memory
                            .insert(&q_addr, &q)
                            .map_err(VirtualMachineError::MemoryError),
                    ) {
                        (Ok(_), Ok(_)) => Ok(()),
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    };
                }
            }
            Err(VirtualMachineError::NoRangeCheckBuiltin)
        }
        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}

//Implements hint: from starkware.cairo.common.math_utils import as_int
//        # Correctness check.
//        value = as_int(ids.value, PRIME) % PRIME
//        assert value < ids.UPPER_BOUND, f'{value} is outside of the range [0, 2**250).'
//        # Calculation for the assertion.
//        ids.high, ids.low = divmod(ids.value, ids.SHIFT)
pub fn assert_250_bit(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Declare constant values
    let upper_bound = bigint!(1).shl(250_i32);
    let shift = bigint!(1).shl(128_i32);
    //Check that ids contains the reference id for each variable used by the hint
    let (value_ref, high_ref, low_ref) = if let (Some(value_ref), Some(high_ref), Some(low_ref)) = (
        ids.get(&String::from("value")),
        ids.get(&String::from("high")),
        ids.get(&String::from("low")),
    ) {
        (value_ref, high_ref, low_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![
                String::from("value"),
                String::from("high"),
                String::from("low"),
            ],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (value_addr, high_addr, low_addr) =
        if let (Ok(Some(value_addr)), Ok(Some(high_addr)), Ok(Some(low_addr))) = (
            get_address_from_reference(
                value_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
            get_address_from_reference(
                high_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
            get_address_from_reference(
                low_ref,
                &vm.references,
                &vm.run_context,
                vm,
                hint_ap_tracking,
            ),
        ) {
            (value_addr, high_addr, low_addr)
        } else {
            return Err(VirtualMachineError::FailedToGetIds);
        };
    //Check that the ids.value is in memory
    match vm.memory.get(&value_addr) {
        Ok(Some(maybe_rel_value)) => {
            //Check that ids.value is an Int value
            let value = if let &MaybeRelocatable::Int(ref value) = maybe_rel_value {
                value
            } else {
                return Err(VirtualMachineError::ExpectedInteger(value_addr.clone()));
            };
            //Main logic
            let int_value = as_int(value, &vm.prime).mod_floor(&vm.prime);
            if int_value > upper_bound {
                return Err(VirtualMachineError::ValueOutside250BitRange(int_value));
            }

            //Insert values into ids.high and ids.low
            let (high, low) = int_value.div_rem(&shift);
            vm.memory
                .insert(&high_addr, &MaybeRelocatable::from(high))
                .map_err(VirtualMachineError::MemoryError)?;
            vm.memory
                .insert(&low_addr, &MaybeRelocatable::from(low))
                .map_err(VirtualMachineError::MemoryError)?;
            Ok(())
        }
        Ok(None) => Err(VirtualMachineError::MemoryGet(value_addr)),
        Err(memory_error) => Err(VirtualMachineError::MemoryError(memory_error)),
    }
}

/*
Implements hint:
%{
    from starkware.cairo.common.math_utils import assert_integer
    assert_integer(ids.a)
    assert_integer(ids.b)
    assert (ids.a % PRIME) < (ids.b % PRIME), \
        f'a = {ids.a % PRIME} is not less than b = {ids.b % PRIME}.'
%}
*/
pub fn assert_lt_felt(
    vm: &mut VirtualMachine,
    ids: HashMap<String, BigInt>,
    hint_ap_tracking: Option<&ApTracking>,
) -> Result<(), VirtualMachineError> {
    //Check that ids contains the reference id for each variable used by the hint
    let (a_ref, b_ref) = if let (Some(a_ref), Some(b_ref)) =
        (ids.get(&String::from("a")), ids.get(&String::from("b")))
    {
        (a_ref, b_ref)
    } else {
        return Err(VirtualMachineError::IncorrectIds(
            vec![String::from("a"), String::from("b")],
            ids.into_keys().collect(),
        ));
    };
    //Check that each reference id corresponds to a value in the reference manager
    let (a_addr, b_addr) = if let (Ok(Some(a_addr)), Ok(Some(b_addr))) = (
        get_address_from_reference(a_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
        get_address_from_reference(b_ref, &vm.references, &vm.run_context, vm, hint_ap_tracking),
    ) {
        (a_addr, b_addr)
    } else {
        return Err(VirtualMachineError::FailedToGetIds);
    };

    match (vm.memory.get(&a_addr), vm.memory.get(&b_addr)) {
        (Ok(Some(MaybeRelocatable::Int(ref a))), Ok(Some(MaybeRelocatable::Int(ref b)))) => {
            // main logic
            // assert_integer(ids.a)
            // assert_integer(ids.b)
            // assert (ids.a % PRIME) < (ids.b % PRIME), \
            //     f'a = {ids.a % PRIME} is not less than b = {ids.b % PRIME}.'
            if a.mod_floor(&vm.prime) < b.mod_floor(&vm.prime) {
                Ok(())
            } else {
                Err(VirtualMachineError::AssertLtFelt(a.clone(), b.clone()))
            }
        }
        (Ok(Some(MaybeRelocatable::RelocatableValue(_))), _) => {
            Err(VirtualMachineError::ExpectedInteger(a_addr.clone()))
        }
        (_, Ok(Some(MaybeRelocatable::RelocatableValue(_)))) => {
            Err(VirtualMachineError::ExpectedInteger(b_addr.clone()))
        }

        _ => Err(VirtualMachineError::FailedToGetIds),
    }
}
