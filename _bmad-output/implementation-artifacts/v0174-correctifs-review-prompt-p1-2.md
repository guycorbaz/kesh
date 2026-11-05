# Prompt — revue de code P1 de la fiche `v0174-correctifs-2` (le serveur : #869, #817, #842, #540, #901), trois lentilles, Opus

Tu relis le développement de la fiche **`v0174-correctifs-2`** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit,
MariaDB 10.11), release **0.17.4**, **sans migration** :
- **#869** — `400 PERIOD_LOCKED` porte `details.lockedThrough` et `details.attempted` (`crates/kesh-api/src/errors.rs`),
  message inchangé ; `docs/api-external.md` ;
- **#817** — le pays d'une adresse écrite (contact, assistant) est contrôlé sur la liste ISO de `kesh-qrbill`
  (`is_iso_3166_alpha2`, devenue publique) ; un pays **inchangé** est toléré à la mise à jour ; `ContactResponse` porte
  `addressCountryKnown` ;
- **#842, #540** — `kesh_db::pool::avec_remise_a_zero_de_session` pose un `after_release`
  (`DO RELEASE_ALL_LOCKS()`, `SET SESSION foreign_key_checks = 1`), appliqué par `main.rs` et `create_pool` ;
  commentaires du symptôme « fin de session » ;
- **#901** — la 0.17.1 datée du 2026-11-04 au CHANGELOG.

