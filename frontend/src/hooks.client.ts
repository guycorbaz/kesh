/**
 * SvelteKit client hooks — auth hydration.
 *
 * Restaure les tokens depuis localStorage avant le rendu de l'app.
 * Cela permet que les load() functions aient accès à authState.isAuthenticated
 * pour rediriger vers /login si nécessaire.
 */

import { authState } from '$lib/app/stores/auth.svelte';

// Restaurer les tokens IMMÉDIATEMENT au chargement de la page
authState.hydrate();
