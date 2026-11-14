# Story 36-2 : *Éclater* — le bouton s'active quand les lignes sont complètes et équilibrées, ou l'écran dit ce qui manque (#1150)

Status: review

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-2`, branche
`story/36-2-eclater-grise`, par le workflow `bmad-create-story`. Release **0.17.8** (phase 1). **Aucune migration** (C-e360-3),
aucun changement serveur ni d'API. Code lu à la tête du worktree, **`dabd9045a`** (tête de la 36-1 en développement : le
correctif de la 36-1 est dans `AccountAutocomplete.svelte`). Étape 0 et choix qui font foi : registre
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-2 ; C-e360-2, -3, -4, **-5**, -11) ; choix propres à la story :
`e36-choix-autonomes-36-2.md` (C-36-2-1 à C-36-2-17). Voie de développement : **voie 2** (`gate-ci` en voie 1).
Validations P1 (Sonnet), P2 (Opus) et P3 (Sonnet) remédiées.*

⚠️ **Base du développement : la 36-1 et la 36-3 fusionnées** (C-36-2-8). Le champ de montant se lit par
`readJournalAmount`, que la 36-3 introduit dans `journal-entries/balance.ts` (fiche `36-3-ecriture-montant-ambigu.md`,
D2) ; il n'existe pas à `dabd9045a`. La branche se refonde sur la tête de la lignée A une fois la 36-3 fusionnée, avant tout
correctif ; R0 peut s'écrire et se jouer avant. L'ordre **36-1 → 36-3 → 36-2** est annoté au fichier d'epic par
l'orchestrateur (commit `1d9d769a5`, « docs(epic-36) : lignée A réordonnée, 36-1 → 36-3 → 36-2 », sur la branche du
kickoff, absente de ce worktree) ; C-e360-5 est annoté par l'orchestrateur au registre du kickoff.

⚠️ **Cette fiche ne tranche pas la cause de #1150.** Elle établit, par lecture, ce que chaque hypothèse prédit ; la **première
tâche du développement est une reproduction** (R0, E2E) qui la constate. Depuis la validation P1, **le contenu des
correctifs ne dépend plus du verdict** (C-36-2-9) ; depuis la validation P3, **leur mise en œuvre non plus**, sauf deux
arrêts écrits (*Si R0 rend…*, table A : H1 contredite à la tête ; R0-a impossible) ; R0-b et la reproduction du grisé de
Guy ne décident que du sort de #1150 (table B, C-36-2-15). Hors de ces deux arrêts, quatre défauts du dialogue, établis à la lecture ou quasi certains,
se corrigent sans condition — l'attendu et l'écart effacés par le catalogue (H4), le montant envoyé en nombre JSON (H5, confirmé par la
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
  `testid` à la tête ni en 0.17.6, se localise **par sa boîte**, qui le distingue de l'indicateur (porteur d'un `testid`)
  et de la région de statut du champ de compte (36-1, `AccountAutocomplete.svelte:542`, sans `rounded-md`) :
  `modal.locator('div.rounded-md.border-destructive:not([data-testid])')` (lu : `TransactionSplitModal.svelte:428`, la
  seule boîte de cette forme sans `testid`, les deux autres, `:417-419` et `:432-434`, en portent un ; l'indicateur,
`:398-400`, aussi) ; son texte est
  ensuite comparé aux textes attendus. Le relevé `message=…` copie ce texte, ou `aucun`. *Écartée* : la regex
  `getByText(/…|n'équilibre pas|Balance non équilibrée/)`, qui, après D3, trouve aussi l'indicateur (mode strict de
  Playwright en échec, validation P2 V2-6). Le passage à `getByTestId('split-client-error')` se fait **au commit de D4,
  qui précède celui de D3** (ordre imposé), dans R0 (b), (e), (f) comme dans les tests neufs. Pour le champ de montant,
  `pressSequentially` (et non `fill`, qui refuse un texte non numérique sur `input[type=number]`).
- **D2 — H5 se corrige sans condition** (C-36-2-2, C-36-2-8) : le montant part en **chaîne décimale normalisée, écrite avec
  un point, à deux décimales**, jamais en nombre JSON — `value.toFixed(2)` de la lecture `clear` de `readJournalAmount`,
  une fois la garde du centime passée (D5). Le type `SplitLine.amount`
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
  admise, ambiguïté « 1.234 » refusée), **importée, non modifiée** ; somme et comparaison en `Big` sur `value`.
  **Garde du centime, dans le dialogue** (C-36-2-13, validation P2 V2-1) : le serveur refuse plus de deux décimales
  (`crates/kesh-api/src/routes/reconciliation.rs:5369` `if s.amount.scale() > 2 {` → `:5371`
  `"splits[{idx}].amount précision invalide (max 2 décimales pour CHF)"`), alors que la grammaire du journal en admet
  quatre (fiche 36-3, D2 : `(?:[.,](\d{1,4}))?`). Une lecture `clear` est donc **acceptée si `value` égale son arrondi au
  centime** (`value.eq(value.round(2))`) — « 60.0000 » et « 0.500 » le sont, et partent `60.00`, `0.50` ; **refusée
  sinon** (« 0.005 », « 30.0050 »), avec un message propre. **Le mode d'arrondi de `big.js` (`Big.RM`) est indifférent**
  (V3-7) : pour une valeur dont la fraction n'a rien au-delà du centime, tout arrondi à deux décimales la rend inchangée ;
  pour une valeur qui a un chiffre non nul au-delà du centime, aucun mode ne la rend inchangée — `eq` ne compare donc
  jamais que l'existence de ce chiffre. L'envoi est toujours `value.toFixed(2)` (jamais `payload` brut, qui garde la
  fraction telle que saisie), et la somme et sa comparaison au montant de la transaction se font en `Big`, exactes
  (`0.10` + `0.20` égalent `0.30`, ce qu'une somme en `Number` sans arrondi manquerait, V-d). La garde vit dans le dialogue ; `readJournalAmount` n'est pas modifié.
  Messages, **une ligne à la fois, dans l'ordre des règles de `clientError`**, le compte d'abord :
  | Saisie d'une ligne | Lecture (`kind`) | Message | AC |
  |---|---|---|---|
  | vide | `empty` | « Ligne N : montant > 0 requis » (clé existante `reconciliation-split-error-amount-positive`) | AC5 a |
  | `0`, `0,00`, `0.00` | `clear`, valeur 0 | « Ligne N : montant > 0 requis » | AC5 b |
  | `-70.00` | `unreadable` (la grammaire n'a pas de signe) | « Ligne N : montant illisible … » (clé neuve, ci-dessous) | AC5 c |
  | `7O`, `70,,0`, `12'34` | `unreadable` | « Ligne N : montant illisible … » | AC5 c |
  | `1.234`, `1,234` | `ambiguous` | « Ligne N : montant ambigu … » (clé neuve, ci-dessous) | AC5 d |
  | `0.005`, `30.0050` | `clear`, fraction au-delà du centime | « Ligne N : deux décimales au plus … » (clé neuve, ci-dessous) | AC5 f |
  | `60.0000`, `0.500` | `clear`, zéros au-delà du centime | aucun ; part `60.00`, `0.50` | AC5 g |
  | `70,00`, `70.00`, `1'200.50` | `clear` | aucun ; compte dans la somme ; part `70.00`, `1200.50` | AC5 e |
  Clés neuves, à côté du bloc `reconciliation-split-error-*` (fr-CH `:1848-1850`), **sans conseil de quatre décimales
  ni exemple « 1.2340 »**, que le serveur refuserait :
  - `reconciliation-split-error-amount-unreadable` —
    fr-CH « Ligne { $line } : montant illisible — chiffres, apostrophe des milliers (1'234), point ou virgule décimale,
    deux décimales au plus, sans signe. » ;
    de-CH « Zeile { $line }: unlesbarer Betrag — Ziffern, Apostroph als Tausendertrennzeichen (1'234), Punkt oder Komma
    als Dezimaltrennzeichen, höchstens zwei Nachkommastellen, ohne Vorzeichen. » ;
    it-CH « Riga { $line }: importo illeggibile — cifre, apostrofo delle migliaia (1'234), punto o virgola decimale, al
    massimo due decimali, senza segno. » ;
    en-CH « Line { $line }: unreadable amount — digits, apostrophe for thousands (1'234), point or comma for decimals, at
    most two decimals, no sign. » ;
  - `reconciliation-split-error-amount-ambiguous` —
    fr-CH « Ligne { $line } : montant ambigu — un point ou une virgule suivis de trois chiffres peuvent séparer les
    milliers ; écrivez 1'234 pour mille deux cent trente-quatre francs. » ;
    de-CH « Zeile { $line }: mehrdeutiger Betrag — ein Punkt oder Komma mit drei folgenden Ziffern kann Tausender
    trennen; schreiben Sie 1'234 für tausendzweihundertvierunddreissig Franken. » ;
    it-CH « Riga { $line }: importo ambiguo — un punto o una virgola seguiti da tre cifre possono separare le migliaia;
    scrivete 1'234 per milleduecentotrentaquattro franchi. » ;
    en-CH « Line { $line }: ambiguous amount — a point or comma followed by three digits may separate thousands; write
    1'234 for one thousand two hundred and thirty-four francs. » ;
  - `reconciliation-split-error-amount-cents` —
    fr-CH « Ligne { $line } : deux décimales au plus — le montant se compte en centimes. » ;
    de-CH « Zeile { $line }: höchstens zwei Nachkommastellen — der Betrag wird in Rappen angegeben. » ;
    it-CH « Riga { $line }: al massimo due decimali — l'importo si esprime in centesimi. » ;
    en-CH « Line { $line }: at most two decimals — the amount is counted in cents. »
  *Écartées* (garde du centime) : refuser aussi « 60.0000 » (des zéros sans valeur, que la lecture rend exacts) ;
  laisser le serveur refuser (message technique, index à partir de 0, après un conseil contraire de l'écran) ; modifier
  `readJournalAmount` (propre au journal, qui garde quatre décimales). Réemploi sans modification vérifié dans le
  worktree de la 36-3 (`kesh-36-3`, commit de développement `0204e8e3c`, `balance.ts:78-79`
  `export type JournalAmountReading =` / `| { kind: 'clear'; value: Big; payload: string }`, `:110`
  `export function readJournalAmount(raw: string): JournalAmountReading {`, `:120-121` : `value = new Big(payload)`) : la
  garde n'a besoin que de `kind` et de `value`. *Écartées* (lecture) : réemployer les clés `journal-entry-form-amount-*` (détenues par `journal-entries`, lint de
  propriété ; et sans numéro de ligne) ; `-70.00` lu « montant > 0 requis » (contraire à la grammaire, validation P1
  V1-2) ; garder `type="number"` et lire `valueAsNumber` (le navigateur jette la saisie qu'il ne lit pas, et rien ne
  peut le dire) ; la grammaire du solde d'ouverture (`bank-accounts/format.ts:280`) ; une grammaire propre au dialogue.
- **D6 — #1151 entre dans la story, sans condition ; la largeur seule, sans défilement** (C-36-2-12, C-36-2-14 ;
  validations P1 V1-4, V1-6, P2 V2-2, P4 V4-1) : `Dialog.Content` du dialogue reçoit `class="sm:max-w-3xl"`. Le dialogue
  ne déborde la fenêtre à aucune largeur : la classe de base le fait `fixed … w-full` et le centre
  (`dialog-content.svelte:31` : `… sm:max-w-sm fixed top-1/2 left-1/2 z-50 w-full -translate-x-1/2 …`), si bien que
  `max-w` ne fait que plafonner une largeur égale à celle de la fenêtre ; entre 640 et 800 px, il perd seulement la marge
  de 1rem que garde `max-w-[calc(100%-2rem)]` sous 640 px, comme le précédent. *(La borne `min(48rem, calc(100%-2rem))`
  ajoutée à la P3 sur une prémisse fausse — « 768 px déborderaient » — est retirée, C-36-2-16 remplacé.)* La largeur
  reprise est celle du précédent du même dossier (`LinkSettlementModal.svelte:194` « Six colonnes : la largeur par défaut (`sm:max-w-sm`) les
  écrasait (recette visuelle). », `:195` `<Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-3xl" …>`), **mais
  ni `max-h` ni `overflow-y-auto`** : la liste du champ de compte est un descendant en `absolute`
  (`AccountAutocomplete.svelte:459` `<div class="relative">`, `:555`
  `class="absolute z-20 mt-1 max-h-60 w-full overflow-auto …"`), et tout ancêtre défilant la rognerait — le précédent ne
  loge aucun champ de compte, et aucun dialogue défilant n'en loge aujourd'hui (`git grep -ln "overflow-y-auto" --
  'frontend/src/**/*.svelte' | xargs /usr/bin/grep -l "AccountAutocomplete"` : vide). Défiler le seul corps des lignes
  rognerait pire (conteneur plus petit) ; rendre la liste hors du conteneur exige de modifier `AccountAutocomplete.svelte`,
  hors de la story. Le dialogue garde donc la hauteur de son contenu, comme aujourd'hui (`dialog-content.svelte:31` n'a
  ni `max-h` ni `overflow`) : avec beaucoup de lignes, son bas sort de l'écran à 1280×720 comme à la tête — limite
  préexistante, **I3, ouverte depuis : #1181** ; le seuil (nombre de lignes à partir duquel `split-submit` sort de la
  fenêtre) n'est pas estimé ici : R0 (h) le **mesure** et le Record l'écrit (V3-4). Le tout **dans
  `TransactionSplitModal.svelte` seulement** (la classe par défaut sert tous les dialogues : non touchée). R0 mesure avant
  et après ; si `sm:max-w-3xl` laisse un champ tronqué pour les valeurs de AC6, la largeur suivante (`sm:max-w-4xl`) est
  prise et dite. **Si l'assertion de la liste (AC6 b) échoue
  malgré tout** : arrêt de cette partie et rapport à l'orchestrateur, sans toucher au composant ; les autres correctifs
  continuent. *Écartées* : `max-h-[85vh] overflow-y-auto` (rogne la liste, validation P2 V2-2) ; défilement du seul corps
  des lignes ; liste en portail (modifie le composant de sept consommateurs) ; garder #1151 conditionnelle.
- **D7 — H1 : `AccountAutocomplete.svelte` n'est pas modifié** (C-36-2-5). À la tête, la lecture prédit que le champ se
  vide et que le message le dit, ce qu'attend l'issue ; un numéro exact tapé puis *Tab* qui choisirait le compte est #1084
  (décision ouverte, consommateurs multiples). **Si R0-a contredit la lecture** (un champ qui montre un compte tapé avec une
  valeur nulle, à la tête), le développement **s'arrête et le dit** : la fiche est à amender avant de toucher le composant
  de sept consommateurs. Si R0-b reproduit en 0.17.6 ce que R0-a ne reproduit plus, la cause de l'observation de Guy est
  H1, **déjà corrigée par la 35-7** ; la note de version le dit.

### Si R0 rend… (C-36-2-9, récrit en cas exclusifs à la validation P3, C-36-2-15)

R0 décide de **deux choses indépendantes**, chacune par sa propre table, lue **de haut en bas : la première ligne qui
s'applique est la seule appliquée**.

**Table A — les correctifs D2 à D6** (H4, H5, champ texte, #1151). Ils reposent sur des défauts établis à la lecture ; ils
ne dépendent **ni de R0-b, ni de la reproduction du grisé de Guy**.

| Ordre | Cas | Ce que fait le développement |
|---|---|---|
| A1 | H1 **contredite à la tête** (R0-a (b) : le champ de compte montre `4000` tapé, la valeur reste `null` — D7) | **arrêt de la story** après R0 et rapport : la fiche est à amender avant de toucher au composant de sept consommateurs |
| A2 | **R0-a impossible** (fixture refusée par la tête, backend de la tête qui ne démarre pas, dialogue qui ne s'ouvre pas) | **arrêt** et rapport, journal cité : sans R0-a, les rouges des correctifs ne sont pas constatés |
| A3 | tout autre résultat — R0-b impossible compris | **D2 à D6 développés** ; le cas de la table B s'écrit au Record |

**Table B — la cause de ce que Guy a vu, et le sort de #1150.** Elle n'a d'objet que sous A3 : sous A1 ou A2, elle
s'écrit au rapport, sans PR. Critère mécanique de « **grisé reproduit** » (validation P4, V4-3) :
- **variantes éligibles : (a) à (d) seulement** — (e), (e′), (f) à (h) jouent des saisies dont le grisé, ou l'état, est
  attendu, et n'entrent jamais dans B ;
- **montants admis : la liste fermée des saisies de ces variantes** — `NN.NN` de (a), que (b) joue aussi, `70,00` de (c),
  `1'200.50` de (d) et leurs compléments, tous positifs, à deux décimales au plus, non ambigus (ce que `readJournalAmount`
  de la 36-3 lit `clear`) ;
- **grisé reproduit** : dans une variante éligible, chaque ligne montre un compte dans son champ (`inputValue()` non vide)
  et un montant admis, la somme de ces montants égale la **valeur absolue** du montant de la transaction (une transaction
  de débit est négative ; le dialogue compare à `txAbsAmount`, `TransactionSplitModal.svelte:126-130`), et
  `split-submit` est **désactivé** ; relevé dans la ligne `R0 <variante> …` du journal.

| Ordre | Cas | Ce que dit le Record | #1150 dans la PR |
|---|---|---|---|
| B1 | grisé reproduit **à la tête** (R0-a) | la variante, sa ligne de journal ; **la même variante rejouée après les correctifs active le bouton** (sinon : arrêt et rapport, correctif incomplet) | `closes` |
| B2 | grisé reproduit **en 0.17.6 seulement** (R0-b), par (b) | cause H1, corrigée par la 35-7 (la note de version le dit) | `closes` |
| B2-bis | grisé reproduit **en 0.17.6 seulement**, par (c) ou (d) | cause H2, corrigée par D5 | `closes` |
| B2-ter | grisé reproduit **en 0.17.6 seulement**, par (a) | cause non nommée par la fiche ; rapport à l'orchestrateur | `refs`, non fermée |
| B3 | grisé reproduit **nulle part**, ou R0-b impossible sans grisé à la tête | la cause n'est pas établie ; **aucun correctif au-delà de D2 à D6** (pas de cause inventée) ; rapport à l'orchestrateur, qui demande un commentaire sur #1150 | `refs`, non fermée |

**H5 réfutée** ((a) réussit à la tête) — indépendamment des deux tables : D2 reste (le champ texte porte de toute façon une
chaîne) ; le Record cite la ligne du journal et explique l'écart à la lecture ; la note de version retire « il était refusé
au moment de l'envoi ». **H2** se constate (critère : `70,00` ou `1'200.00` tapés au clavier laissent un texte affiché, la
valeur liée `null`) sans rien changer : D5 s'applique déjà.

## Critères d'acceptation

1. **AC1 — La reproduction est faite et écrite.** R0-a (tête) et R0-b (image 0.17.6) ont tourné ; leurs journaux sont dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/r0-tete.log` et `r0-0176.log`, leurs captures dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/r0-*` ; le Dev Agent Record porte, pour chaque geste du tableau, l'état
   observé (bouton, valeur montrée, valeur envoyée ou message, mesures) et le verdict H1, H2, H3, H5 (constatée / réfutée),
   avec la ligne du journal citée, puis la ligne appliquée de chacune des tables A et B de *Si R0 rend…*, et le
   seuil mesuré de #1181. Si R0-b n'a pas pu tourner, le Record dit
   pourquoi et ce qui en tient lieu (la lecture de `v0.17.6` ci-dessus).
2. **AC2 — L'éclatement passe depuis l'écran (H5).** Lignes `4000` et `2000` choisies dans la liste, montants tapés au
   clavier dont la somme égale la transaction : *Éclater* est actif, le clic passe l'éclatement, la ligne quitte la liste, et
   l'écriture lue par l'API porte les deux lignes de contrepartie aux montants saisis. Le corps envoyé porte des montants
   **chaînes** normalisées (Vitest : `typeof amount === 'string'`, et `70,00` saisi part `'70.00'`).
3. **AC3 — L'équilibre se dit en chiffres (H4).** Somme ≠ montant : l'indicateur dit « Balance non équilibrée — attendu X,
   écart Y » et le message sous le tableau dit l'attendu et l'écart, **lus au catalogue** en fr-CH (E2E) ; les clés de D3
   existent dans les quatre locales avec leurs variables ; les replis égalent les valeurs fr-CH. Somme = montant (lignes
   complètes) : l'indicateur dit « Balance équilibrée : X », **X étant la valeur absolue du montant de la transaction**,
   sans `{ $` résiduel. **Le catalogue (la clé lue et sa valeur) se vérifie par l'E2E**, R0 (a) avant le clic ; **le Vitest
   V-e ne vérifie que le repli et ses arguments** — son mock d'`i18nMsg` ignore la clé et rend toujours le repli
   (`TransactionSplitModal.test.ts:19-21`). **`reconciliation-split-error-imbalance` est inchangée** dans les quatre `.ftl` (aucune
   variable), et le message de la `400 RECONCILIATION_SPLIT_IMBALANCE` aussi (test Rust existant, inchangé).
4. **AC4 — Tant que le bouton est grisé, l'écran dit ce qui manque, et on le voit.** Pour chacun des gestes « compte non
   choisi », « montant vide, nul, illisible, ambigu ou plus fin que le centime » et « somme fausse », à 1280×720 : `split-submit` désactivé,
   `split-client-error` visible dans la fenêtre (`toBeInViewport`) et nommant la ligne (ou l'écart).
5. **AC5 — Un montant saisi n'est jamais montré sans être lu** (D5, sans condition ; une ligne par cas de la table de D5) :
   a. ligne au compte choisi, montant **vide** → « Ligne N : montant > 0 requis » ;
   b. `0`, puis `0,00` → « Ligne N : montant > 0 requis » ;
   c. `-70.00`, puis `7O` → « Ligne N : montant illisible … » ;
   d. `1.234` → « Ligne N : montant ambigu … » ;
   e. `70,00` et `1'200.50` sont lus, comptent dans la somme, et partent `70.00` et `1200.50` ;
   f. `0.005` et `30.0050` → « Ligne N : deux décimales au plus … » ;
   g. `60.0000` est lu, compte dans la somme, et part `60.00` (le serveur l'accepte : échelle 2) ;
   h. `0.10` et `0.20` sur une transaction de `-0.30` : la somme égale le montant (calcul exact), le bouton s'active, et
      les montants partent `'0.10'` et `'0.20'`.
   Dans les cas a à d et f, le bouton est grisé. Aucune saisie qui active le bouton ne part avec plus de deux décimales.
6. **AC6 — Les champs se lisent, et la liste des comptes aussi (#1151, sans condition).** À 1280×720 :
   a. pour les valeurs de R0 (compte `4000 — Charges CI`, montant `1'200.50`, description de 30 caractères, colonne
      *Projet* présente) : aucun champ tronqué (`scrollWidth <= clientWidth`), pied et message dans la fenêtre ;
   b. six lignes ajoutées, la liste du champ de compte **ouverte sur la dernière** : sa première `option` est
      `toBeInViewport()` sans aucun défilement (ni de la page ni du dialogue), et le `Dialog.Content` ne porte ni
      `overflow-y` autre que `visible` ni `max-height` (`getComputedStyle`).
   À six lignes, R0 (h) **relève** aussi, sans l'asserter, la `boundingBox()` du `Dialog.Content` et la visibilité de
   `split-submit` (`toBeInViewport` relevé), et ajoute des lignes une à une jusqu'à ce que `split-submit` sorte de la
   fenêtre : ce **seuil mesuré** s'écrit au Dev Agent Record (#1181, V3-4).
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
   montant (point ou virgule décimale, apostrophe des milliers, deux décimales au plus, sans signe). PDF régénéré à l'intégration (fichier d'epic,
   *Fichiers communs*).
10. **AC10 — Captures** des états touchés (bouton grisé avec chaque message ; équilibré ; succès ; largeur avant / après ;
    liste des comptes ouverte sur la dernière de six lignes),
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
  (e) `-70.00`, vide, `0,00`, `1.234`, `0.005` → AC4, AC5 a à d, f ; (e′) `60.0000` + le complément → AC5 g, soumis
  jusqu'au succès (le serveur refuserait l'échelle 4) ; (f) somme fausse → AC3, AC4 ; (g) mesures de largeur et de
  visibilité → AC6 a ; (h) six lignes (*Ajouter une ligne* quatre fois), focus sur le champ de compte de la dernière, liste
  ouverte : première `option` `toBeInViewport()`, styles calculés du `Dialog.Content` relevés → AC6 b, capture ; puis
  relevé (non asserté) de la `boundingBox()` du dialogue et de la visibilité de `split-submit`, lignes ajoutées une à une
  jusqu'au seuil où `split-submit` sort de la fenêtre (seuil au Record, #1181). En (a), avant le clic, l'indicateur dit
  « Balance équilibrée : » suivi de la valeur absolue du montant de la transaction, lu au catalogue, sans `{ $` (AC3) :
  c'est l'assertion qui tue M2-ter. Chaque variante **relève** avant d'asserter (`testInfo.attach` + `console.log` d'une ligne
  `R0 <variante> bouton=… valeurs=… message=… mesures=…`, recopiée au journal). **Rouges attendus à la tête** : (a) (H5 :
  pas de succès), (e) (`-70.00` : « montant > 0 requis » au lieu de « illisible » ; `1.234` : lu 1,234 ; `0.005` : lu sans
  message), (e′) (H5, et l'échelle), (f) (H4), (g) (H3) ; (b) **vert attendu** (H1 corrigée par la 35-7) ; (h) **vert
  attendu** à la tête (aucun `overflow` sur le dialogue aujourd'hui) — il garde contre M9 ; (c), (d) selon Chromium. R0-b rejoue `--grep R0` contre l'image 0.17.6 :
  rouges attendus (b) (champ montrant `4000`), (a), (e), (f), (g). L'en-tête de `reconciliation-split.spec.ts` (`:9-14`,
  « couverture business complète … portée par les tests E2E HTTP Rust ») est récrite dans la même tâche : elle deviendrait
  fausse.
- **Vitest** (`TransactionSplitModal.test.ts`) :
  - V-a (AC2) : dans `soumettre`, `mockApi.splitTransaction.mock.calls[0][2]` porte `amount` de type `string` (`'60'`,
    `'40'`) — **rouge** à la tête (nombres `60`, `40`, lecture 2 ci-dessus) ; et une saisie `70,00` / `30,00` part `'70.00'`
    / `'30.00'` — rouge à la tête (le champ `number` de jsdom rend `null` ou un nombre) ;
  - V-b (AC3) : **les deux comptes choisis** par `choisirCompte` (`TransactionSplitModal.test.ts:222-238`, à sortir de son `describe` pour le partager) — sans quoi
    `clientError` rend « Ligne 1 : compte requis » (`:184-189`) avant l'équilibre, et le test rougirait pour une autre
    raison, avant comme après le correctif (le test existant `shows imbalance message`, `:162-181`, n'en choisit aucun) —,
    puis somme fausse → le texte de `split-client-error` et de l'indicateur contient l'attendu (`100.00`) et l'écart
    (`-30.00`) — **rouge** pour le message de `clientError` (son repli ne dit pas l'attendu, `:210`) ; le repli étant désormais
    une chaîne Fluent, la variante mutée « arguments non passés » rend `{ $expected }` et rougit (M2) ;
  - V-c (AC4) : `split-client-error` existe et dit « Ligne 1 : compte requis » à l'ouverture — **rouge** (pas de `testid`) ;
  - V-d (AC5) : les huit lignes de la table de D5, une assertion par saisie (vide, `0`, `0,00`, `-70.00`, `7O`, `1.234`,
    `0.005`, `30.0050`, `60.0000`, `70,00`, `1'200.50`), avec la charge envoyée pour les saisies claires (`60.0000` part
    `'60.00'`) ; et le cas de somme exacte (V4-4) : `0.10` et `0.20` sur une transaction de `-0.30` → bouton actif,
    charges `'0.10'` et `'0.20'` — rouges à la tête : `-70.00` (« montant > 0 »), `1.234` (lu 1,234), `0.005` (aucun
    message), `1'200.50` (non lu), et les charges de `70,00`, de `60.0000`, de `0.10` (nombres JSON) ; le cas `0.10` +
    `0.20` est **vert** à la tête pour le bouton (l'arrondi au centime de `:145` y égalise la somme) et garde contre
    M8-bis ;
  - V-e (AC3, état équilibré, validation P3 V3-2) : les deux comptes choisis, montants `60` et `40` sur une transaction de
    `-100.00` → `split-balance-indicator` contient « Balance équilibrée » **et** `100.00`, et ne contient pas `{ $` —
    **vert** à la tête pour le texte (le repli en gabarit y met le montant) ; il garde contre M2-bis, où le repli, devenu
    chaîne Fluent, s'afficherait brut sans arguments.
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
  - M1 : la charge envoyée rendue au texte brut (`value.toFixed(2)` → la saisie) → V-a (`70,00` part `'70,00'`) ;
  - M1-bis : la charge envoyée en nombre (`Number(value)`) → V-a (`typeof`), R0 (a) ;
  - M1-ter : la charge envoyée en `payload` au lieu de `value.toFixed(2)` → V-d (`60.0000` part `'60.0000'`), R0 (e′)
    (refus du serveur, échelle 4) ;
  - M2 : arguments retirés de l'`i18nMsg` du déséquilibre → V-b, R0 (f) ;
  - M2-bis : arguments retirés de l'`i18nMsg` de l'état équilibré (`reconciliation-split-balance-indicator`) → V-e,
    R0 (a) (avant le clic) ;
  - M2-ter : l'état équilibré lit la clé `reconciliation-split-balance-gap` → **R0 (a)** avant le clic (le catalogue y
    rend « Balance non équilibrée — … ») ; le Vitest V-e n'y peut rien, son mock d'`i18nMsg` ignorant la clé
    (`TransactionSplitModal.test.ts:19-21`) ;
  - M8-bis : la somme et sa comparaison faites en `Number`, sans arrondi (`Number(a) + Number(b) === Math.abs(Number(tx))`),
    au lieu de `Big` → V-d, cas `0.10` + `0.20` sur `-0.30` : `0.1 + 0.2` vaut `0.30000000000000004`, le bouton reste
    grisé ;
  - M3 : valeur fr-CH de `reconciliation-split-error-imbalance-detail` rendue au texte de l'ancienne clé → test de
    catalogue, R0 (f) ;
  - M3-bis : `{ $diff }` ajoutée à `reconciliation-split-error-imbalance` en fr-CH → test de catalogue (garde V1-1) ;
  - M4 : `data-testid` du message retiré → V-c, R0 (b), (f) ;
  - M5 : `type="number"` rétabli → V-d, R0 (c), (d) ;
  - M6 : `kind === 'ambiguous'` traité comme `clear` → V-d (`1.234`), R0 (e) ;
  - M7 : classe de largeur retirée → R0 (g) ;
  - M8 : garde du centime retirée (toute lecture `clear` acceptée) → V-d (`0.005`, `30.0050` sans message) ;
  - M9 : `max-h-[50vh] overflow-y-auto` ajoutés au `Dialog.Content` (la forme du précédent, plus courte pour que six
    lignes la remplissent) → R0 (h) : l'`option` rognée n'est plus `toBeInViewport`. Si M9 n'est tuée que par
    l'assertion de style et non par celle de la visibilité, le dire au Record (assertion de visibilité à revoir).

**E2E** : exigé — le défaut se voit dans le navigateur, et H5 ne se voit qu'à la frontière HTTP (`CLAUDE.md`, § *Un E2E
n'est pas un test comme un autre*).

## Tâches

- [x] Étape 0 : relire à la tête les lignes citées (`TransactionSplitModal.svelte:78`, `:162-214`, `:234-236`, `:263`,
      `:303-317`, `:398-430`, `:452` ; `AccountAutocomplete.svelte:411-442` ; `messages.ftl` fr-CH `:1620-1621`) et citer au
      Change Log ; relancer les deux inventaires de *Dev Notes* et copier leur sortie.
- [x] Bases de la voie 2 (`kesh_36_2`, `kesh_e2e_36_2`, sur `127.0.0.1:3307`) ; `node_modules` par `cp -al`. *(Noms
      donnés par le prompt de développement : `kesh_e362`, `kesh_e2e_e362` ; R0-b sur `kesh_r0b_e362`, conteneur
      `kesh-e362-r0b-1`, port 3363 ; backend du worktree sur 3362.)*
- [x] **R0-a** : écrire R0, build du frontend, backend de la tête sur `kesh_e2e_36_2`, `--grep R0` ; journal `r0-tete.log`,
      captures. **R0-b** : même spec contre `gcorbaz/kesh:0.17.6` sur une base `kesh_e2e_36_2_v0176` neuve de la voie 2
      (créée par `docker exec kesh-mariadb-dev2 …`, sans migration ni seed : l'image migre au démarrage, et le runner
      sème par `seedTestState`), sous le verrou de la voie ; journal `r0-0176.log`. Recette, sur le précédent du job
      `e2e-smoke` (`.github/workflows/ci.yml:414-422` : `docker run -d --name kesh-smoke --network=host -e
      KESH_TEST_MODE=true -e KESH_HOST=127.0.0.1 -e KESH_PORT=3000 -e KESH_COOKIE_SECURE=false -e DATABASE_URL=… …`) —
      le mode test refuse toute adresse d'écoute hors boucle locale, d'où `--network=host` et jamais `-p` :
      ```
      docker image ls gcorbaz/kesh:0.17.6        # présence de l'image, constatée avant tout
      docker run -d --name kesh-36-2-r0b --network=host \
        -v /tmp/kesh-voie2/e36-2-r0b-<n>:/data \
        -e KESH_TEST_MODE=true -e KESH_HOST=127.0.0.1 -e KESH_PORT=<port de la voie, libre par ss -ltnp> \
        -e KESH_COOKIE_SECURE=false \
        -e DATABASE_URL='mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e2e_36_2_v0176' \
        -e KESH_JWT_SECRET='dev-secret-at-least-32-bytes-long-for-testing' \
        -e KESH_ADMIN_USERNAME=admin -e KESH_ADMIN_PASSWORD=e2e-admin-password-12chars \
        -e KESH_INBOX_DIR=/data/inbox -e KESH_DOCUMENTS_DIR=/data/documents -e KESH_ADMIN_BACKUP_DIR=/data/backup \
        -e KESH_SMTP_HOST=smtp.invalid -e KESH_SMTP_USER=e2e -e KESH_SMTP_PASSWORD=e2e \
        -e KESH_SMTP_FROM=kesh@example.invalid \
        gcorbaz/kesh:0.17.6
      ```
      (dossiers `inbox`, `documents`, `backup` créés d'abord sous `/tmp/kesh-voie2/e36-2-r0b-<n>/`, **un dossier neuf par
      essai** (`<n>` = 1, 2, … ; V4-7 : les fichiers root d'un essai ne s'effacent pas sans `sudo`, un second essai ne doit
      pas remonter les pièces et sauvegardes du premier), sous le `TMPDIR` de la voie 2,
      **hors du worktree** (C-36-2-17, V3-6) : l'image n'a pas de directive `USER` (`git grep -n '^USER' -- Dockerfile`
      vide), le conteneur écrit donc en root, et des fichiers root dans le worktree empêcheraient `git worktree remove` ;
      `/tmp` se vide au redémarrage, et l'agent n'a pas à les supprimer. *Écartée* : `--user "$(id -u):$(id -g)"` — l'image
      n'est pas éprouvée sans root, et un refus d'écriture ailleurs que dans `/data` ferait échouer R0-b pour une raison
      sans rapport ; l'image sert son propre
      frontend, donc ni `KESH_STATIC_DIR` ni build). Runner : `KESH_BACKEND_URL=http://127.0.0.1:<port>`,
      `KESH_TEST_MODE=true`, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, `--grep R0`. Fin : `docker logs` copié au
      journal, puis `docker rm -f kesh-36-2-r0b` (son propre conteneur seulement), `DROP DATABASE kesh_e2e_36_2_v0176`. Les
      `data-testid` de D1 existent en 0.17.6 (validation P2, V2-4 : `split-button` à `ReconciliationProposals.svelte:680`,
      `aria-autocomplete="list"` et `role="option"` dans `AccountAutocomplete` 0.17.6). Au premier essai,
      vérifier que les fixtures de la tête tiennent contre l'API 0.17.6 (`wiredBankAccountViaApi` poste
      `{ bankName, iban, qrIban, isPrimary }`, `api-fixtures.ts:234-235` ; `seedTestState` exige `KESH_TEST_MODE=true`) ; un
      échec de fixture se dit au Record et s'en tient à la lecture de `v0.17.6`. Verdicts H1, H2, H3, H5 au Record ; appliquer
      la première ligne qui s'applique de chacune des tables A et B de *Si R0 rend…* ; **arrêt** dans les deux cas A1, A2.
      Seuil de #1181 mesuré par R0 (h), écrit au Record.
- [x] *(Sans objet : la branche partait de `c3f722ca0`, 36-1 et 36-3 fusionnées.)* Refonder la branche sur la lignée A avec la 36-1 **et** la 36-3 fusionnées (`readJournalAmount` présent dans
      `balance.ts`) ; relire `readJournalAmount` et ses `kind` à la tête refondée, ligne citée au Change Log.
- [x] Vitest V-a à V-e et test de catalogue ; rouges journalisés ; commit.
- [x] Correctifs : D2 et D5 (champ texte, `readJournalAmount`, charge en chaîne), D3 (clés ×4, replis, arguments ; ancienne
      clé intacte ; grep des consommateurs de chaque clé touchée), D4 (`testid`), D6 (largeur) ; passage de R0 à
      `split-client-error`.
- [x] `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté à la source (C-e350-18) ; listes de
      `i18n-entrees-a-variables.test.ts` complétées si la garde l'exige.
- [x] Verts ; mutations M1 à M9 (M1-bis, M1-ter, M2-bis, M2-ter, M3-bis, M8-bis comprises) ; captures (AC10) ; manuel
      (AC9).
- [ ] *(En cours et passage en `review` faits ; clôture à venir : `gate-ci` et E2E complet.)* Gates selon `e35-dev-commun.md` § *Quels gates, quand*, voie 2 : en cours, gate ciblé — Vitest des fichiers touchés,
      `npm run check`, `npm run lint-i18n-ownership`, `cargo nextest run -E 'binary(...)'` du crate `kesh-i18n` (les `.ftl` y
      sont lus), E2E ciblé `reconciliation-split.spec.ts` ; **au passage en `review`** : bases remises à neuf,
      `scripts/test-fast.sh --ci` (voie 2), `scripts/gate-ci.sh kesh_ci_36_2` (voie 1), frontend complet ; pas d'E2E
      complet ; **à la clôture** : `gate-ci` et E2E complet sur `kesh_e2e_36_2` recréée, échecs nommés contre
      `docs/testing.md` § *Les échecs attendus*.
- [x] Prompt de la revue P1 à côté de la fiche (trois lentilles ; manuel et PDF nommés ; captures nommées ; axes exercés et
      non exercés).

## Note de version

*Corrigé* (C-e360-11 ; l'orchestrateur l'assemble au CHANGELOG). Les deux phrases qui dépendaient de R0 sont gardées :
R0 a constaté le refus à l'envoi à la tête et en 0.17.6 (`r0-tete.log`, `r0-0176.log` : « envoi=erreur: Erreur serveur
(422) »), et le champ de compte qui montrait `4000` sans le retenir en 0.17.6 (`r0-0176.log`, variante (b)).

- **Réconciliation, *Éclater* : l'éclatement d'une transaction se passe depuis l'écran — il était refusé au moment de
  l'envoi ;** tant que le bouton *Éclater* reste grisé, le message sous les lignes dit ce qui manque : le compte ou le
  montant de quelle ligne, ou le montant attendu et l'écart. Un montant s'écrit avec un point ou une virgule décimale,
  l'apostrophe des milliers (1'200.50) et deux décimales au plus ; un montant ambigu (1.234), illisible ou plus fin que le
  centime est signalé au lieu d'être lu de travers, ignoré ou refusé à l'envoi. En 0.17.6, un numéro de compte tapé sans
  être choisi dans la liste restait affiché sans être retenu ; depuis la 0.17.7, le champ se vide et le message le dit.
  (refs #1150)
- **Réconciliation, *Éclater* : la fenêtre est plus large** ; les comptes, les montants et les descriptions se lisent
  en entier, et la liste des comptes s'ouvre sans être coupée.
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
- **I3, ouverte depuis : #1181** : *Éclater*, beaucoup de lignes — le dialogue n'a ni hauteur maximale ni défilement
  (`dialog-content.svelte:31`, inchangé par D6), si bien qu'au-delà d'un seuil — **mesuré par R0 (h), écrit au Record et
  reporté en commentaire de #1181 par l'orchestrateur ; une lecture l'estime possible dès six lignes** — son bas (date,
  message, bouton *Éclater*) sort de l'écran, comme à la tête. Le défilement rognerait la liste du champ de compte
  (`AccountAutocomplete.svelte:555`, `absolute`) tant que celle-ci est rendue dans le flux : le remède passe par le
  composant (liste en portail ou positionnée par rapport à la fenêtre), hors de cette story.
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

`git grep -n -i 'clat' -- docs/manual/fr/*.tex` (motif `clat` et non `clater`, qui manque « Éclatement » et
`admin-manual.tex:2856` — validation P2, V2-5) : `user-manual.tex:3221` (§ *Éclatement*), `:3223`, `:3228` (« Cliquer sur
\emph{Éclater} »), `:3233`, `:3235` (le paragraphe touché et ses voisins) ; `:494`, `:508`, `:2716`, `:3055`, `:3099`,
`:3102`, `:3137`, `:3140`, `:3146`, `:3149`, `:3156`, `:3216`, `:4204`, `:4206` (renvois ou autres sujets, inchangés) ;
`admin-manual.tex:2856` (transactions déjà importées, inchangé) ; `marketing-brochure.tex:167`, `:390` (inchangés).
L'étape 3 de `:3230`
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

Opus 5.5 (agent de développement de l'Epic 36, en autonomie), worktree `kesh-36-2`, voie 2. Développement parti de
`c3f722ca0` (36-1 et 36-3 fusionnées par l'orchestrateur : `readJournalAmount` présent,
`frontend/src/lib/features/journal-entries/balance.ts:110` `export function readJournalAmount(raw: string): JournalAmountReading {`,
`:78-80` les quatre `kind`). La tâche « refonder la branche » est donc sans objet : la branche partait déjà de la lignée A.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e36-2/` (chaque journal porte son commit en première ligne) :
`r0-tete-build.log`, `r0-tete.log` et `r0-tete-h.log` (R0-a), `r0-0176.log` (R0-b), `rouges-vitest.log` et `rouges.log`,
`cible-apres-correctifs.log`, `cible-i18n-build.log`, `r0-apres.log`, `r0-apres-gh.log`, `r0-apres-2.log`,
`r0-apres-3.log`, `inventaires.log`, `grep-consommateurs-avant.log`, `mut/` (campagnes et journal par mutation) ;
scripts `e2e-36-2.sh`, `r0b-36-2.sh`, `mut/mut-36-2.py`, `mut/run-mut-36-2.sh`.

### Completion Notes List

**R0 — la reproduction (AC1).** Spec : `frontend/tests/e2e/reconciliation-split.spec.ts`, bloc `R0`, variantes (a) à (h)
écrites avant tout correctif (commit `889e574a9`, (h) corrigée en `986cf784f` : la liste du champ de la ligne 1, ouverte
au focus initial, couvrait *Ajouter une ligne* — défaut du test).

| Variante | R0-a, tête (`r0-tete.log`, `r0-tete-h.log`) | R0-b, 0.17.6 (`r0-0176.log`) |
|---|---|---|
| (a) comptes à la souris, `NN.NN` justes | bouton actif, indicateur « Balance », **envoi refusé `422`** | idem, `422` |
| (b) `4000` tapé puis *Tab* | **champ vide**, « Ligne 1 : compte requis », grisé — vert | **champ montrant `4000`**, « Ligne 1 : compte requis », grisé ; `grise_reproduit=true` |
| (c) `70,00` | Chromium affiche `70.00`, bouton actif, `422` | idem |
| (d) `1'200.50` | Chromium écarte l'apostrophe (`1200.50`), actif, `422` | idem |
| (e) `-70.00` / vide / `0,00` / `1.234` / `0.005` | « montant > 0 requis » ×3 ; `1.234` et `0.005` lus, message « L'éclatement n'équilibre pas… » | idem |
| (e′) `60.0000` | actif, `422` | idem |
| (f) somme fausse | indicateur « Balance », message sans attendu ni écart | idem |
| (g) largeur | `compte:151/72, montant:89/74, description:237/90, projet:72/64, dialogue:384x497` | idem (`151/73`…) |
| (h) six lignes, liste ouverte | `overflowY:visible, maxHeight:none`, première option visible ; **seuil #1181 : 9 lignes** | idem, 9 lignes |

Verdicts : **H1** constatée en 0.17.6 (variante (b)) et corrigée à la tête par la 35-7 — non contredite à la tête ;
**H2** réfutée sous Chromium fr-CH : Chromium lit `70,00` et lie `70.00` (le champ **affiche** « 70,00 », capture
`r0-tete-c-virgule.png` ; `input.value` vaut `70.00` et `validity.badInput` est faux), et l'apostrophe tapée est écartée
de `1'200.50` (`1200.50`) — aucun désaccord entre l'affiché et le lié, donc pas de grisé par H2 ; la réfutation ne vaut
que pour Chromium, seul navigateur des E2E. D5 s'applique tout de même (le champ texte rend la saisie telle quelle, et la
lecture la dit) *(corrigé à la remédiation de la revue P1, A5 : la phrase disait « réécrit la saisie »)* ; **H3** constatée (quatre champs tronqués à 384 px) ;
**H4** constatée (« Balance ») ; **H5** constatée (`Erreur serveur (422)` : rejet de l'extracteur JSON, montant en nombre).

**Pour la PR (revue P1, A1)** : **`closes #1150`** (table B2) et **`closes #1151`** (D6), au titre ou au corps de la
PR seulement — les commits et la note de version gardent `refs`. Après fusion : `gh issue view 1150 --json state`, et
de même pour 1151.

**Tables de *Si R0 rend…*** : table A, ligne **A3** (H1 non contredite à la tête, R0-a joué) — D2 à D6 développés.
Table B, ligne **B2** : aucune variante éligible ne reproduit le grisé à la tête (B1 non), la variante (b) le reproduit en
0.17.6 (`R0 (b) grise_reproduit=true`, `r0-0176.log`) — cause **H1, corrigée par la 35-7** ; la note de version le dit ;
**#1150 en `closes`**. À noter : en 0.17.6 comme à la tête, des lignes complètes et justes menaient au refus `422` (H5) —
une seconde cause de « l'éclatement ne passe pas depuis l'écran », corrigée par D2.

**Seuil de #1181 (V3-4)** : à 1280×720, `split-submit` sort de la fenêtre à **9 lignes**, à la tête, en 0.17.6 et après
correctifs (`r0-apres-3.log`).

**Après correctifs** : `reconciliation-split.spec.ts` entière, **11 passed** au dernier commit de code `ee9f15714`
(`r0-final.log`) ; R0 (g) : `compte:253/253, montant:141/141, description:279/279, projet:124/124, dialogue:896x497`.
*(`r0-apres-3.log`, aussi 11 passed, porte en tête `78e841d57` : il a tourné sur l'arbre de travail qui deviendrait
`4d55adbbf`, avant son commit.)*

**Gates** (passage en `review`, selon `e35-dev-commun.md` § *Quels gates, quand*) :
- en cours : Vitest ciblés (`cible-apres-correctifs.log` : 742 passés, 47 fichiers), `npm run check`,
  `lint-i18n-ownership`, `cargo fmt --check` et `cargo nextest run -p kesh-i18n` (41/41, `cible-i18n-build.log`,
  `78e841d57`), E2E ciblé `reconciliation-split.spec.ts` ;
- bases `kesh_e362` et `kesh_e2e_e362` remises à neuf avant les gates *(annoté à la remédiation P1, A3 : la remise à
  neuf a été faite par une commande dont la sortie n'a pas été journalisée — l'affirmation reste sans pièce pour ce
  passage ; à la remédiation, elle est au journal `remise-a-neuf-rem1.log`)* ;
- `scripts/test-fast.sh --ci` (voie 2) : **5445 exécutés, 5445 passés, 13 ignorés** — `test-fast-ci-review.log`, commit
  `7efe74db4` (les commits de code suivants, `95259d3db` et `ee9f15714`, ne touchent que la spec E2E et un test Vitest :
  aucun fichier Rust ni `.ftl`) ;
- frontend complet : `npm run check` 0 erreur (29 avertissements ; « aucun dans les fichiers de la story » reposait sur
  un `grep -c` non journalisé — annoté à la remédiation P1, A3 : la liste par fichier est au journal
  `check-avertissements-rem1.log`), lint vert,
  **Vitest 3598 passés (237 fichiers)**, build — `front-complet-review-2.log`, commit `ee9f15714` (la première passe,
  `front-complet-review.log` à `95259d3db`, rendait 2 rouges de la garde des libellés en dur : 92 candidates au lieu de
  91, `expectedLabel` — compteurs relevés et corrigés en `ee9f15714`) ;
- `scripts/gate-ci.sh kesh_ci_e362` (voie 1) : **5452 passés, 0 échoué, 17 ignorés** — `gate-ci-review.log`, commit
  `ee9f15714` (dernier commit de code).
- **Non lancés : l'E2E complet et le `gate-ci` de clôture** — à la clôture, au dernier commit de code.

**Gates de la remédiation de la revue P1** (dernier commit de code `46c26744f` ; script `gates-rem-36-2.sh rem1`,
chaque journal porte son commit en première ligne) :
- bases `kesh_e362` et `kesh_e2e_e362` recréées, migrées, semées (0 projet) — `remise-a-neuf-rem1.log` ;
- `scripts/test-fast.sh --ci` (voie 2 ; un `.ftl` a changé) : **5445 exécutés, 5445 passés, 13 ignorés** —
  `test-fast-ci-rem1.log` ;
- `npm run check` : 0 erreur, 29 avertissements dans cinq fichiers (`BankProfileForm.svelte` 16, `RuleFormModal.svelte`
  8, `reports/+page.svelte` 2, `design-system/+page.svelte` 2, `ManualReminderDialog.svelte` 1), **aucun dans les
  fichiers de la story** — `check-avertissements-rem1.log` ;
- lint i18n vert, **Vitest 3609 passés (237 fichiers)**, build — `front-complet-rem1.log` ;
- `reconciliation-split.spec.ts` entière : **13 passés** (dont R0 (g0) et (g390)), seuil de #1181 toujours 9 lignes —
  `r0-rem1.log` ;
- greps de propagation (`écart —`, `sans signe.`, largeurs) : `propagation-rem1.log` — aucun résidu dans la story ;
- **non lancés : `gate-ci` et E2E complet** (clôture).

**Mutations** (code commité avant, `7efe74db4` ; script `kesh-gate-logs/e36-2/mut/mut-36-2.py`, motif vérifié unique ;
restauration `git restore --worktree` puis `touch`, arbre propre constaté après chaque campagne) — campagne Vitest
(`mut/campagne-vitest.log`), campagne E2E (`mut/campagne-e2e.log`, `mut/campagne-e2e-2.log`), un journal par mutation
(`mut/<id>.log`) :

| Mutation | Tuée par |
|---|---|
| M1 (charge = saisie brute) | V-a (« 70,00 » part « 70,00 »), V-d |
| M1-bis (charge en nombre) | V-a (`typeof`), V-d |
| M1-ter (charge = `payload`) | V-a, V-d (`60.0000` part `'60.0000'`) |
| M2 (arguments du déséquilibre retirés) | V-b, garde « chaque site fournit les arguments » |
| M2-bis (arguments de l'état équilibré retirés) | V-e, même garde |
| M2-ter (état équilibré sur la clé de l'écart) | **R0 (a)** (E2E) ; aussi, contre l'attendu de la fiche, par le test de catalogue et les gardes d'entrées à variables (la clé de l'état équilibré n'est plus relevée, la clé de l'écart a deux replis) |
| M3 (valeur fr-CH de la clé neuve = ancien texte) | test de catalogue (repli ≠ fr-CH, variables absentes) |
| M3-bis (`{ $diff }` ajoutée à l'ancienne clé) | test de catalogue (V1-1), garde « mêmes variables dans les quatre locales » |
| M4 (`testid` du message retiré) | V-b, V-c, V-d |
| M5 (`type="number"` rétabli) | V-a (« 70,00 »), V-d |
| M6 (ambigu lu comme clair) | V-d (`1.234`) |
| M7 (classe de largeur retirée) | **R0 (g)** (`compte:151/95`…, dialogue 384 px) |
| M7-bis (largeur de la description seule retirée) | **équivalente** : la colonne sans largeur absorbe le reste (`description:288/288`, `mut/M7-bis.log`) |
| M7-ter (les quatre largeurs retirées) | **R0 (g)** (`description:237/206`) |
| M8 (garde du centime retirée) | V-d (`0.005`, `30.0050`) |
| M8-bis (somme et comparaison en `Number`) | V-d (`0.10` + `0.20` sur `-0.30`) |
| M9 (`max-h-[50vh] overflow-y-auto`) | **R0 (h)** — par l'assertion de **visibilité** (`viewport ratio 0`) comme par celle du style (`overflowY: auto`) ; aussi R0 (b), (e), (f), (g) (message ou pied hors de la fenêtre) |

Dix-sept mutations jouées (les quinze de la fiche, M7-bis pour la largeur de colonne de C-36-2-18, puis M7-ter quand
M7-bis s'est révélée équivalente — recompté sur le tableau) : seize tuées, une équivalente (M7-bis).

*Remédiation de la revue P1* (`mut/campagne-rem1-vitest.log`, `campagne-rem1-e2e.log`, `campagne-rem1-e2e-2.log`),
sur le code de `2248951a1` puis `46c26744f` :

| Mutation | Tuée par |
|---|---|
| M10 (`expectedLabel = txAbsAmount`, C-36-2-19 ; A6) | V-b « montant normalisé par le serveur » |
| M11 (`lte(0)` → `lt(0)` : un montant nul accepté ; A6) | V-d `0`, `0,00` |
| M12 (la branche « écart non calculable » neutralisée ; B3) | V-b « une ligne sans montant lu », V-f « montant illisible » |
| M7-bis (redéfinie : largeur de la description du jeu « avec projet ») | **équivalente** (`description:294/294`) — C-36-2-20 |
| M7-bis-b (même chose, jeu « sans projet ») | **équivalente** (`description:363/363`) |
| M7-ter (redéfinie : les deux jeux et la colonne *Projet* vidés) | R0 (g) (`description:237/206`) |
| M7-quater (`w-px` retiré de la colonne du bouton) | **survivante, sans effet mesuré** : `w-px` retiré du code (`46c26744f`) |
| M7-quinquies (jeu « avec projet » appliqué sans projet) | R0 (g0) (`colonne_bouton:173`) |

Vingt-cinq mutations jouées en tout (17 + 8, recompté sur les deux tableaux) : vingt et une tuées, quatre non tuées —
M7-bis deux fois (avant et après la remédiation) et M7-bis-b, équivalentes par construction (C-36-2-20), et M7-quater,
dont le code muté a été retiré. Aucune survivante sur du code resté en place.

**Écarts à la fiche, en cours de développement** :
- **C-36-2-18** — `sm:max-w-3xl` laissait la description tronquée (`description:237/176`, `r0-apres.log`) : D6 prévoyait
  `sm:max-w-4xl` ; il ne suffisait pas seul (partage égal des colonnes) : `sm:max-w-4xl` **et** des largeurs d'en-tête.
- **C-36-2-19** — le serveur rend le montant normalisé (`-388.2`) : l'attendu affiché passe à deux décimales, comme
  l'écart (`r0-apres-2.log`, R0 (f) : « attendu 388.2, écart -358.20 »).
- V-a attend `'60.00'` / `'40.00'` et non `'60'` / `'40'` (la fiche, aux *Tests*, écrivait les seconds) : D2 fixe
  l'envoi à `value.toFixed(2)`, et c'est D2 qui fait foi.
- R0 borne les options à la `listbox` du champ de compte de la ligne (`optionsDeCompte`) : le projet créé par (g) ajoute
  des `<option>` au sélecteur de projet, que `getByRole('option')` prenait (`r0-apres-gh.log`, (h)) — défaut du test.
- `sitesTotal` : **3044**, relevé par la garde (`expected 3044 to be 3041`), non additionné.

### File List

- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` — D2 à D6, C-36-2-18, C-36-2-19.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` — `reconciliation-split-balance-indicator`
  (valeur), clés neuves `-balance-gap`, `-error-imbalance-detail`, `-error-amount-unreadable`, `-error-amount-ambiguous`,
  `-error-amount-cents`.
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` — V-a à V-e ; `choisirCompte` partagé.
- `frontend/src/lib/features/reconciliation/split-i18n.test.ts` — neuf (catalogue).
- `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` — les six entrées de la story.
- `frontend/src/lib/shared/i18n-keys.test.ts` — `sitesTotal` 3044.
- `frontend/tests/e2e/reconciliation-split.spec.ts` — en-tête récrit, bloc `R0`.
- `docs/manual/fr/user-manual.tex` — § *Éclatement d'une transaction (split)* (AC9 ; PDF à l'intégration).
- `_bmad-output/implementation-artifacts/36-2-eclater-bouton-grise.md`, `e36-choix-autonomes-36-2.md` (C-36-2-18,
  C-36-2-19 ; titre de C-36-2-16, V5-3).

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
- 2026-11-14 — Validation P2 (Opus) : 0/0/2/4 → remédiés (Opus 5.5). Prémisses relues au code avant application :
  `reconciliation.rs:5369-5373` (refus au-delà de deux décimales) ; `balance.ts:78-79`, `:110`, `:120-121` dans le worktree
  de la 36-3 (`0204e8e3c`) ; `AccountAutocomplete.svelte:459`, `:555` ; `dialog-content.svelte:31` ;
  `.github/workflows/ci.yml:414-422` ; `TransactionSplitModal.svelte:398-400`, `:417-419`, `:428`, `:432-434` ;
  `TransactionSplitModal.test.ts:162`, `:222-238` ; `v0.17.6:ReconciliationProposals.svelte:680` ; commit `1d9d769a5`.
  - **V2-1 (MEDIUM)** : garde du centime dans le dialogue (C-36-2-13) — `clear` accepté si `value` égale son arrondi au
    centime (« 60.0000 » part `60.00`), refusé sinon avec la clé neuve `reconciliation-split-error-amount-cents` ; envoi
    toujours `value.toFixed(2)` ; textes des deux clés neuves récrits dans les quatre locales, sans « quatre décimales »
    ni « 1.2340 » ; D5 (deux lignes de table), AC5 f et g, V-d, R0 (e) et (e′), mutations M1-ter et M8.
    `readJournalAmount` réemployé sans modification.
  - **V2-2 (MEDIUM)** : D6 ne pose plus `max-h-[85vh] overflow-y-auto`, qui rognerait la liste du champ de compte ; la
    largeur seule (C-36-2-14). Défiler le seul corps des lignes rognerait pire ; une liste hors du conteneur exige de
    modifier `AccountAutocomplete.svelte`, hors de la story. AC6 b (option `toBeInViewport` sur la dernière de six
    lignes, sans défilement), R0 (h), capture, mutation M9 ; conduite si l'assertion échoue : arrêt de cette partie et
    rapport. AC6 perd « à 1024×640, le dialogue défile ». Limite préexistante (beaucoup de lignes) : issue demandée I3 *(ouverte depuis : #1181 — annoté à la validation P3)*.
  - **V2-3 (LOW)** : en-tête récrit (le contenu des correctifs ne dépend plus du verdict, leur mise en œuvre si) ; ligne
    « R0-a impossible » ; la dernière ligne de *Si R0 rend…* suspend D2 à D6 ; ordre de la lignée A cité au commit
    `1d9d769a5` du fichier d'epic ; C-e360-5 annoté par l'orchestrateur.
  - **V2-4 (LOW)** : recette de l'image 0.17.6 écrite à la tâche R0-b (`--network=host`, `KESH_HOST=127.0.0.1`, port de la
    voie, variables de la recette locale), sur `ci.yml:414-422`.
  - **V2-5 (LOW)** : inventaire du manuel refait avec le motif `clat`.
  - **V2-6 (LOW)** : V-b choisit les deux comptes ; le message se localise par sa boîte
    (`div.rounded-md.border-destructive:not([data-testid])`), et le commit de D4 précède celui de D3.
- 2026-11-14 — Validation P3 (Sonnet) : 0/0/2/5 → remédiés (Opus 5.5). Prémisses relues avant application :
  `dialog-content.svelte:31` (classe de base `max-w-[calc(100%-2rem)] … sm:max-w-sm`) ; `Dockerfile` sans `USER`
  (`git grep -n '^USER' -- Dockerfile` vide) ; #1181 ouverte (`gh issue view 1181` : « Éclater : avec beaucoup de lignes, le
  bas du dialogue (message, bouton) sort de l'écran », OPEN).
  - **V3-1 (MEDIUM)** : *Si R0 rend…* récrit en deux tables à lignes exclusives et ordonnées (première ligne applicable
    seule) : la table A (correctifs D2 à D6) n'arrête que sur H1 contredite à la tête ou R0-a impossible, et ne dépend
    ni de R0-b ni de la reproduction du grisé ; la table B décide seule de `closes` / `refs #1150`, sur un critère
    mesurable de « grisé reproduit » (chaque ligne montre un compte et un montant tapé, somme tapée = montant de la
    transaction, `split-submit` désactivé). En-tête accordé (C-36-2-15).
  - **V3-2 (MEDIUM)** : état équilibré testé — AC3 précisé, Vitest V-e, R0 (a) avant le clic, mutations M2-bis et M2-ter.
  - **V3-3 (LOW)** : I3 nommée #1181 partout (D6, *Hors périmètre*, entrée P2 annotée).
  - **V3-4 (LOW)** : R0 (h) relève la boîte du dialogue et la visibilité de `split-submit`, jusqu'au seuil mesuré, écrit
    au Record ; « une dizaine de lignes » retiré.
  - **V3-5 (LOW)**, corrigé dans la story : classe `sm:max-w-[min(48rem,calc(100%-2rem))]` (la largeur de
    `sm:max-w-3xl`, sous la borne de la classe de base) ; AC6 c (700×720), R0 (i), capture, M10 (C-36-2-16).
  - **V3-6 (LOW)** : dossier de R0-b sous `/tmp/kesh-voie2/e36-2-r0b/`, hors du worktree (C-36-2-17).
  - **V3-7 (LOW)** : indifférence du mode d'arrondi écrite à D5 ; AC5 h, cas V-d à quinze chiffres, M8-bis.
- 2026-11-14 — Validation P4 ciblée (Opus) : 0/0/3/4 → remédiés (Opus 5.5), en simplifiant. Prémisses relues :
  `dialog-content.svelte:31` (`fixed … w-full -translate-x-1/2`) ; `TransactionSplitModal.test.ts:19-21` (le mock
  d'`i18nMsg` ignore la clé) ; `TransactionSplitModal.svelte:126-130` (`txAbsAmount`).
  - **V4-1 (MEDIUM)** : retour à `sm:max-w-3xl` — le dialogue, `fixed` et `w-full`, ne déborde à aucune largeur ; la
    borne de la P3 reposait sur une prémisse fausse. Retirés : AC6 c, R0 (i), M10, la capture à 700×720 ; D6 et la note
    de version récrits ; C-36-2-16 marqué remplacé.
  - **V4-2 (MEDIUM)** : M2-ter attribuée à R0 (a) ; AC3 dit ce que vérifie le Vitest (le repli et ses arguments) et ce
    que vérifie l'E2E (le catalogue).
  - **V4-3 (MEDIUM)** : critère du grisé mécanique — variantes éligibles (a) à (d), montants admis = leur liste fermée,
    comparaison à la valeur absolue du montant de la transaction.
  - **V4-4 (LOW)** : le cas à quinze chiffres (AC5 h) remplacé par `0.10` + `0.20` sur `-0.30`, qui tue M8-bis (somme
    en `Number` sans arrondi) ; D5 accordé.
  - **V4-5 (LOW)** : B2 scindée par variante (B2, B2-bis, B2-ter « cause non nommée », `refs`) ; table B sans objet sous
    A1 ou A2 (au rapport, sans PR).
  - **V4-6 (LOW)** : C-36-2-9 annoté comme remplacé, pour la mise en œuvre, par C-36-2-15.
  - **V4-7 (LOW)** : un dossier `/tmp/kesh-voie2/e36-2-r0b-<n>/` neuf par essai de R0-b.
- 2026-11-14 — **Validation terminée en cinq passes** : P1 Sonnet 0/1/3/2 → P2 Opus 0/0/2/4 → P3 Sonnet 0/0/2/5 → P4
  ciblée Opus 0/0/3/4 → P5 ciblée Sonnet 0/0/0/3 (CRITICAL/HIGH/MEDIUM/LOW) ; close sur une passe sans rien au-dessus de
  LOW. Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e36-2/validation-p1.md` à `validation-p5-ciblee.md`.
- 2026-11-14 — **Développement (Opus 5.5)**, depuis `c3f722ca0`. **Étape 0** — les trois LOW de la P5 :
  - **V5-1** : table B, « (f) à (i) » → « (f) à (h) » (ligne lue : `- **variantes éligibles : (a) à (d) seulement** —
    (e), (e′), (f) à (i) jouent des saisies …`).
  - **V5-2** : « `NN.NN` de (a), que (b) joue aussi » (même lieu ; R0 (b) tape bien `NN.NN`).
  - **V5-3** : titre de C-36-2-16 préfixé « (retiré, voir l'annotation P4) » (ligne lue : `### C-36-2-16 — La largeur
    bornée à toute taille de fenêtre (V3-5)`).
  Lignes relues à `c3f722ca0` : `TransactionSplitModal.svelte` `:78` `amount: string;`, `:191-192` (`parseFloat`,
  `!isFinite(n) || n <= 0`), `:207-211` (`reconciliation-split-error-imbalance` sans arguments), `:236` `amount: s.amount,`,
  `:263` `<Dialog.Content data-testid="transaction-split-modal">`, `:313` `type="number"`, `:405-411` (deux replis en
  gabarit sur la même clé), `:428` (la boîte sans `testid`), `:452` `disabled={submitting || !!clientError}` — conformes à
  la fiche. `AccountAutocomplete.svelte` a bougé avec la 36-1 finale : `handleBlur` est à `:434`, `commitDraft` à `:449`,
  `if (!allowClear) {` / `draft = null;` / `return;` à `:462-464` (la fiche citait `:411-442`) — même règle. fr-CH
  `messages.ftl:1622-1623` (la fiche citait `:1620-1621`) : `reconciliation-split-balance-indicator = Balance`,
  `reconciliation-split-error-imbalance = L'éclatement n'équilibre pas le montant de la transaction.` Inventaires H4 et H5
  relancés avant et après (`inventaires.log`) : à `c3f722ca0`, H4 rend `TransactionSplitModal.svelte-210`, `-406`,
  `-411` ; après, aucun site de `TransactionSplitModal` ; H5 ne nomme plus le dialogue. Consommateurs des clés touchées,
  avant (`grep-consommateurs-avant.log`) : `reconciliation-split-balance-indicator` — les quatre `.ftl` et le dialogue
  (`:405`, `:410`) ; `reconciliation-split-error-imbalance` — `crates/kesh-api/src/errors.rs:3430`,
  `failed-proposal-label.ts:379`, `failed-proposal-label.test.ts:79`, le dialogue `:209` et les quatre `.ftl` ; les cinq
  clés neuves — aucun consommateur hors du test de catalogue. Après : `errors.rs` et `failed-proposal-label.ts` inchangés.
  **R0-a** et **R0-b** joués avant tout correctif : tables A3 et B2 (Dev Agent Record). **Rouges** : Vitest (21 sur 45,
  `rouges.log`) et R0-a (8 sur 9 : (a), (c), (d), (e), (e′), (f), (g) pour le produit ; (h) pour un défaut du test, corrigé
  et rejoué vert). Correctifs dans l'ordre imposé : D4 (`14d70feca`), D3 (`ec52cc833`), D2/D5/D6 (`78e841d57`), puis
  C-36-2-18 et C-36-2-19 (`4d55adbbf`) ; manuel (`7efe74db4`). Écarts déclarés au Dev Agent Record.
- 2026-11-15 — **Revue de code P1** (trois lentilles) : B 0/0/0/4, E 0/0/0/6, A 0/0/0/6 — rien au-dessus de LOW. Décision
  de l'orchestrateur (« Tarir la source ») : les LOW se corrigent dans la story, puis une passe ciblée relit ce seul
  correctif. **Remédiation (Opus 5.5)**, commits `2248951a1` et `46c26744f` (dernier commit de code), prémisses relues :
  - **E1 / B2** (lu : `<th class="py-1"></th>` après trois largeurs à 80 %) — deux jeux de largeurs selon la présence de
    *Projet* (C-36-2-20) ; R0 (g0) sans projet : `compte:288/288, montant:167/167, description:357/357,
    colonne_bouton:35` (`r0-rem1.log`) ; capture `r0-rem1-g0-largeur-sans-projet.png`. M7-bis reste équivalente, dit
    pourquoi (C-36-2-20) ; M7-quinquies la remplace comme témoin, tuée.
  - **E2** (lu : `w-[30%]` sans préfixe) — largeurs en `sm:` ; R0 (g390) à 390×844 : dialogue `16+358`, dans la
    fenêtre ; capture `r0-rem1-g390-telephone.png`. Les champs s'y serrent (compte 170/87, description 270/104 :
    attendu d'un tableau à cinq colonnes sur un téléphone). Constat hors story : la page sous le dialogue mesure 1280 px
    de large à 390 px (`R0 (g390) page=1280 fenetre=390`).
  - **B3** (lu : `if (sumAmounts === null || expectedBig === null) return '—';`) — clé neuve
    `reconciliation-split-balance-incomplete` (« écart non calculable »), message de déséquilibre récrit pour dire quoi
    faire (C-36-2-21), quatre locales.
  - **B1 / E5** (lu : `JOURNAL_AMOUNT_RE` sans blanc, aucun rognage) — « sans signe ni espace » dans la clé propre au
    dialogue, quatre locales, et au manuel (C-36-2-22) ; Vitest ` 70`, `70 `, `1 200.50`.
  - **B4 / E3 / E4** — R0 (d) tape U+2019 ; Vitest : `1’200.50`, quinze chiffres (lus) et seize (illisibles), crédit,
    montant de transaction illisible (`expectedBig` nul) et nul, envoi après suppression d'une ligne saisie.
  - **A4** — `griseReproduit` compare la somme affichée à la valeur absolue du montant.
  - **A6** — M10 (`expectedLabel = txAbsAmount`) tuée ; aussi M11 (`lte` → `lt`) et M12 (écart non calculable).
  - **A1** (Record : `closes #1150`, `closes #1151` pour la PR), **A2** (en-tête de la spec), **A3** (deux affirmations
    annotées, pièces aux journaux `remise-a-neuf-rem1.log` et `check-avertissements-rem1.log`), **A5** (verdict H2),
    **E6** (manuel : zéros au-delà admis, `60.0000`).
  - `sitesTotal` 3044 → **3046**, relevé par la garde. Symptôme grepé : `écart —`, `sans signe.`, `w-[3` dans
    `frontend/src`, `crates/kesh-i18n/locales`, `docs/manual/fr` (voir gates de remédiation au Record).
