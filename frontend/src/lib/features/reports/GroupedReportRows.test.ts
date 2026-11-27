// Story 46-2 (AC-4) — rendu des lignes groupées : règle de rendu de la 46-1
// (montant sur le titre en `header` ou si fermé, sinon en pied), solde propre,
// devise d'un groupe, lien vers le grand livre, accordéon et retrait.

import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
}));

import GroupedReportRows, { indentOf, showsAmount } from './GroupedReportRows.svelte';
import type { GroupedRow } from './reports.types';

const period = { startDate: '2026-01-01', endDate: '2026-12-31' };
const columns = [{ id: 'balance', amount: (r: GroupedRow) => r.balance }];

/** Groupe 10 (ouvert) : solde propre, compte 1020 en USD, compte 1000 archivé. */
function lignes(position: 'header' | 'footer', open = true): GroupedRow[] {
	const groupe: GroupedRow = {
		kind: 'group',
		depth: 1,
		accountId: 10,
		accountNumber: '10',
		accountName: 'Actifs circulants',
		balance: '1500.005',
		open,
	};
	if (!open) return [groupe];
	const rows: GroupedRow[] = [
		groupe,
		{ kind: 'own', depth: 2, accountId: 10, accountNumber: '10', accountName: 'Actifs circulants', balance: '100', active: true },
		{ kind: 'account', depth: 2, accountId: 1000, accountNumber: '1000', accountName: 'Caisse', balance: '400', active: false },
		{ kind: 'account', depth: 2, accountId: 1020, accountNumber: '1020', accountName: 'Banque USD', balance: '1000.005', currency: 'USD', balanceCurrency: '1100', active: true },
	];
	if (position === 'footer') rows.push({ ...groupe, kind: 'subtotal', open: undefined });
	return rows;
}

function rendre(rows: GroupedRow[], position: 'header' | 'footer', onToggle = vi.fn()) {
	const table = document.createElement('table');
	const tbody = document.createElement('tbody');
	table.appendChild(tbody);
	document.body.appendChild(table);
	const r = render(GroupedReportRows, {
		target: tbody,
		props: { rows, subtotalPosition: position, columns, period, onToggle },
	});
	return { ...r, onToggle };
}

const montant = (tr: Element | null) =>
	tr?.querySelector('[data-testid="grouped-amount-balance"]')?.textContent?.trim();

