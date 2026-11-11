// Story 34-6 (#953) — la règle du conflit de version des trois éditions de la société.
//
// `companyEditConflict(base, stored, input, fields)` compare, champ par champ, la copie
// prise à l'ouverture de l'édition (`base`), la société relue après le 409 (`stored`) et la
// saisie (`input`). Les définitions sont celles de la fiche, § Conception.

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
	companyEditConflict,
	normalizeCompanyValue,
	normalizeCountry,
	ADDRESS_NORMALIZERS,
	sameBlockValues,
	adoptionDecision,
	PENDING_FIELD_CLASS,
} from './company-conflict';

// Revue P4 (E4-2 ; C-e346-11) : l'ordre d'adoption d'une société reçue, en fonction pure.
describe('adoptionDecision — adopter une société reçue ? (revue P3 et P4)', () => {
	it('aucune société encore : adopter', () => {
		expect(adoptionDecision(null, { version: 1, seq: 4 })).toEqual({ adopt: true, seq: 4 });
	});

	it('version inférieure : refuser, quelle que soit la séquence', () => {
		expect(adoptionDecision({ version: 3, seq: 1 }, { version: 2, seq: 9 })).toEqual({ adopt: false, seq: 1 });
	});

	it('version égale : la séquence de départ départage', () => {
		expect(adoptionDecision({ version: 3, seq: 5 }, { version: 3, seq: 4 })).toEqual({ adopt: false, seq: 5 });
		expect(adoptionDecision({ version: 3, seq: 5 }, { version: 3, seq: 6 })).toEqual({ adopt: true, seq: 6 });
	});

	it('version supérieure partie plus tôt : adopter, sans faire reculer la séquence (E4-2)', () => {
		expect(adoptionDecision({ version: 3, seq: 5 }, { version: 4, seq: 2 })).toEqual({ adopt: true, seq: 5 });
	});
});

// Revue P4 (B4-1 = E4-1 = A4-1) : Tailwind 4 ne produit aucune règle pour une couleur qui
// n'est pas un jeton du thème. Ce contrôle rougit si la classe du champ en vol en nomme un
// qui n'existe pas dans `src/app.css` (le cas de `bg-input`, resté sans effet en P3).
//
// Revue P5 (B5-2 = E5-3 = A5-3, E5-4) : le contrôle couvre tous les utilitaires de couleur et
// le préfixe `!`, accepte les noms à chiffre, ne cherche les jetons que dans le bloc `@theme`
// sans ses commentaires, reconnaît les couleurs intégrées de Tailwind et les utilitaires
// non colorés (`text-sm`, `border-t`), et lit `app.css` depuis ce fichier, non depuis le
// répertoire courant. Ses cas sont épinglés ci-dessous.

/** Les utilitaires Tailwind qui prennent une couleur. */
const COLOR_UTILITIES = [
	'bg', 'text', 'border', 'ring', 'outline', 'fill', 'stroke', 'placeholder', 'decoration',
	'caret', 'accent', 'divide', 'from', 'via', 'to', 'shadow',
];
/** Valeurs qui ne sont pas des couleurs, par utilitaire (tailles, largeurs, styles). */
const NON_COLOR =
	/^(?:text-(?:xs|sm|base|lg|\d?xl|left|center|right|justify|start|end|wrap|nowrap|balance|pretty|clip|ellipsis)|border(?:-[trblxyse])?(?:-\d+)?|border-(?:solid|dashed|dotted|double|hidden|none|collapse|separate)|outline(?:-\d+)?|outline-(?:none|hidden|dashed|dotted|double|solid)|ring(?:-\d+)?|ring-inset|shadow(?:-(?:2xs|xs|sm|md|lg|xl|2xl|none|inner))?|divide-[xy](?:-\d+)?|divide-(?:solid|dashed|dotted|double|none)|decoration-(?:solid|double|dotted|dashed|wavy|\d+|auto|from-font)|stroke-\d+|fill-none|stroke-none|from-\d+%|via-\d+%|to-\d+%)$/;
