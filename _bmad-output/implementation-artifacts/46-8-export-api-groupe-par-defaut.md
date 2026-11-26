# Story 46-8 : l'export des rapports par l'API rend la présentation de l'écran

Status: done

*Décision du mainteneur du 2026-11-24, appliquée à l'Epic 46 avant sa PR : **l'export des rapports par l'API doit
rendre la même chose que l'export lancé depuis l'écran**. La présentation groupée devient le défaut de l'export, avec
les colonnes `Niveau;TypeLigne` au CSV ; l'ancien export à l'octet près n'est plus rendu sans paramètre. L'API n'est pas
encore utilisée en production ; le changement se fait dans la v1 et s'annonce comme une évolution. Branche
`story/46-export-api-groupe`, partie de `pr/46-rapports-groupes` (`511e61d54`). Aucune migration.*

## Prémisse, vérifiée au code (`511e61d54`)

- **L'écran** (`reports-grouping.ts`, `defaultGroupingView`) montre à la première visite la hiérarchie,
  `detailLevel=all`, `subtotalPosition=header`, aucun groupe à la main ; il envoie ces paramètres à la route JSON et à
  l'export (`getReportExportUrl`, `groupingQueryParams`). Case « Hiérarchie des groupes » décochée, il n'en envoie aucun
  (`groupingParamsOf` rend `undefined`).
- **L'API sans paramètre** : les trois exports (`export_balance_sheet`, `export_income_statement`,
  `export_trial_balance`, `routes/reports.rs`) lisaient les groupes par `groupes_de`, qui rend `None` sans
  `detailLevel` ; les rendus `render_grouped_*` retombent alors sur le rendu à plat, à l'octet (fixtures
  `export-{bilan,resultat,balance}-*` de la 46-3 et de la 46-6b). PDF et CSV seulement : il n'y a pas d'export JSON.
  Les routes JSON (`get_balance_sheet`, `get_income_statement`, `get_trial_balance`) rendent à plat sans `detailLevel`.
- **Aucun paramètre ne demandait le rendu à plat** : l'absence de `detailLevel` en tenait lieu. L'écran à plat en
  dépend ; changer le défaut sans moyen explicite aurait fait exporter groupé l'écran à plat.
- **Le MCP** (`mcp/outils/rapports.rs`) passe par les mêmes services (`*_par_groupes`, `Affichage`) que les routes
  JSON, jamais par l'export ; il ne produit aucun fichier.

## Critères d'acceptation

1. **AC-1 — Défaut de l'export.** Sans `detailLevel`, `GET /reports/{balance-sheet,income-statement,trial-balance}/export`
   rend, en PDF et en CSV, ce que rend `detailLevel=all&subtotalPosition=header` (la vue de première visite de l'écran) :
   CSV à l'octet, PDF aux mêmes lignes de texte ; colonnes `Niveau;TypeLigne` au CSV, ligne « Tous les comptes » au PDF.
   `subtotalPosition`, `open`, `closed` s'appliquent seuls au niveau `all`.
2. **AC-2 — Export à plat.** `detailLevel=flat` rend l'export d'avant (fixtures de la 46-3 et de la 46-6b, CSV à l'octet,
   PDF aux mêmes lignes) ; avec `flat`, `subtotalPosition`, `open`, `closed` sont refusés (`400 VALIDATION_ERROR` qui
   nomme le paramètre et `flat`) ; `flat` répété est un paramètre répété ; la casse compte (`FLAT` refusé).
3. **AC-3 — Routes JSON et MCP inchangés.** Sans `detailLevel`, les routes JSON restent à plat, à l'octet (fixtures
   JSON de la 46-1 et de la 46-6a) ; `detailLevel=flat` y vaut l'absence. Les outils MCP restent à plat sans
   `detailLevel` (ils reflètent les routes JSON, pas l'export) ; `flat` n'y est pas ajouté.
4. **AC-4 — L'écran.** Hiérarchie masquée, l'export lancé depuis l'écran porte `detailLevel=flat` et rien d'autre
   (bilan, compte de résultat, balance) ; les autres rapports n'en reçoivent jamais.
5. **AC-5 — Docs.** `docs/api-external.md` (défaut, colonnes, `flat`, refus, évolution de la v1), manuel utilisateur FR
   (`.tex`) ; notes de version de l'epic (46-3, 46-6b) corrigées.

## Choix

- **`detailLevel=flat`, une valeur et non un paramètre neuf** : le contrat garde ses quatre paramètres, la règle du
  paramètre répété et l'ordre des contrôles s'appliquent sans rien ajouter ; `flat` se lit « niveau de détail : à plat ».
- **Le défaut est posé à la route d'export** (`groupes_de_l_export`, `NIVEAU_EXPORT_PAR_DEFAUT = "all"`), non dans le
  service : le service sert aussi l'affichage JSON et le MCP, que la décision ne vise pas.
- **Routes JSON et MCP inchangés** : la décision porte sur l'export ; le JSON groupé est un sur-ensemble du JSON à plat
  qu'un client demande quand il le veut, et le MCP est borné par la taille de sa réponse (la balance groupée à `all`
  dépasse son plafond sans tranche, 46-6a C-46-6a-2). Point soumis au mainteneur au rapport.

