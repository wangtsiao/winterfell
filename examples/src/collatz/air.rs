use std::vec;

use core_utils::{Serializable, AsBytes};
use winterfell::{Air, AirContext, EvaluationFrame, TransitionConstraintDegree, Assertion};
use core_utils::ByteWriter;
use crate::utils::{is_binary, are_equal};

use log::debug;
use super::{
    TRACE_WIDTH, TraceInfo, ProofOptions, FieldElement, BaseElement
};

pub struct PublicInputs {
    pub initial_num: BaseElement,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.initial_num);
    }
}

pub struct CollatzAir {
    context: AirContext<BaseElement>,
    initial_num: BaseElement,
}

impl Air for CollatzAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, pub_inputs: Self::PublicInputs, options: ProofOptions) -> Self {
        let degrees: Vec<TransitionConstraintDegree> = vec![
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(2),
        ];
        assert_eq!(TRACE_WIDTH, trace_info.width());
        CollatzAir { 
            context: AirContext::new(trace_info, degrees, 12, options), 
            initial_num: pub_inputs.initial_num,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        if num_from_state(current) == E::ONE {
            return;
        }

        debug_assert_eq!(TRACE_WIDTH, current.len());
        debug_assert_eq!(TRACE_WIDTH, next.len());

        // enforce that values in all register must be binary
        for i in 0..TRACE_WIDTH {
            result[0] += is_binary(current[i]);
        }

        // enforce that each step follow collatz sequence rule
        let current_num = num_from_state(current);
        let next_num = num_from_state(next);
        result[1]  = current[0] * are_equal(current_num * E::from(3 as u32) + E::ONE, next_num);
        result[1] += (current[0] - E::ONE) * are_equal(current_num, E::from(2 as u32) * next_num);
        println!("evaluate transition {} to {}, constraint result {:?}", current_num, next_num, result);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        debug!(
            "set assertion for initial number {:?}",
            self.initial_num,
        );

        let n = self.initial_num.as_bytes();
        let mut n = n[0] as u32;
        let mut bit_vec: Vec<Self::BaseField> = Vec::new();
        for _ in 0..TRACE_WIDTH {
            if n & 1 == 1 {
                bit_vec.push(Self::BaseField::ONE);
            } else {
                bit_vec.push(Self::BaseField::ZERO);
            }
            n >>= 1;
        }

        // BOUNDARY CONSTRAINT
        vec![
            // enforce the first row is our input initial number
            Assertion::single(0, 0, bit_vec[0]),
            Assertion::single(1, 0, bit_vec[1]),
            Assertion::single(2, 0, bit_vec[2]),
            Assertion::single(3, 0, bit_vec[3]),
            Assertion::single(4, 0, bit_vec[4]),
            Assertion::single(5, 0, bit_vec[5]),

            // enforce the last step row is one
            Assertion::single(0, 11, Self::BaseField::ONE),
            Assertion::single(1, 11, Self::BaseField::ZERO),
            Assertion::single(2, 11, Self::BaseField::ZERO),
            Assertion::single(3, 11, Self::BaseField::ZERO),
            Assertion::single(4, 11, Self::BaseField::ZERO),
            Assertion::single(5, 11, Self::BaseField::ZERO),
        ]
    }
}


fn num_from_state<E: FieldElement<BaseField = BaseElement>>(state: &[E]) -> E {
    let mut n: E = E::ONE;
    for i in (0..TRACE_WIDTH).rev() {
        n *= E::from(2 as u32);
        n += state[i];
    }
    n -= E::from((1 << (TRACE_WIDTH)) as u32);
    n
}
