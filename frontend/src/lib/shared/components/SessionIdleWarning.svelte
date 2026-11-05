<!--
  Story v0174-session-2 (0.17.4, #895) — l'avertissement avant la fermeture de
  la session pour inactivité, avec « Rester connecté ».

  Composant à propriétés (C-v0174s-20) : le layout racine le rend quand
  `sessionIdle.warningRemainingMs` est un nombre, et lui passe le temps restant
  et l'action du bouton.

  - **Aucun déplacement du focus** (C-v0174s-11) : ramener le focus depuis
    l'aperçu PDF ferait recevoir à la fenêtre un `focus` compté comme activité,
    et l'avis se renouvellerait seul. Une phrase statique `role="alert"` est
    annoncée à l'apparition ; le compte à rebours est `aria-hidden`.
  - **Au-dessus d'un dialogue ouvert, sans le fermer** (C-v0174s-13) : `z-[60]`
    (le dialogue est `z-50`, monté après le layout), `pointer-events: auto`
    (`ScrollLock` de bits-ui pose `pointer-events: none` sur `<body>`), et un
    écouteur **natif** `pointerdown` en bouillonnement qui arrête la
    propagation : la couche du dialogue ne décide de se fermer qu'au
    bouillonnement sur `document`. Natif plutôt que `onpointerdown`, que
    Svelte 5 délègue à la racine de l'application. Le module d'inactivité,
    en capture sur `window`, a déjà noté l'activité.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	interface Props {
		/** Temps restant avant la fermeture (ms). */
		remainingMs: number;
		/** Action de « Rester connecté ». */
		onStay: () => void;
	}

	let { remainingMs, onStay }: Props = $props();

	/** `m:ss`, arrondi à la seconde supérieure (C-v0174s-15). */
	const remaining = $derived.by(() => {
		const secs = Math.max(0, Math.ceil(remainingMs / 1000));
		return `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, '0')}`;
	});

	let root: HTMLDivElement | undefined = $state();

	$effect(() => {
		const el = root;
		if (!el) return;
		const stop = (e: Event) => e.stopPropagation();
		el.addEventListener('pointerdown', stop);
		return () => el.removeEventListener('pointerdown', stop);
	});
</script>

<div
	bind:this={root}
	class="pointer-events-auto fixed top-3 left-1/2 z-[60] w-[min(36rem,calc(100vw-1.5rem))] -translate-x-1/2 rounded-md border border-amber-400 bg-amber-50 px-4 py-3 text-sm text-amber-950 shadow-lg"
	data-testid="session-idle-warning"
>
	<p role="alert" data-testid="session-idle-warning-announce">
		{i18nMsg(
			'session-idle-warning-announce',
			"Votre session va bientôt se fermer faute d'activité : cliquez sur « Rester connecté »."
		)}
	</p>
	<div class="mt-2 flex items-center justify-between gap-3">
		<span aria-hidden="true" class="font-medium tabular-nums" data-testid="session-idle-warning-countdown">
			{i18nMsg('session-idle-warning-countdown', 'Fermeture dans { $remaining }.', { remaining })}
		</span>
		<button
			type="button"
			class="rounded-md bg-amber-700 px-3 py-1.5 text-sm font-medium text-white hover:bg-amber-800 focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
			data-testid="session-idle-warning-stay"
			onclick={() => onStay()}
		>
			{i18nMsg('session-idle-warning-stay', 'Rester connecté')}
		</button>
	</div>
</div>
