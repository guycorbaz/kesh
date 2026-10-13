// @vitest-environment node
/**
 * Garde d'ensemble clos des classes de couleur sans jeton défini — Story 28-1
 * (#640, AC7 (4), C-28-1-23, C-28-1-30, C-28-1-34, C-28-1-35).
 *
 * Le défaut d'origine de #640 : une classe de couleur (`bg-destructive`…) dont le
 * jeton n'est pas défini dans le `@theme` d'`app.css`. Tailwind v4 n'engendre
 * alors AUCUNE règle, en silence. Cette garde ne liste pas les formes correctes :
 * elle **inventorie les non-résolus** (CLAUDE.md, « Inventorier les sites NON
 * RÉSOLUS ») et exige qu'ils égalent une table fermée.
 *
 * ## Méthode
 *
 * 1. **Relevé** (`releverCandidats`) : dans tous les `.svelte` et `.ts` de `src/`
 *    (hors `.test.`), le texte est découpé sur `` [\s"'`{}()=] `` — le `=` compris,
 *    pour que la directive `class:bg-x={…}` livre `bg-x` (C-28-1-34). Pour chaque
 *    morceau : préfixes de variante retirés (tout ce qui précède le dernier `:`,
 *    `class:` compris), puis un `!` initial ou final, puis une opacité `/<n>` finale ;
 *    il est retenu s'il s'écrit `<préfixe de couleur>-<nom>` (`MOTIF` ci-dessous).
 * 2. **Décision par Tailwind lui-même** : `__unstable__loadDesignSystem` de
 *    `@tailwindcss/node`, chargé sur `app.css`, puis `candidatesToCss`, qui rend
 *    `null` pour un candidat que rien ne résout. Les candidats qui ne sont pas des
 *    couleurs (`text-sm`, `border-b`, `text-right`) sont résolus par leur propre
 *    utilité et ne remontent donc pas.
 * 3. **Table fermée** des NOMS non résolus (`NON_DEFINIS`), chacun renvoyé à
 *    #645. La garde rougit (a) si un nom neuf non défini apparaît, (b) si un nom
 *    listé devient défini sans que la table soit mise à jour.
 *
 * ## Ce qu'elle ne voit pas (angles morts assumés)
 *
 * - un nom suivi d'une ponctuation de prose (`text-foo.`) : l'ancre `$` l'écarte
 *   (angle mort 12) ; aucune classe ne s'écrit ainsi dans un attribut ;
 * - une classe construite par concaténation (`'bg-' + couleur`) ;
 * - un jeton défini mais mal choisi (contraste) : c'est l'objet d'autres gardes ;
 * - les fichiers `.js` de `src/` et `app.html`, non lus (aucune classe de couleur au
 *   2026-10-13) ;
 * - la table tient des NOMS, non des classes : un `bg-danger` neuf passe, `danger`
 *   y étant déjà (voulu : #645 traite le nom, quelle que soit l'utilité).
 *
 * ## Mettre la table à jour
 *
 * Quand #645 définit un jeton, retirer son nom de la table — c'est le but : le
 * changement se voit en revue. Un écart entre la sortie du compilateur et la
 * table s'examine NOM PAR NOM avant d'être recopié : la table ne se remplit pas
 * depuis la sortie de la garde qu'elle contrôle (le défaut de l'AC7 de l'Epic 23,
 * vert par construction).
 *
 * ## Moteur (C-28-1-35)
 *
 * `@tailwindcss/node` est déclaré au même spécificateur que `tailwindcss` ; il
 * dépend de `tailwindcss` exactement à sa propre version. S'ils divergent, il en
 * embarque une copie et la garde déciderait avec un autre moteur que celui du
 * build : d'où l'assertion de version en tête.
 */
import { describe, it, expect, beforeAll } from 'vitest';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, resolve } from 'node:path';

const RACINE = 'src';

/**
 * Préfixes d'utilités de couleur relevés ; groupe 2 = le nom de couleur. Les
 * préfixes composés (`ring-offset`, `inset-ring`, `inset-shadow`) précèdent leur
 * racine pour que le nom ne commence pas par `offset-`. `shadow`, `from`, `via`,
 * `to` depuis la revue de code P1 (E4).
 */
const MOTIF =
	/^(ring-offset|inset-ring|inset-shadow|bg|text|border|ring|outline|fill|stroke|divide|placeholder|decoration|caret|accent|shadow|from|via|to)-([a-z][a-z0-9-]*)$/;