/** Couleurs que Tailwind 4 fournit sans jeton du thème (la palette n'est pas remise à zéro). */
const BUILTIN = /^(?:white|black|transparent|current|inherit|(?:slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-(?:50|[1-9]00|950))$/;

/** Les jetons `--color-…` déclarés dans le bloc `@theme`, commentaires retirés. */
function themeColorTokens(css: string): Set<string> {
	const sansCommentaires = css.replace(/\/\*[\s\S]*?\*\//g, '');
	const theme = /@theme\s*\{([\s\S]*?)\n\}/.exec(sansCommentaires)?.[1] ?? '';
	return new Set([...theme.matchAll(/--color-([a-z0-9-]+)\s*:/g)].map((m) => m[1]));
}

/** Les couleurs nommées par `classes` qui ne sont ni intégrées ni des jetons de `css`. */
function missingThemeColors(classes: string, css: string): string[] {
	const tokens = themeColorTokens(css);
	const manquants: string[] = [];
	for (const brute of classes.split(/\s+/).filter(Boolean)) {
		const classe = (brute.split(':').pop() ?? '').replace(/^!/, '');
		if (NON_COLOR.test(classe)) continue;
		const m = new RegExp(`^(?:${COLOR_UTILITIES.join('|')})-(.+?)(?:\\/\\d+)?$`).exec(classe);
		if (!m) continue;
		const nom = m[1];
		if (nom.startsWith('[') || (!BUILTIN.test(nom) && !tokens.has(nom))) manquants.push(classe);
	}
	return manquants;
}

describe('PENDING_FIELD_CLASS — chaque couleur nommée est un jeton du thème', () => {
	// `app.css` est résolu depuis CE fichier, et non depuis le répertoire courant (E5-4).
	// `import.meta.url` est une chaîne `file:` ; on la convertit par `fileURLToPath`, car le
	// `URL` global de jsdom n'est pas accepté par `readFileSync`, et `?raw` rend une chaîne
	// vide pour un `.css` sous Vitest.
	const appCss = readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), '../../../app.css'), 'utf-8');

	it('la classe des champs en vol ne nomme que des jetons définis', () => {
		expect(PENDING_FIELD_CLASS).toMatch(/read-only:/);
		expect(missingThemeColors(PENDING_FIELD_CLASS, appCss)).toEqual([]);
	});

	describe('le contrôle lui-même (revue P5)', () => {
		const theme = '@theme {\n  --color-surface-alt: #f8fafc;\n  --color-brand-2: #000;\n}\n';

		it('repère un jeton absent, quel que soit l’utilitaire ou le préfixe', () => {
			expect(missingThemeColors('read-only:bg-input/50', theme)).toEqual(['bg-input/50']);
			expect(missingThemeColors('read-only:ring-input', theme)).toEqual(['ring-input']);
			expect(missingThemeColors('outline-x fill-x placeholder-x', theme)).toEqual(['outline-x', 'fill-x', 'placeholder-x']);
			expect(missingThemeColors('hover:!bg-absent', theme)).toEqual(['bg-absent']);
			expect(missingThemeColors('bg-[--x]', theme)).toEqual(['bg-[--x]']);
		});

		it('lit les noms à chiffre', () => {
			expect(missingThemeColors('bg-brand-2', theme)).toEqual([]);
			expect(missingThemeColors('bg-surface-2', theme)).toEqual(['bg-surface-2']);
		});

		it('ne cherche les jetons que dans @theme, commentaires retirés', () => {
			const ailleurs = `/* --color-x: red; */\n:root { --color-y: blue; }\n${theme}`;
			expect(missingThemeColors('bg-x text-y', ailleurs)).toEqual(['bg-x', 'text-y']);
		});

		it('ne rougit pas sur les couleurs intégrées ni sur les utilitaires non colorés', () => {
			expect(missingThemeColors('bg-white text-sm border-t border-2 text-black bg-slate-500 ring-2 shadow-sm', theme)).toEqual([]);
		});
	});
});

describe('sameBlockValues — un bloc ouvert adopte-t-il une version plus récente ? (revue P1, B-1)', () => {
	it('mêmes valeurs, aux normalisations près : oui', () => {
		expect(sameBlockValues({ phone: 'P', website: null }, { phone: ' P ', website: '' }, CONTACT)).toBe(true);
		expect(
			sameBlockValues({ country: 'CH' }, { country: null }, ['country'] as const, ADDRESS_NORMALIZERS),
		).toBe(true);
	});

	it('un seul champ du bloc a bougé : non', () => {
		expect(sameBlockValues({ phone: 'P', website: 'W' }, { phone: 'P', website: 'W2' }, CONTACT)).toBe(false);
	});
});

