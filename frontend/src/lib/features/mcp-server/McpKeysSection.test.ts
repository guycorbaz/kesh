/**
 * Story 32-7 (refs #723) — la section des clés MCP (AC-8 à AC-13).
 *
 * La propriété de la story : le secret d'une clé, affiché UNE fois (AC-11). Les cinq
 * « ni » ont chacun leur test, et chaque espion son TÉMOIN — un appel posé par le
 * test lui-même et vu par l'espion, puis `mockClear()` — sans lequel un espion posé
 * sur le mauvais objet ferait passer une mutation à vide.
 *
 * Secret simulé : `kesh_mcp_SECRET…` ; témoins : `kesh_mcp_TEMOIN` (aucun des deux
 * littéraux ne contient l'autre).
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./mcp-server.api', () => ({
	listMcpKeys: vi.fn(),
	createMcpKey: vi.fn(),
	revokeMcpKey: vi.fn(),
	setMcpKeySendValidation: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));
vi.mock('$lib/shared/utils/clipboard', () => ({ copyToClipboard: vi.fn(async () => true) }));
// `toast` est une fonction appelable ET porteuse de méthodes (`svelte-sonner` 1.1.0,
// `dist/toast-state.svelte.js:213-226`) : le simulacre est un `vi.fn()` qui porte TOUTES
// les entrées de l'objet exporté, chacune un `vi.fn()` ; les autres exportations du
// module sont reprises de l'original.
vi.mock('svelte-sonner', async () => {
	const reel = await vi.importActual<typeof import('svelte-sonner')>('svelte-sonner');
	const simule = vi.fn() as unknown as Record<string, unknown>;
	for (const nom of Object.keys(reel.toast)) {
		simule[nom] = nom === 'getActiveToasts' ? vi.fn(() => []) : vi.fn();
	}
	return { ...reel, toast: simule };
});

import { toast } from 'svelte-sonner';
import * as api from './mcp-server.api';
import { SecretUnique } from './secret-unique.svelte';
import McpKeysSection from './McpKeysSection.svelte';
import { PORTEES_MCP, type McpKey } from './mcp-server.types';

const mockApi = vi.mocked(api);
// Factice, et hors du format d'un secret réel (`kesh_mcp_` + 27 base62) : les `_` le
// rendent invisible à tout détecteur de secrets (revue de code P1, A-7 ; C-32-7-38).
const SECRET = 'kesh_mcp_SECRET_factice_0123456789';
const TEMOIN = 'kesh_mcp_TEMOIN';

function cle(id: number, over: Partial<McpKey> = {}): McpKey {
	return {
		id,
		name: `clé ${id}`,
		scopes: ['books:read'],
		unknownScopes: [],
		sendValidation: 'required',
		createdByUserId: 987654,
		createdAt: '2026-10-25T10:00:00.123',
		lastUsedAt: null,
		revokedAt: null,
		expiresAt: null,
		version: 1,
		...over,
	};
}

beforeEach(() => {
	vi.clearAllMocks();
	vi.restoreAllMocks();
});
afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

async function monter(cles: McpKey[] = []) {
	mockApi.listMcpKeys.mockResolvedValueOnce(cles);
	const vue = render(McpKeysSection);
	await waitFor(() => expect(vue.queryByTestId('mcp-server-keys-loading')).toBeNull());
	return vue;
}

async function ouvrirEtRemplir(
	vue: { getByTestId: (id: string) => HTMLElement },
	nom = 'agent',
	portee = 'books-read',
) {
	await fireEvent.click(vue.getByTestId('mcp-server-keys-new'));
	await fireEvent.input(vue.getByTestId('mcp-server-form-name'), { target: { value: nom } });
	if (portee) await fireEvent.click(vue.getByTestId(`mcp-server-form-scope-${portee}`));
}

/** Le nombre de nœuds texte du document qui contiennent `s`. */
function noeudsTexte(s: string): number {
	const marcheur = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
	let n = 0;
	while (marcheur.nextNode()) if ((marcheur.currentNode.nodeValue ?? '').includes(s)) n++;
	return n;
}

