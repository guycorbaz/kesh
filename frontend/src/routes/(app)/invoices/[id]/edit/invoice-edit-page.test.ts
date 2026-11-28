/**
 * Story 64-1 (#1758, D3) — la page `/invoices/[id]/edit` refuse d'entrée ce que
 * l'utilisateur ne peut pas enregistrer : une facture qui n'est plus un brouillon
 * (AC4), un rôle sans droit d'écriture (AC5-bis) ; un brouillon, en Administrateur ou
 * en Comptable, garde son formulaire (AC5).
 *
 * Le nom n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Sonde positive à chaque absence.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('$app/state', () => ({
	page: { params: { id: '7' }, url: new URL('http://localhost/invoices/7/edit') },
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));
const roleState = { role: 'Comptable' };
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		get currentUser() {
			return { role: roleState.role, username: 'u', userId: '1' };
		},
	},
}));
const getMock = vi.fn();
vi.mock('$lib/features/invoices/invoices.api', () => ({
	getInvoice: (id: number) => getMock(id),
}));
vi.mock('$lib/components/invoices/InvoiceForm.svelte', async () => ({
	default: (await import('./invoice-form.stub.svelte')).default,
}));

import Page from './+page.svelte';

function invoice(status: string) {
	return { id: 7, status, version: 1 };
}

beforeEach(() => {
	roleState.role = 'Comptable';
});
afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('page de modification d’une facture client (64-1)', () => {
	it.each(['validated', 'cancelled'])(
		'AC4 — facture %s : avis et lien vers la facture, jamais le formulaire (mutation : supprimer la garde de statut)',
		async (status) => {
			getMock.mockResolvedValue(invoice(status));
			const r = render(Page);
			const notice = await r.findByTestId('invoice-edit-not-draft');
			expect(notice.textContent).toMatch(/ne se modifie plus/);
			expect(r.getByTestId('invoice-edit-back').getAttribute('href')).toBe('/invoices/7');
			expect(getMock).toHaveBeenCalledWith(7);
			expect(r.queryByTestId('invoice-form-stub')).toBeNull();
		},
	);

	it.each(['Comptable', 'Admin'])(
		'AC5 — brouillon en %s : le formulaire, pas d’avis (non-régression)',
		async (role) => {
			roleState.role = role;
			getMock.mockResolvedValue(invoice('draft'));
			const r = render(Page);
			await r.findByTestId('invoice-form-stub');
			expect(r.queryByTestId('invoice-edit-not-draft')).toBeNull();
			expect(r.queryByTestId('invoice-edit-forbidden')).toBeNull();
		},
	);

	it('AC5 — une erreur de chargement garde son message', async () => {
		getMock.mockRejectedValue({ code: 'NOT_FOUND', status: 404, message: 'Facture introuvable' });
		const r = render(Page);
		expect((await r.findByText('Facture introuvable')).textContent).toBe('Facture introuvable');
		expect(r.queryByTestId('invoice-form-stub')).toBeNull();
	});

	it('AC5-bis — Consultation : avis de rôle et lien, getInvoice non appelé, jamais le formulaire, même sur un brouillon (mutation : supprimer la garde de rôle)', async () => {
		roleState.role = 'Consultation';
		getMock.mockResolvedValue(invoice('draft'));
		const r = render(Page);
		const notice = await r.findByTestId('invoice-edit-forbidden');
		expect(notice.textContent).toMatch(/ne permet pas de modifier une facture/);
		expect(r.getByTestId('invoice-edit-back').getAttribute('href')).toBe('/invoices/7');
		await waitFor(() => expect(r.queryByTestId('invoice-edit-not-draft')).toBeNull());
		expect(getMock).not.toHaveBeenCalled();
		expect(r.queryByTestId('invoice-form-stub')).toBeNull();
	});
});