const CONTACT = ['phone', 'website'] as const;
const ADDRESS = ['street', 'building', 'postalCode', 'city', 'country'] as const;

function adresse(overrides: Partial<Record<(typeof ADDRESS)[number], string>> = {}) {
	return {
		street: 'Rue Test',
		building: '1',
		postalCode: '1000',
		city: 'Lausanne',
		country: 'CH',
		...overrides,
	};
}

describe('normalisation (comparaison sur les valeurs telles qu’envoyées)', () => {
	it('trim, et vide ⇔ null', () => {
		expect(normalizeCompanyValue('  a  ')).toBe('a');
		expect(normalizeCompanyValue(null)).toBe('');
		expect(normalizeCompanyValue(undefined)).toBe('');
		expect(normalizeCompanyValue('   ')).toBe('');
	});

	it('le pays comme le serveur le range : trim, ASCII en majuscules, vide = CH', () => {
		expect(normalizeCountry(' fr ')).toBe('FR');
		expect(normalizeCountry('')).toBe('CH');
		expect(normalizeCountry(null)).toBe('CH');
		expect(normalizeCountry('  ')).toBe('CH');
		// Seul l'ASCII passe en majuscules, comme `to_ascii_uppercase` côté serveur.
		expect(normalizeCountry('ß')).toBe('ß');
	});
});

