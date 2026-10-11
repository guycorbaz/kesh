/**
 * Garde d'inventaire clos — Story 26-2b (#622, AC11, C-26-2-18).
 *
 * Le symptôme qu'elle ferme : **un compte bancaire montré à un humain sans sa
 * description**, parce qu'un site lit `bankName` à la main au lieu de passer par
 * `bankAccountName` / `bankAccountOptionLabel` (`format.ts`).
 *
 * Méthode (CLAUDE.md § « Inventorier les sites NON RÉSOLUS ») : elle n'énumère
 * pas les formes correctes, elle compte le **jeton** `bankName` dans chaque
 * fichier source du frontend et exige que le résultat égale une table
 * **fermée** `{ fichier → nombre, motif }`. Tout `bankName` hors de la table —
 * balisage, attribut, `{@const}`, script, déstructuration, `.ts` — fait rougir.
 *
 * ## Ce qu'elle ne voit pas (angle mort assumé, C-26-2-18)
 *
 * Un libellé construit **sans** le jeton (`a['bank' + 'Name']`, ou un champ
 * serveur neuf portant le nom de la banque sous un autre nom). Le symptôme reste
 * alors couvert par les tests de site (AC9) et l'E2E.
 *
 * ## Ajouter ou déplacer un `bankName` légitime
 *
 * Mettre à jour la table **avec son motif** : c'est le but — le changement se
 * voit en revue. Nombres mesurés par
 * `grep -row bankName src --include=*.svelte --include=*.ts --include=*.js | grep -v "\.test\.\|\.spec\." | cut -d: -f1 | sort | uniq -c`
 * (occurrences, pas lignes).
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const RACINE = 'src';

/** Nombre d'occurrences du jeton `bankName` (la casse compte : `formBankName` n'est pas compté). */
export function compterJeton(source: string): number {
	return (source.match(/\bbankName\b/g) ?? []).length;
}

/** Fichiers source lus : `.svelte`, `.ts`, `.js`, hors tout nom contenant `.test.` ou `.spec.`. */
function fichiersSource(dir: string): string[] {
	const out: string[] = [];
	for (const entry of readdirSync(dir, { withFileTypes: true })) {
		const chemin = join(dir, entry.name);
		if (entry.isDirectory()) {
			out.push(...fichiersSource(chemin));
		} else if (
			/\.(svelte|ts|js)$/.test(entry.name) &&
			!entry.name.includes('.test.') &&
			!entry.name.includes('.spec.')
		) {
			out.push(chemin);
		}
	}
	return out;
}

/**
 * La table fermée — chemins relatifs à `src/`. Mesurée sur l'arbre de la 26-2b.
 */
const ATTENDU: Record<string, { n: number; motif: string }> = {
	'lib/features/bank-accounts/format.ts': {
		n: 3,
		motif: 'les fonctions de libellé et leur type source — le seul endroit qui compose le nom',
	},
	'lib/features/bank-accounts/bank-accounts.api.ts': {
		n: 6,
		motif: 'porteur : types de l’API, `parseBankAccount`, payloads',
	},
	'lib/features/settings/settings.types.ts': {
		n: 1,
		motif: 'porteur : `BankAccountJson` de `GET /companies/current`',
	},
	'routes/(app)/bank-accounts/+page.svelte': {
		n: 3,
		motif: 'E1 — préremplissage et payloads du formulaire (saisie, pas affichage)',
	},
	'routes/onboarding/+page.svelte': {
		n: 4,
		motif: 'E2 — saisie de la banque à l’onboarding, sans description (C-26-2-2)',
	},
	'lib/features/onboarding/onboarding.svelte.ts': { n: 2, motif: 'E2 — idem, état de l’onboarding' },
	'lib/features/onboarding/onboarding.api.ts': { n: 2, motif: 'E2 — idem, appel de l’API' },
	'lib/features/bank-import/BankProfileForm.svelte': {
		n: 4,
		motif: 'E3 — homonyme : nom d’un profil d’import CSV, pas un compte bancaire',
	},
	'lib/features/bank-import/BankProfileList.svelte': { n: 1, motif: 'E3 — homonyme (profil d’import)' },
	'lib/features/bank-import/BankProfileSelector.svelte': {
		n: 1,
		motif: 'E3 — homonyme (profil d’import)',
	},
	'lib/features/bank-import/bank-profile.types.ts': { n: 1, motif: 'E3 — homonyme (profil d’import)' },
};

/**
 * Nombre de fichiers lus, mesuré. ⚠️ Une **suppression de fichier sans
 * rapport** le fait rougir : il suffit alors d'abaisser le minimum à la
 * nouvelle mesure. Il existe pour qu'une collecte devenue vide ne passe pas à
 * vide (test muet).
 */
const FICHIERS_LUS_MIN = 316;

function releve(): Map<string, { n: number; lignes: number[] }> {
	const out = new Map<string, { n: number; lignes: number[] }>();
	for (const chemin of fichiersSource(RACINE)) {
		const source = readFileSync(chemin, 'utf8');
		const n = compterJeton(source);
		if (n === 0) continue;
		const lignes = source
			.split('\n')
			.flatMap((l, i) => (compterJeton(l) > 0 ? [i + 1] : []));
		out.set(chemin.slice(RACINE.length + 1).replaceAll('\\', '/'), { n, lignes });
	}
	return out;
}

describe('garde d’inventaire clos — le jeton bankName (Story 26-2b, AC11)', () => {
	it('la collecte ne passe pas à vide', () => {
		expect(fichiersSource(RACINE).length).toBeGreaterThanOrEqual(FICHIERS_LUS_MIN);
	});

	it('aucun fichier hors de la table ne lit bankName', () => {
		const horsTable = [...releve()]
			.filter(([f]) => !(f in ATTENDU))
			.map(([f, { n, lignes }]) => `${f} : ${n} occurrence(s), lignes ${lignes.join(', ')}`);
		expect(
			horsTable,
			'un site lit `bankName` à la main : passer par `bankAccountName` / `bankAccountOptionLabel` (format.ts), ou inscrire le fichier à la table avec son motif',
		).toEqual([]);
	});

	it('chaque fichier de la table compte exactement son nombre, et existe encore', () => {
		const r = releve();
		const ecarts = Object.entries(ATTENDU)
			.filter(([f, { n }]) => (r.get(f)?.n ?? 0) !== n)
			.map(([f, { n }]) => `${f} : attendu ${n}, trouvé ${r.get(f)?.n ?? 0}${r.has(f) ? '' : ' (fichier absent ou sans jeton : table périmée)'}`);
		expect(ecarts).toEqual([]);
	});

	it('la somme des occurrences égale la somme de la table', () => {
		const somme = [...releve().values()].reduce((a, { n }) => a + n, 0);
		const attendu = Object.values(ATTENDU).reduce((a, { n }) => a + n, 0);
		expect(somme).toBe(attendu);
	});
});

describe('garde d’inventaire clos — la fonction de décompte, une forme par cas', () => {
	it.each([
		['balisage', '<p>{a.bankName}</p>'],
		['attribut', '<span title={a.bankName}></span>'],
		['@const', '{@const l = `${a.bankName}`}'],
		['libellé construit en script', 'items.map((a) => ({ label: a.bankName }))'],
		['déstructuration', 'const { bankName } = a;'],
		['.ts', 'export const nom = (a: { bankName: string }) => a.iban;'],
	])('%s : compté 1', (_forme, source) => {
		expect(compterJeton(source)).toBe(1);
	});

	it('formBankName (casse différente) : compté 0', () => {
		expect(compterJeton('let formBankName = $state("");')).toBe(0);
	});
});
