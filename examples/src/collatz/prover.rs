use std::{marker::PhantomData};
use core_utils::AsBytes;

use log::debug;
use super::air::{CollatzAir, PublicInputs};
use super::{
    TRACE_WIDTH, ElementHasher, ProofOptions, TraceTable, BaseElement, Prover, FieldElement
};

pub struct CollatzProver<H: ElementHasher> {
    options: ProofOptions,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher> CollatzProver<H> {
    pub fn new(options: ProofOptions) -> Self {
        Self { 
            options, 
            _hasher: PhantomData, 
        }
    }

    pub fn build_trace(
        &self,
        initial_number: usize,
        step: usize,
    ) -> TraceTable<BaseElement> {
        // Allocate memory to hold the trace table
        let trace_length: usize;
        if step.is_power_of_two() == false {
            trace_length = ceil_to_power_of_two(step);
        } else {
            trace_length = step;
        }
        
        assert!(
            trace_length.is_power_of_two(),
            "trace table length must be a power of 2"
        );

        debug!(
            "allocate trace table of length {}",
            trace_length,
        );

        let mut trace: TraceTable<BaseElement> = TraceTable::new(TRACE_WIDTH, trace_length);

        trace.fill(
            |state: &mut [BaseElement]| {
                let mut n = initial_number;
                for i in 0..TRACE_WIDTH {
                    state[i] = BaseElement::new((n & 1_usize) as u128);
                    n >>= 1;
                }
            },
            |_, state: &mut [BaseElement]| {
                // Compute number from a row of state
                let mut n = BaseElement::ONE;
                for i in (0..TRACE_WIDTH).rev() {
                    n *= BaseElement::new(2);
                    n = n + state[i];
                }
                n -= BaseElement::new(1 << (TRACE_WIDTH));


                println!("n = {:?}, update state {:?}", n, state);

                // If the number is ONE, then reach pad table, return directly.
                if n == BaseElement::ONE {
                    return;
                }

                // Update State follow collatz sequence
                if state[0] == BaseElement::ZERO {
                    n = n / BaseElement::new(2);
                } else {
                    n = n * BaseElement::new(3) + BaseElement::ONE;
                }

                // Not sure how to conver BaseElement to integer
                let n_bytes = n.as_bytes();
                let mut n = n_bytes[0] as u32;
                for i in 0..TRACE_WIDTH {
                    state[i] = BaseElement::new((n & 1) as u128);
                    n >>= 1;              
                }
            },
        );

        trace
    }
}

impl<H: ElementHasher> Prover for CollatzProver<H>
where
    H: ElementHasher<BaseField = BaseElement>
{
    type BaseField = BaseElement;
    type Air = CollatzAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        // Trace table is redundant
        let mut n = BaseElement::ONE;
        for i in (0..TRACE_WIDTH).rev() {
            n *= BaseElement::new(2);
            n = n + trace.get(i, 0);
        }
        n -= BaseElement::new(1 << (TRACE_WIDTH));

        PublicInputs { 
           initial_num: n,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

// HELPER FUNCTION
// ---------------------------------------------------------------------
fn ceil_to_power_of_two(n: usize) -> usize {
    let e = (n as f64).log2().ceil() as usize;
    1 << e
}
