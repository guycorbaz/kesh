# Story 25.2-c : Le numéro d'écriture vient d'un compteur, non d'un `MAX + 1`

Status: ready-for-dev

**Issue : [#381].**

## Story

En tant que **comptable dont les livres peuvent être contrôlés**,
je veux qu'**un numéro d'écriture une fois attribué ne soit jamais réattribué à une autre écriture**,
afin qu'une **séquence comptable soit univoque dans le temps**, et pas seulement unique à un instant
donné.

## Le défaut, établi

`crates/kesh-db/src/repositories/journal_entries.rs:343-351` :

```sql
SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries
 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE
```

L'`UNIQUE (company_id, fiscal_year_id, entry_number)` de
`20260412000001_journal_entries.sql:9` garantit l'unicité **à un instant donné** — pas la
contiguïté, pas l'univocité dans le temps. #381 en tire les deux symptômes :

| Symptôme | Ce qu'il donne | Visibilité |
|---|---|---|
| supprimer l'écriture n° 42 au milieu | un **trou** définitif | **visible** — un contrôleur le voit et demande l'explication |
| supprimer la **dernière** | son numéro est **réattribué** à une écriture au contenu différent | ⛔ **muet** — rien ne le signale, jamais |

**C'est la réutilisation que cette story ferme.** Le trou, non : il subsiste, et il reste
*explicable* — il correspond à une facture dévalidée, que le journal d'audit nomme.

## Le remède existe déjà dans Kesh, et il n'est pas à inventer

`invoice_number_sequences` (`20260417000001_invoice_validation.sql:21-33`) est un **compteur
persistant** : colonne `next_number`, verrou `SELECT … FOR UPDATE`, création à la demande
(`INSERT IGNORE`), incrément vérifié à `rows_affected() == 1`
(`crates/kesh-db/src/repositories/invoice_number_sequences.rs:30-97`). Sa portée est
`UNIQUE (company_id, fiscal_year_id)` — **exactement celle d'`entry_number`**.

Un compteur ne redescend jamais : c'est précisément la propriété qui manque au `MAX + 1`.

⚠️ **Le `MAX + 1` des écritures est donc l'exception du dépôt, pas la norme.** Cette story
l'aligne ; elle n'invente pas de mécanisme.

## Acceptance Criteria

1. **Le numéro vient d'un compteur.** Une table `journal_entry_number_sequences`
   — `(company_id, fiscal_year_id)` UNIQUE, `next_number BIGINT NOT NULL DEFAULT 1`, `version`,
   `CHECK (next_number >= 1)`, FK `RESTRICT` vers `companies` et `fiscal_years` — calquée sur
   `invoice_number_sequences`, jusqu'aux noms de contraintes. `create_in_tx` la consulte au lieu du
   `MAX + 1`.

2. **Jamais de réattribution.** Test décisif : créer trois écritures (1, 2, 3), supprimer la
   **dernière**, en créer une quatrième → elle porte le **4**, pas le 3. C'est ce test qui dit si la
   story a atteint son but ; sans lui elle ne prouve rien.

3. **Le trou reste possible, et c'est assumé.** Supprimer l'écriture du milieu laisse un trou. La
   story ne le comble pas et ne renumérote rien : renuméroter réécrirait des écritures existantes,
   ce que le gel de l'Epic 24 interdit précisément.

4. **La migration amorce le compteur depuis l'existant.** Pour chaque `(company_id, fiscal_year_id)`
   ayant au moins une écriture : `next_number = MAX(entry_number) + 1`. Sans cet amorçage, une
   installation en service réattribuerait des numéros dès la première écriture suivante — l'inverse
   du but.

5. **⛔ L'amorçage est un backfill de données : P7 s'applique.** La migration écrit des lignes, donc
   elle doit être **triée** — inscrite au registre `POST_RESTORE_BACKFILLS` ou portée à
   `EXEMPT_MIGRATIONS` **avec justification écrite** (`crates/kesh-db/src/post_restore.rs`). Le test
   `every_data_backfill_migration_is_triaged` échoue sinon en nommant le fichier. ⚠️ Une exemption
   invoquant l'argument de fenêtre **doit** commencer par la chaîne `Hors fenêtre`, sans quoi elle
   échappe au contrôle symétrique.

6. **P5 — l'audit d'idempotence.** Une ligne dans `docs/migrations-idempotence-audit.md` avec son
   verdict, **et les compteurs recomptés depuis le tableau** : les deux sites du total
   (en-tête de section et ligne `Total`) et les trois compteurs de partition, dont la somme doit
   égaler le total. Recompter, jamais incrémenter de confiance.

7. **P6 — le couplage positionnel.** `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
   et **inspecter chaque site** : une migration de plus décale toute fenêtre indexée par position.
   Chaque site doit résoudre par version, ou porter son garde-fou fail-loud.

8. **P2 — pas de bump.** `CREATE TABLE` + `INSERT` est **non-breaking** : un binaire antérieur ignore
   la table. Aucun `kesh_version_min_required`, donc aucun bump Cargo.

9. **La concurrence est tenue.** Deux créations simultanées dans le même exercice n'obtiennent jamais
   le même numéro — le `FOR UPDATE` sur la ligne de séquence remplace le gap lock du `MAX + 1`. Test
   de concurrence, sur le modèle de ceux d'`invoice_number_sequences`.

10. **L'`UNIQUE` reste.** `uq` sur `(company_id, fiscal_year_id, entry_number)` n'est pas retiré : il
    devient le filet du compteur, exactement comme pour les factures.

11. **La documentation dit la vérité.** ⛔ **Le fichier de migration de 2026-04-12 qui promet
    « jamais de trou » NE DOIT PAS être modifié** — P8, `sqlx` en a le checksum, le binaire refuserait
    de démarrer. La rectification va ailleurs : c'était l'objet de **#368, déjà fermée**. Vérifier que
    ce qu'elle a écrit reste juste après cette story, et l'amender au bon endroit si besoin.

12. **#381 se ferme** — `closes #381` sur la PR, `refs` sur les commits.

## Tasks / Subtasks

- [ ] **T1 — La migration** (AC 1, 4, 5, 6, 7, 8)
  - [ ] `CREATE TABLE journal_entry_number_sequences`, calquée jusqu'aux noms de contraintes.
  - [ ] `INSERT … SELECT company_id, fiscal_year_id, MAX(entry_number) + 1 … GROUP BY` — amorçage.
  - [ ] P5 : ligne d'audit + **recompte** des deux totaux et des trois partitions.
  - [ ] P7 : triage, registre ou exemption justifiée.
  - [ ] P6 : grep des sites positionnels, inspection de chacun.
  - [ ] ⛔ **Gate complet, pas ciblé** — la règle l'impose dès qu'un patch touche
        `crates/kesh-db/migrations/`.

- [ ] **T2 — L'allocateur** (AC 1, 9)
  - [ ] `journal_entry_number_sequences::next_number_for(tx, company_id, fiscal_year_id)`, sur le
        modèle exact de l'existant, `rows_affected() == 1` compris.
  - [ ] `create_in_tx` l'appelle ; le `SELECT COALESCE(MAX(...))` disparaît.

- [ ] **T3 — Les tests qui prouvent** (AC 2, 3, 9, 10)
  - [ ] **Le test décisif de l'AC 2**, nommé pour ce qu'il vérifie.
  - [ ] Trou du milieu → toujours un trou, assumé.
  - [ ] Concurrence.
  - [ ] Amorçage : base pré-existante avec écritures → le premier numéro suivant est bien `MAX + 1`.

- [ ] **T4 — Documentation** (AC 11)
  - [ ] Vérifier ce que #368 a écrit ; `CHANGELOG.md` ; manuel si une promesse y figure — **contrôle
        du PDF à plat**.

- [ ] **T5 — Gates complets et PR** (AC 12)

## Dev Notes

### Ordre par rapport à la 25-2-b

**Cette story devrait précéder la 25-2-b**, ou voyager avec elle. La 25-2-b rend la suppression d'une
écriture *délibérée et fréquente* ; l'y laisser arriver avant le compteur, c'est multiplier les
occasions de réattribution muette pendant l'intervalle.

### Sites exacts

| Quoi | Où |
|---|---|
| le `MAX + 1` à remplacer | `crates/kesh-db/src/repositories/journal_entries.rs:343-351` |
| l'`UNIQUE` et le commentaire intouchable | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:9,20` |
| le motif à transposer | `crates/kesh-db/src/repositories/invoice_number_sequences.rs:30-97` |
| sa DDL | `crates/kesh-db/migrations/20260417000001_invoice_validation.sql:21-33` |

### References

- [Source: https://github.com/guycorbaz/kesh/issues/381] — audit du 2026-08-26, comptable § D3, fiscaliste § II.1.
- [Source: https://github.com/guycorbaz/kesh/issues/368] — la rectification du commentaire de migration, **fermée**.
- [Source: CLAUDE.md § Migration breaking policy — P2, P5, P6, P7, P8]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-16 | spec | Story créée, détachée de la 25-2-b. Fait établi le même jour : le compteur monotone **existe déjà** dans le dépôt (`invoice_number_sequences`), à la portée exacte d'`entry_number` — la story l'étend au lieu de l'inventer. |
