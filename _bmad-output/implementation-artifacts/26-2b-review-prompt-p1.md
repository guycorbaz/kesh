# Prompt — revue de code P1, Story 26-2b (la description d'un compte bancaire dans tous les écrans)

*Versionné le 2026-10-11. Trois lentilles (**Sonnet**), contexte frais chacune, en lecture seule. Rotation (D6, rétro
25) : le développeur était Opus → **P1 Sonnet** (passe complète), P2 Opus, Haiku réservé à la passe ciblée de fin de
boucle. La 26-2b est le **rollout** de la 26-2a : la fiche demande une revue **fichier par fichier** (§ *Règle de
splitting préventif*) — chaque lentille parcourt les fichiers du diff un à un et le déclare.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2`. **Diff à relire : `git diff 22004175..60a101a8`**
(`22004175` = clôture de la revue de la 26-2a, dernier commit avant la 26-2b ; `60a101a8` = dernier commit de code de
la 26-2b). Commits de la 26-2b : `0a76c74e` (serveur : `GET /companies/current`, refus de désignation), `50ce1a37`
(écrans D3-D9, types, garde d'inventaire), `f8cfd76c` (manuel, brochure, PDF, CHANGELOG, `docs/api-external.md`),
`60a101a8` (E2E (b)). Fiche : `_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md` —
**partie b** (AC9 à AC12, AC13 (b), AC14 partie b : M8, M8-bis, M9, M10, M16, M16-bis, M17 ; tâches T7 à T11 ; Dev
Notes § *Inventaire clos*). La partie a (colonne, dépôt, validation, exports, écran *Comptes bancaires*) est
**done et revue** : ne la relire que là où la 26-2b l'appelle. Registre des choix :
`_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md` (C-26-2-1 à 34). Issue #622 (la PR la fermera).
Règles : `CLAUDE.md` (§ Review Iteration Rule, § Inventorier les sites NON RÉSOLUS, § Propagation post-patch, § Le
prompt d'une passe doit NOMMER le manuel, § Recompter ses propres comptes rendus).

**Contexte de rebase** : la branche n'est **pas** rebasée sur la 26-1 (consigne de l'orchestrateur, qui rebasera après
sa fusion) ; la section `## [0.13.1]` du CHANGELOG sera fusionnée par union avec celles de la 26-1 et de la 26-3. Ne
pas signaler l'absence des entrées de la 26-1 comme un défaut.

## Axes prioritaires

1. **Inventaire clos des sites (AC9)** : D3 à D9 passent-ils tous par `bankAccountName` / `bankAccountOptionLabel`,
   avec la forme prescrite (`'parens'` pour D7, inchangée sans description, C-26-2-26) ? ⛔ **Ne pas partir de la
   liste de la fiche** : partir du symptôme — tout endroit de `frontend/src` (et des messages serveur) qui montre ou
   propose un compte bancaire, par `bankName`, `iban`, `shortIban`, `bank_name`, `bankAccount`, `listBankAccounts`,
   `bankAccounts` — et vérifier que chacun est résolu ou écrit comme angle mort assumé. Les types en ligne
   (`BankAccountListItem`, `CompanyResponse`) ont-ils disparu comme le veut C-26-2-18 ?
2. **Serveur (AC10)** : `BankAccountJson` (`routes/companies.rs`) ; `first_active_bank_account_linked_to` — verrou
   `LOCK IN SHARE MODE` et texte `FROM bank_accounts` intacts, `bank_account_display_name` ; renommage
   `bank_name` → `bank_label` de `DbError::ClaimAccountLinkedToBankAccount` propagé partout (repo, `kesh-api/src/errors.rs`
   — **y compris les deux replis `format!` en dur** —, tests) ; `details.bankName` conservé ; textes FTL inchangés.
