/*
 * NexSh - The AI-Powered Shell for NexusOS
 * Revolutionary shell with natural language processing and universal package integration
 * 
 * Features:
 * - Natural language command interpretation ("install firefox" -> detects best package format)
 * - Intelligent autocomplete with context awareness and learning
 * - Universal package management integration (any Linux distro packages)
 * - AI-powered command suggestions and error correction
 * - Smart history with semantic search
 * - Real-time system monitoring integration
 * - Security-aware command execution with sandboxing
 * - Cross-shell compatibility (bash/zsh/fish command support)
 */

#include "nexsh.h"
#include "ai/natural-language-processor.h"
#include "ai/command-predictor.h"
#include "ai/smart-completion.h"
#include "packages/universal-package-manager.h"
#include "security/command-security.h"
#include "system/performance-monitor.h"
#include "history/semantic-history.h"

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <memory>
#include <thread>
#include <chrono>
#include <regex>
#include <fstream>

#include <readline/readline.h>
#include <readline/history.h>
#include <unistd.h>
#include <sys/wait.h>
#include <signal.h>

NexSh::NexSh(int argc, char* argv[])
    : m_running(true)
    , m_aiEnabled(true)
    , m_universalPackagesEnabled(true)
    , m_securityEnabled(true)
    , m_currentPath(getcwd(nullptr, 0))
    , m_exitCode(0)
{
    initializeShell();
    parseArguments(argc, argv);
    setupAI();
    setupReadline();
    setupSignalHandlers();
    
    // Welcome message
    displayWelcome();
}

void NexSh::initializeShell()
{
    // Initialize core components
    m_nlProcessor = std::make_unique<NaturalLanguageProcessor>();
    m_commandPredictor = std::make_unique<CommandPredictor>();
    m_smartCompletion = std::make_unique<SmartCompletion>();
    m_packageManager = std::make_unique<UniversalPackageManager>();
    m_security = std::make_unique<CommandSecurity>();
    m_perfMonitor = std::make_unique<PerformanceMonitor>();
    m_history = std::make_unique<SemanticHistory>();
    
    // Load configuration
    loadConfiguration();
    
    // Initialize AI models
    if (m_aiEnabled) {
        m_nlProcessor->initialize();
        m_commandPredictor->loadModels();
        m_smartCompletion->trainOnHistory();
    }
    
    std::cout << "NexSh initialized with AI capabilities" << std::endl;
}

void NexSh::setupAI()
{
    if (!m_aiEnabled) return;
    
    // Configure natural language processing
    m_nlProcessor->enableContextAwareness(true);
    m_nlProcessor->enableLearning(true);
    m_nlProcessor->loadLanguageModels();
    
    // Configure command prediction
    m_commandPredictor->enablePredictiveCompletion(true);
    m_commandPredictor->enableErrorCorrection(true);
    m_commandPredictor->setConfidenceThreshold(0.7f);
    
    // Configure smart completion
    m_smartCompletion->enableSemanticCompletion(true);
    m_smartCompletion->enableContextualSuggestions(true);
    m_smartCompletion->enablePackageAwareCompletion(true);
    
    std::cout << "AI subsystems initialized" << std::endl;
}

void NexSh::setupReadline()
{
    // Configure readline for advanced completion and history
    rl_attempted_completion_function = nexsh_completion;
    rl_completion_entry_function = nexsh_generator;
    rl_bind_key('\t', rl_complete);
    
    // Custom key bindings for AI features
    rl_bind_key_in_map('\033', ai_suggest_command, emacs_standard_keymap);  // Alt key for AI suggestions
    rl_bind_key_in_map('\020', show_predictions, emacs_standard_keymap);   // Ctrl+P for predictions
    rl_bind_key_in_map('\023', semantic_search, emacs_standard_keymap);    // Ctrl+S for semantic search
    
    // Load command history
    std::string historyFile = std::string(getenv("HOME")) + "/.nexsh_history";
    read_history(historyFile.c_str());
    
    std::cout << "Readline configured with AI enhancements" << std::endl;
}

