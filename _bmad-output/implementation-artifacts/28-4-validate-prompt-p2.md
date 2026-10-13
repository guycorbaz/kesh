# Prompt — validation P2 de la spec, Story 28-4 (clore un compte bancaire, en lecture seule)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; **P2 : Opus**
pour les deux lentilles. La remédiation de la P1 a été écrite par Opus : c'est **sa** production que tu relis
d'abord.*

Dépôt `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (une fiche, **deux** stories :
28-4a serveur, AC1-AC10 et **AC6-bis** ; 28-4b écran, AC11-AC17). Epic :
`_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § 28-4. Issue #642 (`gh api repos/guycorbaz/kesh/issues/642`
et `…/642/comments`). Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story
28-4* (C-28-4-1 à C-28-4-20 ; C-28-4-1 complété et C-28-4-12 **réécrit** en P1 ; C-28-4-14 à 20 neufs). Story voisine
non mergée (W30, R8) : `story/28-5`, fiche `28-5-transaction-rejetee-atteignable.md`
(`git show story/28-5:_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md` si la branche est
visible ; sinon, le dire).

**La remédiation à relire** : `git show 32ee4e36` (le commit de la P1, fiche + registre + `sprint-status.yaml`), sur
`git diff 1af07621 32ee4e36`. Les rapports de la P1 (hors dépôt, à lire pour savoir ce qui a été demandé) :
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p1-R.md` et `…-p1-F.md`.

Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy P1-P8 ; § Pattern
batch ; § Règle de splitting préventif ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ; la mémoire
*invariant dans le temps* (tout reclassement fondé sur un invariant : énumérer les écrivains de l'état et dire pour
chacun pourquoi il ne le casse pas).

## Axe premier — les régressions de la remédiation P1 (les deux lentilles)

*La sévérité se déplace vers ce qu'on vient d'écrire* : sur les Epics 22 et 23, la plupart des défauts des passes
tardives venaient du patch précédent. Avant tout le reste, relis **chaque** changement du commit `32ee4e36` et
cherche ce qu'il a cassé ou laissé à moitié :

- **AC7 et § *Concurrence*** (règle « pièces → `bank_accounts` → exercice / écritures / compteur », table *Position
  par chemin*, démonstration « import / suppression d'import sans cycle », cycle résiduel W9 / W24 « rejoué des deux
  côtés », inversion `companies` par clé étrangère) : chaque ligne de la table est-elle **vraie au code**
  (`sed -n` des lignes citées) ? Chaque « rejoué » est-il vrai (`grep -n "retry_on_deadlock\|retry_app_on_deadlock\|retry_with"
  crates/kesh-api/src/routes/`) ? La démonstration W22 / W24 tient-elle, index unique `uq_bank_imports_company_hash`
  et lecture `find_by_company_and_hash` compris ?
- **AC6-bis** (archivé aux gestes qui créent) : les erreurs « existantes » citées le sont-elles (W8 l. 141, W27 l. 763,
  `create_batch` l. 95-98, appariement de la route d'import l. 1312) ? Le partage `lock_open_for_write` /
  `lock_open_unarchived_for_write` est-il cohérent partout où la fiche nomme une garde (table de l'AC6, inventaire,
  T5/T6/T6-bis, mutations) ? L'interdiction faite aux annulations est-elle tenue **partout** — aucune ligne de la fiche
  ne fait lire `archived` à W9, W24, W25, W28, W29, W30 ou à l'archivage ? Le `### Corrigé` du CHANGELOG et la note
  d'`api-external.md` dans la **28-4a** contredisent-ils C-28-4-1 (CHANGELOG à la 28-4b) d'une façon non dite ?
- **AC5 / F6** (archivage : verrou de ligne avant le décompte) : le nouvel ordre des refus du `DELETE` casse-t-il un
  test existant de l'archivage (`grep -n "BANK_ACCOUNT_HAS_TRANSACTIONS\|CANNOT_ARCHIVE_PRIMARY\|OPTIMISTIC" crates/kesh-api/tests/bank_accounts_e2e.rs`) ?
  `archive_for_company` « relit sous le verrou tenu » : est-ce vrai de son `SELECT … FOR UPDATE` (filtre `archived =
  FALSE`) ? Le « L5 se ferme pour les imports » est-il vrai, compte tenu des deux ordres de la course ?
