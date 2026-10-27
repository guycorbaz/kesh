// @vitest-environment node
/**
 * Story 32-7 (refs #723) — deux gardes lexicales de l'écran « Serveur MCP » (AC-15).
 *
 * **Périmètre** : les sources HORS TESTS de `features/mcp-server/` (récursivement) et les
 * fichiers DIRECTEMENT dans `routes/(app)/admin/mcp/` — sans descente : les sous-dossiers
 * `sends/` (32-8c) et `prompts/` (32-10b) sont ceux d'autres écrans.
 *
 * - **G-a — ni stockage, ni journal, ni HTML brut** : aucune de ces sources n'emploie
 *   `localStorage`, `sessionStorage`, `indexedDB`, `caches.`, `document.cookie`, `console.`
 *   ni l'insertion de HTML brut, après masquage des commentaires (`masquerCommentaires`).
 * - **G-b — l'instance du secret ne vit que dans un composant** : `new SecretUnique`
 *   n'apparaît que dans le bloc `<script>` D'INSTANCE d'un `.svelte` — jamais dans un
 *   `.ts`/`.svelte.ts`, ni dans un `<script module>` / `<script context="module">`, ni
 *   hors de tout bloc `<script>`. Une instance de module survit à la fermeture et à la
 *   navigation.
 *
 * Chaque garde a un témoin rouge et un témoin vert (source factice, fonction pure), et
 * une assertion de cardinalité EXACTE du périmètre lu : un chemin faux qui lirait un
 * sous-ensemble ne passerait pas.
 *
 * Angle mort écrit : une écriture indirecte (`window['local' + 'Storage']`, une
 * bibliothèque) échappe à G-a ; les espions de `McpKeysSection.test.ts` (AC-11 (v)) ne
 * couvrent que les chemins exécutés.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { masquerCommentaires } from '$lib/shared/i18n-literal-reader.js';

const FEATURE = 'src/lib/features/mcp-server';
const ROUTE = 'src/routes/(app)/admin/mcp';

/** Les sources de la feature (récursivement) et de la route (sans descente), hors tests. */
function perimetre(): string[] {
	const recursif = (d: string): string[] =>
		readdirSync(d, { withFileTypes: true }).flatMap((e) =>
			e.isDirectory() ? recursif(join(d, e.name)) : [join(d, e.name)],
		);
	const direct = readdirSync(ROUTE, { withFileTypes: true })
		.filter((e) => e.isFile())
		.map((e) => join(ROUTE, e.name));
	return [...recursif(FEATURE), ...direct].filter((f) => !/\.test\.ts$/.test(f)).sort();
}

/** La liste attendue, écrite en dur (recomptée au développement : 9 + 2). */
const PERIMETRE_ATTENDU = [
	'src/lib/features/mcp-server/McpKeySecretNotice.svelte',
	'src/lib/features/mcp-server/McpKeysSection.svelte',
	'src/lib/features/mcp-server/McpServerPanel.svelte',
	'src/lib/features/mcp-server/McpSettingsSection.svelte',
	'src/lib/features/mcp-server/mcp-reglages.ts',
	'src/lib/features/mcp-server/mcp-server.api.ts',
	'src/lib/features/mcp-server/mcp-server.types.ts',
	'src/lib/features/mcp-server/mcp-time.ts',
	'src/lib/features/mcp-server/secret-unique.svelte.ts',
	'src/routes/(app)/admin/mcp/+page.svelte',
	'src/routes/(app)/admin/mcp/+page.ts',
];

/** Les formes refusées par G-a (la marque du HTML brut écrite en deux morceaux). */
const FORMES_GA = new RegExp(
	['localStorage', 'sessionStorage', 'indexedDB', 'caches\\.', 'document\\.cookie', 'console\\.', '\\{' + '@html'].join('|'),
);

