<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	// AC3 (Story 16-1b) : emplacement canonique du runtime i18n. L'import
	// passait par `features/onboarding`, couplage transverse que
	// `shared/utils/i18n.svelte.ts:1-7` proscrit explicitement.
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse, AccountType } from '$lib/features/accounts/accounts.types';
	import { isAccountUnusable } from '$lib/features/accounts/account-validity';

	interface Props {
		accounts: AccountResponse[];
		value: number | null;
		loadError?: boolean;
		disabled?: boolean;
		onSelect: (id: number | null) => void;
		/**
		 * Story 16-1b (D8/AC1) — affiche un bouton d'effacement et active la
		 * sémantique « champ vidé au clavier = effacement explicite » (AC1-bis).
		 *
		 * **Défaut `false` : les 4 consommateurs existants sont stricts inchangés.**
		 * Ne PAS l'activer sur un champ obligatoire — `JournalEntryForm` se
		 * mettrait à nullifier ses lignes au `blur`, soit la dette #271.
		 */
		allowClear?: boolean;
		/**
		 * Story 16-1b (D7/AC2) — marque textuellement une valeur persistée qui
		 * n'est plus sélectionnable (compte archivé, non imputable, ou de type
		 * inattendu), **sans** effacer son libellé.
		 *
		 * **Défaut `false`** : `JournalEntryForm` et les modales de rapprochement
		 * sélectionnent massivement des comptes `Expense` / `Asset` / `Liability` —
		 * un marqueur inconditionnel s'afficherait sur presque toutes leurs lignes.
		 */
		markInvalid?: boolean;
		/** Type de compte attendu ; `undefined` = aucun contrôle de type (AC2). */
		requiredAccountType?: AccountType;
		/**
		 * Compte exempté du seul critère `postable` (miroir de 16-1a D3-bis).
		 *
		 * Le compte de produit **par défaut de la société** peut devenir
		 * non-imputable sans intention. Le backend l'accepte quand même ; si le
		 * frontend le marquait invalide et bloquait l'enregistrement, l'utilisateur
		 * serait enfermé — **le frontend ne doit jamais bloquer ce que 16-1a
		 * accepte**. Les critères `active` et `requiredAccountType` restent, eux,
		 * appliqués à ce compte.
		 */
		postableExemptAccountId?: number | null;
		/**
		 * Remplace le placeholder par défaut (« Compte »). `undefined` = inchangé.
		 *
		 * Story 16-1b D9 : le formulaire de facture y nomme le **compte par défaut
		 * de la société**, parce que le sens de `null` est « suivre le défaut », pas
		 * « aucun compte ». Un champ vide laisserait croire à un oubli.
		 */
		placeholder?: string;
	}

	let {
		accounts,
		value,
		loadError = false,
		disabled = false,
		onSelect,
		allowClear = false,
		markInvalid = false,
		requiredAccountType = undefined,
		postableExemptAccountId = null,
		placeholder = undefined
	}: Props = $props();

	// Identifiant stable pour lier le message d'invalidité au champ. `$props.id()`
	// et non `crypto.randomUUID()` : ce dernier est `undefined` hors contexte
	// sécurisé, donc sur un déploiement LAN en HTTP (cf. NAS Synology).
	const uid = $props.id();
	const invalidMsgId = `account-invalid-${uid}`;

	let query = $state('');
	let open = $state(false);
	let highlightIndex = $state(0);

	// Lorsqu'une valeur arrive depuis l'extérieur, afficher le compte correspondant.
	$effect(() => {
		if (value !== null && !loadError) {
			const acc = accounts.find((a) => a.id === value);
			if (acc) {
				query = `${acc.number} — ${acc.name}`;
			}
		} else if (value === null) {
			query = '';
		}
	});

	// Story 14-3b : sélecteur de SAISIE d'écriture → seuls les comptes postables
	// (le backend rejette désormais une ligne manuelle vers un compte non-postable).
	const active = $derived(accounts.filter((a) => a.active && a.postable));

	/** Le compte désigné par `value`, résolu sur la liste COMPLÈTE (cf. D11). */
	const selected = $derived(value === null ? undefined : accounts.find((a) => a.id === value));

	/** Libellé canonique d'une valeur — source unique de la mise en forme. */
	function labelOf(acc: AccountResponse | undefined): string {
		return acc ? `${acc.number} — ${acc.name}` : '';
	}

	/**
	 * Story 16-1b (AC2) — la valeur persistée est-elle hors de ce que le champ
	 * accepterait aujourd'hui ? Vrai seulement si le compte est **résolu** : une
	 * valeur non résoluble n'a rien à qualifier (et, avec `fetchAccounts(true)`,
	 * ne se produit pas).
	 */
	const isInvalid = $derived.by(() => {
		if (!markInvalid || loadError) return false;
		// Règle déléguée à `account-validity` : `InvoiceForm` applique EXACTEMENT la
		// même pour décider du blocage d'enregistrement. Un champ marqué sans
		// blocage, ou l'inverse, serait pire que l'absence de fonctionnalité.
		return isAccountUnusable(selected, { requiredAccountType, postableExemptAccountId });
	});

	const filtered = $derived.by(() => {
		if (loadError) return [];
		const q = query.trim().toLowerCase();
		if (q === '') return active.slice(0, 20);
		return active
			.filter(
				(a) =>
					a.number.toLowerCase().startsWith(q) ||
					a.name.toLowerCase().includes(q)
			)
			.slice(0, 20);
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		query = target.value;
		open = true;
		highlightIndex = 0;

		// En mode fallback (loadError), l'utilisateur saisit un ID numérique directement.
		if (loadError) {
			const n = Number(target.value);
			onSelect(Number.isFinite(n) && n > 0 ? n : null);
		}
	}

	function handleSelect(acc: AccountResponse) {
		query = `${acc.number} — ${acc.name}`;
		open = false;
		onSelect(acc.id);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (loadError) return;

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			open = true;
			highlightIndex = Math.min(highlightIndex + 1, filtered.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			highlightIndex = Math.max(highlightIndex - 1, 0);
		} else if (e.key === 'Enter') {
			if (open && filtered[highlightIndex]) {
				e.preventDefault();
				handleSelect(filtered[highlightIndex]);
			}
		} else if (e.key === 'Escape') {
			open = false;
		}
	}

	/**
	 * Story 16-1b (AC1-bis) — réconcilie le texte affiché avec la valeur liée.
	 *
	 * **Le champ ne peut jamais afficher un texte qui contredit `value`.** Sans
	 * cela, sur un champ optionnel, le geste naturel pour revenir au défaut
	 * société (tout sélectionner + `Suppr`) laissait `value` inchangée : le champ
	 * vide déclenchait le placeholder « … (défaut société) » et l'utilisateur
	 * enregistrait convaincu d'avoir remis la ligne au défaut, alors que la
	 * facture se postait sur l'ancien compte. Écriture fausse en silence, avec
	 * l'UI qui affirmait le contraire.
	 *
	 * Inactif quand `allowClear` est `false` : un champ obligatoire ne doit pas
	 * se nullifier au `blur` (dette #271).
	 */
	function commitOnBlur() {
		if (!allowClear || loadError) return;

		if (query.trim() === '') {
			// Champ vidé par l'utilisateur = effacement explicite.
			if (value !== null) onSelect(null);
			return;
		}

		// Texte libre jamais validé par une sélection → restaurer la vérité.
		if (value === null) {
			query = '';
		} else {
			const label = labelOf(selected);
			if (label !== '' && query !== label) query = label;
		}
	}

	function handleBlur() {
		// Délai pour permettre un clic sur un item du dropdown. La réconciliation
		// est faite DANS le délai, donc après une éventuelle sélection : si
		// l'utilisateur a cliqué un item, `query` et `value` concordent déjà et
		// `commitOnBlur` est un no-op — d'où « `onSelect(null)` une seule fois ».
		setTimeout(() => {
			open = false;
			commitOnBlur();
		}, 150);
	}
</script>

<div class="relative">
	<Input
		type="text"
		value={query}
		oninput={handleInput}
		onkeydown={handleKeydown}
		onfocus={() => (open = true)}
		onblur={handleBlur}
		{disabled}
		placeholder={loadError
			? i18nMsg('account-autocomplete-unavailable', 'Autocomplétion indisponible — saisir l\'ID du compte')
			: (placeholder ?? i18nMsg('journal-entry-form-col-account', 'Compte'))}
		aria-autocomplete="list"
		aria-expanded={open}
		aria-invalid={isInvalid ? 'true' : undefined}
		aria-describedby={isInvalid ? invalidMsgId : undefined}
	/>

	{#if allowClear && !disabled && (value !== null || query !== '')}
		<!--
			AC1 : le bouton appelle `onSelect(null)` et NE touche PAS `query` —
			c'est le `$effect` (`value === null` → `query = ''`) qui en est la
			source unique. Une double écriture divergerait si le parent refusait
			la mise à jour.

			`onmousedown` + `preventDefault` empêche la perte de focus (même idiome
			que les items du dropdown), ce qui évite la course avec `handleBlur` ;
			`onclick` porte l'action pour que le clavier fonctionne aussi.
		-->
		<button
			type="button"
			class="absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-muted-foreground hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
			aria-label={i18nMsg('common-account-clear', 'Effacer le compte sélectionné')}
			onmousedown={(e) => e.preventDefault()}
			onclick={() => {
				// `value` est la source de vérité : on la remet à `null` et le
				// `$effect` s'occupe de `query`. MAIS si elle vaut DÉJÀ `null`, le
				// parent ne change rien, le `$effect` ne se redéclenche pas, et le
				// `preventDefault` ci-dessus a supprimé le `blur` qui aurait
				// réconcilié le texte : le champ resterait sur une saisie libre
				// contredisant la valeur liée. On vide alors explicitement.
				// (Convergence Blind Hunter + Edge Case Hunter, passe 1 de revue.)
				if (value === null) query = '';
				else onSelect(null);
			}}
		>
			<span aria-hidden="true">×</span>
		</button>
	{/if}

	{#if isInvalid}
		<!--
			Marqueur TEXTUEL : le signal ne doit pas reposer sur la seule couleur.
			`id` + `aria-describedby` sur le champ : sans ce lien, un lecteur d'écran
			annonce « champ invalide » sans jamais énoncer POURQUOI (WCAG 3.3.1).
		-->
		<p id={invalidMsgId} class="mt-1 text-xs text-destructive">
			{i18nMsg('common-account-invalid', 'Compte invalide — non imputable, archivé ou de type inattendu')}
		</p>
	{/if}

	{#if open && !loadError && filtered.length > 0}
		<ul
			class="absolute z-20 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border bg-popover shadow-md"
			role="listbox"
		>
			{#each filtered as acc, i (acc.id)}
				<li
					class="cursor-pointer px-3 py-2 text-sm hover:bg-accent"
					class:bg-accent={i === highlightIndex}
					onmousedown={(e) => {
						e.preventDefault();
						handleSelect(acc);
					}}
					role="option"
					aria-selected={i === highlightIndex}
				>
					<span class="font-mono text-xs mr-2">{acc.number}</span>
					<span>{acc.name}</span>
				</li>
			{/each}
		</ul>
	{/if}
</div>
