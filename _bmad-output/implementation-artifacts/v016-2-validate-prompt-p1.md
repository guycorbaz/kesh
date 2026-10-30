# Prompt — validation P1 de la fiche v016-2 (saisir un relevé bancaire à la main, #816), deux lentilles, Sonnet

Tu valides la **fiche** de la Story v016-2 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) : la
**saisie manuelle d'un relevé bancaire**, pour un compte dont la banque ne fournit ni CAMT.053 ni CSV (#816). La fiche
retient qu'un relevé saisi est un **import de `source_format = 'MANUAL'`**, créé par la même fonction de dépôt que
l'import (`bank_imports::create_with_transactions`), avec un contrôle d'équilibre **exact**, un solde de départ attendu
signalé et à confirmer, des doublons de lignes sans option « ignorer », une action d'audit propre
(`bank_import.entered_manually`), et **une migration** dont la seule instruction relève `kesh_version_min_required` à
`0.16.1`. La story entre dans la release 0.16.1 (candidate rc2).

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel`, partie de `c5101b07`. Langue du rapport :
**français**. Modèle : **Sonnet**. Passe **complète** (première passe de validation). **Contexte frais** : ne t'appuie
sur aucune passe antérieure, ni sur ce que la fiche dit d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v016-2-releve-manuel.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-14).
3. L'issue, **en lecture seule** : `gh issue view 816 --json title,body,comments` ; pour le voisinage, `gh issue view`
   de 418, 526, 671, 687 (`--json title,state,body`).
4. Le code du circuit bancaire : `crates/kesh-db/migrations/20260504000001_bank_imports.sql` et les migrations `bank_*`
   suivantes ; `crates/kesh-db/src/entities/bank_import.rs`, `bank_transaction.rs` ; `crates/kesh-db/src/repositories/`
   `bank_imports.rs`, `bank_transactions.rs`, `bank_accounts.rs` (`BALANCES_SQL`, `statement_gap`), `closing_checks.rs`
   (K3), `post_restore.rs` (`EXEMPT_MIGRATIONS`) ; `crates/kesh-core/src/bank_imports.rs` ; `crates/kesh-api/src/routes/`
   `bank_imports.rs`, `bank_import_fingerprint.rs`, `reconciliation.rs` ; `crates/kesh-api/src/lib.rs` (montage),
   `errors.rs`, `audit_labels.rs`, `routes/admin.rs` (import d'installation, `min_required`), `main.rs`
   (`check_downgrade_protection`) ; `crates/kesh-reconciliation/src/` ; les tests-gardes
   `crates/kesh-api/tests/bank_account_closed_inventory.rs`, `audit_route_registry.rs`, `audit_label_registry.rs`,
   `crates/kesh-db/tests/test_schema_guard.rs`, `migrations_fresh_install.rs`, `migrations_upgrade_path.rs` ; le
   frontend `frontend/src/lib/features/bank-import/`, `frontend/src/lib/features/journal-entries/balance.ts`,
   `frontend/src/routes/(app)/bank-import/`, `frontend/tests/e2e/reconciliation-amount-due.spec.ts`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` § *Tableau de bord* (l. 218-290), *Préparation de la clôture*
   (autour de l. 803), *Import bancaire* (l. 2037-2290), *Réconciliation bancaire* (l. 2290-2513) ;
   `docs/manual/fr/admin-manual.tex` § *Passer à la 0.16.1* (l. 2481) et *Sauvegarde et restauration* (l. 1477) ;
   les PDF correspondants, lus aplatis **vers la sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. Pour les règles : `CLAUDE.md` du worktree (*Migration breaking policy* P1 à P8, *Un appariement automatique propose,
   il ne crée jamais*, *Règle de splitting préventif*, *Recette visuelle*, *Issue Tracking Rule*, liste de contrôle
   pré-release), `docs/agents-orchestration.md`, `CHANGELOG.md` § `[0.16.1]`, `docs/api-external.md`.

## Deux lentilles, en contexte frais chacune

L'orchestrateur lance **deux agents distincts** sur ce même prompt ; chacun ne tient **que** sa lentille.

### Lentille R — la règle et sa preuve

- **Les prémisses** : chaque `fichier:ligne` cité par la fiche et le registre dit-il ce qu'on lui fait dire (numéros de
  `c5101b07`) ? La prémisse corrigée sur le MCP (aucun outil ne lit `bank_transactions` ni `bank_imports`) est-elle
  vraie ? Le seul créateur de `bank_transactions` hors tests est-il bien `create_with_transactions` ?
- **Le modèle (C-v016-2-1)** : un relevé `MANUAL` est-il **réellement** indiscernable d'un import pour le rapprochement
  (les sept chemins de `reconciliation.rs`, les propositions, le rejet et son annulation, l'annulation d'un
  rapprochement, le rattachement à un règlement), la suppression, la tuile (`BALANCES_SQL`), K3, l'export global et la
  sauvegarde ? Cherche **tout** site qui `match`e sur `BankImportSourceFormat` ou `SourceFormatTag`, ou qui lit
  `source_format`, `filename`, `file_hash` ou `statement_id` (`grep -rn` sur `crates/` et `frontend/src`) : un
  `match` non exhaustif, un affichage, un export, un test qui énumère les trois valeurs actuelles.
