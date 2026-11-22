<!--
  Story 38-12b (AC-2, AC-3, socle S7) — le dialogue GÉNÉRIQUE de téléchargement
  par l'API d'un prestataire : PayPal ici, Wise (39-12b) et Stripe (40-8b)
  ensuite, par leur segment de route (`provider`), leurs libellés (`title`,
  `stepLabel`, `notice`) et leur compte rendu (`report`, un snippet).

  1. Formulaire : période (défaut fourni par l'appelant), texte permanent
     (`notice`), refus du serveur affichés par code. Sur `409
     PROVIDER_DOWNLOAD_RUNNING`, le dialogue REPREND le suivi de la tâche en
     cours (`details.jobId`) au lieu d'afficher une erreur ; le serveur refuse
     par compte, quelle que soit la période, et quand celle de la tâche suivie
     (`details.from`/`to`) diffère de la période demandée, un avis le dit
     (« un téléchargement du … au … est déjà en cours ; son suivi est repris »)
     jusqu'à l'issue.
  2. Suivi : `GET …/{jobId}` toutes les secondes ; barre et « fenêtre 5/13 »
     (compteur sans barre quand le total est inconnu), pause annoncée.
  3. Issue : `done` → le compte rendu de l'appelant, « données à jour jusqu'à … »
     (`result.lastRefreshed`) et `onDone`, appelé une seule fois, même s'il
     lève (la page recharge sa revue) ; `failed` → le motif traduit du code,
     « rien n'a été enregistré » ; `404`, erreurs réseau répétées ou `done`
     sans compte rendu → « suivi perdu ».

  Le suivi S'ARRÊTE à la fermeture du dialogue et au démontage (garde de
  génération : aucune interrogation orpheline, même quand la fermeture tombe
  pendant le `POST` du lancement) ; fermer le dialogue n'arrête pas la tâche,
  que le serveur poursuit — le rouvrir et relancer le téléchargement de ce
  compte la retrouve (409), quelle que soit la période. Aucun secret n'est
  jamais affiché.
