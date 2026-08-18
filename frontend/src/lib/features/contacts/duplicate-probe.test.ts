/**
 * Preuves du socle d'appariement — Story 22-2a (#301).
 *
 * ⚠️ **Chaque test nomme la MUTATION qu'il attrape**, et cette mutation a été
 * JOUÉE : `scripts/mutants-22-2a.mjs` applique chacune au module et vérifie
 * qu'elle fait tomber la preuve annoncée — et elle seule. Trois passes de revue
 * en prose avaient laissé passer des preuves qui ne discriminaient rien : une
 * preuve dont la mutation n'a pas été jouée est une preuve dont on ignore le
 * pouvoir.
 */

import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import {
	buildTerm,
	countOthers,
	excludeSelf,
	describeProches,
	findIdeHolder,
	fold,
	isArmed,
	normalizeTerm,
	probeTerm,
	rank
} from './duplicate-probe';
import type { ContactResponse } from './contacts.types';

/** Fabrique minimale — seuls `id`, `name` et `ideNumber` sont lus par le module. */
function c(id: number, name: string, ideNumber: string | null = null): ContactResponse {
	return { id, name, ideNumber } as unknown as ContactResponse;
}

/** Fabrique pour `describeProches` — ville, numéro de client et email. */
function d(
	id: number,
	name: string,
	city = '',
	clientNumber: string | null = null,
	email: string | null = null
): ContactResponse {
	return {
		id,
		name,
		clientNumber,
		email,
		addressStructured: { city }
	} as unknown as ContactResponse;
}

const names = (list: ContactResponse[]) => list.map((x) => x.name);

// ---------------------------------------------------------------------------
// normalizeTerm
// ---------------------------------------------------------------------------

describe('normalizeTerm', () => {
	it.each([
		['Coop-Vaud', 'Coop Vaud'],
		['C++', 'C'],
		['Müller-Weber', 'Müller Weber'],
		['  a--b  ', 'a b'],
		['***', '']
	])('remplace les opérateurs par une espace : %s → %s', (input, expected) => {
		// MUTATION : « supprimer au lieu de remplacer » → rend `CoopVaud`.
		expect(normalizeTerm(input)).toBe(expected);
	});

	it('retire les invisibles de LARGEUR NULLE, sans toucher aux espaces', () => {
		// MUTATION : « ne pas retirer les invisibles de largeur nulle ».
		// Un ZWSP collé depuis un courriel ou Word casse tout appariement — et
		// c'est le combat exact de la Story 22-1 (#294), qui a écrit
		// `is_zero_width` côté Rust pour cette raison.
		expect(normalizeTerm('Coop\u200BVaud')).toBe('CoopVaud');
		expect(normalizeTerm('Coop\u00ADVaud')).toBe('CoopVaud');
		// ⚠️ Une ESPACE marque, elle : `CLI 1` ne doit pas devenir `CLI1`.
		expect(normalizeTerm('CLI 1')).toBe('CLI 1');
	});

	it('stabilise la forme Unicode : NFD et NFC rendent la même chaîne', () => {
		// MUTATION : « omettre normalize('NFC') ». Le test ci-dessus reste vert —
		// aucun de ses cinq cas ne porte d'accent décomposé.
		const nfc = 'Café';
		const nfd = 'Café';
		expect(nfd).not.toBe(nfc); // les deux entrées SONT distinctes en mémoire
		expect(normalizeTerm(nfd)).toBe(normalizeTerm(nfc));
		expect(normalizeTerm(nfd).length).toBe(4);
	});
});

// ---------------------------------------------------------------------------
// buildTerm
// ---------------------------------------------------------------------------

