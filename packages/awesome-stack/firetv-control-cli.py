#!/usr/bin/env python3
"""
Fire TV Control CLI - Command Line Interface for controlling Fire TV Cube
Run with: python3 firetv-control-cli.py --help
"""

import argparse
import requests
import json
import os
import configparser
import sys
from pathlib import Path

CONFIG_FILE = str(Path.home() / ".config" / "firetv-control" / "config.ini")

def ensure_config_exists():
    """Create default config if it doesn't exist"""
    config_dir = os.path.dirname(CONFIG_FILE)
    if not os.path.exists(config_dir):
        os.makedirs(config_dir)
    
    if not os.path.exists(CONFIG_FILE):
        config = configparser.ConfigParser()
        config['DEFAULT'] = {
            'FireTVControllerURL': 'http://192.168.1.150:5000',
            'Timeout': '5'
        }
        config['devices'] = {
            'living_room': 'http://192.168.1.150:5000',
            # Add more devices as needed
        }
        
        with open(CONFIG_FILE, 'w') as f:
            config.write(f)
        print(f"Created default config at {CONFIG_FILE}")
        return config
    
    config = configparser.ConfigParser()
    config.read(CONFIG_FILE)
    return config

def get_device_url(args):
    """Get the appropriate device URL from config or args"""
    config = ensure_config_exists()
    
    if args.device:
        if args.device in config['devices']:
            return config['devices'][args.device]
        else:
            print(f"Device '{args.device}' not found in config")
            sys.exit(1)
    
    if args.url:
        return args.url
    
    return config['DEFAULT']['FireTVControllerURL']

def send_command(url, command, params=None):
    """Send command to Fire TV controller API"""
    if params is None:
        params = {}
    
    endpoint = f"{url}/command"
    
    try:
        response = requests.post(
            endpoint,
            headers={"Content-Type": "application/json"},
            data=json.dumps({"command": command, "params": params}),
            timeout=5
        )
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error sending command: {e}")
        return {"success": False, "error": str(e)}

def get_status(url):
    """Get Fire TV status"""
    endpoint = f"{url}/status"
    
    try:
        response = requests.get(endpoint, timeout=5)
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"Error getting status: {e}")
        return {"connected": False, "error": str(e)}

def main():
    parser = argparse.ArgumentParser(description="Control Fire TV from command line")
    parser.add_argument('--device', help='Device name from config file')
    parser.add_argument('--url', help='Fire TV controller URL (overrides config)')
    
    subparsers = parser.add_subparsers(dest='command', help='Command to execute')
    
    # Status command
    subparsers.add_parser('status', help='Get Fire TV status')
    
    # Power command
    subparsers.add_parser('power', help='Toggle power')
    
    # Navigation commands
    subparsers.add_parser('home', help='Go to home screen')
    subparsers.add_parser('back', help='Go back')
    subparsers.add_parser('menu', help='Show menu')
    subparsers.add_parser('play', help='Play/Pause')
    subparsers.add_parser('up', help='Navigate up')
    subparsers.add_parser('down', help='Navigate down')
    subparsers.add_parser('left', help='Navigate left')
    subparsers.add_parser('right', help='Navigate right')
    subparsers.add_parser('select', help='Select (center button)')
    
    # Volume commands
    subparsers.add_parser('volume_up', help='Volume up')
    subparsers.add_parser('volume_down', help='Volume down')
    subparsers.add_parser('mute', help='Toggle mute')
    
    # Text input
    text_parser = subparsers.add_parser('text', help='Send text input')
    text_parser.add_argument('text_value', help='Text to input')
    
    # Launch app
    app_parser = subparsers.add_parser('app', help='Launch app')
    app_parser.add_argument('app_id', help='App ID or name')
    
    # Configuration
    config_parser = subparsers.add_parser('config', help='Configure devices')
    config_parser.add_argument('--add', nargs=2, metavar=('NAME', 'URL'), 
                             help='Add device: NAME URL')
    config_parser.add_argument('--remove', metavar='NAME', 
                             help='Remove device by name')
    config_parser.add_argument('--list', action='store_true', 
                             help='List configured devices')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    if args.command == 'config':
        config = ensure_config_exists()
        
        if args.add:
            name, url = args.add
            if 'devices' not in config:
                config['devices'] = {}
            config['devices'][name] = url
            with open(CONFIG_FILE, 'w') as f:
                config.write(f)
            print(f"Added device '{name}' with URL '{url}'")
        
        elif args.remove:
            if 'devices' in config and args.remove in config['devices']:
                config['devices'].pop(args.remove)
                with open(CONFIG_FILE, 'w') as f:
                    config.write(f)
                print(f"Removed device '{args.remove}'")
            else:
                print(f"Device '{args.remove}' not found")
        
        elif args.list:
            if 'devices' in config:
                print("Configured devices:")
                for name, url in config['devices'].items():
                    print(f"  {name}: {url}")
            else:
                print("No devices configured")
        
        return
    
    # Get the appropriate device URL
    url = get_device_url(args)
    
    if args.command == 'status':
        status = get_status(url)
        print(json.dumps(status, indent=2))
    
    elif args.command == 'text':
        result = send_command(url, 'text', {'text': args.text_value})
        print(json.dumps(result, indent=2))
    
    elif args.command == 'app':
        # Check if app is a known alias
        app_aliases = {
            'netflix': 'com.netflix.ninja',
            'prime': 'com.amazon.avod',
            'amazon': 'com.amazon.avod',
            'youtube': 'com.google.android.youtube.tv',
            'plex': 'com.plexapp.android',
            'disney': 'com.disney.disneyplus',
            'hulu': 'com.hulu.livingroomplus',
            'hbo': 'com.hbo.hbonow',
            'spotify': 'com.spotify.tv.android',
            'twitch': 'tv.twitch.android.app',
            'kodi': 'org.xbmc.kodi'
        }
        
        app_id = args.app_id
        if app_id.lower() in app_aliases:
            app_id = app_aliases[app_id.lower()]
        
        result = send_command(url, 'launch_app', {'app_id': app_id})
        print(json.dumps(result, indent=2))
    
    else:
        # Map CLI command to API command
        command_map = {
            'play': 'play_pause',
            'up': 'up',
            'down': 'down',
            'left': 'left',
            'right': 'right',
            'select': 'center',
            'mute': 'mute'
        }
        
        api_command = command_map.get(args.command, args.command)
        result = send_command(url, api_command)
        print(json.dumps(result, indent=2))

if __name__ == "__main__":
    main()
