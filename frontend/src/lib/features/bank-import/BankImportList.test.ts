// Story v016-2b (#816, AC-23) — la liste des imports : un relevé saisi à la main
// s'affiche « Saisie manuelle » (et son numéro) au lieu de son `filename` ; un
// fichier garde son nom.
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import BankImportList from './BankImportList.svelte';
import { pinTimeZone } from '$lib/shared/utils/pin-time-zone.test-helper';
import type { BankImportResponse } from './bank-import.types';

function imp(o: Partial<BankImportResponse>): BankImportResponse {
	return {
		id: 1,
		bankAccountId: 3,
		filename: 'releve.csv',
		fileHash: 'h'.repeat(64),
		sourceFormat: 'CSV',
		statementId: null,
		periodFrom: '2026-09-01',
		periodTo: '2026-09-30',
		openingBalance: null,
		closingBalance: null,
		transactionCount: 2,
		importedAt: '2026-10-01T00:00:00',
		...o,
	};
}

afterEach(() => cleanup());

describe('BankImportList — origine du relevé', () => {
	it('fichier : son nom ; saisie : « Saisie manuelle », numéro compris s’il en a un', () => {
		const { getAllByTestId, container } = render(BankImportList, {
			props: {
				imports: [
					imp({ id: 1 }),
					imp({ id: 2, sourceFormat: 'MANUAL', filename: 'saisie-manuelle-2026-09-01-2026-09-30' }),
					imp({ id: 3, sourceFormat: 'MANUAL', filename: 'saisie-manuelle-x', statementId: '9/2026' }),
				],
			},
		});
		expect(getAllByTestId('bank-import-list-link').map((a) => a.textContent)).toEqual([
			'releve.csv',
			'Saisie manuelle',
			'Saisie manuelle n° 9/2026',
		]);
		expect(container.textContent).not.toContain('saisie-manuelle-');
		const entetes = Array.from(container.querySelectorAll('th')).map((t) => t.textContent);
		expect(entetes.slice(0, 2)).toEqual(['Enregistré le', 'Relevé']);
	});
});

// v0172-correctifs (#880, AC-C2) — dates en forme suisse ; l'horodatage d'import (UTC)
// à l'heure de Zurich. Mutation MC2 : un helper retiré d'un site rougit ce test.
describe('BankImportList — dates en forme suisse (v0172, #880)', () => {
	pinTimeZone('UTC');

	it('« Enregistré le » 01.10.2026 02:00 (UTC 00:00), période 01.09.2026 → 30.09.2026, jamais l’ISO', () => {
		const { getAllByTestId, container } = render(BankImportList, { props: { imports: [imp({})] } });
		const cells = Array.from(getAllByTestId('bank-import-list-row')[0].querySelectorAll('td')).map(
			(td) => td.textContent?.trim(),
		);
		expect(cells[0]).toBe('01.10.2026 02:00');
		expect(cells[2]).toBe('01.09.2026 → 30.09.2026');
		expect(container.textContent).not.toMatch(/2026-\d{2}-\d{2}/);
	});
});
