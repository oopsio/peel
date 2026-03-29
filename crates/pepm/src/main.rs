use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, anyhow, Context};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Sha256, Digest};
use tar::Archive;
use flate2::read::GzDecoder;

#[derive(Parser)]
#[command(name = "pepm", version = "0.1.0", about = "Peel Package Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package
    Install {
        name: Option<String>,
        #[arg(long)]
        version: Option<String>,
    },
    /// Initialize a new project
    Init,
    /// Add a dependency
    Add {
        name: String,
        #[arg(short, long)]
        version: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RegistryIndex {
    packages: HashMap<String, Package>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Package {
    name: String,
    versions: HashMap<String, VersionMetadata>,
    latest: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VersionMetadata {
    version: String,
    dependencies: Option<HashMap<String, String>>,
    dist: Dist,
    main: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Dist {
    tarball: String,
    shasum: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PeelProject {
    name: String,
    version: String,
    dependencies: HashMap<String, String>,
}

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/oopsio/peel-registry/main/index.json";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { name, version } => {
            if let Some(n) = name {
                install_package(&n, version).await?;
            } else {
                install_all().await?;
            }
        }
        Commands::Init => {
            init_project()?;
        }
        Commands::Add { name, version } => {
            add_dependency(&name, version).await?;
        }
    }

    Ok(())
}

async fn fetch_index() -> Result<RegistryIndex> {
    let response = reqwest::get(REGISTRY_URL).await?.json::<RegistryIndex>().await?;
    Ok(response)
}

async fn install_package(name: &str, version: Option<String>) -> Result<()> {
    let index = fetch_index().await?;
    let package = index.packages.get(name).ok_or(anyhow!("Package '{}' not found", name))?;
    
    let target_version = version.unwrap_or(package.latest.clone());
    let metadata = package.versions.get(&target_version)
        .ok_or(anyhow!("Version '{}' not found for package '{}'", target_version, name))?;

    println!("Installing {}@{}...", name, target_version);
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Downloading tarball...");
    
    let response = reqwest::get(&metadata.dist.tarball).await?;
    let bytes = response.bytes().await?;
    
    // Verify shasum
    if let Some(expected_shasum) = &metadata.dist.shasum {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        let actual_shasum = hex::encode(result);
        if actual_shasum != *expected_shasum {
            return Err(anyhow!("SHA256 mismatch for '{}': expected {}, got {}", name, expected_shasum, actual_shasum));
        }
    }

    pb.set_message("Extracting...");
    let module_dir = Path::new("peel_modules").join(name);
    fs::create_dir_all(&module_dir)?;

    let decoder = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(decoder);
    archive.unpack(&module_dir)?;

    pb.finish_with_message(format!("Installed {}@{}", name, target_version));
    
    // Install dependencies recursively
    if let Some(deps) = &metadata.dependencies {
        for (dep_name, dep_ver) in deps {
             Box::pin(install_package(dep_name, Some(dep_ver.clone()))).await?;
        }
    }

    Ok(())
}

async fn install_all() -> Result<()> {
    if !Path::new("peel.toml").exists() {
        return Err(anyhow!("No peel.toml found. Run 'pepm init' first."));
    }
    let content = fs::read_to_string("peel.toml")?;
    let project: PeelProject = toml::from_str(&content)?;
    for (name, version) in &project.dependencies {
        install_package(name, Some(version.clone())).await?;
    }
    Ok(())
}

fn init_project() -> Result<()> {
    let name = std::env::current_dir()?
        .file_name()
        .context("Failed to get current directory name")?
        .to_str()
        .context("Invalid directory name")?
        .to_string();
    let project = PeelProject {
        name,
        version: "0.1.0".to_string(),
        dependencies: HashMap::new(),
    };
    let content = toml::to_string(&project)?;
    fs::write("peel.toml", content)?;
    println!("Initialized new Peel project.");
    Ok(())
}

async fn add_dependency(name: &str, version: Option<String>) -> Result<()> {
    install_package(name, version.clone()).await?;
    
    let mut project = if Path::new("peel.toml").exists() {
        let content = fs::read_to_string("peel.toml")?;
        toml::from_str::<PeelProject>(&content)?
    } else {
        return Err(anyhow!("No peel.toml found. Run 'pepm init' first."));
    };

    let index = fetch_index().await?;
    let package = index.packages.get(name).ok_or(anyhow!("Package '{}' not found", name))?;
    let ver = version.unwrap_or(package.latest.clone());
    
    project.dependencies.insert(name.to_string(), ver);
    let content = toml::to_string(&project)?;
    fs::write("peel.toml", content)?;
    println!("Added dependency {}@{}", name, project.dependencies[name]);
    Ok(())
}
