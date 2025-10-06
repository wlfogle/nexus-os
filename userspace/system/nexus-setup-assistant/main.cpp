#include <QApplication>
#include <QMainWindow>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGridLayout>
#include <QLabel>
#include <QPushButton>
#include <QProgressBar>
#include <QTextEdit>
#include <QGroupBox>
#include <QCheckBox>
#include <QComboBox>
#include <QLineEdit>
#include <QTabWidget>
#include <QScrollArea>
#include <QPixmap>
#include <QIcon>
#include <QMessageBox>
#include <QProcess>
#include <QTimer>
#include <QSystemTrayIcon>
#include <QMenu>
#include <QSettings>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QSplashScreen>
#include <QThread>
#include <QMutex>
#include <QWaitCondition>

class NexusSetupAssistant : public QMainWindow {
    Q_OBJECT

public:
    NexusSetupAssistant(QWidget *parent = nullptr);
    ~NexusSetupAssistant();

private slots:
    void setupMediaStack();
    void setupSelfHosting();
    void setupDevelopment();
    void setupGaming();
    void setupSecurity();
    void installCustomPackages();
    void systemOptimization();
    void showAbout();
    void toggleDigitalFortress();
    void openVaultwarden();
    void runDiagnostics();
    void showSystemInfo();
    void installFromDistro();

private:
    void setupUI();
    void setupTrayIcon();
    void loadConfig();
    void saveConfig();
    void runCommand(const QString &command, bool async = true);
    void updateProgress(const QString &message, int value);
    void showNotification(const QString &title, const QString &message);
    
    // UI Components
    QTabWidget *m_tabWidget;
    QTextEdit *m_logOutput;
    QProgressBar *m_progressBar;
    QLabel *m_statusLabel;
    QSystemTrayIcon *m_trayIcon;
    
    // Media Stack Tab
    QGroupBox *m_mediaGroup;
    QCheckBox *m_plexCheck;
    QCheckBox *m_jellyfinCheck;
    QCheckBox *m_sonarrCheck;
    QCheckBox *m_radarrCheck;
    QCheckBox *m_lidarrCheck;
    QCheckBox *m_readarrCheck;
    QCheckBox *m_qbittorrentCheck;
    QCheckBox *m_prowlarrCheck;
    QCheckBox *m_overseerrCheck;
    QCheckBox *m_tautulliCheck;
    
    // Self-Hosting Tab
    QGroupBox *m_hostingGroup;
    QCheckBox *m_nextcloudCheck;
    QCheckBox *m_homeAssistantCheck;
    QCheckBox *m_grafanaCheck;
    QCheckBox *m_prometheusCheck;
    QCheckBox *m_portainerCheck;
    QCheckBox *m_traefikCheck;
    QCheckBox *m_photoprismCheck;
    QCheckBox *m_paperlessCheck;
    
    // Development Tab
    QGroupBox *m_devGroup;
    QCheckBox *m_gitlabCheck;
    QCheckBox *m_jenkinsCheck;
    QCheckBox *m_codeServerCheck;
    QCheckBox *m_aiCodingCheck;
    QCheckBox *m_dockerRegistryCheck;
    QCheckBox *m_redisCheck;
    QCheckBox *m_postgresCheck;
    QCheckBox *m_mongoCheck;
    
    // Gaming Tab
    QGroupBox *m_gamingGroup;
    QCheckBox *m_steamCheck;
    QCheckBox *m_lutrisCheck;
    QCheckBox *m_gameStreamCheck;
    QCheckBox *m_retroArchCheck;
    QCheckBox *m_minecraftCheck;
    
    // Security Tab (Base System)
    QGroupBox *m_securityGroup;
    QPushButton *m_digitalFortressBtn;
    QPushButton *m_vaultwardenBtn;
    QLabel *m_fortressStatus;
    QLabel *m_vaultwardenStatus;
    
    // Package Installation Tab
    QComboBox *m_distroCombo;
    QLineEdit *m_packageEdit;
    QPushButton *m_installBtn;
    QTextEdit *m_packageResults;
    
    // Configuration
    QSettings *m_settings;
    QNetworkAccessManager *m_networkManager;
    
    // Awesome Stack Integration
    QString m_awesomeStackPath;
    bool m_fortressEnabled;
    bool m_vaultwardenRunning;
};

NexusSetupAssistant::NexusSetupAssistant(QWidget *parent)
    : QMainWindow(parent)
    , m_settings(new QSettings("NexusOS", "SetupAssistant", this))
    , m_networkManager(new QNetworkAccessManager(this))
    , m_awesomeStackPath("/run/media/garuda/34c008f3-1990-471c-bd80-c72985c7dc5c/@home/lou/Repos/github/awesome-stack")
    , m_fortressEnabled(false)
    , m_vaultwardenRunning(false)
{
    setWindowTitle("NexusOS Setup Assistant");
    setWindowIcon(QIcon(":/icons/nexusos.png"));
    setMinimumSize(900, 700);
    
    setupUI();
    setupTrayIcon();
    loadConfig();
    
    // Check if awesome-stack is available
    if (QDir(m_awesomeStackPath).exists()) {
        m_statusLabel->setText("✅ Awesome Stack detected - Full features available");
        m_statusLabel->setStyleSheet("color: green; font-weight: bold;");
    } else {
        m_statusLabel->setText("⚠️ Awesome Stack not detected - Limited features");
        m_statusLabel->setStyleSheet("color: orange; font-weight: bold;");
    }
    
    // Show splash screen
    QSplashScreen *splash = new QSplashScreen(QPixmap(":/images/nexusos-splash.png"));
    splash->show();
    splash->showMessage("Initializing NexusOS Setup Assistant...", Qt::AlignBottom | Qt::AlignCenter, Qt::white);
    
    QTimer::singleShot(2000, [splash]() {
        splash->close();
        splash->deleteLater();
    });
}

