<script lang="ts">
  export let data: any = {};
  export let form: any = undefined;
  export let params: any = undefined;
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { authApi } from '$lib/api/auth';
  import { toast } from '$lib/stores/toast';
  import { getDeviceId } from '$lib/utils/device';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import { theme } from '$lib/stores/theme';
  import { Eye, EyeOff, Moon, Sun } from 'lucide-svelte';
  import { onMount } from 'svelte';

  // Action to toggle password visibility (avoids dynamic `type` binding)
  function togglePasswordType(node: HTMLInputElement, show: boolean) {
    node.type = show ? 'text' : 'password';
    return {
      update(show: boolean) { node.type = show ? 'text' : 'password'; }
    };
  }

  let username = '';
  let password = '';
  let remember = false;
  let showPass = false;
  let loading = false;
  let step: 'login' | '2fa' | 'forgot' = 'login';
  let twoFACode = '';
  let sessionToken = '';
  let forgotEmail = '';
  let forgotSent = false;
  let errors: Record<string, string> = {};

  onMount(() => {
    theme.init();
    auth.init();
    import('$lib/stores/auth').then(({ isAuthenticated }) => {
      isAuthenticated.subscribe(v => { if (v) goto('/dashboard'); });
    });
  });

  async function handleLogin() {
    errors = {};
    if (!username.trim()) { errors.username = 'Username wajib diisi'; return; }
    if (!password)        { errors.password = 'Password wajib diisi'; return; }

    loading = true;
    try {
      const res = await authApi.login({
        username: username.trim(),
        password,
        remember,
        device_id: getDeviceId(),
      });
      auth.login(res.data);
      toast.success('Selamat datang!', res.data.user.name);
      goto('/dashboard');
    } catch (err: any) {
      if (err.status === 403 && err.message?.includes('2fa')) {
        sessionToken = err.data?.session_token ?? '';
        step = '2fa';
      } else {
        toast.error('Login gagal', err.message ?? 'Periksa username dan password Anda');
        errors.form = err.message;
      }
    } finally {
      loading = false;
    }
  }

  async function handle2FA() {
    if (!twoFACode || twoFACode.length < 6) { errors.code = 'Masukkan 6 digit kode'; return; }
    loading = true;
    try {
      const res = await authApi.verify2FA({ session_token: sessionToken, code: twoFACode });
      auth.login(res.data);
      goto('/dashboard');
    } catch (err: any) {
      errors.code = err.message ?? 'Kode tidak valid';
    } finally {
      loading = false;
    }
  }

  async function handleForgot() {
    if (!forgotEmail.includes('@')) { errors.email = 'Email tidak valid'; return; }
    loading = true;
    try {
      await authApi.forgotPassword(forgotEmail);
      forgotSent = true;
    } catch (err: any) {
      errors.email = err.message;
    } finally {
      loading = false;
    }
  }

  $: isDark = $theme === 'dark';
</script>

<svelte:head>
  <title>Login — TiketKu</title>
</svelte:head>

<Toast />

