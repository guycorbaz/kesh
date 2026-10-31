# Prompt — revue de code P1 de la Story v016-2a (saisir un relevé bancaire à la main, le backend), trois lentilles, Sonnet

Tu relis le développement de la **Story v016-2a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
*saisir un relevé bancaire à la main — le backend* (#816). Ce qu'il livre : la variante `Manual` (`"MANUAL"`) de
`BankImportSourceFormat` (`kesh-db`) et de `SourceFormatTag` (`kesh-core`) ; le module **sans I/O**
`crates/kesh-core/src/bank_imports/manual.rs` (corps en chaînes, analyse, règles 1 à 6, équilibre exact, empreinte
canonique encodée par longueur, ensembles fermés `ManualStatementReason` / `ManualStatementField` avec `index()`,
`COUNT`, `ALL`) et ses tests `manual_tests.rs` ; **la migration** `crates/kesh-db/migrations/20261030000001_bank_imports_manual_source.sql`
(une seule instruction : `kesh_version_min_required` → `0.17.0`), le **bump des dix crates à 0.17.0** dans le même commit
(P2-bis), `migrations.sha384`, le squash `test-schema/`, `post_restore.rs` (`EXEMPT_MIGRATIONS`), l'audit d'idempotence ;
deux lectures de dépôt (`previous_statement_closing`, `overlapping_statements`) ; la montée sans perte depuis une base
0.16.x peuplée et l'inventaire de la migration (`migrations_upgrade_path.rs`) ; la sauvegarde (`admin_full_import_e2e.rs`) ;
l'extracteur partagé `crates/kesh-api/src/extractors.rs` (sorti de `routes/bank_accounts.rs`) ; le module de routes
`crates/kesh-api/src/routes/bank_statements_manual.rs` (aperçu et création), le montage (`lib.rs`), quatre erreurs
(`errors.rs`) et leurs messages aux quatre catalogues, `ImportAuditAction` et `existing_dedup_keys`
(`routes/bank_imports.rs`), l'action `bank_import.entered_manually` (`audit_labels.rs`), les registres
(`audit_route_registry.rs`, `audit_label_registry.rs`, `bank_account_closed_inventory.rs`, `textes_coherents.rs`) ;
le message `bank-accounts-errors-has-pending-transactions` récrit ; les textes : `docs/api-external.md`, `CHANGELOG.md`
(`## [0.17.0] — Non publié`), le manuel d'administration FR (`.tex` **et** PDF) — § *Passer à la 0.17.0*, `:2270`,
`:2966` —, le manuel utilisateur FR (`:3265`, `.tex` **et** PDF), `README.md` (`:49`, feuille de route), et les neuf
commentaires « depuis la 0.16.1 » passés à la 0.17.0.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0162 diff a458b560...HEAD` (`a458b560` = la fiche validée, avant le développement ;
deux commits de l'agent de fiche de la 2b, `63a68a64` et suivants s'il y en a, s'y mêlent : ils ne touchent que
`_bmad-output/` et ne sont pas l'objet de cette revue). Langue du rapport : français. Modèle : **Sonnet**. Passe
**complète** (première passe d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les quatre propriétés centrales.**
1. **Les données de la 0.16.x se gardent** (contrainte du Project Lead, C-v016-2-16) : la migration ne touche aucune
   ligne ; une base 0.16.x peuplée monte et se relit à l'identique, en base et par le code ; une sauvegarde 0.16.x
   s'importe dans la 0.17.0 ; le retour passe par la sauvegarde d'avant la mise à jour, et les textes le disent.
2. **Rien ne s'écrit sur un refus**, ni à l'aperçu : entête, lignes et trace dans une seule transaction ; un échec de la
   trace n'écrit rien ; chaque refus (`400`, `404`, `409`, `422`) laisse la base intacte. Les doublons de lignes ne sont
   **jamais** ignorés en silence.
3. **L'équilibre est exact** et l'empreinte canonique est stable, sensible à chaque champ et injective ; les longueurs
   se comptent en caractères, après `trim`.
4. **Les ensembles fermés** `reason` (onze) et `field` (treize) sont épinglés sur les chaînes de la fiche parente, et
   leur exhaustivité (C-v016-2-20) n'est pas circulaire.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la sous-fiche
`_bmad-output/implementation-artifacts/v016-2a-releve-manuel-backend.md` — **AC-1 à AC-18, AC-28, AC-29** (hors son point
2), *Tests et mutations prévus*, *À recontrôler à l'étape 0*, le **Dev Agent Record** et le **Change Log** ; la fiche
parente `v016-2-releve-manuel.md` (§ *Le corps de requête*, *Réponse de l'aperçu*, *Codes d'erreur*, *Ensembles fermés*,
*Type d'entrée*, *Ordre des refus*, *Ce que les gardes ne voient pas* ; le § *Ce que la release doit contenir* n'est pas
du développement : ne le juge pas) ; le registre `v016-2-choix-autonomes.md`, **C-v016-2-1 à 19** (fiche) et
**C-v016-2-20 et suivants** (développement) ; la contrainte `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
Le Dev Agent Record déclare des rouges, des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**,
recoupe-les avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/v0162a/` (rouges,
gates, mutations, E2E, manuels).

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni **aucun script du dépôt** ni
  de `kesh-gate-logs` ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni `mysql`/`mariadb`, ni `sqlx`,
  ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`, `sha384sum` (lecture),
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -cF` ou
  `grep -oF … | wc -l` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'Passer à la 0.17.0'`.
  ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) : greper `d’avant`, non `d'avant`, ou un motif sans apostrophe.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (IBAN d'exemple SIX, « Assurance Exemple SA »).

## Les axes obligatoires, pour chaque lentille

1. **La migration contre la *Migration breaking policy* du `CLAUDE.md`, P1 à P8.** P1/P2 : une seule instruction, la
   dernière, version figée ; P2-bis : `grep -n '^version' crates/*/Cargo.toml` rend dix fois `0.17.0`, au **même commit**
   que la migration (`git log --format=%h -- <migration>` et `git show <h> --stat`), `Cargo.lock` suivi ; P3/P4 :
   aucune opération de schéma ; P5 : la ligne d'audit, les **deux** sites du total et les trois compteurs **recomptés
   depuis le tableau** (`ls crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`,
   et le compte des verdicts), le verdict `yes` justifié ; P6 : chaque site de
   `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` (positionnel assumé avec garde, ou résolu par version),
   `migrations_upgrade_path.rs` (`total`, `total - 53`, frontière 34, toutes les copies des valeurs par
   `grep -nE '\b(52|53|86|87)\b|total - 5[23]'`) ; P7 : l'entrée `EXEMPT_MIGRATIONS` et le compteur `22` avec sa ligne
   d'historique ; P8 : le checksum inscrit à `migrations.sha384` égale `sha384sum` du fichier. La sémantique des tests de
   version relue (`Aligned` à `0.17.0`, `DowngradeRefused` à `0.16.1`, `BinaryAhead` à `0.18.0`).
2. **La montée sans perte et la sauvegarde 0.16.x** : `bank_imports_manual_source_preserves_existing_rows` (le montage
   peut-il passer à vide ? la comparaison colonne par colonne couvre-t-elle toutes les colonnes ? la relecture
   applicative passe-t-elle par `from_str` ?), `bank_imports_manual_source_only_raises_min_required` (le découpage
   décommenté), `bank_imports_manual_source_adds_no_table`, `full_import_accepts_a_0_16_backup` (le manifeste récrit
   est-il bien celui qu'écrirait une 0.16.x ? les données exportées par la 0.17.0 ont-elles la forme de celles d'une
   0.16.x ?), `full_import_round_trip_keeps_a_manual_statement`.
3. **Le manuel d'administration (`.tex` et PDF aplati) contre le code** : § *Passer à la 0.17.0* (la migration, le refus
   de la 0.16.x au démarrage **et** à l'import, la sauvegarde complète d'avant la mise à jour, le sens 0.16.x → 0.17.0),
   `:2270` (la forme prescrite), `:2966` (`130 des 137`, `130 + 4 + 3 = 137`, contre `audit_route_registry.rs`), chaque
   `\ref{sec:maj-0-17-0}` et l'absence de `maj-0-16-1` ; le PDF dit-il la même chose que le `.tex` (comparer à
   `git show a458b560:docs/manual/fr/admin-manual.pdf | pdftotext - - | …`) et ne porte-t-il plus `Passer à la 0.16.1`
   ni `129 des 135` ? Le manuel utilisateur (`:3265`) et son PDF. `docs/api-external.md` contre les routes (codes,
   charges utiles, ordre des refus, ensembles fermés, limite de corps).
