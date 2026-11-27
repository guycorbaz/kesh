// Story 55-1a (D15, AC-14, AC-15, AC-20, R-18) — le sélecteur d'exercice des Rapports.

import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import ReportSelector from './ReportSelector.svelte';
import type { EtatMenu } from './fiscal-year-menu';

const FYS = [
	{ id: 2, companyId: 7, name: 'Exercice 2026', startDate: '2026-01-01', endDate: '2026-12-31', status: 'Open', multiCurrency: false },
	{ id: 1, companyId: 7, name: 'Exercice 2025/2026', startDate: '2025-07-01', endDate: '2026-06-30', status: 'Open', multiCurrency: false },
] as never[];

function rendre(menu: EtatMenu, extra: Record<string, unknown> = {}) {
	const props = {
		fiscalYears: FYS,
		menu,
		periodStart: '2026-01-01',
		periodEnd: '2026-12-31',
		onFiscalYearSelect: vi.fn(),
		onDatesInput: vi.fn(),
		onPeriodCommit: vi.fn(),
		onWholeFiscalYear: vi.fn(),
		...extra,
	};
	return { r: render(ReportSelector, props), props };
}

describe('ReportSelector (55-1a)', () => {
	it('AC-14 — un seul libellé « Exercice » par <label for>, aucun aria-label sur le menu', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true });
		const menu = r.getByRole('combobox', { name: 'Exercice' });
		expect(menu.getAttribute('aria-label')).toBeNull();
		expect(menu.id).toBe('report-fiscal-year');
	});

	it('AC-14 — menu en gras, options en graisse normale ; l’exercice seul (années), sans dates', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true });
		const menu = r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement;
		expect(menu.className).toContain('font-semibold');
		const options = [...menu.options];
		expect(options.every((o) => o.className.includes('font-normal'))).toBe(true);
		expect(options.map((o) => o.textContent)).toEqual(['2026', '2025/2026']);
	});

	it('AC-15 — choix d’un exercice : onFiscalYearSelect avec son identifiant', async () => {
		const { r, props } = rendre({ kind: 'fy', id: 2, whole: true });
		await fireEvent.change(r.getByRole('combobox', { name: 'Exercice' }), { target: { value: '1' } });
		expect(props.onFiscalYearSelect).toHaveBeenCalledWith(1);
	});

	it('AC-15 — période partielle : « Exercice entier » présent, clic → onWholeFiscalYear ; bornes exactes : absent', async () => {
		const { r, props } = rendre({ kind: 'fy', id: 2, whole: false });
		await fireEvent.click(r.getByTestId('report-fy-whole'));
		expect(props.onWholeFiscalYear).toHaveBeenCalled();
	});

	it('AC-15 — bornes exactes : pas de « Exercice entier »', () => {
		expect(rendre({ kind: 'fy', id: 2, whole: true }).r.queryByTestId('report-fy-whole')).toBeNull();
	});

	it('AC-15 — deux exercices : option « Plusieurs » sélectionnée et désactivée, message ; « Hors exercice » de même', () => {
		const { r } = rendre({ kind: 'multiple' });
		const menu = r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement;
		const plusieurs = [...menu.options].find((o) => o.textContent === 'Plusieurs')!;
		expect(plusieurs.disabled).toBe(true);
		expect(menu.value).toBe('__multiple');
		expect(r.getByTestId('report-needs-one-fy').getAttribute('role')).toBe('status');
	});

	it('AC-15 — une date hors de tout exercice : option « Hors exercice » sélectionnée et désactivée', () => {
		const menu = rendre({ kind: 'outside' }).r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement;
		expect([...menu.options].find((o) => o.textContent === 'Hors exercice')?.disabled).toBe(true);
		expect(menu.value).toBe('__outside');
	});

	it('un exercice choisi : aucune option désactivée ; unSeulExercice faux : pas de message', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true });
		expect([...(r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement).options].some((o) => o.disabled)).toBe(false);
	});

	it('unSeulExercice faux (Grand livre, 55-2) : « Plusieurs » sans message', () => {
		expect(rendre({ kind: 'multiple' }, { unSeulExercice: false }).r.queryByTestId('report-needs-one-fy')).toBeNull();
	});

	it('AC-15 — « Du » et « Au » dans un groupe « Période » (fieldset + legend)', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true });
		const groupe = r.getByRole('group', { name: 'Période' });
		expect(groupe.tagName).toBe('FIELDSET');
		expect(r.getByLabelText('Du')).toBeTruthy();
		expect(r.getByLabelText('Au')).toBeTruthy();
	});

	it('AC-20 — Du → Au ne confirme rien ; sortie du groupe et Entrée confirment', async () => {
		const { r, props } = rendre({ kind: 'fy', id: 2, whole: true });
		const du = r.getByTestId('report-from');
		const au = r.getByTestId('report-to');
		await fireEvent.focusOut(du, { relatedTarget: au });
		expect(props.onPeriodCommit).not.toHaveBeenCalled();
		await fireEvent.focusOut(au, { relatedTarget: null });
		expect(props.onPeriodCommit).toHaveBeenCalledWith('leave', null);
		await fireEvent.keyDown(du, { key: 'Enter' });
		expect(props.onPeriodCommit).toHaveBeenLastCalledWith('enter');
	});

	it('saisie d’une date : onDatesInput avec les deux dates', async () => {
		const { r, props } = rendre({ kind: 'fy', id: 2, whole: true });
		await fireEvent.input(r.getByTestId('report-to'), { target: { value: '2026-03-31' } });
		expect(props.onDatesInput).toHaveBeenCalledWith('2026-01-01', '2026-03-31');
	});

	it('R-18 — rien n’est désactivé sinon par l’absence d’exercice', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true });
		expect((r.getByTestId('report-from') as HTMLInputElement).disabled).toBe(false);
		expect((r.getByRole('combobox', { name: 'Exercice' }) as HTMLSelectElement).disabled).toBe(false);
	});

	it('aucun exercice : menu et dates désactivés, message d’aujourd’hui, pas de « seul exercice »', () => {
		const vide = rendre({ kind: 'outside' }, { fiscalYears: [] }).r;
		expect((vide.getByTestId('report-from') as HTMLInputElement).disabled).toBe(true);
		expect(vide.getByRole('alert').textContent).toContain('Aucun exercice comptable disponible');
		expect(vide.queryByTestId('report-needs-one-fy')).toBeNull();
	});

	it('période invalide : message role=status', () => {
		const { r } = rendre({ kind: 'fy', id: 2, whole: true }, { periodInvalid: true });
		expect(r.getByTestId('report-period-invalid').getAttribute('role')).toBe('status');
	});
});
