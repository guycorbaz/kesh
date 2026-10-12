# Prompt — validation P2 de la spec, Story 27-1 (supprimer un import bancaire non rapproché)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; la P1 était Sonnet, la **P2 est Opus** pour les deux lentilles. Haiku reste
réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-27-1`, branche `story/27-1-supprimer-import` (base `aadace40`).
Fiche : `_bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md` — état à relire :
commit **`3da1335d`** (remédiation P1). Diff de la remédiation : `git diff eeb359cc 3da1335d`.
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p1-{R,F}.md` — ne pas les recopier ; vérifier que
leurs findings sont réellement fermés, et chercher **ce que la remédiation a introduit**.
Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`. Issue #632 (`gh api repos/guycorbaz/kesh/issues/632`).
Registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md` (C-27-1-1 à 15 ; C-27-1-7 remplacé par
C-27-1-12) ; doctrine du rejeu : `epic-15-choix-autonomes.md` C54-C59. Règles : `CLAUDE.md` (§ Review Iteration
Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch,
le recompte ; § Migration breaking policy ; § Issue Tracking Rule).

## Ce que la remédiation P1 a changé — c'est là que les défauts naissent

Sur les huit passes cumulées des Epics 22-23, sept ont trouvé une régression du patch précédent. Lis d'abord ces
zones, au diff :

1. **AC3, motifs de synchronisation** : (a) passe à `["bank_transactions", "FOR UPDATE"]` (M3 détectée par la
   panique de 10 s) ; (c) passe à `["FOR UPDATE"]` et déclare M14, M15 équivalentes, M14+M15 rouge par
   l'assertion. Le raisonnement tient-il (quelle requête attend, dans chaque cas, et son texte porte-t-il le
   motif ; le motif `["FOR UPDATE"]` peut-il voir une **autre** requête que la seconde suppression — B, la sonde
   elle-même (`information_schema.PROCESSLIST`, `ID <> CONNECTION_ID()`), une autre tâche) ?
2. **AC4-bis, neuf** : tri des identifiants de `post_reject` (`routes/reconciliation.rs:3079`). Ferme-t-il
   vraiment le cycle avec la suppression (ordre des verrous du rejet : `find_pending_by_ids` sur le pool, puis
   `GET_LOCK`, puis les `UPDATE` ; ordre de la suppression : entête puis lignes par `idx_bank_transactions_import`) ?
   Quels contrats change-t-il (ordre de `rejected[]`/`failed[]`, `entity_id` de `reconciliation.rejected`,
   frontend `ReconciliationProposals.svelte`, tests existants de `reconciliation_e2e.rs`, manuel, CHANGELOG) ?
   L'ajout du rejet au périmètre fait-il franchir le seuil du découpage préventif ?
3. **AC8 réécrit** : la levée acceptée du `412 BANK_ACCOUNT_HAS_TRANSACTIONS` (C-27-1-12) et le test H6 —
   faisable avec les helpers de `bank_accounts_e2e.rs` (onboarding complet, `version`, compte principal) ? Le
   renvoi à #465 (C-27-1-13) est-il exact et suffisant ?
4. **Angles morts ajoutés** (restauration, création concurrente, sauvegarde, état `pending` avec lien, annulation
   refusée) : chacun exact au code ?
5. **AC9** : `canManage` devenu prop du composant — la page, le test, M16 et le compte `sitesTotal` (+9, inchangé)
   restent-ils cohérents ?
6. **AC13 / AC14** : les phrases neuves du manuel (archivage, tuile, « aucun geste de l'interface », import
   indélébile) et d'`api-external.md` (§ 7, § 10, paragraphe *Interblocages* l. 662) disent-elles ce que le code
   prescrit garantira ?
7. **Recomptes du Change Log** : 17 AC, 7 MEDIUM distincts en P1, 14 LOW, 17 mutations à jouer, compteurs du
   registre des routes (114/115, 117/118, 90/91, 108/109, six/sept) — recompter depuis la fiche et le code.

## Axes communs aux deux lentilles

- **Inventaires clos** (Dev Notes § Inventaire (a) et (b)), complétés en P1 d'un lecteur (garde d'archivage) et
  des effacements totaux : en reste-t-il un ? Un **lecteur qui compte** ou qui **dérive un état** de
  `bank_transactions`/`bank_imports` (tuiles, rapports, contrôles de continuité de relevés, exports) manquant est
  au moins MEDIUM.
- **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Accepter ou rejeter* (l. 1859+,
  *Rejeter* l. 1873), § *Import dans Kesh* (l. 1690-1719), § *Comptes bancaires* (l. 1643+), § *Annuler un
  rapprochement* (l. 1940+), § *Solde comptable, relevé et écart* (l. 238+) ; et le **PDF** aplati
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #632 et de la décision du Project Lead (epic) couverte par un AC testable, chaque AC par
  une tâche, chaque AC par un test **qui rougirait** ; chaque fait, chemin et numéro de ligne cité par la fiche
  **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (i18n, manuel, CHANGELOG, `api-external.md`)
  confrontés à ce que le code prescrit fera réellement ; les gardes existantes qui changeront de sens
  (`audit_route_registry.rs`, `audit_label_registry.rs`, `audit_log_e2e.rs`, `i18n-keys.test.ts`,
  `lint-i18n-ownership`, `textes_coherents.rs`, `test_schema_guard.rs`) — la fiche les nomme-t-elle toutes ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : la séquence de l'AC2
  est-elle **correcte sous InnoDB REPEATABLE READ** (lecture verrouillante vs vue cohérente, ordre des verrous,
  verrous de clé suivante sur `idx_bank_transactions_import`, cascade `ON DELETE CASCADE` et verrous qu'elle
  prend, insertion d'audit) ? Les trois tests de course de l'AC3 sont-ils **déterministes** et font-ils rougir
  les mutations annoncées (notamment M3, M14, M15 — le motif de `attendre_une_requete_en_cours` voit-il la bonne
  requête) ? Le raisonnement « verrou nommé relâché avant le commit » (`kesh-reconciliation/src/mutex.rs`,
  `accept_once`) est-il exact, et le choix de **ne pas** prendre `with_account_lock` (C-27-1-2) laisse-t-il une
  course ? Croise la suppression avec **chaque** écrivain de l'inventaire (b) et avec `create` / `create_csv`
  (même compte, même fichier), `post_reject`, `cancel_in_tx`, la restauration d'installation, l'export global.
  Le rejeu (`retry_app_on_deadlock`) est-il bien placé (une tentative = une transaction neuve, aucune conversion
  d'erreur avant le prédicat), et le classement `SansEcritureAuJournal` + « rejouée quand même » est-il
  compatible avec les volets (c), (c bis) et (d) du registre ? La route dans `comptable_routes` partage-t-elle
  son chemin avec le `GET` d'`authenticated_routes` sans conflit Axum (précédent `bank-profiles/{id}`) ? Les clés
  d'API : la décision C-27-1-3 tient-elle contre § 4 et § 7 de `docs/api-external.md` ?

## Ce que tu rends

Findings numérotés (`R2-…` ou `F2-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt, `gh` en
écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
