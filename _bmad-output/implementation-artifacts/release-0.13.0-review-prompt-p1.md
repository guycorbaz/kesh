# Prompt — revue documentaire P1 de la release 0.13.0 (deux lentilles, Sonnet)

Tu relis la préparation de la release **v0.13.0** de Kesh (comptabilité suisse), dans le
worktree `/home/gcorbaz/devel/kesh-release`, branche `chore/release-0.13.0`, partie de
`origin/main` `d2460a2d`. Le diff à relire : `git -C /home/gcorbaz/devel/kesh-release diff d2460a2d HEAD`.
Langue du rapport : français.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute AUCUN script du dépôt** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni `make`, ni `latexmk`, ni `cargo`, ni `npm`, ni rien qui compile,
  teste ou écrive. Aucun `git` qui écrit (`commit`, `checkout`, `stash`, `tag`, `push`, `reset`).
  Aucune commande `gh` qui écrit ; `gh issue view` (lecture) est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `pdftotext`
  (vers la sortie standard uniquement), `gh issue view`.

## Lentille A — les textes publiés contre le code et contre `main`

Vérifie que chaque affirmation des textes neufs ou modifiés est vraie dans le code :

1. **Manuel d'administration** (`docs/manual/fr/admin-manual.tex`), paragraphe
   `sec:maj-0-13-sans-retour` et étape 1 de la procédure de mise à jour : le relèvement de
   `kesh_version_min_required` (migration `20261009000001_journal_entry_lines_lettering.sql`),
   le refus de démarrage (`crates/kesh-db/src/version.rs`, `check_downgrade_protection`,
   `crates/kesh-api/src/main.rs`) — **le message cité est-il exactement celui que logue un
   binaire 0.12.1** (`git show v0.12.1:crates/kesh-db/src/version.rs`) ? —, le refus d'import
   (`crates/kesh-api/src/routes/admin.rs`, code HTTP, code d'erreur, message affiché par l'écran
   **de la 0.12.1** : `git show v0.12.1:frontend/src/lib/features/admin-restore/AdminRestorePanel.svelte`
   et le catalogue fr-CH du tag), la voie de retour (un dump recrée-t-il `_kesh_version` ?
   un `.keshbackup` importé dans la 0.13.0 l'abaisserait-il ? `crates/kesh-db/src/backup.rs`),
   la phrase « installation à neuf non concernée ».
2. **Épinglage des URL** : chaque URL `raw.githubusercontent.com/guycorbaz/kesh/v0.13.0/…` du
   manuel désigne-t-elle un fichier qui existe dans l'arbre (`git show HEAD:<chemin>`) ? Les
   phrases ajoutées autour sont-elles justes ? La garde G19
   (`crates/kesh-api/tests/textes_coherents.rs`, `les_url_raw_du_depot_pointent_sur_la_version_publiee`)
   couvre-t-elle bien manuels, README et `website/` ? Une forme d'URL lui échappe-t-elle
   (`raw/main` de `github.com`, URL coupée par `\` en fin de ligne, `$DEPOT/…`) ?
3. **CHANGELOG** section `[0.13.0]` : l'avertissement et la liste « À faire en passant d'une
   0.12.x » — chaque point est-il vrai (compose, `.env`, mots de passe MariaDB, retour arrière,
   Synology) ? La phrase « installation à neuf : points 3 et 4 seulement » tient-elle (avec
   `docker-compose.yml` comme avec `docker-compose.prod.yml`) ? Chaque story `done` de l'Epic 15
   (`grep -E '^\s+15-' _bmad-output/implementation-artifacts/sprint-status.yaml`) visible de
   l'utilisateur a-t-elle son entrée ?
4. **README** (ligne v0.13.0 de la feuille de route) et **site** (`website/index.html`,
   `website/roadmap.html`) : aucune sur-promesse (fonction annoncée absente de `main`), aucune
   sous-déclaration grossière ; les cartes E24/E25 neuves disent-elles vrai (comparer au
   CHANGELOG 0.12.0/0.12.1) ? Les numéros d'issue cités sont-ils les bons ?
5. **PDF** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` porte-t-il
   les phrases neuves, la version 0.13.0, aucun « ?? » ? Idem pour `user-manual.pdf` et
   `marketing-brochure.pdf` (version et date).

## Lentille B — chiffres et versions partout

1. Grep des valeurs, non des formulations : `0\.12\.1`, `0\.13\.0`, `v0\.13`, `/main/`,
   `2026-10-10`, `2026-10-07` sur `README.md`, `CHANGELOG.md` (section 0.13.0), `CLAUDE.md`
   (§ Project Overview et point 4-bis), `website/`, `docs/manual/` (sources et PDF aplatis),
   `docs/manual/shared/kesh-style.sty`, `crates/*/Cargo.toml`, `Cargo.lock` (paquets `kesh-*`).
   Chaque occurrence restante de `0.12.1` est-elle légitime (historique) ? Chaque `0.13.0`
   juste ?
2. **§ Project Overview du `CLAUDE.md`** : recompte chaque chiffre par la commande que le
   paragraphe écrit lui-même (stories `done` hors `epic-N` et `*-retrospective`, ventilation des
   autres statuts, epics ouverts, migrations `ls crates/kesh-db/migrations/*.sql | wc -l`). Les
   totaux de tests backend et frontend se contrôlent contre les journaux
   `/home/gcorbaz/devel/kesh-gate-logs/release-0.13.0-backend.log` et
   `/home/gcorbaz/devel/kesh-gate-logs/release-0.13.0-frontend.log`.
3. `docs/migrations-idempotence-audit.md` : total et partition cohérents avec
   `ls crates/kesh-db/migrations/*.sql | wc -l` (lecture seule).
4. Les macros `\keshVersion`, `\keshReleaseDate`, `\keshTargetRelease` égalent-elles
   `0.13.0` / `2026-10-10` / `v0.13`, et la version Cargo des dix crates 0.13.0 ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/release-0.13.0-review-p1-<A|B>.md`
(selon ta lentille) :

- findings numérotés `A1…`/`B1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW),
  fichier:ligne, preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de
  présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding »
  sans cette liste ne compte pas ;
- rien d'autre n'est écrit nulle part.
