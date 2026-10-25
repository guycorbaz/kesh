# Prompt — validation P3, Story 32-4b — passe ciblée (regression hunter)

*Versionné le 2026-10-25. Passe **ciblée** : **une seule lentille** (regression hunter), contexte frais, **Sonnet**.
Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-4b`, branche `story/32-4b`. **Ta cible est un seul commit** : la remédiation de la
validation P2, **`d7783685`** (`git show d7783685`). Elle ne touche que la fiche
`_bmad-output/implementation-artifacts/32-4b-services-factures.md`, le registre `epic-32-choix-autonomes-324b.md` et
`sprint-status.yaml` — aucune ligne de code. Les rapports de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/324b-validate-p2-{R,F}.md`
(lis-les **après** le diff). Le code de référence est celui de `main` `592ee430` (`git diff --stat 592ee430 d7783685 --
crates docs` est vide).

Pourquoi cette passe : *la sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Ce qui
reste à relire n'est plus la fiche : c'est cette remédiation. Une boucle ne se clôt que sur une passe sans finding
au-dessus de LOW.

⛔ **Aucune commande `gh`**, pas même en lecture.

## Axes, tous à exercer

1. **Chaque hunk de `d7783685`** : traite-t-il le finding de la P2 qu'il vise (R1 = F1, R2 = F6, R3, R4, R5, F2, F3, F4,
   F5, F7, F8) — et lui seul ? Un hunk introduit-il une affirmation fausse au code, une contradiction avec une autre
   partie de la fiche (AC, tâches, *Fichiers touchés*, *Risques*, registre), ou un nombre non recompté ?
2. **Le témoin de M12b (AC11 (a), AC15)** : `" " + "a"×100 + " "` est-il accepté aujourd'hui, par la liste
   (`routes/invoices.rs:619-632`) **et** par l'échéancier (`:1065-1077`) ? Est-il refusé sous M12b (rognage retiré :
   102 > 100) ? Le résultat est-il bien « même résultat que `"a"×100` » (relis `push_where_clauses`,
   `kesh-db/src/repositories/invoices.rs:425-450`, et `due_dates_summary`, `:984-1010`) ? Une autre ligne de l'AC11 (a)
   en devient-elle fausse ?
3. **Le contrôle Q11 (fait 10, AC17 (3))** : exécute le motif tel qu'écrit sur `crates/` — rend-il exactement les dix
   lignes annoncées ? Le texte que l'AC17 (3) prescrit (« aucun second compte MariaDB », la citation « la réponse de
   Guy à Q11 », la phrase de `lecture_seule.rs`) peut-il rendre le contrôle non vide ? Les bornes `147-149`, `158-160`,
   `8-9` sont-elles justes ?
4. **Les bornes de `dueBefore` (R5)** : les trois formes acceptées écrites sont-elles acceptées par
   `build_due_dates_query` (`routes/invoices.rs:1044-1104`) ?
5. **F3, F4, F7, F8** : l'interdiction de nommer `build_due_dates_query` est-elle compatible avec l'AC5 (« le corps
   déplacé, commentaires compris ») ; les lignes `services_rollout.rs:381, 402, 551` sont-elles celles que la fiche cite ;
   quatre factures **validées** et un TTC multiple de 0.05 (`kesh-db/src/test_fixtures.rs:203-212`) sont-ils compatibles
   avec les autres exigences des fixtures (C-32-4b-15, AC12 (a)) ?
6. **F5 et C-32-4b-18** : l'ordre de fusion écrit est-il cohérent entre le registre, *Risques* et T0 ? La consigne « si
   `main` porte déjà la 32-5b » est-elle exécutable ?
7. **Le Change Log P2** : bilan cohérent avec les rapports (13 findings, 11 distincts) ; la déclaration de recyclage
   est-elle juste (M12b d'origine ou né de la P1 ? — relis `git show ced11a88` pour l'AC15 d'origine) ; la propagation
   déclarée a-t-elle laissé un résidu (greppe les valeurs corrigées : `2124-2133`, `147-148`, `158-159`, `(l. 9`,
   `neuf lignes`, l'ancien motif `par Guy \(Q11\)`, `C-32-4b-1 à 17`) hors des entrées datées du Change Log ?

## Ce que tu rends

Findings `P3-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et sortie
copiée), correction proposée. Bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non
exercés** (1 à 7) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet** :
`/home/gcorbaz/devel/kesh-gate-logs/324b-validate-p3.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`). Aucun `git checkout`/`switch` ; **aucune commande qui écrit, compile ou exécute** :
`scripts/*` (dont **`scripts/prepare-release.sh`** et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`, `rustup`,
`npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes**. Aucune donnée réelle de
Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés :
lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, et les filtres en lecture seule `tr`, `sort`, `cut`,
`head`, `awk`, `wc`, `uniq` vers la sortie standard.