NexusSetupAssistant::~NexusSetupAssistant() {
    saveConfig();
}

void NexusSetupAssistant::setupUI() {
    auto *centralWidget = new QWidget;
    setCentralWidget(centralWidget);
    
    auto *mainLayout = new QVBoxLayout(centralWidget);
    
    // Header
    auto *headerLayout = new QHBoxLayout;
    auto *logoLabel = new QLabel;
    logoLabel->setPixmap(QPixmap(":/icons/nexusos-logo.png").scaled(64, 64, Qt::KeepAspectRatio, Qt::SmoothTransformation));
    auto *titleLabel = new QLabel("NexusOS Setup Assistant");
    titleLabel->setStyleSheet("font-size: 24px; font-weight: bold; color: #2196F3;");
    
    headerLayout->addWidget(logoLabel);
    headerLayout->addWidget(titleLabel);
    headerLayout->addStretch();
    
    m_statusLabel = new QLabel("Ready to configure your system");
    m_statusLabel->setStyleSheet("font-size: 12px; color: #666;");
    headerLayout->addWidget(m_statusLabel);
    
    mainLayout->addLayout(headerLayout);
    
    // Tab Widget
    m_tabWidget = new QTabWidget;
    
    // Media Stack Tab
    setupMediaStackTab();
    
    // Self-Hosting Tab
    setupSelfHostingTab();
    
    // Development Tab
    setupDevelopmentTab();
    
    // Gaming Tab
    setupGamingTab();
    
    // Security Tab (Base System)
    setupSecurityTab();
    
    // Package Installation Tab
    setupPackageTab();
    
    // System Info Tab
    setupSystemInfoTab();
    
    mainLayout->addWidget(m_tabWidget);
    
    // Progress and Actions
    auto *bottomLayout = new QVBoxLayout;
    
    m_progressBar = new QProgressBar;
    m_progressBar->setVisible(false);
    bottomLayout->addWidget(m_progressBar);
    
    auto *actionLayout = new QHBoxLayout;
    
    auto *installSelectedBtn = new QPushButton("🚀 Install Selected Components");
    installSelectedBtn->setStyleSheet("QPushButton { background-color: #4CAF50; color: white; font-weight: bold; padding: 10px; border: none; border-radius: 5px; }");
    connect(installSelectedBtn, &QPushButton::clicked, this, &NexusSetupAssistant::installSelected);
    
    auto *diagnosticsBtn = new QPushButton("🔍 System Diagnostics");
    connect(diagnosticsBtn, &QPushButton::clicked, this, &NexusSetupAssistant::runDiagnostics);
    
    auto *optimizeBtn = new QPushButton("⚡ Optimize System");
    connect(optimizeBtn, &QPushButton::clicked, this, &NexusSetupAssistant::systemOptimization);
    
    auto *aboutBtn = new QPushButton("ℹ️ About");
    connect(aboutBtn, &QPushButton::clicked, this, &NexusSetupAssistant::showAbout);
    
    actionLayout->addWidget(installSelectedBtn);
    actionLayout->addWidget(diagnosticsBtn);
    actionLayout->addWidget(optimizeBtn);
    actionLayout->addStretch();
    actionLayout->addWidget(aboutBtn);
    
    bottomLayout->addLayout(actionLayout);
    
    // Log Output
    m_logOutput = new QTextEdit;
    m_logOutput->setMaximumHeight(150);
    m_logOutput->setPlaceholderText("Installation log will appear here...");
    m_logOutput->setStyleSheet("background-color: #1e1e1e; color: #ffffff; font-family: 'Consolas', monospace;");
    bottomLayout->addWidget(new QLabel("Installation Log:"));
    bottomLayout->addWidget(m_logOutput);
    
    mainLayout->addLayout(bottomLayout);
}

