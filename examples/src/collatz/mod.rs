use winterfell::{
    crypto::ElementHasher,
    math::{fields::f128::BaseElement, log2, FieldElement},
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError, TraceInfo
};
use crate::{
    Blake3_192, Blake3_256, Sha3_256, HashFunction, Example, ExampleOptions,
};

use log::debug;
use std::time::Instant;
use core::marker::PhantomData;

mod prover;
use prover::CollatzProver;

mod air;
use air::{CollatzAir, PublicInputs};

// CONSTANTS
// ================================================================================================
const TRACE_WIDTH: usize = 7;

// COLLATZ PATH EXAMPLE
// ================================================================================================
pub fn get_example(
    options: &ExampleOptions,
    initial_number: usize,
) -> Result<Box<dyn Example>, String> {
    let (options, hash_fn) = options.to_proof_options(28, 8);

    match hash_fn {
        HashFunction::Blake3_192 => Ok(Box::new(CollatzExample::<Blake3_192>::new(
            initial_number, options,
        ))),
        HashFunction::Blake3_256 => Ok(Box::new(CollatzExample::<Blake3_256>::new(
            initial_number, options,
        ))),
        HashFunction::Sha3_256 => Ok(Box::new(CollatzExample::<Sha3_256>::new(
            initial_number, options,
        ))),
        _ => Err("The specified hash function cannot be used with this example.".to_string()),
    }
}

pub struct CollatzExample<H: ElementHasher> {
    options: ProofOptions,
    initial_number: usize,
    step: usize,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher> CollatzExample<H> {
    pub fn new(initial_number: usize, options: ProofOptions) -> Self {
        assert!(
            initial_number < 100,
            "initial number must be less than 100"
        );
        let now: Instant = Instant::now();
        let step: usize = compute_collatz(initial_number);
        debug!(
            "comput collatz sequence from {} using step {} in {} ms",
            initial_number,
            step,
            now.elapsed().as_millis(),
        );

        CollatzExample {
            options,
            initial_number,
            step,
            _hasher: PhantomData,
        }
    }
}

impl<H: ElementHasher> Example for CollatzExample<H>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    fn prove(&self) -> StarkProof {
        let prover: CollatzProver<H> = CollatzProver::<H>::new(self.options.clone(), self.step);

        // generate the execution trace
        let now: Instant = Instant::now();
        let trace: TraceTable<BaseElement> = prover.build_trace(self.initial_number, self.step);
        let trace_length: usize = trace.length();

        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {}ms",
            trace.width(),
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs: PublicInputs = PublicInputs {
            initial_num: BaseElement::new(self.initial_number as u128),
            step: BaseElement::new(self.step as u128),
        };
        winterfell::verify::<CollatzAir, H>(proof, pub_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs: PublicInputs = PublicInputs {
            initial_num: BaseElement::new(self.initial_number as u128),
            step: BaseElement::new((self.step + 1) as u128),
        };
        winterfell::verify::<CollatzAir, H>(proof, pub_inputs)
    }
}

fn compute_collatz(initial_number: usize) -> usize {
    let (mut n, mut step) = (initial_number, 0);
    while n > 1 {
        if n & 1 == 1 {
            n = 3 * n + 1;
        } else {
            n >>= 1;
        }
        step += 1;
    }
    step
}
