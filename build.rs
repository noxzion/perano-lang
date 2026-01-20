fn main() {
    cc::Build::new()
        .file("src/nvm/gen.c")
        .flag_if_supported("-Wno-unused-result")
        .compile("nvmcgen");
}