## Tâches

- [x] `NiveauDemande::APlat`, lu de `detailLevel=flat` (`DemandeGroupes::depuis_requete`), refus de
      `subtotalPosition`/`open`/`closed` à plat (`depuis_valeurs`) — `services/rapports.rs`
- [x] `lire_groupes(paires, niveau_par_defaut)`, `groupes_de` (JSON) et `groupes_de_l_export` (exports) —
      `routes/reports.rs` ; quatre tests unitaires
- [x] Tests d'intégration (`reports_export_e2e.rs`, `opening_balances_e2e.rs`, `reports_e2e.rs`)
- [x] Frontend : `FLAT_DETAIL_LEVEL`, `getReportExportUrl` ; Vitest ; E2E (`reports-balance-groupes.spec.ts`, étape 7 bis)
- [x] Docs : `docs/api-external.md`, `user-manual.tex`, module du MCP, notes de version 46-3 et 46-6b

## Tests changés, et pourquoi

- `exports_sans_parametre_identiques_a_la_fixture_de_base` → `exports_a_plat_identiques_a_la_fixture_de_base`, et
  `export_balance_sans_parametre_identique_a_la_fixture_de_base` → `export_balance_a_plat_identique_a_la_fixture_de_base` :
  ils figeaient l'export sans paramètre à l'octet ; l'ancien rendu se demande désormais par `detailLevel=flat`, et c'est
  lui qu'ils gardent, mêmes fixtures (non réécrites).
- `export_groupes_refus_du_contrat` : `subtotalPosition=header` et `open=1` seuls, refusés jusqu'ici, sont acceptés
  (export groupé par défaut) ; les cas portent désormais sur ces paramètres avec `detailLevel=flat`, plus `closed`,
  `FLAT` et `flat` répété.
