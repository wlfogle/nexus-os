# Garuda Hello Architecture

This document describes the architecture and design of Garuda Hello, a Windows Hello equivalent for Garuda Linux that provides biometric authentication.

## Overview

Garuda Hello is a modular biometric authentication system that integrates with the Linux PAM (Pluggable Authentication Modules) system to provide seamless biometric login for users.

## Components

### 1. Common Library (`garuda-hello-common`)

The shared library containing common types, configurations, and utilities used across all components.

**Key responsibilities:**
- Define common data structures (User, BiometricDevice, AuthConfig, etc.)
- Provide configuration management
- Error handling and result types
- Biometric device abstraction layer

### 2. Daemon (`garuda-hello-daemon`)

The main system daemon that handles biometric device management and authentication requests.

**Key responsibilities:**
- Device discovery and initialization
- Template storage and management
- Authentication processing
- IPC communication via Unix sockets
- Security and privilege management

**Architecture details:**
- Runs as a system service with appropriate privileges
- Listens on Unix socket `/run/garuda-hello/daemon.sock`
- Manages biometric templates in secure storage
- Handles concurrent authentication requests

### 3. CLI Tool (`garuda-hello-cli`)

Command-line interface for system administration and user enrollment.

**Key responsibilities:**
- User enrollment and template management
- System configuration
- Device testing and diagnostics
- Administrative operations

**Commands:**
- `enroll` - Enroll biometric templates for a user
- `verify` - Test authentication for a user
- `list-devices` - Show available biometric devices
- `config` - Manage system configuration

### 4. PAM Module (`garuda-hello-pam`)

The PAM module that integrates with the Linux authentication system.

**Key responsibilities:**
- Interface with PAM authentication stack
- Communicate with daemon for authentication requests
- Handle PAM session management
- Provide fallback mechanisms

## Data Flow

```
User Login Attempt
       ↓
   PAM Stack
       ↓
Garuda Hello PAM Module
       ↓
Unix Socket IPC
       ↓
Garuda Hello Daemon
       ↓
Biometric Device
       ↓
Template Matching
       ↓
Authentication Result
```

## Security Considerations

### Template Storage
- Biometric templates are stored encrypted
- Only the daemon has access to template storage
- Templates are never transmitted in plaintext

### IPC Security
- Unix socket communication with appropriate permissions
- Message authentication to prevent tampering
- Privilege separation between components

### Device Access
- Controlled access to biometric devices
- Device capability validation
- Secure device communication protocols

## Configuration

### System Configuration (`/etc/garuda-hello/config.toml`)
- Global authentication settings
- Device-specific configurations
- Security policies
- Fallback options

### User Configuration (`~/.config/garuda-hello/`)
- User-specific authentication preferences
- Enrolled device associations
- Personal security settings

## Integration Points

### PAM Integration
- Configuration in `/etc/pam.d/` files
- Integration with existing authentication chains
- Support for sufficient/required authentication modes

### Desktop Environment Integration
- GNOME/KDE login screen support
- Screen unlock functionality
- sudo/authentication prompt integration

### systemd Integration
- Service management
- Automatic startup and dependency handling
- Proper privilege management

## Device Support

### Supported Device Types
- Fingerprint scanners (USB, built-in)
- Face recognition cameras
- Iris scanners (future)
- Voice recognition (future)

### Device Interface
- Standardized device abstraction layer
- Plugin architecture for new device types
- Hot-plug device support

## Error Handling

### Authentication Failures
- Graceful fallback to password authentication
- Retry mechanisms with backoff
- Audit logging of authentication attempts

### System Failures
- Daemon recovery mechanisms
- Device error handling
- Configuration validation

## Performance Considerations

### Template Matching
- Optimized biometric algorithms
- Caching of frequently used templates
- Parallel processing for multiple devices

### System Resources
- Minimal memory footprint
- Efficient device polling
- Resource cleanup and management

## Future Enhancements

### Planned Features
- Multi-factor authentication combinations
- Advanced anti-spoofing measures
- Cloud template synchronization
- Mobile device integration

### Extensibility
- Plugin system for custom authentication methods
- API for third-party integrations
- Scriptable configuration and management

## Development Guidelines

### Code Organization
- Modular design with clear interfaces
- Comprehensive error handling
- Extensive logging and debugging support

### Testing Strategy
- Unit tests for all components
- Integration tests with mock devices
- Security testing and validation

### Documentation
- API documentation
- User guides and tutorials
- System administrator documentation