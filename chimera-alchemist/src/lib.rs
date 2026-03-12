//! Chimera Alchemist
//!
//! Strategy evolution engine for ChimeraOS.
//! Generates, mutates, evaluates, and selects optimization strategies
//! for the distributed compute fabric.

use std::sync::Arc;
use tokio::sync::RwLock;

use rand::{thread_rng, Rng};

use chimera_intelligence::{IntelligenceEngine, Strategy};



/// Strategy fitness score
#[derive(Clone, Debug)]
pub struct StrategyScore {

    pub strategy: Strategy,

    pub score: f64,
}



/// Alchemist engine
pub struct AlchemistEngine {

    intelligence: Arc<IntelligenceEngine>,

    population: Arc<RwLock<Vec<Strategy>>>,

    scores: Arc<RwLock<Vec<StrategyScore>>>,

    population_size: usize,
}



impl AlchemistEngine {

    pub fn new(
        intelligence: Arc<IntelligenceEngine>,
        population_size: usize,
    ) -> Self {

        let mut population = Vec::new();

        for _ in 0..population_size {

            population.push(Strategy::default());
        }

        Self {

            intelligence,

            population: Arc::new(RwLock::new(population)),

            scores: Arc::new(RwLock::new(Vec::new())),

            population_size,
        }
    }



    /// Mutate strategy parameters
    fn mutate(strategy: &Strategy) -> Strategy {

        let mut rng = thread_rng();

        Strategy {

            learning_rate:
                (strategy.learning_rate *
                 rng.gen_range(0.8..1.2))
                .clamp(0.0001, 0.1),

            gradient_clip:
                (strategy.gradient_clip *
                 rng.gen_range(0.8..1.2))
                .clamp(1.0, 100.0),

            scheduling_bias:
                (strategy.scheduling_bias *
                 rng.gen_range(0.8..1.2))
                .clamp(0.1, 10.0),
        }
    }



    /// Evaluate a strategy
    pub async fn evaluate_strategy(
        &self,
        strategy: Strategy,
    ) -> StrategyScore {

        let params = vec![
            strategy.learning_rate,
            strategy.gradient_clip,
            strategy.scheduling_bias,
        ];

        let metrics =
            self.intelligence
                .optimize(params)
                .await
                .unwrap();

        StrategyScore {

            strategy,

            score: 1.0 / (metrics.loss_value + 1e-6),
        }
    }



    /// Run one evolutionary generation
    pub async fn evolve_generation(&self) {

        let population = self.population.read().await.clone();

        let mut new_scores = Vec::new();

        for strategy in population {

            let score =
                self.evaluate_strategy(strategy.clone()).await;

            new_scores.push(score);
        }

        new_scores.sort_by(|a, b| {

            b.score
                .partial_cmp(&a.score)
                .unwrap()
        });

        let survivors = new_scores
            .iter()
            .take(self.population_size / 2)
            .cloned()
            .collect::<Vec<_>>();

        let mut next_population = Vec::new();

        for s in &survivors {

            next_population.push(s.strategy.clone());

            next_population.push(Self::mutate(&s.strategy));
        }

        *self.population.write().await = next_population;

        *self.scores.write().await = new_scores;
    }



    /// Get best strategy discovered so far
    pub async fn best_strategy(&self) -> Option<Strategy> {

        let scores = self.scores.read().await;

        scores.first().map(|s| s.strategy.clone())
    }



    /// Population snapshot
    pub async fn population(&self) -> Vec<Strategy> {

        self.population.read().await.clone()
    }
}
