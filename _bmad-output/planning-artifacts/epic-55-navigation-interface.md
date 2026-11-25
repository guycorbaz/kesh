# Epic 55 — Navigation de l'interface

**Kickoff : 2026-11-25.** Numéro **55** attribué par le mainteneur le 2026-11-25 ; responsable de l'epic : contributeur.
Branche `epic/55-navigation`, partie de `origin/main` à `1e23edb55` pour le kickoff (fichier d'epic et fiches seulement,
aucun code). **Le développement repart de la PR de l'Epic 46, une fois celle-ci poussée** (§ *Recouvrement avec l'Epic 46*) :
la branche reprend alors cette base avant la première ligne de code de la 55-1a.

**Release visée** : la **prochaine release planifiée après la 0.20.0** — numéro et milestone **fixés par le mainteneur**
(`CLAUDE.md` § *Numérotation des versions* : un numéro n'est acquis qu'au tag). Contrainte d'ordre : l'Epic 55 part de
l'Epic 46 (prévision 0.21.0) ; il ne peut pas sortir avant lui, et sort au plus tôt avec lui ou dans la release suivante.
Un epic = une release.

**Migration** : aucune. **Backend** : **oui, pour la seule 55-3a** (voir § *Backend*), ajout d'un booléen (`hasDocuments`) aux
réponses JSON du journal et du grand livre, d'une route de lecture, `GET /api/v1/journal-entries/{id}/documents`, et d'une
**route d'aperçu sans figement**, `GET /api/v1/invoices/{id}/pdf/preview` (D12) — l'epic avait été annoncé « frontend seulement » ; **à confirmer par le mainteneur (Q1)**.

**Taille** : **six stories**, solde compris — **à la borne** de six d'un epic de correction ou d'amélioration. La 55-3,
ajoutée le 2026-11-25, est découpée en 55-3a et 55-3b à sa validation P1 (plus de cinq modules) ; la 55-1, en 55-1a et
55-1b à sa validation P3. ⚠️ **Signal de non-convergence déclaré au mainteneur** (`CLAUDE.md` § *Règle de splitting
préventif*) : la P3 de la 55-1 a remonté un HIGH après une P2 à MEDIUM, une part de ses défauts naissant des remédiations
de la P2 (recyclage) ; le découpage sépare la présentation de l'Epic 46 (55-1b) de l'URL des rapports (55-1a), et le modèle
est simplifié (la page n'applique jamais l'URL qu'elle écrit). Plus aucune story ne peut s'ajouter sans en sortir une. Le signal s'est **maintenu** à la P4 de la 55-1a (HIGH encore) : le
périmètre a été réduit (rapports par compte analytique hors de l'URL), et la P5 a rendu 0 HIGH.

**Issues** (ouvertes par l'agent du mainteneur ; numéros à reporter) :

| Point | Type | Priorité | Issue |
|---|---|---|---|
| a) Liens de compte partout dans les Rapports | `enhancement` | P4 | #À-VENIR |
| b) Retour du navigateur, état des Rapports dans l'URL | `bug` | P3 | #À-VENIR |
| c) Justificatifs (PDF) d'une écriture depuis le Journal | `enhancement` | à fixer | #À-VENIR (à ouvrir, convenue au téléphone le 2026-11-25) |
| d) Grand livre sans sélecteur d'exercice (*Générer* inutilisable en arrivant d'un autre onglet) | `bug` | P3 | #À-VENIR (texte : `kesh-gate-logs/s55-kickoff/issues-a-ouvrir.md`) ; traité par la 55-2 |

**Décompte au tag précédent** (`CLAUDE.md` § *Priorités des défauts*, à mesurer par le mainteneur, l'agent de l'epic n'ayant
pas `gh`) : issues ouvertes au tag précédent : **À-MESURER** ; dont bugs : **À-MESURER**. Objectif à terme : moins d'issues
ouvertes au tag de cette release qu'au tag précédent (objectif, non barrière).

**Plafond de la story de solde (55-4)** : **2 issues** de dette des stories 55-1a à 55-3b (comme la 46-7).

