# Story 36-1 : *Entrée* dans le champ « compte de produit » n'envoie plus un compte que le champ ne montre pas (#1095)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-1`, branche
`story/36-1-entree-compte-produit`, par le workflow `bmad-create-story`. Release **0.17.8** (phase 1). **Aucune migration**,
aucun changement serveur ni d'API. **Une clé i18n neuve** (`common-account-no-match`, quatre locales). Code lu à la tête du
worktree, **`9d5a142df`** (kickoff de l'Epic 36, sur `main` `42d5f368`). Étape 0 et choix qui font foi : registre
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-1 ; C-e360-1, -2, -3, -4, -11) ; choix propres à la story :
`e36-choix-autonomes-36-1.md` (C-36-1-1 à C-36-1-4). Voie de développement : **voie 2**. Validation non lancée.*

## Story

En tant que titulaire qui monte une facture client ou tient son catalogue d'articles, je veux que la touche *Entrée* dans le
champ « Compte de produit » ne fasse jamais partir le formulaire avec un autre compte que celui que le champ montre, afin de
ne pas enregistrer une facture (ou un article) sur un compte que je crois avoir changé.

## Ce qui se lit au code (`9d5a142df`)

**Le composant** — `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` (411 lignes) :

- le champ montre le brouillon pendant la frappe : `:180` `const query = $derived(draft ?? displayLabel);` ; `draft` vaut
  `null` hors édition (`:175` `let draft = $state<string | null>(null);`) ;
- *Entrée* n'est interceptée que si la liste est ouverte **et** a un élément en surbrillance :
  `:265` `} else if (e.key === 'Enter') {` / `:266` `if (open && filtered[highlightIndex]) {` / `:267` `e.preventDefault();` /
  `:268` `handleSelect(filtered[highlightIndex]);`. Dans tous les autres cas, l'événement suit son cours et le navigateur
  soumet le `<form>` englobant (soumission implicite) **sans `blur`** ;
- seul `handleBlur` (`:283-317`) résout le brouillon : `:300` `if (!allowClear) {` / `:301` `draft = null;` (strict : jeté) ;
  `:305` `if (draft.trim() === '') {` / `:309` `if (value !== null && selected !== undefined) onSelect(null);` (`allowClear` :
  vidé = effacement) ; `:316` `draft = null;` (`allowClear` : texte libre jeté). Le commentaire `:297-299` le dit déjà :
  « Un envoi sans quitter le champ doit donc rendre le focus d'abord » ;
- la liste se ferme sur *Échap* (`:270` `} else if (e.key === 'Escape') {` / `:271` `open = false;`) **en gardant le
  brouillon**, et la liste filtrée est vide quand rien ne correspond (`:223-234`, filtre `:230`
  `a.number.toLowerCase().startsWith(q) ||` / `:231` `a.name.toLowerCase().includes(q)`, sur `active` `:201-208`, qui
  applique `requiredAccountType`) ;
- le mode dégradé ne passe pas par là : `:256` `if (loadError) return;` — le brouillon y **est** l'identifiant, déjà envoyé par
  `handleInput` (`:243-246`).

**Les trois chemins du défaut** — un brouillon est affiché (`draft !== null`) et *Entrée* n'est pas interceptée :

| Chemin | Geste | Ce que le champ montre | Ce qui part |
|---|---|---|---|
| C1 | taper un texte sans correspondance (« zzz », un numéro d'un compte qui n'est pas de type produit), *Entrée* | « zzz » | l'ancien compte (ou `null`) |
| C2 | taper un texte qui correspond, *Échap*, *Entrée* | le texte tapé | l'ancien compte |
| C3 | vider le champ, *Échap*, *Entrée* | champ vide (placeholder « … (défaut société) ») | l'ancien compte |

**Les deux consommateurs atteints** — les seuls dont le champ est dans un `<form>` :

- `frontend/src/lib/components/invoices/InvoiceForm.svelte:697` `<form onsubmit={onSubmit} class="space-y-6">` ; le champ de
  chaque ligne `:877` `<AccountAutocomplete` … `:881` `allowClear` … `:887` `onSelect={(id) => (line.revenueAccountId = id)}` ;
  bouton par défaut `:932` `<Button data-testid="create-invoice-button"` / `:933` `type="submit"` ; `onSubmit` (`:537`)
  lit les lignes et appelle `updateInvoice` ou `createInvoice`, puis redirige vers le détail ;
