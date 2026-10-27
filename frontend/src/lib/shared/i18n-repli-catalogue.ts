/**
 * **Le repli d'une clé égal à sa valeur du catalogue, par préfixe d'écran** —
 * Story 32-10b (refs #723 ; #758), extrait de la garde de la 32-8c
 * (`features/mcp-sends/mcp-sends-i18n.test.ts`) pour être employé aussi par
 * celle de l'écran des prompts (`features/mcp-prompts/mcp-prompts-i18n.test.ts`).
 * Fiche 32-10b, AC-15 (b) : la 32-7, qui étend G13 par préfixe
 * (`PREFIXES_REPLI_EGAL_AU_CATALOGUE`, C-32-7-21), n'a pas fusionné.
 *
 * Ce que tient une garde qui l'emploie, pour un préfixe :
 * - pour chaque clé du préfixe relevée dans les sources de l'écran (lecteur commun
 *   `findCallSites` / `readFallback`), l'ensemble de ses replis littéraux est
 *   **exactement** `[valeur fr-CH]` ;
 * - l'ensemble des clés relevées est **égal**, dans les deux sens, à celui des clés
 *   du catalogue qui commencent par le préfixe — l'appartenance se juge par
 *   `startsWith`, jamais par une sous-chaîne.
 *
 * Angle mort écrit : le reste du catalogue reste hors de ces gardes (#758).
 *
 * ⚠️ Module de **test** : il lit le disque (`node:fs`) et n'est importé par aucun
 * code de l'application. Il vit ici, et non dans `scripts/`, parce que vitest est
 * borné à `src/` (même raison que `i18n-literal-reader.js`).
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { findCallSites, readFallback, masquerCommentaires } from './i18n-literal-reader.js';
import { dansLePerimetreDeFichier } from './i18n-harvest.js';

/** Une source lue : son chemin et son texte. */
export interface SourceLue {
	chemin: string;
	texte: string;
}

/** Tous les fichiers d'un dossier, récursivement. */
export function fichiersDe(dossier: string): string[] {
	const sortie: string[] = [];
	for (const e of readdirSync(dossier, { withFileTypes: true })) {
		const chemin = join(dossier, e.name);
		if (e.isDirectory()) sortie.push(...fichiersDe(chemin));
		else sortie.push(chemin);
	}
	return sortie;
}

/**
 * Les valeurs d'un texte de catalogue `.ftl` — clés de tête et valeurs d'une ligne
 * (aucune clé des préfixes d'écran n'en a plusieurs). Fonction pure.
 */
export function valeursDuCatalogue(texte: string): Map<string, string> {
	const valeurs = new Map<string, string>();
	for (const ligne of texte.split('\n')) {
		const m = /^([a-zA-Z][\w-]*) = ?(.*)$/.exec(ligne);
		if (m) valeurs.set(m[1], m[2]);
	}
	return valeurs;
}

/** Le catalogue d'une locale, lu sous `racineFtl`. */
export function catalogueDe(racineFtl: string, locale: string): Map<string, string> {
	return valeursDuCatalogue(readFileSync(join(racineFtl, locale, 'messages.ftl'), 'utf-8'));
}

/** Relève, pour chaque clé littérale, l'ensemble de ses replis littéraux. Fonction pure. */
export function releveDesReplis(sources: SourceLue[]): Map<string, Set<string>> {
	const parCle = new Map<string, Set<string>>();
	for (const { texte } of sources) {
		const source = masquerCommentaires(texte);
		for (const site of findCallSites(source)) {
			if (site.arg?.kind !== 'literal') continue;
			const repli = readFallback(source, site.afterFirstArg);
			if (!parCle.has(site.arg.value)) parCle.set(site.arg.value, new Set());
			parCle.get(site.arg.value)!.add(repli?.kind === 'literal' ? repli.value : '<repli non littéral>');
		}
	}
	return parCle;
}

/**
 * Fonction pure : les écarts entre un relevé et un catalogue, pour un préfixe.
 * Rend la liste nommée des clés fautives (vide si tout est juste).
 */
export function ecartsAuCatalogue(
	parCle: Map<string, Set<string>>,
	valeurs: Map<string, string>,
	prefixe: string,
): string[] {
	const ecarts: string[] = [];
	const releves = [...parCle.keys()].filter((c) => c.startsWith(prefixe));
	const auCatalogue = [...valeurs.keys()].filter((c) => c.startsWith(prefixe));
	for (const cle of releves) {
		const attendu = valeurs.get(cle);
		const replis = [...parCle.get(cle)!];
		if (attendu === undefined) ecarts.push(`${cle} : absente du catalogue`);
		else if (replis.length !== 1 || replis[0] !== attendu)
			ecarts.push(`${cle} : repli ${JSON.stringify(replis)} ≠ ${JSON.stringify(attendu)}`);
	}
	for (const cle of auCatalogue) {
		if (!parCle.has(cle)) ecarts.push(`${cle} : au catalogue, demandée par aucun site littéral`);
	}
	return ecarts.sort();
}

/** Les sources d'un écran, dans le périmètre de la moisson (tests exclus). */
export function sourcesDesDossiers(dossiers: string[]): SourceLue[] {
	return dossiers
		.flatMap(fichiersDe)
		.filter((f) => dansLePerimetreDeFichier(f.split('/').pop()!))
		.map((chemin) => ({ chemin, texte: readFileSync(chemin, 'utf-8') }));
}