describe('la liste des clés (AC-8)', () => {
	it('colonnes, portées en code, politique, statut, ordre du serveur', async () => {
		const vue = await monter([
			cle(3, { scopes: ['books:read', 'contacts:read'], sendValidation: 'exempt' }),
			cle(2, { revokedAt: '2026-10-25T11:00:00.000' }),
			cle(1, { expiresAt: '2020-01-01T00:00:00.000' }),
		]);
		const ids = [...vue.getByTestId('mcp-server-keys-table').querySelectorAll('tbody tr')].map((tr) =>
			tr.getAttribute('data-testid'),
		);
		expect(ids).toEqual(['mcp-server-key-row-3', 'mcp-server-key-row-2', 'mcp-server-key-row-1']);
		expect(vue.getByTestId('mcp-server-key-scopes-3').textContent).toContain('books:read');
		expect(vue.getByTestId('mcp-server-key-scopes-3').textContent).toContain('contacts:read');
		expect(vue.getByTestId('mcp-server-key-policy-3').textContent?.trim()).toBe('Dispensée');
		expect(vue.getByTestId('mcp-server-key-policy-2').textContent?.trim()).toBe('Requise');
		expect(vue.getByTestId('mcp-server-key-status-3').textContent?.trim()).toBe('Active');
		expect(vue.getByTestId('mcp-server-key-status-2').textContent).toContain('Révoquée le');
		expect(vue.getByTestId('mcp-server-key-status-1').textContent).toContain('Expirée le');
		expect(vue.getByTestId('mcp-server-key-expires-3').textContent?.trim()).toBe('jamais');
		expect(vue.getByTestId('mcp-server-key-last-used-3').textContent?.trim()).toBe('jamais');
		// le créateur n'est pas affiché (C-32-7-16)
		// (revue de code P1, A-5 : le numéro du créateur, non le nom du champ)
		expect(vue.getByTestId('mcp-server-keys-table').textContent).not.toContain('987654');
	});

	it('une portée inconnue en double ne fait pas tomber la liste (revue de code P1, E1)', async () => {
		const vue = await monter([cle(2, { scopes: ['books:read'], unknownScopes: ['admin', 'admin'] }), cle(1)]);
		const portees = vue.getByTestId('mcp-server-key-scopes-2').textContent ?? '';
		expect(portees.split('admin — inconnue de cette version').length - 1).toBe(2);
		// la révocation des autres clés reste offerte
		expect(vue.getByTestId('mcp-server-key-revoke-1')).toBeTruthy();
	});

	it('(a) portées inconnues : « jamais accordée », et « n’ouvre aucun accès » si elles seules', async () => {
		const vue = await monter([
			cle(2, { scopes: ['books:read'], unknownScopes: ['ledger:write'] }),
			cle(1, { scopes: [], unknownScopes: ['ledger:write'] }),
		]);
		expect(vue.getByTestId('mcp-server-key-scopes-2').textContent).toContain(
			'ledger:write — inconnue de cette version — jamais accordée',
		);
		expect(vue.getByTestId('mcp-server-key-scopes-2').textContent).not.toContain('n’ouvre aucun accès');
		expect(vue.getByTestId('mcp-server-key-scopes-1').textContent).toContain('n’ouvre aucun accès');
	});

	it('(b) la note de la dernière utilisation', async () => {
		const vue = await monter([cle(1, { lastUsedAt: '2026-10-25T10:00:00.000' })]);
		expect(vue.getByTestId('mcp-server-key-last-used-1').textContent).not.toContain('jamais');
		const note = vue.getByTestId('mcp-server-keys-last-used-note').textContent ?? '';
		expect(note).toContain('posée chaque fois que la clé est reconnue');
		expect(note).toContain('même si l’appel est ensuite refusé ou n’est pas servi');
		expect(vue.getByTestId('mcp-server-keys-table').textContent).not.toContain('dernier appel');
	});

	it('(c) état vide, échec du chargement (ApiError, puis hors ApiError)', async () => {
		const vue = await monter([]);
		expect(vue.getByTestId('mcp-server-keys-empty').textContent).toContain('Aucune clé MCP.');
		cleanup();
		mockApi.listMcpKeys.mockRejectedValueOnce({ code: 'FORBIDDEN', status: 403, message: 'Accès refusé.' });
		const vue2 = render(McpKeysSection);
		expect((await vue2.findByTestId('mcp-server-keys-load-error')).textContent?.trim()).toBe('Accès refusé.');
		cleanup();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		mockApi.listMcpKeys.mockRejectedValueOnce(new Error('boom'));
		const vue3 = render(McpKeysSection);
		const e = await vue3.findByTestId('mcp-server-keys-load-error');
		expect(e.textContent?.trim()).toBe('Erreur inattendue.');
		expect(e.textContent).not.toContain('boom');
	});
});

