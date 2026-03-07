pub struct Alchemist {
    config: AlchemistConfig,
    llm: Box<dyn LanguageModel + Send + Sync>,
    cell_registry: Arc<CellRegistry>,
    fabric_manager: Arc<FabricManager>,
}

impl Alchemist {
    pub async fn remix(&self, intent: &str) -> Result<MiningStrategy, AlchemistError> {
        let spec = self.parse_intent(intent).await?;
        let strategy = self.generate_strategy(spec).await?;
        Ok(strategy)
    }
}