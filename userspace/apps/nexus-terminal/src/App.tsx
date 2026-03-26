import { useEffect, useState } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Provider } from 'react-redux';
import { store } from './store';
import { WarpStyleTerminal } from './components/terminal/WarpStyleTerminal';
import { invoke } from '@tauri-apps/api/core';
import { logger } from './utils/logger';

function App() {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    // Initialize the app
    const initApp = async () => {
      try {
        logger.info('Starting NexusTerminal initialization', { component: 'App', action: 'initialize' });
        
        // Check if we're in Tauri context
        const isTauriContext = typeof window !== 'undefined' && (window as any).__TAURI__;
        logger.debug('Environment detection', { component: 'App', action: 'env_detect', metadata: { isTauriContext } });
        
        if (isTauriContext) {
          // Initialize AI service and terminal in Tauri context
          try {
            logger.info('Connecting to Tauri backend', { component: 'App', action: 'tauri_connect' });
            await invoke('get_system_info');
            logger.info('Tauri backend connected successfully', { component: 'App', action: 'tauri_connected' });
          } catch (error) {
            logger.warn('Could not connect to Tauri backend', { component: 'App', action: 'tauri_connect_failed' }, error as Error);
            // Continue anyway - app should still work
          }
        } else {
          // Browser context - mock initialization
          logger.info('Running in browser mode', { component: 'App', action: 'browser_mode' });
        }
        
        logger.info('App initialization complete', { component: 'App', action: 'init_complete' });
        setIsReady(true);
      } catch (error) {
        logger.error('Failed to initialize app', { component: 'App', action: 'init_failed' }, error as Error);
        setIsReady(true); // Still allow the app to load
      }
    };

    initApp();
  }, []);

  if (!isReady) {
    return (
      <div className="flex items-center justify-center h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-500 mx-auto"></div>
          <p className="mt-4 text-lg">Initializing NexusTerminal...</p>
          <p className="mt-2 text-sm text-gray-400">Loading AI-powered terminal...</p>
        </div>
      </div>
    );
  }

  return (
    <Provider store={store}>
      <Router>
        <div className="app h-screen bg-gray-900 text-white">
          <Routes>
            <Route path="/" element={<WarpStyleTerminal />} />
          </Routes>
        </div>
      </Router>
    </Provider>
  );
}

export default App;
