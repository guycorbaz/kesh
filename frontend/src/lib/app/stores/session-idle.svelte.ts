/**
 * Suivi d'activité de la session — Story v0172-session (0.17.3, #876, CR #891).
 *
 * La session se ferme **exactement** après la durée d'inactivité N réglée par
 * l'Administrateur (minutes, lue du serveur), et **toute activité** de
 * l'utilisateur dans Kesh la remet à zéro — dans tous les onglets du même
 * navigateur, qui partagent trois clés `localStorage` :
 *
 * - `kesh:session:lastActivity` — l'instant (ms) de la dernière activité ;
 * - `kesh:session:reportedActivity` — la dernière activité rapportée au serveur ;
 * - `kesh:session:heartbeatAt` — l'instant du dernier battement envoyé.
 *
 * Une minuterie d'une seconde par onglet : (a) à `max(lastActivity) + N`, la
 * session est fermée — déconnexion au serveur (attendue jusqu'à 2 s), puis
 * `/login?reason=idle&minutes=N` ; (b) sinon, une activité non rapportée part
 * au serveur par le battement `POST /api/v1/auth/activity` (au plus toutes les
 * 15 s, tous onglets réunis), qui porte l'**âge** de la dernière activité —
 * le serveur place son échéance à `réception − âge + N`.
 *
 * Ce qui n'y est pas, délibérément : aucun verrou entre onglets, aucun onglet
 * maître, aucune suspension. Deux onglets qui envoient le même battement à la
 * même seconde écrivent deux fois la même échéance.
 *
 * Ce qui compte comme activité : `keydown`, `pointerdown`, `pointermove`
 * (seulement si les coordonnées d'écran changent : le navigateur émet des
 * mouvements fictifs quand la page bouge sous un pointeur immobile), `wheel`,
 * `touchstart`, `focus` de la fenêtre elle-même, `dragover`, `drop`, `input`
 * (dictée, collage, clavier virtuel), le retour sur l'onglet
 * (`visibilitychange` vers `visible`), et le clic qui donne le focus à un
 * `<iframe>` de la page (`blur` de la fenêtre, l'aperçu PDF). `scroll` n'est
 * **pas** écouté (une page qui bouge sans geste en émet). Le chargement d'une
 * page ne compte pas, ni les requêtes que Kesh fait seul.
 *
 * Toute valeur lue d'une clé qui n'est pas un nombre fini vaut « absente », et
 * aucune écriture ne se fait sans `Number.isFinite` vrai : un `NaN` écrit une
 * fois couperait les battements de tous les onglets (C-v0172s-52).
 *
 * Sans `localStorage` (navigateur qui le refuse), le module garde ses trois
 * valeurs en mémoire, **continue d'envoyer le battement** de l'activité de
 * son onglet, mais **ne ferme rien de lui-même** : c'est le serveur qui ferme
 * (C-v0172s-45).
 */

import { apiClient } from '$lib/shared/utils/api-client';
import { authState } from '$lib/app/stores/auth.svelte';

/** Clé de la dernière activité (ms). */
export const KEY_LAST_ACTIVITY = 'kesh:session:lastActivity';
/** Clé de la dernière activité rapportée au serveur (ms). */
export const KEY_REPORTED_ACTIVITY = 'kesh:session:reportedActivity';
/** Clé de l'instant du dernier battement envoyé (ms). */
export const KEY_HEARTBEAT_AT = 'kesh:session:heartbeatAt';

/** Pas de la minuterie (ms). */
export const TICK_MS = 1_000;
/** Écart minimal entre deux battements, tous onglets réunis (ms). */
export const HEARTBEAT_MIN_INTERVAL_MS = 15_000;
/** Délai laissé à la déconnexion au serveur avant la redirection (ms). */
export const LOGOUT_TIMEOUT_MS = 2_000;
/** Écriture de l'activité au plus une fois par seconde (ms). */
const ACTIVITY_WRITE_THROTTLE_MS = 1_000;

/** Bornes de N (minutes), miroir du serveur. */
const MIN_MINUTES = 1;
const MAX_MINUTES = 1_440;

