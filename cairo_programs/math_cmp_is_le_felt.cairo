%builtins range_check
from starkware.cairo.common.math_cmp import is_le_felt

func main{range_check_ptr: felt}():
    let (a) = is_le_felt(2,3)
    assert a = 1
    return ()
end