**Règles des fichiers communs** (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : aucune ligne au `CHANGELOG.md` en
cours d'epic (chaque fiche a sa *Note de version*) ; aucun PDF de manuel commité (`.tex` seul) ; catalogues i18n et
`sprint-status.yaml` en bloc propre ; `sitesTotal` de `i18n-keys.test.ts` recompté par la garde après chaque reprise.

## Objectif

L'utilisateur, en tenant ses livres, navigue dans les Rapports comme dans des pages :

1. **Retour et Suivant** du navigateur ramènent au rapport précédent ou suivant, avec sa période et ses filtres ; une vue
   des Rapports se **partage par un lien** et se retrouve au rechargement (55-1a ; la présentation de l'Epic 46, 55-1b).
2. Le **numéro de compte est un lien** vers le grand livre de ce compte partout où un rapport en montre un — Bilan, Compte
   de résultat, Balance, Journaux, Grand livre et, par les comptes désignés, TVA —, par **un seul composant** (55-2).
3. **Les justificatifs d'une écriture** — PDF figé de la facture, PDF de l'avoir, document d'une facture fournisseur —
   s'ouvrent **depuis le Journal**, par un bouton sur l'écriture, sans passer par l'écriture ni la facture, et **sans figer**
   la facture (aperçu, bandeau « PDF non figé ») ; le Grand livre suit (55-3a serveur, 55-3b écran).
4. Le **Grand livre a son sélecteur d'exercice**, avec « dates libres » ; ses dates sont pré-remplies et *Générer*
   utilisable en arrivant d'un autre onglet (55-2, défaut d). « Justificatif » et non « pièce », qui désigne déjà le numéro d'écriture au grand livre.

**Hors de l'epic** : les liens de compte **hors des Rapports** (vue d'une écriture, factures, plan comptable, comptes
bancaires, postes ouverts, soldes d'ouverture, pages des prestataires), avec le saut vers le grand livre du compte sur
l'exercice de l'écriture, la ligne surlignée, n'entrent pas dans l'Epic 55 ; faut-il un epic à part ? (Q9 au mainteneur).
De même, un PDF de facture **produit hors de Kesh** (facture créée par l'API) : évolution, non livrée (55-3a, *Limite
connue et évolution*).

## Origine (relevé de l'utilisateur, transmis par le mainteneur le 2026-11-25)

- **a) Liens de compte.** Sous *Rapports*, le numéro de compte ouvre le grand livre filtré dans le Bilan, la Balance (et le
  Compte de résultat en vue groupée de l'Epic 46) ; il ne l'est pas sous Journaux, TVA ni dans le Grand livre.
- **b) Retour du navigateur.** *Retour* quitte la page Rapports au lieu de revenir au rapport précédent.
- **c) Pièces** (téléphone du 2026-11-25, utilisateur et mainteneur) : voir les PDF associés à une écriture depuis la liste,
  **en particulier dans le Journal**.
