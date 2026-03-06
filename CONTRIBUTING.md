# Contributing to NexusOS 🚀

Thank you for your interest in contributing to NexusOS! We're building the world's first universal Linux distribution with AI mascot companions, and we'd love your help.

## 🌟 Ways to Contribute

### 🔧 Code Contributions
- **Core Systems**: nexuspkg universal package manager, system integration
- **AI Development**: Stella 🐕 & Max Jr. 🐱 assistants
- **Desktop Environment**: NexusDE components (QML/C++)
- **Media Stack**: Service configurations and Docker containers
- **Installation System**: nexus-install.sh installer improvements

### 📚 Documentation
- User guides and tutorials
- API documentation
- Installation instructions
- Troubleshooting guides

### 🎨 Design & Creative
- UI/UX improvements for NexusDE
- Mascot artwork for Stella & Max Jr.
- Branding and visual assets
- Website design

### 🧪 Testing & QA
- Hardware compatibility testing
- Package installation testing
- Performance benchmarking
- Bug reporting and verification

## 🛠️ Development Setup

### Prerequisites
- Pop!_OS 22.04 LTS NVIDIA (recommended)
- 16GB+ RAM
- 500GB+ storage
- Git, Docker, Qt6 development tools

### Getting Started

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/nexusos.git
cd nexusos

# Set up development environment
sudo mkdir -p /opt/nexusos
sudo chown $USER:$USER /opt/nexusos

# Install development dependencies
sudo nala install build-essential cmake qtbase5-dev qtdeclarative5-dev docker.io python3 python3-pip

# Build core components
make -C userspace/system/nexuspkg
make -C userspace/system/nexus-setup-assistant
```

## 📋 Development Guidelines

### Code Style
- **C/C++**: Follow Linux kernel style guidelines
- **Python**: Follow PEP 8 standards
- **QML**: Follow Qt QML coding conventions
- **Shell Scripts**: Use shellcheck for validation

### Commit Guidelines
- Use clear, descriptive commit messages
- Reference issue numbers when applicable
- Keep commits focused and atomic
- Follow conventional commit format:
  ```
  type(scope): description
  
  Examples:
  feat(nexuspkg): add universal package detection
  fix(stella): resolve security scan memory leak
  docs(readme): update installation instructions
  ```

### Testing Requirements
- Test changes on Pop!_OS base system
- Verify package installations don't conflict
- Ensure AI assistants function correctly
- Test media stack deployment
- Run existing test suite (when available)

## 🤖 AI Assistant Development

### Stella 🐕 (Security Guardian)
- **Language**: Python
- **Focus**: Security monitoring, package validation
- **Files**: `core/services/stella/`
- **Tests**: Security scanning, privacy features

### Max Jr. 🐱 (Performance Optimizer)
- **Language**: Python  
- **Focus**: Performance monitoring, system optimization
- **Files**: `core/services/maxjr/`
- **Tests**: Performance metrics, gaming optimization

## 🎮 Gaming Integration

### Requirements
- Maintain gaming performance parity with base Pop!_OS
- Test with popular games (Steam, Lutris)
- Validate GPU switching functionality
- Ensure gaming tools compatibility

## 📺 Media Stack Development

### Service Integration
- Follow awesome-stack patterns
- Use Docker containers for services
- Maintain port consistency (8000-8599 range)
- Test service health monitoring

## 🐛 Bug Reports

### Before Submitting
- Search existing issues
- Test on clean Pop!_OS installation
- Gather system information
- Document reproduction steps

### Bug Report Template
```markdown
**System Information**
- NexusOS Version: 
- Base System: Pop!_OS 22.04 NVIDIA
- Kernel Version:
- Hardware: CPU/GPU/RAM

**Bug Description**
Clear description of the issue

**Steps to Reproduce**
1. Step one
2. Step two
3. Expected vs actual behavior

**Additional Context**
- Log files
- Screenshots
- Related issues
```

## 💡 Feature Requests

### Guidelines
- Check existing feature requests
- Explain the use case and benefit
- Consider implementation complexity
- Align with NexusOS vision

### Feature Request Template
```markdown
**Feature Summary**
Brief description of the requested feature

**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
Other approaches you've thought about

**Additional Context**
Screenshots, mockups, related issues
```

## 🔄 Pull Request Process

### Before Submitting
1. Create feature branch: `git checkout -b feature/amazing-feature`
2. Make changes and test thoroughly
3. Update documentation if needed
4. Commit with clear messages
5. Push to your fork
6. Create pull request

### PR Requirements
- Clear title and description
- Reference related issues
- Include testing information
- Update documentation
- Pass all checks (when CI is set up)

### Review Process
1. Automated checks (coming soon)
2. Code review by maintainers
3. Testing on various hardware
4. Approval and merge

## 🌟 Recognition

Contributors will be:
- Listed in repository contributors
- Mentioned in release notes
- Added to project acknowledgments
- Invited to maintainer team (for significant contributions)

## 📞 Getting Help

### Communication Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time development chat (coming soon)
- **Email**: Direct contact with maintainers

### Development Questions
- Architecture decisions: Open GitHub discussion
- Implementation help: Comment on related issues
- AI assistant behavior: Tag @stella or @maxjr in comments
- Gaming performance: Use #gaming label

## 🎯 Current Priorities

### Phase 1 (Current)
- [ ] nexuspkg package manager completion
- [ ] AI assistant integration
- [ ] Media stack deployment
- [ ] NexusOS branding overlay

### Looking for Help With
- C++ developers for nexuspkg
- Python developers for AI assistants
- QML developers for NexusDE
- Docker experts for media stack
- Gaming enthusiasts for testing

## 📜 Code of Conduct

### Our Standards
- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect different viewpoints and experiences
- Show empathy towards others

### Unacceptable Behavior
- Harassment or discrimination
- Trolling or deliberately disruptive behavior
- Personal attacks or insults
- Publishing private information
- Inappropriate sexual content

## 📄 License Agreement

By contributing to NexusOS, you agree:
- Your contributions will be licensed under GPL-3.0+
- You have the right to submit your contributions
- You understand the open source nature of the project

---

## 🚀 Ready to Contribute?

1. **Star the repository** ⭐ to show your support
2. **Fork the project** 🍴 to start contributing  
3. **Join discussions** 💬 to connect with the community
4. **Pick an issue** 🎯 to work on
5. **Submit your first PR** 🎉

**Welcome to the NexusOS family!** 🐕🐱

*With Stella & Max Jr. cheering you on!*

---

*For questions about contributing, open a GitHub discussion or contact the maintainers.*