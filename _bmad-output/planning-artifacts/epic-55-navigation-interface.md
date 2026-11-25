# Epic 55 — Navigation de l'interface

**Kickoff : 2026-11-25.** Numéro **55** attribué par le mainteneur le 2026-11-25 ; responsable de l'epic : contributeur.
Branche `epic/55-navigation`, partie de `origin/main` à `1e23edb55` pour le kickoff (fichier d'epic et fiches seulement,
aucun code). **Le développement repart de la PR de l'Epic 46, une fois celle-ci poussée** (§ *Recouvrement avec l'Epic 46*) :
la branche reprend alors cette base avant la première ligne de code de la 55-1.

**Release visée** : la **prochaine release planifiée après la 0.20.0** — numéro et milestone **fixés par le mainteneur**
(`CLAUDE.md` § *Numérotation des versions* : un numéro n'est acquis qu'au tag). Contrainte d'ordre : l'Epic 55 part de
l'Epic 46 (prévision 0.21.0) ; il ne peut pas sortir avant lui, et sort au plus tôt avec lui ou dans la release suivante.
Un epic = une release.

**Migration** : aucune. **Backend** : **oui, pour la seule 55-3** (voir § *Backend*), ajout d'un champ aux réponses JSON du
journal et du grand livre — l'epic avait été annoncé « frontend seulement » ; **à confirmer par le mainteneur (Q1)**.

**Taille** : quatre stories, solde compris — sous la borne de six d'un epic de correction ou d'amélioration.

**Issues** (ouvertes par l'agent du mainteneur ; numéros à reporter) :

| Point | Type | Priorité | Issue |
|---|---|---|---|
| a) Liens de compte partout dans les Rapports | `enhancement` | P4 | #À-VENIR |
| b) Retour du navigateur, état des Rapports dans l'URL | `bug` | P3 | #À-VENIR |
| c) Pièces (PDF) d'une écriture depuis le Journal | `enhancement` | à fixer | #À-VENIR (à ouvrir, convenue au téléphone le 2026-11-25) |

**Décompte au tag précédent** (`CLAUDE.md` § *Priorités des défauts*, à mesurer par le mainteneur, l'agent de l'epic n'ayant
pas `gh`) : issues ouvertes au tag précédent : **À-MESURER** ; dont bugs : **À-MESURER**. Objectif à terme : moins d'issues
ouvertes au tag de cette release qu'au tag précédent (objectif, non barrière).

**Plafond de la story de solde (55-4)** : **2 issues** de dette des stories 55-1 à 55-3 (comme la 46-7).

**Règles des fichiers communs** (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : aucune ligne au `CHANGELOG.md` en
cours d'epic (chaque fiche a sa *Note de version*) ; aucun PDF de manuel commité (`.tex` seul) ; catalogues i18n et
`sprint-status.yaml` en bloc propre ; `sitesTotal` de `i18n-keys.test.ts` recompté par la garde après chaque reprise.

## Objectif

L'utilisateur, en tenant ses livres, navigue dans les Rapports comme dans des pages :

1. **Retour et Suivant** du navigateur ramènent au rapport précédent ou suivant, avec sa période et ses filtres ; une vue
   des Rapports se **partage par un lien** et se retrouve au rechargement (55-1).
2. Le **numéro de compte est un lien** vers le grand livre de ce compte partout où un rapport en montre un — Bilan, Compte
   de résultat, Balance, Journaux, Grand livre et, par les comptes désignés, TVA —, par **un seul composant** (55-2).
3. **Les pièces d'une écriture** — PDF figé de la facture, PDF de l'avoir, document d'une facture fournisseur — s'ouvrent
   **depuis le Journal**, par une icône sur l'écriture, sans passer par l'écriture ni la facture ; le Grand livre suit (55-3).

## Origine (relevé de l'utilisateur, transmis par le mainteneur le 2026-11-25)

- **a) Liens de compte.** Sous *Rapports*, le numéro de compte ouvre le grand livre filtré dans le Bilan, la Balance (et le
  Compte de résultat en vue groupée de l'Epic 46) ; il ne l'est pas sous Journaux, TVA ni dans le Grand livre.
- **b) Retour du navigateur.** *Retour* quitte la page Rapports au lieu de revenir au rapport précédent.
- **c) Pièces** (téléphone du 2026-11-25, utilisateur et mainteneur) : voir les PDF associés à une écriture depuis la liste,
  **en particulier dans le Journal**.