void NexusSetupAssistant::setupMediaStackTab() {
    auto *mediaWidget = new QWidget;
    auto *mediaLayout = new QVBoxLayout(mediaWidget);
    
    // Header
    auto *headerLabel = new QLabel("🎬 Media Stack Components");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #FF5722; margin-bottom: 10px;");
    mediaLayout->addWidget(headerLabel);
    
    auto *descLabel = new QLabel("Complete media server ecosystem with automation and management tools.");
    descLabel->setStyleSheet("color: #666; margin-bottom: 15px;");
    mediaLayout->addWidget(descLabel);
    
    // Scroll area for components
    auto *scrollArea = new QScrollArea;
    auto *scrollWidget = new QWidget;
    auto *scrollLayout = new QGridLayout(scrollWidget);
    
    // Media Servers
    m_mediaGroup = new QGroupBox("Media Servers");
    auto *mediaGroupLayout = new QVBoxLayout(m_mediaGroup);
    
    m_plexCheck = new QCheckBox("🎭 Plex Media Server - Premium media experience");
    m_jellyfinCheck = new QCheckBox("🎪 Jellyfin - Open source media server");
    
    mediaGroupLayout->addWidget(m_plexCheck);
    mediaGroupLayout->addWidget(m_jellyfinCheck);
    
    // Automation Tools
    auto *autoGroup = new QGroupBox("Automation & Management");
    auto *autoLayout = new QVBoxLayout(autoGroup);
    
    m_sonarrCheck = new QCheckBox("📺 Sonarr - TV show automation");
    m_radarrCheck = new QCheckBox("🎬 Radarr - Movie automation");
    m_lidarrCheck = new QCheckBox("🎵 Lidarr - Music automation");
    m_readarrCheck = new QCheckBox("📚 Readarr - Book automation");
    m_prowlarrCheck = new QCheckBox("🔍 Prowlarr - Indexer manager");
    m_overseerrCheck = new QCheckBox("🎫 Overseerr - Request management");
    m_tautulliCheck = new QCheckBox("📊 Tautulli - Plex monitoring");
    
    autoLayout->addWidget(m_sonarrCheck);
    autoLayout->addWidget(m_radarrCheck);
    autoLayout->addWidget(m_lidarrCheck);
    autoLayout->addWidget(m_readarrCheck);
    autoLayout->addWidget(m_prowlarrCheck);
    autoLayout->addWidget(m_overseerrCheck);
    autoLayout->addWidget(m_tautulliCheck);
    
    // Download Clients
    auto *downloadGroup = new QGroupBox("Download Clients");
    auto *downloadLayout = new QVBoxLayout(downloadGroup);
    
    m_qbittorrentCheck = new QCheckBox("⬇️ qBittorrent - Torrent client with VPN support");
    downloadLayout->addWidget(m_qbittorrentCheck);
    
    scrollLayout->addWidget(m_mediaGroup, 0, 0);
    scrollLayout->addWidget(autoGroup, 0, 1);
    scrollLayout->addWidget(downloadGroup, 1, 0);
    
    scrollArea->setWidget(scrollWidget);
    scrollArea->setWidgetResizable(true);
    mediaLayout->addWidget(scrollArea);
    
    // Quick Setup Buttons
    auto *quickSetupLayout = new QHBoxLayout;
    auto *basicMediaBtn = new QPushButton("📦 Basic Media Stack");
    auto *fullMediaBtn = new QPushButton("🚀 Complete Media Stack");
    auto *customBtn = new QPushButton("⚙️ Custom Selection");
    
    basicMediaBtn->setToolTip("Plex + Sonarr + Radarr + qBittorrent");
    fullMediaBtn->setToolTip("All media components");
    
    connect(basicMediaBtn, &QPushButton::clicked, [this]() {
        m_plexCheck->setChecked(true);
        m_sonarrCheck->setChecked(true);
        m_radarrCheck->setChecked(true);
        m_qbittorrentCheck->setChecked(true);
    });
    
    connect(fullMediaBtn, &QPushButton::clicked, [this]() {
        m_plexCheck->setChecked(true);
        m_jellyfinCheck->setChecked(true);
        m_sonarrCheck->setChecked(true);
        m_radarrCheck->setChecked(true);
        m_lidarrCheck->setChecked(true);
        m_readarrCheck->setChecked(true);
        m_prowlarrCheck->setChecked(true);
        m_overseerrCheck->setChecked(true);
        m_tautulliCheck->setChecked(true);
        m_qbittorrentCheck->setChecked(true);
    });
    
    quickSetupLayout->addWidget(basicMediaBtn);
    quickSetupLayout->addWidget(fullMediaBtn);
    quickSetupLayout->addWidget(customBtn);
    quickSetupLayout->addStretch();
    
    mediaLayout->addLayout(quickSetupLayout);
    
    m_tabWidget->addTab(mediaWidget, "🎬 Media Stack");
}

