// build.rs

use std::env::{self, VarError};
use std::error::Error;
use std::fs::File;
use std::path::{PathBuf, Path};
use std::process::Command;

use fs_extra::dir::{copy, CopyOptions};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn vex_headers() -> Result<Vec<String>> {
    match env::var("VEX_HEADERS") {
        Ok(paths) => {
            Ok(paths.split(':').map(String::from).collect())
        }
        Err(VarError::NotPresent) => {
            let mut vex = find_vex()?;
            let mut res = Vec::with_capacity(2);
            res.push(vex.to_string_lossy().into_owned());
            vex.pop();
            res.push(vex.to_string_lossy().into_owned());
            Ok(res)
        }
        Err(err) => Err(err.into()),
    }
}

fn apply_patches(valgrind_dir: PathBuf, patch_dir: PathBuf) -> Result<()> {
    let mut patch = Command::new("patch");
    patch
        .arg("-p1")
        .current_dir(valgrind_dir);
    for entry in patch_dir.read_dir()? {
        patch
            .stdin(File::open(entry?.file_name())?)
            .status()?;
    }
    Ok(())
}

fn copy_valgrind(out_dir: &Path, valgrind_dir_name: &str) -> Result<()> {
    let mut options = CopyOptions::default();
    options.copy_inside = true;
    copy(valgrind_dir_name, out_dir, &options)?;
    println!("cargo:rerun-if-changed={}/", valgrind_dir_name);
    match env::var("VEX_PATCHES") {
        Ok(path) => apply_patches(out_dir.join(valgrind_dir_name), PathBuf::from(path))?,
        Err(VarError::NotUnicode(path)) =>
            apply_patches(out_dir.join(valgrind_dir_name), PathBuf::from(path))?,
        Err(VarError::NotPresent) => {}
    }
    Ok(())
}

fn find_vex() -> Result<PathBuf> {
    match env::var("VEX_SRC") {
        Ok(path) => {
            eprintln!("[DEBUG] find_vex: using VEX_SRC env var: {}", path);
            println!("cargo:rerun-if-changed={}", path);
            Ok(PathBuf::from(path))
        },
        Err(VarError::NotUnicode(path)) => {
            eprintln!("[DEBUG] find_vex: VEX_SRC not unicode: {:?}", path);
            Ok(PathBuf::from(path))
        },
        Err(_) => {
            eprintln!("[DEBUG] find_vex: falling back to OUT_DIR logic");
            let out_dir = PathBuf::from(env::var("OUT_DIR")?);
            let valgrind_dir_name = "valgrind";
            let valgrind_dir = out_dir.join(valgrind_dir_name);
            if !valgrind_dir.exists() {
                copy_valgrind(&out_dir, valgrind_dir_name)?;
            }
            if !valgrind_dir.join("configure").exists() {
                Command::new("./autogen.sh")
                    .current_dir(&valgrind_dir)
                    .status()?;
            }
            if !valgrind_dir.join("VEX").join("Makefile").exists() {
                let mut configure = Command::new("./configure");
                configure.current_dir(&valgrind_dir);
                if cfg!(feature = "pic") {
                    configure.arg("CFLAGS=-fPIC");
                }
                configure.status()?;
            }
            Ok(valgrind_dir.join("VEX"))
        },
    }
}

fn compile_vex() -> Result<PathBuf> {
    let src_dir = find_vex()?;
    Command::new("make")
        .env("MAKEFLAGS", env::var("CARGO_MAKEFLAGS").unwrap())
        .current_dir(&src_dir)
        .status()?;

    Ok(src_dir)
}

/// Ensure that libvex*.so is present on the file system.
///
/// Return its directory.
fn ensure_lib() -> Result<PathBuf> {
    match env::var("VEX_LIBS") {
        Ok(path) => {
            eprintln!("[DEBUG] ensure_lib: using VEX_LIBS env var: {}", path);
            Ok(PathBuf::from(path))
        },
        Err(VarError::NotUnicode(path)) => {
            eprintln!("[DEBUG] ensure_lib: VEX_LIBS not unicode: {:?}", path);
            Ok(PathBuf::from(path))
        },
        Err(_) => {
            eprintln!("[DEBUG] ensure_lib: falling back to compile_vex");
            compile_vex()
        },
    }
}

fn main() -> Result<()> {
    // Debug print environment variables at the start
    eprintln!("[DEBUG] VEX_SRC={:?}", std::env::var("VEX_SRC"));
    eprintln!("[DEBUG] VEX_HEADERS={:?}", std::env::var("VEX_HEADERS"));
    eprintln!("[DEBUG] VEX_LIBS={:?}", std::env::var("VEX_LIBS"));
    // --- macOS automation: ensure patched Valgrind is present and envs are set ---
    if cfg!(target_os = "macos") {
        let need_envs = ["VEX_SRC", "VEX_HEADERS", "VEX_LIBS"]
            .iter()
            .any(|&k| std::env::var(k).is_err());
        if need_envs {
            eprintln!("[INFO] macOS detected and VEX env vars not set. Running build-macos.sh to bootstrap...");
            let output = Command::new("./build-macos.sh")
                .arg("--env")
                .output()?;
            if !output.status.success() {
                eprintln!("[ERROR] build-macos.sh failed. Output:\n{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some((key, val)) = line.strip_prefix("export ").and_then(|l| l.split_once('=')) {
                    let val = val.trim_matches('"');
                    std::env::set_var(key, val);
                    eprintln!("[DEBUG] Set {}={}", key, val);
                }
            }
        }
        // Emit rustc-env for all VEX envs so dependents and examples inherit them
        for key in ["VEX_SRC", "VEX_HEADERS", "VEX_LIBS"] {
            if let Ok(val) = std::env::var(key) {
                println!("cargo:rustc-env={}={}", key, val);
            }
        }
    }
    // --- end macOS automation ---

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let host = env::var("HOST")?;

    {
        let (arch, platform) = {
            let mut host_parts = host.as_str().split("-");
            let arch = host_parts.next().unwrap();
            let arch = match arch {
                "x86_64" => "amd64",
                "aarch64" => "arm64",
                other => other,
            };
            let _ = host_parts.next();
            let platform = host_parts.next().unwrap();

            (arch, platform)
        };

        let vex_dir = ensure_lib()?;

        println!("cargo:rustc-link-search=native={}", vex_dir.display());
        let multiarch_lib = format!("vexmultiarch-{}-{}", arch, platform);
        let singlearch_lib = format!("vex-{}-{}", arch, platform);
        let lib_dir = std::path::Path::new(&vex_dir);
        if lib_dir.join(format!("lib{}.a", multiarch_lib)).exists() {
            println!("cargo:rustc-link-lib=static={}", multiarch_lib);
        }
        println!("cargo:rustc-link-lib=static={}", singlearch_lib);
    }

    {
        // Generate bindings
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .blocklist_type("_IRStmt__bindgen_ty_1__bindgen_ty_1")
            .rustified_enum(".*")
            .clang_args(vex_headers()?
                        .into_iter()
                        .map(|dir| format!("-I{}", dir))
            )
            .generate()
            .map_err(|_| "Unable to generate bindings")?;
        bindings.write_to_file(out_dir.join("bindings.rs"))?;
    }

    Ok(())
}
