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
réponses JSON du journal et du grand livre et d'une route de lecture, `GET /api/v1/journal-entries/{id}/documents` — l'epic
avait été annoncé « frontend seulement » ; **accepté par le mainteneur le 2026-11-25 (Q1)**. La route d'aperçu sans
figement et sa marque (`kesh-qrbill`), un temps prévues, **sont sorties de l'epic le 2026-11-26** pour l'epic « PDF
marqués » (§ *Décisions du mainteneur (2026-11-26)*).

**Taille** : **six stories**, solde compris — **à la borne** de six d'un epic de correction ou d'amélioration. La 55-3,
ajoutée le 2026-11-25, est découpée en 55-3a et 55-3b à sa validation P1 (plus de cinq modules) ; la 55-1, en 55-1a et
55-1b à sa validation P3. ⚠️ **Signal de non-convergence déclaré au mainteneur** (`CLAUDE.md` § *Règle de splitting
préventif*) : la P3 de la 55-1 a remonté un HIGH après une P2 à MEDIUM, une part de ses défauts naissant des remédiations
de la P2 (recyclage) ; le découpage sépare la présentation de l'Epic 46 (55-1b) de l'URL des rapports (55-1a), et le modèle
est simplifié (la page n'applique jamais l'URL qu'elle écrit). Plus aucune story ne peut s'ajouter sans en sortir une. Le signal s'est **maintenu** à la P4 de la 55-1a (HIGH encore) : le
périmètre a été réduit (rapports par compte analytique hors de l'URL), et la P5 a rendu 0 HIGH. Le 2026-11-26, le sélecteur
d'exercice (D15) entre dans la 55-1a, portée de M à **L** (cinq modules, à la borne), faute de place pour une septième
story ; le choix est posé au mainteneur (§ *Questions au mainteneur*). Le 2026-11-26 au soir, deux décisions de
l'utilisateur et de l'UX entrent **sans story neuve** : le chargement sans *Générer* (D16) dans la 55-1a, le style des
liens (D17) dans la 55-2, qui gagne un balayage mécanique des liens de l'application (§ *Questions au mainteneur*).

**Issues** (ouvertes par l'agent du mainteneur ; numéros à reporter) :

| Point | Type | Priorité | Issue |
|---|---|---|---|
| a) Liens de compte partout dans les Rapports | `enhancement` | P4 | #À-VENIR |
| b) Retour du navigateur, état des Rapports dans l'URL | `bug` | P3 | #À-VENIR |
| c) Justificatifs (PDF) d'une écriture depuis le Journal | `enhancement` | P4 (Q5, décidé le 2026-11-25) | #À-VENIR (convenue au téléphone le 2026-11-25) |
| d) Grand livre sans sélecteur d'exercice (*Générer* inutilisable en arrivant d'un autre onglet) | `bug` | P3 | #À-VENIR ; traité par la 55-2 |
| e) PDF d'une facture produit hors de Kesh (évolution, **non livrée** par l'epic) | `enhancement` | — | #À-VENIR ; ni livrable ni fermée par l'epic |

Textes des cinq issues, prêts à ouvrir : `kesh-gate-logs/s55-decisions/issues-a-envoyer.md` (celui de d) y est repris de
`kesh-gate-logs/s55-kickoff/issues-a-ouvrir.md`).

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
3. **Les justificatifs d'une écriture** — PDF de la facture (le PDF figé tel quel ; une facture pas encore émise est
   signalée « PDF pas encore émis », **sans aperçu**), PDF de l'avoir, document d'une facture fournisseur — s'ouvrent
   **depuis le Journal**, par un bouton sur l'écriture, sans passer par l'écriture ni la facture, et **sans figer** la
   facture ; le Grand livre suit (55-3a serveur, 55-3b écran). « Justificatif » et non « pièce », qui désigne déjà le
   numéro d'écriture au grand livre.
4. Le **Grand livre a son sélecteur d'exercice** — le même que les autres rapports (D15) ; ses dates sont pré-remplies et
   *Générer* utilisable en arrivant d'un autre onglet (55-2, défaut d).
5. **Un seul sélecteur d'exercice** sur tous les onglets à période (D15) : l'exercice seul, en gras ; « Du » et « Au »
   toujours remplis ; « Exercice entier », « Plusieurs », « Hors exercice » ; l'en-tête qui dit la période réelle (55-1a,
   et 55-2 pour le Grand livre).
6. **Le rapport se charge seul**, sans bouton *Générer*, sauf le Grand livre de tous les comptes (D16, 55-1a et 55-2).
7. **Un lien se voit sans la couleur** : couleur d'aujourd'hui et soulignement léger, partout (D17, 55-2).

## Hors périmètre

