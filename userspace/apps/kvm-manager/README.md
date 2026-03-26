# KVM Manager - Proxmox Edition

A modern, feature-rich KVM virtualization manager built with React, TypeScript, and Tauri. This desktop application provides an intuitive interface for managing virtual machines with advanced monitoring, performance optimization, and a beautiful dark/light theme.

## ✨ Features

### 🖥️ **Virtual Machine Management**
- **Full VM Lifecycle**: Create, start, stop, pause, and delete virtual machines
- **Real-time Monitoring**: Live CPU, memory, disk I/O, and network statistics
- **State Management**: Track VM states (Running, Stopped, Paused, Error)
- **Snapshot Support**: Create and manage VM snapshots
- **Resource Configuration**: Configure CPU, memory, disk, and network settings

### 🎨 **Modern UI/UX**
- **Material Design**: Clean, modern interface built with MUI
- **Dark/Light Theme**: Seamless theme switching with system preference detection
- **Responsive Layout**: Optimized for desktop with mobile-friendly responsive design
- **Performance Optimized**: Code splitting, lazy loading, and React optimization patterns

### 📊 **Dashboard & Analytics**
- **System Overview**: Real-time host system statistics
- **VM Performance Charts**: Interactive charts with Recharts
- **Resource Utilization**: Track memory, CPU, storage, and network usage
- **Health Monitoring**: Monitor VM and host system health

### 🔧 **Advanced Features**
- **Storage Management**: Manage storage pools and volumes
- **Network Configuration**: Configure virtual networks and interfaces
- **Settings Panel**: Customize application behavior and preferences
- **Error Handling**: Comprehensive error boundaries and user feedback
- **State Persistence**: Zustand-powered state management with persistence

## 🚀 **Technology Stack**

### **Frontend**
- **React 19** - Latest React with concurrent features
- **TypeScript 5.9** - Type-safe development with strict configuration
- **Material-UI 6.2** - Modern React component library
- **Zustand 5.0** - Lightweight state management
- **React Router 7.0** - Client-side routing
- **Recharts 2.15** - Interactive data visualization
- **Framer Motion 12** - Smooth animations and transitions

### **Desktop Runtime**
- **Tauri 2.9** - Secure, lightweight desktop app framework
- **Rust Backend** - High-performance system integration

### **Build Tools & Development**
- **Vite 7.1** - Lightning-fast build tool and dev server
- **Vitest 2.1** - Fast unit testing framework
- **ESLint 9.17** - Code linting and formatting
- **Prettier 3.4** - Code formatting
- **TypeScript Strict Mode** - Enhanced type safety

### **Testing & Quality**
- **Testing Library** - Component and integration testing
- **Vitest UI** - Interactive test runner
- **Coverage Reports** - Comprehensive test coverage
- **Type Checking** - Strict TypeScript validation

## 🛠️ **Installation & Setup**

### **Prerequisites**
- **Node.js** ≥ 18.0.0
- **Rust** (latest stable)
- **System Dependencies**:
  - Linux: `libvirt-daemon`, `qemu-system-x86`
  - macOS: QEMU via Homebrew
  - Windows: QEMU for Windows

### **Development Setup**

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/kvm-manager.git
   cd kvm-manager
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start development server**
   ```bash
   npm run tauri:dev
   ```

### **Production Build**

```bash
# Build the application
npm run tauri:build

# The built application will be in src-tauri/target/release/bundle/
```

## 📋 **Available Scripts**

### **Development**
- `npm run dev` - Start Vite development server
- `npm run tauri:dev` - Start Tauri development application
- `npm run preview` - Preview production build

### **Building**
- `npm run build` - Build for production
- `npm run tauri:build` - Build Tauri application

### **Code Quality**
- `npm run lint` - Run ESLint
- `npm run lint:fix` - Fix ESLint issues automatically
- `npm run format` - Format code with Prettier
- `npm run type-check` - Run TypeScript type checking

