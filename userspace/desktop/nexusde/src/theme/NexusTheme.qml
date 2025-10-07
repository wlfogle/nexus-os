/**
 * NexusDE Theme Engine
 * Advanced theming system with dynamic color palettes, animations, and branding
 */

pragma Singleton
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtGraphicalEffects 1.15

Item {
    id: nexusTheme

    // Theme properties
    property bool isDarkTheme: true
    property real animationDuration: 250
    property real longAnimationDuration: 400
    property real shortAnimationDuration: 150
    
    // System state for dynamic theming
    property bool stellaActive: false
    property bool maxJrActive: false
    property bool securityActive: false
    property bool aiOptimizationActive: false
    property real systemLoad: 0.0
    
    // Brand colors - NexusOS signature palette
    readonly property color nexusBlue: "#1E3A8A"        // Primary brand blue
    readonly property color nexusTeal: "#0D9488"        // Secondary teal
    readonly property color nexusPurple: "#7C3AED"      // Accent purple
    readonly property color nexusOrange: "#EA580C"      // Warning/alert orange
    readonly property color nexusGreen: "#059669"       // Success green
    readonly property color nexusRed: "#DC2626"         // Error red
    readonly property color nexusGold: "#D97706"        // Premium gold
    
    // Mascot colors
    readonly property color stellaBlue: "#3B82F6"       // Stella's signature blue
    readonly property color maxJrOrange: "#F97316"      // Max Jr's signature orange
    readonly property color stellaGlow: "#60A5FA"       // Stella's glow effect
    readonly property color maxJrGlow: "#FB923C"        // Max Jr's glow effect
    
    // Dynamic color palette based on theme mode
    readonly property QtObject colors: QtObject {
        // Base colors
        readonly property color background: isDarkTheme ? "#0A0A0B" : "#FFFFFF"
        readonly property color surface: isDarkTheme ? "#1A1B1E" : "#F8F9FA"
        readonly property color surfaceVariant: isDarkTheme ? "#2A2B2E" : "#F1F3F4"
        readonly property color outline: isDarkTheme ? "#3A3B3E" : "#E0E0E0"
        
        // Content colors
        readonly property color onBackground: isDarkTheme ? "#FFFFFF" : "#1A1B1E"
        readonly property color onSurface: isDarkTheme ? "#E4E4E7" : "#27272A"
        readonly property color onSurfaceVariant: isDarkTheme ? "#A1A1AA" : "#71717A"
        readonly property color onOutline: isDarkTheme ? "#71717A" : "#A1A1AA"
        
        // Primary colors
        readonly property color primary: nexusBlue
        readonly property color primaryVariant: isDarkTheme ? "#2563EB" : "#1D4ED8"
        readonly property color onPrimary: "#FFFFFF"
        
        // Secondary colors  
        readonly property color secondary: nexusTeal
        readonly property color secondaryVariant: isDarkTheme ? "#14B8A6" : "#0F766E"
        readonly property color onSecondary: "#FFFFFF"
        
        // Accent colors
        readonly property color accent: nexusPurple
        readonly property color accentVariant: isDarkTheme ? "#8B5CF6" : "#6D28D9"
        readonly property color onAccent: "#FFFFFF"
        
        // State-based colors
        readonly property color success: nexusGreen
        readonly property color warning: nexusOrange  
        readonly property color error: nexusRed
        readonly property color info: nexusBlue
        
        // Interactive states
        readonly property color hover: isDarkTheme ? Qt.rgba(1,1,1,0.08) : Qt.rgba(0,0,0,0.05)
        readonly property color pressed: isDarkTheme ? Qt.rgba(1,1,1,0.12) : Qt.rgba(0,0,0,0.08)
        readonly property color focus: primaryVariant
        readonly property color disabled: isDarkTheme ? Qt.rgba(1,1,1,0.38) : Qt.rgba(0,0,0,0.38)
        
        // Panel and chrome colors
        readonly property color panelBackground: isDarkTheme ? Qt.rgba(0.1,0.1,0.11,0.95) : Qt.rgba(1,1,1,0.95)
        readonly property color panelBorder: isDarkTheme ? Qt.rgba(1,1,1,0.1) : Qt.rgba(0,0,0,0.1)
        readonly property color windowChrome: isDarkTheme ? "#1E1E20" : "#F5F5F5"
        
        // Dynamic mascot-influenced colors
        readonly property color stellaHighlight: stellaActive ? stellaBlue : "transparent"
        readonly property color maxJrHighlight: maxJrActive ? maxJrOrange : "transparent" 
        readonly property color securityHighlight: securityActive ? error : "transparent"
        readonly property color aiHighlight: aiOptimizationActive ? accent : "transparent"
    }
    
    // Typography system
    readonly property QtObject typography: QtObject {
        // Font families
        readonly property string primaryFont: "Inter"
        readonly property string monoFont: "JetBrains Mono"
        readonly property string displayFont: "Inter Display"
        
        // Font sizes (scaled with system)
        readonly property int displayLarge: 57
        readonly property int displayMedium: 45  
        readonly property int displaySmall: 36
        readonly property int headlineLarge: 32
        readonly property int headlineMedium: 28
        readonly property int headlineSmall: 24
        readonly property int titleLarge: 22
        readonly property int titleMedium: 16
        readonly property int titleSmall: 14
        readonly property int bodyLarge: 16
        readonly property int bodyMedium: 14
        readonly property int bodySmall: 12
        readonly property int labelLarge: 14
        readonly property int labelMedium: 12
        readonly property int labelSmall: 11
        
        // Font weights
        readonly property int thin: 100
        readonly property int extraLight: 200
        readonly property int light: 300
        readonly property int regular: 400
        readonly property int medium: 500
        readonly property int semiBold: 600
        readonly property int bold: 700
        readonly property int extraBold: 800
        readonly property int black: 900
    }
    
    // Spacing and sizing system
    readonly property QtObject spacing: QtObject {
        readonly property int xs: 4
        readonly property int sm: 8
        readonly property int md: 16
        readonly property int lg: 24
        readonly property int xl: 32
        readonly property int xxl: 48
        readonly property int xxxl: 64
        
        // Component-specific spacing
        readonly property int panelPadding: 12
        readonly property int buttonPadding: 16
        readonly property int cardPadding: 20
        readonly property int dialogPadding: 24
    }
    
    // Border radius system
    readonly property QtObject radius: QtObject {
        readonly property int none: 0
        readonly property int sm: 4
        readonly property int md: 8
        readonly property int lg: 12
        readonly property int xl: 16
        readonly property int full: 9999
        
        // Component-specific radius
        readonly property int button: 8
        readonly property int card: 12
        readonly property int dialog: 16
        readonly property int panel: 10
    }
    
    // Shadow system
    readonly property QtObject shadows: QtObject {
        // Elevation levels
        readonly property var level1: {
            "color": Qt.rgba(0,0,0, isDarkTheme ? 0.3 : 0.1),
            "offsetX": 0, "offsetY": 1, "blur": 3, "spread": 0
        }
        readonly property var level2: {
            "color": Qt.rgba(0,0,0, isDarkTheme ? 0.4 : 0.15),
            "offsetX": 0, "offsetY": 2, "blur": 6, "spread": 0
        }
        readonly property var level3: {
            "color": Qt.rgba(0,0,0, isDarkTheme ? 0.5 : 0.2),
            "offsetX": 0, "offsetY": 4, "blur": 12, "spread": 0
        }
        readonly property var level4: {
            "color": Qt.rgba(0,0,0, isDarkTheme ? 0.6 : 0.25),
            "offsetX": 0, "offsetY": 8, "blur": 24, "spread": 0
        }
        readonly property var level5: {
            "color": Qt.rgba(0,0,0, isDarkTheme ? 0.7 : 0.3),
            "offsetX": 0, "offsetY": 16, "blur": 48, "spread": 0
        }
    }
    
    // Animation easing curves
    readonly property QtObject easing: QtObject {
        readonly property var standard: Easing.OutCubic
        readonly property var decelerate: Easing.OutQuart
        readonly property var accelerate: Easing.InQuart
        readonly property var sharp: Easing.OutBack
        readonly property var bounce: Easing.OutBounce
        readonly property var elastic: Easing.OutElastic
    }
    
    // Icon system
    readonly property QtObject icons: QtObject {
        readonly property string stellaIcon: "/assets/icons/stella-icon.svg"
        readonly property string maxJrIcon: "/assets/icons/maxjr-icon.svg"
        readonly property string nexusLogo: "/assets/icons/nexus-logo.svg"
        readonly property string securityShield: "/assets/icons/security-shield.svg"
        readonly property string aiChip: "/assets/icons/ai-chip.svg"
        readonly property string gpuCard: "/assets/icons/gpu-card.svg"
        readonly property string powerBolt: "/assets/icons/power-bolt.svg"
    }
    
    // Animation functions
    function createGlowEffect(target, glowColor, intensity) {
        if (!target) return null
        
        return Qt.createComponent("GlowEffect.qml").createObject(target, {
            "glowColor": glowColor,
            "intensity": intensity,
            "animationDuration": animationDuration
        })
    }
    
    function animateProperty(target, property, to, duration, easing) {
        if (!target) return
        
        var animation = Qt.createQmlObject('
            import QtQuick 2.15
            NumberAnimation {
                duration: ' + (duration || animationDuration) + '
                easing.type: ' + (easing || "Easing.OutCubic") + '
            }
        ', target)
        
        animation.target = target
        animation.property = property
        animation.to = to
        animation.start()
        
        return animation
    }
    
    function createRippleEffect(target, x, y) {
        if (!target) return null
        
        return Qt.createComponent("RippleEffect.qml").createObject(target, {
            "x": x, "y": y,
            "rippleColor": colors.primary,
            "duration": shortAnimationDuration
        })
    }
    
    // Theme switching with smooth transitions
    function toggleTheme() {
        var newTheme = !isDarkTheme
        
        // Create transition animation
        var transition = Qt.createQmlObject('
            import QtQuick 2.15
            PropertyAnimation {
                duration: ' + longAnimationDuration + '
                easing.type: Easing.OutCubic
            }
        ', nexusTheme)
        
        // Animate theme switch
        transition.target = nexusTheme
        transition.property = "isDarkTheme"
        transition.to = newTheme
        transition.start()
    }
    
    // Dynamic color adjustments based on system state
    function updateSystemState(stellaState, maxJrState, securityState, aiState, load) {
        stellaActive = stellaState
        maxJrActive = maxJrState  
        securityActive = securityState
        aiOptimizationActive = aiState
        systemLoad = Math.max(0, Math.min(1, load))
        
        // Trigger subtle UI adaptations based on system load
        if (systemLoad > 0.8) {
            // High load - reduce animations and effects
            animationDuration = 150
            shortAnimationDuration = 100
        } else {
            // Normal load - standard animations
            animationDuration = 250
            shortAnimationDuration = 150
        }
    }
    
    // Accessibility helpers
    function getContrastRatio(color1, color2) {
        // Simple contrast ratio calculation
        var l1 = Qt.lighter(color1, 1.0)
        var l2 = Qt.lighter(color2, 1.0)
        return Math.max(l1, l2) / Math.min(l1, l2)
    }
    
    function ensureContrast(backgroundColor, textColor, minRatio) {
        var ratio = getContrastRatio(backgroundColor, textColor)
        if (ratio < (minRatio || 4.5)) {
            return isDarkTheme ? "#FFFFFF" : "#000000"
        }
        return textColor
    }
    
    // Component styling presets
    readonly property QtObject components: QtObject {
        readonly property QtObject button: QtObject {
            readonly property color background: colors.primary
            readonly property color backgroundHover: Qt.darker(colors.primary, 1.1)
            readonly property color backgroundPressed: Qt.darker(colors.primary, 1.2)
            readonly property color text: colors.onPrimary
            readonly property int padding: spacing.buttonPadding
            readonly property int radius: radius.button
            readonly property int fontSize: typography.labelLarge
            readonly property int fontWeight: typography.medium
        }
        
        readonly property QtObject card: QtObject {
            readonly property color background: colors.surface
            readonly property color border: colors.outline
            readonly property int padding: spacing.cardPadding
            readonly property int radius: radius.card
            readonly property var shadow: shadows.level2
        }
        
        readonly property QtObject panel: QtObject {
            readonly property color background: colors.panelBackground
            readonly property color border: colors.panelBorder
            readonly property int padding: spacing.panelPadding
            readonly property int radius: radius.panel
            readonly property var shadow: shadows.level3
        }
    }
}