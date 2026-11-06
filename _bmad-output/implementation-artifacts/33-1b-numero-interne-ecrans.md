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

Établi **en partant du symptôme** : où un numéro de facture fournisseur devient-il visible, sous quelque nom que ce soit ? Un seul `git grep` du
nom de la colonne ne voit pas un numéro relu sous un autre nom (`DocumentOwner.number`, `DocumentRef.number`, `document_label`), ni un écran qui nomme la
facture par son `id`. Quatre commandes, à `796a7b8b` (code identique à `4ef19901`), en excluant les migrations, le squash et les tests :

```sh
git grep -n "supplierInvoiceNumber\|supplier_invoice_number" -- crates frontend/src   # le champ
git grep -nE "DocumentOwner|document_owners|DocumentRef|documentLabel|groupDocument" -- crates frontend/src   # le numéro sous un autre nom
git grep -n "supplierInvoiceId" -- frontend/src                                        # l'écran qui nomme par l'id
git grep -nE "numero(_[a-z]+)?:" -- crates/kesh-api/src/mcp/outils                     # le MCP
```

**Tout site de lecture** y figure avec son sort. L'étape 0 rejoue les quatre commandes : un site neuf s'ajoute au tableau, il ne s'ignore pas.

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
| `payment-batches/+page.svelte:225` (échecs de création) | `#id` | Forme courte, depuis la facture déjà chargée à l'écran (`openInvoices`), repli `#id` |
| `payment-batches/[id]/+page.svelte:138` (factures d'un lot) | `#id` | Forme courte, lien conservé : c'est l'écran où l'on retrouve les factures d'un lot payé (`C-e331-26`) |
| `open-items-labels.ts:155-158`, `documentLabel`, par `DocumentCell.svelte:20,23` (`OpenItemsTable.svelte:124`, `LetteringGroupPanel.svelte:103`, `ProposalsPanel.svelte:35`) | `number ?? « facture fournisseur »` | Numéro interne, à défaut celui du fournisseur, à défaut le texte (AC14) |
| `open-items.ts:153-154`, `groupDocument` (`LetteringGroupPanel.svelte:31,127`, `originLabel`) | `d.number`, rien sans lui | Même ordre : une facture sans numéro du fournisseur y gagne un numéro (AC14) |

`LinkSettlementModal.svelte:290` : factures **clientes** seulement (`c.invoiceId`) : aucun changement, vérifié.

### Serveur : ce que ces écrans lisent

| Site | Sort |
|---|---|
| `routes/journal_entries.rs:196` + `imported_supplier_invoices.rs:375,399` | ajouter `internal_number` |
| `routes/reconciliation.rs:133,1234` + `reconciliation.rs:847,880` (avis) | ajouter `internal_number` |
| `routes/reconciliation.rs:184,1270` (candidate) | ajouter `internal_number` (`None` pour une facture cliente) ; `invoice_number` inchangé |
| `routes/reconciliation.rs:5681,5724` + `reconciliation_cancel.rs:332,387` | ajouter `internal_number` |
| `mcp/outils/ecritures.rs:310,315` | ajouter `numero_interne_facture_fournisseur` |
| `journal_entries.rs:2293`, `document_owners`, et `DocumentOwner` (`:2202`) ; `routes/letterings.rs:331-350`, `DocumentResponse` | ajouter `internal_number` (`CAST(NULL AS CHAR)` aux autres blocs de l'`UNION`) ; `number` inchangé, `C-e331-20` |
| `mcp/outils/postes_ouverts.rs:116,163-176`, `Piece` | ajouter `numero_interne` ; `numero` inchangé |
| `routes/payment_batches.rs:34`, `PaymentBatchItemResponse`, et `:68` `from_parts` | ajouter `internalNumber` et `supplierInvoiceNumber`, par une seule lecture dédiée (`C-e331-26`) |

### Serveur : les libellés rendus dans un refus ou un contrôle

Forme courte, résolue par une seule expression SQL ou fonction (DRY) ; un champ générique `document_label` ne change pas de nom.

| Site | Message |
|---|---|
| `supplier_invoices.rs:956-959` | refus de dévalidation |
| `journal_entries.rs:1116` | règlement détaché, `ModificationGuard::label` (`errors.rs:150,190`) |
| `journal_entries.rs:2457`, `reversal_blockers`, et `letterings.rs:819` | écriture non contre-passable, ligne non lettrable à la main : tous deux lisent `owner.number`, remplacé par une méthode `DocumentOwner::display_label()` (forme courte pour une facture fournisseur, `number` tel quel pour les autres pièces) |
| `payment_batches.rs:596-614` + `errors.rs:1529` + `kesh-api/errors.rs:4814` | refus de confirmation d'un lot |
| `closing_checks.rs:499` (K8) | panneau de clôture ; brouillon numéroté : `FF-… — Fournisseur` |

Le `details.documentNumber` de ces refus (`errors.rs:1471,3925,4155`) suit le message : c'est la seule valeur d'un champ existant qui change, et son nom ne ment pas (`C-e331-20`).

### Le journal d'audit du lettrage

| Site | Sort |
|---|---|
| `letterings.rs:1882`, `SYNC_SUPPLIER_INVOICE_SQL` ; `:1983`, `DocumentRef.number` ; `:1055-1076`, `audit_details` | `documentNumber` des **nouvelles** entrées `lettering.created` et `lettering.removed` d'une facture fournisseur : **forme longue**, gabarit du libellé (`C-e331-25`, à relire par Guy). Les entrées passées ne se réécrivent pas ; le manuel le dit (AC12, AC15) |

### Ce qui garde le numéro du fournisseur, délibérément

| Site | Motif |
|---|---|
| `kesh-reconciliation/src/matching.rs:99` | compare au relevé bancaire : un numéro interne n'y figure jamais (`C-e331-21`) |
| `payment_batches.rs:850-879` (pain.001) | n'emploie ni l'un ni l'autre : la référence de paiement. Inchangé, vérifié |
| `SupplierInvoiceForm.svelte:178,391`, `supplier-invoice-form.ts:37` | saisie du numéro du fournisseur |
| `DocumentResponse.number`, `Piece.numero` | valeur de l'API : `internalNumber` s'ajoute à côté (`C-e331-20`) |
| `exports/csv_tables.rs`, `mcp/outils/factures_fournisseur.rs`, `kesh-seed/pieges.rs:105,591` | 33-1a, ou donnée de démonstration |

### Dérogation à la règle de splitting

Plus de cinq modules : `frontend` supplier-invoices, payment-batches, journal-entries, reconciliation, open-items, settings ; `kesh-api` routes et MCP ; `kesh-db`
huit repositories (`supplier_invoices`, `journal_entries`, `letterings`, `payment_batches`, `closing_checks`, `reconciliation`, `reconciliation_cancel`, `imported_supplier_invoices`). **Dérogation assumée** (`C-e331-18`) : un seul patron, deux formes de libellé, déployé mécaniquement, ce que la règle appelle une
sous-story de déploiement, à revoir fichier par fichier. Un écart de patron découvert en route se signale au lieu de s'absorber.

## Acceptance Criteria

**AC1 — Le libellé, une seule définition par langage.** `frontend/src/lib/features/supplier-invoices/supplier-invoice-label.ts` : `supplierInvoicePreferredNumber`
(numéro ou `null`, pour `documentLabel` et `groupDocument`, qui ont leur propre repli), `supplierInvoiceShortLabel` et `supplierInvoiceLongLabel`, testées (Vitest) : les deux
numéros ; l'interne seul ; le fournisseur seul ; ni l'un ni l'autre ; **numéro blanc = absent** (règle de K8, `closing_checks.rs:499`). Côté serveur,
`kesh_core` `supplier_invoice_short_label` et `supplier_invoice_long_label` (33-1a, AC15), et l'expression SQL de K8 alignée sur elles. **Plus aucun repli**
`supplierInvoiceNumber ??` vers `#id` hors de ce fichier : `git grep -nE 'supplierInvoiceNumber \?\?($| .#)' -- frontend/src` ne rend plus rien (il rend six sites à `cf880931`, et ne touche pas
`SupplierInvoiceForm.svelte:178`, gardé à dessein). `ReconciliationProposals.svelte:543`, qui lit `.invoiceNumber ??`, se contrôle à part.

**AC2 — La colonne.** Liste : `supplier-invoices-col-internal-number` (colonne, `font-mono`), avant **N° fournisseur** ; un brouillon jamais validé rend
`—`, un brouillon numéroté rend son numéro.

**AC3 — Le tri.** `GET /supplier-invoices?sort=internalNumber&order=asc|desc`. `sort` absent : l'ordre d'aujourd'hui (`invoice_date DESC, id DESC`), à la
lettre. Tri **naturel** par `NATURAL_SORT_KEY(internal_number)`, `FF-2026-999` avant `FF-2026-1000` ; sans numéro à la fin
dans les deux sens ; `id` pour départager. **Plancher : MariaDB 10.7**, version d'apparition de la fonction ; la seule version livrée, testée et en production est 10.11
(F16 de la 33-1a, `docs/ci.md:269-281`), et la promesse « compat ≥ 10.6 » se corrige à AC13 (`C-e331-19`). Valeur hors liste : 400 `Validation`, jamais d'interpolation (`C-e331-19`). À l'écran, en-tête cliquable,
`aria-sort`, adresse source de vérité (`?sort=`, `?order=`), comme `status` et `offset` (`+page.svelte:22-29`) ; changer le tri ramène `offset` à 0.

**AC4 — La recherche.** `GET /supplier-invoices?q=…` : trouve sur le numéro interne, celui du fournisseur **et** le nom du fournisseur, par `LIKE … ESCAPE`
(patron `contacts.rs:207-211`), `%` et `_` échappés, scopée société, cumulable avec `status`, `total` comptant le filtre. `q` vide ou blanc : aucun
filtre. Longueur au-delà de 128 : 400. À l'écran, champ `supplier-invoices-search`, `?q=` dans l'adresse, frappe temporisée, `offset` à 0, le jeton de
dernière lecture existant (`+page.svelte`, revue P1 B-4) couvre la course ; état vide distinct : « Aucune facture ne correspond à … ». Ni `sort` ni `q` au
service MCP (`services/factures_fournisseur.rs:119`), qui garde son comportement : paramètres par une structure de filtre à défaut neutre.

**AC5 — Multi-société.** `q` et `sort` ne franchissent pas la société : test `idor` (patron `idor_multi_tenant_e2e.rs`).

**AC6 — La fiche.** Titre en forme courte ; `<dl>` : **N° interne** (`supplier-invoice-detail-internal-number`), **N° fournisseur**
(`supplier-invoice-detail-supplier-number`). Brouillon jamais validé : « attribué à la validation ». **Brouillon numéroté** : le numéro, et « conservé si la
facture est revalidée ; perdu si elle est supprimée ». Le dialogue de suppression d'un brouillon numéroté avertit : « Le numéro sera perdu. Kesh ne le reprendra pas, sauf restauration d'une sauvegarde plus ancienne ou réinitialisation des données. »
Après validation, le numéro paraît **sans recharger** (réponse de la route).

**AC7 — Le réglage.** *Administration → Facturation* (`settings/invoicing/+page.svelte`, section *Numérotation*, `:288`) : champ **Format des factures
fournisseur** (`settings-supplier-internal-number-format`), aide, aperçu, refus sur place. Le champ part dans le corps ; chargé et relu après conflit de
version (`:159,:235`), comme `format`. Aide : « Un préfixe, `{YEAR}`, un séparateur, puis `{SEQ}` ou `{SEQ:03}`. S'applique aux numéros à venir : ceux déjà attribués ne changent pas. » `validateFormatTemplate` reçoit une option
`numbering`, alignée sur `validate_numbering_template` (33-1a, AC16, mêmes cas acceptés et refusés) : une seule clé, `invoices-format-error-numbering`, aux quatre locales. Le champ client, lui, ne change pas de règle.
Type `InvoiceSettings` (`invoices.types.ts`), `settings-invoicing-page.test.ts` suit.

**AC8 — Les sites.** Chaque ligne de l'inventaire est appliquée, et un test le prouve au niveau où il vit : Vitest pour le libellé rendu, `*_e2e` pour les champs
ajoutés et pour les refus (dévalidation refusée d'une facture numérotée : le message porte `FF-…` ; K8 : `closing_checks`). Les champs existants ne changent ni
de nom ni de valeur.

**AC9 — i18n.** Clés neuves et **deux valeurs changées**, aux **quatre** locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH`) : `supplier-invoices-col-number` (« N° fournisseur »,
fr `:2616`, de `:2583`, it `:2584`, en `:2589`) et `supplier-invoices-field-number` (« N° du fournisseur (facultatif) », fr `:2634`, de `:2601`, it `:2602`, en `:2607`). Fallbacks Svelte
alignés, `npm run lint-i18n-ownership` vert. Clés neuves recomptées par `grep -c` aux quatre fichiers, valeurs changées relues à la main : le compte ne les voit pas.

**AC10 — E2E.** `frontend/tests/e2e/supplier-internal-number.spec.ts`, assertions **relatives** (la base partagée a un compteur qui ne redescend pas) :
brouillon sans numéro → validé, numéro au motif `FF-AAAA-NNN` → présent à la liste → trouvé par `q` (numéro interne ; numéro du fournisseur) → tri croissant et
décroissant, vérifiés sur l'ordre des lignes → repassé en brouillon : numéro conservé → revalidé : **le même** → réglage modifié (`KF-{YEAR}-{SEQ:04}`) → la facture
suivante le porte, la première non → format `FF-{YEAR}` refusé sur place, `FF-{SEQ:03}` et `FF-{FY}-{SEQ}` aussi → postes ouverts du compte créanciers : la pièce dit le numéro interne → détail d'un lot : la facture dit le
numéro interne. **Format restauré par un `afterAll`**, qui survit à un échec en cours de spec. `supplier-documents.spec.ts:476` suit (lien : numéro interne). Sans changer de nom : le
`testid` `supplier-invoice-number` (`supplier-invoice-draft-cycle.spec.ts:313`, seule spec à le lire) et la propriété `supplierInvoiceNumber` envoyée à la création (`helpers/api-fixtures.ts`,
`payment-batches`, `v0174-correctifs`, `supplier-invoices`, `supplier-documents`, `supplier-invoice-draft-cycle`) : ces cinq specs restent vertes.

**AC11 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e331b/captures/`) : la liste (les deux colonnes), triée, filtrée, sans résultat ; la fiche
validée, en brouillon jamais validé, en brouillon numéroté, le dialogue de suppression ; le réglage (valide, refus) ; le choix d'un lot et le détail d'un lot ; le lien d'écriture ; la candidate
et l'avis de rapprochement ; une ligne des postes ouverts et un groupe de lettrage. Jointes au Dev Agent Record et au prompt de revue.

**AC12 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, `.tex` et PDF aplati contrôlé). **Liste close**, numéros relus à `796a7b8b` ; un passage neuf découvert au développement s'y ajoute :

- § `sec:factures-fournisseurs` : `\subsection{Le numéro interne}\label{sec:numero-interne-fournisseur}` : à quoi il sert, quand il naît, format par défaut, et que le numéro
  du fournisseur reste facultatif et cherchable.
- **Conduite à tenir pour le classement papier** : écrire le numéro sur le papier **après** la validation, jamais avant ; le relevé de la liste, triée par numéro interne, sert
  à annoter d'un coup les **factures déjà saisies** (numérotées par date à la mise à jour) ; repassée en brouillon et revalidée, une facture **garde** le numéro, rien à
  réécrire ; brouillon numéroté supprimé : Kesh ne reprend pas le numéro, annoter le papier « annulée » et laisser le trou ; **trois cas seulement font revenir un numéro** : la restauration d'une sauvegarde plus ancienne, le retour à la 0.17.4 suivi d'une nouvelle montée, et la réinitialisation des données (renvoi au manuel d'administration, `C-e331-23`) ; date déplacée vers un autre exercice : le
  numéro ne change pas, même si son année n'est plus celle de l'exercice ; format changé : les anciens numéros restent.
- Exercices particuliers : `{YEAR}` est l'année de début de l'exercice ; deux exercices commençant la même année se suivent au lieu de repartir à 1, et un numéro supprimé dans l'un n'est pas repris par l'autre. Le format s'écrit : un préfixe libre, `{YEAR}`, un séparateur sans chiffre, `{SEQ}` ou `{SEQ:NN}` ; le nom de l'exercice n'y entre pas.
- **`:1592` et suivantes** (saisie) : le numéro du fournisseur, facultatif, n'y est pas nommé : à dire. **`:1689`** (« à côté de son numéro ») : le numéro interne ; un brouillon jamais validé n'en a pas. **`:1697`** (« fournisseur, numéro, ») : « numéro du fournisseur ».
- § `sec:brouillon-fournisseur` (`:1685`) et `sec:repasser-brouillon` (`:1732`, relire `:1740` et `:1746-1747` : la « numérotation des écritures » y est nommée et reste juste, le numéro interne s'ajoute à part) : le sort du numéro. `:1945` (« le trou de numérotation ») : préciser « des écritures », le numéro interne étant, lui, conservé.
- **Clôture, `:828`** (K8, « nomme les vingt premiers brouillons ») : un brouillon numéroté s'y nomme par son numéro interne.
- **Lots, `:2000-2005`** : la sélection montre les deux numéros, le détail d'un lot le numéro interne.
- **`:2216`** : le lien dit le numéro interne.
- **Réconciliation, `:2807`** (candidate d'un débit, « son numéro ») : les deux numéros. **`:2753`** (avis de lot, « en nommant la facture et le lot ») : par son numéro interne. **`:2751`** (« ou à son numéro ») : **« au numéro du fournisseur »**, le terme devenant ambigu : le numéro interne n'entre jamais dans la note (`C-e331-21`).
- **Postes ouverts et lettrage, `:3030`** (colonne *pièce*) et **`:3075`** : la pièce d'une facture fournisseur se dit par son numéro interne. **`:1971`** (« le lien vers la facture annulée ») : relu, inchangé.
- **Journal d'audit, § `sec:tracabilite` (`:3590`) ou § `sec:lettrage`** : le `documentNumber` des lettrages d'une facture fournisseur porte le numéro interne et celui du fournisseur à partir de la 0.17.5 ; les entrées passées gardent le seul numéro du fournisseur (`C-e331-25`).
- **Libellés du journal** : `Facture fournisseur FF-… (n° fournisseur) - contact`, les écritures d'avant la mise à jour gardant le leur.
- **`:1141-1146`, défaut voisin, corrigé ici** : « `F-AAAA-NNNN` », « `INV-AAAAMM-NNN` » et « `F{company_id}-AAAA-NNNN` » n'existent pas, la grammaire est `{YEAR}`, `{FY}`, `{SEQ}`,
  `{SEQ:NN}` (`invoice_format.rs:9-12`) ; le mois et `company_id` y sont des placeholders inconnus.

**Manuel d'administration** : renvoi au réglage depuis `sec:maj-0-17-5`. Manuels DE, EN, IT : dossiers sans contenu traduit (`docs/manual/{de,en}/README.md`), rien à faire.

**AC13 — CHANGELOG, API, version de MariaDB.** Section `[0.17.5]` : l'entrée de la 33-1a complétée (écrans, recherche, réglage). `docs/api-external.md` : `q`, `sort`, `order` ; les champs ajoutés à
`:636,:664,:863` ; `internalNumber` dans `document` (`:353-354`, `:417-418`, `:425`). README « Fonctionnalités » relu. **`README.md:60`, `docs/ci.md:271,281`, le commentaire de `ci.yml:49` et `marketing-brochure.tex:345` (avec son PDF)** : « compat ≥ 10.6 » devient « ≥ 10.7 », à cause du tri ; la migration `20260513000001` et son commentaire ne se touchent pas (P8), `docs/kesh-prd-v0.2.md` et `docs/kesh-specifications.txt`, historiques, non plus. `admin-manual.tex:109` : « 10.11+ » reste juste, le membre « compatibilité MySQL 8 maintenue » se retire, `NATURAL_SORT_KEY` étant propre à MariaDB (`C-e331-19`).

**AC14 — Postes ouverts et lettrage.** `DocumentResponse` et `Piece` portent le numéro interne, `number` et `numero` gardent leur valeur (test `*_e2e` aux deux bouts, `open_items_e2e.rs:516` suit). `documentLabel` et `groupDocument` (Vitest) : numéro interne, à défaut celui du fournisseur, à défaut le repli existant ; une facture sans numéro du fournisseur a son groupe nommé. Pour une facture cliente ou un avoir, rien ne change.

**AC15 — L'audit du lettrage** (`C-e331-25`). Une facture fournisseur payée après la mise à jour écrit `documentNumber` à la forme longue (`FF-2026-001 (ABC-123)`, ou `FF-2026-001` seul) ; une entrée d'audit passée reste identique à l'octet (test sur une entrée posée avant le geste suivant, `lettering_documents.rs:626-685` et `:1635` suivent). La clé reste présente, et `null` seulement pour une facture sans aucun numéro.

**AC16 — Les lots** (`C-e331-26`). Toutes les réponses de `from_parts` (`routes/payment_batches.rs:193,237,281,293` : création, détail, confirmation, annulation) rendent les numéros de chaque facture, par une seule lecture, scopée société ; `PaymentBatchItem`, ses cinq lectures (`payment_batches.rs:58,285,564,728,959`) et l'export CSV ne changent pas. Le détail d'un lot dit la forme courte ; un échec de création de même, depuis `openInvoices`.

## Tests qui rougissent avant le correctif

AC1 (fichier absent), AC3 et AC4 (`sort`, `q` ignorés : l'ordre et le total ne changent pas), AC6, AC7 (`FF-{YEAR}` et `FF-{SEQ:03}` acceptés à l'écran), AC8 (le message porte le numéro du fournisseur), AC14, AC15, AC16. Notés au Dev Agent Record.

**Tests existants qui rougiront, à mettre à jour au même commit**, selon que la fixture passe par la validation (numérotée) ou par du SQL (`NULL`, repli sur le numéro du fournisseur) :

- refus : `letterings_e2e.rs:905-918`, `supplier_invoice_draft_e2e.rs:1259`, `journal_entry_reversal_e2e.rs:2053` ;
- forme de `document` : `open_items_e2e.rs:516` (ensemble exact des clés), les fixtures TypeScript de `open-items*.test.ts` ;
- audit : `lettering_documents.rs:666` et `:1635` et suivantes ; MCP : `mcp_outils_e2e.rs` et le test de contrat ;
- écran : `journal-entry-page.test.ts:604,616`, `supplier-documents.spec.ts:476`.

⚠️ `document_owners/reversal_blockers_frozen.rs` fige le refus sur `s.supplier_invoice_number`. Ses fixtures (`document_owners.rs:198`) insèrent en SQL, donc `internal_number` à `NULL` et le repli donne le même résultat : **il reste vert, et ne prouve rien sur le numéro interne**. Un test neuf au même fichier exerce la facture numérotée.

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
| M11 | Option `numbering` retirée, ou réduite à la présence de `{SEQ}` et `{YEAR}` | AC7, AC10 |
| M12 | Une ligne de l'inventaire serveur rendue à l'ancien numéro (une par ligne du tableau des refus) | AC8 |
| M13 | `documentLabel` qui préfère le numéro du fournisseur | AC14 |
| M14 | `groupDocument` sans numéro interne | AC14 |
| M15 | `number` de `DocumentResponse` remplacé par le numéro interne | AC14 (valeur inchangée) |
| M16 | `documentNumber` d'audit gardé au numéro du fournisseur | AC15 |
| M17 | Lecture des numéros d'un lot sans la société | AC16 |

## Tâches

1. **Étape 0** : rejouer les quatre commandes et confronter à l'inventaire ; relire les lignes d'AC12 ; vérifier que la 33-1a est bien sur la branche.
2. Tests rouges.
3. `list` à filtre ; `q`, `sort`, `order` ; sites serveur ; `DocumentOwner`, audit du lettrage, lots.
4. Libellé ; écrans, postes ouverts et lettrage compris ; réglage ; i18n.
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

Le prompt nomme : le **manuel utilisateur** contre le code et le PDF, liste d'AC12 comprise ; l'**inventaire** (rejouer les quatre commandes, y compter les sites non résolus) ; les captures ; la multi-société de `q` et des lots ; le sort de l'audit du lettrage.
Axes exercés et non exercés exigés au rapport ; `scripts/prepare-release.sh` interdit.

## Hors périmètre

Recherche par numéro interne au journal (le libellé de l'écriture la permet déjà) ; recherche MCP ; numéro imprimé sur un PDF ; réécriture des entrées d'audit passées. Si la revue en demande un, c'est une issue.

## Dev Agent Record

*À compléter.*

## Change Log

- 2026-11-06 : fiche créée, `ready-for-dev`.
- 2026-11-06, remédiation de la validation P1 (Opus 5.5, rapports `kesh-gate-logs/e33-validate-p1-e331-R.md` et `-F.md`). Findings touchant cette fiche : 2 MEDIUM et 4 LOW de la lentille F (M1, M2, L1, L2, L6, L7), 1 LOW de la lentille R (L-6), plus la propagation du M-1 de la lentille R. Prémisses relues aux lignes citées, toutes justes.
  - **M1**, remédié : inventaire refait **en partant du symptôme**, quatre commandes. **Changement de portée** : `document_owners`, `DocumentResponse`, `documentLabel`, `groupDocument`, `postes_ouverts.rs`, `reversal_blockers`, `letterings.rs:819`, l'audit du lettrage entrent à l'inventaire, avec trois AC neuves (AC14, AC15, AC16) et chaque site a son choix écrit. La fiche classait `letterings.rs:1882,1983` et `journal_entries.rs:2293` parmi les refus : corrigé, ce sont l'audit et la source de `document`. **Sur l'audit, le choix de l'orchestrateur écarte le « champ à côté » proposé** : `C-e331-25`, à relire par Guy.
  - **M2**, remédié : AC12 en liste close, y compris `:2751`, où « son numéro » devient ambigu.
  - **L1**, remédié (AC9, deux valeurs changées, lignes relues). **L2**, remédié, et son voisin `:225`, découvert au `git grep` (AC16, `C-e331-26`). **L6**, remédié : **plancher 10.7 assumé**, promesse « ≥ 10.6 » corrigée (AC3, AC13, `C-e331-19`). Non exécuté : la version d'apparition de la fonction vient de la documentation de MariaDB, et le test AC3 tourne sur 10.11. **L7**, remédié (AC10, deux listes relues). **L-6**, volet E2E, remédié : `afterAll`.
  - **Propagation du M-1** : `requireSeq` devient l'option `numbering` (AC7, M11), `FF-{SEQ:03}` refusé (AC7, AC10), AC12 *Exercices particuliers*.
  - Décomptes de cette entrée, recomptés sur cette fiche : AC 13 → 16, mutations 12 → 17, repositories touchés 5 → 8 (nommés à la dérogation).
  - Propagation : `grep` de `requireSeq`, `format sans année`, `FF-{SEQ`, `letterings.rs:1882`, `git grep`, `10.6`, `quatre specs` sur les deux fiches et le registre, exécuté après écriture ; `docs/ci.md` corrigé à `:281`, seul résidu.
- 2026-11-06, remédiation de la validation P2 (Opus 5.5, rapport `kesh-gate-logs/e33-validate-p2-e331.md`). Findings touchant cette fiche : 1 MEDIUM et 3 LOW (F-M1, R-L4, F-L1, F-L2), plus la propagation de la simplification de l'orchestrateur (`{FY}` retiré du format fournisseur, 33-1a). Prémisses relues aux lignes citées, justes.
  - **F-M1**, remédié : AC12 dit les trois cas où un numéro revient, le dialogue d'AC6 aussi, sans « jamais ». L'aide d'AC7 reste vraie sans réserve : un changement de format ne fait plus revenir de numéro.
  - **R-L4**, remédié : motif d'AC1 exécuté à `cf880931`, il rend six sites et ne touche pas `SupplierInvoiceForm.svelte:178` ; `:543` à part.
  - **F-L1**, remédié : la brochure à AC13, avec son PDF. **F-L2**, remédié : `admin-manual.tex:109` n'était pas juste, le membre MySQL 8 se retire (`ADD COLUMN IF NOT EXISTS` dans quatre fichiers de migration, et non cinq, compte `git grep -il`).
  - **Propagation** : AC7 (une seule clé, `invoices-format-error-numbering`, au lieu de deux), AC10 (`FF-{FY}-{SEQ}` refusé), AC12 *Exercices particuliers*, M11.
  - Décomptes, recomptés par `grep -c` : AC 16, mutations 17, inchangés.
  - Propagation : `grep` de `{FY}`, `no-seq`, `no-year`, `frère`, `réattribu` sur cette fiche, exécuté après écriture ; `{FY}` ne reste qu'au format client (AC12) et au refus d'AC10.
