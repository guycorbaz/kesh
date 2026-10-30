# Story v017-2 : l'assistant d'installation dit pourquoi le serveur refuse

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172`, branche
`story/v017-2-assistant-motif` partie de `c5101b07` (tête de la candidate 0.16.1-rc1). Release **0.17.0**. Registre :
`v017-2-assistant-motif-choix-autonomes.md` (C-v017-2-1 à C-v017-2-12). Clé courte : `v0172`. Issue : **#809** (bug,
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
- **Aucune perte de données** : sur un refus, rien n'est écrit (prémisse 5).

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
   à la génération d'une QR-facture → défaut voisin, issue à ouvrir (C-v017-2-8).
5. **Confirmée** — « rien n'est écrit sur un refus » : `set_coordinates` valide le nom (`routes/onboarding.rs:526-530`),
   l'adresse (`:532`) et l'IDE (`:535-541`) **avant** `get_or_init_state` et l'ouverture de la transaction (`:544-545`).
   Les autres étapes rejettent leurs valeurs invalides avant toute écriture, ou dans une transaction annulée
   (`conclude_step`).
6. **Les messages des refus sont en français en dur** (`AppError::Validation("…")`, `address_input.rs:61, 68, 90` ;
   `routes/onboarding.rs:163, 202, 267, 378, 423, 527, 538, 594, 601, 607, 873`) : un utilisateur DE/IT/EN lira le motif
   en français. Suivi par **#710** ; accepté ici (C-v017-2-5).
7. **L'utilitaire existe** — `frontend/src/lib/shared/utils/error-message.ts:43-50`, `errorMessageOf(err)` (Story 28-2) :
   message d'une `ApiError` non blanche, sinon « Erreur inattendue. » ; toute valeur non `ApiError` est journalisée.
   Il ne connaît pas de message propre à l'appelant (C-v017-2-2).

### Inventaire clos des `catch` de l'assistant

`grep -rn "catch" frontend/src/routes/onboarding frontend/src/lib/features/onboarding | grep -v test` :

| # | Site | Aujourd'hui | Après la story |
|---|---|---|---|
| S1 | `+page.svelte:39` `chooseLanguage` | nu, littéral | motif / repli `'Erreur lors du choix de langue'` |
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
| — | `+layout.ts:33` chargement de l'état | silence journalisé | **hors périmètre**, issue à ouvrir (C-v017-2-7) |

`/onboarding/restore` rend `AdminRestorePanel` (`AdminRestorePanel.svelte:84`), nommé par #644 : hors périmètre.

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
  inchangés (tableau ci-dessus ; S1 garde son littéral, faute de catalogue avant le choix de la langue — à confirmer à
  l'étape 0 contre les gardes i18n).
- **AC-3 — Le compte bancaire (S8, S10) passe par la même règle**, sans changement visible pour une `ApiError` à
  message : seuls changent le message blanc (repli au lieu d'un toast blanc) et la journalisation d'une panne imprévue.
- **AC-4 — La finalisation (S9, S11) dit son motif.** `reportFinalizeError` : `ONBOARDING_STEP_ALREADY_COMPLETED` garde
  `onboarding-already-finalized` ; **tout le reste** rend `errorMessageOf(err, msg('onboarding-finalize-error', …))`. La
  comparaison à `'VALIDATION'` disparaît. Un 400 `VALIDATION_ERROR` « Impossible de pré-remplir les comptes… » s'affiche
  tel quel.
- **AC-5 — Garde lexicale de l'assistant** (C-v017-2-9). Un test Vitest lit `frontend/src/routes/onboarding/+page.svelte`,
  commentaires masqués (réutiliser `masquerCommentaires` de `$lib/shared/i18n-literal-reader.js`), et échoue si :
  (a) un `catch` n'a pas de liaison (`catch {`) ; (b) le bloc d'un `catch (x)` n'appelle ni `errorMessageOf(` ni
  `reportFinalizeError(`. Plancher : **onze** `catch` recueillis (S1 à S11) — un recueil qui passe à vide rougit.
  Témoins : une source factice à `catch {` et une à `catch (e) { toast.error('x') }` rougissent la fonction de contrôle.

### Le champ Pays

- **AC-6 — Une indication au libellé** (C-v017-2-4) : `{msg('field-country', 'Pays')}` suivi, comme pour l'IDE
  (`+page.svelte:363-366`), de `<span class="font-normal">(…)</span>` portant la clé neuve
  `onboarding-field-country-hint`. Le champ **n'a pas** de `maxlength` (une saisie en toutes lettres doit atteindre le
  serveur et revenir expliquée, non être tronquée en deux lettres acceptées). Valeur initiale `CH` et `placeholder`
  inchangés.
- **AC-7 — Les quatre locales** portent la clé, repli du code égal à la valeur fr-CH :

  | locale | `onboarding-field-country-hint` |
  |---|---|
  | fr-CH | `code ISO à deux lettres, ex. CH, FR, DE` |
  | de-CH | `ISO-Code mit zwei Buchstaben, z. B. CH, FR, DE` |
  | it-CH | `codice ISO di due lettere, ad es. CH, FR, DE` |
  | en-CH | `two-letter ISO code, e.g. CH, FR, DE` |

  Placée à côté d'`onboarding-field-ide-hint` (ligne 141 de chaque `messages.ftl`).

### Le serveur

- **AC-8 — Le contrat du refus est fixé** (C-v017-2-11) : test `coordinates_rejects_country_name` dans
  `crates/kesh-api/tests/onboarding_path_b_e2e.rs`, sur le modèle de `coordinates_validates_ide` (`:171`) : à l'étape 5,
  `POST /api/v1/onboarding/coordinates` avec `"country": "Suisse"` → `400`, `error.code == "VALIDATION_ERROR"`,
  `error.message` contient `code ISO à 2 lettres` ; puis `GET /api/v1/onboarding/state` → `stepCompleted == 5`. Aucun
  code serveur ne change.

### Textes

- **AC-9 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex:201`) : la puce *Coordonnées légales* précise
  « adresse (le pays en code ISO à deux lettres, CH par défaut) » et ajoute une phrase : si Kesh refuse une saisie,
  l'assistant affiche le motif du refus ; on corrige le champ et on continue. PDF régénéré (`make fr` dans
  `docs/manual/`, sous `scripts/mem-guard.sh`) et contrôlé aplati
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "code ISO à deux lettres[^.]*"`).
  Le manuel d'administration (`admin-manual.tex:1069, 1419`) ne détaille pas les champs : inchangé — à vérifier, non à
  supposer.
- **AC-10 — CHANGELOG** : sous `## [0.17.0]` (en-tête que l'orchestrateur fusionnera), section **Corrigé** : « L'assistant
  d'installation affiche le motif d'un refus du serveur à chacune de ses étapes, finalisation comprise, au lieu d'un
  message générique ; le champ Pays indique la forme attendue (code ISO à deux lettres) ([#809](…/issues/809)). » Aucun
  bump de version.

## Tâches

1. **Étape 0** — recontrôler les prémisses 1 à 7 et l'inventaire (`grep` ci-dessus) sur la tête de la branche ;
   lister au Dev Agent Record toute ligne qui a bougé. Vérifier que les gardes i18n (`i18n-un-repli-par-cle.test.ts`,
   `i18n-keys.test.ts`, `lint-i18n-ownership`, la garde des littéraux) acceptent le littéral de S1 passé en repli ; si
   l'une refuse, créer `onboarding-language-error` aux quatre locales (fr « Erreur lors du choix de langue ») et le
   consigner.
2. **Tests rouges d'abord** (AC-1, AC-2 à AC-5, AC-6, AC-8), puis le code.
3. `error-message.ts` (AC-1) ; `+page.svelte` (AC-2 à AC-4, AC-6) ; locales (AC-7).
4. Test d'intégration (AC-8).
5. E2E (§ *E2E*) et captures (§ *Captures*).
6. Manuel et PDF (AC-9), CHANGELOG (AC-10).
7. Propagation : `grep -rn "'VALIDATION'" frontend/src` (vide après) ; `grep -rn "catch {" frontend/src/routes/onboarding`
   (vide après) ; `grep -rn "onboarding-field-country-hint" crates/kesh-i18n/locales frontend/src` (quatre locales + un
   site).

## Tests et mutations prévus

**Vitest** — fichier neuf `frontend/src/routes/onboarding/onboarding-motif.test.ts`, sur le montage
d'`onboarding-restore-link.test.ts` (mocks de `onboarding.api`, `i18n.svelte` à repli, `svelte-sonner`,
`$app/navigation`, `auth.svelte`) :

- pour **chaque** site S1 à S8 et S10 : l'appel d'API rejette `{ code: 'VALIDATION_ERROR', message: 'MOTIF-Sk', status: 400 }`
  → `toast.error` reçoit `'MOTIF-Sk'` ; S7 se déclenche en remplissant NPA et localité puis en soumettant (les gardes
  client `:102-111` passent avant l'appel) ; S2 à S6 par un clic sur le bouton de l'étape ; S1 à l'étape 0 ;
- pour S1, S7, S8 : rejet par `new TypeError('Failed to fetch')` → le repli de l'étape, et `console.error` appelé ;
  rejet par une `ApiError` au message `'  '` → le repli de l'étape ;
- S9 et S11 : `setBankAccount` / `skipBank` résolus, `finalize` rejette `VALIDATION_ERROR` « MOTIF-FIN » → `'MOTIF-FIN'` ;
  rejette `ONBOARDING_STEP_ALREADY_COMPLETED` → le texte d'`onboarding-already-finalized` ; rejette une `TypeError` →
  `onboarding-finalize-error` ;
- AC-6 : à l'étape 5, le libellé du champ `#coord-country` contient `code ISO à deux lettres` et l'élément n'a pas
  d'attribut `maxlength`.

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

Chaque mutation se déclare au Dev Agent Record avec le test qui a rougi, la sortie citée.

## E2E

Spec neuve `frontend/tests/e2e/onboarding-motif.spec.ts` (C-v017-2-10), preset `fresh` (`seedTestState('fresh')`,
connexion `changeme`) : Français → Guidé → Configurer pour la production → Indépendant → Français → coordonnées (nom,
NPA `1000`, localité `Lausanne`, pays `Suisse`) → **un toast contient « code ISO à 2 lettres »** et l'écran reste sur
les coordonnées ; pays `CH` → l'étape bancaire s'affiche (`#bank-name` visible). Sélecteurs réels de
`+page.svelte` (`#coord-name`, `#coord-npa`, `#coord-city`, `#coord-country`, `data-testid="onboarding-org-type-independant"`) ;
**pas** `#coord-address`, disparu (cause de KF-029). `afterEach` : `clearAuthStorage`. Suite E2E complète au dernier
commit de code ; échecs jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* (les
`onboarding*.spec.ts` de KF-029 y sont ; la spec neuve ne doit pas y entrer).

## Captures (thème clair)

Sur la base E2E (`kesh_e2e`, preset `fresh`), jamais sur la base du NAS, rangées sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172/` : (1) l'étape des coordonnées, indication du pays visible ; (2) le toast du
refus « Suisse » avec son motif ; (3) un refus d'IDE (`CHE-000`) avec son motif. Jointes au Dev Agent Record et au prompt
de la revue de code.

## Gates

Frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; backend complet
(`scripts/test-fast.sh`, sous verrou `.gate.lock`, base `kesh_v0172` remise à zéro avant) ; E2E complète au dernier
commit de code. Le Dev Agent Record n'affirme que ce qui a tourné.

## Ce que les gardes ne voient pas

- La garde AC-5 ne lit que `+page.svelte` : un `catch` ajouté dans un composant extrait de l'assistant lui échappe.
- Elle vérifie qu'un appel à `errorMessageOf(`/`reportFinalizeError(` figure dans le bloc, pas que son résultat est
  affiché.
- Le motif reste en français pour DE/IT/EN (#710).
- Un pays de deux lettres hors liste ISO (`SU`, `XX`) est accepté par le serveur (issue à ouvrir, C-v017-2-8).

## Dev Notes

### Fichiers touchés (prévus) et modules

- `frontend/src/lib/shared/utils/error-message.ts` (+ `.test.ts`) — utilitaire partagé, paramètre optionnel seulement.
- `frontend/src/routes/onboarding/+page.svelte`, tests neufs `onboarding-motif.test.ts`, `onboarding-catch-guard.test.ts`.
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — une clé.
- `crates/kesh-api/tests/onboarding_path_b_e2e.rs` — un test.
- `frontend/tests/e2e/onboarding-motif.spec.ts`.
- `docs/manual/fr/user-manual.tex` (+ PDF), `CHANGELOG.md`.

Quatre modules (frontend assistant, utilitaire partagé, i18n, tests serveur) : sous le seuil de découpage.

### Le manuel (axe de revue)

`docs/manual/fr/user-manual.tex:195-204` (étapes de l'assistant en production) ; `admin-manual.tex:1069` et `:1419`
(mentions de l'assistant). Contrôler le PDF aplati, pas seulement le `.tex`.

### Références

`#809`, `#644` (classe voisine, non traitée), `#710` (refus non traduits), `#97` (KF-029). Story 28-2 (`errorMessageOf`,
`erreurs-en-texte.test.ts`). `CLAUDE.md` du worktree : *Test Locally First*, *Recette visuelle*, *Issue Tracking Rule*.

### Issues à ouvrir (par l'orchestrateur)

- **Pays contrôlé à la forme seulement** (C-v017-2-8) — bug, P2 proposé.
- **Chargement de l'assistant en échec silencieux** (`+layout.ts:33`, C-v017-2-7) — bug, P3.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie). Registre C-v017-2-1 à C-v017-2-12.
