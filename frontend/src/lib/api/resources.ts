import { api } from './client';
import type {
  User, Product, Transaction, Ticket, TicketScan,
  SalesReport, ProductReport, AppSettings, DashboardStats,
} from '$lib/types';

// ── Dashboard --------------------------------------------------
export const dashboardApi = {
  stats: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<DashboardStats>(`/reports/dashboard${q}`);
  },
};

// ── Categories -------------------------------------------------
export const categoriesApi = {
  list: () => api.get<any[]>('/categories'),
  create: (body: unknown) => api.post<any>('/categories', body),
  update: (id: string, body: unknown) => api.put<any>(`/categories/${id}`, body),
  delete: (id: string) => api.delete<null>(`/categories/${id}`),
};

// ── Users ------------------------------------------------------
export const usersApi = {
  list: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<User[]>(`/users${q}`);
  },
  get:    (id: string)  => api.get<User>(`/users/${id}`),
  create: (body: unknown) => api.post<User>('/users', body),
  update: (id: string, body: unknown) => api.put<User>(`/users/${id}`, body),
  delete: (id: string) => api.delete<null>(`/users/${id}`),
  roleCounts: () => api.get<Record<string, number>>('/users/role-counts'),
};

// ── Investors -------------------------------------------------
export const investorsApi = {
  list: () => api.get<any[]>('/investors'),
};

// ── Products --------------------------------------------------
export const productsApi = {
  list: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<Product[]>(`/products${q}`);
  },
  get:    (id: string)    => api.get<{ product: Product; variants: any[] }>(`/products/${id}`),
  create: (body: unknown) => api.post<Product>('/products', body),
  update: (id: string, body: unknown) => api.put<Product>(`/products/${id}`, body),
  delete: (id: string)    => api.delete<null>(`/products/${id}`),
};

// ── Variants --------------------------------------------------
export const variantsApi = {
  list:   (productId: string) => api.get<any[]>(`/products/${productId}/variants`),
  create: (productId: string, body: unknown) => api.post<any>(`/products/${productId}/variants`, body),
  update: (productId: string, id: string, body: unknown) => api.put<any>(`/products/${productId}/variants/${id}`, body),
  delete: (productId: string, id: string) => api.delete<null>(`/products/${productId}/variants/${id}`),
};

// ── Transactions ----------------------------------------------
export const transactionsApi = {
  list: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<Transaction[]>(`/transactions${q}`);
  },
  get:    (id: string)    => api.get<{ transaction: Transaction; items: any[]; payments: any[] }>(`/transactions/${id}`),
  create: (body: unknown) => api.post<{ transaction: Transaction; invoice: string }>('/transactions', body),
  refund: (id: string)    => api.post<null>(`/transactions/${id}/refund`, {}),
  reprint:(id: string)    => api.post<any[]>(`/transactions/${id}/reprint`, {}),
};

// ── Tickets ---------------------------------------------------
export const ticketsApi = {
  list:     ()            => api.get<Ticket[]>('/tickets'),
  get:      (id: string)  => api.get<Ticket>(`/tickets/${id}`),
  scan:     (body: { code: string; scan_method: string }) =>
    api.post<{ result: string; ticket: Ticket | null; message: string }>('/tickets/scan', body),
  validate: (code: string) =>
    api.post<{ result: string; ticket: Ticket | null }>('/tickets/validate', { code, scan_method: 'manual' }),
};

// ── Reports ---------------------------------------------------
export const reportsApi = {
  sales: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<SalesReport[]>(`/reports/sales${q}`);
  },
  products: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<ProductReport[]>(`/reports/products${q}`);
  },
  investors: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<any[]>(`/reports/investors${q}`);
  },
  paymentMethods: (params?: Record<string, string>) => {
    const q = params ? '?' + new URLSearchParams(params).toString() : '';
    return api.get<any[]>(`/reports/payment-methods${q}`);
  },
};

// ── Settings --------------------------------------------------
export const settingsApi = {
  get:    () => api.get<AppSettings>('/settings'),
  update: (body: unknown) => api.put<null>('/settings', body),
};
