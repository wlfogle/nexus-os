#ifndef NEXUS_COMPOSITOR_H
#define NEXUS_COMPOSITOR_H

#include <QtWaylandCompositor/QWaylandCompositor>
#include <QtWaylandCompositor/QWaylandSurface>
#include <QtWaylandCompositor/QWaylandOutput>
#include <QtWaylandCompositor/QWaylandView>
#include <QObject>
#include <QProcess>
#include <QRect>
#include <QString>
#include <QList>

// Forward declarations
class WindowIntelligence;
class HybridGPUManager;
class AppIsolation;
class UniversalDetector;
class NexusShell;
class AIWindowExtension;
class UniversalPackageExtension;

// Enums for GPU types and performance modes
enum GPUType {
    GPU_INTEGRATED,
    GPU_DISCRETE,
    GPU_DISCRETE_PREFERRED,
    GPU_AUTO
};

enum PerformanceMode {
    PERFORMANCE_HIGH,
    PERFORMANCE_BALANCED,
    PERFORMANCE_POWERSAVE
};

enum CompositorEffects {
    EFFECTS_DISABLED,
    EFFECTS_MINIMAL,
    EFFECTS_NORMAL,
    EFFECTS_ENHANCED
};

enum PackageType {
    PACKAGE_NATIVE,
    PACKAGE_FLATPAK,
    PACKAGE_SNAP,
    PACKAGE_APPIMAGE,
    PACKAGE_WINE,
    PACKAGE_UNKNOWN
};

enum OptimizationActionType {
    MOVE_TO_WORKSPACE,
    RESIZE_WINDOW,
    CHANGE_GPU_ASSIGNMENT,
    ADJUST_TRANSPARENCY
};

// Data structures for AI and GPU management
struct PackageFormat {
    PackageType type;
    QString name;
    QString version;
    QStringList capabilities;
};

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

struct WindowPlacement {
    QString surfaceId;
    QRect geometry;
    bool shouldFocus;
    GPUType preferredGPU;
    float opacity;
    int workspace;
};

struct WindowLayoutSuggestion {
    QList<WindowPlacement> placements;
    QString reasoning;
    float confidence;
};

struct WorkspaceAnalysis {
    int activeWindows;
    QStringList applicationTypes;
    float cpuUsage;
    float memoryUsage;
    float gpuUsage;
    QString dominantActivity;
};

struct OptimizationAction {
    OptimizationActionType type;
    QWaylandSurface *surface;
    int targetWorkspace;
    QRect newGeometry;
    GPUType targetGPU;
    float opacity;
};

struct WorkspaceOptimizationPlan {
    QList<OptimizationAction> actions;
    QString strategy;
    float expectedImprovement;
};

class NexusCompositor : public QWaylandCompositor
{
    Q_OBJECT
    Q_PROPERTY(bool aiWindowManagement READ aiWindowManagement WRITE setAIWindowManagement NOTIFY aiWindowManagementChanged)
    Q_PROPERTY(bool hybridGpuEnabled READ hybridGpuEnabled WRITE setHybridGpuEnabled NOTIFY hybridGpuEnabledChanged)
    Q_PROPERTY(bool xwaylandEnabled READ xwaylandEnabled WRITE setXWaylandEnabled NOTIFY xwaylandEnabledChanged)
    Q_PROPERTY(QString performanceProfile READ performanceProfile NOTIFY performanceProfileChanged)

public:
    explicit NexusCompositor(QObject *parent = nullptr);
    ~NexusCompositor();

    // Property getters
    bool aiWindowManagement() const { return m_aiWindowManagement; }
    bool hybridGpuEnabled() const { return m_hybridGpuEnabled; }
    bool xwaylandEnabled() const { return m_xwaylandEnabled; }
    QString performanceProfile() const { return m_currentProfile; }

    // Property setters
    void setAIWindowManagement(bool enabled);
    void setHybridGpuEnabled(bool enabled);
    void setXWaylandEnabled(bool enabled);

