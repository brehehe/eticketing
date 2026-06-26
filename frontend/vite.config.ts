import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { VitePWA as SvelteKitPWA } from 'vite-plugin-pwa';
import basicSsl from '@vitejs/plugin-basic-ssl';
import fs from 'fs';

const sslKeyPath = process.env.SSL_KEY_PATH;
const sslCertPath = process.env.SSL_CERT_PATH;
const hasCustomSsl = sslKeyPath && sslCertPath && fs.existsSync(sslKeyPath) && fs.existsSync(sslCertPath);

const plugins = [
	sveltekit(),
	SvelteKitPWA({
		registerType: 'autoUpdate',
		manifest: {
			name: 'TiketKu — Ticketing Management',
			short_name: 'TiketKu',
			description: 'Modern web-based ticketing & POS system',
			theme_color: '#6366f1',
			background_color: '#ffffff',
			display: 'standalone',
			orientation: 'portrait',
			icons: [
				{ src: '/icons/icon-192.png', sizes: '192x192', type: 'image/png' },
				{ src: '/icons/icon-512.png', sizes: '512x512', type: 'image/png' }
			]
		},
		workbox: {
			globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}'],
			runtimeCaching: [
				{
					urlPattern: /^https:\/\/fonts\.googleapis\.com\/.*/i,
					handler: 'CacheFirst',
					options: { cacheName: 'google-fonts-cache', expiration: { maxAgeSeconds: 60 * 60 * 24 * 365 } }
				}
			]
		}
	})
];

if (!hasCustomSsl) {
	plugins.push(basicSsl());
}

export default defineConfig({
	plugins,
	resolve: {
		conditions: ['browser', 'import', 'module', 'default'],
	},
	optimizeDeps: {
		include: [
			'@point-of-sale/webbluetooth-receipt-printer',
			'@point-of-sale/receipt-printer-encoder',
		]
	},
	server: {
		port: 5173,
		host: true,
		https: hasCustomSsl
			? {
					key: fs.readFileSync(sslKeyPath!),
					cert: fs.readFileSync(sslCertPath!)
			  }
			: (true as any),
		proxy: {
			'/api': {
				target: 'http://127.0.0.1:8081',
				changeOrigin: true,
				rewrite: (path) => path
			}
		}
	}
});
