use log::{info, warn};
use rand::Rng;
use std::time::Instant;

/// Checks for common virtualization artifacts and sandbox indicators.
pub fn detect_suspicious_env() -> bool {
    let mut rng = rand::thread_rng();
    
    #[cfg(target_os = "linux")]
    {
        // Check for Docker/QEMU specific hostname patterns
        info!("Running on Linux - checking hardware identifiers...");
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| String::from("unknown"));
        if hostname.contains("docker") || hostname.contains("vagrant") || hostname.contains("qemu") {
            warn!("Containerized/VM environment detected. Exiting.");
            return true;
        }
    }

    // Jitter
    std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(100..500)));
    false
}

/// Implements "Ekko"-style sleep obfuscation.
pub fn stealth_sleep(duration_secs: u64) {
    info!("Initiating stealth sleep for {} seconds...", duration_secs);
    
    let start = Instant::now();
    let end = start + std::time::Duration::from_secs(duration_secs);
    let mut counter: u64 = 0;

    while Instant::now() < end {
        counter = counter.wrapping_add(counter ^ 0x5a5a5a5a);
        if counter % 10000 == 0 {
            // Optional network noise could go here
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    
    info!("Stealth sleep complete. Counter state: {}", counter);
}