void NexSh::run()
{
    std::string input;
    char* line;
    
    while (m_running) {
        // Generate intelligent prompt
        std::string prompt = generatePrompt();
        
        // Get user input with AI assistance
        line = readline(prompt.c_str());
        
        if (!line) {
            // EOF (Ctrl+D)
            std::cout << std::endl;
            break;
        }
        
        input = std::string(line);
        free(line);
        
        // Skip empty lines
        if (input.empty()) {
            continue;
        }
        
        // Add to history
        add_history(input.c_str());
        m_history->addCommand(input, m_currentPath);
        
        // Process command with AI enhancement
        processCommand(input);
    }
    
    cleanup();
}

void NexSh::processCommand(const std::string& input)
{
    try {
        // Check for built-in commands first
        if (handleBuiltinCommand(input)) {
            return;
        }
        
        // Natural language processing for human-like commands
        if (m_aiEnabled && isNaturalLanguageCommand(input)) {
            std::string translatedCommand = m_nlProcessor->translateToCommand(input);
            if (!translatedCommand.empty()) {
                std::cout << "🤖 AI: Translating '" << input << "' to: " << translatedCommand << std::endl;
                executeCommand(translatedCommand);
                return;
            }
        }
        
        // Regular command execution
        executeCommand(input);
        
    } catch (const std::exception& e) {
        std::cerr << "nexsh: error: " << e.what() << std::endl;
        m_exitCode = 1;
    }
}

bool NexSh::handleBuiltinCommand(const std::string& input)
{
    std::vector<std::string> tokens = tokenize(input);
    if (tokens.empty()) return false;
    
    const std::string& command = tokens[0];
    
    if (command == "exit" || command == "quit") {
        m_running = false;
        return true;
    }
    
    if (command == "cd") {
        return handleCd(tokens);
    }
    
    if (command == "ai") {
        return handleAICommand(tokens);
    }
    
    if (command == "nexus-install" || command == "nxi") {
        return handleUniversalInstall(tokens);
    }
    
    if (command == "nexus-search" || command == "nxs") {
        return handleUniversalSearch(tokens);
    }
    
    if (command == "history-search" || command == "hs") {
        return handleSemanticHistorySearch(tokens);
    }
    
    if (command == "predict" || command == "p") {
        return handleCommandPrediction(tokens);
    }
    
    if (command == "explain") {
        return handleCommandExplanation(tokens);
    }
    
    if (command == "optimize") {
        return handleSystemOptimization(tokens);
    }
    
    if (command == "security") {
        return handleSecurityCommand(tokens);
    }
    
    return false;
}

bool NexSh::handleCd(const std::vector<std::string>& tokens)
{
    std::string path;
    
    if (tokens.size() == 1) {
        path = getenv("HOME");
    } else {
        path = tokens[1];
    }
    
    if (chdir(path.c_str()) == 0) {
        m_currentPath = getcwd(nullptr, 0);
        // Update AI context with new directory
        if (m_aiEnabled) {
            m_nlProcessor->updateContext("current_directory", m_currentPath);
            m_commandPredictor->updateWorkingDirectory(m_currentPath);
        }
        return true;
    } else {
        perror("nexsh: cd");
        return true;
    }
}

bool NexSh::handleAICommand(const std::vector<std::string>& tokens)
{
    if (tokens.size() < 2) {
        std::cout << "AI Assistant Commands:\n";
        std::cout << "  ai on/off          - Enable/disable AI features\n";
        std::cout << "  ai explain <cmd>   - Explain what a command does\n";
        std::cout << "  ai suggest         - Get command suggestions for current context\n";
        std::cout << "  ai learn           - Trigger learning from recent commands\n";
        std::cout << "  ai status          - Show AI system status\n";
        return true;
    }
    
    const std::string& subcommand = tokens[1];
    
    if (subcommand == "on") {
        m_aiEnabled = true;
        std::cout << "🤖 AI features enabled" << std::endl;
    } else if (subcommand == "off") {
        m_aiEnabled = false;
        std::cout << "🤖 AI features disabled" << std::endl;
    } else if (subcommand == "explain" && tokens.size() > 2) {
        std::string command = joinTokens(tokens, 2);
        explainCommand(command);
    } else if (subcommand == "suggest") {
        showAISuggestions();
    } else if (subcommand == "learn") {
        triggerAILearning();
    } else if (subcommand == "status") {
        showAIStatus();
    }
    
    return true;
}

