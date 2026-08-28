/**
 * Garde contre les **sélecteurs E2E figés sur un libellé traduit** — KF-043 (#326).
 *
 * ## Le défaut, et pourquoi la suite ne peut pas le voir
 *
 * ⚠️ **La suite E2E ne s'exécute qu'en `fr-CH`.** Le seed CI crée une société en
 * FR/FR et rien ne rejoue la suite dans une autre locale. Un sélecteur Playwright
 * figé sur un libellé français reste donc **vert tant que le français ne change
 * pas** — et la suite est *structurellement* incapable de signaler qu'il ne vaut
 * rien dans les trois autres langues.
 *
 * Le cas fondateur, trouvé en passe 3 de revue de la story 23-3b : le sélecteur
 * `has-text("Administration")` visait un groupe de navigation par son texte, sur
 * **5 occurrences dans 3 fichiers**. Or ce libellé est identique en français, en
 * allemand et en anglais — il ne diffère qu'en **italien** (`Amministrazione`).
 * Le sélecteur serait resté vert dans **trois langues sur quatre**.
 *
 * ⚠️ **Un sélecteur vert dans trois langues sur quatre est plus traître qu'un
 * sélecteur cassé** : rien ne le signale, et il donne l'apparence d'une couverture
 * qu'il n'a pas. Celui-là a été trouvé par un grep, pas par la suite.
 *
 * ## Ce que cette garde prouve, et ce qu'elle ne prouve PAS
 *
 * Elle **ne dit rien** de la justesse de l'application en italien : aucune analyse
 * statique ne le peut. Ce qu'elle rend vérifiable est plus modeste et bien réel —
 * **que la suite cesse de dépendre des libellés**, ce qui est la condition
 * préalable à tout rejeu dans une autre locale (option 3 de #326). Sans elle, la
 * règle « un sélecteur ne se fige jamais sur un libellé traduit » reste une
 * discipline, c'est-à-dire quelque chose qu'on peut affirmer sans l'avoir fait.
 *
 * ## Elle naît PLEINE, et c'est délibéré
 *
 * Le relevé initial rend **101 occurrences**, soit **43 couples (fichier, libellé)
 * distincts** sur 14 fichiers — c'est cette seconde grandeur que la liste porte,
 * un même libellé étant souvent visé plusieurs fois dans une même spec. Les corriger d'un
 * coup demanderait autant de `data-testid` neufs et de réécritures — un chantier
 * sans rapport avec l'issue qui l'a ouverte. L'allowlist ci-dessous les tolère
 * donc **nommément**, et la garde empêche la dette de **grossir** : un sélecteur
 * neuf figé sur du français rougit au gate.
 *
 * C'est le patron de l'Epic 23, dont la note de régime allégé dit l'essentiel :
 * *« leur fonction n'est pas de résorber la dette mais d'empêcher qu'elle
 * grossisse »*. L'allowlist est **décroissante par construction** — une entrée
 * qui ne correspond plus à aucun site fait échouer la garde, faute de quoi la
 * liste deviendrait un cimetière.
 *
 * ## Ce qu'elle vise, et ce qu'elle laisse passer
 *
 * | forme | vue ? | pourquoi |
 * |---|---|---|
 * | `getByText('Créer')`, `getByLabel(…)`, `getByPlaceholder(…)` | ✅ | localise par texte |
 * | `has-text("Administration")` | ✅ | idem, via `locator()` |
 * | `getByRole('button', { name: 'Enregistrer' })` | ✅ | idem, via le nom accessible |
 * | `toContainText('Facture créée')`, `toHaveText(…)` | ❌ **délibérément** | ce sont des **assertions de contenu** : vérifier ce qui s'affiche est l'objet même du test, pas une fragilité |
 * | `getByRole('button', { name: /définitivement/ })` | ❌ | une **regex** n'est pas relevée — angle mort assumé, cf. ci-dessous |
 * | `getByTestId(…)` | ❌ | c'est la forme recommandée |
 *
 * ⚠️ **L'angle mort des regex est assumé, pas ignoré.** `name: /définitivement/`
 * est aussi fragile qu'une chaîne, mais le relevé montre que la forme est rare et
 * qu'elle sert souvent à des correspondances partielles légitimes. L'élargir
 * demanderait de trier des motifs, ce que la story 23-3b a **chiffré puis
 * proscrit** pour une garde voisine : le coût est une garde bruyante, et une
 * garde bruyante finit désactivée.
 *
 * ## Le critère « c'est du français » est une HEURISTIQUE, et elle se trompe
 *
 * Un accent, ou un mot-outil / verbe d'action courant. Elle rate ce qui n'a ni
 * l'un ni l'autre (`Brouillon`, `Total`) et peut relever un nom propre. C'est
 * assumé : le but n'est pas l'exhaustivité mais l'**arrêt de la croissance**, et
 * une exemption coûte une ligne quand un faux négatif coûte un défaut muet.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

// Chemin relatif au cwd de vitest (la racine `frontend/`), comme les gardes
// voisines qui posent `const RACINE = 'src'`.
const RACINE_E2E = 'tests/e2e';

/**
 * Sélecteurs qui **localisent** un élément par son texte. Les assertions de
 * contenu (`toHaveText`, `toContainText`) en sont délibérément absentes.
 */
