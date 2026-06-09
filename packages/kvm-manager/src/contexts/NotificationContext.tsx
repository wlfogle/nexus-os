import React, { createContext, useContext, useState } from 'react';
import { SnackbarProvider, useSnackbar, VariantType } from 'notistack';

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
 children: React.ReactNode;
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

 const showProgress = (message: string, progress: number) => {
 setIsLoading(true);
 setLoadingMessage(message);
 setProgress(Math.min(100, Math.max(0, progress)));
 };

 const hideLoading = () => {
 setIsLoading(false);
 setProgress(null);
 };

 return (
 <LoadingContext.Provider value={{ isLoading, loadingMessage, progress, setLoading, showProgress, hideLoading }}>
 {children}
 </LoadingContext.Provider>
 );
};

const NotificationProvider: React.FC<ProviderProps> = ({ children }) => {
 const { enqueueSnackbar, closeSnackbar } = useSnackbar();

 const showNotification = (message: string, variant: VariantType, options: { persist?: boolean; action?: React.ReactNode } = {}) => {
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

 const showSuccess = (message: string, options: { persist?: boolean; action?: React.ReactNode } = {}) => {
 showNotification(message, 'success', options);
 };

 const showError = (message: string, options: { persist?: boolean; action?: React.ReactNode } = {}) => {
 showNotification(message, 'error', { persist: true, ...options });
 };

 const showWarning = (message: string, options: { persist?: boolean; action?: React.ReactNode } = {}) => {
 showNotification(message, 'warning', options);
 };

 const showInfo = (message: string, options: { persist?: boolean; action?: React.ReactNode } = {}) => {
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

export const AppProviders: React.FC<{ children: React.ReactNode }> = ({ children }) => {
 return (
 <SnackbarProvider
 maxSnack={5}
 dense
 preventDuplicate
 autoHideDuration={3000}
 >
 <LoadingProvider>
 <NotificationProvider>{children}</NotificationProvider>
 </LoadingProvider>
 </SnackbarProvider>
 );
};