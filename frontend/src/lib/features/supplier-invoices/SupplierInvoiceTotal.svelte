<!--
  Story v017-3b (#687, AC-10, C-v017-3-24, C-v017-3-26, C-v017-3-27) — le TTC d'une
  facture fournisseur sur sa fiche, et l'avertissement d'une facture EXISTANTE au total
  sous le centime.

  Depuis la v0.17.0, Kesh refuse de créer une ligne qui n'est pas au centime : un total
  sous le centime ne vient plus que d'une facture enregistrée avant. Kesh ne la modifie
  pas ; il l'affiche aux décimales exactes (`formatExactAmount` — « 100.0040 », jamais
  « 100.00 ») et l'avertit, en renvoyant au manuel. L'avertissement est **statique** : il
  ne promet aucun bouton et ne prescrit aucun geste, et il ne s'affiche **pas** pour une
  facture annulée, qui n'a plus de règlement.

  Le titre du § du manuel est donné en français dans les quatre catalogues : les manuels
  DE, IT et EN ne sont pas traduits.
-->
<script lang="ts">
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { formatExactAmount, hasSubCentime } from '$lib/features/invoices/invoice-helpers';
	import type { SupplierInvoiceStatus } from './supplier-invoices.types';

	interface Props {
		/** Le TTC de la facture (décimal sérialisé, quatre décimales). */
		totalAmount: string;
		status: SupplierInvoiceStatus;
	}

	let { totalAmount, status }: Props = $props();

	const exact = $derived(formatExactAmount(totalAmount));
	const warn = $derived(status !== 'cancelled' && hasSubCentime(totalAmount));
</script>

<span data-testid="supplier-invoice-total">{exact}</span>
{#if warn}
	<p
		class="mt-1 rounded border border-warning bg-warning-soft p-2 text-sm"
		role="note"
		data-testid="supplier-invoice-subcentime-warning"
	>
		{i18nMsg(
			'supplier-invoices-subcentime-total-warning',
			'Le total de cette facture ({ $amount }) n’est pas au centime : un paiement bancaire de ce montant ne pourra être rapproché d’aucune transaction de relevé. Voir le manuel, § « Une facture fournisseur au total sous le centime ».',
			{ amount: exact },
		)}
	</p>
{/if}
