// Story 17-3d — Tests Vitest pour AdminRestorePanel.svelte (UI import installation).
// Couvre AC18 (upload), AC19 (confirmation forte : l'upload n'est déclenché QUE
// par « Confirmer » du modal, jamais par « Importer »), AC20 (erreurs typées +
// succès → déconnexion + redirect /login).

import { describe, it, expect, vi, beforeEach, beforeAll, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

// Story 36-5 : simulacre partiel — `importHoldMs` et `RESTORE_HOLD_MARGIN_MS` restent les vrais.
vi.mock('./admin-restore.api', async (orig) => ({
	...(await orig<typeof import('./admin-restore.api')>()),
	uploadFullImport: vi.fn(),
}));

// Story 36-5 : simulacre partiel — `isApiError` et `LONG_OPERATION_TIMEOUT_MS` restent les vrais.
vi.mock('$lib/shared/utils/api-client', async (orig) => ({
	...(await orig<typeof import('$lib/shared/utils/api-client')>()),
	expireSession: vi.fn(),
}));

/**
 * Story 36-5 — la durée d'inactivité lue par le panneau (`sessionIdle.minutes`), et le
 * VRAI maintien d'`authState` (chargé en `beforeAll`), auquel les simulacres délèguent.
 */
const shared = vi.hoisted(() => ({
	minutes: null as number | null,
	realAuth: null as null | {
		holdForRestore(ms: number): void;
		releaseRestoreHold(): void;
		readonly restoreHold: boolean;
	},
}));

vi.mock('$lib/app/stores/session-idle.svelte', () => ({
	sessionIdle: {
		stop: vi.fn(),
		get minutes() {
			return shared.minutes;
		},
	},
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
		// Story 36-5 — espions qui délèguent au vrai maintien.
		holdForRestore: vi.fn((ms: number) => shared.realAuth?.holdForRestore(ms)),
		releaseRestoreHold: vi.fn(() => shared.realAuth?.releaseRestoreHold()),
		get restoreHold() {
			return shared.realAuth?.restoreHold ?? false;
		},
	},
}));

import * as api from './admin-restore.api';
import { authState } from '$lib/app/stores/auth.svelte';
import { expireSession, LONG_OPERATION_TIMEOUT_MS } from '$lib/shared/utils/api-client';
import { sessionIdle } from '$lib/app/stores/session-idle.svelte';
import { toast } from 'svelte-sonner';
import AdminRestorePanel from './AdminRestorePanel.svelte';

const mockApi = vi.mocked(api);
const mockAuth = vi.mocked(authState);
const mockToast = vi.mocked(toast);
const mockExpire = vi.mocked(expireSession);
const mockStop = vi.mocked(sessionIdle.stop);

