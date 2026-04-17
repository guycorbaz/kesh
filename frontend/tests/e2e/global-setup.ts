/**
 * Story 6.4 T7.7 — globalSetup Playwright.
 *
 * Exécuté **une seule fois avant TOUS les workers** (vs un spec `_smoke`
 * qui tournerait en parallèle avec les autres specs → race condition
 * destructive entre `beforeAll(seedTestState('fresh'))` et
 * `beforeAll(seedTestState('with-company'))` sur la même DB partagée).
 *
 * But : **fail-fast check de connectivité** — valide que le backend est
 * démarré, que `KESH_TEST_MODE` est actif, et que l'endpoint répond AVANT
 * que chaque spec tente son propre `beforeAll: seedTestState(...)`. Le
 * message d'erreur liste les 4 prérequis (backend up, `KESH_TEST_MODE`,
 * `KESH_HOST` loopback, `KESH_BACKEND_URL`) pour que le dev voie tout de
 * suite ce qui manque.
 *
 * **Pas une garantie d'état** (code review P10) : le preset `with-company`
 * seedé ici est considéré comme un check "best-effort". Chaque spec est
 * responsable de re-seeder son preset propre dans son `beforeAll` (ou
 * `beforeEach` pour onboarding). Si `globalSetup` échoue partiellement
 * (ex: truncate ok mais `mark_onboarding_complete` timeout), le spec
 * suivant re-seedera son état propre et corrigera la dérive.
 */

import { seedTestState } from './helpers/test-state';

async function globalSetup(): Promise<void> {
	try {
		await seedTestState('with-company');
	} catch (e) {
		console.error(
			'\n❌ FATAL: globalSetup Playwright a échoué.\n' +
				"   Vérifier que :\n" +
				'   1. Le backend kesh-api est démarré dans un terminal séparé\n' +
				'      (ex: cargo run -p kesh-api)\n' +
				'   2. KESH_TEST_MODE=true est dans l\'env du backend\n' +
				'   3. KESH_HOST=127.0.0.1 (sinon refus démarrage par ConfigError)\n' +
				`   4. Le backend répond sur ${process.env.KESH_BACKEND_URL ?? 'http://127.0.0.1:3000'}\n`,
			e,
		);
		throw e;
	}
}

export default globalSetup;
