%builtins range_check

func assert_le_felt_hint{range_check_ptr}(a, b):
    alloc_locals
    local small_inputs
    %{
        from starkware.cairo.common.math_utils import assert_integer
        assert_integer(ids.a)
        assert_integer(ids.b)
        a = ids.a % PRIME
        b = ids.b % PRIME
        assert a <= b, f'a = {a} is not less than or equal to b = {b}.'

        ids.small_inputs = int(
            a < range_check_builtin.bound and (b - a) < range_check_builtin.bound)
    %}
    assert small_inputs = 1
    return()
end
	
func main{range_check_ptr: felt}():
    assert_le_felt_hint(1,2)
    return ()
end
