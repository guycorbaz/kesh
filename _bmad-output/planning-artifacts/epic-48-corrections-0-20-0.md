# Epic 48 — Corrections de la 0.20.0

*Kickoff du 2026-11-18, tenu en autonomie par l'orchestrateur (Opus 5.5), worktree `kesh-k0200`, branche
`chore/kickoff-0.20.0` (partie de `chore/apres-release-0.19.0`, `00eb6e3c1`). Code lu à `main` = tag **v0.19.0**
(`773df80da`). Release **0.20.0** : publication le **vendredi 2026-11-27**, coupure le **jeudi 2026-11-26 au soir**
(plan : `release-0.20.0.md`). Décisions de Guy du 2026-11-18 (page « kickoff-0.20.0 » et conversation), kesh:D11, kesh:D13,
kesh:D14.*

## Objet

L'Epic 48 était le **réservoir** des stories non faites des Epics 36 et 42 à 44 (kesh:D11). Au kickoff de la 0.20.0, elle
devient **l'épique corrective de la release** : la 0.20.0 « s'attaque principalement aux bugs, avec une ou deux
fonctionnalités en plus » (kesh:D14). Elle porte sept stories : la fin de la réimputation analytique (#1167), quatre
stories de défauts groupés par chemin, et la story de solde.

Elle ne porte **aucune migration** : la seule migration de la 0.20.0 est celle de l'Epic 49 (49-1).

## Dérogation à la borne de six stories

Le `CLAUDE.md` (§ *Numérotation des versions*) borne une épique de correction à **six stories, story de solde comprise**.
L'Epic 48 en compte **sept**. **Décision de Guy du 2026-11-18**, au motif que la 0.20.0 dispose de près de dix jours de
travail, deux de plus qu'une semaine ordinaire (« il y a deux jours de plus »). La dérogation vaut pour cette épique
seulement ; elle ne modifie pas la règle.

La règle de cadence en reste la garde : la date ne bouge pas (kesh:D7, kesh:D11). Une story non close à la coupure sort
vers une épique ultérieure, dans l'**ordre de sortie** ci-dessous.

## Stories

| Clé | Objet | Issues | Prio. | Migration | État au kickoff |
|---|---|---|---|---|---|
| 48-1-reimputation-facture-fournisseur | Changer le compte analytique d'une facture fournisseur, même payée | #1167 (refs) | P3 (CR) | non | fiche `36-16b-reimputation-facture-fournisseur.md`, remédiée P1 sur `story/42-2-reimputation-facture-fournisseur` (`04dedba10`) |
| 48-2-reimputation-ecriture-manuelle | Idem, écriture manuelle ; **#1167 se ferme avec elle** | #1167 | P3 (CR) | non | fiche `36-16c-reimputation-ecriture-manuelle.md` |
| 48-10-montants-ambigus-reglages-filtres | « 1.250 » refusé au prix d'article, aux frais de rappel, au montant minimal, aux filtres des écritures | #1173, #1175, #1176, #1177 | P3, P4 | non | fiche `36-18b-montants-ambigus-reglages-filtres.md` (`backlog`, à valider) |
| 48-11-arrondi-au-centime | Les montants à deux décimales sont arrondis, non tronqués | #1245 | **P2** | non | à écrire |
| 48-12-reponses-perdues-fournisseurs | Plus de doublon ni d'écrasement après une réponse perdue ou un 409 | #975, #984, #985, #986 | **P2** | non | à écrire |
| 48-13-avant-la-vraie-facture | Le chemin de la première facture client QR | #1233, #1243, #1226, #1234 | P2 à P4 | non | à écrire |
| 48-14-solde-dette-epic | Story de solde : dette des revues de la 0.19.0 (règlement groupé) | parmi #1255 à #1265, **au plus quatre** | P3, P4 | non | à écrire |

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

### 48-13 — Avant la vraie facture (#1233, #1243, #1226, #1234)

Les défauts restés sur le chemin d'une première facture client QR, que Guy émettra avant la fin novembre :
- #1233 (P3) : `InvoiceForm.svelte:376-386` — la réponse tardive de `getContact` écrase un client choisi entre-temps ;
- #1243 (P3) : `settings/+page.svelte` (adresse de la société, ~l. 820-840) — `maxlength` tronque en silence ; refus
  explicite, comme le nom depuis #1097 ;
- #1226 (P2) : `kesh-api/src/routes/onboarding.rs:535` — l'assistant ne contrôle le nom de la société que non vide ;
  mêmes contrôles que la route de modification du nom (70 caractères, jeu SIX) ;
- #1234 (P4) : `billing_information` de la QR-facture jamais imprimé sur la section de paiement (`kesh-qrbill`), latent
  tant que l'API le fixe à `None`.

### 48-14 — Story de solde (plafond : quatre issues)

Story de solde de l'épique (`CLAUDE.md`, § *Priorités des défauts*), décidée au kickoff : elle reprend la dette des revues
de la 0.19.0 dont les stories de solde 42-4, 43-4 et 44-2 sont devenues sans objet (kesh:D11) — le règlement groupé,
#1255 à #1265. **Plafond : quatre issues**, fixé au kickoff. Proposition de l'orchestrateur, à arrêter à la fiche (les
bugs d'abord, kesh:D14) :

| Issue | Label | Objet | Fichiers |
|---|---|---|---|
| #1263 | bug, P3 | « Choisissez le compte de l'écart » sans sélecteur après une relecture bloquée | `features/reconciliation/GroupSettlementModal.svelte` |
| #1262 | bug, P4 | liste non relue si la modale est rouverte sur une autre transaction pendant l'envoi | idem |
| #1261 | known-failure, P3 | [KF-063] `disque_sature_pendant_l_ecriture` variable sous `-j1` | `crates/kesh-api/tests/admin_auto_backup_e2e.rs` |
| #1255 | technical-debt, P3 | dé-rapprochement commun aux bras client et fournisseur | `kesh-db/src/repositories/grouped_settlements.rs` et voisins |

Les autres (#1256 à #1260, #1264, #1265 ; P4) restent ouvertes. Aucune de ces issues n'est au milestone v0.20.0 : les
retenues y entrent par décision du Project Lead (point ouvert du plan).

## Ordre de lancement et ordre de sortie

**Lancement** (voie B, plan § *Voies*) : 48-13 et 48-11 d'abord (prioritaires ; 48-11 dès la fusion de #1327), puis 48-12,
48-10, 48-1, 48-2, et la 48-14 en dernier.

**Sortie à la coupure, si le temps manque** (décision de Guy du 2026-11-18) : la **48-14** sort la première, puis la
**48-2** ; **48-11 et 48-13 sont prioritaires** et sont les dernières à sortir. Une story sortie retourne à une épique
ultérieure (kesh:D11) ; l'Epic 48 se clôt avec la release.

## Dépendances entre stories

- **48-2 après 48-1**, ou en parallèle sur des blocs distincts (les fiches 36-16b et 36-16c le prévoient) : elles partagent
  `project_reassignment.rs`, `ProjectReassignDialog.svelte`, `projects.api.ts`, `project-reassignment.spec.ts`, les
  catalogues, `api-external.md` et les manuels. Le second arrivé fait l'union.
- **48-11 après la PR #1327** (Beat), fusionnée dans `main`.
- **48-1 et 48-12** touchent toutes deux l'espace `supplier-invoices` (48-1 : `[id]/+page.svelte` et
  `routes/supplier_invoices.rs` ; 48-12 : `new/`, `[id]/edit/`, peut-être les mêmes route et repository) : fichiers
  distincts côté écran, possiblement communs côté serveur ; le second arrivé résout.
- **48-10 et 48-2** touchent l'espace `journal-entries` sur deux pages distinctes (liste / détail) : pas de conflit attendu.
- **48-13 et l'Epic 49** partagent `frontend/src/lib/components/invoices/InvoiceForm.svelte` (#1233 / lignes de la
  facture) et `crates/kesh-qrbill` (#1234 / rendu des lignes) : voir *Fichiers communs*.
- **48-14** est indépendante (réconciliation).

## Fichiers et modules touchés (déclarés au kickoff)

Lu au code à `773df80da`. « Probable » : la fiche confirme.

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 48-1 | `kesh-db/src/repositories/project_reassignment.rs`, `supplier_invoices.rs` ; `kesh-api/src/routes/supplier_invoices.rs`, `audit_labels.rs`, `errors.rs` | `features/projects/` (appel d'API), `routes/(app)/supplier-invoices/[id]/+page.svelte` | `kesh-i18n` (4 locales), `docs/api-external.md`, manuels (`user-manual.tex`, `admin-manual.tex`), `project-reassignment.spec.ts`, `audit_route_registry.rs`, `services_motif.rs` |
| 48-2 | `kesh-db/src/repositories/project_reassignment.rs` ; `kesh-api/src/routes/journal_entries.rs`, `errors.rs` | `features/projects/`, `routes/(app)/journal-entries/[id]/` | idem 48-1 |
| 48-10 | — | `features/products/product-helpers.ts`, `routes/(app)/products/`, `routes/(app)/settings/dunning/+page.svelte`, `settings/invoicing/+page.svelte`, `routes/(app)/journal-entries/+page.svelte` | `kesh-i18n`, manuels, `montants-ambigus.spec.ts`, `i18n-keys.test.ts`, `dunning-page.test.ts` (neuf) |
| 48-11 | `kesh-core/src/types/money.rs` (fonction commune, probable), `kesh-report/src/pdf.rs`, `kesh-report/src/csv.rs`, `kesh-api/src/exports/csv_tables.rs`, `kesh-api/src/routes/bank_statements_manual.rs` | — | tests par site, garde ; manuel si un format visible change |
| 48-12 | `kesh-api/src/routes/supplier_invoices.rs` et `kesh-db/src/repositories/supplier_invoices.rs` (si clé d'idempotence), `imported_supplier_invoices` (lecture du document) | `routes/(app)/supplier-invoices/new/`, `supplier-invoices/[id]/edit/`, `routes/(app)/bank-accounts/+page.svelte`, `features/supplier-invoices/`, `features/bank-accounts/` | `kesh-i18n`, manuel utilisateur, Vitest et E2E des écrans |
| 48-13 | `kesh-api/src/routes/onboarding.rs` (et le contrôle partagé du nom de `companies.rs`) ; `kesh-qrbill/src/pdf.rs` (`billing_information`) | `lib/components/invoices/InvoiceForm.svelte`, `routes/(app)/settings/+page.svelte` | `kesh-i18n`, manuels |
| 48-14 | `kesh-db/src/repositories/grouped_settlements.rs` (#1255) ; `kesh-api/tests/admin_auto_backup_e2e.rs` (#1261) | `features/reconciliation/GroupSettlementModal.svelte` (#1262, #1263) | `kesh-i18n` si un libellé change |

Modules de l'épique : `kesh-core` (money), `kesh-db` (repositories), `kesh-api` (routes, exports, tests), `kesh-report`
(PDF, CSV), `kesh-qrbill` (section de paiement), `kesh-i18n` (catalogues), frontend (`features/projects`,
`features/products`, `features/reconciliation`, `features/supplier-invoices`, `features/bank-accounts`,
`components/invoices`, routes `settings`, `journal-entries`, `supplier-invoices`, `bank-accounts`, `products`),
manuels FR/DE/IT/EN (`.tex` seulement).

**Splitting préventif à juger à chaque fiche** : 48-12 et 48-13 réunissent chacune quatre issues sur des écrans distincts ;
si l'une dépasse cinq modules, elle se découpe (a/b) ou écrit sa dérogation.

## Fichiers communs (avec l'Epic 49, v0200-menu-sous-groupes et les PR de Beat)

| Fichier | Qui d'autre | Règle |
|---|---|---|
| `lib/components/invoices/InvoiceForm.svelte` | 48-13 (#1233) ; Epic 49 (49-1, 49-2, 49-3 : éditeur de lignes) | **contact à arbitrer avec le Project Lead** (point ouvert du plan) ; recommandation : la 48-13 passe tôt, et le second arrivé à l'intégration résout dans sa branche (une branche d'épique n'est pas une base) |
| `crates/kesh-qrbill/src/pdf.rs`, `types.rs` | 48-13 (#1234, section de paiement) ; 49-1, 49-3 (lignes de la facture) ; PR #1327 de Beat (`largeurs.rs`, `lib.rs`) | zones distinctes du même fichier ; le second arrivé résout |
| `crates/kesh-report/src/pdf.rs` | 48-11 ; PR #1327 | 48-11 part de `main` après #1327 |
| `routes/(app)/settings/+page.svelte` | 48-13 (#1243) ; v0200-menu-sous-groupes (si le menu touche les pages de réglages) | blocs distincts |
| catalogues i18n (4 locales), `sprint-status.yaml` | toutes | chaque épique dans son bloc ; conflit résolu en gardant les deux côtés |
| `i18n-keys.test.ts` (`sitesTotal`), `audit_route_registry.rs` | 48-1, 48-2, 48-10, 48-12, 48-13 ; Epic 49 | recomptés après fusion |
| manuels `.tex` | toutes ; v0200-menu-sous-groupes change les chemins de menu des quatre manuels | le menu fusionne en dernier ; PDF jamais commités |
| `CHANGELOG.md` | — | jamais écrit dans les stories ; assemblé à l'intégration depuis les *Notes de version* |

## Décomptes au kickoff

Recomptés le 2026-11-18 depuis GitHub (commandes du `CLAUDE.md`) : **496 issues ouvertes**, **247 bugs** (`bug` ou
`known-failure`), **14 P1** ouvertes (dont 11 portant `bug` ou `known-failure`) — identiques au tag v0.19.0 ; hors les 56
issues masquées du compte de l'agent de Beat. L'Epic 48 vise **seize** issues portant `bug` ou `known-failure` : treize au
milestone v0.20.0 (#1245, #1173, #1175, #1176, #1177, #975, #984, #985, #986, #1233, #1243, #1226, #1234) et jusqu'à trois
dans la story de solde (#1261, #1262, #1263, si retenues), plus #1167 (CR).
