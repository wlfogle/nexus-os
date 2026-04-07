import { useEffect, useCallback, useState } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { forceMemoryCleanup, selectMemoryStats } from '../store/slices/terminalSlice';

interface MemoryStats {
  totalOutputs: number;
  bufferSizes: Record<string, number>;
  lastCleanup: number;
  terminalCount: number;
  bufferCount: number;
}

interface MemoryMonitorConfig {
  enableAutoCleanup?: boolean;
  cleanupThreshold?: number; // Memory usage threshold in MB
  cleanupInterval?: number; // Auto cleanup interval in ms
  enablePerformanceMonitoring?: boolean;
}

const DEFAULT_CONFIG: Required<MemoryMonitorConfig> = {
  enableAutoCleanup: true,
  cleanupThreshold: 100, // 100MB
  cleanupInterval: 300000, // 5 minutes
  enablePerformanceMonitoring: true,
};

export const useMemoryMonitor = (config: MemoryMonitorConfig = {}) => {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };
  const dispatch = useDispatch();
  const memoryStats = useSelector(selectMemoryStats);
  
  const [performanceStats, setPerformanceStats] = useState({
    heapUsed: 0,
    heapTotal: 0,
    renderTime: 0,
    lastUpdate: Date.now(),
  });

  // Estimate memory usage based on terminal outputs
  const estimateMemoryUsage = useCallback((stats: MemoryStats) => {
    // Rough estimation: each terminal output averages 200 bytes
    const avgOutputSize = 200;
    const estimatedBytes = stats.totalOutputs * avgOutputSize;
    const estimatedMB = estimatedBytes / (1024 * 1024);
    return Math.round(estimatedMB * 100) / 100; // Round to 2 decimal places
  }, []);

  // Manual cleanup trigger
  const triggerCleanup = useCallback(() => {
    dispatch(forceMemoryCleanup(undefined));
    // Manual cleanup triggered
  }, [dispatch]);

  // Check if cleanup is needed
  const checkCleanupNeeded = useCallback((stats: MemoryStats) => {
    const estimatedMemoryMB = estimateMemoryUsage(stats);
    const timeSinceLastCleanup = Date.now() - stats.lastCleanup;
    
    return (
      estimatedMemoryMB > finalConfig.cleanupThreshold ||
      timeSinceLastCleanup > finalConfig.cleanupInterval
    );
  }, [finalConfig.cleanupThreshold, finalConfig.cleanupInterval, estimateMemoryUsage]);

  // Performance monitoring using Performance API
  const updatePerformanceStats = useCallback(() => {
    if (!finalConfig.enablePerformanceMonitoring) return;
    
    const startTime = performance.now();
    
    // Use Performance API if available
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      setPerformanceStats(prev => ({
        ...prev,
        heapUsed: Math.round(memory.usedJSHeapSize / (1024 * 1024) * 100) / 100,
        heapTotal: Math.round(memory.totalJSHeapSize / (1024 * 1024) * 100) / 100,
        lastUpdate: Date.now(),
      }));
    }
    
    const endTime = performance.now();
    setPerformanceStats(prev => ({
      ...prev,
      renderTime: Math.round((endTime - startTime) * 100) / 100,
    }));
  }, [finalConfig.enablePerformanceMonitoring]);

  // Auto cleanup effect
  useEffect(() => {
    if (!finalConfig.enableAutoCleanup) return;

    const shouldCleanup = checkCleanupNeeded(memoryStats);
    if (shouldCleanup) {
      triggerCleanup();
    }
  }, [memoryStats, finalConfig.enableAutoCleanup, checkCleanupNeeded, triggerCleanup]);

  // Performance monitoring effect
  useEffect(() => {
    if (!finalConfig.enablePerformanceMonitoring) return;

    const interval = setInterval(updatePerformanceStats, 5000); // Update every 5 seconds
    
    return () => clearInterval(interval);
  }, [finalConfig.enablePerformanceMonitoring, updatePerformanceStats]);

  // Warning thresholds
  const getMemoryWarningLevel = useCallback((estimatedMB: number): 'safe' | 'warning' | 'critical' => {
    if (estimatedMB < finalConfig.cleanupThreshold * 0.5) return 'safe';
    if (estimatedMB < finalConfig.cleanupThreshold * 0.8) return 'warning';
    return 'critical';
  }, [finalConfig.cleanupThreshold]);

  const estimatedMemoryMB = estimateMemoryUsage(memoryStats);
  const warningLevel = getMemoryWarningLevel(estimatedMemoryMB);

  return {
    // Memory statistics
    memoryStats,
    estimatedMemoryMB,
    warningLevel,
    
    // Performance statistics
    performanceStats,
    
    // Actions
    triggerCleanup,
    
    // Utilities
    isMemoryCritical: warningLevel === 'critical',
    isMemoryWarning: warningLevel === 'warning',
    shouldShowMemoryWarning: warningLevel !== 'safe',
    
    // Debug info
    config: finalConfig,
  };
};

// Hook for memory-aware component optimization
export const useMemoryAwareCallback = <T extends (...args: any[]) => any>(
  callback: T,
  deps: React.DependencyList,
  memoryThreshold: number = 50
): T => {
  const { estimatedMemoryMB, isMemoryCritical } = useMemoryMonitor();
  
  return useCallback(
    (...args: Parameters<T>) => {
      // If memory usage is critical, throttle expensive operations
      if (isMemoryCritical && estimatedMemoryMB > memoryThreshold) {
        console.warn('[MemoryMonitor] Throttling callback due to high memory usage');
        // Could implement debouncing or other throttling strategies here
        return;
      }
      
      return callback(...args);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [...deps, isMemoryCritical, estimatedMemoryMB]
  ) as T;
};

// Hook for memory-aware rendering
export const useMemoryAwareRendering = () => {
  const { isMemoryCritical, estimatedMemoryMB } = useMemoryMonitor();
  
  const shouldSkipExpensiveRender = useCallback((threshold: number = 80) => {
    return isMemoryCritical && estimatedMemoryMB > threshold;
  }, [isMemoryCritical, estimatedMemoryMB]);
  
  const getSimplifiedProps = useCallback(<T extends object>(
    props: T,
    simplificationRules: Partial<T>
  ): T => {
    if (isMemoryCritical) {
      return { ...props, ...simplificationRules };
    }
    return props;
  }, [isMemoryCritical]);
  
  return {
    shouldSkipExpensiveRender,
    getSimplifiedProps,
    isMemoryCritical,
    estimatedMemoryMB,
  };
};