const SELECTEURS = new RegExp(
	[
		String.raw`(?:getByText|getByLabel|getByPlaceholder)\(\s*['"]([^'"]{2,80})['"]`,
		String.raw`has-text\(\s*["']([^"']{2,80})["']`,
		String.raw`getByRole\([^)]*?name:\s*['"]([^'"]{2,80})['"]`,
	].join('|'),
	'g',
);

const ACCENT = /[àâäéèêëîïôöùûüçÀÂÉÈÊËÎÏÔÖÙÛÜÇ]/;
const MOTS_FR =
	/\b(le|la|les|un|une|des|du|de|aucun|aucune|tous|toutes|pour|sur|avec|sans|et|ou|Enregistrer|Supprimer|Modifier|Annuler|Créer|Ajouter|Valider|Fermer|Rechercher|Facture|Compte|Client|Contact|Produit|Écriture|Exercice|Administration|Retour|Français|Guidé|Nouveau|Nouvelle|Marquer|Confirmer|Payée|Continuer)\b/i;

/** Un sélecteur relevé : le fichier, et le texte sur lequel il se fige. */
interface Site {
	fichier: string;
	texte: string;
}

function fichiersSpec(dir: string): string[] {
	const out: string[] = [];
	for (const nom of readdirSync(dir)) {
		const chemin = join(dir, nom);
		if (statSync(chemin).isDirectory()) out.push(...fichiersSpec(chemin));
		else if (nom.endsWith('.spec.ts')) out.push(chemin);
	}
	return out.sort();
}

function releve(): Site[] {
	const out: Site[] = [];
	for (const chemin of fichiersSpec(RACINE_E2E)) {
		const source = readFileSync(chemin, 'utf-8');
		const fichier = relative(RACINE_E2E, chemin);
		for (const m of source.matchAll(SELECTEURS)) {
			const texte = m[1] ?? m[2] ?? m[3];
			if (!texte) continue;
			if (ACCENT.test(texte) || MOTS_FR.test(texte)) out.push({ fichier, texte });
		}
	}
	return out;
}

/** Clé stable d'un site — insensible aux déplacements de lignes. */
const cle = (s: Site) => `${s.fichier} :: ${s.texte}`;

/**
 * Dette relevée le 2026-08-24, à l'ouverture de la garde. **Décroissante :** une
 * entrée qui ne correspond plus à aucun site fait échouer la garde.
 *
 * ⚠️ **On ne complète PAS cette liste pour faire passer un gate.** Un sélecteur
 * neuf se fige sur un `data-testid`, jamais sur un libellé — c'est le seul usage
 * que cette liste ne doit pas avoir.
 */
