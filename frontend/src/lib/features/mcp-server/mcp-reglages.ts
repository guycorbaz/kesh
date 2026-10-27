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
 * Les huit groupes de 16 bits d'une adresse IPv6 écrite (`::` et queue IPv4 pointée
 * admis), ou `null` si le texte n'en est pas une. Sert à reconnaître une adresse mappée
 * (`::ffff:0:0/96`) sous toutes ses graphies — rien de plus : la validation reste au serveur.
 */
function groupesIpv6(adresse: string): number[] | null {
	let texte = adresse.trim().toLowerCase();
	const queue = /(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/.exec(texte);
	if (queue) {
		const o = queue.slice(1).map(Number);
		if (o.some((x) => x > 255)) return null;
		texte = `${texte.slice(0, queue.index)}${((o[0] << 8) | o[1]).toString(16)}:${((o[2] << 8) | o[3]).toString(16)}`;
	}
	const moities = texte.split('::');
	if (moities.length > 2) return null;
	const lire = (s: string) => (s === '' ? [] : s.split(':'));
	const tete = lire(moities[0]);
	const fin = moities.length === 2 ? lire(moities[1]) : [];
	const manquants = 8 - tete.length - fin.length;
	if (moities.length === 1 ? manquants !== 0 : manquants < 1) return null;
	const tous = [...tete, ...Array(moities.length === 2 ? manquants : 0).fill('0'), ...fin];
	if (tous.some((g) => !/^[0-9a-f]{1,4}$/.test(g))) return null;
	return tous.map((g) => parseInt(g, 16));
}

/**
 * La famille et le préfixe d'une entrée **tels que le serveur les lit** : un réseau IPv6
 * inclus dans `::ffff:0:0/96`, de préfixe au moins 96, devient le réseau IPv4 de préfixe
 * `p − 96` — la même normalisation que `reseau_canonique` (`crates/kesh-api/src/mcp/reseau.rs`).
 * Une adresse sans préfixe vaut `/32` ou `/128`. `prefixe` est `null` si l'entrée ne se
 * lit pas (le serveur la refusera ou l'ignorera).
 */
export function familleEtPrefixe(entree: string): { famille: 'IPv4' | 'IPv6'; prefixe: number | null } {
	const e = entree.trim();
	const i = e.lastIndexOf('/');
	const adresse = i < 0 ? e : e.slice(0, i);
	const texteDuPrefixe = i < 0 ? null : e.slice(i + 1).trim();
	const ecrit = texteDuPrefixe === null ? null : /^\d+$/.test(texteDuPrefixe) ? Number(texteDuPrefixe) : NaN;
	if (!adresse.includes(':')) {
		return { famille: 'IPv4', prefixe: ecrit === null ? 32 : Number.isNaN(ecrit) ? null : ecrit };
	}
	const prefixe = ecrit === null ? 128 : Number.isNaN(ecrit) ? null : ecrit;
	const g = groupesIpv6(adresse);
	const mappee = g !== null && g.slice(0, 5).every((x) => x === 0) && g[5] === 0xffff;
	if (mappee && prefixe !== null && prefixe >= 96 && prefixe <= 128) {
		return { famille: 'IPv4', prefixe: prefixe - 96 };
	}
	return { famille: 'IPv6', prefixe };
}

/**
 * Vrai si l'entrée admet toute une famille d'adresses, telle que le serveur la lit :
 * préfixe nul, y compris un réseau mappé de préfixe 96 (`::ffff:0:0/96` = `0.0.0.0/0`).
 */
export function aPrefixeNul(entree: string): boolean {
	return familleEtPrefixe(entree).prefixe === 0;
}

/**
 * La famille d'une entrée, telle que le serveur la lit : une adresse ou un réseau mappé
 * de préfixe au moins 96 compte comme IPv4 (revue de code P1, E2 ; C-32-7-34).
 */
export function familleDe(entree: string): 'IPv4' | 'IPv6' {
	return familleEtPrefixe(entree).famille;
}
