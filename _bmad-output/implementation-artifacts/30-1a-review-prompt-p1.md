# Prompt — revue de code P1 de la Story 30-1a (trois lentilles B, E, A — Sonnet)

Tu relis le développement de la **Story 30-1a — annuler le rapprochement d'un règlement fournisseur** de Kesh
(comptabilité suisse, Rust/Axum + SvelteKit + MariaDB 10.11), dans le worktree `/home/gcorbaz/devel/kesh-30-1`, branche
`story/30-1`. Le diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-30-1 diff 62af52d6 HEAD` (base : dernier
commit de la fiche validée). Langue du rapport : français. Contexte frais : ne présume rien du travail de l'agent.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`, **partie
a** (AC-a1 à AC-a9, tâches 30-1a, tests T-a*, mutations M-a*, angles morts), son **Dev Agent Record** (ce que l'agent
déclare avoir exécuté), le registre `epic-30-choix-autonomes-301.md` (C-30-1-42 à C-30-1-46, choix du développement),
et le `CLAUDE.md` du worktree (§ *Migration breaking policy*, § *Pattern batch*, § *Review Iteration Rule*).
**La 30-1b (proposer, accepter, dialogue, manuel) n'est PAS dans le périmètre** : ne reproche pas son absence.

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `--detach`, ni sur un fichier), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `rebase`, `tag`, `push`, `restore`).
- **N'exécute rien qui écrit, compile ou exécute** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni `cargo`, ni `npm`/`npx`, ni `make`, ni `latexmk`, ni `python3`, ni `sed -i`, ni `perl`, ni
  `docker`, ni `mariadb`. Aucune commande `gh` qui écrit.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, et
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` **vers la sortie standard seulement** (jamais vers un fichier).
- Aucune donnée réelle dans le rapport (`docs/agents-orchestration.md` § 8).

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention. Cherche : défaut de logique, erreur de borne, `match`
non exhaustif ou bras faux, valeur nulle mal traitée (`supplierInvoiceNumber` nullable), régression de contrat JSON
(un champ existant qui change de sens), message d'erreur faux, doc-comment qui contredit le code, code mort, test qui
ne prouve rien (assertion tautologique, décor vide, sonde qui rend vrai trop tôt), DRY (un symétrique client recopié au
lieu d'être appelé). Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301a-review-p1-B.md`.

## Lentille E — Edge cases (chemins, verrous, concurrence)

Parcours chaque branche et chaque ordre de verrous neuf :

1. `crates/kesh-db/src/repositories/reconciliation_cancel.rs` : `classify` (deux bras dans la **même** forme
   verrouillante ; règlement détaché → `entry`), `cancel_blocker` (forme **exemptée** pour `bt.id`), `cancel_in_tx`
   (⛔ aucune lecture non verrouillante avant l'étape 3, y compris pour le bras fournisseur ; ordre facture → écriture +
   exercice ; revérification verrouillante du compte en dernière acquisition), `get_view`. Que se passe-t-il pour une
   transaction rapprochée d'une écriture qui serait **à la fois** un règlement client et l'écriture de règlement d'une
   facture fournisseur (état impossible ? par quelle contrainte ?), pour un règlement fournisseur **par compte interne**
   (caisse) rapproché, pour une facture d'une **autre société** portant la même écriture ?
2. Le cycle constaté au développement (C-30-1-42) : la lecture verrouillante par l'index secondaire contre
   `supplier_invoices::cancel_in_tx` et contre `cancel_settlement_in_tx` (fiche) — la victime écrit-elle quelque chose ?
   chaque route concernée rejoue-t-elle (`grep -n retry` dans les routes) ? D'autres écrivains de
   `settlement_journal_entry_id` (`pay_in_tx`, `confirm_batch`) forment-ils un cycle non rejoué ?
