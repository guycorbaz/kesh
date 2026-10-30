# Story v017-2 : l'assistant d'installation dit pourquoi le serveur refuse

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172`, branche
`story/v017-2-assistant-motif` partie de `c5101b07` (tête de la candidate 0.16.1-rc1). Release **0.17.0**. Registre :
`v017-2-assistant-motif-choix-autonomes.md` (C-v017-2-1 à C-v017-2-13). Clé courte : `v0172`. Issue : **#809** (bug,
P3), en `refs` seulement dans les commits ; la fermeture se porte sur la PR. #644 et #710 sont citées, **non traitées**.
Les numéros de ligne sont ceux de `c5101b07`.*

## Story

En tant qu'Administrateur qui installe Kesh à neuf, je veux que chaque étape de l'assistant me dise **pourquoi** le
serveur refuse ce que j'ai saisi, et que le champ Pays m'indique la forme attendue, afin de corriger ma saisie au lieu
de deviner — comme le 2026-10-28, où « Suisse » au lieu de `CH` n'a produit que « Erreur lors de la sauvegarde des
coordonnées ».

## Origine

- **#809 (P3)**, constaté par Guy le 2026-10-28 à l'installation neuve de la v0.16.0 : à l'étape des coordonnées, le
  serveur refuse (« adresse : le pays doit être un code ISO à 2 lettres (ex. CH) »), l'écran jette le motif. L'issue
  demande (1) le motif du serveur à chaque étape, le générique sinon ; (2) pour le pays, un sélecteur ou une aide.
- **La story ne change aucune écriture** : elle rend visible un motif. Sur un refus, rien n'est écrit — **sauf** pour la
  démonstration, dont les premières validations commitent avant la dernière transaction (prémisse 5, #538).

## Données à garder, migration — ce que la story ne fait pas

**Aucune migration, aucun bump, aucune écriture sur les données existantes** (contrainte du Project Lead du
2026-10-30, `kesh-gate-logs/prompts/v017-donnees-a-garder.md`). La story touche le frontend de l'assistant, l'utilitaire
`errorMessageOf`, les quatre locales, un test d'intégration du serveur et le manuel utilisateur ; aucune table, aucune
route, aucun code serveur ne change. L'inventaire des tables est inchangé : une sauvegarde 0.16.x s'importe comme avant.
Aucune procédure de la story ne se joue sur la base du NAS : tests, E2E et captures se font sur des bases jetables
(`kesh_e2e`, bases `#[sqlx::test]`). (C-v017-2-12)

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

1. **Confirmée** — `frontend/src/routes/onboarding/+page.svelte:126-128`, `submitCoordinates` :
   `} catch {` puis `toast.error(msg('onboarding-address-save-error', 'Erreur lors de la sauvegarde des coordonnées'));`.
   L'erreur n'est pas liée, son motif ne peut pas être lu.
