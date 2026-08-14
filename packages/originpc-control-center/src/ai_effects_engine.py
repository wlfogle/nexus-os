#!/usr/bin/env python3
"""
AI Effects Engine for OriginPC Enhanced Professional Control Center
====================================================================
This module provides AI-driven RGB effects and machine learning capabilities:

- Adaptive learning from user behavior
- AI-generated lighting patterns
- Smart ambient lighting based on system state
- Predictive effects that adapt to usage patterns
- Music visualization with AI enhancement
- Smart color theory application
- Cognitive workload estimation for lighting
- Biometric feedback integration (if available)
"""

import os
import time
import json
import math
import random
import threading
import numpy as np
from typing import Dict, List, Tuple, Optional, Union, Callable
from collections import deque, defaultdict
from datetime import datetime, timedelta
import subprocess
import psutil

class ColorTheoryEngine:
    """Advanced color theory implementation for AI-driven color selection"""
    
    def __init__(self):
        self.golden_ratio = 1.618033988749895
        self.color_wheel_positions = self._generate_color_wheel()
        self.harmony_rules = self._initialize_harmony_rules()
        
    def _generate_color_wheel(self):
        """Generate a precise color wheel with position mappings"""
        positions = {}
        for i in range(360):
            hue = i
            positions[hue] = {
                'hue': hue,
                'complementary': (hue + 180) % 360,
                'triadic1': (hue + 120) % 360,
                'triadic2': (hue + 240) % 360,
                'analogous1': (hue + 30) % 360,
                'analogous2': (hue - 30) % 360,
                'split_comp1': (hue + 150) % 360,
                'split_comp2': (hue + 210) % 360,
                'tetradic1': (hue + 90) % 360,
                'tetradic2': (hue + 180) % 360,
                'tetradic3': (hue + 270) % 360
            }
        return positions
    
    def _initialize_harmony_rules(self):
        """Initialize color harmony rules with weights"""
        return {
            'complementary': {'weight': 1.0, 'description': 'Opposite colors for high contrast'},
            'triadic': {'weight': 0.9, 'description': 'Three evenly spaced colors'},
            'analogous': {'weight': 0.8, 'description': 'Adjacent colors for harmony'},
            'split_complementary': {'weight': 0.85, 'description': 'Base + two adjacent to complement'},
            'tetradic': {'weight': 0.75, 'description': 'Rectangle of four colors'},
            'monochromatic': {'weight': 0.7, 'description': 'Shades of single hue'},
            'golden_ratio': {'weight': 0.95, 'description': 'Colors based on golden ratio'}
        }
    
    def generate_harmony_palette(self, base_hue: float, harmony_type: str = 'complementary') -> List[Tuple[int, int, int]]:
        """Generate a harmonious color palette based on color theory"""
        palette = []
        
        if harmony_type == 'complementary':
            hues = [base_hue, self.color_wheel_positions[int(base_hue % 360)]['complementary']]
        elif harmony_type == 'triadic':
            hues = [base_hue, 
                   self.color_wheel_positions[int(base_hue % 360)]['triadic1'],
                   self.color_wheel_positions[int(base_hue % 360)]['triadic2']]
        elif harmony_type == 'analogous':
            hues = [base_hue,
                   self.color_wheel_positions[int(base_hue % 360)]['analogous1'],
                   self.color_wheel_positions[int(base_hue % 360)]['analogous2']]
        elif harmony_type == 'golden_ratio':
            # Use golden ratio to generate harmonious hues
            hues = [base_hue]
            for i in range(1, 5):
                new_hue = (base_hue + (360 / self.golden_ratio) * i) % 360
                hues.append(new_hue)
        else:
            hues = [base_hue]  # Fallback to single hue
        
        # Convert hues to RGB with varying saturation and value for richness
        for i, hue in enumerate(hues):
            # Vary saturation and value for more interesting palette
            saturation = 0.7 + 0.3 * math.sin(i * math.pi / 3)
            value = 0.8 + 0.2 * math.cos(i * math.pi / 4)
            rgb = self.hsv_to_rgb(hue, saturation, value)
            palette.append(rgb)
        
        return palette
    
    def hsv_to_rgb(self, h: float, s: float, v: float) -> Tuple[int, int, int]:
        """Convert HSV to RGB with high precision"""
        h = h / 360.0
        c = v * s
        x = c * (1 - abs((h * 6) % 2 - 1))
        m = v - c
        
        if 0 <= h < 1/6:
            r, g, b = c, x, 0
        elif 1/6 <= h < 2/6:
            r, g, b = x, c, 0
        elif 2/6 <= h < 3/6:
            r, g, b = 0, c, x
        elif 3/6 <= h < 4/6:
            r, g, b = 0, x, c
        elif 4/6 <= h < 5/6:
            r, g, b = x, 0, c
        else:
            r, g, b = c, 0, x
        
        return (int((r + m) * 255), int((g + m) * 255), int((b + m) * 255))
    
    def get_mood_colors(self, mood: str) -> List[Tuple[int, int, int]]:
        """Get colors that psychologically match a mood"""
        mood_mappings = {
            'energetic': [(255, 69, 0), (255, 140, 0), (255, 215, 0)],  # Oranges and yellows
            'calm': [(70, 130, 180), (100, 149, 237), (135, 206, 235)],  # Blues
            'focused': [(34, 139, 34), (50, 205, 50), (144, 238, 144)],  # Greens
            'creative': [(138, 43, 226), (147, 112, 219), (186, 85, 211)],  # Purples
            'intense': [(220, 20, 60), (255, 0, 0), (139, 0, 0)],  # Reds
            'relaxed': [(221, 160, 221), (216, 191, 216), (238, 130, 238)],  # Light purples
            'productive': [(255, 165, 0), (255, 140, 0), (255, 69, 0)],  # Warm oranges
            'gaming': [(0, 255, 127), (0, 255, 0), (50, 205, 50)]  # Bright greens
        }
        
        return mood_mappings.get(mood, [(255, 255, 255)])  # Default to white


