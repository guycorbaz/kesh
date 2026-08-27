<script lang="ts">
	// Story 9-1 — Vue Bilan. Story 14-3c — section Capitaux propres par rôle.
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { accountRoleKey, type AccountRole } from '$lib/features/accounts/accounts.types';
	import { formatReportAmount, formatSwissDate, isReportEmpty, ledgerHref } from './reports.api';
	import type { AccountBalance, BalanceSheetDto } from './reports.types';

	interface Props {
		dto: BalanceSheetDto;
	}
	let { dto }: Props = $props();

	let empty = $derived(isReportEmpty('balance-sheet', dto));

	const fmt = formatReportAmount;

	// Story 14-3c — groupes de rôle de la section Capitaux propres, dans l'ordre reçu
	// du backend (tri par rang de rôle déjà fait côté serveur — NE PAS re-trier). Les
	// comptes de même rôle sont consécutifs, on les regroupe sous un sous-titre de rôle.
	let equityGroups = $derived.by(() => {
		const groups: { role: AccountRole | null; rows: AccountBalance[] }[] = [];
		for (const row of dto.equity) {
			const last = groups[groups.length - 1];
			if (last && last.role === row.role) {
				last.rows.push(row);
			} else {
				groups.push({ role: row.role, rows: [row] });
			}
		}
		return groups;
	});

	// Total capitaux propres = comptes physiques + 2 lignes calculées virtuelles.
	let totalEquityAll = $derived(
		(() => {
			try {
				return new Big(dto.totalEquity).plus(dto.retainedEarnings).plus(dto.equityResult).toString();
			} catch {
				return '0';
			}
		})(),
	);

	let equityBig = $derived(
		(() => {
			try {
				return new Big(dto.equityResult);
			} catch {
				return new Big(0);
			}
		})(),
	);

	let equityClass = $derived(
		equityBig.gt(0)
			? 'text-green-700 font-semibold'
			: equityBig.lt(0)
				? 'text-red-700 font-semibold'
				: 'text-gray-700',
	);

	// P16 — fallback inclut "(avant clôture)" pour cohérence avec la valeur i18n canonique
	// et préserve la nuance OR Art. 960b (résultat pre-clôture vs définitif).
	let equityLabel = $derived(
		equityBig.gt(0)
			? i18nMsg('reports-equity-result-profit', "Bénéfice de l'exercice")
			: equityBig.lt(0)
				? i18nMsg('reports-equity-result-loss', "Perte de l'exercice")
				: i18nMsg('reports-equity-result-section-title', "Résultat de l'exercice (avant clôture)"),
	);

	// Story 14-1 — Résultat reporté (report à-nouveau virtuel) = cumul P&L des exercices
	// antérieurs. Négatif → « Perte reportée ».
	let retainedBig = $derived(
		(() => {
			try {
				return new Big(dto.retainedEarnings);
			} catch {
				return new Big(0);
			}
		})(),
	);

	let retainedClass = $derived(
		retainedBig.gt(0)
			? 'text-green-700 font-semibold'
			: retainedBig.lt(0)
				? 'text-red-700 font-semibold'
				: 'text-gray-700',
	);

	// Story 14-3c (D1) : la ligne CALCULÉE doit être explicitement marquée « (calculé) »
	// pour la distinguer d'un compte physique de rôle RetainedEarnings itemisé au-dessus.
	// Le cas perte reste `reports-retained-earnings-loss` (« Perte reportée ») comme
	// aujourd'hui ; le cas bénéfice/nul utilise la nouvelle clé marquée calculée.
	let retainedLabel = $derived(
		retainedBig.lt(0)
			? i18nMsg('reports-retained-earnings-loss', 'Perte reportée')
			: i18nMsg('reports-retained-earnings-calculated', 'Résultat reporté (calculé)'),
	);
</script>

