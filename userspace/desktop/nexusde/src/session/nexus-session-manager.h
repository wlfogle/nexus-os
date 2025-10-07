#ifndef NEXUS_SESSION_MANAGER_H
#define NEXUS_SESSION_MANAGER_H

#include <QObject>
#include <QDateTime>
#include <QTimer>
#include <QSettings>
#include <QVariantMap>

// Forward declarations
class HybridGPUController;
class AIPowerManager;
class SessionSecurity;
class SessionIntelligence;

// Enums and data structures
enum GPUType {
    GPU_INTEGRATED,
    GPU_DISCRETE,
    GPU_DISCRETE_PREFERRED,
    GPU_AUTO
};

enum SecurityLevel {
    SECURITY_LOW,
    SECURITY_MEDIUM,
    SECURITY_HIGH,
    SECURITY_CRITICAL
};

enum OptimizationType {
    OPTIMIZATION_GPU,
    OPTIMIZATION_POWER,
    OPTIMIZATION_WORKSPACE,
    OPTIMIZATION_MEMORY,
    OPTIMIZATION_NETWORK
};

// Data structures
struct GPUSwitchingRules {
    GPUType gamingApps;
    GPUType mediaEncoding;
    GPUType rendering3D;
    GPUType videoEditing;
    GPUType webBrowsing;
    GPUType textEditing;
    GPUType fileManagement;
    GPUType aiWorkloads;
};

struct SystemMetrics {
    QDateTime timestamp;
    float cpuUsage;
    float memoryUsage;
    float gpuUsage;
    float powerConsumption;
    float temperature;
    float networkActivity;
    QVariantMap additionalMetrics;
};

struct OptimizationSuggestion {
    OptimizationType type;
    QString description;
    QVariantMap parameters;
    float confidence;
    QString reasoning;
    int priority;
};

class NexusSessionManager : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool sessionActive READ sessionActive NOTIFY sessionActiveChanged)
    Q_PROPERTY(bool aiEnabled READ aiEnabled WRITE setAIEnabled NOTIFY aiEnabledChanged)
    Q_PROPERTY(bool hybridGpuEnabled READ hybridGpuEnabled WRITE setHybridGpuEnabled NOTIFY hybridGpuEnabledChanged)
    Q_PROPERTY(bool powerManagementEnabled READ powerManagementEnabled WRITE setPowerManagementEnabled NOTIFY powerManagementEnabledChanged)
    Q_PROPERTY(bool securityEnabled READ securityEnabled WRITE setSecurityEnabled NOTIFY securityEnabledChanged)
    Q_PROPERTY(QString currentUser READ currentUser CONSTANT)
    Q_PROPERTY(QString sessionId READ sessionId CONSTANT)

public:
    explicit NexusSessionManager(QObject *parent = nullptr);
    ~NexusSessionManager();

    // Property getters
    bool sessionActive() const { return m_sessionActive; }
    bool aiEnabled() const { return m_aiEnabled; }
    bool hybridGpuEnabled() const { return m_hybridGpuEnabled; }
    bool powerManagementEnabled() const { return m_powerManagementEnabled; }
    bool securityEnabled() const { return m_securityEnabled; }
    QString currentUser() const { return m_currentUser; }
    QString sessionId() const { return m_sessionId; }

    // Property setters
    void setAIEnabled(bool enabled);
    void setHybridGpuEnabled(bool enabled);
    void setPowerManagementEnabled(bool enabled);
    void setSecurityEnabled(bool enabled);

    // Session control
    Q_INVOKABLE void startSession();
    Q_INVOKABLE void stopSession();
    Q_INVOKABLE void suspendSession();
    Q_INVOKABLE void resumeSession();
    Q_INVOKABLE void restartSession();

    // System information
    Q_INVOKABLE SystemMetrics getCurrentMetrics() const;
    Q_INVOKABLE QVariantMap getSessionInfo() const;
    Q_INVOKABLE QString getUptime() const;

    // Configuration
    Q_INVOKABLE void loadUserPreferences();
    Q_INVOKABLE void saveUserPreferences();
    Q_INVOKABLE void resetToDefaults();