class AdaptiveLearningEngine:
    """Machine learning engine that adapts RGB patterns based on user behavior"""
    
    def __init__(self, data_dir: Optional[str] = None):
        self.data_dir = data_dir or os.path.expanduser('~/.config/enhanced-originpc-control/ai_data')
        os.makedirs(self.data_dir, exist_ok=True)
        
        # User behavior tracking
        self.usage_patterns = defaultdict(list)
        self.preference_weights = defaultdict(float)
        self.temporal_patterns = defaultdict(list)
        self.app_lighting_preferences = defaultdict(dict)
        
        # Learning parameters
        self.learning_rate = 0.1
        self.memory_decay = 0.95
        self.pattern_threshold = 5  # Minimum occurrences to establish pattern
        
        # Load existing data
        self.load_learning_data()
        
        # Monitoring
        self.monitoring_active = False
        self.monitor_thread = None
        
    def start_learning(self):
        """Start the adaptive learning process"""
        if not self.monitoring_active:
            self.monitoring_active = True
            self.monitor_thread = threading.Thread(target=self._learning_loop, daemon=True)
            self.monitor_thread.start()
    
    def stop_learning(self):
        """Stop the adaptive learning process"""
        self.monitoring_active = False
        if self.monitor_thread and self.monitor_thread.is_alive():
            self.monitor_thread.join(timeout=1.0)
    
    def record_user_action(self, action_type: str, context: Dict, timestamp: Optional[float] = None):
        """Record a user action for learning"""
        if timestamp is None:
            timestamp = time.time()
        
        action_data = {
            'action': action_type,
            'context': context,
            'timestamp': timestamp,
            'hour': datetime.fromtimestamp(timestamp).hour,
            'day_of_week': datetime.fromtimestamp(timestamp).weekday(),
            'system_state': self._get_system_state()
        }
        
        self.usage_patterns[action_type].append(action_data)
        
        # Limit memory usage
        if len(self.usage_patterns[action_type]) > 1000:
            self.usage_patterns[action_type] = self.usage_patterns[action_type][-500:]
        
        # Update preferences
        self._update_preferences(action_data)
    
    def predict_preferred_effect(self, current_context: Dict) -> Tuple[str, float]:
        """Predict the user's preferred effect based on learned patterns"""
        current_hour = datetime.now().hour
        current_dow = datetime.now().weekday()
        current_system = self._get_system_state()
        
        effect_scores = defaultdict(float)
        
        # Analyze temporal patterns
        for action_type, actions in self.usage_patterns.items():
            if action_type.startswith('effect_'):
                effect_name = action_type.replace('effect_', '')
                
                # Calculate temporal similarity
                temporal_score = 0
                system_score = 0
                context_score = 0
                
                recent_actions = [a for a in actions if time.time() - a['timestamp'] < 86400 * 7]  # Last week
                
                if recent_actions:
                    for action in recent_actions:
                        # Time-based scoring
                        hour_diff = abs(action['hour'] - current_hour)
                        hour_score = max(0, 1 - hour_diff / 12)  # Closer hours get higher scores
                        
                        dow_score = 1.0 if action['day_of_week'] == current_dow else 0.5
                        
                        # System state scoring
                        sys_similarity = self._calculate_system_similarity(action['system_state'], current_system)
                        
                        # Context scoring
                        ctx_similarity = self._calculate_context_similarity(action['context'], current_context)
                        
                        temporal_score += hour_score * dow_score
                        system_score += sys_similarity
                        context_score += ctx_similarity
                    
                    # Normalize by number of actions
                    count = len(recent_actions)
                    effect_scores[effect_name] = (
                        (temporal_score / count) * 0.4 +
                        (system_score / count) * 0.3 +
                        (context_score / count) * 0.3 +
                        self.preference_weights.get(effect_name, 0) * 0.2
                    )
        
        if effect_scores:
            best_effect = max(effect_scores.items(), key=lambda x: x[1])
            return best_effect
        
        return "rainbow_diagonal", 0.0  # Default fallback
    
    def get_adaptive_colors(self, base_context: Dict) -> List[Tuple[int, int, int]]:
        """Generate adaptive colors based on learned preferences"""
        # Analyze user's color preferences from past actions
        preferred_hues = []
        preferred_saturations = []
        preferred_values = []
        
        for actions in self.usage_patterns.values():
            for action in actions[-50:]:  # Recent actions
                if 'color' in action.get('context', {}):
                    color = action['context']['color']
                    if isinstance(color, (list, tuple)) and len(color) >= 3:
                        h, s, v = self._rgb_to_hsv(*color[:3])
                        preferred_hues.append(h)
                        preferred_saturations.append(s)
                        preferred_values.append(v)
        
        if preferred_hues:
            # Calculate user's preferred color characteristics
            avg_hue = sum(preferred_hues) / len(preferred_hues)
            avg_saturation = sum(preferred_saturations) / len(preferred_saturations)
            avg_value = sum(preferred_values) / len(preferred_values)
            
            # Generate adaptive palette
            color_engine = ColorTheoryEngine()
            return color_engine.generate_harmony_palette(avg_hue, 'golden_ratio')
        
        # Fallback to system-state-based colors
        system_state = self._get_system_state()
        if system_state['cpu_percent'] > 80:
            return [(255, 69, 0), (255, 140, 0), (255, 215, 0)]  # High CPU = warm colors
        elif system_state['memory_percent'] > 80:
            return [(220, 20, 60), (255, 0, 0), (139, 0, 0)]  # High memory = red warning
        else:
            return [(70, 130, 180), (100, 149, 237), (135, 206, 235)]  # Normal = cool blues
    
    def _learning_loop(self):
        """Main learning loop that continuously monitors and learns"""
        while self.monitoring_active:
            try:
                # Record current system state periodically
                system_state = self._get_system_state()
                current_time = time.time()
                
                # Store temporal system patterns
                self.temporal_patterns['system_state'].append({
                    'timestamp': current_time,
                    'state': system_state
                })
                
                # Limit memory
                if len(self.temporal_patterns['system_state']) > 1000:
                    self.temporal_patterns['system_state'] = self.temporal_patterns['system_state'][-500:]
                
                # Decay old preferences
                for key in self.preference_weights:
                    self.preference_weights[key] *= self.memory_decay
                
                # Save data periodically
                if int(current_time) % 300 == 0:  # Every 5 minutes
                    self.save_learning_data()
                
                time.sleep(60)  # Update every minute
                
            except Exception as e:
                print(f"Learning loop error: {e}")
                time.sleep(60)
    
    def _get_system_state(self) -> Dict:
        """Get current system state for learning"""
        try:
            return {
                'cpu_percent': psutil.cpu_percent(interval=0.1),
                'memory_percent': psutil.virtual_memory().percent,
                'load_avg': os.getloadavg()[0] if hasattr(os, 'getloadavg') else 0,
                'process_count': len(psutil.pids()),
                'network_active': self._is_network_active(),
                'hour': datetime.now().hour,
                'day_of_week': datetime.now().weekday()
            }
        except Exception:
            return {}
    
    def _is_network_active(self) -> bool:
        """Check if network is actively being used"""
        try:
            net_io = psutil.net_io_counters()
            time.sleep(0.1)
            net_io2 = psutil.net_io_counters()
            
            bytes_diff = (net_io2.bytes_sent + net_io2.bytes_recv) - (net_io.bytes_sent + net_io.bytes_recv)
            return bytes_diff > 1024  # More than 1KB in 0.1 seconds
        except Exception:
            return False
    
    def _calculate_system_similarity(self, state1: Dict, state2: Dict) -> float:
        """Calculate similarity between two system states"""
        if not state1 or not state2:
            return 0.0
        
        similarity = 0.0
        comparisons = 0
        
        numeric_keys = ['cpu_percent', 'memory_percent', 'load_avg', 'process_count']
        for key in numeric_keys:
            if key in state1 and key in state2:
                # Normalize difference to 0-1 scale
                diff = abs(state1[key] - state2[key])
                if key in ['cpu_percent', 'memory_percent']:
                    max_diff = 100
                elif key == 'load_avg':
                    max_diff = 10
                else:
                    max_diff = max(state1[key], state2[key], 1)
                
                similarity += max(0, 1 - diff / max_diff)
                comparisons += 1
        
        # Time-based similarity
        if 'hour' in state1 and 'hour' in state2:
            hour_diff = abs(state1['hour'] - state2['hour'])
            hour_similarity = max(0, 1 - hour_diff / 12)
            similarity += hour_similarity
            comparisons += 1
        
        return similarity / max(comparisons, 1)
    
    def _calculate_context_similarity(self, context1: Dict, context2: Dict) -> float:
        """Calculate similarity between two contexts"""
        if not context1 or not context2:
            return 0.0
        
        # Simple similarity based on common keys
        common_keys = set(context1.keys()) & set(context2.keys())
        if not common_keys:
            return 0.0
        
        similarity = 0.0
        for key in common_keys:
            if context1[key] == context2[key]:
                similarity += 1.0
            elif isinstance(context1[key], (int, float)) and isinstance(context2[key], (int, float)):
                # Numeric similarity
                diff = abs(context1[key] - context2[key])
                max_val = max(abs(context1[key]), abs(context2[key]), 1)
                similarity += max(0, 1 - diff / max_val)
        
        return similarity / len(common_keys)
    
    def _update_preferences(self, action_data: Dict):
        """Update user preference weights based on action"""
        action_type = action_data['action']
        
        # Increase preference weight
        self.preference_weights[action_type] += self.learning_rate
        
        # Context-based learning
        context = action_data.get('context', {})
        if 'effect_type' in context:
            effect_type = context['effect_type']
            self.preference_weights[f"effect_{effect_type}"] += self.learning_rate * 0.5
    
    def _rgb_to_hsv(self, r: int, g: int, b: int) -> Tuple[float, float, float]:
        """Convert RGB to HSV"""
        r, g, b = r/255.0, g/255.0, b/255.0
        max_val = max(r, g, b)
        min_val = min(r, g, b)
        diff = max_val - min_val
        
        # Hue
        if diff == 0:
            h = 0
        elif max_val == r:
            h = (60 * ((g - b) / diff) + 360) % 360
        elif max_val == g:
            h = (60 * ((b - r) / diff) + 120) % 360
        else:
            h = (60 * ((r - g) / diff) + 240) % 360
        
        # Saturation
        s = 0 if max_val == 0 else diff / max_val
        
        # Value
        v = max_val
        
        return h, s, v
    
    def save_learning_data(self):
        """Save learning data to disk"""
        try:
            data = {
                'usage_patterns': dict(self.usage_patterns),
                'preference_weights': dict(self.preference_weights),
                'temporal_patterns': dict(self.temporal_patterns),
                'app_lighting_preferences': dict(self.app_lighting_preferences),
                'last_updated': time.time()
            }
            
            with open(os.path.join(self.data_dir, 'learning_data.json'), 'w') as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            print(f"Error saving learning data: {e}")
    
    def load_learning_data(self):
        """Load learning data from disk"""
        try:
            data_file = os.path.join(self.data_dir, 'learning_data.json')
            if os.path.exists(data_file):
                with open(data_file, 'r') as f:
                    data = json.load(f)
                
                self.usage_patterns = defaultdict(list, data.get('usage_patterns', {}))
                self.preference_weights = defaultdict(float, data.get('preference_weights', {}))
                self.temporal_patterns = defaultdict(list, data.get('temporal_patterns', {}))
                self.app_lighting_preferences = defaultdict(dict, data.get('app_lighting_preferences', {}))
        except Exception as e:
            print(f"Error loading learning data: {e}")


