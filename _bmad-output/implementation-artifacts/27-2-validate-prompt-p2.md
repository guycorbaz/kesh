# Prompt — validation P2 de la spec, Story 27-2 (la devise d'un relevé CSV est lue)

*Versionné le 2026-10-12. Passe **complète**, deux lentilles R et F, contexte frais chacune, en lecture seule.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 a été
menée par **Sonnet** ; P2 : **Opus** pour les deux lentilles. La remédiation de P1 a été écrite par Opus : méfie-toi du
biais d'auteur, relis-la comme un texte étranger.*

Dépôt `/home/gcorbaz/devel/kesh-27-2`, branche `story/27-2-devise-import-csv` (base `a2743fd4` = `origin/main`,
v0.13.1 + rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-2-devise-import-csv.md` (remédiation P1 au commit `165ab9eb` ; diff
de la remédiation : `git diff 52d4fd41 165ab9eb`). Issue #624 (`gh api repos/guycorbaz/kesh/issues/624` et
`…/624/comments`). Registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md` (C-27-2-1 à 14).
Rapports de P1 : `/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p1-R.md` et `27-2-validate-p1-F.md` ; décision par
finding : fiche, § *Change Log* → *Validation P1*. Précédent direct : `26-1-import-csv-ubs-natif.md` (même parseur,
même écran, mêmes gardes i18n).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy, P1 à P8 ;
§ Issue Tracking Rule).

⚠️ La fiche **décide qu'aucune migration ni aucun bump de `kesh_version_min_required` n'est nécessaire** (C-27-2-1), en
s'appuyant sur le `deny_unknown_fields` de `ColumnMapping` dans les binaires publiés, et lit P1 de la § Migration
breaking policy « à la lumière de son objet » (exception assumée, par analogie à P4). P1 l'a confirmée ; vérifie-la
encore toi-même, et juge si l'exception est écrite de façon qu'une revue future puisse la contester.

