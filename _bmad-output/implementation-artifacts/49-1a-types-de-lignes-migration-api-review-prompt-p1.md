# Prompt — revue de code P1 de la story `49-1a` (#1242), trois lentilles, première passe complète

Tu relis la story **49-1a « Types de lignes — migration et API »** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit),
release **0.20.0**. Elle porte **la seule migration de la release** (`crates/kesh-db/migrations/20261118000002_invoice_line_types.sql`,
la 96ᵉ), qui s'appliquera à **la base de production de Guy**, où de **vraies factures ont déjà été émises** et remises sur papier,
au **PDF figé**. Elle ajoute `line_type` (`priced`, `comment`, `heading`) à `invoice_lines` et `credit_note_lines`, avec trois
contraintes par table et le retrait de `chk_*_quantity_positive` ; le modèle (`LineType`), les dépôts (création, modification, no-op,
audit, validation, avoir, rapport TVA, part de TVA d'un solde), l'API **gardée** (`lineType` optionnel, `comment`/`heading` refusés
`400`, montants `Option` jugés par type, changement de contrat `422` → `400`), les exports CSV, la sortie MCP `kesh_facture`, la
documentation d'API et le squelette de la section « Passer à la 0.20.0 » du manuel d'administration. **Aucun écran, aucun rendu PDF
ne change** (49-1b et 49-1c).

Worktree `/home/gcorbaz/devel/kesh-49-1`, branche `story/49-1-types-de-lignes`. Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-49-1 diff fe01a0179 aae45788d` — `fe01a0179` = fiche validée (`ready-for-dev`), `aae45788d` =
dernier commit de code ; les commits suivants ne portent que la fiche, le registre et ce prompt (vérifie-le par
`git -C /home/gcorbaz/devel/kesh-49-1 diff --stat aae45788d HEAD`). Langue du rapport :
français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/49-1a-types-de-lignes-migration-api.md` en entier (« Ce qui se lit au code », l'inventaire
des lecteurs, D1 à D14, AC1 à AC15, T1 à T13, **Change Log**, **Dev Agent Record**, Note de version) ; le registre
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491.md` (C-e491-1 à 8) ; le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e491/validation-49-1a-p6-ciblee.md`. Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e491/` : `t0/` (sonde de la
migration : montée, rejeu, retour, remontée), `check-*.log`, `cible-1.log` à `cible-4.log`, `mesure-mcp.log`, `mut/` (le banc
`muter.py`, `specs.py`, `lancer.sh`, `resume.log`, une spec `.json` et un journal `.log` par mutation), `test-fast-review.log` (rouge
du registre des checksums) et `test-fast-review-2.log` (vert), `gate-ci-review.log`, `p6-grep-apres.log`. Le frontend n'est pas
touché et ses gates n'ont pas tourné (vérifie l'absence de fichier sous `frontend/` au diff). **Le registre
`crates/kesh-db/migrations.sha384`** (garde P8) gagne la ligne de la migration : vérifie l'empreinte (lis la ligne, compare-la au
message de `test-fast-review.log`).
**Captures** : sans objet (aucun contrôle visible ne change — vérifie-le : `git diff --stat` ne doit nommer aucun fichier sous
`frontend/`).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/regen-test-schema.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (ils se **lisent**,
  `mut/muter.py` compris) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **La migration (axe premier).** `20261118000002_invoice_line_types.sql` contre `CLAUDE.md` § *Migration breaking policy*, **P1 à
   P8 à la lettre** : non breaking (aucune opération de P3, `kesh_version_min_required` intact, aucun `Cargo.toml` touché) ; le SQL
   de retour de l'en-tête (deux tables, contraintes retirées avant `DROP COLUMN`, `DELETE FROM _sqlx_migrations` à la bonne
   version) **défait-il exactement** la migration ? Le rejeu après un échec entre les deux `ALTER` (D13) est-il sans effet ? Un
   commentaire `/* */` ? Une écriture de données hors `--` (P7) ? P5 : la ligne d'audit et les **deux** totaux (96) et la partition
   (`yes` 12 + `tracked-by-sqlx` 84 = 96) — recompte-les depuis le tableau. P6 : `migrations_upgrade_path.rs` (`total == 96`,
   `total - 62`, frontière 34), chaque site sorti de `git grep -nE "\b95\b|\b61\b|\b60\b"` classé ; le squash
   `crates/kesh-db/test-schema/0001_schema_squash.sql` régénéré (en-tête « 96 migrations ») et égal au schéma migré ; P8. Les
   contraintes : une ligne `priced` garde **exactement** la règle d'avant ? Une ligne neutre peut-elle porter une description vide ?
   Une valeur non neutre peut-elle passer (arrondi, `-0`, `0.0000` contre `0`) ? La sonde T0 (`kesh-gate-logs/e491/t0/`) le
   montre-t-elle vraiment, sur la version de MariaDB du conteneur ?
2. **Les factures réelles ne changent pas.** T1 (`migrations_upgrade_path.rs::line_types_migration_keeps_real_invoices`) : peuplé en
   **SQL brut de forme 0.19.0**, jamais par le dépôt ? Empreinte de **toutes** les colonnes d'avant de six tables (`invoices`, dont
   `pdf_*`, `invoice_lines`, `credit_notes`, `credit_note_lines`, `journal_entry_lines`, `invoice_settlements`) ; témoin chiffré
   (colonnes, lignes) tiré de l'empreinte elle-même ; facture au PDF figé **et** non figée ; payée, impayée arrondie, annulée par un
   avoir à huit lignes, règlement groupé ; TTC et reste dû ; contraintes d'avant (`CHECK_CONSTRAINTS`, `uq_*_position` dans
   `TABLE_CONSTRAINTS`). Les cinq mutations et celles du témoin (`mut/`) tuent-elles chacune par l'assertion qu'elles visent ? Un
   test vert par construction ? Puis l'avoir émis **après** la montée sur une facture d'avant (AC9 f).
3. **Les lecteurs (inventaire clos de la fiche).** Pour chaque site du tableau « Les lecteurs et écrivains », la résolution
   annoncée est-elle au code : `insert_lines` / `compute_total` / `ensure_unpriced_lines_are_neutral` (D10, appelée où et quand ?),
   `is_no_op_change`, snapshots d'audit (facture et avoir), `validate_invoice` (le piège de l'`UPDATE … AND line_type = 'priced'`,
   `priced_line_without_account`, la boucle en mémoire, `explicit_line_account_sites`), `create_credit_note` (`triplets`, comptes à
   verrouiller, sites « compte par défaut », copie de `line_type`), `vat_report.rs`, `invoice_settlements_write.rs`. **Inventorie
   les sites NON RÉSOLUS** (`CLAUDE.md` § *Inventorier les sites NON RÉSOLUS*) : `git grep -nE "invoice_lines|credit_note_lines" --
   'crates/*/src/*'` et `git grep -nE "\.lines\b.*(revenue_account_id|vat_rate|line_total)" -- 'crates/*/src/*'` ; un lecteur qui
   regroupe, compte, écrit ou rend ligne par ligne et que la fiche ne nomme pas est un finding. Rendu : `kesh-qrbill`,
   `issued_invoice_pdf.rs` et le code non-test d'`invoice_pdf_service.rs` sont-ils **inchangés** (AC14) ?
4. **L'API gardée.** `CreateInvoiceLineRequest` (`Option` + `#[serde(default)]`), `validate_line` / `validate_unpriced_line` :
   messages des lignes chiffrées inchangés ? Un corps 0.19.0 rend-il la même réponse au seul `lineType` près ? `comment`/`heading`
   refusés au `POST` **et** au `PUT`, sans écriture, avant toute lecture des taux (`verify_vat_rates_against_db`) ? `null` et
   absence traités pareil ? Un `lineType` inconnu rend-il `422` ? `LineType::open_to_api` est-elle **la seule** porte (une autre
   route — services, MCP, import, `kesh-seed` — peut-elle créer une ligne non chiffrée sans elle) ? Les réponses (`InvoiceLineResponse`,
   `CreditNoteLineResponse`) : le seul ajout est-il `lineType` ?
5. **Exports, MCP, sauvegarde.** CSV : `line_type` en **dernière** colonne des deux fichiers, arité de chaque ligne (T10) ; MCP :
   `type_ligne` dans `LigneFacture` (macro `sortie!` sans attribut de champ, `champ_enum!` de `sortie.rs`), clé `typeLigne` sur chaque
   ligne (T11) et la mesure de taille (seuil écrit au Change Log ; lis C-e491-6) ; import d'une sauvegarde 0.19.0 et aller-retour
   d'une sauvegarde 0.20.0 (T3) ; `backup.rs` sans changement de code hors littéral de test ; `post_restore.rs` sans nouvelle entrée.
6. **Les tests et les mutations.** Chaque AC (AC1 à AC15) a-t-il un test qui peut rougir ? Les journaux `mut/*.log` disent-ils
   ROUGE pour chaque mutation annoncée, et les mutations déclarées équivalentes le sont-elles vraiment ? Les littéraux complétés
   (`line_type: …::Priced`) n'ont-ils rien changé d'autre ? Les tests existants modifiés sont-ils seulement ceux que la fiche permet
   (AC5) ou que le Change Log nomme avec leur motif ?
7. **Le manuel d'administration** (`docs/manual/fr/admin-manual.tex`, § « Passer à la 0.20.0 », `\label{sec:maj-0-20-0}`) contre le
   code : le paragraphe « Pour les intégrations » dit-il vrai (dernière colonne, clé MCP sur chaque ligne, `400`/`422`) ? Le reste
   de la section est **assigné à la 49-1d** (sauvegarde d'avant la mise à jour, compatibilité, retour, échec au milieu) : ne pas le
   compter comme défaut de la 49-1a, mais signaler une contradiction. Le **PDF n'est pas régénéré dans la branche** (règle des
   fichiers communs) : forme du contrôle, `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c
   'Passer à la 0.20.0'` — il rend `0` dans la branche, c'est attendu ; le dire. `docs/api-external.md` (sous-section
   « Les lignes d'une facture client ») contre le code. Aucun manuel utilisateur ne doit avoir changé (AC13 d).
8. **La fiche, le registre, la Note de version** : le Change Log cite-t-il des lignes réellement lues ? Le Dev Agent Record dit-il
   vrai (décomptes recomptés, journaux et commits cités, ce qui n'a pas tourné écrit comme tel) ? La Note de version dit-elle vrai ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, de SQL, d'état ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et séquences (axes 1 à 4 surtout) : rejeu partiel de la migration, retour,
  sauvegarde 0.19.0 / 0.20.0, ligne neutre aux valeurs limites, facture de commentaires seuls, compte par défaut archivé, avoir
  d'une facture à commentaires, concurrence `update`/`validate`.
- **A — Acceptance Auditor** : chaque AC et chaque décision (D1 à D14) contre le code, les tests et les journaux (axes 2, 5 à 8).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e491/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md`. Pour
chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario,
correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas).
Ce qui n'a pas tourné reste écrit : la revue n'exécute rien ; `test-fast --ci`, `gate-ci` et le frontend complet ont tourné au
passage en revue (journaux cités au Dev Agent Record) ; **l'E2E Playwright complet et le démarrage sur base persistante (P8) n'ont
pas tourné** — ils tournent à la clôture.
