# Epic 48 — Corrections de la 0.20.0

*Kickoff du 2026-11-18, tenu en autonomie par l'orchestrateur (Opus 5.5), worktree `kesh-k0200`, branche
`chore/kickoff-0.20.0` (partie de `chore/apres-release-0.19.0`, `00eb6e3c1`). Code lu à `main` = tag **v0.19.0**
(`773df80da`). Release **0.20.0** : publication le **vendredi 2026-11-27**, coupure le **jeudi 2026-11-26 au soir**
(plan : `release-0.20.0.md`). Décisions de Guy du 2026-11-18 (page « kickoff-0.20.0 » et conversation), kesh:D11, kesh:D13,
kesh:D14. Révisé le même jour sur les réponses de Guy aux points ouverts du kickoff (#1234 retirée, ordre de fusion de
la 48-13, issues de la solde au milestone) et les règlements de l'orchestrateur ; puis sur le complément de Guy du même
jour : des factures réelles existent déjà en production ; puis, le même jour encore, sur la découpe de la 48-13 à sa
validation P2 : la **48-15** en est née, et l'épique passe à **huit** stories (décisions de Guy du 2026-11-18). Révisé le
2026-11-19 : la 48-12 se découpe en **48-12a** (serveur) et **48-12b** (écran), qui sortent ensemble ; #984 et #986 passent à
la 0.21.0 ; la story **v0200-corrections-mineures** entre dans l'épique, qui passe à **neuf** stories (décisions de Guy du
2026-11-19).*

> **Factures réelles.** Guy a déjà émis de vraies factures client depuis Kesh (production, remises sur papier à un
> locataire). Les gestes de la **48-1**, de la **48-2** (réimputation) et de la **48-13** s'appliquent à des factures et
> écritures réelles : **leurs fiches citent ce cas dans leurs critères d'acceptation** (une facture validée, payée ou non,
> garde ses montants, sa TVA, son numéro et son PDF archivé ; seul ce que le geste vise change).

## Objet

