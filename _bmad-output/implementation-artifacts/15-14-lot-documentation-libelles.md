# Story 15-14 : Lot de défauts P3/P4 de documentation et de libellés — index

Status: split

<!-- Spécifiée le 2026-10-09 sur origin/main = dc4bc58b (worktree kesh-15-14, branche
     story/15-14-lot-documentation-libelles). Découpée d'emblée en 15-14a et 15-14b (C-15-14-1).
     Cette fiche est l'index du tri ; les critères, tâches et tests sont dans les deux sous-fiches.
     Validation P1 : branche rebasée sur origin/main = bcded0c8 (15-13a mergée, #551 fermée).
     Validation P3 : branche rebasée sur origin/main = 245b91ee (15-7b1 mergée). -->

## Pourquoi ce lot

Décision de Guy : faire baisser le nombre de bugs ouverts — **70** au 2026-10-09, **49** au kickoff de
l'Epic 15 (engagement 1 du § *Priorités des défauts* du `CLAUDE.md`). Le lot réunit des défauts **P3/P4 de
documentation et de libellés** (**69** à la validation P1, même jour, recompté par la commande du
`CLAUDE.md`) : textes qui disent autre chose que le code, sans règle métier neuve, sans
migration, sans changement de comportement.

## Méthode du tri

Toutes les issues ouvertes portant `bug`, `known-failure` ou `documentation`, priorité P3 ou P4
(`gh issue list --state open --limit 500 --json number,title,labels`), chacune lue avec ses commentaires
(`gh api repos/guycorbaz/kesh/issues/N` et `/comments`) et **vérifiée au code de `dc4bc58b`** — aucune
n'était déjà corrigée en entier ; deux l'étaient en partie (#127 : `kesh-cli` et RBAC ; #575 : `exec db`,
corrigé par la 15-13a, mergée depuis — `bcded0c8`).

Exclues d'office par l'orchestrateur (stories en cours ou spécifiées) : #551 (15-13a), #552 et #576
(15-13b), #474 (15-6c), #524 (15-6d), #544 (15-7b1), #528 et #542 (15-7b3), #279 (15-7b2), tout ce qui
touche le lettrage (#518), et #577 (texte du `CLAUDE.md`, affaire de Guy).

## Découpage (C-15-14-1)

| Sous-story | Fiche | Issues fermées | Dépendance |
|---|---|---|---|
| **15-14a** — manuels et libellés | `15-14a-manuels-et-libelles.md` | #539, #547, #488, #291, #458, #449, #432, #569, #321, #323 (refs #459) | **aucune restante** : la 15-7b1, dont elle dépendait (`user-manual.tex:182` ; C-15-14-19), est **mergée** (`245b91ee`, PR #583) — branche rebasée dessus en validation P3. **Conflits de rebase attendus** avec la 15-13b (PR #584, en cours d'intégration : manuel d'administration, `admin-manual.pdf`) : le dernier mergé rebase et **régénère** les PDF ; `vat_rates.rs` (doc-comment de la 15-7b1 contre le `mod tests` neuf) est déjà rebasé — gate complet au développement (exception `kesh-db`) |
| **15-14b** — exploitation et multi-société | `15-14b-exploitation-et-multi-societe.md` | #575, #554, #127 | **après** le merge de la 15-13b (PR #584) et de la 15-14a (15-13a et 15-7b1 : mergées, `bcded0c8` et `245b91ee`) ; textes cibles écrits pour l'état après la 15-13b (C-15-14-21) |

Motif : une **dépendance**, non le nombre de modules. La 15-14b réécrit le manuel d'administration et le
compose de développement, que la 15-13a (+187 lignes au manuel) et la 15-13b réécrivent aussi, et le
complément de #575 n'existe qu'après la 15-13a. Une story unique serait bloquée tout entière par la 15-13b ;
une coupe « doc / libellés » ferait se disputer le manuel utilisateur aux deux moitiés (#569 y a cinq sites —
« quatre » à la spécification, un cinquième trouvé en validation P1).

**Décompte attendu** : 11 issues `bug`/`known-failure` fermées (8 par la 15-14a, 3 par la 15-14b), plus
#291 et #458 (`documentation` seul, hors décompte).

## Écartées (C-15-14-2)

| Issue | Prio. | Raison |
|---|---|---|
| #579 | P4 | B-2 exige un champ neuf dans `DbError::SettlementCounterpartyIsClaimAccount` (kesh-db) et B-6 une refonte ; dette confiée à la 15-6c (C-15-6b-3), en développement |
| #324 | P3 | prémisse fausse au code : une écriture saisie reste modifiable (« rien à valider », manuel `:476-481`) — c'est le français « Valider » qui trompe, arbitrage de vocabulaire de Guy, ~15 sélecteurs E2E/Vitest ; **commentée** sur l'issue le 2026-10-09 (08:24 UTC — validation P2, R-11) |
| #469 | P4 | demande un mécanisme `AppError` résolu par clé, non un libellé |
| #339 | P4 | 48 sites de markup en dur : un rollout i18n |
| #504 | P4 | règle métier (langue du contact) |
| #253 | P4 | contraste CSS |
| #76, #97, #125, #126, #287, #310, #421, #424, #478, #498 | P3/P4 | KF de tests, de flakes ou de couverture |
| #293, #522, #537, #538, #546, #548, #555, #568, #578 | P3 | défauts de comportement |

## Pour l'orchestrateur

- **Validation** : chaque sous-fiche a sa section « Ce que la validation P1 doit regarder ».
- **Issues** : #324 déjà commentée (le 2026-10-09, 08:24 UTC) — **ne pas reposter**. Depuis la validation
  P4 : #585 (P4, « Paramètres → Facturation » est un titre d'écran, non un chemin de menu) **ouverte** par
  l'orchestrateur, hors périmètre ; #459 **commentée** par l'orchestrateur (G5 et la phrase de
  `user-manual.tex:1524` à inverser le jour où l'import s'automatise). Aucune autre issue à créer.
- **État de validation** (P4) : **15-14a close** (0 MEDIUM, C-15-14-32) ; **15-14b** : P5 en passe ciblée
  sur la remédiation P4 (recette de sauvegarde et ses gardes).
- **Ordre** : 15-14a développable dès maintenant (15-7b1 mergée, `245b91ee`) ; 15-14b après les merges de
  15-13b et 15-14a (15-13a et 15-7b1 faites).

## Change Log

- 2026-10-09 — Spécification et découpage (Opus 5.5), sur `dc4bc58b`. Choix C-15-14-1 à C-15-14-10.
- 2026-10-09 — **Validation P1** (Sonnet ×2, lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p1-{R,F}.md`) :
  0 CRITICAL, 0 HIGH ; **11 MEDIUM bruts** (R 5, F 6) — trois recoupements MEDIUM/MEDIUM (R1 = F-1,
  R2 ≈ F-3, R5 ≈ F-2) et deux MEDIUM qui recoupent un LOW de l'autre lentille (R3 ≈ F-11, F-6 ≈ R6, retenus
  MEDIUM) —, soit **8 MEDIUM distincts** ; **17 LOW bruts** (R 9, F 8), dont deux absorbés par un MEDIUM et
  quatre groupes de recoupement (R9 = F-9, R11 = F-7, R10 ≈ R13 ≈ F-8, R14 ≈ F-10 : cinq doublons), soit
  **10 LOW distincts** (15 − 5).
  Tous des **sites d'inventaire manqués** ou des **tests sous-spécifiés** ; aucun fait de fond réfuté.
  Remédiation dans les deux sous-fiches (leur Change Log détaille), branche rebasée sur `bcded0c8` (15-13a
  mergée) et numéros réalignés. Choix C-15-14-11 à C-15-14-16. Signal D5 : non levé (passe 1).
- 2026-10-09 — **Validation P2** (Opus ×2, lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p2-{R,F}.md`) :
  0 CRITICAL, 0 HIGH ; **12 MEDIUM bruts** (R 3, F 9), trois recoupements (R-1 = F-5, R-2 = F-6, R-3 = F-2),
  soit **9 MEDIUM distincts** ; **20 LOW bruts** (R 11, F 9), quatre recoupements (R-5 = L-1, R-9 = L-2,
  R-8 = L-4, R-13 = L-6) et un LOW absorbé par un MEDIUM (R-6 ≈ F-9), soit **15 LOW distincts**. Ventilation :
  15-14a 5 MEDIUM (F-1 partagé, F-4, F-5, F-6, F-9) et 8 LOW (R-14 partagé) ; 15-14b 5 MEDIUM (F-1 partagé,
  F-2, F-3, F-7, F-8) et 7 LOW (R-14 partagé) ; index 1 LOW (R-11). Tous corrigés, **aucun réfuté** ; R-10
  corrigé comme renvoi ambigu (le `fiscal_years.rs:228` visait le repository, non la route).
  **Trend** : P1 8 MEDIUM / 10 LOW (Sonnet ×2) → P2 9 MEDIUM / 15 LOW (Opus ×2). **Trois MEDIUM nés de la
  remédiation P1** (F-5, F-6, F-8), tous des inventaires ou listes déclarés complets ; F-1 (contrôle PDF à
  vide) et F-2 (Snapshot Replication) préexistaient. **Signal D5 levé et déclaré** (MEDIUM → MEDIUM, trois
  recyclés) : **pas de nouveau découpage**, décision de l'orchestrateur (C-15-14-23) — la story est déjà
  coupée, et le recyclage porte sur la complétude des inventaires, traitée à la racine par C-15-14-17 :
  chaque inventaire s'écrit désormais comme une commande exécutée, comptée sur `bcded0c8`, partitionnée.
  Choix C-15-14-17 à C-15-14-24. Détail dans le Change Log de chaque sous-fiche.
- 2026-10-09 — **Validation P3** (Sonnet ×2, lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p3-{R,F}.md`) :
  0 CRITICAL, 0 HIGH ; **5 MEDIUM bruts** (R 3, F 2), un recoupement (R-3 = F-2), soit **4 MEDIUM distincts** ;
  **14 LOW bruts** (R 8 listés — le bilan de son rapport en annonce 6 —, F 6), deux recoupements (L-3 = F-3 ;
  étiquettes de gardes de R = F-6), soit **12 LOW distincts**. Ventilation : 15-14a — R-1 (replis frontend
  sans garde), F-1 (périmètre de l'inventaire « réglages ») ; 15-14b — R-3 = F-2 (inventaire borné aux
  sections Synology) ; planification — R-2 (`sprint-status.yaml` et document d'epic muets sur la dépendance
  envers la 15-7b1). **Les deux lentilles recomptent à l'identique tous les comptes de la remédiation P2** :
  les défauts ne sont plus dans les comptes, mais dans les **périmètres** des commandes, une affirmation de
  couverture par un test, et des artefacts de planification. **Aucun finding réfuté** ; chacun vérifié au
  code (`grep -nF`, commandes rejouées) avant correction.
  **Trend** : P1 8 MEDIUM (Sonnet ×2) → P2 9 MEDIUM (Opus ×2) → **P3 4 MEDIUM** (Sonnet ×2). **Part née de la
  remédiation P2** (`144639d0`) : R-2 entièrement, R-3 = F-2 par la borne `awk` qu'elle avait posée, R-1 en
  partie (l'affirmation venait de la spécification, `144639d0` l'avait étendue) ; F-1 préexistait
  (`f5812294`).
  **Faits nouveaux** : la 15-7b1 est mergée (`origin/main = 245b91ee`, PR #583) — la branche y est rebasée
  (sauvegarde `backup/15-14-avant-rebase-245b91ee` ; registre et sprint-status **par union** : 374 entrées
  de `main` + 24 de la branche = 398, puis 7 neuves = 405), la dépendance de la 15-14a est **satisfaite** ;
  la 15-13b (PR #584) est en cours d'intégration, la 15-14b l'attend toujours.
  Remède à la racine (C-15-14-25) : **toute** commande d'inventaire porte sur le dépôt suivi entier, moins un
  ensemble d'exclusions écrit une fois et justifié ; toutes relancées sur `245b91ee`. G13 neuf (C-15-14-26),
  gardes de la 15-14b renumérotées G14-G18 (C-15-14-31). Choix C-15-14-25 à C-15-14-31.
  **Signal D5 levé et déclaré** (MEDIUM → MEDIUM) : le nombre baisse de moitié, mais R-3 = F-2 et F-1
  **recyclent** la famille des P1/P2 (inventaires incomplets), cette fois par le périmètre. Pas de nouveau
  découpage proposé : la story est déjà coupée, et le recyclage est traité à la racine — plus aucune commande
  ne choisit d'avance où regarder. **Arbitrage laissé à l'orchestrateur et au Project Lead.**
- 2026-10-09 — **Validation P4** (Opus ×2, lentilles R et F ; rapports `kesh-gate-logs/15-14-validate-p4-{R,F}.md`) :
  0 CRITICAL, 0 HIGH ; **5 MEDIUM bruts** (R 2, F 3), deux recoupements (R-1 = F-3, R-2 = F-1), soit **3 MEDIUM
  distincts**, tous sur la **15-14b** et tous nés de la remédiation P3 (`b02e9af7`), dans la recette de
  sauvegarde et ses gardes : post-script `:1608` classé assumé (R-1 = F-3), clause « 300 caractères » de
  G16 rouge sur le texte cible (R-2 = F-1), compte de sauvegarde incapable de recharger (F-2) ; **13 LOW**
  (R 8, F 5 ; R L-4 et F L-3 portent sur la même liste « hors motif », par des sites différents — comptés
  deux fois) : 15-14a R L-2, L-5, L-7, F L-2 et R L-8 ; 15-14b R L-1, L-3, L-4, L-6, F L-1 à L-5, et R L-8,
  F L-2 partagés. **Les deux lentilles recomptent à l'identique tous les comptes de `b02e9af7`.** Aucun
  finding réfuté.
  **Trend** : P1 8 → P2 9 → P3 4 → **P4 3 MEDIUM** (Sonnet ×2, Opus ×2, Sonnet ×2, Opus ×2).
  **15-14a : validation close** (0 MEDIUM ; C-15-14-32), LOW appliqués, `ready-for-dev`. **15-14b** : les
  trois MEDIUM corrigés de façon que texte cible, partition et gardes se tiennent — liste fermée de
  fragments pour G16 (e) (C-15-14-33), `:1608` corrigée (C-15-14-34), deux comptes et deux recettes
  (C-15-14-35), L-1 de F intégré à l'AC 3 (C-15-14-37) — et un **contrôle de cohérence** écrit dans la
  fiche, chaque garde appliquée au texte cible site par site ; il a relevé un défaut de plus, corrigé
  (le titre de `sec:backup-dsm` nomme Hyper Backup avant son label : bornes prises au `\subsection{`).
  **Signal D5 levé de nouveau** (trois MEDIUM nés de la remédiation) : **pas de découpage** (C-15-14-36).
  Choix C-15-14-32 à C-15-14-37.
