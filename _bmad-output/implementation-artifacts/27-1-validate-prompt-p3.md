# Prompt — validation P3 de la spec, Story 27-1 (supprimer un import bancaire non rapproché)

*Versionné le 2026-10-12. Passe **complète**, deux lentilles **Sonnet** (rotation Sonnet ↔ Opus : P1 Sonnet,
P2 Opus), contexte frais chacune, lecture seule. Pas de passe ciblée : la remédiation P2 **change une règle
métier** (décision du Project Lead — la route est fermée aux clés d'API) et touche plusieurs modules prescrits
(`CLAUDE.md`, § *La passe ciblée*, « quand elle ne suffit PAS »).*

Dépôt `/home/gcorbaz/devel/kesh-27-1`, branche `story/27-1-supprimer-import` (base `aadace40`).
Fiche : `_bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md` à **`61c8776d`**.
Diff de la remédiation P2 : `git diff e8e1cc2e 61c8776d`. Rapports antérieurs :
`/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p{1,2}-{R,F}.md` — vérifier que leurs findings sont réellement
fermés, ne pas les recopier. Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`. Issues #632
(objet), #633 (sauvegarde sans instantané), #634 (lot refusé en entier), #465 (export global) —
`gh api repos/guycorbaz/kesh/issues/N` en lecture. Registre : `epic-27-choix-autonomes.md`, C-27-1-1 à 19
(C-27-1-7 remplacé par C-27-1-12 ; C-27-1-3 révisé par C-27-1-16 ; C-27-1-13 révisé). Règles : `CLAUDE.md`
(§ Review Iteration Rule et ses sous-sections ; § Pattern batch ; § Issue Tracking Rule).

## Ce que la remédiation P2 a changé — c'est là que les défauts naissent

1. **Décision du Project Lead (C-27-1-16)** : `DELETE /api/v1/bank-imports/{id}` refuse toute clé d'API — clé
   `read` par le middleware (`API_KEY_READ_ONLY`), clé `read-write` par un contrôle en tête du handler, code **neuf**
   `API_KEY_OPERATION_FORBIDDEN`. Le choix d'un code neuf contre la réutilisation tient-il (doctrine D2,
   `errors.rs:204-217`) ? Le contrôle est-il placé pour qu'aucune trace ni suppression n'ait lieu, et avant le
   rejeu ? Ce choix entre-t-il en conflit avec une garde du dépôt — `admin_pat_denied_e2e.rs` (consommateurs de
   `require_not_pat`, liste des codes 403), `audit_route_registry.rs`, un recensement des variantes d'`AppError`
   ou des codes d'erreur, `docs/api-external.md` § 4 (« seules l'administration et le journal d'audit sont fermés
   aux clés » : la phrase devient-elle fausse ?) ? Tout ce que la fiche disait de la route **pour les clés**
   (trace qui nomme la clé, paragraphe *Interblocages*, § 7, § 10, CHANGELOG) a-t-il été propagé ?
2. **AC3 (b) réécrit** en deux trajectoires (pré-vol `find_pending_by_ids` contre `UPDATE`) ; **AC8** gagne la
   puce du lot refusé en entier (#634) et le renvoi #633 ; exacts au code ?
3. **AC3 (c)** : l'import du montage porte au moins une transaction — le raisonnement sur les verrous tient-il
   alors pour M14, M15, M14+M15 ?
4. **AC13** : phrases neuves (archivable « s'il ne reste aucune transaction », « tant que l'annulation est
   refusée », renvoi aux cinq motifs du § *Annuler un rapprochement* l. 1964-1983, lot de réconciliation à
   recharger, renvoi depuis § *Comptes bancaires*) — vraies au code, et au PDF actuel pour ce qu'elles citent ?
5. **AC10** : onze clés neuves et **une modifiée** (`bank-accounts-errors-has-transactions`, repli
   `errors.rs:2467-2470`) — un test ou un E2E existant compare-t-il ce message ? La phrase ajoutée est-elle vraie
   dans tous les cas ?
6. **Angles morts réécrits** (restauration via `audit_log`, imports que l'écran ne peut pas supprimer — D3, D4,
   annulation refusée —, verrou de trou toutes sociétés, `innodb_snapshot_isolation`) : exacts ?
7. **Recomptes** du Change Log : 4 MEDIUM distincts en P2, 16 LOW bruts, 18 mutations à jouer, H1-H8, onze clés,
   registre des routes (114/115, 117/118, 90/91, 108/109, six/sept) — depuis la fiche et le code.

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

Findings numérotés (`R3-…` ou `F3-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt, `gh` en
écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
