// Story 17-3d — Tests Vitest pour AdminRestorePanel.svelte (UI import installation).
// Couvre AC18 (upload), AC19 (confirmation forte : l'upload n'est déclenché QUE
// par « Confirmer » du modal, jamais par « Importer »), AC20 (erreurs typées +
// succès → déconnexion + redirect /login).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

vi.mock('./admin-restore.api', () => ({
	uploadFullImport: vi.fn(),
}));

// Variables interpolées comme le fait `i18nMsg` (Story 31-1b : les avis portent un nombre).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_m, k: string) => String(args[k] ?? '')) : fallback,
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
import { toast } from 'svelte-sonner';
import AdminRestorePanel from './AdminRestorePanel.svelte';

const mockApi = vi.mocked(api);
const mockAuth = vi.mocked(authState);
const mockToast = vi.mocked(toast);

let replaceMock: ReturnType<typeof vi.fn>;

function selectFile(input: HTMLInputElement) {
	const file = new File([new Uint8Array([0x50, 0x4b])], 'inst.keshbackup');
	Object.defineProperty(input, 'files', { value: [file], configurable: true });
	return fireEvent.change(input);
}

/** Réponse `200` de l'import (Story 31-1b : avec les comptes de pièces de la 31-1a). */
function succes(filesRestored: number, filesMissing: number) {
	return {
		backupCreated: true,
		tablesRestored: 21,
		rowsRestored: 42,
		sourceVersion: '0.15.0',
		sessionInvalidated: true,
		filesRestored,
		filesMissing,
	};
}

type Query = (id: string) => HTMLElement;
type FindQuery = (id: string) => Promise<HTMLElement>;

/** Sélectionne un fichier, ouvre la confirmation et confirme. */
async function importer(getByTestId: Query, findByTestId: FindQuery) {
	await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
	await fireEvent.click(getByTestId('admin-restore-import-button'));
	await fireEvent.click(await findByTestId('admin-restore-confirm'));
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

	it('AC19 : « Confirmer » déclenche uploadFullImport ; succès → « Se reconnecter » → logout + redirect /login', async () => {
		// Story 31-1b (AC19, C-31-1-39) : la déconnexion attend désormais le clic
		// « Se reconnecter » — elle n'est plus immédiate.
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);

		const relogin = await findByTestId('admin-restore-relogin');
		expect(mockApi.uploadFullImport).toHaveBeenCalledTimes(1);
		expect(mockAuth.logout).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();

		await fireEvent.click(relogin);
		// Chaîne async (logout → replace) : attendre sa complétion.
		await vi.waitFor(() => {
			// Succès → déconnexion (DC-D4) + redirection /login.
			expect(mockAuth.logout).toHaveBeenCalledTimes(1);
			expect(replaceMock).toHaveBeenCalledWith('/login');
		});
	});

	it('AC20 : erreur d’import affiche un encart + toast.error, pas de redirect', async () => {
		mockApi.uploadFullImport.mockRejectedValue(new Error('boom'));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
		await fireEvent.click(getByTestId('admin-restore-import-button'));

		const confirm = await findByTestId('admin-restore-confirm');
		await fireEvent.click(confirm);

		expect(await findByTestId('admin-restore-error')).toBeTruthy();
		await vi.waitFor(() => {
			expect(replaceMock).not.toHaveBeenCalled();
			expect(mockAuth.logout).not.toHaveBeenCalled();
		});
	});

	it('AC20 : ré-ouvrir « Importer » après une erreur efface l’encart d’erreur', async () => {
		mockApi.uploadFullImport.mockRejectedValue(new Error('boom'));
		const { getByTestId, findByTestId, queryByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
		await fireEvent.click(getByTestId('admin-restore-import-button'));
		await fireEvent.click(await findByTestId('admin-restore-confirm'));
		// Erreur affichée.
		expect(await findByTestId('admin-restore-error')).toBeTruthy();
		// Re-clic « Importer » → requestImport remet errorMsg=null (rouvre le modal).
		await fireEvent.click(getByTestId('admin-restore-import-button'));
		await vi.waitFor(() => {
			expect(queryByTestId('admin-restore-error')).toBeNull();
		});
	});
});

