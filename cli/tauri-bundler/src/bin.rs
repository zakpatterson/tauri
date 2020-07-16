#[cfg(windows)]
use tauri_bundler::bundle::tauri_config::get as get_tauri_config;
use tauri_bundler::{
  build_project,
  bundle::{bundle_project, check_icons, print_error, print_finished, PackageType, Settings},
};
pub use tauri_bundler::{Error, Result};

use clap::{crate_version, App, AppSettings, Arg, SubCommand};

#[cfg(windows)]
use runas::Command;
use std::env;

// Runs the CLI.
fn run() -> crate::Result<()> {
  let all_formats: Vec<&str> = PackageType::all()
    .iter()
    .map(PackageType::short_name)
    .collect();
  let m = App::new("cargo-tauri-bundler")
    .version(format!("v{}", crate_version!()).as_str())
    .bin_name("cargo")
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::SubcommandRequired)
    .subcommand(
      SubCommand::with_name("tauri-bundler")
        .author("George Burton <burtonageo@gmail.com>, Lucas Fernandes Gonçalves Nogueira <lucas@tauri.studio>, Daniel Thompson-Yvetot <denjell@sfosc.org>, Tensor Programming <tensordeveloper@gmail.com>")
        .about("Bundle Rust executables into OS bundles")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
          Arg::with_name("bin")
            .long("bin")
            .value_name("NAME")
            .help("Bundle the specified binary"),
        )
        .arg(
          Arg::with_name("example")
            .long("example")
            .value_name("NAME")
            .conflicts_with("bin")
            .help("Bundle the specified example"),
        )
        .arg(
          Arg::with_name("format")
            .long("format")
            .value_name("FORMAT")
            .possible_values(&all_formats)
            .multiple(true)
            .help("Which bundle format to produce"),
        )
        .arg(
          Arg::with_name("release")
            .long("release")
            .help("Build a bundle from a target built in release mode"),
        )
        .arg(
          Arg::with_name("target")
            .long("target")
            .value_name("TRIPLE")
            .help("Build a bundle for the target triple"),
        )
        .arg(
          Arg::with_name("features")
            .long("features")
            .value_name("FEATURES")
            .multiple(true)
            .help("Which features to build"),
        )
        .arg(
          Arg::with_name("version")
            .long("version")
            .short("v")
            .help("Read the version of the bundler"),
        ),
    )
    .get_matches();

  #[cfg(windows)]
  {
    if let Ok(tauri_config) = get_tauri_config() {
      if tauri_config.tauri.embedded_server.active {
        let exempt_output = std::process::Command::new("CheckNetIsolation")
          .args(&vec!["LoopbackExempt", "-s"])
          .output()
          .expect("failed to read LoopbackExempt -s");

        if !exempt_output.status.success() {
          panic!("Failed to execute CheckNetIsolation LoopbackExempt -s");
        }

        let output_str = String::from_utf8_lossy(&exempt_output.stdout).to_lowercase();
        if !output_str.contains("win32webviewhost_cw5n1h2txyewy") {
          println!("Running Loopback command");
          Command::new("powershell")
            .args(&vec![
              "CheckNetIsolation LoopbackExempt -a -n=\"Microsoft.Win32WebViewHost_cw5n1h2txyewy\"",
            ])
            .force_prompt(true)
            .status()
            .expect("failed to run Loopback command");
        }
      }
    }
  }

  if let Some(m) = m.subcommand_matches("tauri-bundler") {
    if m.is_present("version") {
      println!("{}", crate_version!());
    } else {
      let output_paths = env::current_dir()
        .map_err(From::from)
        .and_then(|d| Settings::new(d, m))
        .and_then(|s| {
          if check_icons(&s)? {
            build_project(&s)?;
            Ok(s)
          } else {
            Err(crate::Error::IconPathError)
          }
        })
        .and_then(bundle_project)?;
      print_finished(&output_paths)?;
    }
  }
  Ok(())
}

fn main() {
  if let Err(error) = run() {
    print_error(&error.into()).expect("Failed to call print error in main");
  }
}