# Prompt — revue de code P1 de la Story 36-4 (refs #1131, refs #431), trois lentilles, première passe complète

Tu relis la **Story 36-4** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**, Epic 36,
phase 1 : **l'annulation d'un règlement par une clé d'API est attribuée à la clé** — un règlement ou un solde de facture client, le
paiement d'une facture fournisseur —, par la route d'annulation du règlement ou par le dé-rapprochement bancaire qui l'avait passé.
Les deux `cancel_settlement_in_tx` (`crates/kesh-db/src/repositories/invoice_settlements_write.rs`,
`crates/kesh-db/src/repositories/supplier_invoices.rs`), leurs enveloppes `cancel_settlement`,
`reconciliation_cancel::cancel_in_tx` et `cancel` prennent un `Acteur` (type de la 35-8a1) ; les trois routes
(`routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/reconciliation.rs`) le tirent de `CurrentUser`. **Aucune migration.**
Kesh tient des livres réels : une trace d'audit qui ment sur l'auteur est une piste de contrôle fausse (CO art. 957-964, OLICo
art. 9).

Worktree `/home/gcorbaz/devel/kesh-36-4`, branche `story/36-4-acteur-annulation-reglement`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-4 diff 846d64e98 c5d67ce42` (`846d64e98` = la fiche validée, sur le kickoff `9d5a142df` ;
`c5d67ce42` = le dernier commit de code) ; le commit des rouges est `80bc35072` ; `e958fb169` porte les documents et les PDF ; le
dernier, la fiche, le registre, `sprint-status.yaml` et ce prompt (`git log --oneline 846d64e98..HEAD`). ⚠️ Le diff compte ≈ 30 fichiers dont 17 de tests
à conversion **mécanique** (l'argument utilisateur devient `Acteur::utilisateur(…)`), faite par l'outil Edit avec des remplacements
multiples : un remplacement a d'abord touché à tort cinq appels de `supplier_invoices::cancel` / `cancel_in_tx` (l'annulation de la
**facture**, hors périmètre), rétablis sur l'erreur du compilateur — relis la conversion **fichier par fichier** (une valeur changée,
un appel d'une autre fonction touché). Langue du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku).
Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-4-acteur-annulation-reglement.md` en entier (*Ce qui se lit au code*, D1 à D8, AC1 à
AC6, mutations, **Change Log** et **Dev Agent Record**) ; les registres `e36-choix-autonomes-36-4.md` (C-36-4-1 à -5) et
`e36-choix-autonomes-e360.md` (C-e360-7, -9) ; la fiche de la 35-8a2 (`35-8a2-audit-acteur-ecritures.md`) pour le patron. Ne prends
pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-4/` : `rouges.log`, `rouges-run1.log`, `rouges-run2.log`, `recompte-garde-apres.txt`,
`actor-api-key-id-option-apres.txt`, `mut/*.log`, `clippy-1.log`, `test-fast-ci-*.log`, `gate-ci-*.log`, `manuels-make-*.log`,
`inventaire-appels-846d64e98.txt`. Déjà tracés, à ne pas rapporter de nouveau : **#431** (les autres gestes `AConvertir`, lots 36-11a
à 36-11d), **#1130** (le `None` positionnel des autres fonctions à `actor_api_key_id: Option<i64>`), **#1132** (`pieges.rs`,
`mark_emailed`).

**Livrables visibles** : aucun écran ni route ne change ; ce qui change est **ce que le journal d'audit inscrit** (`actor_type`,
`actor_api_key_id`). **Le manuel** : `docs/manual/fr/admin-manual.tex` (ligne 3212, item *Journal d'audit* de la section des clés
d'API) et `docs/manual/fr/user-manual.tex` (lignes 4008-4026, le tableau du journal d'audit) **et leurs PDF**
`docs/manual/fr/admin-manual.pdf`, `docs/manual/fr/user-manual.pdf`, lus **aplatis**, sortie à l'écran :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (forme E6, sans
redirection). `docs/api-external.md` (lignes 404, 565, 653, 739). La **note de version** de la fiche (`## Note de version`) — la
branche n'écrit pas `CHANGELOG.md` (C-e360-11). Le manuel dit-il **exactement** ce que le code fait, geste par geste ? C'est en
vérifiant le manuel qu'on trouve les chemins oubliés. Pas de capture : aucun contrôle visible ne change.

⚠️ **Ce qui n'a pas tourné** : l'**E2E (Playwright)** et le second `gate-ci` — à la clôture (consignes de l'Epic 35). Le frontend
n'est pas touché (aucun gate frontend).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, **ni `2>&1`**, **ni `2>/dev/null`**, ni
  `> /dev/null`. **Aucune substitution de processus** (`<(…)`, `>(…)`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/jouer.sh`, `mut/muter.py`,
  `bases-a-neuf.sh`, `recompte-garde.py`, `inventaire-appels.py` : à **lire**, non à lancer) ; ni `make`, `latexmk`, `cargo`,
  `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`, E9).

## Les axes obligatoires, pour chaque lentille

1. **Les deux `cancel_settlement_in_tx`** : l'`acteur` parvient-il **à chaque trace** qu'ils écrivent — `lettering.removed` (le
   délettrage de la pièce, `letterings::dissolve_*_document_group_in_tx`), `journal_entry.created` / `journal_entry.reversed` et le
   lettrage `reversal` (`journal_entries::reverse_owned_in_tx`), le geste `*.settlement_cancelled` ? Un `acteur.user_id` resté là où
   l'acteur entier devait passer ? Une trace du chemin **non** énumérée (lis le chemin transitivement : `settlement_cancel_blocker`,
   `amount_due`, `ensure_open_in_tx`, `recheck_open_locked_in_tx`, `commit_with_view`, `read_after_write_in_tx`) ?
2. **Les trois routes** : l'acteur est-il tiré **une fois**, hors de la fermeture de rejeu, et le **même** à chaque rejeu ? La route
   du dé-rapprochement : `cancel_reconciliation_once` et `cancel_in_tx` passent-ils l'acteur reçu à **toutes** les branches
   (règlement client, règlement fournisseur, écriture seule, simple détachement `unlink_only`) et à `reconciliation.cancelled` ?
3. **L'enveloppe de test `reconciliation_cancel::cancel`** (D4, C-36-4-2) : son doc-comment dit-il vrai (aucun appelant de
   production : `git grep -nE 'reconciliation_cancel::cancel\(' -- 'crates/*/src'`) ?
