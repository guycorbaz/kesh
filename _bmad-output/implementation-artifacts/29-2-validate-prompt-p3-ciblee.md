# Prompt — validation P3 ciblée de la spec, Story 29-2 (un paiement reçu après l'échéance, ou après un rappel, est proposé)

*Versionné le 2026-10-14. Passe ciblée (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, **Haiku**, contexte
frais, en lecture seule stricte, braquée sur le **seul diff de la remédiation P2**. Rotation (D6, rétro 25) : P1
Sonnet, P2 Opus, P3 ciblée Haiku.*

Dépôt `/home/gcorbaz/devel/kesh-29-2`, branche `story/29-2` (base `16eeed21`, kickoff de l'Epic 29 ; le code au sol est
celui de `16eeed21` : `git diff --stat 16eeed21 HEAD -- crates frontend docs CHANGELOG.md` est vide).

**Objet, et lui seul** : `git show e17368f8 -- _bmad-output/` (remédiation P2). Lis ce diff hunk par hunk ; ouvre la
fiche `_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` et le registre
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` **autour des hunks** seulement.
Rapports remédiés (à lire pour savoir ce qui était demandé, **pas** à refaire) :
`/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p2-R.md` et `-F.md` ; leur traitement : Change Log de la fiche,
entrée « Validation P2 ». Issue du départage : `gh api repos/guycorbaz/kesh/issues/668` et
`gh api repos/guycorbaz/kesh/issues/668/comments` (attendu **rectifié** au regard de l'art. 87 CO).

⚠️ **Haiku et les diffs** (`CLAUDE.md` § *Haiku-specific guardrails*) : l'objet est **un seul commit** ; mais toute
affirmation de **présence** ou d'**absence** d'un texte (« la fiche dit encore X », « Y n'a pas été corrigé ») doit
être **étayée par la sortie d'un `grep -nF "<chaîne>" <fichier>`** recopiée dans le rapport, sur le fichier **final**
(pas sur les numéros de ligne du diff). Un finding sans cette sortie ne compte pas.

## Lentille unique — chasseur de régressions de la remédiation P2

Pour chaque point : le texte neuf est-il **vrai au code** de `16eeed21`, **cohérent** avec le reste de la fiche, et
**complet** (le symptôme corrigé ne subsiste-t-il pas ailleurs) ?

1. **Départage porté par #668 (C-29-2-13, C-29-2-7, C-29-2-11, § *La règle retenue*, § *Zones partagées*,
   Exclusions, Status, References)** : la fiche et le registre citent-ils #668 / la 29-5 **partout** où ils
   renvoyaient le départage à la 29-3 / #527 ? Reste-t-il, hors des relevés datés (entrée P1 du Change Log, annotations
   datées du registre), une phrase qui confie le départage à la 29-3, ou qui prescrit l'ancien critère (« la récente
   en tête », « proximité de la date ») — contraire à l'attendu rectifié de #668 ? L'attendu recopié dans la fiche est-il
   **fidèle** au commentaire de #668 (échues d'abord, puis ancienneté : échéance, puis date) ? La phrase « la 29-5
   doit être mergée avant le tag v0.13.4 » figure-t-elle ? C-29-2-13 est-il bien présenté comme **décision de
   l'orchestrateur**, et non comme choix autonome ?
2. **Aggravation du cas ordinaire** (`F ≥ D − 30` → `F ≥ D − 60` à 30 jours net) : recalcule (`A = F + 30`,
   `A ≥ D − 30`). La troncature `LIMIT 50` décrite (« garde les 50 plus anciennes ») est-elle juste au code
   (`crates/kesh-db/src/repositories/reconciliation.rs:258-259`, `sed -n`) ?
3. **CHANGELOG prescrit (AC8, F2)** : les deux phrases corrigées sont-elles justes contre la règle `A = max(F, E, R)`,
   `F ≤ D + 1`, `A ≥ D − 30` ? Teste un rappel à `E + 10` et un paiement à `E + 45` ; un paiement à l'heure dans les 30
   jours d'une facture à 60 jours. Les autres textes prescrits (`:1305`, `:2029`, `:2031`, `:2902`, `api-external.md`)
   disent-ils la même chose que le CHANGELOG corrigé ?
4. **Manuel `:2029` et `:2031` (F7, R3, C-29-2-14)** : la réécriture « au plus un jour avant la date de la facture »
   est-elle prescrite sans ambiguïté ? La phrase bornée de `:2031` et la phrase neuve sur l'annulation d'un rappel
   sont-elles vraies au code (`routes/reconciliation.rs:1740-1765`, `sed -n`) ? Le motif affiché est-il bien le repli
   générique (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, `grep -nF
   "payment_date_outside_window"`) ?
5. **Contrôle aplati du PDF (R4)** : la boucle prescrite est-elle exécutable telle quelle, et ses six motifs
   rendent-ils, au sol, `1, 1, 1, 1, 1, 3` ? Tu peux rejouer **en lecture** : `pdftotext docs/manual/fr/user-manual.pdf
   - | tr '\n' ' ' | tr -s ' ' | grep -o -- "<motif>" | wc -l` (par un tube, **sans** fichier temporaire). Un motif
   manque-t-il (une formulation de l'ancienne règle que la boucle ne verrait pas) ?
6. **Mutations M1, M9 (F3), M12, M13 (R9), M14, M15 (F4) et AC5 (`Vec` ordonné)** : recalcule chaque attribution sur
   les cas a à k (`D` = 2026-06-15). M14 n'est-elle vraiment tuée **que** par le scalaire du cas g ? M15 (`GROUP BY
   id`) est-elle tuée par la liste assertée comme `Vec` ? L'ordre de la requête (`ORDER BY i.date ASC, i.id ASC`) est-il
   bien celui qu'AC5 dit asserter ? Une mutation déclarée équivalente à tort est un test muet.
7. **AC6 (R5, R6)** : `paid_at` = `D` à minuit ; patron `:2978` et plage `:3051-3086` de
   `crates/kesh-api/tests/reconciliation_e2e.rs` (`sed -n`) ; doc-comment `:2887` : la fiche autorise-t-elle sa
   réécriture sans contredire « assertions inchangées » ?
8. **AC7 (R2)** : les trois specs ajoutées existent-elles et créent-elles des factures échues relancées
   (`grep -nF "overdueDate" frontend/tests/e2e/*.ts`) ? Le calcul `J + 5` → `J + 30` est-il juste ?
9. **Plages de lignes (R7)** : `:117-149`, `matching.rs:97-106` (départage `:104`), `:234-274` — chacune au `sed -n`.
   Reste-t-il une plage ancienne (`234-276`, `96-103`, `113-149`) hors des relevés datés ?
10. **Change Log** : l'annotation datée sous l'entrée P1 **ajoute** sans réécrire l'entrée (compare avec
    `git show 375f2fbf:_bmad-output/implementation-artifacts/29-2-paiement-tardif.md`) ? Les décomptes de l'entrée P2
    se recomptent-ils depuis la source : bruts R 0/0/1/8, F 0/0/2/6 ; fusions ; **2 MEDIUM, 12 LOW distincts** ;
    10 AC, 7 tâches, **16** lignes de mutation, 11 cas, 8 lignes d'exemple, **14** choix, 3 modules ? Commandes
    suggérées : `grep -c '^### C-29-2-' <registre>`, `grep -cE '^\| M[0-9]+[ab]? \|' <fiche>`.
11. **sprint-status.yaml** : le commentaire de la clé `29-2-paiement-tardif` dit-il la même chose que la fiche
    (décomptes, choix 1 à 14, 29-5 avant le tag) ?

Un défaut introduit par la remédiation P2 qui touche un AC, un test prescrit ou un texte prescrit est **au moins
MEDIUM**.

## Ce que tu rends

Findings numérotés `P3-n` (CRITICAL / HIGH / MEDIUM / LOW), chacun avec `fichier:ligne`, **la preuve** (sortie de
`grep -nF` ou de `sed -n` recopiée ; code relu cité), son **origine** (remédiation P2, ou conception antérieure) et la
correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les onze points ci-dessus, un par un, avec ce qui a
été lancé) est **obligatoire** : un « 0 finding » sans elle ne compte pas comme passe. Déclare tout écart au protocole.

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p3-ciblee.md` ; dernier message : le chemin,
le bilan chiffré, une ligne par finding MEDIUM ou plus.

## Interdits — lecture seule stricte

⛔ N'écris **aucun** fichier, sauf le rapport ci-dessus. Aucune commande qui écrit dans le dépôt, dans une base ou sur
GitHub, ni qui compile ou exécute : **aucun `git checkout` / `switch` / `restore` / `reset` / `stash` / `commit` /
`add` / `rebase` / `tag`**, sous aucune forme ; `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`,
`latexmk`, `cargo`, `npm`, `npx`, `sqlx`, `mariadb`, `docker`, `gh issue create` / `comment` / `edit`, `gh pr …`,
SQL. Aucun fichier temporaire (les contrôles passent par un tube). Autorisés : lecture, `grep`, `sed -n`, `awk`,
`pdftotext … -` vers un tube, `git log` / `show` / `diff`, `gh api` en lecture.
