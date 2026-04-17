/**
 * Story 6.4 T7.7 — globalSetup Playwright.
 *
 * Exécuté **une seule fois avant TOUS les workers** (vs un spec `_smoke`
 * qui tournerait en parallèle avec les autres specs → race condition
 * destructive entre `beforeAll(seedTestState('fresh'))` et
 * `beforeAll(seedTestState('with-company'))` sur la même DB partagée).
 *
 * But : fail-fast si le backend n'est pas joignable AVANT que chaque spec
 * tente son propre `beforeAll: seedTestState(...)`. Le message d'erreur
 * liste les 4 prérequis (backend up, KESH_TEST_MODE, KESH_HOST loopback,
 * BACKEND_URL) pour que le dev voie tout de suite ce qui manque — cf.
 * NEW-H1 pass 4.
 *
 * Preset `with-company` choisi car non-destructif : chaque spec re-seed
 * son state propre par-dessus dans son `beforeAll`, sans surprise.
 */

import { seedTestState } from './helpers/test-state';

// `@types/node` n'est pas installé — déclaration ambient minimale pour que
// `svelte-check` tolère l'usage de `process.env` (Playwright tourne sous Node).
declare const process: { env: { readonly [key: string]: string | undefined } };

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
