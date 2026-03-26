import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { RootState } from '../store';
import { addMessage, setLoading, setConnected, setCurrentModel } from '../store/slices/aiSlice';

// Memoized message item component
const MessageItem = React.memo<{
  message: { id: string; content: string; role: string; timestamp: number };
  getRoleIcon: (role: string) => string;
  formatTimestamp: (timestamp: number) => string;
}>(({ message, getRoleIcon, formatTimestamp }) => (
  <div className={`flex ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}>
    <div className={`max-w-[85%] rounded-lg p-3 ${
      message.role === 'user'
        ? 'bg-blue-600 text-white'
        : message.role === 'assistant'
        ? 'bg-gray-700 text-white'
        : 'bg-gray-800 text-gray-300'
    }`}>
      <div className="flex items-start space-x-2">
        <span className="text-sm">
          {getRoleIcon(message.role)}
        </span>
        <div className="flex-1">
          <div className="whitespace-pre-wrap text-sm">
            {message.content}
          </div>
          <div className="text-xs opacity-70 mt-1">
            {formatTimestamp(message.timestamp)}
          </div>
        </div>
      </div>
    </div>
  </div>
));

MessageItem.displayName = 'MessageItem';

// Loading indicator component
const LoadingIndicator = React.memo(() => (
  <div className="flex justify-start">
    <div className="bg-gray-700 rounded-lg p-3 max-w-[85%]">
      <div className="flex items-center space-x-2">
        <span>🤖</span>
        <div className="flex space-x-1">
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse" />
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse delay-100" />
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse delay-200" />
        </div>
      </div>
    </div>
  </div>
));

LoadingIndicator.displayName = 'LoadingIndicator';

const AIAssistant: React.FC = () => {
  const [input, setInput] = useState('');
  const [isExpanded, setIsExpanded] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  
  const dispatch = useDispatch();
  const { messages, isLoading, currentModel, isConnected } = useSelector((state: RootState) => state.ai);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, scrollToBottom]);

  useEffect(() => {
    // Initialize AI connection
    const initializeAI = async () => {
      try {
        const status = await invoke<boolean>('check_ai_connection');
        dispatch(setConnected(status));
        
        if (status) {
          const model = await invoke<string>('get_current_model');
          dispatch(setCurrentModel(model));
          
          dispatch(addMessage({
            role: 'assistant',
            content: '🤖 AI Assistant connected! I can help you with terminal commands, code analysis, system management, and more.'
          }));
        } else {
          dispatch(addMessage({
            role: 'assistant',
            content: '⚠️ AI service not available. Please check Ollama installation.'
          }));
        }
      } catch (error) {
        console.error('Failed to initialize AI:', error);
        dispatch(setConnected(false));
      }
    };

    initializeAI();
  }, [dispatch]);

  const handleSendMessage = useCallback(async () => {
    if (!input.trim() || isLoading) return;

    const userMessage = input.trim();
    setInput('');

    // Add user message
    dispatch(addMessage({
      role: 'user',
      content: userMessage
    }));

    // Set loading state
    dispatch(setLoading(true));

    try {
      // Create structured context object
      const context = {
        workingDirectory: process.cwd || '~',
        shell: 'bash',
        timestamp: Date.now(),
        source: 'ai_assistant'
      };
      
      // Send to AI service
      const response = await invoke<string>('send_ai_message', {
        message: userMessage,
        context: context
      });

      // Add AI response
      dispatch(addMessage({
        role: 'assistant',
        content: response
      }));

    } catch (error) {
      console.error('Failed to send message:', error);
      dispatch(addMessage({
        role: 'assistant',
        content: '❌ Failed to get AI response. Please try again.'
      }));
    } finally {
      dispatch(setLoading(false));
    }
  }, [input, isLoading, dispatch]);

  const handleKeyPress = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  }, [handleSendMessage]);

  const formatTimestamp = useCallback((timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString([], { 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  }, []);

  const getRoleIcon = useCallback((role: string) => {
    switch (role) {
      case 'user':
        return '👤';
      case 'assistant':
        return '🤖';
      case 'system':
        return '⚙️';
      default:
        return '💬';
    }
  }, []);

  const toggleExpanded = useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  // Memoize messages list to prevent unnecessary re-renders
  const memoizedMessages = useMemo(() => messages, [messages]);

  // Memoize connection status indicator
  const connectionStatusClass = useMemo(() => 
    isConnected ? 'bg-green-400' : 'bg-red-400', 
    [isConnected]
  );

  // Memoize expansion button
  const expansionButton = useMemo(() => 
    isExpanded ? '⬅️' : '➡️', 
    [isExpanded]
  );


  return (
    <div className={`flex flex-col bg-gray-900 border-l border-gray-700 transition-all duration-300 ${
      isExpanded ? 'w-96' : 'w-80'
    }`}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-2">
          <span className="text-lg">🤖</span>
          <span className="font-semibold text-white">AI Assistant</span>
          <div className={`w-2 h-2 rounded-full ${connectionStatusClass}`} />
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={toggleExpanded}
            className="text-gray-400 hover:text-white transition-colors"
            title={isExpanded ? 'Collapse' : 'Expand'}
          >
            {expansionButton}
          </button>
          <span className="text-xs text-gray-500">
            {currentModel || 'No model'}
          </span>
        </div>
      </div>

      {/* Messages */}
      <div 
        className="flex-1 overflow-y-auto p-4 space-y-4 forced-scrollbars"
        style={{
          scrollbarWidth: 'thin',
          scrollbarColor: '#4a5568 #2d3748'
        }}
      >
        {memoizedMessages.map((message, index) => (
          <MessageItem
            key={`${message.id}-${index}`}
            message={message}
            getRoleIcon={getRoleIcon}
            formatTimestamp={formatTimestamp}
          />
        ))}
        
        {isLoading && <LoadingIndicator />}
        
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-gray-700 p-4">
        <div className="flex flex-col space-y-2">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={isConnected ? "Ask AI anything..." : "AI not connected"}
            disabled={!isConnected || isLoading}
            className="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white placeholder-gray-400 resize-none focus:outline-none focus:border-blue-500 transition-colors"
            rows={2}
          />
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2 text-xs text-gray-500">
              <span>⌨️ Enter to send</span>
              <span>•</span>
              <span>⇧+Enter for new line</span>
            </div>
            <button
              onClick={handleSendMessage}
              disabled={!input.trim() || !isConnected || isLoading}
              className="px-4 py-1 bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              Send
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AIAssistant;
