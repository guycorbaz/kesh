/**
 * Story 34-11a2 (#972) — règles d'écran de la carte de crédit, tenues hors des
 * composants pour être testées seules. Le serveur reste l'autorité : chaque
 * règle ici a son refus côté serveur (`routes/bank_accounts.rs`).
 */
import type { BankAccountKind } from './bank-accounts.api';

/**
 * Les types de compte du plan auxquels un compte de ce type se lie
 * (C-e341-16) : passif seulement pour une carte, actif ou passif pour un
 * compte bancaire. Miroir de `BankAccountKind::admits_ledger_type` (kesh-db).
 */
export function admitsLedgerType(kind: BankAccountKind, accountType: string): boolean {
	return kind === 'credit_card'
		? accountType === 'Liability'
		: accountType === 'Asset' || accountType === 'Liability';
}

/**
 * Le champ « quatre derniers chiffres » est-il acceptable ? Vide (après
 * `trim`) ou exactement quatre chiffres ASCII.
 */
export function isValidCardLast4(value: string): boolean {
	const v = value.trim();
	return v === '' || /^[0-9]{4}$/.test(v);
}

/** Valeur envoyée pour les quatre chiffres : `null` si le champ est vide. */
export function cardLast4Payload(value: string): string | null {
	const v = value.trim();
	return v === '' ? null : v;
}

/**
 * Les comptes proposés au **règlement d'une facture client** (C-e341-32) :
 * jamais une carte — un client ne paie pas sur la carte du titulaire. Le
 * paiement d'une facture fournisseur, lui, propose la carte.
 */
export function withoutCreditCards<T extends { accountKind: BankAccountKind }>(accounts: T[]): T[] {
	return accounts.filter((a) => a.accountKind !== 'credit_card');
}
