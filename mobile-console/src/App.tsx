import React, { useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { Provider as PaperProvider } from 'react-native-paper';
import { Provider as ReduxProvider } from 'react-redux';
import { StatusBar } from 'react-native';
import { store } from '@store/index';
import { theme } from '@utils/theme';
import { MainNavigator } from '@/navigation/MainNavigator';
import { ConnectionManager } from '@services/ConnectionManager';

const App: React.FC = () => {
  useEffect(() => {
    // Initialize connection manager
    ConnectionManager.initialize();
    
    return () => {
      ConnectionManager.cleanup();
    };
  }, []);

  return (
    <ReduxProvider store={store}>
      <PaperProvider theme={theme}>
        <NavigationContainer>
          <StatusBar barStyle="light-content" backgroundColor="#1a1a1a" />
          <MainNavigator />
        </NavigationContainer>
      </PaperProvider>
    </ReduxProvider>
  );
};

export default App;
