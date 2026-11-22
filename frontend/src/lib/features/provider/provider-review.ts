/**
 * Story 40-7d (AC-2, AC-3) — règles **communes** des pages des prestataires,
 * hors composant pour être testées seules : exercice par défaut, ordre des
 * propositions à accepter, date d'un écart. Reprises de la page PayPal
 * (38-11d), dont `paypal-review.ts` réexporte les noms.
 */
import type { FiscalYearResponse } from '$lib/features/fiscal-years/fiscal-years.types';
import { formatSwissDate } from '$lib/features/reports/reports.api';
import type { ProviderFinding, ProviderProposalBase } from './provider-review.types';

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

/** La date d'un écart, en forme suisse : celle de sa transaction, sinon de sa première écriture. */
export function findingDate(f: ProviderFinding): string {
	const d = f.transaction?.date ?? f.entries?.[0]?.entryDate;
	return d ? formatSwissDate(d) : '';
}
