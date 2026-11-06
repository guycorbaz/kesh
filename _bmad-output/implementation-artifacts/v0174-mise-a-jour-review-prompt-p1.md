# Revue de code P1 — v0174-mise-a-jour-1 (#847), trois lentilles B, E, A (Opus, lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **v0174-mise-a-jour-1** de Kesh
(issue #847 : la sauvegarde avant migration, **version minimale**). Modèle : **Opus**. Contexte frais. Travaille en
français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche les défauts que le code porte en lui-même (erreurs
  avalées, chemins d'échec qui migrent quand même, données perdues, concurrence, panique, durabilité, messages faux).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — plan (*neuve*, *interrompue*, *à jour*, *à
  appliquer*, illisible, incohérent), échappatoire (égale, autre, vide, absente) × plan, dossier, espace (égal au seuil,
  illisible), lecture, écriture, panique, synchronisation du dossier, nom (versions hors forme, longueurs), base partielle
  dont l'inventaire diffère, restauration dans la version de départ.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC, test par test (T1 à T16), mutation par mutation
  (M1 à M22, journaux cités ci-dessous), manuel `.tex` **et PDF**, CHANGELOG, compose, `.env.example`, registre.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-0174m log --oneline 7bd4d5c0..HEAD
git -C /home/gcorbaz/devel/kesh-0174m diff 7bd4d5c0 HEAD -- crates docker-compose.yml docker-compose.prod.yml .env.example CHANGELOG.md docs/manual/fr/admin-manual.tex
```

Lis le diff **aplati** (un seul diff de `7bd4d5c0` à `HEAD`). Fichiers principaux :
`crates/kesh-api/src/pre_upgrade_backup.rs` et `pre_upgrade_backup/tests.rs` (neufs), `crates/kesh-api/src/main.rs`
(étape 3c), `crates/kesh-db/src/backup.rs` (`Inventaire`, `lire_installation`, `export_installation_schema_present`,
`restore_tables_in_tx_avec_inventaire`, garde T16), `crates/kesh-api/src/admin_backup/{export,import}.rs`,
`auto_backup.rs` (visibilités, `BackupEnv::sync_dir`), `routes/admin.rs` (`ensure_backup_dir`), `config.rs`,
`tests/sauvegarde_avant_migration.rs` (T8), `tests/configuration_transmise.rs`, `kesh-db/tests/test_schema_guard.rs`.

Fiche : `/home/gcorbaz/devel/kesh-0174m/_bmad-output/implementation-artifacts/v0174-mise-a-jour-1.md` (lentille A ;
B la lit **après** son examen à l'aveugle, pour classer). Registre :
`v0174-choix-autonomes-v0174m.md` (choix en vigueur : dernière section). Journaux : rouges
`/home/gcorbaz/devel/kesh-gate-logs/v0174m/rouges.log`, mutations `/home/gcorbaz/devel/kesh-gate-logs/v0174m/mut/M*.log`,
gates et recette `/home/gcorbaz/devel/kesh-gate-logs/v0174m/` (le Dev Agent Record de la fiche les cite).

## ⛔ Ne pas réintroduire ce que la version minimale a retiré

La fiche a été **réduite par décision du Project Lead** (C-v0174m-30) après une non-convergence. Ne demande **pas** :
l'écran « Avant mise à jour », une rétention ou une suppression automatique, une règle de choix du fichier au journal,
un message après `MIGRATOR.run`, la garde de format `TYPES_DECODES`, un refus de version de départ `< 0.15.0`, une
relecture de l'archive, le nettoyage du `.partial` sur panique, les avertissements W1 à W3, une seconde jambe de recette,
une procédure de restauration neuve (`DROP`/`CREATE DATABASE`). Un défaut **dans** le périmètre retenu est un finding ;
l'absence d'un élément retiré n'en est pas un.

## Axes obligatoires

1. **Aucune migration sans sauvegarde ni refus** : tout chemin de `sauvegarder_avant_migration` et de `main.rs` qui
   laisserait `MIGRATOR.run` s'exécuter alors que la sauvegarde n'est ni écrite, ni contournée par l'échappatoire égale.
2. **Importabilité par la version de départ** : l'inventaire du schéma présent égale-t-il le `TABLES_TO_TRUNCATE` de la
   version qui a créé ce schéma (convention du préfixe `_`, T16) ? T1-ter le prouve-t-il vraiment (oracle littéral,
   seconde base, comparaison brute) ?
3. **Production inchangée** : export, sauvegarde automatique, pré-import et import passent toujours par l'inventaire du
   binaire ; `table_count` ; gardes lexicales récrites (`chaque_lecture_n_a_qu_un_appelant`) — mordent-elles encore ?
4. **Messages** : textes exacts de la fiche (§ *Messages*), jamais `DATABASE_URL`, conseil de l'échappatoire absent du
   seul refus *incohérent*.
5. **Le manuel** — `docs/manual/fr/admin-manual.tex` **et** `docs/manual/fr/admin-manual.pdf` : contrôle le PDF aplati
   (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `/usr/bin/grep -oF`) ; § *La sauvegarde
   avant mise à jour*, § *Revenir à la version précédente*, variables, `KESH_ADMIN_BACKUP_DIR`. Le manuel dit-il vrai
   contre le code (cas sans sauvegarde, refus, échappatoire, compose antérieur, `.partial`, aucune suppression) ?
6. **Configuration** : compose (deux), `.env.example`, `configuration_transmise` (`SANS_DEFAUT`, compte 31).
7. **Tests et mutations** : chaque mutation est-elle tuée par le test que la fiche nomme, pour la bonne raison (lis les
   `M*.log`) ? Un test passe-t-il à vide (assertion de montage) ?

## ⛔ Ground-truth obligatoire

Toute affirmation de présence ou d'absence d'un texte s'appuie sur une commande
`/usr/bin/grep -nF "<chaîne exacte>" <fichier>` (ou `git -C … show HEAD:<fichier> | /usr/bin/grep -nF …`), **dont tu
copies la sortie**. Bloc multi-lignes : `-A5`. (`grep` est une fonction vers `ugrep` : utilise **`/usr/bin/grep`**.)

## Sévérités

CRITICAL (migration possible sans sauvegarde ni refus, perte ou corruption de données), HIGH (AC faux, sauvegarde
inimportable par la version de départ, test qui ne prouve rien), MEDIUM (incohérence code/fiche/manuel, mutation tuée
pour une mauvaise raison, manuel faux), LOW (rédaction, nit).

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
`mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/` ; pas de `git checkout`/`switch`,
`git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase` ; aucune commande `gh` qui écrit ; pas de
`claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi`, `tee`, **aucune redirection `>` ni fichier temporaire**.
**Tu n'écris qu'un seul fichier** : ton rapport.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/v0174m-review-p1-<B|E|A>.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`B1-1`, `E1-1`, `A1-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif
   proposé.
3. **Axes exercés et non exercés** — liste explicite ; un « 0 finding » sans cette liste ne compte pas.