void NexusSetupAssistant::setupSecurityTab() {
    auto *securityWidget = new QWidget;
    auto *securityLayout = new QVBoxLayout(securityWidget);
    
    // Header
    auto *headerLabel = new QLabel("🛡️ Base Security Components");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #f44336; margin-bottom: 10px;");
    securityLayout->addWidget(headerLabel);
    
    auto *descLabel = new QLabel("Essential security components installed as base system services.");
    descLabel->setStyleSheet("color: #666; margin-bottom: 15px;");
    securityLayout->addWidget(descLabel);
    
    // Digital Fortress Section
    auto *fortressGroup = new QGroupBox("🏰 Digital Fortress - Ultimate Online Invisibility");
    auto *fortressLayout = new QVBoxLayout(fortressGroup);
    
    auto *fortressDesc = new QLabel("Complete digital anonymity suite with browser fingerprinting protection, hardware spoofing, and network anonymization.");
    fortressDesc->setWordWrap(true);
    fortressDesc->setStyleSheet("color: #666; margin-bottom: 10px;");
    fortressLayout->addWidget(fortressDesc);
    
    auto *fortressFeatures = new QLabel(
        "• 🌐 Browser fingerprinting blocked (WebRTC, Canvas, WebGL)\n"
        "• 🔧 Hardware fingerprinting spoofed (CPU, GPU, RAM)\n"  
        "• 🕐 Time fingerprinting masked (Timezone, timing attacks)\n"
        "• 📡 Network anonymization (IPv6 disabled, DNS leak prevention)\n"
        "• 👁️ Continuous monitoring with auto-repair\n"
        "• 🎛️ System tray widget with status indicators"
    );
    fortressFeatures->setStyleSheet("background-color: #e8f5e8; padding: 10px; border-left: 4px solid #4caf50; margin: 10px 0;");
    fortressLayout->addWidget(fortressFeatures);
    
    auto *fortressControls = new QHBoxLayout;
    m_digitalFortressBtn = new QPushButton("🚀 Install & Enable Digital Fortress");
    m_digitalFortressBtn->setStyleSheet("QPushButton { background-color: #2196F3; color: white; font-weight: bold; padding: 10px; border: none; border-radius: 5px; }");
    
    m_fortressStatus = new QLabel("Status: Not Installed");
    m_fortressStatus->setStyleSheet("color: #666; margin-left: 10px;");
    
    auto *fortressToggle = new QPushButton("🔄 Toggle Ghost Mode");
    
    connect(m_digitalFortressBtn, &QPushButton::clicked, [this]() {
        updateProgress("Installing Digital Fortress...", 0);
        runCommand(QString("cd %1/ghost-mode && ./install-ghost-mode.sh").arg(m_awesomeStackPath));
        m_fortressEnabled = true;
        m_fortressStatus->setText("Status: ✅ Installed & Active");
        m_fortressStatus->setStyleSheet("color: green; font-weight: bold;");
    });
    
    connect(fortressToggle, &QPushButton::clicked, this, &NexusSetupAssistant::toggleDigitalFortress);
    
    fortressControls->addWidget(m_digitalFortressBtn);
    fortressControls->addWidget(fortressToggle);
    fortressControls->addWidget(m_fortressStatus);
    fortressControls->addStretch();
    
    fortressLayout->addLayout(fortressControls);
    
    // Vaultwarden Section  
    auto *vaultGroup = new QGroupBox("🔐 Vaultwarden - Self-Hosted Password Manager");
    auto *vaultLayout = new QVBoxLayout(vaultGroup);
    
    auto *vaultDesc = new QLabel("Bitwarden-compatible password manager with premium features for free.");
    vaultDesc->setStyleSheet("color: #666; margin-bottom: 10px;");
    vaultLayout->addWidget(vaultDesc);
    
    auto *vaultFeatures = new QLabel(
        "• 🔑 Password generation and storage\n"
        "• 🌐 Browser extensions for all browsers\n"
        "• 📱 Mobile apps synchronization\n" 
        "• 👥 Organization and sharing support\n"
        "• 🔒 End-to-end encryption\n"
        "• 📊 Security reports and breach monitoring"
    );
    vaultFeatures->setStyleSheet("background-color: #fff3e0; padding: 10px; border-left: 4px solid #ff9800; margin: 10px 0;");
    vaultLayout->addWidget(vaultFeatures);
    
    auto *vaultControls = new QHBoxLayout;
    m_vaultwardenBtn = new QPushButton("🚀 Install Vaultwarden");
    m_vaultwardenBtn->setStyleSheet("QPushButton { background-color: #ff9800; color: white; font-weight: bold; padding: 10px; border: none; border-radius: 5px; }");
    
    m_vaultwardenStatus = new QLabel("Status: Not Installed");
    m_vaultwardenStatus->setStyleSheet("color: #666; margin-left: 10px;");
    
    auto *vaultOpenBtn = new QPushButton("🌐 Open Vaultwarden");
    
    connect(m_vaultwardenBtn, &QPushButton::clicked, [this]() {
        updateProgress("Installing Vaultwarden...", 0);
        runCommand("systemctl --user enable --now vaultwarden.service");
        m_vaultwardenRunning = true;
        m_vaultwardenStatus->setText("Status: ✅ Running on http://localhost:8080");
        m_vaultwardenStatus->setStyleSheet("color: green; font-weight: bold;");
    });
    
    connect(vaultOpenBtn, &QPushButton::clicked, this, &NexusSetupAssistant::openVaultwarden);
    
    vaultControls->addWidget(m_vaultwardenBtn);
    vaultControls->addWidget(vaultOpenBtn);
    vaultControls->addWidget(m_vaultwardenStatus);
    vaultControls->addStretch();
    
    vaultLayout->addLayout(vaultControls);
    
    securityLayout->addWidget(fortressGroup);
    securityLayout->addWidget(vaultGroup);
    securityLayout->addStretch();
    
    m_tabWidget->addTab(securityWidget, "🛡️ Security Base");
}

void NexusSetupAssistant::setupPackageTab() {
    auto *packageWidget = new QWidget;
    auto *packageLayout = new QVBoxLayout(packageWidget);
    
    // Header
    auto *headerLabel = new QLabel("📦 Universal Package Installation");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #9c27b0; margin-bottom: 10px;");
    packageLayout->addWidget(headerLabel);
    
    auto *descLabel = new QLabel("Install packages from ANY Linux distribution using NexusPkg.");
    descLabel->setStyleSheet("color: #666; margin-bottom: 15px;");
    packageLayout->addWidget(descLabel);
    
    // Supported formats
    auto *formatsLabel = new QLabel(
        "Supported formats: 📦 DEB, 📦 RPM, 🗜️ ZST, 📱 AppImage, 📦 Flatpak, 📦 Snap, 🐍 Python, 📦 NPM, 🦀 Cargo, 📁 TAR, 🐳 Containers"
    );
    formatsLabel->setStyleSheet("background-color: #f0f7ff; padding: 10px; border-left: 4px solid #2196f3; margin: 10px 0;");
    packageLayout->addWidget(formatsLabel);
    
    // Installation interface
    auto *installGroup = new QGroupBox("Install Package");
    auto *installLayout = new QGridLayout(installGroup);
    
    installLayout->addWidget(new QLabel("Distribution/Format:"), 0, 0);
    m_distroCombo = new QComboBox;
    m_distroCombo->addItems({
        "Auto-detect",
        "Debian/Ubuntu (.deb)",
        "RedHat/Fedora (.rpm)", 
        "Arch Linux (.pkg.tar.zst)",
        "Alpine Linux (.apk)",
        "Flatpak",
        "Snap",
        "AppImage",
        "Python (pip)",
        "Node.js (npm)",
        "Rust (cargo)",
        "Docker Container",
        "Raw Binary"
    });
    installLayout->addWidget(m_distroCombo, 0, 1, 1, 2);
    
    installLayout->addWidget(new QLabel("Package Name/File:"), 1, 0);
    m_packageEdit = new QLineEdit;
    m_packageEdit->setPlaceholderText("e.g., firefox, ./package.deb, com.spotify.Client");
    installLayout->addWidget(m_packageEdit, 1, 1);
    
    m_installBtn = new QPushButton("🚀 Install");
    m_installBtn->setStyleSheet("QPushButton { background-color: #4caf50; color: white; font-weight: bold; padding: 8px 16px; border: none; border-radius: 4px; }");
    connect(m_installBtn, &QPushButton::clicked, this, &NexusSetupAssistant::installFromDistro);
    installLayout->addWidget(m_installBtn, 1, 2);
    
    packageLayout->addWidget(installGroup);
    
    // Quick install buttons
    auto *quickGroup = new QGroupBox("Quick Install Examples");
    auto *quickLayout = new QGridLayout(quickGroup);
    
    auto *firefoxBtn = new QPushButton("🦊 Firefox (Flatpak)");
    auto *chromeBtn = new QPushButton("🌐 Chrome (DEB)");
    auto *vscodeBtn = new QPushButton("💻 VS Code (DEB)");
    auto *discordBtn = new QPushButton("💬 Discord (Snap)");
    auto *steamBtn = new QPushButton("🎮 Steam (DEB)");
    auto *spotifyBtn = new QPushButton("🎵 Spotify (Flatpak)");
    
    connect(firefoxBtn, &QPushButton::clicked, [this]() {
        m_distroCombo->setCurrentText("Flatpak");
        m_packageEdit->setText("org.mozilla.firefox");
        installFromDistro();
    });
    
    connect(chromeBtn, &QPushButton::clicked, [this]() {
        runCommand("wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add - && echo 'deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main' | sudo tee /etc/apt/sources.list.d/google-chrome.list && sudo apt update && sudo apt install google-chrome-stable");
    });
    
    quickLayout->addWidget(firefoxBtn, 0, 0);
    quickLayout->addWidget(chromeBtn, 0, 1);
    quickLayout->addWidget(vscodeBtn, 0, 2);
    quickLayout->addWidget(discordBtn, 1, 0);
    quickLayout->addWidget(steamBtn, 1, 1);
    quickLayout->addWidget(spotifyBtn, 1, 2);
    
    packageLayout->addWidget(quickGroup);
    
    // Results
    m_packageResults = new QTextEdit;
    m_packageResults->setMaximumHeight(200);
    m_packageResults->setPlaceholderText("Installation results will appear here...");
    m_packageResults->setStyleSheet("background-color: #1e1e1e; color: #ffffff; font-family: 'Consolas', monospace;");
    packageLayout->addWidget(new QLabel("Installation Results:"));
    packageLayout->addWidget(m_packageResults);
    
    m_tabWidget->addTab(packageWidget, "📦 Install Packages");
}

