// Story 15-1c-i (#518) — panneau des propositions (AC5), panneau d'un groupe
// (AC6), lien de code (AC11) ; rôles (AC10).
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup, fireEvent } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

import ProposalsPanel from './ProposalsPanel.svelte';
import LetteringGroupPanel from './LetteringGroupPanel.svelte';
import LetteringCodeLink from './LetteringCodeLink.svelte';
import { group, groupLine, proposal, proposalLine, proposals } from './open-items.test.fixtures';
import type { GroupState, ProposalsState } from './open-items.types';

afterEach(() => cleanup());

function renderProposals(state: ProposalsState, canWrite = true) {
	const onAccept = vi.fn();
	const r = render(ProposalsPanel, { proposals: state, canWrite, busy: false, onAccept });
	return { ...r, onAccept };
}

describe('AC5 — le panneau des propositions (test 8)', () => {
	it('repère « contre-passation » et lignes entières, lues dans la paire', () => {
		const p = proposal({
			reversalPair: true,
			debit: proposalLine({ lineId: 41, entryId: 50, description: 'Achat' }),
			credit: proposalLine({ lineId: 42, entryId: 51, entryNumber: 14, description: 'Contre-passation n° 12' }),
		});
		const { getByTestId } = renderProposals({ status: 'ready', data: proposals([p]) });
		expect(getByTestId('proposal-reversal-41-42').textContent).toContain('contre-passation');
		const credit = getByTestId('proposal-line-42');
		expect(credit.textContent).toContain('Exercice 2026 n° 14');
		expect(credit.textContent).toContain('Contre-passation n° 12');
		expect(credit.querySelector('a[href="/journal-entries/51"]')).not.toBeNull();
	});

	it('pas de repère sans reversalPair', () => {
		const { queryByTestId } = renderProposals({ status: 'ready', data: proposals([proposal()]) });
		expect(queryByTestId('proposal-reversal-1-2')).toBeNull();
	});

	it('« Lettrer » remonte la paire (les deux lignes) — un clic, un geste', async () => {
		const p = proposal();
		const { getByTestId, onAccept } = renderProposals({ status: 'ready', data: proposals([p]) });
		expect(onAccept).not.toHaveBeenCalled();
		await fireEvent.click(getByTestId('proposal-letter-1-2'));
		expect(onAccept).toHaveBeenCalledTimes(1);
		expect(onAccept).toHaveBeenCalledWith(p);
	});

	it('total > paires affichées : « N autres » ; égal : rien', () => {
		const more = renderProposals({ status: 'ready', data: proposals([proposal()], { total: 4 }) });
		expect(more.getByTestId('open-items-proposals-more').textContent).toContain(': 3 —');
		more.unmount();
		const exact = renderProposals({ status: 'ready', data: proposals([proposal()]) });
		expect(exact.queryByTestId('open-items-proposals-more')).toBeNull();
	});

	it('l’échec s’affiche dans le panneau, par le message reçu', () => {
		const { getByTestId } = renderProposals({ status: 'error', message: 'Trop de lignes ouvertes' });
		expect(getByTestId('open-items-proposals-error').textContent).toContain('Trop de lignes ouvertes');
	});

	it('Consultation : aucun « Lettrer » (test 11)', () => {
		const { container } = renderProposals({ status: 'ready', data: proposals([proposal()]) }, false);
		expect(container.querySelector('[data-testid^="proposal-letter-"]')).toBeNull();
	});
});

function renderGroup(state: GroupState, canWrite = true) {
	const onDissolve = vi.fn();
	const onClose = vi.fn();
	const r = render(LetteringGroupPanel, { group: state, canWrite, busy: false, onDissolve, onClose });
	return { ...r, onDissolve, onClose };
}

