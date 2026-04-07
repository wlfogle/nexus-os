#!/usr/bin/env python3
"""
Test script for The Walking Dead Webisodes Downloader
====================================================
Tests the core functionality without GUI dependencies.
"""

import sys
import os
from pathlib import Path

def test_dependencies():
    """Test if required dependencies are available"""
    print("🧪 Testing dependencies...")
    
    try:
        import requests
        print("✅ requests - OK")
    except ImportError:
        print("❌ requests - Missing (install with: pip install requests)")
        return False
    
    try:
        import yt_dlp
        print("✅ yt-dlp - OK")
    except ImportError:
        print("❌ yt-dlp - Missing (install with: pip install yt-dlp)")
        return False
    
    try:
        import tkinter
        print("✅ tkinter - OK")
    except ImportError:
        print("⚠️  tkinter - Missing (install with system package manager)")
        print("   Ubuntu/Debian: sudo apt install python3-tk")
        print("   Arch: sudo pacman -S tk")
        print("   Fedora: sudo dnf install tkinter")
        return False
    
    return True

def test_archive_search():
    """Test Internet Archive search functionality"""
    print("\n🌐 Testing Internet Archive search...")
    
    try:
        import requests
        session = requests.Session()
        session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        })
        
        # Simple test search
        search_url = "https://archive.org/advancedsearch.php"
        params = {
            'q': 'walking dead webisode',
            'fl': 'identifier,title',
            'rows': 5,
            'output': 'json'
        }
        
        response = session.get(search_url, params=params, timeout=10)
        if response.status_code == 200:
            data = response.json()
            results = data.get('response', {}).get('docs', [])
            print(f"✅ Archive.org search working - Found {len(results)} results")
            
            # Show first few results
            for i, item in enumerate(results[:3]):
                title = item.get('title', 'Unknown')
                identifier = item.get('identifier', 'Unknown')
                print(f"   {i+1}. {title} (ID: {identifier})")
            
            return True
        else:
            print(f"❌ Archive.org search failed - HTTP {response.status_code}")
            return False
            
    except Exception as e:
        print(f"❌ Archive.org search error: {e}")
        return False

def test_yt_dlp():
    """Test yt-dlp functionality with a simple URL"""
    print("\n📺 Testing yt-dlp functionality...")
    
    try:
        import yt_dlp
        
        # Test with a simple info extraction (no download)
        ydl_opts = {
            'quiet': True,
            'no_warnings': True,
            'extract_flat': True,
        }
        
        # Use a safe test URL
        test_url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ"  # Rick Roll for testing
        
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            info = ydl.extract_info(test_url, download=False)
            if info:
                print("✅ yt-dlp working - Can extract video info")
                return True
            else:
                print("❌ yt-dlp failed - Could not extract video info")
                return False
                
    except Exception as e:
        print(f"❌ yt-dlp error: {e}")
        return False

def main():
    """Main test function"""
    print("🎬 The Walking Dead Webisodes Downloader - Dependency Test")
    print("=" * 60)
    
    tests = [
        ("Dependencies", test_dependencies),
        ("Archive.org Search", test_archive_search),
        ("yt-dlp Functionality", test_yt_dlp)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n🧪 Running test: {test_name}")
        try:
            if test_func():
                passed += 1
                print(f"✅ {test_name} - PASSED")
            else:
                print(f"❌ {test_name} - FAILED")
        except Exception as e:
            print(f"💥 {test_name} - ERROR: {e}")
    
    print("\n" + "=" * 60)
    print(f"📊 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! The TWD downloader should work correctly.")
        print("✨ You can run the GUI with: python3 twd_webisodes_gui.py")
    else:
        print("⚠️  Some tests failed. Please install missing dependencies:")
        if passed < total:
            print("💡 Install missing packages:")
            print("   pip install yt-dlp requests")
            print("   # For tkinter (system package manager):")
            print("   sudo pacman -S tk  # Arch Linux")
    
    return 0 if passed == total else 1

if __name__ == "__main__":
    exit(main())
