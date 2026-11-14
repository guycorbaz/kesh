# Story 36-2 : *Éclater* — le bouton s'active quand les lignes sont complètes et équilibrées, ou l'écran dit ce qui manque (#1150)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-2`, branche
`story/36-2-eclater-grise`, par le workflow `bmad-create-story`. Release **0.17.8** (phase 1). **Aucune migration** (C-e360-3),
aucun changement serveur ni d'API. Code lu à la tête du worktree, **`dabd9045a`** (tête de la 36-1 en développement : le
correctif de la 36-1 est dans `AccountAutocomplete.svelte`). Étape 0 et choix qui font foi : registre
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-2 ; C-e360-2, -3, -4, **-5**, -11) ; choix propres à la story :
`e36-choix-autonomes-36-2.md` (C-36-2-1 à C-36-2-6). Voie de développement : **voie 2** (`gate-ci` en voie 1).*

⚠️ **Cette fiche ne tranche pas la cause de #1150.** Elle établit, par lecture, ce que chaque hypothèse prédit ; la **première
tâche du développement est une reproduction** (R0, E2E) qui départage ; les correctifs qui dépendent de la cause sont des AC
**conditionnels**, bornés par un critère écrit d'avance. Deux défauts du même dialogue sont en revanche **établis à la
lecture** (H4, H5, ci-dessous) : ils se corrigent sans condition, parce que sans eux l'attendu de l'issue — « le bouton
s'active […] ou l'écran dit ce qui manque » — ne peut pas être atteint.

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
| **H2** | montant tapé `70,00`, `1'200.00` ou `-70.00` | si Chromium fr-CH ne lit pas la saisie : valeur `null`, « Ligne N : montant > 0 requis », pendant que le champ **montre** le texte tapé ; `-70.00` : même message, juste | idem | **à départager** (R0-a) : le comportement du navigateur ne se lit pas au code |
| **H3** | lignes remplies, fenêtre 1280×720 | valeurs tronquées dans des champs de ≈ 5rem ; message visible au-dessus du pied | idem | **à départager** (R0-a), par mesure (`scrollWidth > clientWidth`, `toBeInViewport`) |
| **H4** | somme ≠ montant | indicateur « Balance » en rouge ; message « L'éclatement n'équilibre pas le montant de la transaction. », **sans attendu ni écart** | idem | **établi** (catalogue lu) |
| **H5** | lignes complètes et équilibrées, clic sur *Éclater* | bouton actif ; requête refusée par l'extracteur JSON, message d'erreur, rien n'est passé | idem | **établi à la lecture**, confirmé par R0 |

## Décisions

