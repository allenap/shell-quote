use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Output},
};

/// Return all the paths to binaries on `PATH` with `name`.
///
/// If none are found, return only `name`, bare.
pub(crate) fn find_bins<P: AsRef<std::path::Path>>(name: P) -> Vec<std::path::PathBuf> {
    let name = name.as_ref();
    match std::env::var_os("PATH") {
        Some(path) => {
            // Find every `name` file in `path`, return as absolute paths.
            std::env::split_paths(&path)
                .map(|bindir| bindir.join(name))
                .filter(|bin| bin.exists())
                .collect()
        }
        None => {
            // Return the bare name. If the calling test executes this it will
            // likely fail. This is desirable: we want the test to fail.
            vec![name.into()]
        }
    }
}

pub(crate) fn invoke_shell(bin: &Path, script: &OsStr) -> io::Result<Output> {
    Command::new(bin).arg("-c").arg(script).output()
}
