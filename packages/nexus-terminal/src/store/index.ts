import { configureStore } from '@reduxjs/toolkit';
import terminalReducer from './slices/terminalSlice';
import aiReducer from './slices/aiSlice';
import terminalTabReducer from './slices/terminalTabSlice';

export const store = configureStore({
  reducer: {
    terminal: terminalReducer,
    ai: aiReducer,
    terminalTabs: terminalTabReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['terminal/addOutput'],
      },
    }),
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
