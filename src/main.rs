use reqwest::Client;
use serde_json::Value;
use std::{env::temp_dir, fs::File, io::Write, path::{Path, PathBuf}, process::{Command, Stdio}, thread::sleep, time::Duration};
use tokio::io::AsyncWriteExt;

fn is_vc_dll_present() -> bool {
  let system32 = std::env::var("SystemRoot").map(|root| Path::new(&root).join("System32")).unwrap_or_else(|_| PathBuf::from("C:\\Windows\\System32")); // fallback
  let dlls = ["vcruntime140.dll", "msvcp140.dll", "vcruntime140_1.dll"];
  dlls.iter().any(|dll| system32.join(dll).exists())
}

async fn download_vc_runtime(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
  let url = "https://aka.ms/vs/17/release/vc_redist.x64.exe";
  let response = reqwest::get(url).await?;
  let bytes = response.bytes().await?;
  let mut file = tokio::fs::File::create(path).await?;
  file.write_all(&bytes).await?;
  Ok(())
}

async fn install_vc_runtime(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
  let status = tokio::process::Command::new(path).args(["/quiet", "/norestart"]).stdout(Stdio::null()).stderr(Stdio::null()).status().await?;

  if !status.success() {
    return Err(format!("Installer failed with exit code: {}", status).into());
  }

  Ok(())
}

pub async fn ensure_vc_runtime() -> Result<(), Box<dyn std::error::Error>> {
  println!("Downloading VC Runtime...");
  let installer_path = temp_dir().join("vc_redist.x64.exe");

  if !installer_path.exists() {
    download_vc_runtime(&installer_path).await?;
    println!("Download complete.");
  }

  println!("Installing...");
  install_vc_runtime(&installer_path).await?;
  println!("Installation complete.");

  Ok(())
}

async fn download_msys2_installer() -> Result<PathBuf, Box<dyn std::error::Error>> {
  let repo = "msys2/msys2-installer";
  let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);

  let client = Client::new();
  let response = client.get(&api_url).header("User-Agent", "reqwest").send().await?;
  let release_info: Value = response.json().await?;

  let vec = &vec![];

  let asset = release_info["assets"]
    .as_array()
    .unwrap_or(vec)
    .iter()
    .find(|a| {
      let name = a["name"].as_str().unwrap_or("");
      name.contains("x86_64") && name.ends_with(".exe") && !name.ends_with("sfx.exe")
    })
    .ok_or("No suitable installer found")?;

  let download_url = asset["browser_download_url"].as_str().ok_or("Invalid URL")?;
  let filename = asset["name"].as_str().unwrap_or("msys2-installer.exe");

  println!("Downloading MSYS2 installer: {}", filename);

  let bytes = client.get(download_url).send().await?.bytes().await?;
  let mut file = File::create(filename)?;
  file.write_all(&bytes)?;

  Ok(PathBuf::from(filename))
}

fn install_msys2(installer_path: &Path) -> bool {
  println!("Running MSYS2 installer...");
  let status = Command::new(installer_path).args(["in", "--confirm-command", "--accept-messages", "--root", "C:/msys64"]).status();

  match status {
    Ok(s) if s.success() => {
      println!("MSYS2 installation launched.");
      true
    }
    _ => {
      println!("Failed to start MSYS2 installer.");
      false
    }
  }
}

fn wait_for_msys2_ready(msys_path: &Path) {
  let bash_path = msys_path.join("usr/bin/bash.exe");
  println!("Waiting for MSYS2 to be fully installed...");
  for _ in 0..30 {
    if bash_path.exists() {
      println!("MSYS2 is now available.");
      return;
    }
    sleep(Duration::from_secs(2));
  }
  panic!("MSYS2 did not install correctly.");
}

fn run_bash_command(cmd: &str) -> bool {
  let bash = r"C:\msys64\usr\bin\bash.exe";
  if !Path::new(bash).exists() {
    println!("Error: bash.exe not found at {}", bash);
    return false;
  }

  let status = Command::new(bash).args(["-lc", cmd]).stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status();

  match status {
    Ok(s) if s.success() => true,
    _ => {
      println!("Failed to execute: {}", cmd);
      false
    }
  }
}

fn add_to_path_env(msys_bin: &str) {
  let current_path = std::env::var("PATH").unwrap_or_default();
  if current_path.contains(msys_bin) {
    println!("MSYS2 bin already in PATH.");
    return;
  }

  let new_path = format!("{};{}", current_path, msys_bin);
  let output = Command::new("setx").args(["PATH", &new_path]).output();

  match output {
    Ok(o) if o.status.success() => println!("Successfully added to PATH."),
    _ => println!("Failed to set PATH."),
  }
}

#[tokio::main]
async fn main() {
  let msys_root = Path::new(r"C:\msys64");

  if !is_vc_dll_present() {
    ensure_vc_runtime().await.unwrap();
  }

  // If MSYS2 is not found, download + install it
  if !msys_root.exists() || !msys_root.join("usr/bin/bash.exe").exists() {
    let installer_path = download_msys2_installer().await.unwrap();
    if !install_msys2(&installer_path) {
      return;
    }

    wait_for_msys2_ready(msys_root);

    if installer_path.exists() {
      // msys2-x86_64-latest.exe
      match std::fs::remove_file(installer_path) {
        Ok(_) => println!("Deleted the installer file"),
        Err(_) => eprintln!("Couldn't delete the installer file"),
      }
    }
  }

  println!("Updating MSYS2...");
  if !run_bash_command("pacman -Syu --noconfirm") {
    return;
  }

  println!("Installing GTK4...");
  //pacman -S --noconfirm mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-glade mingw-w64-ucrt-x86_64-toolchain mingw-w64-ucrt-x86_64-pkg-config
  if !run_bash_command("pacman -S --noconfirm mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-glade mingw-w64-ucrt-x86_64-toolchain mingw-w64-ucrt-x86_64-pkg-config") {
    return;
  }

  println!("Adding ucrt64/bin to PATH...");
  add_to_path_env(r"C:\msys64\ucrt64\bin");

  println!("✅ GTK4 setup completed successfully.");
  sleep(Duration::from_secs(5));
}
