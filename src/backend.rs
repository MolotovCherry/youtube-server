use std::{
    fs,
    process::Command,
    thread::{self, JoinHandle},
};

use anyhow::anyhow;
use directories::ProjectDirs;

use crate::hash::JAR_HASH;

pub fn run_backend() -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let handle = thread::spawn(|| {
        // it would've been great to use graalvm for this and compile to a shared lib,
        // which I spent an entire day on, but it turns out there's just too many resource configs,
        // proxy configs, etc etc etc, it's too much and way too hard to get working.
        // at least the following works, you know?

        let project_dir =
            ProjectDirs::from("", "", "youtube-server").expect("Failed to get project directory");

        let data_local = project_dir.data_local_dir();

        if !data_local.exists() {
            fs::create_dir_all(data_local).expect("Failed to create project dir");
        }

        let jar_path = data_local.join(format!("piped-{JAR_HASH}.jar"));

        if !jar_path.exists() {
            fs::write(&jar_path, crate::PIPED_JAR).expect("Failed to write jar");
        }

        let jar_path = jar_path.to_str().ok_or(anyhow!("failed to make string"))?;

        // java needs to be on PATH
        Command::new("java")
            .args([
                "-server",
                "-Xmx1G",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+HeapDumpOnOutOfMemoryError",
                "-XX:+OptimizeStringConcat",
                "-XX:+UseStringDeduplication",
                "-XX:+UseCompressedOops",
                "-XX:+UseNUMA",
                "-XX:+UseG1GC",
                "-jar",
                jar_path,
            ])
            .current_dir(data_local)
            .spawn()?
            .wait()?;

        Ok(())
    });

    Ok(handle)
}