L'Epic 48 était le **réservoir** des stories non faites des Epics 36 et 42 à 44 (kesh:D11). Au kickoff de la 0.20.0, elle
devient **l'épique corrective de la release** : la 0.20.0 « s'attaque principalement aux bugs, avec une ou deux
fonctionnalités en plus » (kesh:D14). Elle porte neuf stories : la fin de la réimputation analytique (#1167), cinq
stories de défauts groupés par chemin, la story des corrections mineures et la story de solde. La 48-12, découpée en
48-12a et 48-12b qui sortent ensemble, compte pour une.

Elle ne porte **aucune migration** : la seule migration de la 0.20.0 est celle de l'Epic 49 (49-1).

## Dérogation à la borne de six stories

Le `CLAUDE.md` (§ *Numérotation des versions*) borne une épique de correction à **six stories, story de solde comprise**.
L'Epic 48 en compte **huit**. **Décisions de Guy du 2026-11-18** : sept au kickoff, au motif que la 0.20.0 dispose de
près de dix jours de travail, deux de plus qu'une semaine ordinaire (« il y a deux jours de plus ») ; **huit** le même
jour, par le même motif et par l'élargissement que Guy a voulu pour faire baisser les bugs — les troncatures silencieuses
#1331 à #1334, ajoutées à la release, qui ont fait découper la 48-13 en 48-13 et 48-15. **Neuf** le 2026-11-19 (décision de
Guy du 2026-11-19) : Guy a ajouté à la release six issues mineures (#1336, #1342, #1343, #1348, #1349, #1375), portées par
la story v0200-corrections-mineures. La dérogation vaut pour cette épique seulement ; elle ne modifie pas la règle.

La règle de cadence en reste la garde : la date ne bouge pas (kesh:D7, kesh:D11). Une story non close à la coupure sort
vers une épique ultérieure, dans l'**ordre de sortie** ci-dessous.

## Stories

| Clé | Objet | Issues | Prio. | Migration | État au kickoff |
|---|---|---|---|---|---|
| 48-1-reimputation-facture-fournisseur | Changer le compte analytique d'une facture fournisseur, même payée | #1167 (refs) | P3 (CR) | non | fiche `36-16b-reimputation-facture-fournisseur.md`, remédiée P1 sur `story/42-2-reimputation-facture-fournisseur` (`04dedba10`) |
| 48-2-reimputation-ecriture-manuelle | Idem, écriture manuelle ; **#1167 se ferme avec elle** | #1167 | P3 (CR) | non | fiche `36-16c-reimputation-ecriture-manuelle.md` |
| 48-10-montants-ambigus-reglages-filtres | « 1.250 » refusé au prix d'article, aux frais de rappel, au montant minimal, aux filtres des écritures | #1173, #1175, #1176, #1177 | P3, P4 | non | fiche `36-18b-montants-ambigus-reglages-filtres.md` (`backlog`, à valider) |
| 48-11-arrondi-au-centime | Les montants à deux décimales sont arrondis, non tronqués | #1245 | **P2** | non | à écrire |
| 48-12a-doublons-fournisseurs-serveur | Le serveur refuse une facture fournisseur en double (création, modification, complétion, dévalidation, API) | #975, #985 | **P2** | non | découpée de la 48-12 le 2026-11-19 ; validation terminée (P5) |
| 48-12b-doublons-fournisseurs-ecran | L'écran montre le refus, le doublon probable et l'issue inconnue sans renvoi muet | #975, #985 | **P2** | non | découpée de la 48-12 le 2026-11-19 ; en validation ; sort avec la 48-12a |
| v0200-corrections-mineures | Clés d'erreur absentes des catalogues ; contacts (e-mail et téléphone coupés, pays seul jeté, bouton grisé sans motif, noms invisibles) ; doc-comment de `unknown-outcome.ts` | #1336, #1342, #1343, #1348, #1349, #1375 | P3, P4 | non | ajoutée le 2026-11-19 ; validation terminée (P4), en développement ; garde générale découpée vers #1406 (v0.21.0) |
| 48-13-avant-la-vraie-facture | Le chemin de la première facture client QR | #1233, #1243, #1226 | P2, P3 | non | fiche `48-13-avant-la-vraie-facture.md`, en validation (P3) |
| 48-15-troncatures-silencieuses | Un texte trop long est refusé en le disant, non coupé en silence (adresse du client, pays du fournisseur, e-mail de la société, prénom et nom ; nom d'article, numéro de client, conditions de paiement) | #1331, #1332, #1333, #1334 | P3, P4 | non | fiche `48-15-troncatures-silencieuses.md` (découpée de la 48-13), en validation (P3) |
| 48-14-solde-dette-epic | Story de solde : dette des revues de la 0.19.0 (règlement groupé) | #1263, #1262, #1261, #1255 (**plafond : quatre**) | P3, P4 | non | à écrire |

Les clés 48-3 à 48-9 du réservoir restent `backlog` (48-5 garde `ready-for-dev`), **hors 0.20.0, pour une release
ultérieure**.

### 48-1 — Réimputation d'une facture fournisseur (#1167)

Le geste « Changer le compte analytique » de la 42-1 (facture client, publiée en v0.19.0), étendu à la facture
fournisseur : la facture et toutes les lignes des écritures qui en découlent (achat, règlements, groupés compris depuis la
0.19.0), tracé à l'audit, permis en période close avec avertissement. Sortie de la 0.19.0 parce qu'elle dépendait de la
colonne `settlement_journal_entry_line_id` de l'Epic 43 : celle-ci est publiée, la dépendance est levée. **Étape 0 de la
fiche** : relire la fiche remédiée à `04dedba10` contre `main` = v0.19.0 (code de l'Epic 43 fusionné depuis), et reprendre
la validation là où elle s'est arrêtée.

### 48-2 — Réimputation d'une écriture manuelle (#1167)

Le même geste sur une écriture manuelle passée : changer le compte analytique de ses lignes sans toucher au reste ; une
écriture contre-passée et sa contre-passation se réimputent ensemble. **#1167 se ferme avec elle** (mot-clé de fermeture
au corps de la PR seulement).

### 48-10 — Montants ambigus : réglages et filtres (#1173, #1175, #1176, #1177)

La règle de la 36-3 et de la 36-18a — « 1.250 » refusé comme ambigu, « 1'250 » accepté, par le prédicat partagé
`frontend/src/lib/shared/utils/amount-ambiguity.ts` — s'étend au prix d'un article (`product-helpers.ts`, `PRICE_RE`), aux
frais de rappel (`settings/dunning/+page.svelte`), au montant minimal de facture (`settings/invoicing/+page.svelte`) et
aux filtres « Montant min / max » de la liste des écritures (`journal-entries/+page.svelte`). Fiche écrite le 2026-11-16
(quatre modules, sous la borne du splitting) ; **à revalider** contre v0.19.0 avant le développement.

### 48-11 — Arrondi au centime (#1245, P2)

`format!("{:.2}", d)` sur un `Decimal` **tronque** (`rust_decimal` 1.43, `to_str_internal`) : 10.0050 sort `10.00`. Remède
décidé : **une fonction commune** « arrondir au centime (`MidpointAwayFromZero`, comme `amount_due_to_centime`) puis
formater », employée par tous les sites ; **un test par site** sur 10.0050 et 5.0060 ; une garde qui interdit `:.2}` sur
un `Decimal` non arrondi.

Inventaire au kickoff (`grep -rnE ':\.2\}' crates`, hors tests), à reprendre **par le symptôme** à la fiche :
- à corriger (issue) : `kesh-api/src/exports/csv_tables.rs:109` (`fmt_decimal`, 29 appels), `kesh-report/src/csv.rs:51`
  (`format_amount_iso`), `kesh-report/src/pdf.rs:276` (`format_swiss_amount`, dont le commentaire « gère signe et
  arrondi » est faux), `kesh-api/src/routes/bank_statements_manual.rs:555` (`amount_string`, à juger) ;
- **non nommés par l'issue, à trier par la fiche** : `kesh-report/src/pdf.rs:1188` (taux `{rate:.2} %`),
  `kesh-report/src/pdf.rs:1459` et `kesh-report/src/csv.rs:741` (pourcentages) — taux et pourcentages, non des montants,
  mais le même troncage ;
- sains (arrondissent avant) : `kesh-qrbill/src/generator.rs:115`, `kesh-payment/src/pain001/mod.rs:125`,
  `kesh-i18n/src/formatting.rs:22`, `kesh-api/src/routes/company_invoice_settings.rs:47`.

La fonction commune a sa place naturelle dans `kesh-core` (`types/money.rs` porte déjà `round_to_centimes`,
`MidpointAwayFromZero`), dont dépendent `kesh-report` et `kesh-api` ; la fiche tranche.

**Part de `main` après la fusion de la PR #1327 de Beat**, qui récrit la mise en page de `crates/kesh-report/src/pdf.rs`
(décision de Guy du 2026-11-18). Tant que #1327 n'est pas fusionnée, la fiche peut s'écrire, non le développement.

### 48-12 — Réponses perdues : factures fournisseur et comptes bancaires (#975, #984, #985, #986, P2)

Quatre écrans où une réponse perdue (délai, coupure) ou un 409 mène à un doublon, à un écrasement non montré, ou à un échec
annoncé pour une écriture réussie :
- #975 : `POST /supplier-invoices` (page `supplier-invoices/new`) sans version ni clé d'idempotence → second brouillon ;
- #985 : complétion d'un document importé (`supplier-invoices/new`) → `409 IMPORT_NOT_PENDING_COMPLETION` affiché en
  échec alors que la facture est créée ;
- #986 : édition (`supplier-invoices/[id]/edit`) → 409 après une modification enregistrée, annoncé en échec, sans relecture ;
- #984 : comptes bancaires (`bank-accounts/+page.svelte`) → après un 409, un second clic écrase la modification
  concurrente sans la montrer (même symptôme que #953, traité pour la société par la 34-6).

Le motif attendu : relire et montrer ce qui a été écrit avant de proposer de réessayer ou de remplacer — comme la 34-6
pour la société. Pour #975, la fiche choisit entre relecture et clé d'idempotence côté serveur (celle-ci toucherait
`kesh-api/src/routes/supplier_invoices.rs` et `kesh-db/src/repositories/supplier_invoices.rs`).

**Révision du 2026-11-19 (décisions de Guy).** « Il ne devrait simplement pas être possible de créer une facture
fournisseur en double » : le serveur refuse le doublon. Même fournisseur et même numéro dans la même année civile → refus
absolu ; même fournisseur, même date et même total TTC → refus par défaut, levé par `confirmNotDuplicate`, audité ;
annulées exclues ; numéro facultatif ; dévalidation d'une annulée contrôlée, confirmable. **#984 et #986 passent à la
0.21.0.** La story se découpe (règle de splitting) en **48-12a** (serveur : `kesh-db`, `kesh-api`, rejeu de `POST` et
`PUT /invoices` après interblocage) et **48-12b** (écran : table d'états, issue inconnue figée, sortie « J'ai vérifié, elle
n'existe pas — reprendre la saisie ») ; elles sortent dans la même release, ou toutes deux à la suivante. La fiche
`48-12-reponses-perdues-fournisseurs.md` n'est plus qu'un renvoi.

### v0200-corrections-mineures (#1336, #1342, #1343, #1348, #1349, #1375)

Ajoutée par Guy le 2026-11-19. Les 21 clés d'erreur appelées par le serveur et absentes des quatre catalogues (#1348, sa
part « clés ») ; e-mail et téléphone d'un contact coupés par `maxlength` (#1336) ; pays saisi seul jeté (#1342) ; bouton
d'enregistrement grisé sans motif visible (#1349) ; prénom ou nom fait de caractères invisibles accepté (#1343, élargi aux
contacts et interlocuteurs) ; doc-comment de `unknown-outcome.ts` (#1375). Cinq modules. La garde générale des clés
demandée par #1348 est **découpée vers #1406 (v0.21.0)** après trois passes de validation qui ont chacune rouvert
l'inventaire (décision de Guy du 2026-11-19).

### 48-13 — Avant la vraie facture (#1233, #1243, #1226)

Les défauts restés sur le chemin d'une première facture client QR, que Guy émettra avant la fin novembre :
- #1233 (P3) : `InvoiceForm.svelte:376-386` — la réponse tardive de `getContact` écrase un client choisi entre-temps ;
- #1243 (P3) : `settings/+page.svelte` (adresse de la société, ~l. 820-840) — `maxlength` tronque en silence ; refus
  explicite, comme le nom depuis #1097 ;
- #1226 (P2) : `kesh-api/src/routes/onboarding.rs:535` — l'assistant ne contrôle le nom de la société que non vide ;
  mêmes contrôles que la route de modification du nom (70 caractères, jeu SIX).

**#1234** (`billing_information` jamais imprimé) est **retirée de la 48-13 et de la 0.20.0** (décision de Guy du
2026-11-18) et renvoyée au milestone « Plus tard » : elle reste latente tant que Kesh ne renseigne pas ce champ. La 48-13
ne touche donc plus `kesh-qrbill`.

**Ordre de fusion** (arbitrage de Guy du 2026-11-18) : la 48-13 **se fusionne avant** les stories de l'Epic 49 qui
touchent `InvoiceForm.svelte`, et avant la 48-15 ; elle est petite et sur le chemin de la première facture de fin
novembre.

### 48-15 — Troncatures silencieuses (#1331, #1332, #1333, #1334)

**Née de la 48-13 à sa validation P2** (2026-11-18) : Guy avait élargi la 48-13 à ces sites « pour diminuer le nombre de
bugs » ; la passe P2 y a compté neuf modules ; le signal de splitting a été déclaré et la story coupée. Un attribut
`maxlength` coupe en silence un texte collé, la valeur coupée s'enregistre et part sur la QR-facture ou le PDF. Le geste,
répété site par site : vérifier la borne du serveur, l'**ajouter** si elle manque, puis retirer le `maxlength`.
- #1331 (P3) : adresse d'un client (`contacts/+page.svelte`, cinq champs) ;
- #1332 (P3) : pays d'un fournisseur neuf, coupé à deux caractères (`SupplierCreatePanel.svelte`) ;
- #1333 (P3) : e-mail de la société sans borne serveur (`companies.rs`) ;
- #1334 (P4) : prénom ou nom seul non bornés (`contacts.rs`, et `onboarding.rs::set_coordinates`) ;
- et cinq champs qui finissent sur le PDF : **nom d'article** (`products/+page.svelte`), **numéro de client**,
  **conditions de paiement**, **raison sociale** et **délai de paiement** (`contacts/+page.svelte`) — les deux derniers
  ajoutés à la validation de la fiche (2026-11-18).

**Dérogation à la règle de splitting** (décision de Guy du 2026-11-18) : sept modules au grain du domaine, mais un seul
geste mécanique répété, rangé en **deux lots ordonnés** (A serveur, puis B écrans) ; écrite dans la fiche. **Dépend de la
48-13 fusionnée** (toutes deux touchent `onboarding.rs::set_coordinates` et `settings/+page.svelte`) ; **ne bloque pas
l'Epic 49** (aucun fichier de l'éditeur de lignes). Sites voisins hors périmètre, une issue par famille, sans milestone :
#1335 à #1340 (et #1005, existante).

