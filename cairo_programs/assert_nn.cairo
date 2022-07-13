%builtins range_check

from starkware.cairo.common.math import assert_nn

func assert_nn_manual_implementation{range_check_ptr}(a):
    %{
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.a)
        assert 0 <= ids.a % PRIME < range_check_builtin.bound, f'a = {ids.a} is out of range.'
    %}
    a = [range_check_ptr]
    let range_check_ptr = range_check_ptr + 1
    return ()
end

func main{range_check_ptr: felt}():
    let x = 64
    tempvar y = 64 * 64
    assert_nn(1)
    assert_nn(x)
    assert_nn(y)

    assert_nn_manual_implementation(1)
    assert_nn_manual_implementation(x)
    assert_nn_manual_implementation(y)

    return ()
end
