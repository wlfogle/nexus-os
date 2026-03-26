import React from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '../store';
import { createTerminal, closeTerminal, setActiveTerminal } from '../store/slices/terminalSlice';
import { clearMessages } from '../store/slices/aiSlice';
import { invoke } from '@tauri-apps/api/core';

const Toolbar: React.FC = () => {
  const dispatch = useDispatch();
  const { terminals, activeTerminalId } = useSelector((state: RootState) => state.terminal);
  const { isConnected } = useSelector((state: RootState) => state.ai);

  const handleNewTerminal = async () => {
    try {
      const terminalId = await invoke<string>('create_terminal', { shell: null });
      dispatch(createTerminal({ id: terminalId }));
    } catch (error) {
      console.error('Failed to create terminal:', error);
    }
  };

  const handleCloseTerminal = async (terminalId: string) => {
    try {
      await invoke('close_terminal', { terminalId });
      dispatch(closeTerminal(terminalId));
    } catch (error) {
      console.error('Failed to close terminal:', error);
    }
  };

  const handleSwitchTerminal = (terminalId: string) => {
    dispatch(setActiveTerminal(terminalId));
  };

  const handleClearAIChat = () => {
    dispatch(clearMessages());
  };

  const handleRestartAI = async () => {
    try {
      await invoke('restart_ai_service');
    } catch (error) {
      console.error('Failed to restart AI service:', error);
    }
  };

  return (
    <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
      {/* Left section - Terminal controls */}
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-2">
          <h1 className="text-lg font-bold text-white flex items-center space-x-2">
            <span>🔗</span>
            <span>NexusTerminal</span>
          </h1>
          <span className="text-xs text-gray-500 bg-gray-700 px-2 py-1 rounded">
            v0.1.0-beta
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={handleNewTerminal}
            className="flex items-center space-x-1 px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 transition-colors"
            title="New Terminal (Ctrl+Shift+T)"
          >
            <span>➕</span>
            <span>New Tab</span>
          </button>

          {/* Terminal tabs */}
          <div className="flex space-x-1">
            {Object.entries(terminals).map(([id, terminal]) => (
              <div
                key={id}
                className={`flex items-center space-x-2 px-3 py-1 rounded text-sm cursor-pointer transition-colors ${
                  id === activeTerminalId
                    ? 'bg-gray-700 text-white'
                    : 'bg-gray-600 text-gray-300 hover:bg-gray-700'
                }`}
                onClick={() => handleSwitchTerminal(id)}
              >
                <span>{terminal.title}</span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleCloseTerminal(id);
                  }}
                  className="text-gray-400 hover:text-red-400 transition-colors"
                  title="Close Terminal"
                >
                  ✕
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Right section - AI controls and status */}
      <div className="flex items-center space-x-4">
        {/* AI Status */}
        <div className="flex items-center space-x-2 text-sm">
          <div className={`flex items-center space-x-1 px-2 py-1 rounded ${
            isConnected ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'
          }`}>
            <div className={`w-2 h-2 rounded-full ${
              isConnected ? 'bg-green-400' : 'bg-red-400'
            }`} />
            <span>{isConnected ? 'AI Online' : 'AI Offline'}</span>
          </div>
        </div>

        {/* AI Controls */}
        <div className="flex items-center space-x-2">
          <button
            onClick={handleClearAIChat}
            className="px-2 py-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded text-sm transition-colors"
            title="Clear AI Chat"
          >
            🗑️
          </button>
          
          <button
            onClick={handleRestartAI}
            className="px-2 py-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded text-sm transition-colors"
            title="Restart AI Service"
          >
            🔄
          </button>
        </div>

        {/* System Info */}
        <div className="flex items-center space-x-2 text-xs text-gray-500">
          <span>🐧 Garuda Linux</span>
          <span>•</span>
          <span>🦀 Rust+Tauri</span>
          <span>•</span>
          <span>⚛️ React</span>
        </div>

        {/* Window Controls */}
        <div className="flex items-center space-x-1">
          <button
            onClick={() => invoke('minimize_window')}
            className="w-6 h-6 bg-yellow-500 rounded-full hover:bg-yellow-600 transition-colors"
            title="Minimize"
          />
          <button
            onClick={() => invoke('toggle_maximize')}
            className="w-6 h-6 bg-green-500 rounded-full hover:bg-green-600 transition-colors"
            title="Maximize/Restore"
          />
          <button
            onClick={() => invoke('close_window')}
            className="w-6 h-6 bg-red-500 rounded-full hover:bg-red-600 transition-colors"
            title="Close"
          />
        </div>
      </div>
    </div>
  );
};

export default Toolbar;
