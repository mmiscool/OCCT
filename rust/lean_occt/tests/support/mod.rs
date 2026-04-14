#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use lean_occt::{ModelDocument, ModelKernel, Shape};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("rust/lean_occt must live two levels below the repo root")
        .to_path_buf()
}

pub fn step_artifact_path(suite: &str, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = repo_root().join("test-artifacts").join("rust").join(suite);
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{name}.step")))
}

pub fn test_guard() -> MutexGuard<'static, ()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("test mutex should not be poisoned")
}

pub fn export_document_shape(
    document: &mut ModelDocument,
    shape_name: &str,
    suite: &str,
    artifact_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = step_artifact_path(suite, artifact_name)?;
    document.export_step(shape_name, &path)?;
    Ok(path)
}

pub fn export_kernel_shape(
    kernel: &ModelKernel,
    shape: &Shape,
    suite: &str,
    artifact_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = step_artifact_path(suite, artifact_name)?;
    kernel.write_step(shape, &path)?;
    Ok(path)
}
