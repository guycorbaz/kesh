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

// Mock i18n : retourne le fallback (déterministe), variables interpolées comme
// le fait `i18nMsg` (Story 31-1b : les avis portent un nombre).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k: string) => String(args[k] ?? '')) : fallback,
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
		mockApi.downloadFullExport.mockResolvedValue({
			filesNotIncluded: 0,
			exceedsImportLimitMib: null,
		});
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
			() =>
				new Promise((resolve) => {
					resolveFn = () => resolve({ filesNotIncluded: null, exceedsImportLimitMib: null });
				}),
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

// Story 31-1b (#503) — ce que le fichier ne contient pas (AC17), ce que l'export a omis
// (AC18), la description complétée (AC22).
describe('AdminBackupPanel — Story 31-1b', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('AC17 : avis permanent sur le .env, avant tout export', () => {
		const { getByTestId } = render(AdminBackupPanel);
		const avis = getByTestId('backup-env-notice');
		expect(avis.textContent).toContain('ne contient pas la configuration du serveur');
		expect(avis.textContent).toContain('.env');
		expect(avis.textContent).toContain("recréez-la avant d'importer");
	});

	it('AC22 : la description nomme les pièces justificatives', () => {
		const { getByTestId } = render(AdminBackupPanel);
		expect(getByTestId('admin-backup-description').textContent).toContain(
			'les pièces justificatives',
		);
	});

	it('AC18 : pièces non incluses → avertissement avec leur nombre', async () => {
		mockApi.downloadFullExport.mockResolvedValue({
			filesNotIncluded: 3,
			exceedsImportLimitMib: null,
		});
		const { getByTestId, findByTestId, queryByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		const avis = await findByTestId('admin-backup-files-not-included');
		expect(avis.textContent).toContain('3');
		expect(avis.textContent).toContain('journal du serveur');
		expect(queryByTestId('admin-backup-exceeds-import-limit')).toBeNull();
	});

	it('AC18 : fichier trop grand pour l’import → avertissement avec le plafond', async () => {
		mockApi.downloadFullExport.mockResolvedValue({
			filesNotIncluded: 0,
			exceedsImportLimitMib: 2048,
		});
		const { getByTestId, findByTestId, queryByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		const avis = await findByTestId('admin-backup-exceeds-import-limit');
		expect(avis.textContent).toContain('2048 Mio');
		expect(avis.textContent).toContain('KESH_ADMIN_IMPORT_MAX_MB');
		expect(queryByTestId('admin-backup-files-not-included')).toBeNull();
	});

	it('AC18 : en-têtes absents ou à zéro → aucun avis', async () => {
		mockApi.downloadFullExport.mockResolvedValue({
			filesNotIncluded: null,
			exceedsImportLimitMib: null,
		});
		const { getByTestId, queryByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		await vi.waitFor(() => expect(mockToast.success).toHaveBeenCalledTimes(1));
		expect(queryByTestId('admin-backup-files-not-included')).toBeNull();
		expect(queryByTestId('admin-backup-exceeds-import-limit')).toBeNull();
	});

	it('AC18 : un nouvel export efface les avis du précédent', async () => {
		mockApi.downloadFullExport.mockResolvedValueOnce({
			filesNotIncluded: 2,
			exceedsImportLimitMib: 2048,
		});
		const { getByTestId, findByTestId, queryByTestId } = render(AdminBackupPanel);
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		await findByTestId('admin-backup-files-not-included');
		mockApi.downloadFullExport.mockResolvedValueOnce({
			filesNotIncluded: 0,
			exceedsImportLimitMib: null,
		});
		await fireEvent.click(getByTestId('admin-backup-export-button'));
		await vi.waitFor(() => {
			expect(queryByTestId('admin-backup-files-not-included')).toBeNull();
			expect(queryByTestId('admin-backup-exceeds-import-limit')).toBeNull();
		});
	});
});
