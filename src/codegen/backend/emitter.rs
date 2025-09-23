use anyhow::Result;
use super::types::BackendState;
use std::path::Path;
use inkwell::targets::{Target, InitializationConfig, RelocMode, CodeModel, FileType, TargetMachine};

pub fn emit_object_for_module<'ctx>(state: &BackendState<'ctx>, out: &Path) -> Result<()> {
    Target::initialize_all(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| anyhow::anyhow!("failed to get target from triple: {}", e))?;
    let cpu = TargetMachine::get_host_cpu_name();
    let features = TargetMachine::get_host_cpu_features();
    let tm = target.create_target_machine(&triple, &cpu.to_string(), &features.to_string(), inkwell::OptimizationLevel::Default, RelocMode::PIC, CodeModel::Default).ok_or_else(|| anyhow::anyhow!("failed to create target machine"))?;
    tm.write_to_file(&state.module, FileType::Object, out).map_err(|e| anyhow::anyhow!("failed to write object file: {:?}", e))?;
    Ok(())
}
