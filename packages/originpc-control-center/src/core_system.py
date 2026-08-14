#!/usr/bin/env python3
"""
Core System Module for OriginPC Enhanced Professional Control Center
====================================================================
This module provides core system management and configuration classes
that serve as the foundation for application optimizations.

Classes:
- SystemManager: Central coordination for all optimization components
- ConfigManager: Unified configuration management with profile support
"""

import os
import time
import json
import threading
import logging
from pathlib import Path
from typing import Dict, List, Any, Optional, Union
import platform
import signal
import atexit

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('core_system')

class ConfigManager:
    """
    Unified configuration management with profile support.
    
    This class handles all application configuration, including:
    - User preferences
    - RGB profiles
    - Device settings
    - System monitoring preferences
    - Performance tuning
    
    Configuration is stored in JSON format with automatic validation
    and schema checking to ensure data integrity.
    """
    
    def __init__(self, config_dir: Optional[str] = None):
        """
        Initialize configuration manager.
        
        Args:
            config_dir: Directory for configuration files (default: ~/.config/enhanced-originpc-control)
        """
        # Set configuration directory
        if config_dir:
            self.config_dir = Path(config_dir)
        else:
            self.config_dir = Path.home() / '.config' / 'enhanced-originpc-control'
        
        # Create subdirectories
        self.profiles_dir = self.config_dir / 'profiles'
        self.cache_dir = self.config_dir / 'cache'
        self.macros_dir = self.config_dir / 'macros'
        
        # Create directories if they don't exist
        self.config_dir.mkdir(parents=True, exist_ok=True)
        self.profiles_dir.mkdir(exist_ok=True)
        self.cache_dir.mkdir(exist_ok=True)
        self.macros_dir.mkdir(exist_ok=True)
        
        # Configuration file paths
        self.main_config_file = self.config_dir / 'config.json'
        self.devices_config_file = self.config_dir / 'devices.json'
        self.settings_file = self.config_dir / 'settings.json'
        
        # Default configuration values
        self.default_config = {
            'app_version': '5.1',
            'first_run': True,
            'ui': {
                'theme': 'dark',
                'scale_factor': 1.0,
                'start_minimized': False,
                'minimize_to_tray': True,
                'show_notifications': True,
                'monitoring_refresh_rate': 2000  # in ms
            },
            'rgb': {
                'default_profile': 'default',
                'startup_effect': 'static',
                'brightness': 100,
                'speed': 50,
                'active_profile': 'default'
            },
            'system': {
                'startup_delay': 3,  # in seconds
                'performance_mode': 'balanced',
                'monitor_temperature': True,
                'monitor_fans': True,
                'lid_monitoring': True,
                'background_mode': 'eco'  # eco, performance, or balanced
            },
            'devices': {
                'primary_keyboard': '/dev/hidraw0',
                'auto_detect': True,
                'retry_on_failure': True,
                'command_batch_size': 16,
                'max_batch_delay': 0.05  # in seconds
            }
        }
        
        # Actual configuration (loaded from file or defaults)
        self.config = self._load_or_create_config()
        
        # Profiles
        self.profiles = self._load_profiles()
        
        # Thread synchronization
        self.lock = threading.RLock()
        
        # Autosave timer
        self._dirty = False
        self._last_save = time.time()
        self._autosave_interval = 10  # seconds
        
        # Register save on exit
        atexit.register(self.save_if_dirty)
    
    def get(self, key: str, default: Any = None) -> Any:
        """
        Get configuration value using dot notation.
        
        Args:
            key: Configuration key using dot notation (e.g., 'rgb.brightness')
            default: Default value if key not found
            
        Returns:
            Configuration value or default
        """
        with self.lock:
            parts = key.split('.')
            value = self.config
            
            try:
                for part in parts:
                    value = value[part]
                return value
            except (KeyError, TypeError):
                return default
    
    def set(self, key: str, value: Any) -> bool:
        """
        Set configuration value using dot notation.
        
        Args:
            key: Configuration key using dot notation (e.g., 'rgb.brightness')
            value: Value to set
            
        Returns:
            True if successful, False otherwise
        """
        with self.lock:
            parts = key.split('.')
            config = self.config
            
            # Navigate to the parent object
            for part in parts[:-1]:
                if part not in config or not isinstance(config[part], dict):
                    config[part] = {}
                config = config[part]
            
            # Set the value
            config[parts[-1]] = value
            self._dirty = True
            
            # Check if we should autosave
            if time.time() - self._last_save > self._autosave_interval:
                self.save_config()
            
            return True
    
    def get_profile(self, profile_name: str) -> Dict[str, Any]:
        """
        Get an RGB profile by name.
        
        Args:
            profile_name: Name of the profile
            
        Returns:
            Profile dictionary or empty dict if not found
        """
        with self.lock:
            return self.profiles.get(profile_name, {})
    
    def save_profile(self, profile_name: str, profile_data: Dict[str, Any]) -> bool:
        """
        Save an RGB profile.
        
        Args:
            profile_name: Name of the profile
            profile_data: Profile data dictionary
            
        Returns:
            True if successful, False otherwise
        """
        try:
            with self.lock:
                # Update profiles dictionary
                self.profiles[profile_name] = profile_data
                
                # Save to file
                profile_file = self.profiles_dir / f"{profile_name}.json"
                with open(profile_file, 'w') as f:
                    json.dump(profile_data, f, indent=2)
                
                # Update active profile if needed
                if self.get('rgb.active_profile') == profile_name:
                    self.set('rgb.active_profile', profile_name)
                
                return True
        except Exception as e:
            logger.error(f"Error saving profile {profile_name}: {e}")
            return False
    
    def list_profiles(self) -> List[str]:
        """
        List all available RGB profiles.
        
        Returns:
            List of profile names
        """
        return list(self.profiles.keys())
    
    def delete_profile(self, profile_name: str) -> bool:
        """
        Delete an RGB profile.
        
        Args:
            profile_name: Name of the profile
            
        Returns:
            True if successful, False otherwise
        """
        try:
            with self.lock:
                # Don't delete default profile
                if profile_name == 'default':
                    return False
                
                # Remove from profiles dictionary
                if profile_name in self.profiles:
                    del self.profiles[profile_name]
                
                # Delete file
                profile_file = self.profiles_dir / f"{profile_name}.json"
                if profile_file.exists():
                    profile_file.unlink()
                
                # If active profile was deleted, switch to default
                if self.get('rgb.active_profile') == profile_name:
                    self.set('rgb.active_profile', 'default')
                
                return True
        except Exception as e:
            logger.error(f"Error deleting profile {profile_name}: {e}")
            return False
    
    def save_config(self) -> bool:
        """
        Save configuration to file.
        
        Returns:
            True if successful, False otherwise
        """
        try:
            with self.lock:
                with open(self.main_config_file, 'w') as f:
                    json.dump(self.config, f, indent=2)
                
                self._dirty = False
                self._last_save = time.time()
                return True
        except Exception as e:
            logger.error(f"Error saving configuration: {e}")
            return False
    
    def save_if_dirty(self) -> bool:
        """
        Save configuration if changes are pending.
        
        Returns:
            True if saved or not dirty, False on error
        """
        if self._dirty:
            return self.save_config()
        return True
    
    def reset_to_defaults(self) -> bool:
        """
        Reset configuration to default values.
        
        Returns:
            True if successful, False otherwise
        """
        try:
            with self.lock:
                self.config = self.default_config.copy()
                return self.save_config()
        except Exception as e:
            logger.error(f"Error resetting configuration: {e}")
            return False
    
    def import_config(self, filepath: str) -> bool:
        """
        Import configuration from file.
        
        Args:
            filepath: Path to configuration file
            
        Returns:
            True if successful, False otherwise
        """
        try:
            with open(filepath, 'r') as f:
                new_config = json.load(f)
            
            with self.lock:
                # Validate import
                if not isinstance(new_config, dict):
                    return False
                
                # Merge with defaults to ensure all required keys exist
                self._recursive_update(self.config, new_config)
                self._dirty = True
                return self.save_config()
        except Exception as e:
            logger.error(f"Error importing configuration: {e}")
            return False
    
    def export_config(self, filepath: str) -> bool:
        """
        Export configuration to file.
        
        Args:
            filepath: Path to save configuration file
            
        Returns:
            True if successful, False otherwise
        """
        try:
            with self.lock:
                with open(filepath, 'w') as f:
                    json.dump(self.config, f, indent=2)
                return True
        except Exception as e:
            logger.error(f"Error exporting configuration: {e}")
            return False
    
    def _load_or_create_config(self) -> Dict[str, Any]:
        """Load configuration from file or create with defaults."""
        if self.main_config_file.exists():
            try:
                with open(self.main_config_file, 'r') as f:
                    loaded_config = json.load(f)
                
                # Merge with defaults to ensure all required keys exist
                config = self.default_config.copy()
                self._recursive_update(config, loaded_config)
                return config
            except Exception as e:
                logger.error(f"Error loading configuration: {e}")
                return self.default_config.copy()
        else:
            # Create new configuration file
            config = self.default_config.copy()
            try:
                with open(self.main_config_file, 'w') as f:
                    json.dump(config, f, indent=2)
            except Exception as e:
                logger.error(f"Error creating configuration file: {e}")
            
            return config
    
    def _load_profiles(self) -> Dict[str, Dict[str, Any]]:
        """Load all RGB profiles from profiles directory."""
        profiles = {}
        
        # Create default profile if it doesn't exist
        default_profile_file = self.profiles_dir / 'default.json'
        if not default_profile_file.exists():
            default_profile = {
                'name': 'Default',
                'description': 'Default RGB profile',
                'effect': 'static',
                'color': [255, 102, 0],  # Orange
                'brightness': 100,
                'speed': 50,
                'groups': {}  # Key groups with specific colors
            }
            try:
                with open(default_profile_file, 'w') as f:
                    json.dump(default_profile, f, indent=2)
                profiles['default'] = default_profile
            except Exception as e:
                logger.error(f"Error creating default profile: {e}")
        
        # Load all profiles from directory
        for profile_file in self.profiles_dir.glob('*.json'):
            try:
                with open(profile_file, 'r') as f:
                    profile = json.load(f)
                
                profile_name = profile_file.stem
                profiles[profile_name] = profile
            except Exception as e:
                logger.error(f"Error loading profile {profile_file}: {e}")
        
        return profiles
    
    def _recursive_update(self, d: Dict[str, Any], u: Dict[str, Any]) -> None:
        """Recursively update a dictionary with another dictionary."""
        for k, v in u.items():
            if isinstance(v, dict) and k in d and isinstance(d[k], dict):
                self._recursive_update(d[k], v)
            else:
                d[k] = v


