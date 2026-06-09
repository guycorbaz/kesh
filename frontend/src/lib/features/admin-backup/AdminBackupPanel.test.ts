// Story 17-3b — Tests Vitest pour AdminBackupPanel.svelte (UI export installation).
// Pattern : mock du module API + i18n + svelte-sonner AVANT l'import du
// composant (hoisting Vitest). Couvre AC8 (bouton déclenche l'export), AC9
// (état chargement désactive le bouton + guard ré-entrance + erreur affichée).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

// Mock du module API (downloadFullExport).
vi.mock('./admin-backup.api', () => ({
	downloadFullExport: vi.fn(),
}));

// Mock i18n : retourne le fallback (déterministe).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

// Mock toast svelte-sonner.
vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

import * as api from './admin-backup.api';
import { toast } from 'svelte-sonner';
import AdminBackupPanel from './AdminBackupPanel.svelte';

const mockApi = vi.mocked(api);
const mockToast = vi.mocked(toast);

describe('AdminBackupPanel — Story 17-3b', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('AC8 : rend le bouton et le titre', () => {
		const { getByTestId } = render(AdminBackupPanel);
		expect(getByTestId('admin-backup-export-button')).toBeTruthy();
		expect(getByTestId('admin-backup-title')).toBeTruthy();
	});

	it('AC8 : clic sur le bouton appelle downloadFullExport', async () => {
		mockApi.downloadFullExport.mockResolvedValue(undefined);
		const { getByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		expect(mockApi.downloadFullExport).toHaveBeenCalledTimes(1);
		// Succès → toast.success
		expect(mockToast.success).toHaveBeenCalledTimes(1);
	});

	it('AC9 : pendant l\'export le bouton est désactivé + guard ré-entrance (2e clic court-circuité)', async () => {
		// Backend lent : la Promise ne se résout pas pendant le test.
		let resolveFn!: () => void;
		mockApi.downloadFullExport.mockImplementation(
			() => new Promise<void>((resolve) => (resolveFn = resolve)),
		);
		const { getByTestId } = render(AdminBackupPanel);
		const button = getByTestId('admin-backup-export-button') as HTMLButtonElement;

		await fireEvent.click(button); // 1er clic → exporting=true
		expect(button.disabled).toBe(true);

		await fireEvent.click(button); // 2e clic pendant l'export → guard
		// downloadFullExport ne doit avoir été appelé qu'une seule fois.
		expect(mockApi.downloadFullExport).toHaveBeenCalledTimes(1);

		resolveFn(); // laisse le 1er export terminer proprement
	});

	it('AC9 : échec de l\'export affiche un encart d\'erreur + toast.error', async () => {
		mockApi.downloadFullExport.mockRejectedValue(new Error('boom'));
		const { getByTestId, findByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		// L'encart d'erreur apparaît (fallback générique car non-ApiError).
		const errorBox = await findByTestId('admin-backup-error');
		expect(errorBox).toBeTruthy();
		expect(mockToast.error).toHaveBeenCalledTimes(1);
	});
});
