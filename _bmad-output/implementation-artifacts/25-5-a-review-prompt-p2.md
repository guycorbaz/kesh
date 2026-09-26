# Prompt — passe 2 de `bmad-code-review`, Story 25-5-a (revue a posteriori)

*Versionné le 2026-09-26. Trois lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku → Opus.
Diffs APLATIS (règle Haiku du `CLAUDE.md` : jamais de séquence de commits).*

Dépôt : **worktree** `/home/gcorbaz/devel/kesh-wt-255a`, branche `story/25-5-a-revue-code`, tête
`84f1b3fe`. ⚠️ Lis les fichiers **dans ce worktree**, pas dans `/home/gcorbaz/devel/kesh` (autre
branche). Fiche : `_bmad-output/implementation-artifacts/25-5-a-export-souverainete.md` — ses AC,
son Dev Agent Record, sa section **Review Findings** et l'entrée **review P1** de son Change Log.

Deux diffs, dans `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/` :
- `25-5-a.diff` (2019 lignes) — la story telle que mergée (`91a20d76`), PDF et `_bmad-output` exclus ;
- `25-5-a-remediation-p1.diff` (401 lignes) — la remédiation de la passe 1 (`main..HEAD`).

⛔ **Ce que la passe 1 a corrigé — premier suspect** (la sévérité se déplace vers le dernier
patch) : le test `export_global_zip_onze_tables_neuves_sortent_et_sont_scopees` peuple désormais
huit tables par SQL direct, **clés étrangères suspendues** sur une connexion dédiée, avec des
identifiants fictifs par société ; `creditor_line1`/`creditor_line2` ajoutées au CSV des pièces
importées ; `sent_to`/`note` des rappels passés par `fmt_opt_str` ; doc-comment de `global.rs`
réécrit ; manuel réécrit (boîte « Ce que cet export couvre ») ; CHANGELOG ; deux tests unitaires.
Reporté : #465 (lectures sans instantané commun) — **ne pas le re-signaler**.

## Lentille 1 — Blind Hunter (les deux diffs SEULS)

Revue adversariale générale des deux diffs. Priorité à la remédiation : le montage SQL du test
(`SET FOREIGN_KEY_CHECKS = 0` — est-il rétabli sur tous les chemins ? la connexion retourne-t-elle
au pool dans cet état si une assertion panique avant ?), les marqueurs (un marqueur de B peut-il
apparaître dans le CSV de A par une autre voie — faux positif / faux négatif ?), l'ordre en-tête /
valeurs des colonnes ajoutées, les textes du manuel et du CHANGELOG.

## Lentille 2 — Edge Case Hunter (diffs + worktree)

1. Le test peuplé : chaque table neuve a-t-elle bien une ligne **par société** qui passe par la
   **vraie** lecture d'export ? Une jointure fausse (p. ex. `payment_batch_items` scopé par
   `supplier_invoices` au lieu de `payment_batches`) serait-elle détectée ? `audit_log` : la ligne
   d'audit `exports.global` écrite par l'export lui-même peut-elle masquer ou fausser l'assertion ?
2. `csv_sanitize` : reste-t-il, dans les **trente** sérialiseurs, une cellule de texte libre qui ne
   passe ni par `txt` ni par `fmt_opt_str` ? (`grep` exhaustif, pas un échantillon.)
3. Colonnes : pour **chacune** des trente tables, colonnes exportées contre le schéma
   (`crates/kesh-db/test-schema/0001_schema_squash.sql`). La passe 1 n'a comparé que les onze neuves.
4. Les sérialiseurs modifiés tiennent-ils le nombre de colonnes (le `csv::Writer` refuse un
   enregistrement de longueur différente de l'en-tête) ?

## Lentille 3 — Acceptance Auditor (diffs + fiche + documentation)

Chaque finding de la section **Review Findings** coché `[x]` : est-il **réellement** corrigé dans
le worktree (`grep -nF` à l'appui) ? L'entrée review P1 du Change Log dit-elle vrai — recompte :
« douze tables assertées », « 27 colonnes sur 27 », « 34/34 », deux tests unitaires neufs, les deux
mutations (relis le code pour juger qu'elles auraient rougi) ? Manuel `docs/manual/fr/user-manual.tex`
**et PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers le
scratchpad) : reste-t-il une phrase qui contredit l'export à trente tables ? CHANGELOG `[0.12.1]` :
chaque phrase vraie ? AC 11 : ce qui est déclaré exécuté l'est-il, et ce qui ne l'est pas est-il dit ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu,
  commande et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente
  ou présente : la commande `grep -nF` exécutée **dans le worktree** et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ni du worktree, n'exécute aucune commande qui écrit dans le dépôt
ou dans une base persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans
`docs/manual/`, tout `git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`/
`worktree`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`, `npx playwright`. Lecture,
`grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et `cargo check` sont autorisés.
