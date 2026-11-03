# Prompt — revue de code P2 de la Story v016-2a, passe CIBLÉE, une lentille, Opus

Tu relis **la seule remédiation de la revue de code P1** de la Story v016-2a de Kesh (comptabilité suisse,
Rust/Axum, MariaDB 10.11) : *saisir un relevé bancaire à la main — le backend* (#816). Passe **ciblée** (`CLAUDE.md`,
§ *La passe ciblée*) : une seule lentille, braquée sur le diff de la remédiation, non sur la story entière. Modèle :
**Opus**. Contexte frais : ne t'appuie sur aucune passe antérieure. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0162 diff 285d2fde..HEAD -- crates docs CHANGELOG.md README.md` (`285d2fde` = la fin du
développement, avant la remédiation ; les commits de `_bmad-output/` sont des comptes rendus). Les rapports de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/v0162a-review-p1-{B,E,A}.md` ; le traitement : Change Log de la sous-fiche
`_bmad-output/implementation-artifacts/v016-2a-releve-manuel-backend.md`, entrée « Revue P1 », et le registre
`v016-2-choix-autonomes.md`, **C-v016-2-24**. Journaux (lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/v0162a/`
(`p1-*.log`, `gate-2.log`, `front-e2e-2.log`, `e2e-2.log`, `mutations/bilan-p1.txt`, `mutations/M25.log` à `M29.log`,
`muter.py` et `muter_p1.py` à **lire**, jamais à exécuter).

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp` ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`,
  `2>&1`). Un fichier créé par erreur se **déclare** au rapport, il ne se supprime pas.
- **Aucun `rm`** sous aucune forme (ni `rm -f`, `rm -rf`, `unlink`, `find -delete`) ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout` ni `git switch`**, ni `git restore`, ni aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `node`/`npm`/`npx`, ni
  `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` ; ni `make`, ni **`cargo`**, ni
  **`docker`**, ni `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection ni `system()`), `cat`,
  `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, et la lecture des sources de dépendances sous
  `~/.cargo/registry/src/` (par exemple `rust_decimal-*/src/str.rs`, `serde-*/src/de/`).
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## Les axes obligatoires

1. **Le verrou et la course (E2).** `routes/bank_statements_manual.rs`, `create` et `create_under_lock` :
   - le verrou est-il bien `kesh_reconciliation::with_account_lock` (même nom `reconcile:<base>:<société>:<compte>`
     que le rapprochement, `crates/kesh-reconciliation/src/mutex.rs`) ? Est-il pris sur une connexion **distincte** de
     la transaction de création, et la transaction de création est-elle **commitée avant** le `RELEASE_LOCK` ? Lis
     `with_account_lock` : où la closure s'exécute-t-elle, quand le verrou est-il rendu ?
   - **tout** ce qui doit être sérialisé est-il sous le verrou (relevé identique, doublons de lignes, écart de solde,
     `INSERT`, trace) ? Une lecture avant le verrou peut-elle faire naître une vue trop tôt (le pré-vol
     `check_statement` lit le compte sur le pool : est-ce inoffensif ?) ;
   - les issues de `with_account_lock` : `AccountLocked` → `409 RECONCILIATION_ACCOUNT_LOCKED` ; `LockReleaseFailed`
     après une création conclue → l'issue de la création (est-ce juste ? le verrou reste-t-il tenu par une connexion
     rendue au pool, et qu'advient-il de la requête suivante ?) ; `Database` ; les autres variantes ; le cas
     `(None, Ok(()))` est-il possible ;
   - interblocage possible entre le verrou nommé et les verrous de ligne (la création prend un verrou partagé sur la
     ligne du compte, la clôture un exclusif ; le rapprochement prend le verrou nommé puis des `FOR UPDATE`) ? Deux
     connexions par requête : risque d'épuisement du pool sous charge ?
   - les tests `create_waits_for_the_account_lock_and_sees_the_committed_twin` (entrelacement forcé, sonde
     `PROCESSLIST` sur `GET_LOCK`, nom du verrou recopié dans le test) et `two_identical_simultaneous_creations_give_one_statement` :
     prouvent-ils ce qu'ils nomment ? M25 (verrou retiré) est-elle tuée par eux, pour la bonne raison ?
   - l'aperçu reste **sans** verrou : est-ce cohérent (il n'écrit rien) ?
2. **La limite de corps (E1).** `MANUAL_STATEMENT_BODY_LIMIT_BYTES` (3 Mio) monté sur les **deux** routes (`lib.rs`),
   indépendant de `KESH_BANK_IMPORT_MAX_MB` ; le calcul « 500 lignes pleines ≈ 2,1 Mo » (le test le mesure-t-il ?) ; le
   visiteur `deserialize_capped_lines` (`kesh-core/src/bank_imports/manual.rs`) : `null`, absence, autre type, 0, 500,
   501, 5000 éléments ; un élément **retenu** mal formé est-il toujours un rejet du corps, un élément **jeté** mal formé
   ne l'est-il plus (est-ce acceptable : le relevé est refusé `too_many_lines`) ? La mémoire est-elle vraiment bornée
   (`size_hint`, `with_capacity`) ? `body_limit_is_its_own` : la borne exacte (à la limite : lu ; un octet au-delà :
   `400 VALIDATION_ERROR` sans `field`). `docs/api-external.md` : le paragraphe de la limite et l'ordre des refus.
3. **La lecture exacte (B1 = A1 = E3).** `parse_decimal` : la partie décimale bornée sur le texte (zéros finals ôtés),
   la forme canonique reconstruite, `Decimal::from_str_exact` (lis sa source : peut-il encore arrondir ou échouer sur
   une forme admise ?) ; le signe (`-0`, `-0.00`, `-1250.40`) ; treize chiffres entiers et deux décimales ; une partie
   entière à zéros de tête ; `"0"`, `"0.0"`, `"00"`. Le test `parse_reads_amounts_exactly_and_refuses_long_fractions`
   et `rule_5_scale_is_checked_after_normalize_on_the_domain_input` (garde de M4 depuis que l'analyse ôte les zéros
   finals). `amount_string` et le zéro négatif. Un comportement **observable** a-t-il changé pour un client (un
   `"50.001"` rend toujours `too_many_decimals` au rang et au champ attendus) ?
4. **Les autres LOW appliqués** (A2, A3, E4, E6, B3, A5, A6) : chacun tient-il, et sans régression ? Le test de l'ordre
   des refus (E4) et sa mutation M29.
5. **Le contrat de l'API** : rien n'a-t-il changé pour un client hors le refus de taille (toujours `400`) et le
   `409 RECONCILIATION_ACCOUNT_LOCKED` sous contention ? La v016-2b se développe en parallèle sur ce contrat.

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/v0162a-review-p2.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`P2-1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne,
   la preuve (sortie copiée), la correction proposée, et s'il est **né de la remédiation** ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
