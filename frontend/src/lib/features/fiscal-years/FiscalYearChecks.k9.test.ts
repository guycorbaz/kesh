// Story 37-5 (AC-9) — l'avis du contrôle K9 dans « Préparer la clôture » : en
// attention, l'aperçu de la réévaluation est montré avant tout clic (Comptable,
// Administrateur) ; Consultation n'a que la liste ; K9 `ok` : ni avis, ni aperçu,
// ni requête d'aperçu, ni bouton — la ligne reste, « Rien à signaler ».

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args
			? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ''))
			: fallback,
	i18nLocale: () => 'fr-CH'
}));

const previewMock = vi.fn();
const postMock = vi.fn();
vi.mock('./fiscal-years.api', () => ({
	previewRevaluation: (...a: unknown[]) => previewMock(...a),
	postRevaluation: (...a: unknown[]) => postMock(...a)
}));
vi.mock('$lib/features/exchange-rates/exchange-rates.api', () => ({
	fetchRates: vi.fn()
}));

import FiscalYearChecks from './FiscalYearChecks.svelte';
import { closingChecks } from './closing-checks.test-fixtures';
import { preview } from './revaluation.test-fixtures';

function k9Attention() {
	return closingChecks({
		FOREIGN_ACCOUNTS_NOT_REVALUED: {
			status: 'attention',
			count: 2,
			details: {
				accounts: [
					{ accountId: 10, number: '1020', name: 'PayPal USD', currency: 'USD', delta: '31.93', reason: 'NOT_REVALUED' },
					{ accountId: 12, number: '1040', name: 'Banque GBP', currency: 'GBP', delta: null, reason: 'RATE_UNAVAILABLE' }
				]
			}
		}
	});
}

beforeEach(() => {
	previewMock.mockReset();
	postMock.mockReset();
});

describe('FiscalYearChecks — K9', () => {
	it('en attention : la liste, et l’aperçu montré avant tout clic (Comptable)', async () => {
		previewMock.mockResolvedValue(preview());
		render(FiscalYearChecks, { loadState: 'ready', checks: k9Attention(), canRevalue: true });
		const statut = screen.getByTestId('fiscal-year-checks-status-FOREIGN_ACCOUNTS_NOT_REVALUED');
		expect(statut.textContent).toMatch(/2 compte\(s\) en devise non réévalué\(s\)/);
		expect(statut.textContent).toMatch(/1020 PayPal USD : écart de/);
		expect(statut.textContent).toMatch(/pas de cours de clôture GBP/);
		expect(await screen.findByTestId('revaluation-panel')).toBeTruthy();
		expect(previewMock).toHaveBeenCalledWith(1, {});
		expect(screen.getByTestId('revaluation-submit').textContent).toMatch(/Réévaluer/);
	});

	it('« Réévaluer » passe les écritures montrées, puis demande la relecture des contrôles', async () => {
		previewMock.mockResolvedValue(preview());
		postMock.mockResolvedValue({ entryIds: [7] });
		const onChecksChanged = vi.fn();
		render(FiscalYearChecks, {
			loadState: 'ready',
			checks: k9Attention(),
			canRevalue: true,
			onChecksChanged
		});
		await fireEvent.click(await screen.findByTestId('revaluation-submit'));
		await waitFor(() => expect(onChecksChanged).toHaveBeenCalled());
		expect(postMock).toHaveBeenCalledWith(1, { accountIds: [10], rates: { USD: '0.9119300000' } });
	});

	it('Consultation : la liste, sans aperçu ni bouton', () => {
		render(FiscalYearChecks, { loadState: 'ready', checks: k9Attention(), canRevalue: false });
		expect(screen.getByTestId('fiscal-year-checks-k9-accounts')).toBeTruthy();
		expect(screen.queryByTestId('fiscal-year-checks-k9-preview')).toBeNull();
		expect(previewMock).not.toHaveBeenCalled();
	});

	it('K9 ok : ni avis, ni aperçu, ni requête ; la ligne dit « Rien à signaler »', () => {
		render(FiscalYearChecks, { loadState: 'ready', checks: closingChecks(), canRevalue: true });
		expect(
			screen.getByTestId('fiscal-year-checks-status-FOREIGN_ACCOUNTS_NOT_REVALUED').textContent
		).toMatch(/Rien à signaler/);
		expect(screen.queryByTestId('fiscal-year-checks-k9-preview')).toBeNull();
		expect(screen.queryByTestId('revaluation-submit')).toBeNull();
		expect(previewMock).not.toHaveBeenCalled();
	});
});
