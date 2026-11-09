fn main() {
    // ESP32 memory layout and runtime linker script.
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
