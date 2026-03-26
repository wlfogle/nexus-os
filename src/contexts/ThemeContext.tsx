import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { createTheme, ThemeProvider, Theme } from '@mui/material/styles';
import { CssBaseline, useMediaQuery } from '@mui/material';

interface ThemeContextType {
  isDarkMode: boolean;
  toggleTheme: () => void;
  theme: Theme;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useCustomTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useCustomTheme must be used within a CustomThemeProvider');
  }
  return context;
};

interface CustomThemeProviderProps {
  children: ReactNode;
}

export const CustomThemeProvider: React.FC<CustomThemeProviderProps> = ({ children }) => {
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');
  const [isDarkMode, setIsDarkMode] = useState(false);

  useEffect(() => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      setIsDarkMode(savedTheme === 'dark');
    } else {
      setIsDarkMode(prefersDarkMode);
    }
  }, [prefersDarkMode]);

  const toggleTheme = () => {
    const newMode = !isDarkMode;
    setIsDarkMode(newMode);
    localStorage.setItem('theme', newMode ? 'dark' : 'light');
  };

  const theme = createTheme({
    palette: {
      mode: isDarkMode ? 'dark' : 'light',
      primary: {
        main: isDarkMode ? '#90caf9' : '#1976d2',
        light: isDarkMode ? '#bbdefb' : '#42a5f5',
        dark: isDarkMode ? '#64b5f6' : '#1565c0',
        contrastText: '#fff',
      },
      secondary: {
        main: isDarkMode ? '#f48fb1' : '#dc004e',
        light: isDarkMode ? '#f8bbd9' : '#e91e63',
        dark: isDarkMode ? '#f06292' : '#c51162',
        contrastText: '#fff',
      },
      background: {
        default: isDarkMode ? '#0a0e27' : '#f5f7fa',
        paper: isDarkMode ? '#1e2139' : '#ffffff',
      },
      surface: {
        main: isDarkMode ? '#252945' : '#ffffff',
      },
      text: {
        primary: isDarkMode ? '#ffffff' : '#2d3748',
        secondary: isDarkMode ? '#cbd5e0' : '#718096',
      },
      divider: isDarkMode ? '#374151' : '#e2e8f0',
      success: {
        main: isDarkMode ? '#68d391' : '#38a169',
        light: isDarkMode ? '#9ae6b4' : '#68d391',
        dark: isDarkMode ? '#48bb78' : '#2f855a',
      },
      warning: {
        main: isDarkMode ? '#fbd38d' : '#ed8936',
        light: isDarkMode ? '#fdd89b' : '#f6ad55',
        dark: isDarkMode ? '#f6ad55' : '#dd6b20',
      },
      error: {
        main: isDarkMode ? '#fc8181' : '#e53e3e',
        light: isDarkMode ? '#feb2b2' : '#fc8181',
        dark: isDarkMode ? '#f56565' : '#c53030',
      },
      info: {
        main: isDarkMode ? '#63b3ed' : '#3182ce',
        light: isDarkMode ? '#90cdf4' : '#63b3ed',
        dark: isDarkMode ? '#4299e1' : '#2c5282',
      },
    },
    typography: {
      fontFamily: [
        '-apple-system',
        'BlinkMacSystemFont',
        '"Segoe UI"',
        'Roboto',
        '"Helvetica Neue"',
        'Arial',
        'sans-serif',
        '"Apple Color Emoji"',
        '"Segoe UI Emoji"',
        '"Segoe UI Symbol"',
      ].join(','),
      h1: {
        fontSize: '2.5rem',
        fontWeight: 700,
        lineHeight: 1.2,
        letterSpacing: '-0.025em',
      },
      h2: {
        fontSize: '2rem',
        fontWeight: 600,
        lineHeight: 1.25,
        letterSpacing: '-0.025em',
      },
      h3: {
        fontSize: '1.75rem',
        fontWeight: 600,
        lineHeight: 1.3,
        letterSpacing: '-0.025em',
      },
      h4: {
        fontSize: '1.5rem',
        fontWeight: 600,
        lineHeight: 1.35,
      },
      h5: {
        fontSize: '1.25rem',
        fontWeight: 600,
        lineHeight: 1.4,
      },
      h6: {
        fontSize: '1.125rem',
        fontWeight: 600,
        lineHeight: 1.4,
      },
      body1: {
        fontSize: '1rem',
        lineHeight: 1.6,
      },
      body2: {
        fontSize: '0.875rem',
        lineHeight: 1.6,
      },
      button: {
        textTransform: 'none',
        fontWeight: 500,
      },
    },
    shape: {
      borderRadius: 12,
    },
    components: {
      MuiCssBaseline: {
        styleOverrides: {
          body: {
            scrollbarWidth: 'thin',
            scrollbarColor: isDarkMode ? '#6B7280 #1F2937' : '#CBD5E0 #F7FAFC',
            '&::-webkit-scrollbar': {
              width: '8px',
            },
            '&::-webkit-scrollbar-track': {
              background: isDarkMode ? '#1F2937' : '#F7FAFC',
            },
            '&::-webkit-scrollbar-thumb': {
              background: isDarkMode ? '#6B7280' : '#CBD5E0',
              borderRadius: '4px',
            },
            '&::-webkit-scrollbar-thumb:hover': {
              background: isDarkMode ? '#9CA3AF' : '#A0AEC0',
            },
          },
        },
      },
      MuiCard: {
        styleOverrides: {
          root: {
            borderRadius: 16,
            boxShadow: isDarkMode 
              ? '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.2)'
              : '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
            border: `1px solid ${isDarkMode ? '#374151' : '#e2e8f0'}`,
            transition: 'transform 0.2s ease-in-out, box-shadow 0.2s ease-in-out',
            '&:hover': {
              transform: 'translateY(-2px)',
              boxShadow: isDarkMode 
                ? '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -2px rgba(0, 0, 0, 0.3)'
                : '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
            },
          },
        },
      },
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            textTransform: 'none',
            fontWeight: 500,
            padding: '8px 16px',
            transition: 'all 0.2s ease-in-out',
          },
          contained: {
            boxShadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
            '&:hover': {
              boxShadow: '0 4px 8px rgba(0, 0, 0, 0.15)',
              transform: 'translateY(-1px)',
            },
          },
        },
      },
      MuiIconButton: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            transition: 'all 0.2s ease-in-out',
            '&:hover': {
              transform: 'scale(1.05)',
            },
          },
        },
      },
      MuiChip: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            fontWeight: 500,
            '&.MuiChip-colorSuccess': {
              backgroundColor: isDarkMode ? '#059669' : '#10b981',
              color: '#ffffff',
            },
            '&.MuiChip-colorWarning': {
              backgroundColor: isDarkMode ? '#d97706' : '#f59e0b',
              color: '#ffffff',
            },
            '&.MuiChip-colorError': {
              backgroundColor: isDarkMode ? '#dc2626' : '#ef4444',
              color: '#ffffff',
            },
          },
        },
      },
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': {
              borderRadius: 8,
              transition: 'all 0.2s ease-in-out',
              '&:hover': {
                transform: 'translateY(-1px)',
              },
              '&.Mui-focused': {
                transform: 'translateY(-1px)',
                boxShadow: `0 0 0 3px ${isDarkMode ? 'rgba(144, 202, 249, 0.16)' : 'rgba(25, 118, 210, 0.16)'}`,
              },
            },
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            borderRadius: 12,
            border: `1px solid ${isDarkMode ? '#374151' : '#e2e8f0'}`,
          },
        },
      },
      MuiAppBar: {
        styleOverrides: {
          root: {
            backgroundColor: isDarkMode ? '#1e2139' : '#ffffff',
            color: isDarkMode ? '#ffffff' : '#2d3748',
            boxShadow: isDarkMode 
              ? '0 1px 3px rgba(0, 0, 0, 0.2)'
              : '0 1px 3px rgba(0, 0, 0, 0.1)',
            borderBottom: `1px solid ${isDarkMode ? '#374151' : '#e2e8f0'}`,
          },
        },
      },
      MuiDrawer: {
        styleOverrides: {
          paper: {
            backgroundColor: isDarkMode ? '#1e2139' : '#ffffff',
            borderRight: `1px solid ${isDarkMode ? '#374151' : '#e2e8f0'}`,
          },
        },
      },
      MuiListItemButton: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            margin: '2px 8px',
            '&.Mui-selected': {
              backgroundColor: isDarkMode ? '#3730a3' : '#dbeafe',
              color: isDarkMode ? '#ffffff' : '#1e40af',
              '&:hover': {
                backgroundColor: isDarkMode ? '#4338ca' : '#bfdbfe',
              },
            },
            '&:hover': {
              backgroundColor: isDarkMode ? '#374151' : '#f3f4f6',
            },
          },
        },
      },
      MuiLinearProgress: {
        styleOverrides: {
          root: {
            borderRadius: 4,
            height: 6,
          },
        },
      },
      MuiCircularProgress: {
        styleOverrides: {
          root: {
            animationDuration: '1.4s',
          },
        },
      },
    },
    spacing: 8,
    transitions: {
      easing: {
        easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
        easeOut: 'cubic-bezier(0.0, 0, 0.2, 1)',
        easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
        sharp: 'cubic-bezier(0.4, 0, 0.6, 1)',
      },
    },
  });

  return (
    <ThemeContext.Provider value={{ isDarkMode, toggleTheme, theme }}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        {children}
      </ThemeProvider>
    </ThemeContext.Provider>
  );
};