2. **Confirmée et précisée** — « le même motif se répète aux autres étapes » : six autres `catch {` nus, aux lignes
   `:39` (langue, `'Erreur lors du choix de langue'`, littéral sans clé), `:48` (mode), `:57` (démonstration), `:65`
   (démarrage de la production), `:73` (type d'organisation), `:81` (langue comptable). « Et sans doute la banque » :
   **non** — `:189-194` et `:208-213` affichent déjà le motif (`isApiError(err) && err.message ? err.message : msg(…)`).
3. **Défaut de la même classe, non nommé par l'issue** — la finalisation, appelée après la banque (`:198`, `:217`), passe
   par `reportFinalizeError` (`:139-160`), dont la branche `if (err.code === 'VALIDATION')` (`:153`) est **morte** : le
   serveur rend le code `VALIDATION_ERROR` (`crates/kesh-api/src/errors.rs:1538-1540`,
   `build_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", &msg)`), et `grep -rn "'VALIDATION'" frontend/src crates`
   ne rend que ce site. Le refus « comptes 1100/3000 manquants » (`crates/kesh-api/src/routes/onboarding.rs:871-876`)
   s'affiche donc « Finalisation échouée — réessayez. ». Intégré (C-v017-2-3).
4. **Confirmée** — le message du pays : `crates/kesh-api/src/address_input.rs:66-71`, forme seulement (deux lettres
   ASCII, après `to_uppercase`), message « adresse : le pays doit être un code ISO à 2 lettres (ex. CH) ». **Précision** :
   `SU` ou `XX` passent ; la liste ISO n'est contrôlée que par `kesh-qrbill` (`crates/kesh-qrbill/src/validation.rs:258-271`)
   à la génération d'une QR-facture → défaut voisin, **#817** (C-v017-2-8).
5. **Confirmée, sauf pour la démonstration** — « rien n'est écrit sur un refus » : `set_coordinates` valide le nom
   (`routes/onboarding.rs:526-530`), l'adresse (`:532`) et l'IDE (`:535-541`) **avant** `get_or_init_state` et
   l'ouverture de la transaction (`:544-545`). S1, S2, S4 à S6 et la banque rejettent leurs valeurs invalides avant
   toute écriture, ou dans une transaction annulée (`conclude_step`, `:1130-1145`). **Exception : la démonstration
   (S3)** — `crates/kesh-seed/src/lib.rs:162-172` : « Seule la **dernière transaction** est atomique avec sa trace. Les
   quatre premières validations […] commitent chacune la leur, AVANT elle. […] relancer `seed-demo` échoue (500) ». Le
   refus que S3 affichera désormais (`routes/onboarding.rs:267`, « … avant de relancer la démo ») peut suivre des
   écritures partielles, et le rejeu qu'il conseille rend 500. Défaut préexistant, suivi par **#538** ; la story n'y
   touche pas (validation P1, R1).
6. **Les messages des refus sont en français en dur** (`AppError::Validation("…")`, `address_input.rs:61, 68, 90` ;
   `routes/onboarding.rs:163, 202, 267, 378, 423, 527, 538, 594, 601, 607, 873`) : un utilisateur DE/IT/EN lira le motif
   en français. Suivi par **#710** ; accepté ici (C-v017-2-5). ⚠️ Le `grep` de #710 (`AppError::Validation("` dans
   `routes/`) ne voit ni `address_input.rs` (hors `routes/`) ni les formes multi-lignes (`address_input.rs:67-70`,
   `routes/onboarding.rs:871-876`) : complément à poser sur #710 (§ *Commentaires d'issue à poser*).
7. **L'utilitaire existe** — `frontend/src/lib/shared/utils/error-message.ts:43-50`, `errorMessageOf(err)` (Story 28-2) :
   message d'une `ApiError` non blanche, sinon « Erreur inattendue. » ; toute valeur non `ApiError` est journalisée.
   Il ne connaît pas de message propre à l'appelant (C-v017-2-2).

### Inventaire clos des `catch` de l'assistant

`grep -rn "catch" frontend/src/routes/onboarding frontend/src/lib/features/onboarding | grep -v test | grep -vE ':[0-9]+:\s*//'`
(le dernier filtre écarte `onboarding.svelte.ts:156`, un commentaire ; reste quatorze moins un = treize lignes, les
treize sites ci-dessous) :

| # | Site | Aujourd'hui | Après la story |
|---|---|---|---|
| S1 | `+page.svelte:39` `chooseLanguage` | nu, littéral | motif / `onboarding-language-error` (clé neuve, repli `'Erreur lors du choix de langue'`) |
| S2 | `+page.svelte:48` `chooseMode` | nu | motif / `onboarding-mode-error` |
| S3 | `+page.svelte:57` `chooseDemoPath` | nu | motif / `onboarding-demo-load-error` |
| S4 | `+page.svelte:65` `chooseProductionPath` | nu | motif / `onboarding-setup-start-error` |
| S5 | `+page.svelte:73` `chooseOrgType` | nu | motif / `onboarding-org-type-error` |
| S6 | `+page.svelte:81` `chooseAccountingLang` | nu | motif / `onboarding-accounting-lang-error` |
| S7 | `+page.svelte:126` `submitCoordinates` | nu | motif / `onboarding-address-save-error` |
| S8 | `+page.svelte:189` `submitBankAccount` | motif à la main | `errorMessageOf(err, msg('onboarding-bank-save-error', …))` |
| S9 | `+page.svelte:200` finalisation après banque | `reportFinalizeError`, branche morte | voir AC-4 |
| S10 | `+page.svelte:208` `handleSkipBank` | motif à la main | `errorMessageOf(err, msg('onboarding-bank-skip-error', …))` |
| S11 | `+page.svelte:219` finalisation après saut | `reportFinalizeError`, branche morte | voir AC-4 |
| — | `onboarding.svelte.ts:163` `finalize` | relance (`throw err`) | inchangé — n'affiche rien, l'appelant affiche |
| — | `+layout.ts:33` chargement de l'état | silence journalisé | **hors périmètre**, **#818** (C-v017-2-7) |

