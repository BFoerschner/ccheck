use clap::Parser;
use colored::*;
use std::{
    fs,
    process::{self, Command, Output},
};

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

fn main() {
    let args = Args::parse();
    let commands = parse_commands(args.file, args.commands);

    println!("Checking Docker image: {}", args.image.to_string().yellow());

    let script_body = generate_script_body(&args.shell, commands);
    let script = match args.shell.as_str() {
        "nu" | "nushell" => script_body,
        _ => format!("#!/bin/sh\n{script_body}"),
    };

    let docker_script_output = build_docker_cmd(&args.image, &args.shell)
        .arg("-c")
        .arg(&script)
        .output();

    match docker_script_output {
        Ok(output) => handle_docker_output(output),
        Err(err) => {
            eprintln!("Failed to run Docker script: {err}");
            process::exit(1);
        }
    }
}

fn generate_script_body(shell: &str, commands: Vec<String>) -> String {
    let script_body = commands
        .iter()
        .map(|cmd| create_availability_check_script(shell, cmd))
        .collect::<Vec<_>>()
        .join("\n");
    script_body
}

fn parse_commands(command_file_path: Option<String>, args_commands: Vec<String>) -> Vec<String> {
    match command_file_path {
        Some(path) => match fs::read_to_string(path) {
            Ok(contents) => create_commands_string(contents),
            Err(err) => {
                eprintln!("Failed to read file: {err}");
                process::exit(1);
            }
        },
        None => args_commands,
    }
}

fn create_commands_string(contents: String) -> Vec<String> {
    contents
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|l| l.trim().to_string())
        .collect()
}

fn handle_docker_output(docker_script_output: Output) {
    if !docker_script_output.status.success() {
        eprintln!(
            "Docker script failed with status code: {}",
            docker_script_output.status
        );
        process::exit(1);
    }

    let mut installed = vec![];
    let mut missing = vec![];
    for line in String::from_utf8_lossy(&docker_script_output.stdout).lines() {
        let line = line.trim();
        if line.contains("not installed") {
            missing.push(line.to_string());
        } else if line.contains("installed") {
            installed.push(line.to_string());
        }
    }
    log_summary(installed, missing)
}

fn create_availability_check_script(shell: &str, cmd: &str) -> String {
    match shell {
        "nu" | "nushell" => get_nu_script(cmd),
        _ => get_posix_script(cmd),
    }
    .replace("{cmd}", cmd)
}

fn get_posix_script(cmd: &str) -> String {
    r#"
        if command -v "{cmd}" >/dev/null 2>&1; then
            echo "{cmd} installed"
        else
            echo "{cmd} not installed"
        fi
    "#
    .replace("{cmd}", cmd)
}

fn get_nu_script(cmd: &str) -> String {
    r#"
        if (try { which {cmd} } catch { null }) != null {
            print "{cmd} installed"
        } else {
            print "{cmd} not installed"
        }
    "#
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

fn log_summary(installed: Vec<String>, missing: Vec<String>) {
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
    } else {
        println!("{}", "The following commands are missing:\n".red());
        for cmd in &missing {
            println!("{}", cmd.replace("not installed", "").red());
        }
        println!("{summary}");
        eprintln!("{} command(s) missing", missing.len());
        process::exit(1);
    }
}
