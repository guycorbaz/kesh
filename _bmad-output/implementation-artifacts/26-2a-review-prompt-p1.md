# Prompt — revue de code P1, Story 26-2a (socle de la description d'un compte bancaire)

*Versionné le 2026-10-11. Trois lentilles (**Sonnet**), contexte frais chacune, en lecture seule. Rotation (D6, rétro
25) : le développeur était Opus → **P1 Sonnet** (passe complète), P2 Opus, Haiku réservé à la passe ciblée de fin de
boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2`. **Diff à relire : `git diff 9b724027..1ea3c98b --
crates frontend docs/migrations-idempotence-audit.md`** (`9b724027` = `origin/main` au départ ; `1ea3c98b` = dernier
commit de code de la 26-2a). Fiche : `_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md` —
**partie a seulement** (AC1 à AC8, AC13 (a), AC14 partie a : M1-M7, M11-M15, M18). La partie b (AC9-12 : autres
écrans, `GET /companies/current`, refus de désignation, garde d'inventaire, manuel, CHANGELOG) n'est **pas** dans ce
diff : son absence n'est pas un finding. Registre des choix : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`
(C-26-2-1 à 30). Issue #622. Règles : `CLAUDE.md` (§ Review Iteration Rule, § Migration breaking policy P1-P8,
§ Inventorier les sites NON RÉSOLUS, § Propagation post-patch, § Recompter ses propres comptes rendus, § Pattern batch).

**Contexte de rebase** : la 26-1 (migration `20261011000001`, bump `min_required` 0.13.1) fusionnera **avant** ; la
26-2a a commencé avant elle (« cas commencée avant » de la fiche, Dev Notes § Coordination) et porte donc ici 78 → 79
migrations, soustracteur 44 → 45. Ces nombres seront refaits au rebase par l'orchestrateur : **ne pas** signaler
l'écart avec les « 80 / 46 » de la fiche comme un défaut, mais **vérifier** que les valeurs présentes sont justes pour
l'arbre tel qu'il est.

## Axes prioritaires

1. **Les lectures et écritures de `bank_accounts`** : la macro `columns!` couvre-t-elle **tous** les `SELECT` qui
   hydratent `BankAccount` (dépôt, route, et ailleurs dans le workspace) ? Un chemin d'écriture touche-t-il la
   description sans le vouloir (upsert d'onboarding, `PATCH`, archivage, démotion, `create`) ? `find_by_id_in_tx`
   est-il scopé ? Le `PUT` efface-t-il bien une description absente — et le frontend l'envoie-t-il toujours ?
2. **Validation serveur** (`routes/bank_accounts.rs`) : ensemble refusé (`is_forbidden_in_description`) contre la
   liste de la fiche (AC3, C-26-2-28), ordre trim / contrôle, ordre des refus banque → description → IBAN → QR-IBAN,
   bornes en caractères, message, audit `created` / `updated`.
3. **Migration et garde-fous** P1-P8 : non-breaking, ligne d'audit et compteurs (recompter depuis le tableau),
   `migrations_upgrade_path.rs` (toutes les mentions), `migrations.sha384`, squash, P7 (n'écrit aucune donnée). **Et
   les tests qui montent un schéma ANTÉRIEUR** (`apply_migrations_up_to`, `migrations_before`) puis appellent du code
   du dépôt : le dev en a trouvé un (`tests/support/lettering_documents.rs`) au gate ; y en a-t-il d'autres,
   y compris dans `kesh-api/tests` ?
4. **Exports** : colonne CSV après `bank_name`, `csv_sanitize`, garde de schéma ; sauvegarde `.keshbackup`
   (aller-retour, sauvegarde antérieure).
5. **Frontend** : `format.ts` (types, `bankAccountName`, `bankAccountOptionLabel`, constante) ; écran *Comptes
   bancaires* (champ, `maxlength`, payloads, cellule) ; `BankAccountJournalLinkForm` ; types de l'API ; i18n quatre
   locales et repli identique au fr-CH ; gardes `i18n-keys.test.ts` et `i18n-libelle-en-dur.test.ts` recomptées.
6. **Tests** : chacun prouve-t-il ce qu'il dit (test muet, assertion tautologique, test qui passerait sans le code) ?
   Mutations M1-M7, M11-M15, M18 : rapport `/home/gcorbaz/devel/kesh-gate-logs/26-2a-mutations.log` — chaque mutant
   a-t-il été tué par le test que la fiche nomme ? E2E (a) : isolement (compte non principal, archivé en `afterEach`).
7. **Le manuel** (`docs/manual/fr/user-manual.tex`, PDF aplati `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) :
   la 26-2a ne l'écrit pas (26-2b) ; vérifier seulement qu'**aucun passage existant** n'est rendu faux par ce que la
   26-2a livre (écran *Comptes bancaires*, export CSV, sauvegarde).
8. **Comptes rendus** : Dev Agent Record et Change Log de la fiche — chaque décompte (tests ajoutés, fichiers, gates)
   recompté depuis la source, avec son périmètre.

## Lentilles

- **B — Blind Hunter** : le diff **seul** (ne lis ni la fiche ni le registre) ; défauts visibles dans le code.
- **E — Edge Case Hunter** : le diff **et** le dépôt (lecture) ; chaque branche, chaque borne, chaque appelant et
  chaque test indirectement touché (axes 1, 3, 6).
- **A — Acceptance Auditor** : le diff **et** la fiche (partie a) ; chaque AC et chaque tâche T1-T6 confrontés au
  code, chaque mutation à son test, chaque affirmation du Dev Agent Record à la source (axes 6, 8).

## Ce que tu rends

Findings numérotés (`B1-…`, `E1-…`, `A1-…`) avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**
(commande et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF`
copiée (attention aux SQL multi-lignes). ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle
ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-2a-review-p1-<B|E|A>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`/`push`, `gh`, `cargo`, `npm`,
`npx`, `docker`, `mariadb`, `sqlx`. Lecture seule : `git show`, `git diff`, `git log`, `grep`, `sed -n`, `cat`,
`pdftotext`.