describe('buildTerm', () => {
	it('Entreprise : la raison sociale, rognée', () => {
		expect(buildTerm('Entreprise', '  Dubarde Sàrl  ', '', '')).toBe('Dubarde Sàrl');
	});

	it('ne fabrique JAMAIS le mot « null » à partir d’un champ absent', () => {
		// MUTATION : « interpoler sans garde ». `ContactResponse.firstName` est
		// `string | null` : une valeur venue du serveur produirait le terme
		// littéral « null Dupont », armé, et chercherait « null ».
		const nul = null as unknown as string;
		expect(buildTerm('Personne', '', nul, 'Dupont')).toBe('Dupont');
		expect(buildTerm('Entreprise', nul, '', '')).toBe('');
		expect(probeTerm('Personne', 'x', nul, nul).armed).toBe(false);
	});

	it('Personne : prénom + nom, y compris quand l’un des deux est vide', () => {
		// MUTATION : « ne lire que la raison sociale » → rend le dispositif
		// entièrement mort pour les personnes physiques.
		expect(buildTerm('Personne', '', 'Jean', 'Dupont')).toBe('Jean Dupont');
		expect(buildTerm('Personne', '', 'Jean', '')).toBe('Jean');
		expect(buildTerm('Personne', '', '', 'Dupont')).toBe('Dupont');
		expect(buildTerm('Personne', 'ignorée', '', '')).toBe('');
	});
});

// ---------------------------------------------------------------------------
// isArmed / probeTerm — l'ORDRE
// ---------------------------------------------------------------------------

describe('isArmed', () => {
	it('mesure sans DÉBORDER LA PILE sur un terme à 200 000 tokens', () => {
		// MUTATION : « Math.max(...tokens.map(t => t.length)) ».
		// Le spread déborde la pile au-delà d'environ 150 000 arguments. Les
		// champs du formulaire sont bornés (255 et 70), donc le cas n'est pas
		// atteignable par l'interface — mais cette borne n'est pas une propriété
		// de ce module, et `some()` supprime la classe entière pour rien.
		expect(() => isArmed(normalizeTerm('a '.repeat(200_000)))).not.toThrow();
	});

	it.each([
		['Jean', true],
		['Dubarde Sàrl', true],
		// ⚠️ Un token de longueur EXACTEMENT 3. Sans lui le seuil n'est épinglé que
		// PAR LE BAS — tous les cas négatifs mesurent 2 — et le porter à 4 laissait
		// les deux suites entièrement vertes, en faisant taire la sonde pour `UBS`,
		// `SBB`, `CFF`, `BCV`, `SIG`. Sur un logiciel de comptabilité suisse, ce
		// n'est pas un cas de laboratoire. Relevé en passe 2 de revue de code.
		['UBS', true],
		['Du', false],
		['An Li', false],
		['', false]
	])('mesure le plus long token du terme normalisé : %s → %s', (raw, expected) => {
		// MUTATION : « mesurer la longueur du terme entier » → « An Li » (5
		// caractères, deux tokens de 2) passerait le seuil.
		expect(isArmed(normalizeTerm(raw))).toBe(expected);
	});
});

describe('probeTerm — normaliser PUIS mesurer', () => {
	it.each([
		['Yo-An', false], // 5 caractères bruts, deux tokens de 2 après normalisation
		['C++', false], // 3 caractères bruts, un token de 1 après normalisation
		['Jean', true],
		['Du', false]
	])('l’ordre des opérations décide de l’armement : %s → armé=%s', (raw, armed) => {
		// MUTATION : « mesurer le seuil AVANT de normaliser » — un simple ordre
		// d'instructions. Elle laisse `Jean` et `Du` corrects et ne fait tomber
		// que `Yo-An` et `C++`. C'est pour eux que ce test existe, et c'est parce
		// que `probeTerm` compose les trois étapes que la mutation est jouable
		// DANS le module.
		expect(probeTerm('Entreprise', raw, '', '').armed).toBe(armed);
	});

	it('compose selon le type : une Personne arme sur prénom + nom', () => {
		expect(probeTerm('Personne', '', 'Jean', 'Dupont')).toEqual({
			normalized: 'Jean Dupont',
			armed: true
		});
	});
});

// ---------------------------------------------------------------------------
// fold — la SYMÉTRIE
// ---------------------------------------------------------------------------

