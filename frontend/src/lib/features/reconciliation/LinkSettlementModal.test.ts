// Story 29-8 (#670, AC4) — tests Vitest de LinkSettlementModal.svelte : chargement,
// liste vide, échec du chargement, aucune présélection, « Rattacher » grisé puis
// actif, avertissement d'écart, succès, erreur, et rechargement sur
// `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` (C-29-8-24).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { LinkCandidate, ReconciliationProposal } from './reconciliation.types';

vi.mock('./reconciliation.api', () => ({
	getLinkCandidates: vi.fn(),
	linkTransaction: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import * as api from './reconciliation.api';
import LinkSettlementModal from './LinkSettlementModal.svelte';

const mockApi = vi.mocked(api);

function proposal(
	bankTransactionId = 42,
	bookingDate = '2026-06-18',
	valueDate: string | null = '2026-06-18',
): ReconciliationProposal {
	return {
		bankTransactionId,
		transaction: {
			bookingDate,
			valueDate,
			amount: '100.03',
			currency: 'CHF',
			counterpartyName: 'Dubarde SàRL',
		},
		candidates: [],
	};
}

function candidate(settlementId: number, gap: string): LinkCandidate {
	return {
		settlementId,
		invoiceId: 9,
		invoiceNumber: `F-2026-00${settlementId}`,
		contactName: 'Dubarde SàRL',
		settledOn: '2026-06-20',
		amount: '100.00',
		gap,
		daysApart: 2,
		journalEntryId: 200 + settlementId,
	};
}

function mount(onSuccess = vi.fn(), onOpenChange = vi.fn()) {
	return {
		onSuccess,
		onOpenChange,
		...render(LinkSettlementModal, {
			open: true,
			onOpenChange,
			proposal: proposal(),
			onSuccess,
		}),
	};
}

describe('LinkSettlementModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('charge les candidats de la transaction, sans présélection ; Rattacher grisé', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00'), candidate(2, '0.03')],
		});
		const { findAllByTestId, getByTestId } = mount();
		const radios = (await findAllByTestId('link-candidate-radio')) as HTMLInputElement[];
		expect(mockApi.getLinkCandidates).toHaveBeenCalledWith(42);
		expect(radios).toHaveLength(2);
		expect(radios.every((r) => !r.checked)).toBe(true);
		expect((getByTestId('link-submit') as HTMLButtonElement).disabled).toBe(true);
		// La phrase permanente.
		expect(getByTestId('link-no-entry').textContent).toContain('ne passe aucune écriture');
		// Le rappel de la transaction.
		expect(getByTestId('link-tx-summary').textContent).toContain('100.03');
	});

	it('choisir un candidat active Rattacher ; un écart nul n’affiche pas d’avertissement', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00')],
		});
		const { findAllByTestId, getByTestId, queryByTestId } = mount();
		const [radio] = await findAllByTestId('link-candidate-radio');
		await fireEvent.click(radio);
		expect((getByTestId('link-submit') as HTMLButtonElement).disabled).toBe(false);
		expect(queryByTestId('link-gap-warning')).toBeNull();
	});

	it('un candidat à écart non nul affiche l’avertissement avant confirmation', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00'), candidate(2, '0.03')],
		});
		const { findAllByTestId, findByTestId } = mount();
		const radios = await findAllByTestId('link-candidate-radio');
		await fireEvent.click(radios[1]);
		const warning = await findByTestId('link-gap-warning');
		expect(warning.textContent).toContain('dépasse ce règlement de 0.03 CHF');
		expect(mockApi.linkTransaction).not.toHaveBeenCalled();
	});

	it('liste vide : le message dit d’enregistrer le règlement depuis la fiche', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({ bankTransactionId: 42, candidates: [] });
		const { findByTestId, getByTestId } = mount();
		const empty = await findByTestId('link-empty');
		expect(empty.textContent).toContain("Enregistrez d'abord le règlement depuis la fiche");
		expect((getByTestId('link-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	it('échec du chargement : message lisible, Rattacher grisé', async () => {
		mockApi.getLinkCandidates.mockRejectedValue({
			code: 'BANK_ACCOUNT_CLOSED',
			message: 'Ce compte bancaire est clos.',
			status: 409,
		});
		const { findByTestId, getByTestId } = mount();
		expect((await findByTestId('link-load-error')).textContent).toContain(
			'Ce compte bancaire est clos.',
		);
		expect((getByTestId('link-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	it('succès : appelle linkTransaction, puis onSuccess', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00')],
		});
		mockApi.linkTransaction.mockResolvedValue({
			bankTransactionId: 42,
			journalEntryId: 201,
			invoiceId: 9,
			settlementId: 1,
		});
		const { findAllByTestId, getByTestId, onSuccess } = mount();
		await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
		await fireEvent.click(getByTestId('link-submit'));
		await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
		expect(mockApi.linkTransaction).toHaveBeenCalledWith(42, 1);
	});

	it('erreur : message lisible, la modal reste ouverte', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00')],
		});
		mockApi.linkTransaction.mockRejectedValue({
			code: 'RECONCILIATION_ACCOUNT_LOCKED',
			message: 'Le compte est verrouillé.',
			status: 409,
		});
		const { findAllByTestId, getByTestId, findByTestId, onSuccess, onOpenChange } = mount();
		await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
		await fireEvent.click(getByTestId('link-submit'));
		expect((await findByTestId('link-error')).textContent).toContain('Le compte est verrouillé.');
		expect(onSuccess).not.toHaveBeenCalled();
		expect(onOpenChange).not.toHaveBeenCalledWith(false);
	});

	it('409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE : la liste est rechargée et le choix remis à vide', async () => {
		mockApi.getLinkCandidates
			.mockResolvedValueOnce({
				bankTransactionId: 42,
				candidates: [candidate(1, '0.00'), candidate(2, '0.00')],
			})
			.mockResolvedValueOnce({ bankTransactionId: 42, candidates: [candidate(2, '0.00')] });
		mockApi.linkTransaction.mockRejectedValue({
			code: 'RECONCILIATION_SETTLEMENT_NOT_LINKABLE',
			message: 'Ce règlement ne peut pas être rattaché à cette transaction.',
			status: 409,
		});
		const { findAllByTestId, getByTestId, findByTestId } = mount();
		await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
		await fireEvent.click(getByTestId('link-submit'));
		await findByTestId('link-error');
		await waitFor(() => expect(mockApi.getLinkCandidates).toHaveBeenCalledTimes(2));
		await waitFor(async () => {
			const radios = (await findAllByTestId('link-candidate-radio')) as HTMLInputElement[];
			expect(radios).toHaveLength(1);
			expect(radios[0].checked).toBe(false);
		});
		expect((getByTestId('link-submit') as HTMLButtonElement).disabled).toBe(true);
	});

	// Revue de code P1, B3 (C-29-8-38) : les deux autres refus d'une course relisent aussi.
	it.each([
		['NOT_FOUND', 404, 'Ressource introuvable.'],
		['RECONCILIATION_TRANSACTION_NOT_PENDING', 404, 'Transaction déjà réconciliée.'],
	])('%s au rattachement : la liste est relue', async (code, status, message) => {
		mockApi.getLinkCandidates
			.mockResolvedValueOnce({ bankTransactionId: 42, candidates: [candidate(1, '0.00')] })
			.mockResolvedValueOnce({ bankTransactionId: 42, candidates: [] });
		mockApi.linkTransaction.mockRejectedValue({ code, message, status });
		const { findAllByTestId, getByTestId, findByTestId } = mount();
		await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
		await fireEvent.click(getByTestId('link-submit'));
		expect((await findByTestId('link-error')).textContent).toContain(message);
		await waitFor(() => expect(mockApi.getLinkCandidates).toHaveBeenCalledTimes(2));
		await findByTestId('link-empty');
	});

	it('un autre refus ne relit pas la liste', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({
			bankTransactionId: 42,
			candidates: [candidate(1, '0.00')],
		});
		mockApi.linkTransaction.mockRejectedValue({
			code: 'RECONCILIATION_ACCOUNT_LOCKED',
			message: 'Le compte est verrouillé.',
			status: 409,
		});
		const { findAllByTestId, getByTestId, findByTestId } = mount();
		await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
		await fireEvent.click(getByTestId('link-submit'));
		await findByTestId('link-error');
		expect(mockApi.getLinkCandidates).toHaveBeenCalledTimes(1);
	});

	// Revue de code P1, A7 (C-29-8-39) : la date rappelée est celle d'où se comptent les jours.
	it('le rappel montre la date de valeur, à défaut la date de comptabilisation', async () => {
		mockApi.getLinkCandidates.mockResolvedValue({ bankTransactionId: 42, candidates: [] });
		const { getByTestId, rerender, onOpenChange, onSuccess } = mount();
		const props = (p: ReconciliationProposal) => ({
			open: true,
			onOpenChange,
			proposal: p,
			onSuccess,
		});
		await rerender(props(proposal(42, '2026-06-18', '2026-06-16')));
		await waitFor(() =>
			expect(getByTestId('link-tx-summary').textContent).toContain('2026-06-16'),
		);
		expect(getByTestId('link-tx-summary').textContent).not.toContain('2026-06-18');
		await rerender(props(proposal(43, '2026-06-18', null)));
		await waitFor(() =>
			expect(getByTestId('link-tx-summary').textContent).toContain('2026-06-18'),
		);
	});

	describe('garde de session (revue de code P1, B1/B2/E3, C-29-8-37)', () => {
		function differe<T>() {
			let resolve!: (v: T) => void;
			let reject!: (e: unknown) => void;
			const promise = new Promise<T>((res, rej) => {
				resolve = res;
				reject = rej;
			});
			return { promise, resolve, reject };
		}
		type Candidats = Awaited<ReturnType<typeof api.getLinkCandidates>>;
		type Lien = Awaited<ReturnType<typeof api.linkTransaction>>;
		const reponse = (bankTransactionId: number, ids: number[]): Candidats => ({
			bankTransactionId,
			candidates: ids.map((i) => candidate(i, '0.00')),
		});
		const lien: Lien = { bankTransactionId: 42, journalEntryId: 201, invoiceId: 9, settlementId: 1 };
		const ids = (rows: HTMLElement[]) => rows.map((r) => r.dataset.settlementId);
		/** Laisse s'écouler les continuations des promesses résolues. */
		const vider = () => new Promise((r) => setTimeout(r, 0));

		it('un chargement périmé ne remplace pas la liste de la transaction courante', async () => {
			const a = differe<Candidats>();
			mockApi.getLinkCandidates.mockReturnValueOnce(a.promise).mockResolvedValueOnce(reponse(43, [7]));
			const { rerender, onOpenChange, onSuccess, findAllByTestId } = mount();
			await rerender({ open: true, onOpenChange, proposal: proposal(43), onSuccess });
			await waitFor(() => expect(mockApi.getLinkCandidates).toHaveBeenLastCalledWith(43));
			expect(ids(await findAllByTestId('link-candidate-row'))).toEqual(['7']);
			// La réponse de la transaction 42 arrive APRÈS celle de la 43.
			a.resolve(reponse(42, [1, 2]));
			await vider();
			expect(ids(await findAllByTestId('link-candidate-row'))).toEqual(['7']);
		});

		it('un refus arrivé après réouverture sur une autre transaction ne touche pas la modale', async () => {
			const envoi = differe<Lien>();
			mockApi.getLinkCandidates
				.mockResolvedValueOnce(reponse(42, [1]))
				.mockResolvedValueOnce(reponse(43, [7]))
				.mockResolvedValue(reponse(42, [1]));
			mockApi.linkTransaction.mockReturnValueOnce(envoi.promise);
			const { rerender, onOpenChange, onSuccess, findAllByTestId, getByTestId, queryByTestId } =
				mount();
			await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
			await fireEvent.click(getByTestId('link-submit'));
			expect(mockApi.linkTransaction).toHaveBeenCalledWith(42, 1);
			// Fermée par Échap pendant l'envoi, puis rouverte sur la transaction 43.
			await rerender({ open: false, onOpenChange, proposal: null, onSuccess });
			await rerender({ open: true, onOpenChange, proposal: proposal(43), onSuccess });
			expect(ids(await findAllByTestId('link-candidate-row'))).toEqual(['7']);
			// La nouvelle session n'hérite pas de l'envoi en cours.
			expect(
				((await findAllByTestId('link-candidate-radio'))[0] as HTMLInputElement).disabled,
			).toBe(false);
			envoi.reject({
				code: 'RECONCILIATION_SETTLEMENT_NOT_LINKABLE',
				message: 'Ce règlement ne peut pas être rattaché à cette transaction.',
				status: 409,
			});
			await vider();
			// Ni message, ni relecture de la liste de la 42.
			expect(queryByTestId('link-error')).toBeNull();
			expect(mockApi.getLinkCandidates).toHaveBeenCalledTimes(2);
			expect(ids(await findAllByTestId('link-candidate-row'))).toEqual(['7']);
		});

		it('un succès arrivé après réouverture ne ferme pas la modale courante', async () => {
			const envoi = differe<Lien>();
			mockApi.getLinkCandidates
				.mockResolvedValueOnce(reponse(42, [1]))
				.mockResolvedValueOnce(reponse(43, [7]));
			mockApi.linkTransaction.mockReturnValueOnce(envoi.promise);
			const { rerender, onOpenChange, onSuccess, findAllByTestId, getByTestId } = mount();
			await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
			await fireEvent.click(getByTestId('link-submit'));
			await rerender({ open: false, onOpenChange, proposal: null, onSuccess });
			await rerender({ open: true, onOpenChange, proposal: proposal(43), onSuccess });
			await findAllByTestId('link-candidate-row');
			envoi.resolve(lien);
			await vider();
			expect(onSuccess).not.toHaveBeenCalled();
			expect(onOpenChange).not.toHaveBeenCalled();
			expect(ids(await findAllByTestId('link-candidate-row'))).toEqual(['7']);
		});

		it('un succès arrivé après fermeture prévient le parent', async () => {
			const envoi = differe<Lien>();
			mockApi.getLinkCandidates.mockResolvedValueOnce(reponse(42, [1]));
			mockApi.linkTransaction.mockReturnValueOnce(envoi.promise);
			const { rerender, onOpenChange, onSuccess, findAllByTestId, getByTestId } = mount();
			await fireEvent.click((await findAllByTestId('link-candidate-radio'))[0]);
			await fireEvent.click(getByTestId('link-submit'));
			await rerender({ open: false, onOpenChange, proposal: null, onSuccess });
			envoi.resolve(lien);
			await waitFor(() => expect(onSuccess).toHaveBeenCalledTimes(1));
			expect(onOpenChange).not.toHaveBeenCalled();
		});
	});

	it('les replis des textes égalent le catalogue fr-CH', () => {
		const ftl = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8');
		const source = readFileSync(
			join('src/lib/features/reconciliation', 'LinkSettlementModal.svelte'),
			'utf-8',
		);
		const catalogue = new Map(
			ftl
				.split('\n')
				.filter((l) => l.startsWith('reconciliation-link-'))
				.map((l) => {
					const i = l.indexOf(' = ');
					return [l.slice(0, i), l.slice(i + 3)] as const;
				}),
		);
		const appels = [...source.matchAll(/i18nMsg\(\s*'(reconciliation-link-[a-z-]+)',\s*(['"])(.*?)\2/gs)];
		expect(appels.length).toBeGreaterThan(5);
		for (const [, cle, , repli] of appels) {
			expect(catalogue.get(cle), cle).toBe(repli);
		}
	});
});
