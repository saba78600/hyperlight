use codegen_api::{Op, SimpleCodegenApi};

#[test]
fn test_create_entry_and_emit_ir() {
    let mut api = SimpleCodegenApi::new("test_module");
    api.create_entry();
    let zero = api.const_i64(0);
    api.build_return(&zero).unwrap();
    let ir = api.emit_ir();
    assert!(
        ir.contains("define i64 @main"),
        "IR should define main: {}",
        ir
    );
}

#[test]
fn test_binop_and_printf_ir() {
    let mut api = SimpleCodegenApi::new("test_binop");
    api.create_entry();
    let a = api.const_i64(4);
    let b = api.const_i64(2);
    let sum = api.build_binop(Op::Add, &a, &b).unwrap();
    let _ = api.call_printf(&sum).unwrap();
    api.build_return(&api.const_i64(0)).unwrap();
    let ir = api.emit_ir();
    // The add may be constant-folded; ensure we at least emit a printf call and a format string
    assert!(
        ir.contains("call i32 (ptr, ...) @printf") || ir.contains("call i32 @printf"),
        "IR should contain a printf call: {}",
        ir
    );
    assert!(
        ir.contains("fmt_i64") || ir.contains("@.str"),
        "IR should contain a format string: {}",
        ir
    );
}
