# Story 17.3f: Documentation export/import installation

Status: ready-for-dev

<!-- Dernière sous-story de l'épopée 17-3 (export/import installation, #112). Extraite de la spec umbrella (Partie F, AC23-25). Dépend de toutes (17-3a..e DONE). Doc-only. -->

## Story

As a **administrateur / DevOps d'une installation Kesh**,
I want **une documentation à jour de l'export/import d'installation via l'UI (manuel admin + matrice des méthodes de sauvegarde) et un CHANGELOG/README reflétant la livraison**,
so that **je sache quand et comment utiliser l'export/import UI vs Hyper Backup vs `mariadb-dump`, et que la roadmap publique soit exacte**.

## Contexte & cadrage

**Épopée 17-3 (#112) :** 17-3a..e **tous DONE**. **17-3f (cette story, doc)** clôture l'épopée. Après son merge → **PR umbrella 17-3** (`feedback_no_partial_prs` satisfait).

**Doc-only** : aucun changement de code applicatif, aucun test, aucune migration. La règle CLAUDE.md « Synchroniser TOUTES les docs » s'applique : manuel admin LaTeX FR (+ PDF régénéré), CHANGELOG, README. Le site web `website/` reste géré au moment de la release (hors scope de cette story ; à vérifier au moment du tag par Guy).

**Découverte ground-truth :** le manuel admin contient **déjà** `\section{Sauvegarde et restauration}` (`docs/manual/fr/admin-manual.tex:987`) avec : importance, script `mariadb-dump`, restauration, test périodique (OLICo Art. 10), et **Backup natif Synology DSM (Hyper Backup)** (`:1102`). 17-3f **ajoute une sous-section « Migration/restauration via l'UI Kesh »** + une **matrice des méthodes** dans cette section existante (pas de nouvelle section).

## Acceptance Criteria

1. **(AC23 — manuel admin)** Dans `docs/manual/fr/admin-manual.tex`, §`Sauvegarde et restauration` : nouvelle sous-section **« Export/import d'installation via l'interface Kesh »** documentant :
   - le format `.keshbackup` (conteneur des données d'installation **complète** — toutes sociétés + utilisateurs + données système — distinct de l'export per-company `/export` CSV) ;
   - le **workflow UI** : `Administration → Sauvegarde complète` (`/admin/backup`) pour exporter ; `Administration → Restaurer / Importer` (`/admin/restore`) pour importer (réservé **rôle Admin**, **non accessible via clé API/PAT**) ;
   - l'**avertissement destructeur** : l'import **remplace TOUTE l'installation** + **déconnexion** (reconnexion avec les identifiants de l'instance importée) ; un **backup automatique pré-import** est créé côté serveur (`KESH_ADMIN_BACKUP_DIR`, défaut `/tmp`) ;
   - le caractère **secret** du `.keshbackup` (contient hash de mots de passe + tokens → chiffrer/protéger en transit, recommander GPG/age) ;
   - les **garde-fous** : refus si version source incompatible (downgrade), si fichier corrompu (SHA), si schéma incompatible ; vérifier la version Kessh destination ≥ version min. du backup ;
   - les **variables d'env** : `KESH_ADMIN_IMPORT_MAX_MB` (défaut 512), `KESH_ADMIN_EXPORT_INMEM_MB` (défaut 50), `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`).

2. **(AC23 — matrice des méthodes)** Une **matrice/tableau comparatif** des 3 méthodes de sauvegarde selon le use case :
   | Méthode | Périmètre | Usage recommandé |
   |---|---|---|
   | Hyper Backup DSM (Story 10-4) | volume Docker MariaDB complet | sauvegarde **planifiée automatique** quotidienne (production NAS) |
   | `mariadb-dump` CLI (Story 10-4) | dump SQL d'une base | sauvegarde **ponctuelle technique** / accès SSH |
   | Export/import UI `.keshbackup` (17-3) | installation Kesh complète (app-level, portable) | **migration entre instances** / restauration self-service **sans SSH** |