describe('AC6 — le panneau d’un groupe (test 9)', () => {
	it('groupe manuel : origine, compte, lignes, et « Délettrer »', async () => {
		const { getByTestId, onDissolve } = renderGroup({ status: 'ready', data: group() });
		expect(getByTestId('lettering-group-origin').textContent).toContain('lettrage manuel');
		expect(getByTestId('lettering-group-account').textContent).toContain('1091');
		expect(getByTestId('lettering-group-account').textContent).toContain('Compte de passage');
		expect(getByTestId('lettering-group-line-2').textContent).toContain('Exercice 2026 n° 13');
		await fireEvent.click(getByTestId('lettering-group-dissolve'));
		expect(onDissolve).toHaveBeenCalledWith('AA');
	});

	it('groupe `document` facture + avoir : « lettrage de la pièce » + numéro de la FACTURE, motif et lien, pas de bouton', () => {
		const g = group({
			origin: 'document',
			manualDissolutionBlockedBy: 'LETTERING_IS_DOCUMENT',
			lines: [
				groupLine({
					id: 1,
					document: { type: 'creditNote', id: 3, number: 'AV-1', invoiceId: null, invoiceNumber: null },
				}),
				groupLine({
					id: 2,
					document: { type: 'invoice', id: 4, number: 'F-9', invoiceId: null, invoiceNumber: null },
				}),
			],
		});
		const { getByTestId, queryByTestId } = renderGroup({ status: 'ready', data: g });
		expect(getByTestId('lettering-group-origin').textContent?.trim()).toBe('lettrage de la pièce F-9');
		expect(queryByTestId('lettering-group-dissolve')).toBeNull();
		expect(getByTestId('lettering-group-blocked').textContent).toContain(
			'il suit la pièce et ses règlements',
		);
		expect(getByTestId('lettering-group-document-link').getAttribute('href')).toBe('/invoices/4');
	});

	it('les deux autres motifs, chacun par son texte', () => {
		const owned = renderGroup({
			status: 'ready',
			data: group({ origin: 'reversal', manualDissolutionBlockedBy: 'LETTERING_LINE_OWNED_BY_DOCUMENT' }),
		});
		expect(owned.getByTestId('lettering-group-origin').textContent).toContain('contre-passation');
		expect(owned.getByTestId('lettering-group-blocked').textContent).toContain(
			'Une de ces lignes appartient à une pièce',
		);
		expect(owned.queryByTestId('lettering-group-document-link')).toBeNull();
		owned.unmount();
		const closed = renderGroup({
			status: 'ready',
			data: group({ manualDissolutionBlockedBy: 'LETTERING_ALL_LINES_IN_CLOSED_PERIODS' }),
		});
		expect(closed.getByTestId('lettering-group-blocked').textContent).toContain('période close');
		expect(closed.queryByTestId('lettering-group-dissolve')).toBeNull();
	});

	it('code inconnu : « aucun groupe ne porte ce code »', () => {
		const { getByTestId } = renderGroup({ status: 'notFound', code: 'ZZ' });
		expect(getByTestId('lettering-group-not-found').textContent).toContain(
			'Aucun groupe ne porte ce code.',
		);
	});

	it('Consultation : ni « Délettrer » ni motif (test 11)', () => {
		const a = renderGroup({ status: 'ready', data: group() }, false);
		expect(a.queryByTestId('lettering-group-dissolve')).toBeNull();
		a.unmount();
		const b = renderGroup(
			{ status: 'ready', data: group({ manualDissolutionBlockedBy: 'LETTERING_IS_DOCUMENT' }) },
			false,
		);
		expect(b.queryByTestId('lettering-group-blocked')).toBeNull();
	});
});

describe('AC11 — le lien de code (test 12)', () => {
	it('mène au groupe, avec compte et date s’ils sont donnés', () => {
		const { getByTestId } = render(LetteringCodeLink, { code: 'AB', accountId: 7, asOf: '2026-03-31' });
		expect(getByTestId('lettering-code-link-AB').getAttribute('href')).toBe(
			'/open-items?accountId=7&asOf=2026-03-31&group=AB',
		);
		expect(getByTestId('lettering-code-link-AB').textContent).toBe('AB');
	});

	it('ne lit aucune clé i18n — le code est son propre texte', () => {
		const src = readFileSync('src/lib/features/open-items/LetteringCodeLink.svelte', 'utf-8');
		expect(src).not.toMatch(/i18nMsg|i18n\.svelte/);
	});
});
