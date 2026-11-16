// @vitest-environment node
/**
 * Story 36-15 (CR #1161, AC-2, AC-8) — l'axe analytique s'appelle « compte analytique »
 * dans le frontend.
 *
 * 1. Aucun site `/projet/i` dans `src/**\/*.{svelte,ts,js}` (hors `*.test.ts`), une fois
 *    les commentaires masqués par `masquerCommentaires` (`//`, `/* … *\/`, `<!-- … -->`) :
 *    les commentaires et doc-comments qui disent « projet » restent hors de la garde
 *    (fiche, D6). Les identifiants (`project`, `projectId`, `/settings/projects`) ne
 *    disent pas « projet » et ne sont pas atteints.
 * 2. Le repli littéral de chaque clé du vocabulaire (les 49 de la fiche, § 1) égale sa
 *    valeur fr-CH (`releveDesReplis`, qui ne lit que les appels `i18nMsg`).
 * 3. Les quatre sites hors `i18nMsg` — `nav-projects` (`i18nKey` / `fallback`),
 *    `reports-project-expenses` et `reports-project-return` (`labelKey` / `fallback`), le
 *    couple `slugKey` / `slugFallback` du nom de fichier exporté — sont **trouvés**
 *    (sinon rouge), puis égaux à leur valeur fr-CH.
 *
 * Angles morts écrits : un libellé assemblé à l'exécution (concaténation, gabarit) ne se
 * lit pas ; les clés hors des 49 ne sont pas contrôlées ici (#758).
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { masquerCommentaires } from './i18n-literal-reader.js';
import { catalogueDe, fichiersDe, releveDesReplis, type SourceLue } from './i18n-repli-catalogue';

const RACINE_FTL = '../crates/kesh-i18n/locales';
const LAYOUT = 'src/routes/(app)/+layout.svelte';
const RAPPORTS = 'src/routes/(app)/reports/+page.svelte';

/** Les 49 clés du vocabulaire analytique (fiche 36-15, § 1). */
const CLES = [
	'journal-entry-form-col-project',
	'journal-entry-project-archived',
	'reconciliation-failed-project-archived',
	'reconciliation-failed-project-not-found',
	'reconciliation-manual-project-label',
	'reconciliation-rules-default-project-archived',
	'reconciliation-rules-labels-default-project',
	'reconciliation-split-th-project',
	'reports-project-expenses',
	'reports-project-expenses-empty',
	'reports-project-expenses-title',
	'reports-project-return',
	'reports-project-return-col-project',
	'reports-project-return-empty',
	'reports-project-return-title',
	'reports-project-selector-label',
	'reports-project-selector-placeholder',
	'reports-filename-project-expenses',
	'reports-filename-project-return',
	'export-global-description',
	'export-global-content-includes',
	'invoice-detail-project-label',
	'invoice-field-project',
	'invoice-project-archived',
	'invoice-project-current',
	'supplier-invoices-detail-project',
	'supplier-invoices-field-project',
	'nav-projects',
	'projects-archived',
	'projects-created',
	'projects-empty',
	'projects-err-code',
	'projects-err-name',
	'projects-field-parent',
	'projects-form-create',
	'projects-form-edit',
	'projects-load-error',
	'projects-new',
	'projects-parent-none',
	'projects-show-archived',
	'projects-subtitle',
	'projects-title',
	'projects-unarchived',
	'projects-updated',
	'audit-log-entity-project',
	'audit-log-action-project-created',
	'audit-log-action-project-updated',
	'audit-log-action-project-archived',
	'audit-log-action-project-unarchived',
] as const;

const PROJET = /projet/i;

/** Les sources lues : `src/**\/*.{svelte,ts,js}` hors `*.test.ts`. */
function sourcesDuFrontend(): SourceLue[] {
	return fichiersDe('src')
		.filter((f) => /\.(svelte|ts|js)$/.test(f) && !f.endsWith('.test.ts'))
		.map((chemin) => ({ chemin, texte: readFileSync(chemin, 'utf-8') }));
}

/** Fonction pure : les lignes qui disent « projet », commentaires masqués. */
export function sitesDisantProjet(sources: SourceLue[]): string[] {
	const sites: string[] = [];
	for (const { chemin, texte } of sources) {
		masquerCommentaires(texte)
			.split('\n')
			.forEach((ligne, n) => {
				if (PROJET.test(ligne)) sites.push(`${chemin}:${n + 1}: ${ligne.trim()}`);
			});
	}
	return sites;
}

/**
 * Fonction pure : le repli d'un site en objet `{ <champ>: '<clé>', fallback: '…' }`
 * (guillemets simples ou doubles, sauts de ligne admis), ou `null` s'il est introuvable.
 */
export function repliEnObjet(source: string, champ: string, cle: string): string | null {
	const motif = new RegExp(
		`${champ}:\\s*(['"])${cle.replace(/-/g, '\\-')}\\1\\s*,\\s*fallback:\\s*(['"])(.*?)\\2`,
		's',
	);
	return motif.exec(source)?.[3] ?? null;
}