void NexusSetupAssistant::installFromDistro() {
    QString format = m_distroCombo->currentText();
    QString package = m_packageEdit->text().trimmed();
    
    if (package.isEmpty()) {
        QMessageBox::warning(this, "Warning", "Please enter a package name or file path.");
        return;
    }
    
    QString command;
    
    if (format.contains("Auto-detect")) {
        command = QString("nexuspkg install %1").arg(package);
    } else if (format.contains("Flatpak")) {
        command = QString("nexuspkg flatpak %1").arg(package);
    } else if (format.contains("Snap")) {
        command = QString("nexuspkg snap %1").arg(package);
    } else if (format.contains(".deb")) {
        command = QString("nexuspkg deb install %1").arg(package);
    } else if (format.contains(".rpm")) {
        command = QString("nexuspkg rpm install %1").arg(package);
    } else if (format.contains(".pkg.tar.zst")) {
        command = QString("nexuspkg zst install %1").arg(package);
    } else if (format.contains("Python")) {
        command = QString("nexuspkg pip %1").arg(package);
    } else if (format.contains("Node.js")) {
        command = QString("nexuspkg npm %1").arg(package);
    } else if (format.contains("Rust")) {
        command = QString("nexuspkg cargo %1").arg(package);
    } else if (format.contains("AppImage")) {
        command = QString("nexuspkg appimage %1").arg(package);
    }
    
    if (!command.isEmpty()) {
        updateProgress(QString("Installing %1...").arg(package), 50);
        runCommand(command);
    }
}

void NexusSetupAssistant::runCommand(const QString &command, bool async) {
    m_logOutput->append(QString("$ %1").arg(command));
    m_logOutput->ensureCursorVisible();
    
    auto *process = new QProcess(this);
    
    if (async) {
        connect(process, &QProcess::readyReadStandardOutput, [this, process]() {
            QByteArray data = process->readAllStandardOutput();
            m_logOutput->append(QString::fromUtf8(data));
            m_logOutput->ensureCursorVisible();
        });
        
        connect(process, &QProcess::readyReadStandardError, [this, process]() {
            QByteArray data = process->readAllStandardError();
            m_logOutput->append(QString("<span style='color: red;'>%1</span>").arg(QString::fromUtf8(data)));
            m_logOutput->ensureCursorVisible();
        });
        
        connect(process, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
                [this, process](int exitCode, QProcess::ExitStatus exitStatus) {
            Q_UNUSED(exitStatus)
            if (exitCode == 0) {
                m_logOutput->append("<span style='color: green;'>✅ Command completed successfully</span>");
                showNotification("Success", "Installation completed successfully!");
            } else {
                m_logOutput->append(QString("<span style='color: red;'>❌ Command failed with exit code %1</span>").arg(exitCode));
                showNotification("Error", "Installation failed!");
            }
            m_progressBar->setVisible(false);
            updateProgress("Ready", 0);
            process->deleteLater();
        });
        
        m_progressBar->setVisible(true);
        m_progressBar->setRange(0, 0); // Indeterminate
    }
    
    process->start("/bin/bash", QStringList() << "-c" << command);
    
    if (!async) {
        process->waitForFinished();
        process->deleteLater();
    }
}

void NexusSetupAssistant::updateProgress(const QString &message, int value) {
    m_statusLabel->setText(message);
    if (value >= 0) {
        m_progressBar->setRange(0, 100);
        m_progressBar->setValue(value);
    }
}

