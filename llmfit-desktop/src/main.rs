#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use llmfit_core::fit::{FitLevel, ModelFit, RunMode};
use llmfit_core::hardware::SystemSpecs;
use llmfit_core::models::ModelDatabase;
use serde::Serialize;

#[derive(Serialize)]
struct GpuInfoJs {
    name: String,
    vram_gb: Option<f64>,
    backend: String,
    count: u32,
    unified_memory: bool,
}

#[derive(Serialize)]
struct SystemInfo {
    total_ram_gb: f64,
    available_ram_gb: f64,
    cpu_name: String,
    cpu_cores: usize,
    gpus: Vec<GpuInfoJs>,
    unified_memory: bool,
}

#[derive(Serialize)]
struct ModelFitInfo {
    name: String,
    params_b: f64,
    quant: String,
    fit_level: String,
    run_mode: String,
    score: f64,
    memory_required_gb: f64,
    memory_available_gb: f64,
    estimated_tps: f64,
    use_case: String,
    notes: Vec<String>,
}

#[tauri::command]
fn get_system_specs() -> Result<SystemInfo, String> {
    let specs = SystemSpecs::detect();
    let gpus = specs
        .gpus
        .iter()
        .map(|g| GpuInfoJs {
            name: g.name.clone(),
            vram_gb: g.vram_gb,
            backend: format!("{:?}", g.backend),
            count: g.count,
            unified_memory: g.unified_memory,
        })
        .collect();
    Ok(SystemInfo {
        total_ram_gb: specs.total_ram_gb,
        available_ram_gb: specs.available_ram_gb,
        cpu_name: specs.cpu_name.clone(),
        cpu_cores: specs.total_cpu_cores,
        gpus,
        unified_memory: specs.unified_memory,
    })
}

#[tauri::command]
fn get_model_fits() -> Result<Vec<ModelFitInfo>, String> {
    let specs = SystemSpecs::detect();
    let db = ModelDatabase::new();

    let mut fits: Vec<ModelFit> = db
        .get_all_models()
        .iter()
        .map(|m| ModelFit::analyze(m, &specs))
        .collect();

    fits = llmfit_core::fit::rank_models_by_fit(fits);

    Ok(fits
        .into_iter()
        .map(|f| ModelFitInfo {
            name: f.model.name.clone(),
            params_b: f.model.parameters_raw.unwrap_or(0) as f64 / 1e9,
            quant: f.best_quant.clone(),
            fit_level: match f.fit_level {
                FitLevel::Perfect => "Perfect".to_string(),
                FitLevel::Good => "Good".to_string(),
                FitLevel::Marginal => "Marginal".to_string(),
                FitLevel::TooTight => "Too Tight".to_string(),
            },
            run_mode: match f.run_mode {
                RunMode::Gpu => "GPU".to_string(),
                RunMode::CpuOffload => "CPU Offload".to_string(),
                RunMode::CpuOnly => "CPU Only".to_string(),
                RunMode::MoeOffload => "MoE Offload".to_string(),
            },
            score: f.score,
            memory_required_gb: f.memory_required_gb,
            memory_available_gb: f.memory_available_gb,
            estimated_tps: f.estimated_tps,
            use_case: format!("{:?}", f.use_case),
            notes: f.notes.clone(),
        })
        .collect())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_system_specs, get_model_fits])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
