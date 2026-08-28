#!/usr/bin/env python3
"""
Hardware Optimization Module for OriginPC Enhanced Professional Control Center
=============================================================================
This module provides advanced hardware optimization classes that improve
performance, reduce power consumption, and enhance system monitoring.

Classes:
- ThreadPool: Centralized thread management for optimized resource usage
- HardwareOptimizer: Manages CPU/GPU performance states
- RGBDeviceOptimizer: Enhances RGB device communication
- SystemMonitor: Advanced system metrics tracking
- PowerOptimizer: Intelligent power management
"""

import os
import time
import json
import logging
import threading
import subprocess
import concurrent.futures
from pathlib import Path
from typing import Dict, List, Any, Optional, Union, Callable
import platform
import queue
import psutil
import glob
import signal
from collections import deque

# Import our custom modules
try:
    from core_system import SystemManager, ConfigManager
    from optimization_classes import RGBCommandBatcher, SystemInfoCache, DeviceManager
except ImportError:
    # For standalone testing
    SystemManager = object
    ConfigManager = object
    RGBCommandBatcher = object
    SystemInfoCache = object
    DeviceManager = object

# Setup logging
logger = logging.getLogger('hardware_optimizations')


class ThreadPool:
    """
    Centralized thread management for optimized resource usage.
    
    This class manages a pool of worker threads for executing tasks,
    balancing resource usage and responsiveness. It implements:
    - Task prioritization
    - Thread count auto-scaling based on system load
    - Resource throttling during high system usage
    - Adaptive task batching
    """
    
    def __init__(self, system_manager):
        """
        Initialize the thread pool.
        
        Args:
            system_manager: SystemManager instance
        """
        self.system_manager = system_manager
        self.config = system_manager.config_manager
        
        # Get thread pool configuration
        self.min_workers = self.config.get('system.thread_pool.min_workers', 2)
        self.max_workers = self.config.get('system.thread_pool.max_workers', max(4, os.cpu_count() or 4))
        self.task_queue_size = self.config.get('system.thread_pool.queue_size', 100)
        
        # Task queue with priority
        self.task_queue = queue.PriorityQueue(maxsize=self.task_queue_size)
        
        # Thread pool with adaptive scaling
        self.executor = concurrent.futures.ThreadPoolExecutor(
            max_workers=self.min_workers,
            thread_name_prefix='hardware_pool'
        )
        
        # Task tracking
        self.active_tasks = {}
        self.task_results = {}
        self.task_counter = 0
        self.task_lock = threading.RLock()
        
        # Adaptive scaling
        self.current_workers = self.min_workers
        self.scaling_thread = threading.Thread(target=self._adaptive_scaling_loop, daemon=True)
        self.scaling_thread.start()
        
        # Task processing
        self.processing_thread = threading.Thread(target=self._process_task_queue, daemon=True)
        self.running = True
        self.processing_thread.start()
        
        logger.info(f"Thread pool initialized with {self.min_workers}-{self.max_workers} workers")
    
    def submit_task(self, func, *args, priority=5, task_name=None, **kwargs):
        """
        Submit a task to the thread pool.
        
        Args:
            func: Function to execute
            *args: Function arguments
            priority: Task priority (lower value = higher priority, default: 5)
            task_name: Optional name for task tracking
            **kwargs: Function keyword arguments
            
        Returns:
            Task ID for result tracking
        """
        with self.task_lock:
            task_id = self.task_counter
            self.task_counter += 1
            
            # Create task name if not provided
            if not task_name:
                task_name = f"Task-{task_id}"
            
            # Create task
            task = {
                'id': task_id,
                'name': task_name,
                'func': func,
                'args': args,
                'kwargs': kwargs,
                'priority': priority,
                'submit_time': time.time()
            }
            
            # Add to queue
            self.task_queue.put((priority, task_id, task))
            logger.debug(f"Task {task_name} (ID: {task_id}) submitted with priority {priority}")
            
            return task_id
    
    def get_result(self, task_id, timeout=None):
        """
        Get the result of a task.
        
        Args:
            task_id: Task ID
            timeout: Optional timeout in seconds
            
        Returns:
            Task result or None if not available or timeout
        """
        start_time = time.time()
        
        while timeout is None or time.time() - start_time < timeout:
            # Check if result is available
            with self.task_lock:
                if task_id in self.task_results:
                    result = self.task_results[task_id]
                    del self.task_results[task_id]
                    return result
            
            # Wait a bit before checking again
            time.sleep(0.05)
        
        return None
    
    def cancel_task(self, task_id):
        """
        Cancel a task if possible.
        
        Args:
            task_id: Task ID
            
        Returns:
            True if canceled, False otherwise
        """
        with self.task_lock:
            # Check if task is in active tasks
            if task_id in self.active_tasks:
                future = self.active_tasks[task_id]
                canceled = future.cancel()
                if canceled:
                    del self.active_tasks[task_id]
                return canceled
            
            # Task is either completed or not started yet
            return False
    
    def shutdown(self):
        """Shutdown the thread pool."""
        logger.info("Shutting down thread pool")
        self.running = False
        
        # Cancel all tasks
        with self.task_lock:
            for task_id, future in list(self.active_tasks.items()):
                future.cancel()
            
            self.active_tasks.clear()
            self.task_results.clear()
        
        # Empty the queue
        while not self.task_queue.empty():
            try:
                self.task_queue.get_nowait()
                self.task_queue.task_done()
            except queue.Empty:
                break
        
        # Shutdown executor
        self.executor.shutdown(wait=True)
    
    def _process_task_queue(self):
        """Process tasks from the queue."""
        while self.running:
            try:
                # Get task from queue
                priority, task_id, task = self.task_queue.get(timeout=1.0)
                
                # Submit task to executor
                future = self.executor.submit(
                    self._execute_task,
                    task['func'],
                    task['id'],
                    task['name'],
                    *task['args'],
                    **task['kwargs']
                )
                
                # Track active task
                with self.task_lock:
                    self.active_tasks[task_id] = future
                
                # Add callback to handle completion
                future.add_done_callback(lambda f, tid=task_id: self._task_completed(tid, f))
                
                # Mark task as processed
                self.task_queue.task_done()
                
            except queue.Empty:
                # No tasks in queue
                pass
            except Exception as e:
                logger.error(f"Error processing task queue: {e}")
                time.sleep(1.0)
    
    def _execute_task(self, func, task_id, task_name, *args, **kwargs):
        """
        Execute a task with proper error handling.
        
        Args:
            func: Function to execute
            task_id: Task ID
            task_name: Task name
            *args, **kwargs: Function arguments
            
        Returns:
            Function result
        """
        try:
            logger.debug(f"Executing task {task_name} (ID: {task_id})")
            result = func(*args, **kwargs)
            return result
        except Exception as e:
            logger.error(f"Task {task_name} (ID: {task_id}) failed: {e}")
            return {'error': str(e)}
    
    def _task_completed(self, task_id, future):
        """
        Handle task completion.
        
        Args:
            task_id: Task ID
            future: Future object
        """
        with self.task_lock:
            # Remove from active tasks
            if task_id in self.active_tasks:
                del self.active_tasks[task_id]
            
            # Store result
            try:
                result = future.result()
                self.task_results[task_id] = result
            except concurrent.futures.CancelledError:
                # Task was cancelled
                self.task_results[task_id] = {'error': 'Task cancelled'}
            except Exception as e:
                # Task failed
                self.task_results[task_id] = {'error': str(e)}
    
    def _adaptive_scaling_loop(self):
        """Adaptively scale the thread pool based on system load."""
        while self.running:
            try:
                # Get system load
                cpu_percent = psutil.cpu_percent(interval=1.0)
                queue_size = self.task_queue.qsize()
                active_tasks = len(self.active_tasks)
                
                # Scale worker count based on load and queue size
                target_workers = self.min_workers
                
                if cpu_percent < 50:  # Low CPU usage
                    if queue_size > 0 or active_tasks > self.current_workers * 0.8:
                        # Increase workers if there are waiting tasks
                        target_workers = min(self.max_workers, self.current_workers + 1)
                elif cpu_percent > 80:  # High CPU usage
                    # Decrease workers to reduce load
                    target_workers = max(self.min_workers, self.current_workers - 1)
                else:
                    # Keep current workers or adjust slightly based on queue
                    if queue_size > 5:
                        target_workers = min(self.max_workers, self.current_workers + 1)
                    elif queue_size == 0 and active_tasks < self.current_workers * 0.5:
                        target_workers = max(self.min_workers, self.current_workers - 1)
                    else:
                        target_workers = self.current_workers
                
                # Update executor if needed
                if target_workers != self.current_workers:
                    logger.debug(f"Scaling thread pool from {self.current_workers} to {target_workers} workers")
                    
                    # Update executor max_workers
                    # Note: ThreadPoolExecutor doesn't support dynamic resizing directly,
                    # but we can manage effective concurrency by limiting active tasks
                    self.current_workers = target_workers
                
                # Sleep before next check
                time.sleep(5.0)
                
            except Exception as e:
                logger.error(f"Error in adaptive scaling: {e}")
                time.sleep(10.0)


