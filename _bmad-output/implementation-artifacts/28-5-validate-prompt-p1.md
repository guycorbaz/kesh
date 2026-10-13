# Prompt — validation P1 de la spec, Story 28-5 (une transaction rejetée redevient atteignable)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5` (base `5aad8b35`, = v0.13.2 + rétrospective de l'Epic 27
+ kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md` (commit `c8acbfe1`).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (§ 28-5, § *Ordre*, § *Recette visuelle*).
Issue #526 (`gh api repos/guycorbaz/kesh/issues/526` et `…/526/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-5-1 à 16). Consignes :
`docs/agents-orchestration.md`. Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé,
les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de
splitting préventif ; § Pattern batch ; § Issue Tracking Rule).

⚠️ La fiche **s'écarte de la lettre de l'issue et de l'epic** (« filtre ou onglet ») : la transaction rejetée se
retrouve dans le **détail de son import**, non par un filtre de la page *Réconciliation* (C-28-5-1). Juge l'écart
toi-même, sans le tenir pour acquis.

## Axes prioritaires (communs aux deux lentilles)

1. **Périmètre minimal** (leçon de l'Epic 28 : la 28-1 a été réduite deux fois en validation). La fiche ferme-t-elle
   #526 **et rien de plus** ? Chaque AC, tâche, test et mutation est-il nécessaire à la fermeture de #526, ou à une
   règle du `CLAUDE.md` (DRY, gardes, documentation) ? Le tableau *Exclu* est-il complet, et chaque exclusion
   tient-elle (a-t-elle son issue de suivi quand c'est un défaut) ? À l'inverse, **l'exclusion du filtre** laisse-t-elle
   #526 réellement fermée — un utilisateur qui vient de rejeter sur la page *Réconciliation* retrouve-t-il sa
   transaction sans connaître le manuel ? Le **décompte des modules** (quatre paquets, huit sites) et la dérogation
   C-28-5-11 sont-ils exacts et recevables ? Un périmètre trop large est au moins MEDIUM ; un périmètre qui ne ferme
   pas l'issue est au moins HIGH.
2. **Inventaire clos** (Dev Notes § *Inventaire des sites NON RÉSOLUS*) : refais les `grep` de la fiche et
   **élargis-les** (un détecteur part du symptôme, pas d'une liste) — `auto_match_rejected_at` sous toutes graphies
   (`autoMatchRejectedAt`, alias SQL, `rejected_at`, `was_previously_rejected`), `kesh-seed`, `test_endpoints.rs`,
   `admin_backup`, `post_restore.rs`, `exports/`, le frontend entier. Un écrivain ou un lecteur du marqueur absent de
   l'inventaire, ou un écran qui montre une transaction rejetée sans le dire, est au moins MEDIUM.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Accepter ou rejeter les propositions*
   (l. 1933 et suivantes, puce *Rejeter* l. 1947), § *Supprimer un import* (l. 1724-1765), § *Annuler un
   rapprochement* (l. 2014 et suivantes). Toute phrase qui dit qu'une transaction rejetée « disparaît », « quitte »,
   « sans aucune action », « aucun geste » : vraie aujourd'hui ? vraie après la story ? Le texte que la fiche
   prescrit pour la puce sera-t-il vrai au code que la fiche prescrit (rôles, journal d'audit, libellé, effet
   comptable) ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`. Une promesse que le code ne tiendrait pas est au moins MEDIUM. Même exercice pour
   `docs/api-external.md` (l. 534-575) et le CHANGELOG prescrit.
4. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — compteurs du registre des routes
   (115 / 109 / 91 / 118 → 116 / 110 / 92 / 119, `crates/kesh-api/tests/audit_route_registry.rs`), `sitesTotal` 2075
   (`frontend/src/lib/shared/i18n-keys.test.ts:601`), six clés × quatre locales, 12 AC, T0-T11, B1-B8, V1-V5,
   M1-M12, seize choix, six captures, et **chaque numéro de ligne cité** (`sed -n`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #526 (corps et deux commentaires) et du § 28-5 de l'epic couverte par un AC testable, ou
  exclue par écrit ; chaque AC par une tâche ; chaque AC par un test **qui rougirait** (et chaque mutation M1-M12 par
  le test annoncé — une mutation équivalente non déclarée est un défaut) ; chaque fait, chemin et numéro de ligne
  cité **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (six clés i18n dans les quatre locales, manuel,
  `api-external.md`, CHANGELOG) confrontés à ce que le code fera ; les gardes existantes qui changeront de sens
  (`audit_route_registry.rs`, `audit_label_registry.rs`, `audit_log_e2e.rs`, `i18n_e2e.rs`, `i18n-keys.test.ts`,
  `i18n-un-repli-par-cle.test.ts`, `lint-i18n-ownership`, `textes_coherents.rs`, `test_schema_guard.rs`,
  `npm run check` sur les fixtures qui construisent un `BankTransactionDto`) — la fiche les nomme-t-elle toutes ?
  Le frontend : la page de détail calcule-t-elle bien `canManage` comme dit, `isReconciled` est-il exporté et
  réutilisable, le client gère-t-il un `204` sur `post` ?
- **F — Faisabilité, concurrence et contrat (adversaire de la conception)** : la séquence de la route (Dev Notes
  § *Séquence*) est-elle **correcte sous InnoDB REPEATABLE READ** — lecture scopée hors verrou, `GET_LOCK`, lecture
  verrouillante `FOR UPDATE`, `UPDATE`, insertion d'audit, relâche du verrou **avant** le commit
  (`crates/kesh-reconciliation/src/mutex.rs`) ? Le raisonnement « aucun cycle, pas de rejeu » (C-28-5-4) tient-il
  contre **chaque** écrivain de `bank_transactions` : `reject_batch`, `accept_one_*`, `post_manual_once`,
  `post_split_once`, `cancel_in_tx`, `bank_imports::delete_unreconciled_in_tx` (27-1, sans verrou nommé), la
  création d'import, la restauration d'installation ? Le classement `SansEcritureAuJournal` est-il compatible avec
  les volets (c), (c bis) et (d) du registre ? `classify_unreject` sur `is_reconciled` : un état atteignable
  donne-t-il un code faux ? Les codes `404 NOT_FOUND` / `409 RECONCILIATION_TRANSACTION_NOT_REJECTED` : anti-énumération
  (KF-002) respectée, collision avec un code existant, cohérence avec `…/cancel` (`BANK_TRANSACTION_NOT_RECONCILED`) ?
  Le champ additif `autoMatchRejectedAt` casse-t-il un consommateur (export, sauvegarde, tests qui comparent un JSON
  entier, `#[serde(flatten)]` de `ReconciliationTransactionResponse`) ? L'extraction des deux auxiliaires de
  `cancel_reconciliation_once` (C-28-5-12) préserve-t-elle **exactement** son comportement, rejeu compris (le
  prédicat lit `DbError::Sqlx`) ? Les clés d'API (C-28-5-6) contre § 4 et § 7 de `docs/api-external.md`. Les régions
  annoncées pour la 28-6 et la 28-7 sont-elles exactes (lis le § 28-6 et le § 28-7 de l'epic) ? Les tests B5
  (verrou tenu) et l'E2E (données uniques, `finally`) sont-ils **déterministes** ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts
Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/
`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` ou tout autre
interpréteur** (même « en lecture seule »), `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un
script du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
