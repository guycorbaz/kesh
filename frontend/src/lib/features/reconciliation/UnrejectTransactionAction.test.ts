/**
 * Story 28-5b (#526) — V2 : la mention « Rejetée le … » et le bouton
 * « Annuler le rejet » d'une ligne du détail d'import.
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `CancelReconciliationDialog.test.ts` — mocks hoistés AVANT l'import du
 * composant.
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { BankTransactionDto } from '$lib/features/bank-import/bank-import.types';

/** Les replis passés à `i18nMsg`, par clé — comparés au catalogue fr-CH (revue P1, E-4). */
const replis = vi.hoisted(() => new Map<string, string>());

vi.mock('./reconciliation.api', () => ({
	unrejectTransaction: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (k: string, fallback: string, args?: Record<string, string | number>) => {
		replis.set(k, fallback);
		return args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, n) => String(args[n])) : fallback;
	},
	i18nLocale: () => 'fr-CH',
}));

/** La valeur fr-CH d'une clé sur une seule ligne du catalogue (patron d'`invoice-cancel.test.ts`). */
function valeurFr(cle: string): string | undefined {
	const texte = readFileSync(join('../crates/kesh-i18n/locales', 'fr-CH', 'messages.ftl'), 'utf-8');
	const prefixe = `${cle} = `;
	return texte
		.split('\n')
		.find((l) => l.startsWith(prefixe))
		?.slice(prefixe.length);
}
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: vi.fn(),
	notifyWarning: vi.fn(),
	notifyInfo: vi.fn(),
}));

import * as api from './reconciliation.api';
import * as notify from '$lib/shared/utils/notify';
import UnrejectTransactionAction from './UnrejectTransactionAction.svelte';

const mockApi = vi.mocked(api);
const mockNotify = vi.mocked(notify);

function rejected(partial: Partial<BankTransactionDto> = {}): BankTransactionDto {
	return {
		id: 42,
		bookingDate: '2026-03-02',
		valueDate: null,
		amount: '10.00',
		currency: 'CHF',
		reference: null,
		details: 'test',
		endToEndId: null,
		transactionId: null,
		counterpartyIban: null,
		counterpartyName: null,
		status: 'pending',
		matchedEntryId: null,
		// Revue P1 (E-1, A-2) : un autre jour qu'aujourd'hui, pour qu'une date du jour formatée à
		// la place de celle du rejet se voie. La date suit le fuseau du navigateur (C-28-5-15) ; la
		// suite ne fixe aucun fuseau et tourne dans celui du runner (poste de dev, CI). À 11:00 UTC,
		// le 4 mars reste le 4 mars de UTC-11 à UTC+12 : l'assertion tient pour tout runner de
		// cette plage, et rougirait sur un runner en UTC+13 ou UTC+14.
		autoMatchRejectedAt: '2026-03-04T11:00:00Z',
		...partial,
	};
}