describe('créer une clé (AC-9, AC-10)', () => {
	it('formulaire_a_l_ouverture_requise_et_aucune_portee', async () => {
		const vue = await monter([]);
		await fireEvent.click(vue.getByTestId('mcp-server-keys-new'));
		expect((vue.getByTestId('mcp-server-form-policy-required') as HTMLInputElement).checked).toBe(true);
		expect((vue.getByTestId('mcp-server-form-policy-exempt') as HTMLInputElement).checked).toBe(false);
		for (const p of ['books-read', 'invoices-read', 'contacts-read', 'documents-read', 'prompts-write']) {
			expect((vue.getByTestId(`mcp-server-form-scope-${p}`) as HTMLInputElement).checked, p).toBe(false);
		}
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValueOnce([cle(9)]);
		await fireEvent.input(vue.getByTestId('mcp-server-form-name'), { target: { value: 'agent' } });
		await fireEvent.click(vue.getByTestId('mcp-server-form-scope-books-read'));
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await waitFor(() => expect(mockApi.createMcpKey).toHaveBeenCalledTimes(1));
		expect(mockApi.createMcpKey.mock.calls[0][0]).toEqual({
			name: 'agent',
			scopes: ['books:read'],
			sendValidation: 'required',
		});
		// rouvert après la création : les défauts, de nouveau
		await vue.findByTestId('mcp-server-secret-notice');
		await fireEvent.click(vue.getByTestId('mcp-server-keys-new'));
		expect((vue.getByTestId('mcp-server-form-policy-required') as HTMLInputElement).checked).toBe(true);
		expect((vue.getByTestId('mcp-server-form-scope-books-read') as HTMLInputElement).checked).toBe(false);
		expect((vue.getByTestId('mcp-server-form-name') as HTMLInputElement).value).toBe('');
	});

	it('les cinq phrases de portée, une par case, dans l’ordre de McpScope::ALL', async () => {
		const vue = await monter([]);
		await fireEvent.click(vue.getByTestId('mcp-server-keys-new'));
		// `PORTEES_MCP` = `McpScope::ALL`, recopié de `crates/kesh-db/src/entities/mcp_key.rs:48-54`
		// (ordre canonique ; revue de code P1, A-3).
		expect(PORTEES_MCP).toEqual(['books:read', 'invoices:read', 'contacts:read', 'documents:read', 'prompts:write']);
		const cases = [...vue.getByTestId('mcp-server-form').querySelectorAll('input[type="checkbox"]')].map((e) =>
			e.getAttribute('data-testid'),
		);
		expect(cases).toEqual([
			'mcp-server-form-scope-books-read',
			'mcp-server-form-scope-invoices-read',
			'mcp-server-form-scope-contacts-read',
			'mcp-server-form-scope-documents-read',
			'mcp-server-form-scope-prompts-write',
		]);
		expect(vue.getByTestId('mcp-server-form-scope-help-books-read').textContent).toContain('le grand livre');
		expect(vue.getByTestId('mcp-server-form-scope-help-invoices-read').textContent).toContain('masqués');
		expect(vue.getByTestId('mcp-server-form-scope-help-contacts-read').textContent).toContain(
			'L’IBAN n’est pas une donnée du carnet d’adresses.',
		);
		expect(vue.getByTestId('mcp-server-form-scope-help-documents-read').textContent).toContain('sans son contenu');
		expect(vue.getByTestId('mcp-server-form-scope-help-prompts-write').textContent).toContain(
			'une clé à validation requise peut proposer',
		);
		expect(vue.getByTestId('mcp-server-form').textContent).toContain(
			'Aucune portée n’écrit dans les livres ni n’administre Kesh.',
		);
		expect(vue.getByTestId('mcp-server-form-policy-help-required').textContent).toContain('Envois du serveur MCP');
		expect(vue.getByTestId('mcp-server-form-policy-help-exempt').textContent).toContain(
			'chaque appel reste journalisé',
		);
	});

	it('(a) refus à l’écran sans appel : nom vide, aucune portée', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue, '   ', 'books-read');
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		expect(vue.getByTestId('mcp-server-form-error').textContent).toContain('Le nom est requis.');
		await fireEvent.input(vue.getByTestId('mcp-server-form-name'), { target: { value: 'x' } });
		await fireEvent.click(vue.getByTestId('mcp-server-form-scope-books-read'));
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		expect(vue.getByTestId('mcp-server-form-error').textContent).toContain('Choisissez au moins une portée.');
		await fireEvent.input(vue.getByTestId('mcp-server-form-name'), { target: { value: 'é'.repeat(256) } });
		await fireEvent.click(vue.getByTestId('mcp-server-form-scope-books-read'));
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		expect(vue.getByTestId('mcp-server-form-error').textContent).toContain('255 caractères au plus');
		expect(mockApi.createMcpKey).not.toHaveBeenCalled();
	});

	it('(b) 400 du serveur : son message, saisies conservées ; expiration envoyée en fin de journée locale', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		await fireEvent.input(vue.getByTestId('mcp-server-form-expires'), { target: { value: '2099-12-31' } });
		mockApi.createMcpKey.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: 'Nom refusé.' });
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		expect((await vue.findByTestId('mcp-server-form-error')).textContent).toContain('Nom refusé.');
		expect((vue.getByTestId('mcp-server-form-name') as HTMLInputElement).value).toBe('agent');
		expect((vue.getByTestId('mcp-server-form-scope-books-read') as HTMLInputElement).checked).toBe(true);
		const envoye = mockApi.createMcpKey.mock.calls[0][0];
		expect(envoye.expiresAt).toBe(new Date(2099, 11, 31, 23, 59, 59, 999).toISOString());
	});

	it('une expiration saisie mais illisible est refusée, jamais envoyée comme « sans expiration » (revue P1, B3)', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		await fireEvent.input(vue.getByTestId('mcp-server-form-expires'), { target: { value: '10000-01-01' } });
		// témoin : la saisie a bien pris (sans quoi le cas passerait à vide, champ vide)
		expect((vue.getByTestId('mcp-server-form-expires') as HTMLInputElement).value).toBe('10000-01-01');
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		expect(vue.getByTestId('mcp-server-form-error').textContent).toContain('Date d’expiration invalide');
		expect(mockApi.createMcpKey).not.toHaveBeenCalled();
	});

	it('le 31.12.9999 sous un fuseau négatif (au-delà de 9999 en UTC) est refusé à l’écran (revue P2, P2-1)', async () => {
		const tzAvant = process.env.TZ;
		process.env.TZ = 'America/New_York';
		try {
			// témoin : le fuseau a pris, et la fin de journée locale bascule bien en l'an 10000
			expect(new Date(2026, 11, 31, 12).getTimezoneOffset()).toBe(300);
			expect(new Date(9999, 11, 31, 23, 59, 59, 999).toISOString().startsWith('+010000-')).toBe(true);
			const vue = await monter([]);
			await ouvrirEtRemplir(vue);
			await fireEvent.input(vue.getByTestId('mcp-server-form-expires'), { target: { value: '9999-12-31' } });
			expect((vue.getByTestId('mcp-server-form-expires') as HTMLInputElement).value).toBe('9999-12-31');
			await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
			expect(vue.getByTestId('mcp-server-form-error').textContent).toContain('Date d’expiration invalide');
			expect(mockApi.createMcpKey).not.toHaveBeenCalled();
		} finally {
			if (tzAvant === undefined) delete process.env.TZ;
			else process.env.TZ = tzAvant;
		}
	});

	it('(c) double soumission impossible', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		let resoudre!: (v: { cle: McpKey; secret: string }) => void;
		mockApi.createMcpKey.mockReturnValueOnce(new Promise((r) => (resoudre = r)));
		mockApi.listMcpKeys.mockResolvedValue([]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await fireEvent.submit(vue.getByTestId('mcp-server-form'));
		expect(mockApi.createMcpKey).toHaveBeenCalledTimes(1);
		resoudre({ cle: cle(1), secret: SECRET });
		await vue.findByTestId('mcp-server-secret-notice');
	});

	it('(d) succès : liste rechargée par GET (le second), jamais complétée par l’objet de création', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue, 'nom du POST');
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9, { name: 'nom du POST' }), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValueOnce([cle(10, { name: 'nom du GET' })]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-key-row-10');
		expect(mockApi.listMcpKeys).toHaveBeenCalledTimes(2);
		expect(vue.getByTestId('mcp-server-keys-table').textContent).toContain('nom du GET');
		expect(vue.getByTestId('mcp-server-keys-table').textContent).not.toContain('nom du POST');
		expect(vue.queryByTestId('mcp-server-form')).toBeNull();
	});
});

