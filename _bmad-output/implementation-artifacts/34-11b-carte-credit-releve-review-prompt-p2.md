# Prompt — revue de code P2 de la Story 34-11b (relevé d'une carte de crédit, virement des deux côtés, #972), trois lentilles, passe complète

Tu relis la **Story 34-11b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**, Epic 34,
**après la remédiation de la revue P1**. Modèle : **Sonnet** (passe complète, contexte frais ; jamais Haiku). Langue du rapport : français.
**Aucune migration.**

La story : l'écriture d'un virement entre deux comptes propres, déjà rapprochée d'un côté, devient candidate au rattachement de l'autre côté
(condition 4, `reconciliation_link.rs`) ; annuler un côté ne retire que son lien (`reconciliation_cancel::cancel_in_tx`) ; `unlinkOnly` à la vue,
aux candidates, à la trace ; `reversalJournalEntryId` nullable ; l'avertissement « déjà comptabilisée » de la 34-4 conseille de rattacher le
second côté ; un manuel pas à pas pour tenir une carte de crédit.

**Ce que la remédiation P1 a changé** (commit `1b83c157` ; rapports P1 :
`/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p1-{B,E,A}.md`) :
- **E1-1/E1-2** : `POST …/cancel` accepte `{ "expectUnlinkOnly": bool }` ; `cancel_in_tx` (étape 3 ter) refuse
  `409 RECONCILIATION_CANCEL_CHANGED` si la décision du geste diffère ; l'écran l'envoie toujours et relit la vue sur ce refus ;
- **B1-1** : (c′) compte à part les non rapprochées (`-one`/`-other` sur `k − m`) et les écritures de virement
  (`-transfer-among-one`/`-other`) ;
- **E1-3** : `other_side_predicate` exclut les écritures de règlement (client, fournisseur payée) ;
- **B1-4** : `CancelOutlook` (`entry_cancel_outlook`, `cancel_blocker`) — une seule lecture de l'autre côté ;
- textes (B1-5, A1-10, B1-6, A1-5, A1-3), limite des comptes partagés au manuel (B1-2), E2E sur un compte bancaire propre (B1-8, A1-6),
  tests neufs (A1-2, A1-7, A1-11), M2b déclarée équivalente (B1-3, E1-4), M7c rejouée avec la dévalidation fournisseur (A1-8).
- **Renvoyé à l'intégration** (consigne) : le PDF du manuel (A1-9, B1-7) et le pas à pas *Dette* (A1-4, la 34-2 est dans la release).

Worktree `/home/gcorbaz/devel/kesh-e34b`, branche `story/34-11b-carte-credit-releve`. Diff **aplati** de la story, avant la fusion de
la 34-11a2 : `git -C /home/gcorbaz/devel/kesh-e34b diff a8dce4a9 1b83c157` (`a8dce4a9` = fusion de la 34-4 dans la base ; `1b83c157` =
dernier commit de code de la story). Diff de la seule remédiation : `git diff 96f8ee70 1b83c157`. Puis la fusion, ci-dessous.