    // Public API for external components
    Q_INVOKABLE void optimizeWorkspaceLayout();
    Q_INVOKABLE void switchToPerformanceProfile(const QString &profile);
    Q_INVOKABLE bool isApplicationSupported(const QString &appId);
    Q_INVOKABLE GPUType getRecommendedGPU(const QString &appId);
    Q_INVOKABLE void enableAIAssist(bool enabled);

public slots:
    // AI-driven window management
    void handleAILayoutSuggestion(const WindowLayoutSuggestion &suggestion);
    void optimizeWorkspaceLayout();
    
    // GPU management
    void handleGPUSwitching(QWaylandSurface *surface, GPUType targetGPU);
    void updatePerformanceProfile(const QString &profile);
    
    // Package format handling
    void handlePackageFormatDetection(const QString &appId, const PackageFormat &format);
    
    // XWayland management
    void onXWaylandFinished(int exitCode, QProcess::ExitStatus exitStatus);

signals:
    // Property change notifications
    void aiWindowManagementChanged(bool enabled);
    void hybridGpuEnabledChanged(bool enabled);
    void xwaylandEnabledChanged(bool enabled);
    void performanceProfileChanged(const QString &profile);
    
    // Workspace and window events
    void windowMovedToWorkspace(QWaylandSurface *surface, int workspace);
    void workspaceOptimized(const QString &strategy);
    void aiSuggestionApplied(const WindowLayoutSuggestion &suggestion);
    
    // GPU events
    void gpuSwitched(QWaylandSurface *surface, GPUType newGPU);
    void performanceModeChanged(PerformanceMode mode);
    
    // System events
    void packageFormatDetected(const QString &appId, PackageType type);
    void securityPolicyApplied(const QString &appId, const QString &policy);

private slots:
    void onSurfaceCreated(QWaylandSurface *surface);
    void onSurfaceDestroyed();
    void updateSystemMetrics();

private:
    // Core setup methods
    void setupCompositor();
    void initializeAI();
    void configureHybridGPU();
    void setupXWayland();
    
    // Performance mode methods
    void enableHighPerformanceMode();
    void enableBalancedMode();
    void enablePowerSaveMode();
    
    // Window management utilities
    void animateWindowToGeometry(QWaylandSurface *surface, const QRect &geometry);
    void moveWindowToWorkspace(QWaylandSurface *surface, int workspace);
    void resizeWindow(QWaylandSurface *surface, const QRect &geometry);
    void setWindowOpacity(QWaylandSurface *surface, float opacity);
    void updateWindowRenderingContext(QWaylandSurface *surface, GPUType gpu);
    
    // Utility methods
    QWaylandSurface* findSurfaceById(const QString &id);
    QWaylandView* findViewForSurface(QWaylandSurface *surface);
    QList<QWaylandSurface*> activeSurfaces();
    
    // Compositor configuration
    void setFrameRate(int fps);
    void setVSync(bool enabled);
    void enableGPUAcceleration(bool enabled);
    void setCompositorEffects(CompositorEffects level);

    // Core components
    WindowIntelligence *m_aiManager;
    HybridGPUManager *m_gpuManager;
    AppIsolation *m_securityManager;
    UniversalDetector *m_packageDetector;
    
    // Wayland components
    QWaylandOutput *m_output;
    NexusShell *m_nexusShell;
    AIWindowExtension *m_aiExtension;
    UniversalPackageExtension *m_packageExtension;
    
    // XWayland support
    QProcess *m_xwaylandProcess;
    
    // Configuration
    bool m_aiWindowManagement;
    bool m_hybridGpuEnabled;
    bool m_xwaylandEnabled;
    QString m_currentProfile;
    
    // Performance tracking
    QTimer *m_metricsTimer;
    QTimer *m_optimizationTimer;
};

#endif // NEXUS_COMPOSITOR_H