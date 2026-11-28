# Story 53-6a — prompt de la revue de code P2 (Sonnet, passe complète, contexte frais)

Tu conduis la **revue de code P2 de la story 53-6a de Kesh** avec le workflow BMAD `bmad-code-review` : invoque la compétence et suis-la. Tu travailles en contexte frais, avec le modèle **Sonnet** (la P1 était Opus). C'est une **passe complète**, non ciblée.

## Contexte

Kesh est un logiciel de comptabilité suisse ; Guy y tient ses livres réels en production. La story corrige un **P1** (#1674) : le dé-rapprochement contre-passait l'écriture à la date du jour, donc parfois dans un autre exercice. Désormais, il la contre-passe à sa propre date, dans son exercice (`CancellationDating::OriginDate`), avec un refus si l'exercice est clos (`FISCAL_YEAR_CLOSED`) ou si la date est sous la borne du verrou de période (`PERIOD_LOCKED`, seuil inclusif ; `400` du socle pour les bras unitaires, `409` sans `details` pour les groupes). **Une erreur ici fausse les livres.**

## Où travailler

- Worktree `/home/gcorbaz/devel/kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`. Relis la tête avec `git log -1` : la remédiation de la P1 a pu y ajouter des commits.
- Diff de la story : `95c577893..HEAD` (`95c577893` est la reprise de `main` à `0e99cd2db`).
- Fiche : `_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md`. Lis D1 à D6, les AC, les mutations et le Dev Agent Record.
- Rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/53-6a-revue-p1.md` (0/0/2/13). **Lis-le après avoir fait ta propre lecture**, pour ne pas t'y ancrer, puis recontrôle chaque remède au code.
- Preuves : `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/` (mutations dans `mutations/resume.txt`, captures dans `captures/` — **regarde les images**, gates de la station dans `gates-station/`).
- `CLAUDE.md` du worktree, en particulier : *Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Pattern batch*, *Recette visuelle*, *Recompter*, *Haiku-specific guardrails* (vérifier par `grep -nF` tout finding CRITICAL ou HIGH qui affirme une absence).

## Ce que la P1 a relevé (à recontrôler, non à recopier)

- **M1** — l'E2E d'AC-15 (1) ne joue pas une écriture d'un **exercice antérieur ouvert**, n'ouvre pas *Voir l'écriture* et ne vérifie pas l'exercice de l'inverse.
- **M2** — AC-5 à AC-7 (« AC-1 à AC-4 pour chacun ») : R5 n'a ni AC-2, ni AC-3, ni AC-4 ; R6 n'a pas de test dédié ; AC-3 n'est joué que pour R1 ; AC-5 ne vérifie ni la facture redevenue à régler ni le lettrage `reversal` dans N. Le Dev Agent Record affirmait une couverture « R2 à R5 » (AC-4) et citait R3 et R4 pour AC-1.
- **LOW L1 à L13** : borne affichée après « avant sa date. » ; phrase TVA du dialogue plus courte que le manuel ; « exercice Exercice 2026 » ; repli vide de `reversalDate` ; angle mort #568 non réécrit pour `OriginDate` ; motif de M7 (mutant équivalent) ; portée de la sonde d'AC-12 c ; AC-10 nul non testé ; attendus d'AC-9 pour R2 ; repli SQL de R3 ; recette visuelle ; AC-11 pour R8 ; course sur la pose du verrou.

## La remédiation de la P1 (commits à relire)

Diff de la remédiation : `d20ec0120..HEAD`. Commits (tous `refs #1674`) :

- `f28faee8f` — **M2** et les LOW de tests et de doc-comments (L5 à L9, L12, L13) : tests neufs par bras (R5 ×3, R6, AC-3 pour R1 à R4, AC-4 pour R2, AC-1 même exercice pour R1, AC-5 facture à régler et lettrage `reversal`), attendus d'AC-9 pour R1 et R2, `reversalDate` nul pour un retrait seul ; doc-comments de l'angle mort #568 sous `OriginDate`, du mutant équivalent M7, de la portée de la sonde d'AC-12 c, de la tolérance du verrou lu sans verrou.
- `9a12c5b0f` — **M1** (E2E d'AC-15 (1) : exercice antérieur ouvert, date d'écriture ≠ date de transaction, l'inverse ouverte par clics, son exercice vérifié) et L1 à L4, L11 (la borne en phrase à part, clé neuve aux quatre locales ; phrase TVA complète ; nom de l'exercice entre guillemets ; tiret si la vue est sans date ; captures neuves).
- les commits suivants : fiche (Dev Agent Record, Change Log) et ce prompt. Relis les tableaux « Tests par critère » et « Mutations » **nom par nom**.

**À contrôler en particulier** : (a) chaque test cité par le Dev Agent Record existe-t-il et vérifie-t-il ce qu'on lui attribue ? (b) la preuve du rouge des tests neufs (campagne `remediation/mutations/` sous `$GATE_LOGS/e53/s536a/` : une mutation de la datation par bras, `OriginDate` → `Today(chrono::Utc::now().date_naive())`, restauration par copie puis `touch`, `git diff` vide) et celle de l'E2E (binaire muté `remediation/kesh-api-mutant-R1`) ; (c) le recomptage `sitesTotal` 3976 et les quatre catalogues (`reconciliation-cancel-blocked-period-locked-through`, les cinq confirmations aux guillemets) ; (d) **le manuel** : la puce de la période verrouillée du § *Annuler un rapprochement* dit-elle la même chose que l'écran ? ; (e) le L10 (montage de repli de R3 pour AC-9) reste **non corrigé, avec sa raison** : la fiche admet le repli, et le chemin réel est joué pour R5.

Pour chaque remède, déclare-le **tenu**, **partiel** ou **absent**, en citant la ligne lue.

## Axes — déclare chacun exercé ou non exercé

1. **Chaque AC contre le code et les tests**, et chaque affirmation du Dev Agent Record (« Tests par critère », mutations, gates) recoupée nom par nom : le test existe-t-il, et vérifie-t-il ce qu'on lui attribue ?
2. **Inventaire clos des chemins d'annulation qui datent une écriture.** Pars du symptôme, non de la liste de la fiche : greppe dans `crates/kesh-db` et `crates/kesh-api` toutes les lectures de la date du jour (`aujourd_hui`, `horloge`) et toutes les créations d'écriture de contre-passation (`create_in_tx_inner(` avec `reverses_entry_id`, appelants de `reverse_in_tx`, `reverse_owned_in_tx`, `cancel_settlement*`, `grouped_settlements::cancel_in_tx`, valeurs `CancellationDating::`). Chaque site est résolu (date d'origine) ou écrit comme angle mort assumé (R7 à R9, datés du jour, Q3).
3. **Verrous et transaction** : ordre des verrous, lecture de l'exercice sous verrou, rollback complet sur refus (le lien défait à l'étape 5 est-il rétabli ?), et ce qu'apporte la sonde `FOR UPDATE NOWAIT` d'AC-12 c.
4. **Refus de période verrouillée** : seuil inclusif partout (règle pure, queue, socle), codes `400` et `409`, `details`, et conformité au *Pattern batch* si une route traite N éléments.
5. **TVA** : la contre-passation reprend-elle exactement les lignes, au même compte et à la même date ? Peut-elle encore tomber dans une autre période de TVA par le dé-rapprochement ?
6. **La survivante M7** : juge-la.
7. **L'écran** : annonce du refus avant le clic ; texte du motif avec la borne ; phrase TVA ; confirmation datée. **Regarde les captures.**
8. **Les quatre catalogues i18n** (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`) : mêmes clés, mêmes variables, replis Rust et TS identiques au FTL fr-CH.
9. **Le manuel, PDF compris.** Copie `docs/manual` sous `/tmp/claude-1000/` (jamais dans le worktree), régénère `fr/user-manual.pdf` et `fr/admin-manual.pdf` dans cette copie (`make fr/user-manual.pdf fr/admin-manual.pdf`), puis aplatis : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. Greppe `datée du jour`, `date du jour`, `aujourd'hui`, `0\.21\.0`. Vérifie aussi `docs/api-external.md` (vue, geste, table des refus, candidates, table générale des codes).
10. **Aucune migration** : `git diff --name-only 95c577893..HEAD -- crates/kesh-db/migrations crates/kesh-db/test-schema` doit rendre vide.
11. **La reprise de `main`** (`git show --cc 95c577893`), si la remédiation y a touché.

Un « 0 finding » sans la liste des axes exercés et non exercés ne compte pas comme passe.

## Tests que tu peux lancer

- Rust ciblé, sous `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, sur la base `kesh_e536` (`scripts/mem-guard.sh cargo nextest run -E 'binary(<binaire>)'`).
- Vitest d'un fichier, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-front.lock`.
- **Jamais de `GRANT` ni de `REVOKE`, jamais de redémarrage ni de recréation du conteneur.**

## Interdits

- Rien hors du worktree, de `/tmp` et de `/home/gcorbaz/devel/kesh-gate-logs/e53/`.
- Aucun correctif : tu n'appliques rien.
- Pas de `gh` en écriture, pas de `scripts/prepare-release.sh` ni d'autre script qui écrit, pas de `docker`, pas de `sed -i`, pas de `git push` ni de `git commit` hors de ce qui suit.
- Rien sous `~/kesh/`, aucune donnée réelle.
- Jamais `pkill`, `killall` ni `kill` par motif.

## Ce que tu produis

- **Rapport** : `/home/gcorbaz/devel/kesh-gate-logs/e53/s536a/53-6a-revue-p2.md`. Il donne les findings classés (CRITICAL, HIGH, MEDIUM, LOW), chacun avec sa preuve (ligne copiée, `grep -nF`) ; l'état de chaque remède de la P1 ; les axes exercés et non exercés ; un décompte recompté.
- **Prompt de la passe P3**, à côté de la fiche, s'il reste un finding au-dessus de LOW. La P3 se fait avec Opus, ou en passe ciblée selon *La passe ciblée* (jamais Haiku pour une passe complète).
- **Commit** du seul prompt : `git commit -- <chemin>`, en français, `refs #1674`, aucun mot-clé de fermeture, terminé par `Co-Authored-By: Claude Sonnet <noreply@anthropic.com>` ; puis pousse la seule branche `story/53-6-annulation-rapprochement-exercice`.

**Réponse finale**, en français : le décompte, une ligne par finding MEDIUM ou plus, les axes exercés et non exercés, et le chemin du rapport.
