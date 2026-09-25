<!--
  Story 25-3-a-1 (#414) — Les règlements d'une facture, et leur annulation.

  Présentationnel : reçoit `settlements` déjà triés serveur (du plus ancien au
  plus récent) et délègue l'annulation au parent (`onCancel`), qui confirme,
  appelle l'API et relit la facture.

  ⛔ **Le bouton ne s'affiche que si le serveur a dit `cancellable`** — calculé
  par la fonction même qui refuserait le clic. Sinon, le MOTIF prend sa place :
  un bouton qui échoue au clic est le défaut que la Story 24-4a a déjà payé.
  ⚠️ Le rôle n'entre pas dans `cancellable` : c'est `canManage` qui masque le
  bouton pour un rôle en lecture seule, le serveur refusant de toute façon.

  Namespace i18n : clés `invoices-*` (composant sous `features/invoices/`,
  contrainte lint #30) ; les motifs communs viennent du module partagé.
-->
<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatInvoiceTotal } from './invoice-helpers';
	import { invoiceSettlementCancelMessage } from './settlement-cancel';
	import type { InvoiceSettlementResponse } from './invoices.types';

	let {
		settlements,
		canManage,
		onCancel,
	}: {
		settlements: InvoiceSettlementResponse[];
		canManage: boolean;
		onCancel: (settlement: InvoiceSettlementResponse) => void;
	} = $props();

	function typeLabel(t: InvoiceSettlementResponse['settlementType']): string {
		return t === 'bank_transfer'
			? i18nMsg('invoices-settlements-type-bank', 'Virement bancaire')
			: i18nMsg('invoices-settlements-type-internal', 'Espèces ou autre compte');
	}
</script>

{#if settlements.length > 0}
	<section class="space-y-2" data-testid="invoice-settlements">
		<h2 class="text-lg font-semibold">
			{i18nMsg('invoices-settlements-title', 'Règlements')}
		</h2>
		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">{i18nMsg('invoices-settlements-col-date', 'Date')}</th>
					<th class="py-2 pr-2 text-right">
						{i18nMsg('invoices-settlements-col-amount', 'Montant')}
					</th>
					<th class="py-2 pr-2">{i18nMsg('invoices-settlements-col-mode', 'Mode')}</th>
					<th class="py-2 pr-2">{i18nMsg('invoices-settlements-col-entry', 'Écriture')}</th>
					<th class="py-2 pr-2"></th>
				</tr>
			</thead>
			<tbody>
				{#each settlements as s (s.id)}
					<tr class="border-b border-border align-top" data-testid="invoice-settlement-row">
						<td class="py-2 pr-2">{s.settledOn}</td>
						<td class="py-2 pr-2 text-right font-mono">{formatInvoiceTotal(s.amount)}</td>
						<td class="py-2 pr-2">{typeLabel(s.settlementType)}</td>
						<td class="py-2 pr-2">
							<a class="underline" href="/journal-entries/{s.journalEntryId}">
								{i18nMsg('invoices-settlements-entry-link', "Voir l'écriture")}
							</a>
						</td>
						<td class="py-2 pr-2 text-right">
							{#if s.cancellable}
								{#if canManage}
									<Button
										variant="outline"
										size="sm"
										onclick={() => onCancel(s)}
										data-testid="invoice-settlement-cancel"
									>
										{i18nMsg('invoices-settlement-cancel-button', 'Annuler le règlement')}
									</Button>
								{/if}
							{:else if s.cancelBlockedBy}
								<span
									class="text-xs text-text-muted"
									data-testid="invoice-settlement-cancel-blocked"
								>
									{invoiceSettlementCancelMessage(s.cancelBlockedBy, s.cancelBlockedLabel)}
								</span>
							{/if}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</section>
{/if}
