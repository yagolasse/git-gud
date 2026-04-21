use clap::{Parser, Subcommand};
use std::path::PathBuf;

use git_gud::services::LogService;
#[derive(Parser)]
#[command(name = "git-gud")]
#[command(about = "Git Gud - A modular Git GUI application with CLI parity", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    #[arg(short, long, global = true)]
    pub log_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Git repository
    Init {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Show repository status
    Status {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Add files to staging area
    Add {
        #[arg(required = true)]
        files: Vec<PathBuf>,
        
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Create a new commit
    Commit {
        #[arg(short, long)]
        message: String,
        
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// List or create branches
    Branch {
        name: Option<String>,
        
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    
    /// Show commit history
    Log {
        #[arg(default_value = ".")]
        path: PathBuf,
        
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Open repository in GUI mode
    Gui {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    LogService::init_with_level(level, cli.log_file.as_deref())?;
    
    log::info!("Starting Git Gud CLI");
    log::debug!("Command: {:?}", cli.command);
    
    match cli.command {
        Commands::Init { path } => {
            log::info!("Initializing repository at: {:?}", path);
            println!("Initializing repository at: {:?}", path);
            // TODO: Implement repository initialization
            Ok(())
        }
        Commands::Status { path } => {
            log::info!("Checking status of repository at: {:?}", path);
            println!("Checking status of repository at: {:?}", path);
            // TODO: Implement status checking
            Ok(())
        }
        Commands::Add { files, path } => {
            log::info!("Adding files {:?} to repository at: {:?}", files, path);
            println!("Adding files {:?} to repository at: {:?}", files, path);
            // TODO: Implement file staging
            Ok(())
        }
        Commands::Commit { message, path } => {
            log::info!("Creating commit with message '{}' at: {:?}", message, path);
            println!("Creating commit with message '{}' at: {:?}", message, path);
            // TODO: Implement commit creation
            Ok(())
        }
        Commands::Branch { name, path } => {
            if let Some(name) = name {
                log::info!("Creating branch '{}' at: {:?}", name, path);
                println!("Creating branch '{}' at: {:?}", name, path);
                // TODO: Implement branch creation
            } else {
                log::info!("Listing branches at: {:?}", path);
                println!("Listing branches at: {:?}", path);
                // TODO: Implement branch listing
            }
            Ok(())
        }
        Commands::Log { path, limit } => {
            log::info!("Showing last {} commits at: {:?}", limit, path);
            println!("Showing last {} commits at: {:?}", limit, path);
            // TODO: Implement commit log
            Ok(())
        }
        Commands::Gui { path } => {
            log::info!("Opening GUI for repository at: {:?}", path);
            println!("Opening GUI for repository at: {:?}", path);
            // Launch GUI with specified path
            crate::run_gui_with_path(Some(path))
        }
    }
}

