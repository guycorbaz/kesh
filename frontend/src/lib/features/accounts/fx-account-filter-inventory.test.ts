/**
 * Story 37-3b (AC-4, NFR-37-3b-3) — garde d'inventaire des sélecteurs de comptes.
 *
 * Tout fichier `.svelte` ou `.ts` de `src/` (hors tests) qui **importe**
 * `fetchAccounts`, `AccountAutocomplete` ou `linkableLedgerAccounts`, ou qui
 * **déclare une prop** de type `AccountResponse[]` (`readonly` admis), porte :
 *
 * - une **marque** de filtre des comptes en devise : `withoutForeignCurrency`,
 *   `isForeignCurrencyAccount`, `excludeForeignCurrency`, les filtres centraux
 *   `linkableLedgerAccounts` et `counterpartyAccounts`, ou un
 *   `<AccountAutocomplete` (qui filtre par défaut) ;
 * - ou une **exemption justifiée** : un commentaire `fx-filtre-comptes : exempté —`
 *   suivi de sa raison.
 *
 * Un fichier qui **désactive** le filtre (`excludeForeignCurrency={false}` ou
 * `={!…}`) doit porter l'exemption. Le test lit le source : il ne se fie pas à la
 * liste du Change Log.
 */
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { describe, expect, it } from 'vitest';

const ROOT = join(process.cwd(), 'src');

const IMPORT_RE = /import\s[^;]*\b(fetchAccounts|AccountAutocomplete|linkableLedgerAccounts)\b[^;]*from\s/;
const PROP_RE = /^\s*\w+\??\s*:\s*(readonly\s+)?AccountResponse\[\]\s*;/m;
const MARKS = [
	'withoutForeignCurrency',
	'isForeignCurrencyAccount',
	'excludeForeignCurrency',
	'linkableLedgerAccounts',
	'counterpartyAccounts',
	'<AccountAutocomplete',
];
const DISABLED_RE = /excludeForeignCurrency=\{\s*(false|!)/;
const EXEMPT_RE = /fx-filtre-comptes : exempté — \S/;

/** Fichier de test (exclu de la garde). */
export function isTestFile(path: string): boolean {
	return /\.test\.(ts|svelte)$|\.spec\.ts$|test-helper|\.test-fixtures\.ts$/.test(path);
}

/**
 * Le fichier relève-t-il de la garde ? Un `.svelte` ou un `.ts` qui importe l'un des
 * trois symboles ; un `.svelte` qui déclare une prop `AccountResponse[]`. Un `.ts` qui
 * ne fait que **prendre** une liste de comptes (paramètre, champ d'un type d'entrée)
 * ne déclare pas de prop : hors garde, sauf import.
 */
export function isCandidate(path: string, source: string): boolean {
	if (isTestFile(path)) return false;
	if (!/\.(svelte|ts)$/.test(path)) return false;
	return IMPORT_RE.test(source) || (path.endsWith('.svelte') && PROP_RE.test(source));
}

/** Le verdict de la garde pour un fichier candidat : `null` s'il est en règle. */
export function verdict(source: string): 'unmarked' | 'disabled-without-exemption' | null {
	const exempt = EXEMPT_RE.test(source);
	if (DISABLED_RE.test(source) && !exempt) return 'disabled-without-exemption';
	if (exempt) return null;
	return MARKS.some((m) => source.includes(m)) ? null : 'unmarked';
}

function walk(dir: string, out: string[]): string[] {
	for (const name of readdirSync(dir)) {
		const p = join(dir, name);
		if (statSync(p).isDirectory()) walk(p, out);
		else out.push(p);
	}
	return out;
}

describe('garde des sélecteurs de comptes en devise (AC-4)', () => {
	const files = walk(ROOT, []);
	const candidates = files
		.map((p) => ({ path: relative(process.cwd(), p), source: readFileSync(p, 'utf8') }))
		.filter((f) => isCandidate(f.path, f.source));

	it('trouve les sélecteurs connus (la garde ne tourne pas à vide)', () => {
		const paths = candidates.map((c) => c.path);
		for (const attendu of [
			'src/lib/components/invoices/InvoiceForm.svelte',
			'src/lib/features/invoices/SettleInvoiceDialog.svelte',
			'src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte',
			'src/lib/features/reconciliation/rules/RuleFormModal.svelte',
			'src/lib/features/settings/CurrencySettingsSection.svelte',
			'src/routes/(app)/settings/opening-balances/+page.svelte',
			'src/routes/onboarding/+page.svelte',
			'src/lib/features/journal-entries/JournalEntryForm.svelte',
		]) {
			expect(paths).toContain(attendu);
		}
		expect(candidates.length).toBeGreaterThanOrEqual(30);
	});

	it('chaque sélecteur filtre les comptes en devise, ou porte une exemption justifiée', () => {
		const fautifs = candidates
			.map((c) => ({ path: c.path, v: verdict(c.source) }))
			.filter((c) => c.v !== null);
		expect(fautifs).toEqual([]);
	});

	it("règles de détection : prop `readonly`, fichiers de test, aides `.ts` sans import", () => {
		expect(isCandidate('src/x/A.svelte', 'let { accounts }: Props;\n\taccounts: readonly AccountResponse[];')).toBe(
			true,
		);
		expect(isCandidate('src/x/Host.test.svelte', "import AccountAutocomplete from './A.svelte';")).toBe(false);
		expect(isCandidate('src/x/a.test.ts', "import { fetchAccounts } from './api';")).toBe(false);
		// Une aide `.ts` qui PREND un `AccountResponse[]` en paramètre ne déclare pas de prop.
		expect(isCandidate('src/x/helper.ts', 'export function f(accounts: AccountResponse[]): void {}')).toBe(false);
		// … ni en champ d'un type d'entrée (`conversion.ts`, `ConversionContext`).
		expect(isCandidate('src/x/ctx.ts', 'interface C {\n\taccounts: AccountResponse[];\n}')).toBe(false);
		for (const aide of [
			'src/lib/features/accounts/account-label.ts',
			'src/lib/features/accounts/account-options.ts',
			'src/lib/features/accounts/account-validity.ts',
			'src/lib/features/accounts/accounts.api.ts',
			'src/lib/features/opening-balances/opening-balances-totals.ts',
			'src/lib/features/bank-accounts/format.ts',
			'src/lib/features/reconciliation/reconciliation-counterparties.ts',
		]) {
			const source = readFileSync(join(process.cwd(), aide), 'utf8');
			expect(isCandidate(aide, source), aide).toBe(false);
		}
	});

	it('verdicts : marque, exemption, désactivation sans exemption', () => {
		expect(verdict('<AccountAutocomplete {accounts} />')).toBeNull();
		expect(verdict('const x = withoutForeignCurrency(accounts);')).toBeNull();
		expect(verdict('fetchAccounts()')).toBe('unmarked');
		expect(verdict('<AccountAutocomplete excludeForeignCurrency={false} />')).toBe(
			'disabled-without-exemption',
		);
		expect(
			verdict('// fx-filtre-comptes : exempté — saisie en devise\n<AccountAutocomplete excludeForeignCurrency={false} />'),
		).toBeNull();
	});
});
