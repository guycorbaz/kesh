<!--
  Story 34-3 (#967, AC-4, C-e342-1) — avis débiteurs / créanciers, sous le sélecteur de
  contrepartie d'une écriture manuelle (ou sous la ligne concernée d'un éclatement).

  Une écriture manuelle sur le compte débiteurs ou créanciers ne règle aucune facture : elle
  resterait due, sans lettrage, et rien ne le signalerait. L'avis le dit et renvoie au geste
  juste (règlement depuis la fiche puis rattachement ; pour un fournisseur, facture validée
  puis acceptée ici). Il ne bloque rien : signaler, jamais bloquer. Il ne s'affiche que pour
  le rôle explicite du compte (`claimRoleOf`), jamais pour un numéro.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import { claimRoleOf } from './reconciliation-counterparties';

	type Props = {
		/** Les comptes proposés, où se lit le compte choisi. */
		accounts: readonly AccountResponse[];
		/** Le compte choisi, `null` tant qu'aucun ne l'est. */
		accountId: number | null;
		/** Ligne d'un éclatement (`data-line`), absente pour l'écriture manuelle. */
		line?: number;
	};

	let { accounts, accountId, line }: Props = $props();

	const role = $derived(
		accountId === null ? null : claimRoleOf(accounts.find((a) => a.id === accountId)),
	);
</script>

{#if role !== null}
	<p
		class="mt-1 rounded-md border border-amber-400 bg-amber-50 px-3 py-2 text-xs text-amber-900"
		data-testid="counterparty-claim-account-notice"
		data-role={role}
		data-line={line}
		role="note"
	>
		{#if role === 'Receivable'}
			{i18nMsg(
				'reconciliation-manual-claim-notice-receivable',
				'Ce compte est celui des débiteurs. Une écriture manuelle ne règle aucune facture client : elle resterait due. Pour un paiement de client, enregistrez le règlement depuis la fiche de la facture, puis rattachez-y la transaction.',
			)}
		{:else}
			{i18nMsg(
				'reconciliation-manual-claim-notice-payable',
				"Ce compte est celui des créanciers. Une écriture manuelle ne règle aucune facture fournisseur : elle resterait à payer. Pour le paiement d'un fournisseur, saisissez et validez la facture sans la payer depuis sa fiche : elle vous sera proposée ici, à accepter. Si elle est déjà payée, rattachez la transaction à ce paiement.",
			)}
		{/if}
	</p>
{/if}
