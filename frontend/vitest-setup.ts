/**
 * Setup vitest partagé — chargé avant chaque fichier de test.
 *
 * ⚠️ **Sa seule raison d'être : neutraliser la KF #175**, un échec de CI qui
 * n'est PAS un test rouge. Les 512 tests passent ; vitest sort néanmoins en `1`
 * parce qu'une erreur non capturée est levée **hors de tout test**, après le
 * démontage de l'environnement jsdom :
 *
 * ```
 * ReferenceError: document is not defined
 *   ❯ Proxy.resetBodyStyle bits-ui/dist/internal/body-scroll-lock.svelte.js:34
 *   ❯ Timeout.cleanupFn      bits-ui/dist/internal/body-scroll-lock.svelte.js:69
 * ```
 *
 * **Le mécanisme, lu dans le source de `bits-ui`.** Quand une modale se démonte,
 * la bibliothèque ne restaure pas le style du `body` sur-le-champ : elle
 * **programme** un `setTimeout` de **24 ms**, délibérément, pour absorber les
 * cycles démontage/remontage d'un même tick (leur issue #1639). Si jsdom est
 * démonté dans cette fenêtre, la minuterie se réveille dans un monde sans
 * `document`.
 *
 * ⚠️ **Démonter explicitement les composants n'y change RIEN** — c'est le
 * démontage lui-même qui arme la minuterie. C'était la première piste
 * envisagée ; la lecture du source l'a écartée.
 *
 * **Ce qui marche** : laisser s'écouler la fenêtre pendant que `document` existe
 * encore. Un `afterAll` suffit, et il est placé là plutôt que dans un `afterEach`
 * pour une raison de coût : l'environnement jsdom est démonté **une fois par
 * fichier**, pas une fois par test. Le prix est donc de 63 attentes et non de
 * 512 — quelques dizaines de millisecondes sur toute la suite, au lieu d'une
 * quinzaine de secondes.
 */
import { afterAll } from 'vitest';

/** 24 ms chez `bits-ui`, plus une marge pour l'ordonnanceur. */
const BITS_UI_CLEANUP_DELAY_MS = 40;

afterAll(async () => {
	await new Promise((resolve) => setTimeout(resolve, BITS_UI_CLEANUP_DELAY_MS));
});
