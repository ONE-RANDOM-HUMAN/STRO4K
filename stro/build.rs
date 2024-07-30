fn main() {
    if std::env::var("CARGO_FEATURE_ASM").is_ok() {
        let out_dir = std::env::var("OUT_DIR").unwrap();

        // We assume that we are not running on KNL/KNM, so we always have AVX-512 VL/DQ/BW
        let avx512 = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap().contains("avx512");
        let avx512: &[_] = if avx512 {
            &["-d", "AVX512"]
        } else {
            &[]
        };


        let status = std::process::Command::new("nasm")
            .args(["-f", "elf64"])
            .arg("-g")
            .args(["-F", "dwarf"])
            .args(["-d", "EXPORT_SYSV"])
            .args(["-d", "NUM_THREADS=1"])
            .args(avx512)
            .arg("combined.asm")
            .args(["-o", &format!("{out_dir}/combined.o")])
            .current_dir("../stro4k/src")
            .status()
            .expect("failed to run nasm");

        assert_eq!(status.code(), Some(0));

        let manifest_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/combined.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/common.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/evaluate.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/game.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/movegen.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/search.asm");
        println!("cargo:rerun-if-changed={manifest_path}/../stro4k/src/uci.asm");

        println!("cargo:rustc-link-arg={out_dir}/combined.o");
    }
}
