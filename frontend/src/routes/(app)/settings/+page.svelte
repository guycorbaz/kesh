<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Separator } from '$lib/components/ui/separator';
	import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
	import { lockBooks, releaseBooksLock } from '$lib/features/settings/settings.api';
	import {
		fetchCompanyCurrent,
		updateCompanyAddress,
		updateCompanyContactDetails,
		updateCompanyEmail
	} from '$lib/features/settings/settings.api';
	import type { CompanyCurrentResponse, CompanyJson } from '$lib/features/settings/settings.types';
	import {
		ADDRESS_NORMALIZERS,
		companyEditConflict,
		conflictNotice,
		type CompanyConflictNoticeState,
		type CompanyEditConflict,
		type CompanyFieldNormalizer,
	} from '$lib/features/settings/company-conflict';
	import CompanyConflictNotice from '$lib/features/settings/CompanyConflictNotice.svelte';
	import SessionIdleSection from '$lib/features/settings/SessionIdleSection.svelte';
	import { bankAccountName } from '$lib/features/bank-accounts/format';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { isPlausibleEmail } from '$lib/shared/utils/email';
	import { notifySuccess } from '$lib/shared/utils/notify';

	// ⚠️ Le troisième argument porte les variables Fluent. Il passe par
	// `i18nMsg` et JAMAIS par un `.replace()` maison : le backend pré-résout
	// Fluent sans arguments, si bien qu'un `{ $date }` ressort entouré des
	// marques d'isolation bidi U+2068/U+2069 — invisibles à l'œil, fatales à
	// toute assertion E2E portant sur la valeur.
	function msg(
		key: string,
		fallback: string,
		args?: Record<string, string | number>
	): string {
		return i18nMsg(key, fallback, args);
	}

	let data = $state<CompanyCurrentResponse | null>(null);
	let loading = $state(true);

	// --- Story 34-6 (#953) : le conflit de version des trois éditions de la société ---
	//
	// Chaque bloc garde, à l'ouverture de l'édition, une copie de ses valeurs enregistrées
	// (`*Base`). Sur un 409, `reloadAfterConflict` relit la société et la juge contre cette
	// copie et la saisie (`companyEditConflict`) : les champs non modifiés reprennent la
	// valeur relue, ceux modifiés des deux côtés sont montrés, et la copie prend les valeurs
	// relues, si bien qu'un second conflit se compare à elles. Avant, la saisie restait et
	// le second clic remplaçait, sans l'avoir montrée, l'écriture concurrente.

	/** Valeurs du bloc e-mail, telles que le formulaire les porte. */
	const EMAIL_FIELDS = ['email'] as const;
	function emailValuesOf(c: CompanyJson): Record<(typeof EMAIL_FIELDS)[number], string> {
		return { email: c.email ?? '' };
	}

	/** Valeurs du bloc téléphone et site. */
	const CONTACT_FIELDS = ['phone', 'website'] as const;
	function contactValuesOf(c: CompanyJson): Record<(typeof CONTACT_FIELDS)[number], string> {
		return { phone: c.phone ?? '', website: c.website ?? '' };
	}

	/**
	 * Valeurs du bloc adresse. `addressStructured` peut manquer après relecture (adresse
	 * retirée ailleurs) : chaque composant vaut alors vide.
	 */
	const ADDRESS_FIELDS = ['street', 'building', 'postalCode', 'city', 'country'] as const;
	function addressValuesOf(c: CompanyJson): Record<(typeof ADDRESS_FIELDS)[number], string> {
		const current = c.addressStructured;
		return {
			street: current?.street ?? '',
			building: current?.building ?? '',
			postalCode: current?.postalCode ?? '',
			city: current?.city ?? '',
			// Une société d'avant les composants n'a pas de pays : CH est le défaut du serveur.
			country: current?.country?.trim() || 'CH',
		};
	}

	type ConflictOutcome<K extends string> =
		| { status: 'reload-failed' }
		| {
				status: 'reloaded';
				/** La nouvelle copie : les valeurs relues. */
				base: Record<K, string>;
				conflict: CompanyEditConflict<K>;
				notice: CompanyConflictNoticeState | null;
		  };

	/**
	 * Relit la société après un 409 et juge le conflit d'un bloc. `data` prend la société
	 * relue. Seul un échec de la relecture rend `reload-failed`.
	 */
	async function reloadAfterConflict<K extends string>(
		base: Record<K, string>,
		input: Record<K, string>,
		fields: readonly K[],
		valuesOf: (c: CompanyJson) => Record<K, string>,
		normalizers: Partial<Record<K, CompanyFieldNormalizer>> = {},
	): Promise<ConflictOutcome<K>> {
		let fresh: CompanyCurrentResponse;
		try {
			fresh = await fetchCompanyCurrent();
		} catch {
			return { status: 'reload-failed' };
		}
		data = fresh;
		const stored = valuesOf(fresh.company);
		const conflict = companyEditConflict(base, stored, input, fields, normalizers);
		return { status: 'reloaded', base: stored, conflict, notice: conflictNotice(conflict, stored) };
	}

	function alreadySavedMessage(): string {
		return msg('settings-company-conflict-already-saved', 'Ces valeurs étaient déjà enregistrées.');
	}

	// Story 20-3b2 — édition inline de l'e-mail société (Reply-To), Admin-only.
	let isAdmin = $derived(authState.currentUser?.role === 'Admin');
	let emailEditing = $state(false);
	let emailValue = $state('');
	let emailSubmitting = $state(false);
	let emailError = $state('');
	let emailBase = $state(emailValuesOf({ email: null } as CompanyJson));
	let emailConflict = $state<CompanyConflictNoticeState | null>(null);

	function startEmailEdit() {
		if (data) emailBase = emailValuesOf(data.company);
		emailValue = emailBase.email;
		emailError = '';
		emailConflict = null;
		emailEditing = true;
	}

	function closeEmailEdit() {
		emailEditing = false;
		emailError = '';
		emailConflict = null;
	}

	async function saveEmail() {
		if (!data) return;
		const trimmed = emailValue.trim();
		if (trimmed && !isPlausibleEmail(trimmed)) {
			emailError = msg('settings-company-email-invalid', 'Adresse e-mail invalide.');
			return;
		}
		emailSubmitting = true;
		emailError = '';
		try {
			// Champ vidé = effacement (Reply-To omis à l'envoi).
			const updated = await updateCompanyEmail({
				email: trimmed || null,
				version: data.company.version,
			});
			data = { ...data, company: updated };
			closeEmailEdit();
			notifySuccess(msg('settings-company-email-saved', 'E-mail de la société enregistré'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// Conflit de version : relire et juger (Story 34-6). Si la relecture
					// échoue (review P3 BH), ne pas prétendre « rechargées » — la version
					// locale reste périmée, re-cliquer redonnerait le même 409 : inviter à
					// recharger la page.
					const r = await reloadAfterConflict(
						emailBase,
						{ email: emailValue },
						EMAIL_FIELDS,
						emailValuesOf,
					);
					if (r.status === 'reload-failed') {
						emailConflict = null;
						emailError = msg(
							'settings-company-email-conflict-reload-failed',
							'Conflit de version et rechargement impossible — rechargez la page.',
						);
					} else if (r.conflict.kind === 'already-saved') {
						emailBase = r.base;
						closeEmailEdit();
						notifySuccess(alreadySavedMessage());
					} else {
						emailBase = r.base;
						emailValue = r.conflict.input.email;
						emailConflict = r.notice;
						if (!r.notice) {
							emailError = msg(
								'settings-company-email-conflict',
								'Conflit de version — les données ont été rechargées, réessayez.',
							);
						}
					}
				} else {
					emailError = err.message;
				}
			} else {
				// `common-error` = clé générique existante (review P1 : éviter le
				// doublon avec `error-unexpected`, gardée en FTL pour la page
				// contacts qui la consommait sans définition).
				emailError = msg('common-error', 'Erreur inattendue');
			}
		} finally {
			emailSubmitting = false;
		}
	}

	// Story 16-3a (#151) — téléphone et site web, rendus sur le PDF de facture.
	// Édition inline sur le patron exact de l'e-mail ci-dessus : c'est le SEUL
	// chemin de saisie pour une instance déjà installée, l'onboarding ne se
	// rejouant pas.
	let contactEditing = $state(false);
	let phoneValue = $state('');
	let websiteValue = $state('');
	let contactSubmitting = $state(false);
	let contactError = $state('');
	let contactBase = $state(contactValuesOf({ phone: null, website: null } as CompanyJson));
	let contactConflict = $state<CompanyConflictNoticeState | null>(null);

	function startContactEdit() {
		if (data) contactBase = contactValuesOf(data.company);
		phoneValue = contactBase.phone;
		websiteValue = contactBase.website;
		contactError = '';
		contactConflict = null;
		contactEditing = true;
	}

	function closeContactEdit() {
		contactEditing = false;
		contactError = '';
		contactConflict = null;
	}

	async function saveContactDetails() {
		if (!data) return;
		contactSubmitting = true;
		contactError = '';
		try {
			// Champ vidé = effacement : la ligne disparaît du PDF (D2).
			const updated = await updateCompanyContactDetails({
				phone: phoneValue.trim() || null,
				website: websiteValue.trim() || null,
				version: data.company.version,
			});
			data = { ...data, company: updated };
			closeContactEdit();
			notifySuccess(msg('settings-company-contact-saved', 'Coordonnées enregistrées'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					// ⚠️ Clés DÉDIÉES au bloc coordonnées, et non celles du bloc
					// e-mail : les deux libellés sont génériques aujourd'hui, mais
					// les partager ferait s'afficher ici toute reformulation
					// destinée au contexte e-mail, sans qu'aucun test ne le voie.
					// (Passe 6 de revue.) L'avis de valeurs, lui, est commun aux trois
					// blocs : il nomme le champ (Story 34-6, C-e343-7).
					const r = await reloadAfterConflict(
						contactBase,
						{ phone: phoneValue, website: websiteValue },
						CONTACT_FIELDS,
						contactValuesOf,
					);
					if (r.status === 'reload-failed') {
						contactConflict = null;
						contactError = msg(
							'settings-company-contact-conflict-reload-failed',
							'Conflit de version et rechargement impossible — rechargez la page.',
						);
					} else if (r.conflict.kind === 'already-saved') {
						contactBase = r.base;
						closeContactEdit();
						notifySuccess(alreadySavedMessage());
					} else {
						contactBase = r.base;
						phoneValue = r.conflict.input.phone;
						websiteValue = r.conflict.input.website;
						contactConflict = r.notice;
						if (!r.notice) {
							contactError = msg(
								'settings-company-contact-conflict',
								'Conflit de version — les données ont été rechargées, réessayez.',
							);
						}
					}
				} else {
					contactError = err.message;
				}
			} else {
				contactError = msg('common-error', 'Erreur inattendue');
			}
		} finally {
			contactSubmitting = false;
		}
	}

	// Story 33-5 (#909) — corriger l'adresse après l'assistant, sur le patron des
	// deux éditions ci-dessus. Le pays reste un champ libre de deux lettres, comme
	// aux contacts : le serveur juge, l'écran affiche son message.
	let addressEditing = $state(false);
	let addressForm = $state({ street: '', building: '', postalCode: '', city: '', country: 'CH' });
	let addressSubmitting = $state(false);
	let addressError = $state('');
	let addressBase = $state({ ...addressForm });
	let addressConflict = $state<CompanyConflictNoticeState | null>(null);

	function startAddressEdit() {
		if (data) addressBase = addressValuesOf(data.company);
		addressForm = { ...addressBase };
		addressError = '';
		addressConflict = null;
		addressEditing = true;
	}

	function closeAddressEdit() {
		addressEditing = false;
		addressError = '';
		addressConflict = null;
	}

	async function saveAddress() {
		if (!data) return;
		addressSubmitting = true;
		addressError = '';
		try {
			const updated = await updateCompanyAddress({
				address: { ...addressForm },
				version: data.company.version,
			});
			data = { ...data, company: updated };
			closeAddressEdit();
			notifySuccess(msg('settings-company-address-saved', 'Adresse de la société enregistrée'));
		} catch (err) {
			if (isApiError(err)) {
				if (err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					const r = await reloadAfterConflict(
						addressBase,
						{ ...addressForm },
						ADDRESS_FIELDS,
						addressValuesOf,
						ADDRESS_NORMALIZERS,
					);
					if (r.status === 'reload-failed') {
						addressConflict = null;
						addressError = msg(
							'settings-company-address-conflict-reload-failed',
							'Conflit de version et rechargement impossible — rechargez la page.',
						);
					} else if (r.conflict.kind === 'already-saved') {
						addressBase = r.base;
						closeAddressEdit();
						notifySuccess(alreadySavedMessage());
					} else {
						addressBase = r.base;
						addressForm = { ...r.conflict.input };
						addressConflict = r.notice;
						if (!r.notice) {
							addressError = msg(
								'settings-company-address-conflict',
								'Conflit de version — les données ont été rechargées, réessayez.',
							);
						}
					}
				} else {
					addressError = err.message;
				}
			} else {
				addressError = msg('common-error', 'Erreur inattendue');
			}
		} finally {
			addressSubmitting = false;
		}
	}

	onMount(async () => {
		try {
			data = await fetchCompanyCurrent();
		} catch {
			// Company may not exist (404) — that's OK
		} finally {
			loading = false;
		}
	});


	/**
	 * Libellé traduit du type d'organisation.
	 *
	 * ⚠️ **C'était le seul défaut ACTIF des huit sites de la story 23-3b, et le plus
	 * embarrassant à l'usage** : l'utilisateur choisit son type d'organisation en langue
	 * localisée pendant l'onboarding — « Indépendant », « PME » —, puis retrouvait ici le
	 * **code brut du backend** (`Independant`, `Association`, `Pme`), dans **toutes** les
	 * langues, français compris.
	 *
	 * ⚠️ **La valeur affichée n'appelait AUCUNE fonction** — `{data.company.orgType}` — ce qui
	 * la place hors de portée de la garde `i18n-libelle-en-dur.test.ts` : il n'y avait ni
	 * fonction ni littéral, le défaut était l'*absence* d'appel. Corrigé à la main.
	 *
	 * ⚠️ **Les clés `onboarding-org-*` existent déjà et sont câblées côté onboarding : on les
	 * RÉUTILISE.** En créer de neuves aurait fait dire deux choses au même terme selon
	 * l'écran. Le repli sur la valeur brute couvre le déploiement mixte — un backend plus
	 * récent peut renvoyer un type que ce bundle ne connaît pas, et mieux vaut le code que
	 * rien. Les trois valeurs viennent de `chart_of_accounts/mod.rs:215`.
	 */
	function orgTypeLabel(orgType: string): string {
		// ⚠️ `?? ''` par défense en profondeur : la colonne porte `NOT NULL` et une contrainte
		// `CHECK`, donc le cas n'est pas atteignable en base saine — mais sans cette garde, un
		// `null` transformerait un affichage vide en EXCEPTION de rendu. Le correctif ne doit
		// pas aggraver le mode d'échec qu'il corrige. (Passe 1 de revue.)
		switch ((orgType ?? '').toLowerCase()) {
			case 'independant':
				return msg('onboarding-org-independant', 'Indépendant');
			case 'association':
				return msg('onboarding-org-association', 'Association');
			case 'pme':
				return msg('onboarding-org-pme', 'PME');
			default:
				return orgType;
		}
	}

	// --- Story 24-4c (#380) : le verrou de période ---
	// Dernière borne SAISISSABLE : la veille. ⚠️ Confort de saisie seulement —
	// le refus qui fait autorité est celui du serveur (`booksLockBoundNotPast`),
	// et il est testé sans passer par l'écran. Sans ce `max`, le formulaire
	// proposait une date que le serveur refuse, sur le geste même que la garde
	// existe pour rattraper.
	const maxLockDate = (() => {
		const d = new Date();
		d.setUTCDate(d.getUTCDate() - 1);
		return d.toISOString().slice(0, 10);
	})();

	let lockThrough = $state('');
	let unlockThrough = $state('');
	let unlockMotif = $state('');
	let lockBusy = $state(false);
	let lockError = $state<string | null>(null);
	let showUnlock = $state(false);

	async function submitLock() {
		if (!lockThrough || lockBusy) return;
		lockBusy = true;
		lockError = null;
		try {
			const updated = await lockBooks(lockThrough);
			if (data) data = { ...data, company: updated };
			lockThrough = '';
		} catch (e) {
			lockError = errorMessageOf(e);
		} finally {
			lockBusy = false;
		}
	}

	async function submitUnlock() {
		if (lockBusy) return;
		// ⛔ Le motif est obligatoire côté SERVEUR ; ce contrôle n'est qu'un
		// confort. Déverrouiller défait une garantie — c'est le seul geste des
		// deux qui doive coûter.
		if (!unlockMotif.trim()) {
			lockError = msg('settings-books-lock-motif-required', 'Un motif est obligatoire pour déverrouiller.');
			return;
		}
		lockBusy = true;
		lockError = null;
		try {
			const updated = await releaseBooksLock(unlockThrough || null, unlockMotif);
			if (data) data = { ...data, company: updated };
			unlockThrough = '';
			unlockMotif = '';
			showUnlock = false;
		} catch (e) {
			lockError = errorMessageOf(e);
		} finally {
			lockBusy = false;
		}
	}

