/**
 * SvelteKit client hooks — auth hydration.
 *
 * Restaure les tokens depuis localStorage AVANT que le contenu de la page
 * ne soit rendu. Cela permet que les load() functions aient accès à
 * authState.isAuthenticated pour rediriger vers /login si nécessaire.
 *
 * L'appel synchrone ici exécute AVANT toute render/load phase (vérifié par
 * la positio du hook dans l'ordre d'initialisation Svelte 5).
 */

import { authState } from '$lib/app/stores/auth.svelte';

// Restaurer les tokens IMMÉDIATEMENT au chargement du client
// Exécution synchrone, avant load() functions + render
authState.hydrate();
