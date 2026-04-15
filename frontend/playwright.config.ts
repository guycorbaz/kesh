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
	use: {
		locale: 'fr-CH',
		timezoneId: 'Europe/Zurich',
	},
};

export default config;