describe('fold', () => {
	it('replie casse et accents', () => {
		expect(fold('Müller')).toBe('muller');
		expect(fold('Café du Marché')).toBe('cafe du marche');
	});

	it('replie les formes de COMPATIBILITÉ — pleine chasse et ligatures', () => {
		// MUTATION : « replier en NFD sans passer par NFKC ».
		// `canonical_key` applique NFKC pour la même raison (Story 22-1).
		expect(fold('ＤＵＢＡＲＤＥ')).toBe(fold('DUBARDE'));
		expect(fold('ﬁrma')).toBe(fold('firma'));
	});

	it('replie `İ` (U+0130) sur un seul caractère, quel que soit l’ordre', () => {
		// MUTATION : « retirer le strip des diacritiques combinants ».
		//
		// Ce test FIGE un comportement là où le doc-comment posait un INTERDIT —
		// « ne pas réordonner » — que l'exécution réfute : les deux ordres rendent
		// `i`. Une consigne ne se vérifie pas ; un test, si.
		//
		// ⚠️ La mutation « minusculer AVANT le strip » a été écartée du banc : elle
		// est ÉQUIVALENTE, donc elle survivrait par construction et dévaluerait le
		// verdict. Un banc ne vaut que si chacune de ses mutations peut mordre.
		expect(fold('İstanbul')).toBe('istanbul');
		expect([...fold('İ')]).toHaveLength(1);
	});

	it('applique AUSSI la normalisation du terme — c’est ce qui rend le repli symétrique', () => {
		// MUTATION : « replier le nom sans lui appliquer normalizeTerm ».
		// C'est le défaut fondateur de la story : le terme perd son trait
		// d'union, le nom le garde, et le doublon EXACT sort dernier.
		expect(fold('Jean-Marc Zwahlen')).toBe(fold('Jean Marc Zwahlen'));
		expect(fold('Coop-Vaud')).toBe('coop vaud');
	});
});

// ---------------------------------------------------------------------------
// rank
// ---------------------------------------------------------------------------

