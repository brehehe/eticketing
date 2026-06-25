// ============================================================
// API Client — typed fetch wrapper with auth header injection
// ============================================================

import type { ApiResponse, PaginatedResponse, ApiError } from '$lib/types';

const BASE = import.meta.env.VITE_API_URL ?? '/api';

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem('tk-token');
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  signal?: AbortSignal
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };
  const token = getToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
    signal,
  });

  if (res.status === 401) {
    // Clear session and redirect to login
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem('tk-token');
      localStorage.removeItem('tk-session');
    }
    if (typeof window !== 'undefined') window.location.href = '/login';
    throw { status: 401, message: 'Sesi berakhir' } as ApiError;
  }

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw {
      status: res.status,
      message: data.message ?? 'Terjadi kesalahan',
      errors: data.errors,
    } as ApiError;
  }

  return data as T;
}

export const api = {
  get:    <T>(path: string, signal?: AbortSignal) =>
    request<ApiResponse<T>>('GET', path, undefined, signal),

  post:   <T>(path: string, body: unknown, signal?: AbortSignal) =>
    request<ApiResponse<T>>('POST', path, body, signal),

  put:    <T>(path: string, body: unknown, signal?: AbortSignal) =>
    request<ApiResponse<T>>('PUT', path, body, signal),

  patch:  <T>(path: string, body: unknown, signal?: AbortSignal) =>
    request<ApiResponse<T>>('PATCH', path, body, signal),

  delete: <T>(path: string, signal?: AbortSignal) =>
    request<ApiResponse<T>>('DELETE', path, undefined, signal),

  paginated: <T>(path: string, signal?: AbortSignal) =>
    request<PaginatedResponse<T>>('GET', path, undefined, signal),
};
