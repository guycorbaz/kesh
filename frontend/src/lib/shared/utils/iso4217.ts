/**
 * Story 37-1 (AC-4, AC-7) — les codes ISO 4217 actifs que l'écran propose pour
 * la devise d'un compte.
 *
 * Miroir de la liste embarquée par le serveur (`kesh-core`, module `currency`) :
 * **monnaies seulement** — les codes réservés (`XXX`, `XTS`), les métaux
 * (`XAU`, `XAG`, `XPT`, `XPD`), les droits de tirage (`XDR`), les unités de
 * compte (`XBA`–`XBD`, `XSU`, `XUA`) et les unités de fonds (`BOV`, `CHE`, `CHW`,
 * `CLF`, `COU`, `MXV`, `USN`, `UYI`, `UYW`) en sont exclus. `CHF` y figure : le
 * serveur le normalise en « devise principale » (vide).
 *
 * ⚠️ Confort de saisie : la liste qui fait autorité est celle du serveur, qui
 * refuse un code inconnu (400 `CURRENCY_UNKNOWN`). Les deux listes doivent rester
 * identiques (ordre alphabétique, aucun doublon ; `iso4217.test.ts`).
 */
export const ISO_4217_CURRENCIES: readonly string[] = [
	'AED', 'AFN', 'ALL', 'AMD', 'AOA', 'ARS', 'AUD', 'AWG', 'AZN',
	'BAM', 'BBD', 'BDT', 'BHD', 'BIF', 'BMD', 'BND', 'BOB', 'BRL', 'BSD', 'BTN', 'BWP', 'BYN', 'BZD',
	'CAD', 'CDF', 'CHF', 'CLP', 'CNY', 'COP', 'CRC', 'CUP', 'CVE', 'CZK',
	'DJF', 'DKK', 'DOP', 'DZD',
	'EGP', 'ERN', 'ETB', 'EUR',
	'FJD', 'FKP',
	'GBP', 'GEL', 'GHS', 'GIP', 'GMD', 'GNF', 'GTQ', 'GYD',
	'HKD', 'HNL', 'HTG', 'HUF',
	'IDR', 'ILS', 'INR', 'IQD', 'IRR', 'ISK',
	'JMD', 'JOD', 'JPY',
	'KES', 'KGS', 'KHR', 'KMF', 'KPW', 'KRW', 'KWD', 'KYD', 'KZT',
	'LAK', 'LBP', 'LKR', 'LRD', 'LSL', 'LYD',
	'MAD', 'MDL', 'MGA', 'MKD', 'MMK', 'MNT', 'MOP', 'MRU', 'MUR', 'MVR', 'MWK', 'MXN', 'MYR', 'MZN',
	'NAD', 'NGN', 'NIO', 'NOK', 'NPR', 'NZD',
	'OMR',
	'PAB', 'PEN', 'PGK', 'PHP', 'PKR', 'PLN', 'PYG',
	'QAR',
	'RON', 'RSD', 'RUB', 'RWF',
	'SAR', 'SBD', 'SCR', 'SDG', 'SEK', 'SGD', 'SHP', 'SLE', 'SOS', 'SRD', 'SSP', 'STN', 'SVC', 'SYP', 'SZL',
	'THB', 'TJS', 'TMT', 'TND', 'TOP', 'TRY', 'TTD', 'TWD', 'TZS',
	'UAH', 'UGX', 'USD', 'UYU', 'UZS',
	'VED', 'VES', 'VND', 'VUV',
	'WST',
	'XAF', 'XCD', 'XCG', 'XOF', 'XPF',
	'YER',
	'ZAR', 'ZMW', 'ZWG',
] as const;

/**
 * Les devises proposées pour un compte : la liste ISO **sans** le CHF, qui est
 * la devise principale et se choisit en laissant le champ vide.
 */
export const FOREIGN_CURRENCIES: readonly string[] = ISO_4217_CURRENCIES.filter((c) => c !== 'CHF');

/** Vrai si `code` (déjà normalisé en majuscules) est une monnaie ISO 4217 active. */
export function isIso4217Currency(code: string): boolean {
	return ISO_4217_CURRENCIES.includes(code);
}
