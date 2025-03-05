//! 🚀 localhost-cli - A simple tool to manage local sites 🌍
//!
//! 📌 Features:
//! - 📂 Create a new site directory inside `sites/`
//! - ⚙️ Initialize `sites/config.json` to store multiple site configurations
//! - 🗂️ Manage multiple sites effortlessly
//!
//! 🛠️ Usage examples:
//! ```sh
//! localhost-cli create mysite    # Creates the 'sites/mysite' directory
//! localhost-cli list             # Lists all configured sites
//! localhost-cli config           # Displays the config.json file
//! ```
//!
//! 🎯 Make local site management easier with **localhost-cli**! 🚀


use clap::{Parser, Subcommand, Args};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use dialoguer::{Input, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};
use tabwriter::TabWriter;
use colored::*;
use serde_json::Value;
use std::{fs, io::{self, Write}};

use std::{
    env,
    path::PathBuf,
};

/// 🚀 A simple CLI to manage local sites 🌍
#[derive(Parser)]
#[command(name = "localhost-cli")]
#[command(version = "1.0")]
#[command(about = "Create and manage local sites effortlessly", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Args)]
struct CreateArgs {
    /// Server name
    #[arg(short, long)]
    name: Option<String>,
    /// Server address
    #[arg(short, long)]
    address: Option<String>,
    /// Ports (comma-separated)
    #[arg(short, long)]
    ports: Option<String>,
    /// Enable `cgi-bin` directory
    #[arg(long)]
    cgi_bin: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    server_address: String,
    server_name: String,
    ports: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    servers: Vec<ServerConfig>,
}

#[derive(Subcommand)]
enum Commands {
    /// 📂 Create a new server configuration
    Create(CreateArgs),

    /// 📜 List all configured sites
    List,

    /// ⚙️ Show the current configuration
    Config,

    // 🗑️ Clean up config.json
    Clean,
}


fn get_config_dir() -> PathBuf {
    let home_dir = env::var("HOME").expect("❌ Failed to get home directory");
    let config_dir = PathBuf::from(format!("{}/.cargo/localhost-cli", home_dir));

    // 📂 Créer le dossier s'il n'existe pas
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("❌ Failed to create config directory");
    }

    config_dir
}

fn get_config_path() -> PathBuf {
    let mut config_path = get_config_dir();
    config_path.push("config.json");
    config_path
}


/// 📜 Save config.json
fn save_config(config: &Config) {
    let config_path = get_config_path();
    let json_data = to_string_pretty(config).expect("❌ Failed to format JSON");
    fs::write(config_path, json_data).expect("❌ Failed to write config.json");
}

/// 🎤 Prompt user input with a nice UI
fn prompt_user(prompt: &str, default: Option<&str>) -> String {
    let input = Input::new();
    
    // Initialize with prompt
    let input = input.with_prompt(prompt);
    
    // Add default value if provided
    let input = if let Some(default_value) = default {
        input.default(default_value.to_string())
    } else {
        input
    };
    
    // Get the text input
    input.interact_text().unwrap()
}

