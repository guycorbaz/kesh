/**
 * **GARDE B — toute clé demandée par le frontend existe au catalogue.**
 *
 * Story 23-1a (#316). Pendant de la garde A (`kesh-i18n/src/loader.rs`,
 * `parity_between_locales`), qui vérifie que les quatre catalogues déclarent le même
 * ensemble de clés.
 *
 * ⚠️ **Le défaut que ces deux gardes ferment est SILENCIEUX.** `i18nMsg(clé, repli)`
 * retombe sur son repli — du français en dur — quand la clé manque, et `loader.rs`
 * charge `fr-CH` comme base de repli des trois autres locales. Un oubli de traduction
 * ne produit donc ni erreur, ni clé brute à l'écran : il produit **du français
 * correct**, servi à un germanophone, tous gates au vert.
 *
 * ⚠️ **L'INVERSION (D4-ter).** Cette garde n'énumère PAS les formes d'appel qui
 * marchent. Cinq passes de revue l'ont tenté, et chacune a trouvé une forme de plus :
 * littéral, gabarit, relais, appel multi-ligne, clé portée par une table, ternaire dans
 * le premier argument, gabarit affecté à une variable, clé fabriquée par une fonction.
 * *Une énumération de formes est ouverte par nature.* La garde inventorie donc le
 * **complément** — les sites qui ne résolvent PAS en littéral — un ensemble clos et
 * comptable, dont l'assertion de cardinalité ne peut pas être défaite par une forme
 * imprévue.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
// Seul import de production autorisé (AC7 / D1-bis) : un utilitaire de lecture, sans
// clé ni catalogue, partagé avec le moissonneur de la story 23-1b.
import { findCallSites, findRelays, masquerCommentaires } from './i18n-literal-reader.js';
// ⚠️ La règle de périmètre vivait ici en DEUX copies inline, à côté de la fonction du module
// partagé — trois exemplaires d'une même règle. Ils coïncidaient au caractère près, ce qui est
// la situation d'avant la dérive, pas une garantie contre elle. (Revue de code 23-1b, passe 3.)
import { dansLePerimetreDeFichier } from './i18n-harvest.js';
import { DETTE_CONNUE } from './i18n-dette-connue.js';

const LOCALES = ['fr-CH', 'de-CH', 'it-CH', 'en-CH'] as const;
const RACINE_FTL = '../crates/kesh-i18n/locales';

/**
 * Valeurs de contrôle, mesurées le 2026-08-19. Un écart se recompte, il ne s'ajuste pas.
 *
 * ⚠️ `sitesTotal` est passé de **1493 à 1497** à la story 23-3, et le changement est DÉLIBÉRÉ :
 * quatre chaînes françaises étaient **en dur** dans les écrans de factures fournisseurs — deux
 * « Chargement… », un « Qté », un « TVA » —, invisibles au moissonneur (qui ne lit que les
 * `i18nMsg`) comme à l'allowlist. Les couvrir crée quatre sites d'appel. C'est une **hausse
 * assumée**, pas une dérive : la borne exacte a fait exactement son travail en l'exigeant.
 */
const ATTENDU = {
	sitesTotal: 1497,
	sitesNonResolus: 33,
	relais: 7,
	sitesGabarit: 10,
	litterauxMin: 1050,
	clesDepuisTsMin: 5
} as const;

/**
 * Les 8 préfixes dynamiques et leurs valeurs, **écrites en dur**.
 *
 * ⚠️ Les lire depuis le code de production ferait qu'une carte vidée par erreur rendrait
 * le test vert à vide — le mode d'échec du test muet. La contrepartie est l'assertion de
 * cardinalité : elle rougit dès que la carte de production évolue sans le test.
 *
 * ⚠️ **Deux exceptions à cette contrepartie, et elles sont ASSUMÉES** :
 *  - `vat-category-*` : la colonne `vat_rates.category` n'a **aucune contrainte CHECK**
 *    (décision de la story 11-1, migration `20260613000001`). Un administrateur qui crée
 *    un taux étend l'espace des clés **sans toucher au code** : aucune assertion ne peut
 *    rougir. Les 5 catégories seedées sont déclarées, les catégories créées ne sont
 *    couvertes par aucune garde.
 *  - `bank-import-info-*` : les valeurs sont poussées par le BACKEND Rust
 *    (`kesh-api/src/routes/bank_imports.rs:693` et `:1668`), le frontend ne recevant
 *    qu'un `informational: string[]`. Il n'existe aucune carte à confronter. Un
 *    troisième code informationnel s'afficherait en `snake_case` brut, dans les quatre
 *    langues, sans qu'aucune garde ne bouge.
 */
