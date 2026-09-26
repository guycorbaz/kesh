/**
 * L'écran du journal d'audit — Story 25-1c-b1 (#378).
 *
 * ⚠️ Chaque test nomme la MUTATION qu'il attrape. Patron :
 * `contacts/contacts-page.test.ts` — mocks hoistés AVANT l'import du composant.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';
import type { AuditLogEntry, AuditLogVocabulary } from '$lib/features/audit-log/audit-log.types';

vi.mock('$app/environment', () => ({ browser: true }));
// ⚠️ `goto` BOUCLE sur l'URL que la page relit (revue P1) : un mock inerte
// masquerait un effet qui écraserait l'URL avant que `onMount` ne la lise.
const gotoMock = vi.fn((u: URL | string) => {
	url.value = new URL(String(u), 'http://localhost');
});
vi.mock('$app/navigation', () => ({ goto: (u: URL | string) => gotoMock(u) }));
const url = { value: new URL('http://localhost/audit-log') };
vi.mock('$app/state', () => ({
	page: {
		get url() {
			return url.value;
		},
	},
}));

const locale = { value: 'fr-CH' };
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
	i18nLocale: () => locale.value,
}));

const notifyErrorMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: (m: string) => notifyErrorMock(m),
	notifySuccess: vi.fn(),
}));

const listMock = vi.fn();
const vocabMock = vi.fn();
const exportMock = vi.fn();
vi.mock('$lib/features/audit-log/audit-log.api', () => ({
	listAuditLog: (q: unknown) => listMock(q),
	getAuditLogVocabulary: () => vocabMock(),
	exportAuditLogCsv: (q: unknown) => exportMock(q),
}));

import Page from './+page.svelte';
import { authState } from '$lib/app/stores/auth.svelte';

/** Le vocabulaire, rendu dans l'ORDRE DES CODES comme la route le rend. */
const VOCAB: AuditLogVocabulary = {
	entityTypes: [
		{ code: 'contact', label: 'Contact' },
		{ code: 'journal_entry', label: 'Écriture' },
	],
	actions: [
		{ code: 'contact.created', label: 'Contact créé' },
		{ code: 'invoice.validated', label: 'Facture validée' },
		{ code: 'journal_entry.created', label: 'Écriture créée' },
	],
};

function entry(partial: Partial<AuditLogEntry> = {}): AuditLogEntry {
	return {
		id: 1,
		createdAt: '2026-09-16T12:26:33.123Z',
		actorLabel: 'comptable',
		actorType: 'user',
		actorApiKeyId: null,
		userId: 2,
		// ⚠️ Code et libellé DIFFÉRENTS : sinon l'assertion sur la colonne
		// serait vraie par construction.
		action: 'contact.created',
		actionLabel: 'Contact créé (libellé)',
		entityType: 'contact',
		entityTypeLabel: 'Contact (libellé)',
		entityId: 12,
		details: { name: 'Dupont SA' },
		...partial,
	};
}

function page1(items: AuditLogEntry[]) {
	return { items, total: items.length, offset: 0, limit: 50 };
}

beforeEach(() => {
	url.value = new URL('http://localhost/audit-log');
	locale.value = 'fr-CH';
	vocabMock.mockResolvedValue(VOCAB);
	listMock.mockResolvedValue(page1([entry()]));
});

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

describe('la garde', () => {
	afterEach(async () => {
		await authState.logout();
	});

	it('load() redirige un rôle Consultation vers / (mutation : garde retirée)', async () => {
		authState.login({ userId: '1', username: 'c', role: 'Consultation', expiresIn: 3600 });
		const { load } = await import('./+page');
		try {
			load();
			expect.unreachable('redirection attendue');
		} catch (err: unknown) {
			const e = err as { status: number; location: string };
			expect(e.status).toBe(302);
			expect(e.location).toBe('/');
		}
	});

	it('load() laisse passer un Comptable', async () => {
		authState.login({ userId: '1', username: 'c', role: 'Comptable', expiresIn: 3600 });
		const { load } = await import('./+page');
		expect(() => load()).not.toThrow();
	});
});

