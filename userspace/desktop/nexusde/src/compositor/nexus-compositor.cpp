/*
 * NexusDE Compositor - Hybrid X11/Wayland Compositor
 * Revolutionary desktop environment for NexusOS
 * 
 * Features:
 * - Seamless X11 and Wayland application support
 * - AI-powered window management and layout optimization
 * - Universal package format detection and integration
 * - Hybrid GPU acceleration with automatic switching
 * - Advanced security isolation between applications
 * - Real-time performance monitoring and optimization
 */

#include "nexus-compositor.h"
#include <QtWaylandCompositor>
#include <QGuiApplication>
#include <QOpenGLContext>
#include <QProcess>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTimer>
#include <QDebug>

#include "ai/window-intelligence.h"
#include "gpu/hybrid-gpu-manager.h"
#include "security/app-isolation.h"
#include "packages/universal-detector.h"

NexusCompositor::NexusCompositor(QObject *parent)
    : QWaylandCompositor(parent)
    , m_aiManager(new WindowIntelligence(this))
    , m_gpuManager(new HybridGPUManager(this))
    , m_securityManager(new AppIsolation(this))
    , m_packageDetector(new UniversalDetector(this))
    , m_xwaylandEnabled(true)
    , m_aiWindowManagement(true)
    , m_hybridGpuEnabled(true)
{
    setupCompositor();
    initializeAI();
    configureHybridGPU();
    setupXWayland();
    
    // Connect AI signals for intelligent window management
    connect(m_aiManager, &WindowIntelligence::windowLayoutSuggestion,
            this, &NexusCompositor::handleAILayoutSuggestion);
    connect(m_aiManager, &WindowIntelligence::workspaceOptimization,
            this, &NexusCompositor::optimizeWorkspaceLayout);
    
    // Connect GPU management signals
    connect(m_gpuManager, &HybridGPUManager::gpuSwitchRequested,
            this, &NexusCompositor::handleGPUSwitching);
    connect(m_gpuManager, &HybridGPUManager::performanceProfileChanged,
            this, &NexusCompositor::updatePerformanceProfile);
    
    // Universal package detection
    connect(m_packageDetector, &UniversalDetector::packageFormatDetected,
            this, &NexusCompositor::handlePackageFormatDetection);
}

void NexusCompositor::setupCompositor()
{
    // Configure Wayland compositor with X11 compatibility
    setRetainedSelectionEnabled(true);
    setSocketName("nexusde-0");
    
    // Initialize output management
    m_output = new QWaylandOutput(this, nullptr);
    m_output->setManufacturer("NexusOS");
    m_output->setModel("NexusDE Display");
    m_output->setPhysicalSize(QSize(300, 200)); // Will be updated dynamically
    
    // Set up hybrid rendering pipeline
    QWaylandCompositor::DefaultExtension extensions = 
        QWaylandCompositor::DefaultExtension::XdgShellV6 |
        QWaylandCompositor::DefaultExtension::WlShell |
        QWaylandCompositor::DefaultExtension::TextInputV2 |
        QWaylandCompositor::DefaultExtension::TextInputV3;
    
    setDefaultExtensions(extensions);
    
    // Custom extensions for NexusDE
    m_nexusShell = new NexusShell(this);
    m_aiExtension = new AIWindowExtension(this);
    m_packageExtension = new UniversalPackageExtension(this);
    
    qDebug() << "NexusDE Compositor initialized with hybrid X11/Wayland support";
}

void NexusCompositor::initializeAI()
{
    if (!m_aiWindowManagement) {
        return;
    }
    
    // Initialize AI models for window intelligence
    m_aiManager->loadModels();
    m_aiManager->enablePredictiveLayouts(true);
    m_aiManager->enableSmartTiling(true);
    m_aiManager->enableWorkspaceIntelligence(true);
    
    // Train on user patterns
    m_aiManager->startUserBehaviorAnalysis();
    
    qDebug() << "AI Window Management initialized";
}

void NexusCompositor::configureHybridGPU()
{
    if (!m_hybridGpuEnabled) {
        return;
    }
    
    // Detect available GPUs
    m_gpuManager->detectGPUs();
    
    // Configure GPU switching rules
    GPUSwitchingRules rules;
    rules.gamingApps = GPU_DISCRETE;
    rules.mediaEncoding = GPU_DISCRETE;
    rules.rendering3D = GPU_DISCRETE;
    rules.videoEditing = GPU_DISCRETE;
    rules.webBrowsing = GPU_INTEGRATED;
    rules.textEditing = GPU_INTEGRATED;
    rules.fileManagement = GPU_INTEGRATED;
    rules.aiWorkloads = GPU_DISCRETE_PREFERRED;
    
    m_gpuManager->setGPURules(rules);
    
    // Enable automatic power management
    m_gpuManager->enableAutomaticPowerManagement(true);
    
    qDebug() << "Hybrid GPU configuration completed";
}

