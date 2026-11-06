# Story 33-1b : le numéro interne à l'écran — liste, fiche, réglage, recherche, manuels (#926)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche
`story/33-0-kickoff`, numéros de ligne de `4ef19901`. Registre : `e33-choix-autonomes-e331.md`. Clé courte `e331b`. **Après la 33-1a**,
dont elle lit `internalNumber` et `supplierInternalNumberFormat`. **Aucune migration.***

## Story

En tant que comptable,
je veux voir, trier et chercher le numéro interne partout où une facture fournisseur est nommée, et en régler le format,
afin de passer sans hésiter de l'écran au classeur et du classeur à l'écran.

## L'inventaire clos

Établi par `git grep -n "supplierInvoiceNumber\|supplier_invoice_number"` à `4ef19901`, en excluant les migrations, le squash et les tests.
**Tout site de lecture** y figure avec son sort. L'étape 0 rejoue la commande : un site neuf s'ajoute au tableau, il ne s'ignore pas.

**Règle** (`C-e331-20`) : *ce qui nomme la facture dans une phrase* dit le numéro interne, à défaut celui du fournisseur, à défaut `#id` (forme
courte). *Ce qui sert à reconnaître la pièce* dit les deux (forme longue : `FF-2026-001 (ABC-123)`). *Ce qui compare à une donnée externe* garde le
numéro du fournisseur.

### Écrans

| Site | Aujourd'hui | Sort |
|---|---|---|
| `supplier-invoices/+page.svelte:255` | `supplierInvoiceNumber ?? #id` | Deux colonnes : **N° interne** (triable), **N° fournisseur** ; `—` si absent |
| `supplier-invoices/[id]/+page.svelte:596` | titre | Titre : forme courte ; deux lignes au `<dl>` (AC6) |
| `SupplierInvoiceForm.svelte:601` | « N° facture fournisseur » | Libellé « N° du fournisseur (facultatif) », `testid` inchangé ; numéro interne affiché en lecture seule s'il existe, sinon « attribué à la validation » |
| `payment-batches/+page.svelte:186` | `numéro ?? #id — contact` | Forme longue : c'est là qu'on identifie les factures à payer |
| `journal-entries/[id]/+page.svelte:562` | lien « Facture fournisseur … » | Forme courte |
| `CancelReconciliationDialog.svelte:92` | phrase de confirmation | Forme courte |
| `ReconciliationProposals.svelte:501` (avis de lot) | phrase | Forme courte |
| `ReconciliationProposals.svelte:543` (candidate) | `contact — numéro (montant)` | Forme longue : le relevé peut citer le numéro du fournisseur |

`LinkSettlementModal.svelte:290` : factures **clientes** seulement (`c.invoiceId`) : aucun changement, vérifié.

### Serveur : ce que ces écrans lisent

| Site | Sort |
|---|---|
| `routes/journal_entries.rs:196` + `imported_supplier_invoices.rs:375,399` | ajouter `internal_number` |
| `routes/reconciliation.rs:133,1234` + `reconciliation.rs:847,880` (avis) | ajouter `internal_number` |
| `routes/reconciliation.rs:184,1270` (candidate) | ajouter `internal_number` (`None` pour une facture cliente) ; `invoice_number` inchangé |
| `routes/reconciliation.rs:5681,5724` + `reconciliation_cancel.rs:332,387` | ajouter `internal_number` |
| `mcp/outils/ecritures.rs:310,315` | ajouter `numero_interne_facture_fournisseur` |

### Serveur : les libellés rendus dans un refus ou un contrôle

Forme courte, résolue par une seule expression SQL ou fonction (DRY) ; un champ générique `document_label` ne change pas de nom.

| Site | Message |
|---|---|
| `supplier_invoices.rs:956-959` | refus de dévalidation |
| `journal_entries.rs:1116,2293` | écriture non modifiable, règlement détaché, `ModificationGuard::label` (`errors.rs:150,190`) |
| `letterings.rs:1882,1983` | pièce d'un lettrage refusé |
| `payment_batches.rs:596-614` + `errors.rs:1529` + `kesh-api/errors.rs:4814` | refus de confirmation d'un lot |
| `closing_checks.rs:499` (K8) | panneau de clôture ; brouillon numéroté : `FF-… — Fournisseur` |

