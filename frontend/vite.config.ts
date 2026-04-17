import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

// Config du proxy `/api → :3000` partagée entre `vite dev` et `vite preview`.
// Sans preview proxy, Playwright (qui lance `npm run preview` sur :4173) ne
// peut pas acheminer `/api/v1/*` vers le backend kesh-api (:3000) → tous les
// appels API du frontend et des tests retournent 404/401. Corrigé Story 6-4.
const apiProxy = {
	'/api': {
		target: 'http://localhost:3000',
		changeOrigin: true
	}
};

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: apiProxy
	},
	preview: {
		proxy: apiProxy
	},
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
	}
});
