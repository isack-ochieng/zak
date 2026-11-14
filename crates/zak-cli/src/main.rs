use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use zakc::{check_source, compile_source};

fn main() -> ExitCode {
    match execute(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("zak: {}", message);
            ExitCode::FAILURE
        }
    }
}

fn execute(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };

    match command {
        "new" => command_new(&args[1..]),
        "check" => command_check(&args[1..]),
        "build" => command_build(&args[1..]).map(|_| ()),
        "run" => command_run(&args[1..]),
        "help" | "--help" | "-h" => { print_help(); Ok(()) }
        "version" | "--version" | "-V" => {
            println!("zak {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        other => Err(format!("unknown command '{}'. Run 'zak help' to see available commands.", other)),
    }
}

fn command_new(args: &[String]) -> Result<(), String> {
    let name = args.first().ok_or_else(|| "usage: zak new <project-name> [directory]".to_string())?;
    if !is_valid_project_name(name) {
        return Err("project name must contain only letters, numbers, '-' or '_'".to_string());
    }
    let destination = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(name));
    if destination.exists() {
        return Err(format!("destination '{}' already exists", destination.display()));
    }

    fs::create_dir_all(destination.join("src"))
        .map_err(|error| format!("could not create project directory: {}", error))?;

    let manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[build]\ntarget = \"host\"\n",
        name
    );
    let source = r#"fn main() -> Int {
    print("Hello from ZAK");
    return 0;
}
"#;

    fs::write(destination.join("zak.toml"), manifest)
        .map_err(|error| format!("could not write zak.toml: {}", error))?;
    fs::write(destination.join("src").join("main.zak"), source)
        .map_err(|error| format!("could not write src/main.zak: {}", error))?;

    println!("Created ZAK project '{}'", name);
    println!("  {}", destination.join("zak.toml").display());
    println!("  {}", destination.join("src/main.zak").display());
    println!("\nNext steps:");
    println!("  cd {}", destination.display());
    println!("  zak check");
    println!("  zak run");
    Ok(())
}

fn command_check(args: &[String]) -> Result<(), String> {
    let root = project_root(args)?;
    let path = source_path(&root);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {}", path.display(), error))?;
    check_source(&source).map_err(format_diagnostics)?;
    println!("{}: no errors found", path.display());
    Ok(())
}

fn command_build(args: &[String]) -> Result<PathBuf, String> {
    let root = project_root(args)?;
    ensure_host_target(args)?;
    let path = source_path(&root);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {}", path.display(), error))?;

    let llvm_ir = compile_source(&source).map_err(format_diagnostics)?;
    let build_dir = root.join("build");
    fs::create_dir_all(&build_dir)
        .map_err(|error| format!("could not create build directory: {}", error))?;

    let ir_path = build_dir.join("main.ll");
    fs::write(&ir_path, llvm_ir)
        .map_err(|error| format!("could not write LLVM IR: {}", error))?;

    let project_name = read_project_name(&root);
    let executable = executable_path(&build_dir, &project_name);
    let output = Command::new("clang")
        .arg("-Wno-override-module")
        .arg(&ir_path)
        .arg("-o")
        .arg(&executable)
        .output()
        .map_err(|error| format!("could not execute clang: {}. Install LLVM/Clang and ensure 'clang' is on PATH.", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LLVM/Clang build failed:\n{}", stderr.trim()));
    }
    println!("Built {}", executable.display());
    println!("LLVM IR: {}", ir_path.display());
    Ok(executable)
}

fn command_run(args: &[String]) -> Result<(), String> {
    let executable = command_build(args)?;
    let status = Command::new(&executable)
        .status()
        .map_err(|error| format!("could not run {}: {}", executable.display(), error))?;
    if !status.success() {
        return Err(format!(
            "program exited with status {}",
            status.code().map_or_else(
                || "terminated by signal".to_string(),
                |code| code.to_string()
            )
        ));
    }
    Ok(())
}

fn project_root(args: &[String]) -> Result<PathBuf, String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--project" {
            let value = args.get(index + 1)
                .ok_or_else(|| "'--project' requires a directory".to_string())?;
            return Ok(PathBuf::from(value));
        }
        index += 1;
    }
    env::current_dir().map_err(|error| format!("could not determine current directory: {}", error))
}

fn ensure_host_target(args: &[String]) -> Result<(), String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--target" {
            let target = args.get(index + 1)
                .ok_or_else(|| "'--target' requires a value".to_string())?;
            if target != "host" {
                return Err(format!("target '{}' is not implemented yet; ZAK 0.1 supports --target host", target));
            }
        }
        index += 1;
    }
    Ok(())
}

fn source_path(root: &Path) -> PathBuf {
    let conventional = root.join("src").join("main.zak");
    if conventional.exists() { conventional } else { root.join("main.zak") }
}

fn read_project_name(root: &Path) -> String {
    if let Ok(contents) = fs::read_to_string(root.join("zak.toml")) {
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "name" {
                    return value.trim().trim_matches('"').to_string();
                }
            }
        }
    }
    "zak-app".to_string()
}

fn executable_path(build_dir: &Path, name: &str) -> PathBuf {
    if cfg!(windows) { build_dir.join(format!("{}.exe", name)) }
    else { build_dir.join(name) }
}

fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|character| {
        character.is_ascii_alphanumeric() || character == '-' || character == '_'
    })
}

fn format_diagnostics(errors: Vec<zakc::diagnostic::Diagnostic>) -> String {
    errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
}

fn print_help() {
    println!(
        "ZAK — native-first application programming language\n\
         \n\
         Usage:\n\
           zak new <name> [directory]  Create a project\n\
           zak check [--project DIR]   Parse and type-check the project\n\
           zak build [--target host]   Compile to a native host executable\n\
           zak run [--target host]     Build and run the project\n\
           zak version                 Print the compiler version\n\
           zak help                    Show this help\n\
         \n\
         ZAK 0.1 currently targets the machine running the compiler.\n\
         Android, iOS, macOS, Windows and Linux adapters are planned."
    );
}
