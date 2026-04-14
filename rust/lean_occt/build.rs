use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn find_library_dir(root: &Path) -> Option<(PathBuf, &'static str)> {
    let candidates = [
        ("libLeanOcctCAPI.so", "dylib"),
        ("libLeanOcctCAPI.dylib", "dylib"),
        ("LeanOcctCAPI.lib", "dylib"),
        ("libLeanOcctCAPI.a", "static"),
    ];

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if let Some(file_name) = path.file_name().and_then(|value| value.to_str()) {
                for (candidate, kind) in candidates {
                    if file_name == candidate {
                        if let Some(parent) = path.parent() {
                            return Some((parent.to_path_buf(), kind));
                        }
                    }
                }
            }
        }
    }

    None
}

fn main() {
    println!("cargo:rerun-if-env-changed=LEAN_OCCT_CAPI_LIB_DIR");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("rust/lean_occt must live two levels below the repo root")
        .to_path_buf();

    let (lib_dir, link_kind) = if let Ok(explicit_dir) = env::var("LEAN_OCCT_CAPI_LIB_DIR") {
        (PathBuf::from(explicit_dir), "dylib")
    } else {
        find_library_dir(&repo_root.join("build")).unwrap_or_else(|| {
            panic!(
                "Could not find LeanOcctCAPI under {}. Build the C API first with CMake or set LEAN_OCCT_CAPI_LIB_DIR.",
                repo_root.join("build").display()
            )
        })
    };

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib={}={}", link_kind, "LeanOcctCAPI");

    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }
}
