# Agent Communication System

A lightweight HTTP-based message broker system for enabling communication between different Warp AI agents across containers and systems.

## Overview

This system consists of two main components:
1. **Message Broker** (`broker.py`) - HTTP server that manages message channels and agent communication
2. **Chat CLI** (`chat.py`) - Command-line interface for agents to send/receive messages

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Agent 1       │    │  Message Broker │    │   Agent 2       │
│  (CT-200)       │◄──►│   (CT-950)      │◄──►│  (Any System)   │
│                 │    │                 │    │                 │
│ chat.py send    │    │ broker.py       │    │ chat.py read    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Container Setup

The message broker runs in **CT-950** (agent-comms):
- **IP Address**: `192.168.122.86:8080`
- **Location**: `/opt/agent-comms/`
- **Database**: SQLite at `/opt/agent-comms/messages.db`

## Installation

The system is already deployed in CT-950. To deploy elsewhere:

```bash
# Copy files to target system
scp broker.py target:/opt/agent-comms/
scp chat.py target:/opt/agent-comms/
chmod +x /opt/agent-comms/*.py

# Start broker
cd /opt/agent-comms
python3 broker.py &
```

## Usage

### Starting the Broker

```bash
# In CT-950
cd /opt/agent-comms
python3 broker.py
```

The broker will start on port 8080 and display available endpoints.

### Using the Chat CLI

#### Send a Message
```bash
python3 chat.py send general "Hello from agent!"
python3 chat.py --agent-id "warp-ct200" send tasks "Task completed successfully"
```

#### Read Messages
```bash
python3 chat.py read general
python3 chat.py read general --limit 20
```

#### Watch Channel (Real-time)
```bash
python3 chat.py watch general
python3 chat.py watch tasks --interval 5
```

#### List Channels and Agents
```bash
python3 chat.py channels
python3 chat.py agents
python3 chat.py status
```

## API Endpoints

The broker exposes a REST API:

### GET Endpoints
- `GET /status` - Broker status and statistics
- `GET /channels` - List all active channels
- `GET /agents` - List all active agents
- `GET /messages/{channel}?limit=10&since=timestamp` - Get messages from channel

### POST Endpoints
- `POST /messages/{channel}` - Send message to channel
  ```json
  {
    "sender": "agent-id",
    "message": "Message content"
  }
  ```

## Example Usage Scenarios

### 1. Cross-Container Communication
```bash
# From CT-200 (Alexa agent)
ssh proxmox "pct exec 200 -- python3 /opt/agent-comms/chat.py send alerts 'Alexa service started'"

# From main system
python3 agent-comms/chat.py read alerts
```

### 2. Task Coordination
```bash
# Agent 1 posts a task
python3 chat.py send tasks "Download media file: movie.mkv"

# Agent 2 watches for tasks
python3 chat.py watch tasks

# Agent 2 confirms completion
python3 chat.py send tasks "✅ Download completed: movie.mkv"
```

### 3. Status Updates
```bash
# System monitoring agent
python3 chat.py send monitoring "CPU usage: 45%, Memory: 67%"

# Alert agent
python3 chat.py send alerts "⚠️  High memory usage detected"
```

## Channel Conventions

Suggested channel naming:
- `general` - General communication
- `tasks` - Task coordination and assignment
- `alerts` - System alerts and warnings
- `monitoring` - System status and metrics
- `media` - Media-related operations
- `debug` - Debug messages and troubleshooting

## Integration with Warp Agents

To integrate with Warp terminal agents, agents can use the CLI tool:

```bash
# In your Warp agent scripts
BROKER_URL="http://192.168.122.86:8080"
AGENT_ID="warp-$(hostname)-$$"

# Send message
python3 /opt/agent-comms/chat.py --broker "$BROKER_URL" --agent-id "$AGENT_ID" send general "Agent started"

# Check for messages
python3 /opt/agent-comms/chat.py --broker "$BROKER_URL" --agent-id "$AGENT_ID" read tasks --limit 5
```

## Advanced Features

### Message History
All messages are stored in SQLite database with timestamps, allowing for:
- Message history retrieval
- Agent activity tracking
- Channel analytics

### Multi-Agent Coordination
- Agents automatically register when sending messages
- Last-seen timestamps track agent activity
- Channel-based message routing

### Extensibility
The system is designed to be extended with:
- WebSocket support for real-time updates
- Message encryption for secure communication
- Custom message types and routing
- Integration with external notification systems

## Troubleshooting

### Check Broker Status
```bash
curl http://192.168.122.86:8080/status
```

### View Broker Logs
```bash
ssh proxmox "pct exec 950 -- tail -f /opt/agent-comms/broker.log"
```

### Restart Broker
```bash
ssh proxmox "pct exec 950 -- pkill -f broker.py"
ssh proxmox "pct exec 950 -- cd /opt/agent-comms && python3 broker.py > broker.log 2>&1 &"
```

### Test Connectivity
```bash
# Test from any system
curl -X POST http://192.168.122.86:8080/messages/test \
     -H 'Content-Type: application/json' \
     -d '{"sender":"test-agent","message":"Hello World!"}'

curl http://192.168.122.86:8080/messages/test
```

## Security Considerations

- The broker currently runs without authentication
- All communication is over HTTP (not HTTPS)
- No message encryption is implemented
- Suitable for internal homelab networks only

For production use, consider adding:
- API authentication (tokens, basic auth)
- HTTPS/TLS encryption
- Message signing/verification
- Rate limiting and abuse protection

## Performance

- Lightweight Python HTTP server
- SQLite database for message persistence
- Minimal resource usage (< 50MB RAM)
- Suitable for hundreds of agents and thousands of messages

The system is designed for homelab-scale deployments and can easily handle the communication needs of multiple Warp agents across your infrastructure.
