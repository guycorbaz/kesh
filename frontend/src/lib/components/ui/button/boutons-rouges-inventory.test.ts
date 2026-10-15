// @vitest-environment node
/**
 * Inventaire clos des boutons rouges posés à la main — Story 28-1 (#640, AC6,
 * C-28-1-11, C-28-1-22, C-28-1-28).
 *
 * La variante `destructive` du `Button` partagé doit être la SEULE source du
 * rouge d'un bouton. Cette garde relève, dans tous les `.svelte` de `src/`, chaque
 * `<button>` et chaque `<Button>` dont l'attribut `class` (littéral ou expression)
 * ou une directive `class:` porte un jeton rouge —
 * `(text|bg|border|ring|outline)-(destructive|error|red-NNN)(/n)?`, préfixes de
 * variante compris — et exige que l'ensemble `{ clé → jetons }` ÉGALE la table
 * fermée `ATTENDU` ci-dessous (patron de `bank-name-inventory.test.ts`, 26-2b).
 * Une entrée en trop, en moins, ou des jetons changés font rougir.
 *
 * Elle exige aussi (f) qu'aucun `<Button variant="destructive">` ne porte
 * d'attribut `class` ni de directive `class:` : c'est ce qui rend vrai, par
 * construction, que tous les usages de la variante rendent le style de la variante.
 *
 * ## Méthode
 *
 * Analyse par `svelte/compiler` (`parse(source, { modern: true })`), non par
 * expression régulière : un `onclick={() => …}` contient un `>`. La CLÉ d'un
 * élément est `<chemin relatif à src/>#<data-testid>` quand il en porte un — la
 * valeur écrite par son TEXTE SOURCE, sans normalisation des gabarits —, sinon
 * `<chemin>@<élément>:<rang>`, le rang comptant à partir de 1 les éléments sans
 * `data-testid` porteurs d'un jeton rouge du même fichier, dans l'ordre du source.
 * Pas de numéro de ligne : une édition sans rapport ferait rougir la garde. Deux
 * éléments de même clé font échouer la garde.
 *
 * ## Ce qu'elle ne voit pas (angles morts assumés)
 *
 * - un rouge passé par une variable de script, par `cn(…)` dans le `<script>`,
 *   ou par un `style=` ;
 * - un rouge porté par un `<a>` ou un `<div role="button">` (aucun au 2026-10-13,
 *   vérifié) ;
 * - un `variant="destructive"` RETIRÉ d'un usage : `variant=` n'est pas une
 *   classe (angle mort 7 de la fiche).
 *
 * ## Ajouter une entrée légitime
 *
 * L'inscrire à la table AVEC SON MOTIF : c'est le but — le changement se voit en
 * revue. Avant, se demander si la variante `destructive` ne convient pas.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { parse } from 'svelte/compiler';

const RACINE = 'src';

/** Jeton rouge, une fois retirés les préfixes de variante. */
const ROUGE = /^!?(text|bg|border|ring|outline)-(destructive|error|red-\d{2,3})(\/\d+)?$/;

interface Noeud {
	type: string;
	start: number;
	end: number;
	name?: string;
	attributes?: Attribut[];
	[k: string]: unknown;
}

interface Attribut {
	type: string;
	name: string;
	start: number;
	end: number;
	value?: true | Noeud | Noeud[];
}