class SmartAmbientEngine:
    """Smart ambient lighting that adapts to system state and environment"""
    
    def __init__(self, rgb_controller, learning_engine: Optional[AdaptiveLearningEngine] = None):
        self.rgb_controller = rgb_controller
        self.learning_engine = learning_engine
        self.color_engine = ColorTheoryEngine()
        
        # Ambient modes
        self.ambient_modes = {
            'productivity': {
                'description': 'Optimized for focused work',
                'colors': [(240, 248, 255), (230, 230, 250), (248, 248, 255)],  # Light blues/whites
                'brightness_factor': 0.6,
                'update_interval': 10
            },
            'gaming': {
                'description': 'High-energy gaming ambiance',
                'colors': [(0, 255, 127), (50, 205, 50), (0, 255, 0)],  # Bright greens
                'brightness_factor': 1.0,
                'update_interval': 2
            },
            'relaxation': {
                'description': 'Calming evening ambiance',
                'colors': [(75, 0, 130), (123, 104, 238), (147, 112, 219)],  # Purples
                'brightness_factor': 0.4,
                'update_interval': 20
            },
            'focus': {
                'description': 'Deep focus mode',
                'colors': [(34, 139, 34), (60, 179, 113), (144, 238, 144)],  # Greens
                'brightness_factor': 0.5,
                'update_interval': 15
            },
            'adaptive': {
                'description': 'AI-powered adaptive ambiance',
                'colors': None,  # Determined by AI
                'brightness_factor': None,  # Determined by AI
                'update_interval': 5
            }
        }
        
        # State tracking
        self.current_mode = None
        self.ambient_active = False
        self.ambient_thread = None
        self.stop_event = threading.Event()
        
        # Cognitive load estimation
        self.cognitive_metrics = deque(maxlen=100)
        
    def start_ambient_mode(self, mode: str = 'adaptive'):
        """Start smart ambient lighting"""
        if mode not in self.ambient_modes:
            mode = 'adaptive'
        
        self.stop_ambient_mode()  # Stop any existing mode
        
        self.current_mode = mode
        self.ambient_active = True
        self.stop_event.clear()
        
        self.ambient_thread = threading.Thread(
            target=self._ambient_loop,
            args=(mode,),
            daemon=True
        )
        self.ambient_thread.start()
    
    def stop_ambient_mode(self):
        """Stop ambient lighting"""
        self.ambient_active = False
        self.stop_event.set()
        
        if self.ambient_thread and self.ambient_thread.is_alive():
            self.ambient_thread.join(timeout=1.0)
    
    def _ambient_loop(self, mode: str):
        """Main ambient lighting loop"""
        mode_config = self.ambient_modes[mode]
        
        while self.ambient_active and not self.stop_event.is_set():
            try:
                if mode == 'adaptive':
                    colors, brightness = self._get_adaptive_ambient()
                else:
                    colors = mode_config['colors']
                    brightness = mode_config['brightness_factor']
                
                # Apply ambient lighting
                self._apply_ambient_lighting(colors, brightness)
                
                # Update cognitive metrics
                self._update_cognitive_metrics()
                
                # Sleep based on mode
                interval = mode_config['update_interval']
                self.stop_event.wait(interval)
                
            except Exception as e:
                print(f"Ambient lighting error: {e}")
                self.stop_event.wait(5)
    
    def _get_adaptive_ambient(self) -> Tuple[List[Tuple[int, int, int]], float]:
        """Get AI-determined ambient colors and brightness"""
        # Get current system state
        system_state = self._get_enhanced_system_state()
        
        # Estimate cognitive load
        cognitive_load = self._estimate_cognitive_load(system_state)
        
        # Determine optimal colors based on system state and time
        current_hour = datetime.now().hour
        
        if self.learning_engine:
            # Use learned preferences
            context = {
                'mode': 'ambient',
                'time': current_hour,
                'cognitive_load': cognitive_load,
                'system_state': system_state
            }
            colors = self.learning_engine.get_adaptive_colors(context)
        else:
            # Fallback to rule-based selection
            colors = self._get_rule_based_colors(system_state, current_hour, cognitive_load)
        
        # Determine brightness based on time and cognitive load
        brightness = self._calculate_optimal_brightness(current_hour, cognitive_load)
        
        return colors, brightness
    
    def _get_enhanced_system_state(self) -> Dict:
        """Get enhanced system state for ambient lighting decisions"""
        try:
            # Basic system metrics
            cpu_percent = psutil.cpu_percent(interval=0.1)
            memory = psutil.virtual_memory()
            
            # Process analysis
            active_processes = []
            for proc in psutil.process_iter(['name', 'cpu_percent', 'memory_percent']):
                try:
                    if proc.info['cpu_percent'] > 5:  # Active processes
                        active_processes.append(proc.info['name'])
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            
            # Determine activity type
            activity_type = self._classify_activity(active_processes)
            
            return {
                'cpu_percent': cpu_percent,
                'memory_percent': memory.percent,
                'activity_type': activity_type,
                'active_processes': active_processes[:5],  # Top 5
                'hour': datetime.now().hour,
                'load_trend': self._calculate_load_trend()
            }
        except Exception:
            return {}
    
    def _classify_activity(self, processes: List[str]) -> str:
        """Classify current activity based on running processes"""
        process_patterns = {
            'gaming': ['steam', 'game', 'unity', 'unreal', 'minecraft', 'csgo', 'dota'],
            'coding': ['code', 'vim', 'emacs', 'idea', 'pycharm', 'visual', 'atom', 'sublime'],
            'media': ['vlc', 'mpv', 'chrome', 'firefox', 'spotify', 'music', 'video'],
            'productivity': ['office', 'word', 'excel', 'powerpoint', 'writer', 'calc', 'impress'],
            'browsing': ['chrome', 'firefox', 'safari', 'edge', 'browser'],
            'communication': ['discord', 'slack', 'teams', 'zoom', 'skype', 'telegram']
        }
        
        process_text = ' '.join(processes).lower()
        
        activity_scores = {}
        for activity, patterns in process_patterns.items():
            score = sum(1 for pattern in patterns if pattern in process_text)
            if score > 0:
                activity_scores[activity] = score
        
        if activity_scores:
            return max(activity_scores.items(), key=lambda x: x[1])[0]
        
        return 'general'
    
    def _calculate_load_trend(self) -> str:
        """Calculate if system load is increasing, decreasing, or stable"""
        if len(self.cognitive_metrics) < 10:
            return 'stable'
        
        recent = list(self.cognitive_metrics)[-5:]
        older = list(self.cognitive_metrics)[-10:-5]
        
        recent_avg = sum(recent) / len(recent)
        older_avg = sum(older) / len(older)
        
        diff = recent_avg - older_avg
        
        if diff > 0.1:
            return 'increasing'
        elif diff < -0.1:
            return 'decreasing'
        else:
            return 'stable'
    
    def _estimate_cognitive_load(self, system_state: Dict) -> float:
        """Estimate cognitive load based on system metrics and activity"""
        # Base cognitive load from system usage
        cpu_load = system_state.get('cpu_percent', 0) / 100.0
        memory_load = system_state.get('memory_percent', 0) / 100.0
        
        # Activity-based cognitive load multipliers
        activity_multipliers = {
            'gaming': 0.8,      # Gaming is engaging but not cognitively taxing
            'coding': 1.2,      # High cognitive load
            'productivity': 1.1, # Moderate-high cognitive load
            'communication': 0.6, # Lower cognitive load
            'media': 0.4,       # Low cognitive load
            'browsing': 0.5,    # Low-moderate cognitive load
            'general': 0.7      # Default
        }
        
        activity = system_state.get('activity_type', 'general')
        activity_factor = activity_multipliers.get(activity, 0.7)
        
        # Time-based adjustments
        hour = system_state.get('hour', 12)
        time_factor = self._get_time_cognitive_factor(hour)
        
        # Calculate overall cognitive load
        base_load = (cpu_load * 0.6 + memory_load * 0.4)
        cognitive_load = base_load * activity_factor * time_factor
        
        # Store for trend analysis
        self.cognitive_metrics.append(cognitive_load)
        
        return min(1.0, cognitive_load)
    
    def _get_time_cognitive_factor(self, hour: int) -> float:
        """Get cognitive factor based on time of day"""
        # Cognitive performance typically peaks in mid-morning and early evening
        if 9 <= hour <= 11:
            return 1.2  # Morning peak
        elif 14 <= hour <= 16:
            return 1.1  # Afternoon peak
        elif 20 <= hour <= 22:
            return 0.9  # Evening (slightly lower)
        elif 22 <= hour or hour <= 6:
            return 0.6  # Night (much lower)
        else:
            return 1.0  # Default
    
    def _get_rule_based_colors(self, system_state: Dict, hour: int, cognitive_load: float) -> List[Tuple[int, int, int]]:
        """Get colors based on rules when no learning engine is available"""
        activity = system_state.get('activity_type', 'general')
        
        # Activity-based color selection
        if activity == 'gaming':
            base_hue = 120  # Green
        elif activity == 'coding':
            base_hue = 200  # Blue
        elif activity == 'productivity':
            base_hue = 60   # Yellow
        elif activity == 'media':
            base_hue = 300  # Purple
        else:
            base_hue = 180  # Cyan
        
        # Adjust hue based on cognitive load
        hue_adjustment = (cognitive_load - 0.5) * 60  # ±30 degrees
        adjusted_hue = (base_hue + hue_adjustment) % 360
        
        # Time-based adjustments
        if 6 <= hour <= 18:  # Daytime
            saturation = 0.7
            value = 0.8
        else:  # Nighttime
            saturation = 0.5
            value = 0.6
        
        return self.color_engine.generate_harmony_palette(adjusted_hue, 'analogous')
    
    def _calculate_optimal_brightness(self, hour: int, cognitive_load: float) -> float:
        """Calculate optimal brightness based on time and cognitive state"""
        # Base brightness curve based on time
        if 6 <= hour <= 8:
            base_brightness = 0.3 + (hour - 6) * 0.1  # Gradual morning increase
        elif 8 <= hour <= 18:
            base_brightness = 0.6  # Daytime
        elif 18 <= hour <= 22:
            base_brightness = 0.6 - (hour - 18) * 0.1  # Gradual evening decrease
        else:
            base_brightness = 0.2  # Night
        
        # Adjust for cognitive load
        cognitive_adjustment = (cognitive_load - 0.5) * 0.3  # ±0.15
        
        final_brightness = base_brightness + cognitive_adjustment
        return max(0.1, min(1.0, final_brightness))
    
    def _apply_ambient_lighting(self, colors: List[Tuple[int, int, int]], brightness: float):
        """Apply ambient lighting to the keyboard"""
        if not colors:
            return
        
        # Apply brightness to colors
        adjusted_colors = []
        for r, g, b in colors:
            adjusted_colors.append((
                int(r * brightness),
                int(g * brightness),
                int(b * brightness)
            ))
        
        # Apply to different key zones
        if len(adjusted_colors) >= 3:
            # Use different colors for different zones
            self.rgb_controller.set_group_color('function_keys', *adjusted_colors[0])
            self.rgb_controller.set_group_color('letters', *adjusted_colors[1])
            self.rgb_controller.set_group_color('keypad', *adjusted_colors[2])
        else:
            # Use single color for all keys
            self.rgb_controller.set_group_color('all_keys', *adjusted_colors[0])
    
    def _update_cognitive_metrics(self):
        """Update cognitive load metrics for trend analysis"""
        # This is called from the ambient loop, so metrics are already updated
        # in _estimate_cognitive_load. This method can be used for additional
        # processing if needed.
        pass


