# External Dependencies and Credits

This project makes use of or references several excellent external projects and tools.

## Third-Party Scripts and Tools

### Proxmox Helper Scripts
- **Source**: [tteck/Proxmox](https://github.com/tteck/Proxmox)
- **License**: MIT License
- **Usage**: Proxmox VM/LXC container automation scripts
- **Description**: Community-maintained Proxmox helper scripts for easy container deployment
- **Integration**: Referenced for Proxmox VM management and container setup

### Installation
To use the referenced Proxmox scripts:
```bash
# Download tteck's Proxmox scripts
wget -O proxmox-scripts.zip https://github.com/tteck/Proxmox/archive/refs/heads/main.zip
unzip proxmox-scripts.zip
cd Proxmox-main/

# Use specific installation scripts as needed
# Example: Install Home Assistant LXC
bash ct/homeassistant.sh
```

## Other External References

### Docker Images
- All Docker services use official or community-maintained images
- See `docker-compose.yml` files for specific image sources and versions

### Documentation Sources
- Home Assistant: https://www.home-assistant.io/
- Proxmox: https://www.proxmox.com/
- Docker: https://docs.docker.com/

## Acknowledgments

We thank the following projects and their maintainers:
- **tteck** for the excellent Proxmox helper scripts
- **Home Assistant Community** for automation examples
- **LinuxServer.io** for many of the Docker images used
- **Traefik Labs** for the reverse proxy solution

## License Compliance

This project respects all upstream licenses and attributions. Any integration or reference to external tools maintains proper attribution and licensing compliance.
