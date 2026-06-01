/// Extended integration tests for candor-sandbox WASM execution.
///
/// These tests compile and run real WebAssembly modules in the
/// wasmtime sandbox to validate actual execution paths, not just
/// construction/error paths.
use candor_sandbox::policy::SandboxPolicy;
use candor_sandbox::wasm_exec::{WasmBackend, WasmExecRequest};

/// A minimal WAT module that returns 42 as exit code.
const MINIMAL_WAT: &str = r#"
(module
    (func (export "run") (result i32)
        i32.const 42
    )
)
"#;

/// A WAT module that loops forever (for fuel limit testing).
const INFINITE_LOOP_WAT: &str = r#"
(module
    (func (export "run") (result i32)
        (loop (br 0))
        i32.const 0
    )
)
"#;

fn compile_wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("Failed to parse WAT")
}

#[tokio::test]
async fn test_wasm_exec_minimal_module() {
    let backend = WasmBackend::default();
    let dir = tempfile::tempdir().unwrap();
    let wasm_path = dir.path().join("test_minimal.wasm");

    let wasm_bytes = compile_wat_to_wasm(MINIMAL_WAT);
    tokio::fs::write(&wasm_path, &wasm_bytes).await.unwrap();

    let request = WasmExecRequest {
        wasm_path,
        function: "run".into(),
        stdin: None,
        timeout_secs: 5,
    };

    let result = backend.execute(&request).await;
    assert!(
        result.is_ok(),
        "WASM execution should succeed, got: {:?}",
        result.err()
    );
    let result = result.unwrap();
    assert_eq!(result.exit_code, 42, "Should return 42 from WASM module");
    assert!(result.fuel_used > 0, "Fuel should be consumed");
}

#[tokio::test]
async fn test_wasm_exec_nonexistent_function() {
    let backend = WasmBackend::default();
    let dir = tempfile::tempdir().unwrap();
    let wasm_path = dir.path().join("test_no_func.wasm");

    let wasm_bytes = compile_wat_to_wasm(MINIMAL_WAT);
    tokio::fs::write(&wasm_path, &wasm_bytes).await.unwrap();

    let request = WasmExecRequest {
        wasm_path,
        function: "nonexistent".into(),
        stdin: None,
        timeout_secs: 5,
    };

    let result = backend.execute(&request).await;
    assert!(result.is_err(), "Should fail with nonexistent function");
}

#[tokio::test]
async fn test_wasm_fuel_limit_exhausted() {
    let policy = SandboxPolicy {
        fuel_limit: Some(100), // Very low fuel — loop will exhaust quickly
        ..SandboxPolicy::default()
    };
    let backend = WasmBackend::new(policy);
    let dir = tempfile::tempdir().unwrap();
    let wasm_path = dir.path().join("test_loop.wasm");

    let wasm_bytes = compile_wat_to_wasm(INFINITE_LOOP_WAT);
    tokio::fs::write(&wasm_path, &wasm_bytes).await.unwrap();

    let request = WasmExecRequest {
        wasm_path,
        function: "run".into(),
        stdin: None,
        timeout_secs: 5,
    };

    let result = backend.execute(&request).await;
    assert!(
        result.is_err(),
        "Infinite loop should be trapped by fuel limit"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("fuel") || err.contains("trap"),
        "Error should mention fuel or trap, got: {}",
        err
    );
}

#[tokio::test]
async fn test_wasm_exec_custom_fuel_works() {
    let policy = SandboxPolicy {
        fuel_limit: Some(1_000_000), // Generous fuel for computing 42
        ..SandboxPolicy::default()
    };
    let backend = WasmBackend::new(policy);
    let dir = tempfile::tempdir().unwrap();
    let wasm_path = dir.path().join("test_fuel.wasm");

    let wasm_bytes = compile_wat_to_wasm(MINIMAL_WAT);
    tokio::fs::write(&wasm_path, &wasm_bytes).await.unwrap();

    let request = WasmExecRequest {
        wasm_path,
        function: "run".into(),
        stdin: None,
        timeout_secs: 5,
    };

    let result = backend.execute(&request).await;
    assert!(result.is_ok(), "Should execute with custom fuel");
}
