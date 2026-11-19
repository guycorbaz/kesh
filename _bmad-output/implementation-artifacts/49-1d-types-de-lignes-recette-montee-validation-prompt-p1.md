# Validation P1 de la fiche 49-1d — passe complète, contexte frais, lecture seule

Tu valides la fiche `/home/gcorbaz/devel/kesh-49-1d/_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md`
(story 49-1d de Kesh, release 0.20.0, Epic 49, issue #1242), selon `bmad-create-story validate` et la *Review Iteration Rule* du
`CLAUDE.md`. **Passe complète** (non ciblée), contexte frais ; modèle : **différent de celui de la rédaction (Opus)**, donc
**Sonnet**. Français soigné. Aucune question : tu tranches et tu dis pourquoi.

## À lire

- `/home/gcorbaz/devel/kesh-49-1d/CLAUDE.md` (règles ; § *Review Iteration Rule*, *Migration breaking policy*, *Ce qu'impose la
  production*, *Test Locally First*) ; `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md`.
- Le périmètre, qui fait foi : `_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md` (sections *49-1*, tableau des
  stories, tableau des fichiers, *Contraintes de production*) et `release-0.20.0.md` (§ *Gates de la release*).
- Les fiches closes `49-1a-types-de-lignes-migration-api.md` (§ *Transmis à la 49-1d*, Change Log) et
  `49-1b-types-de-lignes-pdf.md` (D4, § *Les chemins du PDF figé*).
- Le code, à la tête du worktree `/home/gcorbaz/devel/kesh-49-1d` (`git -C … log -1`) : `scripts/recette-montee.sh`,
  `scripts/tests/recette-montee.test.sh`, `crates/kesh-db/tests/recette_montee_script.rs`, `docs/testing.md`
  (§ *La recette de montée et de retour*), `docs/manual/fr/admin-manual.tex` (sections « Passer à la 0.20.0 » et « 0.19.0 »,
  paragraphe « Compatibilité »), `crates/kesh-api/src/routes/issued_invoice_pdf.rs`, `crates/kesh-api/src/admin_backup/import.rs`,
  `crates/kesh-api/src/lib.rs`, `docs/api-external.md`, et le code du tag v0.19.0 par `git -C … show 773df80da:<chemin>`.
- Le précédent : `/home/gcorbaz/devel/kesh-gate-logs/rel0190/recettes-rel0190.sh`, `recette-complement-rel0190.sh`.

## Axes (tous à exercer)

1. **Périmètre** contre le fichier d'épique et le plan : rien de promis qui manque (peuplement : validées payées et impayées, un
   avoir, un règlement groupé, PDF figés et non figés ; `credit_notes` et `credit_note_lines` à `TABLES` ; `--diff-attendu` des
   deux tables de lignes ; `docs/testing.md` ; section « Passer à la 0.20.0 » avec la sauvegarde en tête ; import d'une sauvegarde
   0.19.0 ; rejeu au dernier commit de code), rien d'ajouté sans le dire. Les écarts déclarés (manuel FR seul, `api` nommé,
   fichiers hors tableau) sont-ils justes et suffisants ?
2. **Prémisses relues au code** : chaque citation `chemin:ligne` de « Ce qui se lit au code » et des décisions — la ligne
   existe-t-elle, dit-elle ce que la fiche lui fait dire ? En particulier : les routes et corps de la **0.19.0** (le peuplement se
   joue contre l'image publiée, non contre la tête) ; la copie des documents par `docker cp` (le conteneur est `--rm` ; l'ordre
   avec `arreter_image`) ; le comportement de `refreeze` (fichier manquant seulement) ; la présence des pièces dans le
   `.keshbackup` ; ce que voit `mutool` d'un PDF (le `/ID` et la date ne sont-ils vraiment pas dessinés ? un texte du jour dans le
   rendu ?).
