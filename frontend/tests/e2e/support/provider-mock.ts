/**
 * Story 38-12b (AC-7, socle S8) — le **serveur de prestataire simulé commun**
 * des E2E : un petit serveur HTTP du bouclage, lancé par la spec sur le port lu
 * de `KESH_E2E_PROVIDER_MOCK_PORT`, vers lequel le backend d'E2E est démarré
 * (`KESH_<P>_BASE_URL=http://127.0.0.1:<port>`, `http` admis vers le seul
 * bouclage). Il joue PayPal ici ; Wise (39-12b) et Stripe (40-8b) y ajoutent
 * leurs routes ([`ProviderMockRoute`]) sans toucher au serveur.
 *
 * **Aucune donnée réelle** : identifiants `0TEST…`, e-mails `@example.invalid`,
 * aucun appel sortant. Le serveur garde la liste des requêtes reçues, pour que
 * la spec vérifie ce que le backend a demandé.
 */
import { createServer, type IncomingMessage, type Server } from 'node:http';

/** Variable du port du serveur simulé (montage de `docs/testing.md`). */
export const PROVIDER_MOCK_PORT_VAR = 'KESH_E2E_PROVIDER_MOCK_PORT';

/** Le port du serveur simulé, `null` si la variable n'est pas posée ou illisible. */
export function providerMockPort(): number | null {
	const raw = process.env[PROVIDER_MOCK_PORT_VAR];
	if (!raw) return null;
	const port = Number(raw);
	return Number.isInteger(port) && port > 0 && port < 65536 ? port : null;
}

/** Une requête reçue. */
export interface RecordedRequest {
	method: string;
	path: string;
	query: URLSearchParams;
	authorization: string | null;
	body: string;
}

/**
 * La réponse d'une route ; `delayMs` la retarde (progression visible) ; `until`
 * la retient jusqu'à ce que la spec la libère (au plus [`HOLD_CAP_MS`]).
 */
export interface MockReply {
	status: number;
	json?: unknown;
	headers?: Record<string, string>;
	delayMs?: number;
	until?: Promise<void>;
}

/**
 * Durée maximale d'une réponse retenue (`MockReply.until`) : sous le délai total
 * d'une requête du backend (60 s), pour qu'une spec qui ne libère pas échoue sur
 * ses propres assertions plutôt que sur un délai dépassé du backend.
 */
export const HOLD_CAP_MS = 45_000;

/** Une route du serveur simulé. */
export interface ProviderMockRoute {
	method: 'GET' | 'POST';
	path: string;
	handle: (req: RecordedRequest) => MockReply;
}

/** Le serveur lancé. */
export interface ProviderMock {
	requests: RecordedRequest[];
	close: () => Promise<void>;
}

function readBody(req: IncomingMessage): Promise<string> {
	return new Promise((resolve, reject) => {
		const chunks: Buffer[] = [];
		req.on('data', (c: Buffer) => chunks.push(c));
		req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
		req.on('error', reject);
	});
}

/** Lance le serveur simulé sur `127.0.0.1:<port>` avec `routes` ; `404` pour toute autre requête. */
export async function startProviderMock(port: number, routes: ProviderMockRoute[]): Promise<ProviderMock> {
	const requests: RecordedRequest[] = [];
	const server: Server = createServer(async (req, res) => {
		const url = new URL(req.url ?? '/', 'http://127.0.0.1');
		const recorded: RecordedRequest = {
			method: req.method ?? 'GET',
			path: url.pathname,
			query: url.searchParams,
			authorization: req.headers.authorization ?? null,
			body: await readBody(req),
		};
		requests.push(recorded);
		const route = routes.find((r) => r.method === recorded.method && r.path === recorded.path);
		const reply: MockReply = route ? route.handle(recorded) : { status: 404, json: { name: 'NOT_FOUND' } };
		if (reply.until) await Promise.race([reply.until, new Promise((r) => setTimeout(r, HOLD_CAP_MS))]);
		if (reply.delayMs) await new Promise((r) => setTimeout(r, reply.delayMs));
		res.writeHead(reply.status, { 'content-type': 'application/json', ...reply.headers });
		res.end(reply.json === undefined ? '' : JSON.stringify(reply.json));
	});
	await new Promise<void>((resolve, reject) => {
		server.once('error', reject);
		server.listen(port, '127.0.0.1', () => resolve());
	});
	return {
		requests,
		close: () => new Promise<void>((resolve) => server.close(() => resolve())),
	};
}