<section class="space-y-4">
	<header class="text-sm text-gray-600">
		<strong>{i18nMsg('reports-filter-period', 'Période')}:</strong>
		{formatSwissDate(dto.period.startDate)} — {formatSwissDate(dto.period.endDate)}
	</header>

	{#if empty}
		<p class="rounded bg-blue-50 p-4 text-blue-900" role="status">
			{i18nMsg('reports-error-no-entries-in-period', 'Aucune écriture dans la période sélectionnée.')}
		</p>
	{:else}
		<div class="grid gap-6 lg:grid-cols-2">
			<div>
				<h3 class="text-lg font-semibold">
					{i18nMsg('reports-section-assets', 'Actifs')}
				</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N° de compte')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.assets as a (a.accountId)}
							<tr class:opacity-60={!a.active}>
								<td class="px-2 py-1 font-mono">
									<a
										class="text-indigo-700 hover:underline"
										href={ledgerHref(a.accountId, dto.period.startDate, dto.period.endDate)}
										title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
									>{a.accountNumber}</a
									>
								</td>
								<td class="px-2 py-1">{a.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(a.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-assets', 'Total actifs')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalAssets)}</td>
						</tr>
					</tfoot>
				</table>
			</div>

			<div>
				<h3 class="text-lg font-semibold">
					{i18nMsg('reports-section-liabilities', 'Passifs')}
				</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N° de compte')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						{#each dto.liabilities as l (l.accountId)}
							<tr class:opacity-60={!l.active}>
								<td class="px-2 py-1 font-mono">
									<a
										class="text-indigo-700 hover:underline"
										href={ledgerHref(l.accountId, dto.period.startDate, dto.period.endDate)}
										title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
									>{l.accountNumber}</a
									>
								</td>
								<td class="px-2 py-1">{l.accountName}</td>
								<td class="px-2 py-1 text-right font-mono">{fmt(l.balance)}</td>
							</tr>
						{/each}
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-liabilities', 'Total passifs')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(dto.totalLiabilities)}</td>
						</tr>
					</tfoot>
				</table>
			</div>

			<!-- Story 14-3c : section Capitaux propres dédiée, groupée par rôle. Les comptes
			     physiques de fonds propres (ex-noyés dans les Passifs) apparaissent ici sous
			     un sous-titre de rôle, suivis des 2 lignes CALCULÉES distinctes (D1). -->
			<div class="lg:col-span-2">
				<h3 class="text-lg font-semibold">
					{i18nMsg('reports-section-equity', 'Capitaux propres')}
				</h3>
				<table class="mt-2 w-full border-collapse">
					<thead>
						<tr class="border-b bg-gray-50 text-left text-sm">
							<th class="px-2 py-1">{i18nMsg('reports-column-account-number', 'N° de compte')}</th>
							<th class="px-2 py-1">{i18nMsg('reports-column-account-name', 'Intitulé')}</th>
							<th class="px-2 py-1 text-right">{i18nMsg('reports-column-balance', 'Solde')}</th>
						</tr>
					</thead>
					<tbody>
						<!-- Clé = `group.role` : identité intrinsèque et STABLE du groupe. Le
					     backend garantit un seul groupe contigu par rôle (tri par rang) → clé
					     unique ; `is_equity_role` garantit un rôle non-null dans `equity` → pas
					     de collision de clé null. Ni l'index (réutilisation de nœud DOM si un
					     groupe disparaît, ECH pass 2) ni un fallback `?? 'none'` ne conviennent. -->
					{#each equityGroups as group (group.role)}
							{#if group.role}
								<tr class="bg-gray-100 text-sm font-semibold">
									<td colspan="3" class="px-2 py-1"
										>{i18nMsg(accountRoleKey(group.role), group.role)}</td
									>
								</tr>
							{/if}
							{#each group.rows as e (e.accountId)}
								<tr class:opacity-60={!e.active}>
									<td class="px-2 py-1 pl-4 font-mono">
										<a
											class="text-indigo-700 hover:underline"
											href={ledgerHref(e.accountId, dto.period.startDate, dto.period.endDate)}
											title={i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')}
										>{e.accountNumber}</a
										>
									</td>
									<td class="px-2 py-1">{e.accountName}</td>
									<td class="px-2 py-1 text-right font-mono">{fmt(e.balance)}</td>
								</tr>
							{/each}
						{/each}
						<tr>
							<td colspan="2" class="px-2 py-1 {retainedClass}">{retainedLabel}</td>
							<td class="px-2 py-1 text-right font-mono {retainedClass}"
								>{fmt(dto.retainedEarnings)}</td
							>
						</tr>
						<tr>
							<td colspan="2" class="px-2 py-1 {equityClass}">{equityLabel}</td>
							<td class="px-2 py-1 text-right font-mono {equityClass}">{fmt(dto.equityResult)}</td>
						</tr>
					</tbody>
					<tfoot>
						<tr class="border-t font-semibold">
							<td colspan="2" class="px-2 py-1"
								>{i18nMsg('reports-total-equity', 'Total capitaux propres')}</td
							>
							<td class="px-2 py-1 text-right font-mono">{fmt(totalEquityAll)}</td>
						</tr>
					</tfoot>
				</table>
			</div>
		</div>

		{#if !dto.equationHolds}
			<p class="rounded bg-red-50 p-3 text-sm text-red-900" role="alert">
				{i18nMsg(
					'reports-equation-warning',
					'⚠️ Équation bilan déséquilibrée (vérifier données source).',
				)}
			</p>
		{/if}
	{/if}
</section>
