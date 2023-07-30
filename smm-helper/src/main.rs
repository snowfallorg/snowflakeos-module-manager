use anyhow::{anyhow, Result};
use clap::{self, FromArgMatches, Subcommand};
use std::{
    fs::{File, self},
    io::Write,
    process::Command,
};

#[derive(Subcommand, Debug)]
enum SubCommands {
    Rebuild {
        /// Run `nixos-rebuild` with the given arguments
        arguments: Vec<String>,
        /// How many generations to keep
        #[arg(short, long)]
        generations: Option<u32>,
    },
    WriteRebuild {
        /// Content to write to file
        #[arg(short, long)]
        content: String,
        /// Write config to file in path output
        #[arg(short, long)]
        path: String,
        /// Run `nixos-rebuild` with the given arguments
        arguments: Vec<String>,
        /// How many generations to keep
        #[arg(short, long)]
        generations: Option<u32>,
    },
}

fn main() {
    let cli = SubCommands::augment_subcommands(clap::Command::new(
        "Helper binary for SnowflakeOS Module Manager",
    ));
    let matches = cli.get_matches();
    let derived_subcommands = SubCommands::from_arg_matches(&matches)
        .map_err(|err| err.exit())
        .unwrap();

    if users::get_effective_uid() != 0 {
        eprintln!("snn-helper must be run as root");
        std::process::exit(1);
    }

    match derived_subcommands {
        SubCommands::Rebuild { arguments, generations } => match rebuild(arguments, generations) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
        SubCommands::WriteRebuild {
            content,
            path,
            arguments,
            generations
        } => {
            match write_file(&content, &path, arguments, generations) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            };
        }
    }
}

fn write_file(content: &str, path: &str, args: Vec<String>, generations: Option<u32>) -> Result<()> {
    let backup = fs::read_to_string(path)?;

    let mut file = File::create(path)?;
    write!(file, "{}", content)?;

    if rebuild(args, generations).is_err() {
        let mut file = File::create(path)?;
        write!(file, "{}", &backup)?;
        Err(anyhow!("Failed to rebuild"))
    } else {
        Ok(())
    }
}

fn rebuild(args: Vec<String>, generations: Option<u32>) -> Result<()> {
    let mut cmd = Command::new("nixos-rebuild").args(args).spawn()?;
    let x = cmd.wait()?;
    if !x.success() {
        return Err(anyhow!("nixos-rebuild failed with exit code {}", x.code().unwrap()));
    }
    if let Some(g) = generations {
        if g > 0 {
            let mut cmd = Command::new("nix-env")
                .arg("--delete-generations")
                .arg("-p")
                .arg("/nix/var/nix/profiles/system")
                .arg(&format!("+{}", g))
                .spawn()?;
            let x = cmd.wait()?;
            if !x.success() {
                return Err(anyhow!(
                    "nix-env --delete-generations failed with exit code {}",
                    x.code().unwrap()
                ));
            }
        }
    }
    Ok(())
}
