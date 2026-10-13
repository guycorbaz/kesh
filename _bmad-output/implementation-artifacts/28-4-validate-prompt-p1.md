# Prompt — validation P1 de la spec, Story 28-4 (clore un compte bancaire, en lecture seule)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (une fiche, **deux** stories :
28-4a serveur, AC1-AC10 ; 28-4b écran, AC11-AC17). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md`
§ 28-4. Issue #642 (`gh api repos/guycorbaz/kesh/issues/642` et `…/642/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story 28-4* (C-28-4-1 à 13). Story
voisine non mergée qui ajoute une route d'écriture inscrite à l'inventaire (W30, R8) : `story/28-5`, fiche
`28-5-transaction-rejetee-atteignable.md` (`git show story/28-5:_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`
si la branche est visible ; sinon, le dire).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy P1-P8 ; § Pattern
batch ; § Règle de splitting préventif ; § Issue Tracking Rule) ; `docs/agents-orchestration.md`.

## Axes prioritaires (communs aux deux lentilles)

1. **Périmètre minimal** (axe propre à cet epic : trois fiches y ont dérivé en élargissant leur périmètre).
   Confronte la section *Périmètre minimal* de la fiche à #642, phrase par phrase : (a) chaque phrase de #642 et de la
   demande du Project Lead est-elle couverte par un AC ? (b) chaque AC, chaque tâche, chaque test **se rattache-t-il à
   une phrase de #642** — ou élargit-il le périmètre sans le dire ? Toute extension non justifiée par #642 est au moins
   **MEDIUM** ; toute exclusion de la table *Dehors* qui laisserait #642 non fermée (promesse de l'issue non tenue,
   sans CR) est au moins **HIGH**. (c) Le décompte des modules est-il juste (refais-le) ; le découpage C-28-4-1 et la
   dérogation de la 28-4a tiennent-ils contre la règle de splitting, et l'écart à la consigne de clé (mère `split` au
   lieu de `ready-for-dev`) est-il fondé ?
2. **Inventaire clos des écrivains** (Dev Notes § *Inventaire*, W1-W30, R1-R8) : refais les `grep` de la fiche **et
   élargis-les** (un détecteur part du symptôme, pas d'une liste) — les sept tables sous toutes graphies (alias SQL,
   `format!`, `QueryBuilder`, `concat!`, chaînes coupées par `\`), les colonnes `bank_account_id`,
   `settlement_bank_account_id`, `is_primary`, `journal_account_id`, `status`, `auto_match_rejected_at` ; les crates
   `kesh-reconciliation`, `kesh-payment`, `kesh-seed`, `kesh-import` ; `admin_backup`, `post_restore`,
   `test_endpoints.rs` ; les routes de `lib.rs` qui portent un `bankAccountId` ou atteignent une de ces tables par un
   autre identifiant (règlement, lot, transaction). Un écrivain absent de l'inventaire, ou mal classé
   (`SansCompteBancaire` / `Invariant` faux), est au moins **HIGH**. Vérifie en particulier W26 et W28 (« à
   vérifier »), et chaque classement **Invariant** contre l'énumération de **tous** les écrivains de l'état invariant
   (`status = 'pending'`, lots `generated`, `is_primary`) — un geste ultérieur qui défait l'invariant
   (réouverture puis reclôture, restauration, annulation, dévalidation) le réfute.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Comptes bancaires* (l. 1642 et
   suivantes), § *Supprimer un import* (l. 1724, phrase l. 1754-1757), § *Import bancaire* (l. 1696), § *Solde
   comptable, relevé et écart* (l. 238), les mentions du compte principal et de la QR-facture (l. 936, 947, 1047) ;
   `docs/manual/fr/admin-manual.tex` § *Passer à la 0.13.2* (l. 1972, modèle de la section 0.13.3). Ce que le manuel dit
   **aujourd'hui** de l'archivage est-il vrai au code ? Ce que l'AC14 prescrit d'écrire sera-t-il vrai au code que la
   fiche prescrit (dont l'angle mort des boutons laissés visibles) ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas
   est au moins MEDIUM.
4. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — migrations (80 → 81), compteurs de
   `docs/migrations-idempotence-audit.md` (80 / 10 / 70 / 0), `migrations_upgrade_path.rs` (`total` 80, soustracteur
   46, frontière 34, et la liste des lignes qui portent `\b(80|46)\b`), `migrations.sha384` (80 lignes), routes du
   registre (115 / 118), `sitesTotal` (2075), les 29 sites de l'extraction, les seize clés UI, AC (17), tests (T-a1-T-a35,
   T-b1-T-b14), mutations (M1-M33), les numéros de ligne cités.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #642 couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** et
  chaque mutation par un test qui la tue (la table des mutations nomme-t-elle le bon test ? une mutation est-elle
  équivalente sans le dire ?) ; chaque fait, chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les
  textes prescrits (i18n ×4, manuels, CHANGELOG, `api-external.md`) confrontés à ce que le code prescrit fera
  réellement ; les gardes existantes qui changeront de sens (`audit_route_registry.rs`, `audit_label_registry.rs`,
  `test_schema_guard.rs`, `migrations_upgrade_path.rs`, `i18n-keys.test.ts`, `lint-i18n-ownership`,
  `textes_coherents.rs`, les tests d'en-tête CSV, `bank-accounts-crud.spec.ts`) — la fiche les nomme-t-elle toutes ?
  Les appelants de `list_by_company` / `listBankAccounts` sont-ils tous traités, et chaque lecture conserve-t-elle le
  compte clos là où #642 le veut lisible ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : la règle de la garde (AC7 :
  verrou **exclusif**, **après** les verrous métier, avant le commit ; clôture en `FOR UPDATE` d'abord puis décomptes
  non verrouillants, **sans** sentinelle) est-elle **correcte sous InnoDB REPEATABLE READ** — naissance de la vue
  cohérente, lecture verrouillante contre vue, montée de verrou partagé → exclusif, ordre des verrous de **chaque**
  chemin gardé (W8, W9, W17-W19, W22-W25, W27, W29, W30) comparé à celui des chemins concurrents (règlement depuis
  l'écran, `accept_one_invoice` sous `with_account_lock`, `create_batch` et `pay_in_tx`, `cancel_in_tx` et
  `cancel_settlement_in_tx`, `PUT` avec sentinelle) ? Cherche le cycle. Les deux tests déterministes de l'AC7
  font-ils rougir M6 et M11, et `attendre_une_requete_en_cours` voit-il la bonne requête ? La contrainte `CHECK` de
  l'AC1 passe-t-elle sur MariaDB 10.11 (syntaxe, `ALGORITHM`, lignes existantes, squash régénéré,
  `test_schema_guard`), et casse-t-elle un chemin existant (bascule du principal dans `PUT`, `upsert_primary_in_tx`,
  restauration d'une sauvegarde) ? Le non-bump de `min_required` (C-28-4-11) tient-il contre P1-P3, P2-bis et l'import
  de sauvegarde ? Le refus par proposition dans `accept` / `reject` respecte-t-il le § *Pattern batch* (et le pré-vol de
  la 27-4) ? Le registre statique de l'AC8 est-il réalisable tel que décrit (extraction, fonction englobante, blocs
  `#[cfg(test)]`, deux sens du diff) et ses angles morts sont-ils les bons ? Les clés d'API (C-28-4-8) contre § 4 et
  § 7 de `docs/api-external.md`.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`,
`perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script
du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle
dans le rapport. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