### 48-14 — Story de solde (plafond : quatre issues)

Story de solde de l'épique (`CLAUDE.md`, § *Priorités des défauts*), décidée au kickoff : elle reprend la dette des revues
de la 0.19.0 dont les stories de solde 42-4, 43-4 et 44-2 sont devenues sans objet (kesh:D11) — le règlement groupé,
#1255 à #1265. **Plafond : quatre issues**, fixé au kickoff. Les quatre retenues (les bugs d'abord, kesh:D14) sont
**au milestone v0.20.0** depuis le 2026-11-18 (décision de Guy) :

| Issue | Label | Objet | Fichiers |
|---|---|---|---|
| #1263 | bug, P3 | « Choisissez le compte de l'écart » sans sélecteur après une relecture bloquée | `features/reconciliation/GroupSettlementModal.svelte` |
| #1262 | bug, P4 | liste non relue si la modale est rouverte sur une autre transaction pendant l'envoi | idem |
| #1261 | known-failure, P3 | [KF-063] `disque_sature_pendant_l_ecriture` variable sous `-j1` | `crates/kesh-api/tests/admin_auto_backup_e2e.rs` |
| #1255 | technical-debt, P3 | dé-rapprochement commun aux bras client et fournisseur | `kesh-db/src/repositories/grouped_settlements.rs` et voisins |

