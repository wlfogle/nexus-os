import React from 'react';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { createStackNavigator } from '@react-navigation/stack';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { useSelector } from 'react-redux';
import { RootState } from '@store/index';

// Screens
import { ConnectionScreen } from '@screens/ConnectionScreen';
import { DashboardScreen } from '@screens/DashboardScreen';
import { TerminalScreen } from '@screens/TerminalScreen';
import { FilesScreen } from '@screens/FilesScreen';
import { SystemScreen } from '@screens/SystemScreen';
import { EcosystemScreen } from '@screens/EcosystemScreen';
import { SettingsScreen } from '@screens/SettingsScreen';
import { AuthScreen } from '@screens/AuthScreen';

export type RootStackParamList = {
  Connection: undefined;
  MainTabs: undefined;
  FileEditor: { filePath: string; content: string };
};

export type TabParamList = {
  Dashboard: undefined;
  Terminal: undefined;
  Files: undefined;
  System: undefined;
  Ecosystem: undefined;
  Settings: undefined;
  Auth: undefined;
};

const Stack = createStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<TabParamList>();

const MainTabs: React.FC = () => {
  return (
    <Tab.Navigator
      screenOptions={({ route }) => ({
        tabBarIcon: ({ focused, color, size }) => {
          let iconName: string;

          switch (route.name) {
            case 'Dashboard':
              iconName = 'dashboard';
              break;
            case 'Terminal':
              iconName = 'terminal';
              break;
            case 'Files':
              iconName = 'folder';
              break;
            case 'System':
              iconName = 'computer';
              break;
            case 'Ecosystem':
              iconName = 'eco';
              break;
            case 'Settings':
              iconName = 'settings';
              break;
            case 'Auth':
              iconName = 'security';
              break;
            default:
              iconName = 'help';
          }

          return <Icon name={iconName} size={size} color={color} />;
        },
        tabBarActiveTintColor: '#4CAF50',
        tabBarInactiveTintColor: '#888',
        tabBarStyle: {
          backgroundColor: '#1a1a1a',
          borderTopColor: '#333',
        },
        headerStyle: {
          backgroundColor: '#1a1a1a',
        },
        headerTintColor: '#fff',
      })}
    >
      <Tab.Screen name="Dashboard" component={DashboardScreen} />
      <Tab.Screen name="Terminal" component={TerminalScreen} />
      <Tab.Screen name="Files" component={FilesScreen} />
      <Tab.Screen name="System" component={SystemScreen} />
      <Tab.Screen name="Ecosystem" component={EcosystemScreen} />
      <Tab.Screen name="Auth" component={AuthScreen} options={{ title: 'Authentication' }} />
      <Tab.Screen name="Settings" component={SettingsScreen} />
    </Tab.Navigator>
  );
};

export const MainNavigator: React.FC = () => {
  const connectionStatus = useSelector((state: RootState) => state.connection.status);

  return (
    <Stack.Navigator
      screenOptions={{
        headerStyle: { backgroundColor: '#1a1a1a' },
        headerTintColor: '#fff',
      }}
    >
      {connectionStatus === 'connected' ? (
        <Stack.Screen
          name="MainTabs"
          component={MainTabs}
          options={{ headerShown: false }}
        />
      ) : (
        <Stack.Screen
          name="Connection"
          component={ConnectionScreen}
          options={{ title: 'Connect to Laptop' }}
        />
      )}
    </Stack.Navigator>
  );
};
