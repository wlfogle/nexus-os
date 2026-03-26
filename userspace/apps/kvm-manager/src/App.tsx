import React, { useState, Suspense, lazy } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  useMediaQuery,
  Avatar,
  Switch,
  FormControlLabel,
} from '@mui/material';
import {
  Dashboard as DashboardIcon,
  Computer,
  Storage as StorageIcon,
  NetworkPing,
  Settings as SettingsIcon,
  Menu as MenuIcon,
  DarkMode,
  LightMode,
  Memory,
} from '@mui/icons-material';
import { useLocation, useNavigate } from 'react-router-dom';

import { CustomThemeProvider, useCustomTheme } from './contexts/ThemeContext';
import { AppProviders } from './contexts/NotificationContext';
import LoadingSpinner from './components/LoadingSpinner';
import ErrorBoundary from './components/ErrorBoundary';

// Lazy load components for better performance
const VirtualMachinesDashboard = lazy(() => import('./components/VirtualMachinesDashboard'));
const Dashboard = lazy(() => import('./pages/Dashboard'));
const Storage = lazy(() => import('./pages/Storage'));
const Networks = lazy(() => import('./pages/Networks'));
const Settings = lazy(() => import('./pages/Settings'));

const DRAWER_WIDTH = 280;

interface NavItem {
  label: string;
  icon: React.ReactNode;
  path: string;
}

const navItems: NavItem[] = [
  { label: 'Dashboard', icon: <DashboardIcon />, path: '/' },
  { label: 'Virtual Machines', icon: <Computer />, path: '/virtual-machines' },
  { label: 'Storage', icon: <StorageIcon />, path: '/storage' },
  { label: 'Networks', icon: <NetworkPing />, path: '/networks' },
  { label: 'Settings', icon: <SettingsIcon />, path: '/settings' },
];

function AppContent() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const { isDarkMode, toggleTheme, theme } = useCustomTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('lg'));
  const location = useLocation();
  const navigate = useNavigate();

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  const handleNavigation = (path: string) => {
    navigate(path);
    if (isMobile) {
      setMobileOpen(false);
    }
  };

  const drawer = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Logo/Header */}
      <Box sx={{ p: 3, display: 'flex', alignItems: 'center', gap: 2 }}>
        <Avatar sx={{ bgcolor: 'primary.main', width: 40, height: 40 }}>
          <Memory />
        </Avatar>
        <Box>
          <Typography variant="h6" sx={{ fontWeight: 600 }}>
            KVM Manager
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Proxmox Edition
          </Typography>
        </Box>
      </Box>

      <Divider sx={{ mx: 2 }} />

      {/* Navigation */}
      <List sx={{ flex: 1, px: 2, pt: 2 }}>
        {navItems.map((item) => (
          <ListItem key={item.path} disablePadding sx={{ mb: 0.5 }}>
            <ListItemButton
              selected={location.pathname === item.path}
              onClick={() => handleNavigation(item.path)}
              sx={{
                borderRadius: 2,
                '&.Mui-selected': {
                  bgcolor: 'primary.main',
                  color: 'primary.contrastText',
                  '&:hover': {
                    bgcolor: 'primary.dark',
                  },
                  '& .MuiListItemIcon-root': {
                    color: 'primary.contrastText',
                  },
                },
              }}
            >
              <ListItemIcon sx={{ minWidth: 40 }}>{item.icon}</ListItemIcon>
              <ListItemText 
                primary={item.label} 
                primaryTypographyProps={{ fontWeight: location.pathname === item.path ? 600 : 400 }}
              />
            </ListItemButton>
          </ListItem>
        ))}
      </List>

      <Divider sx={{ mx: 2 }} />

      {/* Theme Toggle */}
      <Box sx={{ p: 2 }}>
        <FormControlLabel
          control={
            <Switch
              checked={isDarkMode}
              onChange={toggleTheme}
              icon={<LightMode />}
              checkedIcon={<DarkMode />}
            />
          }
          label="Dark Mode"
          sx={{ mb: 0 }}
        />
      </Box>
    </Box>
  );

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      {/* App Bar */}
      <AppBar
        position="fixed"
        elevation={0}
        sx={{
          width: { lg: `calc(100% - ${DRAWER_WIDTH}px)` },
          ml: { lg: `${DRAWER_WIDTH}px` },
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { lg: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1, fontWeight: 600 }}>
            {navItems.find(item => item.path === location.pathname)?.label || 'KVM Manager'}
          </Typography>
        </Toolbar>
      </AppBar>

      {/* Navigation Drawer */}
      <Box
        component="nav"
        sx={{ width: { lg: DRAWER_WIDTH }, flexShrink: { lg: 0 } }}
      >
        <Drawer
          variant="temporary"
          open={mobileOpen}
          onClose={handleDrawerToggle}
          ModalProps={{ keepMounted: true }}
          sx={{
            display: { xs: 'block', lg: 'none' },
            '& .MuiDrawer-paper': { boxSizing: 'border-box', width: DRAWER_WIDTH },
          }}
        >
          {drawer}
        </Drawer>
        <Drawer
          variant="permanent"
          sx={{
            display: { xs: 'none', lg: 'block' },
            '& .MuiDrawer-paper': { boxSizing: 'border-box', width: DRAWER_WIDTH },
          }}
          open
        >
          {drawer}
        </Drawer>
      </Box>

      {/* Main Content */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          width: { lg: `calc(100% - ${DRAWER_WIDTH}px)` },
          minHeight: '100vh',
          bgcolor: 'background.default',
        }}
      >
        <Toolbar /> {/* Spacer for AppBar */}
        <ErrorBoundary>
          <Suspense fallback={<LoadingSpinner />}>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/virtual-machines" element={<VirtualMachinesDashboard />} />
              <Route path="/storage" element={<Storage />} />
              <Route path="/networks" element={<Networks />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </Suspense>
        </ErrorBoundary>
      </Box>
    </Box>
  );
}

function App() {
  return (
    <CustomThemeProvider>
      <AppProviders>
        <Router>
          <AppContent />
        </Router>
      </AppProviders>
    </CustomThemeProvider>
  );
}

export default App;