3. **(AC23 — PDF régénéré)** Le PDF `docs/manual/fr/admin-manual.pdf` est **régénéré** (`latexmk -xelatex` dans `docs/manual/fr/`) et committé (convention projet : versionner les PDF, cf. PR #102).

4. **(AC24 — CHANGELOG)** `CHANGELOG.md` : entrée dans la section `[Non publié]` → `### Added` pour l'**export/import installation via l'UI admin** (#112) : description orientée utilisateur (fiduciaire/PME), mentionne le format `.keshbackup`, les pages `/admin/backup` + `/admin/restore`, le backup pré-import, l'usage migration/restauration self-service, et la distinction vs export per-company.

5. **(AC25 — README)** `README.md` :
   - **« Fonctionnalités »** : ajouter/refléter l'export-import d'installation comme **livré** (pas de marqueur `(à venir)`) ;
   - **« Feuille de route »** : le statut E17 reste cohérent (l'épopée 17-3 livrée ; ne pas marquer E17 entièrement done si d'autres stories E17 — ex. 17-4 recovery — restent).

6. **(Transverse — cohérence)** Aucune sur-promesse ni sous-claim. Les libellés UI cités dans la doc correspondent **exactement** aux libellés i18n FR réels (`nav-admin-backup` = « Sauvegarde complète », `nav-admin-restore` = « Restaurer / Importer »). Pas de mention du chemin de backup interne comme accessible à l'utilisateur final.

## Tasks / Subtasks

- [ ] **T-F1** Manuel admin : ajouter la sous-section « Export/import d'installation via l'interface Kesh » + la matrice des méthodes dans `\section{Sauvegarde et restauration}` (`admin-manual.tex`, après la sous-section Hyper Backup `:1102` ou à un emplacement logique). Cohérence des libellés UI + variables d'env. (AC: 1, 2, 6)
- [ ] **T-F2** Régénérer le PDF : `cd docs/manual/fr && latexmk -xelatex admin-manual.tex` (puis nettoyer les aux). Committer `admin-manual.pdf`. (AC: 3)
- [ ] **T-F3** `CHANGELOG.md` : entrée `### Added` (#112) dans `[Non publié]`, orientée utilisateur. (AC: 4)
- [ ] **T-F4** `README.md` : Fonctionnalités (export/import livré) + Feuille de route (cohérence E17). (AC: 5)

## Dev Notes

### Ground-truth (réutilisation / cohérence)

| Élément | Source | Note |
|---|---|---|
| Section sauvegarde existante | `docs/manual/fr/admin-manual.tex:987` `\section{Sauvegarde et restauration}` (mariadb-dump `:1007`, restauration `:1066`, OLICo `:1087`, **Hyper Backup DSM `:1102`**) | Y **insérer** la sous-section UI + matrice (ne pas créer de nouvelle section) |
| Libellés UI FR réels | `crates/kesh-i18n/locales/fr-CH/messages.ftl` : `nav-admin-backup = Sauvegarde complète`, `nav-admin-restore = Restaurer / Importer`, `admin-backup-page-title`, `admin-restore-page-title`/`confirm-body` | Citer **exactement** ces libellés (AC6) |
| Endpoints + RBAC | `GET /api/v1/admin/full-export` + `POST /api/v1/admin/full-import` (Admin strict, anti-PAT) | Doc : « réservé Admin, pas via clé API » |
| Variables d'env | `crates/kesh-api/src/config.rs` : `KESH_ADMIN_IMPORT_MAX_MB` (512), `KESH_ADMIN_EXPORT_INMEM_MB` (50), `KESH_ADMIN_BACKUP_DIR` (`/tmp`) ; aussi `.env.example` | Documenter les défauts + bornes |
| Format `.keshbackup` | spec umbrella §Format normatif (ZIP + NDJSON/table + manifest, 22 tables, `files/` vide v0.2) | Décrire au niveau utilisateur (pas le détail interne) |
| CHANGELOG structure | `CHANGELOG.md` `## [Non publié]` → `### Added` (déjà une entrée PAT #100) | Ajouter l'entrée #112 dans le même bloc |
| README roadmap | `README.md:179` (E17 v0.2 « 🚧 En cours », mentionne déjà « export/import installation ») + section « Fonctionnalités » `:~29` | Refléter la livraison sans sur-promesse |
| latexmk | `/usr/bin/latexmk` + `/usr/bin/xelatex` **disponibles** | PDF régénérable localement |

### Standards projet (CLAUDE.md)

- **Doc-only commit** : pas de Test Locally First code (pas de Rust/TS modifié) ; la CI sera no-op (cache hit). **MAIS** régénérer le PDF LaTeX (AC3) + vérifier que `admin-manual.tex` compile sans erreur.
- **Synchroniser TOUTES les docs** (checklist pré-push) : README (roadmap + fonctionnalités), CHANGELOG, manuel admin. Le site `website/` = au moment de la release (Guy, hors scope ici). Manuels DE/IT/EN = v0.2+ (FR seul pour l'instant).
- **Inclusion** : toute la doc dans le même commit/PR que la story (pas de PR doc séparée a posteriori).
- **Commit par étape BMAD**, pas de push auto. Branche active : `story/17-3-export-import-installation`.
- **Issue dette v0.3** : penser à tracer l'issue GitHub « Playwright double-instance E2E » (`v0.2-milestone`/`v0.3`) à la livraison de l'épopée (cf. 17-3e) — à faire au moment de la PR umbrella (peut être noté ici en rappel).

### Project Structure Notes

- **Modifié** : `docs/manual/fr/admin-manual.tex` + `docs/manual/fr/admin-manual.pdf` (régénéré) + `CHANGELOG.md` + `README.md`. Aucun fichier de code.
- Manuel **user** (`user-manual.tex`) : l'export/import est une opération **Admin/DevOps** → canoniquement dans le manuel admin. Mention user-manual = optionnelle (non requise par l'AC ; à considérer si une section « rôle Admin » existe côté user).

### References

- [Source: _bmad-output/implementation-artifacts/17-3-export-import-installation.md] — Partie F AC23-25, §Format normatif
- [Source: docs/manual/fr/admin-manual.tex:987-1110] — section sauvegarde existante (mariadb-dump, Hyper Backup)
- [Source: CHANGELOG.md] — section [Non publié]
- [Source: README.md:179] — roadmap E17
- [Source: crates/kesh-i18n/locales/fr-CH/messages.ftl] — libellés UI exacts
- [Source: crates/kesh-api/src/config.rs] — variables d'env
- [Source: CLAUDE.md] — Synchroniser TOUTES les docs, doc-only commit, Issue Tracking

## Dev Agent Record

### Agent Model Used

_(à compléter par dev-story)_

### Debug Log References

### Completion Notes List

### File List

### Change Log

| Date | Étape | Modèle | Résumé |
|------|-------|--------|--------|
| 2026-06-09 | create-story (sous-story) | Opus 4.8 | Story 17-3f (doc) extraite umbrella Partie F (AC23-25). Doc-only, clôt l'épopée 17-3. Scope : manuel admin LaTeX FR (sous-section UI export/import + matrice méthodes Hyper Backup/mariadb-dump/UI dans §Sauvegarde existante :987) + PDF régénéré (latexmk dispo) + CHANGELOG Added #112 + README fonctionnalités/roadmap. Cohérence libellés UI i18n FR + variables d'env. latexmk/xelatex confirmés dispos. T-F1..T-F4. Après merge → PR umbrella 17-3 + issue dette v0.3 Playwright. Prochaine : `bmad-dev-story 17-3f`. |
