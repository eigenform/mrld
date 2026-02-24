

use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

fn main() {

    let mut root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    println!("{:?}", root);
    let out_dir = root.join("target/mrld-kernel/");
    let input = root.join("kernel/src/trampoline.S");
    let ld = root.join("trampoline.ld");

    let obj_out = root.join("target/mrld-kernel/trampoline.o");
    let elf_out = root.join("target/mrld-kernel/trampoline.elf");
    let bin_out = root.join("target/mrld-kernel/trampoline.bin");

    let mut cmd = Command::new("as")
        .arg(&input)
        .arg("-o").arg(&obj_out)
        .status().unwrap();
    if let Some(code) = cmd.code() { 
        if code != 0 { panic!("failed to assemble trampoline?"); }
    }

    let mut cmd = Command::new("ld")
        .arg("-o").arg(&elf_out)
        .arg("-T").arg(&ld)
        //.arg("-b").arg("elf32-i386")
        .arg(&obj_out)
        .status().unwrap();
    if let Some(code) = cmd.code() { 
        if code != 0 { panic!("failed to link trampoline?"); }
    }


    let cmd = Command::new("objcopy")
        .arg("-O").arg("binary")
        .arg("-j").arg(".text")
        .arg("-j").arg(".data")
        .arg("--pad-to").arg("0x9000")
        .arg(&elf_out)
        .arg(&bin_out)
        .status().unwrap();
    if let Some(code) = cmd.code() { 
        if code != 0 { panic!("failed to copy trampoline?"); }
    }

    // Force rebuild when linkerscripts change
    println!("cargo:rerun-if-changed=mrld-kernel.ld");
    println!("cargo:rerun-if-changed=trampoline.ld");

    // Force rebuild if the trampoline changes
    println!("cargo:rerun-if-changed=kernel/src/trampoline.S");


}
