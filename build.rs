fn main() {
    let dependencies = ["src/arch/x86/linker.ld", "src/arch/x86/i386-unknown-none.json"];
    for dep in dependencies {
        println!("cargo:rerun-if-changed={dep}");
    }
}
