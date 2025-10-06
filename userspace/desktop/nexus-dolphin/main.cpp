#include <QApplication>
#include <QCommandLineParser>
#include <QDir>
#include <QUrl>
#include <QStandardPaths>
#include <QStyleFactory>
#include <QDBusConnection>
#include <QDBusMessage>
#include <QLoggingCategory>
#include <QIcon>
#include <QSystemTrayIcon>

#include <KAboutData>
#include <KLocalizedString>
#include <KCrash>
#include <KDBusService>
#include <KConfigGroup>
#include <KSharedConfig>

#include "nexusdolphinmainwindow.h"
#include "nexusdolphinapplication.h"
#include "global.h"

Q_LOGGING_CATEGORY(DolphinDebug, "nexus.dolphin")

int main(int argc, char **argv)
{
    // Initialize Qt Application
    QApplication app(argc, argv);
    app.setWindowIcon(QIcon::fromTheme(QStringLiteral("folder")));
    
    // Set up KDE About Data
    KAboutData aboutData(QStringLiteral("nexus-dolphin"),
                        i18n("NexusDolphin"),
                        QStringLiteral("1.0.0"),
                        i18n("NexusOS File Manager based on Dolphin"),
                        KAboutLicense::GPL_V2,
                        i18n("Copyright 2024 NexusOS Project"));
    
    aboutData.addAuthor(i18n("NexusOS Team"),
                       i18n("Developer"),
                       QStringLiteral("nexusos@example.com"));
    
    aboutData.setDesktopFileName(QStringLiteral("org.nexusos.dolphin"));
    aboutData.setProgramLogo(QIcon::fromTheme(QStringLiteral("folder")).pixmap(48, 48));
    
    KAboutData::setApplicationData(aboutData);
    
    // Initialize KCrash for better crash reporting
    KCrash::initialize();
    
    // Parse command line arguments
    QCommandLineParser parser;
    parser.addHelpOption();
    parser.addVersionOption();
    
    parser.addOption(QCommandLineOption(QStringList() << QStringLiteral("select"),
                                       i18n("The file or folder passed as argument will be selected."),
                                       QStringLiteral("file")));
    
    parser.addOption(QCommandLineOption(QStringList() << QStringLiteral("split"),
                                       i18n("NexusDolphin will get started with a split view.")));
    
    parser.addOption(QCommandLineOption(QStringList() << QStringLiteral("new-window"),
                                       i18n("NexusDolphin will get started with a new window.")));
    
    parser.addOption(QCommandLineOption(QStringList() << QStringLiteral("daemon"),
                                       i18n("Start as system daemon.")));
    
    parser.addPositionalArgument(QStringLiteral("urls"), 
                                i18n("Document(s) to open."), 
                                QStringLiteral("[urls...]"));
    
    aboutData.setupCommandLine(&parser);
    parser.process(app);
    aboutData.processCommandLine(&parser);
    
    // Prevent multiple instances
    KDBusService dbusService(KDBusService::Unique);
    
    // Create application instance
    NexusDolphinApplication nexusApp;
    
    // Handle daemon mode
    if (parser.isSet(QStringLiteral("daemon"))) {
        qCDebug(DolphinDebug) << "Starting NexusDolphin in daemon mode";
        
        // Create system tray icon
        if (QSystemTrayIcon::isSystemTrayAvailable()) {
            QSystemTrayIcon *trayIcon = new QSystemTrayIcon(&app);
            trayIcon->setIcon(QIcon::fromTheme(QStringLiteral("folder")));
            trayIcon->setToolTip(i18n("NexusDolphin File Manager"));
            trayIcon->show();
            
            // Connect to show window on tray icon click
            QObject::connect(trayIcon, &QSystemTrayIcon::activated, [&](QSystemTrayIcon::ActivationReason reason) {
                if (reason == QSystemTrayIcon::Trigger) {
                    nexusApp.createMainWindow();
                }
            });
        }
        
        return app.exec();
    }
    
    // Parse URLs from command line
    QList<QUrl> urls;
    const QStringList args = parser.positionalArguments();
    
    if (args.isEmpty()) {
        // No arguments - open home directory
        urls.append(QUrl::fromLocalFile(QDir::homePath()));
    } else {
        // Convert arguments to URLs
        for (const QString &arg : args) {
            QUrl url = QUrl::fromUserInput(arg, QDir::currentPath(), QUrl::AssumeLocalFile);
            if (url.isValid()) {
                urls.append(url);
            } else {
                qCWarning(DolphinDebug) << "Invalid URL:" << arg;
            }
        }
    }
    
    // Handle file selection
    QString selectFile;
    if (parser.isSet(QStringLiteral("select"))) {
        selectFile = parser.value(QStringLiteral("select"));
    }
    
    // Create main window
    NexusDolphinMainWindow *mainWindow = nexusApp.createMainWindow();
    
    if (!urls.isEmpty()) {
        if (parser.isSet(QStringLiteral("split")) && urls.count() > 1) {
            // Split view with multiple URLs
            mainWindow->openSplitView(urls.at(0));
            if (urls.count() > 1) {
                mainWindow->setRightViewUrl(urls.at(1));
            }
        } else {
            // Single view
            mainWindow->openDirectories(urls, parser.isSet(QStringLiteral("new-window")));
        }
    }
    
    // Select specific file if requested
    if (!selectFile.isEmpty()) {
        mainWindow->selectFile(selectFile);
    }
    
    mainWindow->show();
    
    // Handle D-Bus messages for opening new windows/tabs
    QObject::connect(&dbusService, &KDBusService::activateRequested, 
                    [&](const QStringList &arguments, const QString &workingDirectory) {
        Q_UNUSED(workingDirectory)
        
        if (!arguments.isEmpty()) {
            QList<QUrl> newUrls;
            for (const QString &arg : arguments) {
                if (!arg.startsWith('-')) {  // Skip options
                    QUrl url = QUrl::fromUserInput(arg, QDir::currentPath(), QUrl::AssumeLocalFile);
                    if (url.isValid()) {
                        newUrls.append(url);
                    }
                }
            }
            
            if (!newUrls.isEmpty()) {
                mainWindow->openDirectories(newUrls, false);
            }
        }
        
        mainWindow->activateWindow();
        mainWindow->raise();
    });
    
    qCDebug(DolphinDebug) << "NexusDolphin started successfully";
    
    return app.exec();
}