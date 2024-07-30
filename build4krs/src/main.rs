use std::io::{self, Write};

#[cfg(not(target_family = "unix"))]
compile_error!("Unsupported target");

const LINKERS: [&str; 3] = ["ld", "ld.gold", "ld.lld"];

const LCS: [u8; 5] = [0, 1, 2, 3, 4];
const LPS: [u8; 5] = [0, 1, 2, 3, 4];
const PBS: [u8; 5] = [0, 1, 2, 3, 4];
const MFS: [&str; 5] = ["hc3", "hc4", "bt2", "bt3", "bt4"];

fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4 && args.len() != 5 {
        panic!("Incorrect number of arguments");
    }

    let avx512 = args.get(4).map_or(false, |arg| {
        assert_eq!(arg, "--avx512");
        true
    });

    let temp = std::process::Command::new("mktemp").arg("-d").output()?;

    assert!(temp.status.success());

    let temp = std::str::from_utf8(&temp.stdout).unwrap().trim();
    let result = build4k(
        &args[1],
        args[2].parse().unwrap(),
        args[3].parse().unwrap(),
        avx512,
        temp,
    );

    // Always delete the temp folder even if build4k fails
    std::fs::remove_dir_all(temp)?;
    result
}

fn build4k(output_path: &str, threads: usize, tt_size_mb: usize, avx512: bool, temp: &str) -> io::Result<()> {
    let avx512: &[_] = if avx512 {
        &["-d", "AVX512"]
    } else {
        &[]
    };

    // nasm
    let status = std::process::Command::new("nasm")
        .current_dir(std::fs::canonicalize("stro4k/src")?)
        .args(["-f", "elf64"])
        .args(["-d", &format!("NUM_THREADS={threads}")])
        .args(["-d", &format!("TT_SIZE_MB={tt_size_mb}")])
        .args(avx512)
        .arg("combined.asm")
        .args(["-o", &format!("{temp}/combined.o")])
        .status()?;

    assert!(status.success());

    // build sstrip
    let status = std::process::Command::new("git")
        .current_dir(temp)
        .args(["clone", "https://github.com/aunali1/super-strip"])
        .stderr(std::process::Stdio::null())
        .status()?;

    assert!(status.success());

    let status = std::process::Command::new("make")
        .current_dir(format!("{temp}/super-strip"))
        .stdout(std::process::Stdio::null())
        .status()?;

    assert!(status.success());

    // Test each combination of linker and xz args
    let mut best_size = u64::MAX;
    let mut best_binary = Vec::new();
    for linker in LINKERS {
        let status = std::process::Command::new(linker)
            .current_dir(temp)
            .arg("combined.o")
            .args(["-o", "STRO4K"])
            .args(["-m", "elf_x86_64"])
            .args([
                "--static",
                "-O3",
                "--gc-sections",
                "--as-needed",
                "--omagic",
            ])
            .status()?;

        assert!(status.success());

        let status = std::process::Command::new(format!("{temp}/super-strip/sstrip"))
            .current_dir(temp)
            .args(["-z", "STRO4K"])
            .status()?;

        assert!(status.success());

        for lc in LCS {
            for lp in LPS {
                // lc + lp cannot be greater than 4
                if lc + lp > 4 {
                    break;
                }

                for pb in PBS {
                    for mf in MFS {
                        let opts =
                            format!("--lzma2=lc={lc},lp={lp},pb={pb},mf={mf},nice=273,depth=1000");
                        let status = std::process::Command::new("xz")
                            .current_dir(temp)
                            .arg("-kze9f")
                            .args(["-F", "raw"])
                            .args(["-S", ".xz"])
                            .arg("--x86")
                            .arg(&opts)
                            .arg("STRO4K")
                            .status()?;

                        assert!(status.success());

                        let path = format!("{temp}/STRO4K.xz");
                        let size = std::fs::metadata(&path)?.len();
                        if size < best_size {
                            println!("{size} bytes [{linker}]: {opts}");

                            best_size = size;
                            best_binary = std::fs::read(path)?;
                        }
                    }
                }
            }
        }
    }

    // Write to file
    let mut output = std::fs::File::create(output_path)?;
    std::io::copy(&mut std::fs::File::open("unpack.sh")?, &mut output)?;
    output.write_all(&best_binary)?;

    println!("Final size: {} bytes", output.metadata()?.len());
    drop(output);

    // chmod
    let status = std::process::Command::new("chmod")
        .args(["+x", output_path])
        .status()?;

    assert!(status.success());
    Ok(())
}
