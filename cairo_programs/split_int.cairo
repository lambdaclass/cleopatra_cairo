%builtins range_check

from starkware.cairo.common.math import split_int
from starkware.cairo.common.alloc import alloc

func main{range_check_ptr: felt}():
    alloc_locals
    let value = 456
    let n = 3
    let base = 10
    let bound = 1000
    let output: felt* = alloc()
    split_int(value, n, base, bound, output)
    assert output[0] = 6
    assert output[1] = 5
    assert output[2] = 4
    return()
end
