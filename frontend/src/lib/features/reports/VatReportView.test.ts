// Story 25-4-d2c (#384) — tests Vitest pour VatReportView.svelte, qui n'en avait
// aucun : la section des diminutions de contre-prestation (soldes), la TVA due
// nette, le rapport vide et le bandeau d'écart.
//
// ⚠️ Chaque test nomme la MUTATION qu'il attrape.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import VatReportView from './VatReportView.svelte';
import { isReportEmpty } from './reports.api';
import type { VatReportDto } from './reports.types';

function dto(overrides: Partial<VatReportDto> = {}): VatReportDto {
	return {
		period: { fiscalYearId: 1, startDate: '2026-01-01', endDate: '2026-12-31' },
		rows: [{ rate: '8.10', category: null, baseHt: '1000.00', vatDue: '81.00' }],
		totalBaseHt: '1000.00',
		totalVatDue: '81.00',
		writeOffRows: [],
		totalVatWriteOff: '0.00',
		totalVatDueNet: '81.00',
		totalVatRecoverable: '0.00',
		vatBalance: '81.00',
		reconciliationDelta: '0.00',
		reconciliationStatus: 'ok',
		...overrides,
	};
}

const avecSoldes = (overrides: Partial<VatReportDto> = {}) =>
	dto({
		writeOffRows: [{ rate: '8.10', baseHt: '20.00', vat: '1.62' }],
		totalVatWriteOff: '1.62',
		totalVatDueNet: '79.38',
		vatBalance: '79.38',
		...overrides,
	});

afterEach(() => cleanup());

describe('VatReportView — les soldes', () => {
	it('sans soldes, ni section ni TVA nette : le rapport est inchangé (mutation : condition retirée)', () => {
		const r = render(VatReportView, { props: { dto: dto() } });
		expect(r.queryByTestId('vat-write-off-section')).toBeNull();
		expect(r.queryByTestId('vat-total-vat-due-net')).toBeNull();
		expect(r.getByTestId('vat-balance').textContent).toContain('81.00');
	});

	it('avec soldes, la section par taux et la TVA due nette ; le solde est sur le net (mutation : section absente)', () => {
		const r = render(VatReportView, { props: { dto: avecSoldes() } });
		expect(r.getByTestId('vat-write-off-section')).toBeTruthy();
		const ligne = r.getByTestId('vat-write-off-row');
		expect(ligne.textContent).toContain('8.10 %');
		expect(ligne.textContent).toContain('1.62');
		expect(r.getByTestId('vat-total-vat-due-net').textContent).toContain('79.38');
		expect(r.getByTestId('vat-total-vat-due').textContent).toContain('81.00');
	});

	it('sans vente ni récupérable mais avec un solde : pas « vide » (mutation : garde inchangé)', () => {
		const sansVente = avecSoldes({
			rows: [],
			totalBaseHt: '0.00',
			totalVatDue: '0.00',
			totalVatDueNet: '-1.62',
			vatBalance: '-1.62',
		});
		expect(isReportEmpty('vat', sansVente)).toBe(false);
		const r = render(VatReportView, { props: { dto: sansVente } });
		expect(r.queryByRole('status')).toBeNull();
		expect(r.getByTestId('vat-write-off-section')).toBeTruthy();
	});

	it('rien du tout : le rapport est vide', () => {
		const vide = dto({ rows: [], totalBaseHt: '0.00', totalVatDue: '0.00', totalVatDueNet: '0.00', vatBalance: '0.00' });
		expect(isReportEmpty('vat', vide)).toBe(true);
		const r = render(VatReportView, { props: { dto: vide } });
		expect(r.getByRole('status').textContent).toContain('Aucune écriture');
	});

	it('le bandeau d’écart reste affiché sur un écart (non-régression)', () => {
		const r = render(VatReportView, {
			props: { dto: avecSoldes({ reconciliationStatus: 'delta', reconciliationDelta: '0.50' }) },
		});
		expect(r.getByRole('alert').textContent).toContain('0.50');
	});
});

// Story 55-2 (AC-6, D7) — la ligne « Comptes au grand livre : » des comptes désignés, après le contenu
// du décompte (tableau ou message d'un décompte vide). Rouge sans le correctif (aucun `vat-accounts`).
describe('VatReportView — Story 55-2, comptes au grand livre', () => {
	const DEUX = { payable: { id: 20, number: '2200' }, recoverable: { id: 11, number: '1170' } };

	it('deux comptes : étiquette « TVA due » puis son lien, « TVA récupérable » puis le sien, période du décompte', async () => {
		const { LINK_CLASS } = await import('$lib/shared/navigation/link-class');
		const { getByTestId } = render(VatReportView, { dto: dto(), vatAccounts: DEUX });
		const ligne = getByTestId('vat-accounts');
		expect(ligne.textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Comptes au grand livre : TVA due 2200 · TVA récupérable 1170',
		);
		const liens = [...ligne.querySelectorAll('a')];
		expect(liens.map((a) => a.getAttribute('href'))).toEqual([
			'/reports?tab=general-ledger&accountId=20&from=2026-01-01&to=2026-12-31',
			'/reports?tab=general-ledger&accountId=11&from=2026-01-01&to=2026-12-31',
		]);
		expect(liens[0].getAttribute('class')).toBe(LINK_CLASS);
	});

	it('la ligne vient APRÈS le tableau du décompte', () => {
		const { getByTestId, container } = render(VatReportView, { dto: dto(), vatAccounts: DEUX });
		const table = container.querySelector('table')!;
		expect(table.compareDocumentPosition(getByTestId('vat-accounts')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
	});

	it('un seul compte désigné : sa paire seule', () => {
		const { getByTestId } = render(VatReportView, {
			dto: dto(),
			vatAccounts: { payable: null, recoverable: { id: 11, number: '1170' } },
		});
		expect(getByTestId('vat-accounts').textContent?.replace(/\s+/g, ' ').trim()).toBe(
			'Comptes au grand livre : TVA récupérable 1170',
		);
	});

	it('aucun compte désigné (ou réglages en échec) : la ligne est omise', () => {
		const { queryByTestId } = render(VatReportView, { dto: dto(), vatAccounts: { payable: null, recoverable: null } });
		expect(queryByTestId('vat-accounts')).toBeNull();
		const sansProp = render(VatReportView, { dto: dto() });
		expect(sansProp.queryByTestId('vat-accounts')).toBeNull();
	});

	it('décompte vide : la ligne s’affiche aussi, après le message « aucune écriture »', () => {
		const vide = dto({
			rows: [],
			totalBaseHt: '0.00',
			totalVatDue: '0.00',
			totalVatDueNet: '0.00',
			vatBalance: '0.00',
		});
		expect(isReportEmpty('vat', vide)).toBe(true);
		const { getByTestId, getByRole } = render(VatReportView, { dto: vide, vatAccounts: DEUX });
		const message = getByRole('status');
		expect(message.compareDocumentPosition(getByTestId('vat-accounts')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
	});
});
