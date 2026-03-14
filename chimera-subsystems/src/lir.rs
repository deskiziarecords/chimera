//! LIR Subsystem
//!
//! Low-level Intermediate Representation engine for ChimeraOS.
//!
//! Responsibilities:
//! - deterministic execution model
//! - verifiable instruction pipelines
//! - cost accounting
//! - hashing for integrity validation
//!
//! Designed for algorithmic subsystems (math, physics, signal processing).

use chimera_core::primitives::{Hash, Nonce, OpCost};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use blake3::Hasher;


/// LIR subsystem errors
#[derive(Debug, Error)]
pub enum LirError {

    #[error("program contains no instructions")]
    EmptyProgram,

    #[error("invalid opcode encountered")]
    InvalidOpcode,

    #[error("operand mismatch")]
    OperandMismatch,

    #[error("cost overflow")]
    CostOverflow,

    #[error("hash mismatch")]
    HashMismatch,
}



/// Core instruction opcodes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Opcode {

    /// Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    /// Linear algebra
    Dot,
    MatMul,

    /// Signal processing
    FFT,
    IFFT,

    /// Control
    Load,
    Store,

    /// Placeholder for subsystem extensions
    Custom(u16),
}



/// Instruction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LirInstruction {

    pub opcode: Opcode,

    /// operands stored as generic register indices / values
    pub operands: Vec<u64>,
}



/// LIR program container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LirProgram {

    /// deterministic nonce for reproducibility
    pub nonce: Nonce,

    /// instruction stream
    pub instructions: Vec<LirInstruction>,
}



/// Execution statistics
#[derive(Debug, Clone)]
pub struct LirStats {

    pub instruction_count: usize,
    pub total_cost: OpCost,
}



impl LirInstruction {

    /// Estimate cost of instruction
    pub fn cost(&self) -> OpCost {

        let c = match self.opcode {

            Opcode::Add | Opcode::Sub => 1,
            Opcode::Mul => 2,
            Opcode::Div => 4,

            Opcode::Dot => 6,
            Opcode::MatMul => 20,

            Opcode::FFT | Opcode::IFFT => 50,

            Opcode::Load | Opcode::Store => 2,

            Opcode::Custom(_) => 5,
        };

        OpCost(c)
    }

}



impl LirProgram {

    /// Compute deterministic program hash
    pub fn compute_hash(&self) -> Hash {

        let mut hasher = Hasher::new();

        hasher.update(&self.nonce.0.to_le_bytes());

        for inst in &self.instructions {

            match inst.opcode {

                Opcode::Add => hasher.update(&[0]),
                Opcode::Sub => hasher.update(&[1]),
                Opcode::Mul => hasher.update(&[2]),
                Opcode::Div => hasher.update(&[3]),

                Opcode::Dot => hasher.update(&[4]),
                Opcode::MatMul => hasher.update(&[5]),

                Opcode::FFT => hasher.update(&[6]),
                Opcode::IFFT => hasher.update(&[7]),

                Opcode::Load => hasher.update(&[8]),
                Opcode::Store => hasher.update(&[9]),

                Opcode::Custom(id) => {
                    hasher.update(&[10]);
                    hasher.update(&id.to_le_bytes());
                }
            }

            for op in &inst.operands {
                hasher.update(&op.to_le_bytes());
            }
        }

        Hash::from_bytes(*hasher.finalize().as_bytes())
    }



    /// Calculate total cost of program
    pub fn cost(&self) -> Result<OpCost, LirError> {

        let mut total: u64 = 0;

        for inst in &self.instructions {

            let c = inst.cost().0;

            total = total
                .checked_add(c)
                .ok_or(LirError::CostOverflow)?;
        }

        Ok(OpCost(total))
    }



    /// Validate program integrity
    pub fn validate(&self) -> Result<(), LirError> {

        if self.instructions.is_empty() {
            return Err(LirError::EmptyProgram);
        }

        for inst in &self.instructions {

            match inst.opcode {

                Opcode::Add
                | Opcode::Sub
                | Opcode::Mul
                | Opcode::Div
                | Opcode::Dot
                | Opcode::MatMul
                | Opcode::FFT
                | Opcode::IFFT
                | Opcode::Load
                | Opcode::Store
                | Opcode::Custom(_) => {}
            }

            if inst.operands.len() > 8 {
                return Err(LirError::OperandMismatch);
            }
        }

        Ok(())
    }



    /// Collect runtime statistics
    pub fn stats(&self) -> Result<LirStats, LirError> {

        Ok(LirStats {

            instruction_count: self.instructions.len(),
            total_cost: self.cost()?,

        })
    }

}



/// Simple program builder
pub struct LirBuilder {

    nonce: Nonce,
    instructions: Vec<LirInstruction>,
}



impl LirBuilder {

    pub fn new(nonce: Nonce) -> Self {

        Self {
            nonce,
            instructions: Vec::new(),
        }
    }

    pub fn push(mut self, opcode: Opcode, operands: Vec<u64>) -> Self {

        self.instructions.push(LirInstruction {
            opcode,
            operands,
        });

        self
    }

    pub fn build(self) -> LirProgram {

        LirProgram {
            nonce: self.nonce,
            instructions: self.instructions,
        }
    }

}