/** Jetons rouges d'un texte de classes (préfixes de variante conservés). */
function jetonsRouges(texte: string): string[] {
	return texte
		.split(/[\s"'`{}()=,]+/)
		.filter((m) => m.length > 0 && ROUGE.test(m.slice(m.lastIndexOf(':') + 1)));
}

/** Tous les éléments `<button>` et composants `<Button>` du source, dans l'ordre. */
function elements(ast: unknown): Noeud[] {
	const out: Noeud[] = [];
	const vus = new Set<unknown>();
	const visiter = (n: unknown): void => {
		if (!n || typeof n !== 'object' || vus.has(n)) return;
		vus.add(n);
		if (Array.isArray(n)) {
			n.forEach(visiter);
			return;
		}
		const noeud = n as Noeud;
		if (
			(noeud.type === 'RegularElement' && noeud.name === 'button') ||
			(noeud.type === 'Component' && noeud.name === 'Button')
		) {
			out.push(noeud);
		}
		for (const v of Object.values(noeud)) visiter(v);
	};
	visiter(ast);
	return out.sort((a, b) => a.start - b.start);
}

/** Texte source de la valeur d'un attribut, guillemets exclus, accolades d'une expression seule comprises. */
function valeurSource(source: string, attr: Attribut): string | null {
	const v = attr.value;
	if (v === undefined || v === true) return null;
	if (Array.isArray(v)) {
		if (v.length === 0) return '';
		return source.slice(v[0].start, v[v.length - 1].end);
	}
	return source.slice(v.start, v.end);
}

/** Un bouton rouge relevé : sa clé et ses jetons. */
export interface BoutonRouge {
	cle: string;
	jetons: string[];
}

/**
 * Relève les boutons rouges d'un source Svelte. `chemin` est le chemin relatif à
 * `src/` qui préfixe les clés. Lève si deux éléments produisent la même clé.
 */
export function releverBoutonsRouges(source: string, chemin: string): BoutonRouge[] {
	const ast = parse(source, { modern: true });
	const out: BoutonRouge[] = [];
	const cles = new Set<string>();
	let rang = 0;
	for (const el of elements(ast)) {
		const jetons: string[] = [];
		let testid: string | null = null;
		for (const attr of el.attributes ?? []) {
			if (attr.type === 'Attribute' && attr.name === 'class') {
				jetons.push(...jetonsRouges(source.slice(attr.start, attr.end)));
			} else if (attr.type === 'ClassDirective' && ROUGE.test(attr.name)) {
				jetons.push(`class:${attr.name}`);
			} else if (attr.type === 'Attribute' && attr.name === 'data-testid') {
				testid = valeurSource(source, attr);
			}
		}
		if (jetons.length === 0) continue;
		const cle = testid !== null ? `${chemin}#${testid}` : `${chemin}@${el.name}:${++rang}`;
		if (cles.has(cle)) throw new Error(`clé en double dans l'inventaire des boutons rouges : ${cle}`);
		cles.add(cle);
		out.push({ cle, jetons });
	}
	return out;
}

/** Les `<Button variant="destructive">` du source, relevés par l'analyseur, dans l'ordre. */
export function boutonsDestructifs(source: string): Noeud[] {
	return elements(parse(source, { modern: true })).filter((el) => {
		if (el.type !== 'Component') return false;
		const variante = (el.attributes ?? []).find((a) => a.type === 'Attribute' && a.name === 'variant');
		return variante !== undefined && valeurSource(source, variante) === 'destructive';
	});
}

/**
 * `<Button variant="destructive">` qui portent un attribut `class` ou une directive
 * `class:` (AC6 (f) ; la directive depuis la revue de code P1, B2 : `class:bg-transparent={x}`
 * réécrirait le fond de la variante aussi sûrement qu'un attribut).
 */
export function destructifsAvecClasse(source: string, chemin: string): string[] {
	const out: string[] = [];
	for (const el of boutonsDestructifs(source)) {
		const attrs = el.attributes ?? [];
		if (attrs.some((a) => (a.type === 'Attribute' && a.name === 'class') || a.type === 'ClassDirective')) {
			out.push(`${chemin}:${source.slice(0, el.start).split('\n').length}`);
		}
	}
	return out;
}

function fichiersSvelte(dir: string): string[] {
	const out: string[] = [];
	for (const entry of readdirSync(dir, { withFileTypes: true })) {
		const chemin = join(dir, entry.name);
		if (entry.isDirectory()) out.push(...fichiersSvelte(chemin));
		else if (entry.name.endsWith('.svelte') && !entry.name.includes('.test.')) out.push(chemin);
	}
	return out;
}

function cheminRelatif(fichier: string): string {
	return relative(RACINE, fichier).split(sep).join('/');
}

/**
 * La table fermée — quatorze entrées, mesurées sur l'arbre de la 28-1 et
 * examinées une à une contre la fiche (Dev Notes § Inventaire, classes B à E).
 */
const ATTENDU: Record<string, { jetons: string[]; motif: string }> = {
	// Classe B — <Button> rouge par `class` : déjà visible, rouge de palette au
	// lieu de la variante (angle mort 11, #648).
	'routes/(app)/settings/fiscal-years/+page.svelte#fiscal-year-close-confirm': {
		jetons: ['bg-red-600', 'hover:bg-red-700'],
		motif: 'B — confirmation de clôture d\'exercice, rouge de palette (#648)',
	},
	'routes/(app)/settings/email-templates/+page.svelte#email-template-restore-confirm': {
		jetons: ['bg-red-600', 'hover:bg-red-700'],
		motif: 'B — confirmation de restauration d\'un modèle, rouge de palette (#648)',
	},
	// Classe C — <button> brut bordé, texte rouge : la bordure le fait lire comme
	// un bouton.
	'routes/(app)/settings/api-keys/+page.svelte#api-keys-revoke-{k.id}': {
		jetons: ['text-red-600'],
		motif: 'C — déclencheur de ligne bordé, ouvre une confirmation',
	},
	'routes/(app)/bank-accounts/+page.svelte#archive-button-{ba.id}': {
		jetons: ['text-red-600'],
		motif: 'C — déclencheur de ligne bordé, ouvre une confirmation',
	},
	'lib/features/reconciliation/rules/RulesList.svelte#delete-button-{r.id}': {
		jetons: ['border-error', 'text-error'],
		motif: 'C — déclencheur de ligne bordé (confirm() natif)',
	},
	'routes/(app)/payment-batches/[id]/+page.svelte#payment-batch-cancel': {
		jetons: ['text-destructive'],
		motif: 'C — action de page bordée ; rouge par le jeton défini (AC4)',
	},
	'routes/(app)/supplier-invoices/[id]/+page.svelte#supplier-invoice-settlement-cancel': {
		jetons: ['text-destructive'],
		motif: 'C — action de page bordée ; rouge par le jeton défini (AC4)',
	},
	'routes/(app)/supplier-invoices/[id]/+page.svelte#supplier-invoice-cancel': {
		jetons: ['text-destructive'],
		motif: 'C — action de page bordée ; rouge par le jeton défini (AC4)',
	},
	// Classe D — <button> brut sans bordure, texte rouge : se lit comme du texte
	// ou un lien (angle mort 11, #648).
	'lib/features/contacts/ContactPersonsManager.svelte#{`contact-person-delete-${p.id}`}': {
		jetons: ['text-red-600'],
		motif: 'D — suppression d\'une personne de contact, confirmée (#646) ; texte rouge sans bordure (#648)',
	},
	'lib/features/bank-import/BankProfileList.svelte#bank-import-profile-delete-button': {
		jetons: ['text-error'],
		motif: 'D — suppression d\'un profil d\'import (#648)',
	},
	'routes/(app)/bank-import/[id]/+page.svelte#detail-tx-cancel-reconciliation': {
		jetons: ['text-destructive'],
		motif: 'D — annuler le rapprochement, texte souligné ; rouge par le jeton (AC4, #648)',
	},
	// Classe E — icônes ✕ de retrait d'une ligne de saisie non enregistrée :
	// réversible, contrôle iconique.
	'routes/(app)/supplier-invoices/+page.svelte@button:1': {
		jetons: ['text-destructive'],
		motif: 'E — ✕ de retrait d\'une ligne de saisie',
	},
	'routes/(app)/supplier-invoices/import/+page.svelte@button:1': {
		jetons: ['text-destructive'],
		motif: 'E — ✕ de retrait d\'une ligne de saisie',
	},
	'lib/features/journal-entries/JournalEntryForm.svelte@button:1': {
		jetons: ['hover:text-destructive'],
		motif: 'E — icône X de retrait d\'une ligne, rouge au survol seulement',
	},
};

describe('relevé — auto-tests sur source synthétique', () => {
	it('une directive class: est relevée', () => {
		expect(
			releverBoutonsRouges('<button class:text-destructive={x}>x</button>', 'x.svelte'),
		).toEqual([{ cle: 'x.svelte@button:1', jetons: ['class:text-destructive'] }]);
	});

	it('deux <button> sans data-testid prennent les rangs 1 et 2', () => {
		const src = '<button class="text-error">a</button><button class="text-error">b</button>';
		expect(releverBoutonsRouges(src, 'x.svelte').map((b) => b.cle)).toEqual([
			'x.svelte@button:1',
			'x.svelte@button:2',
		]);
	});

	it('deux éléments au même data-testid font lever la garde', () => {
		const src =
			'<button class="text-error" data-testid="t">a</button>' +
			'<Button class="bg-red-600" data-testid="t">b</Button>';
		expect(() => releverBoutonsRouges(src, 'x.svelte')).toThrow(/x\.svelte#t/);
	});

	it('une valeur de data-testid en expression seule garde ses accolades', () => {
		const src = '<button class="text-error" data-testid={`a-${i}`}>a</button>';
		expect(releverBoutonsRouges(src, 'x.svelte')[0].cle).toBe('x.svelte#{`a-${i}`}');
	});
});

describe('(f) — auto-tests sur source synthétique (revue de code P1, B2 et E3)', () => {
	it('un <Button variant="destructive" class="…"> est fautif, à sa ligne', () => {
		const src = '<p>a</p>\n<Button variant="destructive" class="x">b</Button>';
		expect(destructifsAvecClasse(src, 'x.svelte')).toEqual(['x.svelte:2']);
	});

	it('une directive class: sur un <Button variant="destructive"> est fautive', () => {
		const src = '<Button variant="destructive" class:bg-transparent={x}>b</Button>';
		expect(destructifsAvecClasse(src, 'x.svelte')).toEqual(['x.svelte:1']);
	});

	it('sans class ni directive, ou sur une autre variante, rien n’est fautif', () => {
		const src =
			'<Button variant="destructive" onclick={() => f(a > b)}>a</Button>' +
			'<Button variant="outline" class="x">b</Button>' +
			'<button class="x">c</button>';
		expect(destructifsAvecClasse(src, 'x.svelte')).toEqual([]);
		expect(boutonsDestructifs(src)).toHaveLength(1);
	});
});

describe('AC6 — inventaire clos des boutons rouges posés à la main', () => {
	const fichiers = fichiersSvelte(RACINE);

	it('égale la table fermée (entrée en trop, en moins, jetons changés)', () => {
		const releve: Record<string, string[]> = {};
		for (const f of fichiers) {
			for (const b of releverBoutonsRouges(readFileSync(f, 'utf8'), cheminRelatif(f))) {
				releve[b.cle] = b.jetons;
			}
		}
		const attendu = Object.fromEntries(Object.entries(ATTENDU).map(([k, v]) => [k, v.jetons]));
		expect(releve).toEqual(attendu);
	});

	it('la table compte quatorze entrées, chacune avec son motif', () => {
		expect(Object.keys(ATTENDU)).toHaveLength(14);
		for (const [cle, { motif }] of Object.entries(ATTENDU)) {
			expect(motif.trim().length, cle).toBeGreaterThan(0);
		}
	});

	it('(f) aucun <Button variant="destructive"> ne porte de class', () => {
		const fautifs = fichiers.flatMap((f) =>
			destructifsAvecClasse(readFileSync(f, 'utf8'), cheminRelatif(f)),
		);
		expect(fautifs).toEqual([]);
	});

	// Montage de (f), relevé PAR L'ANALYSEUR — celui que (f) emploie — et confronté au compte
	// textuel : un analyseur aveuglé (comparaison inversée, type mal posé) rendrait moins que
	// le texte. Seuil au compte du 2026-10-13 (25, C-28-1-38) et non à l'égalité : un usage
	// AJOUTÉ n'est pas fautif tant qu'il ne porte ni `class` ni `class:` — ce que (f) vérifie —
	// ; seul un usage RETIRÉ doit rougir, et un `>=` le voit tant que personne n'en ajoute
	// (C-28-1-41).
	it('(f) montage : la variante destructive est bien relevée (au moins 25 usages)', () => {
		let parAnalyseur = 0;
		let parTexte = 0;
		for (const f of fichiers) {
			const src = readFileSync(f, 'utf8');
			parAnalyseur += boutonsDestructifs(src).length;
			parTexte += (src.match(/variant="destructive"/g) ?? []).length;
		}
		expect(parAnalyseur).toBe(parTexte);
		expect(parAnalyseur).toBeGreaterThanOrEqual(25);
	});
});