/**
 * Fonction pure : les replis du nom de fichier exporté des rapports analytiques, lus du
 * couple `slugKey` / `slugFallback` ; `null` si l'une des deux formes est introuvable.
 */
export function replisDuNomDeFichier(
	source: string,
): { expenses: string; return: string } | null {
	const cles =
		/const slugKey =\s*activeTab === 'project-expenses'\s*\?\s*'reports-filename-project-expenses'\s*:\s*'reports-filename-project-return'/.exec(
			source,
		);
	const replis =
		/const slugFallback =\s*activeTab === 'project-expenses'\s*\?\s*'([^']*)'\s*:\s*'([^']*)'/.exec(
			source,
		);
	if (!cles || !replis) return null;
	return { expenses: replis[1], return: replis[2] };
}

describe('vocabulaire analytique — « compte analytique » (CR #1161)', () => {
	const fr = catalogueDe(RACINE_FTL, 'fr-CH');
	const sources = sourcesDuFrontend();

	it('les sondes : un repli « Projet » est détecté, un objet singulier rougit', () => {
		expect(
			sitesDisantProjet([{ chemin: 'x.svelte', texte: "{i18nMsg('x', 'Projet archivé')}" }]),
		).toHaveLength(1);
		expect(
			sitesDisantProjet([
				{ chemin: 'x.svelte', texte: "// projet\n<!-- projet -->\n/* projet */ let a = 1;" },
			]),
		).toEqual([]);
		const objet = "{ i18nKey: 'nav-projects', fallback: 'Compte analytique', href: '/x' }";
		expect(repliEnObjet(objet, 'i18nKey', 'nav-projects')).toBe('Compte analytique');
		expect(repliEnObjet(objet, 'i18nKey', 'nav-projects')).not.toBe(fr.get('nav-projects'));
		expect(repliEnObjet(objet, 'labelKey', 'nav-projects')).toBeNull();
		expect(
			replisDuNomDeFichier(
				"const slugKey =\n activeTab === 'project-expenses'\n ? 'reports-filename-project-expenses'\n : 'reports-filename-project-return';\nconst slugFallback =\n activeTab === 'project-expenses' ? 'a' : 'b';",
			),
		).toEqual({ expenses: 'a', return: 'b' });
		expect(replisDuNomDeFichier('const slugFallback = 1;')).toBeNull();
	});

	it('le catalogue fr-CH porte les 49 clés (anti-test-muet)', () => {
		expect(CLES).toHaveLength(49);
		const absentes = CLES.filter((c) => !fr.has(c));
		expect(absentes).toEqual([]);
		expect(sources.length).toBeGreaterThan(200);
	});

	it('aucun site du frontend ne dit « projet », commentaires masqués', () => {
		expect(sitesDisantProjet(sources)).toEqual([]);
	});

	it('le repli de chaque appel i18nMsg des 49 clés égale sa valeur fr-CH', () => {
		const parCle = releveDesReplis(sources);
		const relevees = CLES.filter((c) => parCle.has(c));
		// Revue P1 (B1-2 = V-A1) : liste FERMÉE des clés hors appel littéral — les sites en
		// objet et les deux noms de fichier (contrôlés au test suivant), et les cinq clés de
		// l'audit, assemblées à l'exécution. Toute autre clé doit être relevée : un appel
		// réécrit sous une forme que le relevé ne lit pas rougit ici (39 relevées).
		expect(CLES.filter((c) => !parCle.has(c))).toEqual([
			'reports-project-expenses',
			'reports-project-return',
			'reports-filename-project-expenses',
			'reports-filename-project-return',
			'nav-projects',
			'audit-log-entity-project',
			'audit-log-action-project-created',
			'audit-log-action-project-updated',
			'audit-log-action-project-archived',
			'audit-log-action-project-unarchived',
		]);
		expect(relevees).toHaveLength(39);
		const ecarts = relevees.flatMap((cle) => {
			const replis = [...parCle.get(cle)!];
			const attendu = fr.get(cle);
			return replis.length === 1 && replis[0] === attendu
				? []
				: [`${cle} : repli ${JSON.stringify(replis)} ≠ ${JSON.stringify(attendu)}`];
		});
		expect(ecarts).toEqual([]);
	});

	it('les quatre sites hors i18nMsg sont trouvés et égaux à leur valeur fr-CH', () => {
		const layout = readFileSync(LAYOUT, 'utf-8');
		const rapports = readFileSync(RAPPORTS, 'utf-8');
		expect(repliEnObjet(layout, 'i18nKey', 'nav-projects')).toBe(fr.get('nav-projects'));
		expect(repliEnObjet(rapports, 'labelKey', 'reports-project-expenses')).toBe(
			fr.get('reports-project-expenses'),
		);
		expect(repliEnObjet(rapports, 'labelKey', 'reports-project-return')).toBe(
			fr.get('reports-project-return'),
		);
		expect(replisDuNomDeFichier(rapports)).toEqual({
			expenses: fr.get('reports-filename-project-expenses'),
			return: fr.get('reports-filename-project-return'),
		});
	});
});