describe('rank', () => {
	const jeanX = [
		c(1, 'Jean Bernard'),
		c(2, 'Jean Dupont'),
		c(3, 'Jean Favre'),
		c(4, 'Jean Martin'),
		c(5, 'Jean Rochat'),
		c(6, 'Jean Zwahlen')
	];

	it('le doublon exact sort EN TÊTE et figure dans les cinq premiers', () => {
		// MUTATION : « rank = identité » → rend l'ordre alphabétique du SQL et
		// évince Zwahlen de la fenêtre de cinq. C'est LE défaut que la story
		// existe pour fermer.
		const out = rank(jeanX, normalizeTerm('Jean Zwahlen'));
		expect(out[0].name).toBe('Jean Zwahlen');
		expect(names(out.slice(0, 5))).toContain('Jean Zwahlen');
	});

	it('un doublon dont le nom porte un TRAIT D’UNION sort quand même en tête', () => {
		// MUTATION : « replier le nom sans lui appliquer normalizeTerm ».
		// La fixture des six `Jean X` reste VERTE sous cette mutation — aucun de
		// ses noms ne porte d'opérateur. C'est pour ce cas que ce test existe.
		const carnet = [...jeanX.slice(0, 5), c(6, 'Jean-Marc Zwahlen')];
		const out = rank(carnet, normalizeTerm('Jean-Marc Zwahlen'));
		expect(out[0].name).toBe('Jean-Marc Zwahlen');
	});

	it('critère 1 : commencer par le terme prime le partage de tokens', () => {
		// MUTATION : « intervertir les critères 1 et 2 ».
		// ⚠️ La fixture NAÏVE ne discrimine rien : un nom qui commence par le
		// terme le contient trivialement, donc sature aussi le critère 2. Il faut
		// la frontière de mot — `jeanne` n'est pas le token `jean`.
		const out = rank([c(1, 'Marie Jean'), c(2, 'Jeanne Dupont')], normalizeTerm('Jean'));
		expect(out[0].name).toBe('Jeanne Dupont');
	});

	it('critère 2 : un token RÉPÉTÉ dans le terme ne compte qu’une fois', () => {
		// MUTATION : « retirer le `new Set(...)` des tokens du terme ».
		//
		// ⚠️ Le doc-comment justifie ce `Set` en toutes lettres, et aucune fixture ne
		// répétait de token : la mutation survivait à toute la suite.
		//
		// ⚠️ **La première fixture écrite ici ne discriminait pas non plus** — ses
		// deux candidats portaient le token répété, donc le `Set` ne changeait pas
		// leur ordre RELATIF. Il faut que la répétition avantage l'UN des deux :
		//   terme replié `bertrand ana ana`
		//   `ana solo`     → sans Set : 2 tokens partagés · avec Set : 1
		//   `bertrand zoe` → sans Set : 1               · avec Set : 1
		// Avec le `Set` les deux marquent 1 et c'est le préfixe commun qui tranche —
		// 9 contre 0 — donc `Bertrand Zoe`. Sans lui, `Ana Solo` passe devant sur un
		// score gonflé par une répétition qui n'apporte rien.
		const out = rank([c(1, 'Ana Solo'), c(2, 'Bertrand Zoe')], normalizeTerm('Bertrand Ana Ana'));
		expect(out[0].name).toBe('Bertrand Zoe');
	});

	it('critère 2 : ramène le doublon RÉORDONNÉ, qu’aucun préfixe ne rattrape', () => {
		// MUTATION : « retirer le critère 2 » → `Zwahlen Jean` retombe derrière
		// les cinq `Jean X` et sort de la fenêtre de cinq.
		const carnet = [...jeanX.slice(0, 5), c(50, 'Zwahlen Jean')];
		const out = rank(carnet, normalizeTerm('Jean Zwahlen'));
		expect(out[0].name).toBe('Zwahlen Jean');
	});

	it('critère 3 : le plus long préfixe commun départage, contre l’alphabétique', () => {
		// MUTATION : « retirer le critère 3 » → l'alphabétique tranche et rend
		// `Duault SA` devant.
		//
		// ⚠️ **Cette fixture a été choisie APRÈS avoir joué la mutation**, et la
		// première ne discriminait rien : `Dubarde SA` contre `Dumont Bar` sur le
		// terme `Dubar` est gagné par le critère 3 ET par l'alphabétique, qui
		// donnent le même vainqueur. Il faut que les deux s'OPPOSENT :
		//   fold('Dubart SA') = 'dubart sa'  → préfixe commun avec 'dubarde' : 5
		//   fold('Duault SA') = 'duault sa'  → préfixe commun : 2
		//   alphabétiquement, 'duault sa' < 'dubart sa'  (index 2 : 'a' < 'b')
		// C'est le même piège que celui du critère 1, et il ne se voit qu'en
		// jouant la mutation.
		const out = rank([c(1, 'Duault SA'), c(2, 'Dubart SA')], normalizeTerm('Dubarde'));
		expect(out[0].name).toBe('Dubart SA');
	});

	it('critère 4 : l’alphabétique tranche, et il s’oppose ici à l’ordre des id', () => {
		// MUTATION : « retirer le critère 4 » → l'`id` tranche et rend `Zoé SA`
		// devant. ⚠️ Une preuve qui n'assert QUE la stabilité laisserait cette
		// mutation verte : le critère 5 suffit à rendre le comparateur total.
		const out = rank([c(101, 'Zoé Services'), c(202, 'Alpha Services')], normalizeTerm('Services'));
		expect(names(out)).toEqual(['Alpha Services', 'Zoé Services']);
	});

	it('critère 5 : deux HOMONYMES STRICTS se classent par id, donc de façon déterministe', () => {
		// MUTATION : « retirer le critère 5 » → le tri stable rend l'ordre
		// d'ENTRÉE, et deux appels sur les mêmes données divergent.
		const a = c(101, 'Jean Dupont');
		const b = c(202, 'Jean Dupont');
		const t = normalizeTerm('Jean Dupont');
		expect(rank([a, b], t).map((x) => x.id)).toEqual([101, 202]);
		expect(rank([b, a], t).map((x) => x.id)).toEqual([101, 202]);
	});

	it('classe sans filtrer ni dupliquer', () => {
		// MUTATION : « faire filtrer rank » → casserait l'arithmétique de
		// countOthers, qui soustrait d'un total serveur.
		const out = rank(jeanX, normalizeTerm('Jean Zwahlen'));
		expect(out).toHaveLength(jeanX.length);
		expect([...out.map((x) => x.id)].sort()).toEqual([1, 2, 3, 4, 5, 6]);
	});
});

// ---------------------------------------------------------------------------
// excludeSelf / countOthers
// ---------------------------------------------------------------------------

describe('excludeSelf', () => {
	it('retire le contact édité, et lui seul', () => {
		// MUTATION : « excludeSelf = identité ». Aucune preuve de countOthers ne
		// l'attrape : elles passent leurs listes en dur.
		expect(excludeSelf([c(1, 'A'), c(42, 'B'), c(7, 'C')], 42).map((x) => x.id)).toEqual([1, 7]);
	});

	it('ne retire rien en création', () => {
		expect(excludeSelf([c(1, 'A')], null).map((x) => x.id)).toEqual([1]);
	});
});