/** Ce que le layout passe au démarrage (champs de transit du `LoginPayload`). */
export interface SessionIdleInit {
	idleTimeoutMinutes?: number;
	idleExpiresIn?: number | null;
}

/** Réponse du battement et de `/me` (champs d'inactivité). */
interface IdleFields {
	idleTimeoutMinutes?: unknown;
	idleExpiresIn?: unknown;
}

function isValidMinutes(n: unknown): n is number {
	return typeof n === 'number' && Number.isInteger(n) && n >= MIN_MINUTES && n <= MAX_MINUTES;
}

function isFiniteNumber(n: unknown): n is number {
	return typeof n === 'number' && Number.isFinite(n);
}

// ---------------------------------------------------------------------------
// Stockage partagé, avec repli en mémoire
// ---------------------------------------------------------------------------

/** `true` tant que `localStorage` accepte les écritures. */
let shared = true;
const memory = new Map<string, number>();

function probeStorage(): boolean {
	try {
		const probe = 'kesh:session:probe';
		window.localStorage.setItem(probe, '1');
		window.localStorage.removeItem(probe);
		return true;
	} catch {
		return false;
	}
}

/** La valeur d'une clé si c'est un nombre fini, `null` sinon (absente). */
function readKey(key: string): number | null {
	if (!shared) {
		const v = memory.get(key);
		return isFiniteNumber(v) ? v : null;
	}
	let raw: string | null;
	try {
		raw = window.localStorage.getItem(key);
	} catch {
		shared = false;
		return readKey(key);
	}
	if (raw === null) return null;
	const v = Number(raw);
	return Number.isFinite(v) ? v : null;
}

/** Écrit une valeur finie ; une valeur non finie n'est jamais écrite. */
function writeKey(key: string, value: number): void {
	if (!Number.isFinite(value)) return;
	memory.set(key, value);
	if (!shared) return;
	try {
		window.localStorage.setItem(key, String(value));
	} catch {
		// Le navigateur refuse : repli en mémoire, sans fermeture locale.
		shared = false;
	}
}

// ---------------------------------------------------------------------------
// État du module
// ---------------------------------------------------------------------------

let minutes = $state<number | null>(null);
let running = false;
/** Incrémenté à chaque `start()` / `stop()` : une réponse tardive d'un démarrage
 * précédent (`/me`) est ignorée. */
let generation = 0;
let timer: ReturnType<typeof setInterval> | null = null;
let closing = false;
let heartbeatInFlight = false;
/** Dernière valeur finie de `lastActivity` lue ou écrite par cet onglet. */
let memLast: number | null = null;
/** Instant de la dernière écriture d'activité par cet onglet (régulation). */
let lastWriteAt = -Infinity;
/** Dernières coordonnées d'écran d'un `pointermove`. */
let lastPointer: { x: number; y: number } | null = null;

function windowMs(): number | null {
	return minutes === null ? null : minutes * 60_000;
}

/** `lastActivity` en vigueur : la clé si elle est finie, sinon la dernière
 * valeur connue du module (ou `now`), réécrite — R5-4. */
function currentLast(now: number): number {
	const stored = readKey(KEY_LAST_ACTIVITY);
	if (stored !== null) {
		memLast = stored;
		return stored;
	}
	const restored = memLast ?? now;
	writeKey(KEY_LAST_ACTIVITY, restored);
	memLast = restored;
	return restored;
}

/** L'échéance est-elle atteinte ? Ne ferme jamais sans stockage partagé. */
function deadlineReached(now: number): boolean {
	const w = windowMs();
	if (w === null || !shared) return false;
	return now >= currentLast(now) + w;
}

// ---------------------------------------------------------------------------
// Conciliation au démarrage
// ---------------------------------------------------------------------------

