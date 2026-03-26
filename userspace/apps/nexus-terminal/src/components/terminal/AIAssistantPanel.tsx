import React, { useState, useRef, useEffect, useCallback } from 'react';
import { useDispatch } from 'react-redux';
import { 
  XMarkIcon, 
  PaperAirplaneIcon,
  TrashIcon,
  ClipboardIcon,
  ExclamationTriangleIcon 
} from '@heroicons/react/24/outline';
import { TerminalTab, AIMessage } from '../../types/terminal';
import { clearAIConversation } from '../../store/slices/terminalTabSlice';
import { cn } from '../../lib/utils';

interface AIAssistantPanelProps {
  isOpen: boolean;
  onClose: () => void;
  tab: TerminalTab;
  onSendMessage: (message: string) => void;
}

const MessageBubble: React.FC<{ message: AIMessage }> = ({ message }) => {
  const isUser = message.role === 'user';
  
  const formatTimestamp = (timestamp: Date) => {
    return new Date(timestamp).toLocaleTimeString([], { 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  };

  const getRoleIcon = (role: string) => {
    switch (role) {
      case 'user': return '👤';
      case 'assistant': return '🤖';
      case 'system': return '⚙️';
      default: return '💬';
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      // Could add a toast notification here
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  };

  return (
    <div className={cn('flex mb-4', isUser ? 'justify-end' : 'justify-start')}>
      <div className={cn(
        'max-w-[80%] rounded-lg px-4 py-3 shadow-lg',
        isUser 
          ? 'bg-blue-600 text-white' 
          : message.role === 'system'
          ? 'bg-gray-700 text-gray-200'
          : 'bg-gray-800 text-white'
      )}>
        <div className="flex items-start justify-between mb-2">
          <div className="flex items-center">
            <span className="text-sm mr-2">{getRoleIcon(message.role)}</span>
            <span className="text-xs opacity-70">
              {formatTimestamp(message.timestamp)}
            </span>
          </div>
          
          {!isUser && (
            <button
              onClick={() => copyToClipboard(message.content)}
              className="ml-2 opacity-50 hover:opacity-100 transition-opacity"
              title="Copy to clipboard"
            >
              <ClipboardIcon className="w-3 h-3" />
            </button>
          )}
        </div>
        
        <div className="whitespace-pre-wrap text-sm leading-relaxed">
          {message.content}
        </div>
      </div>
    </div>
  );
};

const ErrorSummary: React.FC<{ tab: TerminalTab }> = ({ tab }) => {
  if (tab.aiContext.errors.length === 0) return null;

  const latestError = tab.aiContext.errors[tab.aiContext.errors.length - 1];
  
  return (
    <div className="bg-red-900/20 border border-red-800 rounded-lg p-3 mb-4">
      <div className="flex items-center mb-2">
        <ExclamationTriangleIcon className="w-4 h-4 text-red-400 mr-2" />
        <span className="text-red-400 text-sm font-medium">
          Recent Error ({tab.aiContext.errors.length} total)
        </span>
      </div>
      <div className="text-red-300 text-xs font-mono">
        {latestError.command}: {latestError.errorMessage.slice(0, 100)}...
      </div>
    </div>
  );
};

const SuggestionSummary: React.FC<{ tab: TerminalTab }> = ({ tab }) => {
  if (tab.aiContext.suggestions.length === 0) return null;

  return (
    <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-3 mb-4">
      <div className="flex items-center mb-2">
        <span className="text-yellow-400 text-sm font-medium">
          💡 {tab.aiContext.suggestions.length} AI Suggestions Available
        </span>
      </div>
      <div className="space-y-1">
        {tab.aiContext.suggestions.slice(-2).map((suggestion) => (
          <div key={suggestion.id} className="text-yellow-300 text-xs">
            {suggestion.title}: {suggestion.description.slice(0, 60)}...
          </div>
        ))}
      </div>
    </div>
  );
};

export const AIAssistantPanel: React.FC<AIAssistantPanelProps> = ({
  isOpen,
  onClose,
  tab,
  onSendMessage
}) => {
  const dispatch = useDispatch();
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [tab.aiConversation, scrollToBottom]);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const handleSendMessage = useCallback(async () => {
    if (!input.trim() || isLoading) return;

    const message = input.trim();
    setInput('');
    setIsLoading(true);

    try {
      await onSendMessage(message);
    } finally {
      setIsLoading(false);
    }
  }, [input, isLoading, onSendMessage]);

  const handleKeyPress = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      if (e.shiftKey) {
        // Allow new line
        return;
      } else {
        e.preventDefault();
        handleSendMessage();
      }
    }
  }, [handleSendMessage]);

  const handleClearConversation = useCallback(() => {
    if (confirm('Are you sure you want to clear this conversation? This action cannot be undone.')) {
      dispatch(clearAIConversation(tab.id));
    }
  }, [dispatch, tab.id]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-2xl w-full max-w-4xl mx-4 h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <div className="flex items-center">
            <span className="text-xl mr-2">🤖</span>
            <div>
              <h3 className="text-lg font-semibold text-white">AI Assistant</h3>
              <p className="text-sm text-gray-400">
                {tab.shell} • {tab.workingDirectory}
              </p>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <button
              onClick={handleClearConversation}
              className="p-2 text-gray-400 hover:text-red-400 hover:bg-gray-700 rounded transition-colors"
              title="Clear conversation"
            >
              <TrashIcon className="w-4 h-4" />
            </button>
            <button
              onClick={onClose}
              className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
            >
              <XMarkIcon className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Context Info */}
        <div className="p-4 border-b border-gray-700 bg-gray-800/50">
          <ErrorSummary tab={tab} />
          <SuggestionSummary tab={tab} />
          
          <div className="flex items-center text-xs text-gray-500 space-x-4">
            <span>Commands: {tab.aiContext.recentCommands.length}</span>
            <span>Errors: {tab.aiContext.errors.length}</span>
            <span>Messages: {tab.aiConversation.length}</span>
          </div>
        </div>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-4">
          {tab.aiConversation.length === 0 ? (
            <div className="text-center text-gray-400 mt-8">
              <div className="text-4xl mb-4">🤖</div>
              <div className="text-lg mb-2">AI Assistant Ready</div>
              <div className="text-sm">
                Ask me anything about your {tab.shell} session, commands, errors, or development tasks.
              </div>
              <div className="mt-4 text-xs">
                Quick suggestions:
                <div className="mt-2 space-x-2">
                  <button
                    onClick={() => setInput("Explain what I can do with this shell")}
                    className="px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs hover:bg-gray-600"
                  >
                    Shell help
                  </button>
                  <button
                    onClick={() => setInput("Analyze my recent commands")}
                    className="px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs hover:bg-gray-600"
                  >
                    Command analysis
                  </button>
                </div>
              </div>
            </div>
          ) : (
            <>
              {tab.aiConversation.map((message) => (
                <MessageBubble key={message.id} message={message} />
              ))}
              
              {isLoading && (
                <div className="flex justify-start mb-4">
                  <div className="bg-gray-800 rounded-lg px-4 py-3 shadow-lg">
                    <div className="flex items-center">
                      <span className="text-sm mr-2">🤖</span>
                      <div className="flex space-x-1">
                        <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse"></div>
                        <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse delay-100"></div>
                        <div className="w-2 h-2 bg-gray-400 rounded-full animate-pulse delay-200"></div>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </>
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div className="p-4 border-t border-gray-700">
          <div className="flex space-x-3">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Ask AI about commands, errors, or development tasks..."
              disabled={isLoading}
              className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 resize-none focus:outline-none focus:border-blue-500 transition-colors"
              rows={2}
            />
            <button
              onClick={handleSendMessage}
              disabled={!input.trim() || isLoading}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center"
            >
              <PaperAirplaneIcon className="w-4 h-4 mr-2" />
              Send
            </button>
          </div>
          
          <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
            <div>Press Enter to send, Shift+Enter for new line</div>
            <div>Context: {tab.shell} in {tab.workingDirectory}</div>
          </div>
        </div>
      </div>
    </div>
  );
};
