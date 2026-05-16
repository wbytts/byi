export interface HealthResponse {
  status: string;
  service: string;
}

export interface InfoResponse {
  name: string;
  version: string;
  core_message: string;
  frontend_dir: string;
}

export type SyncRemote =
  | {
      provider: 'github';
      repo: string;
      branch: string;
      base_path: string;
      auth: string;
    }
  | {
      provider: 'webdav';
      preset: 'jianguoyun' | 'custom';
      endpoint_url: string;
      username?: string;
      base_path: string;
    };

export interface SyncStatusResponse {
  configured: boolean;
  message: string;
  remote?: SyncRemote;
}

export interface SyncOperationResponse {
  ok: boolean;
  message: string;
  status: SyncStatusResponse;
}

export interface GitHubConfigRequest {
  repo: string;
  branch?: string;
  base_path?: string;
}

export interface WebDavConfigRequest {
  preset?: 'jianguoyun' | 'custom';
  url?: string;
  username?: string;
  base_path?: string;
}

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(await errorMessage(path, response));
  }
  return response.json() as Promise<T>;
}

async function postJson<T>(path: string, body?: unknown): Promise<T> {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(await errorMessage(path, response));
  }
  return response.json() as Promise<T>;
}

async function errorMessage(path: string, response: Response) {
  try {
    const payload = (await response.json()) as { error?: string };
    return payload.error ?? `${path} failed with ${response.status}`;
  } catch {
    return `${path} failed with ${response.status}`;
  }
}

export function fetchHealth() {
  return getJson<HealthResponse>('/api/health');
}

export function fetchInfo() {
  return getJson<InfoResponse>('/api/info');
}

export function fetchSyncStatus() {
  return getJson<SyncStatusResponse>('/api/sync/status');
}

export function saveGitHubConfig(request: GitHubConfigRequest) {
  return postJson<SyncStatusResponse>('/api/sync/config/github', request);
}

export function saveWebDavConfig(request: WebDavConfigRequest) {
  return postJson<SyncStatusResponse>('/api/sync/config/webdav', request);
}

export function runSyncAction(action: 'test' | 'pull' | 'push') {
  return postJson<SyncOperationResponse>(`/api/sync/${action}`);
}
