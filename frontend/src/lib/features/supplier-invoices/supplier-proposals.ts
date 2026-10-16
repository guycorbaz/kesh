/**
 * Fournisseurs proposés pour un document à QR — Story 30-2b (#685, AC14, C-30-2-7).
 *
 * Module **pur** (aucun réseau, DOM ni i18n), testé en secondes de Vitest.
 *
 * ⚠️ **Un appariement automatique PROPOSE, il ne crée ni ne présélectionne jamais**
 * (`CLAUDE.md`). Le danger n'est pas le doublon, bruyant, mais le **faux
 * rattachement**, muet : deux sociétés d'un même groupe aux noms presque identiques
 * restent ici *toutes deux* éligibles, et c'est à l'utilisateur de « Retenir ». D'où
 * l'arbitrage de l'orchestrateur : **le rappel avant la précision** — trois candidats
 * au plus, aucun présélectionné.
 *
 * Deux sources, dans cet ordre, sans doublon :
 * 1. **par IBAN** — les fournisseurs dont une facture passée porte l'IBAN du créancier
 *    (`GET /imported-supplier-invoices/{id}/supplier-candidates`, calculé au serveur) ;
 * 2. **par nom** — une recherche serveur large ({@link searchTermsFor}), puis ici un
 *    **filtre** d'éligibilité sur la clé normalisée ({@link supplierNameKey}) et un
 *    classement (égalité, inclusion, puis `rank` du socle 22-2a pour départager).
 *
 * Angles morts écrits (manuel) : « Muller » sans tréma ni `e` ne rejoint pas
 * « Müller » ; les mots de moins de trois caractères ne sont pas cherchés ; la fenêtre
 * de 20 du socle n'est pas triée par pertinence (#315). **L'absence de proposition ne
 * prouve pas l'absence du fournisseur, et sa présence ne prouve pas que c'est le bon.**
 */

import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import { MIN_TOKEN_LENGTH, fold, normalizeTerm, rank } from '$lib/features/contacts/duplicate-probe';

/** Nombre maximal de propositions affichées (le serveur en rend autant par IBAN). */
export const MAX_PROPOSALS = 3;

/**
 * Formes juridiques retirées de la clé, **de la plus longue à la plus courte** — sans
 * cet ordre, « S.à r.l. » (`s a r l`) perdrait d'abord `s a` et garderait `r`, `l`.
 */
const LEGAL_FORMS: readonly (readonly string[])[] = [
	['s', 'a', 'g', 'l'],
	['s', 'a', 'r', 'l'],
	['s', 'r', 'l'],
	['s', 'a'],
	['gmbh'],
	['sagl'],
	['sarl'],
	['srl'],
	['sas'],
	['ltd'],
	['inc'],
	['cie'],
	['ag'],
	['sa'],
	['kg'],
	['co'],
	['ab'],
];

/** Mots de liaison retirés après les formes juridiques. */
const LINK_WORDS = new Set(['et', 'und', 'e', 'and']);

/** Retire chaque occurrence de la suite `form`, en un passage de gauche à droite. */
function removeSequence(words: string[], form: readonly string[]): string[] {
	const out: string[] = [];
	let i = 0;
	while (i < words.length) {
		if (form.every((w, k) => words[i + k] === w)) {
			i += form.length;
		} else {
			out.push(words[i]);
			i += 1;
		}
	}
	return out;
}

/** Trémas allemands développés : `ä`→`ae`, `ö`→`oe`, `ü`→`ue`, `ß`→`ss`. */
function expandUmlauts(s: string): string {
	return s.replace(/ä/g, 'ae').replace(/ö/g, 'oe').replace(/ü/g, 'ue').replace(/ß/g, 'ss');
}

/**
 * La **clé** d'un nom de fournisseur : l'ensemble de ses mots significatifs, trié.
 *
 * Dans cet ordre : (a) NFC, minuscules ; (b) trémas développés ; (c) autres accents
 * repliés (`fold` du socle 22-2a) ; (d) tout ce qui n'est ni lettre ni chiffre devient
 * une espace (« S.à r.l. » → `s a r l`) ; (e) découpage en mots ; (f) retrait des
 * formes juridiques, de la plus longue à la plus courte, puis des mots de liaison ;
 * (g) les mots restants, sans doublon, triés.
 */
