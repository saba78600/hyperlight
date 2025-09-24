use crate::types::SimpleValue;
use crate::SimpleCodegenApi;
use anyhow::Result;

impl<'ctx> SimpleCodegenApi<'ctx> {
    /// Add a function that returns i64 and accepts parameters described by the
    /// `param_is_float` slice (true => f64, false => i64).
    pub fn add_function(&mut self, name: &str, param_is_float: &[bool]) -> Result<()> {
        let mut params = Vec::new();
        for &is_float in param_is_float {
            if is_float {
                params.push(self.ctx.f64_type().into());
            } else {
                params.push(self.ctx.i64_type().into());
            }
        }
        let fn_type = self.ctx.i64_type().fn_type(&params, false);
        let function = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), function);
        Ok(())
    }

    /// Set the current function by name and position the builder at its end.
    pub fn set_current_function(&mut self, name: &str) -> Result<()> {
        let f = *self
            .functions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown function"))?;
        self.current_fn = Some(f);
        self.current_fn_name = Some(name.to_string());
        Ok(())
    }

    /// Build a call to a function previously added via `add_function`.
    pub fn build_call(
        &mut self,
        name: &str,
        args: &[&SimpleValue<'ctx>],
    ) -> Result<SimpleValue<'ctx>> {
        let function = *self
            .functions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown function"))?;
        // BasicMetadataValueEnum not required here; arguments are converted inline.
        let mut call_args = Vec::new();
        for a in args {
            call_args.push(a.as_basic().into());
        }
        let call_site = self.builder.build_call(function, &call_args, "calltmp")?;
        if let Some(rv) = call_site.try_as_basic_value().left() {
            Ok(SimpleValue::from_basic(rv))
        } else {
            Ok(SimpleValue::from_basic(
                self.ctx.i64_type().const_int(0, false).into(),
            ))
        }
    }

    /// Store the nth parameter of the named function into a previously allocated local.
    pub fn store_param_into_local(
        &mut self,
        function_name: &str,
        param_index: u32,
        local_name: &str,
    ) -> Result<()> {
        let f = *self
            .functions
            .get(function_name)
            .ok_or_else(|| anyhow::anyhow!("function not found"))?;
        let pv = f
            .get_nth_param(param_index)
            .ok_or_else(|| anyhow::anyhow!("param not found"))?;
        let ptr = *self
            .locals
            .get(local_name)
            .ok_or_else(|| anyhow::anyhow!("local not found"))?;
        let _ = self.builder.build_store(ptr, pv);
        Ok(())
    }
}