**La fusion de la 34-11a2 close** (`2dbcc102`, `merge --no-ff` de `story/34-11a2-carte-credit-creation` à `09aaaf09`) est **dans le
périmètre** : `git show --stat 2dbcc102`, `git diff 2dbcc102^1 2dbcc102` (ce que la fusion a apporté à la 34-11b) et
`git diff 2dbcc102^2 2dbcc102` (ce que la 34-11b ajoute à la 34-11a2 close). Conflits résolus à la main : `CHANGELOG.md` (ligne « Pour
l'API » de la 34-11a2 reprise, ordre des refus du `PUT`) et `sprint-status.yaml` (34-11a2 `done`, 34-11b `review`) ; le reste fusionné par
git (dont `docs/api-external.md`, `errors.rs`, `user-manual.tex`, les locales). Cherche : une interaction entre la remédiation P1 de la
34-11a2 (`PATCH` d'une carte, relecture du type, `mask_ibans`, `cardLast4`, union TS) et le relevé de carte, la réconciliation ou l'E2E de
la 34-11b ; des doublons ou contradictions dans le CHANGELOG, `api-external.md` (notes ¹¹/¹²) et le manuel ; un compteur (`sitesTotal`,
gardes de textes) faux après fusion. Les gates du Dev Agent Record ont été rejoués **au commit de fusion**.

⚠️ **Les propriétés centrales.**
1. **Jamais deux écritures pour un virement, jamais un virement compté deux fois, jamais une écriture contre-passée avec un lien vivant, jamais
   une contre-passation que la confirmation excluait.**
2. **Une écriture porte au plus deux transactions**, de comptes bancaires différents et de montants opposés ; jamais une écriture de règlement.
3. **Une seule définition de « l'autre côté »** (`other_side_predicate`) pour la liste, le geste, le compte par page, la vue et l'annulation.
4. **Un appariement automatique propose, il ne crée jamais** (`CLAUDE.md`).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/34-11b-carte-credit-releve.md`
(AC, mutations, **Change Log**, **Dev Agent Record**) ; les registres `e34-choix-autonomes-e341.md` (C-e341-22 à 28, 35 à 37),
`e34-choix-autonomes-e344.md` (C-e344-11), `e34-choix-autonomes-e34b.md` (C-e34b-1 à 12). Recoupe le Dev Agent Record avec les journaux
(lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34b/` : `rouges*.log`, `gates-*.txt`, `gate-*.log`, `front-*.log`, `e2e-r*/`,
`mut/*.log` (`resume.log`, `resume-j3.log`) ; scripts `scripts/mut-e34b.py`, `scripts/mut-run*.sh`, `rouges*.sh`, `gates*.sh`, `e2e-e34b.sh`
(**à lire, jamais à exécuter**). **Captures** (thème clair) : `captures/34-11b-*.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune redirection**
  (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni **`pkill`**, ni **`killall`** ; **aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La décision portée par le geste** (E1-1) : étape 3 ter de `cancel_in_tx` — placée après le verrou de l'écriture et l'`ensure_open`,
   avant toute écriture ? Les deux sens refusés (tests `s34_11b_e1_1_…`, `s34_11b_e1_2_…`) ; sans corps, comportement d'avant ; le rejeu sur
   interblocage (`retry_app_on_deadlock`) et le 409 ; l'écran (`CancelReconciliationDialog.svelte`) : relecture, message, double clic, `view`
   nul ; la page facture (`invoices/[id]/+page.svelte`) qui ouvre le même dialogue ; `reconciliation.api.ts`.
2. **La condition 4 et le prédicat** (`other_side_predicate`, `linkable_side`, `HAS_LINK`, `entry_sql`, `count_already_booked_for_page`,
   `transfer_other_side`) : ordre des `?`/`bind` (le prédicat a gagné deux sous-requêtes sans paramètre) ; l'exclusion des règlements —
   règlement client par compte interne (rattachable une fois, puis ?), facture fournisseur payée puis annulée (paiement détaché : `Entry`) ;
   cas déroulés à la main (virement A→B, comptes partagés, relié ailleurs — forme admise, virement éclaté — refusé, troisième transaction).
3. **`CancelOutlook`** : une lecture par candidate, par vue, par geste ; `None` strict des écrans de facture ; lecteurs de `matched_entry_id`.
4. **Les écrans** : `already-booked.ts` ((a), (b′), (c′) — accords, `k − m`), `LinkSettlementModal.svelte` (« Aujourd'hui… »), les quatre
   locales (clés `-transfer-among-one`/`-other`, `reconciliation-cancel-changed` côté serveur), `sitesTotal`.
5. **Les manuels et la documentation contre le code** : `docs/manual/fr/user-manual.tex` (§ *Tenir une carte de crédit, pas à pas* — rejoue
   l'exemple chiffré —, *Une transaction qui semble déjà comptabilisée*, l'encadré « Dépense déjà réglée », *Rattacher …*, *Annuler un
   rapprochement*), `docs/api-external.md` (corps facultatif, `409 RECONCILIATION_CANCEL_CHANGED`, tableau des erreurs, `kind: entry`),
   `CHANGELOG.md`. **Le PDF n'est pas régénéré dans la story** (à l'intégration) : contrôle le `.tex` et dis-le.
6. **Les tests et les mutations** : chaque mutation (M1 à M13, Macc, F1 à F4 ; M2b déclarée équivalente) a-t-elle été jouée **au dernier
   commit de code**, et le test qui la tue l'est-il pour la bonne raison ? L'E2E dépend-il encore d'un état partagé ?

### Lentille B — Blind Hunter (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p2-B.md`)

Lis le **diff seul** d'abord, sans la fiche ; puis recoupe.

### Lentille E — Edge Case Hunter (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p2-E.md`)

Parcours chaque branche et chaque borne des axes 1 à 3 ; ne rapporte que les cas non traités.

### Lentille A — Acceptance Auditor (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p2-A.md`)

AC-1 à AC-10, les findings P1 un par un (traité, comment, avec quelle preuve ; ou renvoyé, et où c'est écrit), recompte des décomptes du Dev
Agent Record depuis la source, registre e34b.

## Rapport

Pour chaque finding : identifiant (`B2-n`, `E2-n`, `A2-n`), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, preuve, conséquence,
correction proposée. Puis **la liste des axes exercés et de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte pas.
