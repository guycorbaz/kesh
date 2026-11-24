/**
 * La locale du système, contrôlée avant la suite E2E (garde de `global-setup.ts`).
 *
 * La suite se lance sous `LANG=fr_CH.UTF-8` (`docs/testing.md` § *Prérequis Playwright local*).
 * Si la locale nommée n'est **pas installée** sur le poste, rien ne le signale : la glibc
 * retombe en locale C (ASCII) et Chromium, qui en hérite, **ignore sans erreur** un
 * `setInputFiles` dont le chemin n'est pas ASCII — `input.files` reste vide, aucun évènement
 * `change`. Les tests qui déposent un fichier sous `testInfo.outputPath()` (dossier nommé
 * d'après le titre du test, accents compris) attendent alors un bouton qui ne s'active jamais :
 * `backup-restore-new-install.spec.ts` « … manquante après l’import » et « … ne fait pas
 * quitter l’écran » rougissaient ainsi au délai de 120 s (KF-065, #1282).
 *
 * D'où cette garde : elle échoue tôt, et dit quoi installer. Elle ne dit rien quand la
 * variable est absente ou vaut `C`, `POSIX` ou `C.UTF-8` (la CI), ni quand `locale -a` n'est
 * pas disponible sur le poste (elle ne peut alors rien vérifier).
 */

import { execFileSync } from 'node:child_process';

/** Les variables qui choisissent la locale des caractères, dans l'ordre de priorité de la glibc. */
export const VARIABLES_LOCALE = ['LC_ALL', 'LC_CTYPE', 'LANG'] as const;

/** Une locale toujours présente : C, POSIX et leurs variantes à jeu de caractères (`C.UTF-8`). */
const LOCALE_TOUJOURS_LA = /^(C|POSIX)([.@].*)?$/i;

/**
 * La forme sous laquelle `locale -a` liste une locale, pour comparer deux graphies.
 *
 * La glibc normalise le jeu de caractères : minuscules, sans ponctuation (`UTF-8` → `utf8`).
 * `fr_CH.UTF-8`, `fr_CH.utf8` et `fr_CH.UTF8` désignent donc la même locale. La langue, le
 * territoire et le modificateur (`@euro`) restent tels quels.
 */
export function normaliserLocale(nom: string): string {
	const m = /^([^.@]*)(?:\.([^@]*))?(?:@(.*))?$/.exec(nom.trim());
	if (!m) return nom.trim();
	const [, base, jeu, modificateur] = m;
	let r = base;
	if (jeu !== undefined) r += '.' + jeu.toLowerCase().replace(/[^a-z0-9]/g, '');
	if (modificateur !== undefined) r += '@' + modificateur;
	return r;
}

/** Une variable de locale qui nomme une locale absente du poste. */
export interface LocaleManquante {
	variable: (typeof VARIABLES_LOCALE)[number];
	valeur: string;
}

/**
 * Les variables de `env` qui nomment une locale absente de `disponibles` (la sortie de
 * `locale -a`, une locale par élément). Les variables vides, `C`, `POSIX` et `C.*` sont ignorées.
 */
export function localesManquantes(
	env: Record<string, string | undefined>,
	disponibles: readonly string[],
): LocaleManquante[] {
	const connues = new Set(disponibles.map((l) => normaliserLocale(l)));
	const manquantes: LocaleManquante[] = [];
	for (const variable of VARIABLES_LOCALE) {
		const valeur = env[variable]?.trim();
		if (!valeur || LOCALE_TOUJOURS_LA.test(valeur)) continue;
		if (!connues.has(normaliserLocale(valeur))) manquantes.push({ variable, valeur });
	}
	return manquantes;
}

/** La sortie de `locale -a`, ou `null` si la commande n'est pas disponible. */
export function listerLocalesInstallees(): string[] | null {
	try {
		return execFileSync('locale', ['-a'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] })
			.split('\n')
			.map((l) => l.trim())
			.filter((l) => l.length > 0);
	} catch {
		return null;
	}
}

/** Le message d'échec : quelle variable, quelle locale, et comment l'installer. */
export function messageLocaleManquante(manquantes: readonly LocaleManquante[]): string {
	const lignes = manquantes.map((m) => `   ${m.variable}=${m.valeur}`);
	const locale = manquantes[0].valeur;
	return (
		'\n❌ FATAL: locale absente du poste.\n' +
		lignes.join('\n') +
		'\n   Sans elle, Chromium retombe en locale C et ignore sans erreur un `setInputFiles`\n' +
		'   sur un chemin non ASCII (backup-restore-new-install.spec.ts, KF-065).\n' +
		'   Installer la locale, puis vérifier :\n' +
		`      sudo locale-gen ${locale}\n` +
		'      locale -a | grep -i ' +
		normaliserLocale(locale).split(/[.@]/)[0] +
		'\n   (ou lancer la suite sous une locale installée). Cf. docs/testing.md § Prérequis Playwright local.\n'
	);
}

/**
 * Échoue si `LC_ALL`, `LC_CTYPE` ou `LANG` nomme une locale absente de `locale -a`.
 *
 * @param env l'environnement à contrôler (par défaut celui du runner, dont hérite Chromium).
 * @param lister la source des locales installées ; `null` (commande absente) ne fait qu'avertir.
 * @throws Error avec le message de `messageLocaleManquante`.
 */
export function verifierLocaleInstallee(
	env: Record<string, string | undefined> = process.env,
	lister: () => string[] | null = listerLocalesInstallees,
): void {
	const aControler = VARIABLES_LOCALE.some((v) => {
		const valeur = env[v]?.trim();
		return !!valeur && !LOCALE_TOUJOURS_LA.test(valeur);
	});
	if (!aControler) return;
	const disponibles = lister();
	if (disponibles === null) {
		console.warn('⚠️  `locale -a` indisponible : la locale du poste n’est pas vérifiée.');
		return;
	}
	const manquantes = localesManquantes(env, disponibles);
	if (manquantes.length > 0) throw new Error(messageLocaleManquante(manquantes));
}
