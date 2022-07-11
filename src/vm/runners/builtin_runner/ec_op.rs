use std::any::Any;

use crate::math_utils::{ec_add, ec_double};
use crate::types::relocatable::{MaybeRelocatable, Relocatable};
use crate::vm::errors::runner_errors::RunnerError;
use crate::vm::runners::builtin_runner::BuiltinRunner;
use crate::vm::vm_memory::memory::Memory;
use crate::vm::vm_memory::memory_segments::MemorySegmentManager;
use crate::{bigint, bigint_str};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

pub struct EcOpBuiltinRunner {
    included: bool,
    _ratio: usize,
    pub base: Option<Relocatable>,
    cells_per_instance: usize,
    n_input_cells: usize,
    scalar_height: usize,
    _scalar_bits: usize,
    scalar_limit: BigInt,
}

impl EcOpBuiltinRunner {
    pub fn new(included: bool, ratio: usize) -> Self {
        Self {
            included,
            base: None,
            _ratio: ratio,
            n_input_cells: 5,
            cells_per_instance: 7,
            scalar_height: 256,
            _scalar_bits: 252,
            scalar_limit: bigint_str!(
                b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
            ),
        }
    }

    ///Returns True if the point (x, y) is on the elliptic curve defined as
    ///y^2 = x^3 + alpha * x + beta (mod p)
    ///or False otherwise.
    fn point_on_curve(
        x: &BigInt,
        y: &BigInt,
        alpha: &BigInt,
        beta: &BigInt,
        prime: &BigInt,
    ) -> bool {
        (y.pow(2) % prime) == (x.pow(3) + alpha * x + beta) % prime
    }

    ///Returns the result of the EC operation P + m * Q.
    /// where P = (p_x, p_y), Q = (q_x, q_y) are points on the elliptic curve defined as
    /// y^2 = x^3 + alpha * x + beta (mod prime).
    /// Mimics the operation of the AIR, so that this function fails whenever the builtin AIR
    /// would not yield a correct result, i.e. when any part of the computation attempts to add
    /// two points with the same x coordinate.
    fn ec_op_impl(
        mut partial_sum: (BigInt, BigInt),
        mut doubled_point: (BigInt, BigInt),
        m: &BigInt,
        alpha: &BigInt,
        prime: &BigInt,
        height: usize,
    ) -> (BigInt, BigInt) {
        let mut slope = m.clone();
        for _ in 0..height {
            assert!((doubled_point.0.clone() - partial_sum.0.clone())% prime != bigint!(0), "Cannot apply EC operation: computation reched two points with the same x coordinate. \n
            Attempting to compute P + m * Q where:\n
            P = {:?} \n
            m = {}\n
            Q = {:?}.", partial_sum,m, doubled_point);
            if slope.clone() & bigint!(1) != bigint!(0) {
                partial_sum = ec_add(partial_sum, doubled_point.clone(), prime);
            }
            doubled_point = ec_double(doubled_point, alpha, prime);
            slope = slope.clone() >> 1_i32;
        }
        partial_sum
    }
}

impl BuiltinRunner for EcOpBuiltinRunner {
    fn initialize_segments(&mut self, segments: &mut MemorySegmentManager, memory: &mut Memory) {
        self.base = Some(segments.add(memory, None))
    }

    fn initial_stack(&self) -> Result<Vec<MaybeRelocatable>, RunnerError> {
        if self.included {
            if let Some(builtin_base) = &self.base {
                Ok(vec![MaybeRelocatable::RelocatableValue(
                    builtin_base.clone(),
                )])
            } else {
                Err(RunnerError::UninitializedBase)
            }
        } else {
            Ok(Vec::new())
        }
    }

    fn base(&self) -> Option<Relocatable> {
        self.base.clone()
    }
    fn add_validation_rule(&self, _memory: &mut Memory) {}