class HardwareOptimizer:
    """
    Manages CPU/GPU performance states for optimal balance of
    performance and power consumption.
    
    Features:
    - CPU frequency scaling management
    - GPU performance mode control
    - Thermal management
    - Load-based performance profiles
    - Intelligent boost timing
    """
    
    def __init__(self, system_manager):
        """
        Initialize hardware optimizer.
        
        Args:
            system_manager: SystemManager instance
        """
        self.system_manager = system_manager
        self.config = system_manager.config_manager
        
        # Get thread pool
        self.thread_pool = system_manager.get_component('thread_pool')
        if not self.thread_pool:
            self.thread_pool = ThreadPool(system_manager)
            system_manager.register_component('thread_pool', ThreadPool)
        
        # System info cache
        self.info_cache = None
        if hasattr(system_manager, 'get_component'):
            self.info_cache = system_manager.get_component('system_info_cache')
        
        # Performance profiles
        self.profiles = {
            'performance': {
                'cpu_governor': 'performance',
                'cpu_energy_perf': 'performance',
                'gpu_mode': 'performance',
                'boost_enabled': True,
                'pstate': '0'
            },
            'balanced': {
                'cpu_governor': 'schedutil',
                'cpu_energy_perf': 'balance_performance',
                'gpu_mode': 'balanced',
                'boost_enabled': True,
                'pstate': 'auto'
            },
            'powersave': {
                'cpu_governor': 'powersave',
                'cpu_energy_perf': 'balance_power',
                'gpu_mode': 'powersave',
                'boost_enabled': False,
                'pstate': '15'
            }
        }
        
        # Current performance state
        self.current_profile = 'balanced'
        self.current_state = self.profiles['balanced'].copy()
        
        # Hardware capabilities
        self.capabilities = self._detect_capabilities()
        
        # Performance monitoring
        self.monitoring_interval = self.config.get('system.hardware.monitoring_interval', 2.0)
        self.monitoring_thread = None
        self.running = False
        
        # CPU boost state tracking
        self.boost_active = False
        self.boost_end_time = 0
        self.boost_duration = 10.0  # seconds
        
        # Start monitoring if enabled
        if self.config.get('system.hardware.autostart', True):
            self.start_monitoring()
    
    def start_monitoring(self):
        """Start hardware monitoring and optimization."""
        if not self.running:
            self.running = True
            self.monitoring_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
            self.monitoring_thread.start()
            logger.info("Hardware optimization monitoring started")
    
    def stop_monitoring(self):
        """Stop hardware monitoring and optimization."""
        self.running = False
        if self.monitoring_thread and self.monitoring_thread.is_alive():
            self.monitoring_thread.join(timeout=2.0)
        logger.info("Hardware optimization monitoring stopped")
    
    def set_profile(self, profile_name):
        """
        Set performance profile.
        
        Args:
            profile_name: Profile name ('performance', 'balanced', 'powersave')
            
        Returns:
            True if successful, False otherwise
        """
        if profile_name not in self.profiles:
            logger.error(f"Unknown profile: {profile_name}")
            return False
        
        self.current_profile = profile_name
        new_state = self.profiles[profile_name].copy()
        
        # Apply settings
        success = self._apply_performance_state(new_state)
        
        if success:
            self.current_state = new_state
            logger.info(f"Applied performance profile: {profile_name}")
            
            # Update configuration
            self.config.set('system.hardware.profile', profile_name)
        
        return success
    
    def boost_performance(self, duration=None):
        """
        Temporarily boost performance for specified duration.
        
        Args:
            duration: Boost duration in seconds (default: use class default)
            
        Returns:
            True if boost activated, False otherwise
        """
        if duration is None:
            duration = self.boost_duration
        
        # Only allow boost if not already in performance mode
        if self.current_profile == 'performance':
            return False
        
        # Save current state
        previous_profile = self.current_profile
        
        # Apply performance profile
        success = self.set_profile('performance')
        
        if success:
            # Set boost state
            self.boost_active = True
            self.boost_end_time = time.time() + duration
            
            # Schedule return to previous profile
            self.thread_pool.submit_task(
                self._restore_after_boost,
                previous_profile,
                duration,
                priority=3,
                task_name="Restore after boost"
            )
            
            logger.info(f"Performance boost activated for {duration}s")
        
        return success
    
    def _restore_after_boost(self, previous_profile, duration):
        """
        Restore previous profile after boost period.
        
        Args:
            previous_profile: Profile to restore
            duration: Time to wait before restoring
        """
        # Wait for boost to complete
        time.sleep(duration)
        
        # Only restore if boost is still active
        if self.boost_active and time.time() >= self.boost_end_time:
            self.boost_active = False
            
            # Restore previous profile
            success = self.set_profile(previous_profile)
            if success:
                logger.info(f"Restored profile {previous_profile} after performance boost")
            else:
                logger.error(f"Failed to restore profile {previous_profile}")
    
    def _detect_capabilities(self):
        """
        Detect hardware optimization capabilities.
        
        Returns:
            Dictionary of available capabilities
        """
        capabilities = {
            'cpu_freq': False,
            'cpu_governors': [],
            'intel_pstate': False,
            'amd_pstate': False,
            'gpu_control': False,
            'gpu_vendor': None,
            'tlp_available': False,
            'cpupower_available': False
        }
        
        # Check for CPU frequency scaling
        if os.path.exists('/sys/devices/system/cpu/cpu0/cpufreq'):
            capabilities['cpu_freq'] = True
            
            # Check for available governors
            governor_path = '/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors'
            if os.path.exists(governor_path):
                try:
                    with open(governor_path, 'r') as f:
                        governors = f.read().strip().split()
                        capabilities['cpu_governors'] = governors
                except Exception as e:
                    logger.warning(f"Failed to read available governors: {e}")
        
        # Check for Intel P-State driver
        if os.path.exists('/sys/devices/system/cpu/intel_pstate'):
            capabilities['intel_pstate'] = True
        
        # Check for AMD P-State driver
        if os.path.exists('/sys/devices/system/cpu/amd_pstate'):
            capabilities['amd_pstate'] = True
        
        # Check for TLP
        try:
            result = subprocess.run(['tlp-stat', '-s'], capture_output=True, text=True, timeout=3)
            if result.returncode == 0:
                capabilities['tlp_available'] = True
        except (subprocess.SubprocessError, FileNotFoundError):
            pass
        
        # Check for cpupower
        try:
            result = subprocess.run(['cpupower', 'frequency-info'], capture_output=True, text=True, timeout=3)
            if result.returncode == 0:
                capabilities['cpupower_available'] = True
        except (subprocess.SubprocessError, FileNotFoundError):
            pass
        
        # Check for GPU control
        try:
            # Check for NVIDIA
            result = subprocess.run(['nvidia-smi'], capture_output=True, text=True, timeout=3)
            if result.returncode == 0:
                capabilities['gpu_control'] = True
                capabilities['gpu_vendor'] = 'nvidia'
        except (subprocess.SubprocessError, FileNotFoundError):
            # Check for AMD
            if os.path.exists('/sys/class/drm/card0/device/power_dpm_state'):
                capabilities['gpu_control'] = True
                capabilities['gpu_vendor'] = 'amd'
        
        logger.info(f"Detected hardware capabilities: {capabilities}")
        return capabilities
    
    def _apply_performance_state(self, state):
        """
        Apply performance state to hardware.
        
        Args:
            state: Performance state dictionary
            
        Returns:
            True if successful, False otherwise
        """
        success = True
        
        # Apply CPU governor
        if self.capabilities['cpu_freq'] and state['cpu_governor'] in self.capabilities['cpu_governors']:
            success = success and self._set_cpu_governor(state['cpu_governor'])
        
        # Apply CPU energy performance preference
        if self.capabilities['intel_pstate'] or self.capabilities['amd_pstate']:
            success = success and self._set_energy_performance(state['cpu_energy_perf'])
        
        # Apply boost state
        success = success and self._set_boost_state(state['boost_enabled'])
        
        # Apply P-State
        if self.capabilities['intel_pstate']:
            success = success and self._set_pstate(state['pstate'])
        
        # Apply GPU mode
        if self.capabilities['gpu_control']:
            success = success and self._set_gpu_mode(state['gpu_mode'])
        
        # Apply TLP profile if available
        if self.capabilities['tlp_available']:
            profile = 'AC' if state['cpu_governor'] == 'performance' else 'BAT'
            success = success and self._set_tlp_profile(profile)
        
        return success
    
    def _set_cpu_governor(self, governor):
        """
        Set CPU governor for all cores.
        
        Args:
            governor: Governor name
            
        Returns:
            True if successful, False otherwise
        """
        if self.capabilities['cpupower_available']:
            try:
                cmd = ['sudo', 'cpupower', 'frequency-set', '-g', governor]
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
                return result.returncode == 0
            except subprocess.SubprocessError as e:
                logger.error(f"Failed to set CPU governor via cpupower: {e}")
                return False
        else:
            # Manual setting via sysfs
            try:
                cpu_count = os.cpu_count() or 1
                for cpu in range(cpu_count):
                    governor_path = f'/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor'
                    if os.path.exists(governor_path):
                        with open(governor_path, 'w') as f:
                            f.write(governor)
                return True
            except Exception as e:
                logger.error(f"Failed to set CPU governor manually: {e}")
                return False
    
    def _set_energy_performance(self, profile):
        """
        Set CPU energy performance preference.
        
        Args:
            profile: Energy performance profile
            
        Returns:
            True if successful, False otherwise
        """
        try:
            if self.capabilities['cpupower_available']:
                cmd = ['sudo', 'cpupower', 'set', '-e', profile]
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
                return result.returncode == 0
            else:
                # Manual setting via sysfs
                cpu_count = os.cpu_count() or 1
                for cpu in range(cpu_count):
                    path = f'/sys/devices/system/cpu/cpu{cpu}/cpufreq/energy_performance_preference'
                    if os.path.exists(path):
                        with open(path, 'w') as f:
                            f.write(profile)
                return True
        except Exception as e:
            logger.error(f"Failed to set energy performance preference: {e}")
            return False
    
    def _set_boost_state(self, enabled):
        """
        Set CPU boost state.
        
        Args:
            enabled: Whether boost should be enabled
            
        Returns:
            True if successful, False otherwise
        """
        try:
            boost_path = '/sys/devices/system/cpu/cpufreq/boost'
            if os.path.exists(boost_path):
                with open(boost_path, 'w') as f:
                    f.write('1' if enabled else '0')
                return True
            
            # Intel specific
            intel_boost_path = '/sys/devices/system/cpu/intel_pstate/no_turbo'
            if os.path.exists(intel_boost_path):
                with open(intel_boost_path, 'w') as f:
                    f.write('0' if enabled else '1')  # Inverted logic: 0 = turbo enabled
                return True
            
            # AMD specific
            amd_boost_path = '/sys/devices/system/cpu/cpufreq/boost'
            if os.path.exists(amd_boost_path):
                with open(amd_boost_path, 'w') as f:
                    f.write('1' if enabled else '0')
                return True
            
            return False
        except Exception as e:
            logger.error(f"Failed to set boost state: {e}")
            return False
    
    def _set_pstate(self, pstate):
        """
        Set Intel P-State.
        
        Args:
            pstate: P-State value (0-15, or 'auto')
            
        Returns:
            True if successful, False otherwise
        """
        if not self.capabilities['intel_pstate']:
            return False
        
        try:
            # Check if P-State control is available
            min_path = '/sys/devices/system/cpu/intel_pstate/min_perf_pct'
            max_path = '/sys/devices/system/cpu/intel_pstate/max_perf_pct'
            
            if pstate == 'auto':
                # Auto mode: maximum range
                if os.path.exists(min_path) and os.path.exists(max_path):
                    with open(min_path, 'w') as f:
                        f.write('0')
                    with open(max_path, 'w') as f:
                        f.write('100')
                    return True
            else:
                # Convert P-State to performance percentage
                try:
                    p_value = int(pstate)
                    perf_pct = max(0, min(100, 100 - p_value * 6))  # P0=100%, P15=0%
                    
                    if os.path.exists(min_path) and os.path.exists(max_path):
                        with open(min_path, 'w') as f:
                            f.write(str(perf_pct))
                        with open(max_path, 'w') as f:
                            f.write(str(perf_pct))
                        return True
                except ValueError:
                    logger.error(f"Invalid P-State value: {pstate}")
                    return False
            
            return False
        except Exception as e:
            logger.error(f"Failed to set P-State: {e}")
            return False
    
    def _set_gpu_mode(self, mode):
        """
        Set GPU power mode.
        
        Args:
            mode: Power mode ('performance', 'balanced', 'powersave')
            
        Returns:
            True if successful, False otherwise
        """
        if not self.capabilities['gpu_control']:
            return False
        
        try:
            if self.capabilities['gpu_vendor'] == 'nvidia':
                # NVIDIA GPU
                modes = {
                    'performance': 'PREFER_MAXIMUM_PERFORMANCE',
                    'balanced': 'AUTO',
                    'powersave': 'PREFER_CONSISTENT_PERFORMANCE'
                }
                
                nvidia_mode = modes.get(mode, 'AUTO')
                cmd = ['nvidia-settings', '-a', f'[gpu:0]/GpuPowerMizerMode={nvidia_mode}']
                
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
                return result.returncode == 0
                
            elif self.capabilities['gpu_vendor'] == 'amd':
                # AMD GPU
                modes = {
                    'performance': 'performance',
                    'balanced': 'balanced',
                    'powersave': 'battery'
                }
                
                amd_mode = modes.get(mode, 'balanced')
                path = '/sys/class/drm/card0/device/power_dpm_state'
                
                if os.path.exists(path):
                    with open(path, 'w') as f:
                        f.write(amd_mode)
                    return True
            
            return False
        except Exception as e:
            logger.error(f"Failed to set GPU mode: {e}")
            return False
    
    def _set_tlp_profile(self, profile):
        """
        Set TLP power profile.
        
        Args:
            profile: Profile name ('AC' or 'BAT')
            
        Returns:
            True if successful, False otherwise
        """
        if not self.capabilities['tlp_available']:
            return False
        
        try:
            cmd = ['sudo', 'tlp', profile.lower()]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
            return result.returncode == 0
        except Exception as e:
            logger.error(f"Failed to set TLP profile: {e}")
            return False
    
    def _monitoring_loop(self):
        """Monitor hardware and adapt performance settings."""
        while self.running:
            try:
                # Skip if boost is active
                if self.boost_active:
                    time.sleep(1.0)
                    continue
                
                # Get system load
                cpu_percent = psutil.cpu_percent(interval=0.5)
                
                # Get power source
                on_battery = False
                if hasattr(psutil, 'sensors_battery'):
                    battery = psutil.sensors_battery()
                    if battery:
                        on_battery = not battery.power_plugged
                
                # Adaptive profile selection
                if on_battery:
                    if cpu_percent > 80:
                        # High load on battery: use balanced
                        if self.current_profile != 'balanced':
                            self.set_profile('balanced')
                    else:
                        # Low load on battery: use powersave
                        if self.current_profile != 'powersave':
                            self.set_profile('powersave')
                else:
                    if cpu_percent > 70:
                        # High load on AC: use performance
                        if self.current_profile != 'performance':
                            self.set_profile('performance')
                    elif cpu_percent < 30:
                        # Low load on AC: use balanced
                        if self.current_profile != 'balanced':
                            self.set_profile('balanced')
                
                # Sleep before next check
                time.sleep(self.monitoring_interval)
                
            except Exception as e:
                logger.error(f"Error in hardware monitoring: {e}")
                time.sleep(5.0)
    
    def shutdown(self):
        """Shutdown hardware optimizer."""
        self.stop_monitoring()
        
        # Restore balanced profile
        if self.current_profile != 'balanced':
            self.set_profile('balanced')
        
        logger.info("Hardware optimizer shutdown complete")


