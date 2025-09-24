use crate::types::SimpleValue;
use crate::SimpleCodegenApi;
use anyhow::Result;
use inkwell::FloatPredicate as FP;
use inkwell::IntPredicate as IP;

impl<'ctx> SimpleCodegenApi<'ctx> {
    /// Create (or replace) a `main` function that returns i64 and set builder
    /// insertion point at its entry block.
    pub fn create_entry(&mut self) {
        let i64t = self.ctx.i64_type();
        let fn_type = i64t.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let bb = self.ctx.append_basic_block(function, "entry");
        self.builder.position_at_end(bb);
        self.current_fn = Some(function);
        self.current_fn_name = Some("main".to_string());
        self.functions.insert("main".to_string(), function);
        self.blocks.insert(format!("main::{}", "entry"), bb);
    }

    /// Append a basic block to the current function and register it under
    /// the name provided.
    pub fn append_basic_block(&mut self, name: &str) -> Result<()> {
        let function = self
            .current_fn
            .ok_or_else(|| anyhow::anyhow!("no current function"))?;
        let bb = self.ctx.append_basic_block(function, name);
        let key = format!("{}::{}", self.current_fn_name.as_ref().unwrap(), name);
        self.blocks.insert(key, bb);
        Ok(())
    }

    /// Position the builder at the end of a named basic block in the current function.
    pub fn position_at_end(&mut self, name: &str) -> Result<()> {
        let key = format!(
            "{}::{}",
            self.current_fn_name
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no current function"))?,
            name
        );
        let bb = *self
            .blocks
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("unknown block"))?;
        self.builder.position_at_end(bb);
        Ok(())
    }

    /// Save the current insert block onto a stack so it can be restored later.
    pub fn save_insert_block(&mut self) {
        if let Some(bb) = self.builder.get_insert_block() {
            self.saved_blocks.push(bb);
        }
    }

    /// Restore the most recently saved insert block (if any).
    pub fn restore_insert_block(&mut self) {
        if let Some(bb) = self.saved_blocks.pop() {
            self.builder.position_at_end(bb);
        }
    }

    /// Return whether the current insert block has a terminator.
    pub fn current_block_has_terminator(&self) -> bool {
        if let Some(bb) = self.builder.get_insert_block() {
            bb.get_terminator().is_some()
        } else {
            false
        }
    }

    /// Build a conditional branch using a numeric SimpleValue as condition.
    pub fn build_conditional_branch(
        &mut self,
        cond: &SimpleValue<'ctx>,
        then_name: &str,
        else_name: &str,
    ) -> Result<()> {
        use inkwell::values::BasicValueEnum as BVE;
        let then_key = format!(
            "{}::{}",
            self.current_fn_name
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no current function"))?,
            then_name
        );
        let else_key = format!("{}::{}", self.current_fn_name.as_ref().unwrap(), else_name);
        let then_bb = *self
            .blocks
            .get(&then_key)
            .ok_or_else(|| anyhow::anyhow!("unknown then block"))?;
        let else_bb = *self
            .blocks
            .get(&else_key)
            .ok_or_else(|| anyhow::anyhow!("unknown else block"))?;
        match cond.as_basic() {
            BVE::IntValue(i) => {
                let zero = i.get_type().const_int(0, false);
                let cmp = self.builder.build_int_compare(IP::NE, i, zero, "tobool")?;
                self.builder
                    .build_conditional_branch(cmp, then_bb, else_bb)?;
                Ok(())
            }
            BVE::FloatValue(fv) => {
                let zerof = fv.get_type().const_float(0.0);
                let cmp = self
                    .builder
                    .build_float_compare(FP::ONE, fv, zerof, "tobool")?;
                self.builder
                    .build_conditional_branch(cmp, then_bb, else_bb)?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("condition must be numeric")),
        }
    }

    /// Build an unconditional branch to a named basic block in the current function.
    pub fn build_unconditional_branch(&mut self, target_name: &str) -> Result<()> {
        let key = format!(
            "{}::{}",
            self.current_fn_name
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("no current function"))?,
            target_name
        );
        let bb = *self
            .blocks
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("unknown target block"))?;
        self.builder.build_unconditional_branch(bb)?;
        Ok(())
    }
}
