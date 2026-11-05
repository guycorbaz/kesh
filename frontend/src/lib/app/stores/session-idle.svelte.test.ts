// Story v0172-session (0.17.3, #876, CR #891) — le suivi d'activité de la
// session dans le navigateur (AC-6). Horloge et minuteries simulées
// (`vi.useFakeTimers()`), `localStorage` de jsdom vidé à chaque cas,
// `apiClient` simulé, `fetch` de la déconnexion simulé.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

const postMock = vi.fn();
const getMock = vi.fn();
vi.mock('$lib/shared/utils/api-client', () => ({
	apiClient: {
		post: (...a: unknown[]) => postMock(...a),
		get: (...a: unknown[]) => getMock(...a),
	},
}));
const clearSessionMock = vi.fn();
vi.mock('$lib/app/stores/auth.svelte', () => ({
	authState: { clearSession: (...a: unknown[]) => clearSessionMock(...a) },
}));

import {
	sessionIdle,
	KEY_LAST_ACTIVITY,
	KEY_REPORTED_ACTIVITY,
	KEY_HEARTBEAT_AT,
} from './session-idle.svelte';

const N = 2;
const W = N * 60_000;
let replaceMock: ReturnType<typeof vi.fn>;
let fetchMock: ReturnType<typeof vi.fn>;
const originalLocation = window.location;

function fire(type: string, init: Record<string, unknown> = {}, target: EventTarget = window) {
	const e = new Event(type, { bubbles: true });
	Object.assign(e, init);
	target.dispatchEvent(e);
}
function move(x: number, y: number) {
	fire('pointermove', { screenX: x, screenY: y });
}
function key(k: string): number | null {
	const raw = localStorage.getItem(k);
	return raw === null ? null : Number(raw);
}
/** Avance l'horloge et laisse les promesses se résoudre. */
async function advance(ms: number) {
	await vi.advanceTimersByTimeAsync(ms);
}
function startAtLogin() {
	sessionIdle.start({ idleTimeoutMinutes: N, idleExpiresIn: N * 60 });
}

beforeEach(() => {
	vi.useFakeTimers();
	vi.setSystemTime(0);
	localStorage.clear();
	// Un `<iframe>` focalisé par un cas précédent fausserait `activeElement`.
	document.body.innerHTML = '';
	postMock.mockReset().mockResolvedValue({ idleTimeoutMinutes: N, idleExpiresIn: N * 60 });
	getMock.mockReset();
	clearSessionMock.mockReset();
	replaceMock = vi.fn();
	Object.defineProperty(window, 'location', {
		value: { ...originalLocation, replace: replaceMock },
		writable: true,
		configurable: true,
	});
	fetchMock = vi.fn().mockResolvedValue({ ok: true, status: 204 });
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	sessionIdle.stop();
	vi.useRealTimers();
	vi.unstubAllGlobals();
	vi.restoreAllMocks();
	Object.defineProperty(window, 'location', {
		value: originalLocation,
		writable: true,
		configurable: true,
	});
});

describe('session-idle — ce qui remet à zéro (1)', () => {
	const cases: Array<[string, () => void]> = [
		['keydown', () => fire('keydown')],
		['pointerdown', () => fire('pointerdown')],
		['pointermove (coordonnées neuves)', () => move(10, 20)],
		['wheel', () => fire('wheel')],
		['touchstart', () => fire('touchstart')],
		[
			'visibilitychange vers visible',
			() => {
				Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
				fire('visibilitychange', {}, document);
			},
		],
		['focus de la fenêtre', () => window.dispatchEvent(new FocusEvent('focus'))],
		[
			'blur de la fenêtre vers un <iframe>',
			() => {
				const f = document.createElement('iframe');
				document.body.appendChild(f);
				f.focus();
				vi.spyOn(document, 'activeElement', 'get').mockReturnValue(f);
				window.dispatchEvent(new FocusEvent('blur'));
			},
		],
	];
	for (const [nom, geste] of cases) {
		it(`${nom} : 90 s, geste, 90 s ⇒ session ouverte`, async () => {
			startAtLogin();
			await advance(90_000);
			geste();
			await advance(90_000);
			expect(clearSessionMock).not.toHaveBeenCalled();
			expect(replaceMock).not.toHaveBeenCalled();
		});
	}
});

