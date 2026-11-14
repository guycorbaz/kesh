# Story 36-2 : *Éclater* — le bouton s'active quand les lignes sont complètes et équilibrées, ou l'écran dit ce qui manque (#1150)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-2`, branche
`story/36-2-eclater-grise`, par le workflow `bmad-create-story`. Release **0.17.8** (phase 1). **Aucune migration** (C-e360-3),
aucun changement serveur ni d'API. Code lu à la tête du worktree, **`dabd9045a`** (tête de la 36-1 en développement : le
correctif de la 36-1 est dans `AccountAutocomplete.svelte`). Étape 0 et choix qui font foi : registre
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-2 ; C-e360-2, -3, -4, **-5**, -11) ; choix propres à la story :
`e36-choix-autonomes-36-2.md` (C-36-2-1 à C-36-2-12). Voie de développement : **voie 2** (`gate-ci` en voie 1).
Validation P1 (Sonnet) remédiée.*

⚠️ **Base du développement : la 36-1 et la 36-3 fusionnées** (C-36-2-8). Le champ de montant se lit par
`readJournalAmount`, que la 36-3 introduit dans `journal-entries/balance.ts` (fiche `36-3-ecriture-montant-ambigu.md`,
D2) ; il n'existe pas à `dabd9045a`. La branche se refonde sur la tête de la lignée A une fois la 36-3 fusionnée, avant tout
correctif ; R0 peut s'écrire et se jouer avant.

⚠️ **Cette fiche ne tranche pas la cause de #1150.** Elle établit, par lecture, ce que chaque hypothèse prédit ; la **première
tâche du développement est une reproduction** (R0, E2E) qui la constate. Depuis la validation P1, **aucun correctif ne
dépend plus du verdict** (C-36-2-9) : quatre défauts du dialogue, établis à la lecture ou quasi certains, se corrigent sans
condition — l'attendu et l'écart effacés par le catalogue (H4), le montant envoyé en nombre JSON (H5, confirmé par la
validation), le champ `number` qui jette une saisie sans le dire (H2, retirée comme branche), la fenêtre trop étroite (H3,
#1151). R0 dit lequel des gestes reproduit ce que Guy a vu ; la section *Si R0 rend…* (après les Décisions) dit ce que fait
le développement dans chaque cas, arrêt compris.

## Story

En tant que titulaire qui rapproche son relevé bancaire, je veux, dans la fenêtre *Éclater*, que le bouton *Éclater*
s'active dès que mes lignes sont complètes et que leur somme égale le montant de la transaction — et, tant qu'il reste grisé,
que l'écran me dise ce qui manque (le compte ou le montant de quelle ligne, ou l'écart à la somme attendue) —, puis que
l'éclatement passe réellement, afin de ventiler une transaction depuis l'écran sans devoir passer par l'API.

## Ce qui se lit au code (`dabd9045a`)

### Le dialogue — `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` (459 lignes)

- Deux lignes vides à l'ouverture : `:87` `let splits = $state<SplitLine[]>([blankLine(), blankLine()]);`, remises à zéro
  par `:116-122` (`if (open && proposal) { splits = [blankLine(), blankLine()]; …`). Le montant d'une ligne est **typé
  chaîne** : `:78` `amount: string;`.
- **Condition d'activation du bouton** : `:452` `disabled={submitting || !!clientError}`. `clientError` (`:162-214`) rend
  la **première** règle violée, dans cet ordre : aucune transaction (`:163`) ; moins de 2 / plus de 50 lignes (`:168`,
  `:175`) ; puis, ligne par ligne, **compte nul** (`:184` `if (s.counterpartyAccountId === null) {` → clé
  `reconciliation-split-error-account-required`), **montant non lu ou ≤ 0** (`:191` `const n = parseFloat(s.amount);` /
  `:192` `if (!isFinite(n) || n <= 0) {` → `reconciliation-split-error-amount-positive`), description trop longue
  (`:199`) ; enfin **l'équilibre** (`:207` `if (!balanceExact) {`).
- L'équilibre : `:139` `const n = parseFloat(s.amount);`, somme arrondie au centime (`:145`
  `return Math.round(sum * 100) / 100;`), comparée à `|montant de la transaction|` (`:126-130` `txAbsAmount`, `:150-152`).
- Le champ de compte d'une ligne est **strict** (aucun `allowClear`) : `:303` `<AccountAutocomplete` / `:304`
  `accounts={filteredAccounts}` / `:305` `value={line.counterpartyAccountId}` / `:306`
  `onSelect={(id) => (splits[i].counterpartyAccountId = id)}`.
- Le champ de montant : `:311` `<Input` / `:313` `type="number"` / `:314` `step="0.01"` / `:315`
  `bind:value={splits[i].amount}`.
- L'indicateur d'équilibre emploie **la même clé pour ses deux états**, avec deux replis différents : `:405`
  `'reconciliation-split-balance-indicator',` / `:406` `` `Balance équilibrée : ${txAbsAmount}`, `` et `:410`
  `'reconciliation-split-balance-indicator',` / `:411`
  `` `Balance non équilibrée — attendu ${txAbsAmount}, écart ${balanceDiff}`, ``. Le message d'équilibre de `clientError`
  interpole lui aussi dans son seul repli, **sans arguments** : `:209` `'reconciliation-split-error-imbalance',` / `:210`
  `` `Balance non équilibrée (écart ${balanceDiff})`, ``.
- Le message de `clientError` s'affiche sous la date de valeur, **sans `data-testid` ni rôle** : `:427`
  `{:else if clientError}` / `:428` `<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">`
  / `:429` `{clientError}`.
- L'envoi : `:234` `const splitInputs: SplitProposalLine[] = splits.map((s) => ({` / `:236` `amount: s.amount,`.
- Le dialogue ne reçoit **aucune classe de largeur** : `:263` `<Dialog.Content data-testid="transaction-split-modal">`.

### Ce que ces lignes deviennent à l'exécution — trois lectures hors du fichier

