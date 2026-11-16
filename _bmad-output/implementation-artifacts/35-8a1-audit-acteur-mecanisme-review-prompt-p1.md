# Prompt — revue de code P1 de la Story 35-8a1 (#431, #892 item 11), trois lentilles, première passe complète

Tu relis la **Story 35-8a1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **l'audit attribue à la clé d'API ce qu'elle a fait — le mécanisme et le pilote**. (1) Le type
`Acteur { user_id, api_key_id }` (`kesh-db/src/entities/audit_log.rs`), son seul constructeur sans clé `Acteur::utilisateur`, le
constructeur d'entrée `NewAuditLogEntry::par` (délègue à `for_actor`), `letterings::Actor` devenu un alias, `impl From<&CurrentUser>
for Acteur` et `AuditActor::from_current_user` réécrit par `par` (`kesh-api/src/audit.rs`). (2) Le **pilote** : `supplier_invoices::
unvalidate` / `unvalidate_in_tx` prennent `acteur: Acteur` ; la trace `supplier_invoice.unvalidated`, les deux `journal_entry.deleted`
et le `lettering.removed` des paires `reversal` portent la clé ; la route passe `Acteur::from(&current_user)` ; la signature de
`delete_in_tx` ne change pas (35-8a2). (3) La **garde des sites non résolus** `crates/kesh-api/tests/audit_acteur_garde.rs` (neuve) :
cinq motifs, table close **par site** `(fichier, fonction englobante, motif) → (nombre, raison, appelant)`, huit sondes ; et
l'extraction de `strip_line_comments`, `source_assainie`, `accolades_hors_chaines` (avec le parcours des fichiers et `args_de`,
C-e358a1-1) vers `crates/kesh-api/tests/common/source_assainie.rs`, partagé avec `audit_label_registry.rs`, qui connaît désormais
`NewAuditLogEntry::par(`. **Aucune migration** ; aucune route, aucune réponse d'API changée ; le passé ne se corrige pas.

Worktree `/home/gcorbaz/devel/kesh-e358a1`, branche `story/35-8a1-audit-acteur-mecanisme`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e358a1 diff 02be2ad7f bcdd05dd7` — `02be2ad7f` = la base (la 35-1 close, qui porte 35-2a,
35-2b, 35-3 et 35-5a0) ; `bcdd05dd7` = l'implémentation, le seul et dernier commit qui touche du code (le suivant ne porte que la
fiche, le suivi et ce prompt). Langue du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais
Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-8a1-audit-acteur-mecanisme.md` en entier (*Ce qui se lit au code*, D1 à D5, AC3, AC5,
AC6a, mutations M1, M2, M4, M5, M6, **Change Log** et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e350.md` (C-e350-8,
-38 à -40, -48 à -50, -52) et `e35-choix-autonomes-e358a1.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e358a1/` : `inventaire-base-02be2ad7f.txt`,
`rouges.log`, `garde-releve-initial.log`, `gardes-vert.log`, `ac3-vert.log`, `test-fast-ci-*.log`, `gate-ci.log`,
`manuels-make.log`, `mut/` (`M1.log` … `M6.log` pour les gardes, `M1-ac3.log`, `M2-ac3.log`, `M4a-ac3.log`, `M4b-ac3.log` pour
AC3 — le premier passage n'avait pas lancé AC3, dit au Dev Agent Record).

⚠️ **L'E2E complet n'a pas encore tourné** (il se joue à la clôture de la story ; consigne de l'Epic 35). **`gate-ci` a tourné** au
passage en `review` (Dev Agent Record, `gate-ci.log`). Le dire n'est pas un finding ; affirmer le contraire au Dev Agent Record en
serait un. **Aucun contrôle visible ne change** : pas de capture. Les scripts `*.sh`, `*.py` sont à **lire**, jamais à exécuter.
**Manuels** : `docs/manual/fr/admin-manual.tex` (item *Journal d'audit* de la section des clés d'API) et
`docs/manual/fr/user-manual.tex` (*Consulter le journal d'audit*, item *Le tableau*), et leurs PDF `docs/manual/fr/admin-manual.pdf`,
`docs/manual/fr/user-manual.pdf` (à contrôler **aplatis**) ; `docs/api-external.md` (§ dévalidation d'une facture fournisseur, et le
paragraphe des traces du lettrage) ; `CHANGELOG.md` `## [0.17.7] — Non publié`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris) ; **aucune substitution de processus**
  (`<(…)`, `>(…)`) ni substitution de commande qui écrive. Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le type et le pont (D1)** : `Acteur` (dérivations `Debug, Clone, Copy, PartialEq, Eq`), `Acteur::utilisateur`,
   `NewAuditLogEntry::par` (délègue-t-il exactement à `for_actor` ?), l'alias `letterings::Actor` (les littéraux `Actor { … }` de
   production et de tests compilent-ils toujours, sans changer de sens ?), la réexportation dans `entities/mod.rs`,
   `From<&CurrentUser>` et `from_current_user` : le comportement d'une session et d'une clé est-il **strictement** celui d'avant
   pour les quelque soixante appelants de `from_current_user` ? Le site `PONT` a-t-il disparu de l'inventaire ?
2. **Le pilote (D3)** : `unvalidate`, `unvalidate_in_tx` (`supplier_invoices.rs`) — reste-t-il un `user_id` nu dans la fonction
   (`awk` sur sa plage) ? La dissolution (`dissolve_group_in_tx`) reçoit-elle l'acteur ; les deux `delete_in_tx` reçoivent-ils
   `acteur.user_id, acteur.api_key_id` dans le **même ordre** que la signature (`journal_entries.rs`, `delete_in_tx`) ? La route ;
   le rejeu sur interblocage (`retry_on_deadlock`, l'acteur est `Copy`) ; un autre appelant de production d'`unvalidate*` (`git
   grep`) ? Les 30 appels de tests passent-ils `Acteur::utilisateur(<le même user_id qu'avant>)`, sans autre changement
   (`git diff 02be2ad7f bcdd05dd7 -- crates/kesh-db/tests crates/kesh-api/tests`, lignes `-`) ?
3. **La garde (D2)** : les cinq motifs (regex, motif 5 par `args_de`), `fonctions_par_ligne` (signature multi-lignes, déclaration de
   trait, fonction imbriquée, fermeture, `impl` sans méthode, `fn` dans une chaîne), `sites_de`, `ecarts` (bilatéral, nomme
   fichier/fonction/motif), la table `SITES` : **recompte-la depuis la source** (inventaire indépendant, par `git grep` et lecture des
   fonctions) — 85 entrées ; chaque raison finale (`PreAuth`, `Systeme`, `Restauration`, `Definition`, `Mcp`, `AdminSansCle`)
   est-elle vraie au code (route sous `require_not_pat` ou handler `ensure_not_pat`, ligne de `lib.rs` citée juste) ? Une fonction de
   raison finale est-elle **aussi** atteinte par une route comptable (elle devrait être `AConvertir`) ? Les sondes (a) à (h)
   peuvent-elles rougir, et la garde réelle a-t-elle rougi sous les mutations jouées sur l'arbre (M1, M4a, M4b, M5 : `mut/`) ? Les angles
   morts écrits au doc-comment sont-ils exacts (comptes, fichiers) et complets — en trouves-tu un autre ?
4. **L'extraction (C-e358a1-1, -2)** : `common/source_assainie.rs` reprend-il **à l'octet près** le corps des fonctions retirées de
   `audit_label_registry.rs` (`git show 02be2ad7f:crates/kesh-api/tests/audit_label_registry.rs`) ? `audit_label_registry.rs` : ses
   onze tests inchangés ; `FORMES` porte `NewAuditLogEntry::par(` en `(1, Some(2))` ; `audit.rs` reste-t-il juste dans
   `SITES_INDIRECTS` (son action est désormais un paramètre de `par`) ?
5. **Le test d'AC3** (`supplier_invoice_draft_e2e.rs`, trois tests) : tout par l'API, clé créée **par le Comptable** ; la borne
   `MAX(id)` laisse-t-elle hors du relevé les traces de l'annulation ; « aucune autre trace » ; le témoin par session ; peut-il passer
   à vide (zéro trace) ? A-t-il rougi au départ (`rouges.log`) ?
6. **Manuels, API, CHANGELOG (AC6a)** : chaque phrase vraie au code — « une partie des dépôts » (liste), « la modification et la
   suppression d'une écriture » attribuent déjà la clé (vérifie `journal_entries.rs`), « dans toutes ses traces », « les entrées
   écrites avant la v0.17.7 … ne se corrigent pas » ; les **PDF aplatis** disent-ils le `.tex` ? `api-external.md` : les deux
   ajouts ; CHANGELOG : `refs #892`, `refs #431`, la phrase sur les entrées antérieures, **aucun** mot-clé de fermeture.
7. **Les mutations** : M1, M2, M4a, M4b, M5, M6 (`mut/`), chacune tuée par une assertion **nommée** ; M4a et M4b tuées aussi par la
   garde ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `audit_log.rs`, `audit.rs`, `letterings.rs`, le pilote de `supplier_invoices.rs`, la route, la garde, le module
commun, le test d'AC3. Cherche : une clé perdue en chemin ; un test qui passe **à vide** ; une regex trop étroite ou trop large ; une
attribution de fonction fausse ; un écart muet entre la table et le code.

## Lentille E — Edge (chaque branche, chaque borne)

Facture validée, annulée sans paire `reversal` lettrable, annulée avec plusieurs paires ; refus en cours de geste (rien d'écrit) ; clé
`read` (403 avant) ; clé révoquée ; session JWT ; le même geste rejoué sur interblocage ; pour la garde : site dans une macro, dans un
`impl Trait for X`, dans une fermeture, dans un `const`, après un `#[cfg(test)] mod x;`, sous `#[cfg(not(test))]`, coupé sur deux
lignes, fichier `src` entièrement de test. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

AC3, AC5, AC6a : livrés, testés, au lieu prescrit ? L'étape 0 (inventaire relancé, lignes relues sur la base `02be2ad7f`) citée au
Change Log ? Les gates au dernier commit de code (`test-fast --ci` voie 2, `gate-ci` voie 1 ; frontend non touché ; E2E à la
clôture) : chaque chiffre au journal, avec son commit ? Les écarts déclarés (consigne Edit : `replace_all` de l'outil Edit dans la
garde, script Python de mutation ; extraction élargie C-e358a1-1) sont-ils exacts et sans effet sur le livré ? Le Dev Agent Record
n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e358a1-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
