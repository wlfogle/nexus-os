import React, { useState, useEffect } from 'react';
import { cn } from '../../lib/utils';

interface DebugLog {
  id: string;
  timestamp: Date;
  type: 'info' | 'shell' | 'ai' | 'error';
  message: string;
}

interface DebugOverlayProps {
  isVisible: boolean;
  onToggle: () => void;
}

export const DebugOverlay: React.FC<DebugOverlayProps> = ({ isVisible, onToggle }) => {
  const [logs, setLogs] = useState<DebugLog[]>([]);

  useEffect(() => {
    if (!isVisible) return;

    // Override console methods to capture logs
    const originalConsoleLog = console.log;
    const originalConsoleError = console.error;

    console.log = (...args) => {
      const message = args.join(' ');
      
      // Capture ALL console logs for debugging
      const type = message.includes('🐚') ? 'shell' : 
                  message.includes('🤖') ? 'ai' : 
                  message.includes('❌') ? 'error' : 'info';
      
      setLogs(prev => [...prev, {
        id: Date.now().toString(),
        timestamp: new Date(),
        type,
        message
      }]);
      
      originalConsoleLog(...args);
    };

    console.error = (...args) => {
      const message = args.join(' ');
      setLogs(prev => [...prev, {
        id: Date.now().toString(),
        timestamp: new Date(),
        type: 'error',
        message
      }]);
      originalConsoleError(...args);
    };

    return () => {
      console.log = originalConsoleLog;
      console.error = originalConsoleError;
    };
  }, [isVisible]);

  if (!isVisible) {
    return (
      <button
        onClick={onToggle}
        className="fixed top-4 right-4 z-50 bg-gray-800 text-white px-3 py-1 rounded text-sm font-mono hover:bg-gray-700 transition-colors"
        title="Show Debug Logs (Ctrl+Shift+D)"
      >
        DEBUG
      </button>
    );
  }

  return (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-end">
      <div className="w-full h-1/2 bg-black text-green-400 font-mono text-xs p-4 overflow-hidden flex flex-col">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-bold">🐛 NexusTerminal Debug Console</h3>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setLogs([])}
              className="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs"
            >
              Clear
            </button>
            <button
              onClick={onToggle}
              className="px-2 py-1 bg-red-700 hover:bg-red-600 rounded text-xs"
            >
              Close
            </button>
          </div>
        </div>
        
        <div className="text-xs mb-2 text-gray-400">
          💡 Type commands in the AI input to see routing debug logs
        </div>
        
        <div className="flex-1 overflow-y-auto space-y-1">
          {logs.length === 0 ? (
            <div className="text-gray-500">No debug logs yet. Try typing a command like "ls -la"</div>
          ) : (
            logs.map(log => (
              <div key={log.id} className="flex gap-2">
                <span className="text-gray-500 w-20 flex-shrink-0">
                  {log.timestamp.toLocaleTimeString()}
                </span>
                <span className={cn(
                  'w-12 flex-shrink-0',
                  log.type === 'shell' && 'text-blue-400',
                  log.type === 'ai' && 'text-purple-400', 
                  log.type === 'error' && 'text-red-400',
                  log.type === 'info' && 'text-green-400'
                )}>
                  [{log.type.toUpperCase()}]
                </span>
                <span className="flex-1 break-all">{log.message}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export const useDebugOverlay = () => {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.shiftKey && event.key === 'D') {
        event.preventDefault();
        setIsVisible(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return {
    isVisible,
    toggleDebug: () => setIsVisible(prev => !prev)
  };
};
