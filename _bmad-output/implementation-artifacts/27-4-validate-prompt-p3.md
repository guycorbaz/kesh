# Prompt — validation P3 de la spec, Story 27-4 (lot de réconciliation contenant une transaction disparue)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet. P2 : Opus.
P3 : **Sonnet** pour les deux lentilles — passe **complète**, non ciblée.*

Dépôt `/home/gcorbaz/devel/kesh-27-4`, branche `story/27-4-lot-reconciliation-transaction-disparue` (base
`f9725c29`, = merge de la Story 27-1, #635).
Fiche : `_bmad-output/implementation-artifacts/27-4-lot-reconciliation-transaction-disparue.md` — création
`70e1523d`, remédiation de la P1 `7a610943`, **remédiation de la P2 `5202adbf`** (`git show 5202adbf` ;
`git diff cb048239 5202adbf`).
Rapports de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p2-F.md` et `…-p2-R.md` (ceux de la P1 :
`…-p1-F.md`, `…-p1-R.md`).
Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md` (section « Amendement du 2026-10-12 »).
Issue #634 (`gh api repos/guycorbaz/kesh/issues/634` et `…/634/comments`) ; contexte : #632 et la fiche
`27-1-supprimer-import-non-rapproche.md` (AC3 (b), AC8, choix C-27-1-17). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md`, **C-27-4-1 à C-27-4-32** (C-27-4-15 à 25 : la
remédiation de la P1 ; **C-27-4-26 à 32 : celle de la P2** ; C-27-4-15, 17, 22 et 23 portent dans leur titre la
mention de leur révision).
Règles : `CLAUDE.md` — § *Pattern batch — FailedProposal per-proposal* (au cœur de la story), § *Review Iteration
Rule* et ses sous-sections (le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le
recompte des comptes rendus), § *Règle de splitting préventif* (amendement D5 du 2026-10-06), § *Pendant une boucle
de revue* (exception `kesh-db`), § *Issue Tracking Rule*.

## Contexte de la boucle — à lire avant tout

La P2 a trouvé **5 MEDIUM distincts, dont 4 nés de la remédiation P1** (les montages ajoutés). Le signal de
découpage a été déclaré et l'orchestrateur a décidé de **ne pas découper** (C-27-4-31) : la remédiation P2 a été
écrite **par simplification** — l'aide de test existante appelée deux fois plutôt qu'une aide neuve (AC6-8), une
variante d'un montage existant (AC6-5 (b)), un cas **remplacé** plutôt qu'ajouté (AC6-7), une clause SQL (AC4), la
conduite plutôt qu'une liste de libellés au manuel (AC9). Ta passe dit si cette simplification a tenu.

⛔ **Pour chaque finding, réponds explicitement à la question : « un défaut né de cette remédiation (`5202adbf`)
est-il un recyclage ? »** — c'est-à-dire un défaut qui revient sous une autre forme, ou qui naît du correctif
précédent (amendement D5). Classe chaque finding en **(i) né de la remédiation P2 — recyclage**, **(ii) né de la
remédiation P2 — défaut neuf, sans lien avec un finding antérieur**, **(iii) né de la remédiation P1, non vu en
P2**, ou **(iv) préexistant à la création de la fiche**. Le décompte de ces quatre classes figure au bilan.

## Axe premier (commun aux deux lentilles) — les régressions de la remédiation P2

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Relis d'abord
`git diff cb048239 5202adbf`, phrase par phrase, et cherche ce que la remédiation a **cassé ou laissé à moitié** :

1. **AC6-8, observation durable (C-27-4-26)** : deux appels d'`attendre_une_requete_en_cours`
   (`crates/kesh-db/src/test_fixtures.rs:561`) séparés de 500 ms suffisent-ils à désigner l'`UPDATE` **bloqué** de
   `tA` plutôt que celui de `tB` ? Le second appel peut-il voir **autre chose** qu'un `UPDATE` encore en cours
   (une requête d'un test voisin de la même base, la relecture `… FOR UPDATE`, l'`UPDATE` de `tB` lent) ? L'angle
   mort écrit (« ne survit que si l'`UPDATE` de `tB` dure plus de 500 ms ; aucun faux rouge sous le code juste »)
   est-il **exact** — en particulier, l'affirmation « aucun faux rouge » tient-elle dans **tous** les entrelacements
   ? La raison qui écarte `INNODB_TRX` (privilège `PROCESS`) est-elle vraie au dépôt ?
2. **AC6-5 (b), M17 tuée (C-27-4-27)** : l'`UPDATE … SET bank_account_id = <second compte>` du test est-il
   accepté par le schéma (clés étrangères, contraintes, cohérence avec `bank_imports.bank_account_id`, index) ?
   Le pré-vol voit-il bien `tA` sur le compte d'origine, et la relecture `FOR UPDATE` scopée rend-elle `None` ? Sous
   M17, `tA` entre-t-elle **réellement** dans `rejected[]` (garde `status`, `matched_entry_id`, rejet) ? La raison
   réécrite — la **restauration** réinsère les mêmes identifiants (`crates/kesh-db/src/backup.rs`,
   `restore_tables_in_tx`) et ne prend pas le verrou nommé — est-elle exacte au code ? La ligne « restauration
   concurrente » ajoutée aux sites non résolus est-elle juste et bien triée ?
3. **AC6-7 à trois transactions (C-27-4-29)** : `tBas` et `tHaut` rejetées avant le lot, `tMil` supprimée pendant,
   lot `[tHaut, tMil, tBas]` → `failed = [tBas, tMil, tHaut]`. Les **deux** ordres de concaténation fautifs
   rougissent-ils, et aucune autre variante fautive raisonnable ne reste-t-elle verte ? Le sous-lot éligible
   (`[tMil]`) prend-il bien le verrou ? Trois imports « créés dans cet ordre » garantissent-ils `tBas < tMil < tHaut`
   avec les aides du fichier (`seed_bank_transactions`, `reconciliation_e2e.rs`) ?
4. **`AND matched_entry_id IS NULL` (C-27-4-30)** : la clause change-t-elle un comportement **atteignable** ? M19
   est-elle vraiment équivalente (rejoue la recherche des écrivains de `matched_entry_id`) ? M18 l'est-elle encore
   avec la clause ? La ligne du tableau des sites non résolus (étendue aux sites split et règle, `:2300`, `:2701`)
   est-elle exacte ?
5. **AC9, manuel par la conduite (C-27-4-28)** : la puce prescrite — refusée avec son motif sous *Échecs partiels*,
   les autres traitées, la liste rechargée ne la propose plus, rien à refaire — est-elle **vraie au code** pour
   l'acceptation (facture, règle, ventilée) **et** pour le rejet, et pour une suppression avant **et** pendant le
   lot ? Le rechargement automatique (`ReconciliationProposals.svelte:229`, `:248`) a-t-il lieu aussi quand le lot
   échoue en entier (`409`, `400`) ? Le témoin « les autres propositions du lot sont traitées » est-il sans
   ligature ni apostrophe, et absent du PDF aujourd'hui ?
6. **Les corrections LOW** : réserve `innodb_snapshot_isolation` (Pièges connus) exacte au regard de
   `bank_imports.rs:400-405` ; limite de corps d'Axum à l'AC10 (2 Mo par défaut ? `reconciliation.rs:542-550`
   est-il bien la branche atteinte ?) ; `details: null` à l'AC6-4 tue-t-il M3 ; forme `error_code: "…"` imposée à
   l'AC1 ; texte SQL partagé (C-27-4-32) compatible avec la signature générique de
   `find_pending_by_id_for_account` ; table des fichiers communs recalculée.
7. **Propagation** : chaque symptôme corrigé en P2 (`pas de confusion`, `M1-M18`, `M17 et M18`, `aucun chemin ne
   change`, `1464`, `3182-3189`, `450-474`, `2032 → 2034`, `conflit certain`, `deux motifs`, « Transaction
   introuvable » comme témoin du PDF) a-t-il un résidu non historique dans la fiche, le registre ou ce prompt ?
   Greppe la **valeur**, pas la formulation.

## Axes prioritaires (communs aux deux lentilles, repris des P1 et P2)

1. **Inventaire des lots et des sites non résolus** (Dev Notes, deux tableaux et le triage) : refais les commandes
   copiées et **élargis-les** — un détecteur part du symptôme, pas d'une liste. Tout chemin qui lit
   `bank_transactions` pour un lot, tout site qui rend un refus lié à l'état d'une transaction
   (`BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`,
   `RECONCILIATION_TRANSACTION_NOT_PENDING`, `race_during_update`, `OptimisticLockConflict`,
   `AppError::Validation` après une lecture), et tout **écrivain** de `bank_transactions` (dont la restauration),
   dans `routes/reconciliation.rs`, mais aussi `reconciliation_rules.rs`, `bank_imports.rs`, `admin.rs`,
   `backup.rs`, `payment_batches.rs`, `kesh-reconciliation`, le frontend. Un lot ou un site manquant, ou un site
   déclaré « résolu » que la fiche ne résout pas, est au moins **MEDIUM**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Supprimer un import*
   (l. 1722-1760, dont la puce l. 1756-1759), § *Accepter ou rejeter les propositions* (l. 1906-1925, dont la puce
   *Rejeter* l. 1920 et le paragraphe *Un refus n'empêche pas les autres*, l. 1925). Ce que le manuel dit
   **aujourd'hui** est-il vrai au code ? Ce que l'AC9 prescrit d'écrire sera-t-il vrai au code que la fiche
   prescrit — y compris pour les sites qu'elle laisse non résolus ? Contrôle aussi le **PDF**, aplati **et
   normalisé** (ligatures, apostrophe typographique) :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`.
   Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 26 → 27 littéraux
   (`grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`), 28 → 29
   codes, `sitesTotal` 2041 → 2043 (`frontend/src/lib/shared/i18n-keys.test.ts:580`, et la méthode de comptage du
   fichier), 13 AC, AC6-1 à AC6-8 (le 5. en deux cas), T0-T8, mutations **M1-M19 (20 lignes, M7a/M7b)**, **32
   choix**, « refusé en entier » 2 → 1 dans le PDF, chaque numéro de ligne cité — dont ceux ajoutés par la
   remédiation P2 : `reconciliation.rs:3183-3190`, `:1467-1468`, `:2300`, `:2701`, `:542-550`,
   `repositories/reconciliation.rs:18`, `:322`, `:337-356`, `failed-proposal-label.ts:105-107`,
   `failed-proposal-label.test.ts:66`, `bank-import-deletion.ts:12-14`, `ReconciliationProposals.test.ts:450-476`,
   `test_fixtures.rs:547-550`, `bank_imports.rs:400-405`.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #634 couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui
  rougirait** ; chaque fait, chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes
  prescrits (libellés et traductions de l'AC7 et des Dev Notes, manuel, `api-external.md`, CHANGELOG) confrontés à ce
  que le code prescrit fera réellement et à `docs/i18n-glossaire.md` ; les tests existants qui changeront de sens
  (`reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`, `reconciliation_split_e2e.rs`, `rejeu_interblocage_e2e.rs`,
  `reconciliation_repository.rs`, `bank_imports_e2e.rs`, `bank_import_delete.rs`, `failed-proposal-label.test.ts`,
  `ReconciliationProposals.test.ts`, `i18n-keys.test.ts`, `textes_coherents.rs`, `lint-i18n-ownership`) — la fiche
  les nomme-t-elle tous ? La table des mutations : chaque mutation est-elle rattachée à un test qui la tue
  réellement, ou en existe-t-il une **équivalente** non déclarée — ou une déclarée équivalente qui ne l'est pas ?
  Les fichiers communs avec `story/27-2-devise-import-csv` et `story/27-3-profil-apres-apercu`
  (`git diff --stat origin/main...<branche>`, lecture seule) : la table recalculée est-elle complète et exacte ?
- **F — Faisabilité, concurrence et contrat (adversaire de la conception)** : la classification de l'AC1 et sa
  priorité, fondée sur `is_reconciled`, sont-elles justes au regard de **tous** les écrivains de
  `bank_transactions`, restauration comprise ? L'AC4 est-elle correcte **sous InnoDB REPEATABLE READ** : `UPDATE`
  gardé (désormais `bank_account_id`, `matched_entry_id`, `auto_match_rejected_at`) à 0 ligne puis
  `find_by_id_for_account_for_update` (lecture courante vs vue cohérente, verrou d'intervalle sur une ligne absente,
  ordre des verrous contre `bank_imports::delete_unreconciled_in_tx`, contre un lot d'acceptation et contre une
  restauration) ? Les montages des AC6-2, AC6-5 (a) et (b), AC6-7 et **AC6-8** sont-ils déterministes — un `DELETE`
  ou un `UPDATE` tenu par le test peut-il bloquer autre chose que l'`UPDATE` visé ? Le court-circuit de l'AC3 et le
  pré-vol hors de `retry_with` laissent-ils une course ou un double traitement au rejeu ? La fusion de `failed[]`
  est-elle tenable sans ambiguïté ? L'anti-énumération (AC1, AC6-3) tient-elle aussi au temps de réponse et aux
  traces ? Le contrat publié réécrit (`docs/api-external.md:536-548`, plus la limite de corps) : un client existant
  peut-il encore s'y tromper ?

## Ce que tu rends

Findings numérotés (`R3-…` ou `F3-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée — **la plus simple qui ferme le défaut** :
une correction qui ajoute un mécanisme (aide de test neuve, montage supplémentaire) doit dire pourquoi une
suppression ou une déclaration d'angle mort ne suffirait pas. Pour chaque finding, sa **classe (i) à (iv)** et la
réponse à la question du recyclage. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-4-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité et par classe (i)-(iv), une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `docker`, `mariadb`/`mysql`, `bash` ou `sh` sur un
script du dépôt, `gh` en écriture, toute requête SQL, et **tout interpréteur** — `python3`, `python`, `node`,
`deno`, `perl`, `ruby`, `awk` en programme écrit dans un fichier. Rien de `~/kesh-test/` (relevé réel de Guy) ne se
lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `uniq`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard (ou vers un fichier de `kesh-gate-logs/`).
