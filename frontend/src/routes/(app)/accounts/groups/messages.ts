/**
 * Story 46-5b — les textes de l'écran « Créer des groupes » qui dépendent d'un
 * code : messages des propositions (codes de `group-proposals.ts`) et libellés
 * des types de compte. Une clé littérale par cas, pour que les gardes i18n
 * voient chaque clé (aucun gabarit de clé).
 */

import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import type { AccountType } from '$lib/features/accounts/accounts.types';
import type { ProposalMessage } from '$lib/features/accounts/group-proposals';

/** Le libellé traduit d'un type de compte (mêmes clés que l'écran du plan). */
export function typeLabel(t: AccountType | string): string {
	switch (t) {
		case 'Asset':
			return i18nMsg('account-type-asset', 'Actif');
		case 'Liability':
			return i18nMsg('account-type-liability', 'Passif');
		case 'Revenue':
			return i18nMsg('account-type-revenue', 'Produit');
		case 'Expense':
			return i18nMsg('account-type-expense', 'Charge');
		default:
			return String(t);
	}
}

/** Le texte d'un message de proposition, paramètres interpolés. */
export function proposalMessageText(m: ProposalMessage): string {
	const p = m.params;
	switch (m.code) {
		case 'numberEmpty':
			return i18nMsg('accounts-groups-msg-number-empty', 'Numéro du groupe à saisir.');
		case 'numberTooLong':
			return i18nMsg('accounts-groups-msg-number-too-long', 'Numéro trop long : { $max } octets au plus.', { max: p.max });
		case 'nameEmpty':
			return i18nMsg('accounts-groups-msg-name-empty', 'Nom du groupe à saisir.');
		case 'nameTooLong':
			return i18nMsg('accounts-groups-msg-name-too-long', 'Nom trop long : { $max } octets au plus.', { max: p.max });
		case 'mixedTypes':
			return i18nMsg(
				'accounts-groups-msg-mixed-types',
				"Types mêlés ({ $types }) : un groupe ne réunit que des comptes d'un même type. Décochez les comptes d'un type.",
				{ types: p.types.split(', ').map(typeLabel).join(', ') },
			);
		case 'empty':
			return i18nMsg('accounts-groups-msg-empty', 'Aucun compte à ranger.');
		case 'groupTypeMismatch':
			return i18nMsg(
				'accounts-groups-msg-group-type-mismatch',
				'Le compte { $number } « { $name } » est de type { $type }, différent des comptes à y ranger.',
				{ number: p.number, name: p.name, type: typeLabel(p.type) },
			);
		case 'groupHasEntries':
			return i18nMsg(
				'accounts-groups-msg-group-has-entries',
				'Le groupe { $number } « { $name } » porte des écritures : il ne peut pas recevoir de sous-comptes. Saisissez un autre numéro.',
				{ number: p.number, name: p.name },
			);
		case 'numberTakenEntries':
			return i18nMsg(
				'accounts-groups-msg-number-taken-entries',
				'Numéro pris : { $number } « { $name } » porte des écritures et ne peut pas devenir un groupe. Saisissez un autre numéro.',
				{ number: p.number, name: p.name },
			);
		case 'numberTakenArchived':
			return i18nMsg(
				'accounts-groups-msg-number-taken-archived',
				'Numéro pris par le compte archivé { $number } « { $name } ». Saisissez un autre numéro.',
				{ number: p.number, name: p.name },
			);
		case 'numberTakenReusable':
			return i18nMsg(
				'accounts-groups-msg-number-taken-reusable',
				'Numéro pris par { $number } « { $name } », sans écriture. Réutilisez-le comme groupe ou saisissez un autre numéro.',
				{ number: p.number, name: p.name },
			);
		case 'numberIsMember':
			return i18nMsg(
				'accounts-groups-msg-number-is-member',
				"Le numéro { $number } est celui d'un compte de la proposition.",
				{ number: p.number },
			);
		case 'numberDuplicate':
			return i18nMsg(
				'accounts-groups-msg-number-duplicate',
				'Le numéro { $number } est demandé par une autre proposition acceptée.',
				{ number: p.number },
			);
		case 'groupIsMember':
			return i18nMsg(
				'accounts-groups-msg-group-is-member',
				"Le compte { $number } est aussi un compte à ranger d'une autre proposition acceptée : rangez un niveau à la fois.",
				{ number: p.number },
			);
		case 'parentMissing':
			return i18nMsg(
				'accounts-groups-msg-parent-missing',
				"Les comptes n'ont pas de parent commun : choisissez le parent du groupe.",
			);
		case 'parentHasEntries':
			return i18nMsg(
				'accounts-groups-msg-parent-has-entries',
				'Le parent { $number } « { $name } » porte des écritures : choisissez un autre parent.',
				{ number: p.number, name: p.name },
			);
		case 'numberBankLinked':
			return i18nMsg(
				'accounts-groups-msg-number-bank-linked',
				'Le compte { $number } « { $name } » est le compte du compte bancaire « { $bank } » : il reçoit des écritures automatiques et ne peut pas devenir un groupe. Saisissez un autre numéro.',
				{ number: p.number, name: p.name, bank: p.bank },
			);
		case 'parentBankLinked':
			return i18nMsg(
				'accounts-groups-msg-parent-bank-linked',
				'Le parent { $number } « { $name } » est le compte du compte bancaire « { $bank } » : il ne peut pas devenir un groupe. Choisissez un autre parent.',
				{ number: p.number, name: p.name, bank: p.bank },
			);
		case 'root':
			return i18nMsg('accounts-groups-msg-root', 'Sans parent, le groupe sera une racine du plan.');
		case 'mixedNames':
			return i18nMsg(
				'accounts-groups-msg-mixed-names',
				'Ce groupe réunirait { $names } : leurs numéros ne suivent pas les dizaines. Décochez les comptes en trop, ou utilisez le mode « un seul groupe ».',
				{ names: p.names },
			);
		case 'reuseNotPostable':
			return i18nMsg(
				'accounts-groups-msg-reuse-not-postable',
				'Le compte { $number } « { $name } », sans écriture, deviendra le groupe et ne sera plus imputable.',
				{ number: p.number, name: p.name },
			);
		case 'attachExisting':
			return i18nMsg(
				'accounts-groups-msg-attach-existing',
				'Le groupe { $number } « { $name } » existe déjà : les comptes y seront rattachés.',
				{ number: p.number, name: p.name },
			);
		case 'alreadyInGroup':
			return i18nMsg('accounts-groups-msg-already-in-group', 'Déjà dans ce groupe : { $numbers }.', { numbers: p.numbers });
		case 'mixedCurrencies':
			return i18nMsg(
				'accounts-groups-msg-mixed-currencies',
				"Devises mêlées ({ $currencies }) : le groupe s'affichera en CHF.",
				{ currencies: p.currencies },
			);
		case 'uncheckedSamePrefix':
			return i18nMsg(
				'accounts-groups-msg-unchecked-same-prefix',
				'Non cochés mais de même préfixe : { $numbers }.',
				{ numbers: p.numbers },
			);
	}
}
