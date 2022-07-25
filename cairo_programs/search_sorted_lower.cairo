%builtins range_check
from starkware.cairo.common.find_element import search_sorted_lower
from starkware.cairo.common.alloc import alloc

struct MyStruct:
    member a : felt
    member b : felt
end

func main{range_check_ptr}() -> ():
    # Create an array with MyStruct elements (1,2), (3,4), (5,6).
    alloc_locals
    let (local array_ptr : MyStruct*) = alloc()
    assert array_ptr[0] = MyStruct(a=1, b=2)
    assert array_ptr[1] = MyStruct(a=3, b=4)
    assert array_ptr[2] = MyStruct(a=5, b=6)
    let (smallest_ptr : MyStruct*) = search_sorted_lower(
        array_ptr=array_ptr, elm_size=2, n_elms=3, key=2
    )
    assert smallest_ptr.a = 3
    assert smallest_ptr.b = 4
    return ()
end
