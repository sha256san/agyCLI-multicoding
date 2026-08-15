//! Multi-Agent Development Orchestrator (`mag`) CLI binary.

use clap::{Parser, Subcommand};
use mag_config::ProjectConfig;
use mag_git::GitManager;
use mag_manager::{EnvDoctor, Orchestrator};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mag",
    about = "Multi-Agent Software Development Orchestrator",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Natural language development requirement prompt
    #[arg(trailing_var_arg = true)]
    prompt_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new mag multi-agent project
    Init {
        #[arg(default_value = "my-agent-project")]
        name: String,
    },
    /// Show status of orchestrator, agents, and tasks
    Status,
    /// Run system diagnostics (envdoctor)
    Doctor,
    /// Task operations
    Task {
        #[command(subcommand)]
        action: TaskCommands,
    },
    /// Run autonomous multi-agent task workflow
    Run {
        prompt: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List all tasks
    List,
    /// Show details for a specific task
    Show {
        task_id: String,
    },
}

fn print_banner() {
    println!(
        r#"
  __  __         _ _   _              _                    _   
 |  \/  |_  _ __| | |_(_)___ __ _ _ _ (_)___  _ __  __ _ __| |  
 | |\/| | || / _| |  _| / _ \ _` | '_/| (_-< | '  \/ _` / _` |_ 
 |_|  |_|\_,_\__|_|\__|_\___/\__,_|_|  |_/__/ |_|_|_\__,_\__, (_)
                                                          |__/  
 Multi-Agent Software Development Orchestrator (`mag` - Rust Native)
    "#
    );
}

fn find_project_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if current.join(".mag").exists()
            || current.join("Cargo.toml").exists()
            || current.join("mddir").exists()
        {
            return current;
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root_path = find_project_root();
    let db_path = root_path.join(".mag/database.sqlite");

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Init { name } => {
                println!("[*] Initializing Multi-Agent project '{}'...", name);
                std::fs::create_dir_all(".mag/agents")?;
                std::fs::create_dir_all(".mag/logs")?;
                std::fs::create_dir_all("src")?;
                std::fs::create_dir_all("tests")?;
                std::fs::create_dir_all("docs")?;

                let cfg = ProjectConfig::default_project(&name);
                cfg.save_to_file(".mag/config.toml")?;

                let git = GitManager::new(&root_path);
                git.init_repo()?;

                println!("[✓] Project '{}' initialized successfully!", name);
                println!("    - Config: .mag/config.toml");
                println!("    - Git: Initialized");
                println!("    - Database: .mag/database.sqlite");
            }
            Commands::Status => {
                print_banner();
                let orch = Orchestrator::new(&root_path, &db_path)?;
                println!("Manager Status: RUNNING");
                println!("Database:       {:?}\n", db_path);

                println!("{:-<65}", "");
                println!("{:<10} {:<15} {:<10} {:<25}", "AGENT ID", "ROLE", "PORT", "STATUS");
                println!("{:-<65}", "");
                for (role, ep) in &orch.config.agents {
                    println!("{:<10} {:<15} {:<10} [READY]", ep.id, role, ep.port);
                }
                println!("{:-<65}", "");

                println!("\nRecent Tasks:");
                let tasks = orch.storage.list_tasks()?;
                if tasks.is_empty() {
                    println!("  (No tasks found in database)");
                } else {
                    for t in tasks.iter().rev().take(10) {
                        println!("  - [{}] {:<12} | {}", t.id, t.status, t.title);
                    }
                }
            }
            Commands::Doctor => {
                println!("[*] Running EnvDoctor system diagnostics...");
                let report = EnvDoctor::diagnose();
                println!("  Diagnostics Report:");
                for (tool, installed) in &report.tools {
                    let mark = if *installed { "[✓] Available" } else { "[✗] Missing" };
                    println!("    - {:<10}: {}", tool, mark);
                }
                if report.is_healthy {
                    println!("\n[✓] Environment check passed! Core development tools are ready.");
                } else {
                    println!("\n[!] Issues: {}", report.issues.join(", "));
                }
            }
            Commands::Task { action } => {
                let orch = Orchestrator::new(&root_path, &db_path)?;
                match action {
                    TaskCommands::List => {
                        let tasks = orch.storage.list_tasks()?;
                        println!("{:<10} {:<15} {:<10} {:<30}", "TASK ID", "STATUS", "AGENT", "TITLE");
                        println!("{:-<65}", "");
                        for t in tasks {
                            println!("{:<10} {:<15} {:<10} {:<30}", t.id, t.status, t.assigned_agent, t.title);
                        }
                    }
                    TaskCommands::Show { task_id } => {
                        if let Some(t) = orch.storage.get_task(&task_id)? {
                            println!("Task ID:     {}", t.id);
                            println!("Title:       {}", t.title);
                            println!("Description: {}", t.description);
                            println!("Agent:       {} ({})", t.assigned_agent, t.role);
                            println!("Status:      {}", t.status);
                            println!("Retries:     {}/{}", t.retry_count, t.max_retries);
                            if let Some(res) = t.result {
                                println!("Result:      {} - {}", res.status, res.summary);
                            }
                        } else {
                            println!("Task '{}' not found.", task_id);
                        }
                    }
                }
            }
            Commands::Run { prompt } => {
                run_workflow(&prompt, &root_path, &db_path)?;
            }
        }
    } else if !cli.prompt_args.is_empty() {
        let prompt = cli.prompt_args.join(" ");
        run_workflow(&prompt, &root_path, &db_path)?;
    } else {
        print_banner();
        println!("Usage: mag <command> | mag \"<requirement prompt>\"\n");
        println!("Commands:");
        println!("  init    Initialize a new multi-agent project");
        println!("  status  Show orchestrator and agent status");
        println!("  doctor  Run system diagnostics");
        println!("  task    Manage tasks");
    }

    Ok(())
}

fn run_workflow(prompt: &str, root_path: &PathBuf, db_path: &PathBuf) -> anyhow::Result<()> {
    print_banner();
    println!("[*] Received instruction: \"{}\"\n", prompt);

    let orch = Orchestrator::new(root_path, db_path)?;

    println!("[*] Manager: Decomposing requirement into 5-Agent DAG...");
    let tasks = orch.decompose_requirement(prompt)?;
    for t in &tasks {
        let deps = if t.dependencies.is_empty() { "root".into() } else { t.dependencies.join(", ") };
        println!("    - [{}] {:<12} -> Assigned to: {} (depends on: {})", t.id, t.role, t.assigned_agent, deps);
    }

    println!("\n[*] Executing Autonomous Multi-Agent Orchestration Loop...\n");
    let success = orch.run_orchestration_loop(20)?;

    println!("\n{:=<70}", "");
    if success {
        println!(" [✓] MULTI-AGENT WORKFLOW COMPLETED SUCCESSFULLY!");
    } else {
        println!(" [!] MULTI-AGENT WORKFLOW FINISHED WITH SOME FAILED TASKS");
    }
    println!("{:=<70}", "");

    let final_tasks = orch.storage.list_tasks()?;
    for t in final_tasks.iter().rev().take(tasks.len()) {
        let summary = t.result.as_ref().map(|r| r.summary.as_str()).unwrap_or("No result");
        println!("  • [{}] {:<12} | {}", t.id, t.status, summary);
    }
    println!("{:=<70}", "");

    Ok(())
}
