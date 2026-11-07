// Story 33-4 (#606) — un filtre de date ne part qu'une fois la date complète.
import { describe, it, expect, vi } from 'vitest';
import {
	estDateStockable,
	filtreDeLAdresse,
	saisieDeFiltre,
	sortieDeFiltre,
} from './date-filtre';

/** Un champ date réel : jsdom connaît `type="date"` et sa valeur. */
function champ(valeur: string, badInput = false): Event {
	const input = document.createElement('input');
	input.type = 'date';
	input.value = valeur;
	if (badInput) {
		Object.defineProperty(input, 'validity', { value: { badInput: true } });
	}
	return { target: input } as unknown as Event;
}

describe('estDateStockable', () => {
	it('la plage du serveur et le calendrier', () => {
		for (const bon of ['1000-01-01', '2026-02-28', '2028-02-29', '9999-12-31']) {
			expect(estDateStockable(bon), bon).toBe(true);
		}
		for (const mauvais of [
			'0002-11-06',
			'0020-11-06',
			'0202-11-06',
			'0999-12-31',
			'2026-02-30',
			'2026-13-01',
			'+10000-01-01',
			'26-01-01',
			'',
			null,
			undefined,
		]) {
			expect(estDateStockable(mauvais), String(mauvais)).toBe(false);
		}
	});
});

describe('saisieDeFiltre', () => {
	it('une année en cours de frappe n’applique rien (mutation : appliquer sans contrôle)', () => {
		const appliquer = vi.fn();
		for (const v of ['0002-11-06', '0020-11-06', '0202-11-06']) {
			saisieDeFiltre(champ(v), appliquer);
		}
		expect(appliquer).not.toHaveBeenCalled();
	});

	it('la date complète s’applique', () => {
		const appliquer = vi.fn();
		saisieDeFiltre(champ('2026-11-06'), appliquer);
		expect(appliquer).toHaveBeenCalledExactlyOnceWith('2026-11-06');
	});

	it('effacer le champ retire le filtre ; un segment vidé, non (`badInput`)', () => {
		const appliquer = vi.fn();
		saisieDeFiltre(champ('', true), appliquer);
		expect(appliquer).not.toHaveBeenCalled();
		saisieDeFiltre(champ(''), appliquer);
		expect(appliquer).toHaveBeenCalledExactlyOnceWith('');
	});
});

describe('sortieDeFiltre', () => {
	const valeur = (e: Event) => (e.target as HTMLInputElement).value;

	it('une date restée incomplète retrouve celle du filtre', () => {
		const e = champ('0202-11-06');
		sortieDeFiltre(e, '2026-01-31');
		expect(valeur(e)).toBe('2026-01-31');
		const sansFiltre = champ('0202-11-06');
		sortieDeFiltre(sansFiltre, '');
		expect(valeur(sansFiltre)).toBe('');
		const demi = champ('', true);
		sortieDeFiltre(demi, '2026-01-31');
		expect(valeur(demi)).toBe('2026-01-31');
	});

	it('une date complète ou un champ effacé ne sont pas réécrits', () => {
		const e = champ('2026-11-06');
		sortieDeFiltre(e, '2026-01-31');
		expect(valeur(e)).toBe('2026-11-06');
		const vide = champ('');
		sortieDeFiltre(vide, '2026-01-31');
		expect(valeur(vide)).toBe('');
	});
});

describe('filtreDeLAdresse', () => {
	it('ce que le serveur refuserait n’est pas un filtre', () => {
		expect(filtreDeLAdresse('2026-11-06')).toBe('2026-11-06');
		for (const v of ['0202-11-06', '+10000-01-01', 'hier', '', null, undefined]) {
			expect(filtreDeLAdresse(v), String(v)).toBe('');
		}
	});
});