function monter(canManage: boolean, tx = rejected(), onUnrejected = vi.fn()) {
	return { onUnrejected, ...render(UnrejectTransactionAction, { tx, canManage, onUnrejected }) };
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('UnrejectTransactionAction', () => {
	it('canManage=false ⇒ la mention, datée, et aucun bouton (mutation M10 : bouton sans condition)', () => {
		const { getByTestId, queryByTestId } = monter(false);
		const mention = getByTestId('detail-tx-rejected-on').textContent ?? '';
		// La date du REJET, formatée (dateStyle medium, fr-CH), sans l'heure : jour, mois et année
		// (revue P1, E-1 — mutation M23 : `new Date()` au lieu de `new Date(tx.autoMatchRejectedAt)`).
		expect(mention.trim()).toBe('Rejetée le 4 mars 2026');
		expect(queryByTestId('detail-tx-unreject')).toBeNull();
	});

	it('les replis des trois clés du composant égalent mot pour mot le catalogue fr-CH (revue P1, E-4)', () => {
		replis.clear();
		monter(true);
		for (const cle of [
			'reconciliation-unreject-rejected-on',
			'reconciliation-unreject-button',
		]) {
			const attendu = valeurFr(cle);
			expect(attendu, cle).toBeDefined();
			expect(replis.get(cle), cle).toBe(attendu);
		}
		// Le repli du succès n'est lu qu'au clic : il est contrôlé dans le test du succès.
	});

	it('canManage=true ⇒ la mention et le bouton', () => {
		const { getByTestId } = monter(true);
		expect(getByTestId('detail-tx-rejected-on')).not.toBeNull();
		expect(getByTestId('detail-tx-unreject').textContent).toContain('Annuler le rejet');
	});

	it('transaction non rejetée ⇒ rien n’est rendu', () => {
		const { queryByTestId } = monter(true, rejected({ autoMatchRejectedAt: null }));
		expect(queryByTestId('detail-tx-rejected-on')).toBeNull();
		expect(queryByTestId('detail-tx-unreject')).toBeNull();
	});

	it('clic ⇒ UN appel, bouton désactivé pendant ; succès ⇒ notifySuccess puis onUnrejected', async () => {
		let resolve!: () => void;
		mockApi.unrejectTransaction.mockReturnValue(
			new Promise<void>((r) => {
				resolve = r;
			}),
		);
		const { getByTestId, onUnrejected } = monter(true);
		const bouton = getByTestId('detail-tx-unreject') as HTMLButtonElement;
		await fireEvent.click(bouton);
		await waitFor(() => expect(bouton.disabled).toBe(true));
		// Un second clic pendant l'appel n'envoie rien.
		await fireEvent.click(bouton);
		expect(mockApi.unrejectTransaction).toHaveBeenCalledTimes(1);
		// Revue P1 (E-2, A-1, B-L3) — la garde `if (submitting) return` seule : jsdom ne délivre
		// pas un clic sur un bouton désactivé ; on lève `disabled` à la main (un client qui ne le
		// respecterait pas), et le gestionnaire doit encore refuser (mutation M21 : garde retirée).
		bouton.disabled = false;
		await fireEvent.click(bouton);
		expect(mockApi.unrejectTransaction).toHaveBeenCalledTimes(1);
		expect(mockApi.unrejectTransaction).toHaveBeenCalledWith(42);
		expect(onUnrejected).not.toHaveBeenCalled();
		resolve();
		await waitFor(() => expect(onUnrejected).toHaveBeenCalledTimes(1));
		expect(mockNotify.notifySuccess).toHaveBeenCalledTimes(1);
		expect(mockNotify.notifySuccess.mock.calls[0][0]).toContain('Rejet annulé');
		// Revue P1 (E-4) : le repli du succès égale le catalogue fr-CH.
		const attendu = valeurFr('reconciliation-unreject-success');
		expect(attendu).toBeDefined();
		expect(mockNotify.notifySuccess.mock.calls[0][0]).toBe(attendu);
		expect(mockNotify.notifyError).not.toHaveBeenCalled();
	});

	it('échec ⇒ notifyError(message du serveur) ET onUnrejected (mutation M11 : pas de relecture sur échec)', async () => {
		mockApi.unrejectTransaction.mockRejectedValue({
			code: 'RECONCILIATION_TRANSACTION_NOT_REJECTED',
			message: 'Message du serveur, déjà traduit.',
			status: 409,
		});
		const { getByTestId, onUnrejected } = monter(true);
		const bouton = getByTestId('detail-tx-unreject') as HTMLButtonElement;
		await fireEvent.click(bouton);
		await waitFor(() => expect(onUnrejected).toHaveBeenCalledTimes(1));
		expect(mockNotify.notifyError).toHaveBeenCalledWith('Message du serveur, déjà traduit.');
		expect(mockNotify.notifySuccess).not.toHaveBeenCalled();
		// Revue P1 (B-L3) — après l'échec, le bouton redevient actif : on peut réessayer
		// (mutation M22 : `finally { submitting = false }` retiré).
		await waitFor(() => expect(bouton.disabled).toBe(false));
	});
});