3. `crates/kesh-db/src/repositories/payment_batches.rs::create_batch` : verrouillage de tête (liste vide, doublons,
   identifiant d'une autre société, identifiant absent), naissance de la vue (première lecture ordinaire **après** le
   dernier verrou de facture), `FORCE INDEX (PRIMARY)` (le nom existe-t-il dans le schéma réel :
   `crates/kesh-db/migrations/` et `crates/kesh-db/test-schema/`), ordre `failed[]` = ordre de la requête.
4. `crates/kesh-api/src/routes/payment_batches.rs` : borne de 500 sur la longueur **brute**, avant toute lecture ;
   rejeu sûr (une tentative ne rend `Err` qu'annulée ? rien d'écrit hors transaction ? audit écrit une fois ?).
5. Les tests de course (T-a6, T-a11, T-a15, T-a16, T-a14) : chorégraphie réellement forcée ? sondes **sans**
   `INNODB_TRX` ? un test peut-il passer à vide ?

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301a-review-p1-E.md`.

## Lentille A — Acceptance (la fiche contre le code, et le contrat écrit)

1. Pour **chaque** AC-a1 à AC-a9 : satisfait, partiel ou absent — preuve `fichier:ligne`. Pour chaque test T-a1 à
   T-a17 (sauf T-a9, déplacé) : où est-il, asserte-t-il ce que la fiche exige ? Pour chaque mutation M-a1, M-a3, M-a4,
   M-a5, M-a7, M-a8, M-a9 : le Dev Agent Record déclare-t-elle tuée, par quel test, et la mutation décrite est-elle bien
   celle de la fiche ? M-a2 (équivalente) et M-a6 (retirée) ne sont pas à exécuter.
2. **Écarts déclarés** (C-30-1-42 à C-30-1-46) : chacun est-il justifié, et le code les suit-il ?
3. **Textes** : le texte « rapproché » (`settlement-cancel-blocked-bank-match`) est-il identique, mot pour mot, dans
   `crates/kesh-i18n/locales/fr-CH/messages.ftl`, le repli `crates/kesh-api/src/errors.rs` et le repli
   `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts` ? Les trois autres locales disent-elles la même chose ?
   Greppe la **valeur** de l'ancien texte (`cette fiche ou le détail`, `this record or`, `diesem Beleg oder`,
   `questa scheda o`) sur tout le worktree hors `target/` et `node_modules/`.
4. **Contrat écrit** : `docs/api-external.md` (§ *Annuler un rapprochement bancaire*, § des lots de paiement après le
   refus de `pay`, § *Interblocages*) et `CHANGELOG.md` (`## [0.14.0] — Non publié`) disent-ils exactement ce que fait le
   code (champs, codes, statuts, borne, rejeu) ? Aucun mot-clé de fermeture d'issue (`closes`, `fixes`, `resolves`) dans
   les messages de commit (`git log 62af52d6..HEAD`) ni dans le diff.
5. **Manuel** : la fiche réserve le manuel à la 30-1b. Vérifie qu'**aucun** passage de `docs/manual/fr/user-manual.tex`
   ni de son **PDF** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, sortie standard) n'est
   rendu faux par la 30-1a : texte « rapproché » d'une annulation de règlement, création d'un lot de paiement (borne,
   double lot), annulation d'un rapprochement.
6. **Registres** : `crates/kesh-api/tests/audit_route_registry.rs` — les quatre sites (ii), (iii bis), (vi) et sa phrase
   finale disent-ils huit / quinze, et la route est-elle toujours classée `SansEcritureAuJournal` ?

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301a-review-p1-A.md`.

## Rapport (chaque lentille)

- Findings numérotés `B1…` / `E1…` / `A1…`, chacun : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`,
  **preuve copiée** (sortie `grep -nF` ou `sed -n`) pour toute affirmation de présence ou d'absence, correctif proposé.
- **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans cette liste ne compte
  pas comme passe.
- Rien d'autre n'est écrit nulle part.
