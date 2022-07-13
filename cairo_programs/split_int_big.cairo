%builtins range_check

from starkware.cairo.common.math import split_int
from starkware.cairo.common.alloc import alloc

func main{range_check_ptr: felt}():
    alloc_locals
    let value = 3618502788666131213697322783095070105623117215331596699973092056135872020481
    let n = 2
    let base = 2**64
    let bound = 2**64
    let output: felt* = alloc()
    split_int(value, n, base, bound, output)
    assert output[0] = 4003012203950112768
    assert output[1] = 542101086242752
    return()
end
