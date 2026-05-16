import { create } from 'zustand';
import {
  fetchHealth,
  fetchInfo,
  fetchSyncStatus,
  runSyncAction,
  saveGitHubConfig,
  saveWebDavConfig,
  type GitHubConfigRequest,
  type HealthResponse,
  type InfoResponse,
  type SyncOperationResponse,
  type SyncStatusResponse,
  type WebDavConfigRequest,
} from './api';

interface ServerState {
  health?: HealthResponse;
  info?: InfoResponse;
  syncStatus?: SyncStatusResponse;
  lastOperation?: SyncOperationResponse;
  loading: boolean;
  syncLoading: boolean;
  actionLoading?: 'test' | 'pull' | 'push' | 'github' | 'webdav';
  error?: string;
  syncError?: string;
  refresh: () => Promise<void>;
  loadSyncStatus: () => Promise<void>;
  configureGitHub: (request: GitHubConfigRequest) => Promise<void>;
  configureWebDav: (request: WebDavConfigRequest) => Promise<void>;
  runSyncOperation: (action: 'test' | 'pull' | 'push') => Promise<void>;
}

export const useServerStore = create<ServerState>((set) => ({
  loading: false,
  syncLoading: false,
  async refresh() {
    set({ loading: true, error: undefined });
    try {
      const [health, info, syncStatus] = await Promise.all([
        fetchHealth(),
        fetchInfo(),
        fetchSyncStatus(),
      ]);
      set({ health, info, syncStatus, loading: false });
    } catch (error) {
      set({
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  },
  async loadSyncStatus() {
    set({ syncLoading: true, syncError: undefined });
    try {
      const syncStatus = await fetchSyncStatus();
      set({ syncStatus, syncLoading: false });
    } catch (error) {
      set({
        syncLoading: false,
        syncError: error instanceof Error ? error.message : String(error),
      });
    }
  },
  async configureGitHub(request) {
    set({ actionLoading: 'github', syncError: undefined, lastOperation: undefined });
    try {
      const syncStatus = await saveGitHubConfig(request);
      set({ syncStatus, actionLoading: undefined });
    } catch (error) {
      set({
        actionLoading: undefined,
        syncError: error instanceof Error ? error.message : String(error),
      });
    }
  },
  async configureWebDav(request) {
    set({ actionLoading: 'webdav', syncError: undefined, lastOperation: undefined });
    try {
      const syncStatus = await saveWebDavConfig(request);
      set({ syncStatus, actionLoading: undefined });
    } catch (error) {
      set({
        actionLoading: undefined,
        syncError: error instanceof Error ? error.message : String(error),
      });
    }
  },
  async runSyncOperation(action) {
    set({ actionLoading: action, syncError: undefined, lastOperation: undefined });
    try {
      const lastOperation = await runSyncAction(action);
      set({
        lastOperation,
        syncStatus: lastOperation.status,
        actionLoading: undefined,
      });
    } catch (error) {
      set({
        actionLoading: undefined,
        syncError: error instanceof Error ? error.message : String(error),
      });
    }
  },
}));
