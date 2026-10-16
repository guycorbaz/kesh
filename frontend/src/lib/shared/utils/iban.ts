/**
 * IBAN côté écran — Story 30-2b (#685, AC12, AC12-bis, C-30-2-20).
 *
 * **Une seule définition côté frontend** de deux règles que plusieurs écrans
 * emploient : la normalisation d'une saisie d'IBAN et la classification « QR-IBAN ou
 * non ». La classification vivait dans `routes/(app)/bank-accounts/+page.svelte`
 * (#155) ; elle en est extraite, complétée de la condition de pays CH/LI qu'elle
 * n'avait pas, et partagée avec le formulaire de facture fournisseur.
 *
 * Le serveur reste **autoritatif** (`kesh_core::types::{Iban, QrIban}`, somme de
 * contrôle comprise) : ces fonctions servent à router une saisie vers le bon champ et
 * à afficher un message actionnable avant l'aller-retour, jamais à valider.
 */

/**
 * Les blancs retirés d'une saisie d'IBAN : l'ensemble **écrit** de l'AC12-bis —
 * `White_Space` d'Unicode, plus U+200B (espace de largeur nulle) et U+FEFF (BOM).
 *
 * ⚠️ `\s` de JavaScript couvre déjà U+FEFF, U+00A0, U+202F… mais **pas** U+0085, que
 * `char::is_whitespace` retire côté serveur (`normalize_payment_field`,
 * `crates/kesh-api/src/routes/supplier_invoices.rs`) ; U+200B n'est dans aucun des
 * deux. D'où cette liste explicite, et un test de chaque côté qui porte les mêmes
 * sept caractères.
 */
// Points de code explicites : un caractère invisible littéral ne se relit pas et un
// formateur peut le retirer (revue P1 de la 30-2b, A6 = E7). U+0085, puis U+200B.
const IBAN_BLANKS = new RegExp('[\\s' + String.fromCodePoint(0x85, 0x200b) + ']', 'g');

/** Retire les blancs de l'ensemble écrit et passe en majuscules (`''` si rien ne reste). */
export function normalizeIban(raw: string): string {
	return (raw ?? '').replace(IBAN_BLANKS, '').toUpperCase();
}

/**
 * Vrai si la saisie a la forme d'un QR-IBAN : IBAN suisse ou liechtensteinois dont le
 * QR-IID (positions 5 à 9) est dans la plage 30000-31999 (#155, spécification SIX).
 *
 * Ne contrôle ni la longueur ni la somme de contrôle : un QR-IBAN mal saisi est routé
 * vers `creditorQrIban` et le serveur le refuse avec un motif précis
 * (`INVALID_PAYMENT_DETAILS`, `INVALID_IBAN`).
 */
export function qrIbanHasValidQrIid(value: string): boolean {
	const normalized = normalizeIban(value);
	if (!normalized.startsWith('CH') && !normalized.startsWith('LI')) return false;
	if (normalized.length < 9) return false;
	const digits = normalized.slice(4, 9);
	if (!/^\d{5}$/.test(digits)) return false;
	const iid = Number(digits);
	return iid >= 30000 && iid <= 31999;
}
