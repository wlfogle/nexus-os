#!/usr/bin/env python3
"""
Script to add artwork to games in Heroic Games Launcher using SteamGridDB API
"""

import os
import json
import requests
import urllib.parse
from pathlib import Path

HEROIC_CONFIG = os.path.expanduser("~/.config/heroic/sideload_apps/library.json")
STEAMGRIDDB_API_URL = "https://www.steamgriddb.com/api/v2"

# Game name mappings for better search results
GAME_NAME_MAPPINGS = {
    "The Wheel of Time": "Wheel of Time",
    "Battlezone Combat Commander": "Battlezone Combat Commander",
    "Ravenloft - Strahd's Possession": "Ravenloft Strahd's Possession",
    "Might and Magic 9": "Might and Magic IX",
    "Mafia III Definitive Edition": "Mafia III",
    "Dark Sun 2": "Dark Sun Shattered Lands",
    "SpellForce - Platinum Edition": "SpellForce The Order of Dawn",
    "Champions of Krynn": "Champions of Krynn",
    "Cosmo's Cosmic Adventure": "Cosmo's Cosmic Adventure",
    "The Dark Crystal Age of Resistance Tactics": "Dark Crystal Age of Resistance Tactics",
    "Dungeons and Dragons - Dragonshard": "Dungeons Dragons Dragonshard",
    "Dark Sun": "Dark Sun Shattered Lands",
    "Shores Unknown Arrival": "Shores Unknown",
    "Fae Tactics": "Fae Tactics",
    "Asteroids Recharged": "Asteroids Recharged",
    "The Dark Queen of Krynn": "Dark Queen of Krynn",
    "Dark Deity Complete Edition": "Dark Deity",
    "Ravenloft - Stone Prophet": "Ravenloft Stone Prophet",
    "Death Knights of Krynn": "Death Knights of Krynn",
    "Missile Command Recharged": "Missile Command Recharged",
    "The Last Spell": "The Last Spell",
    "DRG.Survivor.v0.4.87d.ALL.DLC": "Deep Rock Galactic Survivor",
    "Dead Island 2": "Dead Island 2",
    "Last_Epoch": "Last Epoch"
}

def search_game_artwork(game_name, api_key=None):
    """Search for game artwork on SteamGridDB (public API, no key needed for basic search)"""
    try:
        # Clean up game name for search
        search_name = GAME_NAME_MAPPINGS.get(game_name, game_name)
        search_name = search_name.replace(" - ", " ").replace(":", "").replace("'", "")
        
        # Try to find game ID first
        search_url = f"https://www.steamgriddb.com/api/v2/search/autocomplete/{urllib.parse.quote(search_name)}"
        headers = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        
        response = requests.get(search_url, headers=headers, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            if data.get("success") and data.get("data"):
                # Get the first game result
                game_id = data["data"][0]["id"]
                
                # Get grids for this game
                grids_url = f"https://www.steamgriddb.com/api/v2/grids/game/{game_id}"
                grids_response = requests.get(grids_url, headers=headers, timeout=10)
                
                if grids_response.status_code == 200:
                    grids_data = grids_response.json()
                    if grids_data.get("success") and grids_data.get("data"):
                        # Get the first grid image
                        grid_url = grids_data["data"][0]["url"]
                        return grid_url, grid_url  # Use same image for cover and square
        
        # Fallback: Try to construct a generic image URL
        return generate_placeholder_artwork(game_name)
        
    except Exception as e:
        print(f"Error searching artwork for {game_name}: {e}")
        return generate_placeholder_artwork(game_name)

def generate_placeholder_artwork(game_name):
    """Generate placeholder artwork URLs"""
    # Use a service like via.placeholder.com or create local placeholders
    placeholder_url = f"https://via.placeholder.com/600x900/2C3E50/FFFFFF?text={urllib.parse.quote(game_name.replace(' ', '+'))}"
    return placeholder_url, placeholder_url

def get_existing_games():
    """Get currently installed games from Heroic"""
    try:
        with open(HEROIC_CONFIG, 'r') as f:
            data = json.load(f)
            return data.get('games', [])
    except (FileNotFoundError, json.JSONDecodeError):
        return []

def update_game_artwork():
    """Update artwork for all games without artwork"""
    games = get_existing_games()
    
    if not games:
        print("No games found in Heroic library")
        return
    
    updated_count = 0
    
    for game in games:
        game_name = game.get("title", "Unknown")
        
        # Skip if game already has artwork
        if game.get("art_cover") and game.get("art_square"):
            print(f"Skipping {game_name} (already has artwork)")
            continue
        
        print(f"Searching artwork for: {game_name}")
        
        # Search for artwork
        cover_url, square_url = search_game_artwork(game_name)
        
        if cover_url and square_url:
            game["art_cover"] = cover_url
            game["art_square"] = square_url
            updated_count += 1
            print(f"  ✓ Added artwork for {game_name}")
        else:
            print(f"  ✗ No artwork found for {game_name}")
    
    if updated_count > 0:
        # Backup existing library
        backup_path = HEROIC_CONFIG + ".artwork_backup"
        if os.path.exists(HEROIC_CONFIG):
            import shutil
            shutil.copy2(HEROIC_CONFIG, backup_path)
            print(f"Backed up library to {backup_path}")
        
        # Save updated library
        library_data = {"games": games}
        with open(HEROIC_CONFIG, 'w') as f:
            json.dump(library_data, f, indent=4)
        
        print(f"Updated artwork for {updated_count} games")
        print("Restart Heroic to see the new artwork")
    else:
        print("No games needed artwork updates")

def main():
    """Main function"""
    print("Heroic Games Launcher - Artwork Updater")
    print("=" * 50)
    
    if not os.path.exists(HEROIC_CONFIG):
        print(f"Heroic config not found: {HEROIC_CONFIG}")
        return
    
    update_game_artwork()

if __name__ == "__main__":
    main()
