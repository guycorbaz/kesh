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
		/**
		 * Nom accessible du champ. Sans lui, le calcul du nom retombe sur le
		 * `placeholder` — lequel vaut ici « … (défaut société) » sur **toutes** les
		 * lignes : les 200 champs d'une facture s'annonçaient tous du même nom, y
		 * compris ceux portant un compte explicite différent. (Passe 2 de revue.)
		 */
		ariaLabel?: string;
		/**
		 * `id` rendu sur le champ de saisie, pour qu'un `<label for>` extérieur
		 * puisse s'y lier (Story 16-2b, #144).
		 *
		 * **Prop opt-in dont le défaut préserve le comportement** : `undefined`
		 * ⇒ aucun attribut `id`, les cinq écrans consommateurs sont inchangés.
		 *
		 * ⚠️ Sans elle, un `<label for="…">` écrit dans la page ne se lie **à
		 * rien** — et le nom accessible du champ retombe alors sur `ariaLabel`,
		 * voire sur le repli `journal-entry-form-col-account`, ce qui donnerait
		 * un libellé « Compte » venu d'une clé *journal-entry* sur une fiche
		 * produit. Le patron de `VatPurchaseAssistant.svelte:156-158` porte
		 * exactement ce défaut, et `svelte-check` ne le voit pas : la règle
		 * `a11y_label_has_associated_control` se satisfait de la présence de
		 * l'attribut `for`, elle ne résout pas les `id` à travers un composant.
		 */
		id?: string;
		/**
		 * `id` d'un élément extérieur décrivant le champ, joint au nom accessible
		 * via `aria-describedby` (Story 16-2b, #144, passe 1 de revue).
		 *
		 * **Prop opt-in dont le défaut préserve le comportement** : `undefined`
		 * ⇒ l'attribut ne porte que le message d'invalidité, comme avant.
		 *
		 * ⚠️ Le `<label for>` seul ne suffit pas quand la valeur **vide** a un
		 * sens : sur la fiche produit, c'est la phrase d'aide — et elle seule —
		 * qui dit que laisser le champ vide n'est pas un oubli mais « suivre le
		 * défaut société ». Sans cette prop, ce sens n'existe qu'à la vue.
		 *
		 * Se compose avec le message d'invalidité : les deux `id` sont joints par
		 * une espace quand `markInvalid` est actif, comme la spec ARIA le prévoit.
		 */
		describedBy?: string;
	}

	let {
		id = undefined,
		describedBy = undefined,
		accounts,
		value,
		loadError = false,
		disabled = false,
		onSelect,
		allowClear = false,
		markInvalid = false,
		requiredAccountType = undefined,
		postableExemptAccountId = null,
		placeholder = undefined,
		ariaLabel = undefined
	}: Props = $props();

	// Identifiant stable pour lier le message d'invalidité au champ. `$props.id()`
	// et non `crypto.randomUUID()` : ce dernier est `undefined` hors contexte
	// sécurisé, donc sur un déploiement LAN en HTTP (cf. NAS Synology).
	const uid = $props.id();
	const invalidMsgId = `account-invalid-${uid}`;

	// ── LE TEXTE AFFICHÉ EST DÉRIVÉ DE LA VALEUR, PLUS UN ÉTAT MUTABLE ────────
	//
	// Refonte de la passe 2 de revue (arbitrage Guy). La conception antérieure
	// gardait le texte en `$state`, écrit à la fois par un `$effect` (depuis
	// `value`) et par la frappe, puis « réconcilié » au `blur` avec 150 ms de
	// retard. Elle a produit DEUX pertes de données silencieuses que trois tours
	// de patch n'ont pas fermées :
	//
	//   1. vider le champ puis cliquer « Enregistrer » — le formulaire lisait la
	//      ligne AVANT que la réconciliation différée ne s'exécute, et persistait
	//      l'ANCIEN compte pendant que l'écran affichait « suit le défaut
	//      société ». Écriture comptable fausse, avec l'UI qui affirmait
	//      activement le contraire ;
	//   2. un `blur` AVANT l'arrivée de la liste des comptes — le champ était
	//      vide faute d'avoir pu résoudre le libellé, ce que la réconciliation
	//      prenait pour un effacement volontaire : le compte disparaissait sans
	//      aucun geste destructif de l'utilisateur.
	//
	// La conception actuelle SUPPRIME la fenêtre au lieu de la garder :
	//
	//   - `displayLabel` est **dérivé** de `value` : il ne peut pas la contredire ;
	//   - `draft` porte le texte transitoire pendant la frappe, `null` hors
	//     édition ;
	//   - le `blur` est **immédiat** (plus de `setTimeout`) et ne peut agir que si
	//     `draft !== null`, c'est-à-dire si l'utilisateur a réellement tapé. Un
	//     simple passage de focus ne touche plus rien.
	//
	// Ce qui rend le commit immédiat sûr : un clic sur un item du dropdown ne
	// déclenche PAS de `blur`, son `onmousedown` appelant `preventDefault()`.
	// Le délai de 150 ms n'avait d'autre rôle que de contourner cette course —
	// il devient inutile, et avec lui le timer jamais nettoyé.

	/** Le compte désigné par `value`, résolu sur la liste COMPLÈTE (cf. D11). */
	const selected = $derived(value === null ? undefined : accounts.find((a) => a.id === value));

	/** Libellé canonique d'une valeur — source unique de la mise en forme. */
	function labelOf(acc: AccountResponse | undefined): string {
		return acc ? `${acc.number} — ${acc.name}` : '';
	}

	/**
	 * Texte que le champ doit afficher pour la valeur liée.
	 *
	 * En mode dégradé, l'identifiant persisté plutôt que du vide : sans lui, une
	 * ligne DÉJÀ ventilée paraissait vierge, et l'utilisateur écrasait un compte
	 * sans savoir qu'il en écrasait un.
	 */
	const displayLabel = $derived.by(() => {
		if (value === null) return '';
		if (loadError) return String(value);
		return labelOf(selected);
	});

	/** Texte en cours de frappe. `null` = aucune édition en cours. */
	let draft = $state<string | null>(null);
	let open = $state(false);
	let highlightIndex = $state(0);

	/** Ce que l'input montre : le brouillon s'il existe, sinon la vérité. */
	const query = $derived(draft ?? displayLabel);

	// Une valeur venue de l'extérieur annule le brouillon : le champ ne peut pas
	// continuer d'afficher une saisie qui ne correspond plus à rien. Reproduit le
	// comportement de l'ancien `$effect` pour les 4 consommateurs existants.
	let lastValue = $state<number | null | undefined>(undefined);
	$effect(() => {
		if (lastValue !== value) {
			lastValue = value;
			draft = null;
		}
	});

	// Story 14-3b : sélecteur de SAISIE d'écriture → seuls les comptes postables
	// (le backend rejette désormais une ligne manuelle vers un compte non-postable).
	// `requiredAccountType` filtre AUSSI les propositions, pas seulement le
	// marqueur : sans cela, le sélecteur de compte de PRODUIT proposait « 4000 —
	// Achats », et le sélectionner marquait aussitôt la ligne invalide en
	// désactivant l'enregistrement. L'interface conduisait l'utilisateur dans
	// l'état bloquant qu'elle venait de créer. (Passe 2 de revue.)
	// `undefined` par défaut : les 4 consommateurs existants sont inchangés.
	const active = $derived(
		accounts.filter(
			(a) =>
				a.active &&
				a.postable &&
				(requiredAccountType === undefined || a.accountType === requiredAccountType)
		)
	);

	/**
	 * Story 16-1b (AC2) — la valeur persistée est-elle hors de ce que le champ
	 * accepterait aujourd'hui ? Vrai seulement si le compte est **résolu** : une
	 * valeur non résoluble n'a rien à qualifier.
	 */
	const isInvalid = $derived.by(() => {
		if (!markInvalid || loadError) return false;
		// Règle déléguée à `account-validity` : `InvoiceForm` applique EXACTEMENT
		// la même pour décider du blocage. Un champ marqué sans blocage, ou
		// l'inverse, serait pire que l'absence de fonctionnalité.
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
		draft = target.value;
		open = true;
		highlightIndex = 0;

		// Mode dégradé (14-3b) : l'utilisateur saisit un ID numérique directement.
		if (loadError) {
			const n = Number(target.value);
			onSelect(Number.isFinite(n) && n > 0 ? n : null);
		}
	}

	function handleSelect(acc: AccountResponse) {
		draft = null; // le libellé retombe sur `displayLabel`
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
	 * Commit du brouillon au `blur` — **immédiat**, et seulement s'il y a eu
	 * édition (AC1-bis).
	 *
	 * `draft === null` signifie « l'utilisateur n'a rien tapé » : un simple
	 * passage de focus ne peut donc plus rien effacer. C'est ce qui ferme le
	 * chemin n°2 décrit en tête de fichier.
	 */
	function handleBlur() {
		open = false;

		// Champ obligatoire : on ne nullifie JAMAIS au blur (dette #271), et le
		// brouillon est conservé tel quel — comportement des 4 consommateurs
		// existants strictement inchangé.
		if (!allowClear || loadError) return;
		if (draft === null) return;

		if (draft.trim() === '') {
			// Effacement explicite — mais SEULEMENT si le champ montrait
			// réellement quelque chose. Un champ vide parce que la liste des
			// comptes n'est pas encore arrivée n'est pas une intention.
			if (value !== null && selected !== undefined) onSelect(null);
			draft = null;
			return;
		}

		// Texte libre jamais validé : on le jette, l'affichage retombe sur la
		// vérité dérivée. Le champ ne peut donc jamais contredire `value`.
		draft = null;
	}
</script>

<div class="relative">
	<Input
		{id}
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
		aria-label={ariaLabel ?? i18nMsg('journal-entry-form-col-account', 'Compte')}
		aria-autocomplete="list"
		aria-expanded={open}
		aria-invalid={isInvalid ? 'true' : undefined}
		aria-describedby={[describedBy, isInvalid ? invalidMsgId : undefined]
			.filter(Boolean)
			.join(' ') || undefined}
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
				// Jeter le brouillon suffit : l'affichage retombe sur `displayLabel`,
				// qui vaut `''` quand la valeur est nulle. Le bouton n'écrit donc
				// jamais le texte canonique — il n'y en a plus qu'un, dérivé.
				//
				// ASYMÉTRIE VOULUE avec le `blur` : celui-ci exige en plus que le
				// compte soit RÉSOLUBLE (`selected !== undefined`), le bouton non.
				// Un champ vide au `blur` est ambigu — il peut l'être parce que la
				// liste des comptes n'est pas arrivée. Un clic sur « Effacer » ne
				// l'est pas : c'est un geste explicite, il doit aboutir même sur un
				// compte que l'on ne sait pas nommer.
				// (Asymétrie relevée non documentée en passe 3 de revue.)
				draft = null;
				if (value !== null) onSelect(null);
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