bool NexSh::handleUniversalInstall(const std::vector<std::string>& tokens)
{
    if (tokens.size() < 2) {
        std::cout << "Usage: nexus-install <package-name> [options]\n";
        std::cout << "Installs packages from any Linux distribution format\n";
        std::cout << "Examples:\n";
        std::cout << "  nexus-install firefox         # Automatically detects best source\n";
        std::cout << "  nexus-install --flatpak gimp  # Force Flatpak\n";
        std::cout << "  nexus-install --deb package   # Use Debian package\n";
        return true;
    }
    
    std::string packageName = tokens[1];
    PackageFormat preferredFormat = PackageFormat::AUTO;
    
    // Parse format preferences
    for (size_t i = 2; i < tokens.size(); i++) {
        if (tokens[i] == "--flatpak") preferredFormat = PackageFormat::FLATPAK;
        else if (tokens[i] == "--snap") preferredFormat = PackageFormat::SNAP;
        else if (tokens[i] == "--appimage") preferredFormat = PackageFormat::APPIMAGE;
        else if (tokens[i] == "--deb") preferredFormat = PackageFormat::DEB;
        else if (tokens[i] == "--rpm") preferredFormat = PackageFormat::RPM;
        else if (tokens[i] == "--arch") preferredFormat = PackageFormat::ARCH;
    }
    
    return m_packageManager->installPackage(packageName, preferredFormat);
}

bool NexSh::handleUniversalSearch(const std::vector<std::string>& tokens)
{
    if (tokens.size() < 2) {
        std::cout << "Usage: nexus-search <search-term>\n";
        std::cout << "Searches packages across all Linux distribution repositories\n";
        return true;
    }
    
    std::string searchTerm = joinTokens(tokens, 1);
    auto results = m_packageManager->searchPackages(searchTerm);
    
    displaySearchResults(results);
    return true;
}

bool NexSh::handleSemanticHistorySearch(const std::vector<std::string>& tokens)
{
    if (tokens.size() < 2) {
        std::cout << "Usage: history-search <semantic-query>\n";
        std::cout << "Examples:\n";
        std::cout << "  hs install firefox        # Find commands that installed Firefox\n";
        std::cout << "  hs network configuration  # Find network-related commands\n";
        return true;
    }
    
    std::string query = joinTokens(tokens, 1);
    auto results = m_history->semanticSearch(query);
    
    displayHistoryResults(results);
    return true;
}

bool NexSh::handleCommandPrediction(const std::vector<std::string>& tokens)
{
    auto predictions = m_commandPredictor->getPredictions(m_currentPath);
    
    std::cout << "🔮 Predicted commands based on current context:\n";
    for (size_t i = 0; i < predictions.size() && i < 5; i++) {
        std::cout << "  " << (i + 1) << ". " << predictions[i].command 
                  << " (confidence: " << predictions[i].confidence << ")\n";
    }
    
    return true;
}

bool NexSh::handleCommandExplanation(const std::vector<std::string>& tokens)
{
    if (tokens.size() < 2) {
        std::cout << "Usage: explain <command>\n";
        return true;
    }
    
    std::string command = joinTokens(tokens, 1);
    explainCommand(command);
    return true;
}

void NexSh::executeCommand(const std::string& command)
{
    // Security check
    if (m_securityEnabled) {
        SecurityAssessment assessment = m_security->assessCommand(command);
        if (assessment.riskLevel == SecurityRisk::HIGH) {
            std::cout << "⚠️  High-risk command detected: " << assessment.reason << std::endl;
            std::cout << "Execute anyway? (y/N): ";
            std::string response;
            std::getline(std::cin, response);
            if (response != "y" && response != "Y") {
                return;
            }
        }
    }
    
    // Execute command
    pid_t pid = fork();
    
    if (pid == 0) {
        // Child process
        execl("/bin/sh", "sh", "-c", command.c_str(), nullptr);
        perror("nexsh: exec failed");
        exit(1);
    } else if (pid > 0) {
        // Parent process
        int status;
        waitpid(pid, &status, 0);
        m_exitCode = WEXITSTATUS(status);
        
        // Update AI with command result
        if (m_aiEnabled) {
            m_commandPredictor->recordCommand(command, m_exitCode == 0);
            m_nlProcessor->updateContext("last_command_success", m_exitCode == 0);
        }
    } else {
        perror("nexsh: fork failed");
        m_exitCode = 1;
    }
}