describe('la page', () => {
	it('affiche actionLabel et entityTypeLabel, jamais les codes (mutation : colonne rendue avec `action`)', async () => {
		const { findByTestId, getByTestId } = render(Page);
		const cell = await findByTestId('audit-log-row-action');
		expect(cell.textContent).toBe('Contact créé (libellé)');
		expect(getByTestId('audit-log-row').textContent).toContain('Contact (libellé)');
		expect(getByTestId('audit-log-row').getAttribute('data-action')).toBe('contact.created');
	});

	it('rend « — » pour un entityId à 0, et déplie le détail en JSON indenté', async () => {
		listMock.mockResolvedValue(page1([entry({ entityId: 0 })]));
		const { findByTestId, getByTestId } = render(Page);
		const row = await findByTestId('audit-log-row');
		expect(row.textContent).toContain('—');
		await fireEvent.click(getByTestId('audit-log-details-toggle'));
		expect(getByTestId('audit-log-details').textContent).toBe(
			JSON.stringify({ name: 'Dupont SA' }, null, 2),
		);
	});

	it('état vide', async () => {
		listMock.mockResolvedValue(page1([]));
		const { findByTestId } = render(Page);
		expect(await findByTestId('audit-log-empty')).toBeTruthy();
	});

	it("⛔ un échec du VOCABULAIRE affiche l'état d'erreur (mutation : échec avalé, filtres vides)", async () => {
		vocabMock.mockRejectedValue({ code: 'X', message: 'vocabulaire indisponible', status: 500 });
		const { findByTestId, queryByTestId } = render(Page);
		expect((await findByTestId('audit-log-error')).textContent).toContain(
			'vocabulaire indisponible',
		);
		expect(queryByTestId('audit-log-filter-action')).toBeNull();
	});

	it("l'erreur de la LISTE laisse les filtres et « Réinitialiser » visibles", async () => {
		listMock.mockRejectedValue({ code: 'VALIDATION_ERROR', message: 'plage invalide', status: 400 });
		const { findByTestId, getByTestId } = render(Page);
		expect((await findByTestId('audit-log-error')).textContent).toContain('plage invalide');
		expect(getByTestId('audit-log-filter-action')).toBeTruthy();
		expect(getByTestId('audit-log-filter-reset')).toBeTruthy();
	});

	it('⛔ la page passe par toSelectOptions : code historique ajouté, options triées par libellé (mutation : vocabulary.actions rendu tel quel)', async () => {
		url.value = new URL('http://localhost/audit-log?action=zz.legacy');
		const { findByTestId } = render(Page);
		const select = (await findByTestId('audit-log-filter-action')) as HTMLSelectElement;
		await waitFor(() => expect(select.options.length).toBe(5));
		expect(select.value).toBe('zz.legacy');
		const labels = Array.from(select.options).map((o) => o.text);
		expect(labels).toContain('zz.legacy');
		// Après « Toutes », le premier PAR LIBELLÉ — « Contact créé » —, puis
		// « Écriture créée » avant « Facture validée » (l'ordre des codes les inverserait).
		expect(labels.slice(1, 4)).toEqual(['Contact créé', 'Écriture créée', 'Facture validée']);
	});

	it("⛔ la page transmet i18nLocale() au tri : sous sv-SE, « Zahlung » avant « Öffnung » (mutation : 'fr-CH' en dur)", async () => {
		locale.value = 'sv-SE';
		vocabMock.mockResolvedValue({
			entityTypes: [],
			actions: [
				{ code: 'a', label: 'Öffnung' },
				{ code: 'b', label: 'Zahlung' },
			],
		});
		const { findByTestId } = render(Page);
		const select = (await findByTestId('audit-log-filter-action')) as HTMLSelectElement;
		await waitFor(() => expect(select.options.length).toBe(3));
		expect(Array.from(select.options).map((o) => o.text).slice(1)).toEqual(['Zahlung', 'Öffnung']);
	});

	it("⛔ la date suit i18nLocale() : sous de-CH, « 16.09.2026 » et pas « sept. » (mutation : navigator.language ou 'fr-CH' en dur)", async () => {
		locale.value = 'de-CH';
		const { findByTestId } = render(Page);
		const row = await findByTestId('audit-log-row');
		expect(row.textContent).toContain('16.09.2026');
		expect(row.textContent).not.toContain('sept.');
	});

	it("⛔ revenir à « Tous » VIDE l'identifiant d'entité (mutation : entityId conservé)", async () => {
		url.value = new URL('http://localhost/audit-log?entityType=contact&entityId=12');
		const { findByTestId, getByTestId } = render(Page);
		const select = (await findByTestId('audit-log-filter-entity-type')) as HTMLSelectElement;
		await waitFor(() => expect(select.value).toBe('contact'));
		expect(listMock.mock.calls[0][0]).toMatchObject({ entityType: 'contact', entityId: 12 });
		select.value = '';
		await fireEvent.change(select);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		const last = listMock.mock.calls[1][0] as Record<string, unknown>;
		expect(last.entityType).toBeUndefined();
		expect(last.entityId).toBeUndefined();
		// ⚠️ La requête seule ne suffit pas : `buildQuery` n'envoie déjà jamais
		// un identifiant sans type. C'est le CHAMP qui doit être vidé — sinon
		// l'ancien identifiant revient dès qu'on rechoisit un type.
		const input = getByTestId('audit-log-filter-entity-id') as HTMLInputElement;
		expect(input.value).toBe('');
		select.value = 'contact';
		await fireEvent.change(select);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(3));
		expect((listMock.mock.calls[2][0] as Record<string, unknown>).entityId).toBeUndefined();
	});

	it("⛔ l'export transmet les filtres affichés (mutation : export appelé avec {})", async () => {
		url.value = new URL('http://localhost/audit-log?action=contact.created&dateFrom=2026-09-01');
		exportMock.mockResolvedValue(undefined);
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('audit-log-export'));
		await waitFor(() => expect(exportMock).toHaveBeenCalledOnce());
		expect(exportMock.mock.calls[0][0]).toMatchObject({
			action: 'contact.created',
			dateFrom: '2026-09-01',
		});
	});

	it('un refus RESULT_TOO_LARGE affiche le message du serveur dans la page, sans notifyError', async () => {
		exportMock.mockRejectedValue({
			code: 'RESULT_TOO_LARGE',
			message: 'Trop de résultats (> 10000).',
			status: 400,
		});
		const { findByTestId } = render(Page);
		await fireEvent.click(await findByTestId('audit-log-export'));
		expect((await findByTestId('audit-log-export-error')).textContent).toContain(
			'Trop de résultats',
		);
		expect(notifyErrorMock).not.toHaveBeenCalled();
	});
});