void NexusSetupAssistant::showNotification(const QString &title, const QString &message) {
    if (m_trayIcon && QSystemTrayIcon::isSystemTrayAvailable()) {
        m_trayIcon->showMessage(title, message, QSystemTrayIcon::Information, 3000);
    }
}

void NexusSetupAssistant::setupTrayIcon() {
    if (!QSystemTrayIcon::isSystemTrayAvailable()) {
        return;
    }
    
    m_trayIcon = new QSystemTrayIcon(this);
    m_trayIcon->setIcon(QIcon(":/icons/nexusos-tray.png"));
    
    auto *trayMenu = new QMenu(this);
    auto *showAction = trayMenu->addAction("Show Setup Assistant");
    trayMenu->addSeparator();
    auto *fortressAction = trayMenu->addAction("Toggle Digital Fortress");
    auto *vaultAction = trayMenu->addAction("Open Vaultwarden");
    trayMenu->addSeparator();
    auto *quitAction = trayMenu->addAction("Quit");
    
    connect(showAction, &QAction::triggered, this, &QWidget::show);
    connect(fortressAction, &QAction::triggered, this, &NexusSetupAssistant::toggleDigitalFortress);
    connect(vaultAction, &QAction::triggered, this, &NexusSetupAssistant::openVaultwarden);
    connect(quitAction, &QAction::triggered, this, &QWidget::close);
    
    m_trayIcon->setContextMenu(trayMenu);
    m_trayIcon->show();
    
    connect(m_trayIcon, &QSystemTrayIcon::activated, [this](QSystemTrayIcon::ActivationReason reason) {
        if (reason == QSystemTrayIcon::DoubleClick) {
            show();
            raise();
            activateWindow();
        }
    });
}

void NexusSetupAssistant::toggleDigitalFortress() {
    if (!QDir(m_awesomeStackPath).exists()) {
        QMessageBox::warning(this, "Error", "Awesome Stack not found. Please ensure the repository is mounted.");
        return;
    }
    
    QString command = QString("cd %1/ghost-mode && ./scripts/ghost-toggle").arg(m_awesomeStackPath);
    runCommand(command);
    
    m_fortressEnabled = !m_fortressEnabled;
    if (m_fortressEnabled) {
        m_fortressStatus->setText("Status: 🟢 Ghost Mode ACTIVE");
        m_fortressStatus->setStyleSheet("color: green; font-weight: bold;");
        showNotification("Digital Fortress", "Ghost Mode activated - You are now invisible online!");
    } else {
        m_fortressStatus->setText("Status: 🔴 Ghost Mode INACTIVE");  
        m_fortressStatus->setStyleSheet("color: red; font-weight: bold;");
        showNotification("Digital Fortress", "Ghost Mode deactivated - Normal browsing restored.");
    }
}

void NexusSetupAssistant::openVaultwarden() {
    QProcess::startDetached("xdg-open", QStringList() << "http://localhost:8080");
}

// Missing function implementations
void NexusSetupAssistant::setupSelfHostingTab() {
    auto *hostingWidget = new QWidget;
    auto *hostingLayout = new QVBoxLayout(hostingWidget);
    
    auto *headerLabel = new QLabel("🏠 Self-Hosting Stack");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #4CAF50; margin-bottom: 10px;");
    hostingLayout->addWidget(headerLabel);
    
    m_hostingGroup = new QGroupBox("Self-Hosting Services");
    auto *hostingGroupLayout = new QVBoxLayout(m_hostingGroup);
    
    m_nextcloudCheck = new QCheckBox("☁️ Nextcloud - Personal cloud storage");
    m_homeAssistantCheck = new QCheckBox("🏠 Home Assistant - Smart home automation");
    m_grafanaCheck = new QCheckBox("📊 Grafana - Monitoring dashboards");
    m_prometheusCheck = new QCheckBox("📈 Prometheus - Metrics collection");
    m_portainerCheck = new QCheckBox("🐳 Portainer - Docker management");
    m_traefikCheck = new QCheckBox("🔀 Traefik - Reverse proxy");
    m_photoprismCheck = new QCheckBox("📸 PhotoPrism - Photo management");
    m_paperlessCheck = new QCheckBox("📄 Paperless - Document management");
    
    hostingGroupLayout->addWidget(m_nextcloudCheck);
    hostingGroupLayout->addWidget(m_homeAssistantCheck);
    hostingGroupLayout->addWidget(m_grafanaCheck);
    hostingGroupLayout->addWidget(m_prometheusCheck);
    hostingGroupLayout->addWidget(m_portainerCheck);
    hostingGroupLayout->addWidget(m_traefikCheck);
    hostingGroupLayout->addWidget(m_photoprismCheck);
    hostingGroupLayout->addWidget(m_paperlessCheck);
    
    hostingLayout->addWidget(m_hostingGroup);
    hostingLayout->addStretch();
    
    m_tabWidget->addTab(hostingWidget, "🏠 Self-Hosting");
}