- **R5 retiré, M19 sans objet, T-a30 réécrit** : reste-t-il, dans la fiche ou le registre, une phrase qui suppose
  une garde dans `invoice_settlements::create_in_tx` ou un appariement dans `accept_one_invoice` ? `accept_one_invoice`
  sans verrou : le raisonnement I1 tient-il contre **tous** les écrivains de `status = 'pending'` et contre une
  clôture **concurrente** d'un lot d'acceptation (le décompte lit la version commitée ; la transaction tenue par le lot
  est-elle encore `pending` pour lui) ?
- **Tests et mutations neufs** (T-a36 à T-a44 ; M10-bis, M14-bis, M20-bis, M34 à M50) : chaque mutation est-elle tuée
  par le test nommé, ou équivalente **avec** une raison vraie ? Les tests déterministes (T-a39, T-a40, T-a41, T-a44)
  sont-ils réalisables avec `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:561-592`) et leurs
  motifs désignent-ils bien la requête **bloquée** ? T-a41 fait-il rougir une garde mal placée **à coup sûr** (choix
  de la victime par InnoDB) ? Les lignes de T-a27 dites « construites par SQL direct » sont-elles les bonnes, et les
  autres se construisent-elles vraiment par l'API ?
- **Recomptes de la remédiation** : 18 AC ; T-a1-T-a44 ; 53 numéros de mutation (M1-M50 + 3 `-bis`), M19 sans objet,
  quatre équivalentes ; 16 sites « Gardée » hors W30 ; « seize » lignes pour T-a27 ; 20 choix ; décomptes de
  `audit_route_registry.rs` (l. 645, 648, 667) — **recompte chacun depuis la source**.

## Axe obligatoire — l'ordre des verrous de CHAQUE chemin gardé

Pour **chaque** site gardé (W8, W9, W13-W15, W17-W20, W22-W25, W27, W28, W29, W30, l'archivage, la clôture et la
réouverture) : écris, **depuis le code au sol** (`5aad8b35`), la suite de ses verrous **dans l'ordre**, garde
comprise à la position que la fiche prescrit — y compris les verrous **implicites** : verrou partagé sur le parent
posé par une clé étrangère à l'`INSERT` d'un enfant (`companies`, `bank_accounts`, `invoices`…), verrous de trou,
verrous pris par les fonctions appelées (`journal_entries::create_in_tx`, `reverse_owned_in_tx`,
`letterings::dissolve_*`, `refuse_if_claim_account`, `company_invoice_settings::*`, `purchase_payable_line`,
`sale_receivable_account`, l'audit). Confronte chaque suite à celle des chemins concurrents (règlement depuis
l'écran, `accept_one_*` sous `with_account_lock`, `create_batch` et `confirm_batch`, annulations, `PUT` / `POST` /
`DELETE /bank-accounts` avec sentinelle `companies`, `claims_for_target`, clôture). **Cherche le cycle** et, pour
chacun, dis s'il est **introduit** par la garde ou **préexistant**, et si les deux routes du cycle sont rejouées sur
`1213`. Rends le résultat sous forme de table (chemin / suite des verrous / partenaires de cycle / introduit ou
préexistant / rejoué). Un cycle introduit par la garde entre deux routes dont l'une n'est pas rejouée et que la fiche
ne nomme pas est au moins **HIGH**.

## Axes communs (repris de la P1)

1. **Périmètre minimal** : la section *Périmètre minimal* contre #642, phrase par phrase ; l'extension n° 6 (archivé)
   est-elle bornée comme elle le dit, et ne s'est-elle pas élargie ailleurs dans la fiche ? Toute extension non
   justifiée par #642 ou par la décision écrite de l'orchestrateur est au moins **MEDIUM** ; toute exclusion qui
   laisserait #642 non fermée est au moins **HIGH**. Le décompte des modules et la dérogation (C-28-4-1, R14).
