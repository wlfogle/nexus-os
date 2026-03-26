# Development Guide

## Prerequisites

### System Requirements
- Proxmox VE server with access to `pct` and `qm` commands
- Rust toolchain (1.60+)
- Node.js (for Tauri frontend)
- Git

### Installing Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Installing Tauri CLI
```bash
cargo install tauri-cli
```

## Building the Project

### Development Build
```bash
# Clone the repository
git clone https://github.com/wlfogle/proxmox-infrastructure-admin.git
cd proxmox-infrastructure-admin

# Run in development mode
cargo tauri dev
```

### Production Build
```bash
# Build for production
cargo tauri build
```

The built application will be available in `src-tauri/target/release/bundle/`.

## Project Structure

```
proxmox-infrastructure-admin/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   └── main.rs         # Main application logic
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   └── build.rs            # Build script
├── dist/                   # Frontend files
│   └── index.html          # Main UI
├── README.md               # Project documentation
├── DEVELOPMENT.md          # Development guide
├── LICENSE                 # MIT license
├── .gitignore             # Git ignore rules
└── package.json           # Node.js metadata
```

## Key Components

### Backend (Rust)
- **Container Management**: Uses `pct` commands to manage LXC containers
- **VM Management**: Uses `qm` commands to manage virtual machines
- **System Overview**: Aggregates status information from all services
- **Tauri Commands**: Exposes backend functionality to frontend

### Frontend (HTML/CSS/JavaScript)
- **Modern UI**: Glassmorphism design with responsive layout
- **Service Categories**: Organized by infrastructure type
- **Real-time Updates**: Live status monitoring and control
- **Interactive Controls**: Start/stop/restart buttons for all services

## Adding New Services

### Adding a Container
1. Update the container definitions in `src-tauri/src/main.rs`:
   ```rust
   let new_containers = vec![
       (300, "New Service", "Service description"),
       // ... other services
   ];
   ```

2. Add category mapping in `get_container_category()` function
3. Add description mapping in `get_container_description()` function

### Adding a VM
1. Update the VM definitions in `get_system_overview()`:
   ```rust
   let vm_definitions = vec![
       (700, "New VM", "VM description"),
       // ... other VMs
   ];
   ```

2. Add name mapping in `get_vm_name()` function
3. Add description mapping in `get_vm_description()` function

## Testing

### Manual Testing
1. Start the application in development mode
2. Verify all containers and VMs are detected
3. Test start/stop/restart functionality
4. Check status updates and notifications

### Integration Testing
The application requires a running Proxmox VE environment for full testing. Ensure you have:
- Properly configured LXC containers with expected IDs
- Virtual machines with expected IDs
- Proper permissions to execute `pct` and `qm` commands

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Ensure the user has sudo privileges or proper Proxmox permissions
   - Check that `pct` and `qm` commands are accessible

2. **Container/VM Not Found**
   - Verify the container/VM IDs match your Proxmox configuration
   - Check that containers/VMs actually exist

3. **Build Errors**
   - Ensure Rust toolchain is up to date: `rustup update`
   - Clear target directory: `cargo clean`

### Debug Mode
Run with debug logging:
```bash
RUST_LOG=debug cargo tauri dev
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Test thoroughly
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## Code Style

- Use `rustfmt` for Rust code formatting
- Follow standard JavaScript conventions for frontend code
- Include descriptive comments for complex logic
- Update documentation when adding new features

## Performance Considerations

- The application polls container/VM status periodically
- Large numbers of containers may impact performance
- Consider implementing caching for frequently accessed data
- Use async/await for all system commands to prevent UI blocking