</script>

<svelte:head>
	<title>{msg('settings-title', 'Paramètres')} - Kesh</title>
</svelte:head>

<h1 class="mb-6 text-2xl font-semibold text-text">
	{msg('settings-title', 'Paramètres')}
</h1>

{#if loading}
	<p class="text-text-muted">{msg('common-loading', 'Chargement…')}</p>
{:else if data}
	<div class="flex flex-col gap-6">
		<!-- Section Organisation -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="mb-4 text-lg font-semibold">{msg('settings-org-title', 'Organisation')}</h2>
			<dl class="grid grid-cols-2 gap-x-6 gap-y-3 text-sm">
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-name', 'Nom')}</dt>
					<dd>{data.company.name}</dd>
				</div>
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-org-type', 'Type')}</dt>
					<dd>{orgTypeLabel(data.company.orgType)}</dd>
				</div>
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">{msg('settings-field-address', 'Adresse')}</dt>
					<!-- Story 33-5 (#909) : le texte reste affiché pendant l'édition — une société
					     d'avant les composants n'a que lui, et il faut le voir pour le ressaisir. -->
					<dd class="flex items-center gap-3">
						<span class="whitespace-pre-line" data-testid="settings-company-address">{data.company.address}</span>
						{#if isAdmin && !addressEditing}
							<Button size="sm" variant="outline" onclick={startAddressEdit} data-testid="settings-company-address-edit">
								{msg('common-edit', 'Modifier')}
							</Button>
						{/if}
					</dd>
					{#if addressEditing}
						<dd class="mt-2 max-w-md space-y-2">
							<div class="grid grid-cols-3 gap-2">
								<div class="col-span-2">
									<label for="company-address-street" class="text-xs text-text-muted">{msg('field-street', 'Rue')}</label>
									<Input id="company-address-street" type="text" bind:value={addressForm.street} maxlength={70} data-testid="settings-company-address-street-input" />
									<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" field="street" />
								</div>
								<div>
									<label for="company-address-building" class="text-xs text-text-muted">{msg('field-building', 'N°')}</label>
									<Input id="company-address-building" type="text" bind:value={addressForm.building} maxlength={16} data-testid="settings-company-address-building-input" />
									<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" field="building" />
								</div>
								<div>
									<label for="company-address-postal-code" class="text-xs text-text-muted">{msg('field-postal-code', 'NPA')}</label>
									<Input id="company-address-postal-code" type="text" bind:value={addressForm.postalCode} maxlength={16} data-testid="settings-company-address-postal-code-input" />
									<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" field="postalCode" />
								</div>
								<div>
									<label for="company-address-city" class="text-xs text-text-muted">{msg('field-city', 'Localité')}</label>
									<Input id="company-address-city" type="text" bind:value={addressForm.city} maxlength={35} data-testid="settings-company-address-city-input" />
									<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" field="city" />
								</div>
								<div>
									<label for="company-address-country" class="text-xs text-text-muted">{msg('field-country', 'Pays')}</label>
									<Input id="company-address-country" type="text" bind:value={addressForm.country} maxlength={2} placeholder="CH" data-testid="settings-company-address-country-input" />
									<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" field="country" />
								</div>
							</div>
							<CompanyConflictNotice notice={addressConflict} testid="settings-company-address-conflict" />
							{#if addressError}
								<p class="text-xs text-destructive" data-testid="settings-company-address-error">
									{addressError}
								</p>
							{/if}
							<div class="flex gap-2">
								<Button size="sm" onclick={saveAddress} disabled={addressSubmitting} data-testid="settings-company-address-save">
									{msg('common-save', 'Enregistrer')}
								</Button>
								<Button
									size="sm"
									variant="outline"
									onclick={closeAddressEdit}
									disabled={addressSubmitting}
									data-testid="settings-company-address-cancel"
								>
									{msg('common-cancel', 'Annuler')}
								</Button>
							</div>
						</dd>
					{/if}
					{#if data.company.addressCountryKnown === false}
						<p class="mt-1 text-xs text-warning" data-testid="settings-company-address-country-unknown">
							{msg(
								'settings-company-address-country-unknown',
								'Le code pays « { $code } » de la société n’est pas un code ISO connu : ses QR-factures ne peuvent pas être produites. Un administrateur peut corriger l’adresse ici (CH, FR, DE…).',
								{ code: data.company.addressStructured.country ?? '' },
							)}
						</p>
					{/if}
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-address-help',
							'Adresse de l’émetteur et du créancier sur vos factures et leurs QR-factures. NPA et localité obligatoires ; pays en code ISO à deux lettres. Un PDF déjà figé ne change pas.',
						)}
					</p>
				</div>
				{#if data.company.ideNumber}
					<div>
						<dt class="font-medium text-text-muted">{msg('settings-field-ide', 'IDE')}</dt>
						<dd>{data.company.ideNumber}</dd>
					</div>
				{/if}
				<div>
					<dt class="font-medium text-text-muted">{msg('settings-field-instance-language', 'Langue des documents')}</dt>
					<dd>{data.company.instanceLanguage}</dd>
				</div>
				<!-- Story 20-3b2 : e-mail de contact (Reply-To des factures envoyées). -->
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">
						{msg('settings-field-company-email', 'E-mail (adresse de réponse)')}
					</dt>
					{#if emailEditing}
						<dd class="mt-1 flex max-w-md items-start gap-2">
							<div class="flex-1">
								<Input
									type="email"
									bind:value={emailValue}
									maxlength={320}
									data-testid="settings-company-email-input"
								/>
								<CompanyConflictNotice notice={emailConflict} testid="settings-company-email-conflict" field="email" />
								<CompanyConflictNotice notice={emailConflict} testid="settings-company-email-conflict" />
								{#if emailError}
									<p class="mt-1 text-xs text-destructive" data-testid="settings-company-email-error">
										{emailError}
									</p>
								{/if}
							</div>
							<Button size="sm" onclick={saveEmail} disabled={emailSubmitting} data-testid="settings-company-email-save">
								{msg('common-save', 'Enregistrer')}
							</Button>
							<Button
								size="sm"
								variant="outline"
								onclick={closeEmailEdit}
								disabled={emailSubmitting}
							>
								{msg('common-cancel', 'Annuler')}
							</Button>
						</dd>
					{:else}
						<dd class="flex items-center gap-3">
							<span data-testid="settings-company-email">{data.company.email ?? '—'}</span>
							{#if isAdmin}
								<Button size="sm" variant="outline" onclick={startEmailEdit} data-testid="settings-company-email-edit">
									{msg('common-edit', 'Modifier')}
								</Button>
							{/if}
						</dd>
					{/if}
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-email-help',
							"Adresse de réponse (Reply-To) des factures envoyées par e-mail. Vide = pas d'adresse de réponse.",
						)}
					</p>
				</div>

				<!-- Story 16-3a (#151) : téléphone et site web, rendus sur le PDF. -->
				<div class="col-span-2">
					<dt class="font-medium text-text-muted">
						{msg('settings-field-company-phone', 'Téléphone')} ·
						{msg('settings-field-company-website', 'Site web')}
					</dt>
					{#if contactEditing}
						<dd class="mt-1 max-w-md space-y-2">
							<Input
								type="text"
								bind:value={phoneValue}
								maxlength={50}
								placeholder={msg('settings-field-company-phone', 'Téléphone')}
								data-testid="settings-company-phone-input"
							/>
							<CompanyConflictNotice notice={contactConflict} testid="settings-company-contact-conflict" field="phone" />
							<Input
								type="text"
								bind:value={websiteValue}
								maxlength={255}
								placeholder={msg('settings-field-company-website', 'Site web')}
								data-testid="settings-company-website-input"
							/>
							<CompanyConflictNotice notice={contactConflict} testid="settings-company-contact-conflict" field="website" />
							<CompanyConflictNotice notice={contactConflict} testid="settings-company-contact-conflict" />
							{#if contactError}
								<p class="text-xs text-destructive" data-testid="settings-company-contact-error">
									{contactError}
								</p>
							{/if}
							<div class="flex gap-2">
								<Button
									size="sm"
									onclick={saveContactDetails}
									disabled={contactSubmitting}
									data-testid="settings-company-contact-save"
								>
									{msg('common-save', 'Enregistrer')}
								</Button>
								<Button
									size="sm"
									variant="outline"
									onclick={closeContactEdit}
									disabled={contactSubmitting}
								>
									{msg('common-cancel', 'Annuler')}
								</Button>
							</div>
						</dd>
					{:else}
						<dd class="flex items-center gap-3">
							<span data-testid="settings-company-phone">{data.company.phone ?? '—'}</span>
							<span data-testid="settings-company-website">{data.company.website ?? '—'}</span>
							{#if isAdmin}
								<Button
									size="sm"
									variant="outline"
									onclick={startContactEdit}
									data-testid="settings-company-contact-edit"
								>
									{msg('common-edit', 'Modifier')}
								</Button>
							{/if}
						</dd>
					{/if}
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-phone-help',
							'Numéro de téléphone affiché sur vos factures. Vide = ligne omise.',
						)}
					</p>
					<p class="mt-1 text-xs text-text-muted">
						{msg(
							'settings-company-website-help',
							'Adresse de votre site, affichée sur vos factures. Vide = ligne omise.',
						)}
					</p>
				</div>
			</dl>
		</section>

		<Separator />

		<!-- Section Comptabilité -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<h2 class="mb-4 text-lg font-semibold">{msg('settings-accounting-title', 'Comptabilité')}</h2>
			<dl class="text-sm">
				<dt class="font-medium text-text-muted">{msg('settings-field-accounting-language', 'Langue comptable')}</dt>
				<dd>{data.company.accountingLanguage}</dd>
			</dl>
		</section>

		<Separator />

		<!-- Section Comptes bancaires -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-bank-title', 'Comptes bancaires')}</h2>
				<Button variant="outline" size="sm" href="/bank-accounts" data-testid="settings-bank-manage-link">
					{msg('settings-bank-manage', 'Gérer dans Administration → Comptes bancaires')}
				</Button>
			</div>
			<p class="mt-2 text-xs text-text-muted">
				{msg(
					'settings-bank-manage-hint',
					'Pour ajouter, modifier, clore ou archiver un compte bancaire, utilisez la page dédiée Administration → Comptes bancaires.',
				)}
			</p>
			{#if data.bankAccounts.length > 0}
				{#each data.bankAccounts as account}
					<div class="mt-3 text-sm">
						<p class="font-medium" data-testid="settings-bank-name-{account.id}">{bankAccountName(account)}</p>
						<p class="text-text-muted">{account.iban}</p>
						{#if account.qrIban}
							<p class="text-text-muted">QR-IBAN: {account.qrIban}</p>
						{/if}
					</div>
				{/each}
			{:else}
				<p class="mt-3 text-sm text-text-muted">{msg('settings-no-bank', 'Aucun compte bancaire configuré.')}</p>
			{/if}
		</section>

		<Separator />

		<!-- Section Clés API (Story 17-2b) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-api-keys-title', 'Clés API')}</h2>
				<Button variant="outline" size="sm" href="/settings/api-keys" data-testid="settings-api-keys-manage-link">
					{msg('settings-api-keys-manage', 'Gérer')}
				</Button>
			</div>
			<p class="mt-2 text-xs text-text-muted">
				{msg(
					'settings-api-keys-hint',
					'Créez des clés d\'accès API pour vos intégrations (IA externe, scripts, logiciels tiers).',
				)}
			</p>
		</section>

		<Separator />

		<!-- Section Utilisateurs -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('settings-users-title', 'Utilisateurs')}</h2>
				<Button variant="outline" size="sm" href="/users">{msg('settings-manage', 'Gérer')}</Button>
			</div>
		</section>

		<Separator />

		<!-- Section Session (Story v0172-session, CR #891) — Administrateur seul :
		     le GET du réglage rend 403 aux autres rôles. -->
		{#if isAdmin}
			<SessionIdleSection />

			<Separator />
		{/if}

		<!-- Section Verrou de période (Story 24-4c, #380) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm" data-testid="settings-books-lock">
			<h2 class="text-lg font-semibold">{msg('settings-books-lock-title', 'Verrou de période')}</h2>
			<p class="mt-2 text-sm text-text-muted">
				{#if data?.company.booksLockedThrough}
					{msg('settings-books-lock-current', 'Livres verrouillés jusqu\'au { $date } inclus.', {
						date: data.company.booksLockedThrough
					})}
				{:else}
					{msg('settings-books-lock-none', "Aucun verrou : toutes les dates d'un exercice ouvert sont acceptées.")}
				{/if}
			</p>

			{#if lockError}
				<p class="mt-2 text-sm text-destructive" data-testid="books-lock-error">{lockError}</p>
			{/if}

			<div class="mt-4 flex flex-wrap items-end gap-2">
				<div>
					<label for="books-lock-through" class="block text-sm font-medium mb-1">
						{msg('settings-books-lock-set', 'Verrouiller jusqu\'au')}
					</label>
					<Input id="books-lock-through" type="date" max={maxLockDate} bind:value={lockThrough} />
				</div>
				<Button size="sm" onclick={submitLock} disabled={lockBusy || !lockThrough} data-testid="books-lock-submit">
					{msg('settings-books-lock-set', 'Verrouiller jusqu\'au')}
				</Button>
				{#if data?.company.booksLockedThrough}
					<Button
						size="sm"
						variant="outline"
						onclick={() => (showUnlock = !showUnlock)}
						data-testid="books-lock-release-toggle"
					>
						{msg('settings-books-lock-release', 'Déverrouiller')}
					</Button>
				{/if}
			</div>

			{#if showUnlock}
				<div class="mt-3 flex flex-wrap items-end gap-2 border-t border-border pt-3">
					<div>
						<label for="books-unlock-through" class="block text-sm font-medium mb-1">
							{msg('settings-books-lock-set', 'Verrouiller jusqu\'au')}
						</label>
						<Input id="books-unlock-through" type="date" max={maxLockDate} bind:value={unlockThrough} />
					</div>
					<div class="grow">
						<label for="books-unlock-motif" class="block text-sm font-medium mb-1">
							{msg('settings-books-lock-motif', 'Motif du déverrouillage')}
						</label>
						<Input id="books-unlock-motif" type="text" bind:value={unlockMotif} />
					</div>
					<Button
						size="sm"
						variant="destructive"
						onclick={submitUnlock}
						disabled={lockBusy}
						data-testid="books-lock-release-submit"
					>
						{msg('settings-books-lock-release', 'Déverrouiller')}
					</Button>
				</div>
			{/if}
		</section>

		<Separator />

		<!-- Section Exercices comptables (Story 3.7) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('fiscal-year-title', 'Exercices comptables')}</h2>
				<Button variant="outline" size="sm" href="/settings/fiscal-years">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-fiscal-years-link', "Créez, renommez ou clôturez les exercices comptables de votre entreprise.")}
			</p>
		</section>

		<Separator />

		<!-- Section Taux de TVA (Story 11-1, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('vat-rates-title', 'Taux de TVA')}</h2>
				<Button variant="outline" size="sm" href="/settings/vat-rates" data-testid="settings-vat-rates-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-vat-rates-link', "Configurez les taux de TVA et leurs dates de validité (changements de taux gérés dans le temps).")}
			</p>
		</section>

		<Separator />

		<!-- Section Modèles d'e-mail (Story 20-2, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('email-templates-title', "Modèles d'e-mail")}</h2>
				<Button variant="outline" size="sm" href="/settings/email-templates" data-testid="settings-email-templates-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-email-templates-link', "Personnalisez le contenu des e-mails envoyés à vos clients (objet et corps, par langue).")}
			</p>
		</section>

		<Separator />

		<!-- Section Rappels débiteurs (Story 21-4, Administrateur) -->
		<section class="rounded-lg border border-border bg-white p-6 shadow-sm">
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold">{msg('dunning-title', 'Rappels débiteurs')}</h2>
				<Button variant="outline" size="sm" href="/settings/dunning" data-testid="settings-dunning-manage-link">{msg('settings-manage', 'Gérer')}</Button>
			</div>
			<p class="mt-2 text-sm text-text-muted">
				{msg('settings-dunning-link', 'Configurez les niveaux de rappel (délais et frais), la période de grâce, et personnalisez les textes d’e-mail de rappel par niveau.')}
			</p>
		</section>
	</div>
{:else}
	<p class="text-text-muted">{msg('settings-no-company', 'Aucune organisation configurée. Complétez l\'onboarding.')}</p>
{/if}
