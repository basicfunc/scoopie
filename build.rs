fn main() {
    // Check if target OS is other than Windows
    if !cfg!(target_os = "windows") {
        // Display an error message during the build
        println!("cargo:warning=This project can only be built on Windows.");
        std::process::exit(1);
    }
}
