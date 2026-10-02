/**
 * Les natures d'un solde — Story 25-4-d2b (API : 25-4-d2a, #384, #490).
 *
 * Un seul module pour les intitulés, partagé par le dialogue et la liste des
 * règlements : une nature ne s'écrit qu'une fois.
 */
import Big from 'big.js';
import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
import { hasSubCentime } from './invoice-helpers';
import type { InvoiceSettingsResponse, WriteOffNature } from './invoices.types';

/** Les quatre natures, dans l'ordre d'affichage. */
export const WRITE_OFF_NATURES: readonly WriteOffNature[] = [
	'discount',
	'bank_fees',
	'bad_debt',
	'rounding',
];

/** Un reste d'arrondi reste sous l'unité de 5 centimes — même borne que le serveur. */
export const ROUNDING_WRITE_OFF_LIMIT = new Big('0.05');

/** L'intitulé d'une nature. */
export function writeOffNatureLabel(nature: WriteOffNature): string {
	switch (nature) {
		case 'discount':
			return i18nMsg('invoices-write-off-nature-discount', 'Escompte accordé');
		case 'bank_fees':
			return i18nMsg('invoices-write-off-nature-bank-fees', 'Frais bancaires');
		case 'bad_debt':
			return i18nMsg('invoices-write-off-nature-bad-debt', 'Perte sur débiteur');
		case 'rounding':
			return i18nMsg('invoices-write-off-nature-rounding', "Reste d'arrondi");
	}
}

/** La phrase d'aide d'une nature : corrige-t-elle la TVA ? */
export function writeOffNatureHelp(nature: WriteOffNature): string {
	return nature === 'discount' || nature === 'bad_debt'
		? i18nMsg(
				'invoices-write-off-help-vat',
				'La TVA due est corrigée au prorata des taux de la facture.',
			)
		: i18nMsg('invoices-write-off-help-no-vat', 'Sans correction de TVA.');
}

/** La nature est-elle proposable pour ce reste exact ? (Arrondi : sous 0.05.) */
export function isNatureOffered(nature: WriteOffNature, amountDue: string): boolean {
	return nature !== 'rounding' || new Big(amountDue).lt(ROUNDING_WRITE_OFF_LIMIT);
}

/** Le réglage qui désigne le compte d'une nature. */
function natureAccountId(
	settings: InvoiceSettingsResponse,
	nature: WriteOffNature,
): number | null {
	switch (nature) {
		case 'discount':
			return settings.defaultDiscountAccountId;
		case 'bank_fees':
			return settings.defaultBankFeesAccountId;
		case 'bad_debt':
			return settings.defaultBadDebtAccountId;
		case 'rounding':
			return settings.defaultRoundingAccountId;
	}
}

/**
 * Ce qui manque pour solder ce reste avec cette nature, d'après les réglages :
 * `'nature'` (le compte de la nature), `'rounding'` (le compte d'arrondi, exigé
 * pour TOUTE nature dès que le reste porte une fraction de centime — le serveur
 * y impute la fraction), ou `null`.
 *
 * ⚠️ Réglages inconnus (`null`, échec de chargement toléré) → `null` : aucun
 * pré-contrôle, le serveur tranche. Un compte désigné mais archivé ne se voit
 * pas d'ici (limite assumée) : le serveur le refuse au clic.
 */
export function missingAccount(
	settings: InvoiceSettingsResponse | null,
	nature: WriteOffNature,
	amountDue: string,
): 'nature' | 'rounding' | null {
	if (settings === null) return null;
	if (natureAccountId(settings, nature) === null) return 'nature';
	if (
		nature !== 'rounding' &&
		hasSubCentime(amountDue) &&
		settings.defaultRoundingAccountId === null
	) {
		return 'rounding';
	}
	return null;
}