describe('le secret, une seule fois (AC-11)', () => {
	it('(i) le DOM : un seul nœud texte, aucun attribut ; plus rien après Fermer', async () => {
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9, { name: 'agent' }), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValue([cle(9, { name: 'agent' })]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		const notice = await vue.findByTestId('mcp-server-secret-notice');
		expect(notice.getAttribute('role')).toBe('alert');
		expect(notice.textContent).toContain('Clé « agent » créée');
		expect(notice.textContent).toContain('Copiez-la maintenant : elle ne sera plus jamais affichée');
		expect(notice.textContent).toContain('jamais dans un .mcp.json versionné');
		const valeur = vue.getByTestId('mcp-server-secret-value');
		expect(valeur.tagName).toBe('CODE');
		expect(valeur.textContent).toBe(SECRET);
		expect(noeudsTexte(SECRET)).toBe(1);
		expect(document.body.outerHTML.split(SECRET).length - 1).toBe(1);
		await waitFor(() => expect(mockApi.listMcpKeys).toHaveBeenCalledTimes(2));
		// (iii) aucune ligne du tableau ne le contient
		expect(vue.getByTestId('mcp-server-keys-table').textContent).not.toContain(SECRET);
		await fireEvent.click(vue.getByTestId('mcp-server-secret-close'));
		expect(document.body.outerHTML).not.toContain(SECRET);
		expect(vue.queryByTestId('mcp-server-secret-notice')).toBeNull();
	});

	it('(i) une nouvelle soumission oublie le secret AVANT le POST : rien après un 400', async () => {
		const oublier = vi.spyOn(SecretUnique.prototype, 'oublier');
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValue([cle(9)]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-secret-value');
		await ouvrirEtRemplir(vue, 'seconde');
		oublier.mockClear();
		mockApi.createMcpKey.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: 'Refus.' });
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-form-error');
		expect(oublier).toHaveBeenCalled();
		expect(oublier.mock.invocationCallOrder[0]).toBeLessThan(mockApi.createMcpKey.mock.invocationCallOrder[1]);
		expect(document.body.outerHTML).not.toContain(SECRET);
	});

	it('(ii) l’état : Fermer appelle oublier() (sonde de M1)', async () => {
		const oublier = vi.spyOn(SecretUnique.prototype, 'oublier');
		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValue([cle(9)]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-secret-close');
		oublier.mockClear();
		await fireEvent.click(vue.getByTestId('mcp-server-secret-close'));
		expect(oublier).toHaveBeenCalledTimes(1);
	});

	it('(iv) aucun journal ni toast ne reçoit le secret ; document.title non plus', async () => {
		const methodes = ['log', 'info', 'warn', 'error', 'debug', 'trace'] as const;
		const espions = methodes.map((m) => vi.spyOn(console, m).mockImplementation(() => {}));
		const t = toast as unknown as ReturnType<typeof vi.fn> & Record<string, ReturnType<typeof vi.fn>>;
		// l'ensemble des entrées relu au paquet installé (svelte-sonner 1.1.0)
		for (const k of ['custom', 'dismiss', 'error', 'getActiveToasts', 'info', 'loading', 'message', 'promise', 'success', 'warning']) {
			expect(typeof t[k]?.mockClear, k).toBe('function');
		}
		// témoins
		console.info(TEMOIN);
		toast(TEMOIN);
		toast.info(TEMOIN);
		expect(espions[1]).toHaveBeenCalledWith(TEMOIN);
		expect(t).toHaveBeenCalledWith(TEMOIN);
		expect(t.info).toHaveBeenCalledWith(TEMOIN);
		espions.forEach((e) => e.mockClear());
		t.mockClear();
		for (const k of Object.keys(t)) if (typeof t[k]?.mockClear === 'function') t[k].mockClear();

		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValue([cle(9)]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-secret-copy');
		await fireEvent.click(vue.getByTestId('mcp-server-secret-copy'));
		await waitFor(() => expect(t.success).toHaveBeenCalled());
		await fireEvent.click(vue.getByTestId('mcp-server-secret-close'));
		await ouvrirEtRemplir(vue, 'seconde');
		mockApi.createMcpKey.mockRejectedValueOnce(new Error('échec simulé'));
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await vue.findByTestId('mcp-server-form-error');

		const appels: unknown[][] = [
			...espions.flatMap((e) => e.mock.calls),
			...t.mock.calls,
			...Object.keys(t).flatMap((k) => (typeof t[k]?.mock === 'object' ? t[k].mock.calls : [])),
		];
		for (const args of appels) {
			for (const a of args) {
				expect(String(a)).not.toContain(SECRET);
				expect(JSON.stringify(a) ?? '').not.toContain(SECRET);
			}
		}
		expect(document.title).not.toContain(SECRET);
	});

	it('(v) ni stockage, ni indexedDB, ni adresse', async () => {
		const setItem = vi.spyOn(Storage.prototype, 'setItem');
		const idb = { open: vi.fn() };
		vi.stubGlobal('indexedDB', idb);
		const push = vi.spyOn(history, 'pushState');
		const replace = vi.spyOn(history, 'replaceState');
		// témoins
		sessionStorage.setItem('k', TEMOIN);
		expect(setItem).toHaveBeenCalledWith('k', TEMOIN);
		sessionStorage.removeItem('k');
		(globalThis as unknown as { indexedDB: typeof idb }).indexedDB.open('t');
		expect(idb.open).toHaveBeenCalledWith('t');
		const adresse = window.location.pathname;
		history.pushState({}, '', `/${TEMOIN}`);
		expect(push).toHaveBeenCalledWith({}, '', `/${TEMOIN}`);
		history.replaceState({}, '', adresse);
		expect(replace).toHaveBeenCalled();
		setItem.mockClear();
		idb.open.mockClear();
		push.mockClear();
		replace.mockClear();

		const vue = await monter([]);
		await ouvrirEtRemplir(vue);
		mockApi.createMcpKey.mockResolvedValueOnce({ cle: cle(9), secret: SECRET });
		mockApi.listMcpKeys.mockResolvedValue([cle(9)]);
		await fireEvent.click(vue.getByTestId('mcp-server-form-submit'));
		await fireEvent.click(await vue.findByTestId('mcp-server-secret-copy'));
		await fireEvent.click(vue.getByTestId('mcp-server-secret-close'));

		for (const args of setItem.mock.calls) expect(args.join(' ')).not.toContain(SECRET);
		expect(idb.open).not.toHaveBeenCalled();
		for (const args of [...push.mock.calls, ...replace.mock.calls]) {
			expect(String(args[2] ?? '')).not.toContain(SECRET);
		}
		expect(window.location.href).not.toContain(SECRET);
	});
});

describe('révoquer (AC-12)', () => {
	it('confirmation, 204 : message et rechargement, avec la version de la ligne', async () => {
		const vue = await monter([cle(4, { version: 7 })]);
		await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-4'));
		expect(vue.getByTestId('mcp-server-key-revoke-confirm-text-4').textContent).toContain(
			'L’agent qui emploie cette clé perd l’accès immédiatement.',
		);
		mockApi.revokeMcpKey.mockResolvedValueOnce(undefined);
		mockApi.listMcpKeys.mockResolvedValueOnce([cle(4, { version: 8, revokedAt: '2026-10-25T12:00:00.000' })]);
		await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-confirm-4'));
		expect(mockApi.revokeMcpKey).toHaveBeenCalledWith(4, 7);
		expect((await vue.findByTestId('mcp-server-keys-message')).textContent).toContain('Clé révoquée.');
		await waitFor(() => expect(vue.getByTestId('mcp-server-key-status-4').textContent).toContain('Révoquée'));
		// le bouton absent sur une clé révoquée
		expect(vue.queryByTestId('mcp-server-key-revoke-4')).toBeNull();
	});

	it('présent sur une clé expirée ; 404/409 : message et rechargement ; hors ApiError : le générique', async () => {
		const vue = await monter([cle(4, { expiresAt: '2020-01-01T00:00:00.000' })]);
		expect(vue.getByTestId('mcp-server-key-revoke-4')).toBeTruthy();
		for (const status of [404, 409]) {
			await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-4'));
			mockApi.revokeMcpKey.mockRejectedValueOnce({ code: 'X', status, message: 'x' });
			mockApi.listMcpKeys.mockResolvedValueOnce([cle(4, { expiresAt: '2020-01-01T00:00:00.000' })]);
			const avant = mockApi.listMcpKeys.mock.calls.length;
			await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-confirm-4'));
			expect((await vue.findByTestId('mcp-server-keys-error')).textContent).toContain(
				'La clé a changé ou n’existe plus ; liste rechargée.',
			);
			await waitFor(() => expect(mockApi.listMcpKeys.mock.calls.length).toBe(avant + 1));
		}
		vi.spyOn(console, 'error').mockImplementation(() => {});
		await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-4'));
		mockApi.revokeMcpKey.mockRejectedValueOnce(new Error('boom'));
		await fireEvent.click(vue.getByTestId('mcp-server-key-revoke-confirm-4'));
		await waitFor(() =>
			expect(vue.getByTestId('mcp-server-keys-error').textContent?.trim()).toBe('Erreur inattendue.'),
		);
	});
});

describe('la politique de validation (AC-13)', () => {
	it('(a) vers dispensée : confirmation ; (b) succès : la ligne remplacée par la clé rendue', async () => {
		const vue = await monter([cle(4, { version: 2 })]);
		await fireEvent.click(vue.getByTestId('mcp-server-key-policy-toggle-4'));
		expect(vue.getByTestId('mcp-server-key-exempt-confirm-text-4').textContent).toContain(
			'sans vous être montrées',
		);
		expect(mockApi.setMcpKeySendValidation).not.toHaveBeenCalled();
		mockApi.setMcpKeySendValidation.mockResolvedValueOnce(cle(4, { version: 3, sendValidation: 'exempt' }));
		await fireEvent.click(vue.getByTestId('mcp-server-key-exempt-confirm-4'));
		expect(mockApi.setMcpKeySendValidation).toHaveBeenCalledWith(4, 'exempt', 2);
		await waitFor(() => expect(vue.getByTestId('mcp-server-key-policy-4').textContent?.trim()).toBe('Dispensée'));
		expect(mockApi.listMcpKeys).toHaveBeenCalledTimes(1);
	});

	it('(a) vers requise : sans confirmation', async () => {
		const vue = await monter([cle(4, { version: 2, sendValidation: 'exempt' })]);
		mockApi.setMcpKeySendValidation.mockResolvedValueOnce(cle(4, { version: 3, sendValidation: 'required' }));
		await fireEvent.click(vue.getByTestId('mcp-server-key-policy-toggle-4'));
		expect(vue.queryByTestId('mcp-server-key-exempt-confirm-4')).toBeNull();
		expect(mockApi.setMcpKeySendValidation).toHaveBeenCalledWith(4, 'required', 2);
		await waitFor(() => expect(vue.getByTestId('mcp-server-key-policy-4').textContent?.trim()).toBe('Requise'));
	});

	it('(b) 409 ou 404 : message et rechargement ; hors ApiError : le générique', async () => {
		const vue = await monter([cle(4, { sendValidation: 'exempt' })]);
		for (const status of [409, 404]) {
			mockApi.setMcpKeySendValidation.mockRejectedValueOnce({ code: 'X', status, message: 'x' });
			mockApi.listMcpKeys.mockResolvedValueOnce([cle(4, { sendValidation: 'exempt' })]);
			const avant = mockApi.listMcpKeys.mock.calls.length;
			await fireEvent.click(vue.getByTestId('mcp-server-key-policy-toggle-4'));
			expect((await vue.findByTestId('mcp-server-keys-error')).textContent).toContain(
				'La clé a changé ou n’existe plus ; liste rechargée.',
			);
			await waitFor(() => expect(mockApi.listMcpKeys.mock.calls.length).toBe(avant + 1));
		}
		vi.spyOn(console, 'error').mockImplementation(() => {});
		mockApi.setMcpKeySendValidation.mockRejectedValueOnce(new Error('boom'));
		await fireEvent.click(vue.getByTestId('mcp-server-key-policy-toggle-4'));
		await waitFor(() =>
			expect(vue.getByTestId('mcp-server-keys-error').textContent?.trim()).toBe('Erreur inattendue.'),
		);
	});

	it('pas de contrôle sur une clé révoquée ; (c) la note du tableau ; (d) le lien vers les envois', async () => {
		const vue = await monter([cle(4, { revokedAt: '2026-10-25T12:00:00.000' })]);
		expect(vue.queryByTestId('mcp-server-key-policy-toggle-4')).toBeNull();
		expect(vue.getByTestId('mcp-server-keys-policy-note').textContent).toContain('Envois du serveur MCP');
		const lien = vue.getByTestId('mcp-server-sends-link') as HTMLAnchorElement;
		expect(lien.getAttribute('href')).toBe('/admin/mcp/sends');
	});
});
