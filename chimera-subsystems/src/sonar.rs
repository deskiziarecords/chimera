//! Signal processing module (Sonar).
//! Phase 3: Processes signal data for pattern recognition.

use chimera_core::primitives::{Hash, OpCost};
use crate::{SubsystemError, SubsystemOperation};
use blake3::Hasher;

pub struct SignalProcessor {
    sample_rate: f64,
    filter_order: usize,
}

impl SignalProcessor {

    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            filter_order: 64,
        }
    }

    pub fn process(&self, input: &[u8]) -> Result<Hash, SubsystemError> {

        let signal = self.bytes_to_signal(input);

        let filtered = self.apply_filter(&signal)?;

        let features = self.extract_features(&filtered);

        Ok(self.features_to_hash(&features))
    }

    fn bytes_to_signal(&self, input: &[u8]) -> Vec<f64> {

        input.iter()
            .map(|&b| (b as f64 / 255.0) * 2.0 - 1.0)
            .collect()
    }

    /// Simple moving-average FIR filter
    fn apply_filter(&self, signal: &[f64]) -> Result<Vec<f64>, SubsystemError> {

        let mut out = Vec::with_capacity(signal.len());

        for i in 0..signal.len() {

            let start = i.saturating_sub(self.filter_order);
            let window = &signal[start..=i];

            let sum: f64 = window.iter().sum();

            out.push(sum / window.len() as f64);
        }

        Ok(out)
    }

    fn extract_features(&self, signal: &[f64]) -> Vec<f64> {

        let mut features = Vec::new();

        // RMS energy
        let rms = (signal.iter().map(|x| x * x).sum::<f64>() / signal.len() as f64).sqrt();
        features.push(rms);

        // mean amplitude
        let mean = signal.iter().sum::<f64>() / signal.len() as f64;
        features.push(mean);

        // zero crossing rate
        let mut crossings = 0;
        for i in 1..signal.len() {
            if signal[i - 1].signum() != signal[i].signum() {
                crossings += 1;
            }
        }

        features.push(crossings as f64 / signal.len() as f64);

        // max amplitude
        let max = signal.iter().fold(0.0_f64, |a, &b| a.max(b.abs()));
        features.push(max);

        features
    }

    fn features_to_hash(&self, features: &[f64]) -> Hash {

        let mut hasher = Hasher::new();

        for f in features {
            hasher.update(&f.to_le_bytes());
        }

        Hash(*hasher.finalize().as_bytes())
    }
}

impl Default for SignalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl SubsystemOperation for SignalProcessor {

    fn execute(&self, input: &[u8]) -> Result<Hash, SubsystemError> {
        self.process(input)
    }

    fn cost(&self) -> OpCost {
        OpCost {
            joules: 0.001,
            seconds: 0.0001,
            dollars: 0.00001,
        }
    }

    fn name(&self) -> &'static str {
        "SignalProcessor"
    }
}
