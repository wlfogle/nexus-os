import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { ConnectionConfig } from '@services/ConnectionManager';

interface ConnectionState {
  status: 'disconnected' | 'connecting' | 'connected' | 'error';
  config: ConnectionConfig | null;
  error: string | null;
  lastConnected: string | null;
}

const initialState: ConnectionState = {
  status: 'disconnected',
  config: null,
  error: null,
  lastConnected: null,
};

export const connectionSlice = createSlice({
  name: 'connection',
  initialState,
  reducers: {
    setConnectionStatus: (state, action: PayloadAction<ConnectionState['status']>) => {
      state.status = action.payload;
      if (action.payload === 'connected') {
        state.error = null;
        state.lastConnected = new Date().toISOString();
      }
    },
    setConnectionConfig: (state, action: PayloadAction<ConnectionConfig>) => {
      state.config = action.payload;
    },
    setAuthToken: (state, action: PayloadAction<string>) => {
      if (state.config) {
        state.config.token = action.payload;
      }
    },
    setConnectionError: (state, action: PayloadAction<string>) => {
      state.error = action.payload;
      state.status = 'error';
    },
    clearConnectionError: (state) => {
      state.error = null;
    },
    resetConnection: (state) => {
      state.status = 'disconnected';
      state.error = null;
    },
  },
});

export const {
  setConnectionStatus,
  setConnectionConfig,
  setAuthToken,
  setConnectionError,
  clearConnectionError,
  resetConnection,
} = connectionSlice.actions;
