// Disable SSR for all app routes — ensures client-side navigation
// preserves reactive stores like $page across route changes
export const ssr = false;
export const prerender = false;