/** Séparateurs de jetons — le `=` compris (C-28-1-34). */
const SEPARATEURS = /[\s"'`{}()=]+/;

/**
 * Candidats de couleur relevés dans un texte source : `bg-x`, `text-y`…, une
 * fois retirés préfixes de variante, `!` initial et opacité.
 */
export function releverCandidats(source: string): string[] {
	const out: string[] = [];
	for (const morceau of source.split(SEPARATEURS)) {
		const candidat = morceau
			.slice(morceau.lastIndexOf(':') + 1)
			.replace(/^!/, '')
			// `!` final : forme canonique de l'important en Tailwind v4 (revue de code P1, E4).
			.replace(/!$/, '')
			.replace(/\/\d+$/, '');
		if (MOTIF.test(candidat)) out.push(candidat);
	}
	return out;
}

/** Nom de couleur d'un candidat : ce qui suit le préfixe (`bg-error-soft` → `error-soft`, `ring-offset-x` → `x`). */
export function nomDe(candidat: string): string {
	return MOTIF.exec(candidat)?.[2] ?? candidat.slice(candidat.indexOf('-') + 1);
}

/**
 * La table fermée des noms de couleur NON définis, mesurée le 2026-10-13 sur
 * l'arbre de la 28-1. Tous renvoyés à #645, qui les définira ou les remplacera.
 */
const NON_DEFINIS: Record<string, string> = {
	muted: '#645 — jeton shadcn, fond des survols outline/ghost',
	'muted-foreground': '#645 — jeton shadcn, texte secondaire (≈ 58 occurrences)',
	foreground: '#645 — jeton shadcn, texte courant',
	background: '#645 — jeton shadcn, fond de outline',
	ring: '#645 — jeton shadcn, anneau et bordure de focus de la base du Button',
	input: '#645 — jeton shadcn, bordure de champ et fond désactivé',
	accent: '#645 — jeton shadcn, survol des éléments de menu',
	'accent-foreground': '#645 — jeton shadcn, texte des éléments de menu survolés',
	card: '#645 — jeton shadcn, fond de carte',
	'error-soft': '#645 — fond pâle des bandeaux de l\'import bancaire',
	'warning-soft': '#645 — fond pâle des avertissements de l\'import',
	danger: '#645 — cadres d\'erreur de la sauvegarde et de la restauration',
	'surface-hover': '#645 — survol des lignes de trois listes',
};

function fichiersSource(dir: string): string[] {
	const out: string[] = [];
	for (const entry of readdirSync(dir, { withFileTypes: true })) {
		const chemin = join(dir, entry.name);
		if (entry.isDirectory()) out.push(...fichiersSource(chemin));
		else if (/\.(svelte|ts)$/.test(entry.name) && !entry.name.includes('.test.')) out.push(chemin);
	}
	return out;
}

function versionDe(chemin: string): string {
	return JSON.parse(readFileSync(chemin, 'utf8')).version;
}

describe('relevé des candidats — auto-tests sur source synthétique', () => {
	it('une directive class: livre son nom (le = est un séparateur)', () => {
		expect(releverCandidats('<span class:text-inexistant={x}>').map(nomDe)).toEqual(['inexistant']);
	});

	it('la frontière est le début du jeton : context-menu ne livre rien', () => {
		expect(releverCandidats('<div class="context-menu">')).toEqual([]);
	});

	it('préfixes de variante, ! et opacité retirés', () => {
		expect(releverCandidats('<p class="hover:!bg-foo/40">').map(nomDe)).toEqual(['foo']);
	});

	it('le ! final de Tailwind v4 est retiré, avec ou sans opacité', () => {
		expect(releverCandidats('<p class="bg-foo! hover:text-bar/40!">').map(nomDe)).toEqual(['foo', 'bar']);
	});

	it('shadow-, from-, via-, to- et ring-offset- livrent leur nom de couleur', () => {
		expect(
			releverCandidats('<p class="shadow-a from-b via-c to-d ring-offset-e inset-ring-f">').map(nomDe),
		).toEqual(['a', 'b', 'c', 'd', 'e', 'f']);
	});
});

describe('AC7 (4) — les noms de couleur non définis égalent la table fermée', () => {
	it('@tailwindcss/node décide avec le moteur du build (versions égales, pas de copie)', () => {
		const vNode = versionDe('node_modules/@tailwindcss/node/package.json');
		const vTw = versionDe('node_modules/tailwindcss/package.json');
		const copie = existsSync('node_modules/@tailwindcss/node/node_modules/tailwindcss');
		expect(
			{ vNode, vTw, copie },
			'monter `tailwindcss`, `@tailwindcss/vite` et `@tailwindcss/node` ensemble',
		).toEqual({ vNode: vTw, vTw, copie: false });
	});

	let nonResolus: Map<string, Set<string>>;

	beforeAll(async () => {
		const { __unstable__loadDesignSystem } = await import('@tailwindcss/node');
		const css = readFileSync(join(RACINE, 'app.css'), 'utf8');
		const ds = await __unstable__loadDesignSystem(css, { base: resolve(RACINE) });

		const sites = new Map<string, Set<string>>();
		for (const fichier of fichiersSource(RACINE)) {
			for (const c of releverCandidats(readFileSync(fichier, 'utf8'))) {
				if (!sites.has(c)) sites.set(c, new Set());
				sites.get(c)!.add(fichier);
			}
		}
		const candidats = [...sites.keys()];
		const css2 = ds.candidatesToCss(candidats);
		nonResolus = new Map();
		candidats.forEach((c, i) => {
			if (css2[i] !== null) return;
			const nom = nomDe(c);
			if (!nonResolus.has(nom)) nonResolus.set(nom, new Set());
			for (const f of sites.get(c)!) nonResolus.get(nom)!.add(f);
		});
	}, 60_000);

	it('le relevé trouve des candidats résolus et non résolus (montage)', () => {
		expect(nonResolus.size).toBeGreaterThan(0);
	});

	it('aucun nom non défini hors de la table (a)', () => {
		const neufs = [...nonResolus.keys()]
			.filter((n) => !(n in NON_DEFINIS))
			.map((n) => `${n} (${[...nonResolus.get(n)!].slice(0, 3).join(', ')})`);
		expect(neufs, 'définir le jeton dans app.css, ou l\'inscrire à la table avec son motif').toEqual([]);
	});

	it('aucun nom de la table devenu défini ou disparu (b)', () => {
		const resolus = Object.keys(NON_DEFINIS).filter((n) => !nonResolus.has(n));
		expect(resolus, 'retirer ces noms de la table').toEqual([]);
	});
});
