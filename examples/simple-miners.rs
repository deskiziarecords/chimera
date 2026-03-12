// examples/simple-miner.rs
use chimera_core::prelude::*;
use chimera_crypto::sha256::Sha256Engine;
use chimera_fabric::TopologyManager;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Core
    chimera_core::init()?;
    tracing::info!("Starting ChimeraOS v{}", chimera_core::VERSION);

    // 2. Initialize Fabric
    let fabric = TopologyManager::new();
    fabric.detect().await?;
    tracing::info!("Hardware topology detected");

    // 3. Simple Mining Loop (CPU)
    let engine = Sha256Engine::new();
    let mut nonce = Nonce::zero();
    let target = Difficulty::default().to_target();
    let start = Instant::now();

    tracing::info!("Starting mining loop (Phase 1 Validation)...");
    
    // Run for 5 seconds to validate hashrate
    while start.elapsed().as_secs() < 5 {
        let hash = engine.compute(nonce, b"chimera-beta-test")?;
        if hash.meets_difficulty(&target) {
            tracing::info!("Share found! Nonce: {}", nonce);
        }
        nonce.increment();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let hashrate = nonce.value() as f64 / elapsed;
    tracing::info!("Validation complete. Average Hashrate: {:.2} H/s", hashrate);

    Ok(())
}
