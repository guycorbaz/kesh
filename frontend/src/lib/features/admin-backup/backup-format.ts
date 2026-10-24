// Story 31-1c (#503, AC33) — formats d'affichage des sauvegardes : date locale
// et taille lisible, dans la locale de l'interface (jamais celle du
// navigateur, cf. `i18nLocale`).

/** Date et heure locales d'un horodatage ISO ; `''` pour une valeur absente ou illisible. */
export function formatBackupDate(iso: string | null, locale: string): string {
	if (!iso) return '';
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return '';
	return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(date);
}

/** Taille lisible (unités décimales localisées : o, ko, Mo, Go). */
export function formatBackupSize(bytes: number, locale: string): string {
	const units = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;
	let value = bytes;
	let i = 0;
	while (value >= 1000 && i < units.length - 1) {
		value /= 1000;
		i += 1;
	}
	return new Intl.NumberFormat(locale, {
		style: 'unit',
		unit: units[i],
		unitDisplay: 'short',
		maximumFractionDigits: i === 0 ? 0 : 1,
	}).format(value);
}