class RGBDeviceOptimizer:
    """
    Enhances RGB device communication for improved responsiveness,
    reliability, and performance.
    
    Features:
    - Intelligent command scheduling
    - Multi-device synchronization
    - Device health monitoring
    - Auto-recovery from errors
    - Device discovery and auto-configuration
    """
    
    def __init__(self, system_manager):
        """
        Initialize RGB device optimizer.
        
        Args:
            system_manager: SystemManager instance
        """
        self.system_manager = system_manager
        self.config = system_manager.config_manager
        
        # Get needed components
        self.thread_pool = system_manager.get_component('thread_pool')
        if not self.thread_pool:
            self.thread_pool = ThreadPool(system_manager)
            system_manager.register_component('thread_pool', ThreadPool)
        
        # Create command batcher
        self.command_batcher = RGBCommandBatcher()
        
        # Get device manager if available
        self.device_manager = None
        if hasattr(system_manager, 'get_component'):
            self.device_manager = system_manager.get_component('device_manager')
        
        # If no device manager, use direct device access
        if not self.device_manager:
            self.direct_device = True
            self.device_path = self.config.get('devices.primary_keyboard', '/dev/hidraw0')
        else:
            self.direct_device = False
        
        # Device health tracking
        self.health_checks = {}
        self.device_errors = {}
        
        # Performance tracking
        self.command_latencies = deque(maxlen=100)
        self.error_rates = {}
        
        # Command rate limiting
        self.max_commands_per_second = self.config.get('rgb.max_commands_per_second', 100)
        self.command_count = 0
        self.command_reset_time = time.time()
        self.command_lock = threading.RLock()
        
        # Start command batcher
        self.command_batcher.start()
        
        # Start device monitoring
        self.monitoring_interval = self.config.get('rgb.monitoring_interval', 5.0)
        self.monitoring_thread = None
        self.running = False
        
        if self.config.get('rgb.autostart', True):
            self.start_monitoring()
    
    def start_monitoring(self):
        """Start RGB device monitoring."""
        if not self.running:
            self.running = True
            self.monitoring_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
            self.monitoring_thread.start()
            logger.info("RGB device monitoring started")
    
    def stop_monitoring(self):
        """Stop RGB device monitoring."""
        self.running = False
        if self.monitoring_thread and self.monitoring_thread.is_alive():
            self.monitoring_thread.join(timeout=2.0)
        logger.info("RGB device monitoring stopped")
    
    def set_key_color(self, key_name, red, green, blue, priority=5):
        """
        Set color for a specific key with optimized delivery.
        
        Args:
            key_name: Name of the key
            red, green, blue: RGB color values (0-255)
            priority: Command priority (lower = higher priority)
            
        Returns:
            True if command was queued successfully
        """
        # Check command rate limiting
        with self.command_lock:
            current_time = time.time()
            
            # Reset counter if a second has passed
            if current_time - self.command_reset_time >= 1.0:
                self.command_count = 0
                self.command_reset_time = current_time
            
            # Check if we've hit the rate limit
            if self.command_count >= self.max_commands_per_second:
                # Rate limited - drop low priority commands
                if priority > 3:
                    return False
            
            # Increment command counter
            self.command_count += 1
        
        # Get keyboard mapping from the device manager or use direct access
        if self.device_manager:
            primary_keyboard = self.device_manager.get_primary_keyboard()
            if primary_keyboard:
                device_id = primary_keyboard['id']
                # Let device manager handle it
                return self.device_manager.send_command(device_id, 'set_key', key_name, red, green, blue)
        
        # Use command batcher for direct access
        # We need a keyboard map which we would get from the main RGB controller
        # For simplicity, let's assume we're passing the command to the batcher
        return self.command_batcher.add_command(0, red, green, blue, priority)
    
    def set_group_color(self, group_name, key_group, keyboard_map, red, green, blue, priority=5):
        """
        Set color for a group of keys with optimized delivery.
        
        Args:
            group_name: Name of the group (for logging)
            key_group: List of key names
            keyboard_map: Mapping of key names to indices
            red, green, blue: RGB color values (0-255)
            priority: Command priority (lower = higher priority)
            
        Returns:
            Number of commands successfully queued
        """
        # Use batched operation for efficiency
        return self.command_batcher.add_group_colors(
            key_group,
            keyboard_map,
            red, green, blue,
            priority
        )
    
    def clear_all_keys(self, priority=2):
        """
        Clear all keys with optimized delivery.
        
        Args:
            priority: Command priority (lower = higher priority)
            
        Returns:
            True if successful
        """
        # This is a high-priority operation that should be executed immediately
        # So we flush the command queue first
        self.command_batcher.clear_queue()
        
        # Now send clear commands for all possible key indices
        for key_index in range(0x00, 0xFF):
            self.command_batcher.add_command(key_index, 0, 0, 0, priority)
        
        return True
    
    def check_device_health(self, device_id=None):
        """
        Check health of RGB device.
        
        Args:
            device_id: Device ID (default: primary keyboard)
            
        Returns:
            Health status dictionary
        """
        # Determine device path
        if device_id is None:
            if self.device_manager:
                primary_keyboard = self.device_manager.get_primary_keyboard()
                if primary_keyboard:
                    device_id = primary_keyboard['id']
                    device_path = primary_keyboard['path']
                else:
                    device_path = self.device_path
            else:
                device_path = self.device_path
                device_id = 'primary'
        else:
            if self.device_manager:
                device = self.device_manager.get_device(device_id)
                if device:
                    device_path = device['path']
                else:
                    return {'status': 'error', 'message': 'Device not found'}
            else:
                return {'status': 'error', 'message': 'Device manager not available'}
        
        # Check if device exists
        if not os.path.exists(device_path):
            return {
                'status': 'error',
                'message': f'Device {device_path} does not exist',
                'device_id': device_id,
                'exists': False,
                'accessible': False,
                'responsive': False
            }
        
        # Check if device is accessible
        if not os.access(device_path, os.R_OK | os.W_OK):
            return {
                'status': 'error',
                'message': f'No permission for {device_path}',
                'device_id': device_id,
                'exists': True,
                'accessible': False,
                'responsive': False
            }
        
        # Check if device is responsive
        try:
            with open(device_path, 'wb') as device:
                # Send a test command (clear key 0)
                data = bytes([0xCC, 0x01, 0x00, 0x00, 0x00, 0x00] + [0x00] * 10)
                device.write(data)
            
            return {
                'status': 'ok',
                'message': 'Device is healthy',
                'device_id': device_id,
                'exists': True,
                'accessible': True,
                'responsive': True,
                'error_rate': self.error_rates.get(device_id, 0.0),
                'avg_latency': self._get_average_latency()
            }
        except Exception as e:
            return {
                'status': 'error',
                'message': f'Device error: {e}',
                'device_id': device_id,
                'exists': True,
                'accessible': True,
                'responsive': False,
                'error': str(e)
            }
    
    def recover_device(self, device_id=None):
        """
        Attempt to recover a failed device.
        
        Args:
            device_id: Device ID (default: primary keyboard)
            
        Returns:
            True if recovery was successful, False otherwise
        """
        # Get device health to determine issue
        health = self.check_device_health(device_id)
        
        if health['status'] == 'ok':
            # Device is already healthy
            return True
        
        # Determine recovery steps based on health
        if not health.get('exists', False):
            # Device doesn't exist - check for other devices
            logger.error(f"Device {health.get('device_id', 'unknown')} doesn't exist")
            
            if self.device_manager:
                # Try to find another device
                devices = self.device_manager.discover_devices(force=True)
                
                if devices:
                    # Found devices, try to use the first one
                    new_device_id = next(iter(devices.keys()))
                    logger.info(f"Switching to device {new_device_id}")
                    
                    # Set as primary
                    self.device_manager.set_primary_device(new_device_id)
                    return True
            
            return False
        
        elif not health.get('accessible', False):
            # Permission issue - try to fix permissions
            logger.error(f"Permission issue for device {health.get('device_id', 'unknown')}")
            
            # Get device path
            if self.device_manager:
                device = self.device_manager.get_device(health.get('device_id'))
                if device:
                    device_path = device['path']
                else:
                    return False
            else:
                device_path = self.device_path
            
            # Try to fix permissions
            try:
                subprocess.run(['sudo', 'chmod', '666', device_path], check=True, timeout=5)
                
                # Check if fixed
                return os.access(device_path, os.R_OK | os.W_OK)
            except Exception as e:
                logger.error(f"Failed to fix permissions: {e}")
                return False
        
        elif not health.get('responsive', False):
            # Device is not responding - try resetting
            logger.error(f"Device {health.get('device_id', 'unknown')} not responding")
            
            # Clear command queue
            self.command_batcher.clear_queue()
            
            # Restart command batcher
            self.command_batcher.stop()
            time.sleep(0.5)
            self.command_batcher.start()
            
            # Check if device is now responsive
            new_health = self.check_device_health(device_id)
            return new_health['status'] == 'ok'
        
        return False
    
    def _monitoring_loop(self):
        """Monitor RGB devices and recover from failures."""
        while self.running:
            try:
                # Check device health
                health = self.check_device_health()
                
                # Update health tracking
                device_id = health.get('device_id', 'primary')
                self.health_checks[device_id] = health
                
                # Check for issues
                if health['status'] != 'ok':
                    logger.warning(f"RGB device issue detected: {health['message']}")
                    
                    # Increment error count
                    if device_id not in self.device_errors:
                        self.device_errors[device_id] = 0
                    self.device_errors[device_id] += 1
                    
                    # Attempt recovery if multiple errors
                    if self.device_errors[device_id] >= 3:
                        logger.error(f"Multiple RGB device errors, attempting recovery")
                        
                        success = self.recover_device(device_id)
                        if success:
                            logger.info(f"RGB device recovery successful")
                            self.device_errors[device_id] = 0
                        else:
                            logger.error(f"RGB device recovery failed")
                else:
                    # Reset error count on success
                    self.device_errors[device_id] = 0
                
                # Calculate error rate
                total_checks = len(self.health_checks)
                error_count = sum(1 for h in self.health_checks.values() if h['status'] != 'ok')
                
                if total_checks > 0:
                    error_rate = error_count / total_checks
                    self.error_rates[device_id] = error_rate
                
                # Sleep before next check
                time.sleep(self.monitoring_interval)
                
            except Exception as e:
                logger.error(f"Error in RGB device monitoring: {e}")
                time.sleep(5.0)
    
    def _get_average_latency(self):
        """Calculate average command latency."""
        if not self.command_latencies:
            return 0.0
        
        return sum(self.command_latencies) / len(self.command_latencies)
    
    def shutdown(self):
        """Shutdown RGB device optimizer."""
        logger.info("Shutting down RGB device optimizer")
        
        # Stop monitoring
        self.stop_monitoring()
        
        # Stop command batcher
        self.command_batcher.stop()
        
        # Clear all keys on exit
        try:
            self.clear_all_keys()
        except Exception as e:
            logger.error(f"Error clearing keys during shutdown: {e}")