- **d) Grand livre** (remarque du 2026-11-25) : son bandeau n'a pas de sélecteur d'exercice ; en arrivant d'un autre
  onglet, *Générer* est inutilisable (`bug` P3).

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
| **Le défaut b)** | après un clic sur un lien de compte depuis le Bilan, *Retour* ramène l'URL à `?tab=balance-sheet` (ou `/reports`) **sans remonter le composant** : l'effet ci-dessus sort tôt (`tab ≠ general-ledger`), `activeTab` reste `general-ledger`. L'URL a changé, l'écran non ; le second *Retour* quitte la page. Les paramètres de période, d'exercice, de compte (hors lien) et de regroupement ne sont jamais dans l'URL | lecture du code, à confirmer par l'E2E rouge de la 55-1a (T1) |
| Période, exercice | `selectedFiscalYearId`, `periodStart`, `periodEnd` : `$state`, jamais écrits dans l'URL | `+page.svelte:82-84` |
| Grand livre | `ledgerFrom`, `ledgerTo`, `ledgerAccountId`, `ledgerIncludeZero` : `$state` | `+page.svelte:109-112` |
| Projets | `selectedProjectId`, `projectMode` | `+page.svelte:91-92` |
| Devise | **aucun filtre de devise** n'existe dans les Rapports, ni à `origin/main` ni sur les branches de l'Epic 46 (`grep currency\|devise` sur la page) | — |
| Regroupement (46) | `GroupingView { shown, detailLevel, subtotalPosition, open, closed }` par rapport groupé (bilan, compte de résultat, balance), mémorisée en `localStorage` par société et rapport, relue **avant la première requête** | `pr/46-rapports-groupes:frontend/src/lib/features/reports/reports-grouping.ts` ; `+page.svelte` (diff de 287 lignes) |
| Liens présents | Bilan (actifs, passifs, fonds propres), Balance ; Compte de résultat **seulement en vue groupée** (`GroupedReportRows.svelte`, lignes `account` et `own`) — **la vue à plat du Compte de résultat n'a pas de lien**, ni à `main` ni sur la PR 46 | `BalanceSheetView.svelte:145,184,239` ; `TrialBalanceView.svelte:69` ; `pr/46…:GroupedReportRows.svelte:151` |
| Liens absents | Journaux (`l.accountNumber`, texte), Grand livre (titre de section, contreparties en numéros), TVA (**aucun numéro de compte affiché** : lignes par taux) | `JournalReportView.svelte:68` ; `GeneralLedgerView.svelte:220,289` ; `VatReportView.svelte` |
| Autres utilisateurs de `ledgerHref` | carte des charges de l'accueil, panneau de réévaluation ; l'E2E de l'accueil vérifie l'URL **à l'ordre des paramètres près** | `ExpensePieCard.svelte:64` ; `RevaluationPanel.svelte:357` ; `tests/e2e/homepage-charts.spec.ts:129,142,233` |
| Liens entrants | `/reports?tab=aged-receivables` (échéancier, rappels) ; `/reports` (soldes d'ouverture) | `invoices/due-dates/+page.svelte:401` ; `invoices/reminders/+page.svelte:277` ; `settings/opening-balances/+page.svelte:691` |
| Justificatifs d'une écriture | aucune table de pièces jointes ; trois documents se rattachent à une écriture : PDF **figé** d'une facture (`invoices.journal_entry_id`, colonnes `pdf_*`), PDF d'un avoir (`credit_notes.journal_entry_id`), document source d'une facture fournisseur (`supplier_invoices.purchase_journal_entry_id` ← `imported_supplier_invoices.supplier_invoice_id`, `status = 'completed'`) ; les règlements y renvoient par `invoice_settlements.journal_entry_id` (plusieurs factures par écriture depuis `20261118000001_grouped_settlements`) et `supplier_invoices.settlement_journal_entry_id` | migrations `20260417000001`, `20260627000001`, `20260628000001`, `20260629000001`, `20261003000001`, `20261118000001` |
| Routes de lecture | `GET /api/v1/invoices/{id}/pdf` (fige au premier rendu), `GET /api/v1/credit-notes/{id}/pdf`, `GET /api/v1/supplier-invoices/{id}/source-document` — **tout rôle authentifié** | `kesh-api/src/lib.rs:1262,1275,1291` |
| DTO des rapports | `JournalEntryRow { entryId, entryNumber, entryDate, description, lines }`, `LedgerLine { entryId, … }` : **aucun champ de justificatif** | `kesh-report/src/journal_report.rs:41` ; `general_ledger.rs:229` |

## Décisions (proposées au kickoff ; celles marquées **Q** attendent le mainteneur)

- **D1 — L'URL est la source de l'état des Rapports.** L'onglet et tous les paramètres de la vue (exercice, période,
  compte, comptes sans mouvement, regroupement de l'Epic 46) sont des paramètres de requête — **les rapports par compte
  analytique** n'y portent que leur onglet (validation P4 de la 55-1a, issue à ouvrir) ;
  l'écran se déduit de l'URL au chargement **et à chaque navigation** (Retour, Suivant, lien), par une fonction pure
  (`reports-url.ts`, lecture et écriture), non par des effets dispersés. (55-1a)
- **D2 — Empiler ou remplacer selon le geste.** Clic sur un onglet et clic sur un lien de compte : **`pushState`** (une
  entrée d'historique) ; *Générer* et flèches du clavier sur les onglets : **`replaceState`** ; les contrôles de
  regroupement et les accordéons n'écrivent pas l'URL, c'est la réponse affichée qui l'écrit (55-1b) ; un champ modifié sans *Générer* n'écrit rien (l'URL décrit le dernier rapport demandé).
  ⚠️ Le clic sur un onglet en `pushState` **renverse D-7c** de la Story 21-7 (« pas d'empilement ») : c'est la demande de
  l'utilisateur (b). (55-1a)
- **D3 — Une URL venue de l'extérieur affiche son rapport ; un clic sur un onglet ne génère rien** (réponse de
  l'utilisateur du 2026-11-25 : revenir à un onglet généré montre déjà son rapport ; reste l'adresse collée, rechargée ou
  atteinte par Retour, 55-1a AC-13). Arrivée sur une
  URL complète (lien collé, rechargement, lien de compte, Retour ou Suivant) : le rapport s'affiche sans clic sur
  *Générer*, **repris de la mémoire** s'il a été produit avec les mêmes paramètres, **redemandé** sinon. Le clic sur un
  onglet, lui, montre le rapport que l'onglet a déjà, ou son invite, **comme aujourd'hui** : aucune requête, et la règle
  revue de la 46-2 (un rapport groupé redemandé à la fin d'une génération en vol) reste intacte. `/reports` sans
  paramètre reste l'écran d'aujourd'hui. *Écarté* (validation P1 de la 55-1, devenue 55-1a) : générer à chaque changement d'onglet.
  *(Amendée le 2026-11-25.)*
- **D4 — Vue de regroupement : URL d'abord, mémoire ensuite.** Les paramètres de regroupement de l'URL priment sur la
  mémoire du navigateur de l'Epic 46 ; absents, la vue des contrôles (qui porte la mémoire) s'applique. La présentation
  n'entre dans l'URL **que par la réponse affichée** (qui complète l'URL), pour qu'un lien copié porte la vue affichée.
  *(Amendée à la validation P4 de la 55-1b.)* (55-1b)
- **D5 — Valeurs invalides : ignorées paramètre par paramètre, URL normalisée** en `replaceState`, sans message. Les
  bornes métier (période hors exercice, `from > to`) restent jugées par l'API, avec ses messages d'aujourd'hui. (55-1a)
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
- **D9 — Un booléen dans le rapport, la liste au clic (Q1 ; décision de l'utilisateur du 2026-11-25).** Chaque écriture du
  journal (et chaque ligne du grand livre) gagne `hasDocuments` (« a au moins un justificatif »), calculé par **une**
  requête groupée sur la période (`UNION ALL` des cinq rattachements, aucune requête par écriture) ; la liste des
  justificatifs se lit **au clic**, par une route de lecture minimale, `GET /api/v1/journal-entries/{id}/documents` (aucune
  route existante ne la donne). PDF et CSV des rapports inchangés ; outil MCP du grand livre inchangé (il recopie ses
  champs, `mcp/outils/grand_livre.rs:175`). (55-3a)
- **D10 — Une écriture de règlement montre la facture réglée** (réponse de l'utilisateur du 2026-11-25 : oui, pour les
  factures fournisseur **et** client ; l'écriture d'une facture client montre son PDF). Le règlement d'une facture client ou fournisseur
  présente le PDF de la facture (ou les PDF de plusieurs factures, règlement groupé) ; *écarté* : rien sur les règlements —
  la pièce d'un paiement est la facture qu'il solde. (55-3a)
- **D11 — Une icône, la visionneuse existante (décision de l'utilisateur du 2026-11-25).** Une seule icône PDF par ligne
  d'écriture, sans compteur ni menu ; au clic, `DocumentPreview` (le lecteur PDF du navigateur, pages à gauche). **Un seul
  justificatif** (le cas courant) : la visionneuse l'ouvre directement, **sans liste** ; **deux ou plus** : la liste en
  haut, un clic dans la liste change le document (simplification de l'utilisateur, 2026-11-25). Droits des routes
  inchangés. (55-3b)
- **D12 — Afficher n'est pas figer (réponse de l'utilisateur du 2026-11-25, ancienne Q7).** Depuis le Journal et les
  Rapports, afficher une facture **ne la fige pas** : une route d'aperçu neuve, `GET /api/v1/invoices/{id}/pdf/preview`
  (fichier figé s'il existe, rendu non figé sinon, **aucune écriture** ni trace d'audit, droits de `/pdf`, *Consultation*
  compris) — aucune voie d'aperçu sans figement n'existe au code (`issued_invoice_pdf.rs` : `Usage` n'a que `Download` et
  `Send`). La liste porte l'état du PDF (`pdfStatus`) ; une facture non figée s'affiche sous un **bandeau** « Aperçu — PDF
  non figé : … ni téléchargée ni envoyée, … peut encore changer » (les termes de Kesh : « figé », « PDF émis »). **La fiche
  de la facture ne change pas** (son téléchargement fige) ; son affichage : Q8. (55-3a, 55-3b)

## Stories

| Clé | Titre | Dépend de | Taille | Fiche |
|---|---|---|---|---|
| 55-0 | Kickoff (ce fichier ; pas de fiche) | — | — | — |
| 55-1a | L'état des Rapports dans l'URL : Retour, Suivant, lien partagé | base PR 46 | M | `55-1a-etat-des-rapports-dans-l-url.md` |
| 55-1b | La présentation des rapports groupés dans l'URL (Epic 46) | 55-1a | S | `55-1b-presentation-dans-l-url.md` |
| 55-2 | Le lien de compte partout : Journaux, Grand livre, TVA, Compte de résultat à plat ; sélecteur d'exercice du Grand livre | 55-1a | M | `55-2-lien-de-compte-partout.md` |
| 55-3a | Les justificatifs d'une écriture dans la réponse du journal et du grand livre, et l'aperçu sans figement (serveur) | base PR 46 ; Q1 ; **backend** | M | `55-3a-justificatifs-serveur.md` |
| 55-3b | Ouvrir les justificatifs depuis le Journal (et le Grand livre) | 55-3a, 55-2 | M | `55-3b-justificatifs-ecran.md` |
| 55-4 | Solde de la dette de l'epic (plafond : **2 issues**) | toutes | S | `55-4-solde-dette-epic.md` |

Ordre : 55-1a → 55-1b → 55-2 → 55-3b → 55-4 ; la 55-3a (serveur, aucun fichier commun avec les autres stories) peut se
faire en parallèle de la 55-1a ou de la 55-2, avant la 55-3b.

## Gates

Règle **proposée** par le responsable de l'epic (celle qu'il applique à ses epics) ; c'est un **écart** à `CLAUDE.md`
§ *Test Locally First*, qui demande la suite E2E complète au dernier commit de code de **chaque** story (« rejouée au push »
interdit, décision D7) : **soumise au mainteneur (Q6)**. Tant que Q6 n'est pas tranchée, ou si elle est refusée, la règle de
`CLAUDE.md` s'applique telle quelle : E2E complet au dernier commit de code de chaque story ; `scripts/gate-ci.sh` et la
suite complète de fin d'epic restent dus dans tous les cas.

- **par story**, au dernier commit de code : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; pour la 55-3a
  en plus `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` et `cargo nextest` sur le rayon
  d'impact ; les **specs E2E de la story et celles des Rapports** (`reports*.spec.ts`, `homepage-charts.spec.ts`,
  `open-items.spec.ts`, `dunning-roundtrip.spec.ts`) ;
- **en fin d'epic** (55-4, T2), après le dernier commit de code : `scripts/gate-ci.sh <base>` et la **suite E2E complète** ;
  « vert » = seulement les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par fichier.

Le Dev Agent Record de chaque story dit exactement ce qui a tourné (« gate ciblé …, gate complet en fin d'epic »).
Chaque story écrit en section *Issues à ouvrir* de son Dev Agent Record les points de revue non corrigés (sévérité,
site, texte) : c'est la source de la story de solde (55-4). Chaque issue ouverte de ces sections porte dans son corps
« Epic 55, story 55-x ».

**Revue du code d'intégration** (`CLAUDE.md` § *Fusions et fichiers communs*) : avant la PR de l'epic vers `main`, et
après chaque reprise de la PR 46 ou de `main`, une revue relit les commits de fusion en diff combiné et cherche les
interactions sans conflit de texte ; journal **à produire** dans `kesh-gate-logs/s55-integration/`, avec sa propre section
*Issues à ouvrir* (marque « Epic 55, revue d'intégration »). La 55-4 n'en prend les points que si elle a eu lieu avant
son T0 ; sinon, ils vont en issues à leur nom.

## Backend

L'epic était annoncé « frontend seulement ». **La 55-3a touche le backend** : `kesh-report` (`journal_report.rs`,
`general_ledger.rs`, `documents.rs` neuf : une définition des rattachements, partagée) et `kesh-api` (un service et une route de lecture, `GET /journal-entries/{id}/documents` ; l'aperçu sans figement, `GET /invoices/{id}/pdf/preview`, D12), sans migration, sans changement
des exports PDF et CSV ; le contrat JSON du journal et du grand livre gagne un booléen **additif**. Les 55-1a, 55-1b, 55-2, 55-3b et 55-4 restent
frontend seulement. **Q1 au mainteneur** : accepter ce périmètre, ou sortir les 55-3a et 55-3b de l'epic.

## Recouvrement avec l'Epic 46

L'Epic 46 (rapports par groupes) n'est pas encore dans `main` ; il modifie la page des Rapports et les vues que l'Epic 55
touche. **L'état d'URL inclut le regroupement de l'Epic 46** (55-1b) (`view`, `level`, `subtotals`, `open`, `closed`).
L'Epic 55 se développe **sur la base de la PR 46** une fois poussée ; le second arrivé dans `main` reprend l'autre et rejoue
les gates (`CLAUDE.md` § *Ordre des releases*). Si la PR 46 tarde, la 55-1a peut partir de `main` **sans** les paramètres de
regroupement, à condition de le dire au Change Log et de les ajouter à la reprise (la fonction `reports-url.ts` est faite
pour s'étendre d'un paramètre sans rien changer d'autre).

## Fichiers communs

Déclarés au kickoff (`CLAUDE.md` § *Fusions et fichiers communs*), relevés le 2026-11-25.

| Fichier ou module | Stories de l'epic | Autres chantiers qui le touchent | Règle |
|---|---|---|---|
| `frontend/src/routes/(app)/reports/+page.svelte` | 55-1a (fort), 55-1b, 55-2, 55-3b | **Epic 46** (`pr/46-rapports-groupes` `511e61d54`, `story/46-export-api-groupe` `8ca669b3f`, `story/46-5` `0f483c7bb` : +287 lignes, `GroupingControls`, `genSeq`/`invalidateInFlight`, `companyId`) | 55 part de la PR 46 ; l'état de regroupement passe par l'URL (D4) |
| `frontend/src/routes/(app)/reports/+page.ts` | 55-1a (lecture) | Epic 46 (`companyId`) | — |
| `frontend/src/lib/features/reports/reports-grouping.ts` | 55-1b (`groupingFromParams` neuve, borne `Number.isSafeInteger` dans `readIdList`) | **Epic 46** (neuf) | réutiliser `sanitizeGroupingView`, `isDetailLevel`, aucune copie |
| `frontend/src/lib/features/reports/GroupingControls.svelte`, `GroupedReportRows.svelte` | 55-2 (`AccountLink` dans `GroupedReportRows`) | **Epic 46** (neufs) | — |
| `frontend/src/lib/features/reports/BalanceSheetView.svelte`, `IncomeStatementView.svelte` | 55-2 | **Epic 46** (branches groupées) ; **Epic 47** (`epic/47-comptes-miroirs` : `MirrorOfNote`) | lien seul remplacé, structure inchangée ; `MirrorOfNote` (Epic 47) reste en texte, issue ouverte par la 55-2 : le second arrivé le sait |
| `frontend/src/lib/features/reports/TrialBalanceView.svelte` | 55-2 | **Epic 46** (46-6b, balance groupée) | idem |
| `frontend/src/lib/features/reports/reports.api.ts` | 55-1a (`ledgerHref` inchangé), 55-2, 55-3b (lecteurs des PDF) | **Epic 46** (paramètres de regroupement, 46-8 : `getReportExportUrl`) | `ledgerHref` : sortie inchangée (D6) |
| `frontend/src/lib/features/reports/reports.types.ts` | 55-3b (`hasDocuments`, `EntryDocument`) | **Epic 46**, Epic 47 | champ ajouté en fin de type |
| `frontend/src/lib/features/reports/JournalReportView.svelte`, `GeneralLedgerView.svelte`, `VatReportView.svelte`, `ReportSelector.svelte` | 55-1a (`ReportSelector` : exercice en gras, AC-14), 55-2, 55-3b | — | 55-2 puis 55-3b |
| `frontend/src/lib/features/reports/reports-url.ts` (neuf), `AccountLink.svelte` (neuf), `EntryDocuments.svelte`, `EntryDocumentsViewer.svelte` (neufs) | 55-1a, 55-1b, 55-2, 55-3b | — | — |
| `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` | 55-3b (propriété facultative `errorMessage`) | — | comportement d'aujourd'hui inchangé sans la propriété |
| `crates/kesh-report/src/journal_report.rs`, `general_ledger.rs`, `documents.rs` (neuf) ; `crates/kesh-api/src/lib.rs`, `routes/journal_entries.rs`, `services/ecritures.rs` (service et route de lecture), `routes/issued_invoice_pdf.rs`, `routes/invoice_pdf.rs` (aperçu, D12) ; tests neufs `crates/kesh-report/tests/documents.rs`, `crates/kesh-api/tests/journal_entry_documents_e2e.rs`, `invoice_pdf_preview_e2e.rs` | 55-3a | ni l'Epic 46 ni l'Epic 47 n'y touchent | champ additif, exports inchangés |
| `crates/kesh-report/src/lib.rs` | 55-3a (`pub mod documents`) | **Epic 46** (+25 lignes, `pub mod`/`pub use`), **Epic 47** | une ligne, dans l'ordre alphabétique des modules |
| `crates/kesh-report/src/csv.rs`, `pdf.rs`, `benches/export.rs` (littéraux de test seulement) | 55-3a | **Epic 46** (PR 46 : `csv.rs` 1119, `pdf.rs` 1776 lignes touchées), **Epic 47** (`csv.rs`, `pdf.rs`, `presentation.rs`) | une ligne `has_documents: false,` par littéral, rien d'autre |
| `docs/api-external.md` (grand livre `:310`, journal `:1164` à `1e23edb55`) | 55-3a | **Epic 46** (+74 lignes vers `:313`), **Epic 47** (vers `:308-311`) | phrase ajoutée en fin de paragraphe, lignes relevées sur la base |
| `crates/kesh-api/tests/reports_e2e.rs` | 55-3a | Epics 46 et 47 (cités par le fichier de l'Epic 46) | bloc de tests propre |
| `crates/kesh-i18n/locales/*/messages.ftl` | 55-2, 55-3b (55-1a si un texte apparaît) | Epic 46 (`reports-groups-*`) | bloc `reports-nav-*` propre à l'epic |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 55-2, 55-3b | tous | recompté par la garde après chaque reprise |
| `frontend/tests/e2e/reports.spec.ts`, `homepage-charts.spec.ts`, `open-items.spec.ts`, `dunning-roundtrip.spec.ts` | 55-1a (comportement de génération, D3) | Epic 46 (`reports-groupes*.spec.ts`) | E2E neufs : `reports-navigation.spec.ts` (55-1a, 55-1b, 55-2), `reports-entry-documents.spec.ts` (55-3b) |
| `docs/manual/fr/user-manual.tex` § *Rapports comptables* (`:4101`-`:4476` à `1e23edb55`) | 55-1a, 55-1b, 55-2, 55-3b (et § 11.6.1 *Le PDF émis est figé*, `:1635` : une phrase, l'affichage depuis les rapports ne fige pas, 55-3b) | Epic 46 (§ *Bilan*, *Compte de résultat*, *Balance*) | `.tex` seul, PDF à la release |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | toutes | tous | bloc propre à l'Epic 55 |

## Réponses de l'utilisateur (2026-11-25)

Remarques d'usager, versées aux fiches ; pour les décisions de produit, la dernière main revient au mainteneur, qui peut
revenir sur chacune.

- **Ancienne Q2 (D3)** : revenir par un clic à un onglet déjà généré montre déjà son rapport ; reste à faire qu'une adresse
  collée, rechargée ou atteinte par Retour affiche son rapport sans *Générer* (55-1a, AC-13 récrit). « Pas de requête en
  double » est une exigence technique, non une demande de l'utilisateur (55-1a, R-4 tranché).
- **Ancienne Q4 (D10)** : oui — un règlement montre le PDF de la facture réglée, fournisseur **et** client ; l'écriture
  d'une facture client montre le PDF de sa facture. Évolution notée : un PDF produit hors de Kesh (facture créée par
  l'API) devra un jour s'afficher à la place du PDF généré ; non livré, le modèle ne l'empêche pas (55-3a).
- **Ancienne Q7 (D12)** : afficher une facture depuis le Journal et les Rapports **ne la fige pas** ; bandeau « PDF non
  figé » ; route d'aperçu neuve (55-3a, 55-3b).
- **D11** : une écriture n'a le plus souvent qu'une facture ; avec un seul justificatif, aucune liste (55-3b).

## Questions au mainteneur

- **Q1** — La 55-3a ajoute un booléen aux réponses JSON du journal et du grand livre, une route de lecture,
  `GET /journal-entries/{id}/documents`, et une route d'aperçu sans figement, `GET /invoices/{id}/pdf/preview` (backend,
  sans migration) : accepté dans cet epic ?
- **Q3** — D7 : le lien de la TVA passe par les comptes désignés des réglages : d'accord ?
- **Q5** — Priorité de l'issue c) (proposition : P4, `enhancement`).
- **Q6** — § *Gates* : E2E complet et `scripts/gate-ci.sh` en fin d'epic seulement, specs des Rapports à chaque story —
  écart à `CLAUDE.md` § *Test Locally First* (D7) : accepté pour cet epic ?
- **Q8** — *(question de l'utilisateur, non tranchée)* Sur la **fiche de la facture**, le téléchargement continue de
  figer. Faut-il que l'**affichage** y passe aussi par l'aperçu, sans figer, avec le même bandeau (le figement restant au
  téléchargement et à l'envoi) ? Hors de l'Epic 55 tel qu'il est écrit ; une story de plus le ferait sortir de la borne
  de six.
- **Q9** — Les liens de compte dans le reste de l'application (vue d'une écriture, factures, plan comptable, comptes
  bancaires, postes ouverts, soldes d'ouverture, pages des prestataires), avec le saut vers le grand livre de l'exercice
  de l'écriture, la ligne surlignée : faut-il un **epic séparé** ?
- **Q10** — L'aperçu d'une facture non figée se télécharge par le lecteur PDF du navigateur, **sans mention** qui le
  distingue du PDF émis ; le bandeau n'est que dans Kesh. Faut-il une mention dans le PDF d'aperçu lui-même (« Aperçu —
  non émis ») ? Proposition : non dans l'Epic 55 (le PDF rendu serait différent du PDF émis), issue à ouvrir si oui.