// Story 31-1b (#503) — ce que le fichier ne contient pas (AC17), ce que la restauration a
// rendu (AC19), l'issue inconnue d'un import long (AC18b).
describe('AdminRestorePanel — Story 31-1b', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		replaceMock = vi.fn();
		Object.defineProperty(window, 'location', {
			configurable: true,
			value: { replace: replaceMock },
		});
	});

	it('AC17 : avis permanent sur le .env', () => {
		const { getByTestId } = render(AdminRestorePanel);
		const avis = getByTestId('backup-env-notice');
		expect(avis.textContent).toContain('ne contient pas la configuration du serveur');
		expect(avis.textContent).toContain('.env');
	});

	it('AC19 : filesMissing = 0 → pièces restaurées, aucun avertissement, « Se reconnecter » attendu', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(2, 0));
		const { getByTestId, findByTestId, queryByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);

		const avis = await findByTestId('admin-restore-success');
		expect(avis.textContent).toContain('2');
		expect(queryByTestId('admin-restore-files-missing')).toBeNull();
		expect(getByTestId('admin-restore-relogin')).toBeTruthy();
		// Aucune redirection avant le clic (C-31-1-39), et plus de toast « vous allez être déconnecté ».
		expect(mockAuth.logout).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();
		expect(mockToast.success).not.toHaveBeenCalled();
	});

	it('AC19 : filesMissing > 0 → avertissement avec leur nombre, et « Se reconnecter »', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(1, 3));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);

		const avertissement = await findByTestId('admin-restore-files-missing');
		expect(avertissement.textContent).toContain('3');
		expect(avertissement.textContent).toContain('document indisponible');
		expect(getByTestId('admin-restore-relogin')).toBeTruthy();
		expect(mockAuth.logout).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();

		await fireEvent.click(getByTestId('admin-restore-relogin'));
		await vi.waitFor(() => {
			expect(mockAuth.logout).toHaveBeenCalledTimes(1);
			expect(replaceMock).toHaveBeenCalledWith('/login');
		});
	});

	it('AC19 : « Se reconnecter » redirige même si logout() échoue', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		mockAuth.logout.mockRejectedValueOnce(new Error('réseau'));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await fireEvent.click(await findByTestId('admin-restore-relogin'));
		await vi.waitFor(() => expect(replaceMock).toHaveBeenCalledWith('/login'));
	});

	it('AC19 : après un import réussi, un second import n’est plus proposé', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-relogin');
		expect((getByTestId('admin-restore-import-button') as HTMLButtonElement).disabled).toBe(true);
		expect((getByTestId('admin-restore-file-input') as HTMLInputElement).disabled).toBe(true);
	});

	const INCONNUE = "La réponse du serveur n'est pas arrivée : l'import a pu aboutir.";

	for (const [nom, erreur] of [
		['TIMEOUT', { code: 'TIMEOUT', message: 'Le serveur ne répond pas. Réessayez ultérieurement.', status: 0 }],
		[
			'NETWORK_ERROR',
			{ code: 'NETWORK_ERROR', message: 'Impossible de contacter le serveur. Vérifiez votre connexion.', status: 0 },
		],
		['502', { code: 'UNKNOWN_ERROR', message: 'Erreur serveur (502)', status: 502 }],
		['504', { code: 'UNKNOWN_ERROR', message: 'Erreur serveur (504)', status: 504 }],
	] as const) {
		it(`AC18b : ${nom} → message d’issue inconnue (reconnectez-vous pour vérifier)`, async () => {
			mockApi.uploadFullImport.mockRejectedValue({ ...erreur });
			const { getByTestId, findByTestId } = render(AdminRestorePanel);
			await importer(getByTestId, findByTestId);
			const encart = await findByTestId('admin-restore-error');
			expect(encart.textContent).toContain(INCONNUE);
			expect(encart.textContent).toContain('Reconnectez-vous');
			expect(encart.textContent).not.toContain('Le serveur ne répond pas');
			expect(encart.textContent).not.toContain('Erreur serveur (');
			expect(encart.textContent).not.toContain('préservé');
			expect(replaceMock).not.toHaveBeenCalled();
		});
	}

	it('AC18b : 503 → message par défaut (celui du serveur), pas l’issue inconnue', async () => {
		mockApi.uploadFullImport.mockRejectedValue({
			code: 'SERVICE_UNAVAILABLE',
			message: 'Service momentanément indisponible.',
			status: 503,
		});
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		const encart = await findByTestId('admin-restore-error');
		expect(encart.textContent).toContain('Service momentanément indisponible.');
		expect(encart.textContent).not.toContain(INCONNUE);
	});
});
