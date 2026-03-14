//! Quantum algorithm validation (Grover's Algorithm).
//! Phase 3: Quantum-inspired optimization validator.

use chimera_core::primitives::{Hash, OpCost};
use crate::{SubsystemError, SubsystemOperation};
use num_complex::Complex64;
use blake3::Hasher;

pub struct GroverValidator {
    qubit_count: usize,
    iterations: u32,
}

impl GroverValidator {

    pub fn new() -> Self {
        Self {
            qubit_count: 8,
            iterations: 10,
        }
    }

    pub fn validate(&self, input: &[u8]) -> Result<Hash, SubsystemError> {

        let mut state = self.initialize_state();

        for _ in 0..self.iterations {

            self.oracle(&mut state, input)?;
            self.diffuser(&mut state)?;
        }

        Ok(self.measure(&state))
    }

    fn initialize_state(&self) -> Vec<Complex64> {

        let size = 1 << self.qubit_count;
        let amplitude = 1.0 / (size as f64).sqrt();

        vec![Complex64::new(amplitude, 0.0); size]
    }

    fn oracle(
        &self,
        state: &mut [Complex64],
        input: &[u8],
    ) -> Result<(), SubsystemError> {

        let target = self.target_index(input);

        if target < state.len() {
            state[target] = -state[target]; // phase inversion
        }

        Ok(())
    }

    fn diffuser(
        &self,
        state: &mut [Complex64],
    ) -> Result<(), SubsystemError> {

        let mean: Complex64 =
            state.iter().sum::<Complex64>() / (state.len() as f64);

        for amp in state.iter_mut() {
            *amp = 2.0 * mean - *amp;
        }

        Ok(())
    }

    fn measure(&self, state: &[Complex64]) -> Hash {

        let mut hasher = Hasher::new();

        for amp in state {
            hasher.update(&amp.re.to_le_bytes());
            hasher.update(&amp.im.to_le_bytes());
        }

        Hash(*hasher.finalize().as_bytes())
    }

    fn target_index(&self, input: &[u8]) -> usize {

        let mut acc: usize = 0;

        for b in input {
            acc = acc.wrapping_mul(31).wrapping_add(*b as usize);
        }

        acc % (1 << self.qubit_count)
    }
}

impl Default for GroverValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SubsystemOperation for GroverValidator {

    fn execute(&self, input: &[u8]) -> Result<Hash, SubsystemError> {
        self.validate(input)
    }

    fn cost(&self) -> OpCost {
        OpCost {
            joules: 0.001,
            seconds: 0.0001,
            dollars: 0.00001,
        }
    }

    fn name(&self) -> &'static str {
        "GroverValidator"
    }
}