`/onboarding/restore` rend `AdminRestorePanel` (composant partagé avec `/admin/restore`), dont les `catch` sont
`AdminRestorePanel.svelte:157` (affiche `importErrorMessage(err)`) et `:181` (silence voulu, `relogin`) ; aucun n'est
un `catch` muet à l'enseigne de #809. Le panneau est nommé par #644 : hors périmètre.

## Critères d'acceptation

### Le motif

- **AC-1 — `errorMessageOf` reçoit un repli optionnel.** Signature `errorMessageOf(err: unknown, repli?: string): string`.
  Sans `repli`, comportement **identique** à aujourd'hui (les cas de `error-message.test.ts` restent verts, inchangés).
  Avec `repli` : `ApiError` à message chaîne non blanche → ce message, sans journalisation ; `ApiError` à message vide,
  blanc ou non-chaîne → `repli`, sans journalisation ; toute autre valeur → `repli`, **et** `console.error` comme
  aujourd'hui. Un `repli` vide ou blanc est traité comme absent (générique). Le doc-comment du module décrit le
  paramètre.
- **AC-2 — Les sept étapes à `catch` nu (S1 à S7) affichent le motif.** Chaque site lie l'erreur et appelle
  `toast.error(errorMessageOf(err, <repli de l'étape>))`, le repli étant le message d'aujourd'hui, clé et texte
  inchangés (tableau ci-dessus). **S1** passe par une clé neuve, `onboarding-language-error` (validation P1, F10 :
  `msg(clé, repli)` retombe sur son repli quand le catalogue manque, l'absence de catalogue ne justifiait pas le
  littéral) : repli `'Erreur lors du choix de langue'`, valeurs aux quatre locales en AC-7.
- **AC-3 — Le compte bancaire (S8, S10) passe par la même règle**, sans changement visible pour une `ApiError` à
  message : seuls changent le message blanc (repli au lieu d'un toast blanc) et la journalisation d'une panne imprévue.
- **AC-4 — La finalisation (S9, S11) dit son motif.** `reportFinalizeError` : `ONBOARDING_STEP_ALREADY_COMPLETED` garde
  `onboarding-already-finalized` ; **tout le reste** rend `errorMessageOf(err, msg('onboarding-finalize-error', …))`. La
  comparaison à `'VALIDATION'` disparaît. Un 400 `VALIDATION_ERROR` « Impossible de pré-remplir les comptes… » s'affiche
  tel quel. **Limite déclarée (validation P1, R3) : le motif s'affiche, la reprise n'est pas traitée.** Après une banque
  enregistrée ou sautée, l'état vaut 7 ; `+page.svelte` n'a pas de branche pour l'étape 7 (la chaîne `{#if}` s'arrête à
  `stepCompleted === 6`) et `+layout.ts:30` redirige vers `/` dès `!isDemo && step >= 7`. Un refus de la finalisation
  laisse donc le toast sur un écran vide, sans bouton de reprise — défaut préexistant, **#821** (P3), hors périmètre.
  La consigne du refus « comptes 1100/3000 » (« Veuillez ajouter ces comptes… », `routes/onboarding.rs:874`) n'est pas
  exécutable dans l'assistant (#44 ; validation P1, F8).
