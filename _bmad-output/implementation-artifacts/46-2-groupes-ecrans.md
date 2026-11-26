# Story 46-2 : écrans du bilan et du compte de résultat par groupes

Status: done

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D3, D4, D9 (étendue), Q2, Q3, Q10 ; FR-46-2,
FR-46-7 à FR-46-10 ; NFR-46-1, NFR-46-5. Release visée **0.21.0** (prévision). Approuvée par le responsable de l'epic et par Guy (revue 1,
2026-11-19) ; fiche écrite le 2026-11-20, à valider en boucle. **Suit la 46-1** (contrat de l'API) **et la 46-3** (export groupé). Aucune migration.

## Story

En tant que comptable,
je veux voir à l'écran le bilan et le compte de résultat dans la hiérarchie de mon plan, ouvrir et fermer les groupes,
choisir un niveau de détail et la place des sous-totaux,
afin de lire mes comptes d'un coup d'œil au niveau qui m'intéresse, et d'exporter exactement ce que je vois.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend/src/lib/features/reports` (vues, types, appels, composant neuf), `frontend/src/routes/(app)/reports`
(barre de contrôles, export), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex`, `frontend/tests/e2e`.
Aucun code Rust : la 46-1 rend déjà `groupedRows` ; l'export groupé est la 46-3, que cette story **suppose fusionnée**
(ordre fixé : 46-1 → 46-3 → 46-2, fichier d'epic ; aucun export groupé masqué derrière un drapeau). Avant la 46-3,
l'export recevrait les paramètres **sans les lire** — `ExportQuery` (`routes/reports.rs:174`) ignore les paramètres
inconnus, il ne les refuse pas en `400` — et rendrait le PDF à plat d'un écran groupé, sans rien signaler.

## Critères d'acceptation

1. **AC-1 — Hiérarchie montrée par défaut (Q10).** À la première visite, le bilan et le compte de résultat s'affichent
   **groupés**, niveau « Tous les comptes » (`detailLevel=all` : tous les groupes ouverts), sous-totaux sur la ligne du
   titre (`header`, décision 5). C'est un changement de l'écran d'aujourd'hui, voulu par le responsable de l'epic ; les E2E existants qui
   lisent le bilan ou le compte de résultat (`frontend/tests/e2e/reports.spec.ts` — dont les deux tests axe, « empty
   state » et « populated state », qui contrôlent désormais l'écran groupé et ses accordéons —,
   `reports-currency.spec.ts`, `reports-export-pdf.spec.ts`, dont l'export d'un bilan vide part avec
   `detailLevel=all` et doit rendre le message « rapport vide ») passent en « hiérarchie masquée » ou sont adaptés,
   sans perdre aucune assertion ; les tests axe restent sur l'écran par défaut (groupé), à zéro violation.
2. **AC-2 — Contrôles.** Au-dessus du bilan et du compte de résultat (pas des autres rapports) :
   - **« Hiérarchie montrée / masquée »** : masquée = l'écran **à plat d'aujourd'hui**, requête **sans** `detailLevel`,
     sous-titres de rôle des fonds propres compris (D4) ; montrée = l'écran groupé. *(Ce contrôle tient lieu de l'entrée
     « À plat » du sélecteur de D9 ; lecture décidée par le responsable de l'epic le 2026-11-20, C-46-2-2.)* Niveau, position et ouvertures sont gardés pendant que la hiérarchie est masquée ;
   - **sélecteur de niveau** « Niveau 1 · … · Niveau n−1 · Tous les comptes », `n` = `grouping.maxDepth` de la dernière
     réponse (profondeur de l'arbre, indépendante de l'ouverture, 46-1), niveaux bornés à 20 ; désactivé quand la
     hiérarchie est masquée, et quand `maxDepth` ≤ 1 (plan sans groupe, rapport vide : seul « Tous les comptes ») ;
   - **« Tout ouvrir »** = niveau « Tous les comptes » et listes manuelles vidées ; **« Tout fermer »** = niveau 1 et
     listes vidées ;
   - **« Sous-totaux : sur le titre / en pied »** (`header` / `footer`).
   Contrôles accessibles au clavier, libellés et `aria-expanded` sur chaque accordéon.
3. **AC-3 — Accordéons et règle d'ouverture (remarque 4 de Guy).** Chaque ligne `group` porte un accordéon. Le niveau
   fixe l'état de départ ; un clic **prime** ensuite : ouvrir un groupe fermé par le niveau l'ajoute à `open` (et le
   retire de `closed`), fermer un groupe ouvert l'ajoute à `closed` (et le retire de `open`). Changer de niveau vide
   les deux listes. L'écran **redemande** le rapport à l'API avec `detailLevel`, `subtotalPosition`, `open`, `closed`
   (une seule source de la règle : la 46-1 ; aucun repli calculé dans le navigateur). Requêtes concurrentes : seule la
   réponse de la dernière demande s'affiche ; pendant le chargement, l'écran garde l'affichage précédent (sans
   clignotement, `aria-busy`). Chaque redemande écrit une trace `report.generated` au journal d'audit, comme un
   changement de période aujourd'hui (46-1 AC-11) : assumé (C-46-2-1), **quel que soit l'état du stockage** du navigateur ; la vue est mémorisée dans la mesure où le navigateur le permet (AC-10).
   *Note du 2026-11-22 (Story 46-7, LOW-1) : depuis la 46-7, un clic qui rend au groupe l'état que lui donne le niveau le retire des deux listes au lieu de l'y inscrire ; `openByLevel` (`reports-grouping.ts`) reprend pour cela la règle `est_ouvert` de la 46-1, gardée par une table de cas partagée avec le Rust. Ce n'est qu'une mémoire de clic minimale : l'état affiché reste celui que l'API rend, et C-46-2-1 (« aucun repli calculé dans le navigateur ») vaut pour l'affichage.*
4. **AC-4 — Rendu (D5, Q2, Q3).** Lignes rendues dans l'ordre de `groupedRows` : retrait par `depth` ; `group` en gras,
   son montant **selon la règle de rendu de la 46-1** (sur le titre en `header` ou si fermé ; sinon sur la ligne
   `subtotal`, en gras, libellé « Total <nom du groupe> ») ; `own` libellée « <numéro> <nom> (solde propre) » ; solde en
   devise d'une ligne `group`/`subtotal` quand l'API le donne (devise + CHF), rien sinon (aucune ligne ajoutée) ; lien
   vers le grand livre (`ledgerHref`) sur les lignes `account` et `own` seulement. Totaux de section, lignes calculées
   du bilan et équation : à leur place d'aujourd'hui. Montants par `formatReportAmount` / `formatCurrencyFigure`
   (arrondi au centime de `roundToCents`, `reports.api.ts:306`), aucun formatage neuf.
5. **AC-5 — Mémoire par visiteur.** Hiérarchie montrée ou masquée, niveau, position, et les listes `open`/`closed`
   **par rapport** (bilan, compte de résultat) sont retenus en `localStorage`, toute lecture et écriture sous
   `try`/`catch`, l'écran rendu par défaut (AC-1) si le stockage est vide, illisible ou refusé. Une valeur relue
   invalide (niveau hors bornes, liste non numérique) est ignorée, non transmise. Clé, restauration avant la
   première requête et groupes disparus : AC-10 (décision du responsable de l'epic du 2026-11-20).
6. **AC-6 — Export fidèle à l'écran.** Les boutons PDF et CSV du bilan et du compte de résultat passent **les mêmes
   paramètres** que la dernière requête affichée (aucun si la hiérarchie est masquée) : `downloadReport` et
   `getReportExportUrl` (`reports.api.ts:334`, `:446`) gagnent un argument facultatif de groupement. Le rendu exporté
   est la 46-3.
7. **AC-7 — Affichage de travail (remarque 3 de Guy, lecture (a)).** Une phrase du manuel (AC-9) dit que les sections
   légales du bilan restent (Actifs, Dettes, Fonds propres) et qu'un affichage replié — au niveau 1 en particulier —
   est un affichage de travail, non des comptes annuels ; à l'écran, une info-bulle discrète sur le sélecteur de
   niveau le rappelle.
8. **AC-8 — Maquette avant les écrans.** Une maquette (HTML statique, hors dépôt, `kesh-gate-logs/epic-46/`) des deux
   positions de sous-total, d'un groupe fermé par le niveau puis ouvert à la main, et des contrôles, est jointe au Dev
   Agent Record et montrée au responsable de l'epic avant le développement des vues.
9. **AC-9 — Textes, tests, manuel.** Libellés en quatre locales (bloc `reports-groups-*` propre à l'epic) ;
   `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts ; Vitest des vues (rendu `header`
   et `footer`, groupe fermé, `own`, devise de groupe, mémoire illisible, AC-10) et du composant ; un E2E Playwright
   (`reports-groupes.spec.ts`) traverse : écran groupé par défaut, niveau 1, ouverture d'un groupe à la main, export
   CSV dont l'URL porte les mêmes paramètres, hiérarchie masquée qui rend l'écran d'aujourd'hui, vue retrouvée au retour (AC-10) ; captures d'écran
   (thème clair) de chaque état au Dev Agent Record ; manuel utilisateur FR, § *Bilan* et *Compte de résultat*
   (`docs/manual/fr/user-manual.tex:3923`, `:3954`), `.tex` seul, PDF non commité.
10. **AC-10 — Vue retrouvée au retour (décision du responsable de l'epic du 2026-11-20).** Le rechargement à chaque clic (C-46-2-1)
    est accepté **quel que soit l'état du stockage** ; l'écran retrouve sa vue **dans la mesure où le navigateur le permet** (stockage refusé ou vide : vue par défaut, sans erreur, rechargement et trace inchangés). En revenant sur la page (même rapport, même société, même
    navigateur), l'écran rétablit le niveau choisi et les groupes ouverts ou fermés à la main (ainsi que
    montrée/masquée et la position, AC-5). Choix KISS : mémorisation côté navigateur en `localStorage`, **clé par
    société et par rapport** (par navigateur, non par utilisateur, comme `kesh-mode` ; l'identifiant de société est
    `company.id` de `fetchCompanyCurrent`, que `+page.ts` doit renvoyer à côté du nom aujourd'hui seul chargé), sous la
    convention de `lib/app/stores/mode.svelte.ts` (lecture et écriture sous
    `try`/`catch`, écran par défaut correct si le stockage est vide, illisible ou refusé) ; **restaurée avant la
    première requête** (la première demande part avec la vue mémorisée, sans passer d'abord par la vue par défaut) ;
    les groupes mémorisés qui n'existent plus dans le plan (compte archivé, groupe dissous) **sont transmis tels
    quels** : la première requête part avant que le plan soit connu, donc le client ne peut pas les filtrer ; l'API les
    ignore sans erreur et les retire de l'écho `grouping` (46-1, § *Le contrat*) ; après chaque réponse, la mémoire
    reprend les listes de l'écho, et ces identifiants disparaissent au prochain enregistrement ; une liste mémorisée de plus de 500 identifiants (limite de la 46-1, `400` au-delà) est ignorée à la lecture comme toute valeur invalide (AC-5). Tests : Vitest de la
    persistance (écriture au changement de niveau, de position, de hiérarchie montrée/masquée et au clic, relecture au
    montage, clé distincte par société et par rapport, stockage vide, illisible et refusé, groupe disparu retiré de la
    mémoire après la réponse de l'API, première requête déjà munie de la vue) ; E2E (`reports-groupes.spec.ts`) :
    choisir un niveau, ouvrir un groupe, quitter la page, revenir → même niveau, même groupe ouvert.

## Tâches / sous-tâches

- [x] **T0** — Reprise de `main` / de la branche d'epic ; vérifier le contrat de la 46-1 et l'export de la 46-3 tels
      que fusionnés (sinon arrêt, le dire au Change Log).
- [x] **T1 — maquette** (AC-8).
- [x] **T2 — types et appels** (AC 3, 6) : `reports.types.ts` (`Grouping`, `GroupedRow`, champs facultatifs des DTO),
      `reports.api.ts` (`getBalanceSheet`, `getIncomeStatement`, `downloadReport`, `getReportExportUrl` avec options).
- [x] **T3 — composant** `GroupedReportRows.svelte` (neuf, partagé par les deux vues et réutilisé par la 46-6b) :
      rendu des lignes, accordéons, règle de rendu (AC-4). Il se paramètre par un **descripteur de colonnes** (une
      colonne de montant ici, le solde en devise restant dans la cellule du libellé comme `AccountCurrencyBalance` aujourd'hui ; la 46-6b l'étend à quatre colonnes pour la balance), non par un composant par rapport.
- [x] **T4 — vues et page** (AC 1, 2, 5, 7, 10) : `BalanceSheetView.svelte`, `IncomeStatementView.svelte` (branche
      groupée, branche à plat inchangée), barre de contrôles dans `routes/(app)/reports/+page.svelte`, mémoire (clé par société et rapport, AC-10).
- [x] **T5 — i18n, Vitest, E2E, captures, manuel** (AC 9).
- [x] **T6 — gates** : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ; E2E complet au dernier commit de
      code.

## Dev Notes

### État du code (relevé le 2026-11-20 à `29b9ec27`)

- `frontend/src/lib/features/reports/BalanceSheetView.svelte` (283 lignes) : sections Actifs, Passifs, Capitaux
  propres groupés **par rôle** (`equityGroups`, ordre reçu du serveur), lignes calculées (`showsRetainedEarningsLine`),
  total des capitaux propres calculé en `Big`, `AccountCurrencyBalance` pour la devise, `ledgerHref` sur chaque compte.
- `IncomeStatementView.svelte` (113 lignes), `TrialBalanceView.svelte` (138 ; 46-6b).
- `reports.api.ts` : `buildQuery` (`:26`), `getBalanceSheet` (`:36`), `getIncomeStatement` (`:45`), `isReportEmpty`
  (`:158`), `formatReportAmount` (`:280`), `roundToCents` (`:306`), `getReportExportUrl` (`:334`), `downloadReport`
  (`:446`). `routes/(app)/reports/+page.svelte` (893 lignes) : onglets, période, export (`:455-480`).
- `localStorage` : précédents sous `lib/app/stores/` (`mode.svelte.ts`, lecture/écriture protégées ; `session-idle.svelte.ts`, sonde d'écriture) — convention reprise par l'AC-10 (relevé à `origin/main`, 2026-11-20).

### Ce qui doit rester

La vue à plat **identique** (hiérarchie masquée) : sous-titres de rôle, ordre des fonds propres, lignes calculées,
garde « rapport vide » ; les autres onglets ; l'export sans paramètre.

### Choix et écarts

- **C-46-2-1** : l'écran redemande à chaque changement plutôt que de replier localement — la règle d'ouverture n'existe
  qu'en Rust (DRY), et l'export reprend les paramètres de la dernière requête, ce qui garantit la fidélité (46-3).
  *Coût accepté* (validation P1) : un aller-retour par clic, et une trace d'audit par redemande. *Écarté* : replier
  dans le navigateur une réponse `all` — il faudrait y recopier la règle d'ouverture, celle du montant (titre ou pied
  selon l'ouverture) et celle des groupes nuls masqués (46-1 AC-5), et l'export ne serait plus fidèle que si les deux
  copies s'accordent.
- **C-46-2-2** : « hiérarchie masquée » = vue à plat d'aujourd'hui (**décidé par le responsable de l'epic le 2026-11-20** ; retenu
  d'abord à la validation P1 : c'est la lecture qui ne double pas l'entrée « À plat » de D9, et le contrôle du responsable de l'epic
  la remplace).
- **C-46-2-3** : « tout ouvrir / tout fermer » passe par le niveau, non par la liste de tous les groupes (URL courte).

### References

- Fiche 46-1 (§ *Le contrat*) ; fichier d'epic ; `CLAUDE.md` § *Recette visuelle* (captures, thème clair), § *Le prompt
  d'une passe doit nommer le manuel* (le PDF compilé localement se contrôle à la revue).

## Note de version

**Added** — Le bilan et le compte de résultat s'affichent dans la **hiérarchie de votre plan comptable** : chaque
groupe avec son sous-total, ouvert ou fermé d'un clic, un **niveau de détail** (niveau 1, 2… ou tous les comptes),
les sous-totaux **sur le titre ou en pied** ; la case « Hiérarchie des groupes » décochée rend la liste à plat
d'avant. Le navigateur retient la présentation de chaque rapport, société par société, et les exports PDF et CSV
reçoivent les mêmes réglages que l'écran.

**Changed** — L'export CSV ou PDF du bilan et du compte de résultat lancé depuis l'écran suit la hiérarchie affichée
(par défaut, tous les comptes) : le CSV porte les colonnes `Niveau;TypeLigne` et les sous-totaux de groupe (sur la ligne du titre par défaut, en lignes `soustotal` en pied). Mais
le CSV groupé (bilan, compte de résultat ou balance) ne s'additionne pas tel quel : le titre d'un groupe (en pied, sa ligne de sous-total) répète les montants de ses comptes, et un groupe fermé n'y porte que son sous-total. Chaque section du bilan et du compte de résultat porte sa ligne de total, la balance sa ligne `Total` ; pour additionner les montants dans un tableur, décocher « Hiérarchie des groupes » pour l'export à plat d'avant, qui porte lui aussi ses lignes de total (à filtrer par `Section` au bilan et au compte de résultat, à écarter à la balance : ligne `Total`) et n'a pas de colonne `TypeLigne` ; les montants groupés sont arrondis au centime.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (développement), contexte de la story seul.

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s46-2/` : `gate-backend.log` (gate final), `gate-backend-1.log` (premier gate,
rouge sur la garde allemande, voir plus bas), `gate-backend-2-bus.log` (refus de la session systemd utilisateur),
`front-*.log` (gate frontend final), `front-*-1.log` (premier passage, gardes i18n), `vitest-cible-1.log`,
`vitest-gardes-2.log`, `cible-i18n.log`, `mutations-1.log`, `mutations-2.log`, `e2e.log`, `e2e-backend.log`.

### Completion Notes List

**T0 — prémisses relevées au code (branche `epic/46-rapports-groupes`, `92434d16`).** Contrat de la 46-1 tel que
fusionné : `crates/kesh-report/src/groupes.rs` — `LigneRapport` (`kind` `group`/`account`/`own`/`subtotal`,
`depth`, `accountId`, `accountNumber`, `accountName`, `balance`, `currency?`, `balanceCurrency?`, `open?`, `active?`,
`role?`), `Grouping` (`detailLevel` entier ou `"all"`, `subtotalPosition`, `maxDepth`, `open`, `closed`),
`AvecGroupes` (`#[serde(flatten)]`, `grouping` et `groupedRows` absents sans `detailLevel`) ; refus d'une liste
vide (`services/rapports.rs:337-351`, « » n'est pas un entier) — d'où l'omission des listes vides côté client. La
46-3 n'est **pas** fusionnée (branche `story/46-3`, `fc538d4a`, en cours) : l'export groupé est branché sur les
paramètres que sa fiche fixe (AC-1 : mêmes noms et formes qu'aux routes JSON, `groupes_de(&paires)`), testés par
leurs URL (`reports.api.groupes.test.ts`, E2E étape 5) ; **il sera exercé de bout en bout (fichier rendu groupé)
après la fusion de la 46-3** — jusque-là, l'export reçoit les paramètres et rend le fichier à plat, ce que
`ExportQuery` ignore sans erreur. La 47-3 (miroirs) n'est pas dans la branche : aucune ligne propre au miroir à
l'écran (rien à faire, la 46-1 range le miroir comme un compte).

**T1 — maquette (AC-8)** : `kesh-gate-logs/epic-46/maquette-46-2.html` (contrôles, sous-totaux sur le titre et en
pied, groupe fermé par le niveau puis ouvert à la main). Elle n'a **pas** pu être montrée au responsable de l'epic
avant le développement des vues (travail de nuit, sans canal) : à lui montrer avec les captures ; écart à l'AC-8
déclaré.

**Réalisé.**
- `reports.types.ts` : `DetailLevel`, `SubtotalPosition`, `GroupingParams`, `GroupingEcho`, `GroupedRow`,
  `BalanceSheetGroupedRows`, `IncomeStatementGroupedRows`, champs facultatifs `grouping` / `groupedRows` des DTO.
- `reports.api.ts` : `groupingQueryParams` (une seule traduction vue → paramètres, listes vides omises),
  `getBalanceSheet` / `getIncomeStatement` / `getReportExportUrl` / `downloadReport` avec un argument facultatif de
  groupement (ignoré par l'URL d'export des autres rapports).
- `reports-grouping.ts` (neuf) : l'état de la vue, ses transitions (`toggleGroup`, `withDetailLevel`, `openAll`,
  `closeAll`, `withEcho`, `detailLevelOptions`) et la mémoire `localStorage` (clé
  `kesh-report-grouping:<société>:<rapport>`, lecture et écriture sous `try`/`catch`, relecture champ par champ,
  listes de plus de 500 ou en conflit ignorées). **Aucune règle d'ouverture** n'y est calculée (C-46-2-1).
- `GroupedReportRows.svelte` (neuf, T3) : lignes groupées paramétrées par un descripteur de colonnes
  (`GroupedAmountColumn`), règle de rendu de la 46-1 (`showsAmount`), retrait par profondeur borné à dix rangs
  (`indentOf`), sur le numéro **et** le libellé (un pied de groupe se lit à son rang), accordéon (`aria-expanded`,
  libellé « Ouvrir / Fermer le groupe <n> <nom> »), solde propre, devise d'un groupe là où son montant est rendu
  (`AccountCurrencyBalance` généralisé, `testIdPrefix`), lien vers le grand livre sur `account` et `own`, montants
  par `formatCurrencyFigure` (arrondi `roundToCents`, sans « -0.00 »).
- `GroupingControls.svelte` (neuf) : hiérarchie montrée/masquée (case), niveau (désactivé hiérarchie masquée, avant
  toute réponse groupée et pour `maxDepth` ≤ 1), « Tout ouvrir », « Tout fermer », sous-totaux ; info-bulle de
  l'AC-7 (`title` et description pour lecteur d'écran).
- `BalanceSheetView.svelte`, `IncomeStatementView.svelte` : branche groupée si la réponse porte `groupedRows`,
  branche à plat inchangée (sous-titres de rôle compris) ; totaux, lignes calculées, équation à leur place.
- `+page.ts` : renvoie `companyId` (`fetchCompanyCurrent`) ; `null` si la société n'est pas lue (vue non mémorisée).
- `+page.svelte` : vues relues **avant la première requête** ; `changeGrouping` mémorise et redemande (seulement si
  un rapport est affiché) ; `generate({ keepDisplay })` garde l'affichage pendant la redemande (`refreshing`,
  `aria-busy` sur le panneau, export désactivé), même garde de course `genSeq` ; à chaque réponse acceptée,
  `displayedGrouping` (paramètres de l'export, AC-6) et l'écho repris en mémoire (AC-10).
- i18n : bloc `reports-groups-*`, quinze clés, quatre locales.
- Manuel utilisateur FR (`.tex` seul) : § *Bilan* (encart art. 959a CO récrit : les groupes affichés sont ceux du
  plan, affichage replié = affichage de travail), § *Compte de résultat*, § neuf *Bilan et compte de résultat par
  groupes* (`sec:rapports-groupes`) ; capture `screenshots/rapport-bilan-groupes.png` ajoutée.

**Écarts et choix.**
- AC-7 : l'info-bulle et le manuel nomment les sections **comme l'écran** (Actifs, Passifs, Capitaux propres), non
  « Dettes, Fonds propres » de la fiche : un rappel doit parler la langue de l'écran qu'il commente.
- AC-4, lien vers le grand livre : la vue à plat du compte de résultat n'en a pas ; la vue groupée en porte sur les
  lignes `account` et `own`, comme l'AC le demande pour les deux rapports.
- « Tout ouvrir », « Tout fermer » et le sélecteur de sous-totaux sont désactivés hiérarchie masquée, comme le niveau
  (ils n'y auraient aucun effet visible ; les choix restent gardés).
- Allemand : « aufklappen / zuklappen » — « schliessen » nu est réservé aux panneaux
  (`la_cloture_d_exercice_ne_parle_pas_comme_un_panneau`, rouge au premier gate).
- E2E existants (AC-1) : `reports.spec.ts` (tests axe compris), `reports-currency.spec.ts`,
  `reports-export-pdf.spec.ts` n'ont pas été modifiés — relus, leurs sélecteurs tiennent sur l'écran groupé par
  défaut (`account-fx-<n>` porté par la ligne de compte groupée, « Total actifs », « Aucune écriture », boutons
  d'export ; le preset `with-company` n'a pas de groupe). **Non rejoués** (règle de la nuit du 22.11 : seule la spec
  ajoutée) ; à rejouer à l'E2E complet de la branche d'intégration.

**Tests ajoutés** (de `92434d16` à `HEAD`, recomptés par `grep -cE "^\s*it\("` / `^test(`) : Vitest **61** —
`reports-grouping.test.ts` 22, `GroupedReportRows.test.ts` 12, `reports-page-groupes.test.ts` 11,
`reports.api.groupes.test.ts` 9, `views-groupes.test.ts` 7 ; E2E **1** (`reports-groupes.spec.ts`, huit étapes).
Gardes mises à jour par leur relevé : `sitesTotal` 3499 → 3515, candidates de `i18n-libelle-en-dur` 130 → 131
(`conforme` 109 → 110).

**Mutations** (copie hors dépôt, `node_modules` réel copié, « tuée » seulement sur une assertion nommée,
`mutations.sh`) : 13 mutations, 13 tuées. Premier passage (`mutations-1.log`) : 11 tuées, M7 (affichage effacé
pendant la redemande) et M12 (exception du stockage non rattrapée) faisaient échouer des tests **sans** assertion
(élément introuvable, exception) : non comptées ; tests renforcés (`48b25a28`), rejouées (`mutations-2.log`) :
tuées. Une mutation équivalente retirée avant passage (liste vide envoyée : `buildQuery` omet déjà la chaîne
vide).

**Gates.** Backend (fmt, clippy `-D warnings`, nextest complet, base `kesh_s462` remise à zéro) au commit
`4654bdb2`, le dernier qui touche autre chose que le frontend étant `4a2f4537` (catalogue allemand) : **6196
exécutés, 6196 passés, 14 ignorés** (`gate-backend.log`). Lancé avec `KESH_NO_MEMGUARD=1` : la session systemd
utilisateur de la station refusait les connexions (« Failed to connect to user scope bus », `gate-backend-2-bus.log`) ;
`heavy.sh` gardait son plancher mémoire. Premier gate (`gate-backend-1.log`) : 1 échec, la garde allemande
ci-dessus, et un test instable passé au second essai (`recette_montee_refuse_ce_qui_sort_de_son_perimetre…`).
Frontend au commit `48b25a28` (dernier commit de code ; les suivants ne portent que la capture du manuel, le
`.tex` et des comptes rendus) : `npm run check` 0 erreur, 29 avertissements (aucun dans un fichier touché par la
story hors deux, préexistants, de `+page.svelte` : `data.fiscalYears[0]` et `lastFyId`, présents à `92434d16`) ;
`lint-i18n-ownership` vert ; Vitest **4688 passés, 283 fichiers** ; `npm run build` vert. Premier passage
(`front-*-1.log`) : trois gardes rouges, attendues — `sitesTotal`, `i18n-libelle-en-dur` (compteurs relevés,
ci-dessus) et `i18n-entrees-a-variables` (arguments de l'accordéon passés par une variable, que la garde ne lit
pas : écrits en ligne, `60739b7b`).
E2E **partiel** (règle de la nuit du 22.11 : seulement la spec ajoutée), au même commit, frontend construit et
binaire `kesh-api` de ce worktree, base `kesh_e2e_s462` neuve : `reports-groupes.spec.ts` **1 passé**
(`e2e.log`). E2E complet et `gate-ci.sh` : à la fin de l'epic, sur la branche d'intégration.
`gate-ci.sh` : non joué (fin d'epic, branche d'intégration).

**Captures** (thème clair, `kesh-gate-logs/s46-2/captures/`, prises par la spec E2E) : défaut tous les comptes,
sous-totaux en pied, niveau 1, groupe ouvert à la main, vue retrouvée au retour, hiérarchie masquée, compte de
résultat.

**Remédiation de la revue P1 (2026-11-22)** — rapport `kesh-gate-logs/s46-2/revue-p1-rapport.md`, 0 C / 0 H / 2 M /
6 L, tous remédiés (commits `7ec208c8`, `b7a53abb`, puis ce compte rendu).
- **M1** : `invalidateInFlight()` (`+page.svelte`) est désormais le seul endroit qui incrémente `genSeq` ; il
  rabaisse `loading` et `refreshing` (et oublie l'onglet en vol). Appelée par les quatre branches de `generate()`,
  l'effet de changement d'exercice et celui de changement de projet. Test du scénario S4 (redemande du bilan
  périmée par la balance âgée) et du changement d'exercice pendant une redemande.
- **M2** : (a) `changeGrouping` redemande aussi pendant le premier chargement du même rapport (`inFlightTab`), en
  chargement complet ; (b) `selectTab`, au retour sur le bilan ou le compte de résultat, redemande si les
  paramètres affichés (`displayedGrouping`) diffèrent de la vue (`sameGroupingParams`, neuf dans
  `reports-grouping.ts`) ; l'écho n'est repris que s'il répond à la vue en cours (garde défensive, inatteignable
  aujourd'hui : une première version y redemandait le rapport, mutation MG survivante — retirée, `b7a53abb`).
  Après un écho, `displayedGrouping` prend les listes de l'écho (même affichage, pas de redemande inutile au retour).
  Tests S1, S2, et « retour sur un onglet d'accord : rien n'est redemandé ».
- **L1** : S3 versée (redemande en erreur, export aux paramètres affichés) et « export désactivé pendant une
  redemande ». **L2** : `<label for>` court, info-bulle hors du `<label>` (`title` sur le conteneur, description par
  `aria-describedby`) ; test du nom accessible. **L3** : `BALANCE_COLUMNS` exporté du module de
  `GroupedReportRows.svelte`. **L4** : manuel — sélecteur inactif avant la première génération et hiérarchie
  masquée, *Tout ouvrir*, *Tout fermer*, *Sous-totaux* inactifs hiérarchie masquée, et changement pendant une
  génération en cours. **L6** : prénom du contributeur retiré de la fiche (« le responsable de l'epic »), grep des
  noms vide sur la fiche, le `.tex`, le code et la spec E2E ; `sprint-status.yaml` en porte encore, sur des lignes
  d'autres stories et la ligne `epic-46`, hors de la story, non touchées.
- **L5 — dépendance à la 46-3, à tenir à la fusion** : sur cette branche, l'export reçoit les paramètres groupés et
  rend le fichier à plat (`ExportQuery` les ignore), et le renvoi du manuel à §17.10 n'y trouve pas encore le
  paragraphe des groupes. **Ne pas fusionner la 46-2 dans une branche publiable sans la 46-3** ; après la fusion de la
  46-3, **rejouer `reports-export-pdf.spec.ts`** (un bilan vide exporté part désormais avec `detailLevel=all`).
- Tests ajoutés par la remédiation (de `d4760e3e` à `b7a53abb`, recomptés par `grep -cE "^\s*it\("`) : **11** —
  `reports-page-groupes.test.ts` 11 → 18, `reports-grouping.test.ts` 22 → 25, `views-groupes.test.ts` 7 → 8.
- Mutations (copie hors dépôt, `node_modules` réel, témoin vert 198 passés, « tuée » sur assertion nommée,
  `kesh-gate-logs/s46-2/mutations-p1rem/`, script `mut.sh`, journaux `mut-*.log`), jouées sur `7ec208c8` : 10 — MA (export pris sur la vue), MB (`refreshing` non rabaissé par
  `invalidateInFlight`), MC (`!refreshing` retiré de `canExport`), ME (pas de redemande pendant le premier
  chargement), MF (pas de redemande au retour), MH (`sameGroupingParams` sans `open`), MI (info-bulle dans un
  `<label>`), MJ (balance âgée revenue à `++genSeq`) **tuées** ; MG (redemande dans la garde de l'écho) **survivante**,
  chemin inatteignable, code retiré par `b7a53abb`, qui ne touche que cette garde : les neuf autres portent sur des
  lignes inchangées.
- Gates au dernier commit de code `b7a53abb` : voir ci-dessous (frontend complet, E2E de la spec de la story) ;
  aucun `.rs` touché, pas de gate Rust. Manuel : copie compilée (`latexmk -xelatex`, rc 0), PDF aplati et grepé,
  les trois phrases neuves présentes, aucun « ?? ».
- Gates de la remédiation, au dernier commit de code `b7a53abb` (journaux `kesh-gate-logs/s46-2/*-p1rem*`) :
  frontend complet — `npm run check` 0 erreur, 29 avertissements (le même nombre qu'au gate précédent),
  `lint-i18n-ownership` vert, Vitest **4699 passés, 283 fichiers** (garde `i18n-keys` comprise : `sitesTotal`
  recompté par la garde, inchangé à 3515, aucun appel `i18nMsg` ajouté ni retiré), `npm run build` vert ; E2E
  **partiel**, la seule spec de la story, `reports-groupes.spec.ts` **1 passé** (frontend reconstruit, binaire
  `kesh-api` du gate précédent : aucun `.rs` touché depuis), captures dans `captures-p1rem/`. Aucun gate Rust.

**Remédiation de la revue P2 (2026-11-22)** — rapport `kesh-gate-logs/s46-2/revue-p2-rapport.md`, 0 C / 0 H / 1 M /
4 L ; commit de code `623bb649`.
- **M1** : `resyncActiveGroupedTab()` (`+page.svelte`) redemande le bilan ou le compte de résultat actif, affiché
  avec une autre vue que celle des contrôles, quand aucune génération n'est en vol ; `selectTab` l'appelle au retour,
  et `afterGeneration(myTab)`, dans les quatre `finally` de `generate`, à la fin d'une génération non périmée d'un
  **autre** onglet. Le retour pendant le chargement d'un autre onglet ne périme donc pas ce chargement : la redemande
  attend sa fin. La génération de l'onglet actif lui-même ne redemande jamais (garde `finishedTab !== activeTab`) :
  une redemande en erreur ne boucle pas. Tests : X1 (bilan, niveau 1, *Générer* sur le compte de résultat, retour au
  bilan pendant le chargement → aucune redemande avant sa fin, une après), et « redemande en erreur : pas de
  redemande en boucle ». Manuel : l'écran *revient* à la présentation de la barre, à la fin de la génération d'un
  autre rapport ; en erreur, message, présentation précédente, *Générer* la redemande (L2 de la revue, dit au manuel).
- **L1** : trois tests, un par mutation survivante — « chargement périmé par un autre onglet : un changement de vue
  sans rapport affiché ne demande rien » (P2B), « export après un écho qui retire un groupe disparu » (P2C),
  « changement de vue pendant le premier chargement : chargement plein » (P2D). **L2** : noté (ci-dessus, manuel).
  **L3** : `BALANCE_COLUMNS` figée (`Object.freeze`, `readonly GroupedAmountColumn[]`, prop `columns` en `readonly`).
  **L4** : `title` porté par le seul ⓘ, retiré du conteneur.
- Tests ajoutés (de `9b5d2a18` à `623bb649`, `grep -cE "^\s*it\("`) : **5**, `reports-page-groupes.test.ts` 18 → 23.
- Mutations (copie `git archive` hors dépôt, `node_modules` réel, témoin vert **203 passés**, une à la fois,
  `kesh-gate-logs/s46-2/mutations-p2rem/`, `mut.sh`, `mut.console`, `mut-*.log`) : P2B, P2C, P2D, P2F, N1 (pas
  d'`afterGeneration` dans le `finally` des rapports classiques), N2 (garde `finishedTab !== activeTab` retirée),
  N3 (pas de resynchronisation dans `selectTab`) **tuées** sur assertion nommée ; P2E (`<label for>` retiré) tuée par
  « le sélecteur de niveau a un nom court… », dont le `getByRole` échoue (TestingLibraryElementError — le script la
  classe « ERREUR » faute d'`AssertionError`, jugée tuée : un test nommé rougit sur ce qu'il vérifie) ; **P2A**
  (garde de l'écho retirée) **survit**, chemin inatteignable déclaré depuis la P1.
- Gates au commit `623bb649` (journaux `kesh-gate-logs/s46-2/*-p2rem*`) : frontend complet — `npm run check`
  0 erreur, 29 avertissements (inchangé), `lint-i18n-ownership` vert, Vitest **4704 passés, 283 fichiers** (garde
  `i18n-keys` comprise, `sitesTotal` inchangé à 3515 : aucun appel `i18nMsg` ajouté ni retiré), `npm run build` vert ;
  E2E **partiel**, `reports-groupes.spec.ts` **1 passé** (frontend reconstruit, binaire `kesh-api` du gate précédent :
  aucun `.rs` touché), captures dans `captures-p2rem/`. Aucun gate Rust. Manuel : copie compilée (`latexmk -g
  -xelatex`, rc 0), PDF aplati : les deux phrases neuves présentes, aucun « ?? » (`user-manual-controle-p2rem.pdf`,
  hors dépôt).
- Revue P3 ciblée (`623bb649`, Claude Opus 5.5) : 0 C / 0 H / 0 M / 4 L, remédiés sans code de production.
  L1 : tests `describe.each` (balance âgée, grand livre, rapport par projet comme « autre chargement » d'X1), mutations
  **N4** (balance âgée), **N5** (grand livre), **N7** (projet) retirant `afterGeneration` de leur `finally`, rejouées
  une à une sur une copie hors dépôt (vrai `node_modules`, `mutations-p3/`, témoin 206 vert) : **toutes tuées** sur
  l'assertion nommée « retour sur le bilan désaccordé pendant … redemandé à la fin de ce chargement ». L2 : phrase au
  manuel (changer la présentation pendant le chargement d'un autre rapport l'interrompt, relancer *Générer*) et
  amélioration tracée (`issues-a-ouvrir.md`, `enhancement`, P4). L3 : noté, **non remédié** — la redemande du bilan
  efface l'erreur de l'autre onglet (`errorMsg = null`) ; ce n'est pas une régression (avant `623bb649`, l'erreur
  s'affichait sur le mauvais onglet et `selectTab` l'effaçait), plutôt une amélioration ; correctif possible : ne
  resynchroniser qu'après une génération réussie. L4 : `TestingLibraryElementError` ajouté à l'expression du verdict de
  `mut.sh` (hors dépôt). Gates frontend au dernier commit de code : `npm run check` 0 erreur, 29 avertissements ;
  `lint-i18n-ownership` vert ; Vitest **4707 passés, 283 fichiers** ; `npm run build` vert (`front-p3rem.log`). Le
  typage du test (`Ecran`) a été corrigé après les mutations (erreurs `svelte-check`), sans changer son comportement.

### File List

- `frontend/src/lib/features/reports/reports-grouping.ts` (neuf)
- `frontend/src/lib/features/reports/GroupedReportRows.svelte` (neuf)
- `frontend/src/lib/features/reports/GroupingControls.svelte` (neuf)
- `frontend/src/lib/features/reports/AccountCurrencyBalance.svelte`
- `frontend/src/lib/features/reports/BalanceSheetView.svelte`
- `frontend/src/lib/features/reports/IncomeStatementView.svelte`
- `frontend/src/lib/features/reports/reports.api.ts`
- `frontend/src/lib/features/reports/reports.types.ts`
- `frontend/src/routes/(app)/reports/+page.svelte`
- `frontend/src/routes/(app)/reports/+page.ts`
- `frontend/src/lib/features/reports/reports-grouping.test.ts` (neuf)
- `frontend/src/lib/features/reports/GroupedReportRows.test.ts` (neuf)
- `frontend/src/lib/features/reports/views-groupes.test.ts` (neuf)
- `frontend/src/lib/features/reports/reports.api.groupes.test.ts` (neuf)
- `frontend/src/routes/(app)/reports/reports-page-groupes.test.ts` (neuf)
- `frontend/src/lib/shared/i18n-keys.test.ts` (compteur)
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` (compteurs)
- `frontend/tests/e2e/reports-groupes.spec.ts` (neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/screenshots/rapport-bilan-groupes.png` (neuf)
- `_bmad-output/implementation-artifacts/46-2-groupes-ecrans.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-20 — Fiche écrite depuis l'esquisse : hiérarchie montrée par défaut (Q10), contrôles, règle d'ouverture
  (remarque 4 de Guy), `header` par défaut (remarque 5), export aux mêmes paramètres, phrase « affichage de travail »
  (remarque 3). Neuf critères. `drafted`.
- 2026-11-20 — Validation P1 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 3 M
  / 3 L, remédiés. M : E2E existants incomplètement inventoriés (`reports-export-pdf.spec.ts`, tests axe de
  `reports.spec.ts`, qui contrôlent désormais l'écran groupé) → AC-1 ; redemande à chaque clic sans dire son coût (une
  trace `report.generated` par demande, `services/rapports.rs:1059`) → AC-3, C-46-2-1 (assumé, repli local écarté) ;
  prémisse fausse « paramètres refusés en `400` avant la 46-3 » (`ExportQuery` les ignore : PDF à plat d'un écran
  groupé, en silence) → ordre 46-1 → 46-3 → 46-2 fixé, drapeau écarté. L : sélecteur désactivé si `maxDepth` ≤ 1,
  borné à 20 ; affichage gardé pendant le chargement ; « hiérarchie masquée » = vue à plat décidée (C-46-2-2).
  Rapport : `kesh-gate-logs/epic-46/validation/46-2-P1.md`.
- 2026-11-20 — Validation P2 (Claude Sonnet 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C / 0 H / 0 M
  / 2 L, remédiés. Vérifié : `ExportQuery` ignore les paramètres inconnus (`routes/reports.rs:174`), E2E cités
  (`reports.spec.ts` axe `:165`, `:176` ; `reports-export-pdf.spec.ts:40` ; `reports-currency.spec.ts:71`), lignes de
  `reports.api.ts` (`:26`, `:36`, `:45`, `:158`, `:280`, `:306`, `:334`, `:446`), vues (283, 113, 138 lignes),
  `+page.svelte` 893 lignes, `mode.svelte.ts`, manuel `:3923`, `:3954`. L : composant paramétré par un descripteur de
  colonnes (T3) ; mention « à confirmer par le responsable de l'epic » remise sur « hiérarchie masquée » = vue à plat (AC-2, C-46-2-2).
  Rapport : `kesh-gate-logs/epic-46/validation/46-2-P2.md`.
- 2026-11-20 — Validation P3 ciblée (Claude Haiku 5.5, contexte frais, lentille sur le seul commit `8a7f0f73`) : 0 C / 0 H / 0 M / 1 L, remédié. L : le descripteur « une colonne de montant » ne disait pas où loge le solde en devise ; il reste dans la cellule du libellé, comme `AccountCurrencyBalance` aujourd'hui (T3). Rapport : `kesh-gate-logs/epic-46/validation/46-123-P3-ciblee.md`.
- 2026-11-20 — Décisions du responsable de l'epic : (1) « hiérarchie masquée » = la liste à plat des comptes, comme l'écran d'aujourd'hui
  (AC-2, C-46-2-2 : « à confirmer » remplacé par « décidé ») ; (2) rechargement à chaque clic accepté (aller-retour API,
  trace `report.generated`) **à condition que la vue soit mémorisée** : AC-10 neuf (`localStorage`, clé par société et
  par rapport, restaurée avant la première requête, groupes disparus ignorés), AC-5, AC-3, T4 et les tests de l'AC-9
  réalignés ; convention réutilisée : `mode.svelte.ts`.
- 2026-11-20 — Validation P-ciblée des décisions du responsable de l'epic (Claude Haiku 5.5, contexte frais, lentille sur le seul commit `644b9e55` ; au code de `origin/main`) : 0 C / 0 H / 1 M / 3 L, remédiés. M : AC-10 se contredisait — « restaurée avant la première requête » et « groupes disparus ignorés, non transmis » ne tiennent pas ensemble, le client ne connaissant pas le plan avant la première réponse ; remédié : les identifiants sont transmis, l'API les ignore et les retire de l'écho (46-1 § *Le contrat*), la mémoire reprend l'écho. L : « même utilisateur » promis sans que la clé porte l'utilisateur (clé par navigateur, comme `kesh-mode`) ; identifiant de société non chargé par `+page.ts` (`company.id` de `fetchCompanyCurrent`) ; tests de l'AC-10 n'écrivant pas position ni montrée/masquée, pourtant retenues (AC-5). Non remédié, à trancher par le responsable de l'epic : AC-3 dit le rechargement accepté « à condition que la vue soit mémorisée », condition sans objet si le stockage est refusé (la vue n'est alors pas retenue, le rechargement reste). Boucle non close (un M remédié). Rapport : `kesh-gate-logs/epic-46/validation/46-2-decisions-ciblee.md`.
- 2026-11-20 — Validation P-ciblée n°2 (Claude Sonnet 5.5, contexte frais, lentille sur le seul commit `935be28e`, au code de `origin/main`) : 0 C / 0 H / 0 M / 1 L, remédié. Décision de l'orchestrateur appliquée : AC-3 et AC-10 acceptent le rechargement et la trace **quel que soit l'état du stockage**, la vue étant mémorisée dans la mesure où le navigateur le permet (stockage refusé ou vide : vue par défaut, sans erreur) ; la condition « à condition que la vue soit mémorisée » est retirée des AC vivants (les entrées datées ci-dessus restent un relevé). L : une liste mémorisée de plus de 500 identifiants aurait fait répondre `400` (46-1) ; ignorée à la lecture (AC-10). Vérifié : contrat 46-1 (inconnus ignorés et retirés de l'écho, `400` au-delà de 500, `open`/`closed` sans `detailLevel` refusés), `+page.ts` ne renvoie que `companyName` et `company.id` vient de `fetchCompanyCurrent` (`settings.types.ts:10`, `:127`), cohérence AC-2, AC-5, AC-9, T4 et 46-3 (PDF de la requête affichée). Rapport : `kesh-gate-logs/epic-46/validation/46-2-decisions-ciblee-2.md`.
- 2026-11-22 — Développement (Claude Opus 5.5) : T0 à T6 ; export groupé branché sur les paramètres de la fiche
  46-3 (non fusionnée), à exercer de bout en bout après sa fusion ; maquette non montrée avant les vues (écart à
  l'AC-8) ; gates de la nuit (fmt, clippy, nextest complet, frontend complet, E2E de la seule spec ajoutée).
  `review`.
- 2026-11-22 — Revue de code P1 (Claude Opus 5.5, complète, contexte frais) : 0 C / 0 H / 2 M / 6 L. M1 : `refreshing`
  resté levé quand une redemande est périmée par un autre chemin (sonde S4). M2 : contrôles montrant une vue que
  l'écran n'affiche pas, pendant le premier chargement (S1) ou après une redemande jetée (S2). L1 : mutations MA, MB,
  MC survivantes ; L2 : libellé accessible du sélecteur ; L3 : `balanceColumns` recopié ; L4 : manuel ; L5 :
  dépendance à la 46-3 ; L6 : prénom dans la fiche. Tous remédiés (`7ec208c8`, `b7a53abb`, détail au Dev Agent
  Record). Boucle non close (deux M remédiés, production touchée) : passe P2 complète, autre modèle, prompt
  `kesh-gate-logs/s46-2/revue-p2-prompt.md`.
- 2026-11-22 — Revue de code P2 (Claude Sonnet 5.5, complète, contexte frais) : 0 C / 0 H / 1 M / 4 L. M1 : retour sur
  un rapport désaccordé pendant le chargement d'un autre onglet, sans redemande (sonde X1). L1 : mutations P2B, P2C,
  P2D survivantes ; L2 : redemande en erreur ; L3 : `BALANCE_COLUMNS` mutable ; L4 : `title` sur le conteneur. Tous
  remédiés (`623bb649`, détail au Dev Agent Record ; L2 dit au manuel). Boucle non close (un M remédié, production
  touchée) : passe P3 ciblée sur `623bb649`, prompt `kesh-gate-logs/s46-2/revue-p3-prompt.md`.
- 2026-11-22 — Revue de code P3 ciblée (Claude Opus 5.5, une lentille, contexte frais, sur `623bb649`) : 0 C / 0 H / 0 M
  / 4 L, tous traités sans code de production (tests, manuel, trace d'amélioration, script de mutation ; L3 noté).
  **Boucle close.** Tendance : P1 0 C / 0 H / 2 M / 6 L (Opus 5.5) → P2 0 / 0 / 1 M / 4 L (Sonnet 5.5) → P3 0 / 0 / 0 / 4 L
  (Opus 5.5, au lieu de Haiku : la remédiation changeait la logique d'état de la page). Gates frontend verts :
  Vitest 4707 / 283 fichiers. `done`.