### Ce qui garde le numéro du fournisseur, délibérément

| Site | Motif |
|---|---|
| `kesh-reconciliation/src/matching.rs:99` | compare au relevé bancaire : un numéro interne n'y figure jamais (`C-e331-21`) |
| `payment_batches.rs:850-879` (pain.001) | n'emploie ni l'un ni l'autre : la référence de paiement. Inchangé, vérifié |
| `SupplierInvoiceForm.svelte:178,391`, `supplier-invoice-form.ts:37` | saisie du numéro du fournisseur |
| `exports/csv_tables.rs`, `mcp/outils/factures_fournisseur.rs`, `kesh-seed/pieges.rs:105,591` | 33-1a, ou donnée de démonstration |

### Dérogation à la règle de splitting

Plus de cinq modules : `frontend` supplier-invoices, payment-batches, journal-entries, reconciliation, settings ; `kesh-api` routes ; `kesh-db`
cinq repositories. **Dérogation assumée** (`C-e331-18`) : un seul patron, deux formes de libellé, déployé mécaniquement, ce que la règle appelle une
sous-story de déploiement, à revoir fichier par fichier. Un écart de patron découvert en route se signale au lieu de s'absorber.

## Acceptance Criteria

**AC1 — Le libellé, une seule définition.** `frontend/src/lib/features/supplier-invoices/supplier-invoice-label.ts` : `supplierInvoiceShortLabel` et
`supplierInvoiceLongLabel`, testées (Vitest) : les deux numéros ; l'interne seul ; le fournisseur seul ; ni l'un ni l'autre ; **numéro blanc = absent**
(règle de K8, `closing_checks.rs:499`). **Plus aucun** `supplierInvoiceNumber ?? ` dans `frontend/src` hors de ce fichier, test `git grep` à l'appui.

**AC2 — La colonne.** Liste : `supplier-invoices-col-internal-number` (colonne, `font-mono`), avant **N° fournisseur** ; un brouillon jamais validé rend
`—`, un brouillon numéroté rend son numéro.

**AC3 — Le tri.** `GET /supplier-invoices?sort=internalNumber&order=asc|desc`. `sort` absent : l'ordre d'aujourd'hui (`invoice_date DESC, id DESC`), à la
lettre. Tri **naturel** par `NATURAL_SORT_KEY(internal_number)` (MariaDB ≥ 10.7, F16 de la 33-1a), `FF-2026-999` avant `FF-2026-1000` ; sans numéro à la fin
dans les deux sens ; `id` pour départager. Valeur hors liste : 400 `Validation`, jamais d'interpolation (`C-e331-19`). À l'écran, en-tête cliquable,
`aria-sort`, adresse source de vérité (`?sort=`, `?order=`), comme `status` et `offset` (`+page.svelte:22-29`) ; changer le tri ramène `offset` à 0.

**AC4 — La recherche.** `GET /supplier-invoices?q=…` : trouve sur le numéro interne, celui du fournisseur **et** le nom du fournisseur, par `LIKE … ESCAPE`
(patron `contacts.rs:207-211`), `%` et `_` échappés, scopée société, cumulable avec `status`, `total` comptant le filtre. `q` vide ou blanc : aucun
filtre. Longueur au-delà de 128 : 400. À l'écran, champ `supplier-invoices-search`, `?q=` dans l'adresse, frappe temporisée, `offset` à 0, le jeton de
dernière lecture existant (`+page.svelte`, revue P1 B-4) couvre la course ; état vide distinct : « Aucune facture ne correspond à … ». Ni `sort` ni `q` au
service MCP (`services/factures_fournisseur.rs:119`), qui garde son comportement : paramètres par une structure de filtre à défaut neutre.

**AC5 — Multi-société.** `q` et `sort` ne franchissent pas la société : test `idor` (patron `idor_multi_tenant_e2e.rs`).