const MOTIFS_DYNAMIQUES: Record<string, readonly string[]> = {
	'journal-': ['achats', 'ventes', 'banque', 'caisse', 'od'],
	'account-type-': ['asset', 'liability', 'revenue', 'expense'],
	'due-dates-filter-': ['all', 'unpaid', 'overdue', 'paid'],
	'reports-filename-': ['balance-sheet', 'income-statement', 'trial-balance', 'journals', 'vat'],
	// ⚠️ angle mort assumé — colonne libre, cf. le commentaire ci-dessus
	'vat-category-': ['normal', 'reduced', 'special', 'exempt', 'custom'],
	'reminders-error-': [
		'invoice-not-found', 'invoice-not-validated', 'invoice-already-paid', 'dunning-paused',
		'no-next-level', 'contact-archived', 'contact-email-missing', 'content-empty',
		'content-too-long', 'not-pdf-ready', 'rate-limited', 'database-error', 'smtp-failed',
		'sent-but-gone', 'sent-not-recorded'
	],
	'imported-supplier-invoices-error-': [
		'unsupported-file-type', 'file-too-large', 'symlink-rejected', 'duplicate',
		'no-qr-code-found', 'invalid-spc-payload', 'invalid-iban', 'pdf-render-error',
		'file-read-error', 'field-too-long'
	],
	// ⚠️ angle mort assumé — valeurs produites par le backend Rust, déclarées APRÈS
	// transformation (`info.replace(/_/g, '-')`), donc en tirets et non en snake_case.
	'bank-import-info-': ['bank-csv-profile-auto-matched', 'bank-csv-multiple-profile-matches']
};

/**
 * ⚠️ **TROISIÈME ensemble ouvert, et il n'a pas de préfixe** : `vat-rates/+page.svelte:52`
 * écrit `i18nMsg(r.label, r.label)` — **la clé elle-même vient de la colonne
 * `vat_rates.label`**, un `VARCHAR` libre sans contrainte `CHECK`. Aucune énumération
 * n'est possible : ce site figure à l'inventaire des non résolus et y restera.
 *
 * Les trois ensembles ouverts de cette story sont donc `vat-category-*`,
 * `bank-import-info-*` et celui-ci. *Aucune garde ne les borne, et c'est écrit plutôt
 * que découvert.*
 */
const ANGLE_MORT_CLE_EN_COLONNE = 'routes/(app)/settings/vat-rates/+page.svelte:52';

/**
 * Cardinalité attendue de chaque préfixe dynamique — **la contrepartie des valeurs
 * écrites en dur**, et elle manquait.
 *
 * ⚠️ Sans elle, `MOTIFS_DYNAMIQUES` peut **rétrécir en silence** : un « nettoyage » qui
 * retire deux valeurs de `journal-` sort deux clés de couverture sans qu'un seul test
 * rougisse — mutation exécutée en passe 3 de revue, **neuf tests verts**. Le docstring
 * de ce fichier promettait pourtant cette protection depuis le premier jet.
 *
 * L'invariant est **à deux places** : pour rétrécir une famille sans bruit, il faudrait
 * éditer la table ET ce compteur. C'est peu, mais c'est exactement ce qui sépare une
 * garde d'une déclaration d'intention.
 *
 * ⚠️ **QUATRIÈME angle mort, écrit ici faute de pouvoir le fermer** : `findRelays`
 * travaille **par fichier**. Un relais qui serait *importé* d'un module partagé ne
 * serait recensé nulle part — et, contrairement aux autres trous, **aucun compteur ne
 * bougerait**. La règle DRY du dépôt pousse vers cette extraction : les sept relais
 * actuels sont sept copies de la même fonction de trois lignes. À traiter en 23-2.
 */