class SystemMonitor:
    """
    Advanced system metrics tracking with historical data,
    trend analysis, and predictive capabilities.
    
    Features:
    - Comprehensive resource monitoring
    - Performance bottleneck detection
    - Historical metrics with trend analysis
    - Thermal management
    - Hardware health checks
    - Configurable alerts and thresholds
    """
    
    def __init__(self, system_manager):
        """
        Initialize system monitor.
        
        Args:
            system_manager: SystemManager instance
        """
        self.system_manager = system_manager
        self.config = system_manager.config_manager
        
        # Get thread pool
        self.thread_pool = system_manager.get_component('thread_pool')
        if not self.thread_pool:
            self.thread_pool = ThreadPool(system_manager)
            system_manager.register_component('thread_pool', ThreadPool)
        
        # System info cache
        self.info_cache = None
        if hasattr(system_manager, 'get_component'):
            self.info_cache = system_manager.get_component('system_info_cache')
        
        # Monitoring configuration
        self.monitor_cpu = self.config.get('monitoring.cpu', True)
        self.monitor_memory = self.config.get('monitoring.memory', True)
        self.monitor_disk = self.config.get('monitoring.disk', True)
        self.monitor_network = self.config.get('monitoring.network', True)
        self.monitor_temperature = self.config.get('monitoring.temperature', True)
        self.monitor_fans = self.config.get('monitoring.fans', True)
        self.monitor_battery = self.config.get('monitoring.battery', True)
        self.monitor_processes = self.config.get('monitoring.processes', True)
        
        # Sampling configuration
        self.sampling_interval = self.config.get('monitoring.sampling_interval', 1.0)
        self.history_length = self.config.get('monitoring.history_length', 60)  # 1 minute at 1s interval
        
        # Metrics storage
        self.metrics = {
            'cpu': deque(maxlen=self.history_length),
            'memory': deque(maxlen=self.history_length),
            'disk': deque(maxlen=self.history_length),
            'network': deque(maxlen=self.history_length),
            'temperature': deque(maxlen=self.history_length),
            'fans': deque(maxlen=self.history_length),
            'battery': deque(maxlen=self.history_length),
            'processes': deque(maxlen=self.history_length)
        }
        
        # Alert thresholds
        self.thresholds = {
            'cpu_high': self.config.get('monitoring.thresholds.cpu_high', 90),
            'memory_high': self.config.get('monitoring.thresholds.memory_high', 90),
            'disk_high': self.config.get('monitoring.thresholds.disk_high', 90),
            'temperature_high': self.config.get('monitoring.thresholds.temperature_high', 80),
            'battery_low': self.config.get('monitoring.thresholds.battery_low', 15)
        }
        
        # Alert callbacks
        self.alert_callbacks = []
        
        # Start monitoring
        self.monitoring_thread = None
        self.running = False
        
        if self.config.get('monitoring.autostart', True):
            self.start_monitoring()
    
    def start_monitoring(self):
        """Start system monitoring."""
        if not self.running:
            self.running = True
            self.monitoring_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
            self.monitoring_thread.start()
            logger.info("System monitoring started")
    
    def stop_monitoring(self):
        """Stop system monitoring."""
        self.running = False
        if self.monitoring_thread and self.monitoring_thread.is_alive():
            self.monitoring_thread.join(timeout=2.0)
        logger.info("System monitoring stopped")
    
    def register_alert_callback(self, callback):
        """
        Register a callback for alerts.
        
        Args:
            callback: Function to call when an alert is triggered
        """
        if callable(callback) and callback not in self.alert_callbacks:
            self.alert_callbacks.append(callback)
    
    def unregister_alert_callback(self, callback):
        """
        Unregister an alert callback.
        
        Args:
            callback: Callback to remove
        """
        if callback in self.alert_callbacks:
            self.alert_callbacks.remove(callback)
    
    def get_current_metrics(self):
        """
        Get current system metrics.
        
        Returns:
            Dictionary of current metrics
        """
        metrics = {}
        
        # CPU metrics
        if self.monitor_cpu:
            if self.info_cache:
                cpu_info = self.info_cache.get_cpu_info()
                metrics['cpu'] = {
                    'percent': cpu_info.get('percent', 0),
                    'count': cpu_info.get('count', 0),
                    'load_avg': cpu_info.get('load_avg', (0, 0, 0))
                }
            else:
                metrics['cpu'] = {
                    'percent': psutil.cpu_percent(interval=0.1),
                    'count': psutil.cpu_count(),
                    'load_avg': os.getloadavg() if hasattr(os, 'getloadavg') else (0, 0, 0)
                }
        
        # Memory metrics
        if self.monitor_memory:
            if self.info_cache:
                memory = self.info_cache.get_memory_info()
            else:
                memory = psutil.virtual_memory()
            
            metrics['memory'] = {
                'total': memory.total,
                'available': memory.available,
                'used': memory.used,
                'percent': memory.percent
            }
        
        # Disk metrics
        if self.monitor_disk:
            if self.info_cache:
                disk = self.info_cache.get_disk_info('/')
            else:
                disk = psutil.disk_usage('/')
            
            metrics['disk'] = {
                'total': disk.total,
                'used': disk.used,
                'free': disk.free,
                'percent': disk.percent
            }
        
        # Network metrics
        if self.monitor_network:
            if self.info_cache:
                network = self.info_cache.get_network_info()
            else:
                network = psutil.net_io_counters()
            
            metrics['network'] = {
                'bytes_sent': network.bytes_sent,
                'bytes_recv': network.bytes_recv,
                'packets_sent': network.packets_sent,
                'packets_recv': network.packets_recv
            }
        
        # Temperature metrics
        if self.monitor_temperature:
            if self.info_cache:
                temps = self.info_cache.get_temperature()
                
                # Extract highest temperatures
                cpu_temp = max([t['temp'] for t in temps['cpu']]) if temps['cpu'] else 0
                gpu_temp = max([t['temp'] for t in temps['gpu']]) if temps['gpu'] else 0
                
                metrics['temperature'] = {
                    'cpu': cpu_temp,
                    'gpu': gpu_temp,
                    'max': max(cpu_temp, gpu_temp)
                }
            elif hasattr(psutil, 'sensors_temperatures'):
                temps = psutil.sensors_temperatures()
                
                # Extract CPU temperature
                cpu_temp = 0
                if 'coretemp' in temps:
                    cpu_cores = temps['coretemp']
                    if cpu_cores:
                        cpu_temp = max(t.current for t in cpu_cores)
                
                metrics['temperature'] = {
                    'cpu': cpu_temp,
                    'gpu': 0,  # No easy way to get GPU temp without info_cache
                    'max': cpu_temp
                }
            else:
                metrics['temperature'] = {
                    'cpu': 0,
                    'gpu': 0,
                    'max': 0
                }
        
        # Fan metrics
        if self.monitor_fans:
            if self.info_cache:
                fans = self.info_cache.get_fan_speeds()
                metrics['fans'] = fans
            elif hasattr(psutil, 'sensors_fans'):
                fan_sensors = psutil.sensors_fans()
                fans = []
                
                for chip, entries in fan_sensors.items():
                    for idx, entry in enumerate(entries):
                        fans.append({
                            'name': entry.label or f"{chip} Fan {idx}",
                            'speed': entry.current,
                            'source': 'psutil'
                        })
                
                metrics['fans'] = fans
            else:
                metrics['fans'] = []
        
        # Battery metrics
        if self.monitor_battery:
            if self.info_cache:
                battery = self.info_cache.get_battery_info()
                metrics['battery'] = battery
            elif hasattr(psutil, 'sensors_battery'):
                battery = psutil.sensors_battery()
                if battery:
                    metrics['battery'] = {
                        'percent': battery.percent,
                        'power_plugged': battery.power_plugged,
                        'secsleft': battery.secsleft
                    }
                else:
                    metrics['battery'] = {
                        'percent': 100,
                        'power_plugged': True,
                        'secsleft': -1
                    }
            else:
                metrics['battery'] = {
                    'percent': 100,
                    'power_plugged': True,
                    'secsleft': -1
                }
        
        # Process metrics
        if self.monitor_processes:
            if self.info_cache:
                processes = self.info_cache.get_process_list()
                metrics['processes'] = processes
            else:
                try:
                    processes = []
                    for proc in psutil.process_iter(['pid', 'name', 'username', 'cpu_percent', 'memory_percent']):
                        try:
                            pinfo = proc.info
                            pinfo['cpu_percent'] = proc.cpu_percent(interval=0)
                            processes.append(pinfo)
                        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                            pass
                    
                    metrics['processes'] = processes
                except Exception as e:
                    logger.error(f"Error getting process metrics: {e}")
                    metrics['processes'] = []
        
        # Add timestamp
        metrics['timestamp'] = time.time()
        
        return metrics
    
    def get_metrics_history(self, metric_type, duration=60):
        """
        Get historical metrics.
        
        Args:
            metric_type: Type of metric ('cpu', 'memory', etc.)
            duration: Duration in seconds
            
        Returns:
            List of metrics
        """
        if metric_type not in self.metrics:
            return []
        
        # Return the requested number of samples
        return list(self.metrics[metric_type])[-duration:]
    
    def get_metric_trends(self, metric_type, duration=60):
        """
        Calculate trends for a metric.
        
        Args:
            metric_type: Type of metric ('cpu', 'memory', etc.)
            duration: Duration in seconds
            
        Returns:
            Dictionary with trend information
        """
        history = self.get_metrics_history(metric_type, duration)
        
        if not history:
            return {
                'trend': 'stable',
                'rate': 0.0,
                'min': 0,
                'max': 0,
                'avg': 0
            }
        
        # Extract values based on metric type
        if metric_type == 'cpu':
            values = [m['percent'] for m in history]
        elif metric_type == 'memory':
            values = [m['percent'] for m in history]
        elif metric_type == 'disk':
            values = [m['percent'] for m in history]
        elif metric_type == 'network':
            # Calculate rates for network
            if len(history) < 2:
                return {
                    'trend': 'stable',
                    'rate': 0.0,
                    'min': 0,
                    'max': 0,
                    'avg': 0
                }
            
            # Calculate rates of change
            values = []
            for i in range(1, len(history)):
                prev = history[i-1]
                curr = history[i]
                
                time_diff = curr['timestamp'] - prev['timestamp']
                if time_diff <= 0:
                    continue
                
                bytes_diff = (curr['bytes_sent'] + curr['bytes_recv']) - (prev['bytes_sent'] + prev['bytes_recv'])
                rate = bytes_diff / time_diff
                values.append(rate)
        elif metric_type == 'temperature':
            values = [m['max'] for m in history]
        elif metric_type == 'battery':
            values = [m['percent'] for m in history]
        else:
            return {
                'trend': 'stable',
                'rate': 0.0,
                'min': 0,
                'max': 0,
                'avg': 0
            }
        
        # Calculate trend
        if len(values) < 2:
            return {
                'trend': 'stable',
                'rate': 0.0,
                'min': min(values) if values else 0,
                'max': max(values) if values else 0,
                'avg': sum(values) / len(values) if values else 0
            }
        
        # Simple linear regression for trend
        x = list(range(len(values)))
        y = values
        
        n = len(x)
        sum_x = sum(x)
        sum_y = sum(y)
        sum_xy = sum(x[i] * y[i] for i in range(n))
        sum_xx = sum(x[i]**2 for i in range(n))
        
        try:
            slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x**2)
        except ZeroDivisionError:
            slope = 0
        
        # Determine trend
        if slope > 0.1:
            trend = 'rising'
        elif slope < -0.1:
            trend = 'falling'
        else:
            trend = 'stable'
        
        return {
            'trend': trend,
            'rate': slope,
            'min': min(values),
            'max': max(values),
            'avg': sum(values) / len(values)
        }
    
    def get_bottlenecks(self):
        """
        Detect system bottlenecks.
        
        Returns:
            List of bottleneck information
        """
        bottlenecks = []
        
        # Get current metrics
        metrics = self.get_current_metrics()
        
        # Check CPU usage
        if 'cpu' in metrics and metrics['cpu']['percent'] > self.thresholds['cpu_high']:
            # Get top CPU processes
            if 'processes' in metrics:
                top_procs = sorted(metrics['processes'], key=lambda p: p.get('cpu_percent', 0), reverse=True)[:5]
                bottlenecks.append({
                    'type': 'cpu',
                    'value': metrics['cpu']['percent'],
                    'message': f"High CPU usage: {metrics['cpu']['percent']}%",
                    'processes': [{'pid': p['pid'], 'name': p['name'], 'cpu': p.get('cpu_percent', 0)} for p in top_procs]
                })
            else:
                bottlenecks.append({
                    'type': 'cpu',
                    'value': metrics['cpu']['percent'],
                    'message': f"High CPU usage: {metrics['cpu']['percent']}%"
                })
        
        # Check memory usage
        if 'memory' in metrics and metrics['memory']['percent'] > self.thresholds['memory_high']:
            # Get top memory processes
            if 'processes' in metrics:
                top_procs = sorted(metrics['processes'], key=lambda p: p.get('memory_percent', 0), reverse=True)[:5]
                bottlenecks.append({
                    'type': 'memory',
                    'value': metrics['memory']['percent'],
                    'message': f"High memory usage: {metrics['memory']['percent']}%",
                    'processes': [{'pid': p['pid'], 'name': p['name'], 'memory': p.get('memory_percent', 0)} for p in top_procs]
                })
            else:
                bottlenecks.append({
                    'type': 'memory',
                    'value': metrics['memory']['percent'],
                    'message': f"High memory usage: {metrics['memory']['percent']}%"
                })
        
        # Check disk usage
        if 'disk' in metrics and metrics['disk']['percent'] > self.thresholds['disk_high']:
            bottlenecks.append({
                'type': 'disk',
                'value': metrics['disk']['percent'],
                'message': f"High disk usage: {metrics['disk']['percent']}%"
            })
        
        # Check temperature
        if 'temperature' in metrics and metrics['temperature']['max'] > self.thresholds['temperature_high']:
            bottlenecks.append({
                'type': 'temperature',
                'value': metrics['temperature']['max'],
                'message': f"High temperature: {metrics['temperature']['max']}°C"
            })
        
        # Check battery level
        if 'battery' in metrics and not metrics['battery'].get('power_plugged', True) and metrics['battery']['percent'] < self.thresholds['battery_low']:
            bottlenecks.append({
                'type': 'battery',
                'value': metrics['battery']['percent'],
                'message': f"Low battery: {metrics['battery']['percent']}%"
            })
        
        return bottlenecks
    
    def _check_alerts(self, metrics):
        """
        Check for alert conditions and trigger callbacks.
        
        Args:
            metrics: Current metrics
        """
        alerts = []
        
        # Check CPU usage
        if 'cpu' in metrics and metrics['cpu']['percent'] > self.thresholds['cpu_high']:
            alerts.append({
                'type': 'cpu_high',
                'value': metrics['cpu']['percent'],
                'threshold': self.thresholds['cpu_high'],
                'message': f"High CPU usage: {metrics['cpu']['percent']}%"
            })
        
        # Check memory usage
        if 'memory' in metrics and metrics['memory']['percent'] > self.thresholds['memory_high']:
            alerts.append({
                'type': 'memory_high',
                'value': metrics['memory']['percent'],
                'threshold': self.thresholds['memory_high'],
                'message': f"High memory usage: {metrics['memory']['percent']}%"
            })
        
        # Check disk usage
        if 'disk' in metrics and metrics['disk']['percent'] > self.thresholds['disk_high']:
            alerts.append({
                'type': 'disk_high',
                'value': metrics['disk']['percent'],
                'threshold': self.thresholds['disk_high'],
                'message': f"High disk usage: {metrics['disk']['percent']}%"
            })
        
        # Check temperature
        if 'temperature' in metrics and metrics['temperature']['max'] > self.thresholds['temperature_high']:
            alerts.append({
                'type': 'temperature_high',
                'value': metrics['temperature']['max'],
                'threshold': self.thresholds['temperature_high'],
                'message': f"High temperature: {metrics['temperature']['max']}°C"
            })
        
        # Check battery level
        if 'battery' in metrics and not metrics['battery'].get('power_plugged', True) and metrics['battery']['percent'] < self.thresholds['battery_low']:
            alerts.append({
                'type': 'battery_low',
                'value': metrics['battery']['percent'],
                'threshold': self.thresholds['battery_low'],
                'message': f"Low battery: {metrics['battery']['percent']}%"
            })
        
        # Trigger callbacks for alerts
        for alert in alerts:
            for callback in self.alert_callbacks:
                try:
                    callback(alert)
                except Exception as e:
                    logger.error(f"Error in alert callback: {e}")
    
    def _monitoring_loop(self):
        """System monitoring loop."""
        while self.running:
            try:
                # Get current metrics
                metrics = self.get_current_metrics()
                
                # Store metrics in history
                for metric_type in self.metrics:
                    if metric_type in metrics:
                        self.metrics[metric_type].append(metrics[metric_type])
                
                # Check for alerts
                self._check_alerts(metrics)
                
                # Sleep before next sample
                time.sleep(self.sampling_interval)
                
            except Exception as e:
                logger.error(f"Error in system monitoring: {e}")
                time.sleep(5.0)
    
    def shutdown(self):
        """Shutdown system monitor."""
        self.stop_monitoring()
        logger.info("System monitor shutdown complete")