Les autres (#1256 à #1260, #1264, #1265 ; P4) restent ouvertes, hors de la release.

## Ordre de lancement et ordre de sortie

**Lancement** (voie B, plan § *Voies*) : 48-13 et 48-11 d'abord (prioritaires ; 48-11 dès la fusion de #1327), la 48-15
dès la 48-13 fusionnée, puis 48-12, 48-10, 48-1, 48-2, et la 48-14 en dernier.

**Sortie à la coupure, si le temps manque** (décisions de Guy du 2026-11-18) : la **48-14** sort la première, puis la
**48-2**, puis selon l'avancement 48-1, 48-10, 48-12 ; la **48-15 sort avant la 48-11 et la 48-13**, prioritaires, qui
sont les dernières à sortir. Une story sortie retourne à une épique
ultérieure (kesh:D11) ; l'Epic 48 se clôt avec la release.

## Dépendances entre stories

- **48-2 après 48-1**, ou en parallèle sur des blocs distincts (les fiches 36-16b et 36-16c le prévoient) : elles partagent
  `project_reassignment.rs`, `ProjectReassignDialog.svelte`, `projects.api.ts`, `project-reassignment.spec.ts`, les
  catalogues, `api-external.md` et les manuels. Le second arrivé fait l'union.
- **48-11 après la PR #1327** (Beat), fusionnée dans `main`.
- **48-15 après la 48-13** fusionnée dans la branche d'épique : `onboarding.rs` (`set_coordinates`) et
  `settings/+page.svelte` (blocs distincts).
- **48-1 et 48-12** touchent toutes deux l'espace `supplier-invoices` (48-1 : `[id]/+page.svelte` et
  `routes/supplier_invoices.rs` ; 48-12 : `new/`, `[id]/edit/`, peut-être les mêmes route et repository) : fichiers
  distincts côté écran, possiblement communs côté serveur ; le second arrivé résout.