- **AC-5 — Garde lexicale de l'assistant** (C-v017-2-9). Un test Vitest lit `frontend/src/routes/onboarding/+page.svelte`,
  commentaires masqués (réutiliser `masquerCommentaires` de `$lib/shared/i18n-literal-reader.js`), et échoue si :
  (a) un `catch` n'a pas de liaison (`catch {`) ; (b) le bloc d'un `catch (x)` n'appelle ni `errorMessageOf(` ni
  `reportFinalizeError(`. Plancher : **onze** `catch` recueillis (S1 à S11) — un recueil qui passe à vide rougit.
  Recueil (validation P1, R6) : après masquage, motif `/(^|[^.\w$:])catch\s*(\(|\{)/g` (le `:` écarte `{:catch}`, le `.`
  écarte `.catch(`) ; le bloc se lit par `corpsDeFonction`/`blocEquilibre` (`i18n-literal-reader.js`). Témoins : une
  source factice à `catch {` et une à `catch (e) { toast.error('x') }` rougissent la fonction de contrôle ; une source
  **conforme** (`catch (e) { toast.error(errorMessageOf(e, 'r')) }`) rend zéro violation.

### Le champ Pays

- **AC-6 — Une indication sous le champ** (C-v017-2-4) : la clé neuve `onboarding-field-country-hint` rendue dans un
  `<p id="coord-country-hint" class="mt-1 text-xs text-text-muted">` **sous** l'`Input` du pays, relié par
  `aria-describedby="coord-country-hint"`. Pas dans le libellé, contrairement à l'IDE (`+page.svelte:363-366`) : la
  cellule du pays occupe un tiers de la grille (`grid-cols-3`, `:339`), et le texte allemand y ferait passer le libellé
  sur plusieurs lignes, décalant le champ de ses voisins (validation P1, F4). Le champ **n'a pas** de `maxlength` (une saisie en toutes lettres doit atteindre le
  serveur et revenir expliquée, non être tronquée en deux lettres acceptées). Valeur initiale `CH` et `placeholder`
  inchangés.
- **AC-7 — Les quatre locales** portent les deux clés neuves, repli du code égal à la valeur fr-CH :

  | locale | `onboarding-field-country-hint` | `onboarding-language-error` |
  |---|---|---|
  | fr-CH | `code ISO à deux lettres, ex. CH, FR, DE` | `Erreur lors du choix de langue` |
  | de-CH | `ISO-Code mit zwei Buchstaben, z. B. CH, FR, DE` | `Fehler bei der Sprachwahl` |
  | it-CH | `codice ISO di due lettere, ad es. CH, FR, DE` | `Errore nella scelta della lingua` |
  | en-CH | `two-letter ISO code, e.g. CH, FR, DE` | `Error while choosing the language` |

  La première à côté d'`onboarding-field-ide-hint` (ligne 141 de chaque `messages.ftl`), la seconde à côté
  d'`onboarding-mode-error` (dont elle suit la forme dans chaque locale).

### Le serveur

- **AC-8 — Le contrat du refus est fixé** (C-v017-2-11) : test `coordinates_rejects_country_name` dans
  `crates/kesh-api/tests/onboarding_path_b_e2e.rs`, sur le modèle de `coordinates_validates_ide` (`:171`) : à l'étape 5,
  `POST /api/v1/onboarding/coordinates` avec `"country": "Suisse"` → `400`, `error.code == "VALIDATION_ERROR"`,
  `error.message` contient `code ISO à 2 lettres` ; puis `GET /api/v1/onboarding/state` → `stepCompleted == 5`. Aucun
  code serveur ne change.

### Textes