**AC6 — La fiche.** Titre en forme courte ; `<dl>` : **N° interne** (`supplier-invoice-detail-internal-number`), **N° fournisseur**
(`supplier-invoice-detail-supplier-number`). Brouillon jamais validé : « attribué à la validation ». **Brouillon numéroté** : le numéro, et « conservé si la
facture est revalidée ; perdu si elle est supprimée ». Le dialogue de suppression d'un brouillon numéroté avertit que le numéro ne sera pas réattribué.
Après validation, le numéro paraît **sans recharger** (réponse de la route).

**AC7 — Le réglage.** *Administration → Facturation* (`settings/invoicing/+page.svelte`, section *Numérotation*, `:288`) : champ **Format des factures
fournisseur** (`settings-supplier-internal-number-format`), aide, aperçu, refus sur place. Le champ part dans le corps ; chargé et relu après conflit de
version (`:159,:235`), comme `format`. Aide : « S'applique aux numéros à venir : ceux déjà attribués ne changent pas. » `validateFormatTemplate` reçoit une option
`requireSeq`, ou une variante, alignée sur `validate_numbering_template` : `invoices-format-error-no-seq`. Le champ client, lui, ne change pas de règle.
Type `InvoiceSettings` (`invoices.types.ts`), `settings-invoicing-page.test.ts` suit.

**AC8 — Les sites.** Chaque ligne de l'inventaire est appliquée, et un test le prouve au niveau où il vit : Vitest pour le libellé rendu, `*_e2e` pour les champs
ajoutés et pour les refus (dévalidation refusée d'une facture numérotée : le message porte `FF-…` ; K8 : `closing_checks`). Les champs existants ne changent ni
de nom ni de valeur.

**AC9 — i18n.** Clés neuves et valeur changée de `supplier-invoices-col-number` (« N° fournisseur ») aux **quatre** locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH`),
fallbacks Svelte alignés, `npm run lint-i18n-ownership` vert. Recompté par `grep -c` aux quatre fichiers.

**AC10 — E2E.** `frontend/tests/e2e/supplier-internal-number.spec.ts`, assertions **relatives** (la base partagée a un compteur qui ne redescend pas) :
brouillon sans numéro → validé, numéro au motif `FF-AAAA-NNN` → présent à la liste → trouvé par `q` (numéro interne ; numéro du fournisseur) → tri croissant et
décroissant, vérifiés sur l'ordre des lignes → repassé en brouillon : numéro conservé → revalidé : **le même** → réglage modifié (`KF-{YEAR}-{SEQ:04}`) → la facture
suivante le porte, la première non → **format restauré en fin de spec** → format `FF-{YEAR}` refusé sur place. `supplier-documents.spec.ts:476` suit (lien :
numéro interne). Les quatre specs qui lisent `supplier-invoice-number` restent vertes sans changer de `testid`.

**AC11 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e331b/captures/`) : la liste (les deux colonnes), triée, filtrée, sans résultat ; la fiche
validée, en brouillon jamais validé, en brouillon numéroté, le dialogue de suppression ; le réglage (valide, refus) ; le choix d'un lot ; le lien d'écriture ; la candidate
et l'avis de rapprochement. Jointes au Dev Agent Record et au prompt de revue.

**AC12 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, `.tex` et PDF aplati contrôlé) :

- § `sec:factures-fournisseurs` : `\subsection{Le numéro interne}\label{sec:numero-interne-fournisseur}` : à quoi il sert, quand il naît, format par défaut, et que le numéro
  du fournisseur reste facultatif et cherchable.
- **Conduite à tenir pour le classement papier** : écrire le numéro sur le papier **après** la validation, jamais avant ; le relevé de la liste, triée par numéro interne, sert
  à annoter d'un coup les **factures déjà saisies** (numérotées par date à la mise à jour) ; repassée en brouillon et revalidée, une facture **garde** le numéro, rien à
  réécrire ; brouillon numéroté supprimé : **le numéro n'est jamais réattribué**, annoter le papier « annulée » et laisser le trou ; date déplacée vers un autre exercice : le
  numéro ne change pas, même si son année n'est plus celle de l'exercice ; format changé : les anciens numéros restent.
