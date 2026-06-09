// Story 17-3d — Tests Vitest pour AdminRestorePanel.svelte (UI import installation).
// Couvre AC18 (upload), AC19 (confirmation forte : l'upload n'est déclenché QUE
// par « Confirmer » du modal, jamais par « Importer »), AC20 (erreurs typées +
// succès → déconnexion + redirect /login).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

vi.mock('./admin-restore.api', () => ({
	uploadFullImport: vi.fn(),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('svelte-sonner', () => ({
	toast: { success: vi.fn(), error: vi.fn() },
}));

// Mock du store auth : on vérifie que logout() est appelé au succès (DC-D4).
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: {
		logout: vi.fn().mockResolvedValue(undefined),
		currentUser: { role: 'Admin' },
	},
}));

import * as api from './admin-restore.api';
import { authState } from '$lib/app/stores/auth.svelte';
import AdminRestorePanel from './AdminRestorePanel.svelte';

const mockApi = vi.mocked(api);
const mockAuth = vi.mocked(authState);

let replaceMock: ReturnType<typeof vi.fn>;

function selectFile(input: HTMLInputElement) {
	const file = new File([new Uint8Array([0x50, 0x4b])], 'inst.keshbackup');
	Object.defineProperty(input, 'files', { value: [file], configurable: true });
	return fireEvent.change(input);
}

describe('AdminRestorePanel — Story 17-3d', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// jsdom : window.location.replace n'est pas implémenté → mock no-op.
		replaceMock = vi.fn();
		Object.defineProperty(window, 'location', {
			configurable: true,
			value: { replace: replaceMock },
		});
	});

	it('AC18 : bouton « Importer » désactivé tant qu’aucun fichier sélectionné', () => {
		const { getByTestId } = render(AdminRestorePanel);
		const button = getByTestId('admin-restore-import-button') as HTMLButtonElement;
		expect(button.disabled).toBe(true);
	});

	it('AC19 : « Importer » N’appelle PAS uploadFullImport (ouvre le modal de confirmation)', async () => {
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);

		const button = getByTestId('admin-restore-import-button') as HTMLButtonElement;
		expect(button.disabled).toBe(false);

		await fireEvent.click(button);
		// L'upload ne doit PAS avoir été déclenché par « Importer ».
		expect(mockApi.uploadFullImport).not.toHaveBeenCalled();
		// Le modal de confirmation est ouvert (bouton « Confirmer » présent).
		expect(await findByTestId('admin-restore-confirm')).toBeTruthy();
	});

	it('AC19 : « Confirmer » déclenche uploadFullImport ; succès → logout + redirect /login', async () => {
		mockApi.uploadFullImport.mockResolvedValue({
			backupCreated: true,
			tablesRestored: 21,
			rowsRestored: 42,
			sourceVersion: '0.1.8',
			sessionInvalidated: true,
		});
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
		await fireEvent.click(getByTestId('admin-restore-import-button'));

		const confirm = await findByTestId('admin-restore-confirm');
		await fireEvent.click(confirm);

		expect(mockApi.uploadFullImport).toHaveBeenCalledTimes(1);
		// Succès → déconnexion (DC-D4) + redirection /login.
		expect(mockAuth.logout).toHaveBeenCalledTimes(1);
		expect(replaceMock).toHaveBeenCalledWith('/login');
	});

	it('AC20 : erreur d’import affiche un encart + toast.error, pas de redirect', async () => {
		mockApi.uploadFullImport.mockRejectedValue(new Error('boom'));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
		await fireEvent.click(getByTestId('admin-restore-import-button'));

		const confirm = await findByTestId('admin-restore-confirm');
		await fireEvent.click(confirm);

		expect(await findByTestId('admin-restore-error')).toBeTruthy();
		expect(replaceMock).not.toHaveBeenCalled();
		expect(mockAuth.logout).not.toHaveBeenCalled();
	});
});
