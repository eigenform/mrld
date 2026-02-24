
use clap::Parser;
use anyhow::{anyhow, Result};
use std::process::*;

use std::path::Path;
use std::env;

mod pxe;

/// `mrld` hacky xtask build system
#[derive(Parser)]
#[command(verbatim_doc_comment)]
enum XtaskCommand { 
    /// Build the bootloader and kernel
    Build,

    /// PXE boot into the bootloader with QEMU
    Qemu { 
        /// Enable GDB server and halt
        #[arg(long, short)]
        gdb: bool,
    },

    /// Start PXE services on the host machine
    Pxe,

    /// Run 'picocom' (for communicating with a target over /dev/ttyUSB0)
    Console,

    /// Run tests
    Test,

    /// Attach GDB to QEMU
    Gdb,
}


/// Build the UEFI bootloader
fn build_boot(root: &Path) -> Result<()> {
    let cmd = Command::new("cargo")
        .args([
            "build", 
            "--package=mrld-boot",
            "--release", 
            "-Z", "build-std=core,alloc,compiler_builtins",
            "--target=x86_64-unknown-uefi",
        ])
        .current_dir(root)
        .spawn()?
        .wait()?;
    if let Some(code) = cmd.code() {
        if code != 0 { 
            return Err(anyhow!("Bootloader build error"));
        }
    }

    Ok(())
}

/// Build the kernel
fn build_kernel(root: &Path) -> Result<()> {
    let cmd = Command::new("cargo")
        .args([
            "build", 
            "-vv",
            "--package=mrld-kernel",
            "--release", 
            "-Z", "build-std=core,alloc,compiler_builtins",
            "-Z", "build-std-features=compiler-builtins-mem",
            //"--target=x86_64-unknown-linux-gnu",
            "-Z", "json-target-spec",
            "--target=mrld-kernel.json",
        ])
        .current_dir(root)
        .spawn()?
        .wait()?;
    if let Some(code) = cmd.code() {
        if code != 0 { 
            return Err(anyhow!("Kernel build error"));
        }
    }


    let cmd = Command::new("cargo")
        .args([
            "build", 
            "--package=mrld-kernel",
            "-Z", "build-std=core,alloc,compiler_builtins",
            "-Z", "build-std-features=compiler-builtins-mem",
            //"--target=x86_64-unknown-linux-gnu",
            "-Z", "json-target-spec",
            "--target=mrld-kernel.json",
        ])
        .current_dir(root)
        .spawn()?
        .wait()?;
    if let Some(code) = cmd.code() {
        if code != 0 { 
            return Err(anyhow!("Kernel build error"));
        }
    }


    Ok(())
}

fn run_tests(root: &Path) -> Result<()> {
    let cmd = Command::new("cargo")
        .args([
            "test", 
            "--package=mrld",
            "--", "--nocapture"
        ])
        .current_dir(root)
        .spawn()?
        .wait()?;
    if let Some(code) = cmd.code() {
        if code != 0 { 
            return Err(anyhow!("Kernel build error"));
        }
    }
    Ok(())

}

fn make_symlinks(root: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;
    use std::io::ErrorKind;
    let pxe_path = root.join("pxe");

    let bootloader_path = root.join("target/x86_64-unknown-uefi/release/mrld-boot.efi");
    let bootloader_link = pxe_path.join("mrld-boot.efi");
    let kernel_path = root.join("target/mrld-kernel/release/mrld-kernel");
    let kernel_link  = pxe_path.join("mrld-kernel");

    let kernel_dbg_path = root.join("target/mrld-kernel/debug/mrld-kernel");
    let kernel_dbg_link  = pxe_path.join("mrld-kernel-debug");


    if let Err(e) = symlink(bootloader_path, bootloader_link) { 
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    };
    if let Err(e) = symlink(kernel_path, kernel_link) { 
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    };
    if let Err(e) = symlink(kernel_dbg_path, kernel_dbg_link) { 
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e.into());
        }
    };


    Ok(())
}