⚠️ **Le périmètre a grandi en P1**, sans issue neuve, sur décision de l'orchestrateur : **AC8-bis** (le formulaire de
profil garde toute clé qu'il ne montre pas — `counterparty` aujourd'hui) et **AC19** (la devise d'une transaction
CAMT.053 se lit sur l'élément qui porte son montant). Ces deux AC n'ont été relus par **personne**.

## Axes prioritaires (communs aux deux lentilles)

1. **Régressions de la remédiation P1** (axe neuf, le plus cher — sur l'Epic 23, sept passes sur huit ont trouvé une
   régression du patch précédent) : lis `git diff 52d4fd41 165ab9eb` ligne à ligne. Chaque patch dit-il vrai au code ?
   Contredit-il un autre AC, un choix du registre, l'ordre des refus (*Points de vigilance*), un test ou une mutation
   existants ? A-t-il laissé ailleurs, dans la fiche ou le registre, une formulation de l'état d'avant (propagation :
   grepe la **valeur** — `counterparty`, `CAMT`, `hors périmètre`, `M17`, `10 premières`, `n'est pas lue`, `0.13.1`) ?
   En particulier :
   - **AC13-12 / M18** : la fixture dérivée (un débit à devise vide, les huit autres débits passés en positif) donne-t-elle
     réellement `d = 9, e = 8` au code (`RowAccumulator::push`, `whole_debit_column`, `parser.rs:446-515`), et `d = e = 8`
     sous la mutation ? `total_errors: 8` est-il juste ? Le refus de signe précède-t-il bien toute lecture de devise ?
   - **AC13-6 / M9** : `ÉUR` et `ÉU` tuent-ils réellement les mutants nommés ? Un autre mutant plausible survit-il ?
   - **AC13-11 / M17** : la fixture à 11 lignes et 11 codes distincts, le code « trié en dernier » sur la première
     ligne : la mutation « 10 premières rencontrées » rougit-elle vraiment ?
   - **AC14-(3a)/(3b) et M5**, **AC5** (contrôle dans chaque bras du `match` de `create_csv`) : réalisable tel quel
     (`routes/bank_imports.rs:1758-1999`) ?
   - **AC11** : le contrôle PDF à motif `n.est` rend-il vraiment 1 sur le PDF actuel (exécute `pdftotext` vers la sortie
     standard), et l'assertion positive est-elle assez précise pour rougir si la phrase neuve manque ?
2. **Le nouveau périmètre** :
   - **AC8-bis (formulaire)** : `frontend/src/lib/features/bank-import/BankProfileForm.svelte` (l. 20-80),
     `bank-profile.types.ts`, `BankProfileForm.test.ts`. La déstructuration prescrite garde-t-elle **toutes** les clés
     non montrées, et seulement elles ? Les deux branches (colonne unique, mode séparé), la création (pas de profil
     existant), le passage d'un mode à l'autre (`amount` ↔ `debit_credit_split`), `debitSign` (hors `columnMapping`) ?
     Le type TypeScript laisse-t-il passer la déstructuration sans `any` ? Les cas Vitest (f)(g)(h) et les mutations
     M21, M22 rougiraient-ils réellement ? Un autre écran ou un autre client du `PUT` (`frontend/src`, E2E
     `createProfileViaUi`) reconstruit-il lui aussi un mapping partiel ?
   - **AC19 (CAMT.053)** : `crates/kesh-import/src/camt053/mod.rs` (l. 140-240, 340-470, 560-640). La règle « devise
     retenue seulement si `path.last()` vaut `TxDtls` ou `Ntry` à l'ouverture de `Amt` » est-elle exacte (ordre de
     `path.push`, l. 183) pour v04 **et** v08 (`v04.rs`, `v08.rs`, préfixes de namespace, `local_name`) ? Laisse-t-elle
     une transaction **sans** devise (`MissingRequiredField`) là où elle en avait une — un `<Ntry>` dont l'`Amt`
     direct n'a pas de `Ccy`, un exporteur qui ne met la devise que sous `AmtDtls` ? Les fixtures existantes
     (`crates/kesh-import/tests/fixtures/camt053/*.xml`, 13 fichiers) et les tests (`camt053_tests.rs`, tests
     `kesh-api` qui importent du CAMT, E2E) restent-ils verts au raisonnement ? La fixture prescrite est-elle
     réalisable sans faire échouer un contrôle de soldes en aval (`kesh-core`, `bank_imports.rs`) ? Le test qui doit
     « rougir avant correction » rougit-il vraiment sur le code actuel (lis l'ordre des éléments prescrit) ? Les
     mutations M19, M20 sont-elles tuées ? Le manuel utilisateur dit-il quelque chose de la devise des transactions
     CAMT qui deviendrait faux (§ *Import dans Kesh*, § *Réconciliation automatique* l. 1845) ?
   - **AC12-bis (manuel d'administration)** : `docs/manual/fr/admin-manual.tex` § *Mise à jour de Kesh* (l. 1920),
     *Passer à la 0.13.1* (l. 1972), *Rollback en cas d'échec* (l. 2223). La section prescrite dira-t-elle vrai au code
     prescrit, et au comportement de la 0.13.1 (`git show v0.13.1:…`) ? La réserve sur le retour est-elle exacte
     (liste en 500 pour toute la société, sélecteur vide sans message, `GET` par identifiant) ?
3. **Inventaire clos des sites non résolus** (fiche, § *Inventaire*) : refais les `grep` copiés et **élargis-les** (un
   détecteur part du symptôme, pas d'une liste) — `CHF` sous toutes graphies, tout lecteur ou écrivain de `currency` /
   `Currency` / `ccy` / `Ccy` sur les chemins CSV **et CAMT** (parseurs, `kesh-core`, routes `bank_imports` et
   `bank_profiles`, `kesh-db`, export global, `.keshbackup`, `post_restore.rs`, `kesh-seed`, `test_endpoints.rs`,
   frontend `bank-import/`). Et, neuf : tout site qui **reconstruit** un `columnMapping` ou un profil complet (perte de
   clé). Un site qui fixe ou suppose une devise, ou qui perd une clé, sans figurer à l'inventaire est au moins **HIGH**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Import CSV multi-encodage*
   (l. 1721-1807), § *Import dans Kesh* (l. 1690-1719), § *Rejet partiel* (l. 1824-1828), § *Réconciliation
   automatique* (l. 1845) ; `admin-manual.tex` (axe 2). Ce que le manuel dit **aujourd'hui** est-il vrai au code ? Ce
   que AC11 et AC12-bis prescrivent d'écrire sera-t-il vrai au code prescrit ? Contrôle les **PDF**, aplatis :
   `pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '` (apostrophe typographique : grepe `n.est`). Une
   promesse que le code ne tiendrait pas est au moins MEDIUM.
5. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — **22 AC** (AC1, AC1-bis, AC2-AC8, AC8-bis,
   AC9-AC12, AC12-bis, AC13-AC19), tâches T0-T8, mutations **M1-M22**, **12** tests AC13, **8** items AC14, **8** cas
   Vitest (a)-(h), **3** items AC19, choix **C-27-2-1 à 14**, les 29 littéraux `ColumnMapping {`
   (`grep -rn "ColumnMapping {" crates`), les numéros de ligne cités (`sed -n`), `sitesTotal: 2032`
   (`i18n-keys.test.ts:575`), les familles dynamiques (`:628`, `:680`), « 13 fichiers » de fixtures CAMT, « neuf
   débits, deux crédits » et « 12 occurrences de `;CHF;` » de `ubs_synthetique.csv` (données synthétiques, versionnées).
   Le tableau *Validation P1* du Change Log : chaque décision correspond-elle à un patch réellement présent dans la
   fiche ou le registre (`grep -nF`) ?

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #624 couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui
  rougirait** — et chaque mutation M1-M22 par un test **qui la tue réellement** (lis le test prescrit, pas son nom) ;
  chaque fait, chemin et numéro de ligne cité par la fiche ou le registre **revérifié** (`sed -n`, `grep -nF`), y compris
  ceux qu'a ajoutés la remédiation (`parser.rs:439-442`, `error.rs:402`, `errors.rs:673-674`/`:2141`,
  `i18n-glossaire.md:161`, `ubs_fichier_reel.rs:97-105`, `camt053/mod.rs:183`/`:372`/`:427`/`:603`/`:618`,
  `admin-manual.tex:1920`/`:1972`/`:2223`, `messages.ftl:1057`, Svelte `input.js:288-289`) ; les textes prescrits
  (quatre locales, replis Svelte et Rust, manuels, CHANGELOG, § *Pour la PR*) confrontés à ce que le code prescrit fera
  réellement ; les gardes existantes qui changeront de sens (`error.rs` `all_is_complete`, `i18n-keys.test.ts` et ses
  familles, `i18n-un-repli-par-cle.test.ts` / `MOTIFS_DYNAMIQUES`, `lint-i18n-ownership`, `textes_coherents.rs` — dont
  G19/G20 —, `test_schema_guard.rs`, `exports/csv_tables.rs`, `csv-line-error-label.test.ts`, `camt053_tests.rs` et son
  en-tête) — la fiche les nomme-t-elle toutes ? Axes 1, 3, 4, 5 en priorité.
- **F — Faisabilité et chemins de la décision (adversaire de la conception)** : axes 1 et 2 en priorité, puis :
  - **Le retour arrière** (C-27-2-1, AC12, AC12-bis) : un profil enregistré par la 0.13.2 **avec** `currency`, relu par
    une 0.13.1 et une 0.13.0 (`git show v0.13.1:…` sur `profile.rs`, `entities/bank_profile.rs`,
    `routes/bank_profiles.rs`, `routes/bank_imports.rs`, `BankImportUpload.svelte`). Y a-t-il **un** chemin de la
    0.13.1 qui lise ce JSON sans `ColumnMapping` typé et importerait en CHF en silence ? Et, neuf : un profil dont
    **AC8-bis** a gardé un `counterparty` change-t-il quoi que ce soit au retour ? Un relevé CAMT importé sous la 0.13.2
    (devise corrigée par AC19) relu par une 0.13.1 ?
  - **L'ordre des refus** (fiche, *Points de vigilance*) et ses interactions avec AC2 amendé (erreur de devise portant
    `debit`) : un cas où le refus de signe, le mélange, la devise non supportée ou le rejet partiel s'évaluent dans le
    mauvais ordre, ou sur des comptes faussés ?
  - **CAMT (AC19) contre le reste du produit** : la réconciliation (`reconciliation.rs:631`, `:2728`, `:777-780`), les
    doublons d'import (empreinte, `transaction_id`), l'aperçu CAMT (`bank_imports.rs:1007`, `:1031`) — la correction
    change-t-elle un comportement visible ailleurs, en bien ou en mal ? Un relevé déjà importé avant la 0.13.2 porte
    des transactions « EUR » au montant en francs : la fiche dit-elle ce qu'il en advient (rien, ou réimport) ?
  - **`kesh-import` publiable** : zéro dépendance interne préservée par AC19 et `DEFAULT_CSV_CURRENCY` ?
  - **L'écran** (AC8, AC8-bis) : comportement réel de Svelte 5.55.1 (`node_modules/svelte/src/internal/client/dom/
    elements/bindings/input.js`) pour un champ vidé, et pour la déstructuration d'un `$state` / d'une prop (`existing`
    est une prop : la déstructuration lit-elle la valeur au moment de l'envoi ?).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Signale à part tout finding **né de la
remédiation P1** (préfixe `[régression P1]`). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo` (y compris `cargo publish --dry-run` et `cargo test`), `npm`, `npx`,
`node`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé
réel de Guy) ne se lit ni ne se cite — et le test `crates/kesh-import/tests/ubs_fichier_reel.rs` ne se lance pas.
Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la
sortie standard.
