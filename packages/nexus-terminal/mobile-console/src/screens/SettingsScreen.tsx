import React, { useState, useEffect } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  Alert,
  Share,
} from 'react-native';
import {
  Card,
  List,
  Switch,
  Button,
  Text,
  Dialog,
  Portal,
  TextInput,
} from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '@store/index';
import { resetConnection } from '@store/slices/connectionSlice';
import { ConnectionManager } from '@services/ConnectionManager';
import DeviceInfo from 'react-native-device-info';

interface AppSettings {
  enableNotifications: boolean;
  keepConnectionAlive: boolean;
  autoReconnect: boolean;
  commandTimeout: number;
  terminalFontSize: number;
  darkTheme: boolean;
  enableLogging: boolean;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

const DEFAULT_SETTINGS: AppSettings = {
  enableNotifications: true,
  keepConnectionAlive: true,
  autoReconnect: true,
  commandTimeout: 30,
  terminalFontSize: 14,
  darkTheme: true,
  enableLogging: false,
  logLevel: 'info',
};

export const SettingsScreen: React.FC = () => {
  const dispatch = useDispatch();
  const { config, status } = useSelector((state: RootState) => state.connection);
  
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [showResetDialog, setShowResetDialog] = useState(false);
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [exportData, setExportData] = useState('');
  const [appInfo, setAppInfo] = useState({
    version: '',
    buildNumber: '',
    deviceId: '',
  });

  useEffect(() => {
    loadSettings();
    loadAppInfo();
  }, []);

  const loadSettings = async () => {
    try {
      const savedSettings = await AsyncStorage.getItem('appSettings');
      if (savedSettings) {
        setSettings({ ...DEFAULT_SETTINGS, ...JSON.parse(savedSettings) });
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  };

  const loadAppInfo = async () => {
    try {
      const version = await DeviceInfo.getVersion();
      const buildNumber = await DeviceInfo.getBuildNumber();
      const deviceId = await DeviceInfo.getUniqueId();
      
      setAppInfo({ version, buildNumber, deviceId });
    } catch (error) {
      console.error('Failed to load app info:', error);
    }
  };

  const saveSettings = async (newSettings: AppSettings) => {
    try {
      await AsyncStorage.setItem('appSettings', JSON.stringify(newSettings));
      setSettings(newSettings);
    } catch (error) {
      Alert.alert('Error', 'Failed to save settings');
    }
  };

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    const newSettings = { ...settings, [key]: value };
    saveSettings(newSettings);
  };

  const handleDisconnect = () => {
    ConnectionManager.disconnect();
    dispatch(resetConnection());
    Alert.alert('Disconnected', 'Successfully disconnected from laptop');
  };

  const handleResetSettings = async () => {
    try {
      await AsyncStorage.removeItem('appSettings');
      await AsyncStorage.removeItem('connectionConfig');
      setSettings(DEFAULT_SETTINGS);
      setShowResetDialog(false);
      Alert.alert('Reset Complete', 'All settings have been reset to defaults');
    } catch (error) {
      Alert.alert('Error', 'Failed to reset settings');
    }
  };

  const handleExportLogs = async () => {
    try {
      // In a real app, you'd collect actual logs here
      const logs = {
        timestamp: new Date().toISOString(),
        appVersion: appInfo.version,
        deviceId: appInfo.deviceId,
        connectionConfig: config ? {
          host: config.host,
          port: config.port,
          secure: config.secure,
        } : null,
        settings,
        connectionStatus: status,
      };
      
      setExportData(JSON.stringify(logs, null, 2));
      setShowExportDialog(true);
    } catch (error) {
      Alert.alert('Error', 'Failed to export logs');
    }
  };

  const shareLogs = async () => {
    try {
      await Share.share({
        message: exportData,
        title: 'Nexus Terminal Logs',
      });
      setShowExportDialog(false);
    } catch (error) {
      Alert.alert('Error', 'Failed to share logs');
    }
  };

  const testConnection = async () => {
    if (!ConnectionManager.isConnected()) {
      Alert.alert('Not Connected', 'Please connect to your laptop first');
      return;
    }

    try {
      const result = await ConnectionManager.executeCommand({
        command: 'echo "Connection test successful"',
      });
      
      Alert.alert(
        'Connection Test',
        result.success ? 'Connection is working properly!' : 'Connection test failed',
        [{ text: 'OK' }]
      );
    } catch (error) {
      Alert.alert('Connection Test Failed', 'Unable to communicate with laptop');
    }
  };

  return (
    <View style={styles.container}>
      <ScrollView>
        {/* Connection Settings */}
        <Card style={styles.card}>
          <Card.Title 
            title="Connection" 
            left={() => <Icon name="link" size={24} color="#4CAF50" />}
          />
          <Card.Content>
            {config && (
              <View style={styles.connectionInfo}>
                <Text style={styles.connectionText}>
                  Connected to: {config.host}:{config.port}
                </Text>
                <Text style={styles.connectionStatus}>
                  Status: {status}
                </Text>
              </View>
            )}
            
            <Button
              mode="outlined"
              onPress={testConnection}
              style={styles.settingButton}
              disabled={status !== 'connected'}
            >
              Test Connection
            </Button>
            
            <Button
              mode="outlined"
              onPress={handleDisconnect}
              style={styles.settingButton}
              disabled={status === 'disconnected'}
            >
              Disconnect
            </Button>
          </Card.Content>
        </Card>

        {/* App Settings */}
        <Card style={styles.card}>
          <Card.Title 
            title="App Settings" 
            left={() => <Icon name="settings" size={24} color="#2196F3" />}
          />
          <Card.Content>
            <List.Item
              title="Enable Notifications"
              description="Receive notifications about system events"
              left={() => <Icon name="notifications" size={24} color="#FF9800" />}
              right={() => (
                <Switch
                  value={settings.enableNotifications}
                  onValueChange={(value) => updateSetting('enableNotifications', value)}
                />
              )}
            />
            
            <List.Item
              title="Keep Connection Alive"
              description="Maintain connection when app is in background"
              left={() => <Icon name="link" size={24} color="#4CAF50" />}
              right={() => (
                <Switch
                  value={settings.keepConnectionAlive}
                  onValueChange={(value) => updateSetting('keepConnectionAlive', value)}
                />
              )}
            />
            
            <List.Item
              title="Auto Reconnect"
              description="Automatically reconnect when connection is lost"
              left={() => <Icon name="refresh" size={24} color="#00BCD4" />}
              right={() => (
                <Switch
                  value={settings.autoReconnect}
                  onValueChange={(value) => updateSetting('autoReconnect', value)}
                />
              )}
            />
            
            <List.Item
              title="Dark Theme"
              description="Use dark theme throughout the app"
              left={() => <Icon name="dark-mode" size={24} color="#9C27B0" />}
              right={() => (
                <Switch
                  value={settings.darkTheme}
                  onValueChange={(value) => updateSetting('darkTheme', value)}
                />
              )}
            />
          </Card.Content>
        </Card>

        {/* Security Settings */}
        <Card style={styles.card}>
          <Card.Title 
            title="Security & Privacy" 
            left={() => <Icon name="security" size={24} color="#F44336" />}
          />
          <Card.Content>
            <List.Item
              title="Enable Logging"
              description="Log app activity for debugging"
              left={() => <Icon name="bug-report" size={24} color="#FF5722" />}
              right={() => (
                <Switch
                  value={settings.enableLogging}
                  onValueChange={(value) => updateSetting('enableLogging', value)}
                />
              )}
            />
            
            <Button
              mode="outlined"
              onPress={handleExportLogs}
              style={styles.settingButton}
            >
              Export Debug Information
            </Button>
          </Card.Content>
        </Card>

        {/* App Information */}
        <Card style={styles.card}>
          <Card.Title 
            title="App Information" 
            left={() => <Icon name="info" size={24} color="#607D8B" />}
          />
          <Card.Content>
            <List.Item
              title="Version"
              description={appInfo.version}
              left={() => <Icon name="info-outline" size={24} color="#4CAF50" />}
            />
            
            <List.Item
              title="Build Number"
              description={appInfo.buildNumber}
              left={() => <Icon name="build" size={24} color="#2196F3" />}
            />
            
            <List.Item
              title="Device ID"
              description={appInfo.deviceId}
              left={() => <Icon name="phone-android" size={24} color="#FF9800" />}
            />
          </Card.Content>
        </Card>

        {/* Reset Settings */}
        <Card style={styles.card}>
          <Card.Title 
            title="Reset" 
            left={() => <Icon name="restore" size={24} color="#F44336" />}
          />
          <Card.Content>
            <Button
              mode="outlined"
              onPress={() => setShowResetDialog(true)}
              style={styles.resetButton}
              buttonColor="#F44336"
              textColor="#F44336"
            >
              Reset All Settings
            </Button>
          </Card.Content>
        </Card>
      </ScrollView>

      {/* Reset Confirmation Dialog */}
      <Portal>
        <Dialog visible={showResetDialog} onDismiss={() => setShowResetDialog(false)}>
          <Dialog.Title>Reset Settings</Dialog.Title>
          <Dialog.Content>
            <Text>
              This will reset all settings to their default values and disconnect from the laptop.
              This action cannot be undone.
            </Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowResetDialog(false)}>Cancel</Button>
            <Button onPress={handleResetSettings} textColor="#F44336">
              Reset
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      {/* Export Logs Dialog */}
      <Portal>
        <Dialog visible={showExportDialog} onDismiss={() => setShowExportDialog(false)}>
          <Dialog.Title>Export Debug Information</Dialog.Title>
          <Dialog.Content>
            <Text style={styles.exportDescription}>
              This will share debug information that can help diagnose issues.
              No sensitive data like passwords or personal files are included.
            </Text>
            <TextInput
              mode="outlined"
              multiline
              numberOfLines={8}
              value={exportData}
              editable={false}
              style={styles.exportData}
            />
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowExportDialog(false)}>Cancel</Button>
            <Button onPress={shareLogs}>Share</Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  card: {
    margin: 8,
    backgroundColor: '#1e1e1e',
  },
  connectionInfo: {
    marginBottom: 16,
    padding: 12,
    backgroundColor: '#2a2a2a',
    borderRadius: 8,
  },
  connectionText: {
    fontSize: 14,
    color: '#4CAF50',
    marginBottom: 4,
  },
  connectionStatus: {
    fontSize: 12,
    color: '#ccc',
  },
  settingButton: {
    marginBottom: 8,
  },
  resetButton: {
    borderColor: '#F44336',
  },
  exportDescription: {
    fontSize: 14,
    color: '#ccc',
    marginBottom: 16,
  },
  exportData: {
    fontSize: 10,
    fontFamily: 'monospace',
  },
});