- **La migration (C-v016-2-2, AC-2)** : le raisonnement tient-il au code (le décodage d'une valeur inconnue échoue-t-il
  bien, et où rend-il une erreur) ? Le relèvement de `min_required` est-il conforme à P1/P2/P2-bis (crates déjà en
  0.16.1 ?) et la procédure de P5, P6, P7, P8 et du squash est-elle complète ? Le triage P7 proposé
  (`ExemptionBasis::Durable`, justification) est-il celui des précédents (`20261009000001`, `20261015000001`) et passe-t-il
  `every_data_backfill_migration_is_triaged` et les contrôles de fenêtre ? Les tests qui figent `0.14.0` sont-ils tous
  nommés (`grep -rn "0\.14\.0" crates`) ? L'option (b) « aucune migration » est-elle écartée pour une raison vraie ?
- **La validation et l'équilibre (AC-4, AC-5)** : chaque règle est-elle décidable, testable et compatible avec les
  colonnes (`DECIMAL(18,2)`, `VARCHAR(255)` en **octets**, `TEXT`) ? L'empreinte canonique est-elle sans ambiguïté
  (deux relevés différents peuvent-ils produire les mêmes octets ?) et compatible avec `chk_bank_imports_hash_len` ?
- **Les doublons et le solde de départ (C-v016-2-5, C-v016-2-6, AC-6, AC-7, AC-11)** : la définition du relevé
  précédent est-elle celle de la tuile ? Le comportement « saisi puis importé » est-il décrit exactement (relis
  `dedup_key_scalar`, `normalize_reference_fallback`, `find_in_dedup_window`) ? L'ordre des refus est-il cohérent avec
  l'ouverture de la transaction et la garde du compte clos (Stories 28-4a, 29-4a) ? La règle « un appariement
  automatique propose, il ne crée jamais » est-elle tenue ?
- **Les gardes (AC-8, AC-15)** : `bank_account_closed_inventory.rs` restera-t-il vraiment vert sans retouche si la route
  ne porte aucun SQL ? Le passage de l'action d'audit en paramètre fait-il bien du site un site **indirect** au sens de
  `audit_label_registry.rs`, et la fiche prescrit-elle la bonne inscription ? `audit_route_registry.rs` exige-t-il
  autre chose pour une route nouvelle (rejeu, statut) ?
- **Les tests et mutations (M1 à M16)** : chaque mutation rougit-elle le test annoncé, et seulement pour la raison
  annoncée ? Une régression probable reste-t-elle sans mutation (par exemple un aperçu qui écrit, une confirmation mal
  câblée côté frontend, la clé `read`) ? Le parcours Playwright (AC-26) est-il réalisable avec les aides existantes
  (proposition d'une facture sur un crédit de son montant) ?
- **Le découpage (C-v016-2-14)** : le décompte de modules est-il juste au sens de la règle de splitting ?

### Lentille F — les textes, l'écran et ce qu'ils affirment

- **Le manuel (AC-27)** — axe obligatoire. Chaque phrase que la fiche dit fausse après la story l'est-elle ? En
  manque-t-il (cherche au `.tex` **et** au PDF aplati : « deux formats », « CAMT.053 porte », « CSV n'en a »,
  « relevé importé », « un relevé a été importé », « importez », « doublons », « ignorer », le contrôle K3) ? La
  sous-section prescrite promet-elle quelque chose que le code prescrit ne tiendra pas ? La conduite « la banque passe
  au CAMT » est-elle juste ?
- **Le CHANGELOG, le § *Passer à la 0.16.1*, le README, `docs/api-external.md` (AC-28 à AC-30, § *Ce que la release doit
  contenir*)** : l'en-tête actuel de `[0.16.1]` (« aucune migration ») devient-il faux, et la fiche le dit-elle sans
  le faire corriger au développeur ? La liste de la release manque-t-elle un geste de la liste de contrôle pré-release
  de `CLAUDE.md` pour une version **avec** migration (sauvegarde avant mise à jour, `prepare-release.sh`, exemptions
  datées, macros, URL, PDF, site) ? Les notes de la rc2 couvrent-elles le retour à la rc1 ?
- **L'écran (AC-19 à AC-25)** : les critères sont-ils décidables par un test ? La règle « *Enregistrer* inactif »
  (écart, ligne invalide, aperçu périmé) est-elle complète et sans contradiction avec le serveur (deux décimales, dates
  dans la période, une ligne vide ignorée) ? La généralisation de `isValidAmount` laisse-t-elle intact le journal ?
  Les libellés de liste, de détail et de la boîte de suppression sont-ils tous couverts ? La recette visuelle (AC-25)
  liste-t-elle chaque état touché ?
- **i18n (AC-24)** : toutes les clés et les codes d'erreur nouveaux dans les quatre catalogues ; la règle de propriété
  des clés (`lint-i18n-ownership`) est-elle compatible avec le préfixe prescrit ?
- **Les décomptes** de la fiche et du registre (87 migrations après la story, 46 occurrences P6, numéros de ligne) :
  recompte-les depuis la source.
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; données réelles absentes
  (l'exemple de corps est-il bien synthétique ?) ; aucun défaut voisin intégré qui ne soit mécanique ; les questions
  au Project Lead (Q1 à Q6) recouvrent-elles toutes les décisions qui engagent l'usage réel de Guy ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0162-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0162-validate-p1-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`, `R2`… ou `F1`, `F2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`),
   le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait
   copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF`
   copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés** de ta lentille, un par un, avec, pour chaque axe exercé, ce que tu as
   lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `mariadb`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`,
  ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