- **D1 — Reproduire d'abord, à la tête et en 0.17.6** (C-e360-5 ; C-36-2-4). La première tâche est un E2E, **R0**, qui joue
  les gestes du tableau sur une vraie transaction en attente et **asserte l'attendu** (il rougit) tout en **relevant** l'état
  observé (bouton, valeurs des champs, texte du message et de l'indicateur, mesures de largeur, capture). Il se joue à la
  tête (R0-a), puis contre l'image publiée `gcorbaz/kesh:0.17.6` (R0-b), pour savoir lequel des gestes reproduit ce que Guy a
  vu. *Écartées* : lire seulement (le comportement d'un `input number` dépend du navigateur) ; un Vitest (jsdom ne reproduit
  ni la saisie d'un `input number` ni la mise en page).
- **D2 — H5 se corrige sans condition** (C-36-2-2) : le montant part en **chaîne décimale écrite avec un point**, jamais en
  nombre JSON. Dans la branche H2 réfutée, la conversion se fait à l'envoi (`String(…)` d'un nombre fini ; `null` n'atteint
  pas l'envoi, `clientError` l'arrête) ; dans la branche H2 confirmée, le champ porte une chaîne et D5 dit laquelle part.
  Le type `SplitLine.amount` dit alors vrai. *Écartée* : ouvrir une issue séparée — l'attendu de #1150 n'est pas atteint
  tant que le bouton actif mène à un refus.
- **D3 — H4 se corrige sans condition** (C-36-2-1, C-36-2-6) : le message et l'indicateur disent le montant attendu et
  l'écart, **depuis le catalogue**, dans les quatre locales. `reconciliation-split-balance-indicator` garde l'état équilibré
  et reçoit `{ $expected }` ; une clé neuve `reconciliation-split-balance-gap` porte l'état non équilibré
  (`{ $expected }`, `{ $diff }`) ; `reconciliation-split-error-imbalance` reçoit `{ $expected }` et `{ $diff }`. Les replis
  deviennent des chaînes Fluent égales à la valeur fr-CH (plus de gabarit `` ` `` `${…}`), et les arguments sont passés. Les
  montants s'affichent comme aujourd'hui (`txAbsAmount`, `balanceDiff`) : leur mise en forme suisse est hors périmètre.
  *Écartées* : retirer la clé partagée et en créer deux (deux retraits de plus dans quatre fichiers, sans gain) ; garder
  « Balance » et compter sur la couleur (le signal ne doit pas reposer sur la seule couleur, `AccountAutocomplete.svelte`
  `:526-529`).
- **D4 — Le message de `clientError` reçoit `data-testid="split-client-error"` et `role="status"`**, sans changer de place
  ni d'ordre des règles. *Écartée* : lister toutes les règles violées à la fois (la première suffit à dire quoi faire, et
  c'est la règle des autres dialogues).
- **D5 — Branche H2 confirmée : le champ de montant devient un champ texte lu par la grammaire du journal** (C-36-2-3) :
  `type="text"`, `inputmode="decimal"`, `amount` gardé en chaîne ; validité par `isValidAmount` et lecture par `parseAmount`
  de `frontend/src/lib/features/journal-entries/balance.ts` (`:61`, `:68`), **importées, non modifiées** ; somme et
  comparaison en `Big` ; envoi de `parseAmount(raw).toString()`. Un montant que la grammaire refuse rend un message propre à la
  ligne (clé neuve `reconciliation-split-error-amount-format`, « Ligne { $line } : montant illisible — écrivez par exemple
  70.00 ») ; un montant lu mais ≤ 0 garde le message actuel. La 36-3 (#1051) change cette grammaire (« 1.234 », apostrophe,
  C-e360-6) : le dialogue en héritera à la fusion, sans grammaire propre. *Écartées* : garder `type="number"` et lire
  `valueAsNumber` (le navigateur jette toujours la saisie qu'il ne lit pas, et rien ne peut le dire) ; la grammaire du solde
  d'ouverture (`bank-accounts/format.ts:280`, propre à ce champ) ; une troisième grammaire. **Branche H2 réfutée** : le champ
  reste `number`, rien de D5 ne s'applique.
- **D6 — Branche H3 confirmée : #1151 entre dans la story** (C-e360-5) : `Dialog.Content` du dialogue reçoit une largeur
  qui laisse lire les valeurs de R0 à 1280×720 (de l'ordre de `sm:max-w-3xl`, à fixer par la mesure) et
  `max-h-[90vh] overflow-y-auto`, **dans `TransactionSplitModal.svelte` seulement** (la classe par défaut de
  `dialog-content.svelte` sert tous les dialogues : non touchée). **Branche H3 réfutée** : #1151 reste ouverte, rien ne
  change à la largeur.
- **D7 — Branche H1 : `AccountAutocomplete.svelte` n'est pas modifié** (C-36-2-5). À la tête, la lecture prédit que le champ
  se vide et que le message le dit, ce qu'attend l'issue ; un numéro exact tapé puis *Tab* qui choisirait le compte est
  #1084 (décision ouverte, consommateurs multiples). **Si R0-a contredit la lecture** (un champ qui montre un compte tapé avec
  une valeur nulle, à la tête), le développement **s'arrête et le dit** : la fiche est à amender avant de toucher le
  composant de sept consommateurs. Si R0-b reproduit en 0.17.6 ce que R0-a ne reproduit plus, la cause de l'observation de
  Guy est H1, **déjà corrigée par la 35-7** ; la note de version le dit.

## Critères d'acceptation

1. **AC1 — La reproduction est faite et écrite.** R0-a (tête) et R0-b (image 0.17.6) ont tourné ; leurs journaux sont dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/r0-tete.log` et `r0-0176.log`, leurs captures dans
   `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/r0-*` ; le Dev Agent Record porte, pour chaque geste du tableau, l'état
   observé (bouton, valeur montrée, valeur envoyée ou message, mesures) et le verdict H1, H2, H3 (confirmée / réfutée), avec la
   ligne du journal citée. Si R0-b n'a pas pu tourner, le Record dit pourquoi et ce qui en tient lieu (la lecture de
   `v0.17.6` ci-dessus).
2. **AC2 — L'éclatement passe depuis l'écran (H5).** Lignes `4000` et `2000` choisies dans la liste, montants tapés au
   clavier dont la somme égale la transaction : *Éclater* est actif, le clic passe l'éclatement, la ligne quitte la liste, et
   l'écriture lue par l'API porte les deux lignes de contrepartie aux montants saisis. Le corps envoyé porte des montants
   **chaînes** (Vitest : `typeof amount === 'string'`).
3. **AC3 — L'équilibre se dit en chiffres (H4).** Somme ≠ montant : l'indicateur dit « Balance non équilibrée — attendu X,
   écart Y » et le message sous le tableau dit l'attendu et l'écart, **lus au catalogue** en fr-CH (E2E) ; les quatre locales
   portent `{ $expected }` (et `{ $diff }` pour les deux textes du déséquilibre) ; les replis égalent les valeurs fr-CH.
   Somme = montant : « Balance équilibrée : X ».
4. **AC4 — Tant que le bouton est grisé, l'écran dit ce qui manque, et on le voit.** Pour chacun des gestes « compte non
   choisi », « montant manquant ou ≤ 0 » et « somme fausse », à 1280×720 : `split-submit` désactivé, `split-client-error`
   visible dans la fenêtre (`toBeInViewport`) et nommant la ligne (ou l'écart). Vrai aussi après une fermeture et une
   réouverture du dialogue.
5. **AC5 (si H2 confirmée) — Un montant saisi n'est jamais montré sans être lu.** `70,00` est lu 70.00 et compte dans la
   somme ; une saisie que la grammaire du journal refuse (`7O`, `70,,0`) laisse le bouton grisé avec « Ligne N : montant
   illisible … » ; `-70.00` rend « Ligne N : montant > 0 requis ». Le montant part tel que `parseAmount` le lit. *Si H2 est
   réfutée*, cet AC est sans objet, et le Record le dit avec la ligne de R0 qui le montre.
6. **AC6 (si H3 confirmée) — Les champs se lisent (#1151).** À 1280×720, pour les valeurs de R0 (compte `4000 — Charges CI`,
   montant à deux décimales, description de 30 caractères, colonne *Projet* présente) : aucun champ tronqué
   (`scrollWidth <= clientWidth`), pied et message dans la fenêtre ; à 1024×640, le dialogue défile au lieu de couper. *Si H3
   est réfutée*, sans objet, dit au Record ; #1151 reste ouverte.
7. **AC7 (H1) — Le champ de compte ne change pas.** `AccountAutocomplete.svelte` n'est pas dans le diff ; R0-a montre, pour
   « `4000` puis *Tab* », un champ vide et « Ligne 1 : compte requis ». Une contradiction arrête la story (D7).
8. **AC8 — Aucune régression.** Les suites existantes du dialogue (`TransactionSplitModal.test.ts`, 19 `it(` à `dabd9045a`,
   recompté par `git grep -c "it(" -- frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts`),
   `ReconciliationProposals.test.ts`, `reconciliation-counterparties.test.ts` et les deux tests de
   `reconciliation-split.spec.ts` restent vertes, assertions inchangées sauf celles que D3 ou D5 changent, dites au Change Log.
9. **AC9 — Le manuel dit le geste** : `docs/manual/fr/user-manual.tex`, § *Éclatement d'une transaction (split)*
   (`:3221-3231`), étape 3 : « Valider » devient *Éclater*, avec la règle du bouton grisé et du message (et, si H2, la forme
   d'un montant). PDF régénéré à l'intégration (fichier d'epic, *Fichiers communs*).
10. **AC10 — Captures** des états touchés (bouton grisé avec chaque message ; équilibré ; succès ; si H3, avant / après la
    largeur), thème clair, base de démonstration jetable, dans `/home/gcorbaz/devel/kesh-gate-logs/e36-2/captures/`.

## Tests à écrire, et la preuve du rouge

Écrits **avant** le correctif ; journal `/home/gcorbaz/devel/kesh-gate-logs/e36-2/rouges.log` (sorties copiées).

- **R0 — E2E**, dans `frontend/tests/e2e/reconciliation-split.spec.ts` (un `test.describe('R0 …')`), patron de
  `reconciliation-manual-entry.spec.ts` : transaction de débit à montant unique (≥ 1 000 pour la variante à apostrophe),
  ouverture par `split-button` de sa ligne, `finally` qui nettoie. Saisie des montants par `pressSequentially` — `fill` refuse
  un texte non numérique sur un `input[type=number]` et masquerait H2. Variantes : (a) comptes choisis à la souris
  (`dispatchEvent('mousedown')` sur l'option), montants `NN.NN` justes → AC2 ; (b) `4000` tapé puis *Tab* → AC4, AC7 ;
  (c) `70,00` ; (d) `1'200.00` ; (e) `-70.00` → AC4, AC5 ; (f) somme fausse → AC3, AC4 ; (g) mesures de largeur et de
  visibilité → AC6. Chaque variante **relève** avant d'asserter (`testInfo.attach` + `console.log` d'une ligne
  `R0 <variante> bouton=… valeurs=… message=… mesures=…`, recopiée au journal). **Rouges attendus à la tête** : (a) (H5 :
  pas de succès), (f) (H4 : pas d'attendu ni d'écart), (g) si H3 ; (b) **vert attendu** (H1 corrigée par la 35-7) ; (c), (d)
  selon Chromium. R0-b rejoue `--grep R0` contre l'image 0.17.6 : rouges attendus (b) (champ montrant `4000`), (a), (f).
- **Vitest** (`TransactionSplitModal.test.ts`) :
  - V-a (AC2) : dans `soumettre`, `mockApi.splitTransaction.mock.calls[0][2]` porte `amount` de type `string` (`'60'`,
    `'40'`) — **rouge** à la tête (nombres `60`, `40`, lecture 2 ci-dessus) ;
  - V-b (AC3) : somme fausse → le texte de `split-client-error` et de l'indicateur contient l'attendu (`100.00`) et l'écart
    (`-30.00`) — **rouge** pour le message de `clientError` (son repli ne dit pas l'attendu, `:210`) ; le repli étant désormais
    une chaîne Fluent, la variante mutée « arguments non passés » rend `{ $expected }` et rougit (M2) ;
  - V-c (AC4) : `split-client-error` existe et dit « Ligne 1 : compte requis » à l'ouverture — **rouge** (pas de `testid`) ;
  - V-d (AC5, si H2) : `70,00` lu, `7O` → « montant illisible », `-70` → « montant > 0 requis ».
- **Catalogue** (`frontend/src/lib/features/reconciliation/split-i18n.test.ts`, `// @vitest-environment node`, patron de
  `already-booked-i18n.test.ts` et de `$lib/shared/i18n-repli-catalogue`) : pour les clés `reconciliation-split-*` relevées
  dans `TransactionSplitModal.svelte`, repli = valeur fr-CH (`ecartsAuCatalogue`) ; les clés de D3 existent dans les quatre
  locales avec leurs variables — **rouge** à la tête (`Balance` ≠ repli).
- **Mutations** (code commité avant ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-2/mut/` ; restauration
  `git restore --worktree -- <fichier>` puis `touch <fichier>`) :
  - M1 : `amount: s.amount` rétabli (la conversion retirée) → V-a, R0 (a) ;
  - M2 : arguments retirés de l'`i18nMsg` du déséquilibre → V-b, R0 (f) ;
  - M3 : valeur fr-CH de `reconciliation-split-error-imbalance` rendue à son texte actuel → test de catalogue, R0 (f) ;
  - M4 : `data-testid` du message retiré → V-c, R0 (b), (f) ;
  - M5 (si H2) : `type="number"` rétabli → V-d, R0 (c) ;
  - M6 (si H3) : classe de largeur retirée → R0 (g).

**E2E** : exigé — le défaut se voit dans le navigateur, et H5 ne se voit qu'à la frontière HTTP (`CLAUDE.md`, § *Un E2E
n'est pas un test comme un autre*).

## Tâches

- [ ] Étape 0 : relire à la tête les lignes citées (`TransactionSplitModal.svelte:78`, `:162-214`, `:234-236`, `:263`,
      `:303-317`, `:398-430`, `:452` ; `AccountAutocomplete.svelte:411-442` ; `messages.ftl` fr-CH `:1620-1621`) et citer au
      Change Log ; relancer les deux inventaires de *Dev Notes* et copier leur sortie.
- [ ] Bases de la voie 2 (`kesh_36_2`, `kesh_e2e_36_2`, sur `127.0.0.1:3307`) ; `node_modules` par `cp -al`.
- [ ] **R0-a** : écrire R0, build du frontend, backend de la tête sur `kesh_e2e_36_2`, `--grep R0` ; journal `r0-tete.log`,
      captures. **R0-b** : même spec contre `gcorbaz/kesh:0.17.6` (image présente sur la station) sur une base
      `kesh_e2e_36_2_v0176` neuve, port propre, sous le verrou de la voie ; journal `r0-0176.log`. Verdicts H1, H2, H3 au
      Record ; **arrêt** si D7 l'exige.
- [ ] Vitest V-a à V-c (V-d si H2) et test de catalogue ; rouges journalisés ; commit.
- [ ] Correctifs : D2 (montant en chaîne), D3 (catalogue ×4, replis, arguments), D4 (`testid`, rôle) ; D5 si H2 ; D6 si H3.
- [ ] `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté à la source (C-e350-18).
- [ ] Verts ; mutations M1 à M4 (M5, M6 selon les branches) ; captures (AC10) ; manuel (AC9).
- [ ] Gates selon `e35-dev-commun.md` § *Quels gates, quand*, voie 2 : en cours, gate ciblé — Vitest des fichiers touchés,
      `npm run check`, `npm run lint-i18n-ownership`, `cargo nextest run -E 'binary(...)'` du crate `kesh-i18n` (les `.ftl` y
      sont lus), E2E ciblé `reconciliation-split.spec.ts` ; **au passage en `review`** : bases remises à neuf,
      `scripts/test-fast.sh --ci` (voie 2), `scripts/gate-ci.sh kesh_ci_36_2` (voie 1), frontend complet ; pas d'E2E
      complet ; **à la clôture** : `gate-ci` et E2E complet sur `kesh_e2e_36_2` recréée, échecs nommés contre
      `docs/testing.md` § *Les échecs attendus*.
- [ ] Prompt de la revue P1 à côté de la fiche (trois lentilles ; manuel et PDF nommés ; captures nommées ; axes exercés et
      non exercés).

## Note de version

*Corrigé* (C-e360-11 ; l'orchestrateur l'assemble au CHANGELOG). Les phrases entre crochets dépendent de R0 : le
développement garde celles que R0 confirme et retire les autres.

- **Réconciliation, *Éclater* : l'éclatement d'une transaction se passe de nouveau depuis l'écran**, et tant que le bouton
  *Éclater* reste grisé, le message sous les lignes dit ce qui manque : le compte ou le montant de quelle ligne, ou le
  montant attendu et l'écart. [Un montant saisi avec une virgule (70,00) est lu.] [La fenêtre est plus large : les comptes et
  les montants se lisent en entier. (refs #1151)] [En 0.17.6, un numéro de compte tapé sans être choisi dans la liste restait
  affiché sans être retenu ; depuis la 0.17.7, le champ se vide et le message le dit.] (refs #1150)

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
à la saisie (`ManualMatchModal.svelte:205` `maxlength={MAX_DESCRIPTION_LEN}`) : issue demandée **I2**. Sonde positive : la
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
utilisée par Guy), issue demandée **I1**) ; `AutoBackupSection.svelte:361/364` (comparé en nombres, commentaire `:94-95`) ;
`BankProfileForm.svelte:226-322` (indices et nombre de lignes d'en-tête, entiers envoyés comme nombres) ;
`RuleFormModal.svelte:223/226` (`priority`, entier) ; `audit-log/+page.svelte:297/301` (`Number(entityIdValue)` `:74`) ;
`settings/dunning/+page.svelte:203`, `:277` (`Number(...)` `:99`, `:156`) — hors symptôme. Sonde positive :
`TransactionSplitModal.svelte:313`. Angle mort : un champ numérique rendu par un composant dont le `type` n'est pas écrit
littéralement au site d'appel.

**H1 — consommateurs du champ strict** : déjà inventoriés par la 36-1 (fiche 36-1, D3 : `JournalEntryForm:397`,
`VatPurchaseAssistant:159`, `:204`, `ManualMatchModal:188`, `TransactionSplitModal:303`) ; aucun n'est touché ici (D7).

### Hors périmètre, et les issues demandées

- **I1** (bug probable, P2 proposé, à confirmer par exécution) : *Paramètres → TVA* : créer ou changer un taux échoue (« La
  création a échoué. ») — `fRate` lié à un `input number` devient un nombre, `fRate.trim()` lève (`vat-rates/+page.svelte:43`,
  `:124`, `:155`, `:287`, `:331`).
- **I2** (`technical-debt`, P4) : `ManualMatchModal.svelte:124-127`, message « description trop longue » appelé sans `max`
  contre un catalogue qui l'attend ; inatteignable par `maxlength` (`:205`), à aligner sur le patron des autres sites.
- #1084 (numéro exact puis *Tab*) et #1081 restent ouvertes ; #1151 n'entre que si H3 est confirmée (D6).
- La mise en forme suisse des montants de l'indicateur (`1'200.00`) : hors périmètre, aucun texte de l'issue ne la demande.

### Fichiers communs avec les autres stories de la phase 1 (tableau du fichier d'epic)

- `AccountAutocomplete.svelte` : **non touché** (D7) — la 36-2 part de la tête de la 36-1 et n'a pas de conflit avec elle.
- `TransactionSplitModal.svelte` : 36-2 seule.
- `journal-entries/balance.ts` : **importé, non modifié**, et seulement si H2 (D5) ; la 36-3 le modifie : aucun conflit de
  fichier, mais le comportement du champ suit la 36-3 après fusion — à rejouer par R0 (c), (d) à l'intégration.
- `messages.ftl` ×4 : à côté du bloc `reconciliation-split-*` (fr-CH `:1618-1621`, `:1836-1850`), jamais en fin de fichier.
- `i18n-keys.test.ts` (`sitesTotal`) : recompté à la source, sans additionner.
- `docs/manual/fr/user-manual.tex` : § *Éclatement* (`:3221-3231`), distinct des paragraphes des 36-3, 36-4, 36-5.
- `CHANGELOG.md` : **non touché** (C-e360-11).

### Modules touchés (règle de splitting)

Production : `frontend/src/lib/features/reconciliation` (le dialogue) et `crates/kesh-i18n` (quatre `messages.ftl`) ;
documentation : `docs/manual/fr`. Tests : `frontend/tests/e2e`, `frontend/src/lib/shared` (`sitesTotal`). Si H2,
`features/journal-entries` est importé, non modifié. **Deux modules de production, cinq au plus en tout** : aucun découpage.

### Le manuel

`git grep -n -i 'clater' -- docs/manual/fr/*.tex` : `user-manual.tex:3221` (§ *Éclatement*), `:3099`, `:3102`, `:3137`,
`:3149`, `:3156`, `:4204`, `:4206` (renvois au bouton, inchangés), `admin-manual.tex:2856` (inchangé). L'étape 3 de `:3230`
dit « Valider » ; le bouton s'appelle *Éclater* et le manuel ne dit ni quand il est grisé ni où lire pourquoi : AC9.

### Ce qui doit rester vrai

- Le serveur reste juge : rien ne change à `POST /api/v1/reconciliation/split` ; le client n'envoie que des chaînes, comme
  l'API externe.
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