- **Les liens de compte hors des Rapports** (vue d'une écriture, factures, plan comptable, comptes bancaires, postes
  ouverts, soldes d'ouverture, pages des prestataires), avec le saut vers le grand livre du compte sur l'exercice de
  l'écriture et la ligne surlignée : **un epic à part (numéro à confirmer)**, décidé par le mainteneur le 2026-11-25 (Q9).
  Il dépend de l'Epic 55. **Ce que l'Epic 55 lui laisse de réutilisable**, vérifié aux fiches :
  - **le composant de lien** : `AccountLink.svelte` (55-2, AC-1), propriétés `{ accountId, number, from, to, linked?,
    testId? }`, sans dépendance à la page des Rapports — un appelant hors des Rapports lui passe les bornes de l'exercice
    de l'écriture ; il vit sous `frontend/src/lib/features/reports/` (l'epic à part le déplacera sous `lib/shared/` s'il le
    juge utile, `ledgerHref` gardant sa sortie à l'octet, D6) ;
  - **le saut** : une adresse `/reports?tab=general-ledger&accountId=…&from=…&to=…` affiche le grand livre de ce compte
    sans *Générer*, à l'arrivée par un lien, Retour ou Suivant (55-1a, D1 à D3) ; `reports-url.ts` est faite pour gagner un
    paramètre sans rien changer d'autre (§ *Recouvrement avec l'Epic 46*) ;
  - ⚠️ **la ligne surlignée n'est pas posée par l'Epic 55** : aucune fiche ne porte un paramètre d'écriture dans l'adresse
    du grand livre ni le surlignage d'une ligne (aucun AC ni aucune tâche des fiches 55-* ; les mentions de la 55-2,
    C-55-2-4 et son Change Log, disent qu'elle n'est pas posée). **Décision du mainteneur du 2026-11-26** : elle revient à
    cet epic à part (le 56 proposé), et **seulement depuis une écriture, une facture ou une transaction** — en un paramètre
    de plus dans `reports-url.ts` et `ledgerHref`. Ce n'est pas un livrable de l'Epic 55.
- **Un PDF de facture produit hors de Kesh** (facture créée par l'API) : évolution, **non livrée** (55-3a, *Limite connue et
  évolution* ; issue e). Règles décidées par le mainteneur le 2026-11-25, dans la limite connue de la 55-3a : le PDF fourni
  **est** la pièce émise — copié dans Kesh, sauvegardé, figé, jamais régénéré ; Kesh ne produit ni second PDF ni QR-facture
  à lui pour une facture arrivée avec son PDF ; « lien seulement » ne viendrait que sur demande, un lien ne passant pas
  dans le `.keshbackup`.
- **Les PDF marqués** : **un epic à part, « PDF marqués »** (numéro à confirmer), décidé par le mainteneur le 2026-11-26.
  Il porte le filigrane, l'en-tête et la QR-facture barrée dans `kesh-qrbill` (ancienne D13), l'aperçu « APERÇU — non
  émis » (ancienne route `GET /invoices/{id}/pdf/preview`, D12), l'aperçu d'un PDF déjà sorti sans être figé, avec son
  bandeau, et le PDF du brouillon. Leurs textes sont gardés aux fiches, section *Reporté à l'epic « PDF marqués »*
  (55-3a, 55-3b). L'Epic 55 lui laisse `pdfStatus`, énumération ouverte, et la visionneuse de la 55-3b ; tant qu'il n'est
  pas livré, une facture pas encore émise est signalée « PDF pas encore émis », sans aperçu.

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

## Décisions (proposées au kickoff ; les questions **Q** tranchées par le mainteneur le 2026-11-25, § *Décisions du mainteneur*)

- **D1 — L'URL est la source de l'état des Rapports.** L'onglet et tous les paramètres de la vue (exercice, période,
  compte, comptes sans mouvement, regroupement de l'Epic 46) sont des paramètres de requête — **les rapports par compte
  analytique** n'y portent que leur onglet (validation P4 de la 55-1a, issue à ouvrir) ;
  l'écran se déduit de l'URL au chargement **et à chaque navigation** (Retour, Suivant, lien), par une fonction pure
  (`reports-url.ts`, lecture et écriture), non par des effets dispersés. (55-1a)
- **D2 — Empiler ou remplacer selon le geste.** Clic sur un onglet et clic sur un lien de compte : **`pushState`** (une
  entrée d'historique) ; *Générer* et flèches du clavier sur les onglets : **`replaceState`** ; les contrôles de
  regroupement et les accordéons n'écrivent pas l'URL, c'est la réponse affichée qui l'écrit (55-1b) ; un champ modifié sans *Générer* n'écrit rien (l'URL décrit le dernier rapport demandé).
  ⚠️ Le clic sur un onglet en `pushState` **renverse D-7c** de la Story 21-7 (« pas d'empilement ») : c'est la demande de
  l'utilisateur (b). (55-1a) *(Amendée par D16 le 2026-11-26 : *Générer* n'existe plus que pour le Grand livre de tous
  les comptes ; le chargement déclenché par l'exercice, la sortie des dates ou Entrée **remplace** l'URL, comme
  *Générer* le faisait.)*
