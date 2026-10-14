# Story 29-4a — revue de code P2, passe ciblée (une lentille R, la remédiation de la P1)

*Prompt versionné de la passe 2 de `bmad-code-review` de la clé `29-4a-reverification-import-lot` (#651). Modèle :
**Sonnet** — la remédiation de la P1 touche du **code de production** (trois enveloppes de rejeu neuves, une aide
d'import neuve) : la passe ciblée n'est pas confiée à Haiku. **Une seule lentille**, en **contexte frais**, braquée
sur le **seul diff de la remédiation** (`CLAUDE.md`, § *La passe ciblée*).*

## Contexte

- Dépôt : `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4`. La branche a été **rebasée** sur `b4abae5d`
  (`main`, Story 29-1a mergée) avant la remédiation ; le commit de la revue P1 rebasé est `daab782f`.
- **Périmètre** : le diff **aplati** de la remédiation, `git diff daab782f..bc9cf85e -- crates CHANGELOG.md` (un seul
  commit de code, `bc9cf85e`). Le commit qui le suit ne porte que la fiche, le registre de sprint et ce prompt.
  S'y ajoute **un site du rebase** : la résolution du conflit de `crates/kesh-db/src/repositories/bank_accounts.rs`
  (`mod tests`, les tests `l_ecart_du_releve_est_arrondi_au_centime` de la 29-1a et
  `les_reverifications_verrouillent_en_partage` de la 29-4a, réunis) — à vérifier : les deux tests sont entiers,
  aucun n'a perdu d'assertion.
- Les rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/29-4a-review-p1-{B,E,A}.md`. Les décisions de
  l'orchestrateur et leur traitement : fiche `_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md`,
  Change Log, entrée « Revue de code P1 de la 29-4a » ; registre
  `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md`, C-29-4-28 à C-29-4-30 et l'annotation de
  C-29-4-14.
- `CLAUDE.md` à la racine fait foi (*Review Iteration Rule*, *Pattern batch*, *Migration breaking policy*).
- **Manuel** : la remédiation ne touche ni `docs/manual/fr/user-manual.tex` ni son PDF. Vérifie que rien de ce
  qu'elle rend vrai n'y est démenti — en particulier : le manuel dit-il quelque part qu'une modification, un
  archivage ou une création de compte bancaire peut échouer en erreur interne pendant un import ? Contrôle le **PDF**
  aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), pas seulement le `.tex`.
  `docs/api-external.md` (§ *Interblocages*, la liste des routes rejouées) n'est pas touché : sa mise à jour est
  prescrite à la 29-4b — dis si la liste est devenue fausse, et dans quel sens.

## Ce qui a changé, à relire

1. `crates/kesh-api/src/routes/bank_accounts.rs` — **trois enveloppes neuves** : `create_bank_account` →
   `create_once`, `update_bank_account` → `update_once`, `archive_bank_account` → `archive_once`, sous
   `crate::retry::retry_app_on_deadlock` (opérations `bank_accounts::create`, `::update`, `::archive`), conclusion par
   `conclure` ; doc-comments réécrits de la clôture, de la réouverture et du `PATCH`.
2. `crates/kesh-api/src/routes/bank_imports.rs` — aide neuve `create_import_or_roll_back` (rollback explicite sur le
   refus de `create_with_transactions`), appelée aux deux sites CAMT et CSV.
3. `crates/kesh-db/src/repositories/bank_accounts.rs` — doc-comments seuls (cycles, règle de position, résidus,
   `innodb_snapshot_isolation`, L1 « cinq chemins », `set_journal_account_id_for_company`, `close_in_tx`).
4. `crates/kesh-db/src/test_fixtures.rs` — doc-comment de `sonde_verrou_nowait`.
5. Tests : `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (T29a-13 à T29a-15, aides `compte_de_banque_imputable`,
   `lourde_tenant_le_compte`, assertion neuve à l'étape (3) de `cycle_a_trois_par_la_file`) ;
   `bank_account_closed_inventory.rs` (T-a47 réécrit ; volet inverse de T29a-5 étendu ; borne haute de W17 ; entrée
   `closing_checks.rs::<hors fonction>` et décompte de `G` à dix) ; `audit_route_registry.rs` (points (ii), (iii bis),
   (vi)) ; doc-comments de `bank_accounts_repository.rs` et `payment_batches_repository.rs`.
6. `CHANGELOG.md` — la ligne #651 du § `[0.13.4]`.

## ⛔ Table obligatoire — les verrous des trois enveloppes neuves

Pour **chacune** des trois routes rejouées par la remédiation (`POST /bank-accounts`, `PUT /bank-accounts/{id}`,
`DELETE /bank-accounts/{id}`), reconstruite **au code** (et aux schémas : migrations et squash `test-schema`) :

| Route | Suite ordonnée des verrous de la tentative (ligne, S/X, explicite / clé étrangère / implicite d'`UPDATE`, intervalle ou non) | Ce qui reste **hors** de l'enveloppe (pré-vol, contrôles) et pourquoi c'est sûr | Partenaires d'un cycle (à deux, à trois par la file) | Victime désignée (poids) | Rejoué de part et d'autre ? |
|---|---|---|---|---|---|

Puis réponds, preuve à l'appui :

- la tentative refait-elle **toute** la transaction ? un effet de bord **hors** transaction (audit, démotion du
  principal, `last_insert_id`) ? une valeur calculée **une fois** hors de l'enveloppe qui deviendrait fausse à la
  tentative suivante — en particulier `was_primary` de l'archivage, lu au pré-vol hors transaction ?
- une conversion qui ferait sortir un `1213` d'`AppError::Database(DbError::Sqlx(_))` avant le prédicat
  (`crate::retry::is_app_deadlock`) — `?` sur `acquire_company_sentinel_lock`, `claims_for_target`,
  `count_*`, `flip_primary_off_for_company`, l'`INSERT` brut de la création (`map_db_error`), `audit_primary_transition` ?
- l'ordre des refus documenté de chaque route (forme → … → sous verrou) est-il **inchangé** ?
- les trois tests prouvent-ils le **rejeu** (événement capturé, transaction de test `.expect`), et non un succès au
  premier essai ? Un test qui passerait **à vide** ? Le cycle de T29a-15 (réglages en exclusif contre la sentinelle)
  est-il bien celui que la doc de `create_bank_account` décrit, ou un autre — et l'écart est-il dit ?
- le cycle « atteignable en 29-4a » désormais écrit aux doc-comments (`I → M → E → I`, `I ↔ M`) : est-il vrai du code
  (l'`INSERT` d'entête de l'import et celui du lot prennent-ils le compte **avant** `companies` ?) ; la
  revérification y est-elle dite, partout, comme ne faisant **jamais** la queue ? Greper le symptôme corrigé
  (`grep -rnE "revérification d.un (autre )?écrivain|demandes? partagées? ultérieures?" crates`).
- `create_import_or_roll_back` : la transaction rendue sur succès est-elle bien celle qui commite ? Le refus rend-il
  le même code qu'avant (`import_create_error`) ? T-a47 réécrit rougit-il si le `rollback` disparaît (M29-29) ?

## Mutations déclarées (à contrôler contre le journal)

`/home/gcorbaz/devel/kesh-gate-logs/294-mutations-review-p1.log` (script versionné à côté :
`294-mutations-review-p1.py`) : M29-26 à M29-33, chacune « tuée par … ». Lis chaque bloc : la tuerie vient-elle de
l'assertion attendue, et non d'une erreur de compilation ? Une mutation **manque**-t-elle (un geste de la remédiation
qu'aucune mutation n'éprouve) ?

## Interdits — lecture seule stricte

- **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni
  `docker`, ni `mariadb`, ni `sqlx`, ni `python3`, ni `perl`, ni `sed -i`, ni script du dépôt — **en particulier**
  `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`.
- **Aucun** `git checkout`, `git switch`, `git stash`, `git reset`, `git commit`, `git rebase`, sous **aucune** forme.
  Seules commandes `git` permises : `git diff`, `git show`, `git log`, `git grep`, `git blame`.
- **Aucun fichier temporaire**, nulle part : aucun fichier écrit **hors** de ton rapport. Permis : `Read`, `grep`,
  `rg`, `pdftotext … -` vers la sortie standard.
- Toute affirmation de **présence** ou d'**absence** d'un code s'appuie sur une sortie `grep -nF` **copiée**.

## Rapport exigé

Fichier : `/home/gcorbaz/devel/kesh-gate-logs/29-4a-review-p2-ciblee.md` (jamais sous `target/`). Contenu :

1. **Axes exercés** et **axes non exercés**, listés — un « 0 finding » sans cette liste ne compte pas.
2. La **table des verrous des trois enveloppes neuves** ci-dessus, remplie.
3. Findings numérotés `R-n`, chacun : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), fichier et ligne, constat,
   preuve (sortie copiée), remède proposé ; et, pour chacun, **s'il naît de la remédiation P1 ou lui préexiste**.
4. Décompte final par sévérité, recompté depuis la liste.