4. **Le tri des `0.16.1`** : `git grep -n '0\.16\.1' -- crates frontend/src docs scripts README.md CHANGELOG.md website`
   et le PDF aplati ; chaque occurrence classée au Dev Agent Record (« historique », « release », « corrigée ») l'est-elle
   à juste titre ? Les URL `raw.githubusercontent.com/…/v0.16.1` et `\keshVersion` doivent rester (garde G19, tag final).

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** : `manual.rs` (analyse stricte des dates et des nombres, `normalize()`, bornes, ordre des
   contrôles, `i32::try_from`), `bank_statements_manual.rs` (chaque `return Err` dans la transaction annule-t-il ?
   l'aperçu n'écrit-il vraiment rien ? un `?` après une écriture laisse-t-il une transaction ouverte ?),
   `extractors.rs` (comportement **inchangé** pour `bank_accounts.rs` ; `$body:ty` au lieu de `ident`), `errors.rs`
   (statuts, charges utiles), `bank_imports.rs` (`ImportAuditAction`, `existing_dedup_keys`, visibilités `pub(crate)`).
   Une requête SQL écrite dans le module de routes ? Un `INSERT`/`UPDATE`/`DELETE` nouveau dans `crates/*/src` (AC-8) ?
2. **Les tests** : chacun prouve-t-il ce qu'il nomme, ou peut-il passer à vide (un `counts` égal parce que rien n'a été
   tenté, un `find` qui ne trouve rien, une boucle vide) ? Le déclencheur du test M22 est-il bien restreint à l'action ?