function concile(now: number, idleExpiresIn: unknown): void {
	const w = windowMs();
	const storedLast = readKey(KEY_LAST_ACTIVITY);
	const storedRep = readKey(KEY_REPORTED_ACTIVITY);
	let last: number;
	let rep: number;
	if (w !== null && isFiniteNumber(idleExpiresIn)) {
		// Dernière activité connue du serveur, jamais dans le futur.
		const serverLast = Math.min(now, now + idleExpiresIn * 1000 - w);
		last = Math.min(now, Math.max(storedLast ?? -Infinity, serverLast));
		rep = Math.min(now, Math.max(storedRep ?? -Infinity, serverLast));
	} else {
		// Jeton sans échéance (émis avant la mise à jour) ou N inconnu :
		// aucun calcul sur `idleExpiresIn`.
		last = storedLast !== null ? Math.min(now, storedLast) : now;
		rep = storedRep !== null ? storedRep : storedLast !== null ? 0 : last;
	}
	writeKey(KEY_LAST_ACTIVITY, last);
	writeKey(KEY_REPORTED_ACTIVITY, rep);
	memLast = last;
}

// ---------------------------------------------------------------------------
// Activité
// ---------------------------------------------------------------------------

function recordActivity(): void {
	const now = Date.now();
	// D'abord l'échéance : une activité au-delà ne ranime pas la session.
	if (deadlineReached(now)) {
		void closeSession();
		return;
	}
	if (now - lastWriteAt < ACTIVITY_WRITE_THROTTLE_MS) return;
	lastWriteAt = now;
	writeKey(KEY_LAST_ACTIVITY, now);
	memLast = now;
}

function onPointerMove(e: Event): void {
	const p = e as PointerEvent;
	const x = p.screenX;
	const y = p.screenY;
	if (lastPointer !== null && lastPointer.x === x && lastPointer.y === y) return;
	lastPointer = { x, y };
	recordActivity();
}

/** La cible est la fenêtre elle-même (tout autre cible de `focus`/`blur` est
 * un nœud du document : un élément qui prend ou perd le focus). */
function targetsWindow(e: Event): boolean {
	return !(e.target instanceof Node);
}

function onFocus(e: Event): void {
	if (targetsWindow(e)) recordActivity();
}

function onBlur(e: Event): void {
	if (!targetsWindow(e)) return;
	if (document.activeElement instanceof HTMLIFrameElement) recordActivity();
}

function onVisibility(): void {
	if (document.visibilityState === 'visible') recordActivity();
}

const WINDOW_EVENTS = ['keydown', 'pointerdown', 'wheel', 'touchstart', 'dragover', 'drop'] as const;
const LISTENER_OPTIONS: AddEventListenerOptions = { capture: true, passive: true };

function addListeners(): void {
	for (const type of WINDOW_EVENTS) window.addEventListener(type, recordActivity, LISTENER_OPTIONS);
	window.addEventListener('pointermove', onPointerMove, LISTENER_OPTIONS);
	window.addEventListener('focus', onFocus, LISTENER_OPTIONS);
	window.addEventListener('blur', onBlur, LISTENER_OPTIONS);
	document.addEventListener('input', recordActivity, LISTENER_OPTIONS);
	document.addEventListener('visibilitychange', onVisibility, LISTENER_OPTIONS);
}

function removeListeners(): void {
	for (const type of WINDOW_EVENTS) window.removeEventListener(type, recordActivity, LISTENER_OPTIONS);
	window.removeEventListener('pointermove', onPointerMove, LISTENER_OPTIONS);
	window.removeEventListener('focus', onFocus, LISTENER_OPTIONS);
	window.removeEventListener('blur', onBlur, LISTENER_OPTIONS);
	document.removeEventListener('input', recordActivity, LISTENER_OPTIONS);
	document.removeEventListener('visibilitychange', onVisibility, LISTENER_OPTIONS);
}

// ---------------------------------------------------------------------------
// Minuterie, battement, fermeture
// ---------------------------------------------------------------------------

function tick(): void {
	if (!running) return;
	const now = Date.now();
	const last = currentLast(now);
	const w = windowMs();
	if (w !== null && shared && now >= last + w) {
		void closeSession();
		return;
	}
	if (heartbeatInFlight) return;
	const rep = readKey(KEY_REPORTED_ACTIVITY) ?? -Infinity;
	// `heartbeatAt` absente ⇒ battement permis (R5-3).
	const hb = readKey(KEY_HEARTBEAT_AT) ?? -Infinity;
	if (!(last > rep) || now - hb < HEARTBEAT_MIN_INTERVAL_MS) return;
	writeKey(KEY_HEARTBEAT_AT, now);
	writeKey(KEY_REPORTED_ACTIVITY, last);
	void sendHeartbeat(now - last, rep, generation);
}

