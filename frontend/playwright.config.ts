import type { PlaywrightTestConfig } from '@playwright/test';

// D4 (review pass 1 G2 D) : force locale `fr-CH` + TZ Europe/Zurich pour
// les e2e — évite la flakiness CI quand le navigateur hérite de `en-US`
// (les regex multi-locale de tests Story 5.4 risquent de matcher la
// mauvaise chaîne ou de nécessiter des assertions inutilement larges).
//
// Story 6.4 (bugfix post-CI 24573000451) : Playwright cible le backend
// kesh-api sur `:3000` qui sert AUSSI la SPA statique (via `ServeDir`
// fallback configuré par `KESH_STATIC_DIR`). Avant, Playwright ciblait
// `vite preview` sur `:4173` qui n'a pas de proxy fonctionnel vers
// `/api/v1/*` en mode preview (seul `vite dev` avait le proxy) — d'où
// les 58/72 échecs CI avec `createContact failed: 401` et redirections
// `/login` silencieuses. Cibler directement le backend qui sert la SPA
// élimine le besoin de preview + proxy, et reflète la topologie prod.
//
// Pour le dev local : `cargo run -p kesh-api` (avec `KESH_STATIC_DIR=
// ../frontend/build` + `KESH_TEST_MODE=true`) puis `npm run test:e2e`.
// Cf. docs/testing.md section « Prérequis Playwright local ».
const config: PlaywrightTestConfig = {
	testDir: 'tests/e2e',
	testMatch: /(.+\.)?(test|spec)\.[jt]s/,
	// Story 6.4 T7.7 : fail-fast si backend/KESH_TEST_MODE pas configuré.
	// S'exécute une seule fois avant tous les workers (évite race condition
	// vs un spec `_smoke` parallélisable).
	globalSetup: './tests/e2e/global-setup.ts',
	use: {
		baseURL: process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000',
		locale: 'fr-CH',
		timezoneId: 'Europe/Zurich',
	},
};

export default config;
