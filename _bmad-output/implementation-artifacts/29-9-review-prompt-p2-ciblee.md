# Prompt — revue de code P2 ciblée, Story 29-9 (lentille T : la remédiation de la passe P1)

*Versionné le 2026-10-15. Une seule lentille, contexte frais, lecture stricte. Modèle : celui que désigne
l'orchestrateur (Haiku est admis pour une passe ciblée — CLAUDE.md, § passe ciblée).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9`.
**Le seul diff à relire** : le commit de remédiation de la revue P1, c'est-à-dire `git diff 4a7c27ac HEAD` (`HEAD` =
ce commit ; `git log -2 --format=%h\ %s` le confirme). Il touche **quatre** fichiers : la fiche
`_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md`, `docs/manual/fr/user-manual.tex`,
`docs/manual/fr/user-manual.pdf` et ce prompt. **Aucune ligne de code de production ni de test** (décision de
l'orchestrateur) : `git diff --stat 4a7c27ac HEAD -- crates frontend` doit être **vide** — c'est le premier contrôle.
Rapports P1 remédiés : `/home/gcorbaz/devel/kesh-gate-logs/299-review-p1-{B,E,A}.md`.

## Ce qui a été fait (à vérifier, pas à croire)

1. A-1 : commande de rebase de la fiche (*Demandes à l'orchestrateur*) = `git rebase --onto origin/main
   1261c4bb8725fec33f6b7920947ed005ee8bd41b story/29-9`.
2. A-2 : saut de défilement de l'AC2 (C-29-9-15) déclaré au Dev Agent Record, « capture non prise ».
3. A-5 : manuel, puce *Rejeter* (`user-manual.tex` ~2101) et phrase de rattachement d'une transaction rejetée (~2134) :
   « jusqu'à la 500\textsuperscript{e} transaction en attente ».
4. A-3, A-4, A-6, A-7 : annotations datées au Status, aux *Debug Log References*, aux décomptes, aux mutations et aux
   tests rouges ; entrée *Revue de code P1* au Change Log.

## Axes (déclare chacun exercé ou non exercé)

1. **Périmètre** : le diff ne touche aucun fichier de `crates/` ni de `frontend/` ; seul `user-manual.pdf` change parmi
   les PDF (`git diff --stat 4a7c27ac HEAD`).
2. **Vérité au code** (le plus cher) : chaque phrase neuve de la fiche dit-elle vrai **au code** ?
   - La description du saut de défilement colle-t-elle à `ReconciliationProposals.svelte` (`showOlder`, `load()`,
     `{#if loading}`, `loading` non gardé par `loadGen` sauf en `finally`) ? Dit-elle « capture non prise » sans
     prétendre l'avoir observé ?
   - « jusqu'à la 500e » est-il vrai de l'écran et de `docs/api-external.md` (paragraphe *unreject*) ? Que devient une
     transaction rejetée au-delà de la 500e : le manuel le dit-il comme le code (avis plafonné, sans bouton) ?
   - A-6 : les dates citées (`299-mut-t3.log` 23:50:45 ; `85d8d9ea` 23:54:23) et la liste M9, M10, M13, M16, M17, M19
     se retrouvent-elles dans les journaux et dans `git log -1 --format=%ad 85d8d9ea` ?
   - A-7 : les journaux nommés existent-ils dans `/home/gcorbaz/devel/kesh-gate-logs/` ? Contiennent-ils bien les
     sorties de tests rouges sous mutation ?
3. **Recomptes** : le Change Log annonce « 15 passés » (gate ciblé), « 2 » occurrences au PDF, « 0 » pour « ?? »,
   B 0/0/0/5, E 0/0/0/9, A 0/0/2/5 : recompte à la source (rapports P1 : `grep -c '^### '` ou lecture), périmètre
   déclaré avec chaque nombre. Les entrées passées du Change Log (P1 de validation, Développement, Report) sont-elles
   **intactes** (annotations datées seulement, pas de réécriture) ? `git diff 4a7c27ac HEAD` ne doit montrer que des
   lignes ajoutées dans ces entrées, hors la commande de rebase corrigée et les deux sites annotés.
4. **Propagation post-patch** : greper la **valeur**, pas la formulation. `grep -rnE "onto origin/main" _bmad-output
   docs` (aucune autre commande vivante avec `4b358373`) ; `grep -rn "500" docs/api-export*` n'est pas requis ; en
   revanche cherche tout autre site du manuel qui parle du retour d'une transaction rejetée dans la liste sans la
   réserve : `grep -n "rejet" docs/manual/fr/user-manual.tex` (lignes ~1725-1731, ~2101, ~2134), `docs/api-external.md`,
   `README.md`, `website/`, le CHANGELOG § `[0.13.4]`.
5. ⛔ **MANUEL ET PDF** (obligatoire) : lis les deux phrases du `.tex`, puis **contrôle le PDF aplati en pipe** —
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "jusqu’à la 500e transaction en
   attente"` (apostrophe typographique « ’ » et ligature « ﬀ » dans le texte extrait : un `grep` naïf rend un faux
   négatif ; **le `500e` apparaît** sans exposant ni espace insécable parasite) : attendu **2** occurrences, **0**
   « ?? », « la page le dit » ×2. Le PDF est-il postérieur au dernier `.tex` modifié (même commit : cherche un indice
   — texte du PDF contenant la phrase neuve — plutôt qu'une date) ? `\textsuperscript{e}` est-il correct au `.tex`
   (accolades équilibrées, pas de `\\` ni de `%` introduits) ? Le reste de la puce *Rejeter* (29-6, 29-8) est-il
   **inchangé** ?
6. **Mots-clés de fermeture** : aucun message de commit de la branche ne porte `closes`/`fixes`/`resolves #N`, même en
   prose (commande de la fiche, l. ~25-27, sur `1261c4bb..HEAD`).

## Ce que tu rends

Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/299-review-p2-T.md` — **le seul fichier que tu écris**. Findings avec
sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie), correction proposée. Pour tout
finding affirmant une absence ou une présence : la sortie d'un `grep -nF` copiée. ⛔ **La liste des axes exercés ET
non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ **Aucun fichier écrit hors du rapport ci-dessus** — pas même le texte aplati du PDF : il se contrôle **en pipe**,
sans fichier (la passe P1 en avait écrit deux dans `kesh-gate-logs`, c'était un écart). Aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`
(**aucun `git checkout` ni `git switch` sous aucune forme**), `sqlx`, `cargo`, `npm`, `npx`, `node`, `python`/`python3`,
`perl`, `ruby`, `sed -i`, `awk` en écriture, `mariadb`, `mysql`, `docker`, `curl`, `wget`, `gh` en écriture,
`pkill`/`kill`. Aucun fichier hors du dépôt, hors `/home/gcorbaz/devel/kesh-gate-logs/` (lecture). Autorisés :
lecture, `grep`, `sed -n`, `wc`, `cmp`, `git log`/`show`/`diff`, `pdftotext` **vers la sortie standard**.
