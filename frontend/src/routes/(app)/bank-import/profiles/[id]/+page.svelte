<script lang="ts">
	import { page } from '$app/state';
	import BankProfileForm from '$lib/features/bank-import/BankProfileForm.svelte';
	import { getBankProfile } from '$lib/features/bank-import/bank-profile.api';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import type { BankProfile } from '$lib/features/bank-import/bank-profile.types';

	let profile = $state<BankProfile | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);

	// Story 29-9 (#655, AC6) — d'un profil à l'autre sans quitter la route. Défaut LATENT au
	// sol : aucun lien de l'application ne mène d'un profil à un autre (la liste et la création
	// mènent ici par une autre route, donc par un montage neuf) ; corrigé pour le jour où il y en
	// aura un.
	//
	// - L'effet dépend de l'identifiant, et de lui seul : SvelteKit RÉASSIGNE `page.params`
	//   (objet neuf) à toute navigation où `page_changed` est vrai, même à `id` égal (une query
	//   string, des données relues). Un `$derived` de la CHAÎNE compare par `===` et ne change
	//   pas : l'effet ne se rejoue qu'à `id` changé. Pas `Number(page.params.id)` : `NaN !== NaN`,
	//   un identifiant invalide relancerait l'effet à chaque réassignation.
	// - `isLoading = true` en tête : sans effet à id égal (l'effet ne se rejoue pas), nécessaire à
	//   id changé — `BankProfileForm` initialise ses champs UNE fois depuis `existing`, il faut
	//   donc le démonter, sans quoi le profil 6 s'afficherait avec les champs du 5.
	// - `loadSeq` : seule la dernière lecture lancée écrit `profile`, `errorMessage` et
	//   `isLoading` ; une réponse plus ancienne, succès comme échec, est ignorée en entier.
	//   Le numéro se prend AVANT la branche « ID invalide » : une lecture ancienne encore en
	//   vol ne doit pas l'écraser.
	// - ⚠️ L'effet exécute aussi la partie synchrone de `getBankProfile(id)` (C-29-9-18) : elle
	//   ne lit aujourd'hui aucun `$state` avant son premier `await` (`request` → `buildHeaders`
	//   → `fetchWithRetry`). Si elle venait à lire un signal (`authState`, par exemple), l'effet
	//   se rejouerait à son changement et `isLoading = true` démonterait le formulaire en
	//   perdant la saisie : le remède serait `untrack(() => getBankProfile(id))`.
	const rawId = $derived(page.params.id);
	let loadSeq = 0;

	$effect(() => {
		const mySeq = ++loadSeq;
		errorMessage = null;
		isLoading = true;
		const id = Number(rawId);
		if (!Number.isFinite(id)) {
			errorMessage = 'ID invalide';
			isLoading = false;
			return;
		}
		void getBankProfile(id)
			.then((p) => {
				if (mySeq !== loadSeq) return;
				profile = p;
				isLoading = false;
			})
			.catch((err) => {
				if (mySeq !== loadSeq) return;
				// Story 28-2 (#520) : « Profil introuvable » était faux pour une
				// panne réseau ; le message du serveur ou du client se suffit.
				errorMessage = errorMessageOf(err);
				isLoading = false;
			});
	});

	// Story 29-9 (#655, AC6, C-29-9-17) — une sauvegarde lancée sur le profil 5, qui se termine
	// après une navigation vers le 6, ne doit pas écrire `profile` : le formulaire monté (celui
	// du 6, initialisé une fois) recevrait `existing` = p5 en gardant les champs du 6, et la
	// sauvegarde suivante écrirait les valeurs du 6 dans le profil 5. La garde porte sur
	// l'identifiant COURANT, évalué à l'appel, et non sur un numéro capturé dans une fermeture
	// passée au formulaire : un composant Svelte 5 lit ses props en direct (`$$props.onSuccess`),
	// si bien que le formulaire démonté du 5 appellerait le gestionnaire courant.
	function onUpdated(updated: BankProfile): void {
		if (updated.id !== Number(rawId)) return;
		profile = updated;
	}
</script>

<h1 class="text-2xl font-semibold">
	{i18nMsg('bank-import-profile-labels-page-title-edit', 'Éditer le profil bancaire')}
</h1>

{#if isLoading}
	<p class="mt-4 text-text-muted">Chargement…</p>
{:else if errorMessage}
	<p class="mt-4 text-error">{errorMessage}</p>
{:else if profile}
	<div class="mt-6">
		<BankProfileForm existing={profile} onSuccess={onUpdated} />
	</div>
{/if}
