<!--
  Story 17-2b — Page de gestion des clés API (PAT) `/settings/api-keys`.
  Calque `bank-accounts/+page.svelte`. Liste + création (secret one-time +
  copie HTTP-LAN-safe) + révocation avec confirmation forte.
  Route session-JWT-cookie uniquement (un PAT ne peut pas l'atteindre, DC6
  backend = filet). Guard backend `require_comptable_role`.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { copyToClipboard } from '$lib/shared/utils/clipboard';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import {
		listApiKeys,
		createApiKey,
		revokeApiKey,
		type ApiKey,
		type ApiKeyScope,
		type NewApiKeyPayload,
	} from '$lib/features/api-keys/api-keys.api';

	let keys = $state<ApiKey[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);

	type Mode = { kind: 'none' } | { kind: 'create' } | { kind: 'revoke-confirm'; id: number };
	let mode = $state<Mode>({ kind: 'none' });
	let submitting = $state(false);

	// Form state (création).
	let formName = $state('');
	let formScope = $state<ApiKeyScope>('read');
	let formExpiresAt = $state(''); // YYYY-MM-DD (input date) ou '' (permanente).
	let formError = $state<string | null>(null);

	// Encart secret one-time — survit à la fermeture du form, jusqu'à fermeture
	// explicite par l'utilisateur. Jamais persisté ailleurs (pas de localStorage).
	let createdSecret = $state<{ name: string; key: string } | null>(null);

	// Borne `min` du date-picker = demain (évite une expiration « aujourd'hui »
	// que le backend pourrait refuser comme déjà passée selon l'heure). Recalculée
	// à l'ouverture du form (`openCreate`) pour ne pas rester périmée si la page
	// reste ouverte au-delà de minuit (code-review Pass 1).
	let formMinDate = $state('');

	function computeMinDate(): string {
		return new Date(Date.now() + 86_400_000).toISOString().slice(0, 10);
	}

	/** True si la clé est expirée par date (sans révocation explicite). */
	function isExpired(k: ApiKey): boolean {
		return !k.revokedAt && k.expiresAt !== null && new Date(k.expiresAt).getTime() < Date.now();
	}

	async function reload() {
		try {
			keys = await listApiKeys();
			loadError = null;
		} catch (err) {
			loadError = isApiError(err) ? err.message : String(err);
		}
	}

	onMount(async () => {
		await reload();
		loading = false;
	});

	/** Formate un `NaiveDateTime` (`2026-06-06T12:34:56.789`) en `YYYY-MM-DD`. */
	function formatDate(value: string | null): string {
		if (!value) return '—';
		return value.slice(0, 10);
	}

	/** Formate en `YYYY-MM-DD HH:mm` (dernière utilisation). */
	function formatDateTime(value: string | null): string {
		if (!value) return i18nMsg('api-keys-labels-never-used', 'Jamais utilisée');
		return value.slice(0, 16).replace('T', ' ');
	}

	function scopeLabel(scope: ApiKeyScope): string {
		return scope === 'read'
			? i18nMsg('api-keys-labels-scope-read', 'Lecture seule')
			: i18nMsg('api-keys-labels-scope-read-write', 'Lecture-écriture');
	}

	function statusLabel(k: ApiKey): string {
		if (k.revokedAt) {
			return i18nMsg('api-keys-labels-status-revoked', 'Révoquée le {$date}', {
				date: formatDate(k.revokedAt),
			});
		}
		if (isExpired(k)) {
			return i18nMsg('api-keys-labels-status-expired', 'Expirée le {$date}', {
				date: formatDate(k.expiresAt),
			});
		}
		if (k.expiresAt) {
			return i18nMsg('api-keys-labels-status-expires', 'Active (expire le {$date})', {
				date: formatDate(k.expiresAt),
			});
		}
		return i18nMsg('api-keys-labels-status-active', 'Active');
	}

	function resetForm() {
		formName = '';
		formScope = 'read';
		formExpiresAt = '';
		formError = null;
	}

	function openCreate() {
		resetForm();
		formMinDate = computeMinDate();
		mode = { kind: 'create' };
	}

	function closeForm() {
		mode = { kind: 'none' };
		resetForm();
	}

	async function submitCreate(e: Event) {
		e.preventDefault();
		if (submitting) return; // garde double-soumission avant flush du `disabled`
		formError = null;
		const name = formName.trim();
		if (name === '') {
			formError = i18nMsg('api-keys-errors-name-required', 'Le nom de la clé est requis.');
			return;
		}
		// Compte en points de code Unicode (`[...name]`) pour matcher le backend
		// (`name.chars().count()`) — sinon un emoji ferait diverger `.length` (UTF-16).
		if ([...name].length > 255) {
			formError = i18nMsg(
				'api-keys-errors-name-too-long',
				'Le nom de la clé est trop long (255 caractères maximum).',
			);
			return;
		}
		submitting = true;
		try {
			const payload: NewApiKeyPayload = { name, scope: formScope };
			if (formExpiresAt !== '') {
				// Le backend attend un RFC 3339 (`DateTime<Utc>`). On vise la fin de
				// la journée sélectionnée pour une expiration intuitive.
				payload.expiresAt = `${formExpiresAt}T23:59:59Z`;
			}
			const created = await createApiKey(payload);
			createdSecret = { name: created.name, key: created.key };
			toast.success(i18nMsg('api-keys-toast-create-success', 'Clé API créée.'));
			await reload();
			closeForm();
		} catch (err) {
			formError = isApiError(err) ? err.message : String(err);
		} finally {
			submitting = false;
		}
	}

	async function handleCopySecret() {
		if (!createdSecret) return;
		const ok = await copyToClipboard(createdSecret.key);
		if (ok) {
			toast.success(i18nMsg('api-keys-toast-copied', 'Clé copiée dans le presse-papiers.'));
		} else {
			toast.error(
				i18nMsg('api-keys-toast-copy-failed', 'Copie impossible — sélectionnez et copiez manuellement.'),
			);
		}
	}

	function openRevokeConfirm(id: number) {
		mode = { kind: 'revoke-confirm', id };
	}

	async function confirmRevoke(id: number) {
		if (submitting) return; // garde double-clic
		const k = keys.find((x) => x.id === id);
		if (!k) {
			closeForm();
			return;
		}
		submitting = true;
		try {
			await revokeApiKey(id, k.version);
			toast.success(i18nMsg('api-keys-toast-revoke-success', 'Clé révoquée.'));
		} catch (err) {
			if (isApiError(err) && err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				toast.error(
					i18nMsg('api-keys-errors-conflict', 'La clé a changé entre-temps — liste rechargée, réessayez.'),
				);
			} else {
				toast.error(isApiError(err) ? err.message : String(err));
			}
		} finally {
			// Toujours recharger + fermer : sur 404 (clé déjà supprimée ailleurs) ou
			// réseau, éviter de laisser le panneau ouvert avec une liste périmée
			// (sinon retry → boucle 404 sur la clé fantôme — code-review Pass 1).
			submitting = false;
			await reload();
			closeForm();
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('api-keys-labels-page-title', 'Clés API')} - Kesh</title>
</svelte:head>

<div class="flex items-center justify-between">
	<div>
		<h1 class="text-2xl font-semibold text-text" data-testid="api-keys-page-title">
			{i18nMsg('api-keys-labels-page-title', 'Clés API')}
		</h1>
		<p class="mt-2 text-sm text-text-muted">
			{i18nMsg(
				'api-keys-labels-page-subtitle',
				'Créez des clés d\'accès API pour vos intégrations (IA externe, scripts, logiciels tiers). Présentez la clé via l\'en-tête « Authorization: Bearer ».',
			)}
		</p>
	</div>
	<Button onclick={openCreate} data-testid="api-keys-create-button" disabled={mode.kind === 'create'}>
		{i18nMsg('api-keys-actions-create', 'Nouvelle clé')}
	</Button>
</div>

<!-- Encart secret one-time — affiché jusqu'à fermeture explicite. -->
{#if createdSecret}
	<div
		class="mt-6 rounded border border-amber-300 bg-amber-50 p-4"
		data-testid="api-keys-secret"
		role="alert"
	>
		<p class="text-sm font-medium text-amber-900" data-testid="api-keys-secret-name">
			{i18nMsg('api-keys-labels-secret-created', 'Clé « {$name} » créée.', {
				name: createdSecret.name,
			})}
		</p>
		<p class="mt-1 text-sm font-medium text-amber-900">
			{i18nMsg(
				'api-keys-labels-secret-warning',
				'Copiez cette clé maintenant : elle ne sera plus jamais affichée.',
			)}
		</p>
		<div class="mt-3 flex items-center gap-2">
			<code
				class="flex-1 break-all rounded bg-white px-3 py-2 font-mono text-sm"
				data-testid="api-keys-secret-value">{createdSecret.key}</code
			>
			<Button onclick={handleCopySecret} data-testid="api-keys-secret-copy">
				{i18nMsg('api-keys-actions-copy', 'Copier')}
			</Button>
			<Button variant="ghost" onclick={() => (createdSecret = null)} data-testid="api-keys-secret-close">
				{i18nMsg('api-keys-actions-close', 'Fermer')}
			</Button>
		</div>
	</div>
{/if}

<div class="mt-6">
	{#if loading}
		<p class="text-text-muted">{i18nMsg('api-keys-labels-loading', 'Chargement…')}</p>
	{:else if loadError}
		<p class="text-red-600" role="alert" data-testid="api-keys-load-error">{loadError}</p>
	{:else}
		<!-- Formulaire création inline -->
		{#if mode.kind === 'create'}
			<form
				onsubmit={submitCreate}
				class="mb-6 rounded border border-border bg-surface-alt p-4"
				data-testid="api-keys-create-form"
			>
				<h2 class="mb-3 text-base font-semibold">
					{i18nMsg('api-keys-actions-create', 'Nouvelle clé')}
				</h2>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label for="api-key-name" class="mb-1 block text-sm font-medium text-text">
							{i18nMsg('api-keys-labels-name', 'Nom')}
						</label>
						<Input
							id="api-key-name"
							bind:value={formName}
							required
							maxlength={255}
							placeholder={i18nMsg('api-keys-labels-name-placeholder', 'ex. Script comptable, Agent IA…')}
							data-testid="api-keys-name-input"
						/>
					</div>
					<div>
						<label for="api-key-scope" class="mb-1 block text-sm font-medium text-text">
							{i18nMsg('api-keys-labels-scope', 'Portée')}
						</label>
						<select
							id="api-key-scope"
							bind:value={formScope}
							class="w-full rounded border border-border px-3 py-2 text-sm"
							data-testid="api-keys-scope-select"
						>
							<option value="read">{i18nMsg('api-keys-labels-scope-read', 'Lecture seule')}</option>
							<option value="read-write"
								>{i18nMsg('api-keys-labels-scope-read-write', 'Lecture-écriture')}</option
							>
						</select>
					</div>
					<div>
						<label for="api-key-expires" class="mb-1 block text-sm font-medium text-text">
							{i18nMsg('api-keys-labels-expires', 'Expiration (optionnelle)')}
						</label>
						<Input
							id="api-key-expires"
							type="date"
							bind:value={formExpiresAt}
							min={formMinDate}
							data-testid="api-keys-expires-input"
						/>
						<p class="mt-1 text-xs text-text-muted">
							{i18nMsg('api-keys-labels-expires-hint', 'Laissez vide pour une clé permanente.')}
						</p>
					</div>
				</div>
				{#if formError}
					<p class="mt-3 text-sm text-red-600" role="alert" data-testid="api-keys-form-error">
						{formError}
					</p>
				{/if}
				<div class="mt-4 flex gap-2">
					<Button type="submit" disabled={submitting} data-testid="api-keys-submit">
						{i18nMsg('api-keys-actions-submit-create', 'Créer la clé')}
					</Button>
					<Button type="button" variant="ghost" onclick={closeForm} data-testid="api-keys-cancel">
						{i18nMsg('api-keys-actions-cancel', 'Annuler')}
					</Button>
				</div>
			</form>
		{/if}

		<!-- Liste -->
		{#if keys.length === 0}
			<p class="text-text-muted" data-testid="api-keys-empty">
				{i18nMsg('api-keys-labels-empty', 'Aucune clé API. Créez-en une pour vos intégrations.')}
			</p>
		{:else}
			<table class="w-full text-sm" data-testid="api-keys-list">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="py-2 pr-4 font-semibold">{i18nMsg('api-keys-labels-name', 'Nom')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('api-keys-labels-scope', 'Portée')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('api-keys-labels-created-at', 'Créée le')}</th>
						<th class="py-2 pr-4 font-semibold"
							>{i18nMsg('api-keys-labels-last-used', 'Dernière utilisation')}</th
						>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('api-keys-labels-status', 'Statut')}</th>
						<th class="py-2"></th>
					</tr>
				</thead>
				<tbody>
					{#each keys as k (k.id)}
						<tr
							class="border-b border-border {k.revokedAt || isExpired(k) ? 'opacity-50' : ''}"
							data-testid="api-key-row-{k.id}"
						>
							<td class="py-2 pr-4">{k.name}</td>
							<td class="py-2 pr-4">{scopeLabel(k.scope)}</td>
							<td class="py-2 pr-4 font-mono text-xs">{formatDate(k.createdAt)}</td>
							<td class="py-2 pr-4 font-mono text-xs">{formatDateTime(k.lastUsedAt)}</td>
							<td class="py-2 pr-4" data-testid="api-key-status-{k.id}">{statusLabel(k)}</td>
							<td class="py-2 whitespace-nowrap">
								{#if !k.revokedAt && !isExpired(k) && mode.kind === 'none'}
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs text-red-600"
										onclick={() => openRevokeConfirm(k.id)}
										data-testid="api-keys-revoke-{k.id}"
									>
										{i18nMsg('api-keys-actions-revoke', 'Révoquer')}
									</button>
								{/if}
							</td>
						</tr>
						<!-- Confirmation de révocation inline -->
						{#if mode.kind === 'revoke-confirm' && mode.id === k.id}
							<tr>
								<td colspan="6" class="py-3">
									<div
										class="rounded border border-red-200 bg-red-50 p-4"
										data-testid="api-keys-revoke-confirm"
									>
										<p class="text-sm">
											{i18nMsg(
												'api-keys-confirm-revoke',
												'Révoquer cette clé ? Toute intégration l\'utilisant cessera immédiatement de fonctionner. Cette action est irréversible.',
											)}
										</p>
										<div class="mt-3 flex gap-2">
											<Button
												onclick={() => confirmRevoke(k.id)}
												disabled={submitting}
												variant="destructive"
												data-testid="api-keys-revoke-confirm-button"
											>
												{i18nMsg('api-keys-actions-confirm-revoke', 'Révoquer')}
											</Button>
											<Button
												type="button"
												variant="ghost"
												onclick={closeForm}
												data-testid="api-keys-revoke-cancel-button"
											>
												{i18nMsg('api-keys-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		{/if}
	{/if}
</div>
