use anyhow::Result;
use clap::Parser;
use colored::*;
use std::{fs, process::Command};

#[derive(Parser)]
#[command(about = "Check if commands are installed inside a Docker Image")]
struct Args {
    image: String,
    commands: Vec<String>,
    #[arg(short, long)]
    file: Option<String>,
    #[arg(short, long, default_value = "zsh")]
    shell: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let commands = match args.file {
        Some(path) => match fs::read_to_string(path) {
            Ok(contents) => contents
                .lines()
                // filter out comments and empty lines
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .map(|l| l.trim().to_string())
                .collect(),
            Err(err) => {
                eprintln!("{err}");
                anyhow::bail!("Failed to read file");
            }
        },
        None => args.commands,
    };

    if commands.is_empty() {
        anyhow::bail!("No commands specified");
    }

    println!("Checking Docker image: {}", args.image.to_string().yellow());
    println!(
        "Checking {} command(s):\n",
        commands.len().to_string().yellow()
    );

    // create a joint script to run all commands at once in the container
    let script_body = commands
        .iter()
        .map(|cmd| create_availability_check_script(&args.shell, cmd))
        .collect::<Vec<_>>()
        .join("\n");

    let script = match args.shell.as_str() {
        "nu" | "nushell" => script_body,
        _ => format!("#!/bin/sh\n{script_body}"),
    };

    let docker_script_output = build_docker_cmd(&args.image, &args.shell)
        .arg("-c")
        .arg(&script)
        .output();

    match docker_script_output {
        Ok(output) => {
            let mut installed = vec![];
            let mut missing = vec![];

            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let line = line.trim();
                if line.contains("not installed") {
                    missing.push(line.to_string());
                } else if line.contains("installed") {
                    installed.push(line.to_string());
                } else if !line.is_empty() {
                    eprintln!("{line}");
                }
            }
            create_summary(installed, missing)
        }
        Err(err) => {
            println!("{err}");
            anyhow::bail!("Failed to run script in container");
        }
    }
}

fn create_availability_check_script(shell: &str, cmd: &str) -> String {
    let nu_script = r#"
        if (try { which {cmd} } catch { null }) != null {
            print "{cmd} installed"
        } else {
            print "{cmd} not installed"
        }
    "#;

    let posix_script = r#"
        if command -v "{cmd}" >/dev/null 2>&1; then
            echo "{cmd} installed"
        else
            echo "{cmd} not installed"
        fi
    "#;

    match shell {
        "nu" | "nushell" => nu_script,
        _ => posix_script,
    }
    .replace("{cmd}", cmd)
}

fn build_docker_cmd(image: &str, shell: &str) -> Command {
    let mut docker_cmd = Command::new("docker");
    docker_cmd
        .arg("run")
        .arg("--rm")
        .arg("--entrypoint=")
        .arg(image) 
        .arg(shell);

    if shell == "zsh" {
        docker_cmd.arg("-i"); // Necessary for it to load .zshrc
    }

    docker_cmd
}

fn create_summary(installed: Vec<String>, missing: Vec<String>) -> Result<()> {
    let summary = format!(
        "\nSummary: {}/{} commands installed",
        installed.len(),
        installed.len() + missing.len()
    )
    .yellow();

    if missing.is_empty() {
        println!("{}", "All commands are installed!".green());
        for cmd in installed {
            println!("{}", cmd.green());
        }
        println!("{summary}");
        Ok(())
    } else {
        println!("{}", "The following commands are missing:\n".red());
        for cmd in &missing {
            println!("{}", cmd.replace("not installed", "").red());
        }
        println!("{summary}");
        anyhow::bail!("{} command(s) missing", missing.len());
    }
}
