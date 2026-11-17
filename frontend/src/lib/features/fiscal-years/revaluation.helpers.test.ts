// Story 37-5 — logique pure de l'écran de réévaluation : cours envoyés à
// l'aperçu et à la passation, boutons de récupération, libellés de source et
// de genre (A30), signe naturel.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
	i18nLocale: () => 'fr-CH'
}));

import {
	canFetchDay,
	canFetchMonth,
	canPost,
	dayNotYetPublished,
	differsFromDefault,
	kindLabel,
	modifiedRates,
	naturalAmount,
	postRequest,
	previewRequest,
	sourceLabel
} from './revaluation.helpers';
import { line, preview, rate } from './revaluation.test-fixtures';

const fallback = rate({
	rate: '0.8846000000',
	rateKind: 'month',
	rateDate: '2024-12-01',
	defaultKind: 'month',
	defaultRate: '0.8846000000',
	sourceDate: null
});

describe('cours envoyés', () => {
	it('l’aperçu ne reçoit que les cours modifiés, des devises cochées', () => {
		const p = preview({
			rates: [rate(), rate({ currency: 'EUR', defaultRate: '0.94', rate: '0.94' })],
			lines: [line(), line({ accountId: 11, currency: 'EUR' })]
		});
		const inputs = { USD: '0.911930', EUR: '0.95' };
		// USD égal au défaut (zéros terminaux ignorés) : non envoyé ; EUR modifié.
		expect(previewRequest(p, inputs, new Set([10, 11]))).toEqual({
			accountIds: [10, 11],
			rates: { EUR: '0.95' }
		});
		// EUR décoché : son cours ne part pas (la requête refuserait la clé).
		expect(previewRequest(p, inputs, new Set([10]))).toEqual({ accountIds: [10], rates: {} });
		expect(modifiedRates({ USD: '' }, p.rates, new Set(['USD']))).toEqual({});
	});

	it('la passation reçoit les cours affichés des seules devises cochées', () => {
		const p = preview({
			rates: [rate(), rate({ currency: 'EUR' })],
			lines: [line(), line({ accountId: 11, currency: 'EUR' })]
		});
		expect(postRequest(p, { USD: '0.91193', EUR: '0,95' }, new Set([10]))).toEqual({
			accountIds: [10],
			rates: { USD: '0.91193' }
		});
	});
});

describe('boutons de récupération (AC-1, AC-6)', () => {
	it('repli mensuel au genre day, date passée : « Récupérer le cours du jour »', () => {
		const p = preview({ rates: [fallback] });
		expect(canFetchDay(p, fallback, '2025-01-15')).toBe(true);
		expect(canFetchMonth(p, fallback, '2025-01-15')).toBe(false);
		expect(dayNotYetPublished(p, fallback, '2025-01-15')).toBe(false);
	});

	it('date de fin non passée : pas de bouton du jour, le message à la place', () => {
		const p = preview({ rates: [fallback] });
		expect(canFetchDay(p, fallback, '2024-12-31')).toBe(false);
		expect(dayNotYetPublished(p, fallback, '2024-12-31')).toBe(true);
	});

	it('genre month : jamais de bouton du jour ; « Récupérer le mois » sans cours, mois publié', () => {
		const p = preview({ closingRateKind: 'month' });
		const sans = rate({ rate: null, rateKind: null, defaultRate: null, defaultSource: null, defaultKind: null, rateDate: null });
		expect(canFetchDay(p, sans, '2025-06-01')).toBe(false);
		expect(canFetchMonth(p, sans, '2025-06-01')).toBe(true);
		// Mois de fin au-delà du mois suivant le mois courant : pas proposé.
		expect(canFetchMonth(p, sans, '2024-10-15')).toBe(false);
		expect(canFetchMonth(p, sans, '2024-11-15')).toBe(true);
	});

	it('cours du jour présent : aucun bouton', () => {
		const p = preview();
		expect(canFetchDay(p, p.rates[0], '2025-01-15')).toBe(false);
		expect(canFetchMonth(p, p.rates[0], '2025-01-15')).toBe(false);
	});
});

describe('libellés', () => {
	it('source : AFC, cours saisi, cours saisi (ecb), manuel', () => {
		expect(sourceLabel(rate())).toBe('AFC');
		expect(sourceLabel(rate({ defaultSource: 'manual' }))).toBe('cours saisi');
		expect(sourceLabel(rate({ defaultSource: 'ecb' }))).toBe('cours saisi (ecb)');
		expect(sourceLabel(rate({ rateKind: 'request', overridden: true }))).toBe('manuel');
	});

	it('genre : jour (date de publication), repli mensuel signalé, mois choisi', () => {
		const p = preview();
		expect(kindLabel(p, rate({ sourceDate: '2024-12-30' }), 'fr-CH')).toBe(
			'cours du 31.12.2024 (publié le 30.12.2024)'
		);
		expect(kindLabel(p, rate(), 'fr-CH')).toBe('cours du 31.12.2024');
		expect(kindLabel(p, fallback, 'fr-CH')).toMatch(/^cours du jour de clôture indisponible : cours mensuel de décembre 2024$/);
		expect(kindLabel(preview({ closingRateKind: 'month' }), fallback, 'fr-CH')).toBe(
			'cours mensuel de décembre 2024'
		);
	});

	it('le champ diffère du cours retenu par la société', () => {
		expect(differsFromDefault('0.9119300', rate())).toBe(false);
		expect(differsFromDefault('0.92', rate())).toBe(true);
		expect(differsFromDefault('', rate())).toBe(false);
	});
});

describe('signe naturel et bouton', () => {
	it('un passif se lit au crédit', () => {
		expect(naturalAmount(line({ accountType: 'Liability' }), '-500.0000')).toBe('500');
		expect(naturalAmount(line(), '1000.0000')).toBe('1000');
	});

	it('« Passer les écritures » : désactivé avec un blocker ou une sélection vide', () => {
		expect(canPost(preview(), new Set([10]))).toBe(true);
		expect(canPost(preview(), new Set())).toBe(false);
		expect(
			canPost(preview({ blockers: [{ code: 'PERIOD_LOCKED', message: 'm', details: {} }] }), new Set([10]))
		).toBe(false);
	});
});
