# Story 15-14 : Lot de défauts P3/P4 de documentation et de libellés — index

Status: split

<!-- Spécifiée le 2026-10-09 sur origin/main = dc4bc58b (worktree kesh-15-14, branche
     story/15-14-lot-documentation-libelles). Découpée d'emblée en 15-14a et 15-14b (C-15-14-1).
     Cette fiche est l'index du tri ; les critères, tâches et tests sont dans les deux sous-fiches.
     Validation P1 : branche rebasée sur origin/main = bcded0c8 (15-13a mergée, #551 fermée). -->

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
| **15-14a** — manuels et libellés | `15-14a-manuels-et-libelles.md` | #539, #547, #488, #291, #458, #449, #432, #569, #321, #323 (refs #459) | aucune dépendance de code ; développable sur `main`. **Conflits de rebase attendus** avec la 15-7b1 (`user-manual.tex:182`, `admin-manual.pdf`, `user-manual.pdf`) et la 15-13b (manuel d'administration) : le dernier mergé rebase et **régénère** les PDF |
| **15-14b** — exploitation et multi-société | `15-14b-exploitation-et-multi-societe.md` | #575, #554, #127 | **après** le merge de 15-13b et 15-14a (15-13a : mergée, `bcded0c8`) |

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
| #324 | P3 | prémisse fausse au code : une écriture saisie reste modifiable (« rien à valider », manuel `:476-481`) — c'est le français « Valider » qui trompe, arbitrage de vocabulaire de Guy, ~15 sélecteurs E2E/Vitest ; **à commenter sur l'issue** |
| #469 | P4 | demande un mécanisme `AppError` résolu par clé, non un libellé |
| #339 | P4 | 48 sites de markup en dur : un rollout i18n |
| #504 | P4 | règle métier (langue du contact) |
| #253 | P4 | contraste CSS |
| #76, #97, #125, #126, #287, #310, #421, #424, #478, #498 | P3/P4 | KF de tests, de flakes ou de couverture |
| #293, #522, #537, #538, #546, #548, #555, #568, #578 | P3 | défauts de comportement |

## Pour l'orchestrateur

- **Validation** : chaque sous-fiche a sa section « Ce que la validation P1 doit regarder ».
- **Issues** : commenter #324 (prémisse réfutée, question de vocabulaire posée à Guy) ; aucune issue à créer.
- **Ordre** : 15-14a quand un agent est libre ; 15-14b après les merges de 15-13b et 15-14a (15-13a faite).

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