3. **Inventaire par le symptôme** : la liste des sites qui ne sont pas comparés est-elle close ? Refais la recherche par une autre
   voie (colonnes `invoice_id` / `credit_note_id` ajoutées par `ALTER`, lettrage, rapprochement, `bank_transactions`, documents) ;
   tout site manquant est un finding.
4. **Décisions** : chacune tient-elle, et l'option écartée l'est-elle pour une bonne raison ? Spécialement D2 (projection : un
   cas où elle laisse passer une altération ? un cas où elle rougit à tort — ordre des colonnes, `ORDER BY 1`, types, `NULL`,
   tableaux JSON réordonnés ?), D3 (droits des fichiers copiés, `docker cp` d'un dossier absent), D5 (la phase 4 rend-elle bien
   sans altérer l'empreinte comparée ? une facture rendue en phase 4 se fige-t-elle avant `empreinte retour` ?), D8 (interaction
   avec la 49-4, la 48-15, la PR #1327).
5. **AC constatables** : chaque AC se constate-t-il par une commande ou une sortie ? Les décomptes de l'AC1 (statuts, règlements,
   figées) sont-ils cohérents avec D1 et avec le code de la 0.19.0 ?
6. **Preuve du rouge** : chaque test neuf rougit-il vraiment sans le correctif ? Une mutation qui survivrait en silence (factice
   qui ne distingue pas les cas, témoin vide) est un finding. Le rouge réel de l'AC9 prouve-t-il ce qu'il dit ?
7. **Factures réelles en production** : un AC dit-il que les factures et avoirs existants restent identiques (lignes, totaux, TVA,
   écritures, numéros, PDF), et la recette le prouve-t-elle pour une facture **validée jamais téléchargée** et pour un **regel** ?
   Rien ne touche la base du NAS ni `~/kesh/` ; aucune donnée réelle.
8. **Manuel, PDF compris** : AC12 décrit-il ce qu'exige le `CLAUDE.md` (sauvegarde d'avant la mise à jour en tête : base,
   `.keshbackup`, dossier des documents) et ce que la 49-1a a transmis (message exact d'`IMPORT_SCHEMA_MISMATCH`, procédure de
   reprise) ? Lis le manuel tel qu'il est, `.tex` **et** PDF : `pdftotext /home/gcorbaz/devel/kesh-49-1d/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   puis grep « 0.20.0 », « 0.19.0 », « sauvegarde ». Un écart entre le texte prévu et le code est un finding.
9. **Gates et règles** : gate ciblé, `gate-ci`, E2E complet au dernier commit, verrou et `mem-guard`, bases `kesh_*_e491d`,
   PDF non commités, aucune migration (P1 à P8 sans objet ?), splitting (modules comptés).
10. **Sécurité du script** : l'invariant d'environnement fermé tient-il avec `KESH_MUTOOL`, `docker cp` et les fichiers neufs sous
    `$D` ? Le crochet hors dépôt est-il relu ?

## Interdits

Lecture seule. **N'écris aucun fichier hors de ton rapport.** Ne lance **ni** `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, `flock`, **ni** aucun script de `scripts/` (`scripts/recette-montee.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
`scripts/mem-guard.sh`, `scripts/prepare-release.sh`, `scripts/tests/*`, `scripts/synology/*`), **ni** `git commit`, `git push`,
`git checkout`, `git switch`, `git stash`, `git reset`, `git merge`, **ni** aucune commande `gh` qui écrit (`gh issue create`,
`gh issue edit`, `gh pr …`). Rien sous `~/kesh/`. `gh issue view` et `git show`, `git log`, `git grep`, `grep`, `sed -n`,
`pdftotext` sont permis.

## Rapport

Écris `/home/gcorbaz/devel/kesh-gate-logs/e491d/validation-49-1d-p1.md` : verdict chiffré (CRITICAL / HIGH / MEDIUM / LOW) ;
chaque finding numéroté `V1-n`, avec sa sévérité, la ligne de la fiche visée, la preuve (ligne de code citée, commande et sortie)
et le remède proposé ; puis la liste des **axes exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas).
Ton dernier message rend le chemin du rapport et le verdict.
