#ifndef NEXSH_H
#define NEXSH_H

#include <string>
#include <vector>
#include <memory>
#include <map>

// Forward declarations
class NaturalLanguageProcessor;
class CommandPredictor;
class SmartCompletion;
class UniversalPackageManager;
class CommandSecurity;
class PerformanceMonitor;
class SemanticHistory;

// Enums for package formats and security levels
enum class PackageFormat {
    AUTO,
    FLATPAK,
    SNAP,
    APPIMAGE,
    DEB,
    RPM,
    ARCH,
    NIXPKGS,
    GUIX
};

enum class SecurityRisk {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL
};

// Data structures for AI predictions and system metrics
struct CommandPrediction {
    std::string command;
    float confidence;
    std::string reasoning;
};

struct SecurityAssessment {
    SecurityRisk riskLevel;
    std::string reason;
    std::vector<std::string> suggestions;
};

struct SystemMetrics {
    float cpuUsage;
    float memoryUsage;
    float diskUsage;
    float networkActivity;
};

struct PackageSearchResult {
    std::string name;
    std::string version;
    std::string description;
    PackageFormat format;
    std::string source;
    bool installed;
};

struct HistoryEntry {
    std::string command;
    std::string directory;
    std::time_t timestamp;
    bool success;
    std::vector<std::string> tags;
};

class NexSh {
public:
    NexSh(int argc, char* argv[]);
    ~NexSh();

    // Main shell loop
    void run();
    int getExitCode() const { return m_exitCode; }

    // Configuration methods
    void setAIEnabled(bool enabled) { m_aiEnabled = enabled; }
    void setUniversalPackagesEnabled(bool enabled) { m_universalPackagesEnabled = enabled; }
    void setSecurityEnabled(bool enabled) { m_securityEnabled = enabled; }

    // AI interaction methods
    void enableAI() { m_aiEnabled = true; }
    void disableAI() { m_aiEnabled = false; }
    bool isAIEnabled() const { return m_aiEnabled; }

private:
    // Core initialization
    void initializeShell();
    void setupAI();
    void setupReadline();
    void setupSignalHandlers();
    void loadConfiguration();
    void cleanup();

    // Command processing
    void processCommand(const std::string& input);
    bool handleBuiltinCommand(const std::string& input);
    void executeCommand(const std::string& command);

    // Built-in command handlers
    bool handleCd(const std::vector<std::string>& tokens);
    bool handleAICommand(const std::vector<std::string>& tokens);
    bool handleUniversalInstall(const std::vector<std::string>& tokens);
    bool handleUniversalSearch(const std::vector<std::string>& tokens);
    bool handleSemanticHistorySearch(const std::vector<std::string>& tokens);
    bool handleCommandPrediction(const std::vector<std::string>& tokens);
    bool handleCommandExplanation(const std::vector<std::string>& tokens);
    bool handleSystemOptimization(const std::vector<std::string>& tokens);
    bool handleSecurityCommand(const std::vector<std::string>& tokens);

    // AI assistant methods
    void explainCommand(const std::string& command);
    void showAISuggestions();
    void triggerAILearning();
    void showAIStatus();

    // Utility methods
    std::vector<std::string> tokenize(const std::string& input);
    std::string joinTokens(const std::vector<std::string>& tokens, size_t start);
    std::string generatePrompt();
    std::string shortenPath(const std::string& path);
    std::string getGitBranch();
    bool isNaturalLanguageCommand(const std::string& input);
    void displayWelcome();
    void displaySearchResults(const std::vector<PackageSearchResult>& results);
    void displayHistoryResults(const std::vector<HistoryEntry>& results);
    void parseArguments(int argc, char* argv[]);

    // Core components
    std::unique_ptr<NaturalLanguageProcessor> m_nlProcessor;
    std::unique_ptr<CommandPredictor> m_commandPredictor;
    std::unique_ptr<SmartCompletion> m_smartCompletion;
    std::unique_ptr<UniversalPackageManager> m_packageManager;
    std::unique_ptr<CommandSecurity> m_security;
    std::unique_ptr<PerformanceMonitor> m_perfMonitor;
    std::unique_ptr<SemanticHistory> m_history;

    // Shell state
    bool m_running;
    bool m_aiEnabled;
    bool m_universalPackagesEnabled;
    bool m_securityEnabled;
    std::string m_currentPath;
    int m_exitCode;

    // Configuration
    std::map<std::string, std::string> m_config;
    std::vector<std::string> m_aliases;
    std::vector<std::string> m_environmentVars;
};

// Global callback functions for readline
extern "C" {
    char** nexsh_completion(const char* text, int start, int end);
    char* nexsh_generator(const char* text, int state);
    int ai_suggest_command(int count, int key);
    int show_predictions(int count, int key);
    int semantic_search(int count, int key);
}

#endif // NEXSH_H