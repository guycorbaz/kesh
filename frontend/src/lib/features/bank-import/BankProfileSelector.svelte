<!--
  Story 8-3 KF #70 — Sélecteur de profil bancaire CSV.

  Affiche un `<select>` listant tous les profils disponibles pour la
  company courante (chargés via `GET /api/v1/bank-profiles`). Pré-sélectionne
  l'`autoMatchedId` retourné par le preview backend (Pass auto-match
  filename regex côté API). L'utilisateur peut explicitement changer le
  profil ; la valeur est ensuite envoyée au backend comme
  `bankProfileId` dans le multipart `POST /preview` et `POST /bank-imports`.
-->
<script lang="ts">
	import type { BankProfile } from './bank-profile.types';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';

	type Props = {
		profiles: BankProfile[];
		autoMatchedId: number | null;
		value: number | null;
		onChange: (id: number | null) => void;
	};
	let { profiles, autoMatchedId, value, onChange }: Props = $props();

	function handleChange(event: Event): void {
		const target = event.currentTarget as HTMLSelectElement;
		const raw = target.value;
		if (raw === '') {
			onChange(null);
		} else {
			onChange(Number(raw));
		}
	}
</script>

<label class="block text-sm font-medium" for="bank-profile-select">
	{i18nMsg('bank-import-labels-bank-profile-selector', 'Profil bancaire CSV')}
</label>
<select
	id="bank-profile-select"
	data-testid="bank-profile-select"
	value={value ?? ''}
	onchange={handleChange}
	class="mt-1 block w-full rounded border-border bg-surface p-2 text-text"
>
	<!-- L6 (Pass 1 review) — clés i18n distinctes pour le placeholder
	     "Auto-détection" (label de l'option vide) et l'annotation
	     "(auto-détecté)" affichée après le nom du profil match — les
	     deux textes ont des sémantiques différentes (placeholder vs
	     annotation parenthétique) et doivent pouvoir être traduits
	     indépendamment. -->
	<option value="">— {i18nMsg('bank-import-labels-bank-profile-auto-detect-placeholder', 'Auto-détection')} —</option>
	{#each profiles as profile (profile.id)}
		<option value={profile.id}>
			{profile.bankName}{#if autoMatchedId === profile.id}
				{' '}({i18nMsg('bank-import-labels-bank-profile-auto-matched', 'auto-détecté')})
			{/if}
		</option>
	{/each}
</select>
