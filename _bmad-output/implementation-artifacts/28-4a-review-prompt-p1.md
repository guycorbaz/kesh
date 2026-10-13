# Prompt — revue de code P1, Story 28-4a (clore un compte bancaire, partie serveur)

*Versionné le 2026-10-13. Trois lentilles (**Opus** — story à risque : migration, gardes sur trente chemins
d'écriture, concurrence), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4`.
**Le diff à revoir** : `git diff eab77cf1 01af1c0e -- . ':(exclude)_bmad-output'` (`eab77cf1` = `origin/main`
sur lequel la branche est rebasée — 28-2, 28-3 et 28-5a mergées ; `01af1c0e` = **dernier commit de code** ;
les commits suivants ne portent que la fiche, le registre des choix, le registre de sprint et ce prompt). ⚠️ Pathspec
`':(exclude)…'` et **non** `':!…'`. Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` —
**partie a seulement** : AC1 à AC10 et AC6-bis ; tâches T0-T9 ; Dev Notes (§ *Inventaire* W1-W30 et R1-R8, § *Les
trois invariants*, § *Concurrence* et sa table *Position par chemin*, § *Angles morts*, § *Tests* T-a1 à T-a48,
§ *Mutations*) ; et le **Dev Agent Record** (relectures de T0, table *Position par chemin* recopiée avec les lignes
retenues, campagne de mutations, gates, décomptes). Choix C-28-4-1 à C-28-4-32 au registre
`epic-28-choix-autonomes.md` (C-28-4-26 à 32 nés du développement). Règles : `CLAUDE.md` du dépôt — § *Migration
breaking policy* (P1 à P8), § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt
d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § *Pattern
batch*, § « Inventorier les sites NON RÉSOLUS ». Issues : #642 (la PR de la 28-4a la **référence** sans la fermer —
`refs #642` ; c'est la 28-4b qui la fermera) et #651 (L1, `refs` seulement).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/` ; rien sous `~/kesh-test/`.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. ⛔ **LE DOUBLE CRITÈRE DE L'AC7, SUR LE CODE ÉCRIT — axe obligatoire, chemin par chemin.** (a) *Aucun verrou de
   ligne ajouté* — explicite (`FOR UPDATE`, `LOCK IN SHARE MODE`, `FOR SHARE`) ou implicite par clé étrangère (aucun
   `INSERT` ni `UPDATE` de colonne à clé étrangère neuf) — *à aucun chemin qui existait sur `eab77cf1`, et aucun
   ordre de verrous changé* ; (b) *aucun déplacement de la naissance de la vue* : la première lecture non
   verrouillante de chaque chemin reste celle du sol, ou la garde posée immédiatement avant elle sans verrou entre
   les deux ; sur un chemin sans lecture ordinaire hors l'audit (W24, W30), la garde suit le dernier verrou pris avant
   la première écriture. Pour **chaque** ligne de la table *Position par chemin* (fiche et sa copie au Dev Agent
   Record), relis le code du diff : W8, W27, W20, W17-W19, W22-W23, W24, W9, W25, W28, W29, W30, W13, W14, W15,
   `POST /bank-accounts`, `upsert_primary_in_tx`, `accept_one_*`, `reject`/`manual`/`split`/`preview`/`create`, W21,
   W26, `first_active_bank_account_linked_to`, et les deux chemins neufs (clôture, réouverture : un seul verrou, en
   première instruction, **la route ne lit rien dans la transaction avant** — `assert_onboarding_complete` lit-il sur
   le pool ?). Méthode : `git diff eab77cf1 01af1c0e -- crates/kesh-db/src crates/kesh-api/src | grep -nE
   'FOR UPDATE|LOCK IN SHARE|FOR SHARE|INSERT INTO|UPDATE '` — chaque occurrence neuve est-elle une instruction
   **existante élargie** (W8, W27, W25 : même ligne, même clé) ou le chemin neuf ? Les quatre gardes posées entre des
   verrous (W9 après (3), W25 après (2-bis), W28 après l'exercice d'achat, W29 après (1-bis)) sont-elles à leur place,
   et le commentaire d'une ligne de chaque site renvoie-t-il au doc-comment du sol qui l'impose ?
2. **Migration et garde-fous P1-P8.** Non-breaking réel (`ADD COLUMN` nullable ; `ADD CONSTRAINT CHECK` satisfaite
   par toute ligne existante) ; aucun bump de `kesh_version_min_required`, ni des versions Cargo (P2-bis : rien à
   bouger) ; ligne d'audit d'idempotence à sa place, compteurs **recomptés** (`ls crates/kesh-db/migrations/*.sql |
   wc -l` ; `grep -c '^| \`20' docs/migrations-idempotence-audit.md` ; partitions 10 / 71 / 0) ;
   `migrations.sha384` : une ligne **ajoutée**, aucune modifiée ; squash régénéré (`test-schema/0001_schema_squash.sql`
   — la contrainte et la colonne y sont-elles, et rien d'autre ?) ; P6 : `migrations_upgrade_path.rs` (81, 47,
   frontière 34, généalogie, jetons `\b(80|46|81|47)\b`) et `lettering_documents_backfill.rs` (colonne posée puis
   retirée autour de `MIGRATOR.run()`, C-28-4-26 non : c'est le montage de la fenêtre — le geste est-il juste, et
   d'autres tests à fenêtre `migrations_before` exercent-ils un geste vivant qui lit `closed_on` ?) ; P7 : la
   migration n'écrit aucune donnée ; P8 : aucune migration antérieure touchée (`git diff --stat eab77cf1
   01af1c0e -- crates/kesh-db/migrations`).
3. **Les invariants I1-I3 et le registre de l'AC8.** Les décomptes de la clôture lisent-ils ce qu'ils doivent
   (`status = 'pending'` — une rejetée compte ; `status = 'generated'`) ? I3 tient-il structurellement (contrainte),
   et la réouverture ne touche-t-elle pas `is_primary` ? Le registre `bank_account_closed_inventory.rs` : extraction
   (verbes et modificateurs, deux lectures du littéral, blocs de test et commentaires exclus), diff dans les **deux**
   sens, anti-muet à 31 sites (W1-W30 + `write_closed_on`, C-28-4-28), second inventaire des trois appelants ; les
   angles morts du doc-comment (trois sites `format!` à nom variable, numéros de `backup.rs` à jour) sont-ils vrais ?
   Cherche un écrivain d'une des sept tables que l'extraction **ne voit pas** (nom non littéral, `QueryBuilder` à
   table poussée, autre crate) et qui ne soit pas énuméré.
4. **Erreurs et rendus.** `DbError::BankAccountClosed` et `AppError::BankAccountClosed` rendus par **une** fonction
   (`bank_account_closed_response`) ; codes et statuts de l'AC3 / AC4 / AC5 ; ordre des refus (clôture : forme, 404,
   version, déjà clos, principal, pending, lots ; archivage : 409 au pré-vol **avant** sentinelle et décompte) ;
   `import_create_error` aux **deux** appels de la route d'import ; `failed[]` par proposition pour R1 et R2 (§ *Pattern
   batch*) ; R8 avant la classification de `unreject`.
5. **Gardes existantes suivies, décomptes recomptés.** `audit_route_registry.rs` (118 / 112 / 121 / 94 / 121 et
   leurs messages), `audit_labels.rs`, G9 (`textes_coherents.rs` — six clés entrées, `has-transactions` suivie), les
   gardes du catalogue (`kesh-i18n/src/loader.rs`, exemption de `bank-accounts-errors-closed`, C-28-4-32), les quatre
   catalogues (clés présentes, repli Rust **égal mot pour mot** au fr-CH). **Recompte depuis la source** tout décompte
   du Dev Agent Record (tests neufs : `git show <ref>:<fichier> | grep -c '#\[sqlx::test\|#\[test\]'` aux **deux**
   bornes ; fichiers au `git diff --stat` ; mutations) et vérifie le **périmètre** déclaré de chaque nombre.
6. ⛔ **LA DOCUMENTATION — `docs/api-external.md`, le CHANGELOG, ET LE MANUEL (PDF compris).** La note « *Depuis la
   v0.13.3* » des deux sections de règlement et la ligne `### Corrigé` disent-elles vrai au code (`404 NOT_FOUND`
   pour un archivé, `409 BANK_ACCOUNT_CLOSED`, la borne de #651) ? **Le manuel n'est volontairement pas touché par la
   28-4a** (il va à la 28-4b) : cherche dans `docs/manual/fr/user-manual.tex`, `admin-manual.tex` **et leurs PDF**
   une phrase que la 28-4a rend **fausse** dès sa fusion — l'archivage d'un compte (« tant qu'un autre compte
   non archivé existe »), le message du `412` d'archivage, un règlement accepté vers un compte archivé, un compte
   qui reparaîtrait dans les sélecteurs —, ainsi que dans le README et `website/`. Contrôle le PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-4a-review-p1-<lettre>-user-manual-flat.txt` (idem `admin-manual`).

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les chemins d'erreur de
  `close_in_tx` / `reopen_in_tx` et de leurs handlers (version, `begin`, commit, audit qui échoue) ; une clôture
  concurrente d'une modification du même compte (`PUT` avec `isPrimary`, démotion faite puis abandon) ; un compte à la
  fois archivé et clos (base restaurée) à chaque garde ; `ensure_open_in_tx` sur un compte absent (`NotFound` dans
  une annulation : quel statut HTTP ?) ; la date `closedOn` (UTC, format invalide, absente) ; `includeClosed` combiné
  à `includeArchived` (les quatre combinaisons) ; l'export CSV et la sauvegarde ; la tuile et `/companies/current`
  sans principal ; les messages en de-CH (« aufgelöst ») et l'exemption fr.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue la campagne de mutations
  (journal `kesh-gate-logs/28-4a-mutations.log`, bilan `28-4a-mutations-bilan.txt`, script
  `/tmp/claude-1000/…/scratchpad/28-4a/mutations-28-4a.py` cité au Dev Agent Record) et cherche une **mutation
  plausible qui resterait verte** (ex. : `company_id` retiré d'une garde ; `closed_on` lu mais non contrôlé dans
  `confirm_batch` ; la clôture qui rend `200` sans écrire ; le refus R1 qui ne liste qu'une proposition ; W25 qui
  garde le compte d'un règlement en espèces). Les équivalences déclarées (M14-bis, M20-bis, M51) et M56 assumée
  sont-elles vraies ? T-a31 et T-a48 : la sonde `attendre_une_requete_en_cours` peut-elle se satisfaire d'une autre
  requête que celle visée ? un test qui passerait à vide si la tâche finit avant la sonde ? Les montages construisent-
  ils l'état qu'ils croient (SQL direct qui casse I1/I2, comptes non principaux, règlements **par virement** pour
  T-a48) ? `etat()` de `bank_accounts_closed_e2e.rs` voit-il **toutes** les écritures qu'un geste refusé pourrait
  faire (lettrage, factures, lots, transactions) ?
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie a (AC1 à AC10,
  AC6-bis) contre le code : satisfait, partiel, absent. Les tests nommés par la fiche (T-a1 à T-a48, hors retirés)
  existent-ils et prouvent-ils ce qu'ils nomment (table de correspondance au Dev Agent Record) ? La dérogation au
  découpage (§ *Dérogation*) : les modifications de `routes/reconciliation.rs` se limitent-elles à R1-R4, R8, W30 et
  aux deux auxiliaires (la 28-6 y travaille) ? Aucun fichier de `frontend/` touché ? Les relectures de T0 (énoncés
  sur #633, numéros de `backup.rs`, W30 contre le code de la 28-5a, table *Position par chemin* contre `eab77cf1`)
  sont-elles faites et justes ? Les chiffres de gates et d'E2E déclarés correspondent-ils aux journaux
  (`kesh-gate-logs/28-4a-*.log`) ? Les choix C-28-4-26 à 32 : justifiés, ou défaut ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4a-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les six axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et les textes aplatis des
PDF au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/
`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo` (aucune sous-commande), `rustc`, `npm`, `npx`, `node`,
`deno`, `bun`, `python`/`python3` (pas même le script de mutations), `perl`, `ruby`, `php`, `sed -i`, `awk` en
écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget`, `gh` en écriture, `pkill`/`kill`,
aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
