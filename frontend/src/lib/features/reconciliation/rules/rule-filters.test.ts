// Story 38-11d (AC-2) — filtres facultatifs d'une règle de rapprochement :
// fonctions pures, champs du formulaire, corps envoyés (posés, vidés, jamais
// touchés), affichage dans la liste.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { ReconciliationRule } from './rules.types';

vi.mock('./rules.api', () => ({
	createRule: vi.fn(async () => ({})),
	updateRule: vi.fn(async () => ({})),
	deleteRule: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

vi.mock('$lib/features/projects/projects.api', () => ({
	listProjects: async () => [],
}));

import RuleFormModal from './RuleFormModal.svelte';
import RulesList from './RulesList.svelte';
import { createRule, updateRule } from './rules.api';
import {
	changedFilters,
	directionPayload,
	emailFilterPayload,
	isValidEmailFilter,
	normalizeEmailFilter,
	textFilterPayload,
} from './rule-filters';

const accounts: AccountResponse[] = [
	{
		id: 10,
		companyId: 1,
		number: '6510',
		name: 'Télécom',
		accountType: 'Expense',
		active: true,
		role: null,
		postable: true,
	} as AccountResponse,
];

function rule(overrides: Partial<ReconciliationRule> = {}): ReconciliationRule {
	return {
		id: 5,
		label: 'Hébergement',
		matchType: 'counterparty_contains',
		matchValue: 'Hosting',
		counterpartyAccountId: 10,
		priority: 100,
		active: true,
		defaultProjectId: null,
		providerType: null,
		counterpartyEmail: null,
		direction: null,
		appliedCount: 0,
		lastAppliedAt: null,
		version: 2,
		createdAt: '2026-11-20T00:00:00',
		updatedAt: '2026-11-20T00:00:00',
		...overrides,
	};
}

async function fill(el: HTMLElement, value: string) {
	await fireEvent.input(el, { target: { value } });
}

describe('rule-filters — fonctions pures', () => {
	it('normalise l’e-mail comme le serveur (blancs retirés, minuscules)', () => {
		expect(normalizeEmailFilter(' Ventes @Exemple.CH ')).toBe('ventes@exemple.ch');
	});

	it('accepte vide ou nom@domaine, refuse le reste', () => {
		expect(isValidEmailFilter('')).toBe(true);
		expect(isValidEmailFilter('  ')).toBe(true);
		expect(isValidEmailFilter('a@b')).toBe(true);
		expect(isValidEmailFilter('ab')).toBe(false);
		expect(isValidEmailFilter('@b')).toBe(false);
		expect(isValidEmailFilter('a@')).toBe(false);
		expect(isValidEmailFilter('a@b@c')).toBe(false);
	});

	it('vide → null, sinon la valeur rognée ou normalisée', () => {
		expect(textFilterPayload('  ')).toBeNull();
		expect(textFilterPayload(' General Payment ')).toBe('General Payment');
		expect(emailFilterPayload('')).toBeNull();
		expect(emailFilterPayload('A@B.CH')).toBe('a@b.ch');
		expect(directionPayload('')).toBeNull();
		expect(directionPayload('debit')).toBe('debit');
	});

	it('changedFilters : seuls les filtres changés, null pour un filtre vidé', () => {
		const before = { providerType: 'General Payment', counterpartyEmail: null, direction: 'debit' as const };
		expect(changedFilters(before, { ...before })).toEqual({});
		expect(
			changedFilters(before, { providerType: null, counterpartyEmail: 'a@b.ch', direction: 'debit' }),
		).toEqual({ providerType: null, counterpartyEmail: 'a@b.ch' });
	});

	it('changedFilters : le sens changé ou effacé part (revue P1, mutation survivante)', () => {
		const before = { providerType: null, counterpartyEmail: null, direction: 'debit' as const };
		expect(changedFilters(before, { ...before, direction: 'credit' })).toEqual({ direction: 'credit' });
		expect(changedFilters(before, { ...before, direction: null })).toEqual({ direction: null });
	});
});

describe('RuleFormModal — filtres (Story 38-11d, AC-2)', () => {
	beforeEach(() => vi.clearAllMocks());

	it('création : les trois filtres posés partent normalisés avec le critère', async () => {
		const r = render(RuleFormModal, { rule: null, accounts, onSuccess: () => {}, onCancel: () => {} });
		await fill(await r.findByTestId('rule-form-label'), 'Hébergement');
		await fill(await r.findByTestId('rule-form-match-value'), 'Hosting');
		await fireEvent.change(await r.findByTestId('rule-form-counterparty'), { target: { value: '10' } });
		await fill(await r.findByTestId('rule-form-provider-type'), ' General Payment ');
		await fill(await r.findByTestId('rule-form-counterparty-email'), 'Factures@Hosting.Example');
		await fireEvent.change(await r.findByTestId('rule-form-direction'), { target: { value: 'debit' } });
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		expect(createRule).toHaveBeenCalledWith(
			expect.objectContaining({
				matchValue: 'Hosting',
				providerType: 'General Payment',
				counterpartyEmail: 'factures@hosting.example',
				direction: 'debit',
			}),
		);
	});

	it('création sans filtre : les trois partent à null', async () => {
		const r = render(RuleFormModal, { rule: null, accounts, onSuccess: () => {}, onCancel: () => {} });
		await fill(await r.findByTestId('rule-form-label'), 'X');
		await fill(await r.findByTestId('rule-form-match-value'), 'Y');
		await fireEvent.change(await r.findByTestId('rule-form-counterparty'), { target: { value: '10' } });
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		expect(createRule).toHaveBeenCalledWith(
			expect.objectContaining({ providerType: null, counterpartyEmail: null, direction: null }),
		);
	});

	it('le critère reste obligatoire même avec des filtres', async () => {
		const r = render(RuleFormModal, { rule: null, accounts, onSuccess: () => {}, onCancel: () => {} });
		await fill(await r.findByTestId('rule-form-label'), 'X');
		await fill(await r.findByTestId('rule-form-counterparty-email'), 'a@b.ch');
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		expect(createRule).not.toHaveBeenCalled();
		expect((await r.findByTestId('rule-form-error')).textContent).toContain('Valeur requise');
	});

	it('e-mail mal formé : refus à l’écran, rien d’envoyé', async () => {
		const r = render(RuleFormModal, { rule: null, accounts, onSuccess: () => {}, onCancel: () => {} });
		await fill(await r.findByTestId('rule-form-label'), 'X');
		await fill(await r.findByTestId('rule-form-match-value'), 'Y');
		await fireEvent.change(await r.findByTestId('rule-form-counterparty'), { target: { value: '10' } });
		await fill(await r.findByTestId('rule-form-counterparty-email'), 'pas-une-adresse');
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		expect(createRule).not.toHaveBeenCalled();
		expect((await r.findByTestId('rule-form-error')).textContent).toContain('nom@domaine');
	});

	it('édition sans toucher aux filtres : aucun filtre dans le PATCH', async () => {
		const existing = rule({ providerType: 'General Payment', counterpartyEmail: 'a@b.ch', direction: 'debit' });
		const r = render(RuleFormModal, { rule: existing, accounts, onSuccess: () => {}, onCancel: () => {} });
		expect(((await r.findByTestId('rule-form-provider-type')) as HTMLInputElement).value).toBe('General Payment');
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		const body = (updateRule as ReturnType<typeof vi.fn>).mock.calls[0][1];
		expect(body).not.toHaveProperty('providerType');
		expect(body).not.toHaveProperty('counterpartyEmail');
		expect(body).not.toHaveProperty('direction');
	});

	it('édition : un filtre vidé part à null, un filtre posé part, l’autre reste absent', async () => {
		const existing = rule({ providerType: 'General Payment', direction: 'debit' });
		const r = render(RuleFormModal, { rule: existing, accounts, onSuccess: () => {}, onCancel: () => {} });
		await fill(await r.findByTestId('rule-form-provider-type'), '');
		await fill(await r.findByTestId('rule-form-counterparty-email'), 'x@y.ch');
		await fireEvent.click(await r.findByTestId('rule-form-submit'));
		const body = (updateRule as ReturnType<typeof vi.fn>).mock.calls[0][1];
		expect(body.providerType).toBeNull();
		expect(body.counterpartyEmail).toBe('x@y.ch');
		expect(body).not.toHaveProperty('direction');
	});
});

describe('RulesList — filtres affichés (Story 38-11d, AC-2)', () => {
	it('montre les filtres posés à côté du critère, rien pour une règle sans filtre', async () => {
		const r = render(RulesList, {
			rules: [
				rule({ id: 1, providerType: 'General Payment', counterpartyEmail: 'a@b.ch', direction: 'credit' }),
				rule({ id: 2 }),
			],
			accounts,
			onEdit: () => {},
			onRefresh: () => {},
		});
		expect((await r.findByTestId('rule-filter-1-0')).textContent).toBe('Type = General Payment');
		expect((await r.findByTestId('rule-filter-1-1')).textContent).toBe('E-mail = a@b.ch');
		expect((await r.findByTestId('rule-filter-1-2')).textContent).toBe('Entrées');
		expect(r.queryByTestId('rule-filter-2-0')).toBeNull();
	});
});
