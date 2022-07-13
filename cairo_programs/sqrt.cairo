%builtins range_check

from starkware.cairo.common.math import sqrt

func main{range_check_ptr: felt}():
    let (result_a) = sqrt(0)
    assert result_a = 0

    let (result_b) = sqrt(2402)
    assert result_b = 49

    let (result_c) = sqrt(361850278866613121369732278309507010562)
    assert result_c = 19022362599493605525

    return()
end
