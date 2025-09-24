use crate::types::SimpleValue;
use crate::SimpleCodegenApi;
use anyhow::Result;

impl<'ctx> SimpleCodegenApi<'ctx> {
    /// Emit an object file for the current module. This encapsulates all the
    /// Inkwell target setup so callers don't need to depend on `inkwell`.
    pub fn emit_object_for_path(&self, out: &std::path::Path) -> Result<()> {
        use inkwell::targets::{
            CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
        };
        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| anyhow::anyhow!("failed to get target from triple: {}", e))?;
        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();
        let tm = target
            .create_target_machine(
                &triple,
                &cpu.to_string(),
                &features.to_string(),
                inkwell::OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow::anyhow!("failed to create target machine"))?;
        tm.write_to_file(&self.module, FileType::Object, out)
            .map_err(|e| anyhow::anyhow!("failed to write object file: {:?}", e))?;
        Ok(())
    }

    /// Helper that builds a call to libc's `printf` for a single argument.
    /// Returns an i64 zero value as a SimpleValue (matching the previous
    /// compiler behavior where `print` returns 0).
    pub fn call_printf(&mut self, val: &SimpleValue<'ctx>) -> Result<SimpleValue<'ctx>> {
        use inkwell::values::BasicValueEnum as BVE;
        let i32_t = self.ctx.i32_type();
        let printf_ty = i32_t.fn_type(
            &[self.ctx.ptr_type(inkwell::AddressSpace::default()).into()],
            true,
        );
        let printf_fn = match self.module.get_function("printf") {
            Some(f) => f,
            None => self.module.add_function("printf", printf_ty, None),
        };
        match val.as_basic() {
            BVE::IntValue(iv) => {
                let fmt_g = self.builder.build_global_string_ptr("%lld\n", "fmt_i64")?;
                let fmt_ptr = fmt_g.as_pointer_value();
                let _ =
                    self.builder
                        .build_call(printf_fn, &[fmt_ptr.into(), iv.into()], "call_printf");
                Ok(SimpleValue::from_basic(
                    self.ctx.i64_type().const_int(0, false).into(),
                ))
            }
            BVE::FloatValue(fv) => {
                let fmt_g = self.builder.build_global_string_ptr("%f\n", "fmt_f64")?;
                let fmt_ptr = fmt_g.as_pointer_value();
                let _ =
                    self.builder
                        .build_call(printf_fn, &[fmt_ptr.into(), fv.into()], "call_printf");
                Ok(SimpleValue::from_basic(
                    self.ctx.i64_type().const_int(0, false).into(),
                ))
            }
            _ => Err(anyhow::anyhow!("print: unsupported argument type")),
        }
    }

    /// Return textual IR
    pub fn emit_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write IR to a file
    pub fn write_ir_to(&self, path: &std::path::Path) -> Result<()> {
        std::fs::write(path, self.emit_ir())?;
        Ok(())
    }
}
