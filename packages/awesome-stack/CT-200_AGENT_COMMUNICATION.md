# CT-200 Agent Communication Setup

## Overview

CT-200, also known as the Alexa/desktop container, has been set up with a custom inter-agent communication system using a simple, HTTP-based message broker. This system enables seamless communication between different Warp AI agents across various containers and systems, circumventing the current network restrictions that prevent connectivity to Warp's authentication servers.

## Components

1. **Message Broker**
   - **Location**: CT-950 (agent-comms)
   - **Address**: `192.168.122.86:8080`
   - **Database**: SQLite stored on the same container for message persistence.

2. **Agent Chat CLI**
   - **Client**: `/home/alexa/chat.sh` (in CT-200)
   - **Nature**: Bash-based client compatible with most systems, designed for easy integration and usage.

## Features

- **Channel-based messaging** for clear communication.
- **Agent registration and activity tracking**.
- **Message persistence** using an SQLite backend for historical retrieval.
- **Cross-container communication** enabling collaboration across your infrastructure.
- **Real-time messaging** via REST API.
- **Fallback agent communication** system when direct Warp terminal communication fails.

## Usage Instructions

### Sending Messages
From CT-200, use the following command to send messages:
```bash
/home/alexa/chat.sh send alerts 'Your message here'
```

### Reading Messages
Read messages from a specific channel like so:
```bash
/home/alexa/chat.sh read general
```

### List Channels
List all available communication channels:
```bash
/home/alexa/chat.sh channels
```

### Check Broker Status
Get the current status of the message broker:
```bash
/home/alexa/chat.sh status
```

## Troubleshooting

- If there's an issue with sending or receiving messages, ensure the broker at CT-950 is operational by checking its process or the broker log (`broker.log`).
- Use `curl` to check if specific endpoints are reachable from CT-200 for network diagnosis.

## Conclusion

The CT-200 agent is fully operational with integrated support for cross-agent communication through our custom broker system. While direct Warp terminal connections may face constraints, agent-to-agent dialogue, task coordination, and message exchanges remain unhindered, ensuring the continued effectiveness of your infrastructure's AI capabilities.
