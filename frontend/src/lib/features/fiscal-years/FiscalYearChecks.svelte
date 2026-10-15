<script lang="ts">
	// Story 29-1a (#232, AC5) — le panneau « Préparer la clôture » : les sept
	// contrôles préalables, dans l'ordre que rend le serveur, chacun avec son état
	// et un lien vers l'écran où l'on agit. Il ne recalcule rien : statuts, compteurs
	// et écarts sont ceux du serveur. Il ne bloque rien non plus.
	//
	// ⛔ Trois états, jamais confondus (M-a10, M-a22) : pendant le chargement, un
	// état d'attente et AUCUNE ligne de contrôle ; après un échec, « indisponible » ;
	// « Rien à signaler » seulement sur une réponse reçue.
	import { CircleCheck, CircleX, Info, OctagonAlert, TriangleAlert } from '@lucide/svelte';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatChfBalance } from '$lib/features/bank-accounts/format';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { checkHelp, checkTitle, entryReasonText, notCheckableText } from './closing-checks-labels';
	import type {
		ClosingCheck,
		ClosingChecksResponse,
		ClosingFiscalYearRef,
		NextFiscalYearRef,
		PendingByAccount,
		StatementIssue
	} from './fiscal-years.types';

	interface Props {
		/** `loading` : requête en cours ; `error` : échec ; `ready` : `checks` reçus. */
		loadState: 'loading' | 'error' | 'ready';
		checks: ClosingChecksResponse | null;
		/** Admin / Comptable : le lien « Créer l’exercice suivant » de K7. */
		onCreateFiscalYear?: () => void;
	}
	let { loadState, checks, onCreateFiscalYear }: Props = $props();

	function severityClass(c: ClosingCheck): string {
		if (c.status === 'ok') return 'text-success';
		switch (c.severity) {
			case 'blocking':
			case 'anomaly':
				return 'text-error';
			case 'warning':
				return 'text-warning';
			case 'info':
				return 'text-info';
		}
	}

	function severityLabel(c: ClosingCheck): string {
		switch (c.severity) {
			case 'blocking':
				return i18nMsg('fiscal-years-checks-severity-blocking', 'Bloquant');
			case 'warning':
				return i18nMsg('fiscal-years-checks-severity-warning', 'À vérifier');
			case 'info':
				return i18nMsg('fiscal-years-checks-severity-info', 'Pour information');
			case 'anomaly':
				return i18nMsg('fiscal-years-checks-severity-anomaly', 'Anomalie');
		}
	}

	function statementText(issue: StatementIssue): string {
		const name = issue.name;
		switch (issue.reason) {
			case 'MISSING':
				return i18nMsg(
					'fiscal-years-checks-k3-missing',
					'{ $name } : aucun relevé arrêté dans l’exercice.',
					{ name }
				);
			case 'ENDS_BEFORE_YEAR_END':
				return i18nMsg(
					'fiscal-years-checks-k3-ends-before',
					'{ $name } : le dernier relevé de l’exercice s’arrête le { $date }, avant sa fin.',
					{ name, date: issue.statementDate ? formatSwissDate(issue.statementDate) : '' }
				);
			case 'GAP':
				return i18nMsg(
					'fiscal-years-checks-k3-gap',
					'{ $name } : écart de { $amount } entre les livres et le relevé du { $date }.',
					{
						name,
						amount: issue.gap === null ? '' : formatChfBalance(issue.gap),
						date: issue.statementDate ? formatSwissDate(issue.statementDate) : ''
					}
				);
		}
	}

	function pendingText(a: PendingByAccount): string {
		return a.rejectedCount > 0
			? i18nMsg(
					'fiscal-years-checks-k2-account-rejected',
					'{ $name } : { $count }, dont { $rejected } écartée(s) des propositions',
					{ name: a.name, count: a.count, rejected: a.rejectedCount }
				)
			: i18nMsg('fiscal-years-checks-k2-account', '{ $name } : { $count }', {
					name: a.name,
					count: a.count
				});
	}

	// Accès typés aux détails : la forme est fixée par le code du contrôle.
	function pendingAccounts(c: ClosingCheck): PendingByAccount[] {
		return (c.details.accounts ?? []) as PendingByAccount[];
	}
	function statementIssues(c: ClosingCheck): StatementIssue[] {
		return (c.details.accounts ?? []) as StatementIssue[];
	}
	function earlierName(c: ClosingCheck): string {
		return (c.details.fiscalYear as ClosingFiscalYearRef | null | undefined)?.name ?? '';
	}
	function nextName(c: ClosingCheck): string {
		return (c.details.fiscalYear as NextFiscalYearRef | null | undefined)?.name ?? '';
	}

	function invoicesHref(c: ClosingChecksResponse): string {
		return `/invoices?status=draft&dateFrom=${c.startDate}&dateTo=${c.endDate}`;
	}
