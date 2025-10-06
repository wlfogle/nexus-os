#ifndef NEXUSDOLPHINMAINWINDOW_H
#define NEXUSDOLPHINMAINWINDOW_H

#include <QPointer>
#include <QUrl>
#include <QList>

#include <KXmlGuiWindow>
#include <KFileItem>
#include <KParts/ReadOnlyPart>

class QSplitter;
class QVBoxLayout;
class QMenuBar;
class QStatusBar;
class QLabel;
class QToolButton;
class QProgressBar;

class KActionCollection;
class KToggleAction;
class KToolBar;
class KUrlNavigator;
class KFileItemModel;
class KDirModel;
class KNewFileMenu;

class NexusDolphinView;
class NexusDolphinTabWidget;
class NexusDolphinContextMenu;
class NexusDolphinApplication;
class NexusSearchBox;
class NexusFilterBar;
class NexusPlacesPanel;
class NexusInformationPanel;
class NexusTerminalPanel;

/**
 * @brief Main window class for NexusDolphin file manager
 *
 * Based on the original Dolphin architecture but adapted for NexusOS
 * with additional AI and system integration features.
 */
class NexusDolphinMainWindow : public KXmlGuiWindow
{
    Q_OBJECT

public:
    explicit NexusDolphinMainWindow();
    ~NexusDolphinMainWindow() override;

    /**
     * Opens the directories specified by @p urls in tabs.
     * @param urls       List of URLs to open
     * @param splitView  If true, opens URLs in split view mode
     */
    void openDirectories(const QList<QUrl> &urls, bool splitView);

    /**
     * Opens the URL in a new tab.
     */
    void openNewTab(const QUrl &url);

    /**
     * Opens split view with the given URL
     */
    void openSplitView(const QUrl &url);

    /**
     * Sets the URL for the right view in split mode
     */
    void setRightViewUrl(const QUrl &url);

    /**
     * Selects the file specified by @p file
     */
    void selectFile(const QString &file);

    /**
     * @return Currently active view
     */
    NexusDolphinView *activeView() const;

    /**
     * @return Currently inactive view (only in split view mode)
     */
    NexusDolphinView *inactiveView() const;

    /**
     * @return True if split view is enabled
     */
    bool isSplitViewEnabled() const;

Q_SIGNALS:
    /**
     * Emitted when the active view has been changed
     */
    void activeViewChanged(NexusDolphinView *view);

    /**
     * Emitted when the URL of a view has changed
     */
    void urlChanged(const QUrl &url);

private Q_SLOTS:
    /**
     * Updates the state of all actions based on the current selection and location
     */
    void updateViewActions();

    /**
     * Updates the window title based on the current location
     */
    void updateWindowTitle();

    /**
     * Creates a new folder in the current location
     */
    void createFolder();

    /**
     * Shows the search bar
     */
    void showSearch();

    /**
     * Toggles the split view
     */
    void toggleSplitView();

    /**
     * Toggles the places panel visibility
     */
    void togglePlacesPanel();

    /**
     * Toggles the information panel visibility
     */
    void toggleInformationPanel();

    /**
     * Toggles the terminal panel visibility
     */
    void toggleTerminalPanel();

    /**
     * Opens the preferences dialog
     */
    void openPreferences();

    /**
     * Shows the about dialog
     */
    void showAbout();

    /**
     * Handles view activation (when clicking between split views)
     */
    void setActiveView(NexusDolphinView *view);

    /**
     * Handles URL changes from the active view
     */
    void activeViewUrlChanged(const QUrl &url);

    /**
     * Updates the status bar information
     */
    void updateStatusBar();

    /**
     * Handles selection changes in the active view
     */
    void selectionChanged(const KFileItemList &selection);

    /**
     * Opens the context menu for the given items
     */
    void openContextMenu(const QPoint &pos, const KFileItem &item, const QList<QUrl> &customActions);

    /**
     * Handles the back navigation action
     */
    void goBack();

    /**
     * Handles the forward navigation action
     */
    void goForward();

    /**
     * Handles the up navigation action
     */
    void goUp();

    /**
     * Handles the home navigation action
     */
    void goHome();

    /**
     * Refreshes the current view
     */
    void refresh();

    /**
     * Stops any ongoing operations
     */
    void stopLoading();

    /**
     * Shows or hides the filter bar
     */
    void showFilterBar();

    /**
     * Handles cut operation
     */
    void cut();

    /**
     * Handles copy operation
     */
    void copy();

    /**
     * Handles paste operation
     */
    void paste();

    /**
     * Handles delete/move to trash operation
     */
    void moveToTrash();

    /**
     * Handles permanent delete operation
     */
    void deleteItems();

    /**
     * Handles rename operation
     */
    void rename();

    /**
     * Handles select all operation
     */
    void selectAll();

    /**
     * Handles invert selection operation
     */
    void invertSelection();

    /**
     * Handles properties dialog
     */
    void showProperties();

    /**
     * Changes the view mode (icons, list, details, etc.)
     */
    void setViewMode(int mode);

    /**
     * Opens terminal in current location
     */
    void openTerminalHere();

    /**
     * Opens the current location in an editor
     */
    void openInEditor();

protected:
    /**
     * @see QWidget::closeEvent()
     */
    void closeEvent(QCloseEvent *event) override;

    /**
     * @see KMainWindow::queryClose()
     */
    bool queryClose() override;

private:
    /**
     * Sets up the initial GUI elements
     */
    void setupGUI();

    /**
     * Creates all actions for menus and toolbars
     */
    void setupActions();

    /**
     * Creates the dock widgets (panels)
     */
    void setupDockWidgets();

    /**
     * Creates the central widget with views
     */
    void setupCentralWidget();

    /**
     * Creates a new view widget
     */
    NexusDolphinView *createView(const QUrl &url);

    /**
     * Connects signals for a view
     */
    void connectView(NexusDolphinView *view);

    /**
     * Updates the view container based on split view state
     */
    void updateViewContainer();

    /**
     * Saves the current window state
     */
    void saveSettings();

    /**
     * Restores the previous window state
     */
    void restoreSettings();

    /**
     * @return True if the active view contains writable items
     */
    bool isWritable() const;

    /**
     * @return True if there are items in the clipboard
     */
    bool hasClipboardContent() const;

private:
    // Central widget and views
    QWidget *m_centralWidget;
    QSplitter *m_splitter;
    NexusDolphinView *m_activeView;
    NexusDolphinView *m_inactiveView;
    
    // Tab support
    NexusDolphinTabWidget *m_tabWidget;
    
    // Panels
    NexusPlacesPanel *m_placesPanel;
    NexusInformationPanel *m_informationPanel;
    NexusTerminalPanel *m_terminalPanel;
    
    // Search and filter
    NexusSearchBox *m_searchBox;
    NexusFilterBar *m_filterBar;
    
    // Navigation
    KUrlNavigator *m_urlNavigator;
    
    // Actions
    KActionCollection *m_actionCollection;
    KToggleAction *m_splitViewAction;
    KToggleAction *m_showPlacesPanelAction;
    KToggleAction *m_showInformationPanelAction;
    KToggleAction *m_showTerminalPanelAction;
    KToggleAction *m_showFilterBarAction;
    
    // Context menu
    KNewFileMenu *m_newFileMenu;
    
    // Status bar
    QLabel *m_statusBarLabel;
    QProgressBar *m_progressBar;
    
    // State
    bool m_isSplitViewEnabled;
    QUrl m_lastUrl;
    
    // Application reference
    NexusDolphinApplication *m_application;
};

#endif // NEXUSDOLPHINMAINWINDOW_H