void NexusSetupAssistant::setupDevelopmentTab() {
    auto *devWidget = new QWidget;
    auto *devLayout = new QVBoxLayout(devWidget);
    
    auto *headerLabel = new QLabel("💻 Development Stack");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #9C27B0; margin-bottom: 10px;");
    devLayout->addWidget(headerLabel);
    
    m_devGroup = new QGroupBox("Development Tools");
    auto *devGroupLayout = new QVBoxLayout(m_devGroup);
    
    m_gitlabCheck = new QCheckBox("🦊 GitLab - Git repository hosting");
    m_jenkinsCheck = new QCheckBox("🔧 Jenkins - CI/CD automation");
    m_codeServerCheck = new QCheckBox("💻 Code Server - VS Code in browser");
    m_aiCodingCheck = new QCheckBox("🤖 AI Coding Assistant - Code completion");
    m_dockerRegistryCheck = new QCheckBox("🐳 Docker Registry - Container images");
    m_redisCheck = new QCheckBox("🗃️ Redis - In-memory database");
    m_postgresCheck = new QCheckBox("🐘 PostgreSQL - Relational database");
    m_mongoCheck = new QCheckBox("🍃 MongoDB - Document database");
    
    devGroupLayout->addWidget(m_gitlabCheck);
    devGroupLayout->addWidget(m_jenkinsCheck);
    devGroupLayout->addWidget(m_codeServerCheck);
    devGroupLayout->addWidget(m_aiCodingCheck);
    devGroupLayout->addWidget(m_dockerRegistryCheck);
    devGroupLayout->addWidget(m_redisCheck);
    devGroupLayout->addWidget(m_postgresCheck);
    devGroupLayout->addWidget(m_mongoCheck);
    
    devLayout->addWidget(m_devGroup);
    devLayout->addStretch();
    
    m_tabWidget->addTab(devWidget, "💻 Development");
}

void NexusSetupAssistant::setupGamingTab() {
    auto *gamingWidget = new QWidget;
    auto *gamingLayout = new QVBoxLayout(gamingWidget);
    
    auto *headerLabel = new QLabel("🎮 Gaming Stack");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #FF9800; margin-bottom: 10px;");
    gamingLayout->addWidget(headerLabel);
    
    m_gamingGroup = new QGroupBox("Gaming Applications");
    auto *gamingGroupLayout = new QVBoxLayout(m_gamingGroup);
    
    m_steamCheck = new QCheckBox("🎮 Steam - Gaming platform");
    m_lutrisCheck = new QCheckBox("🍷 Lutris - Gaming launcher");
    m_gameStreamCheck = new QCheckBox("📡 Game Streaming - Remote gaming");
    m_retroArchCheck = new QCheckBox("👾 RetroArch - Retro gaming");
    m_minecraftCheck = new QCheckBox("⛏️ Minecraft Server - Block building game");
    
    gamingGroupLayout->addWidget(m_steamCheck);
    gamingGroupLayout->addWidget(m_lutrisCheck);
    gamingGroupLayout->addWidget(m_gameStreamCheck);
    gamingGroupLayout->addWidget(m_retroArchCheck);
    gamingGroupLayout->addWidget(m_minecraftCheck);
    
    gamingLayout->addWidget(m_gamingGroup);
    gamingLayout->addStretch();
    
    m_tabWidget->addTab(gamingWidget, "🎮 Gaming");
}

void NexusSetupAssistant::setupSystemInfoTab() {
    auto *infoWidget = new QWidget;
    auto *infoLayout = new QVBoxLayout(infoWidget);
    
    auto *headerLabel = new QLabel("ℹ️ System Information");
    headerLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #607D8B; margin-bottom: 10px;");
    infoLayout->addWidget(headerLabel);
    
    auto *infoText = new QTextEdit;
    infoText->setReadOnly(true);
    infoText->setPlainText("Loading system information...");
    infoLayout->addWidget(infoText);
    
    // Load system info
    QTimer::singleShot(1000, [infoText]() {
        QString info;
        info += "NexusOS System Information\n";
        info += "========================\n\n";
        
        QProcess process;
        process.start("uname", QStringList() << "-a");
        process.waitForFinished();
        info += "Kernel: " + process.readAllStandardOutput().trimmed() + "\n";
        
        process.start("lscpu");
        process.waitForFinished();
        QString cpuInfo = process.readAllStandardOutput();
        QStringList cpuLines = cpuInfo.split('\n');
        for (const QString &line : cpuLines) {
            if (line.contains("Model name:")) {
                info += "CPU: " + line.split(":").value(1).trimmed() + "\n";
                break;
            }
        }
        
        process.start("free", QStringList() << "-h");
        process.waitForFinished();
        QString memInfo = process.readAllStandardOutput();
        QStringList memLines = memInfo.split('\n');
        if (memLines.size() > 1) {
            info += "Memory: " + memLines[1].split(QRegExp("\\s+")).value(1) + "\n";
        }
        
        process.start("df", QStringList() << "-h" << "/");
        process.waitForFinished();
        QString diskInfo = process.readAllStandardOutput();
        QStringList diskLines = diskInfo.split('\n');
        if (diskLines.size() > 1) {
            QStringList parts = diskLines[1].split(QRegExp("\\s+"));
            if (parts.size() >= 4) {
                info += "Disk: " + parts[3] + " available / " + parts[1] + " total\n";
            }
        }
        
        info += "\nNexusPkg Status:\n";
        process.start("nexuspkg", QStringList() << "status");
        if (process.waitForFinished()) {
            info += process.readAllStandardOutput();
        } else {
            info += "NexusPkg not installed\n";
        }
        
        infoText->setPlainText(info);
    });
    
    m_tabWidget->addTab(infoWidget, "ℹ️ System Info");
}

void NexusSetupAssistant::setupMediaStack() {
    runCommand(QString("cd %1 && ./setup-complete-firetv-stack.sh").arg(m_awesomeStackPath));
}

void NexusSetupAssistant::setupSelfHosting() {
    runCommand(QString("cd %1 && docker-compose up -d nextcloud homeassistant grafana").arg(m_awesomeStackPath));
}

void NexusSetupAssistant::setupDevelopment() {
    runCommand("nexuspkg install code gitlab-ce docker");
}

void NexusSetupAssistant::setupGaming() {
    runCommand("nexuspkg flatpak com.valvesoftware.Steam");
    runCommand("nexuspkg install lutris");
}

void NexusSetupAssistant::setupSecurity() {
    // Security setup is handled in setupSecurityTab
}

