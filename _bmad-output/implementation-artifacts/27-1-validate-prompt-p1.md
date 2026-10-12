# Prompt — validation P1 de la spec, Story 27-1 (supprimer un import bancaire non rapproché)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-27-1`, branche `story/27-1-supprimer-import` (base `aadace40`, = v0.13.1 +
rétrospective de l'Epic 26 + kickoff de l'Epic 27).
Fiche : `_bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md` (commit `61383dc4`).
Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`. Issue #632 (`gh api repos/guycorbaz/kesh/issues/632`
et `…/632/comments`). Registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md` (C-27-1-1 à 11) ;
doctrine du rejeu : `epic-15-choix-autonomes.md` C54-C59.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy ; § Issue
Tracking Rule).

⚠️ La fiche **corrige une prémisse de l'epic et de l'issue** (C-27-1-8 : l'unicité du hash a été retirée en 8-3).
Vérifie-le toi-même avant d'en tirer quoi que ce soit.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaires clos** (Dev Notes § Inventaire (a) et (b)) : refais les `grep` copiés dans la fiche, et
   **élargis-les** (un détecteur part du symptôme, pas d'une liste) — `bank_transactions` et `bank_imports`
   sous toutes graphies (alias SQL `bt.`, `bi.`, noms de colonnes `import_id`, `matched_entry_id`,
   `auto_match_rejected_at`, `status`), `QueryBuilder`, requêtes construites par `format!`, `kesh-seed`,
   `admin_backup`, `backup.rs`, `post_restore.rs`, `test_endpoints.rs`, le frontend. Un écrivain ou une table
   référente manquante est au moins **HIGH** : la définition fermée de « rapprochée » (AC1) en dépend.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Accepter ou rejeter les
   propositions* (l. 1859 et suivantes, dont le paragraphe *Rejeter* l. 1873), § *Import dans Kesh*
   (l. 1690-1719), § *Annuler un rapprochement* (l. 1940 et suivantes), § *Solde comptable, relevé et écart*
   (l. 238 et suivantes). Ce que le manuel dit **aujourd'hui** du rejet, du détail d'un import et de la tuile
   *Relevé du …* est-il vrai au code ? Ce que l'AC13 prescrit d'écrire sera-t-il vrai au code que la fiche
   prescrit ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`. Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — `sitesTotal` 2032 et le delta +9
   (`grep -o "i18nMsg(" <fichier>`), les compteurs du registre des routes (114/117/90/« six »), le nombre de
   clés (dix), d'AC (16), de tâches (T0-T8), de mutations (M1-M15), les numéros de ligne cités.

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

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-1-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt, `gh` en
écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés :
lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
