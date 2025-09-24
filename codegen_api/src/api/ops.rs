use crate::types::Op;
use crate::types::SimpleValue;
use crate::SimpleCodegenApi;
use anyhow::Result;

impl<'ctx> SimpleCodegenApi<'ctx> {
    /// Return true if a SimpleValue is an integer value.
    pub fn is_int(&self, v: &SimpleValue<'ctx>) -> bool {
        matches!(v.as_basic(), inkwell::values::BasicValueEnum::IntValue(_))
    }

    /// Return true if a SimpleValue is a float value.
    pub fn is_float(&self, v: &SimpleValue<'ctx>) -> bool {
        matches!(v.as_basic(), inkwell::values::BasicValueEnum::FloatValue(_))
    }

    /// Ensure the given SimpleValue is an i64 value. May perform sign-extension
    /// or float->int conversion as required. Returns a new SimpleValue that is
    /// guaranteed to be an i64 integer value.
    pub fn ensure_i64(&mut self, v: &SimpleValue<'ctx>) -> Result<SimpleValue<'ctx>> {
        use inkwell::values::BasicValueEnum as BVE;
        match v.as_basic() {
            BVE::IntValue(iv) => {
                let it = iv.get_type();
                if it.get_bit_width() < 64 {
                    let sext =
                        self.builder
                            .build_int_s_extend(iv, self.ctx.i64_type(), "ret_ext")?;
                    Ok(SimpleValue::from_basic(sext.into()))
                } else {
                    Ok(SimpleValue::from_basic(iv.into()))
                }
            }
            BVE::FloatValue(fv) => {
                let conv =
                    self.builder
                        .build_float_to_signed_int(fv, self.ctx.i64_type(), "fptosi")?;
                Ok(SimpleValue::from_basic(conv.into()))
            }
            _ => Err(anyhow::anyhow!("unsupported conversion to i64")),
        }
    }

    /// Build a return for any value by first coercing it to i64 (if needed)
    /// and returning that value.
    pub fn build_return(&mut self, v: &SimpleValue<'ctx>) -> Result<()> {
        let iv = self.ensure_i64(v)?;
        self.build_return_i64(&iv)
    }

    /// Build a binary operation. This function hides all Inkwell details and
    /// performs simple numeric coercions (int->float) when needed.
    pub fn build_binop(
        &mut self,
        op: Op,
        a: &SimpleValue<'ctx>,
        b: &SimpleValue<'ctx>,
    ) -> Result<SimpleValue<'ctx>> {
        use inkwell::values::BasicValueEnum as BVE;

        match (a.as_basic(), b.as_basic()) {
            (BVE::IntValue(ai), BVE::IntValue(bi)) => {
                // comparisons produce i64 booleans extended
                use inkwell::IntPredicate as IP;
                match op {
                    Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                        let pred = match op {
                            Op::Eq => IP::EQ,
                            Op::Ne => IP::NE,
                            Op::Lt => IP::SLT,
                            Op::Le => IP::SLE,
                            Op::Gt => IP::SGT,
                            Op::Ge => IP::SGE,
                            _ => unreachable!(),
                        };
                        let cmp = self.builder.build_int_compare(pred, ai, bi, "cmptmp")?;
                        let zext = self.builder.build_int_z_extend(
                            cmp,
                            self.ctx.i64_type(),
                            "bool_to_i64",
                        )?;
                        Ok(SimpleValue::from_basic(zext.into()))
                    }
                    Op::Add => Ok(SimpleValue::from_basic(
                        self.builder.build_int_add(ai, bi, "addtmp")?.into(),
                    )),
                    Op::Sub => Ok(SimpleValue::from_basic(
                        self.builder.build_int_sub(ai, bi, "subtmp")?.into(),
                    )),
                    Op::Mul => Ok(SimpleValue::from_basic(
                        self.builder.build_int_mul(ai, bi, "multmp")?.into(),
                    )),
                    Op::Div => Ok(SimpleValue::from_basic(
                        self.builder.build_int_signed_div(ai, bi, "divtmp")?.into(),
                    )),
                    Op::Mod => Ok(SimpleValue::from_basic(
                        self.builder.build_int_signed_rem(ai, bi, "modtmp")?.into(),
                    )),
                }
            }
            (BVE::FloatValue(af), BVE::FloatValue(bf)) => {
                use inkwell::FloatPredicate as FP;
                match op {
                    Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                        let pred = match op {
                            Op::Eq => FP::OEQ,
                            Op::Ne => FP::ONE,
                            Op::Lt => FP::OLT,
                            Op::Le => FP::OLE,
                            Op::Gt => FP::OGT,
                            Op::Ge => FP::OGE,
                            _ => unreachable!(),
                        };
                        let cmp = self.builder.build_float_compare(pred, af, bf, "fcmp")?;
                        let zext = self.builder.build_int_z_extend(
                            cmp,
                            self.ctx.i64_type(),
                            "bool_to_i64",
                        )?;
                        Ok(SimpleValue::from_basic(zext.into()))
                    }
                    Op::Add => Ok(SimpleValue::from_basic(
                        self.builder.build_float_add(af, bf, "faddtmp")?.into(),
                    )),
                    Op::Sub => Ok(SimpleValue::from_basic(
                        self.builder.build_float_sub(af, bf, "fsubtmp")?.into(),
                    )),
                    Op::Mul => Ok(SimpleValue::from_basic(
                        self.builder.build_float_mul(af, bf, "fmultmp")?.into(),
                    )),
                    Op::Div => Ok(SimpleValue::from_basic(
                        self.builder.build_float_div(af, bf, "fdivtmp")?.into(),
                    )),
                    _ => Err(anyhow::anyhow!("unsupported float op")),
                }
            }
            (BVE::IntValue(ai), BVE::FloatValue(_bf)) => {
                let af =
                    self.builder
                        .build_signed_int_to_float(ai, self.ctx.f64_type(), "sitofp")?;
                self.build_binop(op, &SimpleValue::from_basic(af.into()), b)
            }
            (BVE::FloatValue(_af), BVE::IntValue(bi)) => {
                let bf =
                    self.builder
                        .build_signed_int_to_float(bi, self.ctx.f64_type(), "sitofp")?;
                self.build_binop(op, a, &SimpleValue::from_basic(bf.into()))
            }
            _ => Err(anyhow::anyhow!("unsupported operand types for build_binop")),
        }
    }

    /// Convenience to return an i64 from the current function.
    pub fn build_return_i64(&mut self, v: &SimpleValue<'ctx>) -> Result<()> {
        use inkwell::values::BasicValueEnum as BVE;
        match v.as_basic() {
            BVE::IntValue(iv) => {
                self.builder.build_return(Some(&iv))?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "build_return_i64 expects an integer SimpleValue"
            )),
        }
    }
}
