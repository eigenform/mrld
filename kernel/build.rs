
fn main() {
    // Force rebuild when the linkerscript changes
    println!("cargo:rerun-if-changed=mrld-kernel.ld");
}