public slots:
    // GPU management
    void handleGPUSwitch(const QString &appId, GPUType fromGPU, GPUType toGPU);
    void handleGPUPowerState(const QString &state);
    
    // Power management
    void handlePowerProfileChange(const QString &profile);
    void handleBatteryOptimization(bool enabled);
    void handleThermalThrottling(const QString &reason);
    
    // Security
    void handleSecurityEvent(const QString &event, SecurityLevel level);
    void handleSecurityThreat(const QString &threat);
    
    // AI optimization
    void handleAIOptimization(const OptimizationSuggestion &suggestion);
    void handleWorkspaceOptimization(const QVariantMap &optimization);
    void handleApplicationPrediction(const QStringList &predictions);
    
    // System monitoring
    void updateSystemMetrics();
    void checkOptimizationOpportunities(const SystemMetrics &metrics);

signals:
    // Property change notifications
    void sessionActiveChanged(bool active);
    void aiEnabledChanged(bool enabled);
    void hybridGpuEnabledChanged(bool enabled);
    void powerManagementEnabledChanged(bool enabled);
    void securityEnabledChanged(bool enabled);
    
    // Session events
    void sessionStarted();
    void sessionStopped();
    void sessionSuspended();
    void sessionResumed();
    void sessionError(const QString &error);
    
    // System events
    void systemMetricsUpdated(const SystemMetrics &metrics);
    void performanceAlert(const QString &alert);
    void optimizationApplied(const OptimizationSuggestion &optimization);
    
    // GPU events
    void gpuSwitchCompleted(const QString &appId, GPUType gpu);
    void gpuPowerStateChanged(const QString &state);
    
    // Power events
    void powerProfileChanged(const QString &profile);
    void batteryModeChanged(bool batteryMode);
    void thermalStateChanged(const QString &state);
    
    // Security events
    void securityEventOccurred(const QString &event, SecurityLevel level);
    void digitalFortressStatusChanged(bool active);
    void vaultwardenStatusChanged(bool active);
    
    // AI events
    void aiOptimizationApplied(const OptimizationSuggestion &suggestion);
    void workspaceOptimized(const QVariantMap &optimization);
    void applicationPredicted(const QStringList &predictions);
    
    // Mascot notifications (Stella & Max Jr.)
    void stellaStatusChanged(bool active);
    void stellaSecurityAlert(const QString &event, SecurityLevel level);
    void maxJrStatusChanged(bool active);
    void maxJrOptimizationApplied(const OptimizationSuggestion &suggestion);

private slots:
    void saveSessionState();
    void restoreSessionState();

private:
    // Core initialization
    void initializeSession();
    void setupComponents();
    void connectSignals();
    void setupSessionDirectories();
    void setupDBusSession();
    
    // Service management
    void startCoreServices();
    void stopCoreServices();
    
    // System monitoring
    float getCPUUsage();
    float getMemoryUsage();
    float getThermalInfo();
    float getNetworkActivity();
    
    // Optimization
    void optimizeWorkspaces(const QVariantMap &parameters);
    void optimizeMemoryUsage(const QVariantMap &parameters);
    
    // Configuration
    GPUSwitchingRules loadGPURules();
    void logSecurityEvent(const QString &event, SecurityLevel level);
    
    // Utilities
    QString generateSessionId();
    QString getSessionStateFile();

    // Core components
    HybridGPUController *m_gpuController;
    AIPowerManager *m_powerManager;
    SessionSecurity *m_sessionSecurity;
    SessionIntelligence *m_sessionAI;
    
    // Timers
    QTimer *m_systemMonitor;
    QTimer *m_sessionPersistence;
    
    // Session state
    bool m_sessionActive;
    bool m_aiEnabled;
    bool m_hybridGpuEnabled;
    bool m_powerManagementEnabled;
    bool m_securityEnabled;
    QString m_currentUser;
    QString m_sessionId;
    QDateTime m_sessionStartTime;
    
    // System data
    QList<SystemMetrics> m_systemMetrics;
    QVariantMap m_userPreferences;
    QVariantMap m_sessionData;
};

// Register types for QML
Q_DECLARE_METATYPE(SystemMetrics)
Q_DECLARE_METATYPE(OptimizationSuggestion)
Q_DECLARE_METATYPE(GPUSwitchingRules)
Q_DECLARE_METATYPE(SecurityLevel)
Q_DECLARE_METATYPE(GPUType)

#endif // NEXUS_SESSION_MANAGER_H