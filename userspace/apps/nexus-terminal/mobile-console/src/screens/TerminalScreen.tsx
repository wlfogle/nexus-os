import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  StyleSheet,
  ScrollView,
  TextInput,
  Alert,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';
import { Text, Card, Button, FAB, Chip } from 'react-native-paper';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { ConnectionManager, CommandRequest, CommandResponse } from '@services/ConnectionManager';

interface CommandHistoryItem {
  id: string;
  command: string;
  output: string;
  success: boolean;
  timestamp: Date;
  executionTime: number;
}

const COMMON_COMMANDS = [
  'ls -la', 'pwd', 'ps aux', 'df -h', 'free -h', 
  'top', 'htop', 'netstat -tuln', 'git status', 'cargo build'
];

export const TerminalScreen: React.FC = () => {
  const [command, setCommand] = useState('');
  const [history, setHistory] = useState<CommandHistoryItem[]>([]);
  const [currentDirectory, setCurrentDirectory] = useState('~');
  const [isExecuting, setIsExecuting] = useState(false);
  const [sessionId] = useState(() => `session_${Date.now()}`);
  
  const scrollRef = useRef<ScrollView>(null);
  const inputRef = useRef<TextInput>(null);

  useEffect(() => {
    // Setup WebSocket terminal session
    ConnectionManager.startTerminalSession(sessionId);
    
    const handleCommandOutput = (data: any) => {
      if (data.sessionId === sessionId) {
        // Real-time terminal output would be handled here
        console.log('Terminal output:', data);
      }
    };

    ConnectionManager.on('commandOutput', handleCommandOutput);
    
    return () => {
      ConnectionManager.endTerminalSession(sessionId);
      ConnectionManager.off('commandOutput', handleCommandOutput);
    };
  }, [sessionId]);

  const executeCommand = async () => {
    if (!command.trim()) return;

    const commandToExecute = command.trim();
    setIsExecuting(true);
    
    const startTime = Date.now();
    
    try {
      const request: CommandRequest = {
        command: commandToExecute,
        cwd: currentDirectory === '~' ? undefined : currentDirectory,
      };

      const response: CommandResponse = await ConnectionManager.executeCommand(request);
      
      const historyItem: CommandHistoryItem = {
        id: `cmd_${Date.now()}`,
        command: commandToExecute,
        output: response.output,
        success: response.success,
        timestamp: new Date(),
        executionTime: response.executionTime,
      };

      setHistory(prev => [...prev, historyItem]);
      setCommand('');
      
      // Update current directory if it's a cd command
      if (commandToExecute.startsWith('cd ')) {
        const newDir = commandToExecute.substring(3).trim();
        if (newDir && response.success) {
          setCurrentDirectory(newDir === '~' ? '~' : newDir);
        }
      }
      
      // Auto-scroll to bottom
      setTimeout(() => {
        scrollRef.current?.scrollToEnd({ animated: true });
      }, 100);
      
    } catch (error) {
      const historyItem: CommandHistoryItem = {
        id: `cmd_${Date.now()}`,
        command: commandToExecute,
        output: error instanceof Error ? error.message : 'Command execution failed',
        success: false,
        timestamp: new Date(),
        executionTime: Date.now() - startTime,
      };
      
      setHistory(prev => [...prev, historyItem]);
      Alert.alert('Error', 'Failed to execute command');
    } finally {
      setIsExecuting(false);
    }
  };

  const insertCommand = (cmd: string) => {
    setCommand(cmd);
    inputRef.current?.focus();
  };

  const clearHistory = () => {
    Alert.alert(
      'Clear History',
      'Are you sure you want to clear the command history?',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Clear', onPress: () => setHistory([]) },
      ]
    );
  };

  const formatExecutionTime = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  return (
    <KeyboardAvoidingView 
      style={styles.container} 
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      {/* Command suggestions */}
      <ScrollView 
        horizontal 
        style={styles.suggestionsContainer}
        showsHorizontalScrollIndicator={false}
      >
        {COMMON_COMMANDS.map((cmd, index) => (
          <Chip
            key={index}
            onPress={() => insertCommand(cmd)}
            style={styles.suggestionChip}
            textStyle={styles.suggestionText}
          >
            {cmd}
          </Chip>
        ))}
      </ScrollView>

      {/* Terminal output */}
      <ScrollView 
        ref={scrollRef}
        style={styles.terminal}
        contentContainerStyle={styles.terminalContent}
      >
        {history.map((item) => (
          <View key={item.id} style={styles.commandBlock}>
            <View style={styles.commandHeader}>
              <Text style={styles.promptText}>
                $ {currentDirectory} {item.command}
              </Text>
              <View style={styles.commandMeta}>
                <Text style={styles.timeText}>
                  {formatExecutionTime(item.executionTime)}
                </Text>
                <Icon 
                  name={item.success ? 'check-circle' : 'error'} 
                  size={16} 
                  color={item.success ? '#4CAF50' : '#F44336'} 
                />
              </View>
            </View>
            
            {item.output && (
              <Text style={[
                styles.outputText, 
                { color: item.success ? '#ccc' : '#F44336' }
              ]}>
                {item.output}
              </Text>
            )}
          </View>
        ))}
        
        {history.length === 0 && (
          <View style={styles.emptyState}>
            <Icon name="terminal" size={48} color="#666" />
            <Text style={styles.emptyText}>
              Welcome to Nexus Terminal Remote Console
            </Text>
            <Text style={styles.emptySubtext}>
              Execute commands on your laptop from here
            </Text>
          </View>
        )}
      </ScrollView>

      {/* Command input */}
      <View style={styles.inputContainer}>
        <View style={styles.promptContainer}>
          <Text style={styles.promptLabel}>$ {currentDirectory}</Text>
        </View>
        
        <View style={styles.inputRow}>
          <TextInput
            ref={inputRef}
            style={styles.commandInput}
            value={command}
            onChangeText={setCommand}
            placeholder="Enter command..."
            placeholderTextColor="#666"
            autoCapitalize="none"
            autoCorrect={false}
            multiline={false}
            returnKeyType="send"
            onSubmitEditing={executeCommand}
            editable={!isExecuting}
          />
          
          <Button
            mode="contained"
            onPress={executeCommand}
            disabled={!command.trim() || isExecuting}
            loading={isExecuting}
            style={styles.executeButton}
            compact
          >
            Execute
          </Button>
        </View>
      </View>

      <FAB
        style={styles.fab}
        icon="delete-sweep"
        onPress={clearHistory}
        disabled={history.length === 0}
        size="small"
      />
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#121212',
  },
  suggestionsContainer: {
    maxHeight: 50,
    backgroundColor: '#1a1a1a',
    paddingHorizontal: 8,
    paddingVertical: 8,
  },
  suggestionChip: {
    marginRight: 8,
    backgroundColor: '#333',
  },
  suggestionText: {
    color: '#ccc',
    fontSize: 12,
  },
  terminal: {
    flex: 1,
    backgroundColor: '#0d1117',
    paddingHorizontal: 12,
  },
  terminalContent: {
    paddingVertical: 8,
  },
  commandBlock: {
    marginBottom: 16,
  },
  commandHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  promptText: {
    color: '#4CAF50',
    fontSize: 14,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
    flex: 1,
  },
  commandMeta: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  timeText: {
    color: '#666',
    fontSize: 12,
    marginRight: 4,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  outputText: {
    fontSize: 13,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
    lineHeight: 18,
    paddingLeft: 8,
    borderLeftWidth: 2,
    borderLeftColor: '#333',
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 60,
  },
  emptyText: {
    fontSize: 18,
    color: '#666',
    marginTop: 16,
    textAlign: 'center',
  },
  emptySubtext: {
    fontSize: 14,
    color: '#555',
    marginTop: 8,
    textAlign: 'center',
  },
  inputContainer: {
    backgroundColor: '#1a1a1a',
    borderTopWidth: 1,
    borderTopColor: '#333',
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  promptContainer: {
    marginBottom: 8,
  },
  promptLabel: {
    color: '#4CAF50',
    fontSize: 14,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
  },
  inputRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  commandInput: {
    flex: 1,
    backgroundColor: '#0d1117',
    color: '#ccc',
    fontSize: 14,
    fontFamily: Platform.OS === 'ios' ? 'Menlo' : 'monospace',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: '#333',
    marginRight: 8,
  },
  executeButton: {
    minWidth: 80,
  },
  fab: {
    position: 'absolute',
    margin: 16,
    right: 0,
    bottom: 100,
    backgroundColor: '#F44336',
  },
});
