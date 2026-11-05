# Prompt — validation P2 des fiches v0174-correctifs (release 0.17.4 : #819, #888, #889, #869, #872, #817, #842 et #540, #901), Opus, lentille complète

Tu valides les **fiches** de la story `v0174-correctifs` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11),
release corrective **0.17.4**, **sans migration**, dont le but déclaré par le Project Lead est **moins de bugs**. La story est
découpée en une **fiche-index** et **trois fiches sœurs** :

- **1** — #819 (P2) : l'import CAMT.053 ignore les références d'entité et le CDATA ; la fiche prescrit une accumulation du
  texte par élément, un **mode hérité** du parseur qui reproduit la 0.17.3, et un **alias de clé de doublon** pour qu'une ligne
  déjà importée, reprise par un autre fichier, reste reconnue. Kesh 0.17.3 tient des relevés réels importés.
- **2** — #869 (`details` du `400 PERIOD_LOCKED`), #817 (pays ISO 3166-1 contrôlé à l'écriture), #842 et #540 (un crochet
  `after_release` du pool qui rend les verrous de session et rétablit `FOREIGN_KEY_CHECKS`), #901 (date de la 0.17.1).
- **3** — #888 (toutes les factures validées pour un lot), #889 (date du jour en heure locale : inventaire clos de 15 + 12 sites,
  un helper, une garde Vitest), #872 (projet archivé orphelin dans le sélecteur), #869 côté écran.

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`, partie de `7bd4d5c0`. Langue du rapport :
**français**. Modèle : **Opus**. **Deuxième passe** de validation, **complète** (une seule lentille, qui tient R **et** F).
La passe P1 (Sonnet) a rendu R 0/1/2/8 et F 0/0/4/7 ; la remédiation (commit `95aa52ce`) a notamment : changé la
configuration du lecteur XML de la lecture corrigée (C-v0174c-21, `trim_text(false)`), révisé #817 (pays **inchangé**
toléré à la mise à jour d'un contact, champ `addressCountryKnown`, avis à l'écran — fiche 2 § 2.2, fiche 3 § 3.5), refait la
garde de #889 (allowlist par occurrence), complété `docs/api-external.md` et le symptôme « fin de session ». **Ce qu'il reste à
relire en priorité est ce que la remédiation a écrit** — la sévérité se déplace vers le dernier patch —, sans négliger le
reste : `git show 95aa52ce` en donne le périmètre. **Contexte frais** : ne t'appuie sur aucune
passe antérieure ni sur ce que les fiches disent d'elles-mêmes ; refais les relevés. Les numéros de ligne des fiches sont ceux
de `7bd4d5c0` (= `HEAD` du worktree, hors fichiers `_bmad-output`).

Ta lentille unique couvre les deux faces :

- **R — la règle et sa preuve** : chaque prémisse est-elle vraie au code ? Chaque conception ferme-t-elle le défaut, sans en
  ouvrir un autre ? Chaque AC est-il décidable, chaque test rouge avant le correctif là où la fiche le dit, chaque mutation
  tuée par le test qu'elle nomme ? L'inventaire clos l'est-il ?
- **F — les textes et ce qu'ils affirment** : manuel (`.tex` **et** PDF), CHANGELOG, `docs/api-external.md`, i18n, décomptes,
  numéros de ligne, cohérence entre l'index, les trois fiches et le registre.

## À lire

1. Les fiches, **en entier** : `_bmad-output/implementation-artifacts/v0174-correctifs.md` (index),
   `v0174-correctifs-1.md`, `v0174-correctifs-2.md`, `v0174-correctifs-3.md`.
2. Le registre `_bmad-output/implementation-artifacts/v0174-choix-autonomes-v0174c.md` (C-v0174c-1 à 21) ; le diff de la
   remédiation, `git show 95aa52ce` ; les rapports de la P1, **pour savoir ce qui a été corrigé, non pour t'y fier** :
   `/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p1-{R,F}.md`.
3. Les issues, **en lecture seule** : `gh api repos/guycorbaz/kesh/issues/<n>` et `…/<n>/comments` pour 819, 888, 889, 869,
   872, 817, 842, 540, 901.
4. Le code —
   **1** : `crates/kesh-import/src/camt053/mod.rs` (en entier), `src/lib.rs`, `tests/camt053_golden.rs`, `tests/camt053_tests.rs`,
   `tests/fixtures/README.md`, `tests/fixtures/camt053/v04_characterization.xml`,
   `tests/fixtures/camt053_golden/v04_characterization.json` ; `crates/kesh-core/src/bank_imports.rs` (`DuplicateKey`,
   `detect_duplicate_lines`) ; `crates/kesh-api/src/routes/bank_imports.rs` (`preview`, `create`, `create_camt_under_lock`,
   `preview_csv`, `create_csv_under_lock`, `compute_duplicate_lines_warnings`, `existing_dedup_keys`,
   `detect_duplicate_lines_for_imported`) ; `crates/kesh-api/tests/bank_imports_e2e.rs` ;
   `crates/kesh-db/src/repositories/bank_imports.rs` ; `crates/kesh-reconciliation/src/rules.rs` ; la source de quick-xml
   0.42.0 (`~/.cargo/registry/src/index.crates.io-*/quick-xml-0.42.0/src/{events/mod.rs,escape.rs,reader/mod.rs}`).
   **2** : `crates/kesh-api/src/errors.rs` (le bras `DbError::PeriodLocked`, `build_response`,
   `later_fiscal_year_closed_response`), `crates/kesh-db/src/errors.rs` (`PeriodLocked`), `routes/reconciliation.rs`
   (`period_locked_failed_proposal`), `crates/kesh-api/tests/period_lock_e2e.rs` ; `crates/kesh-api/src/address_input.rs`,
   `crates/kesh-qrbill/src/{validation.rs,lib.rs}`, `routes/contacts.rs` (`validate_common`, création, mise à jour),
   `routes/onboarding.rs`, `routes/companies.rs`, `frontend/src/routes/(app)/contacts/+page.svelte`,
   `frontend/src/lib/features/supplier-invoices/SupplierCreatePanel.svelte` ; `crates/kesh-reconciliation/src/mutex.rs`,
   `crates/kesh-api/src/main.rs` (construction du pool), `crates/kesh-db/src/pool.rs`, `crates/kesh-db/src/backup.rs`
   (`restore_tables_in_tx` et son montage de test d'un pool d'une connexion), la source de sqlx-core 0.8.6
   (`~/.cargo/registry/src/index.crates.io-*/sqlx-core-0.8.6/src/pool/connection.rs`), `crates/kesh-api/tests/reconciliation_e2e.rs`
   (verrou nommé) ; `CHANGELOG.md`, `scripts/prepare-release.sh` (lecture seule), `git for-each-ref refs/tags/v0.17.*`.
   **3** : `frontend/src/routes/(app)/payment-batches/+page.svelte` et son test, `crates/kesh-api/src/services/factures_fournisseur.rs`,
   `crates/kesh-db/src/repositories/supplier_invoices.rs` (`list`), `routes/payment_batches.rs` ; chacun des sites des deux
   tableaux de la fiche 3 (§ 5, § 6), `lib/features/open-items/open-items.ts` et son test,
   `lib/shared/utils/pin-time-zone.test-helper.ts`, `lib/features/reports/reports.api.ts` (`formatSwissDate`, `zurichParts`),
   `lib/features/reminders/{reminder-sent-at.ts,reminder-sent-at.test.ts,ReminderHistory.test.ts}`,
   `crates/kesh-api/src/routes/dunning_reminders.rs`, `crates/kesh-db/src/repositories/companies.rs` (borne du verrou) ;
   `lib/features/supplier-invoices/SupplierInvoiceForm.svelte`, `routes/(app)/supplier-invoices/[id]/edit/{+page.svelte,edit-page.test.ts}`,
   `crates/kesh-db/src/repositories/projects.rs`, `crates/kesh-api/src/routes/projects.rs` ;
   `lib/features/supplier-invoices/{supplier-invoice-unvalidate.ts,supplier-invoice-unvalidate.test.ts}`,
   `routes/(app)/supplier-invoices/[id]/+page.svelte` ; `lib/shared/i18n-un-repli-par-cle.test.ts` (patron de la garde).
   **#817 révisé** : `crates/kesh-api/src/routes/contacts.rs` (en entier : `validate_common`, `update_contact`,
   `ContactResponse`), `crates/kesh-db/src/entities/contact.rs`, les outils MCP qui rendent un contact
   (`git grep -n 'ContactResponse\|kesh_contacts' crates/kesh-api/src`), `frontend/src/lib/features/contacts/contacts.types.ts`,
   `frontend/src/routes/(app)/contacts/+page.svelte`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` — `:213`, `:641`, `:947`, `:1696-1712`, `:1786-1796`, `:1988-2012`,
   `:2360-2410`, `:2693-2710`, `:2886-2895` — et `docs/manual/fr/admin-manual.tex` (sections *Passer à la 0.17.x*, `:2529-2550`).
   **Le PDF aussi**, lu aplati et vers la sortie standard seulement :
   `LC_ALL=C.UTF-8 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`).
   `docs/manual/{de,en,it}/` : confirme qu'il n'y a aucun `.tex`. Et `docs/api-external.md` (toutes les lignes
   `PERIOD_LOCKED`, § 10 *Gestion des erreurs*).
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Règle de splitting préventif*, *Issue Tracking Rule*,
   *Propagation post-patch*, *Recompter ses propres comptes rendus*, *Migration breaking policy*, *Pattern batch*,
   *Inventorier les sites NON RÉSOLUS*) ; `docs/agents-orchestration.md` ;
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v0174-commun.md`.

## Axes — la remédiation de la P1 (en premier)

- **C-v0174c-21** : la lecture corrigée sans `trim_text` produit-elle **exactement** la sortie de la 0.17.3 sur un fichier
  ordinaire (texte avant un enfant, blancs seuls entre balises, `Text("")` vide, BOM, CRLF, tabulation, espace insécable — le
  `trim()` de Rust rogne l'Unicode, quick-xml les seuls blancs XML : la boucle d'aujourd'hui rognait-elle déjà en Rust,
  `camt053/mod.rs:242` ?) ; la boucle de détection de `<Document>` (`:74-104`) supporte-t-elle les `Text` de blancs ?
  La note « mutation du mode hérité équivalente » est-elle vraie ? Les valeurs attendues de la référence corrigée tiennent-elles
  sans rognage du lecteur ?
- **#817 révisé** : la tolérance du pays stocké est-elle sûre (normalisation des deux côtés, pays stocké `NULL` → `CH`,
  adresse effacée puis recréée, adresse rendue vide) ; la création, l'assistant et toute autre route d'écriture sont-ils bien
  sans tolérance ? `addressCountryKnown` : calcul sans adresse, présence dans **toutes** les réponses de contact (liste,
  détail, création, mise à jour, MCP `kesh_contacts` ?) — un DTO miroir écrit à la main l'oublierait-il ? L'avis de l'écran
  et sa clé i18n. Le manuel et le CHANGELOG disent-ils la nouvelle règle ?
- **La garde de #889** : l'allowlist par occurrence (huit entrées, fragments exacts et uniques) est-elle constructible
  telle qu'écrite ; le masque des commentaires couvre-t-il `//`, `/* */`, `<!-- -->` des `.svelte` ; la sonde dans un fichier
  autorisé tue-t-elle MC-889-c ; « 29 lignes à `7bd4d5c0` » est-il juste ?
