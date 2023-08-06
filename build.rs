use std::hash::Hasher;
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    path::{Path, PathBuf},
};
use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let windows = std::env::var("CARGO_CFG_TARGET_OS")? == "windows";
    let host_windows = cfg!(target_os = "windows");

    if windows && host_windows {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile()?;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let piped_dir = manifest_dir.join("piped");
    let node_dir = piped_dir.join("node_modules");
    let dist_dir = piped_dir.join("dist");

    println!("cargo:rerun-if-changed=build.rs");

    // re-run in case these get deleted or whatever
    println!("cargo:rerun-if-changed={}", node_dir.display());
    println!("cargo:rerun-if-changed={}", dist_dir.display());

    let piped_backend_dir = manifest_dir.join("piped-backend");
    let build_dir = piped_backend_dir.join("build");

    println!("cargo:rerun-if-changed={}", piped_backend_dir.display());

    let pnpm_cmd = if windows { "pnpm.cmd" } else { "pnpm" };

    // install dependencies

    // install dependencies
    // pnpm install
    if !node_dir.exists() {
        Command::new(pnpm_cmd)
            .arg("install")
            .current_dir(&piped_dir)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // compile dist
    // pnpm build
    if !dist_dir.exists() {
        Command::new(pnpm_cmd)
            .arg("build")
            .current_dir(&piped_dir)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // compile backend
    if !build_dir.exists() {
        let shell = if host_windows { "cmd" } else { "bash" };

        let gradlew = if host_windows {
            "gradlew.bat"
        } else {
            "gradlew"
        };

        Command::new(shell)
            .args([
                if host_windows { "/c" } else { "-c" },
                &format!("{gradlew} shadowJar"),
            ])
            .current_dir(&piped_backend_dir)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // nasty hack to get to the build exe path
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let mut build_piped_jar = build_dir.join("libs");
    build_piped_jar.push("piped-1.0-all.jar");

    let mut hasher = DefaultHasher::new();
    let piped_jar = fs::read(build_piped_jar).unwrap();
    hasher.write(&piped_jar);
    let hash = hasher.finish();

    let out_file = out_dir.join("hash.rs");
    fs::write(
        out_file,
        format!("mod hash {{ pub const JAR_HASH: u64 = {hash}; }}"),
    )
    .unwrap();

    Ok(())
}