<div class="login-page">
  <!-- Background decoration -->
  <div class="bg-decoration" aria-hidden="true">
    <div class="bg-blob bg-blob-1"></div>
    <div class="bg-blob bg-blob-2"></div>
  </div>

  <!-- Theme toggle -->
  <button
    class="theme-toggle btn btn-ghost btn-icon"
    on:click={theme.toggle}
    aria-label="Toggle tema"
  >
    {#if isDark}
      <Sun size={18} />
    {:else}
      <Moon size={18} />
    {/if}
  </button>

  <!-- Card -->
  <div class="login-card glass fade-in">
    <!-- Logo -->
    <div class="login-brand">
      <div class="brand-logo" aria-hidden="true">
        <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
          <rect width="40" height="40" rx="12" fill="var(--brand-500)"/>
          <path d="M12 20h16M20 12l8 8-8 8" stroke="#fff" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div>
        <h1 class="brand-name">TiketKu</h1>
        <p class="brand-sub">Ticketing Management System</p>
      </div>
    </div>

    <div class="login-divider"></div>

    {#if step === 'login'}
      <!-- Login Form -->
      <form on:submit|preventDefault={handleLogin} novalidate>
        <h2 class="form-title">Masuk ke akun</h2>
        <p class="form-sub">Gunakan kredensial yang diberikan administrator</p>

        <div class="form-fields">
          <div class="field">
            <label class="label" for="username">Username atau Email</label>
            <input
              id="username"
              class="input input-lg"
              class:error={errors.username}
              type="text"
              bind:value={username}
              placeholder="Masukkan username"
              autocomplete="username"
              autocapitalize="none"
              spellcheck="false"
              disabled={loading}
              aria-describedby={errors.username ? 'username-err' : undefined}
            />
            {#if errors.username}
              <span class="field-error" id="username-err" role="alert">{errors.username}</span>
            {/if}
          </div>

          <div class="field">
            <div class="label-row">
              <label class="label" for="password">Password</label>
              <button
                type="button"
                class="forgot-link"
                on:click={() => { step = 'forgot'; errors = {}; }}
              >Lupa password?</button>
            </div>
            <div class="input-wrap">
              <input
                id="password"
                class="input input-lg"
                class:error={errors.password}
                use:togglePasswordType={showPass}
                bind:value={password}
                placeholder="Masukkan password"
                autocomplete="current-password"
                disabled={loading}
                aria-describedby={errors.password ? 'password-err' : undefined}
              />
              <button
                type="button"
                class="toggle-pass"
                on:click={() => (showPass = !showPass)}
                aria-label={showPass ? 'Sembunyikan password' : 'Tampilkan password'}
              >
                {#if showPass}
                  <EyeOff size={16} />
                {:else}
                  <Eye size={16} />
                {/if}
              </button>
            </div>
            {#if errors.password}
              <span class="field-error" id="password-err" role="alert">{errors.password}</span>
            {/if}
          </div>

          <label class="remember-row">
            <input type="checkbox" bind:checked={remember} />
            <span>Ingat perangkat ini</span>
          </label>
        </div>

        {#if errors.form}
          <div class="form-alert" role="alert">{errors.form}</div>
        {/if}

        <button
          type="submit"
          class="btn btn-primary btn-xl w-full"
          disabled={loading}
        >
          {#if loading}
            <Spinner size={18} color="#fff" />
            Memproses...
          {:else}
            Masuk
          {/if}
        </button>
      </form>

    {:else if step === '2fa'}
      <!-- 2FA Form -->
      <form on:submit|preventDefault={handle2FA} novalidate>
        <h2 class="form-title">Verifikasi 2FA</h2>
        <p class="form-sub">Masukkan kode dari aplikasi authenticator Anda</p>

        <div class="form-fields">
          <div class="field">
            <label class="label" for="code">Kode 6 digit</label>
            <input
              id="code"
              class="input input-lg mono"
              class:error={errors.code}
              style="letter-spacing:0.3em;font-size:1.25rem;text-align:center;"
              type="text"
              inputmode="numeric"
              maxlength={6}
              bind:value={twoFACode}
              placeholder="000000"
              disabled={loading}
              autocomplete="one-time-code"
            />
            {#if errors.code}
              <span class="field-error" role="alert">{errors.code}</span>
            {/if}
          </div>
        </div>

        <button type="submit" class="btn btn-primary btn-xl w-full" disabled={loading}>
          {#if loading}<Spinner size={18} color="#fff" />{/if}
          Verifikasi
        </button>
        <button type="button" class="btn btn-ghost w-full mt-2" on:click={() => { step = 'login'; twoFACode = ''; }}>Kembali</button>
      </form>

    {:else if step === 'forgot'}
      <!-- Forgot Password -->
      {#if forgotSent}
        <div class="forgot-success">
          <div class="success-icon" aria-hidden="true">
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
              <circle cx="24" cy="24" r="24" fill="rgba(16,185,129,0.1)"/>
              <path d="M16 24l6 6 10-12" stroke="var(--emerald)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <h2 class="form-title" style="text-align:center;">Email terkirim</h2>
          <p class="form-sub" style="text-align:center;">Cek inbox Anda untuk instruksi reset password</p>
          <button class="btn btn-secondary btn-lg w-full" on:click={() => { step = 'login'; forgotSent = false; }}>
            Kembali ke login
          </button>
        </div>
      {:else}
        <form on:submit|preventDefault={handleForgot} novalidate>
          <h2 class="form-title">Reset Password</h2>
          <p class="form-sub">Masukkan email terdaftar untuk instruksi reset</p>

          <div class="form-fields">
            <div class="field">
              <label class="label" for="forgot-email">Email</label>
              <input
                id="forgot-email"
                class="input input-lg"
                class:error={errors.email}
                type="email"
                bind:value={forgotEmail}
                placeholder="email@domain.com"
                autocomplete="email"
                disabled={loading}
              />
              {#if errors.email}
                <span class="field-error" role="alert">{errors.email}</span>
              {/if}
            </div>
          </div>

          <button type="submit" class="btn btn-primary btn-xl w-full" disabled={loading}>
            {#if loading}<Spinner size={18} color="#fff" />{/if}
            Kirim Instruksi
          </button>
          <button type="button" class="btn btn-ghost w-full mt-2" on:click={() => { step = 'login'; errors = {}; }}>Kembali</button>
        </form>
      {/if}
    {/if}
  </div>

  <p class="login-footer">
    &copy; {new Date().getFullYear()} TiketKu &mdash; All rights reserved
  </p>
</div>

<style>
  .login-page {
    min-height: 100dvh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 16px;
    position: relative;
    overflow: hidden;
    background: var(--bg-base);
  }

  /* Background blobs */
  .bg-decoration { position: fixed; inset: 0; pointer-events: none; z-index: 0; }
  .bg-blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(80px);
    opacity: 0.35;
  }
  .bg-blob-1 {
    width: 500px; height: 500px;
    background: var(--brand-200);
    top: -100px; right: -100px;
  }
  .bg-blob-2 {
    width: 400px; height: 400px;
    background: var(--violet);
    bottom: -100px; left: -80px;
    opacity: 0.15;
  }
  [data-theme='dark'] .bg-blob-1 { opacity: 0.12; }
  [data-theme='dark'] .bg-blob-2 { opacity: 0.07; }

  /* Theme toggle */
  .theme-toggle {
    position: fixed;
    top: 16px;
    right: 16px;
    z-index: 10;
    color: var(--text-2);
  }

  /* Card */
  .login-card {
    width: 100%;
    max-width: 420px;
    padding: 32px;
    position: relative;
    z-index: 1;
  }

  /* Brand */
  .login-brand {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 24px;
  }
  .brand-logo { flex-shrink: 0; }
  .brand-name {
    font-size: 1.375rem;
    font-weight: 700;
    letter-spacing: -0.025em;
    color: var(--text-1);
    margin: 0;
  }
  .brand-sub {
    font-size: 0.8125rem;
    color: var(--text-2);
    margin-top: 2px;
  }

  .login-divider {
    height: 1px;
    background: var(--border);
    margin-bottom: 24px;
  }

  /* Form */
  .form-title {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--text-1);
    margin-bottom: 4px;
  }
  .form-sub {
    font-size: 0.875rem;
    color: var(--text-2);
    margin-bottom: 24px;
  }
  .form-fields { display: flex; flex-direction: column; gap: 16px; margin-bottom: 20px; }

  .label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .forgot-link {
    font-size: 0.8125rem;
    color: var(--text-brand);
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font-family: var(--font-sans);
    transition: opacity var(--ease-fast);
  }
  .forgot-link:hover { opacity: 0.75; }

  .input-wrap { position: relative; }
  .input-wrap .input { padding-right: 42px; }
  .toggle-pass {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-3);
    cursor: pointer;
    border: none;
    background: none;
    display: flex;
    padding: 4px;
    border-radius: var(--r-sm);
    transition: color var(--ease-fast);
  }
  .toggle-pass:hover { color: var(--text-1); }

  .remember-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.875rem;
    color: var(--text-2);
    cursor: pointer;
    user-select: none;
  }
  .remember-row input { accent-color: var(--brand-500); cursor: pointer; }

  .form-alert {
    padding: 10px 14px;
    border-radius: var(--r-md);
    background: rgba(244,63,94,0.08);
    border: 1px solid rgba(244,63,94,0.2);
    color: var(--rose);
    font-size: 0.875rem;
    margin-bottom: 16px;
  }

  .w-full { width: 100%; }
  .mt-2   { margin-top: 8px; }

  /* Forgot success */
  .forgot-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .success-icon { margin-bottom: 4px; }

  /* Footer */
  .login-footer {
    margin-top: 32px;
    font-size: 0.8125rem;
    color: var(--text-3);
    z-index: 1;
    position: relative;
  }

  @media (max-width: 480px) {
    .login-card { padding: 24px 20px; }
  }
</style>
