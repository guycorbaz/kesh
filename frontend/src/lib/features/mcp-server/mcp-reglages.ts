// Story 32-7 (refs #723) — fonctions pures des réglages du serveur MCP (AC-5, AC-6).
//
// L'écran ne valide pas les adresses : le serveur le fait et traduit ses refus. Il
// retire seulement les lignes vides (le serveur refuse une entrée vide) et calcule des
// avertissements INFORMATIFS sur le texte affiché.

/** Les entrées d'une zone de texte : une par ligne, rognées, lignes vides retirées. */
export function entreesDe(texte: string): string[] {
	return texte
		.split('\n')
		.map((l) => l.trim())
		.filter((l) => l !== '');
}

/** Le texte d'une zone, une entrée par ligne. */
export function texteDe(entrees: readonly string[]): string {
	return entrees.join('\n');
}

/**
 * Vrai si l'entrée (rognée) porte un préfixe nul : ce qui suit son dernier `/` vaut 0.
 * Une adresse sans préfixe n'est jamais `/0`.
 */
export function aPrefixeNul(entree: string): boolean {
	const e = entree.trim();
	const i = e.lastIndexOf('/');
	if (i < 0) return false;
	const prefixe = e.slice(i + 1).trim();
	return /^0+$/.test(prefixe);
}

/**
 * La famille d'une entrée, lue sur le texte affiché : IPv6 si elle contient `:`,
 * IPv4 sinon. Une adresse IPv4 mappée (`::ffff:a.b.c.d`) est donc classée IPv6 ici,
 * alors que le serveur la lit sous sa forme IPv4 (angle mort écrit, AC-6 (b)).
 */
export function familleDe(entree: string): 'IPv4' | 'IPv6' {
	return entree.includes(':') ? 'IPv6' : 'IPv4';
}
