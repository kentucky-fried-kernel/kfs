fn main() {
    println!("cargo:rerun-if-changed=src/arch/x86/linker.ld");
    println!("cargo:rerun-if-changed=src/arch/x86/i386-unknown-none.json");
}