beforeAll(async () => {
	const actual = await vi.importActual<typeof import('$lib/app/stores/auth.svelte')>(
		'$lib/app/stores/auth.svelte',
	);
	shared.realAuth = actual.authState;
});

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
		expect(avertissement.textContent).toContain('depuis une autre sauvegarde');
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

	it('AC18b (revue P1, E-LOW-3) : après une issue inconnue, l’import n’est plus proposé', async () => {
		mockApi.uploadFullImport.mockRejectedValue({ code: 'TIMEOUT', message: 'x', status: 0 });
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-relogin');
		expect((getByTestId('admin-restore-import-button') as HTMLButtonElement).disabled).toBe(true);
		expect((getByTestId('admin-restore-file-input') as HTMLInputElement).disabled).toBe(true);
	});

	it('AC18b : après une erreur ordinaire (409), l’import reste proposé', async () => {
		mockApi.uploadFullImport.mockRejectedValue({
			code: 'IMPORT_VERSION_INCOMPATIBLE',
			message: 'x',
			status: 409,
		});
		const { getByTestId, findByTestId, queryByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-error');
		expect(queryByTestId('admin-restore-relogin')).toBeNull();
		expect((getByTestId('admin-restore-import-button') as HTMLButtonElement).disabled).toBe(false);
	});

	it('A-L3 : la confirmation ne promet plus une déconnexion immédiate', async () => {
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await selectFile(getByTestId('admin-restore-file-input') as HTMLInputElement);
		await fireEvent.click(getByTestId('admin-restore-import-button'));
		const dialog = await findByTestId('admin-restore-confirm-dialog');
		expect(dialog.textContent).not.toContain('Vous serez déconnecté');
		expect(dialog.textContent).toContain('votre session sera fermée');
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

// Story 36-5 (#931, KF-056) — le maintien de session pendant l'import et la lecture du
// compte rendu : posé, borné et levé par le panneau seul.
describe('AdminRestorePanel — Story 36-5 (maintien de session)', () => {
	/** B par défaut : la durée d'inactivité inconnue vaut 60 minutes. */
	const B_DEFAUT = 60 * 60_000;
	/** La borne de la phase 1, recalculée depuis les constantes (jamais recopiée). */
	const PHASE_1 = 2 * LONG_OPERATION_TIMEOUT_MS + api.RESTORE_HOLD_MARGIN_MS;

	function enAttente<T>() {
		let resolve: (v: T) => void = () => {};
		let reject: (e: unknown) => void = () => {};
		const promise = new Promise<T>((res, rej) => {
			resolve = res;
			reject = rej;
		});
		return { promise, resolve, reject };
	}

	/** Minuteries simulées, sauf `setInterval` (l'attente de testing-library en dépend). */
	function horlogeSimulee() {
		vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] });
	}

	beforeEach(() => {
		vi.clearAllMocks();
		shared.minutes = null;
		replaceMock = vi.fn();
		Object.defineProperty(window, 'location', {
			configurable: true,
			value: { replace: replaceMock },
		});
	});

	afterEach(() => {
		shared.realAuth?.releaseRestoreHold();
		vi.useRealTimers();
	});

	it('AC2 : maintien posé AVANT l’envoi, réarmé à B au succès, suivi de session arrêté, aucune levée', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(2, 0));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-files-restored');

		expect(mockAuth.holdForRestore).toHaveBeenCalledTimes(2);
		expect(mockAuth.holdForRestore).toHaveBeenNthCalledWith(1, PHASE_1);
		expect(mockAuth.holdForRestore).toHaveBeenNthCalledWith(2, B_DEFAUT);
		expect(mockAuth.holdForRestore.mock.invocationCallOrder[0]).toBeLessThan(
			mockApi.uploadFullImport.mock.invocationCallOrder[0],
		);
		expect(mockStop).toHaveBeenCalledTimes(1);
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		// Un geste après le succès ne lève rien (la levée au geste n'est pas armée).
		await fireEvent.keyDown(window, { key: 'a' });
		await fireEvent.pointerDown(window);
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		expect(authState.restoreHold).toBe(true);
	});

	it('AC5 bis (c) : B = 5 min ⇒ la confirmation pose la borne de la phase 1, le succès B', async () => {
		shared.minutes = 5;
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-success');
		expect(mockAuth.holdForRestore.mock.calls).toEqual([[PHASE_1], [5 * 60_000]]);
		expect(api.importHoldMs(5 * 60_000)).toBe(PHASE_1);
	});

	it('AC3 (a) : échec certain (400) ⇒ message, aucune levée avant le geste, puis une seule au premier keydown', async () => {
		mockApi.uploadFullImport.mockRejectedValue({
			code: 'INVALID_BACKUP_STRUCTURE',
			message: 'x',
			status: 400,
		});
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-error');
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		expect(mockStop).not.toHaveBeenCalled();
		await fireEvent.keyDown(window, { key: 'a' });
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		await fireEvent.keyDown(window, { key: 'b' });
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
	});

	it('AC3 (a) : issue inconnue (TIMEOUT) ⇒ « Se reconnecter », import désactivé, levée au premier pointerdown', async () => {
		mockApi.uploadFullImport.mockRejectedValue({ code: 'TIMEOUT', message: 'x', status: 0 });
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-relogin');
		expect((getByTestId('admin-restore-import-button') as HTMLButtonElement).disabled).toBe(true);
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		expect(mockStop).not.toHaveBeenCalled();
		await fireEvent.pointerDown(window);
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		await fireEvent.pointerDown(window);
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
	});

	it('AC3 (a) : issue inconnue (504), démontage sans geste ⇒ levée, sans fin de session', async () => {
		mockApi.uploadFullImport.mockRejectedValue({ code: 'UNKNOWN_ERROR', message: 'x', status: 504 });
		const { getByTestId, findByTestId, unmount } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-error');
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		unmount();
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		expect(mockExpire).not.toHaveBeenCalled();
		// L'écouteur est retiré avec le panneau.
		await fireEvent.keyDown(window, { key: 'a' });
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
	});

	it('AC3 (a) : un geste PENDANT l’import ne lève rien ; l’écouteur n’est armé qu’après le message', async () => {
		const envoi = enAttente<ReturnType<typeof succes>>();
		mockApi.uploadFullImport.mockReturnValue(envoi.promise);
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await fireEvent.pointerDown(window);
		await fireEvent.keyDown(window, { key: 'a' });
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		envoi.reject({ code: 'INVALID_BACKUP_STRUCTURE', message: 'x', status: 400 });
		await findByTestId('admin-restore-error');
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		await fireEvent.keyDown(window, { key: 'b' });
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
	});

	it('AC4 : l’import refusé en 401 ⇒ levée puis fin de session, aucun message', async () => {
		mockApi.uploadFullImport.mockRejectedValue({
			code: 'UNAUTHENTICATED',
			message: 'Session expirée',
			status: 401,
		});
		const { getByTestId, findByTestId, queryByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await vi.waitFor(() => expect(mockExpire).toHaveBeenCalledTimes(1));
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalled();
		expect(mockAuth.releaseRestoreHold.mock.invocationCallOrder[0]).toBeLessThan(
			mockExpire.mock.invocationCallOrder[0],
		);
		expect(queryByTestId('admin-restore-error')).toBeNull();
	});

	it('AC5 (a) : démontage sans import ⇒ levée, sans fin de session', () => {
		const { unmount } = render(AdminRestorePanel);
		unmount();
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		expect(mockExpire).not.toHaveBeenCalled();
	});

	it('AC5 (a) : démontage pendant l’import ⇒ levée, sans fin de session', async () => {
		mockApi.uploadFullImport.mockReturnValue(enAttente<ReturnType<typeof succes>>().promise);
		const { getByTestId, findByTestId, unmount } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		expect(mockAuth.holdForRestore).toHaveBeenCalledTimes(1);
		unmount();
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		expect(mockExpire).not.toHaveBeenCalled();
	});

	it('AC5 (b) : succès arrivé APRÈS le démontage ⇒ ni réarmement, ni arrêt du suivi, ni minuterie', async () => {
		horlogeSimulee();
		const envoi = enAttente<ReturnType<typeof succes>>();
		mockApi.uploadFullImport.mockReturnValue(envoi.promise);
		const { getByTestId, findByTestId, unmount } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		unmount();
		envoi.resolve(succes(1, 0));
		await vi.advanceTimersByTimeAsync(0);
		await vi.advanceTimersByTimeAsync(B_DEFAUT + 1);
		expect(mockAuth.holdForRestore).toHaveBeenCalledTimes(1);
		expect(mockStop).not.toHaveBeenCalled();
		expect(mockExpire).not.toHaveBeenCalled();
	});

	it('AC5 (c) : démontage pendant la lecture ⇒ levée puis fin de session ; la minuterie de B est effacée', async () => {
		horlogeSimulee();
		mockApi.uploadFullImport.mockResolvedValue(succes(1, 0));
		const { getByTestId, findByTestId, unmount } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-success');
		unmount();
		expect(mockExpire).toHaveBeenCalledTimes(1);
		expect(mockAuth.releaseRestoreHold.mock.invocationCallOrder[0]).toBeLessThan(
			mockExpire.mock.invocationCallOrder[0],
		);
		await vi.advanceTimersByTimeAsync(B_DEFAUT + 1);
		expect(mockExpire).toHaveBeenCalledTimes(1);
	});

	it('AC5 (d) : « Se reconnecter » puis démontage ⇒ pas de fin de session par le nettoyage', async () => {
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		const { getByTestId, findByTestId, unmount } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await fireEvent.click(await findByTestId('admin-restore-relogin'));
		await vi.waitFor(() => expect(replaceMock).toHaveBeenCalledWith('/login'));
		unmount();
		expect(mockExpire).not.toHaveBeenCalled();
	});

	it('AC5 bis (b) : au bout de B après le succès, levée puis fin de session — rien avant', async () => {
		horlogeSimulee();
		mockApi.uploadFullImport.mockResolvedValue(succes(0, 0));
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await findByTestId('admin-restore-success');
		await vi.advanceTimersByTimeAsync(B_DEFAUT - 1);
		expect(mockAuth.releaseRestoreHold).not.toHaveBeenCalled();
		expect(mockExpire).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(1);
		expect(mockExpire).toHaveBeenCalledTimes(1);
		expect(mockAuth.releaseRestoreHold).toHaveBeenCalledTimes(1);
		expect(mockAuth.releaseRestoreHold.mock.invocationCallOrder[0]).toBeLessThan(
			mockExpire.mock.invocationCallOrder[0],
		);
	});

	it('AC5 bis (d) : import plus long que B (1 min) ⇒ maintien gardé, puis borné à B après le succès', async () => {
		horlogeSimulee();
		shared.minutes = 1;
		const envoi = enAttente<ReturnType<typeof succes>>();
		mockApi.uploadFullImport.mockReturnValue(envoi.promise);
		const { getByTestId, findByTestId } = render(AdminRestorePanel);
		await importer(getByTestId, findByTestId);
		await vi.advanceTimersByTimeAsync(61_000);
		expect(authState.restoreHold).toBe(true);
		await vi.advanceTimersByTimeAsync(3_600_000);
		expect(authState.restoreHold).toBe(true);
		envoi.resolve(succes(2, 0));
		await findByTestId('admin-restore-files-restored');
		expect(getByTestId('admin-restore-success')).toBeTruthy();
		await vi.advanceTimersByTimeAsync(59_999);
		expect(authState.restoreHold).toBe(true);
		await vi.advanceTimersByTimeAsync(1);
		expect(authState.restoreHold).toBe(false);
	});
});