3. **Garde d'inventaire (AC11)** `frontend/src/lib/features/bank-accounts/bank-name-inventory.test.ts` : collecte
   (exclusions `.test.`/`.spec.`), expression du jeton, table fermée contre une mesure **refaite** (la commande de la
   fiche), minimum de fichiers lus, somme, cas synthétiques par forme. La garde peut-elle passer à vide ? Prouve-t-elle
   ce qu'elle dit ?
4. **Tests de site** : D4 `payment-batches-page.test.ts` et D9 `settings-page.test.ts` (neufs), D5, D6, D8 (Vitest),
   D3, D7, D8 (E2E (b)), D10 (Rust : `companies_e2e.rs`, `company_invoice_settings_postable_e2e.rs`). Chacun
   rougirait-il si son site régressait ? Test muet, assertion tautologique, simulation qui contourne le code ?
   Mutations : `/home/gcorbaz/devel/kesh-gate-logs/26-2b-mutations.log` — chaque mutant de la partie b tué par le test
   que la fiche nomme ?
5. **E2E (b)** (`frontend/tests/e2e/bank-accounts-crud.spec.ts`) : isolement (comptes non principaux, IBAN distincts,
   archivés en fin de test), sélecteurs par `data-testid`, aucune dépendance au libellé traduit, effet sur les specs
   voisines qui choisissent un compte **par position**.
6. **Le manuel — axe obligatoire** : `docs/manual/fr/user-manual.tex` (section neuve *Comptes bancaires*, renvoi de
   l'onboarding, correction du compte de la QR-facture l. ~911/921/933/1033) **et le PDF** contrôlé aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Le PDF est-il régénéré depuis le `.tex`
   courant ? Chaque phrase dit-elle vrai **contre le code** (où la description s'affiche, ce qu'elle ne touche pas —
   QR-facture, fichier de paiement —, `PUT` qui l'efface, onboarding) ? Brochure (`marketing-brochure.tex` + PDF),
   `docs/api-external.md` (note ⁷), `CHANGELOG.md` (`## [0.13.1]`, `### Ajouté` / `### Modifié`) : même contrôle.
   *C'est en vérifiant le manuel contre le code qu'on trouve les sites oubliés : rapporte aussi ce que cette
   vérification révèle du code.*
7. **Propagation** : chaque symptôme corrigé (« compte bancaire de réception », « ISO 11649 », `bank_name` du refus)
   est-il absent **partout** — code, i18n des quatre locales, manuels, site `website/`, README ? Grep par jeton.
8. **Comptes rendus** : Dev Agent Record de la 26-2b et Change Log — chaque décompte (tests ajoutés, occurrences de la
   garde, gates, mutations) recompté depuis la source, avec son périmètre.

## Lentilles

- **B — Blind Hunter** : le diff **seul** (ne lis ni la fiche ni le registre) ; défauts visibles dans le code,
  fichier par fichier.
- **E — Edge Case Hunter** : le diff **et** le dépôt (lecture) ; chaque site, chaque appelant, chaque test
  indirectement touché, chaque forme que la garde ne voit pas (axes 1, 2, 3, 5, 7).
- **A — Acceptance Auditor** : le diff **et** la fiche (partie b) ; chaque AC et chaque tâche T7-T11 confrontés au
  code, chaque mutation à son test, chaque affirmation du Dev Agent Record à la source, le manuel et son PDF contre
  le code (axes 4, 6, 8).

## Ce que tu rends

Findings numérotés (`B1-…`, `E1-…`, `A1-…`) avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**
(commande et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF`
copiée (attention aux balisages Svelte multi-lignes et au SQL multi-lignes). ⛔ **La liste des axes exercés ET non
exercés, et la liste des fichiers du diff parcourus** — un « 0 finding » sans elles ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-2b-review-p1-<B|E|A>.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`/
`push`/`tag`, `gh`, `cargo`, `npm`, `npx`, `node`, `docker`, `mariadb`, `sqlx`. Lecture seule : `git show`,
`git diff`, `git log`, `grep`, `sed -n`, `cat`, `pdftotext`.
