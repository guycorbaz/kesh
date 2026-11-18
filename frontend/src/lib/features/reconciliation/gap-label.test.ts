/**
 * Story 43-3 (#823) — le sens de l'écart en mots (`gapSenseLabel`) : majuscule en tête de
 * libellé, minuscule après un deux-points (revue P2, L-5), sauf en allemand, où les noms gardent
 * leur majuscule (revue P3 ciblée, L-E : la branche allemande n'était couverte par aucun test).
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const locale = vi.hoisted(() => ({ value: 'fr-CH' }));
const libelles: Record<string, Record<string, string>> = {
	'fr-CH': { 'reconciliation-group-gap-client-under': 'Moins-payé de { $amount }' },
	'de-CH': { 'reconciliation-group-gap-client-under': 'Minderzahlung von { $amount }' },
	'it-CH': { 'reconciliation-group-gap-client-under': 'Pagamento inferiore di { $amount }' },
	'en-CH': { 'reconciliation-group-gap-client-under': 'Underpayment of { $amount }' },
};

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string, args?: Record<string, string | number>) => {
		const raw = libelles[locale.value]?.[key] ?? fallback;
		return args ? raw.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : raw;
	},
	i18nLocale: () => locale.value,
}));

import { gapSenseLabel } from './gap-label';

describe('gapSenseLabel — la casse selon la place et la langue', () => {
	beforeEach(() => {
		locale.value = 'fr-CH';
	});

	it('en tête de libellé, la majuscule reste, quelle que soit la langue', () => {
		for (const l of ['fr-CH', 'de-CH', 'it-CH', 'en-CH']) {
			locale.value = l;
			const label = gapSenseLabel('client', -4000n);
			expect(label.charAt(0)).toBe(label.charAt(0).toLocaleUpperCase());
		}
	});

	it.each([
		['fr-CH', 'moins-payé de 0.40'],
		['it-CH', 'pagamento inferiore di 0.40'],
		['en-CH', 'underpayment of 0.40'],
	])('%s, dans une phrase → minuscule (« %s »)', (l, attendu) => {
		locale.value = l;
		expect(gapSenseLabel('client', -4000n, true)).toBe(attendu);
	});

	it('de-CH, dans une phrase → le nom garde sa majuscule (« Minderzahlung von 0.40 »)', () => {
		locale.value = 'de-CH';
		expect(gapSenseLabel('client', -4000n, true)).toBe('Minderzahlung von 0.40');
	});
});