const CARDINALITES: Record<string, number> = {
	'journal-': 5,
	'account-type-': 4,
	'due-dates-filter-': 4,
	'reports-filename-': 5,
	'vat-category-': 5,
	'reminders-error-': 15,
	'imported-supplier-invoices-error-': 10,
	'bank-import-info-': 2
};

/**
 * **Les familles résolues de l'inventaire (D4-ter, étape 4).**
 *
 * Ces clés atteignent `i18nMsg` sans jamais s'écrire comme littéral au site d'appel :
 * elles vivent dans une table de données (`i18nKey`, `labelKey`), dans un index
 * (`LABEL_KEY[status]`), dans un ternaire, ou sont fabriquées par une fonction
 * (`accountRoleKey`) ou par un gabarit affecté à une variable (`AccountingTooltip`).
 *
 * ⚠️ **C'est ce trou qui a laissé QUATRE ENTRÉES DE LA BARRE DE NAVIGATION** —
 * `nav-credit-notes`, `nav-email-templates`, `nav-projects`,
 * `nav-supplier-invoices-import` — s'afficher en français dans les quatre langues,
 * hors de tout décompte, jusqu'à la passe 3 de revue de cette story.
 *
 * ⚠️ **Valeurs EN DUR**, comme celles de `MOTIFS_DYNAMIQUES` et pour la même raison :
 * les lire depuis la production rendrait le test vert à vide si la table se vidait.
 */
const FAMILLES_RESOLUES: Record<string, readonly string[]> = {
	// routes/(app)/+layout.svelte:154 — table `i18nKey` du menu principal
	'nav-': [
		'home', 'contacts', 'products', 'invoices', 'invoicing-due-dates', 'invoicing-reminders',
		'credit-notes', 'supplier-invoices', 'supplier-invoices-import', 'payment-batches',
		'accounts', 'fiscal-years', 'bank-accounts', 'bank-profiles', 'reconciliation-rules',
		'projects', 'export-global', 'settings', 'opening-balances', 'email-templates',
		'admin-backup', 'admin-restore'
	],
	// routes/(app)/reports/+page.svelte:628 — table `labelKey` des onglets
	'reports-': [
		'balance-sheet', 'income-statement', 'trial-balance', 'journals', 'vat',
		'project-expenses', 'project-return', 'aged-balance'
	],
	// lib/features/invoices/PaymentStatusBadge.svelte:24 — index `LABEL_KEY[status]`
	'payment-status-': ['paid', 'unpaid', 'overdue'],
	// accounts/+page.svelte:62, opening-balances:133, BalanceSheetView:205 —
	// clé FABRIQUÉE par `accountRoleKey()` depuis `ACCOUNT_ROLES`
	'account-role-': [
		'receivable', 'default-revenue', 'payable', 'vat-recoverable', 'vat-payable',
		'vat-settlement', 'equity-capital', 'equity-other', 'retained-earnings',
		'current-year-result', 'none', 'archived-hint'
	],
	// routes/(app)/contacts/+page.svelte:646, :732, :829 — ternaire de littéraux
	'contact-type-': ['personne', 'entreprise'],
	// lib/features/bank-import/BankProfileForm.svelte:256 — ternaire de littéraux
	'bank-import-profile-labels-': [
		'bank-name', 'filename-pattern', 'filename-pattern-help', 'date-format',
		'field-separator', 'decimal-separator', 'encoding', 'header-row-count',
		'column-mapping', 'use-debit-credit-split', 'update', 'create'
	],
	// lib/shared/components/AccountingTooltip.svelte:49, :51 — gabarit affecté à une
	// variable (`const naturalKey = $derived(...)`), invisible en position d'argument
	'tooltip-': [
		'balanced-natural', 'balanced-technical', 'credit-natural', 'credit-technical',
		'debit-natural', 'debit-technical', 'journal-natural', 'journal-technical'
	]
};

