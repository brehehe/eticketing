// ============================================================
// TiketKu — Core Type Definitions
// ============================================================

export type Theme = 'light' | 'dark' | 'system';
export type UserRole = 'admin' | 'kasir' | 'officer' | 'investor';
export type TicketStatus = 'active' | 'used' | 'expired' | 'refunded';
export type TxStatus = 'pending' | 'paid' | 'refunded' | 'cancelled';
export type PaymentMethod = 'cash' | 'qris' | 'transfer' | 'debit' | 'ewallet';
export type PrintJobStatus = 'queued' | 'printing' | 'done' | 'failed';
export type BluetoothState = 'disconnected' | 'connecting' | 'connected' | 'error';

// ── Auth ----------------------------------------------------------------
export interface User {
  id: string;
  name: string;
  username: string;
  email: string;
  role: UserRole;
  status: 'active' | 'inactive';
  avatar?: string;
  last_login?: string;
  device_count: number;
  created_at: string;
}

export interface Session {
  user: User;
  token: string;
  expires_at: string;
  device_id: string;
}

export interface Permission {
  id: string;
  module: string;
  action: string;
  label: string;
}

export interface Role {
  id: string;
  name: string;
  slug: UserRole;
  description: string;
  permissions: string[];
  user_count: number;
}

// ── Product ------------------------------------------------------------
export interface Category {
  id: string;
  name: string;
  slug: string;
  icon?: string;
  product_count: number;
}

export interface ProductVariant {
  id: string;
  product_id: string;
  name: string;
  sku: string;
  price: number;
  investor_price: number;
  quota?: number;
  quota_used: number;
  status: 'active' | 'inactive';
}

export interface Product {
  id: string;
  name: string;
  sku: string;
  category_id: string;
  category?: Category;
  price: number;
  investor_price: number;
  investor_id?: string;
  investor?: Investor;
  quota?: number;
  quota_used: number;
  schedule?: string;
  status: 'active' | 'inactive' | 'draft';
  has_variants: boolean;
  variants?: ProductVariant[];
  ticket_required: boolean;
  print_enabled: boolean;
  scan_enabled: boolean;
  barcode?: string;
  qr_code?: string;
  image?: string;
  created_at: string;
}

// ── Investor -----------------------------------------------------------
export interface Investor {
  id: string;
  name: string;
  email: string;
  phone?: string;
  status: 'active' | 'inactive';
  products?: Product[];
  total_revenue: number;
  created_at: string;
}

// ── Transaction --------------------------------------------------------
export interface CartItem {
  product: Product;
  variant?: ProductVariant;
  qty: number;
  unit_price: number;
  discount: number;
  subtotal: number;
}

export interface TransactionItem {
  id: string;
  transaction_id: string;
  product_id: string;
  product?: Product;
  variant_id?: string;
  variant?: ProductVariant;
  qty: number;
  unit_price: number;
  discount: number;
  subtotal: number;
}

export interface Payment {
  id: string;
  transaction_id: string;
  method: PaymentMethod;
  provider?: string;
  amount: number;
  fee: number;
  ref?: string;
  status: 'pending' | 'paid' | 'failed';
  paid_at?: string;
}

export interface Transaction {
  id: string;
  invoice: string;
  cashier_id: string;
  cashier?: User;
  status: TxStatus;
  items: TransactionItem[];
  payments: Payment[];
  subtotal: number;
  discount: number;
  total: number;
  change: number;
  note?: string;
  created_at: string;
  paid_at?: string;
}

// ── Ticket -------------------------------------------------------------
export interface Ticket {
  id: string;
  code: string;
  qr_data: string;
  transaction_id: string;
  product_id: string;
  product?: Product;
  variant_id?: string;
  visitor_name?: string;
  status: TicketStatus;
  valid_until?: string;
  used_at?: string;
  used_by?: string;
  created_at: string;
}

export interface TicketScan {
  id: string;
  ticket_id: string;
  ticket?: Ticket;
  officer_id: string;
  officer?: User;
  scan_method: 'camera' | 'bluetooth' | 'manual';
  result: 'valid' | 'invalid' | 'expired' | 'already_used';
  note?: string;
  scanned_at: string;
}

// ── Print --------------------------------------------------------------
export interface PrintJob {
  id: string;
  type: 'receipt' | 'ticket';
  payload: ReceiptPayload | TicketPayload;
  status: PrintJobStatus;
  printer_id?: string;
  retries: number;
  created_at: string;
}

export interface ReceiptPayload {
  transaction: Transaction;
  venue: VenueSettings;
}

export interface TicketPayload {
  ticket: Ticket;
  venue: VenueSettings;
}

// ── Reports ------------------------------------------------------------
export interface SalesReport {
  date: string;
  total_transactions: number;
  total_revenue: number;
  total_refunds: number;
  net_revenue: number;
}  

export interface ProductReport {
  product_id: string;
  product_name: string;
  category: string;
  qty_sold: number;
  revenue: number;
  investor_price: number;
  total_hpp: number;
  net_profit: number;
  refunds: number;
  growth_pct: number;
}

// ── Settings -----------------------------------------------------------
export interface VenueSettings {
  logo?: string;
  name: string;
  address: string;
  phone: string;
  email: string;
  operating_hours: string;
}

export interface TicketSettings {
  print_enabled: boolean;
  qr_enabled: boolean;
  expiry_hours: number;
  ticket_prefix: string;
}

export interface PrinterSettings {
  default_printer_id?: string;
  auto_retry: boolean;
  retry_count: number;
  paper_width: '58mm' | '80mm';
}

export interface ShiftSettings {
  shift_enabled: boolean;
  auto_closing: boolean;
  opening_balance: number;
}

export interface SystemSettings {
  theme: Theme;
  session_timeout_min: number;
  redis_url?: string;
  cache_ttl_sec: number;
}

export interface AppSettings {
  venue: VenueSettings;
  ticket: TicketSettings;
  printer: PrinterSettings;
  shift: ShiftSettings;
  system: SystemSettings;
}

// ── Bluetooth ----------------------------------------------------------
export interface BluetoothDevice {
  id: string;
  name: string;
  address: string;
  type: 'printer' | 'scanner';
  state: BluetoothState;
  last_connected?: string;
  is_default: boolean;
}

// ── Dashboard ----------------------------------------------------------
export interface DashboardStats {
  sales_today: number;
  visitors_today: number;
  tickets_sold: number;
  top_product: string;
  revenue_today: number;
  printer_status: BluetoothState;
  bluetooth_status: BluetoothState;
  scan_status: 'online' | 'offline';
  investor_revenue: number;
  peak_hour: string;
  sales_chart: { hour: string; revenue: number }[];
  revenue_chart: { date: string; revenue: number }[];
}

// ── API helpers --------------------------------------------------------
export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string;
  errors?: Record<string, string[]>;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface ApiError {
  status: number;
  message: string;
  errors?: Record<string, string[]>;
}
