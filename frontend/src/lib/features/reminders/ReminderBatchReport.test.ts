// Story 35-2b (#955, AC4) — la ligne d'un échec du lot de rappels affiche, pour un
// PDF refusé, la phrase du serveur qui dit quoi corriger. Branchement seul : la règle
// d'affichage se teste sur `reminderFailureText` (`reminder-error-label.test.ts`).
// Mutation M2 : ne plus lire `details.message` rougit le premier test.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import type { SendReminderBatchResponse } from './reminders.types';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import ReminderBatchReport from './ReminderBatchReport.svelte';

afterEach(() => cleanup());

/** Le texte d'un élément, blancs du gabarit réduits. */
function texte(el: HTMLElement): string {
	return (el.textContent ?? '').replace(/\s+/g, ' ').trim();
}

function rapport(details: Record<string, unknown> | null): SendReminderBatchResponse {
	return {
		accepted: [],
		failed: [{ invoiceId: 12, errorCode: 'INVOICE_NOT_PDF_READY', details }],
	};
}

describe('rapport d’un lot de rappels — refus du PDF (35-2b)', () => {
	it('la ligne porte le libellé puis la phrase du serveur', () => {
		const { getByTestId } = render(ReminderBatchReport, {
			report: rapport({ message: 'La localité du client contient « – » (U+2013).' }),
		});
		expect(texte(getByTestId('reminder-batch-failed-row'))).toBe(
			'#12 — Facture non imprimable en PDF : La localité du client contient « – » (U+2013).',
		);
	});

	it('sans message, la ligne garde le libellé seul', () => {
		const { getByTestId } = render(ReminderBatchReport, { report: rapport(null) });
		expect(texte(getByTestId('reminder-batch-failed-row'))).toBe(
			'#12 — Facture non imprimable en PDF',
		);
	});

	it('un message balisé s’affiche en texte, sans élément créé', () => {
		const { getByTestId } = render(ReminderBatchReport, {
			report: rapport({ message: '<b>gras</b>' }),
		});
		const ligne = getByTestId('reminder-batch-failed-row');
		expect(ligne.textContent).toContain('<b>gras</b>');
		expect(ligne.querySelector('b')).toBeNull();
	});
});