- **Les ajouts** (AC-3.2 bis et la déduplication, AC-1.6 et le repli sans alias, `CLES_RELEVEES` 399 → 402, les sites
  `docs/api-external.md:298`, `:336`, la tâche 4 de la fiche 2, les issues #906 à #910 et les demandées) : justes au code,
  sans résidu ?

## Axes — la règle et sa preuve

- **#819, la prémisse et la clé** : la cause est-elle bien le bras `_ => {}` (relis la boucle entière) ? Les champs touchés
  sont-ils tous ceux que la fiche dit (montants, dates, IBAN compris) ? La clé d'un **relevé** est-elle vraiment indépendante de
  la lecture — l'empreinte d'aperçu (`bank_import_fingerprint`), `statement_id`, la sélection par IBAN, un autre contrôle de
  chevauchement de relevés existent-ils et dépendent-ils du texte lu ? La clé d'une **ligne** dépend-elle seulement de
  `reference` / `end_to_end_id` / `transaction_id` (relis `normalize_reference_fallback` et `existing_dedup_keys`) ?
- **#819, la conception** : l'accumulation « vidée avant tout autre événement » donne-t-elle **exactement** la sortie de la
  0.17.3 pour un fichier sans entité ni CDATA — y compris texte avant un enfant, `Empty`, commentaires, CRLF, BOM, espaces ?
  Le `path` au moment du vidage est-il le bon (avant `push` d'un `Start`, avant `pop` d'un `End`) ? Les API citées de quick-xml
  0.42 existent-elles sous ces noms et signatures ? Le **mode hérité** peut-il reproduire la 0.17.3 à l'octet, et la preuve
  (AC-1.4 : mode hérité contre **toutes** les références 0.39) le démontre-t-elle vraiment, divergence admise UTF-8 comprise ?
  L'**alias** : alignement par index sûr (une lecture qui diffère par le **nombre** de transactions est-elle possible ?) ;
  risque de faux doublon, de doublon manqué ; sites CAMT tous couverts (aperçu **et** création) ; CSV et saisie manuelle
  intouchés ; `DuplicateLine.key` ; `apply_duplicate_lines_filter`. AC-1.5 est-il réalisable tel qu'écrit (dépôt
  `create_with_transactions` et sa transaction, montage « 0.17.3 », soldes, IBAN du compte) ? (c) rougit-il bien sous
  MA-819-4 ? AC-1.7 : la valeur « Co » est-elle juste ? Une fixture ajoutée à `tests/fixtures/camt053/` casserait-elle quelque
  chose (compte de `camt053_tests.rs:3`, `MIN_CRATE_FIXTURES`, références 0.39) — la fiche l'interdit-elle partout ?
  `cargo publish --dry-run -p kesh-import` : `#[doc(hidden)]` suffit-il ?
