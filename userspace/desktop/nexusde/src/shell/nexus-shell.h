#ifndef NEXUS_SHELL_H
#define NEXUS_SHELL_H

#include <QObject>
#include <QQmlApplicationEngine>
#include <QDBusInterface>
#include <QTimer>
#include <QVariantMap>
#include <QTime>
#include <QDateTime>
#include <QRect>

// Forward declarations
class AppIntelligence;
class WorkspaceOptimizer;
class UniversalLauncher;
class SmartPanel;
class AIAssistant;
class SystemMonitor;
class SecurityStatus;
class PackageManager;

// Enums for shell components
enum class PanelPosition {
    Top,
    Bottom,
    Left,
    Right,
    Floating
};

enum class SecurityAlertLevel {
    Low,
    Normal,
    High,
    Critical
};

enum class AICommandType {
    AppLaunch,
    SystemControl,
    WorkspaceManagement,
    PackageManagement,
    SecurityOperation,
    Information,
    Unknown
};

enum class PackageFormat {
    Auto,
    Flatpak,
    Snap,
    AppImage,
    Native,
    Wine
};

enum class SecurityRisk {
    Low,
    Medium,
    High,
    Critical
};

enum class AlertSeverity {
    Info,
    Warning,
    Critical
};

enum class OptimizationType {
    MemoryCleanup,
    GPUSwitching,
    ProcessPriority,
    WorkspaceReorganization
};

enum class GPUType {
    Integrated,
    Discrete,
    Auto
};

// Data structures for AI and system management
struct AICommandResult {
    AICommandType type;
    QString target;
    QVariantMap parameters;
    float confidence;
    QString reasoning;
};

struct AppLaunchContext {
    QString appId;
    int currentWorkspace;
    QVariantMap userPreferences;
    QVariantMap systemState;
    QRect preferredGeometry;
};

struct LaunchRecommendations {
    GPUType preferredGPU;
    int preferredWorkspace;
    QRect preferredGeometry;
    float confidence;
    QString reasoning;
};

struct SecurityAssessment {
    SecurityRisk riskLevel;
    QString reason;
    QStringList suggestions;
    float trustScore;
};

struct PackageInstallRequest {
    QString packageName;
    PackageFormat format;
    SecurityRisk securityLevel;
    bool userConfirmed;
};

struct SearchContext {
    QString query;
    int currentWorkspace;
    QStringList recentApps;
    QVariantMap userPreferences;
    QTime timeOfDay;
    QString systemActivity;
};

struct SearchResult {
    QString name;
    QString description;
    QString category;
    QString iconPath;
    bool installed;
    float relevanceScore;
    QStringList installOptions;
    float securityRating;
};

struct WorkspaceLayout {
    QString name;
    QString description;
    QVariantMap configuration;
    float confidence;
    QString reasoning;
};

struct PerformanceAlert {
    AlertSeverity severity;
    QString message;
    QString component;
    QVariantMap metrics;
    QStringList suggestions;
};

struct OptimizationAction {
    OptimizationType type;
    QVariantMap parameters;
    float expectedImprovement;
    QString description;
};

struct OptimizationPlan {
    QList<OptimizationAction> actions;
    float expectedImprovement;
    QString strategy;
};

class NexusShell : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool aiEnabled READ aiEnabled WRITE setAIEnabled NOTIFY aiEnabledChanged)
    Q_PROPERTY(bool adaptiveUI READ adaptiveUI WRITE setAdaptiveUI NOTIFY adaptiveUIChanged)
    Q_PROPERTY(bool multiMonitorEnabled READ multiMonitorEnabled WRITE setMultiMonitorEnabled NOTIFY multiMonitorEnabledChanged)
    Q_PROPERTY(int currentWorkspace READ currentWorkspace NOTIFY workspaceChanged)
    Q_PROPERTY(float aiConfidence READ aiConfidence WRITE setAIConfidence NOTIFY aiConfidenceChanged)

public:
    explicit NexusShell(QObject *parent = nullptr);
    ~NexusShell();

    // Property getters
    bool aiEnabled() const { return m_aiEnabled; }
    bool adaptiveUI() const { return m_adaptiveUI; }
    bool multiMonitorEnabled() const { return m_multiMonitorEnabled; }
    int currentWorkspace() const { return m_currentWorkspace; }
    float aiConfidence() const { return m_aiConfidence; }

    // Property setters
    void setAIEnabled(bool enabled);
    void setAdaptiveUI(bool enabled);
    void setMultiMonitorEnabled(bool enabled);
    void setAIConfidence(float confidence);

    // Shell management
    void loadShell();
    void showWorkspace(int workspace);
    void hideShell();

    // D-Bus scriptable interface
    Q_SCRIPTABLE void showLauncher();
    Q_SCRIPTABLE void hideLauncher();
    Q_SCRIPTABLE void toggleLauncher();
    Q_SCRIPTABLE QString getAIAssistantStatus();
    Q_SCRIPTABLE void executeAICommand(const QString &command);
    Q_SCRIPTABLE int getCurrentWorkspace();
    Q_SCRIPTABLE void setCurrentWorkspace(int workspace);