export function supplierNameKey(name: string | null | undefined): string[] {
	const lowered = (name ?? '').normalize('NFC').toLowerCase();
	const folded = fold(expandUmlauts(lowered)).replace(/[^\p{L}\p{N}]+/gu, ' ');
	let words = folded.split(' ').filter(Boolean);
	for (const form of LEGAL_FORMS) words = removeSequence(words, form);
	words = words.filter((w) => !LINK_WORDS.has(w));
	return [...new Set(words)].sort();
}

/**
 * Les termes envoyés à la recherche serveur (`listContacts({ search })`, un « OU » de
 * mots en collation qui confond `ü` et `u`) : les mots de la clé et, pour chaque mot
 * qui contient `ae`/`oe`/`ue`, sa variante à tréma et sa variante sans `e`
 * (`mueller` → `mueller müller muller`). Les mots de moins de
 * {@link MIN_TOKEN_LENGTH} caractères ne sont pas cherchés. `''` = rien à chercher.
 */
export function searchTermsFor(name: string | null | undefined): string {
	const terms: string[] = [];
	for (const word of supplierNameKey(name)) {
		if (word.length < MIN_TOKEN_LENGTH) continue;
		terms.push(word);
		if (/ae|oe|ue/.test(word)) {
			terms.push(word.replace(/ae/g, 'ä').replace(/oe/g, 'ö').replace(/ue/g, 'ü'));
			terms.push(word.replace(/ae/g, 'a').replace(/oe/g, 'o').replace(/ue/g, 'u'));
		}
	}
	return [...new Set(terms)].join(' ');
}

/** Une proposition affichée, avec son motif. */
export interface SupplierProposal {
	contact: ContactResponse;
	/** `iban` : même IBAN qu'une facture passée ; `name` : nom proche. */
	reason: 'iban' | 'name';
}

/** 0 = clés égales, 1 = l'une incluse dans l'autre (la plus petite non vide), `null` = non éligible. */
function nameTier(candidate: string[], creditor: string[]): 0 | 1 | null {
	if (candidate.length === 0 || creditor.length === 0) return null;
	const a = new Set(candidate);
	const b = new Set(creditor);
	const aInB = candidate.every((w) => b.has(w));
	const bInA = creditor.every((w) => a.has(w));
	if (aInB && bInA) return 0;
	if (aInB || bInA) return 1;
	return null;
}

/**
 * Compose les propositions : par IBAN d'abord (ordre du serveur), puis par nom —
 * **filtrés** par l'éligibilité de la clé (égalité, ou inclusion d'ensembles de mots,
 * la plus petite non vide), classés égalité puis inclusion, `rank` départageant —,
 * sans doublon, {@link MAX_PROPOSALS} au plus. Les contacts archivés ou non
 * fournisseurs sont écartés, quelle que soit leur source.
 *
 * ⚠️ Le résultat est une **liste à afficher**, jamais une sélection : l'écran ne
 * retient rien tant que l'utilisateur n'a pas cliqué « Retenir ».
 */
export function proposeSuppliers(
	byIban: ContactResponse[],
	byName: ContactResponse[],
	creditorName: string | null | undefined,
): SupplierProposal[] {
	const usable = (c: ContactResponse) => c.active && c.isSupplier;
	const out: SupplierProposal[] = [];
	const seen = new Set<number>();
	for (const c of byIban) {
		if (!usable(c) || seen.has(c.id)) continue;
		seen.add(c.id);
		out.push({ contact: c, reason: 'iban' });
	}

	const creditorKey = supplierNameKey(creditorName);
	const eligible: { contact: ContactResponse; tier: 0 | 1 }[] = [];
	for (const c of byName) {
		if (!usable(c) || seen.has(c.id)) continue;
		const tier = nameTier(supplierNameKey(c.name), creditorKey);
		if (tier === null) continue;
		seen.add(c.id);
		eligible.push({ contact: c, tier });
	}
	const order = new Map(
		rank(
			eligible.map((e) => e.contact),
			normalizeTerm(creditorName ?? ''),
		).map((c, i) => [c.id, i]),
	);
	eligible.sort(
		(x, y) => x.tier - y.tier || (order.get(x.contact.id) ?? 0) - (order.get(y.contact.id) ?? 0),
	);
	for (const e of eligible) out.push({ contact: e.contact, reason: 'name' });

	return out.slice(0, MAX_PROPOSALS);
}
