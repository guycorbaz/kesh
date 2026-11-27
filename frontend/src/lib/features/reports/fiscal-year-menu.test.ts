// Story 55-1a (D15, AC-14, AC-15, AC-17) — les règles du sélecteur d'exercice.

import { describe, expect, it } from 'vitest';
import {
	anneesDe,
	enTetePeriode,
	etatDuMenu,
	formatJourMois,
	libellesExercices,
	nomParDefaut,
	type EtatMenu,
} from './fiscal-year-menu';

const fy = (id: number, name: string, startDate: string, endDate: string) => ({ id, name, startDate, endDate });
const liste = (items: string[]) => new Intl.ListFormat('fr', { type: 'conjunction' }).format(items);

describe('nomParDefaut — réplique de nom_par_defaut (exercice.rs)', () => {
	it('même année, exercice décalé, 29 février', () => {
		expect(nomParDefaut('2025-01-01', '2025-12-31')).toBe('Exercice 2025');
		expect(nomParDefaut('2025-07-01', '2026-06-30')).toBe('Exercice 2025/2026');
		expect(nomParDefaut('2024-02-29', '2025-02-28')).toBe('Exercice 2024/2025');
		expect(anneesDe('2025-07-01', '2026-06-30')).toBe('2025/2026');
	});
	it('formatJourMois : « 01.03 »', () => {
		expect(formatJourMois('2025-03-01')).toBe('01.03');
		expect(formatJourMois('2025-12-31')).toBe('31.12');
	});
});

describe('libellesExercices (AC-14)', () => {
	it('années pour un nom par défaut, nom saisi tel quel', () => {
		const m = libellesExercices([
			fy(1, 'Exercice 2025', '2025-01-01', '2025-12-31'),
			fy(2, 'Exercice 2025/2026', '2025-07-01', '2026-06-30'),
			fy(3, '  Compta TVA ', '2027-01-01', '2027-12-31'),
		]);
		expect([...m.values()]).toEqual(['2025', '2025/2026', '  Compta TVA ']);
	});

	it('mutation tuée : un nom saisi qui ressemble à un nom par défaut d’une autre forme reste tel quel', () => {
		// « Exercice 2026 » pour un exercice 2025 : ce n'est pas le nom que Kesh aurait donné.
		const m = libellesExercices([fy(1, 'Exercice 2026', '2025-01-01', '2025-12-31')]);
		expect(m.get(1)).toBe('Exercice 2026');
	});

	it('nom saisi égal aux années d’un autre : doublon séparé par jour-mois, sur ces seules entrées', () => {
		const m = libellesExercices([
			fy(1, 'Exercice 2025', '2025-01-01', '2025-06-30'),
			fy(2, '2025', '2025-07-01', '2025-12-31'),
			fy(3, 'Exercice 2026', '2026-01-01', '2026-12-31'),
		]);
		expect(m.get(1)).toBe('2025 (01.01–30.06)');
		expect(m.get(2)).toBe('2025 (01.07–31.12)');
		expect(m.get(3)).toBe('2026');
	});

	it('doublon que jour-mois ne sépare pas : bornes complètes', () => {
		const m = libellesExercices([
			fy(1, 'Compta', '2024-01-01', '2024-12-31'),
			fy(2, 'Compta', '2025-01-01', '2025-12-31'),
		]);
		expect(m.get(1)).toBe('Compta (01.01.2024–31.12.2024)');
		expect(m.get(2)).toBe('Compta (01.01.2025–31.12.2025)');
	});
});