-->
<script lang="ts" generics="R extends { lastRefreshed?: string | null }">
	import { onDestroy, type Snippet } from 'svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatSwissDate, formatSwissDateTime } from '$lib/features/reports/reports.api';
	import {
		getProviderDownload,
		startProviderDownload,
		type ProviderDownloadError,
	} from './provider-download.api';
	import {
		POLL_INTERVAL_MS,
		afterPollError,
		failureLines,
		progressLabel,
		progressPercent,
		resumedNotice,
		startRefusal,
		type Period,
	} from './provider-download';

	interface Props {
		open: boolean;
		onOpenChange: (open: boolean) => void;
		/** Segment de route du prestataire (`paypal` → `/api/v1/paypal-downloads`). */
		provider: string;
		/** Libellé du compte du prestataire choisi. */
		account: string;
		/** Titre du dialogue (« Télécharger depuis PayPal »). */
		title: string;
		/** Nom d'une étape (« fenêtre »), déjà traduit. */
		stepLabel: string;
		/** Texte permanent propre au prestataire (délai de publication, historique, tranches). */
		notice: string;
		/** Mention « bac à sable » (hôte d'essai). */
		sandbox?: boolean;
		/** Période proposée à l'ouverture. */
		defaultPeriod: Period;
		/** Libellé d'un code d'erreur de ligne (`details.lines[].code`), propre au prestataire. */
		lineLabel?: (code: string) => string;
		/** Le compte rendu d'une tâche réussie. */
		report: Snippet<[R]>;
		/** Appelé à la fin réussie d'une tâche (la page recharge sa revue). */
		onDone?: (result: R) => void;
	}

	let {
		open,
		onOpenChange,
		provider,
		account,
		title,
		stepLabel,
		notice,
		sandbox = false,
		defaultPeriod,
		lineLabel,
		report,
		onDone,
	}: Props = $props();

	type Phase = 'form' | 'running' | 'done' | 'failed' | 'lost';

	let phase = $state<Phase>('form');
	let from = $state('');
	let to = $state('');
	let starting = $state(false);
	let refusal = $state<string | null>(null);
	let steps = $state<{ done: number; total: number | null }>({ done: 0, total: null });
	let pausedSeconds = $state<number | null>(null);
	let result = $state<R | null>(null);
	let failure = $state<ProviderDownloadError | null>(null);
	/** La période d'une tâche reprise sur `409`, quand elle diffère de celle demandée. */
	let resumed = $state<Period | null>(null);

	// Garde de génération : toute interrogation d'une génération passée s'arrête
	// (fermeture, démontage, nouveau lancement).
	let generation = 0;
	let timer: ReturnType<typeof setTimeout> | null = null;
	let wasOpen = false;

	function stopPolling() {
		generation += 1;
		if (timer !== null) clearTimeout(timer);
		timer = null;
	}

	// À chaque ouverture : formulaire neuf, période proposée ; à la fermeture : le suivi s'arrête.
	$effect(() => {
		if (open && !wasOpen) {
			phase = 'form';
			from = defaultPeriod.from;
			to = defaultPeriod.to;
			refusal = null;
			result = null;
			failure = null;
			resumed = null;
			starting = false;
		}
		if (!open && wasOpen) stopPolling();
		wasOpen = open;
	});

	onDestroy(stopPolling);

	async function start() {
		if (starting) return;
		starting = true;
		refusal = null;
		resumed = null;
		const requested: Period = { from, to };
		// Une fermeture (ou un démontage) pendant le `POST` change la génération :
		// sa réponse ne lance alors aucun suivi (revue P1, L1).
		const myGen = generation;
		try {
			const { jobId } = await startProviderDownload(provider, { account, ...requested });
			if (myGen !== generation) return;
			track(jobId);
		} catch (err) {
			if (myGen !== generation) return;
			const outcome = startRefusal(err);
			if ('resumeJobId' in outcome) {
				resumed = resumedNotice(outcome.running, requested);
				track(outcome.resumeJobId);
			} else refusal = outcome.message;
		} finally {
			starting = false;
		}
	}

	/** `onDone`, une seule fois : une exception de l'appelant ne relance pas le suivi (revue P1, L2). */
	function notifyDone(r: R) {
		try {
			onDone?.(r);
		} catch (err) {
			console.error('[provider-download] onDone a échoué', err);
		}
	}

	/** Suit la tâche `jobId` jusqu'à son issue (ou la fermeture du dialogue). */
	function track(jobId: number) {
		stopPolling();
		const myGen = generation;
		phase = 'running';
		steps = { done: 0, total: null };
		pausedSeconds = null;
		let errors = 0;
		const poll = async () => {
			timer = null;
			if (myGen !== generation) return;
			try {
				const view = await getProviderDownload<R>(provider, jobId);
				if (myGen !== generation) return;
				errors = 0;
				steps = view.steps;
				pausedSeconds = view.pausedUntilSeconds;
				if (view.state === 'done') {
					// Une tâche finie clôt le suivi ; sans compte rendu, rien à montrer
					// (revue P1, L2).
					if (!view.result) {
						phase = 'lost';
						return;
					}
					result = view.result;
					phase = 'done';
					notifyDone(view.result);
					return;
				}
				if (view.state === 'failed') {
					failure = view.error ?? { code: 'INTERNAL_ERROR', message: '' };
					phase = 'failed';
					return;
				}
			} catch (err) {
				if (myGen !== generation) return;
				errors += 1;
				if (afterPollError(err, errors) === 'lost') {
					phase = 'lost';
					return;
				}
			}
			timer = setTimeout(poll, POLL_INTERVAL_MS);
		};
		void poll();
	}

	let percent = $derived(progressPercent(steps));
	let lastRefreshed = $derived(result?.lastRefreshed ?? null);
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-2xl" data-testid="provider-download-dialog">
		<Dialog.Header>
			<Dialog.Title>{title}</Dialog.Title>
		</Dialog.Header>

		<p class="text-sm text-text-muted" data-testid="provider-download-account">
			{i18nMsg('provider-download-account', 'Compte : { $account }', { account })}
			{#if sandbox}
				<span class="ml-2 rounded bg-surface-alt px-2 py-0.5 text-xs" data-testid="provider-download-sandbox">
					{i18nMsg('provider-download-sandbox', 'bac à sable')}
				</span>
			{/if}
		</p>
		<p class="text-sm" data-testid="provider-download-notice">{notice}</p>

		{#if resumed && phase !== 'form'}
			<p class="rounded bg-amber-50 p-3 text-sm text-amber-900" role="status" data-testid="provider-download-resumed">
				{i18nMsg(
					'provider-download-resumed',
					'Un téléchargement du { $from } au { $to } est déjà en cours pour ce compte : son suivi est repris. Lancez votre période quand il sera terminé.',
					{ from: formatSwissDate(resumed.from), to: formatSwissDate(resumed.to) },
				)}
			</p>
		{/if}

		{#if phase === 'form'}
			<div class="flex flex-wrap items-end gap-4">
				<div>
					<label for="provider-download-from" class="mb-1 block text-sm font-medium text-text">
						{i18nMsg('provider-download-from', 'Du')}
					</label>
					<input
						id="provider-download-from"
						type="date"
						bind:value={from}
						class="rounded border border-border px-3 py-2 text-sm"
						data-testid="provider-download-from"
					/>
				</div>
				<div>
					<label for="provider-download-to" class="mb-1 block text-sm font-medium text-text">
						{i18nMsg('provider-download-to', 'Au')}
					</label>
					<input
						id="provider-download-to"
						type="date"
						bind:value={to}
						class="rounded border border-border px-3 py-2 text-sm"
						data-testid="provider-download-to"
					/>
				</div>
			</div>
			{#if refusal}
				<p class="text-sm text-red-600" role="alert" data-testid="provider-download-refusal">{refusal}</p>
			{/if}
		{:else if phase === 'running'}
			<div data-testid="provider-download-running">
				<p class="text-sm" data-testid="provider-download-progress">{progressLabel(steps, stepLabel)}</p>
				{#if percent !== null}
					<div
						class="mt-2 h-2 w-full overflow-hidden rounded bg-surface-alt"
						role="progressbar"
						aria-valuemin={0}
						aria-valuemax={100}
						aria-valuenow={percent}
						data-testid="provider-download-bar"
					>
						<div class="h-full bg-primary" style="width: {percent}%"></div>
					</div>
				{/if}
				{#if pausedSeconds !== null}
					<p class="mt-2 text-sm text-text-muted" data-testid="provider-download-paused">
						{i18nMsg('provider-download-paused', 'Le prestataire demande de patienter : reprise dans { $seconds } s.', {
							seconds: String(pausedSeconds),
						})}
					</p>
				{/if}
				<p class="mt-2 text-xs text-text-muted">
					{i18nMsg(
						'provider-download-close-hint',
						'Vous pouvez fermer cette fenêtre : le téléchargement continue, et le relancer pour ce compte le retrouve, quelle que soit la période.',
					)}
				</p>
			</div>
		{:else if phase === 'done' && result}
			<div data-testid="provider-download-done">
				{@render report(result)}
				{#if lastRefreshed}
					<p class="mt-2 text-sm text-text-muted" data-testid="provider-download-last-refreshed">
						{i18nMsg('provider-download-last-refreshed', 'Données du prestataire à jour jusqu’au { $date }.', {
							date: formatSwissDateTime(lastRefreshed),
						})}
					</p>
				{/if}
			</div>
		{:else if phase === 'failed' && failure}
			<div class="rounded border border-red-200 bg-red-50 p-3 text-sm" role="alert" data-testid="provider-download-failed">
				{#each failureLines(failure, lineLabel) as line, i (i)}
					<p data-testid="provider-download-failed-line">{line}</p>
				{/each}
			</div>
		{:else if phase === 'lost'}
			<p class="rounded border border-red-200 bg-red-50 p-3 text-sm" role="alert" data-testid="provider-download-lost">
				{i18nMsg(
					'provider-download-lost',
					'Suivi perdu : relancez la même période, rien n’est doublé.',
				)}
			</p>
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={() => onOpenChange(false)} data-testid="provider-download-close">
				{i18nMsg('provider-download-close', 'Fermer')}
			</Button>
			{#if phase === 'form'}
				<Button onclick={start} disabled={starting || from === '' || to === ''} data-testid="provider-download-start">
					{starting
						? i18nMsg('provider-download-starting', 'Lancement…')
						: i18nMsg('provider-download-start', 'Télécharger')}
				</Button>
			{:else if phase !== 'running'}
				<Button variant="outline" onclick={() => (phase = 'form')} data-testid="provider-download-again">
					{i18nMsg('provider-download-again', 'Nouvelle période')}
				</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