- `frontend/src/routes/(app)/products/+page.svelte:622` `<form` / `:624` `onsubmit={(e) => {` (dans le dialogue de la fiche
  article) ; le champ `:697` `<AccountAutocomplete` … `:698` `id="form-revenue-account"` … `:703` `allowClear` … `:705`
  `onSelect={(id) => (formRevenueAccountId = id)}` ; bouton par défaut `:732`
  `<Button type="submit" disabled={formSubmitting || !!formValidation}>`.

**Inventaire des consommateurs** — `git grep -n "<AccountAutocomplete" -- frontend/src`, sortie copiée :

```
frontend/src/lib/components/invoices/InvoiceForm.svelte:877:							<AccountAutocomplete
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:397:						<AccountAutocomplete
frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte:159:					<AccountAutocomplete
frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte:204:					<AccountAutocomplete
frontend/src/lib/features/reconciliation/ManualMatchModal.svelte:188:				<AccountAutocomplete
frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte:303:									<AccountAutocomplete
frontend/src/routes/(app)/products/+page.svelte:697:				<AccountAutocomplete
```

Formulaires englobants, sur chaque fichier **et sur ses parents** (`/usr/bin/grep -n "<form\|</form>"`) : `InvoiceForm.svelte`
`697:<form onsubmit={onSubmit} …` et `941:</form>` ; `products/+page.svelte` `622:		<form` et `738:		</form>` ; **vide** pour
`JournalEntryForm.svelte`, `VatPurchaseAssistant.svelte`, `ManualMatchModal.svelte`, `TransactionSplitModal.svelte`, et pour
leurs parents (`git grep -n "<JournalEntryForm\b\|<VatPurchaseAssistant\b\|<ManualMatchModal\b\|<TransactionSplitModal\b"` :
`routes/(app)/journal-entries/+page.svelte:353`, `routes/(app)/journal-entries/[id]/+page.svelte:491`,
`JournalEntryForm.svelte:354`, `ReconciliationProposals.svelte:810`, `:823`, `CounterpartyModalHost.test.svelte:37`, `:47` —
aucun `<form>`). Les cinq consommateurs stricts ne soumettent donc rien sur *Entrée* ; les seules autres touches *Entrée* de
ces écrans sont `JournalEntryForm.svelte:297` (`if (e.key === 'Enter' && index === lines.length - 1) {`, dans le champ
**crédit**, qui ajoute une ligne) et le `Ctrl+S` de `JournalEntryForm.svelte:285`, qui rend déjà le focus (`:289`
`(document.activeElement as HTMLElement | null)?.blur();`, 35-7, E1).

**Les autres chemins d'envoi sans `blur`, cherchés par le symptôme** (un envoi qui part pendant que le focus est dans le
champ) :

- **`Ctrl+S` global** : `routes/(app)/+layout.svelte:34` `if ((e.ctrlKey || e.metaKey) && e.key === 's') {` / `:36`
  `window.dispatchEvent(new CustomEvent('kesh:save'));` — `git grep -n "kesh:save" -- frontend/src` ne rend **que** cette
  ligne : aucun écran ne l'écoute, donc ni la facture ni l'article ne s'enregistrent par `Ctrl+S`. Pas un chemin du défaut ;
  mais le manuel annonce le contraire (`user-manual.tex:4366` « `\keshkey{Ctrl+S} & Sauvegarder / soumettre le formulaire en
  cours` ») : **issue demandée** (Dev Notes, I3).
- **`requestSubmit`** : `git grep -n -E "key === 's'|requestSubmit|ctrlKey" -- frontend/src` (hors tests) rend
  `JournalEntryForm.svelte:285`, `+layout.svelte:34`, `:39`, `journal-entries/+page.svelte:293` — aucun `requestSubmit`.
- **Clic sur « Enregistrer »** : le `mousedown` sur le bouton fait perdre le focus au champ avant le `click`, donc `handleBlur`
  passe d'abord (test existant `InvoiceForm.test.ts:320-349`, « vider puis enregistrer IMMÉDIATEMENT persiste `null` »).