void NexusCompositor::setupXWayland()
{
    if (!m_xwaylandEnabled) {
        return;
    }
    
    // Start XWayland server with optimized settings
    m_xwaylandProcess = new QProcess(this);
    m_xwaylandProcess->setProgram("Xwayland");
    
    QStringList xwaylandArgs;
    xwaylandArgs << ":1" << "-rootless" << "-terminate" << "-core" 
                 << "-listen" << "unix" << "-listen" << "tcp"
                 << "-dpi" << "96" << "-extension" << "MIT-SHM"
                 << "-extension" << "XTEST" << "-extension" << "GLX"
                 << "-extension" << "RANDR" << "-extension" << "RENDER"
                 << "-extension" << "XVideo";
    
    m_xwaylandProcess->setArguments(xwaylandArgs);
    
    connect(m_xwaylandProcess, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
            this, &NexusCompositor::onXWaylandFinished);
    
    // Set environment variables for X11 applications
    qputenv("DISPLAY", ":1");
    qputenv("WAYLAND_DISPLAY", "nexusde-0");
    
    m_xwaylandProcess->start();
    
    qDebug() << "XWayland server started for X11 compatibility";
}

void NexusCompositor::handleAILayoutSuggestion(const WindowLayoutSuggestion &suggestion)
{
    if (!m_aiWindowManagement) {
        return;
    }
    
    // Apply AI-suggested window layout
    for (const auto &windowPlacement : suggestion.placements) {
        QWaylandSurface *surface = findSurfaceById(windowPlacement.surfaceId);
        if (surface) {
            // Animate window to suggested position and size
            animateWindowToGeometry(surface, windowPlacement.geometry);
            
            // Update window properties based on AI suggestion
            if (windowPlacement.shouldFocus) {
                setFocusedSurface(surface);
            }
            
            // Apply GPU assignment if suggested
            if (windowPlacement.preferredGPU != GPU_AUTO) {
                m_gpuManager->assignWindowToGPU(surface, windowPlacement.preferredGPU);
            }
        }
    }
    
    qDebug() << "Applied AI window layout suggestion";
}

void NexusCompositor::optimizeWorkspaceLayout()
{
    // AI-driven workspace optimization
    auto surfaces = activeSurfaces();
    
    if (surfaces.isEmpty()) {
        return;
    }
    
    // Analyze current workspace usage
    WorkspaceAnalysis analysis = m_aiManager->analyzeCurrentWorkspace(surfaces);
    
    // Generate optimization plan
    WorkspaceOptimizationPlan plan = m_aiManager->generateOptimizationPlan(analysis);
    
    // Execute optimization
    for (const auto &action : plan.actions) {
        switch (action.type) {
        case MOVE_TO_WORKSPACE:
            moveWindowToWorkspace(action.surface, action.targetWorkspace);
            break;
        case RESIZE_WINDOW:
            resizeWindow(action.surface, action.newGeometry);
            break;
        case CHANGE_GPU_ASSIGNMENT:
            m_gpuManager->assignWindowToGPU(action.surface, action.targetGPU);
            break;
        case ADJUST_TRANSPARENCY:
            setWindowOpacity(action.surface, action.opacity);
            break;
        }
    }
    
    qDebug() << "Workspace optimization completed";
}

void NexusCompositor::handleGPUSwitching(QWaylandSurface *surface, GPUType targetGPU)
{
    if (!surface || !m_hybridGpuEnabled) {
        return;
    }
    
    // Perform GPU switching for the specific window
    bool success = m_gpuManager->switchWindowGPU(surface, targetGPU);
    
    if (success) {
        // Update window rendering context
        updateWindowRenderingContext(surface, targetGPU);
        
        // Notify AI system of GPU change for learning
        m_aiManager->recordGPUSwitch(surface, targetGPU);
        
        qDebug() << "GPU switching completed for surface" << surface;
    } else {
        qWarning() << "Failed to switch GPU for surface" << surface;
    }
}

