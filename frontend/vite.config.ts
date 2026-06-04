import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// Config du proxy `/api` → backend kesh-api partagée entre `vite dev` et
// `vite preview`. Sans preview proxy, Playwright (qui lance `npm run preview`
// sur :4173) ne peut pas acheminer `/api/v1/*` vers le backend kesh-api →
// tous les appels API du frontend et des tests retournent 404/401. Corrigé
// Story 6-4.
//
// Story 10.3 : `/health` est aussi proxifié — sans ça, `npm run dev` bascule
// en état dégradé perpétuel car le boot-ping `fetch('/health')` du layout
// root (cf. `+layout.svelte` onMount) ainsi que `pollHealth()` hit
// `:5173/health` au lieu du backend → 404 vite → `setDegraded()` fire →
// DegradedBanner permanent même quand kesh-api backend est healthy.
//
// Story v011-4 (port 80 par défaut) : la cible du proxy est désormais
// env-driven. Par défaut `http://localhost` (port 80, mode Docker ; cohérent
// avec le nouveau défaut applicatif). En mode `cargo run` natif sur Linux
// non-root, le backend doit tourner sur un port ≥ 1024 ; le dev set alors
// `KESH_BACKEND_URL=http://localhost:<port>` (cf. `.env.example` § « Conflit
// port 80 » option d) dans son env shell avant `npm run dev` pour aligner le
// proxy. Note : `process.env` est évalué
// au lancement de vite, donc la variable doit être exportée avant la commande
// (pas via `.env` PUBLIC_* qui ne sont pas chargés par défaut dans vite.config).
const PROXY_TARGET = process.env.KESH_BACKEND_URL ?? 'http://localhost';
const apiProxy = {
	'/api': {
		target: PROXY_TARGET,
		changeOrigin: true
	},
	'/health': {
		target: PROXY_TARGET,
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
	// #159 : la version affichée provient désormais du backend au runtime
	// (champ `version` de `GET /health` → store `app-version.svelte.ts`), source
	// unique = `crates/kesh-api/Cargo.toml` via `env!("CARGO_PKG_VERSION")`. Plus
	// de `__APP_VERSION__` build-time depuis `package.json` (qui dérivait, figé à
	// 0.1.0). Cf. `app-version.svelte.ts`.
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
