<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { modeState } from '$lib/app/stores/mode.svelte';
	import DegradedBanner from '$lib/shared/components/DegradedBanner.svelte';
	import { apiHealth } from '$lib/shared/utils/api-health.svelte';
	import { appVersion } from '$lib/shared/utils/app-version.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { loadI18nMessages } from '$lib/shared/utils/i18n.svelte';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();

	// Appliquer data-mode sur <html> pour activer les CSS custom properties
	// du mode Guidé/Expert. Safe car ssr = false (pas de SSR).
	// modeState.value est un getter réactif — $effect re-exécutera quand le mode change.
	$effect(() => {
		document.documentElement.setAttribute('data-mode', modeState.value);
	});

	// Story 10.3 (AC #14bis) : ping /health initial au boot. Si la DB est down
	// dès le 1er chargement (ex. /login après reboot NAS avec DB non encore
	// prête), bascule en état dégradé pour afficher immédiatement le banner —
	// sans attendre qu'un fetch API spontané échoue. Wrappé en try/catch
	// silencieux : un échec ici signifie "DB inaccessible" → setDegraded() qui
	// démarrera le polling périodique de recovery.
	//
	// AC #14ter : `AbortSignal.timeout(2000)` borne le fetch à 2s. Sans ce
	// timeout, un kesh-api qui accepte TCP mais hang HTTP (NAS sous OOM /
	// MariaDB lock) bloque indéfiniment la Promise — le catch ne fire jamais,
	// `setDegraded()` n'est pas appelé, l'utilisateur stare devant une UI sans
	// indication jusqu'au 1er fetch lazy qui timeout à 30s.
	onMount(async () => {
		// Pass 3 H3 : charger les traductions i18n côté root layout pour que
		// `DegradedBanner` (et tous les autres consumers `i18nMsg`) respectent
		// le locale browser. Pré-Pass 3, `loadI18nMessages()` n'était appelé
		// que par `/onboarding/+page.svelte` → tout autre route landing montrait
		// le fallback FR figé indépendamment de la langue utilisateur.
		//
		// **Gate auth obligatoire** : l'endpoint `/api/v1/i18n/messages` est
		// derrière `require_auth` (`crates/kesh-api/src/lib.rs:270`). Sans token,
		// le call déclencherait le path 401 → refresh → fail (pas de refresh
		// token) → `clearSession()` + `window.location.replace('/login?reason
		// =session_expired')` (cf. `api-client.ts:60-104`). Ce redirect casse
		// la navigation `/login` natural state (tests E2E + UX réelle).
		// Skip le call si pas authentifié — fallback FR figé reste affiché sur
		// `/login` (Swiss FR fallback acceptable v0.1). Limitation à corriger
		// v0.2 via endpoint i18n public.
		if (authState.isAuthenticated) {
			void loadI18nMessages();
		}

		try {
			const res = await fetch('/health', { signal: AbortSignal.timeout(2000) });
			// #159 : `/health` renvoie aussi `version` (même en 503 DB-down) — on la
			// capte avant le check d'état pour que le footer/login affiche la version
			// du binaire backend qui tourne, y compris quand la DB est inaccessible.
			const body = (await res.json().catch(() => ({}))) as { db?: unknown; version?: unknown };
			appVersion.set(body.version);
			if (!res.ok || body.db !== true) {
				apiHealth.setDegraded();
			}
		} catch {
			apiHealth.setDegraded();
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<DegradedBanner />

{@render children()}
<Toaster theme="light" />