4. **La garde** (`crates/kesh-api/tests/audit_acteur_garde.rs`) : la table recomptée depuis la source (attendu 87 entrées, 92 sites ;
   `AConvertir` 47/48 ; `User` 69/74, `ApiKeyNone` 8/8, `Utilisateur` 10/10) ; aucune entrée pour les fonctions touchées ; le
   doc-comment de l'angle mort 1 (28 lignes au `git grep`, 21 lignes de paramètre — recompte toi-même la commande qu'il cite) et
   celui de la table (lignes 175-195) : justes ?
5. **Les tests neufs** (AC1 à AC4) et l'aide `crates/kesh-api/tests/common/audit_traces.rs` : rouges prouvés sur la base pour la
   **bonne** raison (`rouges.log`, `rouges-run2.log`) ; l'assertion « toutes traces » peut-elle passer à vide (liste vide, `id`
   relevé après le geste, une trace d'un **autre** geste comptée) ? Les listes d'actions relevées (deux `lettering.created` sur les
   montages en caisse, un seul ailleurs) sont-elles expliquées juste au code (`journal_entries.rs`, étape (5) du lettrage de
   contre-passation) ? `create_key_via_http` déplacée et réexportée (`common/mod.rs`, `#[allow(unused_imports)]`) : ses 38 appelants
   d'avant compilent-ils toujours (la compilation l'a prouvé ; vois-tu un appelant qui n'inclut ni `mod.rs` ni `audit_traces.rs`) ?
6. **La conversion mécanique des tests** (17 fichiers) : relis-la fichier par fichier, en particulier
   `kesh-db/tests/supplier_invoices_repository.rs` (le remplacement multiple rétabli sur cinq appels de `cancel`/`cancel_in_tx`),
   `kesh-db/tests/invoice_settlement.rs` (le tuple `args`), `kesh-api/tests/bank_accounts_closed_e2e.rs` (un seul `acteur` pour
   trois annulations, et `supplier_invoices::cancel_in_tx` laissé à `user_id`).
7. **Mutations** (`mut/*.log`, scripts `mut/muter.py`, `mut/jouer.sh`) : M1 à M8, M1', M2', M3' tuées par les tests ou la garde
   **nommés** à la fiche ; chaque journal finit par « restauré » ; la restauration par `git restore --worktree` puis `touch` (E5).
   Propose des mutations plausibles non jouées.
8. **Manuel, PDF, `api-external.md`, note de version** (AC6) : chaque phrase dit-elle le code, geste par geste ? « ou leur
   annulation » sorti de la liste, « annulation de la facture » dans les **deux** listes d'`admin-manual.tex:3212` (V2-2), les dates
   « avant la v0.17.7 / v0.17.8 » ; les PDF contrôlés aplatis (anciennes formulations à `0`, phrase neuve à `1`).
9. **Le compte rendu** : chaque chiffre (appels convertis, table, gates) recompté depuis la source, avec son périmètre et son
   journal ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E : **non lancé**) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `invoice_settlements_write.rs`, `supplier_invoices.rs`, `reconciliation_cancel.rs`, les trois routes, la garde, l'aide
`audit_traces.rs`, puis les tests. Cherche : une clé perdue entre la route et la trace ; une fermeture qui capture le mauvais acteur ;
un commentaire devenu faux (« au nom de l'utilisateur », « `for_actor` ») ; un import devenu inutile.

## Lentille E — Edge (chaque branche, chaque borne)

Une clé `read-write` d'un Comptable, d'un Administrateur ; une session ; un rejeu sur interblocage ; un règlement **partiel** (aucun
lettrage de pièce : pas de `lettering.removed`) ; un solde (`write_off`) et le règlement qu'il bloque (`WriteOffExists`) ; un
règlement par virement vers un compte **clos** (refus : rien n'est écrit) ; le dé-rapprochement des trois genres et le simple
détachement (`unlink_only`) ; une contre-passation à l'exercice suivant. Chaque cas : comportement au code, test qui le couvre (ou
absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC6 : livré, testé, au lieu prescrit ? Les LOW V2-1 à V2-5 de la validation P2 traités à l'étape 0, ligne lue
citée ? Les écarts déclarés (C-36-4-5, listes d'actions relevées) exacts ? Les gates du passage en `review` : chaque chiffre au
journal, au dernier commit de code, horodaté après lui ? L'E2E déclaré « à la clôture » ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-4/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