/**
 * Les 10 sites de gabarit, **confrontés et non produits**.
 *
 * ⚠️ Le développeur ne doit pas *fabriquer* cette liste par l'extraction — c'est elle
 * qui a déjà échoué cinq fois. Il la *compare*. Un simple compte laisserait d'ailleurs
 * passer un ajout compensant un retrait.
 */
const SITES_GABARIT_ATTENDUS: readonly string[] = [
	'lib/features/bank-import/BankImportUpload.svelte  bank-import-info-',
	'lib/features/journal-entries/JournalEntryForm.svelte  journal-',
	'lib/features/journal-entries/VatPurchaseAssistant.svelte  vat-category-',
	'lib/features/reminders/reminder-error-label.ts  reminders-error-',
	'lib/features/reports/reports.api.ts  reports-filename-',
	'routes/(app)/accounts/+page.svelte  account-type-',
	'routes/(app)/invoices/due-dates/+page.svelte  due-dates-filter-',
	'routes/(app)/journal-entries/+page.svelte  journal-',
	'routes/(app)/settings/vat-rates/+page.svelte  vat-category-',
	'routes/(app)/supplier-invoices/import/+page.svelte  imported-supplier-invoices-error-'
];

/**
 * Préfixes dont **toutes** les clés du catalogue sont demandées par le frontend, donc
 * pour lesquels une clé orpheline est un défaut. Repris du second contrôle de
 * `duplicate-i18n-keys.test.ts` (story 22-2b), que cette garde remplace.
 *
 * ⚠️ **Ne se généralise PAS** : le catalogue sert aussi `kesh-qrbill` et `kesh-report`
 * pour les PDF, si bien qu'une clé sans demandeur côté frontend n'est pas orpheline pour
 * autant — `reports-filename-*` en donne le contre-exemple, avec 7 clés déclarées pour
 * 5 valeurs de `ReportType`. La liste s'étend story par story, jamais par défaut.
 */
const PREFIXES_A_COUVERTURE_CLOSE: readonly string[] = ['contact-duplicate-'];

// ─────────────────────────────────────────────────────────────────────────────

function clesDuCatalogue(locale: string): Set<string> {
	const texte = readFileSync(join(RACINE_FTL, locale, 'messages.ftl'), 'utf-8');
	const cles = new Set<string>();
	for (const ligne of texte.split('\n')) {
		const m = /^([a-zA-Z][\w-]*)\s*=/.exec(ligne);
		if (m) cles.add(m[1]);
	}
	return cles;
}

type Releve = {
	litteraux: Map<string, string>;
	gabarits: string[];
	nonResolus: string[];
	sitesTotal: number;
	clesDepuisTs: Set<string>;
};

