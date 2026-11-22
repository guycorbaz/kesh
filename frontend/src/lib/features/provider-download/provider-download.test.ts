// Story 38-12b (AC-2, AC-3) — règles du dialogue générique de téléchargement :
// période proposée, refus du lancement (reprise sur 409), messages d'échec,
// progression, politique de suivi.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : fallback,
}));

import {
	MAX_POLL_ERRORS,
	afterPollError,
	defaultPeriod,
	failureLines,
	progressLabel,
	progressPercent,
	resumedNotice,
	startRefusal,
} from './provider-download';
import { downloadsRoute } from './provider-download.api';

const apiError = (code: string, status: number, details?: Record<string, unknown>) => ({
	code,
	message: `message du serveur ${code}`,
	status,
	details,
});

describe('defaultPeriod', () => {
	it('l’exercice choisi, sa fin ramenée à aujourd’hui', () => {
		expect(defaultPeriod({ startDate: '2026-01-01', endDate: '2026-12-31' }, '2026-11-22')).toEqual({
			from: '2026-01-01',
			to: '2026-11-22',
		});
	});
	it('un exercice passé entier', () => {
		expect(defaultPeriod({ startDate: '2025-01-01', endDate: '2025-12-31' }, '2026-11-22')).toEqual({
			from: '2025-01-01',
			to: '2025-12-31',
		});
	});
	it('un exercice futur ou aucun : le jour même', () => {
		expect(defaultPeriod({ startDate: '2027-01-01', endDate: '2027-12-31' }, '2026-11-22')).toEqual({
			from: '2026-11-22',
			to: '2026-11-22',
		});
		expect(defaultPeriod(null, '2026-11-22')).toEqual({ from: '2026-11-22', to: '2026-11-22' });
	});
});

describe('resumedNotice', () => {
	const requested = { from: '2026-02-01', to: '2026-02-28' };
	const today = '2026-11-22';
	it('période différente : celle de la tâche suivie', () => {
		expect(resumedNotice({ from: '2026-01-01', to: '2026-01-30' }, requested, today)).toEqual({
			from: '2026-01-01',
			to: '2026-01-30',
		});
		expect(resumedNotice({ from: '2026-02-01', to: '2026-02-27' }, requested, today)).toEqual({
			from: '2026-02-01',
			to: '2026-02-27',
		});
	});
	it('même période, ou période inconnue : rien', () => {
		expect(resumedNotice({ ...requested }, requested, today)).toBeNull();
		expect(resumedNotice(null, requested, today)).toBeNull();
	});
	it('fin demandée future : ramenée à aujourd’hui avant la comparaison, comme au serveur (revue P2, L1)', () => {
		const exercice = { from: '2026-01-01', to: '2026-12-31' };
		expect(resumedNotice({ from: '2026-01-01', to: today }, exercice, today)).toBeNull();
		// Une tâche qui finit avant aujourd'hui reste une autre période.
		expect(resumedNotice({ from: '2026-01-01', to: '2026-11-21' }, exercice, today)).toEqual({
			from: '2026-01-01',
			to: '2026-11-21',
		});
	});
});

describe('startRefusal', () => {
	it('409 PROVIDER_DOWNLOAD_RUNNING : reprendre le suivi de la tâche en cours', () => {
		expect(startRefusal(apiError('PROVIDER_DOWNLOAD_RUNNING', 409, { provider: 'paypal', jobId: 7 }))).toEqual({
			resumeJobId: 7,
			running: null,
		});
	});
	it('409 avec la période de la tâche en cours : elle est rendue (revue P1, M1)', () => {
		expect(
			startRefusal(
				apiError('PROVIDER_DOWNLOAD_RUNNING', 409, { provider: 'paypal', jobId: 7, from: '2026-01-01', to: '2026-01-30' }),
			),
		).toEqual({ resumeJobId: 7, running: { from: '2026-01-01', to: '2026-01-30' } });
		// Mal formée : ignorée, le suivi reprend sans avis.
		expect(
			startRefusal(apiError('PROVIDER_DOWNLOAD_RUNNING', 409, { jobId: 7, from: '01.01.2026', to: '2026-01-30' })),
		).toEqual({ resumeJobId: 7, running: null });
	});
	it('409 sans identifiant lisible : le message du serveur', () => {
		expect(startRefusal(apiError('PROVIDER_DOWNLOAD_RUNNING', 409, { jobId: 'x' }))).toEqual({
			message: 'message du serveur PROVIDER_DOWNLOAD_RUNNING',
		});
	});
	it('période refusée, par motif', () => {
		const m = (reason: string) =>
			(startRefusal(apiError('PROVIDER_PERIOD_INVALID', 400, { reason })) as { message: string }).message;
		expect(m('from_after_to')).toContain('précéder la date de fin');
		expect(m('future')).toContain('dans le futur');
		expect(m('autre')).toContain('Période refusée');
	});
	it('refus too_old : la profondeur d\'historique du prestataire, jamais un nombre faux', () => {
		const m = (details: Record<string, unknown>) =>
			(startRefusal(apiError('PROVIDER_PERIOD_INVALID', 400, details)) as { message: string }).message;
		expect(m({ provider: 'paypal', reason: 'too_old', historyYears: 3 })).toContain('plus de 3 ans');
		expect(m({ provider: 'stripe', reason: 'too_old', historyYears: 30 })).toContain('plus de 30 ans');
		// Sans profondeur lisible : le texte général, sans nombre.
		const sans = m({ provider: 'stripe', reason: 'too_old' });
		expect(sans).toContain('Période refusée');
		expect(sans).not.toMatch(/\d/);
	});
	it('non configuré : renvoi au manuel ; autre refus : le message du serveur', () => {
		expect((startRefusal(apiError('PROVIDER_API_NOT_CONFIGURED', 422)) as { message: string }).message).toContain(
			'manuel d’administration',
		);
		expect(startRefusal(apiError('PROVIDER_ACCOUNT_NOT_FOUND', 404))).toEqual({
			message: 'message du serveur PROVIDER_ACCOUNT_NOT_FOUND',
		});
	});
});

