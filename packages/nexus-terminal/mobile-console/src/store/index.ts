import { configureStore } from '@reduxjs/toolkit';
import { connectionSlice } from './slices/connectionSlice';
import { systemSlice } from './slices/systemSlice';
import { terminalSlice } from './slices/terminalSlice';
import { filesSlice } from './slices/filesSlice';
import { ecosystemSlice } from './slices/ecosystemSlice';

export const store = configureStore({
  reducer: {
    connection: connectionSlice.reducer,
    system: systemSlice.reducer,
    terminal: terminalSlice.reducer,
    files: filesSlice.reducer,
    ecosystem: ecosystemSlice.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST'],
      },
    }),
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