std::string NexSh::generatePrompt()
{
    std::string prompt;
    
    // Add AI status indicator
    if (m_aiEnabled) {
        prompt += "🤖 ";
    }
    
    // Add current user and hostname
    char* user = getenv("USER");
    char hostname[256];
    gethostname(hostname, sizeof(hostname));
    
    prompt += std::string(user) + "@" + hostname;
    
    // Add current directory (shortened)
    std::string shortPath = shortenPath(m_currentPath);
    prompt += ":" + shortPath;
    
    // Add git branch if in a git repository
    std::string gitBranch = getGitBranch();
    if (!gitBranch.empty()) {
        prompt += " (" + gitBranch + ")";
    }
    
    // Add performance indicator if enabled
    if (m_perfMonitor->isEnabled()) {
        auto metrics = m_perfMonitor->getCurrentMetrics();
        if (metrics.cpuUsage > 80) prompt += " 🔥";
        if (metrics.memoryUsage > 90) prompt += " 💾";
    }
    
    // Add exit code indicator for last command
    if (m_exitCode != 0) {
        prompt += " ❌" + std::to_string(m_exitCode);
    }
    
    prompt += "$ ";
    
    return prompt;
}

void NexSh::displayWelcome()
{
    std::cout << R"(
    ╔═══════════════════════════════════════════════════════════════╗
    ║                        🚀 NexSh v1.0                         ║
    ║                  AI-Powered Shell for NexusOS                 ║
    ╚═══════════════════════════════════════════════════════════════╝
    
    Features enabled:
    )" << std::endl;
    
    if (m_aiEnabled) std::cout << "    ✅ AI Command Assistant\n";
    if (m_universalPackagesEnabled) std::cout << "    ✅ Universal Package Management\n";
    if (m_securityEnabled) std::cout << "    ✅ Security Monitoring\n";
    
    std::cout << R"(
    Quick Start:
    • Type naturally: "install firefox", "show network status"
    • Use 'ai help' for AI assistant commands
    • Use 'nxi <package>' for universal package installation
    • Use 'hs <query>' for semantic history search
    • Press Alt for AI command suggestions
    
    Happy computing! 🎉
    )" << std::endl;
}

bool NexSh::isNaturalLanguageCommand(const std::string& input)
{
    // Check for natural language patterns
    std::regex patterns[] = {
        std::regex(R"(^(please\s+)?(install|get|download)\s+.+)", std::regex::icase),
        std::regex(R"(^(show|display|list)\s+.+)", std::regex::icase),
        std::regex(R"(^(find|search|locate)\s+.+)", std::regex::icase),
        std::regex(R"(^(how\s+)?(do\s+)?i\s+.+)", std::regex::icase),
        std::regex(R"(^(what|where|when|why|who)\s+.+)", std::regex::icase),
        std::regex(R"(^(check|monitor|watch)\s+.+)", std::regex::icase)
    };
    
    for (const auto& pattern : patterns) {
        if (std::regex_match(input, pattern)) {
            return true;
        }
    }
    
    return false;
}

// Global callback functions for readline
char** nexsh_completion(const char* text, int start, int end)
{
    return rl_completion_matches(text, nexsh_generator);
}

char* nexsh_generator(const char* text, int state)
{
    // AI-powered completion generation would be implemented here
    // For now, return standard completion
    return nullptr;
}

int ai_suggest_command(int count, int key)
{
    // Show AI command suggestions
    return 0;
}

int show_predictions(int count, int key)
{
    // Show command predictions
    return 0;
}

int semantic_search(int count, int key)
{
    // Trigger semantic history search
    return 0;
}

int main(int argc, char* argv[])
{
    try {
        NexSh shell(argc, argv);
        shell.run();
        return shell.getExitCode();
    } catch (const std::exception& e) {
        std::cerr << "nexsh: fatal error: " << e.what() << std::endl;
        return 1;
    }
}