Trois lentilles, **une par rapport** : **B** (chasseur aveugle : le diff seul), **E** (cas limites), **A** (auditeur
d'acceptation : AC, mutations, Dev Agent Record contre les pièces).

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`. Le diff de cette fiche, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174c show 524fda2a`, puis `git show 53740b59` (l'exception `EXCEPTIONS_V1` de `services_motif.rs` pour `main.rs`, et le manuel sans « ISO 3166-1 », posées au gate). Langue : français. Modèle : **Opus**. Passe **complète**
(première passe). Contexte frais.

Rapports (un par lentille, **le seul fichier que tu écris**) :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p1-2-B.md`, `…-2-E.md`, `…-2-A.md`.

## À lire (lentilles E et A)

- La fiche `_bmad-output/implementation-artifacts/v0174-correctifs-2.md` (AC-2.1 à 2.14, MB-869, MB-817, MB-817-b/c,
  MB-842-a, MB-842-b, MB-540, angles morts AC-2.12 et MB-842-b, Dev Notes, **Dev Agent Record**, Change Log) ; l'index
  `v0174-correctifs.md` ; le registre `v0174-choix-autonomes-v0174c.md`, **C-v0174c-8 à 11, 20, 27**.
- Pièces hors dépôt, lecture seule : `kesh-gate-logs/v0174c/rouges.log`, `rouges-fiche2.log`, `cible-fiche2.log`,
  `mut/mutations.log`, `mut/MB-*.log`, `grep-842-1.txt`, `grep-842-2.txt`, `ac-2-3-period-locked-tests.txt`,
  `etape0-release-all-locks.log`, `nextest-final.log`, `gate-final.log`, `e2e-*.log`.
- Code : `crates/kesh-api/src/{errors.rs,address_input.rs,main.rs,inbox_import.rs}`, `routes/{contacts,bank_imports,
  reconciliation}.rs` ; `crates/kesh-qrbill/src/{lib,validation}.rs` ; `crates/kesh-db/src/{pool,backup}.rs` ;
  `crates/kesh-reconciliation/src/{mutex,errors}.rs` ; tests `crates/kesh-db/tests/pool_session_reset.rs`,
  `crates/kesh-api/tests/{account_lock_session_reset_e2e,contact_address_country_e2e,period_lock_e2e,
  supplier_invoice_draft_e2e,onboarding_path_b_e2e}.rs`. Source de sqlx : `~/.cargo/registry/src/*/sqlx-core-0.8.6/src/
  pool/connection.rs` (`return_to_pool`).
- Manuel : `docs/manual/fr/user-manual.tex` (assistant, adresse d'un contact, repassage en brouillon) **et** son PDF
  aplati ; `docs/api-external.md` (`:298`, `:336`, liste de la dévalidation, § 10).

## Interdits absolus

- **Lecture seule.** Aucun fichier hormis ton rapport, **aucun fichier temporaire**, **aucune redirection**.
- **Aucun `rm`** ; **aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Rien qui compile ou exécute** : ni `cargo`, ni `npm`/`npx`/`node`, ni `python`, ni `make`, ni `docker`, ni
  `mariadb`/`mysql`, ni `sqlx`, ni `curl`, ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs`. **Aucune commande `gh`.**
- Permis : `git diff/show/log/grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `find` (sans `-exec`),
  `wc`, `sort`, `uniq`, la lecture d'images, `timeout 60 pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` puis
  `grep -cF`/`grep -oF` à motifs fixes.
- Toute affirmation de présence ou d'absence : une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Production** : Kesh 0.17.3 tient des livres réels. Aucune donnée réelle ; ne lis rien sous `~/kesh/`, `~/kesh-test/`.

## Axes obligatoires, pour chaque lentille

1. **Le crochet du pool.** Quand `after_release` s'exécute-t-il (`return_to_pool`) : après le `ROLLBACK` différé d'une
   transaction abandonnée ? Sur une connexion dont une requête a été abandonnée en vol (protocole incohérent) ? Une
   erreur ferme-t-elle bien la connexion ? `RELEASE_ALL_LOCKS` rend-il un verrou **légitimement** tenu par un autre
   emprunt (non : il est de session) ? Le crochet est-il posé sur le **pool de production** (`main.rs`, ligne citée) et
   sur `create_pool` ? Un autre pool de l'API qui le devrait (MCP, `pool_lecture_seule`, ferme déjà ; d'autres ?) —
   `git grep -n 'MySqlPoolOptions::new' -- crates`. Effet de `foreign_key_checks = 1` sur un code qui garde `= 0`
   **à travers** plusieurs emprunts (`backup.rs`, `kesh-seed`, `test_fixtures.rs`) ?
2. **Les tests du pool prouvent-ils ?** `pool_session_reset.rs` et `account_lock_session_reset_e2e.rs` : pool d'**une**
   connexion, réacquisition de la même (`CONNECTION_ID`), témoins sans crochet, noms de verrou uniques par base,
   sondage borné. Le squash (`test_schema_guard.rs`). Les angles morts AC-2.12 et MB-842-b sont-ils déclarés au Dev
   Agent Record, et une issue demandée ?
3. **#817** : le contrôle ISO vient **après** la forme ; la tolérance compare après normalisation, **seulement** à la
   mise à jour (`existing.address_country`), jamais à la création ni à l'assistant ; `addressCountryKnown` (adresse
   absente, pays absent, casse) ; une adresse **vide** avec un pays inconnu ; l'assistant (`validate_required`) ; tout
   autre écrivain d'adresse structurée (`git grep -n 'StructuredAddressInput' -- crates`) ; le MCP (`kesh_contacts`)
   qui ne porte pas le champ (choix écrit).
4. **#869** : le corps `PERIOD_LOCKED` à son **site unique** ; message inchangé (AC-2.1) ; AC-2.3 (aucun test ne compare
   le corps entier — inventaire `ac-2-3-period-locked-tests.txt`) ; `docs/api-external.md` (« quatre autres », « 7 avec
   et 4 sans », § 10) recompté depuis le tableau.
5. **Les commentaires du symptôme** : chaque site des deux greps (`grep-842-1.txt`, `grep-842-2.txt`) est-il classé au
   Dev Agent Record (sans rapport, corrigé, tracé) ? Un site resté faux ?
6. **Mutations** MB-869, MB-817, MB-817-b, MB-817-c, MB-842-a (kesh-db et kesh-api), MB-540 : tuées par le test que la
   fiche nomme (`mut/*.log`) ?
7. **Manuel, PDF, CHANGELOG** contre le code : les phrases de #817 (assistant, contact), de #869 ; la date de la 0.17.1.

## Rapport

Pour chaque finding : sévérité, fichier:ligne, preuve copiée, remède. Puis la liste des axes **exercés** et **non
exercés** (raison). Un « 0 finding » sans elle ne compte pas. Décompte recompté depuis la liste.