describe('GroupedReportRows — règle de rendu (46-1)', () => {
	it('showsAmount : header ou fermé sur le titre, sinon en pied', () => {
		const g = lignes('header')[0];
		expect(showsAmount(g, 'header')).toBe(true);
		expect(showsAmount(g, 'footer')).toBe(false);
		expect(showsAmount({ ...g, open: false }, 'footer')).toBe(true);
		expect(showsAmount({ ...g, kind: 'subtotal' }, 'footer')).toBe(true);
	});

	it('header : le sous-total, arrondi au centime, sur la ligne du titre, aucune ligne de pied', () => {
		const { container } = rendre(lignes('header'), 'header');
		expect(montant(container.querySelector('[data-testid="grouped-group-10"]'))).toBe('1’500.01');
		expect(container.querySelector('[data-testid="grouped-subtotal-10"]')).toBeNull();
	});

	it('footer : titre sans montant, ligne « Total <nom> » en gras avec le sous-total', () => {
		const { container } = rendre(lignes('footer'), 'footer');
		expect(montant(container.querySelector('[data-testid="grouped-group-10"]'))).toBe('');
		const pied = container.querySelector('[data-testid="grouped-subtotal-10"]');
		expect(pied?.textContent).toContain('Total Actifs circulants');
		expect(pied?.classList.contains('font-semibold')).toBe(true);
		expect(montant(pied)).toBe('1’500.01');
		// la ligne de pied est la dernière, après la descendance
		const rows = [...container.querySelectorAll('tr')];
		expect(rows.at(-1)).toBe(pied);
	});

	it('footer, groupe fermé : le montant reste sur le titre, la descendance est absente', () => {
		const { container } = rendre(lignes('footer', false), 'footer');
		expect(montant(container.querySelector('[data-testid="grouped-group-10"]'))).toBe('1’500.01');
		expect(container.querySelectorAll('tr')).toHaveLength(1);
	});

	it('solde propre libellé « <nom> (solde propre) », lié au grand livre', () => {
		const { container } = rendre(lignes('header'), 'header');
		const own = container.querySelector('[data-testid="grouped-own-10"]');
		expect(own?.textContent).toContain('Actifs circulants (solde propre)');
		expect(own?.querySelector('a')?.getAttribute('href')).toContain('accountId=10');
	});

	it('lien vers le grand livre sur les comptes, jamais sur un groupe ni un pied', () => {
		const { container } = rendre(lignes('footer'), 'footer');
		expect(container.querySelector('[data-testid="grouped-account-1000"] a')?.getAttribute('href')).toBe(
			'/reports?tab=general-ledger&accountId=1000&from=2026-01-01&to=2026-12-31',
		);
		expect(container.querySelector('[data-testid="grouped-group-10"] a')).toBeNull();
		expect(container.querySelector('[data-testid="grouped-subtotal-10"] a')).toBeNull();
	});

	it('compte archivé estompé', () => {
		const { container } = rendre(lignes('header'), 'header');
		expect(container.querySelector('[data-testid="grouped-account-1000"]')?.classList.contains('opacity-60')).toBe(true);
		expect(container.querySelector('[data-testid="grouped-account-1020"]')?.classList.contains('opacity-60')).toBe(false);
	});

	it('devise : sur le compte, et sur le groupe là où son montant est rendu', () => {
		const rows = lignes('footer').map((r) =>
			r.accountId === 10 && (r.kind === 'group' || r.kind === 'subtotal')
				? { ...r, currency: 'USD', balanceCurrency: '1650.25' }
				: r,
		);
		const { getByTestId, getAllByTestId } = rendre(rows, 'footer');
		expect(getByTestId('account-fx-1020').textContent).toBe('USD 1’100.00');
		// footer, groupe ouvert : la devise du groupe est sur le pied, une seule fois
		expect(getAllByTestId('group-fx-10')).toHaveLength(1);
		expect(getByTestId('grouped-subtotal-10').textContent).toContain('USD 1’650.25');
	});

	it('groupe sans devise : aucune mention ni ligne ajoutée', () => {
		const { container } = rendre(lignes('header'), 'header');
		expect(container.querySelector('[data-testid^="group-fx-"]')).toBeNull();
		expect(container.querySelectorAll('tr')).toHaveLength(4);
	});

	it('accordéon : aria-expanded, libellé, et clic transmis avec la ligne', async () => {
		const { getByTestId, onToggle } = rendre(lignes('header'), 'header');
		const btn = getByTestId('group-toggle-10');
		expect(btn.getAttribute('aria-expanded')).toBe('true');
		expect(btn.getAttribute('aria-label')).toBe('Fermer le groupe 10 Actifs circulants');
		await fireEvent.click(btn);
		expect(onToggle).toHaveBeenCalledWith(expect.objectContaining({ accountId: 10, open: true }));
	});

	it('accordéon fermé : « Ouvrir le groupe »', () => {
		const { getByTestId } = rendre(lignes('header', false), 'header');
		const btn = getByTestId('group-toggle-10');
		expect(btn.getAttribute('aria-expanded')).toBe('false');
		expect(btn.getAttribute('aria-label')).toBe('Ouvrir le groupe 10 Actifs circulants');
	});

	it('retrait croissant avec la profondeur, borné', () => {
		expect(indentOf(1)).toBe('0.5rem');
		expect(indentOf(2)).toBe('1.75rem');
		expect(indentOf(11)).toBe(indentOf(30));
		const { container } = rendre(lignes('header'), 'header');
		const [numero, libelle] = container.querySelectorAll<HTMLElement>('[data-testid="grouped-account-1000"] td');
		expect(numero.style.paddingLeft).toBe('1.75rem');
		expect(libelle.style.paddingLeft).toBe('1.75rem');
		const groupe = container.querySelectorAll<HTMLElement>('[data-testid="grouped-group-10"] td');
		expect(groupe[1].style.paddingLeft).toBe('0.5rem');
	});
});

// Story 55-2 (AC-1, AC-16) — filet de non-régression du lien inchangé (passé par `AccountLink`).
describe('GroupedReportRows — Story 55-2, lien de compte', () => {
	it('lignes `account` et `own` : lien exact, titre, classe LINK_CLASS ; ligne `group` en texte', async () => {
		const { LINK_CLASS } = await import('$lib/shared/navigation/link-class');
		const { container } = rendre(lignes('header'), 'header');
		const a = container.querySelector('[data-testid="grouped-account-1020"] a')!;
		expect(a.getAttribute('href')).toBe('/reports?tab=general-ledger&accountId=1020&from=2026-01-01&to=2026-12-31');
		expect(a.getAttribute('title')).toBe('Voir le détail dans le grand livre');
		expect(a.getAttribute('class')).toBe(LINK_CLASS);
		expect(container.querySelector('[data-testid="grouped-own-10"] a')?.getAttribute('class')).toBe(LINK_CLASS);
		expect(container.querySelector('[data-testid="grouped-group-10"] a')).toBeNull();
	});
});