// NOTE: Other users might have to change this..
const OVMF_CODE: &'static str = "/usr/share/edk2-ovmf/x64/OVMF_CODE.4m.fd";
const OVMF_VARS: &'static str = "/usr/share/edk2-ovmf/x64/OVMF_VARS.4m.fd";

// FIXME: Maybe try to automatically make a symlink in pxe/
fn run_qemu(root: &Path, gdb: bool) -> Result<()> { 

    let pxe_path = root.join("pxe");

    // Warn the user if QEMU would fail to find the bootloader
    let abs_bootfile_path = pxe_path.join("mrld-boot.efi");
    if !abs_bootfile_path.exists() {
        return Err(anyhow!("Couldn't find UEFI bootloader.\n\
            Run 'cargo xtask build' before using QEMU."
        ));
    }

    let drive0 = format!("if=pflash,unit=0,format=raw,readonly=on,file={}", OVMF_CODE);
    let drive1 = format!("if=pflash,unit=1,format=raw,readonly=on,file={}", OVMF_VARS);
    let netdev = format!(
        "user,id=net0,ipv6=off,net=10.200.200.0/24,tftp={},bootfile=mrld-boot.efi",
        pxe_path.into_os_string().to_str().unwrap());

    let mut arghhhs: Vec<&str> = vec![
        "-nodefaults",
        "-nographic",
        "-vga", "virtio",
        "-accel", "kvm",
        "-cpu", "host",
        "-smp", "4",

        //"-d", "cpu", 
        //"-d", "exec",
        //"-d", "pcall",
        //"-d", "cpu_reset",
        //"-d", "int",
        //"-D", "stdio",

        "-m", "4096M",
        "-drive", drive0.as_str(),
        "-drive", drive1.as_str(),
        "-device", "virtio-net-pci,netdev=net0",
        "-netdev", netdev.as_str(),

        // Disable COM1 (0x3f8), use COM2 (0x2f8) instead
        "-serial", "none",
        "-serial", "stdio",
        //"-monitor", "stdio",
        "-boot", "n",
    ];

    if gdb { 
        arghhhs.append(&mut vec![ 
            "-gdb", "tcp::1234", "-S",
        ]);
    }

    Command::new("qemu-system-x86_64")
        .args(arghhhs)
        .current_dir(root)
        .spawn()?
        .wait()?;

    Ok(())
}

fn run_picocom() -> Result<()> {
    Command::new("picocom")
        .args(["-q", "-b", "115200", "/dev/ttyUSB0"])
        .spawn()?
        .wait()?;
    Ok(())
}

fn run_gdb() -> Result<()> { 
    let mut cmd = Command::new("gdb");
    cmd.arg("-ex").arg("set print demangle");
    //cmd.arg("-ex").arg("set arch i386:x64-32");
    cmd.arg("-ex").arg("set print asm-demangle");
    cmd.arg("-ex").arg("set schedule-multiple on");
    cmd.arg("-ex").arg("set debug remote 1");
    cmd.arg("-ex").arg("set debug remote-packet-max-chars unlimited");
    cmd.arg("-ex").arg("file target/mrld-kernel/debug/mrld-kernel");
    cmd.arg("-ex").arg("target remote localhost:1234");
    cmd.arg("-ex").arg("hb kernel_main");
    cmd.arg("-ex").arg("continue");

    cmd.status()?;

    Ok(())
}

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let cmd = XtaskCommand::parse();
    match cmd { 
        XtaskCommand::Build => { 
            build_boot(&root)?;
            build_kernel(&root)?;
            make_symlinks(&root)?;
        },
        XtaskCommand::Test => { 
            run_tests(&root)?;
        },

        XtaskCommand::Qemu { gdb } => {
            run_qemu(&root, gdb)?;
        },
        XtaskCommand::Pxe => {
            //pxe::start(&root)?;
        }
        XtaskCommand::Console => {
            run_picocom()?;
        },
        XtaskCommand::Gdb => { 
            run_gdb()?;
        },
    }
    Ok(())
}
