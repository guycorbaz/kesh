<!--
  Story 28-5b (#526) — « Rejetée le … » et « Annuler le rejet », dans la
  colonne *Rapprochement* du détail d'un import bancaire.

  Contrat : `tx`, `canManage`, `onUnrejected()`.
  - Transaction non rejetée (au sens d'`isRejected`) ⇒ le composant ne rend RIEN.
  - Rejetée ⇒ la mention datée, pour TOUS les rôles : c'est le seul écran qui
    montre une transaction rejetée (elle a quitté la liste de la réconciliation).
  - `canManage` (Admin, Comptable) ⇒ en plus, le bouton *Annuler le rejet* :
    style de lien, non destructif, sans dialogue — le geste est réversible
    (rejeter de nouveau) et ne passe aucune écriture (C-28-5-9).
  - Clic ⇒ bouton désactivé pendant l'appel (un double-clic n'envoie qu'une
    requête) ; succès ⇒ notification puis `onUnrejected()` (la page relit son
    détail : la mention disparaît) ; échec ⇒ message du serveur, déjà traduit,
    puis `onUnrejected()` aussi — l'état a pu changer (rejet annulé ailleurs,
    transaction rapprochée ou supprimée) : on relit.
-->
<script lang="ts">
	import { i18nLocale, i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { notifyError, notifySuccess } from '$lib/shared/utils/notify';
	import type { BankTransactionDto } from '$lib/features/bank-import/bank-import.types';
	import { unrejectTransaction } from './reconciliation.api';
	import { isRejected } from './reconciliation-unreject';

	type Props = {
		tx: BankTransactionDto;
		canManage: boolean;
		onUnrejected: () => void;
	};

	let { tx, canManage, onUnrejected }: Props = $props();

	let submitting = $state(false);

	// C-28-5-15 — la date seule : l'heure n'aide pas à retrouver une transaction.
	const dateFormat = $derived(new Intl.DateTimeFormat(i18nLocale(), { dateStyle: 'medium' }));
	const rejectedOn = $derived(
		tx.autoMatchRejectedAt != null ? dateFormat.format(new Date(tx.autoMatchRejectedAt)) : '',
	);

	async function unreject() {
		if (submitting) return;
		submitting = true;
		try {
			await unrejectTransaction(tx.id);
			notifySuccess(
				i18nMsg(
					'reconciliation-unreject-success',
					'Rejet annulé : la transaction est de nouveau proposée à la réconciliation, dont la liste montre les 100 plus récentes en attente.',
				),
			);
		} catch (err) {
			notifyError(errorMessageOf(err));
		} finally {
			submitting = false;
		}
		onUnrejected();
	}
</script>

{#if isRejected(tx)}
	<span class="text-text-muted" data-testid="detail-tx-rejected-on">
		{i18nMsg('reconciliation-unreject-rejected-on', 'Rejetée le { $date }', { date: rejectedOn })}
	</span>
	{#if canManage}
		<button
			type="button"
			class="ml-2 text-primary underline disabled:opacity-50"
			data-testid="detail-tx-unreject"
			disabled={submitting}
			onclick={unreject}
		>
			{i18nMsg('reconciliation-unreject-button', 'Annuler le rejet')}
		</button>
	{/if}
{/if}