const DETTE_CONNUE: readonly string[] = [
	"api-keys.spec.ts :: Révoquer",
	"contact-client-number.spec.ts :: Annuler",
	"contact-client-number.spec.ts :: Créer",
	"contact-client-number.spec.ts :: Enregistrer",
	"contact-duplicate-probe.spec.ts :: Dubarde Vins Sàrl",
	"contacts.spec.ts :: Annuler",
	"contacts.spec.ts :: Créer",
	"fiscal-years.spec.ts :: Créer",
	"fiscal-years.spec.ts :: Enregistrer",
	"fiscal-years.spec.ts :: Valider",
	"homepage-settings.spec.ts :: Comptabilité",
	"invoices.spec.ts :: Créer la facture",
	"invoices.spec.ts :: Facture",
	"invoices.spec.ts :: Nouvelle facture",
	"journal-entries.spec.ts :: Annuler",
	"journal-entries.spec.ts :: Supprimer",
	"journal-entries.spec.ts :: Valider",
	"onboarding-path-b.spec.ts :: Compte bancaire",
	"onboarding-path-b.spec.ts :: Configurer pour la production",
	"onboarding-path-b.spec.ts :: Continuer",
	"onboarding-path-b.spec.ts :: Coordonnées",
	"onboarding-path-b.spec.ts :: Français",
	"onboarding-path-b.spec.ts :: Guidé",
	"onboarding.spec.ts :: Configurer pour la production",
	"onboarding.spec.ts :: Confirmer",
	"onboarding.spec.ts :: Contact",
	"onboarding.spec.ts :: Continuer",
	"onboarding.spec.ts :: Explorer avec des données de démo",
	"onboarding.spec.ts :: Français",
	"onboarding.spec.ts :: Guidé",
	"onboarding.spec.ts :: Indépendant",
	"onboarding.spec.ts :: Instance de démonstration",
	"onboarding.spec.ts :: Réinitialiser pour la production",
	"onboarding.spec.ts :: Toutes les données de démonstration",
	"product-revenue-account.spec.ts :: Modifier",
	"product-revenue-account.spec.ts :: Nouveau produit",
	"products.spec.ts :: Annuler",
	"products.spec.ts :: Créer",
	"sidebar-navigation.spec.ts :: Administration",
	"vat-purchase-assistant.spec.ts :: Valider",
];

describe('garde E2E — un sélecteur ne se fige pas sur un libellé traduit (KF-043, #326)', () => {
	it('ne relève aucun sélecteur hors de la dette connue', () => {
		const sites = releve();
		const tolerees = new Set(DETTE_CONNUE);
		const neufs = sites.filter((s) => !tolerees.has(cle(s)));

		expect(
			neufs.map(cle).sort(),
			"Sélecteur(s) figé(s) sur un libellé traduit. La suite E2E ne tourne qu'en " +
				"français : un tel sélecteur reste vert ici et ne vaut rien dans les trois " +
				'autres langues. Viser un `data-testid` — ne PAS ajouter à `DETTE_CONNUE`.',
		).toEqual([]);
	});

	it("ne collecte pas à vide — la garde doit voir les fichiers de spec", () => {
		// Borne anti-test-muet : si le chemin des specs change ou que la lecture
		// échoue, le relevé tombe à zéro et la garde devient verte ET aveugle.
		// C'est le mode d'échec que la 23-3b a payé — sa première version était
		// verte parce que son extracteur ne trouvait aucune fonction à analyser.
		expect(fichiersSpec(RACINE_E2E).length).toBeGreaterThanOrEqual(30);
	});

	it('la dette connue ne contient pas de ligne morte', () => {
		const presents = new Set(releve().map(cle));
		const mortes = DETTE_CONNUE.filter((k) => !presents.has(k));

		expect(
			mortes,
			'Entrée(s) de `DETTE_CONNUE` ne correspondant plus à aucun sélecteur : ' +
				'les retirer. Sans ce contrôle la liste devient un cimetière, et la ' +
				'garde une décoration.',
		).toEqual([]);
	});
});
