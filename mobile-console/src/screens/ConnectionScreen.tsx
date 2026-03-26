import React, { useState, useEffect } from 'react';
import {
  View,
  StyleSheet,
  Alert,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
} from 'react-native';
import {
  Text,
  TextInput,
  Button,
  Card,
  Switch,
  ActivityIndicator,
  HelperText,
} from 'react-native-paper';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '@store/index';
import { setConnectionConfig, setConnectionStatus } from '@store/slices/connectionSlice';
import { ConnectionManager, ConnectionConfig } from '@services/ConnectionManager';
import DeviceInfo from 'react-native-device-info';

export const ConnectionScreen: React.FC = () => {
  const dispatch = useDispatch();
  const { status, config, error } = useSelector((state: RootState) => state.connection);
  
  const [host, setHost] = useState(config?.host || '');
  const [port, setPort] = useState(config?.port?.toString() || '8080');
  const [secure, setSecure] = useState(config?.secure || false);
  const [authToken, setAuthToken] = useState(config?.authToken || '');
  const [isConnecting, setIsConnecting] = useState(false);
  const [deviceId, setDeviceId] = useState('');

  useEffect(() => {
    DeviceInfo.getUniqueId().then(setDeviceId);
    
    // Listen for connection state changes
    const handleConnectionStateChange = (newStatus: string) => {
      dispatch(setConnectionStatus(newStatus as any));
      setIsConnecting(false);
    };

    ConnectionManager.on('connectionStateChanged', handleConnectionStateChange);
    
    return () => {
      ConnectionManager.off('connectionStateChanged', handleConnectionStateChange);
    };
  }, [dispatch]);

  const handleConnect = async () => {
    if (!host.trim()) {
      Alert.alert('Error', 'Please enter the laptop IP address or hostname');
      return;
    }

    const portNumber = parseInt(port, 10);
    if (isNaN(portNumber) || portNumber < 1 || portNumber > 65535) {
      Alert.alert('Error', 'Please enter a valid port number (1-65535)');
      return;
    }

    setIsConnecting(true);
    
    const connectionConfig: ConnectionConfig = {
      host: host.trim(),
      port: portNumber,
      secure,
      authToken: authToken.trim() || undefined,
    };

    try {
      dispatch(setConnectionConfig(connectionConfig));
      await ConnectionManager.setConnectionConfig(connectionConfig);
    } catch (err) {
      setIsConnecting(false);
      Alert.alert('Connection Failed', err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const handleDisconnect = () => {
    ConnectionManager.disconnect();
    dispatch(setConnectionStatus('disconnected'));
  };

  const getStatusColor = () => {
    switch (status) {
      case 'connected': return '#4CAF50';
      case 'connecting': return '#FF9800';
      case 'error': return '#F44336';
      default: return '#888';
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'connected': return 'Connected';
      case 'connecting': return 'Connecting...';
      case 'error': return 'Connection Error';
      default: return 'Disconnected';
    }
  };

  return (
    <KeyboardAvoidingView 
      style={styles.container} 
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      <ScrollView contentContainerStyle={styles.scrollContent}>
        <Card style={styles.card}>
          <Card.Title title="Connection Status" />
          <Card.Content>
            <View style={styles.statusRow}>
              <View style={[styles.statusIndicator, { backgroundColor: getStatusColor() }]} />
              <Text style={[styles.statusText, { color: getStatusColor() }]}>
                {getStatusText()}
              </Text>
              {(status === 'connecting' || isConnecting) && (
                <ActivityIndicator size="small" color="#4CAF50" style={styles.spinner} />
              )}
            </View>
            {error && <HelperText type="error">{error}</HelperText>}
            {deviceId && (
              <Text style={styles.deviceId}>Device ID: {deviceId}</Text>
            )}
          </Card.Content>
        </Card>

        <Card style={styles.card}>
          <Card.Title title="Connection Settings" />
          <Card.Content>
            <TextInput
              mode="outlined"
              label="Laptop IP Address or Hostname"
              value={host}
              onChangeText={setHost}
              placeholder="192.168.1.100"
              autoCapitalize="none"
              autoCorrect={false}
              disabled={status === 'connected'}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Port"
              value={port}
              onChangeText={setPort}
              placeholder="8080"
              keyboardType="numeric"
              disabled={status === 'connected'}
              style={styles.input}
            />

            <View style={styles.switchRow}>
              <Text>Use HTTPS/WSS</Text>
              <Switch
                value={secure}
                onValueChange={setSecure}
                disabled={status === 'connected'}
              />
            </View>
            <HelperText type="info">
              Enable for encrypted connections (requires SSL certificate on laptop)
            </HelperText>

            <TextInput
              mode="outlined"
              label="Authentication Token (Optional)"
              value={authToken}
              onChangeText={setAuthToken}
              placeholder="Enter auth token if required"
              autoCapitalize="none"
              autoCorrect={false}
              secureTextEntry
              disabled={status === 'connected'}
              style={styles.input}
            />
          </Card.Content>
        </Card>

        <Card style={styles.card}>
          <Card.Title title="Setup Instructions" />
          <Card.Content>
            <Text style={styles.instructions}>
              1. Ensure Nexus Terminal is running on your laptop
              {'\n'}2. Both devices should be on the same network
              {'\n'}3. Enter your laptop's IP address above
              {'\n'}4. Use the same port as configured in Nexus Terminal
              {'\n'}5. Enable HTTPS if you have SSL certificates configured
            </Text>
          </Card.Content>
        </Card>

        <View style={styles.buttonContainer}>
          {status === 'connected' ? (
            <Button
              mode="contained"
              onPress={handleDisconnect}
              style={styles.button}
              buttonColor="#F44336"
            >
              Disconnect
            </Button>
          ) : (
            <Button
              mode="contained"
              onPress={handleConnect}
              loading={isConnecting || status === 'connecting'}
              disabled={isConnecting || status === 'connecting'}
              style={styles.button}
            >
              Connect to Laptop
            </Button>
          )}
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  scrollContent: {
    padding: 16,
  },
  card: {
    marginBottom: 16,
    backgroundColor: '#1e1e1e',
  },
  input: {
    marginBottom: 8,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  statusIndicator: {
    width: 12,
    height: 12,
    borderRadius: 6,
    marginRight: 8,
  },
  statusText: {
    fontSize: 16,
    fontWeight: 'bold',
    flex: 1,
  },
  spinner: {
    marginLeft: 8,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginVertical: 8,
  },
  instructions: {
    fontSize: 14,
    lineHeight: 20,
    color: '#ccc',
  },
  buttonContainer: {
    marginTop: 16,
  },
  button: {
    paddingVertical: 8,
  },
  deviceId: {
    fontSize: 12,
    color: '#888',
    marginTop: 8,
  },
});
