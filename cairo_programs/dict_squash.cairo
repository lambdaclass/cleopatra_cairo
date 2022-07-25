%builtins range_check

from starkware.cairo.common.dict_access import DictAccess
from starkware.cairo.common.dict import (
    dict_write,
    dict_update,
    dict_squash,
)
from starkware.cairo.common.default_dict import default_dict_new

func main{range_check_ptr}() -> ():
    let (dict_start) = default_dict_new(17)
    let dict_end = dict_start
    dict_write{dict_ptr=dict_end}(0, 1)
    dict_write{dict_ptr=dict_end}(1, 10)
    dict_write{dict_ptr=dict_end}(2, -2)
    dict_update{dict_ptr=dict_end}(0, 1, 2)
    dict_update{dict_ptr=dict_end}(0, 2, 3)
    dict_update{dict_ptr=dict_end}(0, 3, 4)
    dict_update{dict_ptr=dict_end}(1, 10, 15)
    dict_update{dict_ptr=dict_end}(1, 15, 20)
    dict_update{dict_ptr=dict_end}(1, 20, 25)
    dict_update{dict_ptr=dict_end}(2, -2, -4)
    dict_update{dict_ptr=dict_end}(2, -4, -8)
    dict_update{dict_ptr=dict_end}(2, -8, -16)
    let (squashed_dict_start, squashed_dict_end) = dict_squash{
        range_check_ptr=range_check_ptr
    }(dict_start, dict_end)
    assert squashed_dict_end[0] = DictAccess(
        key=0, prev_value=1, new_value=4)
    assert squashed_dict_end[1] = DictAccess(
        key=1, prev_value=10, new_value=25)
    assert squashed_dict_end[2] = DictAccess(
        key=2, prev_value=-2, new_value=-16)
    return ()
end
