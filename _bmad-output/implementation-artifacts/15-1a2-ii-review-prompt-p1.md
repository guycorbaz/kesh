# Prompt — revue de code P1, Story 15-1a2-ii (le lettrage des pièces fournisseurs, et le rattrapage)

*Versionné le 2026-10-10. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1a2-ii`, branche `story/15-1a2-ii-fournisseurs-et-rattrapage`.
**Le diff à revoir** : `git diff 240deef8 HEAD~1` (le commit de développement, rebasé sur `main` qui porte la 15-1a2-i
fusionnée, `240deef8` ; `HEAD` ne porte que ce prompt). Fiche : `_bmad-output/implementation-artifacts/15-1a2-ii-fournisseurs-et-rattrapage.md`
— Reçu de la 15-1a2-i, P2, P3, P4, P6, AC6–AC12, AC16, Tasks, Tests prévus, Dev Notes, et le **Dev Agent Record**, qui
déclare huit choix (C-15-1a2-ii-1 à 8, registre `epic-15-choix-autonomes.md`), dix-huit mutations et des décomptes.
Fiches amont : `15-1a2-i-lettrage-des-pieces-clients.md` (la synchronisation, P3), `15-1a2-0-lettrage-fige-avec-la-periode.md`
(rang 2 bis, `open_period_rule`). Règles : `CLAUDE.md` du dépôt — **§ « Migration breaking policy » P1 à P8**, § « Pattern
batch ». Issue : #518 (la story ne la ferme pas).

**Contexte** : la story (1) lettre les factures **fournisseurs** — le paiement (direct, ou `confirm_batch`) synchronise,
les deux annulations dissolvent avant la contre-passation — sur l'algorithme de la 15-1a2-i **factorisé** (la découverte
change, le reste est commun) ; (2) ajoute **deux migrations de données** : M1 `20261010000001_lettering_documents_backfill.sql`
(au registre de rejeu, classe A, rejouée à chaque import) et M2 `20261010000002_lettering_reversal_pairs_backfill.sql`
(exemptée) — la même règle que la synchronisation, recopiée en SQL, tenue par le test d'accord
`crates/kesh-db/tests/lettering_documents_backfill.rs`. Frontend non touché.

## Lentilles

- **B — Blind Hunter : bugs, SQL, concurrence**, sans présupposer que la fiche a raison. `letterings.rs` : la
  factorisation (`PieceDocument`, `sync_document_in_tx`, `dissolve_document_in_tx`) — le comportement **client** est-il
  strictement inchangé ? `discover_supplier_invoice_document` (statuts `open`/`paid`/`cancelled`, ancre = première ligne
  au crédit de l'achat, verrous `FOR UPDATE`, filtre de société). `supplier_invoices.rs` : place des trois appels,
  exercice tenu (couvre-t-il une ligne du groupe dans **tous** les cas atteignables ? sinon `Invariant` → le paiement ou
  le lot entier échoue), ordre des verrous. **Le SQL de M1 et M2, ligne à ligne** : équivalence exacte avec la
  synchronisation (ancre, `C(I)` / `C(S)`, compte lettrable, règle des périodes — les trois branches —, somme nulle,
  clé = MIN, gardes `lettering_key IS NULL`), appariement **par position** de la contre-passation (rang `ORDER BY
  line_order`, `reverses_entry_id`), partage M1 étape 3 / M2 (`EXISTS` / `NOT EXISTS … purchase_journal_entry_id`), une
  ligne peut-elle être candidate deux fois dans un même `UPDATE` (deux groupes, deux paires) ? Société mélangée ?
  Performance (relevé `kesh-gate-logs/15-1a2-ii-t0-explain.txt`, C-15-1a2-ii-5). Énumère les `Err` que la
  synchronisation fournisseur peut rendre sur un chemin de geste et dis si un état atteignable la produit.
- **E — Edge Case Hunter : bords et états.** Facture fournisseur sans numéro ; payée par compte interne lettrable /
  non lettrable ; payée par lot puis paiement annulé ; payée, annulée, règlement détaché puis lettré à la main puis
  contre-passé (refus ? groupe manuel ?) ; dette `B` devenue non lettrable puis annulation ; exercice de l'achat clos
  et paiement en N+1 puis annulation de la facture (exercice tenu = celui de l'achat, **clos** : refusé avant ? sinon ?) ;
  achat et paiement dans deux exercices ; contre-passation d'une écriture dont une ligne est lettrée `manual` (R6
  saute la ligne) ; base restaurée (AC16) ; reversals d'écritures à lignes répétées ; données héritées anormales
  (paire de longueurs différentes, `reverses_entry_id` vers une autre société, facture `paid` sans écriture). **Cherche
  le geste ultérieur qui défait l'invariant** d'AC9 part ii, et l'état où M1 et la synchronisation divergent hors
  des deux régimes d'AC6.
- **A — Acceptance Auditor : conformité, outillage des migrations, documentation, manuel.** Chaque AC (AC6–AC12,
  AC16) satisfait ; chaque test de « Tests prévus » existe sous son nom et **prouve** ce qu'il nomme (aurait-il échoué
  sans le patch ? assertion de montage devant chaque assertion négative ?) ; AC6 classe-t-il **par construction**
  (jamais par `lines_in_open_period`) ? **Gardes de migration P3/P5/P6/P7/P8** : P5 — `ls crates/kesh-db/migrations/*.sql
  | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`, les trois compteurs **recomptés depuis le
  tableau** ; P6 — `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site ; P7 — registre,
  exemption, compteur `17`, justifications ; P8 — `git diff 240deef8 HEAD -- crates/kesh-db/migrations/20261009000001_journal_entry_lines_lettering.sql`
  vide, `migrations.sha384` (`sha384sum` des deux fichiers) ; squash ; `ALLOWED_REAL_MIGRATOR_FILES`. Décomptes du Dev
  Agent Record **recomptés** (19 tests neufs : `git diff 240deef8 HEAD~1 -- crates | grep -cE
  '^\+\s*#\[(sqlx::test|tokio::test|test)'` + le fichier neuf). **Documentation par la valeur** : `CHANGELOG.md`,
  `docs/api-external.md`, `README.md`, `docs/manual/fr/user-manual.tex`, `admin-manual.tex` — le reçu A2-1 (grep
  `déjà soldée avant la mise à jour|au geste qui solde|pas lettrée par cette version` : plus aucune phrase fausse ?).
  ⛔ **Le manuel se contrôle sur le PDF aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s
  ' '` (et `admin-manual.pdf`) vers `/home/gcorbaz/devel/kesh-gate-logs/`. **Le manuel dit-il vrai du code livré ?**
  (règlement détaché lettrable à la main *par l'API* ; rattrapage sans audit ; période close ; rejeu à l'import et sa
  nuance ; paires libres non rejouées).

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-ii-review-p1-<B|E|A>.md` (ta lettre). Findings avec
sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction
proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF
au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande),
`npm`, `npx`, `docker`, `gh` en écriture, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `sha384sum`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