describe('countOthers', () => {
	it('cas nominal : total 12, 5 affichés, pas d’édition ⇒ 7', () => {
		expect(countOthers(12, [], [c(1, 'A'), c(2, 'B'), c(3, 'C'), c(4, 'D'), c(5, 'E')], null)).toBe(
			7
		);
	});

	it('édition solitaire : l’unique correspondance est la fiche éditée ⇒ 0, jamais 1', () => {
		// MUTATION : « total − affiches.length » → rend 1, et fait afficher
		// « et 1 autre » AU-DESSUS D'UNE LISTE VIDE, en désignant la fiche qu'on
		// modifie. Le cas nominal reste vert sous cette mutation.
		expect(countOthers(1, [c(9, 'Moi')], [], 9)).toBe(0);
	});

	it('un total non fini ne devient pas « et NaN autres »', () => {
		// MUTATION : « retirer la garde Number.isFinite ».
		expect(countOthers(NaN, [], [], null)).toBe(0);
	});

	it('contact édité HORS FENÊTRE : l’écart d’une unité est assumé et figé', () => {
		// Ce test ne prouve pas une correction : il FIGE un comportement connu
		// pour qu'il ne soit pas « réparé » plus tard au prix d'un paramètre de
		// requête. Cf. la précondition de countOthers.
		const fenetre = Array.from({ length: 20 }, (_, i) => c(i + 1, `X${i}`));
		expect(countOthers(25, fenetre, fenetre.slice(0, 5), 999)).toBe(20);
	});
});

// ---------------------------------------------------------------------------
// findIdeHolder
// ---------------------------------------------------------------------------

describe('findIdeHolder', () => {
	const lot = [
		c(1, 'Sans IDE', null),
		c(7, 'Le porteur', 'CHE109322551'),
		c(8, 'Autre IDE', 'CHE123456788')
	];

	it('trouve le porteur réel', () => {
		expect(findIdeHolder(lot, 'CHE109322551', null)?.id).toBe(7);
	});

	it('un contact remonté SANS porter cet IDE n’est pas retenu', () => {
		expect(findIdeHolder([c(1, 'CHE109322551 dans le nom', null)], 'CHE109322551', null)).toBeUndefined();
	});

	it('exclut le contact édité — le signal FRANC ne crie pas sur sa propre fiche', () => {
		// MUTATION : « supprimer && c.id !== editingId ». Les autres preuves de
		// findIdeHolder passent toutes editingId = null et restent vertes.
		expect(findIdeHolder(lot, 'CHE109322551', 7)).toBeUndefined();
		expect(findIdeHolder([...lot, c(9, 'Second porteur', 'CHE109322551')], 'CHE109322551', 7)?.id).toBe(9);
	});

	it('null ne désigne personne, même sur un lot plein de contacts sans IDE', () => {
		// MUTATION : « comparer sans garder la vacuité » → `null === null` ferait
		// de tout contact sans IDE un porteur.
		expect(findIdeHolder(lot, null, null)).toBeUndefined();
		expect(findIdeHolder(lot, '', null)).toBeUndefined();
	});
});

// ---------------------------------------------------------------------------
// Pureté du module
// ---------------------------------------------------------------------------