/** G-a, fonction pure : les lignes fautives d'une source, après masquage des commentaires. */
export function violationsGa(source: string): string[] {
	return masquerCommentaires(source)
		.split('\n')
		.map((l) => l.trim())
		.filter((l) => FORMES_GA.test(l));
}

const NEW_SECRET = /new\s+SecretUnique\b/;

/** G-b, fonction pure : les sites refusés de `new SecretUnique` dans une source. */
export function violationsGb(chemin: string, source: string): string[] {
	const texte = masquerCommentaires(source);
	if (!chemin.endsWith('.svelte')) {
		return NEW_SECRET.test(texte) ? [`${chemin} : hors d'un composant`] : [];
	}
	const fautes: string[] = [];
	let horsScripts = texte;
	for (const m of texte.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)) {
		const attributs = m[1];
		const corps = m[2];
		horsScripts = horsScripts.replace(m[0], '');
		const deModule = /\bmodule\b/.test(attributs) || /context\s*=\s*["']module["']/.test(attributs);
		if (deModule && NEW_SECRET.test(corps)) fautes.push(`${chemin} : <script${attributs}>`);
	}
	if (NEW_SECRET.test(horsScripts)) fautes.push(`${chemin} : hors de tout <script>`);
	return fautes;
}

describe('gardes lexicales de l’écran Serveur MCP (AC-15)', () => {
	it('le périmètre lu est exactement celui attendu (cardinalité 11)', () => {
		expect(perimetre()).toEqual(PERIMETRE_ATTENDU);
		expect(perimetre()).toHaveLength(11);
	});

	it('G-a : témoin rouge, témoins verts', () => {
		expect(violationsGa("sessionStorage.setItem('k', s);")).toEqual(["sessionStorage.setItem('k', s);"]);
		expect(violationsGa('console.info(res);')).toHaveLength(1);
		expect(violationsGa('{' + '@html x}')).toHaveLength(1);
		// témoin vert existant au dépôt (`settings/api-keys/+page.svelte`) : un commentaire
		expect(violationsGa('\t// Le secret vit dans un $state (pas de localStorage).')).toEqual([]);
		expect(violationsGa('<!-- pas de sessionStorage -->')).toEqual([]);
	});

	it('G-a : aucune source du périmètre n’emploie un stockage, un journal ou du HTML brut', () => {
		const fautes = perimetre().flatMap((f) => violationsGa(readFileSync(f, 'utf-8')).map((l) => `${f} : ${l}`));
		expect(fautes).toEqual([]);
	});

	it('G-b : témoins rouges, témoins verts', () => {
		expect(violationsGb('x.ts', 'export const s = new SecretUnique();')).toHaveLength(1);
		expect(
			violationsGb('X.svelte', '<script module>\n\texport const s = new SecretUnique();\n</script>\n<p>x</p>'),
		).toHaveLength(1);
		expect(
			violationsGb('X.svelte', '<script context="module">\n\texport const s = new SecretUnique();\n</script>'),
		).toHaveLength(1);
		expect(violationsGb('X.svelte', '<script lang="ts"></script>\n<p>{new SecretUnique()}</p>')).toHaveLength(1);
		expect(violationsGb('X.svelte', '<script lang="ts">\n\tconst secret = new SecretUnique();\n</script>')).toEqual([]);
		expect(violationsGb('x.ts', '/** Exemple : const s = new SecretUnique(); */')).toEqual([]);
	});

	it('G-b : `new SecretUnique` ne vit que dans un <script> d’instance du périmètre', () => {
		const fautes = perimetre().flatMap((f) => violationsGb(f, readFileSync(f, 'utf-8')));
		expect(fautes).toEqual([]);
		// et il y vit bien quelque part : la garde ne passe pas à vide
		const sites = perimetre().filter((f) => NEW_SECRET.test(masquerCommentaires(readFileSync(f, 'utf-8'))));
		expect(sites).toEqual(['src/lib/features/mcp-server/McpKeysSection.svelte']);
	});
});
