/**
 * La garde de locale de `global-setup.ts` (`locale.ts`) : elle échoue sur une locale absente,
 * se tait sur C, POSIX, C.UTF-8 ou une variable absente, et compare les graphies de la glibc.
 */
import { describe, expect, it, vi } from 'vitest';
import {
	localesManquantes,
	messageLocaleManquante,
	normaliserLocale,
	verifierLocaleInstallee,
} from './locale';

/** Une sortie de `locale -a` d'un poste où seules C et deux locales sont installées. */
const INSTALLEES = ['C', 'C.utf8', 'POSIX', 'en_US.utf8', 'fr_CH.utf8'];

describe('normaliserLocale', () => {
	it('ramène le jeu de caractères à la graphie de locale -a', () => {
		expect(normaliserLocale('fr_CH.UTF-8')).toBe('fr_CH.utf8');
		expect(normaliserLocale('fr_CH.utf8')).toBe('fr_CH.utf8');
		expect(normaliserLocale('de_DE.ISO-8859-15@euro')).toBe('de_DE.iso885915@euro');
	});

	it('laisse une locale sans jeu de caractères telle quelle', () => {
		expect(normaliserLocale('fr_CH')).toBe('fr_CH');
	});
});

describe('localesManquantes', () => {
	it('reconnaît fr_CH.UTF-8 listée fr_CH.utf8', () => {
		expect(localesManquantes({ LANG: 'fr_CH.UTF-8' }, INSTALLEES)).toEqual([]);
	});

	it('signale une locale absente, avec sa variable', () => {
		expect(localesManquantes({ LANG: 'de_LI.UTF-8' }, INSTALLEES)).toEqual([
			{ variable: 'LANG', valeur: 'de_LI.UTF-8' },
		]);
	});

	it('contrôle aussi LC_ALL et LC_CTYPE', () => {
		expect(
			localesManquantes({ LC_ALL: 'it_CH.UTF-8', LC_CTYPE: 'de_LI.UTF-8', LANG: 'fr_CH.UTF-8' }, INSTALLEES),
		).toEqual([
			{ variable: 'LC_ALL', valeur: 'it_CH.UTF-8' },
			{ variable: 'LC_CTYPE', valeur: 'de_LI.UTF-8' },
		]);
	});

	it('ignore les variables absentes, vides, C, POSIX et C.UTF-8', () => {
		for (const valeur of [undefined, '', 'C', 'POSIX', 'C.UTF-8', 'C.utf8']) {
			expect(localesManquantes({ LANG: valeur, LC_ALL: valeur }, ['C', 'POSIX'])).toEqual([]);
		}
	});
});

describe('verifierLocaleInstallee', () => {
	it('échoue sur une locale absente, en disant quoi installer', () => {
		expect(() => verifierLocaleInstallee({ LANG: 'de_LI.UTF-8' }, () => INSTALLEES)).toThrow(
			/sudo locale-gen de_LI\.UTF-8/,
		);
	});

	it('passe sur une locale installée', () => {
		expect(() => verifierLocaleInstallee({ LANG: 'fr_CH.UTF-8' }, () => INSTALLEES)).not.toThrow();
	});

	it('ne lit pas locale -a quand rien n’est à contrôler (la CI : C.UTF-8 ou rien)', () => {
		const lister = vi.fn(() => INSTALLEES);
		verifierLocaleInstallee({ LANG: 'C.UTF-8' }, lister);
		verifierLocaleInstallee({}, lister);
		expect(lister).not.toHaveBeenCalled();
	});

	it('avertit sans échouer quand locale -a est indisponible', () => {
		const avert = vi.spyOn(console, 'warn').mockImplementation(() => {});
		expect(() => verifierLocaleInstallee({ LANG: 'de_LI.UTF-8' }, () => null)).not.toThrow();
		expect(avert).toHaveBeenCalledOnce();
		avert.mockRestore();
	});
});

describe('messageLocaleManquante', () => {
	it('nomme la variable, la commande d’installation et le contrôle', () => {
		const m = messageLocaleManquante([{ variable: 'LANG', valeur: 'fr_CH.UTF-8' }]);
		expect(m).toContain('LANG=fr_CH.UTF-8');
		expect(m).toContain('sudo locale-gen fr_CH.UTF-8');
		expect(m).toContain('locale -a | grep -i fr_CH');
	});
});
