# Prompt — revue de code P1, Story 28-5a (annuler le rejet d'une transaction, côté serveur)

*Versionné le 2026-10-13. Trois lentilles (**Sonnet**), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5`.
**Le diff à revoir** : `git diff da25bf1b 822ba587 -- . ':(exclude)_bmad-output'` (`da25bf1b` = validation close de
la fiche ; `822ba587` = **seul et dernier commit de code** ; les commits suivants ne portent que la fiche, le
registre des choix, le registre de sprint et ce prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le shell
peut interpréter. Fiche : `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md` — **partie a
seulement** : AC1 à AC4, AC7 (a), AC9 (a), AC10, AC12 (a), AC13 ; tâches T0a-T7a ; § *Tests* (B1-B10, M1-M8, M13-M15,
M17) ; Dev Notes (séquence de la route, inventaire des sites non résolus, textes, « ce qui doit rester intact ») ; et
le **Dev Agent Record** (rouge d'abord, campagne de mutations, gate final, décomptes). Choix C-28-5-1 à C-28-5-34 au
registre `epic-28-choix-autonomes.md`. Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus »,
§ « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses
axes exercés ne compte pas », § *Pattern batch* (la route est **unitaire**, pas un lot), § « Migration breaking
policy » (aucune migration attendue). Issue : #526 (la PR de la 28-5a la **référence** sans la fermer — `refs #526` ;
c'est la 28-5b qui la fermera).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