</script>

{#if loadState === 'loading'}
	<p class="text-sm text-text-muted" role="status" data-testid="fiscal-year-checks-loading">
		{i18nMsg('fiscal-years-checks-loading', 'Lecture des contrôles…')}
	</p>
{:else if loadState === 'error' || checks === null}
	<p class="text-sm text-error" role="alert" data-testid="fiscal-year-checks-error">
		{i18nMsg(
			'fiscal-years-checks-error',
			'Contrôles indisponibles pour le moment : fermez ce panneau, puis ouvrez-le de nouveau pour réessayer.'
		)}
	</p>
{:else}
	<p class="mb-3 text-sm text-text-muted">
		{i18nMsg(
			'fiscal-years-checks-intro',
			'Ces contrôles ne bloquent rien : ils signalent ce qui reste à faire ou paraît anormal dans l’exercice. Ils lisent les comptes à cet instant.'
		)}
	</p>
	<ul class="flex flex-col gap-3" data-testid="fiscal-year-checks-list">
		{#each checks.checks as check (check.code)}
			<li
				class="rounded-md border border-border p-3"
				data-testid="fiscal-year-checks-row-{check.code}"
				data-status={check.status}
			>
				<div class="flex items-start gap-2">
					<span class="mt-0.5 {severityClass(check)}" aria-hidden="true">
						{#if check.status === 'ok'}
							<CircleCheck class="h-4 w-4" />
						{:else if check.severity === 'blocking'}
							<OctagonAlert class="h-4 w-4" />
						{:else if check.severity === 'anomaly'}
							<CircleX class="h-4 w-4" />
						{:else if check.severity === 'warning'}
							<TriangleAlert class="h-4 w-4" />
						{:else}
							<Info class="h-4 w-4" />
						{/if}
					</span>
					<div class="flex-1">
						<p class="text-sm font-medium text-text">
							{checkTitle(check.code)}
							{#if check.status === 'attention'}
								<span class="ml-1 text-xs font-normal {severityClass(check)}">
									— {severityLabel(check)}
								</span>
							{/if}
						</p>
						<p class="text-xs text-text-muted">{checkHelp(check.code)}</p>
						<div class="mt-1 text-sm text-text" data-testid="fiscal-year-checks-status-{check.code}">
							{#if check.status === 'ok' && check.code !== 'NEXT_FISCAL_YEAR'}
								<span class="text-success">{i18nMsg('fiscal-years-checks-ok', 'Rien à signaler')}</span>
							{:else if check.code === 'EARLIER_FISCAL_YEAR_OPEN'}
								{i18nMsg(
									'fiscal-years-checks-k1-attention',
									'L’exercice « { $name } », plus ancien, est encore ouvert : clôturez-le d’abord.',
									{ name: earlierName(check) }
								)}
							{:else if check.code === 'PENDING_BANK_TRANSACTIONS'}
								<p>
									{i18nMsg(
										'fiscal-years-checks-k2-attention',
										'{ $count } transaction(s) bancaire(s) de l’exercice en attente de rapprochement.',
										{ count: check.count }
									)}
								</p>
								<ul class="ml-4 list-disc text-xs">
									{#each pendingAccounts(check) as a (a.bankAccountId)}
										<li>{pendingText(a)}</li>
									{/each}
								</ul>
								<a class="text-xs text-primary underline" href="/reconciliation" data-testid="fiscal-year-checks-link-{check.code}">
									{i18nMsg('fiscal-years-checks-k2-link', 'Ouvrir le rapprochement')}
								</a>
							{:else if check.code === 'BANK_STATEMENT_AT_YEAR_END'}
								<ul class="ml-4 list-disc text-xs">
									{#each statementIssues(check) as issue (issue.bankAccountId)}
										<li>{statementText(issue)}</li>
									{/each}
								</ul>
								<a class="text-xs text-primary underline" href="/bank-accounts" data-testid="fiscal-year-checks-link-{check.code}">
									{i18nMsg('fiscal-years-checks-k3-link', 'Ouvrir les comptes bancaires')}
								</a>
							{:else if check.code === 'DRAFT_INVOICES'}
								<p>
									{i18nMsg(
										'fiscal-years-checks-k4-attention',
										'{ $count } facture(s) en brouillon datée(s) dans l’exercice.',
										{ count: check.count }
									)}
								</p>
								<ul class="ml-4 list-disc text-xs">
									{#each check.details.invoices ?? [] as inv (inv.id)}
										<li>
											<a class="underline" href="/invoices/{inv.id}">{inv.label}</a>
											— {formatSwissDate(inv.date)}
										</li>
									{/each}
								</ul>
								<a class="text-xs text-primary underline" href={invoicesHref(checks)} data-testid="fiscal-year-checks-link-{check.code}">
									{i18nMsg('fiscal-years-checks-k4-link', 'Voir les brouillons de l’exercice')}
								</a>
							{:else if check.code === 'IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE'}
								<p>
									{i18nMsg(
										'fiscal-years-checks-k5-attention',
										'{ $count } document(s) à traiter, toutes dates confondues.',
										{ count: check.count }
									)}
								</p>
								<a class="text-xs text-primary underline" href="/supplier-invoices/import" data-testid="fiscal-year-checks-link-{check.code}">
									{i18nMsg('fiscal-years-checks-k5-link', 'Ouvrir les documents à traiter')}
								</a>
							{:else if check.code === 'INCONSISTENT_ENTRIES'}
								<p>
									{i18nMsg(
										'fiscal-years-checks-k6-attention',
										'{ $count } écriture(s) incohérente(s).',
										{ count: check.count }
									)}
								</p>
								<ul class="ml-4 list-disc text-xs">
									{#each check.details.entries ?? [] as e (e.id)}
										<li>
											<a class="underline" href="/journal-entries/{e.id}" data-testid="fiscal-year-checks-entry-{e.id}">
												{i18nMsg('fiscal-years-checks-k6-entry', 'N° { $number } ({ $year }, { $date })', {
													number: e.entryNumber,
													year: e.fiscalYearName,
													date: formatSwissDate(e.entryDate)
												})}
											</a>
											: {e.reasons.map(entryReasonText).join(', ')}
										</li>
									{/each}
								</ul>
							{:else if check.code === 'NEXT_FISCAL_YEAR'}
								{#if check.status === 'ok'}
									<span class="text-success">
										{i18nMsg('fiscal-years-checks-k7-ok', '« { $name } » suit cet exercice.', {
											name: nextName(check)
										})}
									</span>
								{:else}
									<p>
										{i18nMsg(
											'fiscal-years-checks-k7-attention',
											'Aucun exercice ne suit celui-ci : créez l’exercice suivant pour y passer les écritures de l’année qui vient.'
										)}
									</p>
									{#if onCreateFiscalYear}
										<button
											type="button"
											class="text-xs text-primary underline"
											data-testid="fiscal-year-checks-link-{check.code}"
											onclick={onCreateFiscalYear}
										>
											{i18nMsg('fiscal-years-checks-k7-link', 'Créer l’exercice suivant')}
										</button>
									{/if}
								{/if}
							{/if}
							{#if check.code === 'BANK_STATEMENT_AT_YEAR_END' && (check.details.notCheckable ?? []).length > 0}
								<ul class="mt-1 ml-4 list-disc text-xs text-text-muted" data-testid="fiscal-year-checks-not-checkable">
									{#each check.details.notCheckable ?? [] as nc (nc.bankAccountId)}
										<li>{notCheckableText(nc.reason, nc.name)}</li>
									{/each}
								</ul>
							{/if}
						</div>
					</div>
				</div>
			</li>
		{/each}
	</ul>
{/if}
