/**
 * v0172-correctifs (#880) — épingle le fuseau du processus (`process.env.TZ`, que Node
 * relit à chaque calcul de date) le temps des tests du `describe` qui l'appelle, et le
 * rétablit après chacun (`delete` s'il était absent). Précédent :
 * `routes/(app)/bank-accounts/bank-accounts-page.test.ts`.
 *
 * Les écrans affichent les horodatages du serveur (UTC) à l'heure de Zurich : un test qui
 * en lit un épingle le processus à `UTC` (le fuseau du runner de la CI), pour que son
 * résultat ne dépende pas du poste.
 *
 * ⚠️ Module de test : il n'est importé que par des `*.test.ts`.
 */
import { afterEach, beforeEach } from 'vitest';

export function pinTimeZone(zone: string): void {
	let previous: string | undefined;
	beforeEach(() => {
		previous = process.env.TZ;
		process.env.TZ = zone;
	});
	afterEach(() => {
		if (previous === undefined) delete process.env.TZ;
		else process.env.TZ = previous;
	});
}
