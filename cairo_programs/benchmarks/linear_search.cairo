from starkware.cairo.common.alloc import alloc

func search(num_to_find: felt, arr: felt*, current_index: felt, arr_size: felt) -> (index:felt):
        if arr_size == current_index:
                return(-1)
        end

        if arr[current_index] == num_to_find:
                return(current_index)
        else:
                let (index) = search(num_to_find, arr, current_index + 1, arr_size)
                return(index)
        end
end

func fill_array(array: felt*, array_length: felt, iterator: felt):
    if iterator == array_length:
        return()
    end
    assert array[iterator] = iterator
    return fill_array(array, array_length, iterator + 1)
end

func main():
        alloc_locals
        let (felt_array : felt*) = alloc()

        fill_array(felt_array, 10001, 0)

        let (index_1) = search(10000, felt_array, 0, 10001)
        assert index_1 = 10000
        let (index_2) = search(5000, felt_array, 0, 10001)
        assert index_2 = 5000
        ret
end