- **#869** : le site `errors.rs` est-il **le seul** qui rend un `400 PERIOD_LOCKED` HTTP (cherche large : `"PERIOD_LOCKED"`,
  `PeriodLocked` dans `crates/kesh-api/src`, les `ModificationBlocker` qui portent ce code — `crates/kesh-db/src/errors.rs:219-240`
  — et leur rendu) ? Un client (frontend ou test) casse-t-il si `details` apparaît ? AC-2.1 et AC-2.2 sont-ils montables avec
  les helpers des fichiers cités ?
- **#817** : les écrivains d'une adresse sont-ils **tous** recensés (cherche `StructuredAddress {`, `validate_optional`,
  `validate_required`, `address_structured`, les services, le MCP, l'import de contacts s'il existe, la complétion d'un
  document qui crée un fournisseur) ? Le refus strict d'une valeur stockée inchangée est-il défendable, et le manuel / le
  CHANGELOG le disent-ils ? `is_iso_3166_alpha2` rendue publique : effets sur `kesh-qrbill` (tests, `debug_assert!`) ?
  Les écrans affichent-ils déjà `err.message` (lis le chemin d'erreur des deux) ?
- **#842 / #540** : la fiche a-t-elle raison sur `return_to_pool` (ordre `after_release` → `ping` → remise ; abandon pendant une
  requête en vol ; `ROLLBACK` différé de la `Transaction` abandonnée — lis `sqlx-mysql` si besoin) ? `RELEASE_ALL_LOCKS()` libère-
  t-il aussi un verrou qu'un **autre** emprunt légitime tiendrait encore (l'import du dossier `inbox_import.rs` garde sa
  connexion : la rend-il avant la fin ?) ? Un `Err` du crochet ferme-t-il bien la connexion ? `create_pool` et `main.rs` : un
  autre pool de production existe-t-il (`git grep -n 'MySqlPoolOptions::new'`) ? AC-2.9 à 2.11 : montables (pool d'une
  connexion, `connect_with`, sondage, témoin sans crochet) et rouges avant ? `test_schema_guard.rs` reste-t-il vert avec un
  fichier de test neuf ? L'intégration de #540 est-elle vraiment mécanique (« sauf mécanique », prompt) et ferme-t-elle #540
  en entier ?