- `export_balance_lit_les_parametres_de_groupes` : sa référence « sans paramètre » (à plat) devient `detailLevel=flat`.
- `opening_balances_e2e` (report d'ouverture) et `reports_e2e` (report calculé non nul) : leurs lignes attendues sont
  celles du CSV à plat ; l'export porte `detailLevel=flat`.
- Vitest : l'URL d'export sans groupement attendait aucun paramètre ; elle porte `detailLevel=flat`.

## Note de version

- **Changed (API)** — L'export du bilan, du compte de résultat et de la balance par l'API
  (`GET /reports/…/export`) rend désormais, sans paramètre, la même chose que l'export lancé depuis l'écran : la
  présentation par groupes de comptes, tous les groupes ouverts, sous-totaux sur la ligne du titre, avec au CSV les
  colonnes `Niveau;TypeLigne`. L'export à plat d'avant se demande par `detailLevel=flat` (à l'octet près l'ancien
  export sans paramètre). `subtotalPosition`, `open` et `closed` sans `detailLevel`, refusés jusqu'ici, sont acceptés
  à l'export. Les routes JSON et les outils MCP ne changent pas. Évolution de la v1 de l'API.

## Dev Agent Record

Claude Opus 5.5 (`bmad-dev-story`), 2026-11-24. Journaux et captures : `kesh-gate-logs/s46-export/` (hors dépôt).
Gates au dernier commit de code `ef9f2abf2` :

- `cargo fmt --all -- --check` et clippy workspace `-D warnings` : verts (`fmt-a.log`, `clippy-a.log`).
- nextest ciblé, base remise à zéro (`cible-a.log`) : `package(kesh-api) & (kind(lib) | reports_export_e2e |
  reports_e2e | opening_balances_e2e | textes_coherents | dates_bornes_e2e | mcp_outils_e2e | mcp_matrice_manuel)` —
  **1080 exécutés, 1080 passés, 5 ignorés**. Gate complet et `gate-ci.sh` : à la PR de l'epic.
- Frontend (`front-*-a.log`) : `check` 0 erreur (32 avertissements d'avant, 5 fichiers), `lint-i18n-ownership` vert,
  Vitest **5292 passés (309 fichiers)**, `build` vert.
- E2E partiel (`e2e.log`, 4 specs : `reports-balance-groupes` — étape 7 bis neuve —, `reports-groupes-export`,
  `reports-groupes`, `reports-export-pdf`) : **4 passés**.
- Sondes négatives, chacune sur une copie restaurée du fichier : **M1** `groupes_de_l_export` → `lire_groupes(paires,
  None)` (ancien défaut) : 3 tests nommés rouges sur assertion (`export_sans_parametre_groupe_comme_l_ecran`,
  `export_defaut_combine_aux_autres_parametres`, `exports_sans_parametre_rendent_la_presentation_de_l_ecran` —
  « balance-sheet … csv : sans paramètre, la présentation de l'écran »), `m1.log` ; **M2** l'écran à plat sans
  `detailLevel=flat` : 2 tests Vitest rouges (`AssertionError`), `m2.log`.
- Tests neufs (de `511e61d54` à `ef9f2abf2`) : 4 unitaires (`routes/reports.rs`), 1 d'intégration
  (`exports_sans_parametre_rendent_la_presentation_de_l_ecran`), 1 Vitest, 1 étape E2E ; 6 tests réorientés (§ *Tests
  changés*), aucun supprimé.
- Manuel d'administration : il ne décrit pas l'export des rapports par l'API (seulement les outils MCP) ; *(corrigé à la remédiation de la revue P1, LOW-8 : le changement de contrat est porté au paragraphe « Pour les intégrations » de la section `sec:maj-epic-47`, « Passer à la release qui porte les Epics 38, 40, 39, 46 et 47 », ouverte sur cette branche pour lui seul — la branche d'intégration porte une section de même titre et de même label, à réunir à la fusion en gardant les deux textes.)*
  Aucun texte i18n touché.

## Change Log

- 2026-11-24 — Développement (Claude Opus 5.5, `bmad-dev-story`) : décision du mainteneur appliquée ; prompt de la revue
  P1 dans `kesh-gate-logs/s46-export/revue-p1-prompt.md`.
- 2026-11-24 — Revue de code P1 (Claude Opus 5.5, complète, une lentille, `kesh-gate-logs/s46-export/revue-P1.md`) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW**. Les huit corrigés dans la story (règle « Tarir la source »), commit
  `1c71314a0` : LOW-1 JSDoc de `groupingQueryParams` rattachée (constante `FLAT_DETAIL_LEVEL` placée avant) ; LOW-2
  motif `None | Some(NiveauDemande::APlat)`, `match` exhaustif sans attrape-tout ; LOW-3 doc de `NiveauDemande`
  (« `all` ou `flat` ») ; LOW-4 `api-external.md` : « avec `detailLevel` autre que `flat` » (réponse du bilan, balance),
  « et de même avec `detailLevel=flat` » à la balance ; LOW-5 annotation datée sous D8 du fichier d'epic, renvoi à
  NFR-46-1 ; LOW-6 trois tests — l'export CSV **sans paramètre** omet la ligne de report nulle
  (`post_happy_path_od_at_fy_start_appears_in_balance_sheet`), `routes_json_a_plat_identiques_a_la_fixture_de_base`
  (trois routes JSON × deux exercices, `detailLevel=flat` contre les fixtures de base), E2E `reports-groupes.spec.ts`
  étapes 7 et 9 (export à plat du bilan et du compte de résultat depuis l'écran : `detailLevel=flat`, fichier sans
  `Niveau;TypeLigne`) ; LOW-7 « Évolution de la v1 » : montants arrondis au centime, ligne « Tous les comptes » au PDF ;
  LOW-8 section `sec:maj-epic-47` du manuel d'administration, paragraphe « Pour les intégrations », et Dev Agent Record
  précisé. Gates au commit `1c71314a0` (journaux `*-b.log`) : fmt et clippy verts ; nextest ciblé, base remise à zéro,
  même expression qu'au développement — **1081 exécutés, 1081 passés, 5 ignorés** (`cible-b.log` ; 1080 + le test
  neuf de `reports_e2e`) ; `check` 0 erreur (32 avertissements d'avant), `lint-i18n-ownership` vert, Vitest **5292
  passés (309 fichiers)**, `build` vert ; E2E des quatre specs de rapports **4 passés** (`e2e-b.log`). Sondes
  négatives (`mutation-p1.sh`, copies restaurées) : **M3** ligne de report montrée au seul CSV groupé → rouge sur la
  nouvelle assertion (`opening_balances_e2e.rs:431`, `m3.log`) ; **M4** `flat` lu comme `all` → rouge
  (`reports_e2e.rs:2178`, `m4.log`). L'étape E2E neuve n'a pas eu de sonde (M2, au développement, couvre en Vitest
  l'URL à plat). Manuels administration et utilisateur compilés sur une copie hors dépôt (`manuel-src/`), PDF aplati et
  grepé : les phrases neuves y sont ; aucun PDF commité. Passe 2 ciblée à lancer : `revue-p2-ciblee-prompt.md`.

### 2026-11-24 — Clôture
Revue de code : P1 (Opus, complète) 0/0/0/8 → remédiation des huit LOW (`1c71314a0`) → P2 ciblée (Sonnet) 0/0/0/3 → correctif de deux LOW (`742d02e7a`, manuel et doc-comments) → P3 ciblée (Haiku, sur ce seul commit) 0/0/0/1 (longueur d'une ligne de doc, laissée). Boucle close sur une passe sans finding au-dessus de LOW. LOW-3 de la P2 (une seule section `sec:maj-epic-47` à la fusion avec la branche d'intégration de l'Epic 47) noté pour la fusion. Gates au dernier commit de code `1c71314a0` : fmt, clippy, nextest ciblé 1081/1081, Vitest 5292, `check`, `build`, E2E des quatre specs de rapports ; `742d02e7a` ne change que des commentaires et le manuel (fmt vert). Gate complet et `gate-ci.sh` avec la branche de la PR de l'Epic 46.

### 2026-11-25 — Reprise sur `main` (branche `pr/46-rapports-groupes-v2`)
La section `sec:maj-epic-47` n'existe plus : son paragraphe est versé, sous le titre « Les exports des rapports », dans
la section unique « Passer à la 0.20.0 » (`sec:maj-0-20-0`) du manuel d'administration, avec la précision que
l'export à plat arrondit désormais au centime (Story 48-11, déjà dans `main`) ; même précision dans
`docs/api-external.md`.