**Contexte** : story **serveur seul** (aucun fichier frontend, C-28-5-26). `kesh-api` : `routes/reconciliation.rs`
— section neuve « Story 28-5 (#526) » (`bank_account_of_transaction`, `UnrejectOutcome`, `classify_unreject`,
`post_unreject_transaction`, `unreject_once`), `cancel_reconciliation_once` (sa lecture du compte remplacée par
l'auxiliaire), `conclude_locked_attempt` rendu générique, en-tête du module, `mod unreject_classification_tests` (B8)
et `batch_classification_tests::tx` passé `pub(super)` (C-28-5-31) ; `routes/bank_imports.rs`
(`TransactionResponse.auto_match_rejected_at`) ; `lib.rs` (route dans `comptable_routes`) ; `errors.rs` (variante
`ReconciliationTransactionNotRejected`, `409`) ; `audit_labels.rs` (`reconciliation.unrejected`). Tests :
`tests/reconciliation_e2e.rs` (section neuve en fin de fichier, B1-B7, B9, B10 et leurs auxiliaires `*_285`),
`tests/audit_route_registry.rs` (ligne, compteurs, doc, point (iv)), `tests/textes_coherents.rs` (G9). `kesh-db` :
doc-comments seulement (`repositories/reconciliation.rs`, `repositories/bank_imports.rs`). `kesh-i18n` : deux clés ×
quatre catalogues. `docs/api-external.md`.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :

1. **Séquence et verrous.** Lecture du compte hors verrou (scopée société, autocommit), puis `with_account_lock`,
   puis lecture **verrouillante** `find_by_id_for_account_for_update`, classement, `UPDATE`, audit, puis
   `conclude_locked_attempt`. Un refus (`NotFound`, `NotRejected`) n'écrit-il vraiment rien (le `commit` vaut un
   `rollback`) ? L'`UPDATE` ne change-t-il que `auto_match_rejected_at`, `updated_at` et `version` ? Le verrou nommé
   relâché avant le `commit` ouvre-t-il une fenêtre nuisible ? L'argument d'interblocage (Dev Notes, C-28-5-30 : seul
   cycle, la remise à zéro d'onboarding) tient-il au code ? Note : la garde « compte clos » de la 28-4 ne prendra
   aucun verrou (C-28-4-21, C-28-5-34) — elle n'est pas dans ce diff.
2. **Une seule définition de « rapprochée ».** `classify_unreject` appelle-t-elle `is_reconciled` sans second test
   de statut ? Priorité des bras (absente, rapprochée, non marquée, rejetée) ? Un état forgé (rapprochée **et**
   marquée) laisse-t-il le marqueur intact ?
3. **`conclude_locked_attempt` générique.** Logique et `match` inchangés pour `post_manual` et `post_split` ? Le
   `DbError::Sqlx` préservé pour leur rejeu ? Les commentaires internes et le doc-comment disent-ils vrai pour les
   trois appelants ? La lecture de `…/cancel` est-elle déplacée **mot pour mot** (SQL, conversions d'erreur), et son
   propre `match` est-il intouché ?
4. **Codes et anti-énumération.** `404 NOT_FOUND` identique pour « inexistante », « autre société » et « supprimée
   sous verrou » ? `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` avec `details.bankTransactionId` ; `409
   OPTIMISTIC_LOCK_CONFLICT` produit par le mapping des erreurs de base (`errors.rs`, bras `DbError::OptimisticLockConflict`),
   jamais un `204` muet ? `403` pour la Consultation (RBAC de `comptable_routes`) ?
5. **Audit.** Une ligne `reconciliation.unrejected`, `entity_type = 'bank_transaction'`, `for_actor(user_id,
   actor_api_key_id, …)`, `details = { previouslyRejectedAt }` — la **même chaîne** que `autoMatchRejectedAt`
   (`.and_utc()` des deux côtés) ? Aucune trace sur refus ?
6. **Inventaire des sites non résolus** (Dev Notes, D4-ter) : est-il encore **clos** au `822ba587` ? Méthode :
   `grep -rn "auto_match_rejected_at" crates --include=*.rs --include=*.sql | grep -v /tests/` — chaque site neuf
   (la section neuve, `bank_imports.rs`) est-il au tableau ou résolu par lui ?
7. **Gardes et décomptes.** `audit_route_registry.rs` (116, 110, 92, 119 ; doc l. 27, 31-32 ; libellés) ;
   `ACTIONS` à sa place alphabétique ; G9 ; les deux clés dans les **quatre** catalogues, repli Rust **égal mot pour
   mot** au fr-CH (`grep -nF "n'est pas rejetée" crates/kesh-api/src/errors.rs crates/kesh-i18n/locales/fr-CH/messages.ftl`),
   terminologie du glossaire (`docs/i18n-glossaire.md`). **Recompte depuis la source** les décomptes du Dev Agent
   Record (13 tests neufs : `git show <ref>:crates/kesh-api/tests/reconciliation_e2e.rs | grep -c '#\[sqlx::test'`
   et `git show <ref>:crates/kesh-api/src/routes/reconciliation.rs | grep -c '#\[test\]'` aux **deux** bornes
   `da25bf1b` et `822ba587` ; 17 fichiers au `git diff --stat`) et vérifie le **périmètre** déclaré.
8. ⛔ **LA DOCUMENTATION — `docs/api-external.md`, ET LE MANUEL.** La sous-section « Annuler le rejet d'une
   transaction bancaire », le renvoi du paragraphe « Transaction disparue, rapprochée ou rejetée » et la ligne
   `GET /api/v1/bank-imports/{id}` disent-ils **vrai au code** (statuts, codes, rôles, clés, limite de 100 de
   `…/proposals`, `autoMatchRejectedAt` dans les trois réponses qui portent `TransactionResponse`) ? « *depuis la
   v0.13.3* », jamais « Unreleased » ni « à venir ». **Le manuel utilisateur n'est volontairement pas touché par la
   28-5a** (il va à la 28-5b) : cherche dans `docs/manual/fr/user-manual.tex` **et son PDF** une phrase que la 28-5a
   rendrait **fausse** dès sa fusion (ex. : « aucun geste de l'interface ne la remet ensuite dans la liste » parle de
   l'**interface** — reste-t-elle vraie tant que la 28-5b n'est pas fusionnée ? Une phrase sur l'**API** qui dirait
   qu'aucune route ne défait un rejet ?), ainsi que dans `docs/manual/fr/admin-manual.tex`, le README et `website/`.
   Contrôle le **PDF aplati et normalisé** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`
   vers `/home/gcorbaz/devel/kesh-gate-logs/28-5a-review-p1-<lettre>-user-manual-flat.txt`.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les chemins d'erreur de
  `unreject_once` (lecture du compte qui échoue, `begin` qui échoue, `GET_LOCK` qui rend `NULL`, `RELEASE_LOCK` qui
  échoue, `commit` qui échoue après une écriture) ; la conversion `map_err(ReconciliationError::Db)` ; le `?` de
  `audit_log::insert_in_tx` (quelle variante, quel statut HTTP ?) ; une transaction dont le compte bancaire change
  entre la lecture hors verrou et le verrou (est-ce possible ?) ; un identifiant non positif ou démesuré dans le
  chemin ; un corps envoyé quand même ; la sérialisation de `Option<DateTime<Utc>>` (millisecondes, `Z`) ; l'effet du
  champ neuf sur les **autres** lecteurs de `TransactionResponse` (détail d'import, `GET …/transactions/{id}`,
  `…/cancel`).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement M1-M8, M13-M15
  et M17 (journal `kesh-gate-logs/28-5a/mutations-run.txt`, sorties dans `kesh-gate-logs/28-5a/mutations/`) et cherche
  une **mutation plausible qui resterait verte** (ex. : `company_id` retiré de la lecture hors verrou ; le bras
  « rapprochée » écrit `status != 'pending'` seul ; `previouslyRejectedAt` lu **après** l'`UPDATE` ; l'audit écrit
  hors de la fermeture ; un `commit` au lieu d'un `rollback` sur erreur ; `NO_CONTENT` remplacé par `OK`). Les
  équivalences déclarées (M13, M14, M17) sont-elles vraies ? B10 : la sonde (`["bank_transactions", "bank_account_id
  = ?"]`) peut-elle se satisfaire d'une autre requête que la lecture verrouillante ou l'`UPDATE` (le `SELECT … WHERE
  id = ? AND company_id = ?` hors verrou, le `GET_LOCK`, une requête du montage) ? Le test passe-t-il à vide si la
  tâche finit avant la sonde ? Les montages construisent-ils l'état qu'ils croient (B1 : `updated_at` forcé par
  affectation explicite, `version0` relu après le forçage ; B3 : état forgé ; B4 : comptes avant/après) ? Indépendance
  et rejouabilité des tests (noms d'utilisateurs, IBAN, hash).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie a (AC1-AC4, AC7 (a),
  AC9 (a), AC10, AC12 (a), AC13) contre le code : satisfait, partiel, absent. Les tests nommés par la fiche (B1-B10)
  existent-ils sous **ces noms** et prouvent-ils ce qu'ils nomment ? Les régions touchées sont-elles celles du
  tableau *Régions touchées* (et la seule hors tableau, `batch_classification_tests::tx`, est-elle justifiée par
  C-28-5-31) ? « Ce qui doit rester intact » l'est-il (`post_reject`, `post_accept`, `classify_batch_transaction`,
  `get_proposals` : aucune ligne changée — `git diff da25bf1b 822ba587 -- crates/kesh-api/src/routes/reconciliation.rs`) ?
  Aucun fichier de `migrations/`, de `test-schema/`, ni `post_restore.rs`, ni `Cargo.toml`, ni de `frontend/` touché
  (`git diff --stat`). Les chiffres de gates déclarés au Dev Agent Record correspondent-ils aux journaux
  (`kesh-gate-logs/28-5a/final-*.log`, `final-gate.out`, `red.log`, `green1.log`) ? Les choix C-28-5-31 à 34 :
  justifiés, ou défaut ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-5a-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo`
(aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3`, `perl`, `ruby`, `php`,
`sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget` (aucune requête
HTTP), `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`,
`sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
