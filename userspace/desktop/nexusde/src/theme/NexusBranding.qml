/**
 * NexusDE Branding Manager
 * Central system for NexusOS visual identity, logos, and mascot integration
 */

pragma Singleton
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtGraphicalEffects 1.15

Item {
    id: nexusBranding

    // Brand identity
    readonly property string brandName: "NexusOS"
    readonly property string tagline: "The Future of Desktop Computing"
    readonly property string version: "2024.1 Stellar"
    
    // Mascot properties
    readonly property string stellaName: "Stella"
    readonly property string maxJrName: "Max Jr."
    readonly property string stellaRole: "Security & Package Guardian"
    readonly property string maxJrRole: "System Monitor & Optimization Expert"
    
    // Logo variants and assets
    readonly property QtObject logos: QtObject {
        readonly property string primary: "/assets/branding/nexus-logo-primary.svg"
        readonly property string horizontal: "/assets/branding/nexus-logo-horizontal.svg"
        readonly property string mark: "/assets/branding/nexus-mark.svg"
        readonly property string wordmark: "/assets/branding/nexus-wordmark.svg"
        readonly property string stellaIcon: "/assets/branding/stella-icon.svg"
        readonly property string maxJrIcon: "/assets/branding/maxjr-icon.svg"
        readonly property string duoLogo: "/assets/branding/stella-maxjr-duo.svg"
        readonly property string stellaPhoto: "/assets/branding/Stella and Max Jr.jpg"
        readonly property string stellaPhotoCircular: "/assets/branding/stella-maxjr-circular.png"
        readonly property string stellaPhotoBordered: "/assets/branding/stella-maxjr-bordered.jpg"
        readonly property string stellaTechOverlay: "/assets/branding/stella-maxjr-tech-overlay.svg"
    }
    
    // Animation states for branding elements
    readonly property QtObject brandAnimations: QtObject {
        readonly property int logoSpinDuration: 2000
        readonly property int mascotPulseDuration: 1500
        readonly property int brandRevealDuration: 800
        readonly property int typewriterSpeed: 100
        
        // Signature animation sequences
        readonly property var stellaEntrance: {
            "scale": [0.8, 1.1, 1.0],
            "opacity": [0, 0.8, 1.0],
            "rotation": [0, 10, 0],
            "duration": 1200
        }
        
        readonly property var maxJrEntrance: {
            "scale": [0.8, 1.1, 1.0], 
            "opacity": [0, 0.8, 1.0],
            "rotation": [0, -10, 0],
            "duration": 1200
        }
    }
    
    // Brand color system with accessibility considerations
    readonly property QtObject brandColors: QtObject {
        // Primary brand palette
        readonly property color nexusBlue: "#1E3A8A"
        readonly property color nexusBlueDark: "#1E40AF"
        readonly property color nexusBlueLight: "#3B82F6"
        
        // Secondary brand palette  
        readonly property color nexusTeal: "#0D9488"
        readonly property color nexusTealDark: "#0F766E"
        readonly property color nexusTealLight: "#14B8A6"
        
        // Accent colors
        readonly property color nexusPurple: "#7C3AED"
        readonly property color nexusOrange: "#EA580C" 
        readonly property color nexusGreen: "#059669"
        readonly property color nexusGold: "#D97706"
        
        // Mascot signature colors
        readonly property color stellaBlue: "#3B82F6"
        readonly property color stellaSecondary: "#60A5FA"
        readonly property color stellaAccent: "#93C5FD"
        
        readonly property color maxJrOrange: "#F97316"
        readonly property color maxJrSecondary: "#FB923C"
        readonly property color maxJrAccent: "#FDBA74"
        
        // Gradient definitions
        readonly property var stellaGradient: {
            "type": "linear",
            "stops": [
                {"position": 0.0, "color": stellaBlue},
                {"position": 0.5, "color": stellaSecondary},
                {"position": 1.0, "color": stellaAccent}
            ]
        }
        
        readonly property var maxJrGradient: {
            "type": "linear", 
            "stops": [
                {"position": 0.0, "color": maxJrOrange},
                {"position": 0.5, "color": maxJrSecondary},
                {"position": 1.0, "color": maxJrAccent}
            ]
        }
        
        readonly property var nexusGradient: {
            "type": "linear",
            "stops": [
                {"position": 0.0, "color": nexusBlue},
                {"position": 0.3, "color": nexusTeal},
                {"position": 0.7, "color": nexusPurple},
                {"position": 1.0, "color": nexusOrange}
            ]
        }
    }
    
    // Typography system for branding
    readonly property QtObject brandTypography: QtObject {
        readonly property string primaryFont: "Inter Display"
        readonly property string logoFont: "Inter Display" 
        readonly property string taglineFont: "Inter"
        readonly property string mascotFont: "Inter"
        
        readonly property QtObject sizes: QtObject {
            readonly property int brandTitle: 48
            readonly property int productName: 36
            readonly property int version: 24
            readonly property int tagline: 18
            readonly property int mascotLabel: 14
            readonly property int copyright: 12
        }
        
        readonly property QtObject weights: QtObject {
            readonly property int brandTitle: 700
            readonly property int productName: 600
            readonly property int version: 500
            readonly property int tagline: 400
            readonly property int mascotLabel: 500
            readonly property int copyright: 400
        }
    }
    
    // Component factory functions
    function createNexusLogo(parent, size, variant) {
        size = size || 128
        variant = variant || "primary"
        
        var logoComponent = Qt.createComponent("NexusLogo.qml")
        if (logoComponent.status === Component.Ready) {
            return logoComponent.createObject(parent, {
                "width": size,
                "height": size,
                "source": logos[variant] || logos.primary,
                "animated": true
            })
        }
        return null
    }
    
    function createMascotDuo(parent, width, stellaActive, maxJrActive) {
        var duoComponent = Qt.createComponent("MascotDuo.qml")
        if (duoComponent.status === Component.Ready) {
            return duoComponent.createObject(parent, {
                "width": width || 200,
                "stellaActive": stellaActive || false,
                "maxJrActive": maxJrActive || false
            })
        }
        return null
    }
    
    function createBrandedButton(parent, text, variant) {
        variant = variant || "primary"
        
        var buttonComponent = Qt.createComponent("NexusButton.qml")
        if (buttonComponent.status === Component.Ready) {
            return buttonComponent.createObject(parent, {
                "text": text,
                "variant": variant,
                "brandColors": brandColors,
                "typography": brandTypography
            })
        }
        return null
    }
    
    // Branding animation functions
    function animateBrandReveal(target, delay) {
        if (!target) return null
        
        delay = delay || 0
        
        var revealAnimation = Qt.createQmlObject('
            import QtQuick 2.15
            SequentialAnimation {
                PauseAnimation { duration: ' + delay + ' }
                ParallelAnimation {
                    NumberAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: ' + brandAnimations.brandRevealDuration + '
                        easing.type: Easing.OutCubic
                    }
                    NumberAnimation {
                        property: "scale"
                        from: 0.8
                        to: 1.0
                        duration: ' + brandAnimations.brandRevealDuration + '
                        easing.type: Easing.OutBack
                    }
                }
            }
        ', target)
        
        revealAnimation.target = target
        revealAnimation.start()
        return revealAnimation
    }
    
    function animateStellaMascot(target, activationState) {
        if (!target) return null
        
        var stellaAnimation = Qt.createQmlObject('
            import QtQuick 2.15
            SequentialAnimation {
                ParallelAnimation {
                    NumberAnimation {
                        property: "scale"
                        to: ' + (activationState ? '1.1' : '1.0') + '
                        duration: 300
                        easing.type: Easing.OutBack
                    }
                    ColorAnimation {
                        property: "color"
                        to: "' + (activationState ? brandColors.stellaBlue : brandColors.stellaSecondary) + '"
                        duration: 300
                    }
                }
            }
        ', target)
        
        stellaAnimation.target = target
        stellaAnimation.start()
        return stellaAnimation
    }
    
    function animateMaxJrMascot(target, activationState) {
        if (!target) return null
        
        var maxJrAnimation = Qt.createQmlObject('
            import QtQuick 2.15
            SequentialAnimation {
                ParallelAnimation {
                    NumberAnimation {
                        property: "scale"  
                        to: ' + (activationState ? '1.1' : '1.0') + '
                        duration: 300
                        easing.type: Easing.OutBack
                    }
                    ColorAnimation {
                        property: "color"
                        to: "' + (activationState ? brandColors.maxJrOrange : brandColors.maxJrSecondary) + '"
                        duration: 300
                    }
                }
            }
        ', target)
        
        maxJrAnimation.target = target  
        maxJrAnimation.start()
        return maxJrAnimation
    }
    
    // Typewriter effect for taglines and version info
    function createTypewriterEffect(target, text, speed) {
        if (!target || !text) return null
        
        speed = speed || brandAnimations.typewriterSpeed
        
        var typewriterComponent = Qt.createComponent("TypewriterText.qml")
        if (typewriterComponent.status === Component.Ready) {
            return typewriterComponent.createObject(target, {
                "finalText": text,
                "typingSpeed": speed,
                "font": brandTypography.taglineFont
            })
        }
        return null
    }
    
    // Brand compliance and usage guidelines
    readonly property QtObject guidelines: QtObject {
        readonly property real minimumLogoSize: 32
        readonly property real clearSpaceRatio: 0.25  // 25% of logo width/height
        readonly property real maxScaleRatio: 3.0     // Maximum scale multiplier
        
        readonly property var approvedBackgrounds: [
            "#FFFFFF", "#F8F9FA", "#000000", "#1A1B1E",
            brandColors.nexusBlue, brandColors.nexusTeal
        ]
        
        readonly property var prohibitedUsage: [
            "stretching or distorting logos",
            "changing brand colors without approval", 
            "placing logos on busy backgrounds",
            "using low resolution versions"
        ]
    }
    
    // Accessibility and internationalization support
    function getHighContrastVariant(isDarkTheme) {
        return {
            "primary": isDarkTheme ? "#FFFFFF" : "#000000",
            "secondary": isDarkTheme ? "#E5E5E5" : "#333333",
            "accent": brandColors.nexusBlue,
            "stellaColor": isDarkTheme ? "#60A5FA" : "#1E40AF",
            "maxJrColor": isDarkTheme ? "#FB923C" : "#C2410C"
        }
    }
    
    function getBrandingForLocale(locale) {
        // Placeholder for internationalization
        var brandingMap = {
            "en_US": {
                "name": brandName,
                "tagline": tagline,
                "stellaRole": stellaRole,
                "maxJrRole": maxJrRole
            },
            "es_ES": {
                "name": brandName,
                "tagline": "El Futuro de la Computación de Escritorio",
                "stellaRole": "Guardián de Seguridad y Paquetes",
                "maxJrRole": "Experto en Monitoreo y Optimización del Sistema"
            },
            "fr_FR": {
                "name": brandName,
                "tagline": "L'Avenir de l'Informatique de Bureau",
                "stellaRole": "Gardien de Sécurité et de Packages",
                "maxJrRole": "Expert en Surveillance et Optimisation Système"
            }
        }
        
        return brandingMap[locale] || brandingMap["en_US"]
    }
}