## Ce qui se lit au code (relevé du 2026-11-25)

Base `origin/main` à `1e23edb55` ; branches de l'Epic 46 lues par `git show <branche>:<fichier>` : `pr/46-rapports-groupes`
(`511e61d54`, la PR de l'epic, squash de 46-1 à 46-7 au-dessus des Epics 38 à 40), `story/46-export-api-groupe`
(`8ca669b3f`, 46-8, descend de `pr/46-rapports-groupes`), `story/46-5` (`0f483c7bb`, développée à part ; **aucun écart** avec
`pr/46-rapports-groupes` sous `frontend/src/lib/features/reports/` ni `frontend/src/routes/(app)/reports/`).

| Point | Constat | Où |
|---|---|---|
| Onglet | `activeTab` est un `$state` du composant ; `selectTab` écrit `?tab=<id>` en **`replaceState`** (Story 21-7, D-7c, « pas d'empilement ») ; lu **une fois** au montage (`onMount`) | `routes/(app)/reports/+page.svelte:85`, `:546-551`, `:608-615` |
| Lien de compte | `ledgerHref(accountId, from, to)` rend `/reports?tab=general-ledger&accountId=…&from=…&to=…` ; un `<a href>` : SvelteKit **empile** une entrée d'historique | `features/reports/reports.api.ts:574-582` |
| Arrivée par un lien | un `$effect` lit `tab`, `from`, `to`, `accountId` **seulement si `tab=general-ledger`**, et génère le grand livre | `+page.svelte:567-586` |
| **Le défaut b)** | après un clic sur un lien de compte depuis le Bilan, *Retour* ramène l'URL à `?tab=balance-sheet` (ou `/reports`) **sans remonter le composant** : l'effet ci-dessus sort tôt (`tab ≠ general-ledger`), `activeTab` reste `general-ledger`. L'URL a changé, l'écran non ; le second *Retour* quitte la page. Les paramètres de période, d'exercice, de compte (hors lien) et de regroupement ne sont jamais dans l'URL | lecture du code, à confirmer par l'E2E rouge de la 55-1 (T1) |
| Période, exercice | `selectedFiscalYearId`, `periodStart`, `periodEnd` : `$state`, jamais écrits dans l'URL | `+page.svelte:82-84` |
| Grand livre | `ledgerFrom`, `ledgerTo`, `ledgerAccountId`, `ledgerIncludeZero` : `$state` | `+page.svelte:109-112` |
| Projets | `selectedProjectId`, `projectMode` | `+page.svelte:91-92` |
| Devise | **aucun filtre de devise** n'existe dans les Rapports, ni à `origin/main` ni sur les branches de l'Epic 46 (`grep currency\|devise` sur la page) | — |
| Regroupement (46) | `GroupingView { shown, detailLevel, subtotalPosition, open, closed }` par rapport groupé (bilan, compte de résultat, balance), mémorisée en `localStorage` par société et rapport, relue **avant la première requête** | `pr/46-rapports-groupes:frontend/src/lib/features/reports/reports-grouping.ts` ; `+page.svelte` (diff de 287 lignes) |
| Liens présents | Bilan (actifs, passifs, fonds propres), Balance ; Compte de résultat **seulement en vue groupée** (`GroupedReportRows.svelte`, lignes `account` et `own`) — **la vue à plat du Compte de résultat n'a pas de lien**, ni à `main` ni sur la PR 46 | `BalanceSheetView.svelte:145,184,239` ; `TrialBalanceView.svelte:69` ; `pr/46…:GroupedReportRows.svelte:151` |
| Liens absents | Journaux (`l.accountNumber`, texte), Grand livre (titre de section, contreparties en numéros), TVA (**aucun numéro de compte affiché** : lignes par taux) | `JournalReportView.svelte:68` ; `GeneralLedgerView.svelte:220,289` ; `VatReportView.svelte` |
| Autres utilisateurs de `ledgerHref` | carte des charges de l'accueil, panneau de réévaluation ; l'E2E de l'accueil vérifie l'URL **à l'ordre des paramètres près** | `ExpensePieCard.svelte:64` ; `RevaluationPanel.svelte:357` ; `tests/e2e/homepage-charts.spec.ts:129,142,233` |
| Liens entrants | `/reports?tab=aged-receivables` (échéancier, rappels) ; `/reports` (soldes d'ouverture) | `invoices/due-dates/+page.svelte:401` ; `invoices/reminders/+page.svelte:277` ; `settings/opening-balances/+page.svelte:691` |
| Pièces d'une écriture | aucune table de pièces jointes ; trois documents se rattachent à une écriture : PDF **figé** d'une facture (`invoices.journal_entry_id`, colonnes `pdf_*`), PDF d'un avoir (`credit_notes.journal_entry_id`), document source d'une facture fournisseur (`supplier_invoices.purchase_journal_entry_id` ← `imported_supplier_invoices.supplier_invoice_id`, `status = 'completed'`) ; les règlements y renvoient par `invoice_settlements.journal_entry_id` (plusieurs factures par écriture depuis `20261118000001_grouped_settlements`) et `supplier_invoices.settlement_journal_entry_id` | migrations `20260417000001`, `20260627000001`, `20260628000001`, `20260629000001`, `20261003000001`, `20261118000001` |
| Routes de lecture | `GET /api/v1/invoices/{id}/pdf` (fige au premier rendu), `GET /api/v1/credit-notes/{id}/pdf`, `GET /api/v1/supplier-invoices/{id}/source-document` — **tout rôle authentifié** | `kesh-api/src/lib.rs:1262,1275,1291` |
| DTO des rapports | `JournalEntryRow { entryId, entryNumber, entryDate, description, lines }`, `LedgerLine { entryId, … }` : **aucun champ de pièce** | `kesh-report/src/journal_report.rs:41` ; `general_ledger.rs:245` |

## Décisions (proposées au kickoff ; celles marquées **Q** attendent le mainteneur)

- **D1 — L'URL est la source de l'état des Rapports.** L'onglet et tous les paramètres de la vue (exercice, période,
  compte, comptes sans mouvement, compte analytique et mode, regroupement de l'Epic 46) sont des paramètres de requête ;
  l'écran se déduit de l'URL au chargement **et à chaque navigation** (Retour, Suivant, lien), par une fonction pure
  (`reports-url.ts`, lecture et écriture), non par des effets dispersés. (55-1)
- **D2 — Empiler ou remplacer selon le geste.** Changement d'onglet et clic sur un lien de compte : **`pushState`** (une
  entrée d'historique) ; *Générer*, changement d'exercice, contrôles de regroupement et accordéons : **`replaceState`**.
  ⚠️ Le changement d'onglet en `pushState` **renverse D-7c** de la Story 21-7 (« pas d'empilement ») : c'est la demande de
  l'utilisateur (b). (55-1)
- **D3 — Une URL complète affiche son rapport (Q2).** Quand l'URL porte l'onglet et les paramètres requis de ce rapport
  (l'exercice pour les rapports classiques, `from` et `to` pour le grand livre, le compte analytique pour les rapports
  par projet ; rien pour la balance âgée), le rapport s'affiche sans clic sur *Générer* — **repris de la mémoire** s'il a
  déjà été produit avec les mêmes paramètres, **redemandé** sinon. `/reports` sans paramètre reste l'écran d'aujourd'hui
  (« Sélectionnez un exercice et cliquez sur Générer »). Conséquence voulue : un changement d'onglet, l'exercice étant
  choisi, affiche le rapport de l'onglet (une requête, une trace `report.generated` au journal d'audit, comme un clic sur
  *Générer* aujourd'hui). *Écarté* : ne générer qu'au clic — un lien partagé ou un *Retour* afficherait alors un onglet vide.
- **D4 — Vue de regroupement : URL d'abord, mémoire ensuite.** Les paramètres de regroupement de l'URL priment sur la
  mémoire du navigateur de l'Epic 46 ; absents, la vue mémorisée s'applique et l'URL est complétée (`replaceState`), pour
  qu'un lien copié porte la vue affichée. La vue affichée reste mémorisée comme le fait déjà l'Epic 46 (dernière vue
  affichée par société et rapport). (55-1)
- **D5 — Valeurs invalides : ignorées paramètre par paramètre, URL normalisée** en `replaceState`, sans message. Les
  bornes métier (période hors exercice, `from > to`) restent jugées par l'API, avec ses messages d'aujourd'hui. (55-1)
- **D6 — Un seul composant de lien de compte** (`AccountLink.svelte`) qui rend le numéro et le lien de `ledgerHref` ;
  toutes les vues l'emploient. `ledgerHref` garde sa sortie **à l'octet près** (ordre `tab`, `accountId`, `from`, `to`)
  pour l'accueil et la réévaluation. (55-2)
- **D7 — TVA : les comptes désignés (Q3).** Le décompte TVA ne montre aucun compte. La 55-2 ajoute une ligne « Comptes au
  grand livre : TVA due `NNNN` · TVA récupérable `MMMM` » (réglages de facturation, lisibles par tout rôle), chaque numéro
  en lien vers son grand livre sur la période du décompte ; compte non désigné : sa mention est omise. Une ligne à part, et
  non un lien sur les montants : la TVA due du décompte vient des factures, non du solde du compte (manuel, § *Décompte
  TVA*). *Écarté* : ajouter des comptes au DTO de la TVA (backend). *(Amendé à la validation P1 de la 55-2.)* (55-2)
- **D8 — Grand livre : contreparties et titre.** Les numéros de contrepartie deviennent des liens (numéro → compte par la
  liste des comptes déjà chargée par la page) ; le numéro du titre d'une section n'est un lien que quand le grand livre
  montre plusieurs comptes (un lien vers lui-même n'apprend rien). (55-2)
- **D9 — Les pièces viennent de l'API du rapport, en une requête (Q1).** Chaque écriture du journal (et chaque ligne du
  grand livre) gagne un champ `documents: [{ kind, id, label }]`, rempli par **une seule requête groupée** sur la période
  (`UNION ALL` des quatre rattachements, aucune requête par écriture) ; PDF et CSV des rapports inchangés ; outil MCP du
  grand livre inchangé (il recopie ses champs, `mcp/outils/grand_livre.rs:175`). (55-3)
- **D10 — Une écriture de règlement montre la pièce réglée (Q4).** Le règlement d'une facture client ou fournisseur
  présente le PDF de la facture (ou les PDF de plusieurs factures, règlement groupé) ; *écarté* : rien sur les règlements —
  la pièce d'un paiement est la facture qu'il solde. (55-3)
- **D11 — L'ouverture passe par les routes existantes**, leurs droits inchangés ; plusieurs pièces : une courte liste.
  (55-3)

## Stories

| Clé | Titre | Dépend de | Taille | Fiche |
|---|---|---|---|---|
| 55-0 | Kickoff (ce fichier ; pas de fiche) | — | — | — |
| 55-1 | L'état des Rapports dans l'URL : Retour, Suivant, lien partagé | base PR 46 | M | `55-1-etat-des-rapports-dans-l-url.md` |
| 55-2 | Le lien de compte partout : Journaux, Grand livre, TVA, Compte de résultat à plat | 55-1 | S | `55-2-lien-de-compte-partout.md` |
| 55-3 | Les pièces d'une écriture depuis le Journal (et le Grand livre) | 55-1 (base) ; **backend** | M | `55-3-pieces-des-ecritures.md` |
| 55-4 | Solde de la dette de l'epic (plafond : **2 issues**) | toutes | S | `55-4-solde-dette-epic.md` |

Ordre : 55-1 → 55-2 → 55-3 → 55-4. La 55-3 peut commencer sa partie serveur en parallèle de la 55-2 (aucun fichier serveur
commun) ; ses vues se posent après la 55-2 (`JournalReportView.svelte`, `GeneralLedgerView.svelte` communs).

## Gates

Règle de l'epic (décision du responsable de l'epic, appliquée à ses epics depuis l'Epic 37 ; **écart assumé** à
`CLAUDE.md` § *Test Locally First*, qui demande l'E2E complet au dernier commit de code de chaque story) :

- **par story**, au dernier commit de code : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; pour la 55-3
  en plus `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` et `cargo nextest` sur le rayon
  d'impact ; les **specs E2E de la story et celles des Rapports** (`reports*.spec.ts`, `homepage-charts.spec.ts`,
  `open-items.spec.ts`, `dunning-roundtrip.spec.ts`) ;
