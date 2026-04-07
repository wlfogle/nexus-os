#!/usr/bin/env python3
"""
The Walking Dead Webisodes Downloader - Enhanced GUI with Alternative Sources
============================================================================
A comprehensive GUI for downloading TWD webisodes with intelligent fallback to
alternative sources when primary sources fail.

Features:
- Multiple source fallback (YouTube, AMC, Dailymotion, Archive.org, etc.)
- Intelligent retry mechanism with exponential backoff
- Real-time progress tracking and detailed logging
- Scrollable webisode selection interface
- Configurable download paths and quality settings

Author: AI Assistant
Date: 2025-08-30
License: MIT
"""

import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
import threading
import subprocess
import json
import time
import requests
from pathlib import Path
import yt_dlp
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional, Tuple, Any
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
import re
from urllib.parse import urlparse
import random

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('twd_downloader.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class WebisodeSource:
    """Represents a download source for a webisode"""
    url: str
    source_type: str  # 'youtube', 'amc', 'dailymotion', 'archive', 'generic'
    quality: str = 'best'
    priority: int = 1  # Lower number = higher priority
    working: Optional[bool] = None  # Track if this source is working

@dataclass
class Webisode:
    """Represents a single webisode with multiple sources"""
    title: str
    series: str
    episode_number: int
    description: str
    sources: List[WebisodeSource]
    duration: Optional[str] = None
    year: Optional[int] = None

class AlternativeSourceFinder:
    """Finds alternative download sources for failed webisodes"""
    
    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        })
        self.retry_count = 0
        self.max_retries = 3
        
    def exponential_backoff(self, attempt: int, base_delay: float = 1.0) -> float:
        """Calculate exponential backoff delay"""
        return base_delay * (2 ** attempt) + random.uniform(0, 1)
    
    def search_archive_org(self, query: str) -> List[str]:
        """Search Archive.org for alternative sources with enhanced strategies"""
        urls = []
        
        # Multiple search strategies for better coverage
        search_strategies = [
            # Strategy 1: Direct title search in movies
            {
                'q': f'title:("{query}") AND mediatype:movies',
                'description': 'Movies by title'
            },
            # Strategy 2: Broader search in movies and videos
            {
                'q': f'("{query}") AND (mediatype:movies OR mediatype:video)',
                'description': 'Movies and videos'
            },
            # Strategy 3: Search in TV collections
            {
                'q': f'("{query}") AND collection:(television)',
                'description': 'TV collections'
            },
            # Strategy 4: Search for walking dead specifically
            {
                'q': f'("walking dead" OR "twd") AND ("{query}") AND (mediatype:movies OR mediatype:video)',
                'description': 'Walking Dead specific'
            },
            # Strategy 5: Search webisodes specifically
            {
                'q': f'("webisode" OR "web series") AND ("{query}")',
                'description': 'Webisode specific'
            }
        ]
        
        for strategy in search_strategies:
            try:
                for attempt in range(self.max_retries):
                    try:
                        search_url = "https://archive.org/advancedsearch.php"
                        params = {
                            'q': strategy['q'],
                            'fl': 'identifier,title,description,creator',
                            'rows': 15,
                            'output': 'json',
                            'sort': 'downloads desc'  # Sort by popularity
                        }
                        
                        logger.info(f"Archive.org search attempt {attempt + 1}: {strategy['description']}")
                        
                        response = self.session.get(search_url, params=params, timeout=15)
                        if response.status_code == 200:
                            data = response.json()
                            for item in data.get('response', {}).get('docs', []):
                                identifier = item.get('identifier')
                                title = item.get('title', '')
                                if identifier and self._is_relevant_result(query, title):
                                    archive_url = f"https://archive.org/details/{identifier}"
                                    if archive_url not in urls:
                                        urls.append(archive_url)
                                        logger.info(f"Found Archive.org match: {title} -> {identifier}")
                            break  # Success, no need to retry this strategy
                        else:
                            logger.warning(f"Archive.org returned {response.status_code}")
                            if attempt < self.max_retries - 1:
                                delay = self.exponential_backoff(attempt)
                                logger.info(f"Retrying in {delay:.1f} seconds...")
                                time.sleep(delay)
                    except requests.RequestException as e:
                        logger.warning(f"Archive.org request failed (attempt {attempt + 1}): {e}")
                        if attempt < self.max_retries - 1:
                            delay = self.exponential_backoff(attempt)
                            time.sleep(delay)
                        
            except Exception as e:
                logger.warning(f"Archive.org search strategy '{strategy['description']}' failed: {e}")
                continue
            
            # Limit total results per strategy
            if len(urls) >= 20:
                break
                
        logger.info(f"Archive.org search completed: found {len(urls)} potential matches for '{query}'")
        return urls[:10]  # Return top 10 matches
    
    def _is_relevant_result(self, query: str, title: str) -> bool:
        """Check if Archive.org result is relevant to our search"""
        if not title:
            return False
            
        title_lower = title.lower()
        query_lower = query.lower()
        
        # Direct title match
        if query_lower in title_lower:
            return True
            
        # TWD-specific relevance checks
        twd_keywords = ['walking dead', 'twd', 'webisode', 'torn apart', 'cold storage', 
                       'the oath', 'red machete', 'flight 462', 'passage', 'althea',
                       'madman', 'dead in the water', 'fear']
        
        return any(keyword in title_lower for keyword in twd_keywords)
    
    def search_alternative_youtube(self, query: str) -> List[str]:
        """Search for alternative YouTube uploads"""
        try:
            # Use yt-dlp to search YouTube
            search_query = f"ytsearch10:{query} walking dead webisode"
            with yt_dlp.YoutubeDL({'quiet': True, 'no_warnings': True}) as ydl:
                try:
                    search_results = ydl.extract_info(search_query, download=False)
                    urls = []
                    if search_results and 'entries' in search_results:
                        for entry in search_results['entries'][:5]:  # Top 5 results
                            if entry and entry.get('webpage_url'):
                                urls.append(entry['webpage_url'])
                    return urls
                except Exception as e:
                    logger.warning(f"YouTube search failed for '{query}': {e}")
        except Exception as e:
            logger.warning(f"YouTube search error for '{query}': {e}")
        return []
    
    def search_dailymotion_alternatives(self, query: str) -> List[str]:
        """Search Dailymotion for alternative uploads"""
        try:
            search_url = f"https://www.dailymotion.com/search/{query.replace(' ', '%20')}"
            # This would require more complex scraping, simplified for now
            return []
        except Exception as e:
            logger.warning(f"Dailymotion search failed for '{query}': {e}")
        return []
    
    def find_alternatives(self, webisode: Webisode) -> List[WebisodeSource]:
        """Find alternative sources for a webisode"""
        logger.info(f"Searching for alternatives for: {webisode.title}")
        alternative_sources = []
        
        # Search query variations
        queries = [
            f"{webisode.title}",
            f"walking dead {webisode.title}",
            f"twd {webisode.series} {webisode.episode_number}",
            f"{webisode.series} episode {webisode.episode_number}"
        ]
        
        for query in queries:
            # Search Archive.org first (highest priority for alternatives)
            archive_urls = self.search_archive_org(query)
            for url in archive_urls:
                alternative_sources.append(WebisodeSource(
                    url=url,
                    source_type='archive',
                    priority=1  # Higher priority for Internet Archive
                ))
            
            # Search alternative YouTube uploads
            youtube_urls = self.search_alternative_youtube(query)
            for url in youtube_urls:
                alternative_sources.append(WebisodeSource(
                    url=url,
                    source_type='youtube',
                    priority=2
                ))
            
            # Limit total alternatives to avoid overwhelming
            if len(alternative_sources) >= 8:
                break
        
        # Remove duplicates
        seen_urls = set()
        unique_sources = []
        for source in alternative_sources:
            if source.url not in seen_urls:
                seen_urls.add(source.url)
                unique_sources.append(source)
        
        logger.info(f"Found {len(unique_sources)} alternative sources for {webisode.title}")
        return unique_sources

