# Validation P2 de la fiche 49-1d — passe complète, contexte frais, lecture seule

Tu valides la fiche `/home/gcorbaz/devel/kesh-49-1d/_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md`
(story 49-1d de Kesh, release 0.20.0, Epic 49, issue #1242), selon `bmad-create-story validate` et la *Review Iteration Rule* du
`CLAUDE.md`. **Passe complète** (non ciblée), contexte frais, modèle **Opus** (la P1 était Sonnet). Français soigné. Aucune
question : tu tranches et tu dis pourquoi.

## À lire d'abord : la P1 et sa remédiation

- Le rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e491d/validation-49-1d-p1.md` (0/0/6/4).
- L'entrée « Passe de validation P1 » du Change Log de la fiche, et le diff de la remédiation :
  `git -C /home/gcorbaz/devel/kesh-49-1d log --oneline -4` puis `git -C … show <commit "validation P1 remédiée">`.
- Pour chaque finding V1-1 à V1-10 : la remédiation le résout-elle **au fond**, sans en créer un autre ? La sévérité se déplace
  vers ce qu'on vient d'écrire : D1 (préconditions), D4 et D5 (300 dpi, décompte exact, numéro dans le texte), D8
  (`--designation-longue`, dérogation de la 49-4), D10 (gestes en phases 2 et 4, extraits comparés), D11 (script complémentaire
  hors dépôt), AC10 (rouges réels R1 à R4 par une copie mutée du script), T7, T11, T13 à T16, AC14 (manuel).

## Ensuite, tout le reste

- `/home/gcorbaz/devel/kesh-49-1d/CLAUDE.md` ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md`.
- Périmètre, qui fait foi : `_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md` (§ *49-1*, tableaux, *Contraintes
  de production*, la 49-4 à `:130` et `:154`, fusionnée par `d4f5bba7f`) et `release-0.20.0.md` (§ *Gates de la release*).
- Fiches closes `49-1a-types-de-lignes-migration-api.md` (§ *Transmis à la 49-1d*) et `49-1b-types-de-lignes-pdf.md` (D4).
- Le code à la tête du worktree : `scripts/recette-montee.sh`, `scripts/tests/recette-montee.test.sh` (en entier : la P1 ne l'a lu
  que par extraits — les factices peuvent-ils porter T1 à T16 ?), `crates/kesh-db/tests/recette_montee_script.rs`,
  `docs/testing.md`, `docs/manual/fr/admin-manual.tex`, `crates/kesh-api/src/routes/issued_invoice_pdf.rs`,
  `invoice_pdf_service.rs`, `company_invoice_settings.rs`, `reports.rs`, `crates/kesh-api/src/admin_backup/import.rs`,
  `crates/kesh-db/tests/migrations_upgrade_path.rs` (`line_types_migration_resumes_after_a_half_run` : D11 c en reprend-il
  fidèlement l'état ?), et le code du tag v0.19.0 par `git -C … show 773df80da:<chemin>`.
- Le précédent : `/home/gcorbaz/devel/kesh-gate-logs/rel0190/recettes-rel0190.sh`, `recette-complement-rel0190.sh`,
  `recette-refus-import-rel0190.sh`.

## Axes (tous à exercer)

1. **Remédiation de la P1** (ci-dessus), finding par finding.
2. **Périmètre** contre le fichier d'épique et le plan ; écarts déclarés justes et suffisants (dont D10 et D11, qui dépassent la
   lettre du tableau sur arbitrage de l'orchestrateur).
3. **Prémisses relues au code**, chaque `chemin:ligne` cité ; routes et corps de la **0.19.0** pour tout geste joué contre l'image
   publiée (peuplement, gestes de la phase 4, D11) ; la route d'import de l'assistant (D11 b) existe-t-elle sur une installation
   neuve de la 0.19.0, et sous quel nom ?
4. **Inventaires par le symptôme** (les deux tableaux) : listes closes ? refais-les par une autre voie.
5. **Décisions** : chacune tient-elle, l'option écartée l'est-elle pour une bonne raison ? En particulier : D10 (les deux phases
   partent-elles vraiment du même état ? un geste dont le résultat dépend de l'heure, d'un `version`, d'une séquence ou d'un rendu
   précédent, qui rendrait les extraits inégaux à tort ? les colonnes d'instant sont-elles toutes exclues, et seulement elles ?) ;
   D5 (le décompte 5 / 9 est-il juste ?) ; D11 (c) (l'état « à mi-chemin » posé à la main est-il celui que laisse sqlx ?).
6. **AC constatables** et cohérents entre eux (numérotation AC1 à AC16, renvois croisés, décomptes de D1, de D5, de D10).
7. **Preuve du rouge** : chaque test et chaque rouge réel (R1 à R4) rougit-il sans dépendre de l'hôte ? Une mutation qui survivrait
   en silence est un finding. La copie mutée sous `RACINE` (AC10) passe-t-elle les gardes du script (`sous`, `RACINE`) et ne reste-t-elle
   pas dans le dépôt ?
8. **Factures réelles en production** : AC9 dit-il que les factures et avoirs existants restent identiques (lignes, totaux, TVA,
   écritures, numéros, PDF), et la recette le prouve-t-elle pour une facture validée jamais téléchargée et pour un regel ? La
   réserve de D4 (« même dessin », non « même fichier ») et la dérogation de la 49-4 sont-elles dites partout où il le faut (fiche,
   manuel, Note de version) ? Rien ne touche la base du NAS ni `~/kesh/`.
9. **Manuel, PDF compris** : AC14 contre le code, la section 0.19.0 et ce que la 49-1a a transmis. Lis le manuel `.tex` **et** PDF :
   `pdftotext /home/gcorbaz/devel/kesh-49-1d/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis grep « 0.20.0 »,
   « 0.19.0 », « sauvegarde », « Compatibilité ».
10. **Gates, station, charge** : gates conformes au `CLAUDE.md` ; verrou et `mem-guard` ; bases `kesh_*_e491d` ; PDF non commités ;
    aucune migration ; splitting (modules comptés, D10 et D11 compris) ; l'estimation de charge des Dev Notes est-elle crédible pour
    la coupure du jeudi 2026-11-26 ?
11. **Sécurité des scripts** : périmètre fermé de `recette-montee.sh` intact (`KESH_MUTOOL`, `docker cp`, `--designation-longue`,
    copie mutée) ; les deux scripts hors dépôt (crochet, script complémentaire) relus comme du code, sans accès réel.

## Interdits

Lecture seule. **N'écris aucun fichier hors de ton rapport.** Ne lance **ni** `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, `flock`, `mutool` sur autre chose qu'un PDF du dépôt, **ni** aucun script de `scripts/` (`scripts/recette-montee.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/prepare-release.sh`, `scripts/tests/*`,
`scripts/synology/*`) ni de `kesh-gate-logs/` (`rel0190/*.sh`, `rel0200/*.sh`), **ni** `git commit`, `git push`, `git checkout`,
`git switch`, `git stash`, `git reset`, `git merge`, **ni** aucune commande `gh` qui écrit (`gh issue create`, `gh issue edit`,
`gh pr …`). Rien sous `~/kesh/`. Permis : `gh issue view`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `pdftotext`.

## Rapport

Écris `/home/gcorbaz/devel/kesh-gate-logs/e491d/validation-49-1d-p2.md` : verdict chiffré (CRITICAL / HIGH / MEDIUM / LOW) ;
pour chaque finding V1-n de la P1, « résolu » ou non, avec la preuve ; chaque finding neuf numéroté `V2-n`, avec sa sévérité, la
ligne de la fiche visée, la preuve (ligne de code citée, commande et sortie) et le remède proposé ; puis la liste des **axes exercés
et non exercés** (un « 0 finding » sans cette liste ne compte pas). Ton dernier message rend le chemin du rapport et le verdict.