describe('pureté du module', () => {
	// Chemin depuis la racine de `frontend/`, où vitest s'exécute.
	// ⚠️ Pas `import.meta.url` : sous jsdom il n'est pas une URL `file:`.
	const source = readFileSync('src/lib/features/contacts/duplicate-probe.ts', 'utf-8');

	it('n’importe rien qui touche au réseau, au DOM ou au routeur', () => {
		// MUTATION : « introduire un import réseau » → le gate quitte le régime
		// de la seconde et redevient un test de composant.
		for (const forbidden of [
			"from '$app/",
			"from '$lib/shared/utils/api-client",
			// ⚠️ Les DEUX formes. L'AC nomme l'ALIAS (`$lib/features/contacts/contacts.api`)
			// et seule la RELATIVE était éprouvée : un import par alias passait la
			// preuve sans rien empêcher. Relevé en passe 2 de revue de code.
			"from './contacts.api",
			'contacts.api',
			"from '$lib/components/"
		]) {
			expect(source).not.toContain(forbidden);
		}
	});

	it('n’appelle aucune globale de réseau, d’horloge ou de DOM', () => {
		// MUTATION : « appeler Date.now() sans import ». En JavaScript ces
		// globales s'appellent SANS ligne `import` : une preuve qui ne lirait que
		// les imports serait verte.
		const body = source.replace(/\/\*\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
		for (const forbidden of ['fetch(', 'setTimeout(', 'setInterval(', 'Date.now(', 'document.', 'window.']) {
			expect(body).not.toContain(forbidden);
		}
	});
});

// ---------------------------------------------------------------------------
// describeProches — l'INVARIANT, pas la cascade
// ---------------------------------------------------------------------------

describe('describeProches', () => {
	it('deux fiches qui partagent une VILLE NON VIDE restent distinctes', () => {
		// MUTATION : « replier sur l'id seulement quand la cascade est vide »,
		// c'est-à-dire `bouts.length === 0 ? ... : ...` — la cascade LIVRÉE.
		//
		// ⚠️ C'est le défaut que la revue de code a trouvé, et il avait survécu à
		// son PROPRE doc-comment : celui-ci décrivait exactement ce cas — « le père
		// et le fils de la même localité ONT une localité » — au-dessus d'un code
		// qui s'arrêtait à la localité. La preuve d'alors employait une ville VIDE,
		// donc elle éprouvait la branche où le défaut ne se manifeste pas.
		const out = describeProches([d(7, 'Jean Dupont', 'Lausanne'), d(9, 'Jean Dupont', 'Lausanne')]);
		expect(out[0]).not.toBe(out[1]);
		expect(out).toEqual([' — Lausanne · #7', ' — Lausanne · #9']);
	});

	it('ne colle PAS d’id quand la ligne entière est déjà distincte', () => {
		// MUTATION : « ajouter l'id systématiquement ». Elle tiendrait l'invariant
		// — et noierait chaque proposition sous un numéro qui ne dit rien à
		// personne. L'invariant est « jamais identiques », pas « toujours numérotées ».
		expect(describeProches([d(1, 'Alpha SA', 'Lausanne'), d(2, 'Beta SA', 'Lausanne')])).toEqual([
			' — Lausanne',
			' — Lausanne'
		]);
	});

	it('la collision se juge sur la LIGNE ENTIÈRE, nom compris', () => {
		// MUTATION : « ne comparer que le descripteur ». Le test ci-dessus la
		// laisserait verte s'il n'assertait que la distinction ; celui-ci fixe
		// l'autre moitié, et les deux ensemble bornent le comportement.
		const out = describeProches([
			d(1, 'Alpha SA', 'Lausanne'),
			d(2, 'Alpha SA', 'Lausanne'),
			d(3, 'Beta SA', 'Lausanne')
		]);
		expect(out).toEqual([' — Lausanne · #1', ' — Lausanne · #2', ' — Lausanne']);
	});

	it('la cascade reste celle d’avant : ville, numéro, puis email en dernier recours', () => {
		expect(describeProches([d(1, 'A', 'Nyon', 'CLI-4', 'a@b.ch')])).toEqual([' — Nyon · CLI-4']);
		expect(describeProches([d(2, 'B', '', 'CLI-5', 'b@b.ch')])).toEqual([' — CLI-5']);
		// MUTATION : « pousser l'email même quand la ville est là » → rendrait
		// ' — Nyon · CLI-4 · a@b.ch', une ligne illisible.
		expect(describeProches([d(3, 'C', '', null, 'c@b.ch')])).toEqual([' — c@b.ch']);
	});

	it('deux homonymes SANS RIEN se replient sur leur id, comme avant', () => {
		expect(describeProches([d(4, 'Jean Dupont'), d(5, 'Jean Dupont')])).toEqual([
			' — #4',
			' — #5'
		]);
	});

	it('une ville faite d’ESPACES ne compte pas pour un descripteur', () => {
		// MUTATION : « filtrer sur la seule vacuité, sans trim ».
		expect(describeProches([d(6, 'X', '   ')])).toEqual([' — #6']);
	});

	it('rend UN suffixe par proposition, aligné sur l’index', () => {
		// MUTATION : « filtrer la liste » — l'alignement par index est le contrat.
		expect(describeProches([d(1, 'A', 'Nyon'), d(2, 'B'), d(3, 'C', 'Bex')])).toHaveLength(3);
		expect(describeProches([])).toEqual([]);
	});
});