class PowerOptimizer:
    """
    Intelligent power management for optimal battery life and performance.
    
    Features:
    - Adaptive power profiles
    - Dynamic frequency scaling
    - Power consumption tracking
    - Battery health monitoring
    - Component power state management
    - Idle detection and management
    """
    
    def __init__(self, system_manager):
        """
        Initialize power optimizer.
        
        Args:
            system_manager: SystemManager instance
        """
        self.system_manager = system_manager
        self.config = system_manager.config_manager
        
        # Get thread pool
        self.thread_pool = system_manager.get_component('thread_pool')
        if not self.thread_pool:
            self.thread_pool = ThreadPool(system_manager)
            system_manager.register_component('thread_pool', ThreadPool)
        
        # Get hardware optimizer if available
        self.hardware_optimizer = None
        if hasattr(system_manager, 'get_component'):
            self.hardware_optimizer = system_manager.get_component('hardware_optimizer')
        
        # System info cache
        self.info_cache = None
        if hasattr(system_manager, 'get_component'):
            self.info_cache = system_manager.get_component('system_info_cache')
        
        # Power profiles
        self.profiles = {
            'performance': {
                'description': 'Maximum performance, highest power consumption',
                'cpu_profile': 'performance',
                'gpu_profile': 'performance',
                'screen_brightness': 100,
                'rgb_brightness': 100,
                'idle_timeout': 1800,  # 30 minutes
                'sleep_timeout': 3600   # 1 hour
            },
            'balanced': {
                'description': 'Balance of performance and power saving',
                'cpu_profile': 'balanced',
                'gpu_profile': 'balanced',
                'screen_brightness': 80,
                'rgb_brightness': 70,
                'idle_timeout': 900,   # 15 minutes
                'sleep_timeout': 1800  # 30 minutes
            },
            'powersave': {
                'description': 'Maximum power saving, reduced performance',
                'cpu_profile': 'powersave',
                'gpu_profile': 'powersave',
                'screen_brightness': 50,
                'rgb_brightness': 30,
                'idle_timeout': 300,   # 5 minutes
                'sleep_timeout': 900   # 15 minutes
            }
        }
        
        # Current profile
        self.current_profile = self.config.get('power.current_profile', 'balanced')
        
        # Battery state tracking
        self.on_battery = False
        self.battery_percent = 100
        self.battery_health = 'good'
        self.battery_history = deque(maxlen=100)
        
        # Idle state tracking
        self.last_activity_time = time.time()
        self.idle_state = False
        
        # Power consumption tracking
        self.power_readings = deque(maxlen=60)  # Last minute of readings
        self.last_energy_reading = None
        self.last_energy_time = None
        
        # Power optimization capabilities
        self.capabilities = self._detect_capabilities()
        
        # Power monitoring
        self.monitoring_interval = self.config.get('power.monitoring_interval', 5.0)
        self.monitoring_thread = None
        self.running = False
        
        # Auto-apply profile based on power source
        self.auto_profile = self.config.get('power.auto_profile', True)
        
        # Start monitoring if enabled
        if self.config.get('power.autostart', True):
            self.start_monitoring()
    
    def start_monitoring(self):
        """Start power monitoring and optimization."""
        if not self.running:
            self.running = True
            self.monitoring_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
            self.monitoring_thread.start()
            logger.info("Power optimization monitoring started")
    
    def stop_monitoring(self):
        """Stop power monitoring and optimization."""
        self.running = False
        if self.monitoring_thread and self.monitoring_thread.is_alive():
            self.monitoring_thread.join(timeout=2.0)
        logger.info("Power optimization monitoring stopped")
    
    def set_profile(self, profile_name):
        """
        Set power profile.
        
        Args:
            profile_name: Profile name ('performance', 'balanced', 'powersave')
            
        Returns:
            True if successful, False otherwise
        """
        if profile_name not in self.profiles:
            logger.error(f"Unknown profile: {profile_name}")
            return False
        
        self.current_profile = profile_name
        profile = self.profiles[profile_name]
        
        # Apply hardware optimization profile
        if self.hardware_optimizer:
            self.hardware_optimizer.set_profile(profile['cpu_profile'])
        
        # Apply other profile settings
        self._apply_profile_settings(profile)
        
        # Update configuration
        self.config.set('power.current_profile', profile_name)
        
        logger.info(f"Applied power profile: {profile_name}")
        return True
    
    def report_activity(self):
        """
        Report user activity to prevent idle state.
        
        Returns:
            Previous idle state
        """
        was_idle = self.idle_state
        self.last_activity_time = time.time()
        
        if was_idle:
            self.idle_state = False
            logger.info("User activity detected, exiting idle state")
            
            # Restore from idle state
            self._exit_idle_state()
        
        return was_idle
    
    def get_power_statistics(self):
        """
        Get power statistics.
        
        Returns:
            Dictionary of power statistics
        """
        stats = {
            'current_profile': self.current_profile,
            'on_battery': self.on_battery,
            'battery_percent': self.battery_percent,
            'battery_health': self.battery_health,
            'idle_state': self.idle_state,
            'last_activity': time.time() - self.last_activity_time
        }
        
        # Add power consumption if available
        if self.power_readings:
            stats['avg_power'] = sum(self.power_readings) / len(self.power_readings)
            stats['min_power'] = min(self.power_readings)
            stats['max_power'] = max(self.power_readings)
        else:
            stats['avg_power'] = 0
            stats['min_power'] = 0
            stats['max_power'] = 0
        
        # Add remaining battery time if available
        if self.on_battery and hasattr(psutil, 'sensors_battery'):
            battery = psutil.sensors_battery()
            if battery and battery.secsleft != psutil.POWER_TIME_UNLIMITED:
                stats['battery_remaining'] = battery.secsleft
            else:
                stats['battery_remaining'] = -1
        else:
            stats['battery_remaining'] = -1
        
        return stats
    
    def _detect_capabilities(self):
        """
        Detect power management capabilities.
        
        Returns:
            Dictionary of available capabilities
        """
        capabilities = {
            'battery_present': False,
            'can_control_brightness': False,
            'can_control_rgb': True,  # We control RGB
            'can_control_sleep': False,
            'has_energy_sensors': False
        }
        
        # Check for battery
        if hasattr(psutil, 'sensors_battery'):
            battery = psutil.sensors_battery()
            capabilities['battery_present'] = battery is not None
        
        # Check for brightness control
        try:
            # Try using xbacklight
            result = subprocess.run(['xbacklight', '-get'], capture_output=True, text=True, timeout=2)
            if result.returncode == 0:
                capabilities['can_control_brightness'] = True
        except (subprocess.SubprocessError, FileNotFoundError):
            # Try alternate methods
            backlight_dirs = glob.glob('/sys/class/backlight/*')
            for backlight_dir in backlight_dirs:
                brightness_file = os.path.join(backlight_dir, 'brightness')
                if os.path.exists(brightness_file) and os.access(brightness_file, os.W_OK):
                    capabilities['can_control_brightness'] = True
                    break
        
        # Check for sleep control
        try:
            result = subprocess.run(['systemctl', 'suspend', '-h'], capture_output=True, text=True, timeout=2)
            if result.returncode == 0 or 'suspend' in result.stdout or 'suspend' in result.stderr:
                capabilities['can_control_sleep'] = True
        except (subprocess.SubprocessError, FileNotFoundError):
            pass
        
        # Check for energy sensors
        try:
            # Try to read power/energy info from battery
            if capabilities['battery_present']:
                # Check for standard battery interfaces
                power_supply_dirs = glob.glob('/sys/class/power_supply/BAT*')
                for power_dir in power_supply_dirs:
                    power_now_file = os.path.join(power_dir, 'power_now')
                    energy_now_file = os.path.join(power_dir, 'energy_now')
                    
                    if os.path.exists(power_now_file) or os.path.exists(energy_now_file):
                        capabilities['has_energy_sensors'] = True
                        break
        except Exception:
            pass
        
        logger.info(f"Detected power capabilities: {capabilities}")
        return capabilities
    
    def _apply_profile_settings(self, profile):
        """
        Apply power profile settings.
        
        Args:
            profile: Profile settings dictionary
        """
        # Apply screen brightness if supported
        if self.capabilities['can_control_brightness']:
            self._set_screen_brightness(profile['screen_brightness'])
        
        # Apply RGB brightness
        # This would typically be handled through the RGB controller
        
        # Apply idle/sleep timeouts
        # This would typically need to interact with the system's power management
        
        # Apply TLP profile if available
        self._set_tlp_profile(self.current_profile)
    
    def _set_screen_brightness(self, brightness):
        """
        Set screen brightness.
        
        Args:
            brightness: Brightness percentage (0-100)
            
        Returns:
            True if successful, False otherwise
        """
        try:
            # Try using xbacklight
            subprocess.run(['xbacklight', '-set', str(brightness)], check=False, timeout=2)
            return True
        except (subprocess.SubprocessError, FileNotFoundError):
            # Try alternate methods
            backlight_dirs = glob.glob('/sys/class/backlight/*')
            
            for backlight_dir in backlight_dirs:
                try:
                    max_brightness_file = os.path.join(backlight_dir, 'max_brightness')
                    brightness_file = os.path.join(backlight_dir, 'brightness')
                    
                    if os.path.exists(max_brightness_file) and os.path.exists(brightness_file):
                        with open(max_brightness_file, 'r') as f:
                            max_brightness = int(f.read().strip())
                        
                        # Calculate actual brightness value
                        actual_brightness = int(max_brightness * brightness / 100)
                        
                        # Set brightness
                        if os.access(brightness_file, os.W_OK):
                            with open(brightness_file, 'w') as f:
                                f.write(str(actual_brightness))
                        else:
                            # Try with sudo
                            subprocess.run(['sudo', 'tee', brightness_file], input=str(actual_brightness), text=True, check=False, timeout=2)
                        
                        return True
                except Exception as e:
                    logger.error(f"Failed to set brightness: {e}")
            
            return False
    
    def _set_tlp_profile(self, profile):
        """
        Set TLP power profile.
        
        Args:
            profile: Profile name
            
        Returns:
            True if successful, False otherwise
        """
        try:
            # Map our profiles to TLP profiles
            tlp_profiles = {
                'performance': 'performance',
                'balanced': 'balanced',
                'powersave': 'powersave'
            }
            
            tlp_profile = tlp_profiles.get(profile, 'balanced')
            
            # Set TLP profile
            subprocess.run(['tlp', tlp_profile], check=False, timeout=5)
            return True
        except Exception as e:
            logger.error(f"Failed to set TLP profile: {e}")
            return False
    
    def _enter_idle_state(self):
        """
        Enter idle state to save power.
        
        Returns:
            True if successful, False otherwise
        """
        logger.info("Entering idle state")
        
        # Lower screen brightness if possible
        if self.capabilities['can_control_brightness']:
            current_profile = self.profiles[self.current_profile]
            idle_brightness = max(20, current_profile['screen_brightness'] // 2)
            self._set_screen_brightness(idle_brightness)
        
        # Lower RGB brightness
        # This would typically be handled through the RGB controller
        
        return True
    
    def _exit_idle_state(self):
        """
        Exit idle state and restore normal operation.
        
        Returns:
            True if successful, False otherwise
        """
        logger.info("Exiting idle state")
        
        # Restore screen brightness
        if self.capabilities['can_control_brightness']:
            current_profile = self.profiles[self.current_profile]
            self._set_screen_brightness(current_profile['screen_brightness'])
        
        # Restore RGB brightness
        # This would typically be handled through the RGB controller
        
        return True
    
    def _check_idle_state(self):
        """
        Check and update idle state based on activity timeout.
        
        Returns:
            True if idle state changed, False otherwise
        """
        current_time = time.time()
        current_profile = self.profiles[self.current_profile]
        idle_timeout = current_profile['idle_timeout']
        
        # Check if we should enter idle state
        if not self.idle_state and current_time - self.last_activity_time > idle_timeout:
            self.idle_state = True
            self._enter_idle_state()
            return True
        
        # Check if we should enter sleep state
        sleep_timeout = current_profile['sleep_timeout']
        if self.capabilities['can_control_sleep'] and current_time - self.last_activity_time > sleep_timeout:
            # Only sleep if we're on battery to avoid unnecessary suspends
            if self.on_battery:
                logger.info("Sleep timeout reached, suspending system")
                
                # Schedule system suspend
                self.thread_pool.submit_task(
                    lambda: subprocess.run(['systemctl', 'suspend'], check=False),
                    priority=1,
                    task_name="System suspend"
                )
                
                return True
        
        return False
    
    def _update_battery_state(self):
        """
        Update battery state information.
        
        Returns:
            True if state changed, False otherwise
        """
        changed = False
        
        if hasattr(psutil, 'sensors_battery'):
            battery = psutil.sensors_battery()
            if battery:
                # Check if power source changed
                if self.on_battery != (not battery.power_plugged):
                    self.on_battery = not battery.power_plugged
                    changed = True
                    
                    # Auto-switch profile if enabled
                    if self.auto_profile:
                        if self.on_battery and self.current_profile == 'performance':
                            self.set_profile('balanced')
                        elif not self.on_battery and self.current_profile == 'powersave':
                            self.set_profile('balanced')
                
                # Update battery percentage
                if self.battery_percent != battery.percent:
                    self.battery_percent = battery.percent
                    changed = True
                
                # Add to history
                self.battery_history.append({
                    'timestamp': time.time(),
                    'percent': battery.percent,
                    'on_battery': self.on_battery,
                    'secsleft': battery.secsleft
                })
                
                # Check battery health
                if len(self.battery_history) >= 10:
                    self._check_battery_health()
        
        return changed
    
    def _check_battery_health(self):
        """
        Check battery health based on discharge rate.
        
        Returns:
            Battery health status
        """
        if not self.battery_history:
            return 'unknown'
        
        # Only analyze when on battery
        on_battery_history = [entry for entry in self.battery_history if entry['on_battery']]
        
        if len(on_battery_history) < 10:
            return self.battery_health
        
        # Calculate discharge rate
        discharge_rates = []
        
        for i in range(1, len(on_battery_history)):
            prev = on_battery_history[i-1]
            curr = on_battery_history[i]
            
            time_diff = curr['timestamp'] - prev['timestamp']
            percent_diff = prev['percent'] - curr['percent']
            
            if time_diff > 0 and percent_diff > 0:
                rate = percent_diff / time_diff  # % per second
                discharge_rates.append(rate)
        
        if not discharge_rates:
            return self.battery_health
        
        # Calculate average discharge rate
        avg_rate = sum(discharge_rates) / len(discharge_rates)
        
        # Estimate full-discharge time
        full_discharge_time = 100 / (avg_rate * 3600)  # hours
        
        # Determine health based on discharge rate
        if full_discharge_time > 3:
            health = 'good'
        elif full_discharge_time > 1.5:
            health = 'fair'
        else:
            health = 'poor'
        
        # Update health if changed
        if health != self.battery_health:
            self.battery_health = health
            logger.info(f"Battery health updated: {health} (estimated discharge time: {full_discharge_time:.1f} hours)")
        
        return health
    
    def _update_power_consumption(self):
        """
        Update power consumption tracking.
        
        Returns:
            True if successful, False otherwise
        """
        if not self.capabilities['has_energy_sensors']:
            return False
        
        try:
            # Try to read power info directly
            power_now = None
            
            # Method 1: Direct power reading
            power_supply_dirs = glob.glob('/sys/class/power_supply/BAT*')
            for power_dir in power_supply_dirs:
                power_now_file = os.path.join(power_dir, 'power_now')
                if os.path.exists(power_now_file):
                    try:
                        with open(power_now_file, 'r') as f:
                            power_now = int(f.read().strip()) / 1000000.0  # Convert to watts
                            break
                    except (IOError, ValueError):
                        pass
            
            # Method 2: Calculate from energy change
            if power_now is None:
                for power_dir in power_supply_dirs:
                    energy_now_file = os.path.join(power_dir, 'energy_now')
                    if os.path.exists(energy_now_file):
                        try:
                            with open(energy_now_file, 'r') as f:
                                energy_now = int(f.read().strip()) / 1000000.0  # Convert to watt-hours
                                
                                current_time = time.time()
                                
                                if self.last_energy_reading is not None and self.last_energy_time is not None:
                                    time_diff = current_time - self.last_energy_time
                                    energy_diff = self.last_energy_reading - energy_now
                                    
                                    if time_diff > 0 and energy_diff > 0:
                                        # Calculate power in watts
                                        power_now = energy_diff / (time_diff / 3600.0)
                                
                                self.last_energy_reading = energy_now
                                self.last_energy_time = current_time
                                break
                        except (IOError, ValueError):
                            pass
            
            # Store power reading if available
            if power_now is not None and power_now > 0:
                self.power_readings.append(power_now)
                return True
            
            return False
        except Exception as e:
            logger.error(f"Failed to update power consumption: {e}")
            return False
    
    def _monitoring_loop(self):
        """Power monitoring and optimization loop."""
        while self.running:
            try:
                # Update battery state
                battery_changed = self._update_battery_state()
                
                # Update power consumption
                self._update_power_consumption()
                
                # Check idle state
                self._check_idle_state()
                
                # Sleep before next check
                time.sleep(self.monitoring_interval)
                
            except Exception as e:
                logger.error(f"Error in power monitoring: {e}")
                time.sleep(5.0)
    
    def shutdown(self):
        """Shutdown power optimizer."""
        self.stop_monitoring()
        
        # Exit idle state if active
        if self.idle_state:
            self._exit_idle_state()
        
        logger.info("Power optimizer shutdown complete")