**Le même symptôme hors de ce composant** — un champ de recherche qui montre un texte tapé pendant que le formulaire garde
une autre sélection. Détecteur par lecture, parti du rôle ARIA d'un sélecteur (`git grep -n -l -E 'aria-autocomplete|
role="combobox"|role="listbox"' -- frontend/src`, hors tests) : `lib/components/invoices/ContactPicker.svelte` et
`lib/features/journal-entries/AccountAutocomplete.svelte` (sonde positive : ce dernier, le défaut connu, est trouvé). Angle mort
écrit : un sélecteur sans rôle ARIA ; le seul autre motif voisin (`git grep -n -E "<datalist|Combobox|Command\.Input"`) est
`settings/vat-rates/+page.svelte:279` `<datalist …>`, champ libre dont le texte **est** la valeur — pas le défaut.
`ContactPicker.svelte`, dans le `<form>` de la facture (`InvoiceForm.svelte:714`
`<ContactPicker selected={selectedContact} onSelect={onContactSelect} />`), présente le symptôme sous **deux** formes que la
36-1 ne traite pas : son `onBlur` ne résout rien (`:110-113`, `:112` `setTimeout(() => (open = false), 150);`) — un nom tapé et non
choisi reste affiché pendant que l'ancien client part, par *Entrée* comme par clic sur *Enregistrer* ; et son *Entrée*
(`:98` `} else if (e.key === 'Enter' && highlighted >= 0 && results[highlighted]) {`) choisit dans les résultats de la
recherche **précédente** pendant les 300 ms de l'anti-rebond (`:77`). **Hors périmètre, deux issues demandées** (Dev Notes,
I1, I2) : autre composant, autre logique (recherche serveur), et le chemin *blur* n'est pas celui de #1095.

**Tests existants** :

- `frontend/src/lib/features/journal-entries/AccountAutocomplete.test.ts` (456 lignes) : aucun test d'*Entrée*
  (`/usr/bin/grep -n "Enter" …AccountAutocomplete.test.ts` vide) ; patron de montage `:21-27` (mock `i18nMsg`, rendu par
  `@testing-library/svelte`), helper `acc()` `:29-39` ;
- `frontend/src/lib/components/invoices/InvoiceForm.test.ts` : `invoiceWith()` `:107`, `accountInputs()` `:130`, `settle()`
  `:140`, mock `updateInvoiceMock` (test `:320-349`) ;
- `frontend/src/routes/(app)/products/products-page.test.ts` : mocks `createProduct`, `updateProduct` (`:41-46`) ; trois tests
  de chargement seulement (`:89-113`) ;
- E2E : `frontend/tests/e2e/invoice-revenue-account.spec.ts` (montage d'un brouillon et des comptes `:47-121`, champs
  `page.locator('input[aria-autocomplete="list"]')` `:118`) ; `frontend/tests/e2e/product-revenue-account.spec.ts`
  (`mkRevenueAccount` `:68`, `openCreateDialog` `:82`, `openEditDialog` `:89`, champ `#form-revenue-account`).