- **#888** : l'ordre serveur est-il total ; la boucle termine-t-elle toujours ; les tests existants du fichier restent-ils verts ?
- **#889, l'inventaire clos** : rejoue toi-même les `git grep` du § 4 de la fiche 3 et les décomptes (20, 15, 5 ; 12 corrigés
  + 3 conformes ; 12 affichages, 9 + 3) ; cherche **plus large** (`new Date(` suivi d'un formatage, `Intl.DateTimeFormat` sans
  `timeZone`, `Date.now()` arithmétique, `getDate()`, dates passées à `max=`/`min=`, valeurs par défaut de formulaires, noms
  de fichiers ailleurs). Chaque « conforme » l'est-il ? `paidAt` est-il bien une date à minuit **partout** où il s'écrit (cherche
  tous les écrivains de `paid_at`) ? Le couplage `ManualReminderDialog` / `reminder-sent-at` : la nouvelle règle peut-elle
  produire un `sent_at` futur ? La garde : décidable, sans faux positif sur ses propres chaînes, sonde positive ?
- **#872** : la prémisse corrigée (projet d'en-tête) est-elle juste — aucune ligne ne porte de projet ? Le rendu des orphelins
  casse-t-il la page de création ou un autre appelant de `SupplierInvoiceForm` ? AC-3.10 rouge avant ?