void NexusCompositor::updatePerformanceProfile(const QString &profile)
{
    // Update compositor performance based on current profile
    if (profile == "performance") {
        enableHighPerformanceMode();
    } else if (profile == "balanced") {
        enableBalancedMode();
    } else if (profile == "powersave") {
        enablePowerSaveMode();
    }
    
    // Notify all components of profile change
    emit performanceProfileChanged(profile);
    
    qDebug() << "Performance profile updated to:" << profile;
}

void NexusCompositor::handlePackageFormatDetection(const QString &appId, const PackageFormat &format)
{
    // Handle different package formats with appropriate sandboxing and GPU assignment
    switch (format.type) {
    case PACKAGE_FLATPAK:
        m_securityManager->enableFlatpakSandbox(appId);
        m_gpuManager->setDefaultGPU(appId, GPU_INTEGRATED);
        break;
    case PACKAGE_SNAP:
        m_securityManager->enableSnapSandbox(appId);
        m_gpuManager->setDefaultGPU(appId, GPU_INTEGRATED);
        break;
    case PACKAGE_APPIMAGE:
        m_securityManager->enableAppImageSandbox(appId);
        m_gpuManager->setDefaultGPU(appId, GPU_AUTO);
        break;
    case PACKAGE_NATIVE:
        // Native packages get full system access but smart GPU assignment
        m_gpuManager->setDefaultGPU(appId, GPU_AUTO);
        break;
    case PACKAGE_WINE:
        // Windows apps through Wine get discrete GPU preference
        m_gpuManager->setDefaultGPU(appId, GPU_DISCRETE_PREFERRED);
        break;
    }
    
    qDebug() << "Package format detected:" << format.name << "for app:" << appId;
}

void NexusCompositor::enableHighPerformanceMode()
{
    // Configure compositor for maximum performance
    setFrameRate(165); // High refresh rate
    setVSync(true);
    enableGPUAcceleration(true);
    setCompositorEffects(EFFECTS_MINIMAL);
    
    // Switch default GPU to discrete
    m_gpuManager->setDefaultGPUMode(GPU_DISCRETE_PREFERRED);
    
    qDebug() << "High performance mode enabled";
}

void NexusCompositor::enableBalancedMode()
{
    // Balanced performance and power consumption
    setFrameRate(60);
    setVSync(true);
    enableGPUAcceleration(true);
    setCompositorEffects(EFFECTS_NORMAL);
    
    // Use hybrid GPU switching
    m_gpuManager->setDefaultGPUMode(GPU_AUTO);
    
    qDebug() << "Balanced mode enabled";
}

void NexusCompositor::enablePowerSaveMode()
{
    // Optimize for power efficiency
    setFrameRate(30);
    setVSync(false);
    enableGPUAcceleration(false);
    setCompositorEffects(EFFECTS_DISABLED);
    
    // Prefer integrated GPU
    m_gpuManager->setDefaultGPUMode(GPU_INTEGRATED);
    
    qDebug() << "Power save mode enabled";
}

// Window management helper functions
void NexusCompositor::animateWindowToGeometry(QWaylandSurface *surface, const QRect &geometry)
{
    // Smooth animation to new window geometry
    // Implementation would use Qt's animation framework
    // For now, direct assignment
    if (auto *view = findViewForSurface(surface)) {
        view->setPosition(geometry.topLeft());
        view->setSize(geometry.size());
    }
}

void NexusCompositor::moveWindowToWorkspace(QWaylandSurface *surface, int workspace)
{
    // Move window to different workspace
    if (auto *view = findViewForSurface(surface)) {
        view->setWorkspace(workspace);
        emit windowMovedToWorkspace(surface, workspace);
    }
}

void NexusCompositor::resizeWindow(QWaylandSurface *surface, const QRect &geometry)
{
    if (auto *view = findViewForSurface(surface)) {
        view->setSize(geometry.size());
        view->setPosition(geometry.topLeft());
    }
}

void NexusCompositor::setWindowOpacity(QWaylandSurface *surface, float opacity)
{
    if (auto *view = findViewForSurface(surface)) {
        view->setOpacity(opacity);
    }
}

QWaylandSurface* NexusCompositor::findSurfaceById(const QString &id)
{
    // Find surface by unique identifier
    for (auto *surface : surfaces()) {
        if (surface->client() && surface->client()->processId() == id.toInt()) {
            return surface;
        }
    }
    return nullptr;
}

NexusCompositor::~NexusCompositor()
{
    if (m_xwaylandProcess && m_xwaylandProcess->state() == QProcess::Running) {
        m_xwaylandProcess->terminate();
        if (!m_xwaylandProcess->waitForFinished(3000)) {
            m_xwaylandProcess->kill();
        }
    }
}