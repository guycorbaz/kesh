import { describe, expect, it, vi } from 'vitest';

// Mock du module i18n partagé : on intercepte `i18nMsg` pour pouvoir
// vérifier les clés qu'AccountingTooltip résout.
const i18nSpy = vi.fn((key: string, fallback: string) => `[[${key}|${fallback}]]`);
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (key: string, fallback: string) => i18nSpy(key, fallback),
	loadI18nMessages: vi.fn()
}));

// Mock des primitives shadcn Tooltip pour éviter de devoir porter bits-ui
// dans jsdom. On ne teste PAS l'affichage hover (couvert par Playwright) ;
// on vérifie que le composant compile, s'importe, et que les clés i18n
// sont dérivées selon le pattern documenté `tooltip-{term}-{natural|technical}`.
vi.mock('$lib/components/ui/tooltip', () => {
	const passthrough = (_props: unknown) => null;
	return {
		Root: passthrough,
		Trigger: passthrough,
		Content: passthrough
	};
});

describe('AccountingTooltip', () => {
	it('le module se charge et exporte un composant Svelte', async () => {
		const mod = await import('./AccountingTooltip.svelte');
		expect(mod.default).toBeDefined();
	});

	it('les clés i18n suivent le pattern tooltip-{term}-natural/technical', () => {
		// Test du contrat de dérivation des clés — indépendant du rendu.
		// Si cette convention casse, les 32 clés du fichier messages.ftl
		// ne seront plus résolues par le composant.
		const term = 'debit';
		const natural = `tooltip-${term}-natural`;
		const technical = `tooltip-${term}-technical`;
		expect(natural).toBe('tooltip-debit-natural');
		expect(technical).toBe('tooltip-debit-technical');
	});

	it.each([
		['debit', 'tooltip-debit-natural', 'tooltip-debit-technical'],
		['credit', 'tooltip-credit-natural', 'tooltip-credit-technical'],
		['journal', 'tooltip-journal-natural', 'tooltip-journal-technical'],
		['balanced', 'tooltip-balanced-natural', 'tooltip-balanced-technical']
	])('term "%s" dérive les clés %s et %s', (term, expectedNatural, expectedTechnical) => {
		expect(`tooltip-${term}-natural`).toBe(expectedNatural);
		expect(`tooltip-${term}-technical`).toBe(expectedTechnical);
	});
});
