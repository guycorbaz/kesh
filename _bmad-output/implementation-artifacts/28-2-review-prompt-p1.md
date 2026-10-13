# Prompt — revue de code P1, Story 28-2 (une erreur d'API ne s'affiche plus « [object Object] »)

*Versionné le 2026-10-13. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-2`, branche `story/28-2`. **Le diff à revoir** :
`git diff 01ab2473 4ff3a11e -- . ':(exclude)_bmad-output'` (`01ab2473` = validation close de la fiche ;
`4ff3a11e` = dernier commit de code ; les commits suivants ne portent que la fiche, le registre et ce prompt).
⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Fiche :
`_bmad-output/implementation-artifacts/28-2-erreurs-object-object.md` — AC1 à AC11, Tâches T0-T10, Dev Notes
(inventaire des sites NON RÉSOLUS, classes A à E, angles morts n° 1 à 5, mutations M1-M27), et le **Dev Agent
Record** (choix C-28-2-1 à C-28-2-34 au registre `epic-28-choix-autonomes.md` § Story 28-2, mutations, décomptes,
gates, écarts). Règles : `CLAUDE.md` du dépôt — § « Inventorier les sites NON RÉSOLUS », § « Recompter ses propres
comptes rendus », § « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Une passe qui
ne déclare pas ses axes exercés ne compte pas », § « Règle de splitting préventif » (dérogation C-28-2-9). Issues :
#520 (la PR la fermera) ; #644 (classe D, référencée : angle mort assumé, non corrigé ici).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt, hormis les captures citées
ci-dessous et ton rapport.

**Contexte** : story **frontend**, plus trois clés × quatre catalogues FTL (`crates/kesh-i18n/locales/*/messages.ftl`),
sans une ligne de Rust. Modules neufs `frontend/src/lib/shared/utils/api-error.ts` (`isApiError`, réexporté par
`api-client.ts`) et `error-message.ts` (`errorMessageOf`). `api-client.ts` : `errorMessageOf` retiré,
`serverStatusMessage`, messages `NETWORK_ERROR` / `TIMEOUT` / « Erreur serveur » par `i18nMsg` (cycle d'import
`api-client` ↔ `i18n.svelte`, C-28-2-4). Classe A (4 sites) et B (16 sites) : `settings/+page.svelte`,
`BankProfileList.svelte` (dont `handleDelete`, C-28-2-20), `bank-import/profiles/[id]/+page.svelte`,
`bank-accounts/+page.svelte`, `DeleteBankImportAction.svelte`, `settings/api-keys/+page.svelte`,
`CancelReconciliationDialog.svelte`, `BankAccountJournalLinkForm.svelte`, `ManualMatchModal.svelte`,
`TransactionSplitModal.svelte`, `reports/+page.svelte` (`formatError`), et les quatre importeurs d'`errorMessageOf`.
Garde neuve `frontend/src/lib/shared/erreurs-en-texte.test.ts` (G1-G7, exemption `console`, portées, planchers) ;
`blocEquilibre` dans `i18n-literal-reader.js` (`corpsDeFonction` devient son appel en accolades). Tests : neufs
(`error-message.test.ts`, `BankProfileList.test.ts`, `profile-page.test.ts`, `api-keys-page.test.ts`,
`reports-page.test.ts`), complétés (`api-client.test.ts`, `settings-page.test.ts`, `bank-accounts-page.test.ts`,
`DeleteBankImportAction.test.ts`, `CancelReconciliationDialog.test.ts`, `BankAccountJournalLinkForm.test.ts`,
`ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`, `i18n-literal-reader.test.ts`, `i18n-keys.test.ts`
— `sitesTotal` 2075 → 2079). E2E `frontend/tests/e2e/error-messages.spec.ts`. `CHANGELOG.md`
(`## [0.13.3] — Non publié`). Recette visuelle (B2) : captures hors dépôt
`/home/gcorbaz/devel/kesh-gate-logs/28-2-captures/books-lock-{avant,apres}-{light,dark}.png` — à regarder.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **L'utilitaire.** `errorMessageOf` suit-il le tableau de l'AC1, dans l'ordre (`ApiError` à message chaîne non
   vide après `trim()` → tel quel ; `ApiError` à message vide, blanc ou non textuel → générique ; tout le reste →
   générique **et** `console.error('[errorMessageOf]', err)`) ? Jamais `'[object Object]'`, `''`, `'undefined'`,
   `'null'` ? Le repli `'Erreur inattendue.'` est-il celui des autres sites d'`error-unexpected` ?
   `error-message.ts` ne dépend-il ni du client HTTP ni de ses `vi.mock` ?
2. **Le client.** Les trois textes : repli = texte français d'avant, **au caractère près** ; `{ $status }` en forme
   Fluent ; un `message` du corps toujours repris tel quel ; les `code` inchangés (`ForgotPasswordForm`, `login`
   aiguillent sur `NETWORK_ERROR`) ; le cycle d'import réellement bénin (aucun usage au niveau supérieur, ni dans
   `i18n.svelte.ts` ni dans `api-client.ts`) ; `loadI18nMessages` en échec ne récurse pas.
3. **Les vingt sites, un par un, contre les tableaux de l'AC3 et de l'AC4** — la parade de la dérogation au
   découpage (C-28-2-9). Branches spécifiques préservées (`BANK_ACCOUNT_NOT_CONFIGURED` ×2,
   `REPORT_PERIOD_OUT_OF_FISCAL_YEAR`, `BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, 404 de `DeleteBankImportAction`,
   `OPTIMISTIC_LOCK_CONFLICT` d'`api-keys`, `reconciliationCancelErrorMessage`) ; imports devenus inutiles retirés ;
   `handleDelete` notifie sans remplacer la liste. **Inventaire recompté au `HEAD`** par les commandes du § Inventaire
   (classe A = 0, B = 0, C = 3 inchangées). Un vingt et unième site, hors inventaire, de la même forme ?
4. **La garde.** Le recueil des liaisons (formes `catch`, typées, rappels `.catch` fléchés, nus, `function`,
   `async`, corps-expression) ; la portée (accolade après la parenthèse **fermante** de la liaison ; corps-expression
   jusqu'à la parenthèse qui ferme `.catch(`) ; les règles G1-G7 (G3 à bornes de mot, sur tout le fichier) ;
   l'exemption `console` **par position** dans la liste d'arguments, jamais par fichier ; une portée `null` en
   **échec** sur les trois chemins ; les planchers (319 / 160 / 4 / 5 mesurés, 310 / 150 / 4 / 5 posés) ; le préambule
   « ce qu'elle voit / ne voit pas » complet. `blocEquilibre` : `corpsDeFonction` inchangé en comportement ?
5. **Les tests prouvent-ils ce qu'ils nomment** : chaque cas « générique » rejette une valeur qui n'est **pas** une
   `ApiError` ; les tests de soumission des modales soumettent **réellement** (le vrai `AccountAutocomplete`, l'appel
   API constaté) ; `_messages` remis à vide entre deux cas ; aucune assertion uniquement négative.
6. **i18n** : trois clés × 4 catalogues, sous le commentaire de la story, textes du § *Textes proposés* ;
   `i18n-repli-divergent-actif` (fr-CH = repli au caractère près), `i18n-un-repli-par-cle`,
   `i18n-entrees-a-variables`, `parity_between_locales` ; `sitesTotal` 2079 recompté ; `lint-i18n-ownership`.
7. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `marketing-brochure.tex` ET leurs PDF.** Le
   seul passage qui cite un de ces messages (`user-manual.tex:1871-1873`, encadré de l'import CSV, « Impossible de
   contacter le serveur… ») dit-il encore **vrai** ? Le manuel ne change pas (AC11) : est-ce juste ? **Cherche une
   phrase des manuels, du README ou du site qui cite un message modifié** — « Erreur chargement profils »,
   « Profil introuvable », un message technique d'`Error` (« Failed to fetch »), un message affiché sur un écran
   touché. Contrôle les **PDF aplatis** (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire `.` à sa place dans un motif.
8. **CHANGELOG** : une seule section `## [0.13.3] — Non publié`, rubrique `### Corrigé`, la phrase de l'AC11 ; rien
   qui promette au-delà du code (angle mort n° 5 : traduction seulement une fois le catalogue chargé) ; « Unreleased »
   ni « à venir » absents.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. L'utilitaire et le client (un
  `message` d'`ApiError` qui n'est pas une chaîne ; `i18nMsg` appelé pendant l'évaluation des modules ; ordre
  d'import en SSR / `hooks.client.ts` ; un écran qui affichait volontairement le `message` d'une `Error` — admin
  backup, `ReconciliationProposals.svelte:206/:224` — et qui affiche désormais le générique : voulu (C-28-2-3) ou
  régression ?). Chaque site : état après erreur (`errorMsg` remis à zéro ? drapeau `bankNotConfigured` ?),
  `handleDelete` (notification, liste rechargée, rejet de `load()` lui-même). `formatError` : une `ApiError` au
  message vide rend `''` (C-28-2-33) — défaut ou périmètre ? La garde : un faux positif ou un faux négatif sur une
  forme réelle du dépôt (gabarit imbriqué, regex, `catch` dans une chaîne, `.catch` en balisage Svelte).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations M1
  à M27 du Dev Agent Record et cherche une **mutation plausible qui resterait verte** (ex. : `errorMessageOf` qui
  rend `err.message` d'une `Error` sans `instanceof` — `(err as Error).message ?? …` — ; une exemption `console`
  calculée sur la ligne au lieu de la liste d'arguments ; G1 limitée aux noms `e`/`err` ; `blocEquilibre` qui ne
  saute plus les regex en parenthèses ; un plancher qui ne rougit pas si le recueil perd les rappels `async` ; le
  scénario 2 de l'E2E vert avec un littéral français). Tests qui passent à vide ? `vi.mock` qui court-circuite le
  code testé (i18n simulé : la traduction est-elle prouvée quelque part avec le **vrai** `i18nMsg` ?) ? L'E2E :
  interception seule, aucune écriture, rejouabilité, `data-testid` seuls, et ce que le scénario 2 n'établit pas
  (C-28-2-23) correctement écrit.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 11) contre le code : satisfait,
  partiel, absent. Les tests nommés à l'AC8 et l'AC9 existent-ils et prouvent-ils ce qu'ils nomment ? L'AC5
  (inventaire recompté, classes A = 0, B = 0, C = 3, D comptée et suivie par #644, E conforme). **Décomptes recomptés
  depuis la source** (sites `isApiError(`, liaisons et planchers, `sitesTotal`, clés i18n, tests ajoutés, Vitest /
  nextest / E2E déclarés) et périmètre de chaque décompte. Les choix C-28-2-30 à 34 : justifiés, ou défaut ? Les
  écarts déclarés au Dev Agent Record : complets ? La recette visuelle : les quatre captures existent, et ce que le
  Dev Agent Record en dit (constat sur `text-destructive`, absence de thème sombre) est-il vrai ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-2-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, `python3`, `python`, `perl`, `sed -i`, `docker`, `gh` en écriture, aucune requête SQL ni HTTP — **aucun
interpréteur**, même pour un script « en lecture seule ». Aucun fichier hors du dépôt, sauf les quatre captures
citées. Autorisés : lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