// ---------------------------------------------------------------------------
// PayPal (Story 38-12a : jeton OAuth, recherche des transactions)
// ---------------------------------------------------------------------------

/** Une transaction fictive, dans la forme de l'API de recherche de PayPal. */
export interface PaypalMockTransaction {
	id: string;
	/** Instant de la transaction, ISO 8601 en UTC (`…T10:00:00+00:00`). */
	at: string;
	/** Montant signé, deux décimales. */
	gross: string;
	currency: string;
	name: string;
	email: string;
}

/** Réglages du jeu PayPal. */
export interface PaypalMockOptions {
	transactions: PaypalMockTransaction[];
	/** Délai de chaque page (la progression se voit à l'écran). */
	pageDelayMs?: number;
	/**
	 * Répondre **une fois** `429` (`Retry-After` en secondes) à la première page
	 * de la fenêtre de rang `rateLimitWindow` (à partir de 1), pour que la pause
	 * se voie.
	 */
	rateLimitOnce?: { window: number; retryAfterSeconds: number };
	/**
	 * Retenir les réponses des fenêtres de rang `window` et suivants (à partir de
	 * 1) jusqu'à `until` : la tâche reste en cours tant que la spec n'a pas joué ce
	 * qui doit se passer pendant qu'elle tourne (le `409` d'une relance), sans
	 * dépendre du temps qu'il lui reste (revue P2 de la 38-12b, L4).
	 */
	holdFromWindow?: { window: number; until: Promise<void> };
	/** `last_refreshed_datetime` des pages. */
	lastRefreshed: string;
}

function paypalTx(t: PaypalMockTransaction): unknown {
	return {
		transaction_info: {
			transaction_id: t.id,
			transaction_event_code: 'T0007',
			transaction_initiation_date: t.at,
			transaction_status: 'S',
			transaction_amount: { currency_code: t.currency, value: t.gross },
			transaction_subject: 'Commande fictive',
		},
		payer_info: { email_address: t.email, payer_name: { full_name: t.name } },
	};
}

/**
 * Les routes de PayPal : `POST /v1/oauth2/token` (Basic exigé, corps
 * `grant_type=client_credentials`) et `GET /v1/reporting/transactions` (jeton
 * exigé ; ne rend que les transactions de la fenêtre demandée, une page).
 */
export function paypalMockRoutes(options: PaypalMockOptions): ProviderMockRoute[] {
	const token = 'jeton-fictif-e2e';
	const windows: string[] = [];
	let limited = false;
	return [
		{
			method: 'POST',
			path: '/v1/oauth2/token',
			handle: (req) =>
				req.authorization?.startsWith('Basic ') && req.body === 'grant_type=client_credentials'
					? { status: 200, json: { access_token: token, token_type: 'Bearer', expires_in: 32400 } }
					: { status: 401, json: { error: 'invalid_client' } },
		},
		{
			method: 'GET',
			path: '/v1/reporting/transactions',
			handle: (req) => {
				if (req.authorization !== `Bearer ${token}`) return { status: 401, json: { name: 'AUTHENTICATION_FAILURE' } };
				const start = req.query.get('start_date') ?? '';
				const end = req.query.get('end_date') ?? '';
				if (!windows.includes(start)) windows.push(start);
				const rank = windows.indexOf(start) + 1;
				const until =
					options.holdFromWindow && rank >= options.holdFromWindow.window ? options.holdFromWindow.until : undefined;
				if (options.rateLimitOnce && !limited && rank === options.rateLimitOnce.window) {
					limited = true;
					return {
						status: 429,
						headers: { 'retry-after': String(options.rateLimitOnce.retryAfterSeconds) },
						json: { name: 'RATE_LIMIT_REACHED' },
						until,
					};
				}
				const from = Date.parse(start);
				const to = Date.parse(end);
				const inside = options.transactions.filter((t) => {
					const at = Date.parse(t.at);
					return at >= from && at <= to;
				});
				const body: Record<string, unknown> = {
					start_date: start,
					end_date: end,
					page: 1,
					total_pages: inside.length > 0 ? 1 : 0,
					last_refreshed_datetime: options.lastRefreshed,
				};
				if (inside.length > 0) body.transaction_details = inside.map(paypalTx);
				return { status: 200, json: body, delayMs: options.pageDelayMs, until };
			},
		},
	];
}
