use crate::KaniOptions;
use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use std::io::{ErrorKind};

use anyhow::bail;
use chrono::prelude::*;

//Verifies Kani is installed and launches it
pub(crate) fn launch_kani(cli: KaniOptions) -> anyhow::Result<()> {
    //Check Kani is installed
    match Command::new("cargo").args(["kani","--version"]).spawn() {
        Ok(_) => println!("Kani installed!"),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("error: {}",e.kind());
                bail!("Kani not installed!")
            } else {
                println!("error: {}",e.kind());
                println!("Unknown error")
            }
        }
    };


    //Log Date
    let date = Local::now().format("%Y-%m-%d-%H:%M:%S").to_string();

    //Check log file exists
    if !Path::new("./kani_test/log/").exists() {
        fs::create_dir_all("./kani_test/log/")?;
    }

    let dir_name = String::from(format!("./kani_test/log/{}/", date));
    fs::create_dir_all(&dir_name)?;

    let dir = Path::new(&dir_name);
    let _ = env::set_current_dir(&dir);

    //Log Format
    let log_name = format!("{}.log", date);
    let log = File::create(log_name).expect("failed to open log");

    //Actually compose the command
    let mut cmd = Command::new("../../../kani/scripts/cargo-kani");
//    let mut cmd = Command::new("cargo");
//    cmd.arg("kani");
    cmd.stdout(log);

    //Pass any desired environment variables
    // cmd.envs(env::vars());
    cmd.args(kernel_flags());

    //Add kani args
    if let Some(args) = cli.kani_options {
        cmd.arg(args);
    }

    //Exclude packages that make kani crash
    cmd.args(exclude_list());

    //Capture CBMC options
    if let Some(args) = cli.cbmc_options {
        cmd.args(cbmc_flags());
        cmd.arg(args);
    }

    println!("KANI CMD:{}", (pretty_cmd(&cmd)));
    if true == cli.print_kani_argument {
        println!("KANI CMD:{}", (pretty_cmd(&cmd)));
    }

    let status = cmd.status()?;
    if !status.success() {
        // if status.exit_ok().is_ok(){
        // }
        // anyhow::bail!("Failed to run Kani: {}", pretty_cmd(&cmd));
    }

    Ok(())

    // match cmd.spawn() {
    //     Err(e) => {
    //         return Err(e.into());
    //     }
    //     Ok(_v) => {
    //         return Ok(());
    //     }
    // }
}

fn pretty_cmd(cmd: &Command) -> String {
    format!(
        "{} {:?}",
        cmd.get_envs()
            .map(|(key, val)| format!("{:?}={:?}", key, val))
            .fold(String::new(), |a, b| a + &b),
        cmd
    )
}

pub fn kernel_flags() -> Vec<String> {
    let mut flags: Vec<_> = Vec::new();

    flags.extend_from_slice(
        &[
            //"--output-format",
            //"terse",
            "--enable-unstable",
            "--ignore-global-asm",
            "-Zstubbing",
        ]
        .map(String::from)
        .to_vec(),
    );

    flags
}

pub fn cbmc_flags() -> Vec<String> {
    let mut flags: Vec<_> = Vec::new();

    flags.extend_from_slice(
        &[
            "--cbmc-args",
            // "--show-properties"
        ]
        .map(String::from)
        .to_vec(),
    );

    flags
}

pub fn exclude_list() -> Vec<String> {
    let mut exclude_packages: Vec<_> = Vec::new();

    exclude_packages.extend_from_slice(
        &[
            "--workspace",
            "--exclude",
            "monitor",
            "unicode-bidi", // "twizzler-abi"
        ]
        .map(String::from)
        .to_vec(),
    );

    exclude_packages
}