- **AC-9 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex:201`) : la puce *Coordonnées légales* précise
  « adresse (le pays en code ISO à deux lettres, CH par défaut) » et ajoute une phrase vraie au code : si Kesh refuse
  une saisie, l'assistant affiche le motif que le serveur donne, **qui peut être rédigé en français quelle que soit la
  langue choisie** (#710) ; on corrige le champ et on continue (validation P1, F2). PDF régénéré (`make fr` dans
  `docs/manual/`, sous `scripts/mem-guard.sh`) et contrôlé aplati, **zéro avant, un après**, sur deux fragments sans
  ligature (`ff`, `ffi`) ni apostrophe, absents du PDF actuel (validation P1, F6) :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF "code ISO à deux lettres"` et le même
  `grep -cF` sur un fragment de la phrase ajoutée choisi à la rédaction (par exemple « quelle que soit la langue
  choisie ») — fragment et deux comptes cités au Dev Agent Record. La capture `onboarding-production.png` (légende
  `user-manual.tex:196`) n'existe pas au dépôt (`docs/manual/fr/screenshots/` ne porte que `_placeholder.png`) : la
  légende reste exacte, l'image absente n'est pas traitée ici (validation P1, F5). Le manuel d'administration
  (`admin-manual.tex:1069, 1419`) ne détaille pas les champs : inchangé — à vérifier, non à supposer.