### **Testing**
- `npm run test` - Run tests in watch mode
- `npm run test:run` - Run tests once
- `npm run test:ui` - Open Vitest UI
- `npm run coverage` - Generate test coverage report

## 🏗️ **Architecture**

### **Project Structure**
```
kvm-manager/
├── src/
│   ├── components/          # Reusable React components
│   │   ├── __tests__/      # Component tests
│   │   ├── ErrorBoundary.tsx
│   │   ├── LoadingSpinner.tsx
│   │   └── VmCard.tsx
│   ├── contexts/           # React context providers
│   │   ├── NotificationContext.tsx
│   │   └── ThemeContext.tsx
│   ├── pages/              # Route-based page components
│   │   ├── Dashboard.tsx
│   │   ├── VirtualMachines.tsx
│   │   └── Settings.tsx
│   ├── stores/             # Zustand state stores
│   │   └── vmStore.ts
│   ├── types/              # TypeScript type definitions
│   │   └── index.ts
│   ├── test/               # Test utilities and setup
│   │   └── setup.ts
│   ├── App.tsx             # Main application component
│   └── main.tsx            # Application entry point
├── src-tauri/              # Rust backend
│   ├── src/                # Rust source code
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
└── dist/                   # Build output
```

### **State Management**
- **Zustand Store**: Centralized VM state management
- **React Context**: UI theme and notification management
- **Local Storage**: Persistent user preferences
- **Real-time Updates**: WebSocket-like polling for live data

### **Performance Optimizations**
- **Code Splitting**: Lazy-loaded route components
- **React.memo**: Memoized components to prevent unnecessary rerenders
- **useMemo/useCallback**: Optimized hooks for expensive computations
- **Bundle Splitting**: Vendor libraries separated for better caching

## 🔒 **Security Features**

- **Tauri Security**: Sandboxed environment with minimal attack surface
- **Type Safety**: Comprehensive TypeScript coverage
- **Input Validation**: Zod/Yup schema validation
- **Error Boundaries**: Graceful error handling and recovery
- **Content Security Policy**: Configured CSP headers

## 🧪 **Testing Strategy**

### **Unit Tests**
- Component behavior testing
- State management logic
- Utility function validation
- Mock Tauri API calls

### **Integration Tests**
- Component interaction testing
- Store integration
- Route navigation
- Error boundary testing

### **Coverage Goals**
- **Components**: >80% coverage
- **Stores**: >90% coverage
- **Utilities**: >95% coverage

## 🤝 **Contributing**

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** following the code style
4. **Add tests** for new functionality
5. **Run the test suite**: `npm run test:run`
6. **Run linting**: `npm run lint`
7. **Commit your changes**: `git commit -m 'Add amazing feature'`
8. **Push to the branch**: `git push origin feature/amazing-feature`
9. **Open a Pull Request**

### **Development Guidelines**
- Follow TypeScript strict mode
- Write tests for new components and features
- Use conventional commit messages
- Ensure all checks pass before submitting PR
- Update documentation for new features

## 🐛 **Troubleshooting**

### **Common Issues**

**Build Failures**
```bash
# Clear node modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Clear Tauri cache
rm -rf src-tauri/target
```

**Permission Issues (Linux)**
```bash
# Add user to libvirt group
sudo usermod -a -G libvirt $USER
# Restart session for changes to take effect
```

**Development Server Issues**
```bash
# Check if port 3000 is available
lsof -ti:3000
# Kill process if needed
kill $(lsof -ti:3000)
```

## 📜 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **Acknowledgments**

- **Tauri Team** - For the excellent desktop app framework
- **React Team** - For the amazing frontend library
- **MUI Team** - For the beautiful component library
- **Vite Team** - For the fast build tooling
- **Open Source Community** - For the countless libraries that make this possible

---

**Built with ❤️ using modern web technologies**