describe('revue P1', () => {
	it("⛔ passer d'un type à un AUTRE vide aussi l'identifiant (mutation : vidé seulement au retour à « Tous »)", async () => {
		url.value = new URL('http://localhost/audit-log?entityType=contact&entityId=12');
		const { findByTestId, getByTestId } = render(Page);
		const select = (await findByTestId('audit-log-filter-entity-type')) as HTMLSelectElement;
		await waitFor(() => expect(select.value).toBe('contact'));
		select.value = 'journal_entry';
		await fireEvent.change(select);
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		expect(listMock.mock.calls[1][0]).toMatchObject({ entityType: 'journal_entry' });
		expect((listMock.mock.calls[1][0] as Record<string, unknown>).entityId).toBeUndefined();
		expect((getByTestId('audit-log-filter-entity-id') as HTMLInputElement).value).toBe('');
	});

	it('⛔ une réponse PÉRIMÉE ne remplace pas la plus récente (mutation : jeton de requête retiré)', async () => {
		let resolveFirst!: (v: unknown) => void;
		listMock
			.mockImplementationOnce(() => new Promise((r) => (resolveFirst = r)))
			.mockResolvedValueOnce(page1([entry({ id: 2, actionLabel: 'RÉCENTE' })]));
		const { findByTestId, getByTestId } = render(Page);
		const select = (await findByTestId('audit-log-filter-action')) as HTMLSelectElement;
		await waitFor(() => expect(select.options.length).toBe(4));
		select.value = 'contact.created';
		await fireEvent.change(select);
		expect((await findByTestId('audit-log-row-action')).textContent).toBe('RÉCENTE');
		resolveFirst(page1([entry({ id: 1, actionLabel: 'PÉRIMÉE' })]));
		await new Promise((r) => setTimeout(r, 20));
		expect(getByTestId('audit-log-row-action').textContent).toBe('RÉCENTE');
	});

	it("⛔ une plage inversée est refusée DANS LA PAGE, traduite, sans appel à la route", async () => {
		url.value = new URL('http://localhost/audit-log?dateFrom=2026-09-20&dateTo=2026-09-01');
		const { findByTestId } = render(Page);
		expect((await findByTestId('audit-log-error')).textContent).toContain(
			'La date de début doit précéder',
		);
		expect(listMock).not.toHaveBeenCalled();
	});

	it('⛔ une page vide au-delà de la fin garde un « Précédent » (mutation : pagination absente de l’état vide)', async () => {
		url.value = new URL('http://localhost/audit-log?offset=500');
		listMock.mockResolvedValue({ items: [], total: 3, offset: 500, limit: 50 });
		const { findByTestId } = render(Page);
		await findByTestId('audit-log-empty');
		await fireEvent.click(await findByTestId('audit-log-prev'));
		await waitFor(() => expect(listMock).toHaveBeenCalledTimes(2));
		expect((listMock.mock.calls[1][0] as Record<string, unknown>).offset).toBe(450);
	});

	it("⛔ l'URL de départ n'est pas écrasée au montage (mutation : effet d'URL avant la lecture)", async () => {
		url.value = new URL('http://localhost/audit-log?action=contact.created&entityType=contact&entityId=12');
		render(Page);
		await waitFor(() => expect(listMock).toHaveBeenCalled());
		expect(listMock.mock.calls[0][0]).toMatchObject({
			action: 'contact.created',
			entityType: 'contact',
			entityId: 12,
		});
		expect(url.value.searchParams.get('action')).toBe('contact.created');
		expect(url.value.searchParams.get('entityId')).toBe('12');
	});
});
