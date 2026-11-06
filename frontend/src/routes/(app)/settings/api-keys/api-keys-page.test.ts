// Story 28-2 (#520, AC9 — B6) — premier test de la page des clés d'API : un
// rejet du chargement de la liste qui n'est pas une `ApiError` affiche le
// générique traduit, ni le message technique ni « [object Object] ».
//
// Le nom du fichier n'est pas `+page.test.ts` : SvelteKit réserve le préfixe `+`.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';

// Revue P1 E-L5 : création et révocation pilotées pour les cas B7/B8.
const { listApiKeysMock, createApiKeyMock, revokeApiKeyMock } = vi.hoisted(() => ({
	listApiKeysMock: vi.fn(),
	createApiKeyMock: vi.fn(),
	revokeApiKeyMock: vi.fn(),
}));

// v0172-correctifs (#880) : le simulacre interpole les variables (les dates des statuts).
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));
vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }));
vi.mock('$lib/features/api-keys/api-keys.api', () => ({
	listApiKeys: () => listApiKeysMock(),
	createApiKey: (p: unknown) => createApiKeyMock(p),
	revokeApiKey: (id: number, v: number) => revokeApiKeyMock(id, v),
}));

import { toast } from 'svelte-sonner';
import Page from './+page.svelte';
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';

// Story v0174-correctifs-3 (#889, AC-3.6, site 14) — à 00 h 30 le 5 novembre à Zurich,
// l'expiration minimale d'une clé est le lendemain du jour du poste : le 6.
describe('page des clés d’API — expiration minimale locale (#889)', () => {
	pinClock();
	it('le minimum du champ d’expiration est le 6 novembre', async () => {
		listApiKeysMock.mockResolvedValue([]);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('api-keys-create-button'));
		expect((await findByTestId('api-keys-expires-input') as HTMLInputElement).min).toBe(
			'2026-11-06',
		);
	});
});

describe('page des clés d’API — échec du chargement (Story 28-2, B6)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
	});

	it('rejet non-ApiError → le générique', async () => {
		listApiKeysMock.mockRejectedValue(new Error('boom'));
		const { findByTestId } = render(Page);
		const erreur = await findByTestId('api-keys-load-error');
		expect(erreur.textContent).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
		expect(erreur.textContent).not.toContain('[object Object]');
	});

	it('rejet ApiError → son message', async () => {
		listApiKeysMock.mockRejectedValue({ code: 'FORBIDDEN', message: 'Accès refusé.', status: 403 });
		const { findByTestId } = render(Page);
		expect((await findByTestId('api-keys-load-error')).textContent).toBe('Accès refusé.');
	});
});

// Story 28-2 (revue P1 E-L5 — B7, B8) — création et révocation rejetées hors ApiError.
describe('page des clés d’API — échec imprévu d’une écriture (Story 28-2, B7/B8)', () => {
	const cle = {
		id: 7,
		name: 'CI',
		scope: 'read',
		createdAt: '2026-01-01T00:00:00',
		lastUsedAt: null,
		revokedAt: null,
		expiresAt: null,
		version: 3,
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.spyOn(console, 'error').mockImplementation(() => {});
		listApiKeysMock.mockResolvedValue([cle]);
	});

	it('B7 — création rejetée hors ApiError → le générique dans le formulaire', async () => {
		createApiKeyMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('api-keys-create-button'));
		await fireEvent.input(await r.findByTestId('api-keys-name-input'), { target: { value: 'Export' } });
		await fireEvent.click(await r.findByTestId('api-keys-submit'));
		await waitFor(() => expect(createApiKeyMock).toHaveBeenCalledTimes(1));
		const erreur = await r.findByTestId('api-keys-form-error');
		expect(erreur.textContent?.trim()).toBe('Erreur inattendue.');
		expect(erreur.textContent).not.toContain('boom');
	});

	it('B8 — révocation rejetée hors ApiError → le générique en toast', async () => {
		revokeApiKeyMock.mockRejectedValue(new Error('boom'));
		const r = render(Page);
		await fireEvent.click(await r.findByTestId('api-keys-revoke-7'));
		await fireEvent.click(await r.findByTestId('api-keys-revoke-confirm-button'));
		await waitFor(() => expect(revokeApiKeyMock).toHaveBeenCalledWith(7, 3));
		await waitFor(() => expect(toast.error).toHaveBeenCalledWith('Erreur inattendue.'));
	});
});