describe('session-idle — ce qui ne compte pas (2)', () => {
	it('un second pointermove aux mêmes coordonnées ne compte pas', async () => {
		startAtLogin();
		await advance(10_000);
		move(5, 5);
		const posee = key(KEY_LAST_ACTIVITY);
		await advance(10_000);
		move(5, 5);
		expect(key(KEY_LAST_ACTIVITY)).toBe(posee);
	});
	it('un blur sans <iframe> focalisé, un scroll, un focus d’élément ne comptent pas', async () => {
		startAtLogin();
		const avant = key(KEY_LAST_ACTIVITY);
		await advance(5_000);
		window.dispatchEvent(new FocusEvent('blur'));
		const div = document.createElement('div');
		document.body.appendChild(div);
		div.dispatchEvent(new Event('scroll'));
		window.dispatchEvent(new Event('scroll'));
		const input = document.createElement('input');
		document.body.appendChild(input);
		input.focus();
		expect(key(KEY_LAST_ACTIVITY)).toBe(avant);
	});
});

describe('session-idle — l’échéance (3, 3 bis, 4)', () => {
	it('ferme à N exactement, après la déconnexion attendue', async () => {
		let resoudre: (v: unknown) => void = () => {};
		fetchMock.mockReturnValue(new Promise((r) => (resoudre = r)));
		startAtLogin();
		await advance(W - 1_000);
		expect(fetchMock).not.toHaveBeenCalled();
		expect(clearSessionMock).not.toHaveBeenCalled();
		await advance(1_000);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
		expect(url).toBe('/api/v1/auth/logout');
		expect(init.method).toBe('POST');
		expect(init.credentials).toBe('include');
		expect(init.signal).toBeInstanceOf(AbortSignal);
		expect(clearSessionMock).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();
		resoudre({ ok: true, status: 204 });
		await advance(0);
		expect(clearSessionMock).toHaveBeenCalledWith({ broadcast: false });
		expect(replaceMock).toHaveBeenCalledWith('/login?reason=idle&minutes=2');
	});
	it('une déconnexion qui ne répond pas est abandonnée à 2 s, puis la redirection a lieu', async () => {
		let signal: AbortSignal | undefined;
		fetchMock.mockImplementation((_u: string, init: RequestInit) => {
			signal = init.signal ?? undefined;
			return new Promise(() => {});
		});
		startAtLogin();
		await advance(W);
		expect(replaceMock).not.toHaveBeenCalled();
		await advance(1_999);
		expect(replaceMock).not.toHaveBeenCalled();
		await advance(1);
		expect(signal?.aborted).toBe(true);
		expect(replaceMock).toHaveBeenCalledWith('/login?reason=idle&minutes=2');
	});
	it('une activité après l’échéance ferme et ne réécrit pas lastActivity', async () => {
		startAtLogin();
		const avant = key(KEY_LAST_ACTIVITY);
		// Minuterie arrêtée (onglet en arrière-plan, veille) : seul l'événement
		// observe l'échéance ; les écouteurs restent posés.
		vi.clearAllTimers();
		vi.setSystemTime(W + 500);
		fire('keydown');
		await advance(0);
		expect(key(KEY_LAST_ACTIVITY)).toBe(avant);
		expect(fetchMock).toHaveBeenCalledWith('/api/v1/auth/logout', expect.anything());
	});
});