    fn deduce_memory_cell(
        &mut self,
        address: &MaybeRelocatable,
        memory: &Memory,
    ) -> Result<Option<MaybeRelocatable>, RunnerError> {
        if let &MaybeRelocatable::RelocatableValue(ref relocatable) = address {
            //Constant values declared here
            const EC_POINT_INDICES: [(usize, usize); 3] = [(0, 1), (2, 3), (5, 6)];
            const M_INDEX: usize = 4;
            const OUTPUT_INDICES: (usize, usize) = EC_POINT_INDICES[2];
            let alpha: BigInt = bigint!(1);
            let beta: BigInt = bigint_str!(
                b"3141592653589793238462643383279502884197169399375105820974944592307816406665"
            );
            let field_prime = bigint_str!(
                b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
            );

            let index = relocatable.offset % self.cells_per_instance;
            //Index should be an output cell
            if index != OUTPUT_INDICES.0 && index != OUTPUT_INDICES.1 {
                return Ok(None);
            }
            let instance =
                MaybeRelocatable::from((relocatable.segment_index, relocatable.offset - index));
            //All input cells should be filled, and be integer values
            //If an input cell is not filled, return None
            let mut input_cells = Vec::<&BigInt>::with_capacity(self.n_input_cells);
            for i in 0..self.n_input_cells {
                match memory.get(&instance.add_usize_mod(i, None)) {
                    Err(_) => return Err(RunnerError::MemoryGet(instance.add_usize_mod(i, None))),
                    Ok(value) => match value {
                        None => return Ok(None),
                        Some(addr) => {
                            if let &MaybeRelocatable::Int(ref num) = addr {
                                input_cells.push(num);
                            } else {
                                return Err(RunnerError::ExpectedInteger(
                                    instance.add_usize_mod(i, None),
                                ));
                            }
                        }
                    },
                };
            }
            //Assert that m is under the limit defined by scalar_limit.
            if input_cells[M_INDEX] >= &self.scalar_limit {
                return Err(RunnerError::EcOpBuiltinScalarLimit(
                    self.scalar_limit.clone(),
                ));
            }

            // Assert that if the current address is part of a point, the point is on the curve
            for pair in &EC_POINT_INDICES[0..1] {
                assert!(
                    EcOpBuiltinRunner::point_on_curve(
                        input_cells[pair.0],
                        input_cells[pair.1],
                        &alpha,
                        &beta,
                        &field_prime
                    ),
                    "EcOpBuiltin: point {:?} is not on the curve",
                    pair
                );
            }
            let result = EcOpBuiltinRunner::ec_op_impl(
                (input_cells[0].clone(), input_cells[1].clone()),
                (input_cells[2].clone(), input_cells[3].clone()),
                input_cells[4],
                &alpha,
                &field_prime,
                self.scalar_height,
            );
            match index - self.n_input_cells {
                0 => Ok(Some(MaybeRelocatable::Int(result.0))),
                _ => Ok(Some(MaybeRelocatable::Int(result.1))),
                //Default case corresponds to 1, as there are no other possible cases
            }
        } else {
            Err(RunnerError::NonRelocatableAddress)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::vm_memory::memory::Memory;
    use crate::{bigint, bigint_str};

    #[test]
    fn get_initial_stack_for_ecop_not_included() {
        let builtin = EcOpBuiltinRunner::new(false, 8);
        let initial_stack = builtin.initial_stack();
        assert_eq!(initial_stack, Ok(Vec::new()));
    }

    #[test]
    fn point_is_on_curve_a() {
        let x = bigint_str!(
            b"874739451078007766457464989774322083649278607533249481151382481072868806602"
        );
        let y = bigint_str!(
            b"152666792071518830868575557812948353041420400780739481342941381225525861407"
        );
        let alpha = bigint!(1);
        let beta = bigint_str!(
            b"3141592653589793238462643383279502884197169399375105820974944592307816406665"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert!(EcOpBuiltinRunner::point_on_curve(
            &x, &y, &alpha, &beta, &prime
        ));
    }

    #[test]
    fn point_is_on_curve_b() {
        let x = bigint_str!(
            b"3139037544796708144595053687182055617920475701120786241351436619796497072089"
        );
        let y = bigint_str!(
            b"2119589567875935397690285099786081818522144748339117565577200220779667999801"
        );
        let alpha = bigint!(1);
        let beta = bigint_str!(
            b"3141592653589793238462643383279502884197169399375105820974944592307816406665"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert!(EcOpBuiltinRunner::point_on_curve(
            &x, &y, &alpha, &beta, &prime
        ));
    }

    #[test]
    fn point_is_not_on_curve_a() {
        let x = bigint_str!(
            b"874739454078007766457464989774322083649278607533249481151382481072868806602"
        );
        let y = bigint_str!(
            b"152666792071518830868575557812948353041420400780739481342941381225525861407"
        );
        let alpha = bigint!(1);
        let beta = bigint_str!(
            b"3141592653589793238462643383279502884197169399375105820974944592307816406665"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert!(!EcOpBuiltinRunner::point_on_curve(
            &x, &y, &alpha, &beta, &prime
        ));
    }

    #[test]
    fn point_is_not_on_curve_b() {
        let x = bigint_str!(
            b"3139037544756708144595053687182055617927475701120786241351436619796497072089"
        );
        let y = bigint_str!(
            b"2119589567875935397690885099786081818522144748339117565577200220779667999801"
        );
        let alpha = bigint!(1);
        let beta = bigint_str!(
            b"3141592653589793238462643383279502884197169399375105820974944592307816406665"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert!(!EcOpBuiltinRunner::point_on_curve(
            &x, &y, &alpha, &beta, &prime
        ));
    }

    #[test]
    fn compute_ec_op_impl_valid_a() {
        let partial_sum = (
            bigint_str!(
                b"3139037544796708144595053687182055617920475701120786241351436619796497072089"
            ),
            bigint_str!(
                b"2119589567875935397690285099786081818522144748339117565577200220779667999801"
            ),
        );
        let doubled_point = (
            bigint_str!(
                b"874739451078007766457464989774322083649278607533249481151382481072868806602"
            ),
            bigint_str!(
                b"152666792071518830868575557812948353041420400780739481342941381225525861407"
            ),
        );
        let m = bigint!(34);
        let alpha = bigint!(1);
        let height = 256;
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let result =
            EcOpBuiltinRunner::ec_op_impl(partial_sum, doubled_point, &m, &alpha, &prime, height);
        assert_eq!(
            result,
            (
                bigint_str!(
                    b"1977874238339000383330315148209250828062304908491266318460063803060754089297"
                ),
                bigint_str!(
                    b"2969386888251099938335087541720168257053975603483053253007176033556822156706"
                )
            )
        );
    }

    #[test]
    fn compute_ec_op_impl_valid_b() {
        let partial_sum = (
            bigint_str!(
                b"2962412995502985605007699495352191122971573493113767820301112397466445942584"
            ),
            bigint_str!(
                b"214950771763870898744428659242275426967582168179217139798831865603966154129"
            ),
        );
        let doubled_point = (
            bigint_str!(
                b"874739451078007766457464989774322083649278607533249481151382481072868806602"
            ),
            bigint_str!(
                b"152666792071518830868575557812948353041420400780739481342941381225525861407"
            ),
        );
        let m = bigint!(34);
        let alpha = bigint!(1);
        let height = 256;
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let result =
            EcOpBuiltinRunner::ec_op_impl(partial_sum, doubled_point, &m, &alpha, &prime, height);
        assert_eq!(
            result,
            (
                bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                ),
                bigint_str!(
                    b"3598390311618116577316045819420613574162151407434885460365915347732568210029"
                )
            )
        );
    }

    #[test]
    /* Data taken from this program execution:
       %builtins output ec_op
       from starkware.cairo.common.cairo_builtins import EcOpBuiltin
       from starkware.cairo.common.serialize import serialize_word
       from starkware.cairo.common.ec_point import EcPoint
       from starkware.cairo.common.ec import ec_op

       func main{output_ptr: felt*, ec_op_ptr: EcOpBuiltin*}():
           let x: EcPoint = EcPoint(2089986280348253421170679821480865132823066470938446095505822317253594081284, 1713931329540660377023406109199410414810705867260802078187082345529207694986)

           let y: EcPoint = EcPoint(874739451078007766457464989774322083649278607533249481151382481072868806602,152666792071518830868575557812948353041420400780739481342941381225525861407)
           let z: EcPoint = ec_op(x,34, y)
           serialize_word(z.x)
           return()
           end
    */
    fn deduce_memory_cell_ec_op_for_preset_memory_valid() {
        let mut memory = Memory::new();
        let mut builtin = EcOpBuiltinRunner::new(true, 256);
        for _ in 0..4 {
            memory.data.push(Vec::new());
        }
        memory
            .insert(
                &MaybeRelocatable::from((3, 0)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2962412995502985605007699495352191122971573493113767820301112397466445942584"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 1)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"214950771763870898744428659242275426967582168179217139798831865603966154129"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 2)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"874739451078007766457464989774322083649278607533249481151382481072868806602"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 3)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"152666792071518830868575557812948353041420400780739481342941381225525861407"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 4)),
                &MaybeRelocatable::Int(bigint!(34)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 5)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                )),
            )
            .unwrap();

        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((3, 6)), &memory);
        assert_eq!(
            result,
            Ok(Some(MaybeRelocatable::from(bigint_str!(
                b"3598390311618116577316045819420613574162151407434885460365915347732568210029"
            ))))
        );
    }

    #[test]
    fn deduce_memory_cell_ec_op_for_preset_memory_unfilled_input_cells() {
        let mut memory = Memory::new();
        let mut builtin = EcOpBuiltinRunner::new(true, 256);
        for _ in 0..4 {
            memory.data.push(Vec::new());
        }
        memory
            .insert(
                &MaybeRelocatable::from((3, 1)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"214950771763870898744428659242275426967582168179217139798831865603966154129"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 2)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"874739451078007766457464989774322083649278607533249481151382481072868806602"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 3)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"152666792071518830868575557812948353041420400780739481342941381225525861407"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 4)),
                &MaybeRelocatable::Int(bigint!(34)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 5)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                )),
            )
            .unwrap();

        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((3, 6)), &memory);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn deduce_memory_cell_ec_op_for_preset_memory_addr_not_an_output_cell() {
        let mut memory = Memory::new();
        let mut builtin = EcOpBuiltinRunner::new(true, 256);
        for _ in 0..4 {
            memory.data.push(Vec::new());
        }
        memory
            .insert(
                &MaybeRelocatable::from((3, 0)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2962412995502985605007699495352191122971573493113767820301112397466445942584"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 1)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"214950771763870898744428659242275426967582168179217139798831865603966154129"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 2)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"874739451078007766457464989774322083649278607533249481151382481072868806602"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 3)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"152666792071518830868575557812948353041420400780739481342941381225525861407"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 4)),
                &MaybeRelocatable::Int(bigint!(34)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 5)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                )),
            )
            .unwrap();

        let result = builtin.deduce_memory_cell(&MaybeRelocatable::from((3, 3)), &memory);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn deduce_memory_cell_ec_op_for_preset_memory_non_integer_input() {
        let mut memory = Memory::new();
        let mut builtin = EcOpBuiltinRunner::new(true, 256);
        for _ in 0..4 {
            memory.data.push(Vec::new());
        }
        memory
            .insert(
                &MaybeRelocatable::from((3, 0)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2962412995502985605007699495352191122971573493113767820301112397466445942584"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 1)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"214950771763870898744428659242275426967582168179217139798831865603966154129"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 2)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"874739451078007766457464989774322083649278607533249481151382481072868806602"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 3)),
                &MaybeRelocatable::from((1, 2)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 4)),
                &MaybeRelocatable::Int(bigint!(34)),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 5)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                )),
            )
            .unwrap();

        assert_eq!(
            builtin.deduce_memory_cell(&MaybeRelocatable::from((3, 6)), &memory),
            Err(RunnerError::ExpectedInteger(MaybeRelocatable::from((3, 3))))
        );
    }

    #[test]
    fn deduce_memory_cell_ec_op_for_preset_memory_m_over_scalar_limit() {
        let mut memory = Memory::new();
        let mut builtin = EcOpBuiltinRunner::new(true, 256);
        for _ in 0..4 {
            memory.data.push(Vec::new());
        }
        memory
            .insert(
                &MaybeRelocatable::from((3, 0)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2962412995502985605007699495352191122971573493113767820301112397466445942584"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 1)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"214950771763870898744428659242275426967582168179217139798831865603966154129"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 2)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"874739451078007766457464989774322083649278607533249481151382481072868806602"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 3)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"152666792071518830868575557812948353041420400780739481342941381225525861407"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 4)),
                //Scalar Limit + 1
                &MaybeRelocatable::Int(bigint_str!(
                    b"3618502788666131213697322783095070105623107215331596699973092056135872020482"
                )),
            )
            .unwrap();
        memory
            .insert(
                &MaybeRelocatable::from((3, 5)),
                &MaybeRelocatable::Int(bigint_str!(
                    b"2778063437308421278851140253538604815869848682781135193774472480292420096757"
                )),
            )
            .unwrap();

        let error = builtin.deduce_memory_cell(&MaybeRelocatable::from((3, 6)), &memory);
        assert_eq!(
            error,
            Err(RunnerError::EcOpBuiltinScalarLimit(
                builtin.scalar_limit.clone()
            ))
        );
    }
}
