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
