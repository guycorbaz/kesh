/**
 * Story 38-12b (AC-7, socle S8) — le **serveur de prestataire simulé commun**
 * des E2E : un petit serveur HTTP du bouclage, lancé par la spec sur le port lu
 * de `KESH_E2E_PROVIDER_MOCK_PORT`, vers lequel le backend d'E2E est démarré
 * (`KESH_<P>_BASE_URL=http://127.0.0.1:<port>`, `http` admis vers le seul
 * bouclage). Il joue PayPal (38-12b) et Stripe (40-8b, [`stripeMockRoutes`]) ;
 * Wise (39-12b) y ajoute ses routes ([`ProviderMockRoute`]) sans toucher au
 * serveur.
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
 * La réponse d'une route ; `delayMs` la retarde (progression visible) ; `hold`
 * la retient jusqu'à ce que la spec la libère par `ProviderMock.release()` (au
 * plus [`HOLD_CAP_MS`]).
 */
export interface MockReply {
	status: number;
	json?: unknown;
	headers?: Record<string, string>;
	delayMs?: number;
	hold?: boolean;
}

/**
 * Durée maximale d'une réponse retenue (`MockReply.hold`) : sous le délai total
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
	/** Libère les réponses retenues (`MockReply.hold`), présentes et à venir. */
	release: () => void;
	/** Libère les réponses retenues, puis arrête le serveur. */
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
	// La retenue appartient à CE serveur, non au module : un serveur neuf (une
	// relance de la spec) retient de nouveau, et `close()` la libère, sans
	// attendre `HOLD_CAP_MS` si la spec a échoué avant de libérer (revue P3, LOW-1).
	let release: () => void = () => {};
	const held = new Promise<void>((resolve) => (release = resolve));
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
		if (reply.hold) {
			let cap: ReturnType<typeof setTimeout> | undefined;
			await Promise.race([held, new Promise((r) => (cap = setTimeout(r, HOLD_CAP_MS)))]);
			clearTimeout(cap);
		}
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
		release: () => release(),
		close: () => {
			release();
			// `server.close()` attend la fin des connexions gardées ouvertes (keep-alive,
			// ~5 s) : on les ferme aussitôt (revue P4, observation).
			const closed = new Promise<void>((resolve) => server.close(() => resolve()));
			server.closeAllConnections();
			return closed;
		},
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
	 * 1) jusqu'à `ProviderMock.release()` : la tâche reste en cours tant que la
	 * spec n'a pas joué ce qui doit se passer pendant qu'elle tourne (le `409`
	 * d'une relance), sans dépendre du temps qu'il lui reste (revue P2 de la
	 * 38-12b, L4).
	 */
	holdFromWindow?: { window: number };
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
				const hold = options.holdFromWindow !== undefined && rank >= options.holdFromWindow.window;
				if (options.rateLimitOnce && !limited && rank === options.rateLimitOnce.window) {
					limited = true;
					return {
						status: 429,
						headers: { 'retry-after': String(options.rateLimitOnce.retryAfterSeconds) },
						json: { name: 'RATE_LIMIT_REACHED' },
						hold,
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
				return { status: 200, json: body, delayMs: options.pageDelayMs, hold };
			},
		},
	];
}

// ---------------------------------------------------------------------------
// Stripe (Story 40-8a : versements payés, transactions de solde d'un versement)
// ---------------------------------------------------------------------------

/** Une charge fictive d'un versement Stripe (montants décimaux, deux décimales). */
export interface StripeMockCharge {
	/** Identifiant de la transaction de solde (`txn_TEST…`). */
	id: string;
	/** Identifiant de la charge (`ch_TEST…`), la `source`. */
	source: string;
	/** Brut et frais, `"10.00"`. */
	gross: string;
	fee: string;
	/** Instant de création, ISO 8601 en UTC. */
	createdAt: string;
}