**Outillage de test** : ni `@testing-library/user-event` (`frontend/package.json`, seuls `@testing-library/svelte` `:30` et
`jsdom` `:33`), ni soumission implicite dans jsdom (`node_modules/jsdom/lib/jsdom/living/nodes/HTMLInputElement-impl.js` ne
l'implémente pas ; `HTMLFormElement-impl.js` porte `requestSubmit`). Un test Vitest ne peut donc constater que
l'`defaultPrevented` de l'événement ; la soumission réelle ne se voit qu'en E2E.

## Décisions

- **D1 — Principe : tant qu'un brouillon est affiché, *Entrée* ne laisse jamais partir le formulaire ; elle résout le
  brouillon, et une seconde *Entrée* envoie** (C-36-1-1). La correction vit **dans le composant**, pas dans les deux
  consommateurs : c'est lui qui sait qu'un brouillon est affiché, et le défaut est le sien (le même geste sur un futur
  consommateur dans un `<form>` serait couvert). Nouveau comportement de `handleKeydown` sur *Entrée*, hors mode dégradé
  (inchangé, `:256`) :
  1. **aucun brouillon** (`draft === null`) : inchangé — un élément en surbrillance d'une liste ouverte est choisi
     (`preventDefault`), sinon l'événement suit son cours et le formulaire part ; le champ montre alors `displayLabel`, la
     vérité. *Entrée* sur un champ non touché reste donc le geste d'envoi ;
  2. **brouillon, liste ouverte, élément en surbrillance** : inchangé — l'élément est choisi (`preventDefault`), le champ
     montre son libellé, rien ne part. **Un texte qui correspond à un compte unique ou à plusieurs** suit cette règle : c'est
     l'élément en surbrillance — le premier, ou celui que les flèches ont désigné — qui est choisi, **visible avant le
     choix** ;
  3. **brouillon non vide, liste fermée** (après *Échap*, chemin C2), **avec correspondances** : `preventDefault`, la liste
     **se rouvre** (`open = true`, `highlightIndex = 0`), rien n'est choisi. Même pour une correspondance unique : *Échap* a
     écarté la liste, la rouvrir montre ce qu'une *Entrée* suivante choisira, au lieu de choisir sans le montrer ;
  4. **brouillon non vide sans aucune correspondance** (chemin C1, liste ouverte ou fermée) : `preventDefault`, le brouillon
     **reste** affiché, et un message dit qu'aucun compte ne correspond (D2). Rien n'est choisi, rien ne part ;
  5. **brouillon vide** (`draft.trim() === ''`) **sans élément en surbrillance** (liste fermée, chemin C3) :
     `preventDefault`, puis **la même résolution que le `blur`** — `allowClear` : effacement explicite, sous la même condition
     (`value !== null && selected !== undefined`) ; strict : brouillon jeté, compte retenu réaffiché. Le champ montre alors la
     vérité ; rien ne part.

  La résolution du brouillon (`:291-316`) est **extraite** en une fonction (`commitDraft()`) que `handleBlur` et le cas 5
  appellent tous deux — une seule règle, écrite une fois (DRY). **Options écartées** : (a) résoudre le brouillon comme le
  `blur` **puis** laisser partir le formulaire (remède 1 de l'issue) — le champ montrerait l'ancien compte au moment de
  l'envoi, mais la facture partirait et la page changerait sans que l'utilisateur voie que sa saisie a été jetée ; (b) choisir
  d'office la correspondance unique, même liste fermée — un choix que l'écran n'a pas montré, contraire à la 35-7 (D2 : seul
  un choix explicite sélectionne) ; (c) corriger dans chaque consommateur (`onkeydown` sur le `<form>`) — deux copies, et le
  composant resterait piégé pour le suivant. **Réversible** : oui, une fonction.
- **D2 — Le message « aucun compte ne correspond »** (C-36-1-2). Clé **neuve** `common-account-no-match`, espace de noms
  global `common-` (autorisé partout par `lint-i18n-ownership.js:12`
  `const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];`, sans entrée à `KNOWN_VIOLATIONS`),
  rangée **sous `common-account-invalid`** dans les quatre `messages.ftl` (fr-CH `:2648`, de-CH `:2520`, en-CH `:2525`, it-CH
  `:2518` à `9d5a142df`), jamais en fin de fichier. Texte (le développement peut l'ajuster, le sens reste) :
  - fr-CH : `Aucun compte ne correspond à cette saisie : choisissez un compte dans la liste.`
  - de-CH : `Kein Konto entspricht dieser Eingabe: Wählen Sie ein Konto aus der Liste.`
  - en-CH : `No account matches this entry: choose an account from the list.`
  - it-CH : `Nessun conto corrisponde a questo inserimento: scegliete un conto dall'elenco.`

  Rendu sous le champ, comme le marqueur d'invalidité (`:384` `<p id={invalidMsgId} class="mt-1 text-xs text-destructive">`),
  avec un `id` propre joint à `aria-describedby` (`:337-339`, composé avec `describedBy` et le marqueur) et `aria-live="polite"`.
  Il s'affiche au cas 4 de D1 et **disparaît** à la frappe suivante, au choix d'un compte, au bouton d'effacement et au `blur`.
  Valable pour les sept consommateurs : il ne nomme ni produit ni facture. **Option écartée** : jeter le brouillon sans message
  (le champ réafficherait l'ancien compte) — vrai, mais l'utilisateur voit son texte remplacé sans savoir pourquoi, et doit le
  retaper pour corriger une faute de frappe. **Réversible** : oui.
- **D3 — Le sort des sept consommateurs** (C-36-1-3). Aucun consommateur n'est modifié. Les deux `allowClear` dans un `<form>`
  (`InvoiceForm`, fiche article) reçoivent le correctif et **un test chacun** (AC3, AC4). Les cinq stricts
  (`JournalEntryForm:397`, `VatPurchaseAssistant:159`, `:204`, `ManualMatchModal:188`, `TransactionSplitModal:303`) ne sont
  dans aucun `<form>` : pour eux, *Entrée* n'envoyait rien, et le changement visible se borne aux cas 3 à 5 de D1 (liste
  rouverte, message, brouillon résolu au lieu de rester affiché) ; leurs suites restent vertes, sans test neuf (le composant
  seul le couvre, AC1). ⚠️ `TransactionSplitModal` est l'objet de la 36-2 (hypothèse H1 de C-e360-5) : la 36-2 part de la
  36-1 fusionnée (C-e360-4) et dira si le message de D2 suffit à son cas.
- **D4 — Le manuel ne change pas** (C-36-1-4). Il ne décrit aucun geste clavier du champ de compte :
  `git grep -n -i -E "touche entr|clavier|surbrillance|liste déroulante" -- docs/manual/fr/user-manual.tex` rend `:60`
  (convention `\keshkey`), `:95` (session), `:312` (graphiques), `:1800` (création d'un fournisseur), `:4360` (raccourcis) —
  rien sur le compte de produit ; `:1129` (fiche article) et `:1159` (ligne de facture) décrivent le sens du champ, non sa
  saisie. Le message de D2 se lit de lui-même. Comme la 35-7 (AC5). **Option écartée** : une phrase à `:1159` — elle
  décrirait un geste clavier que le manuel ne décrit nulle part ailleurs, pour un écran sur deux. **Réversible** : une phrase
  à ajouter si la revue le demande ; elle toucherait `user-manual.tex`, commun à 36-3 et 36-5 (paragraphes distincts).

## Critères d'acceptation

1. **AC1 — Le composant** (`AccountAutocomplete.test.ts`, `describe` neuf « Entrée (Story 36-1, #1095) » ; l'événement est
   constaté par la valeur rendue de `fireEvent.keyDown`, `false` quand `preventDefault` a été appelé) :
   - a. `allowClear`, valeur `1` (`3000 — Ventes`) : taper `zzz`, *Entrée* → événement empêché ; `onSelect` jamais appelé ; le
     champ montre `zzz` ; le message « Aucun compte ne correspond » est affiché et son `id` figure dans `aria-describedby` ;
   - b. même montage : taper `zzz`, *Échap*, *Entrée* → idem (C1 liste fermée) ;
   - c. deux comptes `3000`, `3001` : taper `300`, *Entrée* → événement empêché ; `onSelect(id de 3000)` une fois ; le champ
     montre `3000 — Ventes` (non-régression du cas 2, vert au départ, **dit**) ; taper `300`, *flèche bas*, *Entrée* →
     `onSelect(id de 3001)` ;
   - d. taper `3001` (correspondance unique), *Échap*, *Entrée* → événement empêché ; `onSelect` jamais appelé ; la liste
     (`role="listbox"`) est de nouveau affichée ; une seconde *Entrée* → `onSelect(id de 3001)` ;
   - e. `allowClear`, valeur `1` : vider le champ, *Échap*, *Entrée* → événement empêché ; `onSelect(null)` **une** fois ; le
     champ est vide ;
   - f. **strict**, valeur `1` : vider le champ, *Échap*, *Entrée* → événement empêché ; `onSelect` jamais appelé ; le champ
     montre `3000 — Ventes` ;
   - g. valeur `1`, aucune frappe : focus, *Entrée* → événement **non** empêché, `onSelect` jamais appelé (le formulaire peut
     partir : le champ montre la vérité) ;
   - h. après le cas a : une frappe (`zz`) → le message disparaît ; de même après un `blur` ;
   - i. mode dégradé (`loadError`) : taper `7`, *Entrée* → événement non empêché, comportement inchangé.
2. **AC2 — Les tests existants** : la suite `AccountAutocomplete.test.ts` existante reste verte **sans modification de ses
   assertions** (le `blur` garde exactement sa règle, désormais dans `commitDraft()`), ainsi que celles de `JournalEntryForm`,
   `ManualMatchModal`, `TransactionSplitModal`, `InvoiceForm`, `products-page`.
3. **AC3 — La facture** (`InvoiceForm.test.ts`, `describe` neuf) — la soumission implicite y est **simulée** par un helper
   écrit une fois dans le fichier : `fireEvent.keyDown(champ, { key: 'Enter' })`, puis, si l'événement n'a pas été empêché,
   `champ.form!.requestSubmit()` (ce que fait le navigateur). Brouillon édité dont la ligne porte `3200 — Honoraires` :
   - a. taper `zzz`, *Entrée* simulée → `updateInvoiceMock` **jamais appelé** ; le champ montre `zzz` ;
   - b. taper le numéro exact d'un autre compte de produit, *Entrée* (choisit), *Entrée* (envoie) → `updateInvoiceMock`
     appelé une fois, `lines[0].revenueAccountId` = l'identifiant de ce compte.
4. **AC4 — L'article** (`products-page.test.ts`, `describe` neuf) — même helper, sur le dialogue d'édition d'un article dont
   le compte de produit est `A` : taper `zzz`, *Entrée* simulée → ni `updateProduct` ni `createProduct` appelés, le dialogue
   reste ouvert, le champ montre `zzz`. (Le montage du dialogue est à écrire ; s'il exige plus que les mocks existants
   `:26-58`, le dire au Change Log.)
5. **AC5 — E2E, à la frontière** (le défaut se voit dans le navigateur : la soumission implicite n'existe qu'en vrai) :
   - a. **facture** (`invoice-revenue-account.spec.ts`, test neuf, montage du test `:47-121`) : brouillon dont la ligne 1
     porte le compte `honoraires` posé par l'API ; `/invoices/{id}/edit` ; `fill('zzz')` dans le premier champ
     `input[aria-autocomplete="list"]`, `press('Enter')` → l'URL reste `/edit`, le message est visible, **aucune requête
     `PUT /api/v1/invoices/{id}`** n'est partie (compteur `page.on('request')` posé avant le geste) ; puis `fill(<numéro de
     marchandises>)`, `press('Enter')` → le champ montre `<numéro> — <nom>` ; `press('Enter')` → l'URL devient le détail, et
     `GET /api/v1/invoices/{id}` rend `lines[0].revenueAccountId` = `marchandises.id` ;
   - b. **article** (`product-revenue-account.spec.ts`, test neuf, `mkRevenueAccount` et `openEditDialog`) : article créé par
     l'API avec le compte `A` ; dialogue d'édition ; `#form-revenue-account` : `fill('zzz')`, `press('Enter')` → le dialogue
     reste ouvert, le message est visible, aucune requête `PUT /api/v1/products/{id}` ; `GET` de l'article : compte `A`.
   Rouges au départ : le premier *Entrée* soumet (redirection vers le détail ; dialogue fermé).
6. **AC6 — Textes** : la clé `common-account-no-match` dans les quatre `messages.ftl`, à côté de `common-account-invalid` ;
   parité des locales verte (`crates/kesh-i18n/src/loader.rs:771` `fn parity_between_locales() {`) ;
   `npm run lint-i18n-ownership` vert ; `sitesTotal` d'`frontend/src/lib/shared/i18n-keys.test.ts:985` (`sitesTotal: 3036,`)
   **recompté à la source** par la garde (un `i18nMsg(` ajouté : 3037 attendu, à constater, non à écrire d'avance —
   C-e350-18). Aucun manuel modifié (D4).
7. **AC7 — Captures** (thème clair, base de démonstration jetable, `/home/gcorbaz/devel/kesh-gate-logs/e36-1/captures/`) :
   ligne de facture avec `zzz` et le message ; fiche article, même état ; ligne de facture après *Échap* puis *Entrée* (liste
   rouverte).

## Tests à écrire, et la preuve du rouge

Écrits **avant** le correctif et commités avec lui seulement après le constat ; journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-1/rouges.log` (Vitest ciblé et E2E ciblé, sortie copiée).

- **Rouges au départ** : AC1 a, b, d, e, f, h (l'événement n'est pas empêché ; pas de message) ; AC3 a (`updateInvoiceMock`
  appelé avec `3200`) ; AC4 (`updateProduct` appelé) ; AC5 a et b (redirection ; dialogue fermé ; `PUT` parti).
- **Verts au départ, et dits** (non-régression) : AC1 c, g, i ; AC3 b (la première *Entrée* choisit déjà dans une liste
  ouverte, la seconde envoie déjà : ce test garde contre un blocage excessif, M2) ; AC2.
- **Mutations** (code commité avant ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-1/mut/` ; restauration
  `git restore --worktree -- <fichier>` puis `touch <fichier>` ; une mutation équivalente se dit) :
  - M1 : la branche neuve retirée (retour à `:266-269`) → AC1 a, b, d, e, f ; AC3 a ; AC4 ; AC5 ;
  - M2 : `preventDefault` sur **toute** *Entrée* (même sans brouillon) → AC1 g ; AC3 b (la seconde *Entrée* ne part plus) ;
  - M3 : cas 4, le brouillon jeté (`draft = null`) au lieu d'être gardé → AC1 a (le champ ne montre plus `zzz`) ;
  - M4 : cas 3, la correspondance choisie au lieu de rouvrir la liste → AC1 d (`onSelect` appelé) ;
  - M5 : cas 5, `commitDraft()` non appelé → AC1 e et f ;
  - M6 : le message non effacé à la frappe → AC1 h ;
  - M7 : `aria-describedby` sans l'`id` du message → AC1 a.

## Tâches

- [ ] Étape 0 : relire à la tête du worktree les lignes citées ci-dessus (`AccountAutocomplete.svelte:255-317`,
      `InvoiceForm.svelte:697`, `:877-888`, `:932-933`, `products/+page.svelte:622-627`, `:697-705`, `:732`) ; relancer
      `git grep -n "<AccountAutocomplete" -- frontend/src` et le contrôle des `<form>` ; citer les sorties au Change Log.
- [ ] Bases de la voie 2 (`kesh_36_1`, `kesh_e2e_36_1` sur `127.0.0.1:3307`), `node_modules` par `cp -al`.
- [ ] Tests d'AC1, AC3, AC4, AC5 écrits ; rouges constatés et journalisés ; commit.
- [ ] `AccountAutocomplete.svelte` : `commitDraft()` extraite de `handleBlur` ; *Entrée* selon D1 ; message de D2 et son
      `aria-describedby` ; commentaires du fichier mis à jour (`:297-299` dit qu'un envoi sans quitter le champ doit rendre le
      focus d'abord : à compléter par « ou passer par *Entrée*, qui résout le brouillon sans envoyer ») ; doc de la prop
      `allowClear` relue.
- [ ] `messages.ftl` ×4 (D2) ; `sitesTotal` recompté.
- [ ] Verts ; mutations M1 à M7 ; captures (AC7).
- [ ] Gates (voie 2 ; `gate-ci` en voie 1), selon `e35-dev-commun.md` § *Quels gates, quand* : en cours, gate ciblé — Vitest
      des fichiers touchés, `npm run check`, `npm run lint-i18n-ownership`, et `cargo nextest run -E 'binary(...)'` du crate
      `kesh-i18n` (les `.ftl` y sont lus) ; **au passage en `review`** : bases remises à neuf, `scripts/test-fast.sh --ci`
      (voie 2), `scripts/gate-ci.sh kesh_ci_36_1` (voie 1), frontend complet (`check`, `lint-i18n-ownership`, `test:unit`,
      `build`) ; **pas d'E2E** ; **à la clôture** : `gate-ci` et E2E complet sur `kesh_e2e_36_1` recréée, au dernier commit de
      code, échecs nommés contre `docs/testing.md` § *Les échecs attendus*.
- [ ] Prompt de la revue P1 à côté de la fiche (trois lentilles ; axes exercés et non exercés ; manuel nommé — D4 à
      contester ; captures nommées).

## Note de version

*Corrigé* (C-e360-11 ; l'orchestrateur l'assemble au CHANGELOG à l'intégration) :

- **Facture et article : la touche Entrée dans le champ « Compte de produit » n'enregistre plus l'ancien compte pendant que
  le champ montre le texte tapé.** Entrée choisit le compte en surbrillance dans la liste, rouvre la liste si vous l'aviez
  fermée, ou signale qu'aucun compte ne correspond à votre saisie ; une seconde Entrée enregistre. (refs #1095)

## Dev Notes

### Hors périmètre, et les issues demandées

- **I1 — `ContactPicker`, le client de la facture** (bug, P2 proposé) : un nom tapé et non choisi reste affiché au `blur`
  (`ContactPicker.svelte:110-113`), pendant que l'ancien client part — par *Entrée* comme par *Enregistrer*. Même symptôme que
  #1095, autre composant ; à ouvrir, une issue.
- **I2 — `ContactPicker`, *Entrée* pendant l'anti-rebond** (bug, P3 proposé) : *Entrée* dans les 300 ms qui suivent la frappe
  (`:77`) choisit le premier résultat de la recherche **précédente** (`:98`) ; le champ montre alors ce contact (visible, mais
  pas celui cherché). À ouvrir, une issue.
- **I3 — `Ctrl+S` annoncé, sans effet** (documentation ou bug, P3 proposé) : le manuel (`user-manual.tex:4366`) dit que
  `Ctrl+S` sauvegarde le formulaire en cours ; l'événement `kesh:save` (`+layout.svelte:36`) n'a aucun écouteur, seule
  l'écriture manuelle a le sien. À ouvrir, une issue.
- **I4 — *Entrée* sur un champ de compte vide et non touché choisit le premier compte de la liste** (bug, P3 proposé) : le
  focus ouvre la liste (`AccountAutocomplete.svelte:327` `onfocus={() => (open = true)}`), `query` vide propose les vingt
  premiers comptes (`:226`), `highlightIndex` vaut 0 : la première *Entrée* choisit ce compte (visible), la seconde envoie.
  Sur la ligne de facture, une double *Entrée* pour enregistrer pose donc le premier compte de produit au lieu du défaut
  société. Le champ ne ment pas — hors de la classe de #1095 —, mais le choix n'a pas été désigné ; le corriger exige de
  distinguer une surbrillance désignée (frappe ou flèches) d'une surbrillance par défaut, ce qui change la sélection au
  clavier des sept consommateurs. À ouvrir, une issue ; à reconsidérer à la validation si l'orchestrateur veut l'intégrer.
- **I5 — Mode dégradé** (bug, P4 proposé) : liste des comptes non chargée, un texte non numérique (`abc`) envoie `null`
  (`:243-246`) et reste affiché, au `blur` (`:288`) comme à *Entrée* (`:256`). Voisin de #1085 (le brouillon gardé quand la
  liste revient), distinct. À ouvrir, une issue.
- Les issues voisines déjà ouvertes, inchangées : #1081 (barre de défilement de la liste), #1084 (numéro exact puis *Tab*),
  #1085 (mode dégradé puis liste revenue).

### Fichiers communs avec les autres stories de la phase 1 (tableau du fichier d'epic)

- `AccountAutocomplete.svelte` : **36-2** (selon la cause, H1) — en série, la 36-2 part de la 36-1 fusionnée.
- `messages.ftl` ×4 : 36-3 (sûr), 36-2, 36-5 (possibles) — la clé de la 36-1 est rangée sous `common-account-invalid`, loin des
  blocs `journal-entry-*`, `reconciliation-split-*`, `admin-restore-*`.
- `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) : recompté à la source par chaque branche, puis après chaque
  fusion, sans additionner (C-e350-18).
- `CHANGELOG.md` : **non touché** (C-e360-11) ; la note de version est ci-dessus.
- `docs/manual/fr/user-manual.tex` : non touché (D4).

### Modules touchés (règle de splitting)

Code de production : **deux** — `frontend/src/lib/features/journal-entries` (le composant) et `crates/kesh-i18n` (les quatre
`messages.ftl`). Tests seulement : `frontend/src/lib/components/invoices`, `frontend/src/routes/(app)/products`,
`frontend/src/lib/shared` (`i18n-keys.test.ts`), `frontend/tests/e2e`. Six au total, dont quatre par leurs seuls tests ; aucun
découpage proposé — séparer un test de son correctif n'apporterait rien, et le code de production tient en un fichier.

### Ce qui doit rester vrai

- Le champ ne contredit jamais la valeur envoyée — désormais **aussi** à l'instant d'un envoi par *Entrée*.
- Un champ strict n'est jamais nullifié (#271) : ni par le `blur`, ni par *Entrée*.
- *Entrée* sur un champ non touché reste le geste d'envoi du formulaire.
- Le mode dégradé ne change pas. Rien ne change au serveur. Aucune migration (C-e360-3). Fixtures synthétiques seulement.

### Références

- Issue #1095 (P2, `bug`) ; Story 35-7 (#1000 ; D2 : seul un choix explicite sélectionne ; E1 : le `Ctrl+S` de l'écriture
  manuelle rend le focus) ; Story 16-1b (`allowClear`) ; #271.
- Fichier d'epic `_bmad-output/planning-artifacts/epic-36-premiere-facture-et-suite.md` (phase 1, ligne 1 ; fichiers communs).
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Test Locally First* (E2E).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-14 — Fiche créée (`bmad-create-story`, Opus 5.5), code lu à `9d5a142df`. Décisions D1 à D4 (C-36-1-1 à C-36-1-4) ;
  cinq issues demandées à l'orchestrateur (I1 à I5). Validation non lancée.