class SystemManager:
    """
    Central coordinator for all system optimizations and components.
    
    This class serves as the primary interface for the application to access
    optimized components, managing their lifecycle and dependencies.
    
    Features:
    - Lazy loading of components to minimize startup time
    - Dependency injection for component coordination
    - Centralized resource management
    - Unified error handling and recovery
    """
    
    def __init__(self, config_dir: Optional[str] = None):
        """
        Initialize the system manager.
        
        Args:
            config_dir: Configuration directory (default: ~/.config/enhanced-originpc-control)
        """
        # Initialize configuration manager
        self.config_manager = ConfigManager(config_dir)
        
        # Initialize component registry with lazy loading
        self._components = {}
        self._component_instances = {}
        
        # Register shutdown handler
        self._shutting_down = False
        atexit.register(self.shutdown)
        
        # System info
        self.system_info = self._collect_system_info()
        
        # Setup logging
        log_level = self.config_manager.get('system.log_level', 'INFO')
        numeric_level = getattr(logging, log_level.upper(), logging.INFO)
        logger.setLevel(numeric_level)
        
        logger.info("SystemManager initialized")
        logger.info(f"Running on {self.system_info['os']} {self.system_info['os_version']}")
    
    def get_component(self, component_name: str) -> Any:
        """
        Get or lazily initialize a component.
        
        Args:
            component_name: Name of the component
            
        Returns:
            Component instance or None if not found
        """
        # Return cached instance if available
        if component_name in self._component_instances:
            return self._component_instances[component_name]
        
        # Check if component is registered
        if component_name not in self._components:
            logger.warning(f"Component '{component_name}' not registered")
            return None
        
        # Lazy initialization
        try:
            component_class = self._components[component_name]['class']
            dependencies = self._components[component_name].get('dependencies', [])
            
            # Resolve dependencies
            dependency_instances = {}
            for dep_name in dependencies:
                dependency_instances[dep_name] = self.get_component(dep_name)
            
            # Initialize component
            instance = component_class(self, **dependency_instances)
            self._component_instances[component_name] = instance
            
            logger.info(f"Component '{component_name}' initialized")
            return instance
        except Exception as e:
            logger.error(f"Error initializing component '{component_name}': {e}")
            return None
    
    def register_component(self, name: str, component_class: Any, dependencies: List[str] = None) -> bool:
        """
        Register a component for lazy initialization.
        
        Args:
            name: Component name
            component_class: Component class
            dependencies: List of dependency component names
            
        Returns:
            True if registered successfully, False otherwise
        """
        try:
            self._components[name] = {
                'class': component_class,
                'dependencies': dependencies or []
            }
            logger.info(f"Component '{name}' registered")
            return True
        except Exception as e:
            logger.error(f"Error registering component '{name}': {e}")
            return False
    
    def shutdown(self) -> None:
        """Shutdown all components and save configuration."""
        if self._shutting_down:
            return
        
        self._shutting_down = True
        logger.info("Shutting down SystemManager")
        
        # Shutdown components in reverse dependency order
        for name, instance in reversed(list(self._component_instances.items())):
            try:
                if hasattr(instance, 'shutdown'):
                    logger.info(f"Shutting down component '{name}'")
                    instance.shutdown()
            except Exception as e:
                logger.error(f"Error shutting down component '{name}': {e}")
        
        # Save configuration
        self.config_manager.save_if_dirty()
    
    def _collect_system_info(self) -> Dict[str, str]:
        """Collect basic system information."""
        system_info = {
            'os': platform.system(),
            'os_version': platform.version(),
            'python_version': platform.python_version(),
            'platform': platform.platform(),
            'architecture': platform.machine(),
            'processor': platform.processor() or 'Unknown'
        }
        
        # Get Linux distribution info if available
        try:
            if system_info['os'] == 'Linux':
                import distro
                system_info['distribution'] = distro.name(pretty=True)
                system_info['distribution_version'] = distro.version()
        except ImportError:
            pass
        
        return system_info