- **AC-10 — CHANGELOG** : sous `## [0.17.0]` (en-tête que l'orchestrateur fusionnera), section **Corrigé** : « L'assistant
  d'installation affiche, à chacune de ses étapes de saisie, finalisation comprise, le motif d'un refus tel que le serveur
  le donne, au lieu d'un message générique ; le champ Pays indique la forme attendue (code ISO à deux lettres)
  ([#809](…/issues/809)). » (« de saisie » et « tel que le serveur le donne » : validation P1, F3 — le chargement de
  l'état, #818, et la restauration, #644, restent hors du correctif, et le motif peut être en français, #710.) Aucun
  bump de version.

## Tâches

1. **Étape 0** — recontrôler les prémisses 1 à 7 et l'inventaire (`grep` ci-dessus) sur la tête de la branche ;
   lister au Dev Agent Record toute ligne qui a bougé.
2. **Tests rouges d'abord** (AC-1, AC-2 à AC-5, AC-6, AC-8), puis le code.
3. `error-message.ts` (AC-1) ; `+page.svelte` (AC-2 à AC-4, AC-6) ; locales (AC-7).
4. Test d'intégration (AC-8).
5. E2E (§ *E2E*) et captures (§ *Captures*).
6. Manuel et PDF (AC-9), CHANGELOG (AC-10).
7. Propagation : `grep -rn "'VALIDATION'" frontend/src` (vide après) ; `grep -rn "catch {" frontend/src/routes/onboarding`
   (vide après) ; `grep -rn "onboarding-field-country-hint\|onboarding-language-error" crates/kesh-i18n/locales frontend/src`
   (quatre locales + un site chacune) ; `grep -rnF "Erreur lors du choix de langue" frontend/src` (le seul repli de
   `msg(…)`).

## Tests et mutations prévus

**Vitest** — fichier neuf `frontend/src/routes/onboarding/onboarding-motif.test.ts`, sur le montage
d'`onboarding-restore-link.test.ts` (mocks de `onboarding.api`, `i18n.svelte` à repli, `svelte-sonner`,
`$app/navigation`, `auth.svelte`) :

- pour **chaque** site S1 à S8 et S10 : l'appel d'API rejette `{ code: 'VALIDATION_ERROR', message: 'MOTIF-Sk', status: 400 }`
  → `toast.error` reçoit `'MOTIF-Sk'` ; S7 se déclenche en remplissant **nom**, NPA et localité puis en soumettant (les
  gardes client `:98-111` exigent les trois avant l'appel ; validation P1, R4) ; S2 à S6 par un clic sur le bouton de
  l'étape ; S1 à l'étape 0 ;
- pour **chacun** des sites S1 à S8 et S10 (un `it.each` sur la table de l'inventaire, le repli attendu étant le texte
  de la clé de l'étape ; validation P1, R2) : rejet par `new TypeError('Failed to fetch')` → le repli **de cette étape**,
  et `console.error` appelé ; pour S1, S7, S8 en plus : rejet par une `ApiError` au message `'  '` → le repli de
  l'étape ;
- S9 et S11 : `setBankAccount` / `skipBank` résolus, `finalize` rejette `VALIDATION_ERROR` « MOTIF-FIN » → `'MOTIF-FIN'` ;
  rejette `ONBOARDING_STEP_ALREADY_COMPLETED` → le texte d'`onboarding-already-finalized` ; rejette une `TypeError` →
  `onboarding-finalize-error` ;
- mise en œuvre (validation P1, R5) : `_loading` est un état de module, et `finalize` relâche son verrou par
  `setTimeout(0)` après un rejet (`onboarding.svelte.ts:156-170`) — un `it` par scénario de finalisation, ou une
  macrotâche attendue entre deux ; les simulations de `setBankAccount`/`skipBank`/`fetchState` rendent un
  `OnboardingState` **complet** (`stepCompleted`, `isDemo`, `uiMode`, `isStub`) ;
- AC-6 : à l'étape 5, `#coord-country` porte `aria-describedby="coord-country-hint"`, l'élément `#coord-country-hint`
  contient `code ISO à deux lettres`, et le champ n'a pas d'attribut `maxlength`.

**`error-message.test.ts`** — cas du repli (AC-1) : ApiError à message → message ; message blanc → repli ; TypeError
→ repli + `console.error` ; repli blanc → générique ; sans repli → inchangé.

**Garde** — `frontend/src/routes/onboarding/onboarding-catch-guard.test.ts` (AC-5), témoins compris.

**Rust** — `coordinates_rejects_country_name` (AC-8).

**Mutations** (sous verrou de gate, binaire touché après restauration, cf. la mémoire « mutation restaurée, binaire
périmé » — pour Vitest, relancer suffit) :

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | S7 remis en `catch { toast.error(msg('onboarding-address-save-error', …)) }` | test S7 ; garde AC-5 |
| M2 | S5 remis en `catch {` nu | test S5 ; garde AC-5 |
| M3 | `errorMessageOf` ignore `repli` (rend toujours le générique) | cas repli d'`error-message.test.ts` ; tests TypeError de S1/S7/S8 |
| M4 | `errorMessageOf` rend `repli` même pour une ApiError à message | cas « ApiError → message » avec repli ; tous les tests MOTIF-Sk |
| M5 | `reportFinalizeError` rétablit `if (err.code === 'VALIDATION')` et le générique sinon | tests S9/S11 « MOTIF-FIN » |
| M6 | retrait de l'indication du pays | test AC-6 |
| M7 | `maxlength={2}` ajouté au champ pays | test AC-6 |
| M8 | la garde AC-5 ne recueille plus rien (motif de `catch` cassé) | plancher de la garde |
| M9 | S8 : `errorMessageOf(err)` sans repli | test TypeError de S8 |
| M10 | S5 reçoit le repli de S2 (`onboarding-mode-error`) ; puis, séparément, S10 appelle `errorMessageOf(err)` sans repli | test TypeError de S5 ; test TypeError de S10 (validation P1, R2) |
| M11 | S1 rétablit le littéral sans clé (`errorMessageOf(err, 'Erreur lors du choix de langue')`) | test TypeError de S1 : la simulation d'`i18nMsg` est un `vi.fn` qui rend le repli **et** enregistre ses appels ; le test exige un appel avec `'onboarding-language-error'` (le texte seul ne distingue pas la clé du littéral) |

Chaque mutation se déclare au Dev Agent Record avec le test qui a rougi, la sortie citée.

## E2E

Spec neuve `frontend/tests/e2e/onboarding-motif.spec.ts` (C-v017-2-10), preset `fresh` (`seedTestState('fresh')`,
connexion `changeme` par `#username`/`#password` et `button[type="submit"]`, comme `onboarding-path-b.spec.ts:14-19`).
**L'état est amené par l'API, seul l'écran des coordonnées se joue par l'interface** (validation P1, F1 ; consigne de
l'orchestrateur) : `authedApiContext(page)` (`helpers/test-state.ts:174`), puis, dans l'ordre et en vérifiant chaque
statut 200, `POST /api/v1/onboarding/language` `{ language: 'FR' }`, `/mode` `{ mode: 'guided' }`,
`/start-production`, `/org-type` `{ orgType: 'Independant' }`, `/accounting-language` `{ language: 'FR' }` (corps de
`onboarding.api.ts`) ; puis `page.goto('/onboarding')` (rechargement complet : l'état se relit, étape 5). Sur l'écran :
`#coord-name`, `#coord-npa` `1000`, `#coord-city` `Lausanne`, `#coord-country` `Suisse`, soumission par
`page.locator('form button[type="submit"]')` → **un toast `[data-sonner-toast]` contient « code ISO à 2 lettres »** et
`#coord-country` reste visible ; `#coord-country` `CH`, soumission → `#bank-name` visible. Aucun sélecteur de texte
(la garde `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts` tient la liste des couples fichier/libellé, et un
fichier neuf n'y figure pas), **aucun `data-testid` à ajouter** ; **pas** `#coord-address`, disparu (cause de KF-029).
`afterEach` : `clearAuthStorage`. Ordre alphabétique (`workers: 1`) : la spec passe avant `onboarding-path-b`, qui
reseede en `fresh` — sans effet sur elle. Suite E2E complète au dernier commit de code ; échecs jugés fichier par
fichier contre `docs/testing.md` § *Les échecs attendus* (les `onboarding*.spec.ts` de KF-029 y sont ; la spec neuve ne
doit pas y entrer). `docs/testing.md:153` (tableau des presets) reçoit la spec neuve dans la ligne `fresh` (validation
P1, F9).

## Captures (thème clair)

Sur la base E2E (`kesh_e2e`, preset `fresh`), jamais sur la base du NAS, rangées sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172/` : (1) l'étape des coordonnées, indication du pays visible ; (2) le toast du
refus « Suisse » avec son motif ; (3) un refus d'IDE (`CHE-000`) avec son motif ; (4) l'étape des coordonnées en
**de-CH** (le libellé le plus long, validation P1, F4). Jointes au Dev Agent Record et au prompt
de la revue de code.

## Gates

Frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
(`scripts/test-fast.sh`, sous verrou `.gate.lock`, base `kesh_v0172` remise à zéro avant) ; E2E complète au dernier
commit de code. Le Dev Agent Record n'affirme que ce qui a tourné.

## Ce que les gardes ne voient pas

- La garde AC-5 ne lit que `+page.svelte` : un `catch` ajouté dans un composant extrait de l'assistant lui échappe.
- Elle vérifie qu'un appel à `errorMessageOf(`/`reportFinalizeError(` figure dans le bloc, pas que son résultat est
  affiché.
- Elle ne vérifie pas que l'argument d'`errorMessageOf(` est la liaison du `catch` (`catch (err) { … errorMessageOf(autre) }`
  passe) ; `{:catch}` et `.catch(` sont hors recueil (aucun dans l'assistant à `c5101b07`).
- Le motif reste en français pour DE/IT/EN (#710).
- Un pays de deux lettres hors liste ISO (`SU`, `XX`) est accepté par le serveur (#817, C-v017-2-8).
- Démonstration (S3) : le motif affiché peut suivre des écritures partielles, et le rejeu qu'il conseille rend 500
  (#538, prémisse 5).
- Finalisation refusée : toast sur un écran vide, sans reprise (#821, AC-4).

## Dev Notes

### Fichiers touchés (prévus) et modules

- `frontend/src/lib/shared/utils/error-message.ts` (+ `.test.ts`) — utilitaire partagé, paramètre optionnel seulement.
- `frontend/src/routes/onboarding/+page.svelte`, tests neufs `onboarding-motif.test.ts`, `onboarding-catch-guard.test.ts`.
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — deux clés.
- `crates/kesh-api/tests/onboarding_path_b_e2e.rs` — un test.
- `frontend/tests/e2e/onboarding-motif.spec.ts` ; `docs/testing.md` (une ligne, tableau des presets).
- `docs/manual/fr/user-manual.tex` (+ PDF), `CHANGELOG.md`.

Quatre modules (frontend assistant, utilitaire partagé, i18n, tests serveur) : sous le seuil de découpage.

### Le manuel (axe de revue)

`docs/manual/fr/user-manual.tex:196-203` (légende de la capture absente `:196`, énumération `:198-203` ; étapes de l'assistant en production) ; `admin-manual.tex:1069` et `:1419`
(mentions de l'assistant). Contrôler le PDF aplati, pas seulement le `.tex`.

### Références

`#809`, `#644` (classe voisine, non traitée), `#710` (refus non traduits), `#97` (KF-029), `#538` (démonstration non
atomique), `#44` (ajout des comptes manquants à la finalisation), `#817`, `#818`, `#821`. Story 28-2 (`errorMessageOf`,
`erreurs-en-texte.test.ts`). `CLAUDE.md` du worktree : *Test Locally First*, *Recette visuelle*, *Issue Tracking Rule*.

### Issues ouvertes à la suite de la fiche

- **#817** — pays contrôlé à la forme seulement (C-v017-2-8), P2.
- **#818** — chargement de l'assistant en échec silencieux (`+layout.ts:33`, C-v017-2-7), P3.
- **#821** — finalisation refusée : écran vide, sans reprise (validation P1, R3), P3.

### Commentaires d'issue à poser (par l'orchestrateur)

- **#710** (validation P1, F7) : « Complément d'inventaire, relevé à la fiche v017-2 (#809) : le `grep -rn
  'AppError::Validation("' crates/kesh-api/src/routes/` de cette issue ne voit pas (a) les refus hors de `routes/`, dont
  `crates/kesh-api/src/address_input.rs` (« adresse : le pays doit être un code ISO à 2 lettres (ex. CH) », le message
  même de #809, et les longueurs d'adresse) ; (b) les formes multi-lignes laissées par rustfmt, où la chaîne suit
  `AppError::Validation(` à la ligne suivante (`address_input.rs:67-70`, `routes/onboarding.rs:871-876`) ; (c) les
  messages de `kesh-core` repris par interpolation (`che_number.rs`, via « IDE invalide : {e} » ; idem IBAN). Un
  inventaire clos devra chercher `AppError::Validation(` sur plusieurs lignes, dans tout `crates/kesh-api/src/`. »

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie). Registre C-v017-2-1 à C-v017-2-12.
- 2026-10-30 — **Validation P1** (Sonnet, lentilles R et F en contexte frais ; rapports
  `kesh-gate-logs/v017-2-assistant-motif-validate-p1-{R,F}.md`) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 12 LOW. Remédiation
  (Opus 5.5), chaque prémisse relue au code avant application :
  - MEDIUM, tous appliqués : **R1** prémisse 5 bornée (démonstration non atomique, `kesh-seed/src/lib.rs:162-172`, #538),
    Origine et C-v017-2-12 amendés ; **R2** test « TypeError → repli de l'étape » étendu aux neuf sites S1-S8, S10, mutation
    M10 ; **R3** limite déclarée en AC-4 (étape 7 sans écran, `+layout.ts:30`), #821 ; **F1** E2E : état amené par l'API,
    seul l'écran des coordonnées joué, aucun sélecteur de texte, aucun `data-testid` ; **F2** phrase du manuel : le motif
    peut être en français (#710).
  - LOW appliqués : R4 (nom requis avant l'appel), R5 (verrou `_loading`, états simulés complets), R6 (motif de recueil,
    témoin conforme, angles morts), R7 (`AdminRestorePanel.svelte:157/:181`, commande d'inventaire filtrée du
    commentaire `onboarding.svelte.ts:156`), F3 (CHANGELOG « étapes de saisie », « tel que le serveur le donne »), F4
    (indication sous le champ, `aria-describedby`, capture de-CH), F5 (lignes 196, 198-203 ; capture absente déclarée),
    F6 (contrôle du PDF zéro avant / un après, fragments sans ligature), F7 (complément pour #710 au § *Commentaires
    d'issue à poser*), F8 (consigne « ajouter ces comptes » non exécutable, #44), F9 (`docs/testing.md` aux fichiers
    touchés), F10 (clé `onboarding-language-error` aux quatre locales au lieu du littéral de S1, mutation M11 ;
    l'étape 0 ne porte plus la condition).
  - LOW non pris : aucun.
  - Issues : #817, #818, #821 ouvertes par l'orchestrateur ; la fiche les cite. Prochaine : validation P2 (Opus,
    passe complète, prompt `v017-2-assistant-motif-validate-prompt-p2.md`).
