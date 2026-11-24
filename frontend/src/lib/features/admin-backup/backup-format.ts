// Story 31-1c (#503, AC33) — formats d'affichage des sauvegardes : date et
// taille lisible, dans la locale de l'interface (jamais celle du navigateur,
// cf. `i18nLocale`). Story 50-6 (#1511) : la date est à l'heure de Zurich.

import { formatInstant } from '$lib/features/reports/reports.api';

/** Date et heure d'un horodatage ISO, **à l'heure de Zurich** ; `''` pour une valeur absente ou illisible. */
export function formatBackupDate(iso: string | null, locale: string): string {
	return formatInstant(iso, 'dateHeure', locale) ?? '';
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