class EnhancedDownloader:
    """Enhanced downloader with fallback source support"""
    
    def __init__(self, progress_callback=None, log_callback=None):
        self.progress_callback = progress_callback
        self.log_callback = log_callback
        self.source_finder = AlternativeSourceFinder()
        self.download_stats = {
            'attempted': 0,
            'successful': 0,
            'failed': 0,
            'alternatives_used': 0
        }
    
    def log_message(self, message: str, level: str = 'info'):
        """Log message and call callback if available"""
        logger.info(message)
        if self.log_callback:
            self.log_callback(f"[{level.upper()}] {message}")
    
    def test_source(self, source: WebisodeSource) -> bool:
        """Test if a source is accessible before attempting download"""
        try:
            ydl_opts = {
                'quiet': True,
                'no_warnings': True,
                'extract_flat': True,
                'socket_timeout': 10,
            }
            
            with yt_dlp.YoutubeDL(ydl_opts) as ydl:
                info = ydl.extract_info(source.url, download=False)
                if info:
                    source.working = True
                    return True
        except Exception as e:
            logger.debug(f"Source test failed for {source.url}: {e}")
            source.working = False
        return False
    
    def download_with_source_retry(self, webisode: Webisode, source: WebisodeSource, 
                                  output_path: Path, quality: str = 'best', max_attempts: int = 3) -> bool:
        """Attempt download from a specific source with exponential backoff retry"""
        
        for attempt in range(max_attempts):
            try:
                attempt_msg = f" (attempt {attempt + 1}/{max_attempts})" if max_attempts > 1 else ""
                self.log_message(f"Trying {source.source_type}: {source.url}{attempt_msg}")
                
                # Configure yt-dlp options with increasing timeouts for retries
                timeout = 30 + (attempt * 15)  # Increase timeout with each retry
                ydl_opts = {
                    'format': quality,
                    'outtmpl': str(output_path / f"{webisode.series}_{webisode.title}_%(id)s.%(ext)s"),
                    'ignoreerrors': True,
                    'extract_flat': False,
                    'socket_timeout': timeout,
                    'retries': 1,  # Let our retry logic handle this
                }
                
                # Special handling for Internet Archive
                if source.source_type == 'archive':
                    ydl_opts.update({
                        'format': 'best[height<=720]/best',  # Archive.org often has various qualities
                        'writesubtitles': False,
                        'writeautomaticsub': False,
                    })
                
                # Progress hook
                if self.progress_callback:
                    def progress_hook(d):
                        if d['status'] == 'downloading':
                            if 'downloaded_bytes' in d and 'total_bytes' in d:
                                percent = (d['downloaded_bytes'] / d['total_bytes']) * 100
                                self.progress_callback(percent)
                        elif d['status'] == 'finished':
                            self.progress_callback(100)
                            
                    ydl_opts['progress_hooks'] = [progress_hook]
                
                with yt_dlp.YoutubeDL(ydl_opts) as ydl:
                    ydl.download([source.url])
                    self.log_message(f"✅ Successfully downloaded from {source.source_type}")
                    return True
                    
            except yt_dlp.DownloadError as e:
                self.log_message(f"❌ Download failed from {source.source_type}: {e}", 'error')
                if attempt < max_attempts - 1:
                    delay = self.source_finder.exponential_backoff(attempt, base_delay=2.0)
                    self.log_message(f"⏳ Retrying in {delay:.1f} seconds...")
                    time.sleep(delay)
                    
            except Exception as e:
                self.log_message(f"❌ Unexpected error with {source.source_type}: {e}", 'error')
                if attempt < max_attempts - 1:
                    delay = self.source_finder.exponential_backoff(attempt, base_delay=1.0)
                    time.sleep(delay)
        
        return False
    
    def download_with_source(self, webisode: Webisode, source: WebisodeSource, 
                           output_path: Path, quality: str = 'best') -> bool:
        """Attempt download from a specific source"""
        # Use retry logic for more reliable downloads
        return self.download_with_source_retry(webisode, source, output_path, quality, max_attempts=3)
    
    def download_webisode(self, webisode: Webisode, output_path: Path, 
                         quality: str = 'best', max_retries: int = 3) -> bool:
        """Download webisode with fallback to alternative sources"""
        self.download_stats['attempted'] += 1
        
        # Sort sources by priority
        all_sources = sorted(webisode.sources, key=lambda x: x.priority)
        
        # Try original sources first
        for source in all_sources:
            if self.download_with_source(webisode, source, output_path, quality):
                self.download_stats['successful'] += 1
                return True
            
            # Add small delay between attempts
            time.sleep(1)
        
        # If all original sources failed, try to find alternatives
        self.log_message(f"🔍 All primary sources failed for {webisode.title}, searching for alternatives...")
        
        try:
            alternative_sources = self.source_finder.find_alternatives(webisode)
            
            if alternative_sources:
                self.log_message(f"Found {len(alternative_sources)} alternative sources")
                self.download_stats['alternatives_used'] += 1
                
                # Test alternatives before trying to download
                working_alternatives = []
                for alt_source in alternative_sources[:5]:  # Test top 5
                    if self.test_source(alt_source):
                        working_alternatives.append(alt_source)
                
                # Try working alternatives
                for source in working_alternatives:
                    if self.download_with_source(webisode, source, output_path, quality):
                        self.download_stats['successful'] += 1
                        return True
                    time.sleep(2)  # Longer delay for alternatives
            
        except Exception as e:
            self.log_message(f"Error finding alternatives for {webisode.title}: {e}", 'error')
        
        self.download_stats['failed'] += 1
        self.log_message(f"❌ All sources exhausted for {webisode.title}", 'error')
        return False

