use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub name: String,
    pub provider: String,
    pub parameter_count: String,
    pub min_ram_gb: f64,
    pub recommended_ram_gb: f64,
    pub min_vram_gb: Option<f64>,
    pub quantization: String,
    pub context_length: u32,
    pub use_case: String,
}

/// Intermediate struct matching the JSON schema from the scraper.
/// Extra fields are ignored when mapping to LlmModel.
#[derive(Deserialize)]
struct HfModelEntry {
    name: String,
    provider: String,
    parameter_count: String,
    #[serde(default)]
    parameters_raw: Option<u64>,
    min_ram_gb: f64,
    recommended_ram_gb: f64,
    min_vram_gb: Option<f64>,
    quantization: String,
    context_length: u32,
    use_case: String,
}

const HF_MODELS_JSON: &str = include_str!("../data/hf_models.json");

pub struct ModelDatabase {
    models: Vec<LlmModel>,
}

impl ModelDatabase {
    pub fn new() -> Self {
        let entries: Vec<HfModelEntry> =
            serde_json::from_str(HF_MODELS_JSON).expect("Failed to parse embedded hf_models.json");

        let models = entries
            .into_iter()
            .map(|e| LlmModel {
                name: e.name,
                provider: e.provider,
                parameter_count: e.parameter_count,
                min_ram_gb: e.min_ram_gb,
                recommended_ram_gb: e.recommended_ram_gb,
                min_vram_gb: e.min_vram_gb,
                quantization: e.quantization,
                context_length: e.context_length,
                use_case: e.use_case,
            })
            .collect();

        ModelDatabase { models }
    }

    pub fn get_all_models(&self) -> &Vec<LlmModel> {
        &self.models
    }

    pub fn find_model(&self, query: &str) -> Vec<&LlmModel> {
        let query_lower = query.to_lowercase();
        self.models
            .iter()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.provider.to_lowercase().contains(&query_lower)
                    || m.parameter_count.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn models_fitting_system(&self, available_ram_gb: f64, has_gpu: bool, vram_gb: Option<f64>) -> Vec<&LlmModel> {
        self.models
            .iter()
            .filter(|m| {
                // Check RAM requirement
                let ram_ok = m.min_ram_gb <= available_ram_gb;
                
                // If model requires GPU and system has GPU, check VRAM
                if let Some(min_vram) = m.min_vram_gb {
                    if has_gpu {
                        if let Some(system_vram) = vram_gb {
                            ram_ok && min_vram <= system_vram
                        } else {
                            // GPU detected but VRAM unknown, allow but warn
                            ram_ok
                        }
                    } else {
                        // Model prefers GPU but can run on CPU with enough RAM
                        ram_ok && available_ram_gb >= m.recommended_ram_gb
                    }
                } else {
                    ram_ok
                }
            })
            .collect()
    }
}