// v0172-correctifs (#880, AC-C2) — les dates des clés en forme suisse. `createdAt`,
// `revokedAt`, `lastUsedAt` sont des instants UTC (`NOW(3)`), affichés à l'heure de Zurich ;
// l'expiration est une DATE DE CALENDRIER (envoyée `T23:59:59Z`), affichée sans conversion,
// et `isExpired` la lit en UTC, comme le serveur. Chaque cas est joué sous deux fuseaux de
// processus : le résultat ne doit dépendre que de `Europe/Zurich`.
describe.each(['Europe/Zurich', 'UTC'])('page des clés d’API — dates (v0172, #880 ; TZ=%s)', (zone) => {
	let tz: string | undefined;
	beforeEach(() => {
		vi.clearAllMocks();
		tz = process.env.TZ;
		process.env.TZ = zone;
		vi.useFakeTimers({ toFake: ['Date'] });
	});
	afterEach(() => {
		vi.useRealTimers();
		if (tz === undefined) delete process.env.TZ;
		else process.env.TZ = tz;
	});

	function cle(o: Record<string, unknown>) {
		return {
			id: 7,
			name: 'CI',
			scope: 'read',
			createdAt: '2026-01-01T23:30:00',
			lastUsedAt: null,
			revokedAt: null,
			expiresAt: null,
			version: 3,
			...o,
		};
	}
	async function statut(o: Record<string, unknown>): Promise<string> {
		listApiKeysMock.mockResolvedValue([cle(o)]);
		const r = render(Page);
		return (await r.findByTestId('api-key-status-7')).textContent?.trim() ?? '';
	}

	it('création 02.01.2026 (UTC 23:30 ⇒ Zurich), dernière utilisation 02.07.2026 00:30 ; jamais l’ISO (MC2)', async () => {
		vi.setSystemTime(new Date('2026-08-01T12:00:00Z'));
		listApiKeysMock.mockResolvedValue([cle({ lastUsedAt: '2026-07-01T22:30:00.123' })]);
		const r = render(Page);
		const row = await r.findByTestId('api-key-row-7');
		const cells = Array.from(row.querySelectorAll('td')).map((td) => td.textContent?.trim());
		expect(cells[2]).toBe('02.01.2026');
		expect(cells[3]).toBe('02.07.2026 00:30');
		expect(row.textContent).not.toMatch(/2026-\d{2}-\d{2}/);
	});

	it('révoquée : « Révoquée le 04.03.2026 »', async () => {
		vi.setSystemTime(new Date('2026-08-01T12:00:00Z'));
		expect(await statut({ revokedAt: '2026-03-04T10:00:00' })).toBe('Révoquée le 04.03.2026');
	});

	it('expiration au 31.12.2026, active : « Active (expire le 31.12.2026) » — jamais 01.01.2027 (MC5)', async () => {
		vi.setSystemTime(new Date('2026-12-01T12:00:00Z'));
		expect(await statut({ expiresAt: '2026-12-31T23:59:59' })).toBe('Active (expire le 31.12.2026)');
	});

	it('expiration au 31.12.2026, expirée : « Expirée le 31.12.2026 » (MC5)', async () => {
		vi.setSystemTime(new Date('2027-01-15T12:00:00Z'));
		expect(await statut({ expiresAt: '2026-12-31T23:59:59' })).toBe('Expirée le 31.12.2026');
	});

	it('cas de bord, 2026-12-31T23:30Z (00:30 le 01.01 à Zurich) : encore active jusqu’à 23:59:59 UTC, comme au serveur (MC6)', async () => {
		vi.setSystemTime(new Date('2026-12-31T23:30:00Z'));
		expect(await statut({ expiresAt: '2026-12-31T23:59:59' })).toBe('Active (expire le 31.12.2026)');
	});
});