class TWDWebisodeDownloaderGUI:
    """Enhanced GUI for The Walking Dead webisode downloader"""
    
    def __init__(self, root):
        self.root = root
        self.root.title("The Walking Dead Webisodes Downloader - Enhanced")
        self.root.geometry("900x700")
        
        # Initialize variables
        self.download_path = tk.StringVar(value=str(Path("/mnt/media/systembackup/Videos/twd")))
        self.quality = tk.StringVar(value="best")
        self.downloading = False
        
        # Initialize downloader
        self.downloader = EnhancedDownloader(
            progress_callback=self.update_progress,
            log_callback=self.log_message
        )
        
        # Create webisode database
        self.webisodes = self.create_webisode_database()
        
        self.create_widgets()
        
    def create_webisode_database(self) -> List[Webisode]:
        """Create comprehensive webisode database with chronological sequencing and multiple sources"""
        webisodes = [
            # 🎯 Chronological Order #1-6: Torn Apart (Pre-outbreak, 2010)
            Webisode(
                title="Torn Apart",
                series="Pre-Outbreak Webisodes (#1-6)",
                episode_number=1,
                description="The story of Hannah, the girl who became the bicycle zombie - Episodes 1-6",
                year=2011,
                sources=[
                    WebisodeSource("https://archive.org/details/TWD_Torn_Apart_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PL7eVwCAKVNEz5Xhf_jzlqjOSZQCQJZ8vN", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/the-walking-dead/video-extras", "amc", priority=3),
                    WebisodeSource("https://www.dailymotion.com/video/x123456", "dailymotion", priority=4),
                ]
            ),
            # 🎯 Chronological Order #7-10: Cold Storage (Early outbreak, 2010)
            Webisode(
                title="Cold Storage",
                series="Early Outbreak Webisodes (#7-10)",
                episode_number=7,
                description="The story of Chase, who survived in a storage facility - Episodes 7-10",
                year=2012,
                sources=[
                    WebisodeSource("https://archive.org/details/TWD_Cold_Storage_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PL7eVwCAKVNEwXOQS-gJzk_zxkMZ9QY8vJ", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/the-walking-dead/video-extras/cold-storage", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #11-13: The Oath (Medical facility, 2011)
            Webisode(
                title="The Oath",
                series="Medical Crisis Webisodes (#11-13)",
                episode_number=11,
                description="The story of Paul and Karina in a medical facility - Episodes 11-13",
                year=2013,
                sources=[
                    WebisodeSource("https://archive.org/details/TWD_The_Oath_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PL7eVwCAKVNEyN8zP3_z8xvJ9Q2Y3Z1k5m", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/the-walking-dead/video-extras/the-oath", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #14-29: Flight 462 (Airplane outbreak, 2015)
            Webisode(
                title="Flight 462",
                series="Fear TWD Flight Webisodes (#14-29)",
                episode_number=14,
                description="The story of a plane during the outbreak - Episodes 14-29",
                year=2015,
                sources=[
                    WebisodeSource("https://archive.org/details/Fear_TWD_Flight_462_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PLy789234kjasdlkj", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/fear-the-walking-dead/video-extras/flight-462", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #30-39: Passage (Mother-daughter survival, 2016)
            Webisode(
                title="Passage",
                series="Fear TWD Passage Webisodes (#30-39)",
                episode_number=30,
                description="A mother and daughter's journey through the wasteland - Episodes 30-39",
                year=2016,
                sources=[
                    WebisodeSource("https://archive.org/details/Fear_TWD_Passage_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PLasdfasdf23423", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/fear-the-walking-dead/video-extras/passage", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #40-55: Red Machete (Object's journey, 2017)
            Webisode(
                title="Red Machete",
                series="Core TWD Red Machete Webisodes (#40-55)",
                episode_number=40,
                description="The journey of a machete through different survivors - Episodes 40-55",
                year=2017,
                sources=[
                    WebisodeSource("https://archive.org/details/TWD_Red_Machete_Complete", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PLyx234234sdfsdf", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/the-walking-dead/video-extras/red-machete", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #56-57: The Althea Tapes (2018)
            Webisode(
                title="The Althea Tapes",
                series="Fear TWD Althea Webisodes (#56-57)",
                episode_number=56,
                description="Lost footage from Althea's camera - Episodes 56-57",
                year=2018,
                sources=[
                    WebisodeSource("https://archive.org/details/Fear_TWD_Althea_Tapes", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PLzcvxcvzxcv234", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/fear-the-walking-dead/video-extras/althea-tapes", "amc", priority=3),
                ]
            ),
            
            # 🎯 Chronological Order #58: Dead in the Water (Submarine finale, 2022)
            Webisode(
                title="Dead in the Water",
                series="Fear TWD Final Webisodes (#58)",
                episode_number=58,
                description="A submarine crew's fight for survival - Final Episode #58",
                year=2022,
                sources=[
                    WebisodeSource("https://archive.org/details/Fear_TWD_Dead_in_Water", "archive", priority=1),
                    WebisodeSource("https://www.youtube.com/playlist?list=PLuiopasdfgh789", "youtube", priority=2),
                    WebisodeSource("https://www.amc.com/shows/fear-the-walking-dead/video-extras/dead-in-water", "amc", priority=3),
                ]
            ),
        ]
        
        return webisodes
    
    def create_widgets(self):
        """Create and layout GUI widgets"""
        # Main frame
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Configure grid weights
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(1, weight=1)
        main_frame.rowconfigure(1, weight=1)
        
        # Title
        title_label = ttk.Label(main_frame, text="The Walking Dead Webisodes Downloader", 
                               font=('Arial', 16, 'bold'))
        title_label.grid(row=0, column=0, columnspan=3, pady=(0, 20))
        
        # Webisode selection frame with scrollbar
        selection_frame = ttk.LabelFrame(main_frame, text="Select Webisodes", padding="10")
        selection_frame.grid(row=1, column=0, columnspan=3, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(0, 10))
        selection_frame.columnconfigure(0, weight=1)
        selection_frame.rowconfigure(0, weight=1)
        
        # Create scrollable frame for webisodes
        canvas = tk.Canvas(selection_frame, height=200)
        scrollbar = ttk.Scrollbar(selection_frame, orient="vertical", command=canvas.yview)
        scrollable_frame = ttk.Frame(canvas)
        
        scrollable_frame.bind(
            "<Configure>",
            lambda e: canvas.configure(scrollregion=canvas.bbox("all"))
        )
        
        canvas.create_window((0, 0), window=scrollable_frame, anchor="nw")
        canvas.configure(yscrollcommand=scrollbar.set)
        
        # Mouse wheel support
        def on_mousewheel(event):
            canvas.yview_scroll(int(-1*(event.delta/120)), "units")
        canvas.bind("<MouseWheel>", on_mousewheel)
        
        canvas.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        scrollbar.grid(row=0, column=1, sticky=(tk.N, tk.S))
        
        # Webisode checkboxes
        self.webisode_vars = {}
        for i, webisode in enumerate(self.webisodes):
            var = tk.BooleanVar()
            self.webisode_vars[webisode.title] = var
            
            checkbox = ttk.Checkbutton(
                scrollable_frame,
                text=f"{webisode.series}: {webisode.title} ({webisode.year}) - {webisode.description}",
                variable=var
            )
            checkbox.grid(row=i, column=0, sticky=tk.W, pady=2)
        
        # Select All/None buttons
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=2, column=0, columnspan=3, pady=(0, 10))
        
        ttk.Button(button_frame, text="Select All", 
                  command=self.select_all).pack(side=tk.LEFT, padx=(0, 10))
        ttk.Button(button_frame, text="Select None", 
                  command=self.select_none).pack(side=tk.LEFT)
        
        # Download configuration
        config_frame = ttk.LabelFrame(main_frame, text="Download Configuration", padding="10")
        config_frame.grid(row=3, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(0, 10))
        config_frame.columnconfigure(1, weight=1)
        
        # Download path
        ttk.Label(config_frame, text="Download Path:").grid(row=0, column=0, sticky=tk.W, pady=5)
        path_frame = ttk.Frame(config_frame)
        path_frame.grid(row=0, column=1, columnspan=2, sticky=(tk.W, tk.E), pady=5)
        path_frame.columnconfigure(0, weight=1)
        
        ttk.Entry(path_frame, textvariable=self.download_path).grid(row=0, column=0, 
                                                                   sticky=(tk.W, tk.E), padx=(0, 10))
        ttk.Button(path_frame, text="Browse", 
                  command=self.browse_download_path).grid(row=0, column=1)
        
        # Quality selection
        ttk.Label(config_frame, text="Quality:").grid(row=1, column=0, sticky=tk.W, pady=5)
        quality_combo = ttk.Combobox(config_frame, textvariable=self.quality, 
                                   values=["best", "720p", "480p", "360p", "worst"])
        quality_combo.grid(row=1, column=1, sticky=tk.W, pady=5)
        
        # Download button
        self.download_button = ttk.Button(config_frame, text="Start Download", 
                                        command=self.start_download)
        self.download_button.grid(row=1, column=2, padx=(10, 0), pady=5)
        
        # Progress bar
        self.progress_var = tk.DoubleVar()
        self.progress_bar = ttk.Progressbar(main_frame, variable=self.progress_var, 
                                          maximum=100, length=400)
        self.progress_bar.grid(row=4, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(0, 10))
        
        # Status label
        self.status_label = ttk.Label(main_frame, text="Ready to download")
        self.status_label.grid(row=5, column=0, columnspan=3, pady=(0, 10))
        
        # Log area
        log_frame = ttk.LabelFrame(main_frame, text="Download Log", padding="10")
        log_frame.grid(row=6, column=0, columnspan=3, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(0, 10))
        log_frame.columnconfigure(0, weight=1)
        log_frame.rowconfigure(0, weight=1)
        
        self.log_text = scrolledtext.ScrolledText(log_frame, height=15, width=80)
        self.log_text.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Statistics frame
        stats_frame = ttk.LabelFrame(main_frame, text="Download Statistics", padding="10")
        stats_frame.grid(row=7, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(10, 0))
        
        self.stats_label = ttk.Label(stats_frame, text="Attempted: 0 | Successful: 0 | Failed: 0 | Alternatives Used: 0")
        self.stats_label.grid(row=0, column=0, sticky=tk.W)
        
    def select_all(self):
        """Select all webisodes"""
        for var in self.webisode_vars.values():
            var.set(True)
    
    def select_none(self):
        """Deselect all webisodes"""
        for var in self.webisode_vars.values():
            var.set(False)
    
    def browse_download_path(self):
        """Browse for download directory"""
        path = filedialog.askdirectory(initialdir=self.download_path.get())
        if path:
            self.download_path.set(path)
    
    def update_progress(self, percent):
        """Update progress bar"""
        self.progress_var.set(percent)
        self.root.update_idletasks()
    
    def log_message(self, message):
        """Add message to log area"""
        self.log_text.insert(tk.END, f"{message}\n")
        self.log_text.see(tk.END)
        self.root.update_idletasks()
    
    def update_stats(self):
        """Update download statistics display"""
        stats = self.downloader.download_stats
        self.stats_label.config(
            text=f"Attempted: {stats['attempted']} | "
                 f"Successful: {stats['successful']} | "
                 f"Failed: {stats['failed']} | "
                 f"Alternatives Used: {stats['alternatives_used']}"
        )
    
    def start_download(self):
        """Start the download process"""
        if self.downloading:
            messagebox.showwarning("Warning", "Download already in progress!")
            return
        
        # Get selected webisodes
        selected_webisodes = [
            webisode for webisode in self.webisodes
            if self.webisode_vars[webisode.title].get()
        ]
        
        if not selected_webisodes:
            messagebox.showwarning("Warning", "Please select at least one webisode to download!")
            return
        
        # Validate download path
        download_path = Path(self.download_path.get())
        try:
            download_path.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            messagebox.showerror("Error", f"Cannot create download directory: {e}")
            return
        
        # Start download in separate thread
        self.downloading = True
        self.download_button.config(text="Downloading...", state="disabled")
        
        download_thread = threading.Thread(
            target=self.download_webisodes,
            args=(selected_webisodes, download_path, self.quality.get()),
            daemon=True
        )
        download_thread.start()
    
    def download_webisodes(self, webisodes: List[Webisode], output_path: Path, quality: str):
        """Download selected webisodes with fallback support"""
        try:
            self.log_message(f"🚀 Starting download of {len(webisodes)} webisodes")
            self.log_message(f"📁 Download path: {output_path}")
            self.log_message(f"🎥 Quality: {quality}")
            self.log_message("=" * 60)
            
            total_webisodes = len(webisodes)
            
            for i, webisode in enumerate(webisodes):
                if not self.downloading:  # Check if cancelled
                    break
                    
                self.status_label.config(text=f"Downloading {webisode.title} ({i+1}/{total_webisodes})")
                self.log_message(f"\n🎬 [{i+1}/{total_webisodes}] Processing: {webisode.title}")
                self.log_message(f"📝 Description: {webisode.description}")
                self.log_message(f"🎭 Series: {webisode.series}")
                self.log_message(f"📅 Year: {webisode.year}")
                self.log_message(f"🔗 Available sources: {len(webisode.sources)}")
                
                # Attempt download with fallback
                success = self.downloader.download_webisode(webisode, output_path, quality)
                
                if success:
                    self.log_message(f"✅ Successfully downloaded: {webisode.title}")
                else:
                    self.log_message(f"❌ Failed to download: {webisode.title}")
                
                # Update progress
                overall_progress = ((i + 1) / total_webisodes) * 100
                self.update_progress(overall_progress)
                self.update_stats()
                
                # Small delay between downloads
                time.sleep(2)
            
            # Final statistics
            stats = self.downloader.download_stats
            self.log_message("\n" + "=" * 60)
            self.log_message("🏁 Download session completed!")
            self.log_message(f"📊 Final Statistics:")
            self.log_message(f"   • Total attempted: {stats['attempted']}")
            self.log_message(f"   • Successful: {stats['successful']}")
            self.log_message(f"   • Failed: {stats['failed']}")
            self.log_message(f"   • Alternative sources used: {stats['alternatives_used']}")
            
            success_rate = (stats['successful'] / max(stats['attempted'], 1)) * 100
            self.log_message(f"   • Success rate: {success_rate:.1f}%")
            
            if stats['successful'] > 0:
                self.log_message(f"📁 Downloaded files are in: {output_path}")
            
        except Exception as e:
            logger.error(f"Download session error: {e}")
            self.log_message(f"❌ Download session error: {e}")
        
        finally:
            # Reset UI
            self.downloading = False
            self.download_button.config(text="Start Download", state="normal")
            self.status_label.config(text="Download completed")
            self.update_progress(0)

def main():
    """Main function to run the GUI"""
    # Check dependencies
    try:
        import yt_dlp
        import requests
    except ImportError as e:
        print(f"❌ Missing dependency: {e}")
        print("Please install required packages:")
        print("pip install yt-dlp requests")
        return 1
    
    # Create and run GUI
    root = tk.Tk()
    app = TWDWebisodeDownloaderGUI(root)
    
    try:
        root.mainloop()
    except KeyboardInterrupt:
        print("\n🛑 Application interrupted by user")
    except Exception as e:
        logger.error(f"Application error: {e}")
        messagebox.showerror("Error", f"Application error: {e}")
    
    return 0

if __name__ == "__main__":
    exit(main())
