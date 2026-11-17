<script lang="ts">
	// Story 37-5 — la réévaluation de fin d'exercice des comptes en devise : un seul
	// composant (AC-6, AC-9, DRY), en deux modes.
	//
	// - `full` : l'écran de l'action « Réévaluer les devises » — cours modifiables par
	//   devise (valables pour cette passation seulement : le champ n'écrit aucun cours
	//   saisi), cases par compte, « Passer les écritures ».
	// - `notice` : l'avis du contrôle K9 — l'aperçu lu sans `accountIds` ni `rates`
	//   (cours par défaut, présélection), cours en lecture seule, « Réévaluer » qui passe
	//   les écritures montrées. Pas de lien « Modifier les cours ou la sélection » (A36).
	//
	// Règles de relecture (AC-1, AC-4) : toute relecture porte la sélection courante et
	// les cours MODIFIÉS de ses seules devises ; seule la dernière réponse est retenue ;
	// à sélection vide, aucune relecture. Après une récupération de cours, un champ non
	// modifié reprend le nouveau cours par défaut. La passation reçoit les cours AFFICHÉS
	// des devises d'au moins un compte coché.
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { todayInZurich } from '$lib/shared/utils/local-date';
	import { formatChfBalance } from '$lib/features/bank-accounts/format';
	import { formatSwissDate, ledgerHref } from '$lib/features/reports/reports.api';
	import { fetchRates } from '$lib/features/exchange-rates/exchange-rates.api';
	import { fetchFailureText } from '$lib/features/exchange-rates/exchange-rates-labels';
	import { formatRate } from '$lib/features/exchange-rates/exchange-rates-grid';
	import { postRevaluation, previewRevaluation } from './fiscal-years.api';
	import {
		canFetchDay,
		canFetchMonth,
		canPost,
		closingKindReminder,
		dayNotYetPublished,
		differsFromDefault,
		isMonthlyFallback,
		kindLabel,
		monthFetchRequest,
		naturalAmount,
		originLabel,
		postRequest,
		previewRequest,
		sourceLabel
	} from './revaluation.helpers';
	import type { RevaluationPreview, RevaluationRate } from './fiscal-years.types';

	interface Props {
		fiscalYearId: number;
		mode: 'full' | 'notice';
		/** Après une passation réussie (identifiants des écritures). */
		onPosted?: (entryIds: number[]) => void;
		/** Avis de K9 : après une récupération de cours réussie (relire les contrôles). */
		onRatesFetched?: () => void;
		/** Le jour de Zurich (tests à horloge fixée). */
		today?: string;
	}
	let { fiscalYearId, mode, onPosted, onRatesFetched, today = todayInZurich() }: Props =
		$props();

	let preview = $state<RevaluationPreview | null>(null);
	let loadState = $state<'loading' | 'error' | 'ready'>('loading');
	let loadError = $state('');
	let inputs = $state<Record<string, string>>({});
	let modified = $state<Record<string, boolean>>({});
	let selected = $state<Set<number>>(new Set());
	let fetchMessage = $state<Record<string, string>>({});
	let fetching = $state<string | null>(null);
	let posting = $state(false);
	let postError = $state('');
	let posted = $state<number[]>([]);
	/** Relectures en cours : « Passer les écritures » attend la dernière. */
	let rereading = $state(0);
	let request = 0;

	const editable = $derived(mode === 'full');
	const locale = $derived(i18nLocale());

	/** Applique une réponse : les champs non modifiés reprennent le cours par défaut. */
	function apply(p: RevaluationPreview, keepSelection: boolean): void {
		preview = p;
		const next: Record<string, string> = { ...inputs };
		for (const r of p.rates) {
			if (!modified[r.currency]) next[r.currency] = r.rate ?? '';
		}
		inputs = next;
		if (!keepSelection) {
			selected = new Set(p.lines.filter((l) => l.selected).map((l) => l.accountId));
		}
		loadState = 'ready';
	}

	async function load(): Promise<void> {
		const token = ++request;
		loadState = 'loading';
		try {
			const p = await previewRevaluation(fiscalYearId, {});
			if (token !== request) return;
			modified = {};
			apply(p, false);
		} catch (err) {
			if (token !== request) return;
			loadError = errorMessageOf(err);
			loadState = 'error';
		}
	}

	/** Relecture avec la sélection courante et les cours modifiés ; rien à sélection vide. */
	async function reread(): Promise<void> {
		if (!preview || selected.size === 0) return;
		const token = ++request;
		rereading += 1;
		try {
			const p = await previewRevaluation(fiscalYearId, previewRequest(preview, inputs, selected));
			if (token !== request) return;
			apply(p, true);
		} catch (err) {
			if (token !== request) return;
			postError = errorMessageOf(err);
		} finally {
			rereading -= 1;
		}
	}

	function toggle(accountId: number, checked: boolean): void {
		const next = new Set(selected);
		if (checked) next.add(accountId);
		else next.delete(accountId);
		selected = next;
		postError = '';
		void reread();
	}

	function changeRate(rate: RevaluationRate, value: string): void {
		inputs = { ...inputs, [rate.currency]: value };
		modified = { ...modified, [rate.currency]: differsFromDefault(value, rate) };
		void reread();
	}

	async function fetchUnit(rate: RevaluationRate, unit: 'day' | 'month'): Promise<void> {
		if (!preview) return;
		fetching = `${rate.currency}-${unit}`;
		fetchMessage = { ...fetchMessage, [rate.currency]: '' };
		try {
			await fetchRates(
				unit === 'day' ? { date: preview.endDate } : monthFetchRequest(preview.endDate)
			);
			if (mode === 'notice') {
				// L'avis relit sans sélection ni cours (cours par défaut, présélection),
				// puis le panneau relit les contrôles (37-5-CR8-P2-2).
				await load();
				onRatesFetched?.();
			} else {
				// L'écran complet relit avec la sélection ; à sélection vide (devise sans
				// cours : aucune case cochable), l'aperçu se recharge entier, faute de quoi
				// le cours récupéré ne paraîtrait jamais (37-5-CR-P2-1).
				if (selected.size === 0) await load();
				else await reread();
			}
		} catch (err) {
			// 404 RATE_NOT_PUBLISHED, 503 RATE_SOURCE_UNAVAILABLE : le message, l'aperçu tel quel.
			fetchMessage = { ...fetchMessage, [rate.currency]: fetchFailureText(err) };
		} finally {
			fetching = null;
		}
	}

	async function submit(): Promise<void> {
		if (!preview || rereading > 0 || loadState === 'loading') return;
		posting = true;
		postError = '';
		try {
			const r = await postRevaluation(fiscalYearId, postRequest(preview, inputs, selected));
			posted = r.entryIds;
			onPosted?.(r.entryIds);
			// Les deux modes relisent : l'avis de K9 resté monté (un compte archivé
			// ou sans cours le garde en attention) ne montre pas les écarts passés.
			modified = {};
			await load();
		} catch (err) {
			postError = errorMessageOf(err);
		} finally {
			posting = false;
		}
	}

	onMount(() => {
		void load();
	});

	function chf(v: string | null): string {
		return v === null ? '—' : formatChfBalance(v);
	}
