fn main() {
    cc::Build::new()
        .file("src/init.S")
        .compile("asm");
}