- Exercices particuliers : année de début d'exercice dans `{YEAR}`, `{FY}` pour le nom ; deux exercices commençant la même année se suivent au lieu de repartir à 1.
- § `sec:brouillon-fournisseur` (`:1685`) et `sec:repasser-brouillon` : le sort du numéro. `:2216` : le lien dit le numéro interne.
- **Libellés du journal** : `Facture fournisseur FF-… (n° fournisseur) - contact`, les écritures d'avant la mise à jour gardant le leur.
- **`:1141-1146`, défaut voisin, corrigé ici** : « `F-AAAA-NNNN` », « `INV-AAAAMM-NNN` » et « `F{company_id}-AAAA-NNNN` » n'existent pas, la grammaire est `{YEAR}`, `{FY}`, `{SEQ}`,
  `{SEQ:NN}` (`invoice_format.rs:9-12`) ; le mois et `company_id` y sont des placeholders inconnus.

**Manuel d'administration** : renvoi au réglage depuis `sec:maj-0-17-5`. Manuels DE, EN, IT : dossiers sans contenu traduit (`docs/manual/{de,en}/README.md`), rien à faire.

**AC13 — CHANGELOG et API.** Section `[0.17.5]` : l'entrée de la 33-1a complétée (écrans, recherche, réglage). `docs/api-external.md` : `q`, `sort`, `order` ; les champs ajoutés à
`:636,:664,:863`. README « Fonctionnalités » relu.

## Tests qui rougissent avant le correctif

AC1 (fichier absent), AC3 et AC4 (`sort`, `q` ignorés : l'ordre et le total ne changent pas), AC6, AC7 (`FF-{YEAR}` accepté à l'écran), AC8 (le message porte le numéro du fournisseur). Notés au Dev Agent Record.

## Mutations nommées

| # | Mutation | Tuée par |
|---|---|---|
| M1 | Forme courte qui préfère le fournisseur | AC1 |
| M2 | Blanc non traité comme absent | AC1 |
| M3 | Tri textuel au lieu de naturel | AC3 (`999` / `1000`) |
| M4 | `q` sans `ESCAPE` | AC4 (`%`) |
| M5 | `q` sans le numéro du fournisseur | AC4 |
| M6 | `total` sans le filtre `q` | AC4 |
| M7 | Recherche sans `company_id` | AC5 |
| M8 | `sort` inconnu accepté | AC3 |
| M9 | Tri qui ne remet pas `offset` à 0 | AC3 |
| M10 | Réglage non envoyé au `PUT` | AC7, AC10 |
| M11 | `requireSeq` retiré | AC7, AC10 |
| M12 | Une ligne de l'inventaire serveur rendue à l'ancien numéro (une par ligne du tableau des refus) | AC8 |

## Tâches

1. **Étape 0** : rejouer le `git grep` et confronter à l'inventaire ; vérifier que la 33-1a est bien sur la branche.
2. Tests rouges.
3. `list` à filtre ; `q`, `sort`, `order` ; sites serveur.
4. Libellé ; écrans ; réglage ; i18n.
5. Mutations.
6. Gates, E2E complet, captures.
7. Manuels, PDF, CHANGELOG, API.

## Gates

⛔ **Gate complet en cours de boucle** dès qu'un repository est touché (exception `kesh-db`). Commandes lourdes : `flock /tmp/station-heavy.lock flock
/home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh <commande>`, une par prise, `free -g` avant. Backend : `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `scripts/test-fast.sh --ci`. Frontend : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`. **E2E complet au dernier commit de code**, 7 à
10 échecs attendus, jugés fichier par fichier. `make fr` sous verrou, PDF aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) et grepés. Aucune migration : `ls
crates/kesh-db/migrations/*.sql | wc -l` vaut 90 avant et après.

## Revue

Le prompt nomme : le **manuel utilisateur** contre le code et le PDF ; l'**inventaire** (rejouer le `git grep`, y compter les sites non résolus) ; les captures ; la multi-société de `q`.
Axes exercés et non exercés exigés au rapport ; `scripts/prepare-release.sh` interdit.

## Hors périmètre

Recherche par numéro interne au journal (le libellé de l'écriture la permet déjà) ; recherche MCP ; numéro imprimé sur un PDF. Si la revue en demande un, c'est une issue.

## Dev Agent Record

*À compléter.*

## Change Log

- 2026-11-06 : fiche créée, `ready-for-dev`.
