/**
 * Debounce helper — pattern standard sans dépendance externe.
 *
 * Story 3.4 : utilisé pour les inputs text de filtre (description,
 * amountMin, amountMax) — évite un burst de requêtes réseau pendant
 * la frappe utilisateur.
 *
 * La fonction retournée expose une méthode `.cancel()` pour annuler
 * le timeout en cours — à appeler au démontage du composant Svelte
 * pour éviter un `loadFiltered()` sur un composant démonté.
 */

export interface DebouncedFn<Args extends unknown[]> {
	(...args: Args): void;
	/** Annule le timeout en cours — à appeler au démontage du composant. */
	cancel(): void;
	/**
	 * Exécute **tout de suite** l'appel en attente, s'il y en a un (sans effet
	 * sinon) — p. ex. avant un envoi qui ne doit pas partir sur un état que
	 * l'appel différé aurait changé (Story 37-3b, revue de code P1).
	 */
	flush(): void;
}

export function debounce<Args extends unknown[]>(
	fn: (...args: Args) => void,
	delay: number
): DebouncedFn<Args> {
	let timeoutId: ReturnType<typeof setTimeout> | null = null;
	let pendingArgs: Args | null = null;

	const debounced = ((...args: Args) => {
		if (timeoutId) clearTimeout(timeoutId);
		pendingArgs = args;
		timeoutId = setTimeout(() => {
			timeoutId = null;
			const a = pendingArgs as Args;
			pendingArgs = null;
			fn(...a);
		}, delay);
	}) as DebouncedFn<Args>;

	debounced.flush = () => {
		if (!timeoutId) return;
		clearTimeout(timeoutId);
		timeoutId = null;
		const a = pendingArgs as Args;
		pendingArgs = null;
		fn(...a);
	};

	debounced.cancel = () => {
		if (timeoutId) {
			clearTimeout(timeoutId);
			timeoutId = null;
		}
		pendingArgs = null;
	};

	return debounced;
}
