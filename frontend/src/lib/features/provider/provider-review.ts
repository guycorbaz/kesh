/**
 * Story 40-7d (AC-2, AC-3) — règles **communes** des pages des prestataires,
 * hors composant pour être testées seules : exercice par défaut, ordre des
 * propositions à accepter, date d'un écart. Reprises de la page PayPal
 * (38-11d), dont `paypal-review.ts` réexporte les noms.
 */
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { formatSwissDate } from '$lib/features/reports/reports.api';
import type { ProviderFinding, ProviderProposalBase } from './provider-review.types';
import { isApiError } from '$lib/shared/utils/api-error';

/**
 * Story 40-9 (revue P1 de la 40-7d, L7) — les refus de l'import qui se règlent à
 * l'écran *Comptes bancaires* (refus du cœur d'import commun : aucun compte du
 * prestataire, aucun compte pour une devise du fichier, deux comptes pour une
 * devise, compte clos). Liste blanche : un refus de format, de doublon, une
 * erreur réseau ou inconnue ne propose pas le lien.
 */
export const IMPORT_ERRORS_CONCERNING_ACCOUNTS: readonly string[] = [
	'PROVIDER_ACCOUNT_NOT_FOUND',
	'PROVIDER_CURRENCY_ACCOUNT_MISSING',
	'PROVIDER_CURRENCY_DUPLICATE',
	'BANK_ACCOUNT_CLOSED',
];

/** Story 40-9 — le code d'un refus de l'API, `null` pour une autre erreur. */
export function importErrorCode(err: unknown): string | null {
	return isApiError(err) ? err.code : null;
}

/** Story 40-9 — le refus d'import de code `code` appelle-t-il le lien « Comptes bancaires » ? */
export function importErrorConcernsAccounts(code: string | null): boolean {
	return code !== null && IMPORT_ERRORS_CONCERNING_ACCOUNTS.includes(code);
}

/**
 * L'exercice proposé par défaut : celui qui contient `today` (`AAAA-MM-JJ`),
 * sinon le plus récent ; `null` sans exercice.
 */
export function defaultFiscalYearId(years: readonly FiscalYearResponse[], today: string): number | null {
	const current = years.find((y) => y.startDate <= today && today <= y.endDate);
	if (current) return current.id;
	const sorted = [...years].sort((a, b) => b.startDate.localeCompare(a.startDate));
	return sorted[0]?.id ?? null;
}

/**
 * Les propositions cochées (`selected`, par clé `keyOf`), **dans l'ordre des
 * dates** puis de la transaction (`idOf`) : l'ordre d'envoi de l'acceptation.
 */
export function selectedInDateOrder<P extends ProviderProposalBase>(
	proposals: readonly P[],
	selected: ReadonlySet<string>,
	keyOf: (p: P) => string,
	idOf: (p: P) => number,
): P[] {
	return proposals
		.filter((p) => selected.has(keyOf(p)))
		.sort((a, b) => a.transaction.date.localeCompare(b.transaction.date) || idOf(a) - idOf(b));
}

/**
 * La date d'un écart, en forme suisse : celle de sa transaction, sinon de sa
 * première écriture, sinon `details.date` (Story 39-11d2 : l'écart de
 * continuité Wise, `balance_gap`, n'a ni transaction ni écriture).
 */
export function findingDate(f: ProviderFinding): string {
	const fromDetails = typeof f.details?.date === 'string' ? f.details.date : undefined;
	const d = f.transaction?.date ?? f.entries?.[0]?.entryDate ?? fromDetails;
	return d ? formatSwissDate(d) : '';
}
