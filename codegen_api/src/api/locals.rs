use crate::types::SimpleValue;
use crate::SimpleCodegenApi;
use anyhow::Result;
use inkwell::values::PointerValue;

impl<'ctx> SimpleCodegenApi<'ctx> {
    /// Provide a pointer to a local by name if needed by caller code.
    pub fn get_local_ptr(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.locals.get(name).copied()
    }

    /// Create a 64-bit integer constant value.
    pub fn const_i64(&self, v: i64) -> SimpleValue<'ctx> {
        SimpleValue::from_basic(self.ctx.i64_type().const_int(v as u64, true).into())
    }

    /// Create a 64-bit float constant value.
    pub fn const_f64(&self, v: f64) -> SimpleValue<'ctx> {
        SimpleValue::from_basic(self.ctx.f64_type().const_float(v).into())
    }

    /// Allocate an i64 local in the entry block and optionally initialize it.
    pub fn alloc_local_i64(&mut self, name: &str, init: Option<&SimpleValue<'ctx>>) -> Result<()> {
        let function = self
            .current_fn
            .expect("no current function; call create_entry first");
        let entry = function
            .get_first_basic_block()
            .expect("function missing entry block");
        let tmp = self.ctx.create_builder();
        if let Some(first) = entry.get_first_instruction() {
            tmp.position_before(&first);
        } else {
            tmp.position_at_end(entry);
        }
        let alloca = tmp.build_alloca(self.ctx.i64_type(), name)?;
        if let Some(iv) = init {
            self.builder.build_store(alloca, iv.as_basic())?;
        }
        self.locals.insert(name.to_string(), alloca);
        Ok(())
    }

    /// Allocate an f64 local in the entry block and optionally initialize it.
    pub fn alloc_local_f64(&mut self, name: &str, init: Option<&SimpleValue<'ctx>>) -> Result<()> {
        let function = self
            .current_fn
            .expect("no current function; call create_entry first");
        let entry = function
            .get_first_basic_block()
            .expect("function missing entry block");
        let tmp = self.ctx.create_builder();
        if let Some(first) = entry.get_first_instruction() {
            tmp.position_before(&first);
        } else {
            tmp.position_at_end(entry);
        }
        let alloca = tmp.build_alloca(self.ctx.f64_type(), name)?;
        if let Some(iv) = init {
            self.builder.build_store(alloca, iv.as_basic())?;
        }
        self.locals.insert(name.to_string(), alloca);
        Ok(())
    }

    /// Store an i64 value into a previously allocated local.
    pub fn store_local_i64(&mut self, name: &str, val: &SimpleValue<'ctx>) -> Result<()> {
        let ptr = *self
            .locals
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        self.builder.build_store(ptr, val.as_basic())?;
        Ok(())
    }

    /// Store an f64 value into a previously allocated local.
    pub fn store_local_f64(&mut self, name: &str, val: &SimpleValue<'ctx>) -> Result<()> {
        let ptr = *self
            .locals
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        self.builder.build_store(ptr, val.as_basic())?;
        Ok(())
    }

    /// Load an i64 local and return a SimpleValue wrapper.
    pub fn load_local_i64(&mut self, name: &str) -> Result<SimpleValue<'ctx>> {
        let ptr = *self
            .locals
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        let loaded =
            self.builder
                .build_load(self.ctx.i64_type(), ptr, &format!("load_{}", name))?;
        Ok(SimpleValue::from_basic(loaded))
    }

    /// Load an f64 local and return a SimpleValue wrapper.
    pub fn load_local_f64(&mut self, name: &str) -> Result<SimpleValue<'ctx>> {
        let ptr = *self
            .locals
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        let loaded =
            self.builder
                .build_load(self.ctx.f64_type(), ptr, &format!("load_{}", name))?;
        Ok(SimpleValue::from_basic(loaded))
    }
}
