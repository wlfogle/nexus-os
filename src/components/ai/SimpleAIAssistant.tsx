import React, { useState } from 'react';

const SimpleAIAssistant: React.FC = () => {
  const [messages, setMessages] = useState([
    { role: 'assistant', content: 'Hello! I can help you with your terminal and code.' },
    { role: 'user', content: 'Test message from user' },
    { role: 'assistant', content: 'This is a response from the AI assistant that should be scrollable when there are many messages.' }
  ]);
  const [input, setInput] = useState('');

  const sendMessage = () => {
    if (!input.trim()) return;
    setMessages(prev => [...prev, 
      { role: 'user', content: input },
      { role: 'assistant', content: `You said: "${input}". This is a test response.` }
    ]);
    setInput('');
  };

  return (
    <div style={{
      width: '100%',
      height: '100vh',
      display: 'flex',
      flexDirection: 'column',
      backgroundColor: '#1a1a1a',
      color: 'white'
    }}>
      {/* Title Bar */}
      <div style={{
        height: '40px',
        backgroundColor: '#333',
        display: 'flex',
        alignItems: 'center',
        padding: '0 16px',
        borderBottom: '1px solid #555',
        fontSize: '14px',
        fontWeight: 'bold'
      }}>
        🤖 Nexus AI Assistant
      </div>

      {/* Messages Area - SCROLLABLE */}
      <div style={{
        flex: 1,
        overflowY: 'scroll',
        padding: '16px',
        backgroundColor: '#1a1a1a'
      }}>
        {messages.map((msg, i) => (
          <div key={i} style={{
            marginBottom: '12px',
            display: 'flex',
            justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start'
          }}>
            <div style={{
              maxWidth: '80%',
              padding: '8px 12px',
              borderRadius: '8px',
              backgroundColor: msg.role === 'user' ? '#0066cc' : '#333',
              color: 'white'
            }}>
              {msg.content}
            </div>
          </div>
        ))}
        
        {/* Add lots of test content to force scrolling */}
        {Array.from({length: 50}, (_, i) => (
          <div key={`test-${i}`} style={{
            padding: '8px',
            margin: '4px 0',
            backgroundColor: '#2a2a2a',
            borderRadius: '4px',
            fontSize: '12px'
          }}>
            Test scrollable content line {i + 1}
          </div>
        ))}
      </div>

      {/* Input Area */}
      <div style={{
        height: '60px',
        backgroundColor: '#2a2a2a',
        borderTop: '1px solid #555',
        display: 'flex',
        alignItems: 'center',
        padding: '0 16px',
        gap: '8px'
      }}>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Type your message..."
          style={{
            flex: 1,
            padding: '8px 12px',
            borderRadius: '4px',
            border: '1px solid #555',
            backgroundColor: '#1a1a1a',
            color: 'white',
            fontSize: '14px'
          }}
        />
        <button
          onClick={sendMessage}
          style={{
            padding: '8px 16px',
            borderRadius: '4px',
            border: 'none',
            backgroundColor: '#0066cc',
            color: 'white',
            cursor: 'pointer',
            fontSize: '14px'
          }}
        >
          Send
        </button>
      </div>
    </div>
  );
};

export default SimpleAIAssistant;
