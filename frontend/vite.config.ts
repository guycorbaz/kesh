import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// Config du proxy `/api → :3000` partagée entre `vite dev` et `vite preview`.
// Sans preview proxy, Playwright (qui lance `npm run preview` sur :4173) ne
// peut pas acheminer `/api/v1/*` vers le backend kesh-api (:3000) → tous les
// appels API du frontend et des tests retournent 404/401. Corrigé Story 6-4.
//
// Story 10.3 : `/health` est aussi proxifié — sans ça, `npm run dev` bascule
// en état dégradé perpétuel car le boot-ping `fetch('/health')` du layout
// root (cf. `+layout.svelte` onMount) ainsi que `pollHealth()` hit
// `:5173/health` au lieu de `:3000/health` → 404 vite → `setDegraded()` fire
// → DegradedBanner permanent même quand kesh-api backend est healthy.
const apiProxy = {
	'/api': {
		target: 'http://localhost:3000',
		changeOrigin: true
	},
	'/health': {
		target: 'http://localhost:3000',
		changeOrigin: true
	}
};

// H1 Pass 1 code review (Story 8-4) — `svelteTesting()` ajoute les
// resolve conditions browser pour `@testing-library/svelte` en
// jsdom (sans ce plugin, `mount(...)` plante avec
// `lifecycle_function_unavailable` car Svelte importe la version
// server-side par défaut).
export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), svelteTesting()],
	// Story 10.3 : injecte la version Kesh depuis `package.json` au build pour
	// l'afficher en pied de la page de login (preuve que le frontend est servi
	// correctement même DB down). Fallback `'dev'` pour les build hors-npm
	// (npx vite build sans cycle npm) — sans fallback, `__APP_VERSION__`
	// vaudrait le mot-clé JS `undefined` → render `Kesh vundefined`.
	define: {
		__APP_VERSION__: JSON.stringify(process.env.npm_package_version ?? 'dev')
	},
	server: {
		proxy: apiProxy
	},
	preview: {
		proxy: apiProxy
	},
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts', 'tests/**/*.test.ts'],
	}
});