async function sendHeartbeat(idleMs: number, previousReported: number, gen: number): Promise<void> {
	heartbeatInFlight = true;
	try {
		const res = await apiClient.post<IdleFields>('/api/v1/auth/activity', {
			idleMs: Math.max(0, Math.round(idleMs)),
		});
		if (gen === generation && isValidMinutes(res?.idleTimeoutMinutes)) {
			minutes = res.idleTimeoutMinutes;
		}
	} catch {
		// Échec (réseau, 5xx) : l'activité redevient « à rapporter » ; `heartbeatAt`
		// garde sa valeur, si bien que l'envoi se retente au pas de 15 s suivant.
		// Un 401 définitif a déjà effacé la session et redirigé (chemin 401).
		if (gen === generation) writeKey(KEY_REPORTED_ACTIVITY, previousReported);
	} finally {
		heartbeatInFlight = false;
	}
}

async function closeSession(): Promise<void> {
	if (closing) return;
	closing = true;
	const n = minutes;
	sessionIdle.stop();
	const controller = new AbortController();
	let timeout: ReturnType<typeof setTimeout> | undefined;
	const delay = new Promise<void>((resolve) => {
		timeout = setTimeout(() => {
			controller.abort();
			resolve();
		}, LOGOUT_TIMEOUT_MS);
	});
	try {
		await Promise.race([
			fetch('/api/v1/auth/logout', {
				method: 'POST',
				credentials: 'include',
				headers: { 'Content-Type': 'application/json' },
				body: '{}',
				signal: controller.signal,
			}).catch(() => undefined),
			delay,
		]);
	} finally {
		clearTimeout(timeout);
	}
	authState.clearSession({ broadcast: false });
	window.location.replace(`/login?reason=idle&minutes=${n ?? ''}`);
	closing = false;
}

// ---------------------------------------------------------------------------
// Démarrage
// ---------------------------------------------------------------------------

async function learnFromMe(gen: number): Promise<void> {
	try {
		const me = await apiClient.get<IdleFields>('/api/v1/auth/me');
		if (gen !== generation) return;
		if (isValidMinutes(me?.idleTimeoutMinutes)) {
			minutes = me.idleTimeoutMinutes;
			concile(Date.now(), me.idleExpiresIn);
		}
	} catch {
		// Le serveur fait foi : rien n'est fermé localement ; retenté au
		// démarrage suivant.
	}
}

/** L'interface du module (C-v0172s-50). */
export const sessionIdle = {
	/**
	 * Démarre le suivi : conciliation avec le serveur, écouteurs, minuterie.
	 * N inconnu ⇒ un seul `GET /api/v1/auth/me`.
	 */
	start(init: SessionIdleInit): void {
		if (typeof window === 'undefined') return;
		sessionIdle.stop();
		generation += 1;
		const gen = generation;
		running = true;
		closing = false;
		shared = probeStorage();
		// La mémoire ne sert que de repli (et de copie) : elle repart vide.
		memory.clear();
		lastPointer = null;
		lastWriteAt = -Infinity;
		memLast = null;
		minutes = isValidMinutes(init.idleTimeoutMinutes) ? init.idleTimeoutMinutes : null;
		// Sans N, la conciliation se fait sans calcul sur `idleExpiresIn` (branche
		// « jeton sans échéance ») ; `/me` la refait avec les valeurs du serveur.
		concile(Date.now(), minutes !== null ? init.idleExpiresIn : null);
		if (minutes === null) void learnFromMe(gen);
		addListeners();
		timer = setInterval(tick, TICK_MS);
	},

	/** Retire écouteurs et minuterie ; n'écrit rien. Idempotente. */
	stop(): void {
		if (!running) return;
		running = false;
		generation += 1;
		removeListeners();
		if (timer !== null) {
			clearInterval(timer);
			timer = null;
		}
	},

	/** N neuf (réponse d'un battement, ou enregistrement de l'écran). */
	setMinutes(n: number): void {
		if (isValidMinutes(n)) minutes = n;
	},

	/** N courant, lu par la fermeture (`?minutes=`). */
	get minutes(): number | null {
		return minutes;
	},
};