describe('failureLines', () => {
	const fin = 'Rien n’a été enregistré : relancez le téléchargement.';
	it('limitation de débit : motif, fenêtre et page, rien d’enregistré', () => {
		expect(
			failureLines({
				code: 'PROVIDER_RATE_LIMITED',
				message: 'x',
				details: { provider: 'paypal', window: '2026-01-01…2026-01-30', page: 2 },
			}),
		).toEqual([
			'Le prestataire limite le nombre de demandes, même après six essais : réessayez plus tard.',
			'Période 2026-01-01…2026-01-30, page 2.',
			fin,
		]);
	});
	it('devises sans compte : nommées', () => {
		const lines = failureLines({
			code: 'PROVIDER_CURRENCY_ACCOUNT_MISSING',
			message: 'x',
			details: { provider: 'paypal', currencies: ['GBP', 'JPY'] },
		});
		expect(lines[0]).toContain('GBP, JPY');
		expect(lines.at(-1)).toBe(fin);
	});
	it('réponse refusée : lignes libellées par le prestataire, identifiant répété', () => {
		const lines = failureLines(
			{
				code: 'PROVIDER_RESPONSE_INVALID',
				message: 'x',
				details: {
					provider: 'paypal',
					lines: [{ line: 3, field: 'transaction_status', code: 'STATUS', value: 'X' }],
				},
			},
			(code) => `libellé ${code}`,
		);
		expect(lines).toEqual([
			'La réponse du prestataire n’a pas pu être lue.',
			'Transaction 3, transaction_status : libellé STATUS',
			fin,
		]);
		expect(
			failureLines({
				code: 'PROVIDER_RESPONSE_INVALID',
				message: 'x',
				details: { provider: 'paypal', reason: 'duplicate_transaction', transactionId: '0TESTA' },
			})[1],
		).toContain('0TESTA');
	});
	it('chaque code connu a son motif ; un code inconnu garde le message du serveur', () => {
		for (const code of ['PROVIDER_AUTH_FAILED', 'PROVIDER_PERMISSION_DENIED', 'PROVIDER_UNAVAILABLE', 'PROVIDER_API_ERROR']) {
			const [first] = failureLines({ code, message: 'serveur', details: { provider: 'paypal' } });
			expect(first).not.toBe('serveur');
		}
		expect(failureLines({ code: 'INTERNAL_ERROR', message: 'Erreur interne du serveur' })).toEqual([
			'Erreur interne du serveur',
			fin,
		]);
		expect(failureLines({ code: 'INTERNAL_ERROR', message: ' ' })[0]).toBe('Le téléchargement a échoué.');
	});
});

describe('progression', () => {
	it('« fenêtre 5/13 », barre en pour cent ; total inconnu : compteur sans barre', () => {
		expect(progressLabel({ done: 5, total: 13 }, 'fenêtre')).toBe('fenêtre 5/13');
		expect(progressPercent({ done: 5, total: 13 })).toBe(38);
		expect(progressLabel({ done: 5, total: null }, 'fenêtre')).toBe('fenêtre 5');
		expect(progressPercent({ done: 5, total: null })).toBeNull();
		expect(progressPercent({ done: 0, total: 0 })).toBe(100);
		expect(progressPercent({ done: 14, total: 13 })).toBe(100);
	});
});

describe('afterPollError', () => {
	it('404 : suivi perdu d’emblée ; erreur réseau : perdu au-delà du plafond', () => {
		expect(afterPollError(apiError('NOT_FOUND', 404), 1)).toBe('lost');
		expect(afterPollError(new TypeError('Failed to fetch'), 1)).toBe('retry');
		expect(afterPollError(new TypeError('Failed to fetch'), MAX_POLL_ERRORS - 1)).toBe('retry');
		expect(afterPollError(new TypeError('Failed to fetch'), MAX_POLL_ERRORS)).toBe('lost');
	});
});

describe('downloadsRoute', () => {
	it('la route d’un prestataire', () => {
		expect(downloadsRoute('paypal')).toBe('/api/v1/paypal-downloads');
	});
});
