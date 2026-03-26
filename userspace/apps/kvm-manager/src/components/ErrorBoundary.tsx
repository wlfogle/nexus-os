import React, { Component, ReactNode } from 'react';
import {
  Box,
  Typography,
  Button,
  Paper,
  Alert,
  AlertTitle,
  Stack,
} from '@mui/material';
import { ErrorOutline, Refresh } from '@mui/icons-material';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return {
      hasError: true,
      error,
    };
  }

  override componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // Log error to console in development
    if (import.meta.env.MODE === 'development') {
      console.error('ErrorBoundary caught an error:', error, errorInfo);
    }

    // Call custom error handler if provided
    this.props.onError?.(error, errorInfo);
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  handleReload = () => {
    window.location.reload();
  };

  override render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: '50vh',
            p: 3,
          }}
        >
          <Paper
            elevation={3}
            sx={{
              p: 4,
              textAlign: 'center',
              maxWidth: 600,
              width: '100%',
            }}
          >
            <ErrorOutline
              sx={{
                fontSize: 64,
                color: 'error.main',
                mb: 2,
              }}
            />
            
            <Typography variant="h5" gutterBottom>
              Something went wrong
            </Typography>
            
            <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
              We encountered an unexpected error. This has been logged and we're looking into it.
            </Typography>

            <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
              <AlertTitle>Error Details</AlertTitle>
              <Typography variant="body2" component="pre" sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
                {this.state.error?.message || 'Unknown error occurred'}
              </Typography>
            </Alert>

            <Stack direction="row" spacing={2} justifyContent="center">
              <Button
                variant="contained"
                onClick={this.handleReset}
                startIcon={<Refresh />}
              >
                Try Again
              </Button>
              <Button
                variant="outlined"
                onClick={this.handleReload}
              >
                Reload Page
              </Button>
            </Stack>

            {import.meta.env.MODE === 'development' && this.state.errorInfo && (
              <Box sx={{ mt: 3, textAlign: 'left' }}>
                <Typography variant="subtitle2" gutterBottom>
                  Stack Trace (Development Only):
                </Typography>
                <Paper
                  variant="outlined"
                  sx={{
                    p: 2,
                    bgcolor: 'grey.50',
                    maxHeight: 200,
                    overflow: 'auto',
                  }}
                >
                  <Typography
                    variant="body2"
                    component="pre"
                    sx={{
                      fontFamily: 'monospace',
                      fontSize: '0.7rem',
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-word',
                    }}
                  >
                    {this.state.errorInfo.componentStack}
                  </Typography>
                </Paper>
              </Box>
            )}
          </Paper>
        </Box>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
