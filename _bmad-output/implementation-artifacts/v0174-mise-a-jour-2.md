# Story v0174-mise-a-jour-2 : la 0.17.4 — la sauvegarde avant mise à jour dans la liste du serveur (#847, lot B)

Status: ready-for-dev

*Sous-fiche de `v0174-mise-a-jour.md` (parente en `split`, C-v0174m-18), écrite le 2026-11-05 à la remédiation de la
validation P1 (Opus 5.5, autonomie). Même worktree, même branche, mêmes bases que la sous-fiche 1 ; se développe **après**
`v0174-mise-a-jour-1.md`, dont elle ne reprend que le dossier `pre-upgrade/` et l'expression `NOM_PRE_UPGRADE`
(§ *Nom* de la 1). Registre `v0174-choix-autonomes-v0174m.md` (C-v0174m-9). Numéros de ligne de `7bd4d5c0`. Aucune
migration.*

## Story

En tant qu'**administrateur**, je veux voir dans *Administration → Sauvegarde complète*, liste *Sauvegardes sur le
serveur*, les sauvegardes écrites avant une mise à jour, avec leur type, et pouvoir les télécharger, pour récupérer celle
qui me ramène à la version précédente sans accès au disque du serveur.

## Ce que le code dit (`7bd4d5c0`)

- `auto_backup.rs:264` `enum BackupKind { Auto, PreImport }`, `as_str` (`"auto"`, `"pre-import"`), `parse`, `nom_valide`
  (`NOM_AUTO` `:65`, `NOM_PRE_IMPORT` `:71`).
- `lister_les_sauvegardes` (`:367`) : `auto/` (illisible → journalisé, liste continue) et la racine (illisible → `Err`) ;
  fichiers réguliers seulement (`fichiers_reguliers`, `symlink_metadata`) ; tri par date puis nom.
- `routes/backup_schedule.rs` : `list_backups` (`kind: b.kind.as_str()`), `download_backup` (`BackupKind::parse`, puis
  `nom_valide`, puis recherche dans la liste, `ouvrir_la_sauvegarde` en `O_NOFOLLOW`, audit `admin.backup_downloaded`
  avec `kind`).
- `frontend/src/lib/features/admin-backup/backup-schedule.api.ts:69` : `kind: 'auto' | 'pre-import'` ;
  `AutoBackupSection.svelte:530-532` : ternaire à deux branches, clés `admin-backup-server-kind-auto` (« Automatique ») et
  `admin-backup-server-kind-pre-import` (« Avant import ») ; `messages.ftl` fr-CH `:1871-1872`.

## Conception

- `BackupKind::PreUpgrade` : `as_str` = `"pre-upgrade"`, dossier `<backup_dir>/pre-upgrade/` (constante
  `PRE_UPGRADE_SUBDIR`, partagée avec la sous-fiche 1), `nom_valide` = `NOM_PRE_UPGRADE`. `lister_les_sauvegardes` parcourt
  trois dossiers ; `pre-upgrade/` absent → rien ; illisible → journalisé, la liste continue (comme `auto/`).
- Le téléchargement ne change pas de code : il hérite du type neuf par `parse` et `nom_valide`.
- Frontend : `kind: 'auto' | 'pre-import' | 'pre-upgrade'` ; le ternaire devient une table de libellés
  (`Record<ServerBackupKind, () => string>`), une entrée par type, sans branche par défaut silencieuse (un type inconnu du
  frontend affiche le `kind` brut).
- i18n : `admin-backup-server-kind-pre-upgrade` — fr-CH « Avant mise à jour », de-CH « Vor der Aktualisierung », it-CH
  « Prima dell'aggiornamento », en-CH « Before update » ; repli de l'écran = texte fr-CH (règle des replis) ;
  `npm run lint-i18n-ownership`.
- Manuel : la phrase réservée par la sous-fiche 1 (§ *La sauvegarde avant mise à jour*, *Les sauvegardes du serveur* :
  trois types, « Avant mise à jour »), PDF régénéré.

## Acceptance Criteria

**AC-1** — `GET /api/v1/admin/backups` liste les fichiers réguliers de `pre-upgrade/` au nom exact, `kind = "pre-upgrade"` ;
un lien, un `.partial`, un nom hors expression n'y sont pas. (T11.)

**AC-2** — `GET /api/v1/admin/backups/pre-upgrade/{nom}` sert le fichier ; audit `admin.backup_downloaded` avec
`kind = "pre-upgrade"` ; nom hors expression ou absent → 404 ; refusé aux clés d'API (comme les autres types). (T11.)

**AC-3** — Un `pre-upgrade/` illisible n'empêche pas la liste des autres types. (T11-bis.)

**AC-4** — L'écran affiche « Avant mise à jour » (et ses trois traductions) pour ce type ; les deux autres libellés sont
inchangés. (T13, captures.)

## Tests et mutations

| Test | Assertion |
|---|---|
| **T11** (`crates/kesh-api/tests/admin_auto_backup_e2e.rs`) | un fichier valide dans `pre-upgrade/`, un lien, un `.partial`, un nom invalide : la liste porte le seul fichier valide, `kind = "pre-upgrade"` ; le téléchargement rend ses octets et écrit l'audit ; nom invalide → 404 ; la même requête par une clé d'API → refus (le code de `ensure_not_pat`, relevé au code, asserté) |
| **T11-bis** (unitaire, `auto_backup.rs`) | `pre-upgrade/` en `0o000` : la liste rend les sauvegardes de `auto/` et de la racine |
| **T13** (Vitest, `AutoBackupSection`) | une ligne `kind: 'pre-upgrade'` affiche « Avant mise à jour » ; `auto` et `pre-import` inchangés |

| Mutation | Tuée par |
|---|---|
| **M11** `PreUpgrade` absent de `lister_les_sauvegardes` | T11 |
| **M11-bis** `nom_valide` de `PreUpgrade` qui accepte tout | T11 (nom invalide → 404) |
| **M12** le libellé de `pre-upgrade` retombe sur « Avant import » | T13 |
| **M12-bis** `ensure_not_pat` retiré de `download_backup` | T11 (clé d'API) |

## Tâches

1. Étape 0 : la sous-fiche 1 est livrée (`NOM_PRE_UPGRADE`, `PRE_UPGRADE_SUBDIR`).
2. T11, T11-bis, T13 en rouge ; relever.
3. `BackupKind::PreUpgrade`, liste ; frontend ; i18n quatre locales.
4. Mutations M11, M11-bis, M12.
5. Manuel (phrase réservée) + PDF.
6. Captures (thème clair, base de démonstration jetable) : la liste avec une ligne de chaque type, fr-CH (et de-CH si
   `KESH_LANG=de` est simple à poser), dans `/home/gcorbaz/devel/kesh-gate-logs/v0174m/captures/` ; jointes au Dev Agent
   Record et au prompt de la revue de code.
7. Gate complet, Vitest, build, E2E complet au dernier commit de code (communs avec la 1 si les deux se livrent ensemble).

## Change Log

- 2026-11-05 — Sous-fiche créée à la remédiation de la validation P1 (C-v0174m-18 : découpage de la fiche, R1-10) ;
  contenu repris du lot B du premier jet, complété (AC-2 sur les clés d'API, T11-bis, M11-bis, table de libellés).
  Prochaine étape : validation P2 avec la parente et la sous-fiche 1.
- 2026-11-05 — Remédiation de la validation P2 : V2-17 (T11 asserte le refus par une clé d'API, M12-bis). La 1 ne
  supprime plus aucune sauvegarde (C-v0174m-25) : la liste les montre toutes, sans changement ici.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
