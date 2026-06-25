import { api } from './client';
import type { Session } from '$lib/types';

export interface LoginPayload {
  username: string;
  password: string;
  remember?: boolean;
  device_id?: string;
}

export interface TwoFAPayload {
  session_token: string;
  code: string;
}

export const authApi = {
  login: (payload: LoginPayload) =>
    api.post<Session>('/auth/login', payload),

  logout: () =>
    api.post<null>('/auth/logout', {}),

  me: () =>
    api.get<Session['user']>('/auth/me'),

  refreshToken: () =>
    api.post<Session>('/auth/refresh', {}),

  forgotPassword: (email: string) =>
    api.post<null>('/auth/forgot-password', { email }),

  resetPassword: (token: string, password: string) =>
    api.post<null>('/auth/reset-password', { token, password }),

  verify2FA: (payload: TwoFAPayload) =>
    api.post<Session>('/auth/2fa/verify', payload),

  setup2FA: () =>
    api.post<{ qr_url: string; secret: string }>('/auth/2fa/setup', {}),

  sessions: () =>
    api.get<{ id: string; device: string; ip: string; created_at: string }[]>('/auth/sessions'),

  revokeSession: (id: string) =>
    api.delete<null>(`/auth/sessions/${id}`),
};
