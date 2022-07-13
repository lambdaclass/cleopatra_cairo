from starkware.cairo.common.math import assert_not_equal
from starkware.cairo.common.alloc import alloc


func compare_different_arrays(array_a: felt*, array_b: felt*, array_length: felt, iterator: felt):
    if iterator == array_length:
        return()
    end
    assert_not_equal(array_a[iterator], array_b[iterator])
    return compare_different_arrays(array_a, array_b, array_length, iterator + 1)
end

func fill_array(array: felt*, base: felt, step: felt, array_length: felt, iterator: felt):
    if iterator == array_length:
        return()
    end
    assert array[iterator] = base + step * iterator
    return fill_array(array, base, step, array_length, iterator + 1)
end
	
func main():
    alloc_locals
    tempvar array_length = 10
    let (array_a : felt*) = alloc()
    let (array_b : felt*) = alloc()
    fill_array(array_a, 3, 90, array_length, 0)
    fill_array(array_b, 7, 3, array_length, 0)
    compare_different_arrays(array_a, array_b, array_length, 0)
    return ()
end