1. **Le catalogue l'emporte sur le repli** : `frontend/src/lib/shared/utils/i18n.svelte.ts:20`
   `const raw = _messages[key] || fallback;`. Or le catalogue porte, en fr-CH :
   `crates/kesh-i18n/locales/fr-CH/messages.ftl:1620` `reconciliation-split-balance-indicator = Balance` et `:1621`
   `reconciliation-split-error-imbalance = L'éclatement n'équilibre pas le montant de la transaction.` (de même en de-CH
   `:1543-1544`, en-CH `:1543-1544`, it-CH `:1543-1544`). **En production, l'indicateur ne dit que « Balance », en vert ou en
   rouge, et le message ne dit ni le montant attendu ni l'écart** — les replis qui le disaient ne s'affichent jamais. Les
   tests Vitest du dialogue ne le voient pas : ils remplacent `i18nMsg` par son repli (`TransactionSplitModal.test.ts:19-22`),
   et aucun E2E n'ouvre le dialogue (ci-dessous). → **H4, établi.**
   ⚠️ **`reconciliation-split-error-imbalance` n'est pas propre au dialogue** (validation P1, V1-1) :
   `git grep -n "split-error-imbalance" -- crates frontend/src` rend aussi `crates/kesh-api/src/errors.rs:3430`
   (`"reconciliation-split-error-imbalance",` dans le `let msg = t(` de `:3429`, sans arguments : le `message` de la
   `400 RECONCILIATION_SPLIT_IMBALANCE`), `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:379` (libellé
   de l'échec partiel `RECONCILIATION_SPLIT_IMBALANCE`, `:377-380`, sans arguments) et `failed-proposal-label.test.ts:79`.
   Lui ajouter des variables changerait le message de l'API et afficherait `{ $expected }` dans l'étiquette : **elle reste
   intacte** (D3). `reconciliation-split-balance-indicator` n'a, elle, que le dialogue pour consommateur (`git grep -n
   "split-balance-indicator" -- crates frontend/src` : les quatre `.ftl`, `TransactionSplitModal.svelte:405`, `:410` ;
   `:400` et le test, `split-balance-indicator` comme `data-testid`).
2. **Un `bind:value` sur un champ `number` rend un nombre, non une chaîne.** `frontend/src/lib/components/ui/input/input.svelte:44`
   `{type}` / `:45` `bind:value` ; Svelte 5.55.1 (`frontend/node_modules/svelte/src/internal/client/dom/elements/bindings/input.js`,
   lu dans le dépôt principal, mêmes dépendances) : `:31` `value = is_numberlike_input(input) ? to_number(value) : value;`,
   `:280-282` (`var type = input.type; return type === 'number' || type === 'range';`), `:288-289`
   (`return value === '' ? null : +value;`). `splits[i].amount` vaut donc, à l'exécution, un **nombre** (`70`) ou **`null`**
   (saisie que le navigateur ne lit pas, champ vide), malgré le type `string` de `:78`. `parseFloat` s'en accommode
   (`parseFloat(null)` → `NaN` → « montant > 0 requis ») ; **l'envoi, non** : `:236` envoie `amount: 70`, nombre JSON.
3. **Le serveur refuse un montant en nombre JSON.** `crates/kesh-api/src/routes/reconciliation.rs:5261` `pub struct SplitLineInput {`
   / `:5263` `pub amount: Decimal,` ; `:5320` `Json(body): Json<SplitBody>,` (`:31` `use axum::Json;`). `rust_decimal` est
   compilé avec `serde-str` (`crates/kesh-api/Cargo.toml:47` `rust_decimal = { version = "1.41", features = ["serde-str"] }` ;
   `cargo tree -e features -i rust_decimal --offline` ne rend ni `serde-float` ni `serde-with-arbitrary-precision`), ce qui
   choisit, dans `rust_decimal-1.43.0/src/serde.rs:289-296`, `deserializer.deserialize_str(DecimalVisitor)` ; et
   `serde_json-1.0.154/src/de.rs:1535-1544` refuse tout ce qui ne commence pas par `"` (`_ => Err(self.peek_invalid_type(&visitor)),`).
   Les tests Rust envoient des chaînes (`crates/kesh-api/tests/reconciliation_split_e2e.rs:487` `"amount": "5000"`), l'issue
   dit que « le même éclatement réussit par l'API » — par une clé, donc en chaînes. **Une fois le bouton actif, l'envoi depuis
   l'écran est refusé par l'extracteur JSON** (rejet `axum`, avant le handler). → **H5, établi à la lecture, à confirmer par
   R0** : aucun test ne fait traverser la frontière HTTP à un éclatement saisi à l'écran — le Vitest `soumettre`
   (`TransactionSplitModal.test.ts:240-256`) remplace `splitTransaction` par un mock, et `tests/e2e/reconciliation-split.spec.ts`
   ne fait que vérifier l'absence du bouton sur une base vide (`:63` `await expect(page.getByTestId('split-button')).toHaveCount(0);`),
   sa propre en-tête renvoyant la couverture aux tests Rust (`:9-14`).

### Le champ de compte — `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` (à la tête : 36-1 comprise)

- **Aujourd'hui (0.17.7 + 36-1)** : au `blur`, `:411` `function handleBlur() {` → `:419` `commitDraft();` ; pour un champ
  strict, `:439` `if (!allowClear) {` / `:440` `draft = null;` / `:441` `return;` — le brouillon est **jeté**, le champ
  réaffiche `displayLabel` (vide si aucun compte n'est retenu). *Entrée* après une frappe choisit le compte désigné
  (`:375` `const designe = open ? filtered[highlightIndex] : undefined;`, l'index valant `0` dès que le brouillon est non
  blanc et la liste non vide, `:271-272`). Le dialogue *Éclater* n'est dans aucun `<form>` (fiche de la 36-1, D3).
- **En 0.17.6, version que Guy utilisait le 2026-11-13** (installée sur le NAS ce jour-là, `CLAUDE.md`) :
  `git show v0.17.6:frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte`, `handleBlur` (`:282`) commence par
  `if (!allowClear || loadError) return;` sous le commentaire « le brouillon est conservé tel quel ». **Un numéro tapé puis
  quitté par *Tab* restait affiché dans le champ, la valeur restant `null`** : la ligne paraissait complète, le bouton restait
  grisé. La 35-7 (#1000, 0.17.7) a changé ce `blur`.

### Le parent et la largeur

- `ReconciliationProposals.svelte:97-98` (`splitOpen`, `splitProposal`) ne sont écrits qu'à l'ouverture (`:256-257`) et à la
  fermeture (`:262-263`, `:826-827`) : la remise à zéro de `:116-122` ne se redéclenche pas pendant la saisie (hypothèse d'un
  effacement par le parent **écartée par lecture**).
- `frontend/src/lib/components/ui/dialog/dialog-content.svelte:31` : classes par défaut `… sm:max-w-sm fixed top-1/2 left-1/2
  … -translate-y-1/2 …`, **24rem de large**, sans `max-h` ni `overflow-y-auto`. Quatre à cinq colonnes (`:282-296`) y tiennent
  dans ≈ 22rem : c'est #1151 (« trop étroite, les champs ne se lisent pas »).

### L'écran sur lequel jouer — fixtures E2E existantes

`frontend/tests/e2e/reconciliation-manual-entry.spec.ts` monte déjà une transaction en attente : `wiredBankAccountViaApi`
(`helpers/api-fixtures.ts:223`), `createManualDebitStatementViaApi` (`:347`), montant unique par exécution, nettoyage en
`finally` (rapprochement annulé, import supprimé). Le bouton de ligne : `ReconciliationProposals.svelte:730`
`data-testid="split-button"`. Comptes du préréglage `with-company` : `crates/kesh-db/src/test_fixtures.rs:129-133`
(`1000 Caisse CI`, `1100 Banque CI`, `2000 Capital CI`, `3000 Ventes CI`, `4000 Charges CI`). Navigateur : Chromium seul,
`locale: 'fr-CH'`, fenêtre par défaut 1280×720 (`frontend/playwright.config.ts:56-59`, aucun `projects`).

### Ce que prédit chaque hypothèse

| | Geste | Prédiction à la tête (`dabd9045a`) | Prédiction en 0.17.6 | Statut |
|---|---|---|---|---|
| **H1** | compte tapé (`4000`), puis *Tab* | champ **vide**, « Ligne 1 : compte requis », bouton grisé — l'écran dit ce qui manque | champ montrant `4000`, valeur nulle, même message, bouton grisé | **à départager** (R0-a, R0-b) ; à la tête, la lecture prédit l'attendu de l'issue déjà tenu |
| **H2** | montant tapé `70,00`, `1'200.00` ou `-70.00` | si Chromium fr-CH ne lit pas la saisie : valeur `null`, « Ligne N : montant > 0 requis », pendant que le champ **montre** le texte tapé ; `-70.00` : même message, juste | idem | **constatée par R0-a**, critère écrit (C-36-2-9) ; le correctif ne dépend plus d'elle : le champ devient texte sans condition (D5) |
| **H3** | lignes remplies, fenêtre 1280×720 | valeurs tronquées dans des champs de ≈ 5rem ; message visible au-dessus du pied | idem | **quasi certaine** (≈ 22rem pour quatre à cinq colonnes ; validation P1, V1-4) : #1151 intégrée sans condition (D6) ; R0 la mesure (`scrollWidth > clientWidth`, `toBeInViewport`) |
| **H4** | somme ≠ montant | indicateur « Balance » en rouge ; message « L'éclatement n'équilibre pas le montant de la transaction. », **sans attendu ni écart** | idem | **établi** (catalogue lu) |
| **H5** | lignes complètes et équilibrées, clic sur *Éclater* | bouton actif ; requête refusée par l'extracteur JSON, message d'erreur, rien n'est passé | idem | **établi à la lecture**, confirmé par la validation P1 (chaîne relue de bout en bout) ; R0 le constate |

## Décisions

- **D1 — Reproduire d'abord, à la tête et en 0.17.6** (C-e360-5 ; C-36-2-4). La première tâche est un E2E, **R0**, qui joue
  les gestes du tableau sur une vraie transaction en attente et **asserte l'attendu** (il rougit) tout en **relevant** l'état
  observé (bouton, valeurs des champs, texte du message et de l'indicateur, mesures de largeur, capture). Il se joue à la
  tête (R0-a), puis contre l'image publiée `gcorbaz/kesh:0.17.6` (R0-b), pour savoir lequel des gestes reproduit ce que Guy a
  vu. *Écartées* : lire seulement (le comportement d'un `input number` dépend du navigateur) ; un Vitest (jsdom ne reproduit
  ni la saisie d'un `input number` ni la mise en page).
  **Localisateurs de R0** (C-36-2-10, validation P1 V1-3) : R0 se joue avant D4 et contre 0.17.6 ; il ne s'appuie donc que
  sur des `data-testid` présents aux deux versions — `split-button`, `split-row-N`, `split-amount-N`, `split-description-N`,
  `split-submit`, `split-balance-indicator`, `transaction-split-modal` — et sur les rôles existants (`getByRole('option')`
  dans la liste du champ de compte, `input[aria-autocomplete="list"]` dans une ligne). Le message de `clientError`, sans
  `testid` à la tête ni en 0.17.6, se localise **par son texte** dans le dialogue :
  `modal.getByText(/compte requis|montant > 0 requis|n'équilibre pas|Balance non équilibrée/)` (textes du catalogue fr-CH
  `:1621`, `:1848-1849` et des replis) ; le relevé `message=…` copie ce texte, ou `aucun`. Le passage à
  `getByTestId('split-client-error')` se fait **au commit de D4**, dans R0 (b), (e), (f) comme dans les tests neufs. Pour
  le champ de montant, `pressSequentially` (et non `fill`, qui refuse un texte non numérique sur `input[type=number]`).
- **D2 — H5 se corrige sans condition** (C-36-2-2, C-36-2-8) : le montant part en **chaîne décimale normalisée, écrite avec
  un point**, jamais en nombre JSON — la charge (`payload`) que rend `readJournalAmount` (D5). Le type `SplitLine.amount`
  dit alors vrai : le champ texte porte une chaîne. *Écartée* : ouvrir une issue séparée — l'attendu de #1150 n'est pas
  atteint tant que le bouton actif mène à un refus.
- **D3 — H4 se corrige sans condition, par des clés propres au dialogue** (C-36-2-1, C-36-2-7, qui remplace C-36-2-6) :
  le message et l'indicateur disent le montant attendu et l'écart, **depuis le catalogue**, dans les quatre locales.
  - `reconciliation-split-balance-indicator` (seul consommateur : le dialogue) garde l'état équilibré et reçoit
    `{ $expected }` (« Balance équilibrée : { $expected } ») ;
  - clé neuve `reconciliation-split-balance-gap` pour l'état non équilibré (« Balance non équilibrée — attendu
    { $expected }, écart { $diff } ») ;
  - clé neuve `reconciliation-split-error-imbalance-detail` pour le message de `clientError` (« L'éclatement n'équilibre
    pas le montant de la transaction : attendu { $expected }, écart { $diff }. ») ;
  - **`reconciliation-split-error-imbalance` reste intacte** dans les quatre locales, comme ses trois autres sites
    (`errors.rs:3430`, `failed-proposal-label.ts:379`, `failed-proposal-label.test.ts:79`) : le message de l'API ne change
    pas.
  Les replis deviennent des chaînes Fluent égales à la valeur fr-CH (plus de gabarit `${…}`), et les arguments sont passés.
  Toute clé dont la valeur change ou qui naît reçoit le grep de ses consommateurs, Rust et TS
  (`git grep -n "<clé>" -- crates frontend/src`), sortie copiée au Change Log ; si une garde des entrées à variables
  l'exige (`frontend/src/lib/shared/i18n-entrees-a-variables.test.ts`, listes `:196-200`, `:223-227`), les clés neuves y
  sont inscrites. Les montants s'affichent comme aujourd'hui (`txAbsAmount`, `balanceDiff`) : leur mise en forme suisse est
  hors périmètre. *Écartées* : donner des variables à `reconciliation-split-error-imbalance` (change le message de l'API,
  validation P1 V1-1) ; passer les chiffres aussi à l'API et à l'étiquette d'échec partiel (changement de contrat serveur,
  hors périmètre « aucun changement serveur ») ; garder « Balance » et compter sur la couleur (le signal ne doit pas
  reposer sur la seule couleur, `AccountAutocomplete.svelte:526-529`).
- **D4 — Le message de `clientError` reçoit `data-testid="split-client-error"`**, sans changer de place ni d'ordre des
  règles, **sans rôle `status`** (C-36-2-11) : l'indicateur voisin porte déjà `aria-live="polite"` (`:401`) et dit le
  déséquilibre ; une seconde région vivante l'annoncerait deux fois. *Écartées* : `role="status"` (double annonce,
  validation P1 V1-5) ; lister toutes les règles violées à la fois (la première suffit à dire quoi faire, et c'est la
  règle des autres dialogues).
- **D5 — Le champ de montant devient un champ texte lu par le lecteur du journal, sans condition** (C-36-2-9, qui remplace
  C-36-2-3) : `type="text"`, `inputmode="decimal"`, `amount` gardé en chaîne ; lecture par `readJournalAmount` de
  `frontend/src/lib/features/journal-entries/balance.ts` (introduite par la 36-3, D2 de sa fiche : apostrophe des milliers
  admise, ambiguïté « 1.234 » refusée), **importée, non modifiée** ; somme et comparaison en `Big` sur `value` ; envoi de
  `payload`. Messages, **une ligne à la fois, dans l'ordre des règles de `clientError`**, le compte d'abord :
  | Saisie d'une ligne | Lecture (`kind`) | Message | AC |
  |---|---|---|---|
  | vide | `empty` | « Ligne N : montant > 0 requis » (clé existante `reconciliation-split-error-amount-positive`) | AC5 a |
  | `0`, `0,00`, `0.00` | `clear`, valeur 0 | « Ligne N : montant > 0 requis » | AC5 b |
  | `-70.00` | `unreadable` (la grammaire n'a pas de signe) | « Ligne N : montant illisible … » (clé neuve, ci-dessous) | AC5 c |
  | `7O`, `70,,0`, `12'34` | `unreadable` | « Ligne N : montant illisible … » | AC5 c |
  | `1.234`, `1,234` | `ambiguous` | « Ligne N : montant ambigu … » (clé neuve, ci-dessous) | AC5 d |
  | `70,00`, `70.00`, `1'200.50` | `clear` | aucun ; compte dans la somme ; part `70.00`, `1200.50` | AC5 e |
  Clés neuves, à côté du bloc `reconciliation-split-error-*` (fr-CH `:1848-1850`) :
  `reconciliation-split-error-amount-unreadable` (fr-CH « Ligne { $line } : montant illisible — chiffres, apostrophe des
  milliers (1'234), point ou virgule décimale, quatre décimales au plus ; sans signe. ») et
  `reconciliation-split-error-amount-ambiguous` (fr-CH « Ligne { $line } : montant ambigu — un point ou une virgule suivis
  de trois chiffres peuvent séparer les milliers ; écrivez 1'234 ou 1.2340. ») ; de-CH, it-CH, en-CH : les textes des clés
  `journal-entry-form-amount-unreadable` / `-ambiguous` de la 36-3, préfixés de la ligne, et suivis de la mention du signe
  pour le premier. *Écartées* : réemployer les clés `journal-entry-form-amount-*` (détenues par `journal-entries`, lint de
  propriété ; et sans numéro de ligne) ; `-70.00` lu « montant > 0 requis » (contraire à la grammaire, validation P1
  V1-2) ; garder `type="number"` et lire `valueAsNumber` (le navigateur jette la saisie qu'il ne lit pas, et rien ne
  peut le dire) ; la grammaire du solde d'ouverture (`bank-accounts/format.ts:280`) ; une grammaire propre au dialogue.
- **D6 — #1151 entre dans la story, sans condition** (C-36-2-12, validation P1 V1-4 et V1-6) : `Dialog.Content` du
  dialogue reçoit `class="max-h-[85vh] overflow-y-auto sm:max-w-3xl"`, la convention du précédent du même dossier
  (`LinkSettlementModal.svelte:194` « Six colonnes : la largeur par défaut (`sm:max-w-sm`) les écrasait (recette
  visuelle). », `:195` `<Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-3xl" data-testid="link-settlement-modal">`),
  **dans `TransactionSplitModal.svelte` seulement** (la classe par défaut de `dialog-content.svelte` sert tous les
  dialogues : non touchée). R0 mesure avant et après ; si `sm:max-w-3xl` laisse un champ tronqué pour les valeurs de AC6,
  la largeur suivante (`sm:max-w-4xl`) est prise et dite. *Écartée* : la garder conditionnelle (la largeur se calcule,
  validation P1).
- **D7 — H1 : `AccountAutocomplete.svelte` n'est pas modifié** (C-36-2-5). À la tête, la lecture prédit que le champ se
  vide et que le message le dit, ce qu'attend l'issue ; un numéro exact tapé puis *Tab* qui choisirait le compte est #1084
  (décision ouverte, consommateurs multiples). **Si R0-a contredit la lecture** (un champ qui montre un compte tapé avec une
  valeur nulle, à la tête), le développement **s'arrête et le dit** : la fiche est à amender avant de toucher le composant
  de sept consommateurs. Si R0-b reproduit en 0.17.6 ce que R0-a ne reproduit plus, la cause de l'observation de Guy est
  H1, **déjà corrigée par la 35-7** ; la note de version le dit.

### Si R0 rend… (C-36-2-9)

| Résultat de R0 | Ce que fait le développement | #1150 dans la PR |
|---|---|---|
| H5 constatée (variante (a) : refus à l'envoi) — attendu | D2 à D6 | `closes` si un geste de R0 (à la tête ou en 0.17.6) reproduit aussi le bouton grisé de Guy ; sinon voir la dernière ligne |
| **H5 réfutée** ((a) passe à la tête) | D2 reste (le champ texte porte de toute façon une chaîne) ; le Record cite la ligne du journal et explique l'écart à la lecture ; la note de version retire « se passe de nouveau depuis l'écran » | jamais fermée sur ce motif |
| H1 reproduite en 0.17.6 seulement (R0-b (b)) | D2 à D6 ; D7 ; la note de version dit la 35-7 | `closes` |
| H2 constatée (le critère : `70,00` ou `1'200.00` tapés au clavier laissent un texte affiché et la valeur liée `null`, relevé par `inputValue()` et l'état du bouton) | rien de plus : D5 s'applique déjà ; verdict au Record | `closes` |
| H1 contredite à la tête (D7) | **arrêt** après R0, rapport à l'orchestrateur, aucun correctif | — |
| **Aucun geste ne reproduit le bouton grisé sur des lignes que Guy jugerait complètes** (ni à la tête, ni en 0.17.6, ou R0-b impossible) | **arrêt après R0 et rapport à l'orchestrateur, sans correctif d'une cause inventée** ; l'orchestrateur dit si D2 à D6 (défauts établis indépendamment de la cause) se développent. Le Record et un commentaire demandé sur #1150 écrivent que la cause n'est pas établie | `refs`, non fermée |

## Critères d'acceptation

1. **AC1 — La reproduction est faite et écrite.** R0-a (tête) et R0-b (image 0.17.6) ont tourné ; leurs journaux sont dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/r0-tete.log` et `r0-0176.log`, leurs captures dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/r0-*` ; le Dev Agent Record porte, pour chaque geste du tableau, l'état
   observé (bouton, valeur montrée, valeur envoyée ou message, mesures) et le verdict H1, H2, H3, H5 (constatée / réfutée),
   avec la ligne du journal citée, puis la ligne de *Si R0 rend…* appliquée. Si R0-b n'a pas pu tourner, le Record dit
   pourquoi et ce qui en tient lieu (la lecture de `v0.17.6` ci-dessus).
2. **AC2 — L'éclatement passe depuis l'écran (H5).** Lignes `4000` et `2000` choisies dans la liste, montants tapés au
   clavier dont la somme égale la transaction : *Éclater* est actif, le clic passe l'éclatement, la ligne quitte la liste, et
   l'écriture lue par l'API porte les deux lignes de contrepartie aux montants saisis. Le corps envoyé porte des montants
   **chaînes** normalisées (Vitest : `typeof amount === 'string'`, et `70,00` saisi part `'70.00'`).
3. **AC3 — L'équilibre se dit en chiffres (H4).** Somme ≠ montant : l'indicateur dit « Balance non équilibrée — attendu X,
   écart Y » et le message sous le tableau dit l'attendu et l'écart, **lus au catalogue** en fr-CH (E2E) ; les clés de D3
   existent dans les quatre locales avec leurs variables ; les replis égalent les valeurs fr-CH. Somme = montant :
   « Balance équilibrée : X ». **`reconciliation-split-error-imbalance` est inchangée** dans les quatre `.ftl` (aucune
   variable), et le message de la `400 RECONCILIATION_SPLIT_IMBALANCE` aussi (test Rust existant, inchangé).
4. **AC4 — Tant que le bouton est grisé, l'écran dit ce qui manque, et on le voit.** Pour chacun des gestes « compte non
   choisi », « montant vide, nul, illisible ou ambigu » et « somme fausse », à 1280×720 : `split-submit` désactivé,
   `split-client-error` visible dans la fenêtre (`toBeInViewport`) et nommant la ligne (ou l'écart).
5. **AC5 — Un montant saisi n'est jamais montré sans être lu** (D5, sans condition ; une ligne par cas de la table de D5) :
   a. ligne au compte choisi, montant **vide** → « Ligne N : montant > 0 requis » ;
   b. `0`, puis `0,00` → « Ligne N : montant > 0 requis » ;
   c. `-70.00`, puis `7O` → « Ligne N : montant illisible … » ;
   d. `1.234` → « Ligne N : montant ambigu … » ;
   e. `70,00` et `1'200.50` sont lus, comptent dans la somme, et partent `70.00` et `1200.50`.
   Dans les cas a à d, le bouton est grisé.
6. **AC6 — Les champs se lisent (#1151, sans condition).** À 1280×720, pour les valeurs de R0 (compte `4000 — Charges CI`,
   montant `1'200.50`, description de 30 caractères, colonne *Projet* présente) : aucun champ tronqué
   (`scrollWidth <= clientWidth`), pied et message dans la fenêtre ; à 1024×640, le dialogue défile au lieu de couper.
7. **AC7 (H1) — Le champ de compte ne change pas.** `AccountAutocomplete.svelte` n'est pas dans le diff ; R0-a montre, pour
   « `4000` puis *Tab* », un champ vide et « Ligne 1 : compte requis ». Une contradiction arrête la story (D7).
8. **AC8 — Aucune régression.** Les suites existantes du dialogue (`TransactionSplitModal.test.ts`, 19 `it(` à `dabd9045a`,
   recompté par `git grep -c "it(" -- frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts`),
   `ReconciliationProposals.test.ts`, `reconciliation-counterparties.test.ts` et les deux tests de
   `reconciliation-split.spec.ts`, `failed-proposal-label.test.ts` et les tests Rust de `reconciliation_split_e2e.rs`
   restent verts, assertions inchangées sauf celles que D3 ou D5 changent (les saisies `fireEvent.input` sur `split-amount-N`
   restent valides sur un champ texte), dites au Change Log.
9. **AC9 — Le manuel dit le geste** : `docs/manual/fr/user-manual.tex`, § *Éclatement d'une transaction (split)*
   (`:3221-3231`), étape 3 : « Valider » devient *Éclater*, avec la règle du bouton grisé et du message, et la forme d'un
   montant (point ou virgule décimale, apostrophe des milliers, sans signe). PDF régénéré à l'intégration (fichier d'epic,
   *Fichiers communs*).
10. **AC10 — Captures** des états touchés (bouton grisé avec chaque message ; équilibré ; succès ; largeur avant / après),
    thème clair, base de démonstration jetable, dans `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/`.

## Tests à écrire, et la preuve du rouge

Écrits **avant** le correctif ; journal `/home/gcorbaz/devel/kesh-gate-logs/e36-2/rouges.log` (sorties copiées).

- **R0 — E2E**, dans `frontend/tests/e2e/reconciliation-split.spec.ts` (un `test.describe('R0 …')`), patron de
  `reconciliation-manual-entry.spec.ts` : transaction de débit à montant unique (≥ 1 000 pour la variante à apostrophe),
  ouverture par `split-button` de sa ligne, `finally` qui nettoie ; un projet créé par l'API pour la variante (g) si le
  préréglage n'en porte pas (colonne *Projet*). Localisateurs : D1 (aucun `testid` neuf avant D4). Saisie des montants par
  `pressSequentially` — `fill` refuse un texte non numérique sur un `input[type=number]` et masquerait H2. Variantes :
  (a) comptes choisis à la souris (`dispatchEvent('mousedown')` sur l'option), montants `NN.NN` justes → AC2 ; (b) `4000`
  tapé puis *Tab* → AC4, AC7 ; (c) `70,00` ; (d) `1'200.50` → AC5 e (relevé H2 : `inputValue()` et état du bouton) ;
  (e) `-70.00`, vide, `0,00`, `1.234` → AC4, AC5 a à d ; (f) somme fausse → AC3, AC4 ; (g) mesures de largeur et de
  visibilité → AC6. Chaque variante **relève** avant d'asserter (`testInfo.attach` + `console.log` d'une ligne
  `R0 <variante> bouton=… valeurs=… message=… mesures=…`, recopiée au journal). **Rouges attendus à la tête** : (a) (H5 :
  pas de succès), (e) (`-70.00` : « montant > 0 requis » au lieu de « illisible » ; `1.234` : lu 1,234), (f) (H4), (g) (H3) ;
  (b) **vert attendu** (H1 corrigée par la 35-7) ; (c), (d) selon Chromium. R0-b rejoue `--grep R0` contre l'image 0.17.6 :
  rouges attendus (b) (champ montrant `4000`), (a), (e), (f), (g). L'en-tête de `reconciliation-split.spec.ts` (`:9-14`,
  « couverture business complète … portée par les tests E2E HTTP Rust ») est récrite dans la même tâche : elle deviendrait
  fausse.
- **Vitest** (`TransactionSplitModal.test.ts`) :
  - V-a (AC2) : dans `soumettre`, `mockApi.splitTransaction.mock.calls[0][2]` porte `amount` de type `string` (`'60'`,
    `'40'`) — **rouge** à la tête (nombres `60`, `40`, lecture 2 ci-dessus) ; et une saisie `70,00` / `30,00` part `'70.00'`
    / `'30.00'` — rouge à la tête (le champ `number` de jsdom rend `null` ou un nombre) ;
  - V-b (AC3) : somme fausse → le texte de `split-client-error` et de l'indicateur contient l'attendu (`100.00`) et l'écart
    (`-30.00`) — **rouge** pour le message de `clientError` (son repli ne dit pas l'attendu, `:210`) ; le repli étant désormais
    une chaîne Fluent, la variante mutée « arguments non passés » rend `{ $expected }` et rougit (M2) ;
  - V-c (AC4) : `split-client-error` existe et dit « Ligne 1 : compte requis » à l'ouverture — **rouge** (pas de `testid`) ;
  - V-d (AC5) : les six lignes de la table de D5, une assertion par saisie (vide, `0`, `0,00`, `-70.00`, `7O`, `1.234`,
    `70,00`, `1'200.50`) — rouges à la tête : `-70.00` (« montant > 0 »), `1.234` (lu 1,234), `1'200.50` (non lu), et la
    charge de `70,00`.
- **Catalogue** (`frontend/src/lib/features/reconciliation/split-i18n.test.ts`, `// @vitest-environment node`, patron de
  `already-booked-i18n.test.ts` et de `$lib/shared/i18n-repli-catalogue`). **Sources : le seul fichier
  `TransactionSplitModal.svelte`**, relevé par `releveDesReplis` ; l'écart n'est pris que dans le sens « repli du dialogue ≠
  valeur fr-CH » (les clés `reconciliation-split-*` du catalogue lues ailleurs — `reconciliation-split-button-label` par
  `ReconciliationProposals.svelte:733`, `reconciliation-split-error-imbalance` par `failed-proposal-label.ts` — ne font pas
  rougir). Assertions : repli = valeur fr-CH pour chaque clé du dialogue ; les clés de D3 et D5 existent dans les quatre
  locales avec leurs variables ; **`reconciliation-split-error-imbalance` ne porte aucune variable**, dans aucune locale
  (garde de V1-1) — **rouge** à la tête (`Balance` ≠ repli ; clés neuves absentes).
- **Mutations** (code commité avant ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-2/mut/` ; restauration
  `git restore --worktree -- <fichier>` puis `touch <fichier>`) :
  - M1 : la charge envoyée rendue au texte brut (`payload` → la saisie) → V-a (`70,00` part `'70,00'`), R0 (c) (refus
    serveur) ;
  - M1-bis : la charge envoyée en nombre (`Number(payload)`) → V-a (`typeof`), R0 (a) ;
  - M2 : arguments retirés de l'`i18nMsg` du déséquilibre → V-b, R0 (f) ;
  - M3 : valeur fr-CH de `reconciliation-split-error-imbalance-detail` rendue au texte de l'ancienne clé → test de
    catalogue, R0 (f) ;
  - M3-bis : `{ $diff }` ajoutée à `reconciliation-split-error-imbalance` en fr-CH → test de catalogue (garde V1-1) ;
  - M4 : `data-testid` du message retiré → V-c, R0 (b), (f) ;
  - M5 : `type="number"` rétabli → V-d, R0 (c), (d) ;
  - M6 : `kind === 'ambiguous'` traité comme `clear` → V-d (`1.234`), R0 (e) ;
  - M7 : classe de largeur retirée → R0 (g).

**E2E** : exigé — le défaut se voit dans le navigateur, et H5 ne se voit qu'à la frontière HTTP (`CLAUDE.md`, § *Un E2E
n'est pas un test comme un autre*).

## Tâches

- [ ] Étape 0 : relire à la tête les lignes citées (`TransactionSplitModal.svelte:78`, `:162-214`, `:234-236`, `:263`,
      `:303-317`, `:398-430`, `:452` ; `AccountAutocomplete.svelte:411-442` ; `messages.ftl` fr-CH `:1620-1621`) et citer au
      Change Log ; relancer les deux inventaires de *Dev Notes* et copier leur sortie.
- [ ] Bases de la voie 2 (`kesh_36_2`, `kesh_e2e_36_2`, sur `127.0.0.1:3307`) ; `node_modules` par `cp -al`.
- [ ] **R0-a** : écrire R0, build du frontend, backend de la tête sur `kesh_e2e_36_2`, `--grep R0` ; journal `r0-tete.log`,
      captures. **R0-b** : même spec contre `gcorbaz/kesh:0.17.6` (image présente sur la station) sur une base
      `kesh_e2e_36_2_v0176` neuve, port propre, sous le verrou de la voie ; journal `r0-0176.log`. Au premier essai,
      vérifier que les fixtures de la tête tiennent contre l'API 0.17.6 (`wiredBankAccountViaApi` poste
      `{ bankName, iban, qrIban, isPrimary }`, `api-fixtures.ts:234-235` ; `seedTestState` exige `KESH_TEST_MODE=true`) ; un
      échec de fixture se dit au Record et s'en tient à la lecture de `v0.17.6`. Verdicts H1, H2, H3, H5 au Record ; appliquer
      la ligne de *Si R0 rend…* ; **arrêt** dans les deux cas qu'elle nomme.
- [ ] Refonder la branche sur la lignée A avec la 36-1 **et** la 36-3 fusionnées (`readJournalAmount` présent dans
      `balance.ts`) ; relire `readJournalAmount` et ses `kind` à la tête refondée, ligne citée au Change Log.
- [ ] Vitest V-a à V-d et test de catalogue ; rouges journalisés ; commit.
- [ ] Correctifs : D2 et D5 (champ texte, `readJournalAmount`, charge en chaîne), D3 (clés ×4, replis, arguments ; ancienne
      clé intacte ; grep des consommateurs de chaque clé touchée), D4 (`testid`), D6 (largeur) ; passage de R0 à
      `split-client-error`.
- [ ] `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté à la source (C-e350-18) ; listes de
      `i18n-entrees-a-variables.test.ts` complétées si la garde l'exige.
- [ ] Verts ; mutations M1 à M7 (M1-bis, M3-bis comprises) ; captures (AC10) ; manuel (AC9).
- [ ] Gates selon `e35-dev-commun.md` § *Quels gates, quand*, voie 2 : en cours, gate ciblé — Vitest des fichiers touchés,
      `npm run check`, `npm run lint-i18n-ownership`, `cargo nextest run -E 'binary(...)'` du crate `kesh-i18n` (les `.ftl` y
      sont lus), E2E ciblé `reconciliation-split.spec.ts` ; **au passage en `review`** : bases remises à neuf,
      `scripts/test-fast.sh --ci` (voie 2), `scripts/gate-ci.sh kesh_ci_36_2` (voie 1), frontend complet ; pas d'E2E
      complet ; **à la clôture** : `gate-ci` et E2E complet sur `kesh_e2e_36_2` recréée, échecs nommés contre
      `docs/testing.md` § *Les échecs attendus*.
- [ ] Prompt de la revue P1 à côté de la fiche (trois lentilles ; manuel et PDF nommés ; captures nommées ; axes exercés et
      non exercés).

## Note de version

*Corrigé* (C-e360-11 ; l'orchestrateur l'assemble au CHANGELOG). Les phrases entre crochets dépendent de R0 (*Si R0
rend…*) : le développement garde celles que R0 confirme et retire les autres.

- **Réconciliation, *Éclater* : [l'éclatement d'une transaction se passe depuis l'écran — il était refusé au moment de
  l'envoi ;]** tant que le bouton *Éclater* reste grisé, le message sous les lignes dit ce qui manque : le compte ou le
  montant de quelle ligne, ou le montant attendu et l'écart. Un montant s'écrit avec un point ou une virgule décimale et
  l'apostrophe des milliers (1'200.50) ; un montant ambigu (1.234) ou illisible est signalé au lieu d'être lu de travers ou
  ignoré. [En 0.17.6, un numéro de compte tapé sans être choisi dans la liste restait affiché sans être retenu ; depuis la
  0.17.7, le champ se vide et le message le dit.] (refs #1150)
- **Réconciliation, *Éclater* : la fenêtre est plus large et défile** ; les comptes et les montants se lisent en entier.
  (refs #1151)

## Dev Notes

### Inventaires par le symptôme

**H4 — un texte dont seul le repli porte la valeur** (gabarit `${…}` dans le repli d'un `i18nMsg` appelé sans arguments :
le catalogue l'emporte et la valeur disparaît, ou le catalogue attend une variable que personne ne passe). Commande :

```
cd frontend && git grep -n -A3 "i18nMsg(" -- 'src/**/*.svelte' 'src/**/*.ts' | /usr/bin/grep -E '`[^`]*\$\{' | /usr/bin/grep -v '\.test\.ts'
```

Sortie (sites retenus ; les autres lignes rendues sont des clés dynamiques ou des gabarits hors repli) :
`ManualMatchModal.svelte-126` `` `Description trop longue (max ${MAX_DESCRIPTION_LEN} caractères)`, `` ;
`TransactionSplitModal.svelte-210`, `-406`, `-411` (ci-dessus) ; `GeneralLedgerView.svelte-302`,
`reports/+page.svelte-207`, `invoices/[id]/+page.svelte-991`, `settings/fiscal-years/+page.svelte-506`, `-529`, `-718`,
`-784` — ces sept derniers **passent leurs arguments** (`{ shown: …, total: … }`, `{ number: … }`, `{ name: … }`, lus aux
lignes suivantes), hors symptôme. Verdicts : `TransactionSplitModal` **dans le périmètre** (D3) ; `ManualMatchModal:124-127`
**hors périmètre** — sans arguments, contre un catalogue qui attend `{ $max }` (`fr-CH/messages.ftl:1611`), mais inatteignable
à la saisie (`ManualMatchModal.svelte:205` `maxlength={MAX_DESCRIPTION_LEN}`) : **#1179**. Le symptôme voisin — une clé
dont la valeur changerait sous d'autres consommateurs — se cherche par `git grep -n "<clé>" -- crates frontend/src` pour
chaque clé touchée (D3) ; à la rédaction : `reconciliation-split-error-imbalance` (quatre sites, laissée intacte),
`reconciliation-split-balance-indicator` (le dialogue seul). Sonde positive : la
commande rend `TransactionSplitModal.svelte-210`, cas connu. Angles morts : repli construit par concaténation (`+`) ;
`i18nMsg(` dont le repli commence au-delà de trois lignes ; textes dont le repli et le catalogue divergent **sans** gabarit
(détectables par `ecartsAuCatalogue` sur tout le frontend, non lancé ici : le test de catalogue de la story le fait pour le
seul préfixe `reconciliation-split-`).

**H5 — un `type="number"` lié par `bind:value` dont la valeur part comme chaîne.** Commande :

```
cd frontend && git grep -n -B1 -A6 'type="number"' -- 'src/**/*.svelte' | /usr/bin/grep -E 'type="number"|bind:value'
```

Sortie et verdicts : `TransactionSplitModal.svelte:313/315` (**périmètre**, D2) ; `settings/vat-rates/+page.svelte:287` et
`:331` (`bind:value={fRate}`, `let fRate = $state('');` `:43`, puis `rate: fRate.trim(),` `:124`, `:155` — `trim` sur un
nombre lève une `TypeError`, attrapée par `catch` qui affiche « La création a échoué. » `:131-134` : **création et changement
d'un taux de TVA probablement impossibles à l'écran**, à confirmer par exécution — **hors périmètre** (autre écran, TVA non
utilisée par Guy), **#1178**) ; `AutoBackupSection.svelte:361/364` (comparé en nombres, commentaire `:94-95`) ;
`BankProfileForm.svelte:226-322` (indices et nombre de lignes d'en-tête, entiers envoyés comme nombres) ;
`RuleFormModal.svelte:223/226` (`priority`, entier) ; `audit-log/+page.svelte:297/301` (`Number(entityIdValue)` `:74`) ;
`settings/dunning/+page.svelte:203`, `:277` (`Number(...)` `:99`, `:156`) — hors symptôme. Sonde positive :
`TransactionSplitModal.svelte:313`. Angle mort : un champ numérique rendu par un composant dont le `type` n'est pas écrit
littéralement au site d'appel.

**H1 — consommateurs du champ strict** : déjà inventoriés par la 36-1 (fiche 36-1, D3 : `JournalEntryForm:397`,
`VatPurchaseAssistant:159`, `:204`, `ManualMatchModal:188`, `TransactionSplitModal:303`) ; aucun n'est touché ici (D7).

### Hors périmètre, et les issues demandées

- **I1, ouverte depuis : #1178** (bug probable, à confirmer par exécution) : *Paramètres → TVA* : créer ou changer un taux
  échoue (« La création a échoué. ») — `fRate` lié à un `input number` devient un nombre, `fRate.trim()` lève
  (`vat-rates/+page.svelte:43`, `:124`, `:155`, `:287`, `:331`).
- **I2, ouverte depuis : #1179** (`technical-debt`) : `ManualMatchModal.svelte:124-127`, message « description trop longue »
  appelé sans `max` contre un catalogue qui l'attend ; inatteignable par `maxlength` (`:205`).
- Le message de la `400 RECONCILIATION_SPLIT_IMBALANCE` (`errors.rs:3429-3431`) et l'étiquette d'échec partiel
  (`failed-proposal-label.ts:377-380`) ne disent pas l'attendu ni l'écart ; les leur donner changerait le contrat de l'API
  (D3, écartée) — non demandé par l'issue, aucune issue demandée.
- #1084 (numéro exact puis *Tab*) et #1081 restent ouvertes ; #1151 entre dans la story (D6) : la PR la porte en `closes`.
- La mise en forme suisse des montants de l'indicateur (`1'200.00`) : hors périmètre, aucun texte de l'issue ne la demande.

### Fichiers communs avec les autres stories de la phase 1 (tableau du fichier d'epic)

- `AccountAutocomplete.svelte` : **non touché** (D7) — la 36-2 part de la tête de la 36-1 et n'a pas de conflit avec elle.
- `TransactionSplitModal.svelte` : 36-2 seule.
- `journal-entries/balance.ts` : **importé, non modifié** (D5) ; la 36-3 y introduit `readJournalAmount` : **la 36-2 se
  développe à partir de la 36-1 et de la 36-3 fusionnées** (C-36-2-8) — la lignée A devient 36-1 → 36-3 → 36-2 pour le
  correctif, R0 pouvant se jouer avant. Le fichier d'epic (« 36-3 peut se développer en parallèle de 36-2 ») est à
  ajuster par l'orchestrateur.
- `messages.ftl` ×4 : à côté du bloc `reconciliation-split-*` (fr-CH `:1618-1621`, `:1836-1850`), jamais en fin de fichier.
- `i18n-keys.test.ts` (`sitesTotal`) : recompté à la source, sans additionner.
- `docs/manual/fr/user-manual.tex` : § *Éclatement* (`:3221-3231`), distinct des paragraphes des 36-3, 36-4, 36-5.
- `CHANGELOG.md` : **non touché** (C-e360-11).

### Modules touchés (règle de splitting)

Production : `frontend/src/lib/features/reconciliation` (le dialogue) et `crates/kesh-i18n` (quatre `messages.ftl`) ;
documentation : `docs/manual/fr`. Tests : `frontend/tests/e2e`, `frontend/src/lib/shared` (`sitesTotal`, et peut-être
`i18n-entrees-a-variables.test.ts`). `features/journal-entries` est importé, non modifié. **Deux modules de production,
cinq au plus en tout** : aucun découpage. Aucun fichier Rust modifié (`errors.rs` et l'ancienne clé restent intacts, D3).

### Le manuel

`git grep -n -i 'clater' -- docs/manual/fr/*.tex` : `user-manual.tex:3221` (§ *Éclatement*), `:3099`, `:3102`, `:3137`,
`:3149`, `:3156`, `:4204`, `:4206` (renvois au bouton, inchangés), `admin-manual.tex:2856` (inchangé). L'étape 3 de `:3230`
dit « Valider » ; le bouton s'appelle *Éclater* et le manuel ne dit ni quand il est grisé ni où lire pourquoi : AC9.

### Ce qui doit rester vrai

- Le serveur reste juge : rien ne change à `POST /api/v1/reconciliation/split` ; le client n'envoie que des chaînes, comme
  l'API externe. Le message de la `400 RECONCILIATION_SPLIT_IMBALANCE` et l'étiquette de l'échec partiel restent ceux
  d'aujourd'hui (`reconciliation-split-error-imbalance` intacte).
- Un champ de compte strict n'est jamais nullifié et ne contredit jamais sa valeur une fois quitté (35-7, 36-1).
- Le dialogue ne propose jamais le compte de la banque (34-3) ; l'avis débiteurs / créanciers suit sa ligne.
- Aucune migration ; fixtures synthétiques seulement ; rien sous `~/kesh/`.

### Références

- Issues #1150 (P2, `bug`), #1151 (P4, `enhancement`), #1084, #1081 ; registre `e36-choix-autonomes-e360.md` (§ *Étape 0* —
  36-2, C-e360-5) ; `e36-choix-autonomes-36-2.md`.
- Story 35-7 (#1000 ; `blur` du champ strict) ; 36-1 (fiche `36-1-entree-compte-produit.md`, D1, D3) ; 34-3 (contreparties) ;
  8-5a-bis (le dialogue) ; 28-2 (tests de soumission).
- `CLAUDE.md` : § *Recette visuelle* ; § *Un E2E n'est pas un test comme un autre* ; § *Le prompt d'une passe doit NOMMER le
  manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-14 — Création de la fiche (Opus 5.5), code lu à `dabd9045a`. Cause non tranchée (C-e360-5) : cinq hypothèses, dont
  deux établies à la lecture (H4 : le catalogue efface l'attendu et l'écart ; H5 : le montant part en nombre JSON, refusé par
  `serde-str`) et trois à départager par la reproduction R0, à la tête et en 0.17.6. Choix C-36-2-1 à C-36-2-6. Issues
  demandées : I1 (TVA, `fRate.trim`), I2 (`ManualMatchModal`, `{ $max }`).
- 2026-11-14 — Validation P1 (Sonnet) : 0/1/3/2 → remédiés (Opus 5.5). Prémisses relues au code avant application :
  `errors.rs:3429-3431` et `failed-proposal-label.ts:377-380` (consommateurs sans arguments de
  `reconciliation-split-error-imbalance`) ; `LinkSettlementModal.svelte:194-195` ; `readJournalAmount` dans la fiche de la
  36-3 (`kesh-36-3`, `a78115b69`, D2) ; `api-fixtures.ts:234-235` ; `ReconciliationProposals.svelte:733`.
  - **V1-1 (HIGH)** : clé neuve `reconciliation-split-error-imbalance-detail` pour le message du dialogue ; l'ancienne
    intacte dans les quatre locales ; garde au test de catalogue (M3-bis) ; grep des consommateurs de toute clé touchée
    (D3, C-36-2-7).
  - **V1-2, V1-4 (MEDIUM)** : le champ de montant passe sans condition au texte, lu par `readJournalAmount` de la 36-3,
    charge en chaîne normalisée ; base = 36-1 et 36-3 fusionnées (C-36-2-8, C-36-2-9). Messages tranchés : vide, `0`,
    `0,00` → « montant > 0 requis » ; `-70.00`, `7O` → « illisible » ; `1.234` → « ambigu » (D5, AC5 a à e). Section *Si
    R0 rend…* : H5 réfutée, aucune hypothèse reproduite (arrêt et rapport, `refs #1150`), H1 contredite (arrêt). H3 jugée
    quasi certaine par la passe : #1151 intégrée sans condition (D6, C-36-2-12), sur le précédent `LinkSettlementModal`.
  - **V1-3 (MEDIUM)** : localisateurs de R0 sans D4 (`testid` présents aux deux versions, message par son texte), passage
    à `split-client-error` au commit de D4 (D1, C-36-2-10).
  - **V1-5 (LOW)** : clause « après fermeture et réouverture » retirée d'AC4 (rien ne l'exerçait) ; `role="status"`
    retiré de D4 (double annonce avec l'indicateur `aria-live`, C-36-2-11) ; mutations récrites pour la seule branche qui
    reste (M1, M1-bis) ; sources du test de catalogue bornées au dialogue, écart dans un seul sens.
  - **V1-6 (LOW)** : D6 aligné sur `LinkSettlementModal.svelte:195` (`max-h-[85vh]`) ; en-tête de
    `reconciliation-split.spec.ts:9-14` à récrire avec R0 ; vérification des fixtures contre 0.17.6 au premier essai de R0-b.
  - I1 et I2 sont ouvertes depuis : #1178, #1179.
