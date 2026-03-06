// ═══════════════════════════════════════════════════════════════════════════
// NexusOS KDE Plasma Desktop Layout Script
// Run: plasma-apply-wallpaperimage, then qdbus ... evaluateScript < this file
// Or: plasma-apply-desktoptheme; then use this via Plasma Scripting Console
// ═══════════════════════════════════════════════════════════════════════════

// Remove existing panels
var panels = panels();
for (var i = 0; i < panels.length; i++) {
    panels[i].remove();
}

// ── Bottom Panel ────────────────────────────────────────────────────────
var panel = new Panel;
panel.location = "bottom";
panel.height = 44;
panel.alignment = "center";

// Application Launcher
var launcher = panel.addWidget("org.kde.plasma.kickoff");
launcher.currentConfigGroup = ["General"];
launcher.writeConfig("icon", "start-here-kde");
launcher.writeConfig("favoritesDisplay", 0);
launcher.writeConfig("applicationsDisplay", 0);

// Task Manager
var taskManager = panel.addWidget("org.kde.plasma.icontasks");
taskManager.currentConfigGroup = ["General"];
taskManager.writeConfig("launchers", [
    "applications:systemsettings.desktop",
    "applications:org.kde.dolphin.desktop",
    "applications:org.kde.konsole.desktop",
    "applications:firefox.desktop",
    "applications:steam.desktop"
]);
taskManager.writeConfig("groupingStrategy", 1);
taskManager.writeConfig("showTooltips", true);
taskManager.writeConfig("indicateAudioStreams", true);

// Spacer
panel.addWidget("org.kde.plasma.panelspacer");

// System Tray
var systray = panel.addWidget("org.kde.plasma.systemtray");

// Digital Clock
var clock = panel.addWidget("org.kde.plasma.digitalclock");
clock.currentConfigGroup = ["Appearance"];
clock.writeConfig("showDate", true);
clock.writeConfig("dateFormat", "shortDate");
clock.writeConfig("use24hFormat", 2);

// Show Desktop button
panel.addWidget("org.kde.plasma.showdesktop");

// ── Desktop Settings ────────────────────────────────────────────────────
var desktops = desktops();
for (var d = 0; d < desktops.length; d++) {
    var desktop = desktops[d];
    desktop.wallpaperPlugin = "org.kde.image";
    desktop.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];
    desktop.writeConfig("Image", "/usr/share/wallpapers/NexusOS/contents/images/3840x2160.png");
    desktop.writeConfig("FillMode", 2);
}