describe('companyEditConflict — les trois cas', () => {
	it('aucun champ du bloc n’a changé (version bougée par un autre bloc) : unchanged, rien de nommé', () => {
		const base = adresse();
		const input = adresse({ postalCode: '1003' });
		const r = companyEditConflict(base, adresse(), input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('unchanged');
		expect(r.changed).toEqual([]);
		expect(r.merged).toEqual([]);
		expect(r.input).toEqual(input);
	});

	it('la valeur relue égale la saisie modifiée : already-saved', () => {
		const base = adresse();
		const input = adresse({ city: 'Morges' });
		const r = companyEditConflict(base, adresse({ city: 'Morges' }), input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('already-saved');
	});

	it('modifié des deux côtés, à des valeurs différentes : changed, le champ nommé, la saisie gardée', () => {
		const base = adresse();
		const input = adresse({ city: 'Morges' });
		const r = companyEditConflict(base, adresse({ city: 'Nyon' }), input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('changed');
		expect(r.changed).toEqual(['city']);
		expect(r.merged).toEqual([]);
		expect(r.input.city).toBe('Morges');
	});
});

describe('companyEditConflict — champ modifié ailleurs à la valeur de la saisie (AC-5 bis)', () => {
	const base = { phone: '+41 21 000 00 00', website: 'https://old.example' };
	const input = { phone: 'T', website: 'X' };

	it('a) l’autre écrit le site X seul : unchanged, aucun champ nommé', () => {
		const r = companyEditConflict(base, { ...base, website: 'X' }, input, CONTACT);
		expect(r.kind).toBe('unchanged');
		expect(r.changed).toEqual([]);
		expect(r.merged).toEqual([]);
	});

	it('b) l’autre écrit le site X et le téléphone U : seul le téléphone est nommé', () => {
		const r = companyEditConflict(base, { phone: 'U', website: 'X' }, input, CONTACT);
		expect(r.kind).toBe('changed');
		expect(r.changed).toEqual(['phone']);
	});
});

describe('companyEditConflict — coordonnées : chaque champ compte (M5)', () => {
	it('le site changé des deux côtés, à des valeurs différentes, est nommé', () => {
		const base = { phone: 'P', website: 'W' };
		const r = companyEditConflict(base, { phone: 'P', website: 'W2' }, { phone: 'P', website: 'W3' }, CONTACT);
		expect(r.kind).toBe('changed');
		expect(r.changed).toEqual(['website']);
	});
});

describe('companyEditConflict — formes des valeurs', () => {
	it('vide contre null : égaux', () => {
		const r = companyEditConflict({ email: null }, { email: '' }, { email: 'a@b.ch' }, ['email'] as const);
		expect(r.kind).toBe('unchanged');
	});

	it('les espaces ne font pas une différence (M4)', () => {
		const r = companyEditConflict(
			{ email: 'a@b.ch' },
			{ email: 'c@d.ch' },
			{ email: '  c@d.ch  ' },
			['email'] as const,
		);
		expect(r.kind).toBe('already-saved');
	});

	it('un pays en minuscules égale le pays rangé en majuscules', () => {
		const base = adresse({ country: 'CH' });
		const r = companyEditConflict(base, adresse({ country: 'FR' }), adresse({ country: 'fr' }), ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('already-saved');
	});

	it('un pays vide égale CH', () => {
		const base = adresse({ country: 'FR' });
		const r = companyEditConflict(base, adresse({ country: 'CH' }), adresse({ country: '' }), ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('already-saved');
	});

	it('une adresse retirée ailleurs (composants null) : chaque composant vaut vide', () => {
		const vide = { street: null, building: null, postalCode: null, city: null, country: null };
		const r = companyEditConflict(adresse(), vide, adresse({ city: 'Morges' }), ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('changed');
		expect(r.changed).toEqual(['city']);
		// Rue, n°, NPA non modifiés et vidés ailleurs : repris (vides). Le pays vide vaut CH : inchangé.
		expect(r.merged).toEqual(['street', 'building', 'postalCode']);
		expect(r.input.street).toBe('');
		expect(r.input.country).toBe('CH');
	});
});

describe('companyEditConflict — fusion à trois voies (AC-5 ter, C-e343-14)', () => {
	it('le NPA corrigé reste, la rue de l’autre est reprise, et le cas est unchanged', () => {
		const base = adresse();
		const input = adresse({ postalCode: '1003' });
		const stored = adresse({ street: 'Avenue Neuve' });
		const r = companyEditConflict(base, stored, input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('unchanged');
		expect(r.merged).toEqual(['street']);
		expect(r.changed).toEqual([]);
		expect(r.input).toEqual(adresse({ street: 'Avenue Neuve', postalCode: '1003' }));
	});

	it('fusion et conflit ensemble : la rue reprise, la ville nommée', () => {
		const r = companyEditConflict(
			adresse(),
			adresse({ street: 'Avenue Neuve', city: 'Nyon' }),
			adresse({ city: 'Morges' }),
			ADDRESS,
			ADDRESS_NORMALIZERS,
		);
		expect(r.kind).toBe('changed');
		expect(r.changed).toEqual(['city']);
		expect(r.merged).toEqual(['street']);
		expect(r.input).toEqual(adresse({ street: 'Avenue Neuve', city: 'Morges' }));
	});

	it('une saisie sans modification n’est jamais already-saved : unchanged, avec la fusion', () => {
		const base = adresse();
		const stored = adresse({ street: 'Avenue Neuve' });
		const r = companyEditConflict(base, stored, adresse(), ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('unchanged');
		expect(r.merged).toEqual(['street']);
		expect(r.input).toEqual(stored);
	});

	it('une saisie sans modification, et rien de changé : unchanged, sans fusion', () => {
		const r = companyEditConflict(adresse(), adresse(), adresse(), ADDRESS, ADDRESS_NORMALIZERS);
		expect(r.kind).toBe('unchanged');
		expect(r.merged).toEqual([]);
	});

	it('modifié et repris : tout égale la saisie après fusion → already-saved', () => {
		const r = companyEditConflict(
			adresse(),
			adresse({ street: 'Avenue Neuve', city: 'Morges' }),
			adresse({ city: 'Morges' }),
			ADDRESS,
			ADDRESS_NORMALIZERS,
		);
		expect(r.kind).toBe('already-saved');
	});
});

describe('companyEditConflict — second conflit, la copie relue (AC-5, M2)', () => {
	it('après un premier conflit sur la ville, une écriture sur la rue seule : la ville n’est plus en conflit', () => {
		const base0 = adresse();
		const input = adresse({ city: 'Morges' });
		const stored1 = adresse({ city: 'Nyon' });
		const r1 = companyEditConflict(base0, stored1, input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r1.changed).toEqual(['city']);
		// La copie prend les valeurs relues.
		const base1 = stored1;
		const stored2 = adresse({ city: 'Nyon', street: 'Avenue Neuve' });
		const r2 = companyEditConflict(base1, stored2, r1.input, ADDRESS, ADDRESS_NORMALIZERS);
		expect(r2.kind).toBe('unchanged');
		expect(r2.changed).toEqual([]);
		expect(r2.merged).toEqual(['street']);
		expect(r2.input.city).toBe('Morges');
	});
});
