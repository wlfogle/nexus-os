import { useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { invoke } from '@tauri-apps/api/core';
import { selectActiveTab, addAIMessage, updateTabTerminalId } from '../store/slices/terminalTabSlice';
import { commandRoutingService } from '../services/commandRouting';
import { routingLogger } from '../utils/logger';
import { SHELL_CONFIGS, ShellType } from '../types/terminal';

export const useInputRouting = () => {
  const dispatch = useDispatch();
  const activeTab = useSelector(selectActiveTab);
  
  const ensureTerminalId = useCallback(async (): Promise<string | null> => {
    if (!activeTab) return null;
    if (activeTab.terminalId) return activeTab.terminalId;

    try {
      const shellConfig = SHELL_CONFIGS[activeTab.shell as ShellType] ?? SHELL_CONFIGS[ShellType.BASH];
      const createdTerminalId = await invoke<string>('create_terminal', {
        shell: shellConfig.executable,
        args: shellConfig.args,
        cwd: activeTab.workingDirectory === '~' ? null : activeTab.workingDirectory,
        env: activeTab.environmentVars
      });

      dispatch(updateTabTerminalId({ tabId: activeTab.id, terminalId: createdTerminalId }));
      routingLogger.info('Created missing terminal session on demand', 'create_terminal_on_demand', { tabId: activeTab.id, terminalId: createdTerminalId });

      return createdTerminalId;
    } catch (error) {
      routingLogger.error('Failed to create terminal session on demand', error as Error, 'create_terminal_on_demand_failed', { tabId: activeTab.id });

      return null;
    }
  }, [activeTab, dispatch]);

  // Use unified command routing service for smart command detection
  const isShellCommand = useCallback((input: string): boolean => {
    const result = commandRoutingService.isShellCommand(input);

    routingLogger.routeDecision(input, result, 1.0, 'Simple shell command check');

    return result;
  }, []);

  // Handle input routing between AI and shell with enhanced analysis
  const handleInput = useCallback(async (input: string, onAIResponse?: (message: string) => void) => {
    const trimmed = input.trim();

    if (!trimmed || !activeTab) return;
    
    routingLogger.info(`Processing input routing`, 'handle_input', { input: trimmed });
    
    try {
      // Get detailed routing analysis for better decision making
      const routingResult = await commandRoutingService.routeCommand(trimmed);
      
      routingLogger.routeAnalysis(trimmed, routingResult.confidence, routingResult.reason);
      
      if (routingResult.isShellCommand) {
        // Execute as shell command
        const terminalId = await ensureTerminalId();

        if (terminalId) {
          try {
            routingLogger.info(`Executing shell command`, 'shell_execute', { command: trimmed, terminalId });
            await invoke('write_to_terminal', { 
              terminal_id: terminalId, 
              data: `${trimmed}\r` 
            });

            routingLogger.shellExecution(trimmed, true);
            
            // Provide feedback on low confidence routing
            if (routingResult.confidence < 0.8) {
              routingLogger.warn(`Low confidence shell routing`, undefined, 'low_confidence_shell', {
                confidence: routingResult.confidence,
                suggestion: `If this wasn't a shell command, try asking: "help me with ${trimmed}"`
              });
            }
          } catch (error) {
            const errorMessageText = error instanceof Error ? error.message : String(error);

            if (errorMessageText.toLowerCase().includes('not found')) {
              const recreatedTerminalId = await ensureTerminalId();

              if (recreatedTerminalId) {
                try {
                  await invoke('write_to_terminal', {
                    terminal_id: recreatedTerminalId,
                    data: `${trimmed}\r`
                  });
                  routingLogger.shellExecution(trimmed, true);

                  return;
                } catch (retryError) {
                  routingLogger.shellExecution(trimmed, false, retryError as Error);
                }
              }
            } else {
              routingLogger.shellExecution(trimmed, false, error as Error);
            }
            
            // On shell execution error, offer AI assistance
            if (onAIResponse) {
              const errorMessage = `I had trouble executing "${trimmed}". Let me help you troubleshoot this command.`;

              onAIResponse(errorMessage);
              
              // Also add the AI message to store
              dispatch(addAIMessage({
                tabId: activeTab.id,
                message: {
                  role: 'assistant',
                  content: errorMessage,
                  timestamp: new Date(),
                  metadata: { error_recovery: true, failed_command: trimmed }
                }
              }));
            }
          }
        } else {
          routingLogger.error('No terminal ID available for shell command execution', undefined, 'no_terminal_id', { command: trimmed });
        }

        return; // Important: return early for shell commands
      } else {
        // Send to AI assistant
        routingLogger.aiRequest(trimmed);
        
        // Add user message immediately for AI queries only
        dispatch(addAIMessage({
          tabId: activeTab.id,
          message: {
            role: 'user',
            content: trimmed,
            timestamp: new Date()
          }
        }));
        
        // Provide feedback on low confidence routing
        if (routingResult.confidence < 0.8) {
          routingLogger.warn(`Low confidence AI routing`, undefined, 'low_confidence_ai', {
            confidence: routingResult.confidence,
            suggestion: `If you meant to run a command, try: "${trimmed}" directly`
          });
        }
        
        try {
          // Start AI request with enhanced context
          const startTime = Date.now();
          const timer = routingLogger.performanceTimer('AI Request');
          
          // Enhanced prompt with routing context
          const enhancedPrompt = `User Query: ${trimmed}\n\nContext: Terminal session in ${activeTab.workingDirectory} using ${activeTab.shell}\nRouting confidence: ${(routingResult.confidence * 100).toFixed(1)}%\nReason: ${routingResult.reason}`;
          
          routingLogger.info('Sending AI request', 'ai_request', { query: trimmed, contextLength: enhancedPrompt.length });
          
          // Send to AI with enhanced context
          const aiResponse = await invoke('ai_chat_with_memory', {
            message: trimmed,
            conversation_id: activeTab.id,
            context: enhancedPrompt
          }) as string;
          
          const responseTime = Date.now() - startTime;

          timer.end({ component: 'AIRequest', metadata: { query: trimmed, responseLength: aiResponse.length } });
          
          // Add AI response
          dispatch(addAIMessage({
            tabId: activeTab.id,
            message: {
              role: 'assistant',
              content: aiResponse,
              timestamp: new Date(),
              metadata: {
                response_time_ms: responseTime,
                context_type: 'enhanced',
                routing_confidence: routingResult.confidence
              }
            }
          }));
          
          // Call optional callback with AI response
          if (onAIResponse) {
            onAIResponse(aiResponse);
          }
          
        } catch (error) {
          routingLogger.error('AI request failed', error as Error, 'ai_request_failed', { query: trimmed });
          
          const errorMessage = `❌ Sorry, I encountered an error: ${error instanceof Error ? error.message : 'Unknown error'}`;
          
          dispatch(addAIMessage({
            tabId: activeTab.id,
            message: {
              role: 'assistant',
              content: errorMessage,
              timestamp: new Date(),
              metadata: { error: true }
            }
          }));
          
          if (onAIResponse) {
            onAIResponse(errorMessage);
          }
        }
      }
    } catch (routingError) {
      routingLogger.error('Command routing failed, using fallback', routingError as Error, 'routing_fallback', { input: trimmed });
      
      // Fallback to simple heuristic when routing service fails
      if (isShellCommand(trimmed)) {
        const terminalId = await ensureTerminalId();

        if (terminalId) {
          try {
            routingLogger.info('Executing fallback shell command', 'fallback_shell', { command: trimmed });
            await invoke('write_to_terminal', { 
              terminal_id: terminalId, 
              data: `${trimmed}\r` 
            });
          } catch (error) {
            routingLogger.error('Fallback shell execution failed', error as Error, 'fallback_failed', { command: trimmed });
          }
        }
      } else {
        // Fallback AI handling
        dispatch(addAIMessage({
          tabId: activeTab.id,
          message: {
            role: 'user',
            content: trimmed,
            timestamp: new Date()
          }
        }));
        
        if (onAIResponse) {
          onAIResponse(`Processing your request: "${trimmed}"`);
        }
      }
    }
  }, [activeTab, isShellCommand, dispatch, ensureTerminalId]);

  return {
    handleInput,
    isShellCommand
  };
};
