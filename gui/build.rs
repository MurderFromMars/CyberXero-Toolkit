fn main() {
    // Rebuild if the build script itself changes
    println!("cargo:rerun-if-changed=build.rs");

    // Rebuild if any resource file changes
    println!("cargo:rerun-if-changed=resources");

    // Rebuild if Cargo.toml changes (dependencies, etc.)
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Rebuild if source files change
    println!("cargo:rerun-if-changed=src");

    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "xyz.cyberxero.cyberxero-toolkit.gresource",
    );
}