- **D3 — Une URL venue de l'extérieur affiche son rapport ; un clic sur un onglet ne génère rien** (réponse de
  l'utilisateur du 2026-11-25 : revenir à un onglet généré montre déjà son rapport ; reste l'adresse collée, rechargée ou
  atteinte par Retour, 55-1a AC-13). Arrivée sur une
  URL complète (lien collé, rechargement, lien de compte, Retour ou Suivant) : le rapport s'affiche sans clic sur
  *Générer*, **repris de la mémoire** s'il a été produit avec les mêmes paramètres, **redemandé** sinon. Le clic sur un
  onglet, lui, montre le rapport que l'onglet a déjà, ou son invite, **comme aujourd'hui** : aucune requête, et la règle
  revue de la 46-2 (un rapport groupé redemandé à la fin d'une génération en vol) reste intacte. `/reports` sans
  paramètre reste l'écran d'aujourd'hui. *Écarté* (validation P1 de la 55-1, devenue 55-1a) : générer à chaque changement d'onglet.
  *(Amendée le 2026-11-25.)* *(Amendée par D16 le 2026-11-26 : un clic sur un onglet **montre le rapport des champs** —
  repris de la mémoire s'il a la même clé, chargé sinon ; l'invite « cliquez sur Générer » disparaît. « Une URL venue de
  l'extérieur affiche son rapport » est inchangé.)*
- **D4 — Vue de regroupement : URL d'abord, mémoire ensuite.** Les paramètres de regroupement de l'URL priment sur la
  mémoire du navigateur de l'Epic 46 ; absents, la vue des contrôles (qui porte la mémoire) s'applique. La présentation
  n'entre dans l'URL **que par la réponse affichée** (qui complète l'URL), pour qu'un lien copié porte la vue affichée.
  *(Amendée à la validation P4 de la 55-1b.)* (55-1b)
- **D5 — Valeurs invalides : ignorées paramètre par paramètre, URL normalisée** en `replaceState`, sans message. Les
  bornes métier (période hors exercice, `from > to`) restent jugées par l'API, avec ses messages d'aujourd'hui. (55-1a)
- **D6 — Un seul composant de lien de compte** (`AccountLink.svelte`) qui rend le numéro et le lien de `ledgerHref` ;
  toutes les vues l'emploient. `ledgerHref` garde sa sortie **à l'octet près** (ordre `tab`, `accountId`, `from`, `to`)
  pour l'accueil et la réévaluation. (55-2)
- **D15 — Le sélecteur d'exercice des Rapports et du Grand livre (décision de l'utilisateur et avis UX du 2026-11-26, à
  confirmer par le mainteneur ; question ci-dessous).** Le libellé « Exercice » une seule fois, hors du menu, par un vrai
  `<label for>` (l'`aria-label` en double est retiré : « Exercice, liste déroulante, 2025 ») ; un `<select>` natif dont la
  valeur fermée est en gras (`select` 600, `option` 400), qui montre les **années** (« 2025 », « 2025/2026 ») quand le nom
  est celui que Kesh aurait donné (`nom_par_defaut`, `crates/kesh-core/src/accounting/exercice.rs:33-38`), sinon le **nom
  saisi**, et les **bornes** ajoutées aux seuls doublons (« 2025 (01.03–30.06) ») ; le menu ne montre plus les dates
  (`ReportSelector.svelte:55`). Choisir un exercice remplit « Du » et « Au » ; les dates sont toujours remplies. Dates
  changées à la main : même exercice → le menu le montre, même partiel, avec un lien « Exercice entier » ; exercices
  différents → option désactivée « Plusieurs » (DE « Mehrere », IT « Diversi », EN « Multiple ») ; une date hors de tout
  exercice → « Hors exercice » (« Ausserhalb », « Fuori esercizio », « Outside fiscal years ») ; début après fin → le menu
  ne change pas ; l'option désactivée disparaît dès qu'un exercice est choisi. **Le même `ReportSelector` sur tous les
  onglets**, Grand livre compris, vocabulaire « Du » / « Au ». Rapport **périmé** grisé, « Paramètres modifiés — cliquez
  sur Générer ». **En-tête** de la période réelle : « Exercice 2025 — du 01.01.2025 au 31.12.2025 », « du … au … (partie de
  l'exercice 2025/2026) », « du … au … (exercices 2024/2025 et 2025/2026) ». **Remplace** le gras partiel par
  `components/ui/select` (ancien AC-14 de la 55-1a, ses R-5 et R-7 ; R-3 de la 55-2) et l'entrée « — dates libres » de la
  55-2 ; `ui/select` n'est plus nécessaire. *Choix des fiches* (C-55-1-8) : sur un onglet qui ne porte que sur un exercice,
  « Plusieurs » et « Hors exercice » désactivent *Générer*, l'API exigeant un exercice qui contienne la période ; le Grand
  livre génère. Démonstration hors du dépôt : `kesh-gate-logs/s55-selecteur/selecteur-exercice-demo.html`. (55-1a AC-14 à
  AC-19, 55-2 AC-11 à AC-15) *(Amendée par D16 le 2026-11-26 : le rapport « périmé » et son message « Paramètres
  modifiés — cliquez sur Générer » deviennent le simple grisé du rechargement ; avec « Plusieurs » ou « Hors exercice » sur
  un onglet à un seul exercice, rien n'est chargé et un message dit pourquoi, C-55-1-8 récrit.)*
- **D16 — Le rapport se charge seul, sans *Générer* (décision de l'utilisateur et avis UX du 2026-11-26, soir).**
  - **Rapports concernés** : ceux qui ont une période — Bilan, Compte de résultat, Balance, Journaux, TVA, et le Grand
    livre **filtré sur un compte**.
  - **Quand il se charge** : à l'ouverture de la page et à l'arrivée sur un onglet (un exercice est toujours choisi,
    D15) ; au choix d'un exercice dans le menu ; quand le focus quitte **l'ensemble** des deux dates, regroupées dans un
    `<fieldset>` (ou `role="group"`) nommé « Période », par `focusout` dont le `relatedTarget` est hors du groupe —
    passer de « Du » à « Au » ne déclenche rien ; à la touche Entrée dans un champ de date.
  - **À deux conditions** : les dates ont changé depuis la dernière requête de l'onglet, et la période est valide (début
    au plus tard à la fin, `from ≤ to`, la règle de l'API). **La dernière demande l'emporte** : les précédentes sont
    annulées (`AbortController`) et périmées (`invalidateInFlight`).
  - **Des dates tapées puis quittées par un clic sur un autre onglet** passent dans l'URL de cet onglet, qui se charge
    avec elles.
  - **Exception** : le Grand livre **non filtré**, tous comptes, garde un bouton (*Générer*), parce qu'il est lourd.
  - **États** : pendant le chargement, « Chargement… » en `role="status"` ; pendant un rechargement, l'ancien rapport
    reste affiché, grisé, avec `aria-busy="true"` ; sans ligne, « Aucune écriture sur cette période. ».
  - **Textes** : « Sélectionnez un exercice et cliquez sur Générer » disparaît ; le seul message qui reste sert au cas
    où aucun exercice n'existe (`reports-error-no-fiscal-year-available`).
  - **Le rapport périmé** de D15 (55-1a AC-16) devient le grisé du rechargement ; « Plusieurs » ou « Hors exercice » sur un
    onglet limité à un exercice : rien n'est chargé, un message dit pourquoi (`reports-nav-needs-one-fy`).
  - *Choix des fiches, posés au mainteneur* : le Grand livre filtré se charge aussi au choix du compte et de « comptes sans
    mouvement » ; la balance âgée (sans période) se charge à l'arrivée sur l'onglet ; les deux rapports par compte
    analytique, qui exigent un compte analytique et que la décision ne nomme pas, gardent *Générer* ; les exports suivent
    le rapport **affiché**, non les champs. (55-1a AC-20 à AC-26, 55-2 AC-11 et AC-12 amendés)
- **D17 — Le style des liens (décision de l'utilisateur et avis UX du 2026-11-26, soir).** Partout, tableaux compris : la
  couleur d'aujourd'hui **et** un soulignement léger, `underline decoration-indigo-700/40 underline-offset-2
  hover:decoration-current` (WCAG 1.4.1 : la couleur n'est pas le seul signe). Une constante partagée, `LINK_CLASS`
  (`text-indigo-700` et ce soulignement), dans `frontend/src/lib/shared/navigation/link-class.ts` ; `AccountLink` la porte,
  l'Epic 56 la reprend. Les liens de l'application qui ne sont **pas soulignés au repos** (inventaire clos au T0 de la
  55-2) prennent le même soulignement léger en gardant leur couleur (`LINK_UNDERLINE_CLASS`, `decoration-current/40`) ;
  la variante `link` de `components/ui/button` aussi. Les liens déjà soulignés en plein restent tels quels (question au
  mainteneur). (55-2 AC-1 amendé, AC-16)
- **D7 — TVA : les comptes désignés (Q3, acceptée par le mainteneur le 2026-11-25).** Le décompte TVA ne montre aucun compte. La 55-2 ajoute une ligne « Comptes au
  grand livre : TVA due `NNNN` · TVA récupérable `MMMM` » (réglages de facturation, lisibles par tout rôle), chaque numéro
  en lien vers son grand livre sur la période du décompte ; compte non désigné : sa mention est omise — **sans compte désigné, aucun lien**. Une ligne à part, et
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
- **D12 — Afficher n'est pas figer (réponse de l'utilisateur du 2026-11-25, ancienne Q7 ; récrite le 2026-11-26 sur
  décision du mainteneur).** Depuis le Journal et les Rapports, afficher une facture **ne la fige pas**, et l'Epic 55
  n'a **aucun aperçu** : une facture **figée** s'ouvre par son PDF figé (`GET /api/v1/invoices/{id}/pdf`, qui sert le
  fichier figé sans rien écrire) ; une facture **pas encore émise** est signalée « PDF pas encore émis », sans aucune
  lecture de PDF (`/pdf` la figerait). La liste porte l'état du PDF (`pdfStatus`, `frozen` ou `notFrozen`, énumération
  ouverte à l'aperçu à venir et au PDF externe). Une validée non figée dont un PDF est déjà sorti (`emailed_at`, empreinte
  d'audit) n'est pas figée pour l'Epic 55 : « PDF pas encore émis », texte sans nuance (il ne dit pas que la facture n'a
  jamais été envoyée). Aucune route de PDF neuve, aucune dérogation à l'arbitrage 8 de la 25-6-b. (55-3a, 55-3b)
- **D13 — La marque d'un PDF non émis (Q10) : reportée** à l'epic « PDF marqués » (décision du mainteneur du
  2026-11-26) ; texte gardé à la 55-3a, section *Reporté à l'epic « PDF marqués »*.
- **D14 — La fiche de la facture garde son bouton (Q8, refusée par le mainteneur le 2026-11-25).** Le téléchargement depuis
  la fiche fige le PDF au premier téléchargement, comme aujourd'hui : c'est le **geste d'émission**. La fiche n'a pas
  d'aperçu. (55-3a, 55-3b)

## Stories

| Clé | Titre | Dépend de | Taille | Fiche |
|---|---|---|---|---|
| 55-0 | Kickoff (ce fichier ; pas de fiche) | — | — | — |
| 55-1a | L'état des Rapports dans l'URL : Retour, Suivant, lien partagé ; le sélecteur d'exercice (D15) ; le chargement sans *Générer* (D16) | base PR 46 | **L** (M avant D15, 2026-11-26 ; cinq modules, à la borne ; D16 n'ajoute aucun module) | `55-1a-etat-des-rapports-dans-l-url.md` |
| 55-1b | La présentation des rapports groupés dans l'URL (Epic 46) | 55-1a | S | `55-1b-presentation-dans-l-url.md` |
| 55-2 | Le lien de compte partout : Journaux, Grand livre, TVA, Compte de résultat à plat ; sélecteur d'exercice du Grand livre ; le style des liens dans toute l'application (D17) | 55-1a | M (+ balayage mécanique, D17) | `55-2-lien-de-compte-partout.md` |
| 55-3a | Les justificatifs d'une écriture dans la réponse du journal et du grand livre (serveur) | base PR 46 ; **backend** (Q1 acceptée) | **M** (XL avant que l'aperçu marqué ne sorte de l'epic, 2026-11-26) | `55-3a-justificatifs-serveur.md` |
| 55-3b | Ouvrir les justificatifs depuis le Journal (et le Grand livre) | 55-3a, 55-2 | M | `55-3b-justificatifs-ecran.md` |
| 55-4 | Solde de la dette de l'epic (plafond : **2 issues**) | toutes | S | `55-4-solde-dette-epic.md` |

Ordre : 55-1a → 55-1b → 55-2 → 55-3b → 55-4 ; la 55-3a (serveur, aucun fichier commun avec les autres stories) peut se
faire en parallèle de la 55-1a ou de la 55-2, avant la 55-3b.

## Gates

Règle de `CLAUDE.md` § *Test Locally First* (E2E complet au dernier commit de code de chaque story), **sauf la
dérogation ci-dessous**, accordée par le mainteneur le 2026-11-25 (Q6).

### Dérogation

Dérogation **partielle** à `CLAUDE.md` § *Test Locally First* (« L'E2E complet tourne au DERNIER COMMIT DE CODE de la
story », décision D7), décidée par le mainteneur le 2026-11-25, pour cet epic seulement :

- **les stories purement écran** — 55-1a, 55-1b, 55-2, 55-3b, et la 55-4 si elle ne touche pas le serveur — jouent **à
  chaque story**, au dernier commit de code : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`, et les **specs
  E2E des Rapports et de la navigation** (`reports*.spec.ts`, dont `reports-navigation.spec.ts` et
  `reports-entry-documents.spec.ts`, `homepage-charts.spec.ts`, `open-items.spec.ts`, `dunning-roundtrip.spec.ts`) ; **l'E2E
  complet et `scripts/gate-ci.sh <base>` tournent en fin d'epic** (55-4, T2 ; `<base>` est une base jetable
  `kesh_ci_<suffixe>`, non une référence git, `scripts/gate-ci.sh:8-10`), après le dernier commit de code, **avant la
  PR vers `main`** ;
- **la story des justificatifs qui touche le serveur, la 55-3a**, passe **à sa clôture** le gate complet (`cargo fmt
  --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo nextest` du workspace) **et l'E2E complet**,
  comme la règle le veut ; une 55-4 qui toucherait le serveur ferait de même ;
- **resserrement du 2026-11-26** (décision du mainteneur) : **à la clôture de chaque story écran qui touche un
  `messages.ftl` ou le manuel** (`.tex`) — 55-1a, 55-1b, 55-2, 55-3b, et la 55-4 si elle les touche —, on joue aussi
  `cargo nextest run -p kesh-i18n` et `cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` (gardes Rust des
  catalogues et des textes : parité des quatre locales, `kesh-i18n/src/loader.rs` ; `textes_coherents`), sans attendre la
  fin d'epic ;
- **si l'E2E de fin d'epic rougit** (hors des échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier
  par fichier), **l'epic n'entre pas dans `main`** avant que la story fautive soit trouvée et corrigée, et les gates de
  fin d'epic rejoués.

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
`general_ledger.rs`, `documents.rs` neuf : une définition des rattachements, partagée) et `kesh-api` (un service et une
route de lecture, `GET /journal-entries/{id}/documents`, qui porte `pdfStatus`), sans migration, sans changement des
exports PDF et CSV ni du PDF émis ; le contrat JSON du journal et du grand livre gagne un booléen **additif**. Les 55-1a,
55-1b, 55-2, 55-3b et 55-4 restent frontend seulement. **Accepté par le mainteneur le 2026-11-25 (Q1).** L'aperçu sans
figement et la marque (`kesh-qrbill`, `kesh-i18n`, fichiers du PDF de la facture) **sont sortis de l'epic le 2026-11-26**
(epic « PDF marqués ») : ni `kesh-qrbill` ni le PDF de la facture ne sont touchés.

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
| `frontend/src/lib/features/reports/JournalReportView.svelte`, `GeneralLedgerView.svelte`, `VatReportView.svelte`, `ReportSelector.svelte` | 55-1a (`ReportSelector` : le sélecteur d'exercice de tous les onglets, D15 ; en-tête de période dans les six vues, `ReportPeriodHeading`), 55-2 (Grand livre branché sur `ReportSelector`), 55-3b | — | 55-1a, puis 55-2, puis 55-3b |
| `frontend/src/lib/features/reports/reports-url.ts`, `fiscal-year-menu.ts`, `ReportPeriodHeading.svelte` (neufs, 55-1a), `AccountLink.svelte` (neuf), `EntryDocuments.svelte`, `EntryDocumentsViewer.svelte` (neufs) | 55-1a, 55-1b, 55-2, 55-3b | — | — |
| `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` | 55-3b (propriété facultative `errorMessage`) | — | comportement d'aujourd'hui inchangé sans la propriété |
| `crates/kesh-report/src/journal_report.rs`, `general_ledger.rs`, `documents.rs` (neuf) ; `crates/kesh-api/src/lib.rs`, `routes/journal_entries.rs`, `services/ecritures.rs` (service et route de lecture) ; tests neufs `crates/kesh-report/tests/documents.rs`, `crates/kesh-api/tests/journal_entry_documents_e2e.rs` | 55-3a | ni l'Epic 46 ni l'Epic 47 n'y touchent | champ additif, exports inchangés |
| `frontend/src/routes/(app)/+layout.svelte`, en-tête, titre de l'onglet | **aucune** story de l'epic ne les touche ; la page des Rapports porte son propre `<title>` (`reports/+page.svelte:618-620` à `1e23edb55`), que l'epic ne change pas | **#1582** (`fbfa64a27`, à `main` depuis le kickoff ; commit absent du worktree, non lu) | à la reprise de `main` : si #1582 touche aussi le `<title>` ou le `<svelte:head>` de `reports/+page.svelte`, le second arrivé garde les deux intentions |
| `crates/kesh-report/src/lib.rs` | 55-3a (`pub mod documents`) | **Epic 46** (+25 lignes, `pub mod`/`pub use`), **Epic 47** | une ligne, dans l'ordre alphabétique des modules |
| `crates/kesh-report/src/csv.rs`, `pdf.rs`, `benches/export.rs` (littéraux de test seulement) | 55-3a | **Epic 46** (PR 46 : `csv.rs` 1119, `pdf.rs` 1776 lignes touchées), **Epic 47** (`csv.rs`, `pdf.rs`, `presentation.rs`) | une ligne `has_documents: false,` par littéral, rien d'autre |
| `docs/api-external.md` (grand livre `:310`, journal `:1164` à `1e23edb55`) | 55-3a | **Epic 46** (+74 lignes vers `:313`), **Epic 47** (vers `:308-311`) | phrase ajoutée en fin de paragraphe, lignes relevées sur la base |
| `crates/kesh-api/tests/reports_e2e.rs` | 55-3a | Epics 46 et 47 (cités par le fichier de l'Epic 46) | bloc de tests propre |
| `crates/kesh-i18n/locales/*/messages.ftl` | 55-1a (dix clés du sélecteur, D15 ; D16 : `reports-nav-stale` et les invites retirées, `reports-loading`, `reports-error-no-entries-in-period` et `reports-ledger-instruction` récrites ; le groupe « Période » reprend `reports-filter-period`, existante), 55-2, 55-3b | Epic 46 (`reports-groups-*`) | bloc `reports-nav-*` propre à l'epic |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 55-1a, 55-2, 55-3b | tous | recompté par la garde après chaque reprise |
| `frontend/tests/e2e/reports.spec.ts`, `homepage-charts.spec.ts`, `open-items.spec.ts`, `dunning-roundtrip.spec.ts` | 55-1a (comportement de génération, D3) | Epic 46 (`reports-groupes*.spec.ts`) | E2E neufs : `reports-navigation.spec.ts` (55-1a, 55-1b, 55-2), `reports-entry-documents.spec.ts` (55-3b) |
| `docs/manual/fr/user-manual.tex` § *Rapports comptables* (`:4101`-`:4476` à `1e23edb55`) | 55-1a, 55-1b, 55-2, 55-3b (et § 11.6.1 *Le PDF émis est figé*, `:1635` : une phrase, l'affichage depuis les rapports ne fige pas et une facture pas encore émise y est signalée « PDF pas encore émis », 55-3b) | Epic 46 (§ *Bilan*, *Compte de résultat*, *Balance*) | `.tex` seul, PDF à la release |
| `frontend/src/lib/shared/navigation/link-class.ts` (neuf : `LINK_CLASS`, `LINK_UNDERLINE_CLASS`) | 55-2 (D17) | **Epic 56** (le reprend ; sa fiche 56-1 le plaçait dans `highlight.ts`) | l'Epic 56 l'importe, sans copie |
| Liens non soulignés au repos hors des Rapports (relevé du 2026-11-26 à `fbfa64a27`, inventaire clos au T0 de la 55-2) : `lib/features/auth-recovery/ForgotPasswordForm.svelte:164`, `ResetPasswordForm.svelte:243`, `routes/login/+page.svelte:197`, `routes/(app)/supplier-invoices/new/+page.svelte:193,223`, `supplier-invoices/[id]/edit/+page.svelte:144`, `lib/features/bank-import/BankProfileList.svelte:83`, `routes/(app)/invoices/[id]/+page.svelte:979`, `lib/components/ui/button/button.svelte:21` (variante `link`) | 55-2 (D17, une classe par site) | **Epic 56** (`invoices/[id]`), **Epic 39** (`bank-import`), **Epic 53** (saisie, non ces lignes) | une classe changée par site ; le second arrivé garde la classe neuve |
| `frontend/tests/e2e/*.spec.ts` qui cliquent *Générer* sur un rapport à période (relevé au T0 de la 55-1a : `grep -rln "reports-generate\|ledger-generate\|name: 'Générer'" frontend/tests/e2e`) | 55-1a (D16) | Epic 46 (`reports-groupes*.spec.ts`), Epic 47 | le clic disparaît ou devient la sortie des dates ; adaptation nommée au Change Log |
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
  figé » ; route d'aperçu neuve (55-3a, 55-3b). *(Revu le 2026-11-26 par le mainteneur : sans aperçu dans l'Epic 55 ; une
  facture pas encore émise est signalée « PDF pas encore émis », D12.)*
- **D11** : une écriture n'a le plus souvent qu'une facture ; avec un seul justificatif, aucune liste (55-3b).

## Décisions du mainteneur (2026-11-25)

Versées aux décisions D7, D12 à D14, aux sections *Backend*, *Gates* (*Dérogation*) et *Hors périmètre*, et aux fiches.
*(Q1, pour l'aperçu, et Q10 sont revues par les décisions du 2026-11-26 ci-dessous : l'aperçu marqué sort de l'epic.)*

- **Q1 — acceptée, avec conditions.** `hasDocuments` et la route qui liste les justificatifs d'une écriture : acceptés tels
  quels. L'aperçu `GET /invoices/{id}/pdf/preview` : accepté si le PDF **porte lui-même** la marque de Q10 sur chaque page.
  Une facture **déjà figée** : l'aperçu montre le PDF figé **tel quel, sans filigrane**. L'aperçu non figé ne sert qu'aux
  factures **validées dont le PDF n'est jamais sorti**. (55-3a, 55-3b ; sa lecture au code : question *Aperçu d'un PDF
  déjà sorti*)
- **Q10 — la marque dans le PDF d'aperçu non figé** : filigrane « APERÇU » en diagonale, nettement moins foncé que le
  texte ; en-tête « Aperçu — non émis » ; QR-facture barrée et non payable ; un seul mécanisme pour « APERÇU » et
  « BROUILLON » ; capture d'une facture de plusieurs lignes (D13 ; 55-3a).
- **Q3 — oui** : le lien de la TVA passe par les comptes désignés des paramètres de la TVA ; sans compte désigné, aucun
  lien (D7 ; 55-2).
- **Q5 — oui** : issue c) en P4, `enhancement`.
- **Q6 — dérogation partielle** (§ *Gates*, *Dérogation*).
- **Q8 — non** : la fiche de la facture garde son bouton, qui fige au premier téléchargement : c'est le geste d'émission
  (D14).
- **Q9 — oui, un epic à part** (numéro à confirmer), qui dépend de l'Epic 55 (§ *Hors périmètre*).
- **PDF produit hors de Kesh** (évolution, non livrée) : le PDF fourni est la pièce émise — copié, sauvegardé, figé, jamais
  régénéré ; ni second PDF ni QR-facture de Kesh ; « lien seulement » sur demande seulement (§ *Hors périmètre* ; 55-3a ;
  issue e).

## Décisions du mainteneur (2026-11-26)

Versées aux décisions D12 à D14, à l'en-tête (*Migration*, *Backend*), aux sections *Objectif*, *Hors périmètre*, *Backend*,
*Gates* (*Dérogation*), *Stories* et *Fichiers communs*, et aux fiches :

1. **Les PDF marqués sortent de l'Epic 55** et forment un petit epic à part, « PDF marqués » (numéro à confirmer) : le
   filigrane, l'en-tête et la QR-facture barrée dans `kesh-qrbill` ; l'aperçu « APERÇU — non émis » ; le PDF du
   brouillon. Retirés de la 55-3a (route `GET /invoices/{id}/pdf/preview`, marque, AC-9, AC-11, AC-12, T2quater, recette
   visuelle) et rangés en section *Reporté à l'epic « PDF marqués »* (55-3a, 55-3b), sans être supprimés. L'Epic 55
   garde la navigation, `hasDocuments`, la route qui liste les justificatifs d'une écriture et l'icône : une facture
   **figée** ouvre son PDF figé ; une facture **pas encore émise** affiche « PDF pas encore émis », sans aperçu, tant que
   l'epic « PDF marqués » n'est pas livré (texte aux quatre locales, 55-3b AC-5, AC-8). La 55-3a passe de XL à **M** ;
   `pdfStatus` garde ses deux valeurs, énumération ouverte.
2. **Un PDF déjà sorti sans être figé** (`emailed_at`, ou une empreinte au journal d'audit) : décidé pour l'epic « PDF
   marqués » — il recevra l'aperçu marqué, sous un bandeau « Un PDF de cette facture est sorti le … sans être gardé ;
   ceci est un aperçu. » (55-3a, section reportée ; tranche l'ancienne question *Aperçu d'un PDF déjà sorti*). Pour
   l'Epic 55, une telle facture n'est pas figée : « PDF pas encore émis » ; **aucune nuance de texte**, la phrase de la
   55-3b ne disant pas que la facture n'a jamais été envoyée (55-3a, *Afficher sans figer, dans l'Epic 55*).
3. **Ligne surlignée** : elle revient à l'epic à part des liens de compte (le 56 proposé), et seulement depuis une
   écriture, une facture ou une transaction ; ce n'est pas un livrable de l'Epic 55 (§ *Hors périmètre* ; 55-2,
   C-55-2-4).
4. **Resserrement de la dérogation Q6** : à la clôture de chaque story écran qui touche `messages.ftl` ou le manuel,
   `cargo nextest run -p kesh-i18n` et `binary(textes_coherents)` (§ *Dérogation* ; 55-1a, 55-1b, 55-2 R-2, 55-3b R-3,
   55-4). Tranche l'ancienne question *Gardes Rust des textes*.
5. Les questions tranchées sont retirées ; reste la numérotation.

## Décisions de l'utilisateur et de l'UX (2026-11-26, soir)

Prises avec l'experte UX le 2026-11-26, transmises par le coordinateur ; versées à D2, D3, D15, D16, D17, à l'*Objectif*,
aux *Stories*, aux *Fichiers communs* et aux fiches 55-1a (AC-20 à AC-26, R-14 à R-16), 55-1b (une note), 55-2 (AC-1
amendé, AC-11 et AC-12 amendés, AC-16), 55-3b (une note). Les fiches sont au plafond de huit passes : une passe ciblée
les relit, hors décompte, et ses points au-dessus de LOW vont au **T0** (journal
`kesh-gate-logs/s55-56-decisions/`).

1. **Plus de bouton *Générer*** pour les rapports à période, sauf le Grand livre de tous les comptes (D16).
2. **Style des liens** partout, tableaux compris (D17).

## Questions au mainteneur

- **D16, choix des fiches à confirmer** : (a) le Grand livre filtré se charge aussi au choix du compte et de « comptes
  sans mouvement » ; (b) la balance âgée se charge à l'arrivée sur l'onglet ; (c) les rapports par compte analytique
  gardent *Générer* (la décision ne les nomme pas, et ils exigent un compte analytique) ; (d) « Plusieurs » ou « Hors
  exercice » sur un onglet à un exercice : l'ancien rapport est **retiré** de l'écran, remplacé par le message (et non
  laissé grisé, ce qui le ferait passer pour un rechargement) ; (e) les exports suivent le rapport affiché. *Défaut* : oui
  aux cinq.
- **D17, portée du balayage** : la 55-2 souligne les liens **non soulignés au repos** de toute l'application (neuf sites,
  une classe chacun, hors des Rapports) ; ils touchent six modules de plus, en changements d'une classe. *Défaut* : les
  accepter comme balayage mécanique de la 55-2 (règle de splitting : la sous-story de déploiement « strictement
  mécanique », revue fichier par fichier). Les liens déjà soulignés en plein (`text-primary underline`, une trentaine)
  sont conformes et restent tels quels ; les passer au soulignement léger serait une harmonisation à part. *Défaut* :
  non, dans cet epic.
- **Passe ciblée D16 et D17 du 2026-11-26 au soir** (Opus, `kesh-gate-logs/s55-56-decisions/epic-55-d16-d17-validation.md`, fichier d'epic 0/0/1/1, 55-1a 0/1/9/5, 55-2
  0/0/1/3, 55-1b et sprint-status 0/0/0/0, 55-3b 0/0/0/1), aucune remédiation, points au T0 (55-1a R-16 à R-26, 55-2 R-7).
  **À trancher par vous avant le T0 de la 55-1a** : (a) le HIGH R-16 — annuler les requêtes demande de toucher
  `lib/shared/utils/api-client.ts` (un sixième module, au-delà de la borne) ; *défaut* proposé : se contenter de périmer
  la réponse (`invalidateInFlight`), sans annulation réseau ; (b) le Grand livre ne reçoit pas les dates tapées sur un
  onglet classique (R-21) ; (c) relancer un chargement en erreur (R-25) ; (d) la multiplication des traces d'audit et le
  chargement par les flèches (R-24). La liste des choix de D16 est à réunir en une seule (R-26, DE-11).

- **Sélecteur d'exercice, version de l'utilisateur et de l'UX : d'accord ?** (D15 ; 55-1a AC-14 à AC-19, 55-2 AC-11 ;
  démonstration et captures : `kesh-gate-logs/s55-selecteur/selecteur-exercice-demo.html`). Avec, à confirmer en même
  temps : *Générer* désactivé sur « Plusieurs » et « Hors exercice » hors du Grand livre (C-55-1-8), et la 55-1a portée de
  M à L (cinq modules, à la borne) plutôt qu'une septième story.
  *Passe ciblée D15 du 2026-11-26* (`kesh-gate-logs/s55-selecteur/selecteur-2026-11-26-validation.md`, 0/0/3/0 pour ce fichier) : le décompte des modules est bas (six avec
  `i18n-keys.test.ts`, SE-6) — le vrai choix est une story séparée avec dérogation à la borne de six stories, ou la 55-1a en
  L ; d'autres choix des fiches sont à confirmer avec D15 (exports désactivés quand le rapport est périmé, compte du Grand
  livre dans le périmé, en-tête « (hors exercice) », message `reports-nav-needs-one-fy`, SE-5) ; D5 reste à aligner sur
  l'AC-15 (SE-1). Renvoyés au T0 de la 55-1a (R-9, R-12, R-13).

- **Numérotation** — à confirmer par l'utilisateur : le **56** pour l'epic à part des liens de compte (Q9) ; un numéro
  pour l'epic « PDF marqués » (esquisse hors du dépôt, `kesh-gate-logs/s55-decisions-2/epic-pdf-marques-esquisse.md`).
