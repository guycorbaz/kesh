/**
 * Le dialogue partagé « Changer le compte analytique » — Story 36-16a (#1167, T11).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import ProjectReassignDialog from './ProjectReassignDialog.svelte';
import type { ProjectResponse } from './projects.types';

function projet(id: number, code: string, archived = false): ProjectResponse {
	return {
		id,
		parentId: null,
		code,
		name: `Compte ${code}`,
		description: null,
		archived,
		startDate: null,
		endDate: null,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
	};
}

const projects = [projet(11, 'P'), projet(12, 'Q'), projet(13, 'ARCH', true)];

const rows = [
	{ id: 7, label: 'Ligne 1', currentProjectId: 11 },
	{ id: 8, label: 'Ligne 2', currentProjectId: 13 },
];

function open(submit = vi.fn(async () => {}), onDone = vi.fn()) {
	const r = render(ProjectReassignDialog, { props: { rows, projects, submit, onDone } });
	return { r, submit, onDone };
}

afterEach(() => cleanup());

describe('ProjectReassignDialog', () => {
	it('Enregistrer est inactif tant qu’aucune rangée n’a changé (mutation : bouton toujours actif)', async () => {
		const { r, submit } = open();
		const save = r.getByTestId('project-reassign-save') as HTMLButtonElement;
		expect(save.disabled).toBe(true);
		await fireEvent.click(save);
		expect(submit).not.toHaveBeenCalled();
	});

	it('deux rangées : `submit` reçoit chacune avec son `id`, sans accusé (mutation : `id` perdu)', async () => {
		const { r, submit, onDone } = open();
		await fireEvent.change(r.getByTestId('project-reassign-select-7'), {
			target: { value: '12' },
		});
		await fireEvent.change(r.getByTestId('project-reassign-select-8'), {
			target: { value: '' },
		});
		await fireEvent.click(r.getByTestId('project-reassign-save'));
		await waitFor(() => expect(onDone).toHaveBeenCalledTimes(1));
		expect(submit).toHaveBeenCalledWith(
			[
				{ id: 7, projectId: 12 },
				{ id: 8, projectId: null },
			],
			false,
		);
	});

	it('le compte archivé n’est proposé qu’à la rangée qui le porte (mutation : archivés partout)', () => {
		const { r } = open();
		const values = (testId: string) =>
			Array.from((r.getByTestId(testId) as HTMLSelectElement).options).map((o) => o.value);
		expect(values('project-reassign-select-7')).toEqual(['', '11', '12']);
		expect(values('project-reassign-select-8')).toEqual(['', '11', '12', '13']);
	});

	it('409 : avertissement nommant l’exercice et la date du verrou, puis Confirmer renvoie avec l’accusé (mutation : accusé non renvoyé)', async () => {
		const submit = vi
			.fn()
			.mockRejectedValueOnce({
				code: 'PROJECT_REASSIGNMENT_CLOSED_PERIOD',
				status: 409,
				message: 'Confirmez.',
				details: { entryCount: 3, fiscalYears: ['Exercice 2025'], lockedThrough: '2026-03-31' },
			})
			.mockResolvedValueOnce(undefined);
		const onDone = vi.fn();
		const { r } = open(submit, onDone);
		await fireEvent.change(r.getByTestId('project-reassign-select-7'), {
			target: { value: '12' },
		});
		await fireEvent.click(r.getByTestId('project-reassign-save'));

		const warning = await r.findByTestId('project-reassign-warning');
		expect(warning.textContent).toContain('3');
		expect(r.getByTestId('project-reassign-warning-fiscal-years').textContent).toContain(
			'Exercice 2025',
		);
		expect(r.getByTestId('project-reassign-warning-locked').textContent).toContain('31.03.2026');
		expect(onDone).not.toHaveBeenCalled();

		await fireEvent.click(r.getByTestId('project-reassign-confirm'));
		await waitFor(() => expect(onDone).toHaveBeenCalledTimes(1));
		expect(submit).toHaveBeenCalledTimes(2);
		expect(submit.mock.calls[1][1]).toBe(true);
	});

	it('409 sur une seule écriture : la clé plate du singulier (mutation : toujours le pluriel)', async () => {
		const submit = vi.fn().mockRejectedValueOnce({
			code: 'PROJECT_REASSIGNMENT_CLOSED_PERIOD',
			status: 409,
			message: 'Confirmez.',
			details: { entryCount: 1, fiscalYears: ['Exercice 2025'], lockedThrough: null },
		});
		const { r } = open(submit);
		await fireEvent.change(r.getByTestId('project-reassign-select-7'), {
			target: { value: '12' },
		});
		await fireEvent.click(r.getByTestId('project-reassign-save'));
		const entries = await r.findByTestId('project-reassign-warning-entries');
		expect(entries.textContent).toContain('Une écriture');
		expect(r.queryByTestId('project-reassign-warning-locked')).toBeNull();
	});

	it('403 : le message de l’API, sans Confirmer (mutation : Confirmer montré sur 403)', async () => {
		const submit = vi.fn().mockRejectedValueOnce({
			code: 'PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY',
			status: 403,
			message: 'Seul un Administrateur connecté à l’interface web…',
			details: { entryCount: 1, fiscalYears: [], lockedThrough: '2026-03-31' },
		});
		const { r, onDone } = open(submit);
		await fireEvent.change(r.getByTestId('project-reassign-select-7'), {
			target: { value: '12' },
		});
		await fireEvent.click(r.getByTestId('project-reassign-save'));

		const error = await r.findByTestId('project-reassign-error');
		expect(error.textContent).toContain('Seul un Administrateur');
		expect(r.queryByTestId('project-reassign-confirm')).toBeNull();
		expect(r.queryByTestId('project-reassign-warning')).toBeNull();
		expect(onDone).not.toHaveBeenCalled();
	});

	// Story 43-6 (#823) — facture réglée en groupe : aucun code neuf à l'écran,
	// le message traduit par le serveur s'affiche tel quel.
	it('409 règlement groupé : le message de l’API, sans avertissement ni Confirmer (mutation : traité comme la période close)', async () => {
		const submit = vi.fn().mockRejectedValueOnce({
			code: 'PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT',
			status: 409,
			message: 'Cette facture est réglée avec d’autres par un même virement…',
			details: { invoiceId: 7, journalEntryId: 42 },
		});
		const { r, onDone } = open(submit);
		await fireEvent.change(r.getByTestId('project-reassign-select-7'), {
			target: { value: '12' },
		});
		await fireEvent.click(r.getByTestId('project-reassign-save'));

		const error = await r.findByTestId('project-reassign-error');
		expect(error.textContent).toContain('réglée avec d’autres par un même virement');
		expect(r.queryByTestId('project-reassign-confirm')).toBeNull();
		expect(r.queryByTestId('project-reassign-warning')).toBeNull();
		expect(onDone).not.toHaveBeenCalled();
	});
});
