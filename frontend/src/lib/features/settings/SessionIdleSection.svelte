<script lang="ts">
	/**
	 * Story v0172-session (0.17.3, CR #891) — section « Session » de
	 * Administration → Paramètres : la durée d'inactivité après laquelle une
	 * session se ferme. Rendue par la page pour l'Administrateur seul.
	 *
	 * Verrou optimiste comme l'e-mail de la société : un 409 recharge l'état et
	 * invite à recommencer. Après un enregistrement, la session de
	 * l'Administrateur applique aussitôt la nouvelle durée
	 * (`sessionIdle.setMinutes`) ; les autres sessions, à leur prochaine
	 * activité.
	 */
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { sessionIdle } from '$lib/app/stores/session-idle.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { notifySuccess } from '$lib/shared/utils/notify';
	import { fetchSessionSettings, updateSessionSettings } from './settings.api';
	import type { SessionSettings } from './settings.types';

	let settings = $state<SessionSettings | null>(null);
	let value = $state('');
	let error = $state('');
	let loadError = $state('');
	let submitting = $state(false);

	function apply(s: SessionSettings) {
		settings = s;
		value = String(s.sessionIdleMinutes);
	}

	onMount(async () => {
		try {
			apply(await fetchSessionSettings());
		} catch {
			loadError = i18nMsg(
				'settings-session-load-error',
				"Impossible de lire la durée d'inactivité.",
			);
		}
	});

	/** Un entier dans les bornes, ou `null`. */
	function parsed(s: SessionSettings): number | null {
		const raw = value.trim();
		if (!/^[0-9]+$/.test(raw)) return null;
		const n = Number(raw);
		return n >= s.min && n <= s.max ? n : null;
	}

	async function save() {
		if (!settings) return;
		const n = parsed(settings);
		if (n === null) {
			error = i18nMsg(
				'settings-session-idle-invalid',
				'Saisissez un nombre entier de { $min } à { $max } minutes.',
				{ min: settings.min, max: settings.max },
			);
			return;
		}
		submitting = true;
		error = '';
		try {
			const updated = await updateSessionSettings({
				sessionIdleMinutes: n,
				version: settings.version,
			});
			apply(updated);
			sessionIdle.setMinutes(updated.sessionIdleMinutes);
			notifySuccess(i18nMsg('settings-session-saved', "Durée d'inactivité enregistrée"));
		} catch (err) {
			if (isApiError(err) && err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
				try {
					apply(await fetchSessionSettings());
					error = i18nMsg(
						'settings-session-conflict',
						'Conflit de version — les données ont été rechargées, réessayez.',
					);
				} catch {
					error = i18nMsg(
						'settings-session-conflict-reload-failed',
						'Conflit de version et rechargement impossible — rechargez la page.',
					);
				}
			} else if (isApiError(err)) {
				error = err.message;
			} else {
				error = i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			submitting = false;
		}
	}
</script>

<section
	class="rounded-lg border border-border bg-white p-6 shadow-sm"
	data-testid="settings-session"
>
	<h2 class="text-lg font-semibold">{i18nMsg('settings-session-title', 'Session')}</h2>
	{#if loadError}
		<p class="mt-2 text-sm text-destructive" data-testid="settings-session-load-error">{loadError}</p>
	{:else if settings}
		<form
			class="mt-3 space-y-2"
			onsubmit={(e) => {
				e.preventDefault();
				void save();
			}}
		>
			<label class="flex flex-wrap items-center gap-2 text-sm" for="settings-session-idle-input">
				<span>{i18nMsg('settings-session-idle-label', 'Fermer la session après')}</span>
				<Input
					id="settings-session-idle-input"
					type="text"
					inputmode="numeric"
					class="w-24"
					bind:value
					aria-invalid={error ? 'true' : undefined}
					data-testid="settings-session-idle-input"
				/>
				<span>{i18nMsg('settings-session-idle-unit', "minutes d'inactivité")}</span>
			</label>
			<p class="text-xs text-text-muted" data-testid="settings-session-idle-help">
				{i18nMsg(
					'settings-session-idle-help',
					'De { $min } à { $max } minutes. Toute activité dans Kesh remet le compte à zéro ; une session applique une nouvelle durée à sa prochaine activité.',
					{ min: settings.min, max: settings.max },
				)}
			</p>
			{#if error}
				<p class="text-xs text-destructive" role="alert" data-testid="settings-session-idle-error">
					{error}
				</p>
			{/if}
			<Button type="submit" size="sm" disabled={submitting} data-testid="settings-session-save">
				{i18nMsg('common-save', 'Enregistrer')}
			</Button>
		</form>
	{/if}
</section>