- **Le découpage** (C-v0174c-1) : conforme à la règle (≤ 5 modules par fiche, rollout mécanique) ? L'ordre 2 → 3 suffit-il ;
  un couplage 1 ↔ 2 ou 1 ↔ 3 est-il ignoré ?
- **Le périmètre et les règles du dépôt** : défaut voisin intégré à tort (#540, `users/+page.svelte`, `JournalEntryForm`) ou
  renvoyé à tort ? Point écarté **sans** issue demandée (« on trace toujours ») ? Aucune migration
  (`ls crates/kesh-db/migrations/*.sql | wc -l` = 89) ; aucune donnée réelle ; aucun mot-clé de fermeture prescrit dans un
  commit ; gate complet et E2E complet au dernier commit de code de chaque fiche ; exception `kesh-db` pour la fiche 2 ;
  captures en thème clair ; i18n quatre locales.

## Axes — les textes et ce qu'ils affirment

- **Le manuel** — axe obligatoire : chaque phrase prescrite au § *Manuel* de l'index est-elle **vraie au code après
  correctif** ? L'inventaire est-il clos : cherche dans le `.tex` **et le PDF aplati** toute phrase qui devient fausse ou muette
  (les doublons, les règles d'affectation, le pays « CH par défaut », « seules les factures validées sont proposées », le
  repassage en brouillon sous verrou, la modification d'un brouillon, l'import d'un fichier refusé comme XML invalide, la date
  du jour). Le manuel d'administration doit-il dire autre chose (mise à jour, comportement du pool) ?
- **CHANGELOG** : chaque puce est-elle vraie, rangée sous *Corrigé*, avec ses liens, sans fermeture ? Dit-elle trop (« jusqu'à
  une demi-heure » : vérifie `idle_timeout` / `max_lifetime` du pool de `main.rs`) ou trop peu (le refus d'un fichier à entité
  inconnue, changement de comportement) ?
- **`docs/api-external.md`** : la phrase prescrite pour `:493` et ses décomptes (« 11 lignes, 7 avec `details` et 4 sans »)
  sont-ils justes, recomptés depuis le tableau ? D'autres phrases deviennent-elles fausses ?
- **i18n** : les deux clés neuves, leurs textes fr-CH, les gardes de décompte nommées ; un texte visible change-t-il sans clé ?
- **Les décomptes et numéros de ligne** (huit appels de `with_account_lock`, 249 codes ISO, 89 migrations, 20/15/5, 12+3,
  9+3, six fichiers d'affichages, 13 fichiers de dates du jour, `:493`, les lignes citées de chaque fiche) : recompte-les
  depuis la source (`git grep -c`, `sed -n`, `wc -l`) ; la ventilation est-elle cohérente avec son total ?
- **Cohérence** entre l'index, les trois fiches et le registre (numéros de choix, issues, mutations, ordre, clés de statut,
  `sprint-status.yaml`) ; une affirmation de l'une contredite par une autre ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois** (outil Write), à
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p2.md`. Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, en disant pour chacun s'il naît de la remédiation (`95aa52ce`) ou du texte d'origine, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), la
   fiche (1, 2, 3, index, registre), le lieu (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie
   `grep -nF` ou extrait copié) qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans
   sortie copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec, pour chaque axe exercé, ce que tu as lu ou cherché. Un « 0
   finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité **et par fiche**, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au rapport ; il
  ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni `latexmk`,
  ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `mariadb`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/install-hooks.sh`, ni aucun
  autre script du dépôt (les lire avec `cat` est permis).
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/Projets actuels/kesh/`, ni le NAS ; aucune donnée réelle dans les
  rapports.
- Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture),
  `git log`, `git show`, `git diff`, `git for-each-ref` (en lecture), `pdftotext … -` vers la sortie standard, `gh api` en `GET`.
