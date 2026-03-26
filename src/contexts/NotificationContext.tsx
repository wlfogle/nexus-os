import React, { createContext, useContext, useState, ReactNode } from 'react';
import { SnackbarProvider, useSnackbar, VariantType } from 'notistack';
import { 
  Backdrop, 
  CircularProgress, 
  Typography, 
  Box, 
  LinearProgress,
  IconButton,
} from '@mui/material';
import { Close as CloseIcon } from '@mui/icons-material';
import { styled } from '@mui/material/styles';

// Styled components for better loading UI
const LoadingBackdrop = styled(Backdrop)(({ theme }) => ({
  zIndex: theme.zIndex.drawer + 1,
  backgroundColor: 'rgba(0, 0, 0, 0.7)',
  backdropFilter: 'blur(4px)',
}));

const LoadingContainer = styled(Box)(({ theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  gap: theme.spacing(3),
  padding: theme.spacing(4),
  backgroundColor: theme.palette.background.paper,
  borderRadius: theme.spacing(2),
  boxShadow: theme.shadows[10],
  maxWidth: 400,
  width: '90%',
}));

interface LoadingContextType {
  isLoading: boolean;
  loadingMessage: string;
  progress: number | null;
  setLoading: (loading: boolean, message?: string, progress?: number | null) => void;
  showProgress: (message: string, progress: number) => void;
  hideLoading: () => void;
}

const LoadingContext = createContext<LoadingContextType | undefined>(undefined);

export const useLoading = () => {
  const context = useContext(LoadingContext);
  if (!context) {
    const error = Error('useLoading must be used within a LoadingProvider');
    throw error;
  }
  return context;
};

interface NotificationContextType {
  showSuccess: (message: string, options?: { persist?: boolean; action?: ReactNode }) => void;
  showError: (message: string, options?: { persist?: boolean; action?: ReactNode }) => void;
  showWarning: (message: string, options?: { persist?: boolean; action?: ReactNode }) => void;
  showInfo: (message: string, options?: { persist?: boolean; action?: ReactNode }) => void;
  closeAll: () => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const useNotifications = () => {
  const context = useContext(NotificationContext);
  if (!context) {
    const error = Error('useNotifications must be used within a NotificationProvider');
    throw error;
  }
  return context;
};

interface ProviderProps {
  children: ReactNode;
}

const LoadingProvider: React.FC<ProviderProps> = ({ children }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState('Loading...');
  const [progress, setProgress] = useState<number | null>(null);

  const setLoading = (loading: boolean, message = 'Loading...', progressValue: number | null = null) => {
    setIsLoading(loading);
    setLoadingMessage(message);
    setProgress(progressValue);
  };

  const showProgress = (message: string, progressValue: number) => {
    setIsLoading(true);
    setLoadingMessage(message);
    setProgress(Math.min(100, Math.max(0, progressValue)));
  };

  const hideLoading = () => {
    setIsLoading(false);
    setProgress(null);
  };

  return (
    <LoadingContext.Provider value={{ isLoading, loadingMessage, progress, setLoading, showProgress, hideLoading }}>
      {children}
      <LoadingBackdrop open={isLoading}>
        <LoadingContainer>
          {progress !== null ? (
            <Box sx={{ width: '100%' }}>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Box sx={{ width: '100%', mr: 1 }}>
                  <LinearProgress variant="determinate" value={progress} sx={{ height: 8 }} />
                </Box>
                <Box sx={{ minWidth: 35 }}>
                  <Typography variant="body2" color="text.secondary">
                    {Math.round(progress)}%
                  </Typography>
                </Box>
              </Box>
              <Typography variant="h6" sx={{ textAlign: 'center', mt: 2 }}>
                {loadingMessage}
              </Typography>
            </Box>
          ) : (
            <>
              <CircularProgress size={60} thickness={4} />
              <Typography variant="h6" sx={{ textAlign: 'center' }}>
                {loadingMessage}
              </Typography>
            </>
          )}
        </LoadingContainer>
      </LoadingBackdrop>
    </LoadingContext.Provider>
  );
};

const NotificationProvider: React.FC<ProviderProps> = ({ children }) => {
  const { enqueueSnackbar, closeSnackbar } = useSnackbar();

  const showNotification = (message: string, variant: VariantType, options: { persist?: boolean; action?: ReactNode } = {}) => {
    const action = options.action || (
      <IconButton
        size="small"
        aria-label="close"
        color="inherit"
        onClick={() => closeSnackbar()}
      >
        <CloseIcon fontSize="small" />
      </IconButton>
    );

    enqueueSnackbar(message, {
      variant,
      persist: options.persist ?? false,
      action,
      anchorOrigin: {
        vertical: 'bottom',
        horizontal: 'right',
      },
    });
  };

  const showSuccess = (message: string, options: { persist?: boolean; action?: ReactNode } = {}) => {
    showNotification(message, 'success', options);
  };

  const showError = (message: string, options: { persist?: boolean; action?: ReactNode } = {}) => {
    showNotification(message, 'error', { persist: true, ...options });
  };

  const showWarning = (message: string, options: { persist?: boolean; action?: ReactNode } = {}) => {
    showNotification(message, 'warning', options);
  };

  const showInfo = (message: string, options: { persist?: boolean; action?: ReactNode } = {}) => {
    showNotification(message, 'info', options);
  };

  const closeAll = () => {
    closeSnackbar();
  };

  return (
    <NotificationContext.Provider value={{ showSuccess, showError, showWarning, showInfo, closeAll }}>
      {children}
    </NotificationContext.Provider>
  );
};

export const AppProviders: React.FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <SnackbarProvider
      maxSnack={5}
      dense
      preventDuplicate
      autoHideDuration={3000}
    >
      <LoadingProvider>
        <NotificationProvider>
          {children}
        </NotificationProvider>
      </LoadingProvider>
    </SnackbarProvider>
  );
};

// Hook for combined loading and notification operations
export const useAsyncOperation = () => {
  const { setLoading, hideLoading } = useLoading();
  const { showSuccess, showError } = useNotifications();

  const executeAsync = async <T,>(
    operation: () => Promise<T>,
    options: {
      loadingMessage?: string;
      successMessage?: string;
      errorMessage?: string;
      onSuccess?: (result: T) => void;
      onError?: (error: any) => void;
    } = {}
  ): Promise<T | null> => {
    try {
      setLoading(true, options.loadingMessage || 'Processing...');
      const result = await operation();
      
      if (options.successMessage) {
        showSuccess(options.successMessage);
      }
      
      options.onSuccess?.(result);
      return result;
    } catch (error) {
      const errorMsg = options.errorMessage || (error instanceof Error ? error.message : 'An error occurred');
      showError(errorMsg);
      options.onError?.(error);
      return null;
    } finally {
      hideLoading();
    }
  };

  return { executeAsync };
};
