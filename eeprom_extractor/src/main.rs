use anyhow::{bail, Context, Result};
use clap::Parser;
use goblin::elf::Elf;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "Extract the entire .eeprom_data section into a single file")]
struct Args {
    /// Path to the compiled ELF binary
    #[arg(long, env = "EEPROM_BIN")]
    bin: PathBuf,

    /// Output file path for the section's raw contents
    #[arg(long, env = "EEPROM_OUT", default_value = "eeprom.eep")]
    out_file: PathBuf,

    /// Optional: explicit section name
    #[arg(long, env = "EEPROM_SECTION", default_value = ".eeprom_data")]
    section: String,

    /// If set, strip the section from the binary in-place using objcopy
    #[arg(long, env = "EEPROM_STRIP", default_value_t = true)]
    strip: bool,

    /// Path to objcopy (gnu-objcopy or llvm-objcopy) if `--strip` is used
    #[arg(long, env = "OBJCOPY", default_value = "objcopy")]
    objcopy: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut data = Vec::new();
    File::open(&args.bin)
        .with_context(|| format!("open {}", args.bin.display()))?
        .read_to_end(&mut data)?;

    let elf = Elf::parse(&data)?;

    // Locate the target section
    let (_sec_idx, sec) = elf
        .section_headers
        .iter()
        .enumerate()
        .find_map(|(i, sh)| {
            let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("");
            if name == args.section { Some((i, sh)) } else { None }
        })
        .with_context(|| format!("{} section not found", args.section))?
    let sec_start = sec.sh_offset as usize;
    let sec_end = (sec.sh_offset + sec.sh_size) as usize;
    if sec_end > data.len() {
        bail!("section bounds exceed file length");
    }

    let bytes = &data[sec_start..sec_end];
    let mut out = File::create(&args.out_file)?;
    out.write_all(bytes)?;

    if args.strip {
        let status = std::process::Command::new(&args.objcopy)
            .arg("--remove-section").arg(&args.section)
            .arg(&args.bin)
            .status()
            .context("objcopy invocation failed")?;
        if !status.success() {
            bail!("objcopy returned non-zero status");
        }
    }

    Ok(())
}