- **en fin d'epic** (55-4, T2), après le dernier commit de code : `scripts/gate-ci.sh <base>` et la **suite E2E complète** ;
  « vert » = seulement les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par fichier.

Le Dev Agent Record de chaque story dit exactement ce qui a tourné (« gate ciblé …, gate complet en fin d'epic »).
Chaque story écrit en section *Issues à ouvrir* de son Dev Agent Record les points de revue non corrigés (sévérité,
site, texte) : c'est la source de la story de solde (55-4).

## Backend

L'epic était annoncé « frontend seulement ». **La 55-3 touche le backend** : `kesh-report` (`journal_report.rs`,
`general_ledger.rs`, une fonction de lecture des pièces, partagée), sans migration, sans route neuve, sans changement des
exports PDF et CSV ; le contrat JSON du journal et du grand livre gagne un champ **additif**. Les 55-1, 55-2 et 55-4 restent
frontend seulement. **Q1 au mainteneur** : accepter ce périmètre, ou sortir la 55-3 de l'epic.

## Recouvrement avec l'Epic 46

L'Epic 46 (rapports par groupes) n'est pas encore dans `main` ; il modifie la page des Rapports et les vues que l'Epic 55
touche. **L'état d'URL de la 55-1 inclut le regroupement de l'Epic 46** (`view`, `level`, `subtotals`, `open`, `closed`).
L'Epic 55 se développe **sur la base de la PR 46** une fois poussée ; le second arrivé dans `main` reprend l'autre et rejoue
les gates (`CLAUDE.md` § *Ordre des releases*). Si la PR 46 tarde, la 55-1 peut partir de `main` **sans** les paramètres de
regroupement, à condition de le dire au Change Log et de les ajouter à la reprise (la fonction `reports-url.ts` est faite
pour s'étendre d'un paramètre sans rien changer d'autre).

## Fichiers communs

Déclarés au kickoff (`CLAUDE.md` § *Fusions et fichiers communs*), relevés le 2026-11-25.

| Fichier ou module | Stories de l'epic | Autres chantiers qui le touchent | Règle |
|---|---|---|---|
| `frontend/src/routes/(app)/reports/+page.svelte` | 55-1 (fort), 55-2, 55-3 | **Epic 46** (`pr/46-rapports-groupes` `511e61d54`, `story/46-export-api-groupe` `8ca669b3f`, `story/46-5` `0f483c7bb` : +287 lignes, `GroupingControls`, `genSeq`/`invalidateInFlight`, `companyId`) | 55 part de la PR 46 ; l'état de regroupement passe par l'URL (D4) |
| `frontend/src/routes/(app)/reports/+page.ts` | 55-1 (lecture) | Epic 46 (`companyId`) | — |
| `frontend/src/lib/features/reports/reports-grouping.ts` | 55-1 (lecture et écriture de la vue depuis l'URL) | **Epic 46** (neuf) | réutiliser `sanitizeGroupingView`, `isDetailLevel`, aucune copie |
| `frontend/src/lib/features/reports/GroupingControls.svelte`, `GroupedReportRows.svelte` | 55-2 (`AccountLink` dans `GroupedReportRows`) | **Epic 46** (neufs) | — |
| `frontend/src/lib/features/reports/BalanceSheetView.svelte`, `IncomeStatementView.svelte` | 55-2 | **Epic 46** (branches groupées) ; **Epic 47** (`epic/47-comptes-miroirs` : `MirrorOfNote`) | lien seul remplacé, structure inchangée |
| `frontend/src/lib/features/reports/TrialBalanceView.svelte` | 55-2 | **Epic 46** (46-6b, balance groupée) | idem |
| `frontend/src/lib/features/reports/reports.api.ts` | 55-1 (`ledgerHref` inchangé), 55-2, 55-3 (types d'appels) | **Epic 46** (paramètres de regroupement, 46-8 : `getReportExportUrl`) | `ledgerHref` : sortie inchangée (D6) |
| `frontend/src/lib/features/reports/reports.types.ts` | 55-3 (`documents`) | **Epic 46**, Epic 47 | champ ajouté en fin de type |
| `frontend/src/lib/features/reports/JournalReportView.svelte`, `GeneralLedgerView.svelte`, `VatReportView.svelte`, `ReportSelector.svelte` | 55-1 (`ReportSelector`), 55-2, 55-3 | — | 55-2 puis 55-3 |
| `frontend/src/lib/features/reports/reports-url.ts` (neuf), `AccountLink.svelte` (neuf), `EntryDocuments.svelte` (neuf) | 55-1, 55-2, 55-3 | — | — |
| `crates/kesh-report/src/journal_report.rs`, `general_ledger.rs`, `pieces.rs` (neuf) | 55-3 | Epic 46 n'y touche pas ; Epic 47 (`general_ledger.rs` ? à relever au T0 de la 55-3) | champ additif, exports inchangés |
| `crates/kesh-api/tests/reports_e2e.rs` | 55-3 | Epics 46 et 47 (cités par le fichier de l'Epic 46) | bloc de tests propre |
| `crates/kesh-i18n/locales/*/messages.ftl` | 55-2, 55-3 (55-1 si un texte apparaît) | Epic 46 (`reports-groups-*`) | bloc `reports-nav-*` propre à l'epic |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 55-2, 55-3 | tous | recompté par la garde après chaque reprise |
| `frontend/tests/e2e/reports.spec.ts`, `homepage-charts.spec.ts`, `open-items.spec.ts`, `dunning-roundtrip.spec.ts` | 55-1 (comportement de génération, D3) | Epic 46 (`reports-groupes*.spec.ts`) | E2E neufs dans `reports-navigation.spec.ts` |
| `docs/manual/fr/user-manual.tex` § *Rapports comptables* (`:4101`-`:4476` à `1e23edb55`) | 55-1, 55-2, 55-3 | Epic 46 (§ *Bilan*, *Compte de résultat*, *Balance*) | `.tex` seul, PDF à la release |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | toutes | tous | bloc propre à l'Epic 55 |

## Questions au mainteneur

- **Q1** — La 55-3 ajoute un champ aux réponses JSON du journal et du grand livre (backend, sans migration) : accepté dans
  cet epic ?
- **Q2** — D3 : un changement d'onglet affiche aussitôt le rapport de l'onglet (l'exercice étant choisi), au lieu
  d'attendre *Générer* : d'accord ?
- **Q3** — D7 : le lien de la TVA passe par les comptes désignés des réglages : d'accord ?
- **Q4** — D10 : une écriture de règlement montre le PDF de la facture réglée : d'accord ?
- **Q5** — Priorité de l'issue c) (proposition : P4, `enhancement`).
