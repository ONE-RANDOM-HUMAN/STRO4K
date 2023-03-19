fn main() {
    if std::env::var("CARGO_FEATURE_ASM").is_ok() {
        let out_dir = std::env::var("OUT_DIR").unwrap();

        let status = std::process::Command::new("nasm")
            .args([
                "-f",
                "elf64",
                "-g",
                "-F",
                "dwarf",
                "-d",
                "EXPORT_SYSV",
                "combined.asm",
                "-o",
                &format!("{out_dir}/combined.o"),
            ])
            .current_dir("../stro4k/src")
            .status()
            .expect("failed to run nasm");

        assert_eq!(status.code(), Some(0));

        let manifest_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/combined.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/common.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/game.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/movegen.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/search.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/uci.asm");

        println!("cargo:rustc-link-arg={out_dir}/combined.o");
    }
}
