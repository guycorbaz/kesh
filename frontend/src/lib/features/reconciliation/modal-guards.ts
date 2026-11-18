/**
 * Story 43-3 (#823, D4) — les gardes de concurrence d'une modale de réconciliation.
 *
 * Deux gardes, testées seules (`modal-guards.test.ts`) :
 *
 * - **`createSessionGuard`** — une session neuve à chaque ouverture, fermeture, changement de
 *   transaction et avant chaque soumission : une réponse lancée dans une session précédente
 *   n'agit jamais sur la modale courante (patron de `LinkSettlementModal`, revue P1 de la
 *   29-8, C-29-8-37).
 * - **`createLatestRequest`** — une recherche à la fois : délai anti-rebond, requête précédente
 *   avortée par son `AbortController`, et toute réponse dépassée (génération plus ancienne, ou
 *   signal avorté) ignorée, qu'elle arrive dans l'ordre ou non.
 *
 * Fabriques et fermetures, non des classes : la garde des libellés en dur
 * (`i18n-libelle-en-dur.test.ts`) n'inspecte pas les méthodes de classe et refuse toute classe
 * non admise dans `src/`.
 *
 * ⚠️ L'abandon est local : `apiClient` n'accepte pas de signal (choix C-e433-1), si bien que la
 * requête réseau va à son terme ; son résultat est jeté. Le signal est passé à la fonction de
 * chargement, qui pourra le transmettre le jour où le client HTTP le prendra.
 *
 * Employées par `GroupSettlementModal` seulement : `LinkSettlementModal` garde ses propres
 * compteurs en 0.19.0 (dette #1235).
 */

/** Le compteur de session d'une modale. Ni `$state` ni lu par un effet : il ne déclenche rien. */
export interface SessionGuard {
	/** Ouvre une session neuve et la rend ; toute session précédente est périmée. */
	next(): number;
	/** La session courante. */
	current(): number;
	/** `session` est-elle encore la session courante ? */
	isCurrent(session: number): boolean;
}

/** Une garde de session neuve. */
export function createSessionGuard(): SessionGuard {
	let session = 0;
	return {
		next: () => ++session,
		current: () => session,
		isCurrent: (s) => s === session,
	};
}

/** Ce que rend une requête lancée par `LatestRequest` à ses rappels. */
export interface LatestRequestHandlers<T> {
	/** La réponse de la requête la plus récente. */
	onResult: (value: T) => void;
	/** L'échec de la requête la plus récente. */
	onError: (error: unknown) => void;
	/** Appelé quand la requête part réellement (après le délai), pour l'état « chargement ». */
	onStart?: () => void;
}

/**
 * Une seule requête vivante à la fois. `run` annule le délai en attente et avorte la requête en
 * vol ; seule la plus récente rend son résultat. `cancel` fait de même sans rien lancer (à la
 * fermeture de la modale).
 */
export interface LatestRequest<T> {
	/**
	 * Lance `load` après le délai anti-rebond (`immediate` : sans délai). Toute requête
	 * précédente, en attente ou en vol, est périmée.
	 */
	run(
		load: (signal: AbortSignal) => Promise<T>,
		handlers: LatestRequestHandlers<T>,
		immediate?: boolean,
	): void;
	/** Périme tout : délai annulé, requête en vol avortée, réponse à venir ignorée. */
	cancel(): void;
	/** Une requête est-elle en attente ou en vol ? */
	busy(): boolean;
}

/** Une garde « dernière requête seulement », de délai anti-rebond `delayMs`. */
export function createLatestRequest<T>(delayMs: number): LatestRequest<T> {
	let gen = 0;
	let timer: ReturnType<typeof setTimeout> | null = null;
	let controller: AbortController | null = null;

	function supersede(): void {
		gen += 1;
		if (timer !== null) {
			clearTimeout(timer);
			timer = null;
		}
		if (controller !== null) {
			controller.abort();
			controller = null;
		}
	}

	function run(
		load: (signal: AbortSignal) => Promise<T>,
		handlers: LatestRequestHandlers<T>,
		immediate = false,
	): void {
		supersede();
		const mine = gen;
		const start = () => {
			timer = null;
			if (mine !== gen) return;
			const own = new AbortController();
			controller = own;
			handlers.onStart?.();
			let pending: Promise<T>;
			try {
				pending = load(own.signal);
			} catch (error) {
				pending = Promise.reject(error);
			}
			pending.then(
				(value) => {
					if (mine !== gen || own.signal.aborted) return;
					controller = null;
					handlers.onResult(value);
				},
				(error: unknown) => {
					if (mine !== gen || own.signal.aborted) return;
					controller = null;
					handlers.onError(error);
				},
			);
		};
		if (immediate || delayMs <= 0) {
			start();
		} else {
			timer = setTimeout(start, delayMs);
		}
	}

	return {
		run,
		cancel: supersede,
		busy: () => timer !== null || controller !== null,
	};
}
