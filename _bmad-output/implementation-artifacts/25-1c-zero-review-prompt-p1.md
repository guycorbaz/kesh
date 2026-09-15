# Prompt — passe 1 de `bmad-code-review`, Story 25-1c-zero

*Versionné le 2026-09-15. **Trois lentilles** en contexte frais, orthogonales à l'auteur de
l'implémentation (Opus 5) : Blind Hunter (Sonnet), Edge Case Hunter (Haiku 4.5), Acceptance
Auditor (Sonnet).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-zero-audit-company-id`.

## L'objet

**Le diff `main...HEAD` APLATI, hors documentation BMAD et PDF** — 15 fichiers, 954 lignes :

```sh
git diff main...HEAD -- . ':(exclude)_bmad-output' ':(exclude)*.pdf'
```

Une copie figée est dans le scratchpad de la session :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/review-25-1c-zero-code.diff`.

⚠️ **Lis le diff aplati, jamais la séquence de commits** : les numéros de ligne d'un diff
multi-commit désignent des états intermédiaires et font conclure à l'absence d'un patch qui est bien
là.

La spécification est `_bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md` — **18
critères, 10 tâches**, validée en quatre passes. Son § *Completion Notes* et son Change Log
déclarent ce qui a été fait, **six mutations tuées** et un gate à 2330/2330 : **ne les crois pas sur
parole, et cherche ce qu'ils n'ont pas vu.**

## Ce que la story fait

Elle ajoute à `audit_log` une colonne `company_id` (`BIGINT NULL`, sans FK) et son index, la remplit
pour les entrées existantes par une migration (`UPDATE … JOIN users`), et pour les entrées futures
par un **sous-SELECT** dans `repositories/audit_log.rs::insert_in_tx` — sans toucher aucun des 106
sites qui écrivent une entrée. La migration est **exemptée** du rejeu post-restore
(`post_restore.rs`, `ExemptionBasis::PerishableSince`). Les compteurs de migrations, le squash de
test, le registre des checksums et le manuel d'administration sont mis à jour.

⚠️ **Ne conteste PAS les décisions arbitrées** par le Project Lead : le sous-SELECT, la colonne
nullable sans FK, l'absence de `COALESCE` et de rejeu post-restore, l'exemption périssable, et le
fait que la story **mesure** sans les résoudre les deux sous-cas du restore (AC 8). Conteste leur
**mise en œuvre**.

## ⛔ Les axes, et tu déclareras lesquels tu as exercés

1. **Le SQL.** L'ordre des colonnes de l'`INSERT` correspond-il à l'ordre des `?` et des `.bind()` ?
   La migration est-elle valide sur MariaDB 10.11, ré-entrante, et son backfill exact ? Un
   `COALESCE`, un `LEFT JOIN`, un `NOT NULL`, une FK ou un `UPDATE _kesh_version` s'est-il glissé
   quelque part ?
2. **`COLUMNS` et `AuditLogEntry`** sont-ils en bijection ? Une autre requête du dépôt lit-elle
   `audit_log` vers `AuditLogEntry` sans passer par `COLUMNS` ?
3. **Les tests prouvent-ils ce qu'ils prétendent ?** Cherche l'assertion qui passerait **même si le
   code était faux** — c'est le défaut le plus coûteux du dépôt. Les identifiants sont-ils vraiment
   désalignés partout où la spec l'exige ? Le test de caractérisation (AC 8) distingue-t-il la copie
   locale de la copie d'archive ? Le rejeu de l'AC 14 (c) exécute-t-il bien le SQL embarqué ?
4. **Les garde-fous de migration (P5 à P8).** Compteurs recomptés **depuis la source** ; justification
   de l'exemption conforme aux tests d'`EXEMPT_MIGRATIONS` ; `migrations_upgrade_path.rs` sans
   résidu `33`/`67` ; squash régénéré et non édité ; checksum exact.
5. **Les commentaires et les messages.** Chaque doc-comment, commentaire ou message d'assertion
   ajouté ou modifié dit-il **vrai** ? Un énoncé porte-t-il un total qui se périmera ?
6. **Les manuels.** `docs/manual/fr/admin-manual.tex:1786` a été corrigé. Dit-il vrai maintenant,
   sans sur-promettre ? ⚠️ Contrôle le **PDF aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`)
   — **en LaTeX le souligné s'écrit `\_`**, et un grep sur `company_id` ne voit jamais
   `company\_id`. Cherche un **autre** site, dans les manuels, le `README.md` ou `website/`.
7. **Le périmètre.** Rien ne doit anticiper la 25-1c (méthode de lecture filtrée, route, type exposé
   au frontend). `backup.rs`, `export.rs`, `import.rs` et `NewAuditLogEntry` doivent être intacts.

## Ce que tu rends

- **Les findings** : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), `fichier:ligne`, **la commande
  ou l'extrait qui l'établit**, et le correctif. Pour un scénario, montre que **son état de départ est
  atteignable**.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » non
  adossé à cette liste ne compte pas comme passe. **Ne qualifie jamais de « vérifié » un point que tu
  n'as pas exécuté.** Déclarer un axe non exercé et en tirer un finding est contradictoire.

## Discipline imposée par ce dépôt

⛔ **Tout finding `CRITICAL` ou `HIGH` qui affirme l'ABSENCE d'un code attendu ou la PRÉSENCE d'un
anti-pattern doit être établi par un `grep -nF` (fixed-string) cité dans ton rapport.** Sans cette
preuve il sera écarté.

⚠️ **P8** : la migration n'est pas publiée, mais elle a déjà été appliquée aux bases de dev. Un
finding qui exige de **modifier le fichier de migration** doit le dire explicitement : il impose de
reconstruire ces bases.

## Interdits

⛔ **N'écris AUCUN fichier du dépôt et n'exécute AUCUNE commande qui mute** — nommément
`scripts/prepare-release.sh` (il bumpe les versions Cargo), `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`checkout`/`add`/`stash`/`reset`, `sqlx migrate`, et toute
écriture dans les bases `kesh` ou `kesh_e2e`. `cargo check`, `cargo clippy` et `cargo nextest run`
**en lecture** sont autorisés.