3. **Les textes** : cohérence interne du CHANGELOG (forme `## [0.17.0] — Non publié`, que `scripts/prepare-release.sh`
   **lit** — ne pas l'exécuter), du README (statut de candidate), des catalogues (quatre locales, parité) ; aucun
   mot-clé de fermeture (`closes`, `fixes`, `resolves`) dans les messages de commit (`git log a458b560..HEAD --format=%B`).

## Lentille E — Edge cases

1. **L'analyse et les règles** : `"-0"`, `"+5"`, `" 5 "`, `"5."`, `".5"`, `"0005.00"`, une partie entière de 14 chiffres
   à zéros de tête, `"1e3"`, un montant au maximum de 13 chiffres et ses sommes (débordement de `DECIMAL(18,2)` par 500
   lignes ?), une date `2026-02-29`, `9999-12-31`, une période d'un jour, 0 et 500 lignes, un texte de 255 caractères
   de quatre octets, un `statementNumber` fait de blancs, une ligne `valueDate` vide. L'empreinte : deux relevés qui ne
   diffèrent que par `null` contre chaîne vide, par l'ordre des lignes, par `"50"`/`"50.00"`.
2. **Les gardes en base** : relevé identique puis doublons puis écart de solde, dans cet ordre ; `previous_statement_closing`
   avec deux relevés de même `period_to` et même `imported_at` ; un relevé précédent sans solde ; la concurrence (compte
   clos entre le pré-vol et la création — test de dépôt) ; un compte archivé/clos à l'aperçu.
3. **Les rejets du corps** : chacun rend-il `400 VALIDATION_ERROR` sans `field` ni `reason` (sans `Content-Type`, au-delà
   de la limite, JSON illisible, `"amount": 50`, `lines` non tableau, `bankAccountId` absent ou chaîne) ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-1 à AC-18, AC-28, AC-29** hors point 2), rends « tenu », « non tenu » ou « non vérifiable en lecture
seule », avec la preuve. Le § *À recontrôler à l'étape 0* : chaque point recontrôlé au Dev Agent Record, ligne lue
citée ? Les cinq LOW de la validation P5 appliqués (P1 à P5) ? Les mutations M1 à M24 (et M23-bis) : chacune tuée **par le
test nommé**, pour la raison visée (journal de campagne) ? Les compteurs de garde recalés (ancienne et nouvelle valeur)
se recomptent-ils ? Les choix **C-v016-2-20 et suivants** sont-ils justifiés, l'un contredit-il un AC ? Chaque nombre du
Dev Agent Record (tests, gates, mutations, E2E et ses échecs nominatifs contre `docs/testing.md` § *Les échecs
attendus*) se recompte-t-il depuis sa source, avec son périmètre ? Les **défauts observés** demandés en issue sont-ils
réels (prémisse lue au code) ? `sprint-status.yaml` : la clé à `review`.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0162a-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/v0162a-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0162a-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v016-2a ou **antérieur** (un défaut antérieur se **signale**, il ne se
   corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