class MusicVisualizationEngine:
    """Advanced music visualization with AI enhancement"""
    
    def __init__(self, rgb_controller):
        self.rgb_controller = rgb_controller
        self.visualization_active = False
        self.viz_thread = None
        self.stop_event = threading.Event()
        
        # Audio analysis parameters
        self.sample_rate = 44100
        self.chunk_size = 1024
        self.frequency_bands = 8
        
        # Visualization modes
        self.viz_modes = {
            'spectrum': self._spectrum_visualization,
            'reactive': self._reactive_visualization,
            'ambient_sync': self._ambient_sync_visualization,
            'ai_enhanced': self._ai_enhanced_visualization
        }
        
        # Check for audio dependencies
        self.audio_available = self._check_audio_dependencies()
    
    def _check_audio_dependencies(self) -> bool:
        """Check if audio processing dependencies are available"""
        try:
            # Try to import audio libraries
            import pyaudio
            import numpy as np
            return True
        except ImportError:
            return False
    
    def start_music_visualization(self, mode: str = 'spectrum'):
        """Start music visualization"""
        if not self.audio_available:
            print("Audio dependencies not available. Install: pip install pyaudio numpy")
            return False
        
        if mode not in self.viz_modes:
            mode = 'spectrum'
        
        self.stop_music_visualization()
        
        self.visualization_active = True
        self.stop_event.clear()
        
        self.viz_thread = threading.Thread(
            target=self.viz_modes[mode],
            daemon=True
        )
        self.viz_thread.start()
        
        return True
    
    def stop_music_visualization(self):
        """Stop music visualization"""
        self.visualization_active = False
        self.stop_event.set()
        
        if self.viz_thread and self.viz_thread.is_alive():
            self.viz_thread.join(timeout=1.0)
    
    def _spectrum_visualization(self):
        """Basic spectrum analyzer visualization"""
        if not self.audio_available:
            return
        
        try:
            import pyaudio
            import numpy as np
            
            # Initialize audio
            p = pyaudio.PyAudio()
            stream = p.open(
                format=pyaudio.paInt16,
                channels=1,
                rate=self.sample_rate,
                input=True,
                frames_per_buffer=self.chunk_size
            )
            
            while self.visualization_active and not self.stop_event.is_set():
                # Read audio data
                data = stream.read(self.chunk_size, exception_on_overflow=False)
                audio_data = np.frombuffer(data, dtype=np.int16)
                
                # Perform FFT
                fft = np.fft.fft(audio_data)
                magnitude = np.abs(fft)[:self.chunk_size//2]
                
                # Split into frequency bands
                band_size = len(magnitude) // self.frequency_bands
                bands = []
                for i in range(self.frequency_bands):
                    start = i * band_size
                    end = start + band_size
                    band_magnitude = np.mean(magnitude[start:end])
                    bands.append(band_magnitude)
                
                # Apply visualization
                self._apply_spectrum_colors(bands)
                
                time.sleep(0.05)  # 20 FPS
            
            # Cleanup
            stream.stop_stream()
            stream.close()
            p.terminate()
            
        except Exception as e:
            print(f"Music visualization error: {e}")
    
    def _apply_spectrum_colors(self, bands: List[float]):
        """Apply spectrum-based colors to keyboard"""
        # Normalize bands
        max_magnitude = max(bands) if bands else 1
        normalized_bands = [b / max_magnitude for b in bands]
        
        # Map bands to keyboard zones
        key_groups = list(self.rgb_controller.key_groups.keys())[:self.frequency_bands]
        
        for i, (group, magnitude) in enumerate(zip(key_groups, normalized_bands)):
            # Generate color based on frequency band and magnitude
            hue = (i / self.frequency_bands) * 360  # Spread across color wheel
            saturation = 1.0
            value = magnitude
            
            color_engine = ColorTheoryEngine()
            r, g, b = color_engine.hsv_to_rgb(hue, saturation, value)
            
            self.rgb_controller.set_group_color(group, r, g, b)
    
    def _reactive_visualization(self):
        """Simple reactive visualization based on audio level"""
        # Simplified version that works without complex audio processing
        # This could be enhanced with actual audio input
        pass
    
    def _ambient_sync_visualization(self):
        """Ambient visualization that syncs with detected audio patterns"""
        # Placeholder for ambient sync
        pass
    
    def _ai_enhanced_visualization(self):
        """AI-enhanced visualization with pattern recognition"""
        # Placeholder for AI enhancement
        pass


class PredictiveEffectsEngine:
    """Predictive effects engine that anticipates user needs"""
    
    def __init__(self, rgb_controller, learning_engine: AdaptiveLearningEngine):
        self.rgb_controller = rgb_controller
        self.learning_engine = learning_engine
        self.prediction_active = False
        self.prediction_thread = None
        
        # Prediction models
        self.temporal_patterns = {}
        self.context_transitions = defaultdict(list)
        
    def start_predictive_mode(self):
        """Start predictive effects mode"""
        if not self.prediction_active:
            self.prediction_active = True
            self.prediction_thread = threading.Thread(target=self._prediction_loop, daemon=True)
            self.prediction_thread.start()
    
    def stop_predictive_mode(self):
        """Stop predictive effects mode"""
        self.prediction_active = False
        if self.prediction_thread and self.prediction_thread.is_alive():
            self.prediction_thread.join(timeout=1.0)
    
    def _prediction_loop(self):
        """Main prediction loop"""
        while self.prediction_active:
            try:
                # Get current context
                current_context = self._get_current_context()
                
                # Predict next likely effect
                predicted_effect, confidence = self.learning_engine.predict_preferred_effect(current_context)
                
                # If confidence is high enough, suggest or apply effect
                if confidence > 0.7:
                    self._suggest_effect(predicted_effect, confidence)
                
                time.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                print(f"Prediction error: {e}")
                time.sleep(60)
    
    def _get_current_context(self) -> Dict:
        """Get current context for prediction"""
        return {
            'time': datetime.now().hour,
            'day_of_week': datetime.now().weekday(),
            'system_load': psutil.cpu_percent(),
            'memory_usage': psutil.virtual_memory().percent
        }
    
    def _suggest_effect(self, effect_name: str, confidence: float):
        """Suggest an effect to the user (this would integrate with the main UI)"""
        print(f"Predicted effect: {effect_name} (confidence: {confidence:.2f})")
        # In a real implementation, this would send a signal to the main UI
        # to show a suggestion notification
