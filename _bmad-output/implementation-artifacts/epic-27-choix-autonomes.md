# Epic 27 — registre des choix autonomes

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.*

---

## Story 27-1 — Supprimer un import dont aucune transaction n'est rapprochée (#632)

Fiche : `27-1-supprimer-import-non-rapproche.md`. Choix faits à la création de la fiche (2026-10-12),
en autonomie, sur `aadace40` (kickoff de l'Epic 27).

### C-27-1-1 — « Rapprochée » : une définition fermée sur les deux colonnes ; la rejetée reste supprimable

- **Contexte** : l'epic demande de trancher « rapprochée » et le sort d'une transaction **rejetée**.
  Inventaire au sol (fiche, Dev Notes § Inventaire (b)) : sept écrivains de `bank_transactions` en
  production ; cinq posent `status = 'reconciled'` **et** `matched_entry_id` ensemble, l'annulation de
  rapprochement remet les deux à `pending`/`NULL`, le rejet pose seulement `auto_match_rejected_at`
  (statut `pending`, aucun lien). Mais `matched_entry_id` est en `ON DELETE SET NULL` : un état
  `reconciled` sans lien est concevable (la garde de `journal_entries::delete` l'empêche aujourd'hui,
  elle ne l'a pas toujours fait), et l'inverse (`pending` avec lien) ne naît d'aucun chemin connu.
- **Retenu** : une transaction est **rapprochée** si `status <> 'pending'` **ou** `matched_entry_id IS NOT NULL`
  — la même disjonction côté serveur (garde) et côté écran (bouton). Une transaction **rejetée**
  (`auto_match_rejected_at` posé, `pending`, sans lien) **n'est pas rapprochée** : elle ne porte aucune
  écriture, et la supprimer avec son import est précisément ce qui permet de la réimporter « neuve » — le
  manuel disait jusqu'ici qu'aucun geste ne la remettait en comptabilité.
- **Écartées** : `status = 'reconciled'` seul (manque un lien orphelin hypothétique) ; refuser aussi les
  rejetées (rendrait l'import d'un relevé dont une ligne a été écartée indélébile, sans raison comptable).
- **Réversible** : oui (une condition).

### C-27-1-2 — Concurrence : verrous de ligne dans l'ordre import → transactions, rejeu sur interblocage ; pas de verrou nommé

- **Contexte** : la suppression peut croiser une acceptation de réconciliation (`accept`, `manual`, `split`),
  un rejet, une annulation de rapprochement. Les flux de réconciliation prennent un verrou nommé
  `GET_LOCK('reconcile:…:{compte}')` **relâché avant leur commit** (`kesh-reconciliation/src/mutex.rs`), et
  ce qui les protège réellement est leur `UPDATE … WHERE status = 'pending' AND version = ?` (verrou de ligne
  tenu jusqu'au commit, `rows_affected` contrôlé). Doctrine du dépôt : C54 de l'Epic 15 (l'ordre réduit la
  fréquence, le rejeu rend l'interblocage invisible) et règle de la 15-8a (le premier acte est le
  `FOR UPDATE`, aucune lecture ordinaire avant).
- **Retenu** : dans une transaction, **premier acte** `SELECT … FROM bank_imports WHERE id = ? AND company_id = ?
  FOR UPDATE`, puis `SELECT … FROM bank_transactions WHERE company_id = ? AND import_id = ? ORDER BY id FOR UPDATE`,
  décision prise **sur le résultat de cette lecture verrouillante**, puis `DELETE` de l'entête. La route est
  enveloppée par `retry_app_on_deadlock("bank_imports::delete", …)` (fonction « une tentative »), parce qu'un
  lot d'acceptation qui verrouille plusieurs transactions du même import dans l'ordre des propositions peut
  former un cycle avec le balayage par identifiant. Classement au registre des routes :
  `SansEcritureAuJournal` (elle n'écrit pas au journal), **septième** route de ce statut rejouée quand même
  (point (vi) d'`audit_route_registry.rs`).
- **Écartées** : prendre aussi `with_account_lock` (n'ajoute rien à la correction, puisqu'il est relâché
  avant le commit, et ajouterait un `409 RECONCILIATION_ACCOUNT_LOCKED` de plus à expliquer) ; un compteur
  `version` sur `bank_imports` (exigerait une migration, pour une garde que l'état des lignes verrouillées
  donne déjà).
- **Réversible** : oui.

### C-27-1-3 — La route : `DELETE /api/v1/bank-imports/{id}`, Comptable+, ouverte aux clés `read-write`, 204 / 404 / 409 — (révisé : fermée aux clés, voir C-27-1-16)

- **Contexte** : `POST /api/v1/bank-imports` vit dans `comptable_routes` (`lib.rs:629-633`), donc
  Comptable et Admin, et ouverte aux clés d'API `read-write` (la seule route mutante fermée aux clés hors
  administration est la gestion des clés — *inexact, relevé en validation P3 (R3-10) : la consultation du journal
  d'audit l'était déjà, `routes/audit_log.rs:6-14`*). Précédent de forme : `DELETE /api/v1/bank-profiles/{id}` rend
  `204`. Le détail d'un import d'une autre société rend `404 NOT_FOUND` (`DbError::NotFound`).
- **Retenu** : montée dans `comptable_routes` à côté du `POST` ; `204 No Content` ; import inconnu ou d'une
  autre société → `404 NOT_FOUND` (anti-énumération, KF-002) ; une transaction rapprochée →
  **`409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`**, `details: { reconciledCount, transactionCount }`, message
  localisé qui nomme le nombre. Ouverte aux clés `read-write` comme le `POST` (la clé est nommée à l'audit,
  `for_actor`) ; une clé `read` reçoit `403 API_KEY_READ_ONLY` par le middleware. Documentée dans
  `docs/api-external.md`.
- **Écartées** : `422` (les refus `BankImport*` de l'import sont en `422`, mais ce sont des refus de
  **contenu** du fichier ; ici, c'est l'**état** de la ressource qui s'oppose — `409`, comme
  `INVOICE_MUST_BE_UNVALIDATED_FIRST`) ; route d'administration (fermerait la suppression aux clés et au
  Comptable qui a importé, sans raison : rien de comptable n'est effacé).
- **Réversible** : oui avant publication ; après, le code et le statut font partie du contrat d'API.

### C-27-1-4 — Les transactions partent par la clé étrangère `ON DELETE CASCADE`

- **Contexte** : `fk_bank_transactions_import … ON DELETE CASCADE` (migration `20260504000001`, l. 78-79 ;
  squash l. 232). Aucune autre table ne référence `bank_transactions` ni `bank_imports` (inventaire (a)).
- **Retenu** : un seul `DELETE FROM bank_imports WHERE id = ? AND company_id = ?`, `rows_affected` contrôlé
  (≠ 1 → `NotFound`) ; les transactions, déjà verrouillées, partent par la cascade. Le test vérifie qu'il
  n'en reste aucune.
- **Écartée** : un `DELETE FROM bank_transactions` explicite d'abord — redondant avec la cascade, et sa
  suppression serait une mutation équivalente qu'aucun test ne peut faire rougir.
- **Réversible** : oui.

### C-27-1-5 — Trace `bank_import.deleted` sur l'entité `bank_imports`, dans la transaction de la suppression

- **Retenu** : action **`bank_import.deleted`**, type d'entité `bank_imports` (code réel existant, au pluriel),
  `entity_id` = l'identifiant de l'import supprimé (pointeur logique : `audit_log.entity_id` n'est pas une
  clé étrangère et « survit aux DELETE », squash l. 97). `details_json` : `filename`, `file_hash`,
  `source_format`, `bank_account_id`, `period_from`, `period_to`, `transaction_count` (le nombre de lignes
  **effectivement supprimées**, compté sous verrou, et non la colonne `bank_imports.transaction_count` :
  c'est ce qui a été effacé qui se trace), `rejected_count`
  (transactions rejetées emportées). Écrite avec `NewAuditLogEntry::for_actor` dans la transaction, avant le
  commit ; aucune trace sur un refus. Libellé dans les quatre catalogues.
- **Écartée** : réutiliser `bank_import.created` avec un modificateur (illisible au journal).
- **Réversible** : oui avant publication.

### C-27-1-6 — L'écran : un composant de la feature, bouton désactivé avec son motif, dialogue, retour à la liste

- **Retenu** : composant neuf `features/bank-import/DeleteBankImportAction.svelte` (bouton + dialogue),
  inséré dans le détail d'un import. Masqué pour le rôle Consultation (même règle que le bouton *Annuler le
  rapprochement* de la même page). Si une transaction est rapprochée : bouton **visible et désactivé**, avec
  la phrase qui dit combien et comment lever le blocage (annuler leur rapprochement, bouton de la ligne).
  Dialogue : nomme le fichier, la période, le nombre de transactions, dit que le même fichier se réimporte
  ensuite et que la suppression ne s'annule pas. Après succès : notification puis retour à
  `/bank-import`. Le prédicat est un helper TS pur, testé, qui reprend C-27-1-1.
- **Écartées** : bouton masqué quand l'import est rapproché (l'utilisateur ne saurait pas que la fonction
  existe ni pourquoi elle lui est refusée) ; action dans la liste *Imports précédents* (pas d'information sur
  les transactions à cet endroit).
- **Réversible** : oui.

### C-27-1-7 — Un import d'un compte bancaire archivé se supprime aussi ; le manuel dit qu'il ne se réimporte pas

- **Contexte** : `POST /bank-imports` refuse un compte archivé (`404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`,
  `bank_imports.rs:1063-1066`) et aucune route ne réactive un compte bancaire archivé.
- **Retenu** : la suppression ne regarde pas l'archivage (rien de comptable n'est effacé) ; le manuel dit
  qu'un relevé d'un compte archivé ne se réimporte plus.
- **Écartée** : refuser la suppression sur un compte archivé (une règle de plus pour un cas marginal).
- **Réversible** : oui.

### C-27-1-8 — Aucune migration

- **Retenu** : aucune. L'unicité `uq_bank_imports_company_hash` invoquée par l'epic et l'issue **n'existe plus**
  depuis la migration `20260507000001_bank_imports_relax_hash_unique.sql` (Story 8-3) : le blocage du réimport
  est applicatif (`find_by_company_and_hash`, refus `422 BANK_IMPORT_DUPLICATE_FILE` sans
  `confirmDuplicateFile`). Supprimer l'entête suffit à le lever. La fiche corrige ce point de l'epic sans
  toucher le fichier d'epic.
- **Réversible** : sans objet.

### C-27-1-9 — Pas de découpage préventif ; découpage de repli nommé

- **Contexte** : modules touchés — `kesh-db` (dépôt `bank_imports`), `kesh-api` (route, erreurs, montage,
  libellés d'audit, registres de test), `kesh-i18n` (4 catalogues), frontend (`features/bank-import`, page
  `bank-import/[id]`), documentation (manuel, `api-external.md`, CHANGELOG, README). Cinq ensembles au sens du
  CLAUDE.md, un seul domaine métier.
- **Retenu** : une story ; dérogation écrite à la fiche. Si la validation montre une non-convergence
  (recyclage d'un défaut), découper en **27-1a** (dépôt, route, erreurs, audit, i18n serveur, tests Rust,
  `api-external.md` — `refs #632`) et **27-1b** (écran, i18n frontend, Vitest, E2E, manuel, CHANGELOG,
  README — `closes #632`).
- **Réversible** : oui.

### C-27-1-10 — E2E : une fixture CSV synthétique neuve et son profil créé par l'API

- **Retenu** : `frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv` (trois lignes inventées, dates de 2026
  propres à la spec) et un profil créé par `POST /api/v1/bank-profiles` au motif `^csv_e2e_delete_import\.csv$` ;
  spec neuve `bank-import-delete.spec.ts`. Un fichier propre à la spec garantit que l'aperçu du réimport n'a
  aucun avertissement « déjà importé » venant d'une autre spec.
- **Écartée** : réutiliser `csv_utf8_minimal.csv` ou `camt053_v04_minimal.xml` (importés par d'autres specs
  sur la même base : l'avertissement de doublon dépendrait de l'ordre d'exécution, KF-030).
- **Réversible** : oui.

### C-27-1-11 — CHANGELOG sous `## [0.13.2] — Non publié`

- **Retenu** : la section est créée par la story (forme attendue par `scripts/prepare-release.sh:237`), entrée
  sous `### Ajouté`. La ligne de feuille de route v0.13.2 du README reste à la release (orchestrateur) ; la
  story met à jour la ligne *Import bancaire* de la section *Fonctionnalités*.
- **Réversible** : oui.

### Révision de C-27-1-7 (remédiation P1, 2026-10-12)

- **Constat** (revue P1, R3/F6) : la prémisse est quasi vide. Un compte bancaire ne s'archive pas tant qu'il porte
  **une** transaction (`412 BANK_ACCOUNT_HAS_TRANSACTIONS`) : un compte archivé n'a donc que des imports à zéro
  transaction, ou des données antérieures à la garde. Le cas réel est l'inverse — supprimer rend archivable.
- **Retenu** : C-27-1-7 est **remplacé par C-27-1-12** ; la phrase du manuel sur le compte archivé est retirée de
  l'AC13. La suppression continue de ne pas regarder l'archivage (rien ne change au code).

### C-27-1-12 — La suppression lève la garde d'archivage d'un compte bancaire, et c'est accepté (arbitrage de l'orchestrateur)

- **Contexte** : `count_transactions_for_bank_account` (`repositories/bank_accounts.rs:729-751`, « comptage
  inconditionnel (auditabilité CO Art. 958f) ») fait refuser l'archivage en `412` dès qu'une ligne de
  `bank_transactions` existe. Jusqu'ici aucune ligne ne disparaissait ; la story en supprime.
- **Retenu** : on accepte que supprimer les imports non rapprochés d'un compte le rende archivable. Des
  transactions jamais rapprochées ne sont pas des écritures (aucune pièce au journal) ; la suppression laisse sa
  propre trace `bank_import.deleted` (fichier, empreinte, période, nombres) : l'auditabilité passe par la trace, non
  par la conservation des lignes. AC8 le dit, test H6 (archivage refusé avant, permis après), une phrase au manuel,
  une ligne à `api-external.md`, une phrase au doc-comment de la garde.
- **Écartées** : faire compter à la garde les imports supprimés (exigerait de conserver une trace en table, donc
  une migration) ; refuser la suppression sur un compte dont c'est le dernier import (contredit la décision du
  Project Lead).
- **Réversible** : oui.

### C-27-1-13 — Sauvegarde prise pendant une suppression : angle mort préexistant, renvoyé à #465 (arbitrage de l'orchestrateur)

- **Contexte** (revue P1, F5) : l'export `.keshbackup` lit ses tables une à une, sans instantané commun
  (`admin_backup/export.rs:59-62`) ; `bank_transactions` est lue avant `bank_imports`. Une suppression validée
  entre les deux donne une sauvegarde aux transactions orphelines, que la restauration (sous
  `FOREIGN_KEY_CHECKS = 0`) recréerait.
- **Retenu** : pas de correctif ici ; l'AC8 ne promet plus rien d'une sauvegarde prise **pendant** une
  suppression, et l'angle mort renvoie à [#465](https://github.com/guycorbaz/kesh/issues/465), ouverte pour ce
  défaut général.
- **Réversible** : sans objet.

### C-27-1-14 — Le lot de rejet verrouille par identifiant croissant (traité dans la story)

- **Contexte** (revue P1, F3) : `reject_batch` pose ses `UPDATE` dans l'ordre reçu et `post_reject` n'est pas
  rejouée ; contre la suppression (verrous par identifiant croissant), un lot `[7, 3]` forme un cycle dont le rejet
  serait la victime probable, en `500`.
- **Retenu** : le correctif est borné — **trier** les identifiants avant `reject_batch`
  (`routes/reconciliation.rs:3079`) ; les doublons étant déjà refusés en `400`, l'ensemble traité ne change pas.
  Deux ordres croissants ne forment pas de cycle, et le rejet ne touche pas `bank_imports`. Effets visibles écrits
  (ordre de `rejected[]`/`failed[]`, `entity_id` de la trace = plus petit identifiant rejeté), test H7,
  mutation M17, ligne `### Modifié` au CHANGELOG. AC4-bis.
- **Écartées** : envelopper `post_reject` du rejeu (extraction d'une fonction « une tentative » sous verrou nommé,
  patron `post_manual` : plus lourd, et le rejet resterait la victime d'un cycle qu'on sait fermer) ; écrire
  l'angle mort et ouvrir une issue (le correctif tient en une ligne).
- **Réversible** : oui.

### C-27-1-15 — La preuve de la course : motifs de synchronisation et statut des mutations corrigés

- **Contexte** (revue P1, R1/F1/F2, R2/F9) : le motif `["bank_transactions"]` de l'AC3 (a) ne voyait pas le
  `DELETE FROM bank_imports` bloqué sous M3 et pouvait attraper en vol le `SELECT` muté ; le motif de l'AC3 (c) ne
  voyait pas la seconde suppression bloquée sous M14 ; M6 était équivalente par `rollback`, M1 mal attribuée à D2.
- **Retenu** : (a) `["bank_transactions", "FOR UPDATE"]`, M3 rouge par **absence du motif** (panique à 10 s,
  déterministe), écrit comme tel ; (c) `["FOR UPDATE"]`, M14 et M15 **équivalentes** chacune (l'un ou l'autre
  contrôle suffit à `NotFound`), M14+M15 rouge par l'assertion ; M6 retirée du tableau avec sa raison ; M1 → D4,
  H2 (H2 pose `reconciled` sans lien). Nouvelles mutations M16 (`canManage`), M17 (tri du rejet).
- **Réversible** : oui (fiche).

### C-27-1-16 — La suppression d'un import est fermée aux clés d'API (décision du Project Lead, révise C-27-1-3)

- **Décision** (Guy, 2026-10-12, transmise par l'orchestrateur après la validation P2) : « **les api n'ont pas besoin
  de supprimer un import, du moins pour le moment** ».
- **Retenu** : `DELETE /api/v1/bank-imports/{id}` refuse **toute** clé d'API. Une clé `read` est arrêtée par le
  middleware (`403 API_KEY_READ_ONLY`, inchangé) ; une clé `read-write` atteint le handler, dont la première
  instruction — avant l'enveloppe de rejeu — est `if current_user.api_key_id.is_some()` ⇒ `403`, sans suppression
  ni trace. **Code neuf** `API_KEY_OPERATION_FORBIDDEN` (variante `AppError::ApiKeyOperationForbidden`, message
  « Cette opération n'est pas accessible via une clé API. Utilisez l'interface web. », quatre catalogues).
- **Pourquoi un code neuf** : la doctrine D2 de la Story 22-4a (`errors.rs:208-210`) interdit de réutiliser un code
  dont le message mentirait : `API_KEY_MANAGEMENT_FORBIDDEN` parle de gestion de clés (sa réutilisation pour le
  journal d'audit est documentée comme un héritage, `api-external.md` § 4), `API_KEY_ADMIN_FORBIDDEN` de routes
  d'administration — la route est Comptable. Le libellé neuf est générique (« cette opération »), réutilisable pour
  une prochaine route fermée aux clés sans mentir. Pas de `require_not_pat` : `admin_pat_denied_e2e` en interdit un
  second consommateur.
- **Conséquences** : `api-external.md` décrit la route comme **non ouverte aux clés** (§ 4, § 7, § 10) ; le
  paragraphe *Interblocages* (l. 662), qui n'énumère que les routes ouvertes aux clés, reste inchangé (le correctif
  P1 F7 est retiré) ; test H8 (clé `read-write` ⇒ `403`, rien supprimé, aucune trace ; clé `read` ⇒ `403
  API_KEY_READ_ONLY`) ; mutation M18. La route reste Comptable+ pour une session.
- **Réversible** : oui — ouvrir la route aux clés revient à retirer le contrôle et à documenter.

### Révision de C-27-1-3 (validation P2, 2026-10-12)

- La puce « ouverte aux clés `read-write` comme le `POST` ; une clé `read` reçoit `403 API_KEY_READ_ONLY` ; la trace
  nomme la clé » est **remplacée par C-27-1-16**. Le reste de C-27-1-3 (montage dans `comptable_routes`, `204`,
  `404 NOT_FOUND`, `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`) tient.

### Révision de C-27-1-13 (validation P2, 2026-10-12)

- **Constat** (revue P2, R2-1) : #465 suit l'export de souveraineté CSV (`exports/global.rs`), pas la sauvegarde
  `.keshbackup` (`admin_backup/export.rs`, `backup.rs::export_table`). L'orchestrateur a ouvert
  [#633](https://github.com/guycorbaz/kesh/issues/633) (« La sauvegarde .keshbackup lit ses tables sans instantané
  commun — une écriture concurrente peut y laisser des lignes orphelines »).
- **Retenu** : la fiche renvoie à **#633** pour la sauvegarde et garde **#465** pour l'export global CSV, qui a le
  même défaut.

### C-27-1-17 — Un lot de réconciliation préparé avant une suppression est refusé en entier : écrit, renvoyé à #634 (arbitrage de l'orchestrateur)

- **Contexte** (revue P2, F2-1, R2-5) : le pré-vol `find_pending_by_ids` de `post_accept` et `post_reject`
  (`reconciliation.rs:950-963`, `:3049-3064`), lu sur le pool, refuse le lot **entier** en `400 VALIDATION_ERROR`
  dès qu'une transaction cochée n'existe plus — le cas courant après une suppression, page *Réconciliation* restée
  ouverte. La fiche disait « `failed[]` », vrai seulement dans la fenêtre pré-vol → `UPDATE`.
- **Retenu** : pas de correction de code dans la story ; l'orchestrateur a ouvert
  [#634](https://github.com/guycorbaz/kesh/issues/634). AC3 (b) réécrit (deux trajectoires), puce à l'AC8, angle
  mort, phrase au manuel (recharger la page *Réconciliation* après une suppression), renvoi #634.
- **Réversible** : sans objet.

### C-27-1-18 — Pas de découpage après la P2 (décision de l'orchestrateur)

- **Contexte** (revue P2, R2-7, F2-8) : au grain des « modules métier de premier niveau » du `CLAUDE.md`, la story en
  touche plus de cinq depuis l'AC4-bis ; au grain des crates, cinq.
- **Retenu** : pas de découpage. Les défauts de la P2 sont distincts de ceux de la P1 et aucun n'est un recyclage
  (amendement D5 du `CLAUDE.md`, 2026-10-06) ; la sévérité baisse (7 → 4 MEDIUM distincts, 0 HIGH) ; le domaine
  reste unique. Signal déclaré au Project Lead par l'orchestrateur. Repli 27-1a/27-1b de C-27-1-9 inchangé.
- **Réversible** : oui.

### C-27-1-19 — Le message du refus d'archivage nomme le remède (revue P2, R2-10)

- **Retenu** : une phrase ajoutée au message `bank-accounts-errors-has-transactions` (quatre catalogues et repli
  `errors.rs:2467-2470`) : « Les imports dont aucune transaction n'est rapprochée peuvent être supprimés (Quotidien →
  Importer des relevés). » — vraie dans tous les cas (elle ne promet pas l'archivage), sans toucher au code
  d'erreur `BANK_ACCOUNT_HAS_TRANSACTIONS`. L'arbitrage de fond de C-27-1-12 (CO 958f) est signalé à Guy par
  l'orchestrateur.
- **Réversible** : oui.

### Complément à C-27-1-16 — le précédent contraire d'`audit_log.rs` (validation P3, R3-9/F3-6)

- `routes/audit_log.rs:6-14` est le seul précédent d'une route **Comptable** fermée aux clés, et il a fait le choix
  inverse : `ensure_not_pat` en première instruction, code `API_KEY_MANAGEMENT_FORBIDDEN` **réutilisé** (« un variant
  de plus pour un libellé serait du bruit »). C-27-1-16 garde le code neuf : ce précédent documente lui-même, dans
  `api-external.md` § 4, que son libellé « parle de gestion de clés pour des raisons historiques » — c'est la
  confusion que D2 veut éviter, et le libellé générique neuf servira la prochaine route fermée sans mentir.
  L'alternative écartée est donc ce précédent. Comme celui de D2 (`loader.rs:663`), le message neuf est tenu par un
  test qui l'exige distinct de ses deux voisins (mutation M19).

### C-27-1-20 — L'angle mort « restauration contre suppression » réduit à sa phrase minimale (arbitrage de l'orchestrateur, validation P3)

- **Contexte** (revue P3, R3-1/F3-1) : écrit en P1, réécrit en P1 (F4) et en P2 (F2-3/R2-9), cet angle mort était
  faux en P3 — la remédiation P2 faisait vider `audit_log` par la restauration, que `backup.rs:493` saute. Un défaut
  né du correctif précédent, sur le même texte : critère de **recyclage** (amendement D5).
- **Retenu** : sortie par **retrait**, non par une quatrième réécriture. La fiche et le doc-comment prescrit ne
  disent plus que : la restauration supprime les lignes de `bank_transactions` puis celles de `bank_imports`
  (`backup.rs:492-500`, ordre des l. 67-68), la suppression verrouille l'entête puis les lignes ; un interblocage est
  possible, rejoué côté suppression par `retry_app_on_deadlock` ; la restauration n'étant exclusive que des autres
  imports (15-12b), une suppression concurrente d'une restauration n'est pas gardée au-delà. Aucune mécanique interne
  de `backup.rs` au-delà de ce que ces lignes disent. La ligne du bilan P2 de la fiche porte « corrigé en P3 ».
- **Écartée** : une quatrième réécriture détaillée (victime, attente, installation restaurée) — chaque réécriture a
  introduit une erreur, et aucune n'était exigée par une garde ni par le code.
- **Réversible** : oui. Le constat est déclaré au Project Lead par l'orchestrateur.

### C-27-1-21 — `FORCE INDEX (idx_bank_transactions_import)` sur la lecture verrouillante (dev, F10)

- **Contexte** : l'AC2 demande de constater le plan et de forcer l'index si l'optimiseur en choisit un autre.
  `EXPLAIN` sur `kesh_271` (MariaDB 10.11, 500 transactions dont 450 pour un import, `ANALYZE TABLE` fait) : pour
  l'import minoritaire, `key = idx_bank_transactions_import` (`ref`, 50 lignes) ; pour l'import **majoritaire**,
  `key = PRIMARY` (`type = index`, 500 lignes) — un balayage de l'index primaire qui, sous `FOR UPDATE`, verrouillerait
  toutes les lignes examinées, toutes sociétés confondues. Avec `FORCE INDEX`, `ref` sur
  `idx_bank_transactions_import` dans les deux cas. Sortie : `kesh-gate-logs/27-1-explain.log`.
- **Retenu** : `FORCE INDEX (idx_bank_transactions_import)` dans la requête de l'étape 2.
- **Écartée** : se fier à l'optimiseur — le plan dépend de la distribution, et c'est le cas le plus fréquent (un
  import qui porte l'essentiel des lignes d'une petite installation) qui le fait basculer.
- **Réversible** : oui (un mot-clé). Le motif de synchronisation de l'AC3 (a) (`bank_transactions`, `FOR UPDATE`)
  reste porté par le texte.

### C-27-1-22 — Second « dernier relevé » du manuel corrigé ; commentaires frontend laissés (dev, grep du symptôme)

- **Contexte** : l'AC13 prescrit de corriger `user-manual.tex:250-251`. Le grep du symptôme (`dernier relevé`) rend
  aussi `user-manual.tex:274` (« la tuile affiche tout de même son dernier relevé ») et quatre commentaires ou noms
  de test du frontend (`BankAccountsCard.svelte:4`, `bank-accounts.api.ts:10, 45`, `bank-accounts.api.test.ts:91`).
- **Retenu** : la phrase du manuel l. 274 suit l'AC13 (« son relevé de plus grande période »), PDF compris ; les
  commentaires du frontend, hors du périmètre de la fiche et sans effet visible, ne sont pas touchés.
- **Écartée** : réécrire les commentaires frontend — un diff hors périmètre, dans des fichiers que la story ne touche
  pas.
- **Réversible** : oui.

### Complément à C-27-1-22 — les commentaires frontend corrigés à la revue de code P1 (E7)

- La lentille E (P1) a relevé que ces commentaires contredisaient désormais le manuel. Ils disent « relevé de plus
  grande période » (`bank-accounts.api.ts:10, 45`, `BankAccountsCard.svelte:4`, nom du `describe` de
  `bank-accounts.api.test.ts`), de même que le doc-comment du champ dans `routes/bank_accounts.rs`. Commentaires
  seulement, aucun comportement touché.

### C-27-1-23 — Le plan de l'étape 2 gardé par `EXPLAIN` sur la constante exécutée, avec un témoin (revue de code P1, E1)

- **Contexte** : retirer le `FORCE INDEX` de C-27-1-21 ne faisait rougir aucun test (E1, MEDIUM) — le résultat de la
  suppression ne dépend pas du plan, seule la portée des verrous en dépend.
- **Retenu** : le SQL de l'étape 2 devient `bank_imports::LOCK_IMPORT_TRANSACTIONS_SQL` (`#[doc(hidden)] pub`), exécuté
  par `delete_unreconciled_in_tx` et passé à `EXPLAIN` par `lock_query_uses_the_import_index`
  (`kesh-db/tests/bank_import_delete.rs`) sur la forme de `27-1-explain.log` : 500 lignes, un import de 450 et un de
  50, `ANALYZE TABLE`. Asserté : `key = idx_bank_transactions_import`, `type = ref`. **Témoin** : la même requête
  privée de l'indication doit, sur ce montage, ne pas passer par l'index de l'import — sinon le test ne prouverait
  plus rien et le dit. Mutation M21 (indication retirée) : rouge (`PRIMARY`, `type index`).
- **Écartées** : un test de portée de verrou (une ligne d'un autre import, `NOWAIT`) — il prouve l'effet plutôt que le
  plan, mais dépend du verrou de trou et de la distribution d'`id` entre imports, plus fragile ; un angle mort
  assumé — la voie `EXPLAIN` était praticable.
- **Risque assumé** : le témoin rougira si une autre version de MariaDB choisit d'elle-même le bon index ; c'est le
  signal voulu (l'indication serait alors à réexaminer), pas un faux positif silencieux.
- **Réversible** : oui.

### C-27-1-24 — AC4-bis : l'ordre des verrous tenu par une sonde `NOWAIT` ; `failed[]` non asserté (revue de code P1, E5)

- **Contexte** : H7 lit les sorties triées ; une variante « ordre reçu, sorties triées après coup » y restait verte.
  E5 proposait aussi d'asserter l'ordre de `failed[]`.
- **Retenu** : H7-bis (`post_reject_locks_the_batch_by_increasing_id`) — une transaction de test tient `lo`, le lot
  `[hi, lo]` part en tâche, et pendant son attente une sonde `FOR UPDATE NOWAIT` sur `hi` doit réussir. Mutation M26
  (tri retiré, `rejected[]` trié avant la trace) : H7-bis rouge, H7 vert — ce que le finding annonçait.
  `failed[]` n'est **pas** asserté : le pré-vol `find_pending_by_ids` (filtre `status = 'pending'` et
  `auto_match_rejected_at IS NULL`) refuse en `400` tout lot portant une ligne non éligible — comportement déjà
  figé par `post_reject_after_accept_returns_already_reconciled_failed` —, si bien que `failed[]` n'est atteignable
  que par une course entre ce pré-vol et le verrou du compte ; son ordre est celui de la boucle, que H7-bis tient.
- **Écartée** : monter cette course pour lire `failed[]` — deux sondes de plus pour une sortie dont l'ordre découle
  déjà de ce qui est testé.
- **Réversible** : oui.

### C-27-1-25 — Le message du 409 lu par le catalogue fr-CH, langue globale posée dans H2 (revue de code P1, E4)

- **Contexte** : `bank_imports_e2e` ne chargeait pas la langue des erreurs : H2 lisait le repli Rust.
- **Retenu** : H2 pose `init_error_i18n(fr-CH)` et asserte le texte exact. Catalogue chargé, une clé absente rend la
  clé brute (M24 : rouge) et un `{ $count }` mal nommé un texte sans le nombre. La garde G9 compare désormais au
  catalogue les trois replis de la story (M25 : rouge) ; le doc-comment de H6, qui l'affirmait à tort, est corrigé.
  La langue étant globale au processus, le binaire entier a été rejoué en un seul processus
  (`cargo test --test bank_imports_e2e -- --test-threads=1`) : 39/39.
- **Écartée** : poser `de-CH` pour distinguer catalogue et repli — sous `cargo test`, les autres tests du binaire,
  qui lisent des textes français, changeraient de résultat selon l'ordre.
- **Réversible** : oui.

### C-27-1-26 — Laissés de la revue de code P1 : B-2 et E6 (revue de code P1)

- **B-2** (le 409 de course afficherait « (0) » si `details` manquait) : laissé. Le serveur envoie toujours
  `reconciledCount` (bras `BankImportHasReconciledTransactions` d'`errors.rs`) ; un repli conditionnel exigerait une
  clé de plus dans les quatre catalogues pour un cas inatteignable.
- **E6** (rejeu sur interblocage et `rollback` explicite de `delete_once` sans mutation rouge) : angle mort assumé,
  déjà écrit au doc-comment de la route ; mutations « rejeu retiré » et « rollback retiré » non jouées : vertes par
  construction (aucun test ne provoque de 1213 sur cette route ; le `Drop` de la transaction annule de toute façon).
- **Réversible** : oui.

### C-27-1-27 — Table des matières des PDF : passes supplémentaires, Makefile inchangé (revue de code P1, observation TOC)

- **Contexte** : la TOC de `user-manual.pdf` était décalée (deux passes seulement au `make`).
- **Retenu** : `xelatex` relancé jusqu'à disparition de « Rerun » (admin : 2 passes, utilisateur : 1, sur les `.aux`
  existants), puis contrôle de chaque entrée de la TOC contre le pied de page de sa page (`pdftotext -layout`) :
  95 entrées (utilisateur) et 122 (administration), 0 écart.
- **Écartée** : changer le Makefile (passes ou `latexmk`) — défaut antérieur à la story, hors périmètre ; à signaler
  à l'orchestrateur (une issue P3 vaudrait).
- **Réversible** : oui.

- **Révision (revue de code P2, 2026-10-12)** : l'orchestrateur a demandé de corriger le Makefile dans la story —
  voir C-27-1-30, qui remplace la partie « Écartée » ci-dessus.

### C-27-1-28 — Le dialogue de suppression ne se ferme pas pendant l'envoi, `open` lié (revue de code P2, B2-1)

- **Contexte** : le wrapper `Dialog` tient sa propre copie d'`open` ; le composant la passait à sens unique
  (`{open}`). Fermé par Échap, la croix ou un clic extérieur pendant l'envoi, le dialogue disparaissait, l'erreur
  du `409` s'affichait dans un dialogue invisible, et le bouton ne le rouvrait plus.
- **Retenu** : les deux remèdes ensemble — garde dans `Dialog.Content` (`escapeKeydownBehavior`,
  `interactOutsideBehavior` à `'ignore'`, `showCloseButton={!submitting}`), patron de `WriteOffDialog.svelte`, et
  `bind:open` (un seul état). Deux Vitest : fermeture refusée pendant l'envoi puis erreur visible ; fermé par Échap,
  rouvert au clic suivant. M36, M37, M38 rouges.
- **Écartée** : `bind:open` seul (le dialogue se fermerait encore pendant l'envoi, l'erreur n'aurait plus de
  place) ; garde seule (le `{open}` à sens unique reste un piège pour toute autre fermeture).
- **Angle mort** : `interactOutsideBehavior` n'est exercé par aucun test — un clic extérieur ne se simule pas
  fidèlement sous jsdom ; la garde suit le même opérateur que celle d'Échap, qui est tenue.
- **Réversible** : oui.

### C-27-1-29 — Message du 409 de course : formule neutre « { $count } transaction(s) … ont été », non un sélecteur Fluent (revue de code P2, B2-4)

- **Contexte** : « Une transaction de cet import a été rapprochée entre-temps ({ $count }) » rendait « Une
  transaction … (3) ».
- **Retenu** : la formule à marque de pluriel que portent déjà les deux messages voisins de la story
  (`bank-import-delete-blocked`, `bank-import-errors-has-reconciled-transactions`), dans les quatre locales et au
  repli Svelte.
- **Écartée** : un sélecteur `[one] … *[other]` — aucune clé du dépôt n'en emploie pour ces messages, et le
  repli Svelte (`i18nMsg`) ne l'interprète pas.
- **Réversible** : oui.

### C-27-1-30 — Makefile des manuels : passes jusqu'à stabilisation (revue de code P2, demande de l'orchestrateur)

- **Contexte** : deux passes fixes laissaient des TOC périmées (C-27-1-27). LaTeX ne signale pas une `.toc`
  changée ; « Rerun » ne couvre que certaines références.
- **Retenu** : la règle `%.pdf` relance `xelatex` tant que l'empreinte de `.aux` + `.toc` + `.out` diffère de la
  passe précédente ou que le journal porte `Rerun to get`, `Rerun LaTeX`, `Please rerun` ou `Label(s) may have
  changed` ; au moins 2 passes, au plus 6, échec bruyant au-delà. Le motif nu `Rerun` est écarté : la bannière du
  paquet `rerunfilecheck` le porte à chaque passe (premier essai : « instable après 6 passes »). Contrôle : depuis
  zéro (`make clean`, copie dans le scratchpad), 3 passes pour les deux manuels, `.toc` identiques octet pour octet
  à celles du dépôt ; une passe stable est un point fixe, la TOC imprimée est celle que la passe a recalculée.
- **Écartée** : `latexmk` — non employé ailleurs dans le Makefile, dépendance de plus pour le même résultat.
- **Observation, non traitée** : six titres de section restent orphelins en bas de page (utilisateur : § 3, 5, 6,
  9, 15 ; administration : § 5) ; la TOC pointe vers la page suivante, où commence leur texte. Préexistant
  (identique dans les PDF de `5881450b`), affaire de mise en page et non de TOC.
- **Réversible** : oui.

### C-27-1-31 — Consultation : Vitest de page plutôt qu'E2E (revue de code P2, E-P2-5)

- **Retenu** : `src/routes/(app)/bank-import/[id]/bank-import-detail-page.test.ts` (patron des tests de page du
  dépôt, mocks hoistés) : Admin et Comptable voient le bouton, Consultation non, détail chargé. M39
  (`canManage={true}` passé par la page) rouge.
- **Écartée** : un E2E Consultation — demanderait un second utilisateur au seed E2E pour la même preuve.
- **Réversible** : oui.

### C-27-1-32 — « Supprimer un import » : renvoi depuis la sous-section CSV plutôt que déplacement (revue de code P3, A3-4)

- **Contexte** : la sous-sous-section *Supprimer un import* (`sec:supprimer-import`) est rangée sous *Import
  CAMT.053*, alors qu'elle vaut pour tout import — le cas d'usage de #632 est un CSV mal réglé.
- **Retenu** : le moindre changement — une phrase de renvoi à la fin de *Import avec un profil existant* (« Un
  import CSV déjà confirmé avec un mauvais profil se supprime comme un import CAMT.053 … (§ 13.1.3) »). La
  numérotation et les trois renvois existants restent inchangés. PDF régénéré par `make user` (2 passes, stable),
  contrôlé aplati.
- **Écartée** : déplacer la section sous `\section{Import bancaire}` avant les deux sous-sections — renumérote
  la TOC et les renvois pour un gain de rangement seul.
- **Réversible** : oui.

### C-27-1-33 — Vitest « Annuler » : `click()` natif et non `fireEvent.click` (revue de code P3, E3-2)

- **Contexte** : `fireEvent.click` livre l'événement à l'écouteur Svelte même sur un bouton `disabled` ; le
  dialogue se fermait dans le test alors qu'un navigateur n'active pas un contrôle désactivé.
- **Retenu** : `HTMLElement.click()`, que jsdom n'active pas sur un contrôle désactivé, comme un navigateur ; le
  test asserte `disabled` puis l'absence de fermeture, puis le retour à l'état actif après le `409`. M42
  (`disabled` retiré) rouge.
- **Réversible** : oui.

---

## Story 27-2 — La devise d'un relevé CSV est lue, et le contrôle de devise joue (#624)

Fiche : `27-2-devise-import-csv.md`. Choix faits à la création de la fiche (2026-10-12), en autonomie, sur
`a2743fd4` (`origin/main`, rétrospective de l'Epic 26).

### C-27-2-1 — Le mécanisme : une colonne `currency` facultative dans `ColumnMapping`, sans migration

- **Contexte** : le parseur CSV écrit `"CHF"` en dur, pour le relevé (`parser.rs:364`) et pour chaque transaction
  (`parser.rs:680`). Il faut que le profil puisse **dire où lire la devise**. Deux lieux de stockage possibles :
  le JSON `bank_profiles.column_mapping` (les indices de colonnes y vivent tous), ou une colonne SQL dédiée
  (précédent `debit_sign`, Story 26-1, choisi « parce que la 0.13.0 désérialise `column_mapping` en
  `deny_unknown_fields` » — en-tête de `20261011000001_bank_profiles_debit_sign.sql`).
- **Retenu** : `ColumnMapping.currency: Option<usize>`, `#[serde(skip_serializing_if = "Option::is_none", default)]`
  comme ses voisins `reference` / `details` / `counterparty`. **Aucune migration, aucun bump de
  `kesh_version_min_required`, aucun bump Cargo.** Raisons :
  1. l'indice entre de lui-même dans `all_indices()` et `max_index()` : la collision avec une autre colonne et le
     hors-borne au premier enregistrement sont refusés **sans code neuf** ;
  2. un profil **sans** colonne Devise se sérialise **octet pour octet** comme avant (le champ est omis) : une
     0.13.1 le lit sans rien voir ;
  3. un profil **avec** colonne Devise, relu par une 0.13.1, est **refusé bruyamment** : `ColumnMapping` y porte
     `#[serde(deny_unknown_fields)]` (vérifié aux tags : `git show v0.13.1:crates/kesh-import/src/csv/profile.rs |
     grep -n deny_unknown_fields` → `37:#[serde(deny_unknown_fields)]` ; `v0.13.0` → `29:`), donc
     `parse_column_mapping` rend `DbError::Invariant` → 500 sur la liste des profils et sur tout import par ce
     profil. **Aucun import silencieusement faux** : la politique de migration vise les *downgrades silencieux
     corrupteurs* (§ Migration breaking policy, introduction), ce retour-là est bruyant et n'écrit rien.
  4. une colonne SQL dédiée serait, elle, **ignorée en silence** par une 0.13.1 (qui forcerait CHF de nouveau) et
     imposerait donc le bump `min_required` → `0.13.2`, le bump Cargo du workspace dans la même PR (P2-bis), une
     migration (P5, P6, P7, squash `test-schema`, `test_schema_guard`), pour un résultat moins bon.
- **Écartées** : colonne SQL dédiée + bump (plus lourde, et rend le retour arrière impossible au lieu de bruyant) ;
  lecture de la devise dans le **préambule** UBS (`Évaluation en:;CHF;`) — hors périmètre, c'est #621 ; détection
  automatique d'une colonne titrée « Devise » / « Währung » — contraire à la règle du Project Lead de la 26-1
  (« un réglage explicite, jamais une déduction »).
- **Conséquence écrite** (CHANGELOG, AC12 ; manuel d'administration, AC12-bis) : avant un retour à une 0.13.1,
  retirer la colonne Devise de **tous** les profils qui la lisent.
- **Portée réelle de la dégradation** (amendé en validation P1, lentilles R et F, vérifié aux tags) : **un seul** profil
  à colonne Devise fait échouer `GET /api/v1/bank-profiles` (500) pour **toute la société** sous la 0.13.1 — la liste
  désérialise chaque profil (`BankProfileResponse::try_from_db`, `routes/bank_profiles.rs:66` de la 0.13.1). L'écran
  *Profils bancaires* est inutilisable ; l'écran d'import **avale** l'erreur (`git show
  v0.13.1:frontend/src/lib/features/bank-import/BankImportUpload.svelte`, l. 66-71 : `catch { availableProfiles = []; }`)
  et affiche un sélecteur de profil **vide, sans message** ; l'import CAMT n'est pas touché. Le profil ne se corrige
  plus par l'écran sous la 0.13.1 (le `GET` par identifiant échoue aussi) : la colonne se retire **avant** le retour,
  sous la 0.13.2 — oubliée, elle se retire encore **par l'API** de la 0.13.1 (`PUT` complet sans
  `columnMapping.currency`, ou `DELETE` : ni l'un ni l'autre ne désérialise le profil enregistré, `git show
  v0.13.1:crates/kesh-db/src/repositories/bank_profiles.rs` l. 264 compare la chaîne brute ; amendé en validation P2,
  F7) — l'identifiant et les réglages qu'exige un `PUT` complet se lisant dans l'export global de la 0.13.1, que le
  `GET` ne rend plus (C-27-2-22) —, le SQL restant le dernier recours. Une sauvegarde d'installation 0.13.2 s'importe dans une 0.13.1 (pas de bump au
  manifeste), avec la même conséquence. Rien n'est importé à tort, aucune écriture : bruyant. ⚠️ Tout cela suppose
  qu'aucune migration de la 0.13.2 ne relève `kesh_version_min_required` (vrai pour 27-1, 27-2, 27-3 au 2026-10-12 ;
  validation P2, F10).
- **La prémisse est figée** (validation P2, F1 = R10 ; C-27-2-15) : ce raisonnement tient tout entier au
  `deny_unknown_fields` de `ColumnMapping` dans le binaire **antérieur**. La 0.13.2 sera le binaire antérieur de la
  prochaine clé : l'attribut y est gardé par un test (AC1-(5), mutation M23) et justifié au doc-comment de la struct,
  là où une revue future lira l'exception — et non plus dans ce seul registre.
- **Lecture de P1** : la définition de P1 (« un état du schéma qu'un binaire antérieur ne peut plus consommer
  correctement »), prise à la lettre, couvrirait ce cas. Elle est lue **à la lumière de l'objet de la politique** — sa
  phrase d'ouverture : « protège contre les downgrades **silencieux corrupteurs** » — et de sa forme (P2 prescrit un
  `UPDATE` **dans une migration**, P3 énumère des DDL). Ici, aucun DDL, et le binaire antérieur **refuse** au lieu
  d'importer faux. C'est une exception **assumée**, par analogie à P4 (une opération qui reste techniquement sans
  danger pour un binaire antérieur), écrite ici pour qu'une revue ne la classe pas CRITICAL « sans bump » par lecture
  littérale de P3 — qui ne vise d'ailleurs que des migrations. Le précédent `debit_sign` (26-1) est le cas inverse et
  confirme la ligne : la 0.13.0 aurait **ignoré** la colonne et inversé les débits **en silence** → bump.
- **Reste, assumé** (validation P1, F5-b) : un onglet de la 0.13.1 resté ouvert pendant la mise à jour, ou un client
  d'API, qui fait un `PUT` sans `currency` ramène le profil au CHF supposé, sans erreur (remplacement complet) ; seul
  l'avertissement `bank_csv_currency_assumed` le signale ensuite. Poste unique : risque faible.
- **Réversible** : oui (champ facultatif, jamais exigé).

### C-27-2-2 — Un relevé CSV est dans une seule devise : un fichier qui en mélange est refusé en entier

- **Contexte** : `ImportedStatement.currency` est une valeur unique (un `<Stmt>` CAMT.053 n'a qu'un compte, donc une
  devise) ; une colonne Devise lue ligne par ligne peut en rendre plusieurs.
- **Retenu** : le parseur (agnostique, `kesh-import`) exige que toutes les transactions **valides** portent la même
  devise ; sinon `CsvError::MixedCurrencies { currencies }` (devises distinctes, triées, **au plus 10**), refus
  **non confirmable** → `422 BANK_CSV_MIXED_CURRENCIES`, `details.currencies`. Le refus de signe
  (`AmountSignMismatch`, 26-1) le précède — « régler le profil d'abord » ; il précède lui-même le rejet partiel.
- **Écartées** : prendre la devise de la première ligne et mettre les autres en erreur de ligne (arbitraire, et
  confirmable : on importerait les lignes en CHF d'un relevé multi-devises sans que rien ne le dise au solde) ;
  ramener le mélange à `BANK_IMPORT_UNSUPPORTED_CURRENCY` dans le parseur (le parseur ne connaît pas le périmètre
  CHF, `kesh-core/src/bank_imports.rs:458-461`).
- **Réversible** : oui.

### C-27-2-3 — Le contrôle : `validate_currency_supported_v0_1` tel quel, appelé enfin sur le chemin CSV

- **Contexte** : le contrôle existe (`kesh-core/src/bank_imports.rs:467`) mais n'est appelé que sur le chemin
  CAMT (`routes/bank_imports.rs:961` aperçu, `:1127` import) ; `preview_csv` (`:1609`) et `create_csv` (`:1758`)
  ne l'appellent pas.
- **Retenu** : même fonction, **non modifiée**, sur le relevé CSV : à l'aperçu, avertissement
  `warnings.unsupportedCurrency` (l'écran l'affiche déjà et grise *Confirmer*, `BankImportUpload.svelte:226`, :596) ;
  à l'import, `422 BANK_IMPORT_UNSUPPORTED_CURRENCY` **avant** le traitement du rejet partiel (le relevé valide d'un
  `PartialFailure` est contrôlé avant que `confirmPartialImport` soit consulté : on ne demande pas de confirmer des
  lignes pour refuser ensuite le fichier).
- **Écartée** : étendre `validate_currency_supported_v0_1` à la devise **de chaque transaction**, pour les deux
  formats — dangereux pour CAMT : `camt053/mod.rs:168-178` pose `t.currency` depuis **tout** `<Amt Ccy>` rencontré
  sous `<TxDtls>` (y compris, si le fichier en porte, `AmtDtls/InstdAmt/Amt`), alors que le montant ne vient que de
  `TxDtls/Amt` (`:427`). Un relevé CHF porteur d'un paiement par carte en EUR pourrait être refusé à tort. Le
  **contrôle** CAMT n'est pas touché. *(Amendé en validation P1 : l'hypothèse sur `:168-178` est confirmée au code par
  les deux lentilles ; le **parseur** CAMT est corrigé à l'ancrage de la devise — C-27-2-11 —, ce qui rend la
  transaction d'un paiement par carte CHF de nouveau. L'extension du contrôle à chaque transaction reste écartée :
  elle n'apporte rien tant qu'aucun compte ne porte de devise, C-27-2-4.)*
- **Réversible** : oui.

### C-27-2-4 — Sans colonne Devise au profil : CHF supposé, et dit à chaque aperçu

- **Contexte** : l'issue propose, à défaut, de « documenter que le profil CSV suppose un compte en CHF ». Le
  contrôle « par la devise du compte bancaire » est impossible : `bank_accounts` ne porte aucune devise
  (`crates/kesh-db/src/entities/bank_account.rs:24-53`, `grep -rln currency crates/kesh-db/migrations/` ne cite que
  `bank_imports` et `imported_supplier_invoices`) — tous les comptes sont implicitement en CHF dans cette version.
- **Retenu** : profil sans colonne Devise → devise `DEFAULT_CSV_CURRENCY = "CHF"` (constante publique nommée et
  documentée de `kesh-import`, au lieu de deux littéraux), **et** avertissement informatif
  `bank_csv_currency_assumed` à chaque aperçu d'un tel profil (« Le profil ne lit aucune colonne Devise : le relevé
  est supposé en francs suisses (CHF). »). Le manuel le dit, et conseille de régler la colonne.
- **Écartées** : rendre la colonne obligatoire (casserait tous les profils existants et les banques dont le CSV n'a
  pas de colonne Devise) ; ajouter une devise au compte bancaire (périmètre multi-devises) ; documentation seule
  (l'hypothèse resterait invisible à l'import, ce qui est exactement le défaut).
- **Réversible** : oui.

### C-27-2-5 — Lecture d'une cellule Devise : forme ISO, casse normalisée, erreurs de ligne ordinaires

- **Retenu** : cellule `trim`ée ; vide → `EMPTY_MANDATORY_FIELD` (valeur `currency`) — une fois la colonne
  réglée, la devise est obligatoire, jamais remplacée par CHF ; exactement trois lettres ASCII → mise en
  majuscules (`chf` → `CHF`) ; autre chose (`Fr.`, `€`, `CH`, `CHF1`) → nouveau code de ligne `INVALID_CURRENCY`.
  Ce sont des erreurs de ligne **ordinaires** (rejet partiel, confirmables), comme une date illisible : la ligne
  écartée n'entre pas en comptabilité. Pas de contrôle contre la liste ISO 4217 : la forme suffit au parseur, et
  toute devise autre que CHF est de toute façon refusée par C-27-2-3.
- **Ordre dans `parse_row`** : longueur, date, montant, **puis devise** — un montant illisible ou de signe
  contraire masque la devise de la ligne (la ligne est écartée ou le fichier refusé de toute façon).
- **Écartées** : CHF pour une cellule vide (rouvrirait #624 ligne par ligne) ; refus du fichier entier sur une
  cellule illisible (sans motif : la ligne n'est pas importée).
- **Réversible** : oui.

### C-27-2-6 — L'avertissement « Devise non supportée v0.1. » est réécrit

- **Contexte** : la clé `bank-import-warnings-unsupported-currency` dit « v0.1 » dans les quatre locales
  (`fr-CH:1065`, `de-CH:1007`, `it-CH:1007`, `en-CH:1007`) ; le correctif la rend atteignable par le CSV, chemin le
  plus employé.
- **Retenu** : « Devise non supportée : seul le franc suisse (CHF) est pris en charge. Devise du relevé : » (la devise
  suit, comme aujourd'hui), quatre locales, repli Svelte identique.
- **Étendu en validation P1** (R4-1) : les autres textes du même refus qui disent « v0.1 » hors des `.ftl` suivent —
  `crates/kesh-api/src/errors.rs:673-674` (doc-comment et message `thiserror`), `:2141` (repli de
  `bank-import-errors-unsupported-currency`, aligné sur son texte `fr-CH`), `docs/i18n-glossaire.md:161` (exemple).
- **Réversible** : oui.

### C-27-2-7 — L'écran du profil : un champ de plus, traduit ; le reste du formulaire n'est pas repris

- **Retenu** : champ « devise (facultative) » (`data-testid="col-currency-input"`), vide par défaut, initialisé depuis
  `existing.columnMapping.currency`, envoyé seulement s'il est rempli, avec une aide traduite. Libellé et aide
  passent par `i18nMsg` (clés neuves, quatre locales).
- **Non repris, signalé** : les libellés voisins (`date`, `débit`, `crédit`, `montant`, `référence`, `détails`,
  `BankProfileForm.svelte:259-283`) sont du français en dur. *(Amendé en validation P1 : la perte du `counterparty`
  posé par l'API, d'abord signalée ici avec une issue demandée, est **corrigée dans la story** — C-27-2-12, AC8-bis —
  sur décision de l'orchestrateur ; plus d'issue à ouvrir. Le formulaire n'a toujours pas de **champ** contrepartie.)*
- **Réversible** : oui.

### C-27-2-8 — Le profil UBS recommandé lit la colonne 4

- **Retenu** : le tableau du manuel (*Le relevé CSV d'UBS*) gagne « Colonne devise : 4 (« Devise ») » ; le test
  local sur le vrai relevé (`crates/kesh-import/tests/ubs_fichier_reel.rs`, ignoré par défaut) mappe la colonne 4 et
  vérifie que **chaque** transaction et le relevé sont en CHF — contrôle rendu « conforme / non conforme »
  seulement, lancé par l'orchestrateur (aucune donnée réelle dans le dépôt).
- **Réversible** : oui.

### C-27-2-9 — Fixtures : dérivées de `ubs_synthetique.csv`, jamais du fichier réel

- **Retenu** : les cas EUR et mélange se **dérivent dans le test** de `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`
  (remplacement de `;CHF;` sur les **seules** lignes de données : le préambule porte aussi `;CHF;`, `Évaluation
  en:;CHF;` ligne 7 — 12 occurrences pour 11 lignes ; le helper saute préambule et en-tête et asserte le nombre de
  remplacements, validation P1 R9-1 ; il saute `header_row_count` lignes **non vides** — la fixture a dix lignes
  physiques avant la première donnée pour `header_row_count = 9`, la ligne 9 étant vide et avalée par le lecteur `csv`,
  validation P2 R5) — une seule source ; les cas limites (cellule vide, `chf`,
  `Fr.`, collision, hors-borne) en petits CSV écrits dans le test. Aucun fichier neuf sous `fixtures/csv/`, rien lu
  sous `~/kesh-test/`. *(La seule fixture neuve de la story est CAMT, synthétique : C-27-2-11.)*
- **Réversible** : oui.

### C-27-2-10 — Hors périmètre, signalés et non traités

- ~~**CAMT.053, devise par transaction**~~ — *sorti de cette liste en validation P1* : hypothèse confirmée au code par
  les deux lentilles (`camt053/mod.rs:168-178` contre `:372`, `:427`), corrigée dans la story (C-27-2-11, AC19). Plus
  d'issue à ouvrir.
- **Le préambule UBS** (`Évaluation en:`) : #621.
- **Semver de `kesh-import`** : un champ public ajouté à `ColumnMapping`, un variant à `CsvError` et à
  `CsvLineErrorCode` — même nature que les ajouts de la 26-1 (`CsvProfile.debit_sign`, `HeaderSkipsData`,
  `AmountSignMismatch`), sans `#[non_exhaustive]` : le crate n'est pas publié, le `--dry-run` reste le seul
  contrôle exigé.

### C-27-2-11 — CAMT.053 : la devise d'une transaction se lit sur l'élément qui porte son montant (validation P1)

- **Contexte** : les deux lentilles de P1 confirment au code l'hypothèse de C-27-2-3 : à l'ouverture de **tout**
  `<Amt Ccy>`, `camt053/mod.rs:168-178` écrase la devise de la transaction (sous `<TxDtls>`) ou de l'entrée (sinon),
  le dernier rencontré gagnant, alors que le montant ne se lit que sur `TxDtls/Amt` (`:427`) et `Ntry/Amt` (`:372`). Un
  `<AmtDtls><InstdAmt><Amt Ccy="EUR">`, des frais `<Chrgs>` ou des intérêts `<Intrst>` en devise étrangère donnent à
  une transaction d'un compte CHF la devise EUR, au montant en francs ; les propositions automatiques et l'acceptation
  d'une règle l'écartent ensuite, le rapprochement manuel, l'éclatement et l'acceptation d'une facture l'admettent
  (amendé en validation P3, F1 = R1 ; C-27-2-20). Aucune
  fixture du dépôt ne porte `<AmtDtls>` ni `<Chrgs>` (`grep -n "Chrgs\|AmtDtls" crates/kesh-import/tests/fixtures/camt053/*.xml`
  → aucune sortie), d'où l'absence de test qui l'aurait montré.
- **Retenu** (périmètre élargi par l'orchestrateur, même sujet que #624 : la devise des imports ; pas d'issue neuve) :
  la devise n'est retenue que si l'`Amt` est l'enfant **direct** de `<TxDtls>` ou de `<Ntry>` (`path.last()` à
  l'ouverture, `Amt` n'étant poussé qu'après) — le même élément que le montant. Fixture synthétique
  `v04_amtdtls_foreign_currency.xml`, tests écrits et vus **rouges** avant la correction, test d'intégration, mutations
  M19 (bras `TxDtls`) et M20 (bras `Ntry`).
- **Écartées** : contrôler chaque transaction contre CHF (C-27-2-3 : refuserait à tort un relevé CHF à paiement par
  carte en EUR) ; préférer `AmtDtls/TxAmt` quand il existe (le montant enregistré est celui de `TxDtls/Amt` : la devise
  doit être la sienne) ; laisser en l'état avec une issue (le défaut est dans le même sujet, et le nombre de bugs doit
  baisser à la release).
- **Angle mort assumé** (validation P2, F8) : une entrée dont l'`Amt` direct n'a pas de `Ccy`, mais dont un montant
  d'information en porte un, en prenait la devise ; elle est désormais refusée (`MissingRequiredField(currency)`,
  `camt053/mod.rs:572-575`), et avec elle tout le relevé. Le XSD rend `Ccy` obligatoire : seul un exporteur non conforme
  produit ce cas ; le refus est bruyant ; reprendre la devise d'un montant d'information serait le défaut même. Pas de
  test exigé.
- **Réversible** : oui (un ancrage dans le parseur, aucune donnée persistée autrement qu'avant pour un fichier sans
  montant d'information).

### C-27-2-12 — Le formulaire de profil garde toute clé qu'il ne montre pas (validation P1, F1)

- **Contexte** : `BankProfileForm.svelte:56-69` reconstruit `columnMapping` de ses seuls champs ; le `PUT` remplace le
  profil entier ; un `counterparty` posé par l'API est donc **perdu** au premier enregistrement à l'écran — chemin que
  la story prescrit justement (ajouter la colonne Devise aux profils existants).
- **Retenu** (périmètre élargi par l'orchestrateur, pas d'issue neuve) : le mapping envoyé part du mapping existant
  **privé des clés que le formulaire possède** (une constante nommée : `date`, `amount`, `debit_credit_split`,
  `reference`, `details`, `currency`), puis reçoit les valeurs du formulaire. Garde `counterparty` aujourd'hui, et
  toute clé future que le formulaire ne montrerait pas. Forme de la constante et de la fonction : C-27-2-17 ; les deux
  sens du changement de mode sont testés (AC8-bis (v7) et (v8), validation P2, R1).
- **Écartées** : recopier `counterparty` seul (le défaut renaîtrait à la prochaine clé) ; recopier tout le mapping
  existant (un passage colonne unique → mode séparé laisserait `amount` **et** `debit_credit_split` : refus `422`) ;
  ajouter un champ contrepartie (hors sujet, et rien ne l'exige).
- **Réversible** : oui.

### C-27-2-13 — Le plafond de 10 devises rapportées : les 10 premières du tri, sans indicateur (validation P1, R6-1)

- **Retenu** : `MixedCurrencies.currencies` porte les 10 premières de l'ensemble **trié** des devises distinctes des
  lignes valides (un `BTreeSet`), jamais les 10 premières rencontrées ; aucun indicateur de troncature — le message
  « le relevé porte plusieurs devises (…) » reste vrai, et un relevé à plus de 10 devises n'est pas un relevé de compte.
  Le test AC13-11 place le code qui se trie en dernier sur la **première** ligne, pour départager.
- **Écartée** : un champ `truncated` / `total` (patron des erreurs de ligne) — un champ d'API de plus pour un cas que
  rien ne produit.
- **Réversible** : oui (ajout d'un champ facultatif).

### C-27-2-14 — Le mélange de devises ne se juge que sur les lignes valides (validation P1, R7-1 / F4)

- **Contexte** : la devise est lue **après** la date et le montant (C-27-2-5) : une ligne écartée pour une autre raison
  ne révèle jamais sa devise.
- **Retenu** : assumé et écrit (AC3, manuel § *Rejet partiel*). Un fichier CHF dont les lignes EUR sont toutes
  invalides pour une autre raison se confirme en partiel et importe ses lignes CHF, **exactes** ; les lignes EUR restent
  listées comme invalides — aucun montant faux n'entre en comptabilité.
- **Écartée** : lire la devise avant la date et le montant (une ligne illisible donnerait un refus de mélange sur une
  cellule peut-être décalée — moins lisible que l'erreur de ligne d'aujourd'hui, et sans gain comptable).
- **Réversible** : oui.

### C-27-2-15 — `deny_unknown_fields` de `ColumnMapping` : figé par un test et justifié au code (validation P2, F1 = R10)

- **Contexte** : C-27-2-1 (aucune migration, aucun bump) repose entièrement sur le fait qu'un binaire **antérieur**
  refuse une clé inconnue de `ColumnMapping` au lieu de l'ignorer. Aucun test ne gardait l'attribut
  (`grep -rn "deny_unknown\|unknown field" crates/kesh-import/src crates/kesh-db/src crates/kesh-api/tests` → la seule
  ligne `profile.rs:37`), et l'exception n'était écrite qu'ici. Or la 0.13.2 sera le binaire antérieur de la prochaine
  clé : l'attribut retiré, ce retour-là deviendrait **silencieux** — le downgrade corrupteur que la politique vise.
- **Retenu** : un test AC1-(5) (`unknown_key_is_rejected_so_older_binaries_refuse_loudly`, clé `bogus` → `Err`) que tue
  la mutation M23 (attribut retiré) ; un doc-comment `///` sur `ColumnMapping` qui dit le contrat (ajouter un champ =
  refus bruyant par les binaires antérieurs, sans bump ; retirer l'attribut = retour silencieux depuis la version
  suivante) et renvoie à C-27-2-1.
- **Écartées** : un garde-fou dans `textes_coherents.rs` ou un test d'architecture (plus lourd ; le test sérde suffit à
  faire rougir) ; ne rien faire (l'exception resterait invérifiable par qui lit le code).
- **Réversible** : oui.

### C-27-2-16 — Les transactions déjà importées avec une devise fausse : remède écrit, rien de corrigé d'office (validation P2, F2)

- **Contexte** : les deux défauts corrigés ont pu écrire des données fausses — un CSV d'un compte en devise importé en
  `CHF` au montant du fichier (#624), un paiement CAMT par carte resté « en EUR » au montant en francs (AC19), que les
  propositions automatiques et l'acceptation d'une règle écartent (`reconciliation.rs:631`, `:780`, `:2728`), mais que
  le rapprochement manuel, l'éclatement et l'acceptation d'une facture admettent (C-27-2-20). La mise à jour ne les touche pas ; le réimport seul ne les
  remplace pas (fichier en doublon, `BANK_IMPORT_DUPLICATE_FILE` ; empreinte de ligne sans la devise,
  `routes/bank_imports.rs:1532-1539`).
- **Retenu** : écrire le remède au CHANGELOG (AC12) et au manuel d'administration (AC12-bis, paragraphe
  « Transactions déjà importées », patron de `admin-manual.tex:1993`) : **supprimer l'import non rapproché** (Story 27-1,
  même release) puis le réimporter ; s'il a des transactions rapprochées, la suppression est refusée
  (`409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, fiche 27-1, AC5) — annuler d'abord leur rapprochement (*Annuler le
  rapprochement* contre-passe l'écriture propre ou annule le règlement, `reconciliation_cancel.rs`, doc-module), puis
  supprimer et réimporter. Une transaction CAMT restée « en EUR » a pu être rapprochée à la main, par éclatement ou par
  l'acceptation d'une facture : elle suit alors la même règle (annuler d'abord son rapprochement). *(La validation P2
  avait écrit qu'elles n'étaient pas rapprochables : faux, corrigé en validation P3, F1 = R1.)* Un CSV d'un compte en devise réimporté avec la
  colonne Devise est refusé : il n'a pas sa place dans Kesh (C-27-2-4).
- **Écartées** : une migration de correction (impossible : rien en base ne dit quelle transaction CSV venait d'un compte
  en devise, et une transaction CAMT ne garde pas son XML) ; un outil de réparation dédié (hors périmètre ; la
  suppression d'import de la 27-1 suffit) ; le silence (le précédent de la 0.13.1 écrit ce paragraphe pour le sens des
  débits).
- **Dépendance** : le remède suppose la 27-1 dans la même release. Si la 27-1 n'y était pas, le paragraphe se récrit
  (SQL ou restauration) — à revérifier par l'orchestrateur à la fusion.
- **Réversible** : oui (texte).

### C-27-2-17 — Les clés possédées par le formulaire : une constante, une fonction, aucune liste répétée (validation P2, F4 = R2)

- **Contexte** : la remédiation P1 prescrivait à la fois une déstructuration (qui écrit la liste des clés dans son
  motif) et « une constante nommée partagée » — deux sources de vérité ; et la déstructuration de
  `existing?.columnMapping ?? {}` (type `ColumnMapping | {}`) risquait l'erreur TS2339 sous `strict: true`
  (non compilé, déduit par les deux lentilles).
- **Retenu** : `OWNED_COLUMN_MAPPING_KEYS = [...] as const satisfies readonly (keyof ColumnMapping)[]` et
  `keepUnownedKeys(existing?: ColumnMapping): Partial<ColumnMapping>` qui copie le mapping puis `delete` chaque clé
  possédée — sans `any` ni cast ; `satisfies` n'est employé au dépôt que dans des `.test.ts`, jamais dans un `.svelte`
  de production, mais le compilateur Svelte 5.55.1 le retire (`remove_typescript_nodes.js:89`, `TSSatisfiesExpression`)
  — admis par lecture, sans précédent, `npm run check` et `npm run build` tranchent en T4 (corrigé en validation P3,
  R9 : « déjà employé au dépôt » était inexact pour ce contexte). `ColumnMapping` s'ajoute à l'`import type` existant
  (`verbatimModuleSyntax`, validation P3, F6). Mutations M22 (tout
  recopier) et M24 (constante privée d'une clé) tuées par (v7)/(v8).
- **Écartées** : la déstructuration seule (pas de constante, liste dans le motif, typage incertain) ; un filtre
  `Object.fromEntries(Object.entries(…))` (perd le typage, cast nécessaire).
- **Réversible** : oui. *Non compilé à la rédaction : `npm run check` le confirme en T4.*

### C-27-2-18 — Hypothèse R9 (P2) : un `<TxDtls>` sans `Amt` propre prend le montant de l'entrée entière — non établie comme défaut, rapportée

- **Hypothèse** (lentille R, P2, R9) : en camt.053.001.**04**, `EntryTransaction4` n'aurait pas d'`Amt` direct ; un vrai
  relevé à plusieurs `<TxDtls>` par `<Ntry>` donnerait alors à chacun le montant de l'entrée (`t.amount.unwrap_or(
  ntry_amount)`, `camt053/mod.rs:603`).
- **Ce que le code établit** (lu) : le repli `:603` donne à **tout** `<TxDtls>` sans `TxDtls/Amt` direct le montant de
  l'entrée, quelle que soit la version (`v04.rs` et `v08.rs` ne font que dispatcher vers le même parseur). Avec **un**
  `<TxDtls>`, c'est juste ; avec **deux ou plus** dont l'un n'a pas d'`Amt` direct, la somme des transactions dépasse le
  montant de l'entrée. Le contrôle de solde le signale (`BankImportBalanceMismatch`), mais il est **confirmable**
  (`confirmBalanceMismatch`, contrôle à `routes/bank_imports.rs:1141`, `if !fields.confirm_balance_mismatch` ; `:433`
  n'est que la lecture du champ multipart — ligne corrigée en validation P3, F5) : bruyant, pas bloquant.
- **Ce que le schéma établit** : dans le seul XSD versionné (001.08,
  `docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd:705`), `EntryTransaction10/Amt` est
  `minOccurs="0"` — l'absence est donc **conforme** en 001.08 aussi. Aucun XSD 001.04 au dépôt : l'affirmation
  « `EntryTransaction4` n'a pas d'`Amt` » n'est **ni établie ni réfutée** ici (à ma connaissance elle est fausse —
  `Amt` et `CdtDbtInd` figurent, facultatifs, dans le `TxDtls` de la 001.04, et les fixtures v04 du dépôt les portent —,
  mais une connaissance non sourcée n'est pas une preuve).
- **Ce que les fixtures établissent** (comptage par `<Ntry>`, `awk` sur les 13 fichiers) : deux fixtures portent
  plusieurs `<TxDtls>` par entrée — `v04_with_subtxs.xml` (3, tous avec `Amt` direct) et `v04_ntry_sign_fallback.xml`
  (**2** éléments `<TxDtls>`, tous avec `Amt` direct ; `grep -c` → 3, la troisième occurrence étant une mention en commentaire, l. 48 — la P3, R6, avait écrit 3 en comptant ce texte, rétabli en validation P4, P4-1) ; les treize autres entrées
  à `<TxDtls>` (dont celle de `v04_prefixed_namespace.xml`, comptée à part, ses balises étant préfixées ;
  `v04_truncated.xml`, coupé, exclu) en ont **un seul**, sans `Amt` direct (repli juste).
  **Aucune fixture n'exerce le cas litigieux** (≥ 2 `<TxDtls>` dont un sans `Amt` direct) : aucun test ne le couvre, ni
  dans un sens ni dans l'autre.
- **Conclusion** : comportement **établi au code** pour une forme d'entrée conforme au XSD 001.08 mais **dont l'existence
  chez une banque n'est pas établie** ; ce n'est donc pas un défaut établi. Pour l'établir, il manque : (1) une source
  qui montre la forme en usage — un relevé réel d'une banque suisse (contrôlé en local, rapporté « présent / absent »),
  ou le guide SPS camt.053 (SIX) qui dirait `TxDtls/Amt` facultatif et non rempli pour les lots ; (2) une fixture
  synthétique à deux `<TxDtls>` dont un sans `Amt`, et un test qui montre la somme fausse. Le comportement correct
  serait alors à trancher (refuser l'entrée, ou répartir via `AmtDtls/TxAmt`).
- **Retenu** : **hors périmètre** de la 27-2 (AC19 ne change rien à ce repli : la devise y suit l'entrée, ce qui est
  juste) ; rapporté à l'orchestrateur, qui décide d'une issue.
- **Réversible** : sans objet.

### C-27-2-19 — Les « v0.1 » restants du refus de devise : réécrits là où ils décrivent ce refus, gardés là où ils racontent l'histoire (validation P2, R7)

- **Contexte** : le grep prescrit en P1 laissait hors champ trois textes du même refus — `kesh-core/src/errors.rs:93`
  (message au journal), `kesh-import/src/error.rs:5` et `kesh-core/src/bank_imports.rs:25`, `:457` (doc-comments).
- **Retenu** : les réécrire (« v0.1 » → « cette version », ou retiré du message) dans le même patch qu'AC10, et élargir
  le grep de contrôle à `crates/` (`grep -rnF "v0.1" crates --include=*.rs | grep -i "devise\|currency"`). Gardés : le
  nom `validate_currency_supported_v0_1` (C-27-2-3) et les mentions « v0.1 mono-CHF » des tests et de la réconciliation
  (`reconciliation.rs:628`, `reconciliation_e2e.rs`, `bank_imports_e2e.rs:443`, `bank_imports.rs:1125`), qui décrivent
  une convention toujours vraie.
- **Écartée** : renommer la fonction (bruit de diff sans gain ; C-27-2-3).
- **Réversible** : oui.

### C-27-2-20 — La garde de devise de la réconciliation est partielle : les textes le disent, la garde n'est pas ajoutée (validation P3, F1 = R1)

- **Contexte** : la remédiation P2 avait écrit, au CHANGELOG prescrit (AC12), à C-27-2-16 et par renvoi à AC12-bis,
  que « les transactions CAMT restées en EUR ne sont pas rapprochables ». Les deux lentilles de P3 l'ont réfuté au
  code : seules les propositions automatiques (`reconciliation.rs:631`, `:780`) et l'acceptation d'une règle (`:2728`)
  écartent une transaction non CHF ; `accept_one_invoice` (`:1428`), `accept_one_split` (`:2119`), `post_manual_once`
  (`:3532`) et `post_split_once` (`:3984`) ne lisent pas la devise. Le dépôt le documente en partie — pour la seule acceptation d'une facture (validation P4, P4-5)
  (`crates/kesh-api/tests/reconciliation_e2e.rs:2241-2262`, « l'opération réussit silencieusement » ; test `#[ignore]`
  `post_accept_filters_currency_mismatch`, l. 1975), et le catalogue dit « avec une facture ou une règle »
  (`reconciliation-failed-currency-mismatch`, `fr-CH:1185`).
- **Retenu** : corriger le **texte** partout où il paraît (AC12 bloc 2 et puce CAMT, AC12-bis, AC19, inventaire (i),
  C-27-2-11, C-27-2-16) : une transaction CAMT restée « en EUR » a pu être rapprochée à la main, par éclatement ou par
  acceptation d'une facture, et suit alors la règle commune — annuler d'abord son rapprochement, puis supprimer
  l'import. **Aucune garde n'est ajoutée** aux quatre chemins.
- **Écartée** : ajouter la garde dans la 27-2 (hors sujet — la 27-2 corrige l'**entrée** de la devise ; la garde de
  réconciliation est un défaut distinct, connu du dépôt depuis l'Epic 8, et elle changerait le comportement de quatre
  chemins de `reconciliation.rs` sans AC ni E2E prévus). Rapporté à l'orchestrateur, qui vérifie qu'une issue la
  suit, ou en ouvre une.
- **Pourquoi MEDIUM et non HIGH** (rappel des rapports) : le remède reste juste — le `409` de la 27-1 nomme le geste
  (annuler d'abord) ; c'était une cause affirmée fausse, pas un piège.
- **Réversible** : oui (texte).

### C-27-2-21 — Textes du refus de devise : liste fermée des « v0.1 » gardés, parenthèse fausse de `reconciliation_e2e.rs` récrite, apostrophe droite (validation P3, R2, R7)

- **Contexte** : AC10 promettait que le grep `"v0.1"` des crates « ne rend plus que les sites gardés ci-dessus » ; il
  en rend trois de plus, non nommés (`kesh-db/src/repositories/reconciliation.rs:9`, `:104`,
  `kesh-core/src/bank_imports.rs:725`). En les relisant, `reconciliation_e2e.rs:2253-2254` affirme « le parser CSV
  rejette EUR/USD » — faux aujourd'hui (c'est #624), vrai après la 27-2 pour un profil qui lit la colonne Devise
  seulement. Et la fiche ne choisissait pas l'apostrophe des textes `fr-CH` neufs (le catalogue mêle `'` et `’`), alors
  que G9 compare octet pour octet.
- **Retenu** : AC10 donne la **liste fermée** des sites gardés, et le dev recopie la sortie du grep au Dev Agent
  Record (inventaire des non-résolus plutôt qu'énumération, `CLAUDE.md`) ; la parenthèse de
  `reconciliation_e2e.rs:2253-2254` est récrite, « v0.1 mono-CHF » gardé ; apostrophe **droite**, comme les clés
  voisines `bank-import-csv-errors-*` ; la garde `i18n-repli-divergent-actif.test.ts` est nommée.
- **Écartées** : réécrire les trois sites non nommés (ils décrivent des conventions vraies ou un message de test) ;
  l'apostrophe typographique (minoritaire dans la famille, et rien ne l'impose).
- **Réversible** : oui.

### C-27-2-22 — Réparer un profil sous la 0.13.1 : identifiant et réglages lus dans l'export global (validation P3, F2)

- **Contexte** : le `PUT` complet que prescrivait la P2 exige l'identifiant et tous les champs du profil ; or sous la
  0.13.1, `GET /bank-profiles` et `GET /bank-profiles/{id}` rendent 500 pour un profil à colonne Devise.
- **Retenu** : nommer la source — l'**export global** de la 0.13.1, `bank_profiles.csv` (colonnes `id`,
  `column_mapping_json`…), qui recopie la chaîne brute sans désérialiser : `git show
  v0.13.1:crates/kesh-api/src/exports/csv_tables.rs` l. 1041-1075 et `exports/global.rs:320`
  (`bank_profiles::list_all_by_company`) — **lus**, non exécutés. Le `DELETE` reste le geste simple (il perd le
  profil) ; le SQL, le dernier recours.
- **Écartée** : renvoyer seulement au SQL (le recours existe sans lui).
- **Réversible** : oui (texte).

### C-27-2-23 — Le helper de dérivation d'AC13 : « ligne vide » = zéro caractère (validation P3, R4)

- **Contexte** : la P2 a prescrit de sauter `header_row_count` lignes « non vides » sans définir le mot ; le lecteur
  `csv` n'avale qu'une ligne **sans aucun caractère**, une ligne d'espaces ou de `;` est un enregistrement.
- **Retenu** : `line.is_empty()` une fois le terminateur retiré, jamais `trim()` ; écrit au doc-comment du helper.
  Aucun test ne rougirait (la fixture n'a pas de telle ligne) : c'est une règle de rédaction, pas une preuve.
- **Réversible** : oui.

### C-27-2-24 — Le CHANGELOG prescrit : texte à publier séparé des notes de traçabilité, rangé dans les rubriques de la 0.13.1 (validation P3, R8)

- **Contexte** : le texte prescrit portait des renvois « (validation P2, F2 ; C-27-2-16…) » **dans** les puces à
  publier, et trois puces (« À faire », « Retour à une 0.13.1 », « API ») n'appartenaient à aucune rubrique.
- **Retenu** : bloc « ⚠️ À faire en passant à la 0.13.2 » numéroté en tête de section (ajout de la colonne,
  transactions déjà importées, retour à une 0.13.1), `### Modifié` (API), `### Corrigé` (trois entrées) — patron de la
  0.13.1 ; renvois, preuves et réserve de fusion dans des *Notes pour le dev*, non publiées ; lien d'issue écrit en
  ligne (le CHANGELOG n'a aucune définition de lien de référence).
- **Réversible** : oui.

### C-27-2-25 — L'E2E AC15-1 importe un fichier propre, dérivé en mémoire, et asserte le nombre de transactions (validation P3, R5)

- **Contexte** : AC15-1 réimportait `ubs_synthetique.csv`, déjà importé dans la même base et sur le même compte par le
  test voisin de la 26-1 : fichier en doublon (*Confirmer* grisé), lignes en doublon (import à zéro transaction), motif
  de profil partagé. « Chaque transaction en CHF » serait passé **à vide**.
- **Retenu** : `ubs_synthetique_devise.csv`, dérivé en mémoire en décalant d'une année les dates des seules lignes de
  données (l'empreinte de ligne porte la date), banque et motif propres, `toHaveLength(11)` asserté avant les devises ;
  AC15-2 dérive son EUR du même buffer. Aucune fixture neuve sur disque.
- **Écartées** : un second compte bancaire (montage plus lourd, IBAN de test à inventer) ; une remise à zéro de la base
  entre tests (`seedTestState` par test : coût et couplage aux autres specs).
- **Réversible** : oui.

### C-27-2-26 — `OWNED_COLUMN_MAPPING_KEYS` : la constante garde ses six clés, le doc-comment dit pourquoi, quatre mutants sont déclarés équivalents (validation P3, F6 = R3)

- **Contexte** : le doc-comment prescrit en P2 justifiait chaque clé par un risque qui ne vaut que pour `amount` et
  `debit_credit_split` (écrites dans un seul mode) ; pour `date`, `reference`, `details`, `currency`, écrites dans les
  deux modes, le spread du formulaire écrase la valeur recopiée — omettre ces clés de la constante est sans effet.
- **Retenu** : la constante garde les six clés (sûre si le formulaire cesse un jour d'écrire une clé dans un mode) ;
  doc-comment reformulé ; AC17 déclare **équivalents** les quatre mutants d'omission, pour qu'aucune lentille ne les
  compte comme survivants.
- **Écartée** : réduire la constante à deux clés (exact aujourd'hui, fragile demain : la sûreté reposerait sur une
  propriété du formulaire que rien ne teste).
- **Réversible** : oui.

### C-27-2-27 — AC19 : une troisième entrée de fixture, la forme dominante, et la mutation « première devise gagne » (validation P3, F3)

- **Contexte** : les deux entrées prescrites ont un `Amt` direct (entrée 1) ou pas de `<TxDtls>` (entrée 2) ; la forme
  la plus répandue au dépôt — un `<TxDtls>` **sans** `Amt` propre — n'était pas exercée, et un mutant « la première
  devise rencontrée gagne » passait les tests (1) et (2) tout en laissant le défaut sur cette forme.
- **Retenu** : entrée 3 (`<Ntry>` à `Amt` CHF direct, un `<TxDtls>` sans `Amt` mais avec `AmtDtls/InstdAmt/Amt` EUR),
  test (1-bis) — devise CHF, montant de l'entrée —, mutation M25 qu'elle seule tue ; AC14-8 asserte trois
  transactions.
- **Réversible** : oui.

### C-27-2-28 — La place du `409 BANK_IMPORT_PREVIEW_STALE` de la 27-3 dans l'ordre des refus (validation P3, F4)

- **Contexte** : la fiche demandait de l'« insérer dans l'ordre des refus » sans dire où.
- **Retenu** : CSV — après la résolution du profil (et son `404 BANK_CSV_NO_PROFILE_MATCH`), avant tout parse, donc
  avant le contrôle de devise ; CAMT — avant `DUPLICATE_FILE` (asymétrie de la 27-3, C-27-3-24), sans effet sur le
  contrôle de devise CAMT. Repris de la fiche 27-3 (AC8), lue sur sa branche.
- **Réversible** : oui (ordre écrit ; la fusion le tient).

### C-27-2-29 — Le helper `unsupported_currency` sert aussi l'import CAMT (développement)

- **Contexte** : AC5 prescrit de factoriser le bloc de contrôle (aperçu CAMT, `:961-971`) pour le chemin CSV. L'import
  CAMT (`create`, `:1126-1130`) avait sa propre forme, `if let Err(BankImportUnsupportedCurrency(c)) = …`, qui
  **ignorait** toute autre variante de `CoreError`.
- **Retenu** : un seul helper (`routes/bank_imports.rs`, `unsupported_currency(&stmt) -> Result<Option<String>,
  AppError>`) pour les **quatre** sites — aperçu et import, CAMT et CSV. Effet de bord sur l'import CAMT : une variante
  inattendue de `CoreError` rend désormais `500 Internal` (journalisée) au lieu d'être ignorée. Aujourd'hui
  `validate_currency_supported_v0_1` ne rend aucune autre variante : sans effet observable.
- **Écartée** : laisser la forme CAMT d'origine (deux formes du même contrôle, contraire à la règle DRY que l'AC cite).
- **Réversible** : oui.

### C-27-2-30 — AC15-1 et AC15-2 dans un seul test E2E (développement)

- **Contexte** : AC15-2 emploie « même profil » et « le buffer de (1) ».
- **Retenu** : un seul `test(...)` — le profil créé à l'écran, l'import CHF (AC15-1), puis l'aperçu EUR du même nom de
  fichier (AC15-2) ; les deux parties sont commentées. Deux tests auraient dû recréer le profil (nom unique, `409`
  sinon) ou dépendre de l'ordre d'exécution.
- **Écartée** : deux tests à profils distincts (plus lent, et le second ne vérifierait plus « le même profil »).
- **Réversible** : oui.

### C-27-2-31 — Les mutations sont tuées par les tests Rust et Vitest ; l'E2E n'est pas muté (développement)

- **Contexte** : le tableau de l'AC17 cite, pour M1, M3, M13 et M14, des tests E2E (AC15) parmi les tests qui doivent
  rougir ; l'AC demande qu'**au moins un** test nommé rougisse.
- **Retenu** : chaque mutation est jouée contre le binaire de tests le plus proche (`kesh-import`, `bank_imports_e2e`,
  Vitest du formulaire) et tuée par un test nommé du tableau ; la suite Playwright n'est pas rejouée sous mutation
  (≈ 10 min par rejeu, sous verrou partagé). L'E2E reste la preuve que la valeur traverse la frontière HTTP (AC15),
  exercée au gate sur le code non muté.
- **Écartée** : rejouer Playwright sous M1, M3, M13, M14 (huit rejeux complets pour des mutations déjà tuées).
- **Réversible** : oui (une passe de revue peut demander le rejeu).

### C-27-2-32 — Apostrophes des textes neufs en-CH et it-CH : celles des voisins de bloc (revue de code P1, A-3, E-8)

- **Contexte** : C-27-2-21 ne prescrivait l'apostrophe droite que pour fr-CH ; les textes neufs en-CH et it-CH
  l'avaient prise aussi, à côté de voisins typographiques (`profile’s`, `dell’impostazione`, textes de la 26-1).
  Aucune garde ne contrôle l'apostrophe des catalogues autres que fr-CH.
- **Retenu** : chaque texte neuf prend l'apostrophe des clés voisines de son bloc. en-CH : l'aide de la colonne
  Devise passe à `line’s` (bloc des messages CSV, tout en `’`). it-CH : `bank-import-csv-errors-mixed-currencies`
  et l'aide de la colonne Devise passent à `’` (voisines `amount-sign-*` de la 26-1) ; l'avertissement
  `bank-import-warnings-unsupported-currency` et l'information `bank-import-info-bank-csv-currency-assumed` gardent
  `'`, comme la seule voisine qui en porte (`bank-import-errors-malformed-xml`). fr-CH et de-CH inchangés.
- **Écartées** : tout passer en `’` dans en-CH et it-CH (réécrirait des clés hors du périmètre de la story) ; ne
  rien changer (rendu hétérogène dans un même bloc).
- **Réversible** : oui (textes seuls ; aucun test ne lit ces valeurs).

---

## Story 27-3 — Changer de profil après l'aperçu le recalcule ; empreinte d'aperçu vérifiée à la création (#626)

Fiche : `27-3-profil-apres-apercu.md`. Issue #626 (P2). Choix écrits le 2026-10-12 en autonomie : C-27-3-1 à 19 par
l'agent de spécification (Opus 5.5), C-27-3-20 à 27, 28 à 34 et 35 à 44 par les agents de remédiation des validations
P1, P2 et P3, C-27-3-45 par l'orchestrateur à la validation P4 ciblée, C-27-3-46 à 50 au développement, C-27-3-51 à 59
aux revues de code P1, P2 ciblée et P3 ciblée. Tenus d'abord dans un fichier séparé
(`epic-27-choix-autonomes-27-3.md`), versés ici **par union** au rebase sur la 27-2, sans renuméroter (préfixe
`C-27-3-`), puis le fichier séparé supprimé.

### C-27-3-1 — Changer de profil **relance** l'aperçu (plutôt que bloquer la confirmation)

- **Contexte** : l'issue laisse deux options — relancer l'aperçu, ou rendre la confirmation impossible tant qu'il
  n'est pas recalculé.
- **Retenu** : relancer aussitôt (AC1). Le geste de l'utilisateur dit ce qu'il veut voir ; l'aperçu coûte une
  requête sans écriture.
- **Écarté** : bloquer *Confirmer* avec un bouton « Recalculer » — un clic de plus sans information, et un état
  « aperçu périmé mais affiché » que l'utilisateur peut mal lire.
- **Réversible** : oui (écran seul).

### C-27-3-2 — Le déclencheur est le geste (`onChange`), jamais un `$effect` sur `bankProfileId`

- **Contexte** : `bankProfileId` est aussi affecté par le code (présélection KF #70, `:135-142`).
- **Retenu** : relance depuis le gestionnaire du sélecteur ; la présélection n'en déclenche aucune (AC4, mutation M2).
- **Écarté** : `$effect` — relancerait l'aperçu après chaque présélection (deux appels par dépôt, boucle possible).
- **Réversible** : oui.

### C-27-3-3 — Concurrence côté écran : numéro de séquence, sans anti-rebond ; sélecteurs actifs pendant l'aperçu, désactivés pendant la confirmation

- **Contexte** : certains navigateurs émettent `change` à chaque flèche sur un `<select>` fermé ; un compte changé ou *Annuler* pendant
  un aperçu en vol laissait déjà une réponse tardive s'afficher.
- **Retenu** : la dernière demande gagne, les réponses périmées sont ignorées (AC3) ; pendant une confirmation, les
  deux sélecteurs sont `disabled` (un changement à ce moment ne peut plus rien corriger).
- **Écarté** : anti-rebond (délai arbitraire, test plus fragile) ; désactiver les sélecteurs pendant l'aperçu
  (perte du focus clavier à chaque flèche).
- **Précisé à la validation P1** : *Annuler* n'est jamais rendu pendant un aperçu (il vit dans le bloc
  `{#if preview}`) — seul le changement de compte invalide un aperçu en vol ; *Annuler* est désactivé avec les
  sélecteurs pendant la confirmation (C-27-3-21).
- **Réversible** : oui.

### C-27-3-4 — Pendant le recalcul : ancien aperçu retiré, zone de dépôt masquée, texte « Recalcul… », cases remises à zéro

- **Contexte** : laisser l'ancien aperçu visible pendant le calcul montre des données qui ne seront pas importées ;
  la zone de dépôt (affichée dès que `preview` est nul) permettait un second dépôt concurrent.
- **Retenu** : AC2. Les cases de confirmation sont remises à zéro à chaque recalcul (prolonge F1 Pass 3).
- **Écarté** : ancien aperçu grisé (même risque de lecture, plus de code) ; conserver les cases cochées quand
  l'avertissement « survit » au recalcul (identité d'un avertissement difficile à établir, confirmation implicite).
- **Réversible** : oui.

### C-27-3-5 — Après un aperçu en échec, changer de profil relance avec le fichier retenu

- **Contexte** : aujourd'hui un refus (signe, aucun profil reconnu) oblige à redéposer ; c'est précisément le cas où
  l'on veut essayer un autre profil.
- **Retenu** : AC1-bis (sauf `BANK_ACCOUNT_REQUIRED`, où aucun fichier n'est retenu).
- **Écarté** : ne relancer que si un aperçu est affiché (lecture littérale de l'issue) — laisse le cas le plus utile
  de côté.
- **Réversible** : oui.

### C-27-3-6 — La confirmation part d'un instantané de l'aperçu affiché

- **Contexte** : `buildFlags()` lit l'état courant du sélecteur.
- **Retenu** : AC5 — profil **résolu** par l'aperçu, empreinte, fichier, compte, figés à la réception ; pour un CAMT,
  aucun `bankProfileId` (le serveur l'ignore déjà).
- **Écarté** : continuer à lire l'état courant en comptant sur la relance — un seul oubli et le défaut revient.
- **Réversible** : oui.

### C-27-3-7 — Défense serveur par **empreinte sans état** (SHA-256), ni table, ni jeton signé

- **Contexte** : l'issue #626 demande un écran juste (« changer de profil relance l'aperçu, ou rend la confirmation
  impossible tant que l'aperçu n'a pas été recalculé ») ; elle ne parle pas du serveur. La story **ajoute** une
  défense serveur, parce qu'un écran corrigé reste exposé à un profil modifié dans un autre onglet (D3) et qu'un client
  périmé ne doit pas importer avec un mapping jamais vu. *(Contexte reformulé à la validation P2, R13 : la première
  rédaction attribuait cette demande à l'issue, qui ne la contient pas.)*
- **Retenu** : l'aperçu rend `previewFingerprint`, la création la recalcule sur ses propres entrées et compare
  (AC6-AC8). Pas de migration, pas d'expiration, pas de nettoyage.
- **Écarté** : jeton stocké (migration, expiration, ne ferme rien de plus) ; HMAC (la menace est le client périmé,
  pas le client malveillant, qui a le droit d'importer) ; `updated_at` du profil (il ne couvre ni le compte ni le fichier ;
  la première rédaction invoquait une « résolution à la seconde », fausse : la colonne est un `DATETIME(6)`, validation P3, F2 = R3, C-27-3-39).
- **Réversible** : oui (champ ajouté, facultatif).

### C-27-3-8 — Contenu de l'empreinte : domaine versionné, société, compte (id + IBAN normalisé), hash du fichier, format, profil résolu complet hors horodatages

- **Contexte** : il faut couvrir tout ce qui change la lecture du fichier, et rien de ce qui n'y touche pas.
- **Retenu** : AC7. Le profil par **déstructuration exhaustive** de `BankProfile` (un champ ajouté casse la
  compilation). Renommer un profil (`bank_name`) entre l'aperçu et la confirmation rend donc `409` : conséquence
  assumée (rare, et suivie d'un recalcul automatique).
- **Écarté** : seulement les champs « de mapping » — liste à tenir à la main, qui se périme en silence (le motif
  « inventorier les non-résolus plutôt qu'énumérer les formes » du CLAUDE.md) ; sérialiser le `CsvProfile` de
  `kesh-import` (crate publiable, on n'y ajoute pas de dérivation pour cet usage).
- **Réversible** : oui (balise `v1` pour une évolution).

### C-27-3-9 — Empreinte **facultative** à la création

- **Contexte** : `POST /api/v1/bank-imports` est ouvert aux clés d'API (`docs/api-external.md` § 7 : toute route
  `/api/v1/*` de l'interface est consommable) ; les tests d'intégration créent sans aperçu.
- **Retenu** : absente → comportement actuel ; l'écran l'envoie toujours. Angle mort assumé D12 (client sans
  empreinte, dont un onglet chargé avant la mise à jour), écrit dans `docs/api-external.md`.
- **Écarté** : obligatoire — rupture d'API pour une protection qui vise l'écran.
- **Réversible** : oui (rendre obligatoire plus tard serait une rupture, à annoncer).

### C-27-3-10 — `409 BANK_IMPORT_PREVIEW_STALE` ; forme invalide ou doublon → `400` ; précédence après les refus antérieurs à la résolution du profil

- **Contexte** : les refus métier de l'import sont en `422` ; un conflit d'état entre lecture et écriture est un
  `409` ailleurs dans Kesh (conflits de version).
- **Retenu** : `409` ; comparaison CSV après `resolve_csv_profile`, sur le profil lu dans la transaction (aucune
  fenêtre avec le parse) ; `422 BANK_IMPORT_DUPLICATE_FILE` non confirmé reste prioritaire (AC8, I10).
- **Écarté** : `422` (ce n'est pas un défaut du fichier) ; déplacer la résolution du profil avant la détection de
  doublon (change l'ordre des refus existants).
- **Réversible** : oui.

### C-27-3-11 — Sur `409`, l'écran recalcule et l'annonce, sans reconfirmer seul

- **Contexte** : profil modifié dans un autre onglet (D3).
- **Retenu** : AC9 — recalcul automatique + avis `role="status"` ; l'utilisateur reconfirme.
- **Écarté** : afficher l'erreur seule (l'utilisateur devrait deviner qu'il faut redéposer) ; reconfirmer
  automatiquement après recalcul (importerait sans que l'utilisateur ait vu le nouvel aperçu — le défaut même).
- **Réversible** : oui.

### C-27-3-12 — L'état de la base (doublons, fichier déjà importé) reste hors empreinte

- **Contexte** : un autre import entre l'aperçu et la confirmation change les doublons.
- **Retenu** : la création les recalcule déjà et refuse un fichier déjà importé non confirmé ; les lignes doublons
  sont ignorées par défaut. Les inclure ferait rougir l'empreinte pour un événement que la création traite déjà.
- **Réversible** : oui.

### C-27-3-13 — CAMT.053 : même mécanisme, profil absent de l'empreinte ; changer de profil relance aussi

- **Contexte** : le profil n'a pas d'effet sur un CAMT.
- **Retenu** : empreinte rendue et vérifiée pour les deux formats (compte et fichier comptent) ; relance uniforme
  côté écran (une requête sans effet, mais aucune branche par format à tenir).
- **Écarté** : ne relancer que pour un CSV — exigerait de connaître le format côté écran avant l'aperçu.
- **Réversible** : oui.

### C-27-3-14 — Manuel : l'encadré C-26-3-15 est réécrit ; les « redéposez le fichier » du refus de signe restent

- **Contexte** : `user-manual.tex:1743`, `:1784` et le message serveur `amount-sign-refused-profile` disent de régler
  le profil puis de redéposer.
- **Retenu** : inchangés — le profil se modifie dans *Administration*, hors de l'écran d'import, et re-choisir la
  même valeur du sélecteur n'émet aucun `change`. Seuls `:1697-1698`, `:1794-1796` et `:1801-1807` changent (AC12).
- **Réversible** : oui.

### C-27-3-15 — Aucune migration, aucun bump de version

- **Retenu** : AC15 ; la release 0.13.2 fera le bump Cargo. Pas de `min_required` (aucun changement de schéma).

### C-27-3-16 — L'empreinte vit dans un module neuf, hors de `bank_imports.rs`

- **Contexte** : la 27-1 modifie `bank_imports.rs` et `errors.rs`.
- **Retenu** : `routes/bank_import_fingerprint.rs` (fonction + tests unitaires) ; dans `errors.rs`, variante et
  mapping placés près de `BankCsvProfileNotFound`, loin des hunks de la 27-1.
- **Réversible** : oui.

### C-27-3-17 — E2E : contenu dérivé en mémoire de la fixture UBS synthétique

- **Contexte** : `ubs_synthetique.csv` est importée par un test 26-1 de la même spec : la réutiliser telle quelle
  déclenche « fichier déjà importé » et l'ignorance des 11 lignes en doublon.
- **Retenu** : `buffer` de `setInputFiles`, année des dates remplacée (2027 pour E1, 2028 pour E2), nom de fichier
  propre. Aucune donnée réelle (relevé de Guy hors de portée, fixture synthétique du dépôt).
- **Écarté** : nouveau fichier de fixture (copie qui dérive de l'original).
- **Réversible** : oui.

### C-27-3-18 — Pas de trace d'audit pour le refus `409`

- **Retenu** : comme les autres refus de l'import (aucune écriture, aucune trace) ; I4 vérifie l'absence.
- **Réversible** : oui.

### C-27-3-19 — Trois clés i18n seulement ; le « Chargement… » en dur reste hors périmètre

- **Contexte** : `BankImportUpload.svelte:305` porte un « Chargement… » non traduit (et d'autres libellés en dur du
  même écran : « Ligne », « Code », « Valeur », les intitulés de l'aperçu).
- **Retenu** : ne pas les traiter ici (dette d'i18n de l'écran, distincte du défaut) ; trois clés neuves (AC10),
  `sitesTotal` +2 (AC11).
- **Réversible** : oui. **À signaler à l'orchestrateur** : issue P4 éventuelle pour les libellés en dur de l'écran
  d'import.
- **Révisé à la validation P1 (2026-10-12)** : sur consigne de l'orchestrateur (le nombre de bugs doit baisser à la
  release, pas de nouvelle issue), les libellés en dur de l'écran d'import sont **traduits dans cette story** —
  C-27-3-25. Ce choix-ci reste pour l'historique ; la proposition d'issue P4 est retirée.

## Validation P1 (2026-10-12) — choix de remédiation

### C-27-3-20 — `reset()` éteint les indicateurs et l'avis, en plus d'invalider la séquence

- **Contexte** : F1 (HIGH) / R3. Une réponse périmée n'éteint plus l'indicateur (AC3) ; si `reset()` (changement de
  compte pendant un aperçu) ne le fait pas non plus, `isPreviewing` reste vrai, la zone de dépôt reste masquée
  (AC2) et l'écran est bloqué jusqu'au rechargement.
- **Retenu** : `reset()` remet `isPreviewing` et `isConfirming` à faux et efface l'avis de l'AC9 ; V8 asserte
  l'indicateur absent et la zone présente, dès le changement et après la réponse tardive ; mutation M19.
- **Écarté** : éteindre l'indicateur dans le `finally` de toute réponse (rallume *Confirmer* pendant qu'un aperçu
  plus récent est en vol — le piège déjà écrit aux Dev Notes).
- **Réversible** : oui.

### C-27-3-21 — Pas d'*Annuler* pendant un aperçu ; *Annuler* désactivé pendant la confirmation

- **Contexte** : F2 / R1 — D11 et V10 supposaient un *Annuler* pendant un aperçu en vol, or le bouton n'est rendu
  qu'avec un aperçu (`:354`) que l'AC2 retire. F5 / R6 — pendant une confirmation, *Annuler* reste actif ; suivi d'un
  `409`, il ferait réapparaître l'aperçu après l'annulation.
- **Retenu** : D11 réécrit « inatteignable » (la sortie pendant un aperçu est le changement de compte, D5) ; V10 et la
  mention de V10 en M3 retirées, V10 réaffecté (C-27-3-22) ; *Annuler* `disabled` pendant la confirmation, comme les
  sélecteurs (AC3, D15, V9, M8).
- **Écarté** : rendre un *Annuler* dans l'état de chargement (un site d'invalidation de plus à tester, pour un
  aperçu qui dure une requête sans écriture) ; garder *Annuler* actif et garder la relance `409` par un jeton de
  séquence capturé au début de la confirmation (deux mécanismes là où un `disabled` suffit : une fois les trois
  contrôles désactivés, aucun geste **sur l'écran** n'invalide plus la confirmation).
- **Précisé à la validation P2** (R14) : la première rédaction disait « plus aucun geste ne peut invalider la
  confirmation » ; **quitter la page** en reste un (démontage du composant, D19) — inoffensif, assumé (C-27-3-34).
- **Réversible** : oui.

### C-27-3-22 — M6 est tuée par un test CAMT, et non par un test serveur

- **Contexte** : F3 / R2 — M6 renvoyait à I5, test Rust qui ne peut pas tuer une mutation du Svelte (erreur de
  catégorie), et à une variante de V1 contradictoire avec l'AC2. Sélecteur et instantané ne divergent que pour un
  CAMT (l'instantané n'envoie aucun profil).
- **Retenu** : V10 réécrit — aperçu sans `csvProfileMatch`, sélecteur sur A, *Confirmer* → `createBankImport` sans
  `bankProfileId`.
- **Écarté** : déclarer M6 équivalente (elle ne l'est pas : le CAMT la distingue).
- **Réversible** : oui.

### C-27-3-23 — M16 en deux ; la fenêtre transactionnelle est tenue par construction, non par un test

- **Contexte** : F4 / R4 — la première forme de M16 (comparaison avant la résolution, profil relu hors transaction)
  n'était tuée par aucun test, et la précédence de l'AC8 n'était testée qu'à moitié.
- **Retenu** : I11 (profil supprimé → `404`, non `409`) et I12 (compte inconnu → `404`) ; M16a (comparaison déplacée
  avant la résolution, un profil supprimé rend `409`) → I11 ; M16b (empreinte sur l'id demandé) → I6 — **faux, corrigé à
  la validation P2** : I6 ne tue pas M16b, M16b → I6-bis (C-27-3-29). La variante
  « relu hors transaction mais présent » est écrite **équivalente** en test déterministe, non comptée, tenue par
  construction (une seule variable `profile` comparée puis parsée).
- **Écarté** : un test de concurrence (injection d'un `UPDATE` entre lecture et parse : outillage absent, test
  fragile pour une fenêtre que le code ferme par construction).
- **Réversible** : oui.

### C-27-3-24 — Précédence asymétrique entre CSV et CAMT : conservée, écrite et testée

- **Contexte** : F7 / R11 — CSV : `422 DUPLICATE_FILE` avant `409` ; CAMT : `409` avant `422 DUPLICATE_FILE` (la
  détection de doublon CAMT se fait dans la transaction, la comparaison avant).
- **Retenu** : conserver ; écrire l'ordre à `docs/api-external.md` (AC13) ; I10 teste le cumul pour les deux formats.
- **Écarté** : déplacer la comparaison CAMT dans la transaction après la détection de doublon (aligne l'ordre, mais
  ouvre une transaction pour rien sur un import refusé) ; aucune incidence fonctionnelle à aligner.
- **Réversible** : oui.

### C-27-3-25 — Les libellés en dur de l'écran d'import sont traduits dans cette story

- **Contexte** : C-27-3-19 les laissait hors périmètre ; l'orchestrateur demande de les traduire ici, dans les quatre
  locales, sans ouvrir d'issue.
- **Retenu** : AC17 — périmètre l'écran `/bank-import` (page et trois composants), inventaire clos H1-H11 avec ses
  non-résolus assumés (symboles, codes bruts, gabarit `bank-import-info-*`) ; textes fr-CH repris à l'identique ; deux
  clés globales réutilisées (`common-loading`, `error-unexpected`, autorisées par `lint-i18n-ownership`), 21 clés
  neuves, une clé partagée entre l'aperçu et la liste (`bank-import-labels-period`) ; `sitesTotal` 2032 → 2063, 2072
  à la fusion avec la 27-1 — **livré : 2062, et 2071 à la fusion** (C-27-3-46 ; propagé en revue de code P1, A2) ; 2073 rebasée sur la 27-2.
- **Écarté** : traduire aussi la fiche d'import `/bank-import/{id}` (une quinzaine de libellés en dur) — autre écran,
  modifié par la 27-1 au bloc adjacent : conflit certain. **Signalé à l'orchestrateur**, qui décide (après la fusion
  de la 27-1 : une story ou une issue). Reformuler les textes fr (« Solde d'ouverture », « relevé(s) ») — hors du
  défaut, et ferait mentir des captures.
- **Réversible** : oui.

### C-27-3-26 — `handleFileInput` vide `input.value`

- **Contexte** : R12 — l'`<input type=file>` n'est jamais vidé ; après un refus, la zone reste montée et re-choisir
  le même fichier n'émet pas `change`, alors que le manuel et le message serveur disent « redéposez le fichier ».
  Constat raisonné, non exécuté (la validation est en lecture seule) ; l'AC2 démonte la zone pendant tout aperçu,
  ce qui le ferme en pratique, mais par accident de rendu.
- **Retenu** : AC16 — une ligne, `input.value = ''` après lecture du fichier ; V12 (espion du setter, `user-event`
  absent) ; mutation M17.
- **Écarté** : s'en remettre au démontage de l'AC2 (une évolution du rendu rouvrirait le défaut en silence).
- **Réversible** : oui.

### C-27-3-27 — Le titre et le CHANGELOG disent « depuis l'écran »

- **Contexte** : R9 — « le serveur refuse un import jamais prévisualisé » sur-promet : un client sans empreinte
  importe (D12).
- **Retenu** : titre « … refuse un import dont le profil a changé depuis l'aperçu » ; CHANGELOG « depuis l'écran » et
  « facultatif pour un client d'API ». Le nom du fichier de la fiche ne change pas.
- **Réversible** : oui. *(Titre, énoncé et CHANGELOG élargis au compte et au fichier à la validation P3 : C-27-3-44.)*

### C-27-3-28 — L'avis du `409` est neutre : « Le profil, le fichier ou le compte a changé »

- **Contexte** : validation P2, F1 = R2 — l'avis prescrit (« Le profil a été modifié depuis la prévisualisation… »)
  imputait tout `409` au profil. Or, depuis l'écran, l'instantané fige l'objet `File` et l'id du compte, et un `409`
  peut venir du contenu du profil (D3), de l'IBAN du compte modifié ailleurs (D7) ou des octets du fichier modifié sur
  le disque (D18) ; pour un CAMT, le profil n'est pas dans l'empreinte, l'avis y était faux à chaque fois. Le message
  serveur, lui, disait déjà « Le profil, le fichier ou le compte a changé ».
- **Retenu** : avis aligné sur le message serveur, dans les quatre locales et le repli (AC9, AC10) ; le texte de
  recalcul perd « avec le profil choisi » (« Recalcul de la prévisualisation… »), faux lui aussi après un `409` sur un
  CAMT ; le manuel (AC12) cite le profil, le compte et le fichier.
- **Écarté** : un avis par cause (l'écran ne sait pas laquelle des entrées a changé — il faudrait que le serveur
  rende le détail de l'empreinte, ce qui la rendrait structurée pour un gain d'affichage) ; un avis « profil » pour
  un CSV et « compte » pour un CAMT (plus de code, et faux pour le fichier modifié).
- **Réversible** : oui (textes seuls). *(Validation P3 : le texte de l'avis est réécrit pour rester vrai pendant et après un recalcul en échec — C-27-3-36 ; le « fichier modifié » n'est un `409` que sous un navigateur qui relit le fichier — C-27-3-35.)*

### C-27-3-29 — M16b est tuée par I6-bis, création sans `bankProfileId`

- **Contexte** : validation P2, F2 = R1 (régression de la remédiation P1) — M16b (« empreinte sur l'id demandé au
  lieu du profil résolu ») était affectée à I6, où la création envoie l'id résolu : demandé et résolu y sont le même
  profil, la mutation est équivalente dans I6, qui reste vert.
- **Retenu** : I6-bis — aperçu et création sur *Auto-détection*, aucun id des deux côtés, un seul profil concordant,
  empreinte de l'aperçu → `201` ; sous la mutation, l'empreinte de la création porte `null` à la place du profil →
  `409`. M16b réécrite précisément (`fields.bank_profile_id.and_then(<relecture>)`, `null` sans id). I6 reste, pour
  l'AC8 (auto-détection puis explicite). Helper existant `build_multipart_with_text_fields` (F8).
- **Écarté** : muter aussi l'aperçu (la mutation ne porterait plus sur le site de création qu'elle doit éprouver).
- **Réversible** : oui (tests).

### C-27-3-30 — La garde de séquence est éprouvée sur une réponse périmée en vol, et sur une réponse en erreur

- **Contexte** : validation P2, R3 (régression de la remédiation P1, V3 réécrit) — dans l'ordre C, A, B, la réponse
  la plus récente arrive la première : aucune réponse périmée n'arrive pendant qu'un aperçu plus récent est en vol, si
  bien qu'un `finally` qui éteint l'indicateur sans condition survit ; et V3 / V8 n'employaient que des réponses
  réussies, si bien qu'une garde posée sur la seule branche de succès survivait aussi.
- **Retenu** : V3 dans l'ordre A, C, B (indicateur présent et aucun aperçu après A) ; variante rejetée de V8 (ni
  erreur, ni indicateur) ; mutations M21 et M22 ; T3 écrit qu'une réponse périmée, succès ou erreur, ne touche ni
  `preview`, ni l'erreur, ni `isPreviewing`.
- **Écarté** : un test par permutation des trois réponses (six ordres) — A, C, B est le seul qui fasse arriver une
  réponse périmée avant la plus récente **et** une autre après.
- **Réversible** : oui (tests).

### C-27-3-31 — L'avis s'efface en trois sites, chacun tenu par un test

- **Contexte** : validation P2, R4 — l'AC9 promet que l'avis disparaît au geste suivant, mais la branche `409` ne
  passe pas par `reset()` et T3 ne prescrivait que l'effacement par `reset()` ; aucun test ni mutation ne tenait la
  disparition.
- **Retenu** : trois sites prescrits (T3) — `reset()` (compte, *Annuler*), `runPreview` pour toute raison autre que
  `'stale'` (dépôt, profil), `handleConfirm` dès le clic ; V6-bis (un rendu par geste) ; mutation M20.
- **Écarté** : un `$effect` qui efface l'avis à tout changement de `preview` (effacerait l'avis à l'arrivée même du
  recalcul qu'il annonce) ; un délai d'expiration (arbitraire, l'utilisateur peut ne pas l'avoir lu).
- **Réversible** : oui.

### C-27-3-32 — L'annotation « (auto-détecté) » n'apparaît que pour un aperçu `autoMatched`

- **Contexte** : validation P2, F9 — `BankImportUpload.svelte:252` alimente l'annotation par
  `csvProfileMatch.profileId`, que `preview_csv` pose toujours pour un CSV : un profil choisi avant le dépôt est déjà
  annoté « (auto-détecté) » à tort, et l'AC1, qui relance l'aperçu à chaque choix, rendrait l'erreur systématique.
- **Retenu** : `autoMatchedId` vaut l'id seulement si `csvProfileMatch.autoMatched` (AC4) ; V1 (pas d'annotation sur
  B), V4 (annotation sur le présélectionné) ; mutation M23. La phrase du manuel (« mention « auto-détecté » » sous
  *Auto-détection*) reste vraie et devient exacte.
- **Écarté** : laisser tel quel (défaut préexistant, mais que la story aggrave) ; une issue séparée (correctif d'une
  ligne dans un fichier que la story réécrit).
- **Réversible** : oui.

### C-27-3-33 — Sur `409`, `isConfirming` s'éteint avant le recalcul

- **Contexte** : validation P2, F5 — si la branche `409` attend le recalcul (`await runPreview(…)`) avant le
  `finally` qui éteint `isConfirming`, les sélecteurs restent désactivés pendant tout le recalcul, contre l'AC3
  (« pendant un aperçu, les sélecteurs restent actifs »), et un recalcul qui traîne fige l'écran.
- **Retenu** : `isConfirming = false` puis `void runPreview(…, 'stale')` (AC9, T3) ; assertion dans V6 (sélecteurs
  actifs pendant le recalcul retenu) ; mutation M24.
- **Écarté** : garder l'`await` et désactiver délibérément pendant le recalcul (aucune raison : le recalcul est un
  aperçu comme un autre, protégé par la séquence).
- **Réversible** : oui.

### C-27-3-34 — D18 (fichier modifié sur le disque) et D19 (navigation pendant la confirmation) : écrits, sans code neuf

- **Contexte** : validation P2, F3 et R14 — l'inventaire D1-D17 se disait clos sans le fichier source modifié entre
  l'aperçu et la confirmation (l'objet `File` est lu paresseusement) ni le démontage du composant pendant une
  confirmation ; F3 relève aussi la suppression d'un import par la 27-1, de la classe D13.
- **Retenu** : D18 fermé par l'empreinte existante (hash du fichier, AC7-4 → `409`, recalcul, avis neutre
  C-27-3-28) ; D19 inoffensif et assumé — une requête d'aperçu sur un composant démonté, sans effet visible ni
  écriture —, sans garde de démontage ; D13 nomme la suppression d'un import.
- **Écarté** : une garde de démontage (`onDestroy` qui invalide la séquence) — un site de plus à tester pour une
  requête sans écriture ; relire le fichier avant la confirmation pour comparer côté client (le serveur le fait déjà).
- **Réversible** : oui. *(Validation P3, F1 = R1 : « D18 fermé » ne valait que côté serveur ; sous un navigateur qui échoue l'envoi, aucun `409`, aucun recalcul — révisé par C-27-3-35.)*

### C-27-3-35 — D18 : le recalcul n'est promis que pour le profil et le compte ; pour le fichier, le comportement dépend du navigateur

- **Contexte** : validation P3, F1 = R1 (MEDIUM, régression de la remédiation P2) — l'AC12 promettait au manuel que
  Kesh « recalcule la prévisualisation et l'annonce » quand le fichier est modifié sur le disque, et C-27-3-34
  déclarait D18 « fermé par l'empreinte ». Or la ligne D18 elle-même écrit que Chrome échoue l'envoi en
  `ERR_UPLOAD_FILE_CHANGED` : la requête ne part pas, `toFetchApiError` rend `NETWORK_ERROR`
  (`api-client.ts:287-296`), il n'y a ni `409`, ni recalcul, ni avis ; *Confirmer* redevient actif et chaque clic
  échoue de même, l'objet `File` de l'instantané restant celui de l'aperçu.
- **Retenu** : l'AC12 ne promet le recalcul que pour le profil et le compte modifiés ailleurs ; pour le fichier, il
  écrit « selon le navigateur » les deux issues — recalcul, ou erreur de connexion répétée — et la sortie (*Annuler*,
  puis redéposer). D18 distingue « serveur : fermé » (rien n'est importé dans les deux cas) et « écran : non fermé,
  assumé ». L'AC9 cite D18 comme source de `409` seulement pour un navigateur qui relit le fichier, et nomme
  l'erreur réseau. Le comportement des navigateurs reste un constat **raisonné, non reproduit** : le texte prescrit
  est vrai quel que soit celui des deux qu'adopte un navigateur donné.
- **Écarté** : un message dédié à l'erreur réseau pendant une confirmation (« le fichier a peut-être changé ») — il
  faudrait distinguer `ERR_UPLOAD_FILE_CHANGED` d'une vraie panne, ce que `fetch` ne permet pas (le `TypeError` ne
  porte pas la cause) ; relire le fichier avant d'envoyer pour comparer son hash à celui de l'aperçu (lecture complète
  côté client, jusqu'à 10 Mio, pour un cas rare et sans risque de données) ; retirer « le fichier » de l'avis neutre
  (faux sous Firefox, qui produit bien un `409`).
- **Réversible** : oui (textes ; un message dédié peut s'ajouter plus tard).

### C-27-3-36 — L'avis du `409` dit « Kesh la recalcule. Vérifiez le résultat » : vrai pendant, après et en cas d'échec

- **Contexte** : validation P3, R2 = F3 — « elle a été recalculée. Vérifiez-la avant de confirmer » est affiché dès
  la réception du `409`, donc pendant le recalcul (à côté de « Recalcul de la prévisualisation… »), et reste affiché
  quand le recalcul échoue (E2), où il n'y a aucune prévisualisation à vérifier.
- **Retenu** : « … a changé depuis la prévisualisation : Kesh la recalcule. Vérifiez le résultat avant de
  confirmer. » (présent, et « le résultat » couvre le nouvel aperçu comme l'erreur), dans les quatre locales et le
  repli (AC9, AC10).
- **Écarté** : afficher l'avis seulement après le succès du recalcul — il disparaîtrait justement dans le cas E2, où
  l'utilisateur a le plus besoin de savoir pourquoi l'écran a changé ; deux clés (constat / suite) — un site et une
  garde de plus pour le même effet.
- **Réversible** : oui (texte).

### C-27-3-37 — La remise à zéro des cases sur la voie `409` est tenue par une variante de V6 et par M26

- **Contexte** : validation P3, R6 — l'AC2 remet les cases à zéro « au dépôt initial ou au recalcul », mais seul le
  changement de profil était éprouvé (V7) ; une mutation qui épargne la raison `'stale'` survivait, et le paramètre
  `reason` rend précisément cette divergence plausible.
- **Retenu** : variante « cases » de V6 (`duplicateFile`, case cochée, `409`, case décochée après le recalcul) ;
  mutation M26 ; T3 écrit « pour toute raison, `'stale'` comprise ».
- **Écarté** : un test neuf numéroté (la variante tient dans le montage de V6).
- **Réversible** : oui (tests).

### C-27-3-38 — Le troisième site d'effacement de l'avis a sa mutation ; `isConfirming` dans `reset()` est un angle mort déclaré

- **Contexte** : validation P3, R5 = F7 — M20 ne rougit que V6-bis (a) et (b) ; rien n'éprouvait que V6-bis (c)
  rougirait si `reset()` n'effaçait plus l'avis. Et `reset()` remet `isConfirming` à faux alors qu'elle n'est
  appelable pendant une confirmation que par le succès, dont le `finally` le fait déjà.
- **Retenu** : mutation M25 (`reset()` n'efface plus l'avis) → V6-bis (c) ; l'extinction d'`isConfirming` reste dans
  `reset()`, écrite **défensive, non éprouvée** (AC3, T3) — angle mort assumé, aucune mutation ne la vise.
- **Écarté** : retirer l'assignation (une future voie d'appel de `reset()` pendant une confirmation — un nouveau bouton,
  un raccourci — laisserait l'écran figé) ; un test qui appellerait `reset()` de l'extérieur (le composant ne
  l'expose pas, l'ouvrir pour un test changerait son interface).
- **Réversible** : oui.

### C-27-3-39 — D17 sans « précision à la seconde » : `updated_at` est un `DATETIME(6)`

- **Contexte** : validation P3, F2 = R3 — `bank_profiles.updated_at` est `DATETIME(6) … ON UPDATE
  CURRENT_TIMESTAMP(6)` (`20260505000001_bank_profiles.sql:34`, aucune migration ultérieure ne la touche) ; le sous-cas
  « deux profils concordants de même `updated_at` » de D17 et l'argument d'écart de C-27-3-7 reposaient sur une
  résolution à la seconde, fausse. F2 relève aussi un cas voisin non nommé : aperçu sur un profil explicite B,
  création avec l'empreinte mais sans `bankProfileId`.
- **Retenu** : D17 et AC13 retirent le sous-cas (non retenu, écrit comme tel), nomment le cas « aperçu explicite,
  création sans id » (`409` ou `404`) ; C-27-3-7 corrigé (l'écart d'`updated_at` tient : il ne couvre ni le compte ni
  le fichier).
- **Écarté** : garder le sous-cas « à la microseconde » dans la doc d'API publique (une phrase sur un cas
  inatteignable hors `UPDATE` multi-lignes y ferait plus de bruit que de bien).
- **Réversible** : oui (textes).

### C-27-3-40 — Pour un CAMT, changer de profil relance quand même l'aperçu

- **Contexte** : validation P3, F8 — l'aperçu d'un CAMT ignore le profil ; le relancer au changement de profil
  produit un aperçu identique et remet les cases à zéro.
- **Retenu** : assumé, écrit à l'AC1. Aucun risque de données ; une requête et des cases à recocher pour rien.
- **Écarté** : ne pas relancer quand l'aperçu affiché n'a pas de `csvProfileMatch` — une branche de plus dans le
  geste, à tester et à muter, et qui devrait ménager l'AC1-bis (aperçu en échec, format inconnu de l'écran).
- **Réversible** : oui (écran seul).

### C-27-3-41 — La raison du dernier aperçu vit dans un état `previewReason`, posé par chaque `runPreview`

- **Contexte** : validation P3, F6 — le texte de l'indicateur dépend de la raison (recalcul ou dépôt), que le gabarit
  ne peut lire que dans un `$state`, que T3 ne nommait pas.
- **Retenu** : `previewReason` (`'drop' | 'profile' | 'stale'`), posé par chaque appel de `runPreview` avant son
  premier `await` ; jamais lu périmé, donc `reset()` n'y touche pas. Texte tenu par V2 et V13 (b). *(Amendé par C-27-3-45 : `isConfirming` prime sur `previewReason` ; V9 tient la confirmation.)*
- **Écarté** : le remettre à zéro dans `reset()` (une écriture de plus sans effet observable — c'est l'appel suivant
  qui le pose).
- **Réversible** : oui.

### C-27-3-42 — Manuel : `make user`, et seul `user-manual.pdf` est committé

- **Contexte** : validation P3, R7 = F10 — `make fr` reconstruit les trois PDF (`Makefile:44`) ; la 27-1 modifie
  `admin-manual.pdf` et `marketing-brochure.pdf` et remplace les deux passes fixes du `Makefile` par des passes
  jusqu'à stabilisation (C-27-1-27). Le tableau des fichiers communs omettait le `Makefile` et portait des chiffres
  périmés.
- **Retenu** : `make user` (cible existante, `Makefile:48`), seul `user-manual.pdf` committé ; table des matières du
  PDF contrôlée (ancien `Makefile`) ; régénération après la fusion. Tableau des fichiers communs relevé au commit
  `3c36f721` de la 27-1, à recompter à la fusion.
- **Écarté** : porter ici le `Makefile` de la 27-1 (deux branches qui modifient le même fichier pour le même effet :
  conflit garanti) ; `make fr` puis `git checkout` des deux autres PDF (geste de plus, facile à oublier).
- **Réversible** : oui.

### C-27-3-43 — M10 en deux : la comparaison CAMT a sa propre mutation

- **Contexte** : validation P3, F9 — M10 « comparaison retirée » n'était affectée qu'à des tests CSV ; retirer la
  seule comparaison de `create` (CAMT) n'était tuée que par I10, non cité.
- **Retenu** : M10a (CSV → I4, I5, I7, E2), M10b (CAMT → I10). Vingt-huit mutations en tout (avec M25, M26) — vingt-neuf depuis M27 (C-27-3-45).
- **Écarté** : une ligne M10 citant aussi I10 (une mutation qui retire les deux comparaisons à la fois ne dit pas
  laquelle des deux un test tient).
- **Réversible** : oui (tests).

### C-27-3-44 — Titre, énoncé et CHANGELOG nomment le profil, le compte et le fichier

- **Contexte** : validation P3, F4 — l'avis, le message serveur et le manuel sont neutres (C-27-3-28), mais le titre
  (repris dans la PR), l'énoncé et le CHANGELOG prescrit imputaient encore le refus au seul profil.
- **Retenu** : « … refuse un import dont le profil, le compte ou le fichier a changé depuis l'aperçu » ; énoncé et
  CHANGELOG idem. C-27-3-27 (« depuis l'écran ») reste.
- **Écarté** : laisser le titre sur le seul profil au motif qu'il énonce le défaut de l'issue (la PR le reprend tel
  quel, et il promettrait moins que ce qui est livré).
- **Réversible** : oui.

### C-27-3-45 — L'indicateur donne la priorité à la confirmation

- **Contexte** : validation P4 ciblée, P4-1 (Haiku, vérifié par l'orchestrateur) — le texte de l'indicateur était lu
  dans `previewReason` seul, qui n'a pas de valeur « confirmation » et garde sa valeur après l'aperçu : après un
  changement de profil, *Confirmer* aurait affiché « Recalcul de la prévisualisation… » pendant l'import (AC2, V9).
- **Retenu** : `isConfirming` d'abord (→ `common-loading`), `previewReason` ensuite. V9 se joue après un recalcul ;
  mutation M27 (priorité retirée) tuée par V9.
- **Écarté** : une quatrième valeur `'confirm'` dans `previewReason` (deux états diraient la même chose, et
  `reset()` devrait la remettre à zéro).
- **Réversible** : oui.

### C-27-3-46 — Les deux `catch` partagent `setError` : un site `error-unexpected` au lieu de deux

- **Contexte** : développement, AC17 (H3) et AC11 — la fiche comptait deux sites pour `Erreur inattendue.` (`:150`, `:185`), un par `catch`. Le refactor de T3 sépare `runPreview` de `handleConfirm`, et les deux posaient l'erreur de la même façon (code, message, lignes du refus de signe).
- **Retenu** : une fonction `setError(err)` appelée par les deux `catch` (DRY, CLAUDE.md) ; `sitesTotal` 2032 → **2062** (+30, et non +31), écart ventilé au commentaire de `i18n-keys.test.ts`. V13 (c) tient toujours les deux chemins (aperçu et confirmation rejetés par une erreur qui n'est pas une `ApiError`).
- **Écarté** : garder deux blocs identiques pour retrouver le nombre de la fiche (duplication pour un compteur).
- **Réversible** : oui. À la fusion avec la 27-1 : 2032 + 30 + 9 = **2071** (et non 2072) ; avec la 27-2 aussi (+2) : **2073**, confirmé par la garde au rebase.

### C-27-3-47 — Le recalcul d'un `409` demande le profil de l'instantané, explicitement

- **Contexte** : développement, AC9 — « même fichier, même compte, profil de l'instantané ». Pour un CSV, l'instantané porte le profil **résolu** (`csvProfileMatch.profileId`), même si l'aperçu avait été demandé sur *Auto-détection*.
- **Retenu** : `runPreview(snap.file, snap.accountId, snap.bankProfileId ?? null, 'stale')` — l'id résolu est envoyé explicitement. Conséquence assumée : la réponse n'est plus `autoMatched`, et l'annotation « (auto-détecté) » ne réapparaît pas après un tel recalcul (le profil reste sélectionné). Pour un CAMT, aucun profil (l'aperçu l'ignore).
- **Écarté** : relancer avec `bankProfileId` du sélecteur (égal pour un CSV pendant une confirmation, puisque les sélecteurs sont désactivés, mais contraire à la lettre de l'AC9 et dépendant de l'état de l'écran) ; relancer sur *Auto-détection* quand l'aperçu l'était (un autre profil concordant pourrait passer en tête, D17).
- **Réversible** : oui.

### C-27-3-48 — `parse_fingerprint_field` ne tolère aucun blanc

- **Contexte** : développement, AC6 — les autres champs multipart sont lus après `trim()` (`bankAccountId`, drapeaux). L'empreinte est produite par le serveur et renvoyée telle quelle.
- **Retenu** : forme stricte (64 caractères `[0-9a-f]`, sans `trim`) : un blanc autour est un défaut du client, signalé en `400`. U4 le tient (cas « espace de tête »).
- **Écarté** : `trim()` par parité avec les autres champs (une empreinte n'est jamais saisie à la main).
- **Réversible** : oui.

### C-27-3-49 — Une troisième fonction au module : `check_preview_fingerprint`

- **Contexte** : développement, T1 — la fiche prescrit `preview_fingerprint` et `parse_fingerprint_field`. La comparaison (« absente → accepter ; différente → 409 ») s'écrit aux deux créations (CSV, CAMT).
- **Retenu** : `check_preview_fingerprint(sent, recomputed)` dans le module neuf, appelée par `create` et `create_csv` ; un test unitaire la couvre. Les sites de `bank_imports.rs` restent des appels à chemin complet (`crate::routes::bank_import_fingerprint::…`), sans toucher au bloc `use` que modifie la 27-1.
- **Écarté** : deux `match` écrits en place (la même règle deux fois) ; un `use` en tête de `bank_imports.rs` (hunk dans le bloc d'imports de la 27-1).
- **Réversible** : oui.

### C-27-3-50 — V3 joue la course sur *Auto-détection*, B, C

- **Contexte** : développement, V3 — « A → B → C rapides ». Le montage commun des tests affiche d'abord l'aperçu du profil A ; re-choisir A n'émettrait pas `change`.
- **Retenu** : la course part d'*Auto-détection* (le premier des trois), puis B, puis C, réponses dans l'ordre premier, C, B — l'ordre exigé par C-27-3-30 (une réponse périmée arrivant pendant qu'une plus récente est en vol). Trois profils au mock (A, B, C).
- **Écarté** : un montage propre à V3 sans aperçu initial (il n'exercerait pas le remplacement d'un aperçu affiché).
- **Réversible** : oui (test seul).

### C-27-3-51 — Après un `409`, le sélecteur relit la liste des profils ; l'avis reste avec l'erreur

- **Contexte** : revue de code P1, B-1 (LOW) — la liste des profils n'était chargée qu'au montage : un profil renommé, supprimé ou créé ailleurs gardait son ancien état dans le sélecteur, précisément dans le cas que vise le `409`. Deux corrections proposées : effacer l'avis quand le recalcul `'stale'` échoue, ou relire les profils à la branche `409`.
- **Retenu** : la relecture — `loadProfiles()`, partagée par `onMount` et la branche `409` (non attendue, comme le recalcul). Un échec de la relecture **garde** la liste en place (au montage, elle est vide : comportement inchangé), pour qu'un rafraîchissement raté ne fasse pas disparaître le sélecteur. Tenu par V6-ter et sa variante, mutations M30 et M31. Le manuel le dit (« relit la liste des profils »), avec le cas du recalcul en échec (« le message d'erreur s'affiche avec l'avis : choisissez un autre profil »).
- **Écarté** : effacer l'avis quand le recalcul échoue — contraire à l'AC9 et à C-27-3-36 (l'avis est vrai dans les trois états, recalcul en échec compris), et V6 (variante) l'interdit ; relire les profils à **tout** recalcul (requête de plus à chaque geste, sans cas d'usage : hors `409`, rien ne signale un changement ailleurs).
- **Réversible** : oui.

### C-27-3-52 — « L'IBAN du compte » au manuel et au CHANGELOG ; l'avis de l'écran inchangé

- **Contexte** : revue de code P1, E-8 = A4 (LOW) — le manuel promettait un refus pour « le compte bancaire modifié ailleurs », alors que l'empreinte ne voit du compte que son id et son IBAN normalisé (U3 : nom de banque, description, version sans effet). Le CHANGELOG disait de même « un compte … modifié ».
- **Retenu** : manuel et CHANGELOG nomment l'IBAN ; le manuel ajoute que les autres changements du compte (nom de la banque, description) ne bloquent pas l'import. L'avis `bank-import-labels-preview-recomputed` et le message `bank-import-errors-preview-stale` (« le compte a changé ») ne sont **pas** retouchés : ils décrivent un `409` qui a eu lieu, et pour ce `409` « le compte a changé » est vrai ; ils ne promettent rien. `docs/api-external.md` disait déjà « que l'IBAN du compte a changé ».
- **Écarté** : réécrire les deux clés dans les quatre locales (huit textes, replis Svelte et Rust, `TABLE` de `textes_coherents.rs`) pour une précision qu'aucun lecteur de l'avis n'exploite.
- **Réversible** : oui.

### C-27-3-53 — E1 et E2 rejouables : suffixe de passage et année propre à E1

- **Contexte** : revue de code P1, E-6 (LOW) — noms de profils fixes sous `UNIQUE (company_id, bank_name)` et fichiers fixes : un second passage sur la même base échouait à la création du profil, et E1, qui importe, aurait rencontré son propre fichier en doublon. Même défaut chez les voisines 26-1, que la doctrine « base neuve » couvre.
- **Retenu** : `RUN = Date.now().toString(36)` suffixe les noms de profils et de fichiers d'E1 et d'E2 ; E1 déplace les dates de la fixture en `E1_YEAR` = 2100 + (`Date.now()` mod 7000), jamais 2026-2028 (années des autres tests d'import). E2 n'importe rien : son année 2028 reste fixe. Les voisines 26-1 ne sont pas touchées (hors périmètre de la story).
- **Écarté** : déclarer seulement (la correction coûte six lignes) ; un nettoyage en fin de test (un test interrompu le laisserait de toute façon).
- **Réversible** : oui (test seul).

### C-27-3-54 — Un test de page pour `common-loading` de la page d'import

- **Contexte** : revue de code P1, E-7 (LOW) — le site H11 (`+page.svelte`, « Chargement… ») n'était rendu par aucun test ; seule la borne `sitesTotal` le gardait.
- **Retenu** : `routes/(app)/bank-import/bank-import-page.test.ts` (patron des tests de page, nom sans `+`), mock i18n en mode clé : pendant le chargement, `common-loading` et aucun « Chargement » ; ensuite l'écran d'import. Mutation M32.
- **Écarté** : l'ajouter à V13 (V13 rend des composants, pas la page, dont l'`onMount` appelle `apiClient`).
- **Réversible** : oui (test seul).

### C-27-3-55 — `api-external.md` : un titre pour les notes du tableau

- **Contexte** : revue de code P1, A6 (LOW) — la nouvelle sous-section, placée où la fiche le prescrit, s'insère juste avant les notes ², ³, ¹ du tableau du § 7, qui se lisaient alors sous « l'empreinte d'aperçu » (elles pendaient déjà sous « Annuler un rapprochement bancaire »).
- **Retenu** : un titre `### Notes du tableau des ressources`, et une phrase qui renvoie au tableau en tête du § 7. L'ordre des notes (², ³, ¹) n'est pas touché.
- **Écarté** : déplacer les notes sous le tableau (diff large dans un fichier que la 27-1 touche aussi au § 10) ; déplacer la sous-section neuve (l'emplacement est prescrit).
- **Réversible** : oui.

### C-27-3-56 — B-2 (`onSuccess` qui lève) : signalé, non traité

- **Contexte** : revue de code P1, B-2 (LOW) — si `onSuccess` levait, l'aperçu resterait confirmable après un import réussi.
- **Retenu** : ne rien changer. L'ordre `onSuccess` puis `reset()` est celui de la base, et le seul appelant (`+page.svelte`, `void refresh()`) ne peut pas lever. Héritage, hors du défaut de la story.
- **Écarté** : appeler `reset()` avant `onSuccess` (change l'ordre que l'appelant observe, sans cas réel).
- **Réversible** : oui.

### C-27-3-57 — Profil supprimé ailleurs : le choix revient à l'auto-détection, sans recalcul automatique

- **Contexte** : revue de code P2 ciblée, M1 (MEDIUM) — après la relecture des profils au `409` (C-27-3-51), un profil choisi puis supprimé ailleurs n'est plus une option : le sélecteur affichait « Auto-détection » pendant que `bankProfileId` gardait l'id supprimé, si bien qu'un redépôt repartait avec lui (404) ; liste vide, le sélecteur disparaissait sans issue.
- **Retenu** : dans `loadProfiles()`, après une relecture réussie, un `bankProfileId` absent de la liste revient à `null`. Le recalcul déjà en vol garde son profil (pris dans l'instantané), échoue, et son erreur s'affiche avec l'avis : l'utilisateur choisit un autre profil, ou redépose le fichier pour l'auto-détection — seule issue quand la liste est vide. Le manuel le dit. Garde de séquence ajoutée à `loadProfiles()` (L4) : seule la dernière relecture pose la liste. Tenu par V6-quater (trois cas) ; mutations M36, M37.
- **Écarté** : relancer d'office le recalcul sans profil quand le profil a disparu — l'auto-détection choisirait un profil que personne n'a demandé, et le manuel ne pourrait plus dire « si le recalcul échoue … le profil a été supprimé » ; un recalcul en double en course avec celui du `409`.
- **Réversible** : oui.

### C-27-3-58 — Revue de code P3 ciblée, LOW-1 et LOW-2 : deux angles morts assumés de `loadProfiles()`, sans code

- **Contexte** : la passe P3 ciblée (Haiku, une lentille) relève deux défauts résiduels LOW dans `loadProfiles()`
  (`BankImportUpload.svelte`), hors régression de la remédiation P2.
  - **LOW-1** — « la plus récente gagne » : une relecture réussie est écartée dès qu'une relecture plus récente
    existe, même si celle-ci échoue ensuite ; la liste en place reste alors celle d'avant (vide au montage, ce qui
    masque le sélecteur jusqu'au rechargement). Fenêtre : la relecture du montage encore en vol quand un `409`
    survient **et** que la relecture du `409` échoue.
  - **LOW-2** — `listBankProfiles(1, 50)` ne lit que la première page, quand le serveur plafonne à 200
    (`bank_profiles.rs`, `per_page.clamp(1, 200)`). Au-delà de 50 profils, un profil hors de la page 1 est remis
    à `null` à chaque relecture : le sélecteur affiche « Auto-détection » pendant que l'instantané porte l'id.
    L'envoi reste fidèle à l'aperçu affiché (l'instantané) : défaut d'**affichage**, aucun mauvais rattachement.
    La même borne de 50 vaut déjà pour la liste du montage, antérieure à la story.
- **Retenu** : **angles morts écrits, aucun code**. Une correction toucherait du code de production, ce qui
  rouvrirait la boucle (une passe ciblée P4 de plus) pour deux défauts LOW, l'un à fenêtre très étroite, l'autre
  à partir de 51 profils — l'unique utilisateur réel en a un.
- **Écarté** : pour LOW-1, un indicateur `lastAppliedSeq` (n'écarter qu'au profit d'une relecture **réussie**) ;
  pour LOW-2, `per_page` porté à 200, ou ne remettre à `null` que si la liste est complète
  (`list.total <= list.items.length`). Les trois restent de bons correctifs pour une story ultérieure.
- **Réversible** : oui (écran seul).

### C-27-3-59 — Revue de code P3 ciblée, MEDIUM-3 : le manuel distingue les deux cas du retour à l'auto-détection

- **Contexte** : le manuel disait « pour l'auto-détection, redéposez le fichier » après tout recalcul en échec ; la
  lentille relève que l'option *Auto-détection* du sélecteur relance l'aperçu sans redépôt (`handleProfileChange`).
  Sa correction proposée (« choisissez-la dans le sélecteur ») était **elle-même inexacte** dans le cas du profil
  supprimé : le sélecteur y est **déjà** revenu sur *Auto-détection* (C-27-3-57), et choisir l'option déjà
  sélectionnée n'émet pas `change` (`BankProfileSelector.svelte`, `onchange={handleChange}`) — rien ne se relance.
- **Retenu** : deux cas écrits. Recalcul en échec avec un profil encore présent : choisir un autre profil **ou**
  *Auto-détection* dans le sélecteur, sans redépôt (cohérent avec l'encadré, « inutile de redéposer le fichier
  pour essayer un autre profil »). Profil supprimé : le sélecteur est déjà sur *Auto-détection* ; choisir un autre
  profil, ou redéposer pour l'auto-détection ; redéposer encore quand plus aucun profil n'existe.
- **Écarté** : la formulation proposée par la lentille, fausse dans le cas même qu'elle visait ; relancer d'office
  le recalcul sans profil (déjà écarté par C-27-3-57).
- **Réversible** : oui (manuel seul).

---

## Story 27-4 — Un lot de réconciliation qui contient une transaction disparue (#634)

Fiche : `27-4-lot-reconciliation-transaction-disparue.md`. Choix faits à la création de la fiche (2026-10-12),
en autonomie, sur `f9725c29` (merge de la 27-1).

### C-27-4-1 — Une classification unique, en quatre lignes, partagée par les deux lots et la reprise sous verrou

- **Contexte** : le pré-vol (`find_pending_by_ids`) ne sait dire que « présent et éligible » ou « absent » ; le
  traitement sous verrou de l'acceptation a ses propres codes, le rejet n'en a pas d'exacts.
- **Retenu** : une fonction pure classe la ligne lue (ou son absence) : absente → `BANK_TRANSACTION_NOT_FOUND` ;
  rapprochée → `RECONCILIATION_ALREADY_RECONCILED` ; rejetée → `RECONCILIATION_ALREADY_REJECTED` ; sinon éligible.
  `match` exhaustif sur le statut *(clause remplacée par C-27-4-19 : appel d'`is_reconciled`, échec fermé)*. Elle sert aux deux pré-vols et à la reprise du rejet (AC4).
- **Écartées** : garder la lecture filtrée et rendre `BANK_TRANSACTION_NOT_FOUND` pour toute absence (un utilisateur
  lirait « introuvable » pour une transaction qu'il vient de rapprocher dans un autre onglet) ; une fonction par
  route (deux copies divergent).
- **Réversible** : oui.

### C-27-4-2 — Autre compte, autre société, supprimée : un seul refus par proposition, sans détail

- **Contexte** : l'issue demande si un identifiant d'un autre compte relève d'un refus global.
- **Retenu** : `failed[]`, `BANK_TRANSACTION_NOT_FOUND`, `details: null`, par la même requête scopée société et
  compte. Le § *Pattern batch* range le « FK manquant » par proposition ; le compte bancaire du corps, lui, reste
  contrôlé globalement (`404`). Aucune différence observable entre supprimée et étrangère : pas d'énumération.
- **Écartées** : refus global `400` pour un autre compte (garde la faille de l'issue pour un client qui mélange deux
  comptes, et un identifiant d'autre société devrait alors rendre la même chose qu'un supprimé — donc aussi un refus
  global, ce qui ramène au défaut) ; `details.bankAccountId` (révèlerait le compte d'une transaction étrangère).
- **Réversible** : oui (code publié : un changement ultérieur serait un changement de contrat).

### C-27-4-3 — Transaction rejetée : refusée par proposition, code neuf `RECONCILIATION_ALREADY_REJECTED`

- **Contexte** : le pré-vol refusait (en `400` global) une transaction rejetée, à l'acceptation comme au rejet ; le
  manuel dit « l'acceptation d'une proposition la refuse » (`user-manual.tex:1920`).
- **Retenu** : refus par proposition, code dédié, aux deux routes. Le comportement voulu (refuser) est conservé,
  seule sa portée change. Re-rejeter n'est pas idempotent : la ligne dit qu'elle l'était déjà.
- **Écartées** : accepter une transaction rejetée (contredit le manuel et ouvre un chemin d'acceptation que
  l'interface ne montre pas) ; réutiliser `RECONCILIATION_ALREADY_RECONCILED` (faux) ou
  `RECONCILIATION_TRANSACTION_NOT_PENDING` (faux : elle est `pending`).
- **Réversible** : oui.

### C-27-4-4 — Un lot sans proposition éligible ne prend ni transaction ni verrou

- **Retenu** : `200` immédiat, `failed[]` seul. Le verrou du compte peut être tenu cinq secondes par un autre lot :
  un lot dont rien n'est à faire n'a pas à attendre ni à rendre `409`. Testable (verrou tenu par le test) : la
  mutation qui retire le court-circuit rend `409` en 5 s.
- **Écartée** : un seul chemin, verrou compris (plus simple, mais un `409` pour un lot sans travail).
- **Réversible** : oui.

### C-27-4-5 — Ordre de `failed[]` : celui du lot reçu pour `accept`, croissant pour `reject`

- **Retenu** : `accept` rend ses refus dans l'ordre des propositions reçues (le lot y est traité dans cet ordre) ;
  `reject` les rend triés par identifiant croissant, prolongeant le contrat publié par la 27-1 (« `rejected[]` et
  `failed[]` sortent triés »).
- **Écartée** : refus du pré-vol en tête, puis ceux du verrou (ordre dépendant d'un détail d'implémentation).
- **Réversible** : oui.

### C-27-4-6 — `batch_size` de la trace d'acceptation : la taille du lot reçu

- **Retenu** : `body.proposals.len()`, passé explicitement à `accept_batch` — le sens du champ d'audit ne change pas
  parce qu'une proposition a été écartée au pré-vol.
- **Écartée** : taille du sous-lot éligible (changerait en silence la lecture des traces existantes).
- **Réversible** : oui.

### C-27-4-7 — Rejet : garde de l'`UPDATE` élargie et reprise exacte par lecture verrouillante — (complété : `AND matched_entry_id IS NULL`, voir C-27-4-30)

- **Contexte** : `reject_batch` ne recharge pas la transaction sous verrou ; une suppression ou un rejet concurrent
  entre le pré-vol et l'`UPDATE` sortait en `RECONCILIATION_ALREADY_RECONCILED` (« déjà réconciliée ») ou, pour un
  rejet concurrent, ré-horodatait la ligne et la tracait deux fois.
- **Retenu** : `UPDATE … AND bank_account_id = ? AND auto_match_rejected_at IS NULL` ; à 0 ligne, relecture
  `FOR UPDATE` (lecture courante, pas la vue cohérente) et reclassement par C-27-4-1. Même ligne, même ordre de
  verrous que l'`UPDATE` qui précède : aucun cycle neuf avec la suppression d'import.
- **Écartées** : relecture ordinaire (lirait l'instantané, d'avant la suppression) ; ne rien changer et compter sur
  le libellé (le code publié resterait faux pour un client d'API).
- **Réversible** : oui.

### C-27-4-8 — Acceptation : les trois sites `race_during_update` restent au serveur, le libellé devient honnête

- **Contexte** : `accept_one_invoice`, `accept_one_split` et `accept_one_rule` rendent
  `RECONCILIATION_ALREADY_RECONCILED` + `details.reason = "race_during_update"` quand l'`UPDATE` de la transaction ne
  touche aucune ligne — y compris quand elle vient d'être supprimée.
- **Retenu** : serveur inchangé ; l'écran lit `details.reason` et affiche « La transaction a été modifiée pendant le
  traitement — rapprochée, rejetée ou supprimée par une autre opération. » La fenêtre est de quelques millisecondes,
  la reprise toucherait le flux d'acceptation de facture (lettrage, audit, rejeu).
- **Écartée** : reprise exacte dans les trois flux (gain marginal, rayon d'impact large).
- **Réversible** : oui. Site non résolu, écrit à la fiche.

### C-27-4-9 — Plus de 500 identifiants : `400` global à la validation du corps

- **Contexte** : au-delà de `MAX_IN_CLAUSE_IDS`, le dépôt rend une erreur `Protocol`, qui sort en `500`.
- **Retenu** : `400 VALIDATION_ERROR` avant toute lecture, borne lue d'une seule constante `pub` de `kesh-db`. C'est
  un défaut du corps de la requête, exception explicite du § *Pattern batch*.
- **Écartée** : laisser le `500` (hors du sujet de l'issue, mais sur la ligne même qu'on réécrit).
- **Réversible** : oui.

### C-27-4-10 — `find_pending_by_ids` remplacée par une lecture sans filtre de statut

- **Retenu** : `find_by_ids_for_account` (nom proposé), même scope et même plafond, sans `status` ni
  `auto_match_rejected_at` ; l'ancienne fonction, sans autre appelant, est retirée (pas de code mort). Contrat API
  et CHANGELOG écrits comme un changement (`400` → `200` + `failed[]`).
- **Écartée** : garder la lecture filtrée et une seconde requête pour les absents (deux lectures, deux instants).
- **Réversible** : oui.

### C-27-4-11 — `RECONCILIATION_TRANSACTION_NOT_PENDING` des variantes ventilée et règle : non harmonisé

- **Retenu** : inchangé. Déjà par proposition, libellé juste ; le remplacer par les trois codes de C-27-4-1
  changerait un code publié sans bénéfice pour l'issue. Site non résolu, écrit à la fiche.
- **Réversible** : oui.

### C-27-4-12 — L'écran ne change pas de structure

- **Contexte** : `ReconciliationProposals.svelte` affiche déjà `failed[]` ligne par ligne, avec la transaction
  relevée avant le lot, et recharge la liste après.
- **Retenu** : libellés seulement (AC7) ; un Vitest et un E2E prouvent le comportement. Pas de rechargement de la
  liste **avant** l'envoi (il ferait disparaître une ligne cochée sans dire pourquoi).
- **Réversible** : oui.

### C-27-4-13 — Pas de découpage ; manuel FR seul

- **Retenu** : cinq modules au plus (dépôt, route, frontend, catalogues, documentation) — sous le seuil du
  découpage préventif. Seul le manuel FR existe (`docs/manual/{de,en,it}/` ne portent qu'un `README.md`).
- **Réversible** : —

### C-27-4-14 — E2E par *Rejeter* et une règle d'affectation

- **Contexte** : une ligne n'est cochable que si elle a une candidate ; le rejet n'écrit rien au grand livre.
- **Retenu** : deux imports CAMT par l'API, une règle qui couvre les deux transactions, suppression de l'un après
  l'ouverture de la page, clic *Rejeter*. Le montant et les références sont uniques à chaque exécution (base E2E
  partagée).
- **Écartée** : *Accepter* contre des factures (fixtures plus lourdes, même preuve de traversée HTTP).
- **Réversible** : oui.

### Remédiation de la validation P1 (2026-10-12, Opus)

Rapports : `kesh-gate-logs/27-4-validate-p1-F.md` (1 MEDIUM, 6 LOW) et `27-4-validate-p1-R.md` (3 MEDIUM, 11 LOW).

### C-27-4-15 — M11 tuée par un montage dédié (AC6-8), non déclarée équivalente (validation P1, F1 / R1-9 (d)) — (complété : observation durable, voir C-27-4-26)

- **Contexte** : sous le montage de l'AC6-2, la suppression est validée avant la première lecture ordinaire du lot :
  une relecture sans `FOR UPDATE` la voit aussi, et la pièce centrale de l'AC4 n'est prouvée par aucun test.
- **Retenu** : le montage proposé par la lentille F — `DELETE` de `tA` **non validé** sur une connexion du test, lot
  `reject [tA, tB]` (`tB < tA`) dont la relecture de `tB` ouvre la vue cohérente, attente de l'`UPDATE` de `tA`
  bloqué (`attendre_une_requete_en_cours`), puis validation de la suppression. Sous M11, la vue montre `tA`
  éligible → repli `race_during_update` : le test rougit.
- **Écartée** : déclarer M11 équivalente (laisserait la seule raison d'être du `FOR UPDATE` sans preuve).
- **Réversible** : oui (test seul).

### C-27-4-16 — Oracle de l'AC6-5 : valeur sentinelle, `version` inchangée, aucune trace ne cite la transaction (validation P1, R1-1 / F4)

- **Contexte** : « une seule trace pour elle » était contradictoire : le rejet concurrent est posé par `UPDATE`
  direct, qui ne trace pas ; l'attendu correct est zéro, et le mutant M10 en produit une.
- **Retenu** : `auto_match_rejected_at = '2026-01-01 00:00:00.000'` posé par le test, `version` relue ; après le lot,
  sentinelle et `version` inchangées, `tA` absente de `rejected[]`, aucune trace `reconciliation.rejected` ne la
  liste, l'unique trace du lot porte `[tB]`.
- **Écartée** : un second `POST /reject` concurrent (il bloquerait lui aussi sur le verrou nommé que tient le test).
- **Réversible** : oui.

### C-27-4-17 — Un cas de rejet qui mêle refus du pré-vol et refus sous verrou, lot non trié (validation P1, R1-2) — (révisé : trois transactions, voir C-27-4-29)

- **Contexte** : aucun test ne mêlait les deux sources de refus du rejet ; une fusion « pré-vol puis verrou »,
  chacune triée, survivait.
- **Retenu** : AC6-7 — `tBas` supprimée pendant le lot (refus sous verrou), `tHaut` rejetée avant (refus du
  pré-vol), lot envoyé `[tHaut, tBas]` → `failed = [tBas, tHaut]`. L'AC6-4 précise qu'il est envoyé non trié. M9
  rattachée aux deux.
- **Réversible** : oui.

### C-27-4-18 — Contrat : les codes d'une transaction changée pendant le traitement, et une conduite unique (validation P1, R1-3 / F6)

- **Contexte** : l'AC10 promettait « sans `details` » pour `RECONCILIATION_ALREADY_RECONCILED`, qui porte
  `details.reason = "race_during_update"` aux trois sites d'acceptation et au repli du rejet ; elle taisait
  `RECONCILIATION_TRANSACTION_NOT_PENDING` des variantes ventilée et règle.
- **Retenu** : l'AC10 distingue les codes du pré-vol (`details` nul) de ceux d'une transaction changée pendant le
  traitement, écrit `details.reason = "race_during_update"` et `…_NOT_PENDING`, et donne une conduite unique (relire
  `GET /proposals`). Le CHANGELOG le dit. L'écran (AC7) lisait déjà `race_during_update` ; sa note nomme désormais
  le quatrième site d'émission (repli du rejet).
- **Écartée** : harmoniser les codes sous verrou (C-27-4-8 et C-27-4-11 tiennent).
- **Réversible** : oui (texte).

### C-27-4-19 — « Rapprochée » au sens d'`is_reconciled` de la 27-1 ; plus de `match` exhaustif (validation P1, F2 / R1-5)

- **Contexte** : l'AC1 classait sur `status` seul, troisième définition de « rapprochée » à côté
  d'`is_reconciled` (`bank_imports.rs:349`, `status <> 'pending' || matched_entry_id`) ; l'état `pending` avec lien
  y était éligible ici, rapproché là.
- **Retenu** : la classification **appelle** `is_reconciled(status.as_str(), matched_entry_id)`. Remplace la clause
  « `match` exhaustif sur `BankTransactionStatus`, sans `_` » de C-27-4-1 : un statut futur est refusé
  (`<> 'pending'`), la classification échoue fermée, ce qui protège autant qu'une erreur de compilation sans
  dupliquer la règle. Test unitaire « `pending` avec lien », mutation M16. Doc-comment d'`is_reconciled` complété
  (toujours écrite deux fois ; désormais appelée deux fois côté serveur).
- **Écartée** : garder le `match` sur le statut et consigner la divergence (deux définitions qui peuvent dériver,
  et l'acceptation d'une facture sur cet état écrirait une seconde écriture).
- **Réversible** : oui.

### C-27-4-20 — `MAX_IN_CLAUSE_IDS` rendue `pub` sous son nom (validation P1, F5 / R1-6)

- **Contexte** : la constante borne aussi `find_contacts_by_ids` (`GET /proposals`).
- **Retenu** : nom conservé — un seul plafond défensif d'`IN`, que lisent les deux lectures du dépôt et les deux
  routes de lot. Le renommage proposé (« ex. `MAX_BATCH_IDS` ») est retiré de l'AC5.
- **Réversible** : oui.

### C-27-4-21 — La relecture verrouillante de l'AC4 est une fonction de dépôt (validation P1, R1-8)

- **Contexte** : aucune fonction du dépôt ne fait de `SELECT … FOR UPDATE` par identifiant ; `reject_batch` ne
  reçoit pas `bank_account_id`.
- **Retenu** : `find_by_id_for_account_for_update(tx, company_id, bank_account_id, id)` dans
  `repositories/reconciliation.rs`, testée au dépôt (AC11) ; `reject_batch` reçoit `bank_account_id` à la place du
  `tx_map`. Le caractère courant de la lecture est prouvé par l'AC6-8.
- **Écartée** : SQL en ligne dans la route (lecture de `bank_transactions` hors du dépôt, sans test propre).
- **Réversible** : oui.

### C-27-4-22 — Table des mutations : M7 scindée, M6 rattachée, M17 et M18 déclarées équivalentes d'emblée (validation P1, R1-9 / F3) — (révisé pour M17 : tuée, voir C-27-4-27)

- **Retenu** : M7a / M7b (le court-circuit vit dans deux handlers ; l'AC3 se teste sur les deux routes) ; M6
  rattachée à l'assertion `details: null` d'AC6-1 et AC6-3 (la comparaison des deux corps ne la tue pas si la
  valeur vient du corps de la requête) ; M17 (`AND bank_account_id = ?` retiré) équivalente — aucun chemin ne
  change le compte d'une transaction (`grep -rnE "SET[^;]*bank_account_id *="` ne rend que
  `settlement_bank_account_id`) ; M18 (repli « relue éligible ») équivalente — non atteignable sous la lecture
  courante.
- **Réversible** : oui.

### C-27-4-23 — Manuel : la puce *Supprimer un import* nomme les deux motifs ; la l. 1920 n'est pas touchée (validation P1, F7 / R1-10) — (révisé pour la puce : la conduite, non les motifs, voir C-27-4-28)

- **Contexte** : une suppression qui tombe pendant le traitement d'une acceptation sort en « modifiée pendant le
  traitement », non en « introuvable » ; une transaction rejetée pendant qu'un lot d'acceptation attend le verrou
  est encore acceptée.
- **Retenu** : la puce neuve nomme les deux motifs. La l. 1920 (« l'acceptation d'une proposition la refuse ») reste :
  vraie pour une liste périmée, la course est un choix de conception déjà écrit (« je rétracte »), sans effet
  comptable, et la décrire alourdirait un manuel d'usage pour un cas de quelques secondes entre deux onglets.
- **Écartée** : réécrire la l. 1920 avec la course (texte plus juste, mais qui décrit un instant que l'utilisateur
  ne peut ni voir ni provoquer volontairement).
- **Réversible** : oui (texte).

### C-27-4-24 — E2E : valeur de règle unique par exécution, nettoyage en `finally` (validation P1, R1-12)

- **Retenu** : la contrepartie qui déclenche la règle est réécrite dans le relevé à chaque exécution, comme les
  montants ; la règle et l'import restant sont supprimés en `finally`. Une règle tirée de la fixture commune
  rendrait sélectionnables des transactions d'autres specs (pollution d'état entre specs, `CLAUDE.md` § E2E).
- **Réversible** : oui.

### C-27-4-25 — Les angles morts assumés sont tous de catégorie C : aucune issue ouverte (validation P1, R1-13)

- **Contexte** : le tableau « sites NON RÉSOLUS » ne vivait que dans la fiche ; la § *Issue Tracking Rule* veut
  GitHub pour tout comportement défaillant hors du travail courant.
- **Retenu** : triage site par site, écrit à la fiche — `race_during_update` d'acceptation et vue cohérente
  d'`accept_batch` (code générique désormais documenté, rien d'écrit) ; `…_NOT_PENDING` (code publié, juste) ;
  recharge d'`accept_one_invoice` sans filtre de rejet (voulue, sans effet comptable) ; `pending` avec lien sous
  verrou et repli éligible du rejet (non atteignables) ; `/manual`, `/split` (hors § *Pattern batch*). Aucun n'est un
  comportement défaillant : aucune issue, et la fiche le dit. Si l'orchestrateur en juge autrement, la liste est là.
- **Réversible** : oui.

### Remédiation de la validation P2 (2026-10-12, Opus)

Rapports : `kesh-gate-logs/27-4-validate-p2-F.md` (4 MEDIUM, 4 LOW) et `27-4-validate-p2-R.md` (3 MEDIUM, 7 LOW).
Consigne de l'orchestrateur : simplifier, ne pas ajouter de mécanisme fragile.

### C-27-4-26 — AC6-8 : le blocage de `tA` observé durablement, l'angle mort écrit (validation P2, R2-1 / F2-5)

- **Contexte** : les motifs de l'observateur désignent aussi l'`UPDATE` de `tB`, de même texte ; s'il est surpris en
  vol, la suppression est validée avant l'ouverture de la vue cohérente et M11 survit. « Pas de confusion » ne visait
  que la relecture `FOR UPDATE`.
- **Retenu** : l'aide existante appelée deux fois, séparées de 500 ms — aucun mécanisme neuf. Angle mort écrit au
  doc-comment du test, comme A2-3 de la 27-1 : M11 ne survit que si un `UPDATE` par clé primaire dure plus de 500 ms ;
  le code juste ne peut rendre aucun faux rouge ; une survivante se rejoue avant d'être déclarée.
- **Écartées** : `information_schema.INNODB_TRX` (`trx_state = 'LOCK WAIT'`, le seul trait propre à l'attente) — exige
  le privilège `PROCESS`, que l'utilisateur de dev n'a pas (`kesh`, droits par base, `01-dev-grants.sql` ; la CI
  tourne en `root` — précision F3-2 de la validation P3) ; un filtre `TIME_MS` (même minutage, sémantique de la
  colonne pendant une attente de verrou non vérifiée) ; retirer le montage et déclarer M11 non tuée (la seule preuve
  du `FOR UPDATE` disparaîtrait, alors qu'un observateur simple suffit).
- **Réversible** : oui (test seul).

### C-27-4-27 — M17 tuée : la garde `bank_account_id` défend la réutilisation des identifiants par la restauration (validation P2, R2-2 / F2-3)

- **Contexte** : M17 avait été déclarée équivalente parce qu'« aucun chemin ne change le compte d'une transaction ».
  C'est faux : `restore_tables_in_tx` vide `bank_transactions` puis la recharge avec ses identifiants, sans prendre le
  verrou nommé de réconciliation ; un identifiant vu sur le compte X par le pré-vol peut désigner, sous verrou, une
  ligne du compte Y.
- **Retenu** : la raison réécrite à l'AC4 ; M17 tuée par l'AC6-5 (b), montage de l'AC6-5 : pendant l'attente du
  verrou, `UPDATE bank_transactions SET bank_account_id = <second compte de la même société> WHERE id = tA` ; attendu
  `BANK_TRANSACTION_NOT_FOUND`, `tA` non rejetée, aucune trace ne la cite. Ligne « restauration concurrente » ajoutée
  aux sites non résolus, catégorie C (angle mort préexistant de la restauration).
- **Écartée** : garder M17 non tuée, angle mort assumé (le montage ne coûte qu'une variante d'un test existant).
- **Réversible** : oui.

### C-27-4-28 — Manuel : la puce *Supprimer un import* décrit la conduite, non les motifs (validation P2, R2-3 / F2-1 / F2-8)

- **Contexte** : la puce réécrite en P1 promettait « Transaction introuvable » ou « modifiée pendant le traitement ».
  Une proposition **règle** ou **ventilée** rend `RECONCILIATION_TRANSACTION_NOT_PENDING` (« n’est plus en
  attente »), et pour une facture la suppression pendant l'attente du verrou sort le plus souvent en « introuvable » :
  le « Contexte » de C-27-4-23 se trompait de motif.
- **Retenu** : la puce dit que la transaction est refusée avec son motif sous *Échecs partiels*, que les autres
  propositions sont traitées et que la liste rechargée ne la propose plus — rien à refaire ; aucun libellé cité.
  Témoin du PDF : « les autres propositions du lot sont traitées ». La décision sur la l. 1920 (C-27-4-23) tient.
- **Écartée** : nommer les trois motifs (liste close qui se périme au premier code ajouté).
- **Réversible** : oui (texte).

### C-27-4-29 — AC6-7 à trois transactions : un refus sous verrou entre deux refus du pré-vol (validation P2, F2-2)

- **Contexte** : avec deux transactions, la concaténation « verrou puis pré-vol », chacune triée, rendait l'ordre
  attendu : M9 n'était tuée que dans un ordre.
- **Retenu** : l'AC6-7 **remplacé** (pas de cas ajouté) : `tBas` et `tHaut` rejetées avant le lot, `tMil` supprimée
  pendant ; lot `[tHaut, tMil, tBas]`, attendu `[tBas, tMil, tHaut]`. Les deux ordres rougissent.
- **Réversible** : oui (test seul).

### C-27-4-30 — L'`UPDATE` du rejet porte `matched_entry_id IS NULL` (validation P2, F2-4)

- **Contexte** : le pré-vol dit « rapprochée » au sens d'`is_reconciled`, l'`UPDATE` du rejet sur `status` seul ; le
  site manquait à l'inventaire des sites non résolus.
- **Retenu** : une clause de plus ; la relecture reclasse alors l'état `pending` avec lien par la fonction de l'AC1.
  M18 reste équivalente ; M19 (clause retirée) déclarée équivalente — état qu'aucun chemin vivant ne produit.
- **Écartée** : inscrire le site au tableau sans le résoudre (une clause coûte moins qu'un angle mort).
- **Réversible** : oui.

### C-27-4-31 — Pas de découpage après la P2 (décision de l'orchestrateur, constat D5)

- **Contexte** : P1 4 MEDIUM → P2 5 MEDIUM distincts, dont 4 nés de la remédiation P1 : le signal de la règle de
  *splitting préventif* est levé, et c'est un **recyclage** au sens de l'amendement D5 (défauts nés du correctif
  précédent), non des défauts d'origine neufs.
- **Retenu** : pas de découpage — décision de l'orchestrateur, dans l'autonomie que Guy lui a déléguée le
  2026-10-12, sur le précédent de la 27-3 (« ne pas découper »). Remède : simplifier — réutiliser l'aide existante
  (AC6-8), une variante d'un montage existant (AC6-5 (b)), remplacer un cas plutôt qu'en ajouter (AC6-7), une clause
  SQL (AC4), la conduite plutôt qu'une liste au manuel (AC9). Le périmètre reste sous cinq modules.
- **Écartée** : découper (un test, un texte, une clause par finding : rien qu'une sous-story tiendrait mieux).
- **Réversible** : oui — si la P3 recycle encore, le signal sera redéclaré.

### C-27-4-32 — La relecture verrouillante partage le texte SQL de `find_pending_by_id_for_account` (validation P2, R2-9)

- **Contexte** : `find_by_id_for_account_for_update` est la requête de `find_pending_by_id_for_account` plus
  `FOR UPDATE` ; la recopier ferait deux lectures qui peuvent diverger.
- **Retenu** : un texte SQL partagé par une **fonction privée** qui le construit (`format!` sur `BANK_TX_COLUMNS`),
  appelée par les deux lectures, la verrouillante y ajoutant ` FOR UPDATE`. *(Précisé en validation P3, F3-3 /
  R3-2 : la « constante » de la première rédaction n'est pas réalisable — Rust ne concatène pas deux `const &str`
  sans `const_format`, absente du workspace.)*
- **Écartée** : un paramètre booléen « verrouillante » sur la fonction existante (change la signature d'une
  fonction générique sur l'exécuteur, appelée par `accept_one_invoice` et par un test de dépôt, pour un seul besoin).
- **Réversible** : oui.

### C-27-4-33 — Développement : une seule fonction de fusion de `failed[]` pour les deux routes

- **Contexte** : l'AC2 veut l'ordre des propositions reçues (`accept`), l'AC4 l'ordre croissant (`reject`) ; deux
  fusions écrites à part pouvaient diverger.
- **Retenu** : `merge_failed_in_order(ids, preflight, locked)` — les refus du pré-vol dans une `HashMap` par
  identifiant, ceux du verrou y sont ajoutés, puis la liste suit `ids` (lot reçu pour `accept`, lot trié pour
  `reject`). Les doublons étant refusés en `400` avant, chaque identifiant n'a qu'une source. Testée en unitaire et
  par l'AC2 (a) et l'AC6-7.
- **Écartée** : concaténer puis trier (le tri d'`accept` n'est pas un tri par clé).
- **Réversible** : oui.

### C-27-4-34 — Développement : la classification rend `Option<FailedProposal>`, le plafond est une fonction

- **Contexte** : l'AC1 veut une fonction pure, codes écrits au site ; l'AC5, une borne unique lue par les deux routes.
- **Retenu** : `classify_batch_transaction(id, Option<&BankTransaction>) -> Option<FailedProposal>` (`None` =
  éligible) ; `check_batch_size(len)`, appelée juste après le contrôle « lot vide » des deux routes, message
  « un lot ne peut pas dépasser 500 transactions » (la valeur lue de `MAX_IN_CLAUSE_IDS`).
- **Écartée** : un `enum` intermédiaire (Éligible / Refus(code)) — il aurait déplacé la construction des codes hors du
  site que compte l'AC7.
- **Réversible** : oui.

### C-27-4-35 — Développement : `find_by_id_for_account_for_update` générique sur l'exécuteur

- **Contexte** : la fiche écrit la signature avec `tx` ; ses voisines du dépôt prennent un `Executor` générique.
- **Retenu** : générique, comme `find_pending_by_id_for_account` dont elle partage le texte SQL ; le doc-comment
  avertit qu'appelée en autocommit elle ne verrouille rien au-delà de l'instruction. Seul appelant : `reject_batch`,
  avec `&mut **tx_outer`.
- **Écartée** : `&mut Transaction` imposé (s'écarte du patron du fichier pour une garantie que le seul appelant tient).
- **Réversible** : oui.

### C-27-4-36 — Développement : l'AC6-3 en un seul test, l'AC3 en un seul test

- **Contexte** : l'AC6-3 demande la réécriture du test « autre compte » et un jumeau « autre société » comparé en
  JSON ; l'AC3 admet un test qui appelle `/accept` puis `/reject` sous le même verrou tenu.
- **Retenu** : `post_accept_refuses_cross_account_and_cross_company_transactions_identically` (renommé, les deux cas,
  une aide qui vérifie que rien n'est écrit et une qui neutralise l'identifiant avant comparaison) ;
  `batches_without_eligible_transaction_take_no_lock` (les deux routes, statut seul, sans borne de temps — F3-4).
- **Réversible** : oui (tests seuls).

### C-27-4-37 — Développement : montage de l'AC2 (a)

- **Contexte** : il faut, dans un lot non trié, un absent d'identifiant bas, une éligible refusée sous verrou et un
  absent d'identifiant haut.
- **Retenu** : absent bas = une transaction d'un **autre compte** de la même société, créée la première ; éligible
  refusée sous verrou = facture `draft` (`RECONCILIATION_INVOICE_NOT_ELIGIBLE`) ; absent haut = identifiant
  inexistant. Assertion de montage `low_absent < mid`.
- **Réversible** : oui (test seul).

### C-27-4-38 — Développement : l'E2E constate le rejet par la liste et le statut, non par un champ de rejet

- **Contexte** : `GET /api/v1/reconciliation/transactions/{id}` ne porte pas `auto_match_rejected_at`
  (`TransactionResponse`, `bank_imports.rs:245`).
- **Retenu** : l'E2E asserte le compteur `1`, l'*Échec partiel* de A (`data-error-code`, libellé récrit), l'absence de
  `reconciliation-error`, l'absence des deux lignes dans la liste rechargée, et le statut `pending` de B côté serveur.
  Le rejet lui-même est prouvé côté Rust (AC6-1). La règle (`POST /api/v1/reconciliation/rules`,
  `counterparty_contains` sur une contrepartie unique) et l'import B sont supprimés en `finally`.
- **Écartée** : ajouter le champ à la réponse (changement de contrat hors périmètre).
- **Réversible** : oui (test seul).

### C-27-4-39 — Développement : un test unitaire hors du décompte `error_code: "…"` de l'AC7

- **Contexte** : le test de `merge_failed_in_order` construit des `FailedProposal` ; écrit `error_code: "X"…`, il
  aurait ajouté un 28ᵉ littéral au décompte de la méthode de l'AC7.
- **Retenu** : `error_code: String::from("TEST")`, commenté ; le décompte rend 27 (`grep -ohE 'error_code: "[A-Z_]+"' …
  | sort -u | wc -l`).
- **Réversible** : oui.

### C-27-4-40 — Revue de code P1, E1 : un AC6-5 (c), « rapprochée sans lien » pendant l'attente du verrou

- **Contexte** : la garde `status = 'pending'` de l'`UPDATE` de `reject_batch` pouvait être retirée sans qu'aucun
  test ne rougisse : les montages de course du rejet couvraient la suppression, le rejet concurrent et le changement
  de compte, pas l'acceptation terminée pendant l'attente. Or `matched_entry_id` est `ON DELETE SET NULL` : l'état
  `reconciled` **sans** lien est atteignable, et seule la garde de statut l'écarte.
- **Retenu** : test `post_reject_does_not_reject_a_transaction_reconciled_meanwhile`, sur le patron de l'AC6-5 (a) :
  `status = 'reconciled'` posé **sans** lien pendant l'attente (le montage l'asserte), refus
  `RECONCILIATION_ALREADY_RECONCILED`, `details` nul, ligne inchangée, trace du seul rejet de l'autre transaction.
  Mutation **M20** (garde retirée) jouée : rouge sur ce seul test.
- **Écartée** : poser aussi un lien (`matched_entry_id`) — la garde `matched_entry_id IS NULL` aurait alors refusé à
  la place de la garde de statut, et M20 survivait.
- **Réversible** : oui (test seul).

### C-27-4-41 — Revue de code P1, E4 / B LOW-1 : les deux imports nettoyés en `finally`, compteur exact

- **Retenu** : `importA` suivi comme `importB` et supprimé en `finally` (un `404` y est sans effet, A étant supprimé
  dans le corps du test) ; le compteur de succès asserté par `toHaveText(/^\s*1\s+\D/)` (« 1 opération(s)… »), qui
  refuse « 10 » ou « 11 », au lieu de `toContainText('1')`.
- **Réversible** : oui (test seul).

### C-27-4-42 — Revue de code P1, E2 : `company_id` retiré des lectures scopées par compte, mutations équivalentes

- **Contexte** : le filtre `company_id = ?` de `find_by_ids_for_account` et du texte SQL partagé
  `bank_tx_by_id_for_account_sql` (lecture par identifiant, avec ou sans `FOR UPDATE`) n'est tué par aucun test.
- **Retenu** : déclarées **équivalentes**, après exécution : **M21** (`company_id` neutralisé dans
  `find_by_ids_for_account`) et **M22** (idem dans `bank_tx_by_id_for_account_sql`) survivent (100 tests verts sur
  `reconciliation_e2e`, `reconciliation_repository` et la classification). Un compte bancaire n'appartient qu'à une
  société et le filtre `bank_account_id` écarte déjà les lignes de l'autre société : aucun état produit par un
  écrivain du dépôt ne les distingue. Le filtre reste, en défense en profondeur (règle de scoping multi-tenant).
- **Écartée** : un test qui fabrique une transaction dont `company_id` contredit celui de son compte — un état
  qu'aucun chemin ne produit, qui prouverait la clause sans rien dire du produit.
- **Réversible** : sans objet.

### C-27-4-43 — Revue de code P1, E5 : le libellé « rapprochée, rejetée ou supprimée » reste générique

- **Contexte** : pour l'acceptation, la cause « rejetée » n'est pas atteignable sous verrou (le rejet partage le
  verrou du compte, et l'`UPDATE` d'acceptation ne filtre pas `auto_match_rejected_at`).
- **Retenu** : texte inchangé (catalogues, `failed-proposal-label.ts`, contrat) : il sert aussi le repli du rejet, où
  la cause est juste, et un libellé générique ne trompe pas (C-27-4-8). Surcharge de précision sans effet.
- **Réversible** : oui.

### C-27-4-44 — Revue de code P1, A-5 : « Sie » majuscule après deux-points, gardé en allemand

- **Retenu** : inchangé. En allemand, une phrase complète qui suit un deux-points prend la majuscule ; le catalogue
  `de-CH` emploie déjà cette forme pour une chose désignée par « sie » (`error-lettering-line-owned-by-document`,
  `journal-entries-reverse-blocked-supplier-invoice`, `reconciliation-failed-rule-no-longer-matches`). Le contexte
  (« Transaktion … : Sie wurde … gelöscht ») ne laisse pas lire la forme de politesse.
- **Écartée** : reformuler sans pronom les deux seules clés de la story — elles divergeraient de leurs voisines.
- **Réversible** : oui.

### C-27-4-45 — Revue de code P1, A-1 : l'ancien message cité au contrat d'API, résidu légitime

- **Retenu** : `docs/api-external.md` cite toujours « bankTransactions […] n'appartiennent pas au bankAccountId » —
  dans l'avertissement de **changement de contrat**, pour qu'un client reconnaisse le `400` qu'il traitait. Le
  critère « vide » de l'AC2 vise le code et les textes servis, pas cette citation ; trié au Dev Agent Record.
- **Réversible** : oui.

### C-27-4-46 — Rebase sur `origin/main` (27-2, 27-3) avant les LOW de la revue

- **Contexte** : la branche partait de `f9725c29` (27-1) ; `main` porte depuis la 27-2 (`e04122dc`) et la 27-3
  (`f2b03df9`).
- **Retenu** : rebase **d'abord**, LOW ensuite, pour que les numéros de ligne de l'inventaire (E3) soient ceux du
  `HEAD` final. Conflits réunis : registre et `sprint-status.yaml` par union ; `CHANGELOG.md`, une seule section
  `[0.13.2]`, un seul `### Modifié`, un seul `### Corrigé` ; `sitesTotal` **2075** (2073 de `main` + 2, vérifié par
  la garde) ; PDF du manuel utilisateur **régénéré** par le Makefile de `main` (jamais fusionné en binaire).
  `reconciliation.rs`, `reconciliation_e2e.rs`, les catalogues et `api-external.md` ont fusionné sans conflit.
- **Écartée** : merger `main` dans la branche (historique moins lisible, la PR étant squashée de toute façon).
- **Réversible** : oui (avant push).
