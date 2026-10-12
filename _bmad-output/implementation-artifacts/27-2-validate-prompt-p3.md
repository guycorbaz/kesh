# Prompt — validation P3 de la spec, Story 27-2 (la devise d'un relevé CSV est lue)

*Versionné le 2026-10-12. Passe **complète**, deux lentilles R et F, contexte frais chacune, en lecture seule.
Rotation (D6, rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 :
**Sonnet** ; P2 : **Opus** ; P3 : **Sonnet** pour les deux lentilles. Les remédiations de P1 et de P2 ont été écrites par
Opus : relis-les comme un texte étranger.*

Dépôt `/home/gcorbaz/devel/kesh-27-2`, branche `story/27-2-devise-import-csv` (base `a2743fd4` = `origin/main`,
v0.13.1 + rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-2-devise-import-csv.md` (remédiation P2 au commit `dba7a1ae` ; diff
de la remédiation : `git diff ef7c22ca dba7a1ae`). Issue #624 (`gh api repos/guycorbaz/kesh/issues/624` et
`…/624/comments`). Registre : `_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md` (C-27-2-1 à 19).
Rapports de P2 : `/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p2-R.md` et `27-2-validate-p2-F.md` ; décision par
finding et **constat D5** : fiche, § *Change Log* → *Validation P2*. Branches voisines (lecture seule, `git show`) :
`story/27-1-supprimer-import` (suppression d'un import non rapproché, dont dépend le remède de C-27-2-16) et
`story/27-3-profil-apres-apercu`. Précédent direct : `26-1-import-csv-ubs-natif.md`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif et son
amendement D5 ; § Migration breaking policy, P1 à P8 ; § Issue Tracking Rule).

⚠️ **Le critère de bascule D5 est écrit au Change Log** : si cette passe trouve un MEDIUM **né de la remédiation P2**
dans AC8-bis ou AC19, le signal devient recyclage. Dis donc, pour **chaque** finding MEDIUM ou plus, s'il naît du
diff `ef7c22ca..dba7a1ae` (préfixe `[régression P2]`), de la remédiation P1 (`[régression P1]`) ou de la fiche d'origine.

## Axes prioritaires (communs aux deux lentilles)

1. **Régressions de la remédiation P2** (axe premier, le plus cher — sur l'Epic 23, sept passes sur huit ont trouvé une
   régression du patch précédent) : lis `git diff ef7c22ca dba7a1ae` ligne à ligne. Chaque patch dit-il vrai au code ?
   Contredit-il un autre AC, un choix du registre, l'ordre des refus (*Points de vigilance*), un test ou une mutation ?
   A-t-il laissé ailleurs une formulation de l'état d'avant (propagation : grepe la **valeur** — `29`, `sinon par SQL`,
   `(f)`/`(g)`/`(h)`, `M22`, `C-27-2-14`, `:1825`, `575-578`, `578-583`, `001.04`, `v0.1`) ? En particulier :
   - **AC1-(5) / M23** et le doc-comment prescrit sur `ColumnMapping` : le test rougit-il réellement sous M23 (attribut
     retiré : `serde` ignore-t-il alors la clé, `crates/kesh-import/src/csv/profile.rs:34-60`) ? Le doc-comment dit-il
     vrai sur les binaires antérieurs (`git show v0.13.1:…/profile.rs`) ?
   - **AC8-bis — `OWNED_COLUMN_MAPPING_KEYS` et `keepUnownedKeys`** : la forme prescrite est-elle correcte en
     TypeScript `strict` (`frontend/tsconfig.json`, TypeScript 5.9.3 ; `satisfies` sur un tableau `as const` ; `delete`
     sur une propriété de `Partial<ColumnMapping>` ; type du spread `{ ...keepUnownedKeys(…), ...champs }` contre
     `CreateOrUpdateBankProfileRequest.columnMapping`) ? Le raisonnement est **déduit, non compilé** : juge-le au texte
     des règles, et dis ce que tu n'as pas pu établir. Les cas Vitest (v6)-(v9) et les mutations M21, M22, M24 :
     chacune est-elle tuée par le cas que nomme AC17 (lis le cas prescrit, pas son nom) ? Le formulaire écrit-il
     `reference`, `details`, `currency` dans **les deux** branches (`BankProfileForm.svelte:56-69`), de sorte qu'une
     omission de ces clés dans la constante soit sans effet — ou un mutant survit-il ?
   - **AC12 / AC12-bis — « Transactions déjà importées »** (C-27-2-16) : le remède dit-il vrai au code de la **27-1**
     (`git show story/27-1-supprimer-import:…` : fiche `27-1-supprimer-import-non-rapproche.md`, AC1 et AC5 ; code de
     la branche si présent) — définition de « rapprochée », code `409`, refus des clés d'API ? À celui de l'annulation
     de rapprochement (`crates/kesh-db/src/repositories/reconciliation_cancel.rs`, doc-module) ? L'affirmation « les
     transactions CAMT restées en EUR ne sont pas rapprochables » tient-elle sur **tous** les chemins de rapprochement
     (`routes/reconciliation.rs` : propositions, règles, rapprochement manuel, éclatement manuel — `:631`, `:777-780`,
     `:2728`) ? Le réimport après suppression : `BANK_IMPORT_DUPLICATE_FILE` levé, empreinte de ligne
     (`routes/bank_imports.rs:1532-1539`) ?
   - **AC12 — réparation par l'API de la 0.13.1** (F7) : `PUT` et `DELETE` de la 0.13.1 ne désérialisent-ils vraiment
     pas le profil enregistré (`git show v0.13.1:crates/kesh-api/src/routes/bank_profiles.rs`, l. 259-300 ;
     `…/kesh-db/src/repositories/bank_profiles.rs`, l. 200-320) ? La réponse du `PUT` (`try_from_db` du profil
     **neuf**) réussit-elle ?
   - **AC13 — helper de dérivation** (R5) : « sauter `header_row_count` lignes non vides » est-il la règle exacte du
     lecteur (`parser.rs:185-260`, doc-module l. 14-17 ; `csv::ReaderBuilder` avec `has_headers` et `flexible`) — une
     ligne d'espaces, une ligne `;;;` sont-elles « vides » pour le lecteur ? AC13-8 (R4), AC13-6 (`CH1`, `CH`) : les
     mutants nommés meurent-ils ?
   - **AC10 — garde G9** (`crates/kesh-api/tests/textes_coherents.rs:679`) : l'entrée prescrite pour
     `bank-import-csv-errors-mixed-currencies` (paramètre `{ $currencies }` → `{currencies}`) fonctionne-t-elle avec le
     mécanisme réel de la garde (comparaison d'un littéral entre guillemets après normalisation) et un repli écrit en
     `format!` ? Les textes `fr-CH` prescrits portent-ils des apostrophes typographiques que le repli devra reproduire ?
     Les gestes de `i18n-keys.test.ts` (l. 628-633, 680-682) sont-ils les bons ?
   - **AC10 — « v0.1 » restants** (R7, C-27-2-19) : la liste des sites gardés et réécrits est-elle juste et complète
     (`grep -rnF "v0.1" crates --include=*.rs | grep -i "devise\|currency\|chf"`) ?
   - **C-27-2-18 (hypothèse R9 de P2)** : la conclusion « non établi » est-elle juste ? Les preuves citées (repli
     `camt053/mod.rs:603`, XSD 001.08 l. 705, comptage des fixtures, contrôle de solde confirmable
     `routes/bank_imports.rs:433`) sont-elles exactes ? Ne tranche pas le schéma 001.04 de mémoire : dis ce que le dépôt
     établit.
2. **Le périmètre élargi en P1** (AC8-bis, AC19), relu une troisième fois : AC19 (`camt053/mod.rs:140-240`, `340-470`,
   `560-640`) — l'angle mort F8 désormais écrit (`:572-575`) est-il le seul ? Une fixture CAMT existante ou un test
   `kesh-api` / E2E qui importe du CAMT changerait-il de résultat ?
3. **Inventaire clos des sites non résolus** (fiche, § *Inventaire*) : refais les `grep` copiés et élargis-les — `CHF`
   sous toutes graphies, tout lecteur ou écrivain de `currency` / `Ccy` sur les chemins CSV et CAMT, tout site qui
   reconstruit un `columnMapping` ou un profil complet. Un site qui fixe ou suppose une devise, ou qui perd une clé, sans
   figurer à l'inventaire est au moins **HIGH**.
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (§ *Import dans Kesh* l. 1690-1719,
   § *Import CSV multi-encodage* l. 1721-1807, § *Rejet partiel* l. 1824-1828, § *Réconciliation automatique* l. 1845,
   *Annuler le rapprochement* l. 1244 et l. 1945) et `admin-manual.tex` (§ *Mise à jour de Kesh* l. 1920, *Passer à la
   0.13.1* l. 1972 et son paragraphe « Transactions déjà importées » l. 1993, *Rollback* l. 2223). Ce que AC11 et
   AC12-bis prescrivent d'écrire sera-t-il vrai au code prescrit **et** au code de la 27-1 ? Contrôle les **PDF**,
   aplatis : `pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '`. Les trois motifs de contrôle
   prescrits (`le relevé est supposé en CHF`, `supprimer l.import`, `Passer à la 0.13.2`) rendent-ils 0 aujourd'hui (pour
   pouvoir rougir) ? Une promesse que le code ne tiendrait pas est au moins MEDIUM.
5. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — **22 AC**, tâches T0-T8, **5** tests AC1,
   **12** tests AC13, **8** items AC14, **9** cas Vitest (v1)-(v9), **3** items AC19, mutations **M1-M24** (T1 : 14,
   T2 : 5, T4 : 5), choix **C-27-2-1 à 19**, **27** littéraux `ColumnMapping {`, « 18 constats distincts » de P2, les
   numéros de ligne cités (`sed -n`) — dont ceux qu'a ajoutés ou corrigés la remédiation P2 (`parser.rs:597-602`,
   `:629`, `:14-17`, `camt053/mod.rs:572-575`, `routes/bank_imports.rs:1831`, `:433`, `:1532-1539`,
   `textes_coherents.rs:679`, `fixtures/README.md:25-39`, `admin-manual.tex:1993`, XSD 001.08 `:705`). Le tableau
   *Validation P2* du Change Log : chaque décision correspond-elle à un patch réellement présent (`grep -nF`) ?

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #624 couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui
  rougirait** — et chaque mutation M1-M24 par un test **qui la tue réellement** ; chaque fait, chemin et numéro de ligne
  cité par la fiche ou le registre **revérifié** ; les textes prescrits (quatre locales, replis Svelte et Rust, manuels,
  CHANGELOG, § *Pour la PR*) confrontés à ce que le code prescrit fera réellement ; les gardes existantes qui changeront
  de sens (`all_is_complete`, `i18n-keys.test.ts` et ses familles, `i18n-un-repli-par-cle.test.ts` /
  `MOTIFS_DYNAMIQUES`, `lint-i18n-ownership`, `textes_coherents.rs` G9/G19/G20, `csv-line-error-label.test.ts`,
  `camt053_tests.rs` et son en-tête, `tests/fixtures/README.md`) — la fiche les nomme-t-elle toutes ? Axes 1, 3, 4, 5 en
  priorité.