describe('session-idle — le battement (5, 5 bis, 8, 16)', () => {
	it('mouvements continus pendant 60 s ⇒ au plus 5 battements ; sans activité, aucun', async () => {
		startAtLogin();
		await advance(30_000);
		expect(postMock).not.toHaveBeenCalled();
		let x = 0;
		for (let t = 0; t < 600; t++) {
			move(x++, 0);
			await advance(100);
		}
		expect(postMock.mock.calls.length).toBeGreaterThan(0);
		expect(postMock.mock.calls.length).toBeLessThanOrEqual(5);
	});
	it('idleMs exact sur la grille d’une seconde', async () => {
		startAtLogin();
		expect(key(KEY_LAST_ACTIVITY)).toBe(0);
		await advance(1_500);
		fire('keydown');
		await advance(500);
		expect(postMock).toHaveBeenCalledTimes(1);
		expect(postMock.mock.calls[0]).toEqual(['/api/v1/auth/activity', { idleMs: 500 }]);
		expect(key(KEY_HEARTBEAT_AT)).toBe(2_000);
		await advance(8_000);
		fire('keydown');
		await advance(6_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(2);
		expect(postMock.mock.calls[1]).toEqual(['/api/v1/auth/activity', { idleMs: 7_000 }]);
	});
	it('un échec du battement ne ferme rien et se retente au pas de 15 s suivant', async () => {
		postMock.mockRejectedValueOnce({ code: 'NETWORK_ERROR' });
		startAtLogin();
		await advance(1_000);
		fire('keydown');
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		const hb = key(KEY_HEARTBEAT_AT);
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		expect(key(KEY_HEARTBEAT_AT)).toBe(hb);
		await advance(13_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(2);
		expect(clearSessionMock).not.toHaveBeenCalled();
	});
});

describe('session-idle — un onglet vaut pour tous (6, 7)', () => {
	it('lastActivity écrite par un autre onglet repousse l’échéance ; heartbeatAt récent ⇒ pas de second battement', async () => {
		startAtLogin();
		await advance(100_000);
		localStorage.setItem(KEY_LAST_ACTIVITY, String(Date.now()));
		localStorage.setItem(KEY_HEARTBEAT_AT, String(Date.now()));
		await advance(1_000);
		expect(postMock).not.toHaveBeenCalled();
		await advance(60_000);
		expect(replaceMock).not.toHaveBeenCalled();
	});
	it('la réponse du battement fixe N (5 minutes)', async () => {
		postMock.mockResolvedValue({ idleTimeoutMinutes: 5, idleExpiresIn: 300 });
		startAtLogin();
		await advance(1_000);
		fire('keydown');
		await advance(1_000);
		expect(sessionIdle.minutes).toBe(5);
		await advance(4 * 60_000);
		expect(replaceMock).not.toHaveBeenCalled();
		await advance(60_000);
		expect(replaceMock).toHaveBeenCalledWith('/login?reason=idle&minutes=5');
	});
});

describe('session-idle — abaissement depuis l’écran (AC-8)', () => {
	it('setMinutes(1) : l’échéance de l’onglet suit aussitôt la nouvelle valeur', async () => {
		startAtLogin();
		await advance(30_000);
		sessionIdle.setMinutes(1);
		expect(sessionIdle.minutes).toBe(1);
		await advance(29_000);
		expect(fetchMock).not.toHaveBeenCalled();
		await advance(1_000);
		expect(fetchMock).toHaveBeenCalledTimes(1);
		await advance(0);
		expect(replaceMock).toHaveBeenCalledWith('/login?reason=idle&minutes=1');
	});
	it('setMinutes ignore une valeur hors bornes', () => {
		startAtLogin();
		sessionIdle.setMinutes(0);
		sessionIdle.setMinutes(1441);
		sessionIdle.setMinutes(1.5);
		expect(sessionIdle.minutes).toBe(N);
	});
});

describe('session-idle — sondes, arrêt (9, 10, 13)', () => {
	it('une requête de fond ne modifie pas lastActivity', async () => {
		startAtLogin();
		const avant = key(KEY_LAST_ACTIVITY);
		setInterval(() => void getMock('/api/v1/admin/backup-schedule'), 1_000);
		await advance(5_000);
		expect(key(KEY_LAST_ACTIVITY)).toBe(avant);
	});
	it('stop() retire les écouteurs et la minuterie', async () => {
		startAtLogin();
		sessionIdle.stop();
		sessionIdle.stop();
		await advance(5_000);
		fire('keydown');
		expect(key(KEY_LAST_ACTIVITY)).toBe(0);
		await advance(W * 2);
		expect(replaceMock).not.toHaveBeenCalled();
	});
	it('dragover, drop et input comptent', async () => {
		startAtLogin();
		const input = document.createElement('input');
		document.body.appendChild(input);
		for (const [i, geste] of [
			() => fire('dragover'),
			() => fire('drop'),
			() => input.dispatchEvent(new Event('input', { bubbles: true })),
		].entries()) {
			await advance(5_000);
			geste();
			expect(key(KEY_LAST_ACTIVITY)).toBe((i + 1) * 5_000);
		}
	});
});

describe('session-idle — la conciliation (11, 12, 12 bis, 17, 18)', () => {
	it('idleExpiresIn = 30 avec N = 2 ⇒ lastActivity = now − 90 s, fermeture à 30 s', async () => {
		vi.setSystemTime(1_000_000);
		sessionIdle.start({ idleTimeoutMinutes: N, idleExpiresIn: 30 });
		expect(key(KEY_LAST_ACTIVITY)).toBe(1_000_000 - 90_000);
		await advance(29_000);
		expect(fetchMock).not.toHaveBeenCalled();
		await advance(1_000);
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
	for (const variante of ['null', 'absent'] as const) {
		const init = variante === 'null' ? { idleTimeoutMinutes: N, idleExpiresIn: null } : { idleTimeoutMinutes: N };
		it(`idleExpiresIn ${variante}, valeurs stockées : gardées`, async () => {
			vi.setSystemTime(500_000);
			localStorage.setItem(KEY_LAST_ACTIVITY, '480000');
			localStorage.setItem(KEY_REPORTED_ACTIVITY, '470000');
			sessionIdle.start(init);
			expect(key(KEY_LAST_ACTIVITY)).toBe(480_000);
			expect(key(KEY_REPORTED_ACTIVITY)).toBe(480_000 > 470_000 ? 470_000 : 0);
			await advance(1_000);
			expect(postMock).toHaveBeenCalledTimes(1);
			expect(postMock.mock.calls[0][1]).toEqual({ idleMs: 21_000 });
		});
		it(`idleExpiresIn ${variante}, sans valeur stockée : now, aucun battement, jamais NaN`, async () => {
			vi.setSystemTime(500_000);
			sessionIdle.start(init);
			expect(key(KEY_LAST_ACTIVITY)).toBe(500_000);
			expect(key(KEY_REPORTED_ACTIVITY)).toBe(500_000);
			await advance(1_000);
			expect(postMock).not.toHaveBeenCalled();
			fire('keydown');
			await advance(1_000);
			expect(postMock).toHaveBeenCalledTimes(1);
			for (const k of [KEY_LAST_ACTIVITY, KEY_REPORTED_ACTIVITY, KEY_HEARTBEAT_AT]) {
				expect(localStorage.getItem(k)).not.toBe('NaN');
			}
			await advance(W);
			expect(replaceMock).toHaveBeenCalled();
		});
		it(`idleExpiresIn ${variante}, clé reportedActivity à "NaN" : corrigée`, async () => {
			vi.setSystemTime(500_000);
			localStorage.setItem(KEY_LAST_ACTIVITY, '480000');
			localStorage.setItem(KEY_REPORTED_ACTIVITY, 'NaN');
			sessionIdle.start(init);
			expect(localStorage.getItem(KEY_REPORTED_ACTIVITY)).toBe('0');
			await advance(1_000);
			expect(postMock).toHaveBeenCalledTimes(1);
			for (let i = 0; i < 20; i++) {
				await advance(1_000);
				for (const k of [KEY_LAST_ACTIVITY, KEY_REPORTED_ACTIVITY, KEY_HEARTBEAT_AT]) {
					expect(localStorage.getItem(k)).not.toBe('NaN');
				}
			}
		});
	}
	it('lastActivity effacée en cours de session : réécrite, fermeture à l’échéance attendue (12 bis)', async () => {
		startAtLogin();
		await advance(10_000);
		fire('keydown');
		localStorage.removeItem(KEY_LAST_ACTIVITY);
		await advance(1_000);
		expect(key(KEY_LAST_ACTIVITY)).toBe(10_000);
		await advance(W - 1_000 - 1_000);
		expect(fetchMock).not.toHaveBeenCalled();
		await advance(1_000);
		expect(fetchMock).toHaveBeenCalledTimes(1);
	});
	it('jamais d’activité future (17)', () => {
		vi.setSystemTime(1_000_000);
		sessionIdle.start({ idleTimeoutMinutes: 1, idleExpiresIn: 3_000 });
		expect(key(KEY_LAST_ACTIVITY)).toBe(1_000_000);
		sessionIdle.stop();
		localStorage.setItem(KEY_LAST_ACTIVITY, String(5_000_000));
		sessionIdle.start({ idleTimeoutMinutes: N, idleExpiresIn: null });
		expect(key(KEY_LAST_ACTIVITY)).toBe(1_000_000);
	});
	it('activité non rapportée au démarrage : un battement au premier tic (18)', async () => {
		const t = 100_000;
		vi.setSystemTime(t + 1_000);
		localStorage.setItem(KEY_LAST_ACTIVITY, String(t));
		localStorage.setItem(KEY_REPORTED_ACTIVITY, String(t - 20_000));
		// serverLast = t − 20 s : idleExpiresIn = (t − 20 s + N) − (t + 1 s).
		sessionIdle.start({ idleTimeoutMinutes: N, idleExpiresIn: (W - 21_000) / 1000 });
		expect(key(KEY_REPORTED_ACTIVITY)).toBe(t - 20_000);
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		expect(postMock.mock.calls[0][1]).toEqual({ idleMs: 2_000 });
		expect(key(KEY_HEARTBEAT_AT)).toBe(t + 2_000);
	});
});

describe('session-idle — sans localStorage, N inconnu (14, 15)', () => {
	it('localStorage indisponible : battement envoyé, aucune fermeture locale', async () => {
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('QuotaExceeded');
		});
		startAtLogin();
		await advance(1_000);
		fire('keydown');
		await advance(1_000);
		expect(postMock).toHaveBeenCalledTimes(1);
		await advance(W * 2);
		expect(fetchMock).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();
	});
	it('N inconnu : un seul GET /me, ses champs appliqués', async () => {
		getMock.mockResolvedValue({ idleTimeoutMinutes: 3, idleExpiresIn: 180 });
		sessionIdle.start({});
		await advance(0);
		expect(getMock).toHaveBeenCalledTimes(1);
		expect(getMock).toHaveBeenCalledWith('/api/v1/auth/me');
		expect(sessionIdle.minutes).toBe(3);
		await advance(3 * 60_000);
		expect(replaceMock).toHaveBeenCalledWith('/login?reason=idle&minutes=3');
	});
	it('N inconnu et /me en échec : aucune fermeture', async () => {
		getMock.mockRejectedValue({ code: 'NETWORK_ERROR' });
		sessionIdle.start({});
		await advance(W * 3);
		expect(getMock).toHaveBeenCalledTimes(1);
		expect(replaceMock).not.toHaveBeenCalled();
	});
});

// Revue de code P1 (N-A1/N-E1) — après `stop()`, aucun des onze écouteurs ne
// produit plus rien. Chaque geste est tiré au-delà de l'échéance : un écouteur
// resté posé appellerait `recordActivity`, qui fermerait la session (le
// témoin de chaque cas, sans `stop()`, le montre).
describe('session-idle — stop() retire chacun des onze écouteurs (10)', () => {
	const gestes: Array<[string, () => void]> = [
		['keydown', () => fire('keydown')],
		['pointerdown', () => fire('pointerdown')],
		['wheel', () => fire('wheel')],
		['touchstart', () => fire('touchstart')],
		['dragover', () => fire('dragover')],
		['drop', () => fire('drop')],
		['pointermove', () => move(321, 654)],
		['focus de la fenêtre', () => window.dispatchEvent(new FocusEvent('focus'))],
		[
			'blur de la fenêtre vers un <iframe>',
			() => {
				const f = document.createElement('iframe');
				document.body.appendChild(f);
				vi.spyOn(document, 'activeElement', 'get').mockReturnValue(f);
				window.dispatchEvent(new FocusEvent('blur'));
			},
		],
		[
			'input sur le document',
			() => {
				const input = document.createElement('input');
				document.body.appendChild(input);
				input.dispatchEvent(new Event('input', { bubbles: true }));
			},
		],
		[
			'visibilitychange vers visible',
			() => {
				Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
				fire('visibilitychange', {}, document);
			},
		],
	];
	it('les cas couvrent les onze écouteurs du module', () => {
		expect(gestes).toHaveLength(11);
	});
	for (const [nom, geste] of gestes) {
		it(`${nom} : témoin sans stop() ⇒ fermeture`, async () => {
			startAtLogin();
			vi.setSystemTime(W + 5_000);
			geste();
			await advance(0);
			expect(fetchMock).toHaveBeenCalledWith('/api/v1/auth/logout', expect.anything());
		});
		it(`${nom} : après stop() ⇒ rien`, async () => {
			startAtLogin();
			sessionIdle.stop();
			vi.setSystemTime(W + 5_000);
			geste();
			await advance(0);
			expect(fetchMock).not.toHaveBeenCalled();
			expect(postMock).not.toHaveBeenCalled();
			expect(key(KEY_LAST_ACTIVITY)).toBe(0);
		});
	}
	it('stop() retire la minuterie : aucune minuterie en attente', async () => {
		// jsdom diffuse chaque écriture de `localStorage` par une minuterie à 0 ms
		// (événement `storage`) : on les laisse passer, puis on compte.
		await advance(0);
		const avant = vi.getTimerCount();
		startAtLogin();
		await advance(0);
		expect(vi.getTimerCount()).toBe(avant + 1);
		sessionIdle.stop();
		expect(vi.getTimerCount()).toBe(avant);
	});
});

// Revue de code P1 (N-E2) — `localStorage` qui refuse EN COURS de session : le
// module bascule en mémoire et ne ferme pas un utilisateur actif.
describe('session-idle — localStorage refusé en cours de session (14 bis)', () => {
	async function activiteContinue(jusqua: number) {
		while (Date.now() < jusqua) {
			fire('keydown');
			await advance(20_000);
		}
	}
	it('écriture refusée après le démarrage : battement envoyé, aucune fermeture', async () => {
		startAtLogin();
		await advance(10_000);
		fire('keydown');
		expect(key(KEY_LAST_ACTIVITY)).toBe(10_000);
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('QuotaExceeded');
		});
		await advance(20_000);
		const avant = postMock.mock.calls.length;
		await activiteContinue(W + 60_000);
		expect(postMock.mock.calls.length).toBeGreaterThan(avant);
		expect(fetchMock).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();
	});
	it('lecture refusée après le démarrage : battement envoyé, aucune fermeture', async () => {
		startAtLogin();
		await advance(10_000);
		fire('keydown');
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new Error('SecurityError');
		});
		await advance(20_000);
		const avant = postMock.mock.calls.length;
		const debut = Date.now();
		await activiteContinue(W + 60_000);
		const envoyes = postMock.mock.calls.length - avant;
		expect(envoyes).toBeGreaterThan(0);
		// Le repli en mémoire garde la régulation à 15 s (une lecture qui
		// rendrait « absente » sans basculer la ferait sauter).
		expect(envoyes).toBeLessThanOrEqual(Math.ceil((Date.now() - debut) / 15_000) + 1);
		expect(fetchMock).not.toHaveBeenCalled();
		expect(replaceMock).not.toHaveBeenCalled();
	});
});

// Revue de code P1 (N-B1/N-E4) — une clé vide ou blanche vaut absente, et une
// borne non finie de la conciliation aussi.
describe('session-idle — valeurs vides et non finies (12 ter)', () => {
	for (const vide of ['', '   ']) {
		it(`lastActivity à ${JSON.stringify(vide)} : absente, réécrite, aucune fermeture`, async () => {
			vi.setSystemTime(1_000_000);
			startAtLogin();
			await advance(10_000);
			fire('keydown');
			expect(key(KEY_LAST_ACTIVITY)).toBe(1_010_000);
			localStorage.setItem(KEY_LAST_ACTIVITY, vide);
			await advance(1_000);
			expect(fetchMock).not.toHaveBeenCalled();
			expect(key(KEY_LAST_ACTIVITY)).toBe(1_010_000);
		});
	}
	it('idleExpiresIn démesuré (−1e308) : aucune borne infinie, lastActivity = now', async () => {
		vi.setSystemTime(1_000_000);
		sessionIdle.start({ idleTimeoutMinutes: N, idleExpiresIn: -1e308 });
		expect(key(KEY_LAST_ACTIVITY)).toBe(1_000_000);
		await advance(1_000);
		expect(fetchMock).not.toHaveBeenCalled();
	});
});
