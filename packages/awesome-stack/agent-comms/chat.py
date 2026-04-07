#!/usr/bin/env python3
"""
Agent Chat CLI - Command-line interface for agent communication
"""

import json
import sys
import time
import requests
import argparse
from datetime import datetime

class AgentChat:
    def __init__(self, broker_url="http://192.168.122.86:8080", agent_id=None):
        self.broker_url = broker_url.rstrip('/')
        self.agent_id = agent_id or f"agent-{int(time.time())}"
        
    def send_message(self, channel, message):
        """Send a message to a channel"""
        try:
            response = requests.post(
                f"{self.broker_url}/messages/{channel}",
                json={"sender": self.agent_id, "message": message},
                timeout=5
            )
            if response.status_code == 200:
                print(f"✅ Message sent to #{channel}")
                return True
            else:
                print(f"❌ Failed to send message: {response.status_code}")
                return False
        except requests.RequestException as e:
            print(f"❌ Connection error: {e}")
            return False
    
    def get_messages(self, channel, limit=10):
        """Get recent messages from a channel"""
        try:
            response = requests.get(
                f"{self.broker_url}/messages/{channel}?limit={limit}",
                timeout=5
            )
            if response.status_code == 200:
                data = response.json()
                messages = data.get("messages", [])
                if messages:
                    print(f"\n📨 Recent messages in #{channel}:")
                    print("=" * 50)
                    for msg in reversed(messages):  # Show oldest first
                        timestamp = msg["timestamp"][:19]  # Remove microseconds
                        print(f"[{timestamp}] {msg['sender']}: {msg['message']}")
                    print("=" * 50)
                else:
                    print(f"📭 No messages in #{channel}")
                return messages
            else:
                print(f"❌ Failed to get messages: {response.status_code}")
                return []
        except requests.RequestException as e:
            print(f"❌ Connection error: {e}")
            return []
    
    def list_channels(self):
        """List all channels"""
        try:
            response = requests.get(f"{self.broker_url}/channels", timeout=5)
            if response.status_code == 200:
                data = response.json()
                channels = data.get("channels", [])
                if channels:
                    print("\n📋 Available channels:")
                    for channel in channels:
                        print(f"  #{channel}")
                else:
                    print("📭 No channels found")
                return channels
            else:
                print(f"❌ Failed to get channels: {response.status_code}")
                return []
        except requests.RequestException as e:
            print(f"❌ Connection error: {e}")
            return []
    
    def list_agents(self):
        """List all agents"""
        try:
            response = requests.get(f"{self.broker_url}/agents", timeout=5)
            if response.status_code == 200:
                data = response.json()
                agents = data.get("agents", [])
                if agents:
                    print("\n👥 Active agents:")
                    for agent in agents:
                        print(f"  {agent['id']} (last seen: {agent['last_seen'][:19]})")
                else:
                    print("👤 No agents found")
                return agents
            else:
                print(f"❌ Failed to get agents: {response.status_code}")
                return []
        except requests.RequestException as e:
            print(f"❌ Connection error: {e}")
            return []
    
    def get_status(self):
        """Get broker status"""
        try:
            response = requests.get(f"{self.broker_url}/status", timeout=5)
            if response.status_code == 200:
                data = response.json()
                print(f"\n📊 Broker Status:")
                print(f"  Status: {data['status']}")
                print(f"  Timestamp: {data['timestamp'][:19]}")
                print(f"  Channels: {data['channels']}")
                print(f"  Agents: {data['agents']}")
                return data
            else:
                print(f"❌ Failed to get status: {response.status_code}")
                return None
        except requests.RequestException as e:
            print(f"❌ Connection error: {e}")
            return None
    
    def watch_channel(self, channel, interval=2):
        """Watch a channel for new messages"""
        print(f"👁️  Watching #{channel} (Ctrl+C to stop)")
        last_msg_id = 0
        
        try:
            while True:
                messages = self.get_messages(channel, limit=50)
                if messages:
                    # Show only new messages
                    new_messages = [msg for msg in messages if msg["id"] > last_msg_id]
                    if new_messages:
                        for msg in sorted(new_messages, key=lambda x: x["id"]):
                            if msg["sender"] != self.agent_id:  # Don't show our own messages
                                timestamp = msg["timestamp"][:19]
                                print(f"[{timestamp}] {msg['sender']}: {msg['message']}")
                        last_msg_id = max(msg["id"] for msg in messages)
                
                time.sleep(interval)
                
        except KeyboardInterrupt:
            print("\n🛑 Stopped watching")

def main():
    parser = argparse.ArgumentParser(description="Agent Communication CLI")
    parser.add_argument("--broker", default="http://192.168.122.86:8080", 
                       help="Broker URL (default: http://192.168.122.86:8080)")
    parser.add_argument("--agent-id", help="Agent ID (default: auto-generated)")
    
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Send command
    send_parser = subparsers.add_parser("send", help="Send a message")
    send_parser.add_argument("channel", help="Channel name")
    send_parser.add_argument("message", nargs="+", help="Message text")
    
    # Read command
    read_parser = subparsers.add_parser("read", help="Read messages from channel")
    read_parser.add_argument("channel", help="Channel name")
    read_parser.add_argument("--limit", type=int, default=10, help="Number of messages to show")
    
    # Watch command
    watch_parser = subparsers.add_parser("watch", help="Watch channel for new messages")
    watch_parser.add_argument("channel", help="Channel name")
    watch_parser.add_argument("--interval", type=int, default=2, help="Update interval in seconds")
    
    # List commands
    subparsers.add_parser("channels", help="List all channels")
    subparsers.add_parser("agents", help="List all agents")
    subparsers.add_parser("status", help="Show broker status")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    chat = AgentChat(args.broker, args.agent_id)
    
    if args.command == "send":
        message = " ".join(args.message)
        chat.send_message(args.channel, message)
        
    elif args.command == "read":
        chat.get_messages(args.channel, args.limit)
        
    elif args.command == "watch":
        chat.watch_channel(args.channel, args.interval)
        
    elif args.command == "channels":
        chat.list_channels()
        
    elif args.command == "agents":
        chat.list_agents()
        
    elif args.command == "status":
        chat.get_status()

if __name__ == "__main__":
    main()