/** Un versement fictif payé, en CHF. */
export interface StripeMockPayout {
	id: string;
	/** Instant d'arrivée, ISO 8601 en UTC (le filtre `arrival_date`). */
	arrivalAt: string;
	automatic: boolean;
	charges: StripeMockCharge[];
}

/** Réglages du jeu Stripe. */
export interface StripeMockOptions {
	/** La clé restreinte attendue (`Bearer <clé>`). */
	key: string;
	payouts: StripeMockPayout[];
	/**
	 * Retenir les transactions de ce versement jusqu'à `ProviderMock.release()` :
	 * la progression « versement 1/2 » se voit, sans dépendre du temps.
	 */
	holdPayout?: string;
}

const minor = (decimal: string) => Math.round(Number(decimal) * 100);
const epoch = (iso: string) => Math.floor(Date.parse(iso) / 1000);

/** Le versement dans la forme de `GET /v1/payouts` (montant = somme des nets). */
function stripePayout(p: StripeMockPayout): unknown {
	const amount = p.charges.reduce((sum, c) => sum + minor(c.gross) - minor(c.fee), 0);
	return {
		id: p.id,
		object: 'payout',
		amount,
		currency: 'chf',
		arrival_date: epoch(p.arrivalAt),
		automatic: p.automatic,
		status: 'paid',
	};
}

/** Les transactions de solde d'un versement, dont la sienne (`type: payout`), à écarter. */
function stripeBalanceTransactions(p: StripeMockPayout): unknown[] {
	const own = (stripePayout(p) as { amount: number }).amount;
	return [
		{
			id: `txn_PO${p.id.slice(3)}`,
			object: 'balance_transaction',
			amount: -own,
			fee: 0,
			net: -own,
			currency: 'chf',
			created: epoch(p.arrivalAt),
			available_on: epoch(p.arrivalAt),
			reporting_category: 'payout',
			type: 'payout',
			description: 'STRIPE PAYOUT',
			source: p.id,
		},
		...p.charges.map((c) => ({
			id: c.id,
			object: 'balance_transaction',
			amount: minor(c.gross),
			fee: minor(c.fee),
			net: minor(c.gross) - minor(c.fee),
			currency: 'chf',
			created: epoch(c.createdAt),
			available_on: epoch(p.arrivalAt),
			reporting_category: 'charge',
			type: 'charge',
			description: 'Vente fictive',
			source: c.source,
		})),
	];
}

/**
 * Les routes de Stripe : `GET /v1/payouts` (clé exigée ; versements dont
 * l'arrivée tombe dans `[arrival_date[gte], arrival_date[lt][`, une page) et
 * `GET /v1/balance_transactions?payout=…` (clé exigée ; une page).
 */
export function stripeMockRoutes(options: StripeMockOptions): ProviderMockRoute[] {
	const authorized = (req: RecordedRequest) => req.authorization === `Bearer ${options.key}`;
	const refused: MockReply = { status: 401, json: { error: { type: 'invalid_request_error', code: 'api_key_invalid' } } };
	return [
		{
			method: 'GET',
			path: '/v1/payouts',
			handle: (req) => {
				if (!authorized(req)) return refused;
				const gte = Number(req.query.get('arrival_date[gte]'));
				const lt = Number(req.query.get('arrival_date[lt]'));
				const data = options.payouts
					.filter((p) => epoch(p.arrivalAt) >= gte && epoch(p.arrivalAt) < lt)
					.sort((a, b) => epoch(b.arrivalAt) - epoch(a.arrivalAt))
					.map(stripePayout);
				return { status: 200, json: { object: 'list', data, has_more: false } };
			},
		},
		{
			method: 'GET',
			path: '/v1/balance_transactions',
			handle: (req) => {
				if (!authorized(req)) return refused;
				const payout = options.payouts.find((p) => p.id === req.query.get('payout'));
				if (!payout) return { status: 404, json: { error: { code: 'resource_missing' } } };
				return {
					status: 200,
					json: { object: 'list', data: stripeBalanceTransactions(payout), has_more: false },
					hold: options.holdPayout === payout.id,
				};
			},
		},
	];
}
