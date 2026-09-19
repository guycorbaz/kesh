# Prompt — passe 1 de `bmad-code-review`, Story 25-2-c

*Versionné le 2026-09-19. **Trois lentilles** en contexte frais, orthogonales à l'auteur de
l'implémentation (Opus 5) : Blind Hunter (Sonnet), Edge Case Hunter (Haiku 4.5), Acceptance
Auditor (Sonnet).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-c-numero-ecriture-par-compteur`.

## L'objet

**Le diff `main...HEAD` APLATI, hors documentation BMAD et PDF** — 19 fichiers, 1382 lignes de
diff :

```sh
git diff main...HEAD -- . ':(exclude)_bmad-output' ':(exclude)*.pdf'
```

Une copie figée est dans le scratchpad de la session :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/b4a4f4f4-facb-4366-8d5d-2d8994016d0b/scratchpad/review-25-2-c-code.diff`.

⚠️ **Lis le diff aplati, jamais la séquence de commits** : les numéros de ligne d'un diff
multi-commit désignent des états intermédiaires et font conclure à l'absence d'un patch qui est bien
là.

La spécification est `_bmad-output/implementation-artifacts/25-2-c-numero-ecriture-par-compteur.md`
— **12 critères**. Son Change Log déclare deux gates verts (backend 2335/2335, E2E 214/9 attendus),
des mutations tuées et huit garde-fous réparés : **ne les crois pas sur parole, et cherche ce
qu'ils n'ont pas vu.**

## Ce que la story fait

Le numéro d'écriture (`journal_entries.entry_number`) était tiré d'un `MAX(entry_number) + 1` :
supprimer la **dernière** écriture faisait réattribuer son numéro. La story introduit une table
`journal_entry_number_sequences` (migration `20260917000001`, amorcée depuis l'existant) et un
allocateur `repositories/journal_entry_number_sequences.rs::next_number_for`, calqué sur
`invoice_number_sequences`. `create_in_tx` l'appelle.

⛔ **Une décision de conception n'est PAS dans la spec, et elle est contestable** : l'allocateur
retient **le plus grand** de son compteur et de `MAX(entry_number) + 1` (« plancher »). Elle a été
prise pendant l'implémentation pour qu'une écriture insérée hors de l'allocateur ne bloque pas
toute création. Examine-la comme du code neuf : tient-elle la propriété de l'AC 2 dans **tous**
les cas, ouvre-t-elle une fenêtre de concurrence, masque-t-elle un défaut ?

Autour : nettoyage du compteur dans les helpers de test et dans `kesh-seed` (FK `RESTRICT` vers
`fiscal_years`), entrée de la table dans `backup.rs::TABLES_TO_TRUNCATE`, triage P7 dans
`post_restore.rs` (la migration **exemptée**, deux entrées retirées du registre de rejeu), squash
de test régénéré, registre de checksums, compteurs de `migrations_upgrade_path.rs`, tests d'import
et d'export, manuel utilisateur, CHANGELOG.

⚠️ **Ne conteste PAS** : le choix d'un compteur persistant calqué sur `invoice_number_sequences` ;
le trou assumé (AC 3) ; l'absence de bump de version (AC 8) ; le fait que l'en-tête de la
migration `20260917000001` dise « la contrainte d'unicité refuserait l'insertion » — c'est
périmé depuis le plancher, **connu**, consigné dans la fiche, et le fichier est figé par P8.

## ⛔ Les axes, et tu déclareras lesquels tu as exercés

1. **L'allocateur.** Ordre des verrous : `SELECT … FOR UPDATE` sur le compteur, puis lecture du
   `MAX` — sérialisation réelle de deux créations concurrentes ? Risque d'interblocage avec un
   autre chemin qui verrouille `journal_entries` puis le compteur ? ⚠️ **`INSERT IGNORE` en
   MariaDB transforme AUSSI en avertissement une violation de FK ou de `CHECK`**, pas seulement le
   doublon : que se passe-t-il pour un `fiscal_year_id` inexistant ? Le contrôle
   `rows_affected() == 1` est-il atteignable dans les cas qu'il prétend couvrir ?
2. **Tous les chemins qui créent une écriture passent-ils par l'allocateur ?** Inventorie **tous**
   les `INSERT INTO journal_entries` du dépôt hors tests (clôture, réouverture, import, extourne,
   rapprochement, seed) — et, pour chacun qui n'y passe pas, dis ce que le plancher en fait.
3. **Tous les chemins qui suppriment un exercice ou une société.** La FK `RESTRICT` du compteur
   rend-elle impossible une suppression que le produit **livré** effectue (reset démo, onboarding,
   import d'installation, suppression d'exercice vide) ? `kesh-seed` a été corrigé : y en a-t-il
   d'autres ? Cherche **large**.
4. **Sauvegarde et restauration.** La table est-elle exportée, importée dans le bon ordre de FK,
   vidée à l'import ? La justification des trois exemptions de `post_restore.rs` est-elle **vraie**
   (recalcule la fenêtre) ? Après restauration d'un backup, le compteur peut-il se retrouver
   en-dessous des numéros en service — et le plancher le rattrape-t-il ?
5. **Les tests prouvent-ils ce qu'ils prétendent ?** Cherche l'assertion qui passerait **même si
   le code était faux**. Le test décisif de l'AC 2, le test de concurrence de l'AC 9, le test
   d'amorçage de l'AC 4 (`tests/journal_entry_number_sequences_bootstrap.rs`) : chacun rougirait-il
   si sa propriété était violée ? Les tests réécrits d'`admin_full_import_e2e.rs` (« substitution
   de source ») testent-ils encore ce qu'ils testaient ?
6. **Les garde-fous de migration (P5 à P8).** Compteurs de `docs/migrations-idempotence-audit.md`
   recomptés **depuis la source** (`ls crates/kesh-db/migrations/*.sql | wc -l`, lignes du
   tableau, somme des partitions) ; `migrations_upgrade_path.rs` sans résidu des anciennes valeurs
   (grep des **jetons** `68` et `34`) ; squash régénéré et non édité ; checksum exact.
7. **Les commentaires et les messages.** Chaque doc-comment ou message ajouté dit-il **vrai** ?
   Un total qui se périmera ?
8. **Les manuels et le CHANGELOG.** `docs/manual/fr/user-manual.tex` dit-il vrai, sans
   sur-promettre (le trou subsiste, la réattribution est fermée) ? ⚠️ Contrôle le **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`). Cherche un **autre**
   site qui parle de numérotation des écritures — manuels DE/IT/EN, admin-manual, `README.md`,
   `website/` — et qui mentirait désormais.

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`, **la
  commande ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de
  départ est atteignable** en production.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe. **Ne qualifie jamais de « vérifié » un point que
  tu n'as pas exécuté.**

## Discipline imposée par ce dépôt

⛔ **Tout finding `CRITICAL` ou `HIGH` qui affirme l'ABSENCE d'un code attendu ou la PRÉSENCE d'un
anti-pattern doit être établi par un `grep -nF` (fixed-string) cité dans ton rapport.** Sans cette
preuve il sera écarté.

⚠️ **P8** : la migration `20260917000001` n'est pas publiée, mais elle est appliquée aux bases de
dev. Un finding qui exige de **modifier le fichier de migration** doit le dire explicitement.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh` (il bumpe les versions Cargo), `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/`reset`, `sqlx migrate`,
`cargo fmt` sans `--check`, et toute écriture dans les bases `kesh` ou `kesh_e2e`. Lecture de
fichiers, `grep`, `git diff`/`log`/`show`, `pdftotext` et `cargo check` sont autorisés.