/// 🚀 Create a new server with an interactive CLI
fn create_server(args: CreateArgs) {
    let mut config = load_config();

    println!("\n🌍 Let's set up your new server!\n");

    // 🌟 Demander le nom du serveur
    let server_name = loop {
        let name = prompt_user("📛 Enter server name:", None);
        if name.is_empty() {
            println!("❌ Server name cannot be empty!");
            continue;
        }

        // 🛑 Vérifier l'unicité du nom
        if config.servers.iter().any(|s| s.server_name == name) {
            println!("⚠️ Server name `{}` is already taken. Try another!", name);
            continue;
        }

        break name;
    };

    // 🌐 Demander l'adresse IP
    let default_address = format!("127.0.0.{}", config.servers.len() + 2);
    let server_address = loop {
        let address = prompt_user("📡 Enter server IP address:", Some(&default_address));

        // 🛑 Vérifier l'unicité de l'adresse
        if config.servers.iter().any(|s| s.server_address == address) {
            println!("⚠️ IP `{}` is already assigned. Pick another!", address);
            continue;
        }

        break address;
    };

    // 🎯 Demander les ports
    let default_ports = "8080".to_string();
    let ports_input = prompt_user("🛠️ Enter ports (comma-separated):", Some(&default_ports));
    let ports: Vec<String> = ports_input.split(',').map(|p| p.trim().to_string()).collect();

    // 📂 Demander si `cgi-bin` est nécessaire
    let has_cgi_bin = Confirm::new()
        .with_prompt("📂 Do you need a `cgi-bin` folder?")
        .default(false)
        .interact()
        .unwrap();

    let server = ServerConfig {
        server_address: server_address.clone(),
        server_name: server_name.clone(),
        ports,
    };

    // 🔧 Ajouter à la configuration
    config.servers.push(server);
    save_config(&config);

    // 🌀 Barre de progression pour la création des fichiers
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} Setting up...").unwrap());
    pb.enable_steady_tick(Duration::from_millis(100));

    thread::sleep(Duration::from_secs(1));

    // 📁 Créer le dossier du site
    let mut site_path = get_config_dir();
    site_path.push("sites");
    site_path.push(&server_name);
    
    if !site_path.exists() {
        fs::create_dir_all(&site_path).expect("❌ Failed to create site directory");
    }

    // 📂 Ajouter `cgi-bin` si demandé
    if has_cgi_bin {
        let mut cgi_path = site_path.clone();
        cgi_path.push("cgi-bin");
        fs::create_dir_all(&cgi_path).expect("❌ Failed to create cgi-bin directory");
    }

    pb.finish_with_message("✅ Setup complete!");

    println!("\n🎉 Server `{}` is ready!", server_name);
    println!("📂 Directory: {}", site_path.display());
    if has_cgi_bin {
        println!("📁 `cgi-bin` folder created!");
    }
}




fn clean_json() {
    let config_path = get_config_path();
    fs::remove_file(&config_path).expect("❌ Failed to remove config.json");
    println!("✅ config.json successfully removed!");
}

/// 📜 Load config.json
fn load_config() -> Config {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).expect("❌ Failed to read config.json");
        serde_json::from_str(&content).unwrap_or_else(|_| Config { servers: vec![] })
    } else {
        Config { servers: vec![] }
    }
}

/// 📜 List all servers
fn list_servers() {
    let config = load_config();

    if config.servers.is_empty() {
        println!("⚠️ No servers found! Use `localhost-cli create <name>` to add one.");
        return;
    }

    println!("📜 Configured servers:\n");
    for (i, server) in config.servers.iter().enumerate() {
        println!(
            "🔹 {}. {} ({})\n   🔗 Ports: {}\n",
            i + 1,
            server.server_name,
            server.server_address,
            server.ports.join(", ")
        );
    }
}

/// ⚙️ Show full config.json content in a readable format
fn show_config() {
    let config_path = get_config_path();

    if !config_path.exists() {
        println!("{}", "⚠️ No configuration found! Use `localhost-cli create <name>` to add a server.".yellow());
        return;
    }

    let content = fs::read_to_string(config_path).expect("❌ Failed to read config.json");
    let json: Value = serde_json::from_str(&content).expect("❌ Failed to parse JSON");

    if let Some(servers) = json["servers"].as_array() {
        println!("\n{}", "⚙️ Current Configuration:\n".bold().blue());

        let mut tw = TabWriter::new(io::stdout()).padding(2);

        writeln!(tw, "{}\t{}\t{}\t{}", 
            "Server Name".bold().underline(), 
            "IP Address".bold().underline(), 
            "Ports".bold().underline(), 
            "CGI-Bin".bold().underline()
        ).unwrap();

        writeln!(tw, "{}", "-".repeat(50)).unwrap();

        for server in servers {
            let name = server["server_name"].as_str().unwrap_or("N/A");
            let ip = server["server_address"].as_str().unwrap_or("N/A");
            let ports = server["ports"].as_array()
                .map(|p| p.iter().map(|v| v.as_str().unwrap_or("")).collect::<Vec<_>>().join(", "))
                .unwrap_or("N/A".to_string());

            let has_cgi_bin = if server["has_cgi_bin"].as_bool().unwrap_or(false) {
                "✅ Yes".green()
            } else {
                "❌ No".red()
            };

            writeln!(tw, "{}\t{}\t{}\t{}", name, ip, ports, has_cgi_bin).unwrap();
        }

        tw.flush().unwrap();
    } else {
        println!("{}", "❌ Invalid configuration format!".red());
    }
}



fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => {
            create_server(args);
        }
        Commands::List => {
            list_servers();
        }
        Commands::Config => {
            show_config();
        },
        Commands::Clean => {
            clean_json();
        }
    }
}