- **F — Faisabilité et chemins de la décision (adversaire de la conception)** : axes 1 et 2 en priorité, puis :
  - **Le retour arrière** (C-27-2-1, AC12, AC12-bis) : un profil enregistré par la 0.13.2 avec `currency`, relu par une
    0.13.1 et une 0.13.0 — un chemin qui importerait en CHF en silence ? La réparation par l'API (F7) marche-t-elle
    réellement, `GET` en 500 compris (le client doit connaître tous les champs du profil pour un `PUT` complet : où les
    trouve-t-il) ?
  - **Le remède des transactions déjà importées** (C-27-2-16) contre la 27-1 telle qu'elle est sur sa branche : un cas
    où il ne marche pas (import partiellement rapproché, transaction rejetée, import CAMT multi-relevés, droits) ?
  - **L'ordre des refus** (*Points de vigilance*), y compris le refus `409 BANK_IMPORT_PREVIEW_STALE` de la 27-3 que la
    fiche demande d'y insérer à la fusion.
  - **`kesh-import` publiable** : zéro dépendance interne préservée.
  - **L'écran** (AC8, AC8-bis) : comportement réel de Svelte 5.55.1 pour la prop `existing` lue à l'envoi, et pour
    `keepUnownedKeys` appelé sur un `$state` ou une prop proxifiée (`{ ...existing }` d'un proxy Svelte : copie
    superficielle correcte ?).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Préfixe `[régression P2]` ou `[régression P1]`
selon l'origine (cf. critère D5 ci-dessus). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus avec son origine.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo` (y compris `cargo publish --dry-run` et `cargo test`), `npm`, `npx`,
`node`, `tsc`, `svelte-check`, **`python3`** et **`python`** (même en `-I`, même en lecture seule : la passe P2 l'a fait,
c'est un écart), `perl`, `ruby`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, toute requête SQL. Rien de
`~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite — et le test `crates/kesh-import/tests/ubs_fichier_reel.rs`
ne se lance pas. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture sur la sortie standard, `cat -A`, `wc`,
`git log`/`show`/`diff`/`blame`/`grep`, `gh api` en lecture, `pdftotext` vers la sortie standard.
