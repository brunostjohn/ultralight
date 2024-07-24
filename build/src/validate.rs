use std::path::Path;

pub(crate) fn validate_directory_contents<P: AsRef<Path>>(dir: P, contents: &[&str]) -> bool {
    let dir = dir.as_ref();
    for content in contents {
        let path = dir.join(content);
        println!("cargo::rerun-if-changed={}", path.display());
        if !path.exists() {
            println!(
                "cargo::warning={} does not exist, will redownload",
                path.display()
            );
            return false;
        }
    }

    true
}
