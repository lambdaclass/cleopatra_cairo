%builtins range_check

from starkware.cairo.common.bool import TRUE, FALSE
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math_cmp import is_le

#Returns TRUE is array_a is greater than array_b, which means every element in a is greater than every element in b
func compare_greater_array{range_check_ptr: felt}(array_a: felt*, array_b: felt*, array_length: felt, iterator: felt ) -> (r: felt):
    if iterator == array_length:
        return(TRUE)
    end
    let (comparison) = is_le(array_a[iterator], array_b[iterator])
    if comparison == TRUE:
        return(FALSE)
    end
    return compare_greater_array(array_a, array_b, array_length, iterator + 1)
end

func fill_array(array: felt*, base: felt, step: felt, array_length: felt, iterator: felt):
    if iterator == array_length:
        return()
    end
    assert array[iterator] = base + step * iterator
    return fill_array(array, base, step, array_length, iterator + 1)
end
	
func main{range_check_ptr: felt}():
    alloc_locals
    tempvar array_length = 10
    let (array_a : felt*) = alloc()
    let (array_b : felt*) = alloc()
    fill_array(array_a, 9, 5, array_length, 0)
    fill_array(array_b, 7, 3, array_length, 0)
    let result : felt = compare_greater_array(array_a, array_b, array_length, 0)
    assert result = TRUE
    return ()
end