public slots:
    // AI Command handling
    void handleAICommand(const QString &command, const QVariantMap &context = QVariantMap());
    void handleAppLaunch(const QString &appId);
    
    // Application and package management
    void launchApplication(const QString &appId, const QVariantMap &options = QVariantMap());
    void installPackage(const QString &packageName, const QString &preferredFormat = QString());
    
    // Search and discovery
    void handleSearchQuery(const QString &query);
    
    // Workspace management
    void handleLayoutSuggestion(const WorkspaceLayout &layout);
    void switchWorkspace(int workspaceIndex);
    
    // Performance and optimization
    void handlePerformanceAlert(const PerformanceAlert &alert);
    void optimizeSystemResources();
    void performPeriodicOptimization();
    
    // Security
    void handleSecurityThreat(const QString &threat);
    void updateSecurityIndicator(const QString &status);

signals:
    // Property change notifications
    void aiEnabledChanged(bool enabled);
    void adaptiveUIChanged(bool enabled);
    void multiMonitorEnabledChanged(bool enabled);
    void workspaceChanged(int workspace);
    void aiConfidenceChanged(float confidence);
    
    // Application events
    void applicationLaunched(const QString &appId);
    void applicationClosed(const QString &appId);
    
    // Package management events
    void installationStarted(const QString &packageName);
    void installationCompleted(const QString &packageName);
    void installationError(const QString &error);
    
    // Search events
    void searchResults(const QVariant &results);
    void searchCompleted(const QString &query);
    
    // AI events
    void aiResponse(const QString &response);
    void layoutSuggestion(const QVariant &layout);
    void layoutApplied(const QString &layoutName);
    
    // Performance events
    void criticalPerformanceAlert(const QString &message);
    void performanceOptimizationSuggestion(const QVariant &suggestion);
    void systemOptimized(float improvement);
    
    // Security events
    void securityWarning(const QString &warning);
    void threatDetected(const QString &threat);
    void securityStatusChanged(const QString &status);

private slots:
    void onPackageInstalled(const QDBusMessage &message);
    void onPackageInstallError(const QDBusError &error);

private:
    // Core initialization
    void initializeShell();
    void setupAI();
    void setupComponents();
    void setupQMLContext();
    void connectSignals();
    void setupDBus();
    void loadConfiguration();
    void saveConfiguration();
    
    // Multi-monitor support
    void setupMultiMonitor();
    void createShellInstance(QScreen *screen, int index);
    void configureScreen(QScreen *screen, int index);
    
    // Background services
    void startBackgroundServices();
    
    // AI command processing
    void handleSystemControl(const QString &target, const QVariantMap &parameters);
    void handleWorkspaceManagement(const QString &target, const QVariantMap &parameters);
    void handlePackageManagement(const QString &target, const QVariantMap &parameters);
    void handleSecurityOperation(const QString &target, const QVariantMap &parameters);
    void handleInformationRequest(const QString &target, const QVariantMap &parameters);
    
    // Application management
    bool executeAppLaunch(const QString &appId, const AppLaunchContext &context);
    void handleAppLaunchFailure(const QString &appId, const AppLaunchContext &context);
    void updateAppUsageStats(const QString &appId);
    
    // Package management
    PackageFormat parsePackageFormat(const QString &format);
    void requestUserConfirmation(const QString &packageName, const SecurityAssessment &security);
    
    // Optimization
    void applyCriticalOptimizations(const OptimizationPlan &plan);
    void performBasicOptimization();
    void performMemoryCleanup(const QVariantMap &parameters);
    void optimizeGPUUsage(const QVariantMap &parameters);
    void adjustProcessPriorities(const QVariantMap &parameters);
    void reorganizeWorkspaces(const QVariantMap &parameters);
    void optimizeUILayout(const QVariantMap &patterns);
    void cleanupResources();
    
    // Workspace management
    void applyWorkspaceLayout(const WorkspaceLayout &layout);
    
    // System information
    QVariantMap getUserPreferences(const QString &appId);
    QVariantMap getSystemState();
    QStringList getRecentApplications(int count);
    QVariantMap getUserSearchPreferences();
    QString getSystemActivity();
    
    // Logging and monitoring
    void logPerformanceInfo(const PerformanceAlert &alert);

    // Core components
    AppIntelligence *m_appIntelligence;
    WorkspaceOptimizer *m_workspaceOptimizer;
    
    // UI components
    UniversalLauncher *m_universalLauncher;
    SmartPanel *m_smartPanel;
    AIAssistant *m_aiAssistant;
    SystemMonitor *m_systemMonitor;
    SecurityStatus *m_securityStatus;
    PackageManager *m_packageManager;
    
    // QML and UI
    QQmlApplicationEngine *m_qmlEngine;
    
    // D-Bus interfaces
    QDBusInterface *m_compositorInterface;
    QDBusInterface *m_packageDaemonInterface;
    
    // Timers
    QTimer *m_optimizationTimer;
    
    // Configuration
    bool m_aiEnabled;
    bool m_adaptiveUI;
    bool m_multiMonitorEnabled;
    int m_currentWorkspace;
    float m_aiConfidence;
    
    // Runtime state
    QVariantMap m_configuration;
};

// Register types for QML
Q_DECLARE_METATYPE(AICommandResult)
Q_DECLARE_METATYPE(SearchResult)
Q_DECLARE_METATYPE(WorkspaceLayout)
Q_DECLARE_METATYPE(PerformanceAlert)
Q_DECLARE_METATYPE(OptimizationPlan)

#endif // NEXUS_SHELL_H