%builtins output pedersen range_check bitwise

from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.cairo.common.hash import hash2
from starkware.cairo.common.bitwise import bitwise_and, bitwise_or, bitwise_xor
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.serialize import serialize_word

func get_hash(hash_ptr: HashBuiltin*, num_a: felt, num_b: felt) -> (hash_ptr: HashBuiltin*, r):
    with hash_ptr:
        let (result) = hash2(num_a, num_b)
    end
    return (hash_ptr=hash_ptr, r=result)
end

func distort_value{range_check_ptr: felt, bitwise_ptr: BitwiseBuiltin*}(value: felt, secondary_value: felt, loop_num: felt) -> (r: felt):
    if loop_num == 0:
        return (value)
    end

    # Check that 0 <= secondary_value < 2**64.
    [range_check_ptr] = secondary_value
    assert [range_check_ptr + 1] = 2 ** 64 - 1 - secondary_value
    let range_check_ptr = range_check_ptr + 2

    let (distorted_value_a) = bitwise_xor(value, secondary_value)
    let (distorted_value_b) = bitwise_or(value, secondary_value/2)
    let (distorted_value) = bitwise_and(distorted_value_a, distorted_value_b)

    return distort_value(distorted_value, secondary_value*3, loop_num - 1)
end

func builtins_wrapper{output_ptr: felt*, range_check_ptr: felt, bitwise_ptr: BitwiseBuiltin*, pedersen_ptr: HashBuiltin*}(num_a: felt, num_b: felt):
    let (distorted_num_b) = distort_value(num_b, 6783043740, 20)
    let (pedersen_ptr, result: felt) = get_hash(pedersen_ptr, num_a, distorted_num_b)
    assert result = 1705936988874506830037172232662562674195194978736118624789869153703579404549
    serialize_word(result)

    return ()
end

func builtins_wrapper_iter{output_ptr: felt*, range_check_ptr: felt, bitwise_ptr: BitwiseBuiltin*, pedersen_ptr: HashBuiltin*}(num_a: felt, num_b: felt, n_iterations: felt):
    builtins_wrapper(num_a, num_b)
    if n_iterations != 0:
        builtins_wrapper_iter(num_a, num_b, n_iterations-1)
        tempvar output_ptr = output_ptr
        tempvar range_check_ptr = range_check_ptr
        tempvar bitwise_ptr = bitwise_ptr
        tempvar pedersen_ptr = pedersen_ptr 
    else:
        tempvar output_ptr = output_ptr
        tempvar range_check_ptr = range_check_ptr
        tempvar bitwise_ptr = bitwise_ptr
        tempvar pedersen_ptr = pedersen_ptr 
    end

    return ()
end

func main{output_ptr: felt*, pedersen_ptr: HashBuiltin*, range_check_ptr: felt, bitwise_ptr: BitwiseBuiltin*}():
    let num_a = 123568
    let num_b = 5673940
    builtins_wrapper_iter(num_a, num_b, 1000)

    return()
end
