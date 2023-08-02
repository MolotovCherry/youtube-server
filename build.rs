use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let windows = std::env::var("CARGO_CFG_TARGET_OS")? == "windows";
    let host_windows = cfg!(target_os = "windows");

    if windows && host_windows {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile()?;
    }

    Ok(())
}