/** Parcourt `src` et classe chaque site d'appel. */
function relever(): Releve {
	const litteraux = new Map<string, string>();
	const gabarits: string[] = [];
	const nonResolus: string[] = [];
	const clesDepuisTs = new Set<string>();
	let sitesTotal = 0;

	const parcourir = (rep: string) => {
		for (const e of readdirSync(rep, { withFileTypes: true })) {
			const chemin = join(rep, e.name);
			if (e.isDirectory()) {
				parcourir(chemin);
				continue;
			}
			// ⚠️ Les fichiers `.test.*` sont hors collecte (D5-bis) : `i18n.svelte.test.ts`
			// demande `une-cle` et `compteur`, clés FICTIVES qui doivent le rester.
			if (!dansLePerimetreDeFichier(e.name)) continue;

			const texte = readFileSync(chemin, 'utf-8');
			const relatif = chemin.replace(/^src\//, '');
			for (const site of findCallSites(texte)) {
				sitesTotal += 1;
				if (site.arg === null) {
					nonResolus.push(`${relatif}:${site.line}`);
				} else if (site.arg.kind === 'template') {
					const prefixe = site.arg.value.slice(0, site.arg.value.indexOf('${'));
					gabarits.push(`${relatif}  ${prefixe}`);
				} else {
					if (!litteraux.has(site.arg.value)) litteraux.set(site.arg.value, relatif);
					if (chemin.endsWith('.ts')) clesDepuisTs.add(site.arg.value);
				}
			}
		}
	};
	parcourir('src');
	return { litteraux, gabarits, nonResolus, sitesTotal, clesDepuisTs };
}

/** Les clés que le frontend demande RÉELLEMENT : littéraux ∪ expansions des motifs. */
function clesDemandees(r: Releve): Set<string> {
	const toutes = new Set(r.litteraux.keys());
	for (const table of [MOTIFS_DYNAMIQUES, FAMILLES_RESOLUES]) {
		for (const [prefixe, valeurs] of Object.entries(table)) {
			for (const v of valeurs) toutes.add(prefixe + v);
		}
	}
	return toutes;
}

// ─────────────────────────────────────────────────────────────────────────────

describe('garde i18n — les clés demandées existent au catalogue', () => {
	const releve = relever();
	const catalogues = Object.fromEntries(LOCALES.map((l) => [l, clesDuCatalogue(l)]));
	const fr = catalogues['fr-CH'];
	const dette = new Set(DETTE_CONNUE);

	it('la collecte ne passe pas à vide', () => {
		// Bornes anti-test-muet : un motif d'extraction cassé rendrait tout ce qui suit
		// vert sans rien vérifier.
		expect(releve.litteraux.size).toBeGreaterThanOrEqual(ATTENDU.litterauxMin);
		// ⚠️ Borne par EXTENSION, posée à la valeur mesurée et non « mesuré moins une
		// marge » : deux des cinq clés `.ts` viennent d'appels multi-lignes en TypeScript
		// pur (`notify.ts`). Une borne à 3 serait verte sur leur perte.
		expect(releve.clesDepuisTs.size).toBeGreaterThanOrEqual(ATTENDU.clesDepuisTsMin);
		expect(releve.sitesTotal).toBe(ATTENDU.sitesTotal);
	});

	it('toute clé demandée existe au catalogue, ou figure à la dette connue', () => {
		const absentes: string[] = [];
		for (const cle of clesDemandees(releve)) {
			if (fr.has(cle) || dette.has(cle)) continue;
			absentes.push(`${cle}  (${releve.litteraux.get(cle) ?? 'expansion de motif'})`);
		}
		expect(absentes, `clés demandées et absentes des catalogues :\n  ${absentes.join('\n  ')}`)
			.toEqual([]);
	});

	it("l'allowlist de dette ne se fossilise pas", () => {
		const demandees = clesDemandees(releve);
		const obsoletes: string[] = [];
		for (const cle of DETTE_CONNUE) {
			if (fr.has(cle)) obsoletes.push(`${cle} : désormais au catalogue — retirer la ligne`);
			else if (!demandees.has(cle))
				// ⚠️ DEUX causes, et la seconde se vérifie d'abord : la feature a été
				// retirée (retirer la ligne), ou l'extracteur ne la voit plus (le réparer).
				obsoletes.push(`${cle} : plus demandée — feature retirée, OU extracteur cassé`);
		}
		expect(obsoletes, `entrées de dette obsolètes :\n  ${obsoletes.join('\n  ')}`).toEqual([]);
	});

	it('les 7 relais locaux sont recensés — cardinalité assertée', () => {
		// ⚠️ **Ce garde-fou manquait au premier jet, alors que la tâche le déclarait fait.**
		// Sans lui, un huitième relais ajouté sans que le test le sache rendrait TOUTES
		// ses clés invisibles, en silence — le défaut que le recensement des relais avait
		// été créé pour clore, et qui avait déjà coûté 29 clés et un dossier entier.
		const trouves = new Set<string>();
		const parcourir = (rep: string) => {
			for (const e of readdirSync(rep, { withFileTypes: true })) {
				const chemin = join(rep, e.name);
				if (e.isDirectory()) {
					parcourir(chemin);
					continue;
				}
				if (!dansLePerimetreDeFichier(e.name)) continue;
				for (const nom of findRelays(masquerCommentaires(readFileSync(chemin, 'utf-8')))) {
					trouves.add(`${chemin}:${nom}`);
				}
			}
		};
		parcourir('src');
		expect(trouves.size, `relais recensés :\n  ${[...trouves].join('\n  ')}`).toBe(ATTENDU.relais);
	});

	it('le troisième ensemble ouvert est nommé, pas seulement compté', () => {
		// AC7-ter : les trois ensembles ouverts portent un commentaire. Les deux premiers
		// ont un préfixe et vivent dans MOTIFS_DYNAMIQUES ; le troisième n'en a pas — sa
		// clé vient d'une colonne libre —, il est donc nommé par son site.
		expect(releve.nonResolus).toContain(ANGLE_MORT_CLE_EN_COLONNE);
	});

	it('chaque préfixe dynamique porte sa cardinalité — la table ne rétrécit pas en silence', () => {
		expect(Object.keys(CARDINALITES).sort()).toEqual(Object.keys(MOTIFS_DYNAMIQUES).sort());
		for (const [prefixe, valeurs] of Object.entries(MOTIFS_DYNAMIQUES)) {
			expect(valeurs.length, `${prefixe} : ${valeurs.length} valeurs déclarées`).toBe(
				CARDINALITES[prefixe]
			);
		}
	});

	it('les 8 préfixes dynamiques sont déclarés, et leurs 10 sites confrontés', () => {
		expect([...new Set(releve.gabarits)].sort()).toEqual([...SITES_GABARIT_ATTENDUS].sort());
		expect(releve.gabarits.length).toBe(ATTENDU.sitesGabarit);
		const prefixesVus = new Set(releve.gabarits.map((g) => g.split('  ')[1]));
		expect([...prefixesVus].sort()).toEqual(Object.keys(MOTIFS_DYNAMIQUES).sort());
	});

	it('les familles résolues de l\'inventaire couvrent bien leurs clés du catalogue', () => {
		// Chaque famille déclarée doit correspondre à des clés réelles : une famille dont
		// aucune valeur n'existerait au catalogue signalerait une table renommée ou vidée.
		for (const [prefixe, valeurs] of Object.entries(FAMILLES_RESOLUES)) {
			const auCatalogue = valeurs.filter((v) => fr.has(prefixe + v)).length;
			expect(auCatalogue, `${prefixe} : aucune de ses ${valeurs.length} valeurs au catalogue`)
				.toBeGreaterThan(0);
		}
	});

	it("l'inventaire des sites non résolus est borné", () => {
		// ⚠️ **L'assertion qui porte la garantie.** Elle ne dépend d'aucune énumération de
		// formes : un site d'une forme inconnue y tombe automatiquement. Si elle rougit,
		// un nouveau site d'indirection est apparu — il doit être RÉSOLU (ses clés
		// déclarées) ou ÉCRIT comme angle mort, jamais simplement recompté.
		expect(
			releve.nonResolus.length,
			`sites dont le premier argument n'est ni littéral ni gabarit :\n  ${releve.nonResolus.join('\n  ')}`
		).toBe(ATTENDU.sitesNonResolus);
		// ⚠️ **8 de ces 33 sont des DÉCLARATIONS de fonction**, pas des sites d'appel :
		// `function msg(key: string, …)` et la déclaration d'`i18nMsg` elle-même sont
		// capturées par le motif `nom(`. Avec les 7 corps `return i18nMsg(key, fallback)`,
		// **15 des 33 sont du boilerplate de relais**. Vérifié en passe 3 de revue contre
		// les parseurs TypeScript et Svelte : 1485 sites d'appel réels, tous concordants.
	});

	it('aucune clé orpheline sur les préfixes à couverture close', () => {
		expect(PREFIXES_A_COUVERTURE_CLOSE.length).toBeGreaterThanOrEqual(1);
		const demandees = clesDemandees(releve);
		const couvertes = [...fr].filter((k) =>
			PREFIXES_A_COUVERTURE_CLOSE.some((p) => k.startsWith(p))
		);
		expect(couvertes.length).toBeGreaterThanOrEqual(5);
		const orphelines = couvertes.filter((k) => !demandees.has(k));
		expect(orphelines, `clés au catalogue que personne ne demande :\n  ${orphelines.join('\n  ')}`)
			.toEqual([]);
	});
});