2. **Inventaire clos des écrivains** (W1-W30, R1-R8, R5 retiré) : refais les `grep` **et élargis-les** (un détecteur
   part du symptôme) — les sept tables sous toutes graphies, les colonnes `bank_account_id`,
   `settlement_bank_account_id`, `is_primary`, `journal_account_id`, `status`, `auto_match_rejected_at`, `archived`,
   `closed_on` ; les crates annexes, `admin_backup`, `post_restore`, `test_endpoints.rs`. Un écrivain absent, ou mal
   classé, est au moins **HIGH**. Chaque classement **Invariant** contre l'énumération de **tous** les écrivains de
   l'état (`status = 'pending'`, lots `generated`, `is_primary`, `archived`) et contre le geste ultérieur qui le défait.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Comptes bancaires* (l. 1642 et
   suivantes), § *Supprimer un import* (l. 1724, l. 1754-1757), § *Import bancaire* (l. 1696), § *Solde comptable*
   (l. 238-262), le règlement par virement, les mentions du principal (l. 936, 947, 1047) ;
   `docs/manual/fr/admin-manual.tex` — l. 1943 (étape 1 de la mise à jour), § *Rollback en cas d'échec* (l. 2234,
   l. 2251), § *Passer à la 0.13.2* (l. 1972). Ce que l'AC14 prescrit d'écrire sera-t-il vrai au code prescrit ?
   Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
4. **Recomptes** : chaque nombre de la fiche depuis la source (migrations 80 → 81, compteurs de
   `docs/migrations-idempotence-audit.md`, `migrations_upgrade_path.rs` et ses lignes `\b(80|46)\b`,
   `migrations.sha384`, routes 115 / 118 et les assertions, `sitesTotal` 2075, les 29 sites, les seize clés UI, AC,
   tests, mutations, choix, numéros de ligne cités).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #642 couverte par un AC testable, chaque AC par une tâche et par un test **qui rougirait**, chaque
  mutation par un test qui la tue ; chaque fait, chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ;
  les textes prescrits (i18n ×4, dont les deux messages modifiés de l'AC9 et le test `bank_accounts_e2e.rs:2316-2320`
  qui doit rester vert à l'octet ; manuels ; CHANGELOG `### Ajouté` et `### Corrigé` ; `api-external.md`) confrontés à
  ce que le code prescrit fera ; les gardes existantes qui changent de sens (G9 de `textes_coherents.rs`, l. 711 ;
  `audit_route_registry.rs` ; `audit_label_registry.rs` ; `test_schema_guard.rs` ; `migrations_upgrade_path.rs` ;
  `i18n-keys.test.ts` ; `lint-i18n-ownership` ; en-têtes CSV ; `bank-accounts-crud.spec.ts`) — la fiche les nomme-t-elle
  toutes ? Les consommateurs de `listBankAccounts` et de `/companies/current` (tuile, Paramètres,
  `ReconciliationProposals.svelte`) sont-ils tous traités, et la signature à deux booléens (C-28-4-20) passe-t-elle
  `npm run check` à chaque appelant (lecture des appels) ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : l'axe obligatoire ci-dessus,
  en entier, puis : la règle de la garde est-elle correcte sous InnoDB REPEATABLE READ (naissance de la vue,
  lecture verrouillante contre vue, montée de verrou partagé → exclusif — la fiche dit désormais ce cas « à venir,
  non présent » : est-ce vrai ?) ; les tests déterministes font-ils rougir les mutations qu'ils prétendent tuer ; la
  contrainte `CHECK` de l'AC1 et le non-bump de `min_required` (C-28-4-11) ; le refus par proposition dans `accept` /
  `reject` contre § *Pattern batch* et le pré-vol de la 27-4 ; le registre statique de l'AC8 (extraction hors
  commentaires et hors `mod tests`, fonction englobante, deux sens du diff, classement de W8 « gardée par ses
  appelants » : un statut par couple `(fichier, fonction)` peut-il dire vrai pour une fonction appelée par trois
  chemins dont un sans garde ?) ; l'AC6-bis : les erreurs réutilisées sont-elles atteignables et rendent-elles le
  code HTTP annoncé par la route (route par route, mappage `DbError` → `AppError` lu) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la **remédiation P1**
(commit `32ee4e36`) ou de la conception d'origine. Toute affirmation de présence ou d'absence = sortie de `grep -nF`
copiée. La lentille F rend aussi la **table de l'axe obligatoire**. ⛔ **La liste des axes exercés ET non exercés** —
un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`,
`perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script
du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle
dans le rapport. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard (ou vers un fichier d'aplatissement **dans** `kesh-gate-logs/`).
