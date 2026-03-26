import React, { useState, useEffect } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  Alert,
  Platform,
} from 'react-native';
import {
  Card,
  TextInput,
  Button,
  Text,
  Switch,
  List,
  Dialog,
  Portal,
  ActivityIndicator,
  Chip,
} from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { useDispatch, useSelector } from 'react-redux';
import { RootState } from '@store/index';
import { setConfig, setAuthToken } from '@store/slices/connectionSlice';
import { ConnectionManager } from '@services/ConnectionManager';
import CryptoJS from 'crypto-js';

interface AuthConfig {
  authMethod: 'token' | 'certificate' | 'password';
  token?: string;
  username?: string;
  password?: string;
  certificate?: string;
  privateKey?: string;
  enableSSL: boolean;
  verifyHostname: boolean;
  allowSelfSigned: boolean;
}

const DEFAULT_AUTH: AuthConfig = {
  authMethod: 'token',
  enableSSL: true,
  verifyHostname: true,
  allowSelfSigned: false,
};

export const AuthScreen: React.FC = () => {
  const dispatch = useDispatch();
  const { config } = useSelector((state: RootState) => state.connection);
  
  const [authConfig, setAuthConfig] = useState<AuthConfig>(DEFAULT_AUTH);
  const [showPassword, setShowPassword] = useState(false);
  const [showCertDialog, setShowCertDialog] = useState(false);
  const [isGeneratingToken, setIsGeneratingToken] = useState(false);
  const [savedConfigs, setSavedConfigs] = useState<Array<{
    name: string;
    config: AuthConfig;
    timestamp: string;
  }>>([]);

  useEffect(() => {
    loadSavedConfigs();
    loadAuthConfig();
  }, []);

  const loadAuthConfig = async () => {
    try {
      const saved = await AsyncStorage.getItem('authConfig');
      if (saved) {
        setAuthConfig({ ...DEFAULT_AUTH, ...JSON.parse(saved) });
      }
    } catch (error) {
      console.error('Failed to load auth config:', error);
    }
  };

  const loadSavedConfigs = async () => {
    try {
      const saved = await AsyncStorage.getItem('savedAuthConfigs');
      if (saved) {
        setSavedConfigs(JSON.parse(saved));
      }
    } catch (error) {
      console.error('Failed to load saved configs:', error);
    }
  };

  const saveAuthConfig = async (config: AuthConfig) => {
    try {
      await AsyncStorage.setItem('authConfig', JSON.stringify(config));
      setAuthConfig(config);
      
      // Update Redux store
      if (config.token) {
        dispatch(setAuthToken(config.token));
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to save authentication configuration');
    }
  };

  const saveConfigWithName = async (name: string) => {
    try {
      const newConfig = {
        name,
        config: authConfig,
        timestamp: new Date().toISOString(),
      };
      
      const updated = [...savedConfigs.filter(c => c.name !== name), newConfig];
      await AsyncStorage.setItem('savedAuthConfigs', JSON.stringify(updated));
      setSavedConfigs(updated);
      
      Alert.alert('Saved', `Configuration "${name}" has been saved`);
    } catch (error) {
      Alert.alert('Error', 'Failed to save configuration');
    }
  };

  const loadSavedConfig = (configItem: typeof savedConfigs[0]) => {
    setAuthConfig(configItem.config);
    Alert.alert('Loaded', `Configuration "${configItem.name}" has been loaded`);
  };

  const deleteSavedConfig = async (name: string) => {
    try {
      const updated = savedConfigs.filter(c => c.name !== name);
      await AsyncStorage.setItem('savedAuthConfigs', JSON.stringify(updated));
      setSavedConfigs(updated);
    } catch (error) {
      Alert.alert('Error', 'Failed to delete configuration');
    }
  };

  const generateSecureToken = async () => {
    setIsGeneratingToken(true);
    
    try {
      // Generate a secure random token
      const randomBytes = CryptoJS.lib.WordArray.random(32);
      const token = randomBytes.toString(CryptoJS.enc.Hex);
      
      setAuthConfig(prev => ({ ...prev, token }));
      
      Alert.alert(
        'Token Generated',
        'A secure token has been generated. Make sure to configure your laptop with this token.',
        [{ text: 'OK' }]
      );
    } catch (error) {
      Alert.alert('Error', 'Failed to generate token');
    } finally {
      setIsGeneratingToken(false);
    }
  };

  const testAuthentication = async () => {
    if (!config) {
      Alert.alert('Error', 'Please configure connection first');
      return;
    }

    try {
      // Test connection with current auth config
      const testResult = await ConnectionManager.testConnection({
        ...config,
        authMethod: authConfig.authMethod,
        token: authConfig.token,
        username: authConfig.username,
        password: authConfig.password,
        enableSSL: authConfig.enableSSL,
      });

      Alert.alert(
        'Authentication Test',
        testResult.success 
          ? 'Authentication successful!' 
          : `Authentication failed: ${testResult.error}`,
        [{ text: 'OK' }]
      );
    } catch (error) {
      Alert.alert('Test Failed', 'Unable to test authentication');
    }
  };

  const validateAndSave = () => {
    // Validate configuration
    if (authConfig.authMethod === 'token' && !authConfig.token) {
      Alert.alert('Validation Error', 'Token is required for token authentication');
      return;
    }
    
    if (authConfig.authMethod === 'password' && (!authConfig.username || !authConfig.password)) {
      Alert.alert('Validation Error', 'Username and password are required');
      return;
    }
    
    if (authConfig.authMethod === 'certificate' && (!authConfig.certificate || !authConfig.privateKey)) {
      Alert.alert('Validation Error', 'Certificate and private key are required');
      return;
    }

    saveAuthConfig(authConfig);
    Alert.alert('Saved', 'Authentication configuration has been saved');
  };

  return (
    <View style={styles.container}>
      <ScrollView>
        {/* Authentication Method */}
        <Card style={styles.card}>
          <Card.Title 
            title="Authentication Method" 
            left={() => <Icon name="security" size={24} color="#4CAF50" />}
          />
          <Card.Content>
            <View style={styles.chipContainer}>
              <Chip
                selected={authConfig.authMethod === 'token'}
                onPress={() => setAuthConfig(prev => ({ ...prev, authMethod: 'token' }))}
                style={styles.chip}
                icon="vpn-key"
              >
                Token
              </Chip>
              <Chip
                selected={authConfig.authMethod === 'password'}
                onPress={() => setAuthConfig(prev => ({ ...prev, authMethod: 'password' }))}
                style={styles.chip}
                icon="password"
              >
                Password
              </Chip>
              <Chip
                selected={authConfig.authMethod === 'certificate'}
                onPress={() => setAuthConfig(prev => ({ ...prev, authMethod: 'certificate' }))}
                style={styles.chip}
                icon="certificate"
              >
                Certificate
              </Chip>
            </View>
          </Card.Content>
        </Card>

        {/* Token Authentication */}
        {authConfig.authMethod === 'token' && (
          <Card style={styles.card}>
            <Card.Title 
              title="Token Authentication" 
              left={() => <Icon name="vpn-key" size={24} color="#2196F3" />}
            />
            <Card.Content>
              <TextInput
                mode="outlined"
                label="Authentication Token"
                value={authConfig.token || ''}
                onChangeText={(text) => setAuthConfig(prev => ({ ...prev, token: text }))}
                secureTextEntry={!showPassword}
                right={
                  <TextInput.Icon
                    icon={showPassword ? 'eye-off' : 'eye'}
                    onPress={() => setShowPassword(!showPassword)}
                  />
                }
                style={styles.input}
              />
              
              <Button
                mode="outlined"
                onPress={generateSecureToken}
                loading={isGeneratingToken}
                disabled={isGeneratingToken}
                style={styles.button}
              >
                Generate Secure Token
              </Button>
              
              <Text style={styles.helperText}>
                Copy this token to your laptop's Nexus Terminal configuration
              </Text>
            </Card.Content>
          </Card>
        )}

        {/* Password Authentication */}
        {authConfig.authMethod === 'password' && (
          <Card style={styles.card}>
            <Card.Title 
              title="Password Authentication" 
              left={() => <Icon name="password" size={24} color="#FF9800" />}
            />
            <Card.Content>
              <TextInput
                mode="outlined"
                label="Username"
                value={authConfig.username || ''}
                onChangeText={(text) => setAuthConfig(prev => ({ ...prev, username: text }))}
                style={styles.input}
              />
              
              <TextInput
                mode="outlined"
                label="Password"
                value={authConfig.password || ''}
                onChangeText={(text) => setAuthConfig(prev => ({ ...prev, password: text }))}
                secureTextEntry={!showPassword}
                right={
                  <TextInput.Icon
                    icon={showPassword ? 'eye-off' : 'eye'}
                    onPress={() => setShowPassword(!showPassword)}
                  />
                }
                style={styles.input}
              />
            </Card.Content>
          </Card>
        )}

        {/* Certificate Authentication */}
        {authConfig.authMethod === 'certificate' && (
          <Card style={styles.card}>
            <Card.Title 
              title="Certificate Authentication" 
              left={() => <Icon name="certificate" size={24} color="#9C27B0" />}
            />
            <Card.Content>
              <Button
                mode="outlined"
                onPress={() => setShowCertDialog(true)}
                style={styles.button}
              >
                Configure Certificate
              </Button>
              
              {authConfig.certificate && (
                <Text style={styles.certificateInfo}>
                  Certificate configured ✓
                </Text>
              )}
            </Card.Content>
          </Card>
        )}

        {/* SSL/Security Settings */}
        <Card style={styles.card}>
          <Card.Title 
            title="Security Settings" 
            left={() => <Icon name="https" size={24} color="#F44336" />}
          />
          <Card.Content>
            <List.Item
              title="Enable SSL/TLS"
              description="Use encrypted connection"
              left={() => <Icon name="lock" size={24} color="#4CAF50" />}
              right={() => (
                <Switch
                  value={authConfig.enableSSL}
                  onValueChange={(value) => setAuthConfig(prev => ({ ...prev, enableSSL: value }))}
                />
              )}
            />
            
            <List.Item
              title="Verify Hostname"
              description="Verify server certificate hostname"
              left={() => <Icon name="verified" size={24} color="#2196F3" />}
              right={() => (
                <Switch
                  value={authConfig.verifyHostname}
                  onValueChange={(value) => setAuthConfig(prev => ({ ...prev, verifyHostname: value }))}
                  disabled={!authConfig.enableSSL}
                />
              )}
            />
            
            <List.Item
              title="Allow Self-Signed"
              description="Accept self-signed certificates"
              left={() => <Icon name="warning" size={24} color="#FF9800" />}
              right={() => (
                <Switch
                  value={authConfig.allowSelfSigned}
                  onValueChange={(value) => setAuthConfig(prev => ({ ...prev, allowSelfSigned: value }))}
                  disabled={!authConfig.enableSSL}
                />
              )}
            />
          </Card.Content>
        </Card>

        {/* Saved Configurations */}
        {savedConfigs.length > 0 && (
          <Card style={styles.card}>
            <Card.Title 
              title="Saved Configurations" 
              left={() => <Icon name="bookmark" size={24} color="#00BCD4" />}
            />
            <Card.Content>
              {savedConfigs.map((item, index) => (
                <List.Item
                  key={index}
                  title={item.name}
                  description={`${item.config.authMethod} • ${new Date(item.timestamp).toLocaleDateString()}`}
                  left={() => <Icon name="folder" size={24} color="#607D8B" />}
                  right={() => (
                    <View style={styles.configActions}>
                      <Button
                        mode="text"
                        compact
                        onPress={() => loadSavedConfig(item)}
                      >
                        Load
                      </Button>
                      <Button
                        mode="text"
                        compact
                        textColor="#F44336"
                        onPress={() => deleteSavedConfig(item.name)}
                      >
                        Delete
                      </Button>
                    </View>
                  )}
                />
              ))}
            </Card.Content>
          </Card>
        )}

        {/* Actions */}
        <Card style={styles.card}>
          <Card.Content>
            <Button
              mode="contained"
              onPress={validateAndSave}
              style={styles.button}
            >
              Save Configuration
            </Button>
            
            <Button
              mode="outlined"
              onPress={testAuthentication}
              style={styles.button}
            >
              Test Authentication
            </Button>
            
            <Button
              mode="outlined"
              onPress={() => {
                Alert.prompt(
                  'Save Configuration',
                  'Enter a name for this configuration:',
                  [
                    { text: 'Cancel', style: 'cancel' },
                    { text: 'Save', onPress: (name) => name && saveConfigWithName(name) }
                  ],
                  'plain-text'
                );
              }}
              style={styles.button}
            >
              Save as Template
            </Button>
          </Card.Content>
        </Card>
      </ScrollView>

      {/* Certificate Dialog */}
      <Portal>
        <Dialog visible={showCertDialog} onDismiss={() => setShowCertDialog(false)}>
          <Dialog.Title>Certificate Configuration</Dialog.Title>
          <Dialog.Content>
            <TextInput
              mode="outlined"
              label="Certificate (PEM format)"
              value={authConfig.certificate || ''}
              onChangeText={(text) => setAuthConfig(prev => ({ ...prev, certificate: text }))}
              multiline
              numberOfLines={6}
              style={styles.input}
            />
            
            <TextInput
              mode="outlined"
              label="Private Key (PEM format)"
              value={authConfig.privateKey || ''}
              onChangeText={(text) => setAuthConfig(prev => ({ ...prev, privateKey: text }))}
              multiline
              numberOfLines={6}
              style={styles.input}
            />
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setShowCertDialog(false)}>Cancel</Button>
            <Button onPress={() => setShowCertDialog(false)}>Save</Button>
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
  input: {
    marginBottom: 16,
  },
  button: {
    marginBottom: 8,
  },
  chipContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  chip: {
    marginRight: 8,
    marginBottom: 8,
  },
  helperText: {
    fontSize: 12,
    color: '#888',
    textAlign: 'center',
    marginTop: 8,
  },
  certificateInfo: {
    fontSize: 14,
    color: '#4CAF50',
    textAlign: 'center',
    marginTop: 16,
  },
  configActions: {
    flexDirection: 'row',
  },
});