</script>

{#if loadState === 'loading' && preview === null}
	<p class="text-sm text-text-muted" role="status" data-testid="revaluation-loading">
		{i18nMsg('fiscal-years-revaluation-loading', 'Lecture de l’aperçu…')}
	</p>
{:else if loadState === 'error' && preview === null}
	<p class="text-sm text-error" role="alert" data-testid="revaluation-error">{loadError}</p>
{:else if preview}
	<div class="flex flex-col gap-3 text-sm" data-testid="revaluation-panel" data-mode={mode}>
		{#if loadState === 'error'}
			<!-- Une relecture échouée après une passation : l'aperçu montré est périmé. -->
			<p class="text-error" role="alert" data-testid="revaluation-error">{loadError}</p>
		{/if}
		<p class="text-text-muted" data-testid="revaluation-closing-kind">
			{closingKindReminder(preview, locale)} —
			<a class="text-primary underline" href="/settings/exchange-rates">
				{i18nMsg('fiscal-years-revaluation-rates-link', 'Cours de change')}
			</a>
		</p>

		<!-- Les cours, par devise -->
		<ul class="flex flex-col gap-2" data-testid="revaluation-rates">
			{#each preview.rates as rate (rate.currency)}
				<li class="rounded-md border border-border p-2" data-testid="revaluation-rate-{rate.currency}">
					<div class="flex flex-wrap items-center gap-2">
						<span class="w-12 font-medium">{rate.currency}</span>
						{#if editable && rate.defaultRate !== null}
							<!-- Sans cours retenu par la société, pas de champ : ses comptes ne se
							     cochent pas, un cours tapé ici ne partirait jamais. Le cours se
							     récupère ou se saisit à l'écran Cours de change (AC-1). -->
							<Input
								class="w-36"
								aria-label={i18nMsg('fiscal-years-revaluation-rate-input', 'Cours { $currency }', {
									currency: rate.currency
								})}
								data-testid="revaluation-rate-input-{rate.currency}"
								value={inputs[rate.currency] ?? ''}
								onchange={(e) => changeRate(rate, (e.currentTarget as HTMLInputElement).value)}
							/>
						{:else}
							<span data-testid="revaluation-rate-value-{rate.currency}">
								{rate.rate === null ? '—' : formatRate(rate.rate)}
							</span>
						{/if}
						{#if rate.rate !== null}
							<span class="text-text-muted" data-testid="revaluation-rate-source-{rate.currency}">
								{sourceLabel(rate)}
							</span>
							<span
								class={isMonthlyFallback(preview, rate) ? 'text-warning' : 'text-text-muted'}
								data-testid="revaluation-rate-kind-{rate.currency}"
							>
								{kindLabel(preview, rate, locale)}
							</span>
						{:else}
							<span class="text-warning" data-testid="revaluation-rate-missing-{rate.currency}">
								{i18nMsg(
									'fiscal-years-revaluation-rate-missing',
									'Aucun cours de clôture : récupérez-le, ou saisissez-le à l’écran Cours de change.'
								)}
							</span>
						{/if}
						{#if canFetchDay(preview, rate, today)}
							<Button
								variant="outline"
								size="sm"
								data-testid="revaluation-fetch-day-{rate.currency}"
								disabled={fetching !== null}
								onclick={() => fetchUnit(rate, 'day')}
							>
								{i18nMsg('fiscal-years-revaluation-fetch-day', 'Récupérer le cours du { $date }', {
									date: formatSwissDate(preview.endDate)
								})}
							</Button>
						{/if}
						{#if canFetchMonth(preview, rate, today)}
							<Button
								variant="outline"
								size="sm"
								data-testid="revaluation-fetch-month-{rate.currency}"
								disabled={fetching !== null}
								onclick={() => fetchUnit(rate, 'month')}
							>
								{i18nMsg('fiscal-years-revaluation-fetch-month', 'Récupérer le mois')}
							</Button>
						{/if}
					</div>
					{#if dayNotYetPublished(preview, rate, today)}
						<p class="mt-1 text-xs text-text-muted" data-testid="revaluation-day-not-published-{rate.currency}">
							{i18nMsg(
								'fiscal-years-revaluation-day-not-published',
								'Date de clôture pas encore passée : cours du jour non publié.'
							)}
						</p>
					{/if}
					{#if editable && differsFromDefault(inputs[rate.currency] ?? '', rate)}
						<p class="mt-1 text-xs text-text-muted" data-testid="revaluation-rate-default-{rate.currency}">
							{i18nMsg(
								'fiscal-years-revaluation-rate-default',
								'Cours retenu par la société : { $rate }. Ce cours ne vaut que pour cette passation : la clôture vérifie au cours retenu par la société.',
								{ rate: formatRate(rate.defaultRate) }
							)}
						</p>
					{/if}
					{#if fetchMessage[rate.currency]}
						<p class="mt-1 text-xs text-error" role="alert" data-testid="revaluation-fetch-error-{rate.currency}">
							{fetchMessage[rate.currency]}
						</p>
					{/if}
				</li>
			{/each}
		</ul>

		{#if preview.preselectionTruncated}
			<p class="text-warning" data-testid="revaluation-truncated">
				{i18nMsg(
					'fiscal-years-revaluation-truncated',
					'Plus de 498 comptes en devise : seuls les 498 premiers par numéro sont cochés. Passez la réévaluation en plusieurs fois.'
				)}
			</p>
		{/if}

		<!-- Les comptes : une ligne = une écriture (R9) -->
		<div class="overflow-x-auto">
			<table class="w-full text-xs" data-testid="revaluation-table">
				<thead>
					<tr class="text-left text-text-muted">
						{#if editable}<th class="p-1"></th>{/if}
						<th class="p-1">{i18nMsg('fiscal-years-revaluation-col-account', 'Compte')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-balance', 'Solde en devise')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-chf', 'CHF actuel')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-rate', 'Cours')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-target', 'CHF cible')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-previous', 'Déjà réévalué')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-after', 'Nouvelle valeur cumulée')}</th>
						<th class="p-1 text-right">{i18nMsg('fiscal-years-revaluation-col-delta', 'Écart')}</th>
						<th class="p-1">{i18nMsg('fiscal-years-revaluation-col-entry', 'Écriture prévue')}</th>
					</tr>
				</thead>
				<tbody>
					{#each preview.lines as line (line.accountId)}
						<tr
							class="border-t border-border {line.selectable ? '' : 'text-text-muted opacity-70'}"
							data-testid="revaluation-line-{line.accountId}"
						>
							{#if editable}
								<td class="p-1">
									<input
										type="checkbox"
										aria-label={i18nMsg('fiscal-years-revaluation-select', 'Réévaluer { $account }', {
											account: `${line.accountNumber} ${line.accountName}`
										})}
										data-testid="revaluation-select-{line.accountId}"
										checked={selected.has(line.accountId)}
										disabled={!line.selectable ||
											(!selected.has(line.accountId) && selected.size >= 498)}
										onchange={(e) => toggle(line.accountId, (e.currentTarget as HTMLInputElement).checked)}
									/>
								</td>
							{/if}
							<td class="p-1">
								<a
									class="underline"
									href={ledgerHref(line.accountId, preview.startDate, preview.endDate)}
									data-testid="revaluation-ledger-{line.accountId}"
								>
									{line.accountNumber} {line.accountName}
								</a>
								{#if line.reason === 'ACCOUNT_ARCHIVED'}
									<span class="block text-warning" data-testid="revaluation-archived-{line.accountId}">
										{i18nMsg(
											'fiscal-years-revaluation-archived',
											'Compte archivé : réactivez-le pour le réévaluer (la clôture le compte aussi).'
										)}
									</span>
								{:else if line.reason === 'ACCOUNT_NOT_POSTABLE'}
									<span class="block text-warning" data-testid="revaluation-not-postable-{line.accountId}">
										{i18nMsg(
											'fiscal-years-revaluation-not-postable',
											'Compte non imputable portant un solde : rendez-le imputable pour le réévaluer (la clôture le compte aussi).'
										)}
									</span>
								{:else if line.reason === 'RATE_UNAVAILABLE'}
									<span class="block text-warning">
										{i18nMsg('fiscal-years-revaluation-line-no-rate', 'Pas de cours de clôture pour cette devise.')}
									</span>
								{/if}
							</td>
							<td class="p-1 text-right">{naturalAmount(line, line.deviseNet)} {line.currency}</td>
							<td class="p-1 text-right">{chf(naturalAmount(line, line.chfNet))}</td>
							<td class="p-1 text-right">{line.rate === null ? '—' : formatRate(line.rate)}</td>
							<td class="p-1 text-right">
								{line.targetChf === null ? '—' : chf(naturalAmount(line, line.targetChf))}
							</td>
							<td class="p-1 text-right" data-testid="revaluation-previous-{line.accountId}">
								{chf(line.previousRevaluationTotal)}
								{#each line.previousEntryIds as entryId (entryId)}
									<a
										class="ml-1 underline"
										href="/journal-entries/{entryId}"
										data-testid="revaluation-previous-entry-{entryId}"
									>#{entryId}</a>
								{/each}
							</td>
							<td class="p-1 text-right" data-testid="revaluation-after-{line.accountId}">
								{chf(line.revaluationTotalAfter)}
							</td>
							<td class="p-1 text-right font-medium" data-testid="revaluation-delta-{line.accountId}">
								{chf(line.delta)}
							</td>
							<td class="p-1" data-testid="revaluation-entry-{line.accountId}">
								{#if line.label}
									<span class="block">{line.label}</span>
								{/if}
								{#if line.counterpartAccountId !== null}
									<span class="block text-text-muted">
										{i18nMsg(
											'fiscal-years-revaluation-counterpart',
											'Contrepartie : { $account } ({ $origin })',
											{
												account: `${line.counterpartAccountNumber ?? ''} ${line.counterpartAccountName ?? ''}`.trim(),
												origin: originLabel(line.fxAccountsOrigin)
											}
										)}
									</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<p data-testid="revaluation-totals">
			{i18nMsg('fiscal-years-revaluation-totals', 'Gains : { $gains } — Pertes : { $losses }', {
				gains: chf(preview.totals.gains),
				losses: chf(preview.totals.losses)
			})}
		</p>
		<p class="text-xs text-text-muted">
			{i18nMsg(
				'fiscal-years-revaluation-append-only',
				'Chaque passation ajoute ses écritures, sans modifier les précédentes ; une réévaluation se corrige par suppression de son écriture ou par un nouveau passage.'
			)}
		</p>

		{#if preview.blockers.length > 0}
			<ul class="flex flex-col gap-1 text-error" role="alert" data-testid="revaluation-blockers">
				{#each preview.blockers as b, i (i)}
					<li data-testid="revaluation-blocker-{b.code}">{b.message}</li>
				{/each}
			</ul>
		{/if}
		{#if postError}
			<p class="text-error" role="alert" data-testid="revaluation-post-error">{postError}</p>
		{/if}
		{#if posted.length > 0}
			<p class="text-success" data-testid="revaluation-posted">
				{i18nMsg('fiscal-years-revaluation-posted', '{ $count } écriture(s) passée(s) :', {
					count: posted.length
				})}
				{#each posted as entryId (entryId)}
					<a class="ml-1 underline" href="/journal-entries/{entryId}">#{entryId}</a>
				{/each}
			</p>
		{/if}

		<div class="flex justify-end">
			<Button
				type="button"
				data-testid="revaluation-submit"
				disabled={posting || rereading > 0 || loadState === 'loading' || !canPost(preview, selected)}
				onclick={submit}
			>
				{mode === 'full'
					? i18nMsg('fiscal-years-revaluation-submit', 'Passer les écritures')
					: i18nMsg('fiscal-years-revaluation-submit-notice', 'Réévaluer')}
			</Button>
		</div>
	</div>
{/if}
