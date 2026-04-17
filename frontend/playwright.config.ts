import type { PlaywrightTestConfig } from '@playwright/test';

// D4 (review pass 1 G2 D) : force locale `fr-CH` + TZ Europe/Zurich pour
// les e2e — évite la flakiness CI quand le navigateur hérite de `en-US`
// (les regex multi-locale de tests Story 5.4 risquent de matcher la
// mauvaise chaîne ou de nécessiter des assertions inutilement larges).
const config: PlaywrightTestConfig = {
	webServer: {
		command: 'npm run build && npm run preview',
		port: 4173
	},
	testDir: 'tests/e2e',
	testMatch: /(.+\.)?(test|spec)\.[jt]s/,
	// Story 6.4 T7.7 : fail-fast si backend/KESH_TEST_MODE pas configuré.
	// S'exécute une seule fois avant tous les workers (évite race condition
	// vs un spec `_smoke` parallélisable).
	globalSetup: './tests/e2e/global-setup.ts',
	use: {
		locale: 'fr-CH',
		timezoneId: 'Europe/Zurich',
	},
};

export default config;