- **48-10 et 48-2** touchent l'espace `journal-entries` sur deux pages distinctes (liste / détail) : pas de conflit attendu.
- **48-13 avant l'Epic 49** : elles partagent `frontend/src/lib/components/invoices/InvoiceForm.svelte` (#1233 / lignes
  de la facture) ; la 48-13 se fusionne d'abord, les stories de l'Epic 49 partent ensuite de la branche à jour (arbitrage
  de Guy du 2026-11-18).
- **48-14** est indépendante (réconciliation).

## Fichiers et modules touchés (déclarés au kickoff)

Lu au code à `773df80da`. « Probable » : la fiche confirme.

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 48-1 | `kesh-db/src/repositories/project_reassignment.rs`, `supplier_invoices.rs` ; `kesh-api/src/routes/supplier_invoices.rs`, `audit_labels.rs`, `errors.rs` | `features/projects/` (appel d'API), `routes/(app)/supplier-invoices/[id]/+page.svelte` | `kesh-i18n` (4 locales), `docs/api-external.md`, manuels (`user-manual.tex`, `admin-manual.tex`), `project-reassignment.spec.ts`, `audit_route_registry.rs`, `services_motif.rs` |
| 48-2 | `kesh-db/src/repositories/project_reassignment.rs` ; `kesh-api/src/routes/journal_entries.rs`, `errors.rs` | `features/projects/`, `routes/(app)/journal-entries/[id]/` | idem 48-1 |
| 48-10 | — | `features/products/product-helpers.ts`, `routes/(app)/products/`, `routes/(app)/settings/dunning/+page.svelte`, `settings/invoicing/+page.svelte`, `routes/(app)/journal-entries/+page.svelte` | `kesh-i18n`, manuels, `montants-ambigus.spec.ts`, `i18n-keys.test.ts`, `dunning-page.test.ts` (neuf) |
| 48-11 | `kesh-core/src/types/money.rs` (fonction commune, probable), `kesh-report/src/pdf.rs`, `kesh-report/src/csv.rs`, `kesh-api/src/exports/csv_tables.rs`, `kesh-api/src/routes/bank_statements_manual.rs` | — | tests par site, garde ; manuel si un format visible change |
| 48-12a | `kesh-db/src/repositories/supplier_invoices.rs`, `imported_supplier_invoices.rs` ; `kesh-api/src/routes/supplier_invoices.rs`, `imported_supplier_invoices.rs`, `invoices.rs` (rejeu), `errors.rs` | — | `kesh-i18n` (trois clés `error-supplier-invoice-*`), manuels utilisateur et d'administration, `docs/api-external.md`, `audit_route_registry.rs` (fiche : cinq modules) |
| 48-12b | — | `routes/(app)/supplier-invoices/new/`, `[id]/`, `[id]/edit/`, `features/supplier-invoices/` | `kesh-i18n`, manuel utilisateur, Vitest et E2E (fiche : cinq modules) |
| v0200-corrections-mineures | `kesh-api/src/errors.rs` (un site), `routes/contacts.rs`, `contact_persons.rs`, `onboarding.rs`, `helpers.rs`, `address_input.rs` ; `kesh-i18n` (21 clés + 1, test des 21 clés) | `routes/(app)/contacts/`, `features/contacts/`, `shared/utils/unknown-outcome.ts` (doc-comment) | manuel utilisateur, Vitest, E2E `contacts.spec.ts` (fiche : cinq modules) |
| 48-13 | `kesh-api/src/routes/onboarding.rs` (et le contrôle partagé du nom de `companies.rs`) | `lib/components/invoices/InvoiceForm.svelte`, `routes/(app)/settings/+page.svelte` | `kesh-i18n`, manuels |
| 48-15 | lot A : `kesh-api/src/routes/contacts.rs`, `companies.rs`, `onboarding.rs` (`set_coordinates`), `helpers.rs` (borne partagée) | lot B : `routes/(app)/contacts/+page.svelte`, `routes/(app)/settings/+page.svelte`, `routes/(app)/products/+page.svelte`, `features/supplier-invoices/SupplierCreatePanel.svelte` | `kesh-i18n` (une clé), manuels, Vitest, E2E |
| 48-14 | `kesh-db/src/repositories/grouped_settlements.rs` (#1255) ; `kesh-api/tests/admin_auto_backup_e2e.rs` (#1261) | `features/reconciliation/GroupSettlementModal.svelte` (#1262, #1263) | `kesh-i18n` si un libellé change |

Modules de l'épique : `kesh-core` (money), `kesh-db` (repositories), `kesh-api` (routes, exports, tests), `kesh-report`
(PDF, CSV), `kesh-i18n` (catalogues), frontend (`features/projects`,
`features/products`, `features/reconciliation`, `features/supplier-invoices`, `features/bank-accounts`,
`components/invoices`, routes `settings`, `contacts`, `journal-entries`, `supplier-invoices`, `bank-accounts`, `products`),
manuels FR/DE/IT/EN (`.tex` seulement).

**Splitting préventif** (règlement de l'orchestrateur du 2026-11-18) : la règle s'applique à chaque fiche. Elle a
joué pour la 48-13, coupée à sa validation P2 (neuf modules) en 48-13 et 48-15 ; la 48-15 (sept modules, un seul geste
mécanique) porte une **dérogation** décidée par Guy le 2026-11-18. Elle a joué aussi pour la 48-12, découpée le
2026-11-19 en 48-12a et 48-12b (cinq modules chacune), et pour la story v0200-corrections-mineures, dont la garde
générale des clés est partie vers #1406 (signal de recyclage déclaré, décision de Guy du 2026-11-19).

## Fichiers communs (avec l'Epic 49, v0200-menu-sous-groupes et les PR de Beat)

| Fichier | Qui d'autre | Règle |
|---|---|---|
| `lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) ; Epic 49 (49-1, 49-2, 49-3 : éditeur de lignes) | **la 48-13 se fusionne d'abord** (arbitrage de Guy du 2026-11-18) ; les stories de l'Epic 49 qui touchent le fichier partent ensuite de la branche à jour |
| `crates/kesh-report/src/pdf.rs` | 48-11 ; PR #1327 | 48-11 part de `main` après #1327 |
| `routes/(app)/settings/+page.svelte` | 48-13 (#1243) ; 48-15 (#1333, e-mail) ; v0200-menu-sous-groupes (si le menu touche les pages de réglages) | blocs distincts ; la 48-15 après la 48-13 |
| `crates/kesh-api/src/routes/onboarding.rs` (`set_coordinates`) | 48-13 (#1226) ; 48-15 (#1334) | la 48-15 part de la 48-13 fusionnée |
| `routes/(app)/contacts/+page.svelte`, `routes/(app)/products/+page.svelte` | 48-15 ; 48-10 (`products`, prix de l'article) | blocs distincts ; le second arrivé résout |
| catalogues i18n (4 locales), `sprint-status.yaml` | toutes | chaque épique dans son bloc ; conflit résolu en gardant les deux côtés |
| `i18n-keys.test.ts` (`sitesTotal`), `audit_route_registry.rs` | 48-1, 48-2, 48-10, 48-12, 48-13, 48-15 ; Epic 49 | recomptés après fusion |
| manuels `.tex` | toutes ; v0200-menu-sous-groupes change les chemins de menu des quatre manuels | le menu fusionne en dernier ; PDF jamais commités |
| `CHANGELOG.md` | — | jamais écrit dans les stories ; assemblé à l'intégration depuis les *Notes de version* |

## Décomptes au kickoff

Recomptés le 2026-11-18 depuis GitHub (commandes du `CLAUDE.md`) : **496 issues ouvertes**, **247 bugs** (`bug` ou
`known-failure`), **14 P1** ouvertes (dont 11 portant `bug` ou `known-failure`) — identiques au tag v0.19.0 ; hors les 56
issues masquées du compte de l'agent de Beat. L'Epic 48 vise **dix-neuf** issues portant `bug` ou `known-failure`, toutes
au milestone v0.20.0 (recompté le 2026-11-18 au soir) : douze dans les stories 48-10 à 48-13 (#1245, #1173, #1175, #1176,
#1177, #975, #984, #985, #986, #1233, #1243, #1226), quatre dans la 48-15 (#1331, #1332, #1333, #1334) et trois dans la
story de solde (#1261, #1262, #1263) ; plus #1255 (`technical-debt`) et #1167 (CR). Le même soir, le stock est à **507
issues ouvertes** et **257 bugs** : les issues ouvertes par les fiches de la release (dont #1330 à #1340) le font monter.