describe('etatDuMenu (AC-15)', () => {
	const FYS = [
		fy(1, 'Exercice 2025', '2025-01-01', '2025-12-31'),
		fy(2, 'Exercice 2026', '2026-01-01', '2026-12-31'),
		// non contigu : 2027 manque
		fy(3, 'Exercice 2028', '2028-01-01', '2028-12-31'),
	];
	const prec: EtatMenu = { kind: 'fy', id: 2, whole: true };

	it('bornes exactes : l’exercice, sans lien « Exercice entier »', () => {
		expect(etatDuMenu('2025-01-01', '2025-12-31', FYS, prec)).toEqual({ kind: 'fy', id: 1, whole: true });
	});
	it('partie d’un exercice : l’exercice, avec le lien (mutation `<` au lieu de `≤` tuée par les bornes)', () => {
		expect(etatDuMenu('2025-01-01', '2025-03-31', FYS, prec)).toEqual({ kind: 'fy', id: 1, whole: false });
		expect(etatDuMenu('2025-12-31', '2025-12-31', FYS, prec)).toEqual({ kind: 'fy', id: 1, whole: false });
	});
	it('deux exercices : « Plusieurs »', () => {
		expect(etatDuMenu('2025-06-01', '2026-06-30', FYS, prec)).toEqual({ kind: 'multiple' });
	});
	it('une date hors, les deux hors, exercices non contigus : « Hors exercice »', () => {
		expect(etatDuMenu('2024-12-31', '2025-01-31', FYS, prec)).toEqual({ kind: 'outside' });
		expect(etatDuMenu('2027-01-01', '2027-02-01', FYS, prec)).toEqual({ kind: 'outside' });
		expect(etatDuMenu('2026-06-01', '2028-06-01', FYS, prec)).toEqual({ kind: 'multiple' });
	});
	it('début après fin : le précédent (mutation « Hors exercice » tuée)', () => {
		expect(etatDuMenu('2026-06-30', '2026-01-01', FYS, prec)).toBe(prec);
	});
	it('date vide ou incomplète : le précédent', () => {
		expect(etatDuMenu('', '2026-01-01', FYS, prec)).toBe(prec);
		expect(etatDuMenu('2026-01-01', '0202-01', FYS, prec)).toBe(prec);
	});
});

describe('enTetePeriode (AC-17)', () => {
	const FYS = [
		fy(1, 'Exercice 2024/2025', '2024-07-01', '2025-06-30'),
		fy(2, 'Exercice 2025/2026', '2025-07-01', '2026-06-30'),
		fy(3, 'Compta TVA', '2026-07-01', '2027-06-30'),
	];
	it('exercice entier, nom par défaut', () => {
		expect(enTetePeriode('2024-07-01', '2025-06-30', FYS, liste)).toEqual({
			key: 'reports-nav-heading-fy',
			args: { years: '2024/2025', from: '01.07.2024', to: '30.06.2025' },
		});
	});
	it('exercice entier, nom saisi', () => {
		expect(enTetePeriode('2026-07-01', '2027-06-30', FYS, liste)).toEqual({
			key: 'reports-nav-heading-named',
			args: { name: 'Compta TVA', from: '01.07.2026', to: '30.06.2027' },
		});
	});
	it('partie d’un exercice : années, ou nom saisi entre guillemets', () => {
		expect(enTetePeriode('2026-01-01', '2026-03-31', FYS, liste)).toEqual({
			key: 'reports-nav-heading-part',
			args: { fiscalYear: '2025/2026', from: '01.01.2026', to: '31.03.2026' },
		});
		expect(enTetePeriode('2026-08-01', '2026-08-31', FYS, liste).args).toMatchObject({ fiscalYear: '« Compta TVA »' });
	});
	it('plusieurs exercices : la liste par Intl.ListFormat (trois exercices)', () => {
		expect(enTetePeriode('2025-01-01', '2026-12-31', FYS, liste)).toEqual({
			key: 'reports-nav-heading-several',
			args: { fiscalYears: '2024/2025, 2025/2026 et Compta TVA', from: '01.01.2025', to: '31.12.2026' },
		});
	});
	it('hors exercice', () => {
		expect(enTetePeriode('2023-01-01', '2024-12-31', FYS, liste).key).toBe('reports-nav-heading-outside');
	});
});
