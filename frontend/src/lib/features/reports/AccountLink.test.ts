// Story 55-2 (D6, AC-1, AC-16) — le lien de compte des Rapports : `href` exact (sortie de `ledgerHref`,
// inchangée à l'octet, D6), titre, classe `LINK_CLASS`, repli en texte.
//
// ⚠️ Chaque test nomme la mutation qu'il attrape.

import { describe, expect, it, vi } from 'vitest';
import { render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import AccountLink from './AccountLink.svelte';
import { LINK_CLASS, LINK_UNDERLINE_CLASS } from '$lib/shared/navigation/link-class';

const TITRE = 'Voir le détail dans le grand livre';

describe('AccountLink (55-2)', () => {
	it('rend le numéro en lien vers le grand livre du compte, sur la période donnée', () => {
		const { getByRole } = render(AccountLink, {
			accountId: 42,
			number: '1020',
			from: '2026-01-01',
			to: '2026-06-30',
			testId: 'lien-1020',
		});
		const a = getByRole('link', { name: '1020' });
		// Mutation : ordre des paramètres, ou une autre fonction que `ledgerHref` → rouge.
		expect(a.getAttribute('href')).toBe('/reports?tab=general-ledger&accountId=42&from=2026-01-01&to=2026-06-30');
		expect(a.getAttribute('title')).toBe(TITRE);
		// Mutation : une autre classe que `LINK_CLASS` → rouge.
		expect(a.getAttribute('class')).toBe(LINK_CLASS);
		expect(a.getAttribute('data-testid')).toBe('lien-1020');
	});

	it('la classe est la décision D17, à l’octet : couleur et soulignement léger au repos', () => {
		// Mutation : `LINK_CLASS` sans `underline` (le lien ne se voit plus sans la couleur) → rouge.
		expect(LINK_CLASS).toBe(
			'text-indigo-700 underline decoration-indigo-700/40 underline-offset-2 hover:decoration-current',
		);
		expect(LINK_CLASS.split(' ')).toContain('underline');
		expect(LINK_UNDERLINE_CLASS).toBe('underline decoration-current/40 underline-offset-2 hover:decoration-current');
		expect(LINK_UNDERLINE_CLASS.split(' ')).toContain('underline');
	});

	it('sans compte (`accountId` nul) : le numéro en texte, sans lien', () => {
		const { queryByRole, getByTestId } = render(AccountLink, {
			accountId: null,
			number: '9999',
			from: '2026-01-01',
			to: '2026-12-31',
			testId: 't',
		});
		// Mutation : lien rendu quand même → rouge.
		expect(queryByRole('link')).toBeNull();
		expect(getByTestId('t').textContent).toBe('9999');
	});

	it('`linked={false}` : le numéro en texte, même avec un compte', () => {
		const { queryByRole, container } = render(AccountLink, {
			accountId: 7,
			number: '1020',
			from: '2026-01-01',
			to: '2026-12-31',
			linked: false,
		});
		expect(queryByRole('link')).toBeNull();
		expect(container.textContent).toBe('1020');
	});
});