void NexusSetupAssistant::installCustomPackages() {
    // This is handled in the package tab
}

void NexusSetupAssistant::systemOptimization() {
    runCommand(QString("cd %1 && ./hardware_optimization_vm.sh").arg(m_awesomeStackPath));
}

void NexusSetupAssistant::showAbout() {
    QMessageBox::about(this, "About NexusOS Setup Assistant",
        "<h3>NexusOS Setup Assistant v1.0</h3>"
        "<p>Universal Linux distribution with native package compatibility.</p>"
        "<p><b>Features:</b></p>"
        "<ul>"
        "<li>📦 Install packages from ANY Linux distribution</li>"
        "<li>🎬 Complete media stack integration</li>"
        "<li>🛡️ Digital Fortress security suite</li>"
        "<li>🔐 Vaultwarden password manager</li>"
        "<li>🏠 Self-hosting infrastructure</li>"
        "</ul>"
        "<p>Built with Qt and powered by awesome-stack.</p>");
}

void NexusSetupAssistant::runDiagnostics() {
    updateProgress("Running system diagnostics...", 0);
    runCommand("nexuspkg status && systemctl --user status vaultwarden && docker ps");
}

void NexusSetupAssistant::showSystemInfo() {
    m_tabWidget->setCurrentIndex(m_tabWidget->count() - 1); // Show system info tab
}

void NexusSetupAssistant::installSelected() {
    QStringList commands;
    
    // Media Stack
    if (m_plexCheck->isChecked()) commands << "docker run -d --name plex -p 32400:32400 plexinc/pms-docker";
    if (m_jellyfinCheck->isChecked()) commands << "docker run -d --name jellyfin -p 8096:8096 jellyfin/jellyfin";
    if (m_sonarrCheck->isChecked()) commands << "docker run -d --name sonarr -p 8989:8989 linuxserver/sonarr";
    if (m_radarrCheck->isChecked()) commands << "docker run -d --name radarr -p 7878:7878 linuxserver/radarr";
    if (m_qbittorrentCheck->isChecked()) commands << "docker run -d --name qbittorrent -p 8080:8080 linuxserver/qbittorrent";
    
    // Self-Hosting
    if (m_nextcloudCheck->isChecked()) commands << "docker run -d --name nextcloud -p 8080:80 nextcloud";
    if (m_homeAssistantCheck->isChecked()) commands << "docker run -d --name homeassistant -p 8123:8123 homeassistant/home-assistant";
    
    // Development
    if (m_codeServerCheck->isChecked()) commands << "docker run -d --name code-server -p 8080:8080 codercom/code-server";
    
    // Gaming
    if (m_steamCheck->isChecked()) commands << "nexuspkg flatpak com.valvesoftware.Steam";
    
    if (commands.isEmpty()) {
        QMessageBox::information(this, "No Selection", "Please select components to install.");
        return;
    }
    
    updateProgress("Installing selected components...", 0);
    for (const QString &cmd : commands) {
        runCommand(cmd);
    }
}

void NexusSetupAssistant::loadConfig() {
    // Load saved settings
    m_fortressEnabled = m_settings->value("fortress_enabled", false).toBool();
    m_vaultwardenRunning = m_settings->value("vaultwarden_running", false).toBool();
    
    // Update UI based on loaded config
    if (m_fortressEnabled) {
        m_fortressStatus->setText("Status: ✅ Installed & Active");
        m_fortressStatus->setStyleSheet("color: green; font-weight: bold;");
    }
    
    if (m_vaultwardenRunning) {
        m_vaultwardenStatus->setText("Status: ✅ Running on http://localhost:8080");
        m_vaultwardenStatus->setStyleSheet("color: green; font-weight: bold;");
    }
}

void NexusSetupAssistant::saveConfig() {
    // Save current settings
    m_settings->setValue("fortress_enabled", m_fortressEnabled);
    m_settings->setValue("vaultwarden_running", m_vaultwardenRunning);
    m_settings->sync();
}

// Include the moc file for Qt's meta-object system
#include "main.moc"

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName("NexusOS Setup Assistant");
    app.setApplicationVersion("1.0.0");
    app.setOrganizationName("NexusOS");
    
    // Apply dark theme
    app.setStyleSheet(R"(
        QMainWindow {
            background-color: #2b2b2b;
            color: #ffffff;
        }
        QTabWidget::pane {
            border: 1px solid #555555;
            background-color: #3c3c3c;
        }
        QTabBar::tab {
            background-color: #555555;
            color: #ffffff;
            padding: 8px 12px;
            margin-right: 2px;
            border-top-left-radius: 4px;
            border-top-right-radius: 4px;
        }
        QTabBar::tab:selected {
            background-color: #2196F3;
        }
        QGroupBox {
            font-weight: bold;
            border: 2px solid #555555;
            border-radius: 8px;
            margin: 10px 0px;
            padding-top: 10px;
        }
        QGroupBox::title {
            subcontrol-origin: margin;
            left: 10px;
            padding: 0 5px 0 5px;
        }
        QCheckBox {
            spacing: 8px;
            margin: 4px;
        }
        QCheckBox::indicator {
            width: 18px;
            height: 18px;
            border-radius: 2px;
            border: 1px solid #555555;
        }
        QCheckBox::indicator:checked {
            background-color: #2196F3;
            border: 1px solid #2196F3;
        }
        QPushButton {
            background-color: #555555;
            color: #ffffff;
            border: none;
            padding: 8px 12px;
            border-radius: 4px;
            font-weight: bold;
        }
        QPushButton:hover {
            background-color: #666666;
        }
        QPushButton:pressed {
            background-color: #444444;
        }
    )");
    
    NexusSetupAssistant assistant;
    assistant.show();
    
    return app.exec();
}