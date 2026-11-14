# Story 36-1 : *Entrée* dans le champ « compte de produit » n'envoie plus un compte que le champ ne montre pas, ni un compte que personne n'a désigné (#1095, #1165)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-1`, branche
`story/36-1-entree-compte-produit`, par le workflow `bmad-create-story`. Release **0.17.8** (phase 1). **Aucune migration**,
aucun changement serveur ni d'API. **Une clé i18n neuve** (`common-account-no-match`, quatre locales). Code lu à la tête du
worktree, **`9d5a142df`** (kickoff de l'Epic 36, sur `main` `42d5f368`). Étape 0 et choix qui font foi : registre
`e36-choix-autonomes-e360.md` (§ *Étape 0* — 36-1 ; C-e360-1, -2, -3, -4, -11) ; choix propres à la story :
`e36-choix-autonomes-36-1.md` (C-36-1-1 à C-36-1-5). Voie de développement : **voie 2**. Validation P1 (Sonnet) remédiée ;
**#1165 intégrée** à la story par décision de l'orchestrateur (C-36-1-5).*

## Story

En tant que titulaire qui monte une facture client ou tient son catalogue d'articles, je veux que la touche *Entrée* dans le
champ « Compte de produit » ne fasse jamais partir le formulaire avec un autre compte que celui que le champ montre, afin de
ne pas enregistrer une facture (ou un article) sur un compte que je crois avoir changé — et qu'elle ne choisisse jamais un
compte que je n'ai désigné ni par une saisie ni par les flèches (#1165), afin qu'une double *Entrée* sur une ligne neuve ne
pose pas le premier compte de produit de la liste à la place du compte par défaut de la société.

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

**Le chemin de #1165 — une surbrillance que personne n'a désignée** (le champ ne ment pas, mais le compte n'a pas été
choisi) : `:177` `let highlightIndex = $state(0);` ; le focus ouvre la liste (`:327` `onfocus={() => (open = true)}`) ; une
saisie vide propose les vingt premiers comptes (`:226` `if (q === '') return active.slice(0, 20);`) ; `handleInput` remet la
surbrillance à 0 quelle que soit la saisie (`:240` `highlightIndex = 0;`). D'où deux gestes qui posent le premier compte de la
liste, la seconde *Entrée* envoyant le formulaire :

| Chemin | Geste | Ce qui est choisi |
|---|---|---|
| C4 | ligne neuve (valeur nulle, « … (défaut société) »), focus, *Entrée* | le premier compte de produit de la liste |
| C5 | vider un champ renseigné (liste ouverte), *Entrée* | idem, au lieu de l'effacement qu'accorde le `blur` |

(Validation P1, V1-1 et V1-2. Sur un champ **non touché à valeur résolue**, `query` vaut « 3000 — Ventes », que ni
`startsWith` ni `includes` ne retrouvent : `filtered` est vide et *Entrée* laisse partir le formulaire — propriété du filtre,
fixée par AC1 g.)

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
**crédit**, qui ajoute une ligne) et le `Ctrl+S` de `JournalEntryForm.svelte:285`, qui rend déjà le focus (`:290`
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
  `page.locator('input[aria-autocomplete="list"]')` `:115`) ; `frontend/tests/e2e/product-revenue-account.spec.ts`
  (`mkRevenueAccount` `:68`, `openCreateDialog` `:82`, `openEditDialog` `:89`, champ `#form-revenue-account`).

**Outillage de test** : ni `@testing-library/user-event` (`frontend/package.json`, seuls `@testing-library/svelte` `:30` et
`jsdom` `:33`), ni soumission implicite dans jsdom (`node_modules/jsdom/lib/jsdom/living/nodes/HTMLInputElement-impl.js` ne
l'implémente pas ; `HTMLFormElement-impl.js` porte `requestSubmit`). Un test Vitest ne peut donc constater que
l'`defaultPrevented` de l'événement ; la soumission réelle ne se voit qu'en E2E.

## Décisions

- **D1 — Deux principes : (i) tant qu'un brouillon est affiché, *Entrée* ne laisse jamais partir le formulaire ; elle résout
  le brouillon, et une seconde *Entrée* envoie ; (ii) une surbrillance ne vaut choix que si elle a été désignée — par une
  saisie non blanche ou par une flèche** (C-36-1-1 ; (ii) : C-36-1-5, #1165 intégrée). La correction vit **dans le
  composant**, pas dans les deux consommateurs : c'est lui qui sait qu'un brouillon est affiché, et le défaut est le sien (le
  même geste sur un futur consommateur dans un `<form>` serait couvert).

  **La désignation, par l'index de surbrillance.** Invariant : **`highlightIndex ≥ 0` si et seulement si un compte a été
  désigné** — par une saisie non blanche (le premier compte correspondant) ou par une flèche. Toutes les transitions de l'index
  dans le composant, relevées par `/usr/bin/grep -n "highlightIndex\|open = \|draft = " AccountAutocomplete.svelte` à
  `9d5a142df` (lignes lues : `177:	let highlightIndex = $state(0);`, `240:		highlightIndex = 0;`,
  `261:			highlightIndex = Math.min(highlightIndex + 1, filtered.length - 1);`,
  `264:			highlightIndex = Math.max(highlightIndex - 1, 0);`, `271:			open = false;` (*Échap*), `284:		open = false;` (`blur`),
  `250:		draft = null;` / `251:		open = false;` (choix), `189:			draft = null;` (`$effect` de la valeur, `:186-191`),
  `327:		onfocus={() => (open = true)}`, `370:				draft = null;` (bouton ×), `397`/`403` (lecture, rendu)), et la
  règle de chacune :

  | Transition | Aujourd'hui | Règle 36-1 |
  |---|---|---|
  | valeur initiale (`:177`) | `0` | `-1` |
  | focus (`:327`) | inchangé | `-1` |
  | *Échap* (`:270-271`) | inchangé | `-1` |
  | choix (`:249-253`) | inchangé | `-1` |
  | `blur` (`:283-284`) | inchangé | `-1` |
  | **recalcul** : la suite des identifiants de `filtered` change, ou `value` change — frappe (`:236-240`), bouton × (`:370-371`), valeur posée par le parent (`$effect` `:186-191`), arrivée des comptes après chargement, liste qui rétrécit sous l'index | `0` à la frappe seulement ; sinon inchangé | `0` si le brouillon est **non blanc** et `filtered` non vide (désignation par la saisie), `-1` sinon (V2-2, V2-1 b) |
  | *flèche bas* (`:258-261`) | ouvre, `+1` borné | ouvre ; depuis `-1` → `0`, puis `+1` borné ; `filtered` vide → `-1` |
  | *flèche haut* (`:262-264`) | `-1` borné à `0`, **n'ouvre pas** | **ouvre la liste** ; depuis `-1` → le **dernier** élément (`filtered.length - 1`, patron du combobox ARIA), sinon `-1` borné à `0` ; `filtered` vide → `-1` (V2-1 a) |

  **Mise en œuvre recommandée du recalcul** : `highlightIndex` en `$derived` **réassignable** (Svelte ≥ 5.25 ; le dépôt est en
  `^5.54.0`, `frontend/package.json:35`), dérivé de la **chaîne** des identifiants de `filtered`, de `value` et du caractère
  blanc du brouillon, et réassigné par les flèches, le focus, *Échap*, le choix et le `blur`. Une chaîne et non le tableau :
  un nouveau tableau aux mêmes comptes (rendu du parent) ne doit pas effacer une désignation par les flèches. **Repli
  accepté** : un `$effect` sur les mêmes dépendances, si le `$derived` réassignable ne se prête pas au cas — à dire au Change
  Log. **Options écartées** : remettre l'index à `-1` dans chaque gestionnaire (la liste rétrécie et l'arrivée des comptes
  n'ont pas de gestionnaire : c'est par elles que V2-1 b passait) ; un booléen `designated` à côté d'un index resté à `0`
  (deux états à garder d'accord, surbrillance affichée qui ne vaut pas choix).

  Aucune ligne n'est mise en évidence à `-1` (`class:bg-accent` et `aria-selected` suivent l'index, `:397`, `:403`). **Survol**
  (V2-3) : `:396` `hover:bg-accent` donne aujourd'hui au survol la couleur de la désignation, alors qu'*Entrée* ne choisit pas
  l'option survolée ; il passe à `hover:bg-muted` pour que la couleur `bg-accent` ne signifie que la désignation au clavier.
  La condition de choix de `:266` (`open && filtered[highlightIndex]`) reste la même : par l'invariant, elle ne vaut que pour
  un compte désigné.

  **Règle de `handleKeydown` sur *Entrée*** — **close par construction** (V2-1) : E0, E1, E2 sont essayées dans cet ordre ;
  ce qui reste a **toujours** un brouillon (`draft !== null`), et E3, E4, E5 en sont la **partition** sur deux seuls critères,
  brouillon blanc ou non, `filtered` vide ou non — **aucune condition sur `open` ni sur l'index**. Le code s'écrit donc
  `if (E0) return; if (E1) {…} else if (draft === null) {/* E2 */} else { e.preventDefault(); if (blanc) … else if (vide) … else … }`,
  et aucun état ne laisse partir l'événement avec un brouillon affiché.

  | # | État | Règle | Chemin | AC | Mutation |
  |---|---|---|---|---|---|
  | E0 | `loadError`, ou composition IME en cours (`e.isComposing`, V1-7) | inchangé : retour sans rien faire, l'événement suit son cours | — | AC1 i, AC1 j | M8 |
  | E1 | sinon, `open && filtered[highlightIndex]` (un compte désigné, par l'invariant) — saisie qui correspond à **un ou plusieurs** comptes, ou flèches | `preventDefault`, l'élément est choisi, le champ montre son libellé, rien ne part | — | AC1 c, k, p, q | M11, M13 |
  | E2 | sinon, **aucun brouillon** (`draft === null`) — non touché, valeur résolue **ou** nulle, ou après le bouton × | rien n'est choisi ; l'événement suit son cours : le formulaire part, le champ montrant `displayLabel` (le compte, ou vide = « défaut société » et `null` envoyé) | C4 | AC1 g, l, r | M2, M9, M15 |
  | — | sinon (**un brouillon est affiché**) | `preventDefault` **toujours**, puis E3, E4 ou E5 | — | AC1 q (M14) | M1, M14 |
  | E3 | … brouillon non blanc, `filtered` non vide | la liste **s'ouvre** ou reste ouverte (`open = true`, index `0`), rien n'est choisi | C2 | AC1 d | M4 |
  | E4 | … brouillon non blanc, `filtered` vide | le brouillon **reste** ; le message de D2 s'affiche **si la liste des comptes est chargée** (`accounts.length > 0`) — pendant le chargement, sans message (V1-3) | C1 | AC1 a, b, m | M3, M12 |
  | E5 | … brouillon **blanc** (`draft.trim() === ''`) | **la résolution du `blur`** (`commitDraft()`) : `allowClear` → effacement explicite, sous la même condition (`value !== null && selected !== undefined`) ; strict → brouillon jeté, compte retenu réaffiché ; liste fermée, index `-1` | C3, C5 | AC1 e, f, n, o | M5, M10 |

  Une *Entrée* suivante, sans brouillon, tombe sur E2 et envoie. **Un texte qui correspond à un compte unique ou à
  plusieurs** suit E1 (liste ouverte, premier compte désigné par la saisie ou celui des flèches) ou E3 (liste fermée : elle se
  rouvre) : seul un élément visible et désigné est choisi.

  La résolution du brouillon (`:291-316`) est **extraite** en une fonction (`commitDraft()`) que `handleBlur` et E5
  appellent tous deux — une seule règle, écrite une fois (DRY). **Options écartées** : (a) résoudre le brouillon comme le
  `blur` **puis** laisser partir le formulaire (remède 1 de l'issue) — le champ montrerait l'ancien compte au moment de
  l'envoi, mais la facture partirait et la page changerait sans que l'utilisateur voie que sa saisie a été jetée ; (b) choisir
  d'office la correspondance unique, même liste fermée — un choix que l'écran n'a pas montré, contraire à la 35-7 (D2 : seul
  un choix explicite sélectionne) ; (c) corriger dans chaque consommateur (`onkeydown` sur le `<form>`) — deux copies, et le
  composant resterait piégé pour le suivant ; (d) laisser #1165 à une story ultérieure (version d'origine de cette fiche) —
  elle rouvrirait le même gestionnaire, ses tests, ses deux E2E et ses captures, et laisserait sur l'écran de la première
  facture une double *Entrée* qui pose un compte de produit non désigné (V1-2). **Réversible** : oui, une fonction.
- **D2 — Le message « aucun compte ne correspond »** (C-36-1-2). Clé **neuve** `common-account-no-match`, espace de noms
  global `common-` (autorisé partout par `frontend/scripts/lint-i18n-ownership.js:16`
  `const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];`, sans entrée à `KNOWN_VIOLATIONS`),
  rangée **sous `common-account-invalid`** dans les quatre `messages.ftl` (fr-CH `:2648`, de-CH `:2520`, en-CH `:2525`, it-CH
  `:2518` à `9d5a142df`), jamais en fin de fichier. Texte (le développement peut l'ajuster, le sens reste) :
  - fr-CH : `Aucun compte ne correspond à cette saisie : choisissez un compte dans la liste.`
  - de-CH : `Kein Konto entspricht dieser Eingabe: Wählen Sie ein Konto aus der Liste.`
  - en-CH : `No account matches this entry: choose an account from the list.`
  - it-CH : `Nessun conto corrisponde a questo inserimento: scegliete un conto dall'elenco.`

  Rendu sous le champ, comme le marqueur d'invalidité (`:384` `<p id={invalidMsgId} class="mt-1 text-xs text-destructive">`),
  avec un `id` propre joint à `aria-describedby` (`:337-339`, composé avec `describedBy` et le marqueur). **Annonce** (V1-4) :
  une région `role="status"` **toujours présente** dans le DOM, dont seul le contenu change (vide hors E4) — un conteneur
  `aria-live` inséré en même temps que son texte n'est pas annoncé de façon fiable ; le lien `aria-describedby` vise cette
  même région, et n'est posé que lorsqu'elle a un contenu. AC1 a vérifie `role="status"` et le lien.
  Il s'affiche à l'état E4 de D1 et **disparaît** à la frappe suivante, au choix d'un compte, au bouton d'effacement et au `blur`.
  **Un seul point d'effacement** (V2-6) : E4 retient le brouillon refusé (`noMatchDraft = draft`), et le message est **dérivé**
  — visible tant que `draft === noMatchDraft` (non nul). Les quatre déclencheurs changent tous `draft` (frappe `:238`, choix
  `:250`, bouton × `:370`, `blur` via `commitDraft()`) : aucun n'a à effacer le message lui-même, et une seule mutation (M6,
  le message lu sur un drapeau gardé au lieu de la comparaison) les fait tous rougir. Conséquence assumée : retaper
  exactement le texte refusé fait reparaître le message, qui reste vrai.
  Valable pour les sept consommateurs : il ne nomme ni produit ni facture. **Option écartée** : jeter le brouillon sans message
  (le champ réafficherait l'ancien compte) — vrai, mais l'utilisateur voit son texte remplacé sans savoir pourquoi, et doit le
  retaper pour corriger une faute de frappe. **Réversible** : oui.
- **D3 — Le sort des sept consommateurs** (C-36-1-3). Aucun consommateur n'est modifié. Les deux `allowClear` dans un `<form>`
  (`InvoiceForm`, fiche article) reçoivent le correctif et **un test chacun** (AC3, AC4). Les cinq stricts
  (`JournalEntryForm:397`, `VatPurchaseAssistant:159`, `:204`, `ManualMatchModal:188`, `TransactionSplitModal:303`) ne sont
  dans aucun `<form>` : pour eux, *Entrée* n'envoyait rien. **Mais le principe (ii) de D1 change la sélection au clavier
  des sept** : un champ ouvert sans saisie ne montre plus de surbrillance, et *Entrée* n'y choisit plus le premier compte
  (E2, E5) ; la première *flèche bas* désigne désormais le **premier** compte (aujourd'hui le deuxième, l'index partant de
  `0`) ; la *flèche haut* ouvre la liste sur le dernier compte ; le × et une valeur posée par le parent annulent la
  désignation ; le survol prend `bg-muted` ; le reste — frappe puis *Entrée*, flèches dans une liste ouverte, clic — est
  inchangé (validation P2). Couverture : le composant porte la
  règle et ses tests (AC1, toutes les lignes de la table de D1) ; les consommateurs ne font que passer `accounts`, `value`,
  `onSelect` et leurs options, sans logique clavier propre au champ (lu : aucun `onkeydown` sur `<AccountAutocomplete`, sortie
  du `grep` de *Ce qui se lit au code*) ; aucun test existant ne sélectionne un compte au clavier
  (`git grep -n -E "ArrowDown|ArrowUp|key: 'Enter'|press\('Enter'\)|'Enter'" -- "frontend/src/**/*.test.ts" frontend/tests/e2e`
  ne rend que `SupplierInvoiceForm.test.ts:400`, `audit-log.spec.ts:98`, `bank-manual-statement.spec.ts:192`,
  `homepage-charts.spec.ts:140`, `:151`, `:155`, `supplier-documents.spec.ts:267` — aucun sur un champ de compte ; les
  sélections passent par le `mousedown` d'une option). Les suites des sept restent vertes (AC2), et les deux consommateurs
  dans un `<form>` reçoivent en plus le cas C4 (AC3 c). Sans test neuf pour les cinq stricts. ⚠️ `TransactionSplitModal` est
  l'objet de la 36-2 (hypothèse H1 de C-e360-5) : la 36-2 part de la 36-1 fusionnée (C-e360-4) et dira si le message de D2
  suffit à son cas.
- **D4 — Le manuel ne change pas** (C-36-1-4). Il ne décrit aucun geste clavier du champ de compte :
  `git grep -n -i -E "touche entr|clavier|surbrillance|liste déroulante" -- docs/manual/fr/user-manual.tex` rend `:60`
  (convention `\keshkey`), `:95` (session), `:312` (graphiques), `:1800` (création d'un fournisseur), `:4360` (raccourcis) —
  rien sur le compte de produit ; `:1129` (fiche article) et `:1159` (ligne de facture) décrivent le sens du champ, non sa
  saisie. Le message de D2 se lit de lui-même. Comme la 35-7 (AC5). **Option écartée** : une phrase à `:1159` — elle
  décrirait un geste clavier que le manuel ne décrit nulle part ailleurs, pour un écran sur deux. **Réversible** : une phrase
  à ajouter si la revue le demande ; elle toucherait `user-manual.tex`, commun à 36-3 et 36-5 (paragraphes distincts).

## Critères d'acceptation

1. **AC1 — Le composant** (`AccountAutocomplete.test.ts`, `describe` neuf « Entrée (Story 36-1, #1095, #1165) » ;
   l'événement est constaté par la valeur rendue de `fireEvent.keyDown`, `false` quand `preventDefault` a été appelé ; chaque
   ligne de la table de D1 a au moins un cas). **Montage** (V2-5) : les tests existants passent `onSelect: vi.fn()` sans jamais
   réinjecter la valeur (`AccountAutocomplete.test.ts:171-203`) ; les cas qui constatent l'affichage **après** un choix ou un
   effacement (c, e, k, n, p, q, r) réinjectent la valeur comme un parent : `onSelect` espion qui appelle
   `rerender({ value: id })` (ou un composant hôte). Sans cela, l'affichage retomberait sur la valeur de départ :
   - a. `allowClear`, valeur `1` (`3000 — Ventes`) : taper `zzz`, *Entrée* → événement empêché ; `onSelect` jamais appelé ; le
     champ montre `zzz` ; le message « Aucun compte ne correspond » est affiché dans un élément `role="status"` dont l'`id`
     figure dans `aria-describedby` ;
   - b. même montage : taper `zzz`, *Échap*, *Entrée* → idem (C1 liste fermée) ;
   - c. deux comptes `3000`, `3001`, **valeur `null`** (V2-5), valeur réinjectée : taper `300`, *Entrée* → événement empêché ;
     `onSelect(id de 3000)` une fois ; le champ montre `3000 — Ventes` (non-régression de E1, vert au départ, **dit**) ; taper
     `300`, *flèche bas*, *Entrée* → `onSelect(id de 3001)` ;
   - d. taper `3001` (correspondance unique), *Échap*, *Entrée* → événement empêché ; `onSelect` jamais appelé ; la liste
     (`role="listbox"`) est de nouveau affichée ; une seconde *Entrée* → `onSelect(id de 3001)` ;
   - e. `allowClear`, valeur `1` : vider le champ, *Échap*, *Entrée* → événement empêché ; `onSelect(null)` **une** fois ; le
     champ est vide ;
   - f. **strict**, valeur `1` : vider le champ, *Échap*, *Entrée* → événement empêché ; `onSelect` jamais appelé ; le champ
     montre `3000 — Ventes` ;
   - g. E2, **non touché à valeur résolue** : valeur `1`, focus, *Entrée* → événement **non** empêché, `onSelect` jamais
     appelé (le formulaire peut partir : le champ montre le compte) ;
   - h. après le cas a, quatre variantes, une par déclencheur (V2-6) : une frappe (`zz`) ; un `blur` ; un clic sur le bouton ×
     (valeur réinjectée) ; un choix (taper `30`, clic sur l'option `3000`) → le message disparaît chaque fois, et la région
     `role="status"` reste présente, vide ;
   - i. E0, mode dégradé (`loadError`) : taper `7`, *Entrée* → événement non empêché, comportement inchangé ;
   - j. E0, composition IME : taper `zzz`, puis `keyDown` *Entrée* avec `isComposing: true` → événement **non** empêché, aucun
     message ;
   - k. E1 sans saisie : valeur nulle, focus, *flèche bas*, *Entrée* → événement empêché, `onSelect(id du premier compte de la
     liste)` ; de même valeur `1`, vider le champ, *flèche bas*, *Entrée* → le premier compte ;
   - l. E2, **non touché à valeur nulle** (#1165, chemin C4) : valeur nulle, `allowClear`, focus → la liste est affichée et
     **aucune** option ne porte `aria-selected="true"` ; *Entrée* → événement **non** empêché, `onSelect` jamais appelé, le
     champ reste vide ;
   - m. E4 pendant le chargement : `accounts = []`, valeur nulle, taper `zzz`, *Entrée* → événement empêché, `onSelect` jamais
     appelé, **aucun** message ;
   - n. E5, **vidé, liste ouverte**, `allowClear` (chemin C5) : valeur `1`, vider le champ (sans *Échap*), *Entrée* →
     événement empêché ; `onSelect(null)` **une** fois ; le champ est vide ;
   - o. E5, vidé, liste ouverte, **strict** : valeur `1`, vider le champ, *Entrée* → événement empêché ; `onSelect` jamais
     appelé ; le champ montre `3000 — Ventes` ;
   - p. *flèche haut* depuis `-1`, liste fermée (V2-1 a) : `allowClear`, valeur `1`, comptes `3000`, `3001`, vider le champ,
     *Échap*, *flèche haut* → la liste est affichée, la **dernière** option porte `aria-selected="true"` ; *Entrée* → événement
     empêché, `onSelect(id de 3001)` ; et, sans *flèche haut* (vider, *Échap*, *Entrée*), c'est E5 (cas e) ;
   - q. liste arrivée sous un brouillon (V2-1 b) : monté avec `accounts = []`, valeur `null`, taper `300`, *flèche bas*, puis
     `rerender({ accounts: [3000, 3001] })` → l'option `3000` porte `aria-selected="true"` (désignée par la saisie) ;
     *Entrée* → événement **empêché**, `onSelect(id de 3000)` ;
   - r. bouton × après une désignation (V2-2) : `allowClear`, valeur `1`, valeur réinjectée, taper `30` (option `0` désignée),
     clic sur × → `onSelect(null)` une fois, la liste reste affichée, **aucune** option ne porte `aria-selected="true"` ;
     *Entrée* → événement **non** empêché, `onSelect` n'est pas rappelé. (Une valeur posée par le parent, liste ouverte,
     brouillon nul, change toujours `filtered` avec elle : le cas se confond avec r ou avec g ; pas de test séparé, dit.)
2. **AC2 — Les tests existants** : la suite `AccountAutocomplete.test.ts` existante reste verte **sans modification de ses
   assertions** (le `blur` garde exactement sa règle, désormais dans `commitDraft()`), ainsi que celles de `JournalEntryForm`,
   `ManualMatchModal`, `TransactionSplitModal`, `InvoiceForm`, `products-page`.
3. **AC3 — La facture** (`InvoiceForm.test.ts`, `describe` neuf) — la soumission implicite y est **simulée** par un helper
   écrit une fois dans le fichier : `fireEvent.keyDown(champ, { key: 'Enter' })`, puis, si l'événement n'a pas été empêché,
   `champ.form!.requestSubmit()` (ce que fait le navigateur). Brouillon édité dont la ligne porte `3200 — Honoraires` :
   - a. taper `zzz`, *Entrée* simulée → `updateInvoiceMock` **jamais appelé** ; le champ montre `zzz` ;
   - b. taper le numéro exact d'un autre compte de produit, *Entrée* (choisit), *Entrée* (envoie) → `updateInvoiceMock`
     appelé une fois, `lines[0].revenueAccountId` = l'identifiant de ce compte ;
   - c. (#1165, C4) brouillon édité dont la ligne n'a **pas** de compte (`revenueAccountId: null`) : focus sur le champ,
     *Entrée* simulée → l'événement n'est pas empêché, le formulaire part : `updateInvoiceMock` appelé une fois,
     `lines[0].revenueAccountId` **`null`** (le défaut société), et non le premier compte de produit. Rouge au départ : la
     première *Entrée* choisit le premier compte et rien ne part.
4. **AC4 — L'article** (`products-page.test.ts`, `describe` neuf) — même helper, sur le dialogue d'édition d'un article dont
   le compte de produit est `A` : taper `zzz`, *Entrée* simulée → ni `updateProduct` ni `createProduct` appelés, le dialogue
   reste ouvert, le champ montre `zzz`. **Montage** (V1-6) : à écrire — `listProductsMock` rend aujourd'hui `items: []`
   (`products-page.test.ts:86`), `updateProduct` est un `vi.fn()` anonyme (`:44`, à nommer), et il faut cliquer *Modifier*
   (`+page.svelte:573-574`, `aria-label` « Modifier ») pour ouvrir le `Dialog` (`:612-613`). Un précédent existe dans le dépôt :
   `routes/(app)/contacts/contacts-page.test.ts` ouvre le `Dialog` de formulaire de sa page en jsdom et l'y referme
   (`fermer()`, `:174-182`), ce qui rend le montage plausible — non démontré par exécution. **Repli** : si le dialogue ne se
   monte pas en jsdom, AC4 tombe, c'est déclaré au Change Log avec la cause, et la preuve de l'article est portée par l'E2E
   seul (AC5 b), qui doit alors tuer M1.
5. **AC5 — E2E, à la frontière** (le défaut se voit dans le navigateur : la soumission implicite n'existe qu'en vrai) :
   - a. **facture** (`invoice-revenue-account.spec.ts`, test neuf, montage du test `:47-121`) : brouillon dont la ligne 1
     porte le compte `honoraires` posé par l'API ; `/invoices/{id}/edit` ; `fill('zzz')` dans le premier champ
     `input[aria-autocomplete="list"]`, `press('Enter')` → **attendre le message visible** ; l'URL reste `/edit` ; aucune
     requête `PUT /api/v1/invoices/{id}` (compteur `page.on('request')` posé avant le geste, lu **après** l'attente du
     message) ; puis relecture `GET /api/v1/invoices/{id}` : `lines[0].revenueAccountId` = `honoraires.id` et `version`
     inchangée (V1-5 : la preuve d'absence d'envoi est la relecture, le compteur n'est qu'un indice) ; puis `fill(<numéro de
     marchandises>)`, `press('Enter')` → le champ montre `<numéro> — <nom>` ; `press('Enter')` → l'URL devient le détail, et
     `GET /api/v1/invoices/{id}` rend `lines[0].revenueAccountId` = `marchandises.id` ;
   - a-bis. **facture, #1165** (même fichier, test neuf) : brouillon dont la ligne 2 n'a pas de compte ; `/edit` ; *Tab*
     jusqu'au champ de compte de la ligne 2 (ou `focus()`), `press('Enter')` → l'URL devient le détail ; `GET` :
     `lines[1].revenueAccountId` = `null`. Rouge au départ : la première *Entrée* pose le premier compte de produit et l'URL
     reste `/edit` (une seconde l'enregistrerait) ;
   - b. **article** (`product-revenue-account.spec.ts`, test neuf, `mkRevenueAccount` et `openEditDialog`) : article créé par
     l'API avec le compte `A` ; dialogue d'édition ; `#form-revenue-account` : `fill('zzz')`, `press('Enter')` → le dialogue
     reste ouvert, le message est visible (attendu avant toute lecture), aucune requête `PUT /api/v1/products/{id}` ; `GET`
     de l'article : compte `A`.
   Rouges au départ : le premier *Entrée* soumet (redirection vers le détail ; dialogue fermé).
6. **AC6 — Textes** : la clé `common-account-no-match` dans les quatre `messages.ftl`, à côté de `common-account-invalid` ;
   parité des locales verte (`crates/kesh-i18n/src/loader.rs:771` `fn parity_between_locales() {`) ;
   `npm run lint-i18n-ownership` vert ; `sitesTotal` d'`frontend/src/lib/shared/i18n-keys.test.ts:985` (`sitesTotal: 3036,`)
   **recompté à la source** par la garde (un `i18nMsg(` ajouté : 3037 attendu, à constater, non à écrire d'avance —
   C-e350-18). Aucun manuel modifié (D4).
7. **AC7 — Captures** (thème clair, base de démonstration jetable, `/home/gcorbaz/devel/kesh-gate-logs/e36-1/captures/`) :
   ligne de facture avec `zzz` et le message ; fiche article, même état ; ligne de facture après *Échap* puis *Entrée* (liste
   rouverte) ; ligne neuve au focus, liste ouverte **sans** surbrillance (#1165), **pointeur hors de la liste** (V2-3) ; la
   même liste, pointeur sur une option (survol `bg-muted`, distinct de la désignation).

## Tests à écrire, et la preuve du rouge

Écrits **avant** le correctif et commités avec lui seulement après le constat ; journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-1/rouges.log` (Vitest ciblé et E2E ciblé, sortie copiée).

- **Rouges au départ** : AC1 a, b, d, e, f, h, m (l'événement n'est pas empêché ; pas de message) ; AC1 k (la *flèche bas*
  part de l'index `0` et désigne le **deuxième** compte) ; AC1 l (l'option `0` porte `aria-selected="true"`, puis *Entrée*
  l'empêche et choisit) ; AC1 n, o (le premier compte est choisi) ; AC3 a (`updateInvoiceMock` appelé avec `3200`) ; AC3 c
  (premier compte choisi, rien ne part) ; AC4 (`updateProduct` appelé) ; AC5 a, a-bis et b (redirection ; premier compte
  posé ; dialogue fermé ; `PUT` parti) ; AC1 p (la *flèche haut* n'ouvre pas la liste) ; AC1 q (l'index reste `-1` à
  l'arrivée des comptes, *Entrée* non empêchée) ; AC1 r (l'option `0` reste désignée après le ×, *Entrée* la choisit).
- **Verts au départ, et dits** (non-régression) : AC1 c, g, i, j (aujourd'hui `zzz` n'est jamais intercepté, donc pas en
  composition non plus : j ne devient utile qu'avec le correctif, tué par M8) ; AC3 b (la première *Entrée* choisit déjà dans
  une liste ouverte, la seconde envoie déjà : ce test garde contre un blocage excessif, M2) ; AC2.
- **Mutations** (code commité avant ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-1/mut/` ; restauration
  `git restore --worktree -- <fichier>` puis `touch <fichier>` ; une mutation équivalente se dit) :
  - M1 : la branche neuve retirée (retour à `:266-269`) → AC1 a, b, d, e, f ; AC3 a ; AC4 ; AC5 ;
  - M2 : `preventDefault` sur **toute** *Entrée* (même sans brouillon) → AC1 g, l ; AC3 b, c (l'*Entrée* d'envoi ne part
    plus) ;
  - M3 : E4, le brouillon jeté (`draft = null`) au lieu d'être gardé → AC1 a (le champ ne montre plus `zzz`) ;
  - M4 : E3, la correspondance choisie au lieu de rouvrir la liste → AC1 d (`onSelect` appelé) ;
  - M5 : E5, `commitDraft()` non appelé → AC1 e, f, n, o ;
  - M6 : le message lu sur un drapeau posé à E4 et jamais remis à faux, au lieu de la comparaison `draft === noMatchDraft`
    (le seul point d'effacement de D2) → AC1 h, ses quatre variantes (frappe, `blur`, ×, choix) ;
  - M7 : `aria-describedby` sans l'`id` du message → AC1 a ;
  - M8 : la garde `e.isComposing` retirée → AC1 j ;
  - M9 : l'index de surbrillance remis à `0` au focus (au lieu de `-1`) → AC1 l ; AC3 c ; AC5 a-bis (#1165, C4) ;
  - M10 : une frappe qui laisse un brouillon blanc met l'index à `0` (comme `:240` aujourd'hui) → AC1 n, o (C5) ;
  - M11 : les flèches ne désignent pas depuis `-1` (l'index y reste) → AC1 k ;
  - M12 : le message affiché même sans liste chargée → AC1 m ;
  - M13 : la *flèche haut* reprise d'aujourd'hui (`:262-264` : n'ouvre pas, `Math.max(-2, 0)`) → AC1 p ;
  - M14 : **mutation double**, la table codée à la lettre de sa version P1 (E3 seulement liste fermée, E5 seulement index
    `-1`, sinon l'événement suit son cours) **et** le recalcul de l'index retiré (M15) — seule combinaison qui atteint le trou
    que la construction ferme : AC1 q (événement non empêché, alors que M15 seule le laisse empêché par le repli). M14 seule
    est **équivalente** sous l'invariant (dit) ;
  - M15 : le recalcul de l'index retiré (l'index ne change qu'aux frappes et aux flèches) → AC1 q (`onSelect` non appelé,
    aucune option désignée) ; AC1 r (le × laisse l'option `0` désignée, *Entrée* la choisit). Retirer la seule dépendance à
    `value` (en gardant celle à `filtered`) est **équivalent** : brouillon nul, un changement de valeur change toujours
    `filtered` (dit) ;
  - M16 : le survol rendu à `hover:bg-accent` → aucune assertion de test (classe CSS) ; contrôlé par la capture d'AC7, dit.

## Tâches

- [ ] Étape 0 : relire à la tête du worktree les lignes citées ci-dessus (`AccountAutocomplete.svelte:255-317`,
      `InvoiceForm.svelte:697`, `:877-888`, `:932-933`, `products/+page.svelte:622-627`, `:697-705`, `:732`) ; relancer
      `git grep -n "<AccountAutocomplete" -- frontend/src` et le contrôle des `<form>` ; citer les sorties au Change Log.
- [ ] Bases de la voie 2 (`kesh_36_1`, `kesh_e2e_36_1` sur `127.0.0.1:3307`), `node_modules` par `cp -al`.
- [ ] Tests d'AC1, AC3, AC4, AC5 écrits ; rouges constatés et journalisés ; commit.
- [ ] `AccountAutocomplete.svelte` : `commitDraft()` extraite de `handleBlur` ; index de surbrillance à `-1` sans
      désignation (D1, principe ii) ; *Entrée* selon la table de D1, garde `isComposing` comprise ; région `role="status"` de
      D2 et son `aria-describedby` ; commentaires du fichier mis à jour (`:297-299` dit qu'un envoi sans quitter le champ doit rendre le
      focus d'abord : à compléter par « ou passer par *Entrée*, qui résout le brouillon sans envoyer ») ; doc de la prop
      `allowClear` relue.
- [ ] `messages.ftl` ×4 (D2) ; `sitesTotal` recompté.
- [ ] Verts ; mutations M1 à M15 (M16 par la capture) ; captures (AC7).
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
- **Facture et article : Entrée dans un champ « Compte de produit » vide ne choisit plus d'office le premier compte de la
  liste.** Un compte n'est choisi au clavier que si vous l'avez désigné, en tapant ou avec les flèches ; une ligne laissée
  vide suit le compte de produit par défaut de la société, même après une double Entrée. Le même champ de compte, dans les
  écritures manuelles, l'assistant d'achat et le rapprochement bancaire, suit la même règle ; sur ces sept champs, la
  première flèche vers le bas désigne désormais le premier compte de la liste (auparavant le deuxième), la flèche vers le
  haut ouvre la liste sur le dernier, et le survol de la souris n'a plus la couleur d'un compte désigné. (refs #1165)

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
- **I4 — ouverte en #1165, puis intégrée à la story** (décision de l'orchestrateur, C-36-1-5 ; D1, principe ii ; chemins C4,
  C5). La PR porte `closes #1095` et `closes #1165`.
- **I5 — Mode dégradé** (bug, P4 proposé) : liste des comptes non chargée, un texte non numérique (`abc`) envoie `null`
  (`:243-246`) et reste affiché, au `blur` (`:288`) comme à *Entrée* (`:256`). Voisin de #1085 (le brouillon gardé quand la
  liste revient), distinct. À ouvrir, une issue.
- **#1168 — pas d'`aria-activedescendant`** (validation P2, V2-4 ; ouverte par l'orchestrateur) : la désignation au clavier
  n'est annoncée à aucun lecteur d'écran (`git grep -n "activedescendant" -- frontend/src` ne rend que
  `ContactPicker.svelte:137`). Préexistant, hors périmètre.
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
- Une *Entrée* ne choisit jamais un compte que l'utilisateur n'a pas désigné (saisie non blanche ou flèche) ; sur un champ
  non touché — compte affiché, ou vide et suivant le défaut société —, elle laisse partir le formulaire, qui envoie ce que le
  champ montre.
- Le mode dégradé ne change pas. Rien ne change au serveur. Aucune migration (C-e360-3). Fixtures synthétiques seulement.

### Références

- Issue #1095 (P2, `bug`) ; issue #1165 (la surbrillance non désignée) ; validation P1
  `/home/gcorbaz/devel/kesh-gate-logs/e36-1/validation-p1.md` ; Story 35-7 (#1000 ; D2 : seul un choix explicite sélectionne ; E1 : le `Ctrl+S` de l'écriture
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
- 2026-11-14 — **Validation P1 (Sonnet) : 0/0/2/6 → remédiés** (Opus 5.5, par l'outil Edit ; rapport
  `kesh-gate-logs/e36-1/validation-p1.md`). Prémisses relues au code à `9d5a142df` avant chaque remède :
  - **V1-1 (MEDIUM), retenu** : `:240` `highlightIndex = 0;` dans `handleInput`, `:226` (vingt comptes pour une saisie vide),
    `:266` — vider puis *Entrée* sans *Échap* choisit bien le premier compte. Remède : chemin C5 écrit, D1 réécrit en table
    d'états (E0 à E5), chaque état avec sa règle, son AC et sa mutation : non touché à valeur nulle (E2, AC1 l, M9), non touché
    à valeur résolue (E2, AC1 g, M2), vidé liste ouverte (E5, AC1 n, o, M10), vidé après *Échap* (E5, AC1 e, f, M5).
  - **V1-2 (MEDIUM), retenu** : `:177` `let highlightIndex = $state(0);`, `:327` `onfocus={() => (open = true)}` — la première
    *Entrée* sur un champ vide non touché choisit le premier compte. **#1165 intégrée** (décision de l'orchestrateur,
    C-36-1-5) : principe (ii) de D1, index `-1` sans désignation ; l'affirmation fausse « Entrée sur un champ non touché reste
    le geste d'envoi » retirée de D1 et de *Ce qui doit rester vrai* (remplacée par la règle exacte), AC1 k, l, AC3 c, AC5
    a-bis, M9 à M11 ajoutés ; D3 dit l'effet sur les sept consommateurs et sa couverture (grep des tests au clavier cité) ;
    note de version : un second point `refs #1165` ; I4 renvoie à #1165.
  - **V1-3 (LOW), retenu** : condition d'E5 écrite sur la désignation et non « sans élément en surbrillance » ; pendant le
    chargement (`accounts = []`), E4 bloque sans message (AC1 m, M12).
  - **V1-4 (LOW), retenu** : région `role="status"` toujours présente, contenu variable, au lieu d'un `<p aria-live>` inséré ;
    AC1 a la vérifie.
  - **V1-5 (LOW), retenu** : AC5 a attend le message, lit le compteur après, et relit `GET /invoices/{id}` (compte et
    `version` inchangés) ; AC5 b attend le message.
  - **V1-6 (LOW), retenu** : montage d'AC4 décrit (`products-page.test.ts:86` liste vide, `:44` mock anonyme, bouton
    *Modifier* `+page.svelte:573-574`, `Dialog` `:612-613`), précédent `contacts-page.test.ts:174-182` cité, non démontré par
    exécution ; repli écrit (E2E AC5 b seul, déclaré au Change Log, tuant M1).
  - **V1-7 (LOW), retenu** : garde reprise de `SupplierCreatePanel.svelte:114`
    `if (e.key !== 'Enter' || e.isComposing) return;` ; état E0, AC1 j, M8.
  - **V1-8 (LOW), retenu** : `lint-i18n-ownership.js:16` (lu : `16:const GLOBAL_NAMESPACES = […]`) au lieu de `:12`.
  - Grep de la phrase corrigée (`/usr/bin/grep -n "geste d'envoi"` sur la fiche et le registre) : ne reste que la citation de
    l'ancienne phrase dans cette entrée. Registre : C-36-1-5 ajouté, C-36-1-1 et C-36-1-3 complétés.
- 2026-11-14 — **Validation P2 (Opus) : 0/0/2/7 → remédiés** (Opus 5.5, par l'outil Edit ; rapport
  `kesh-gate-logs/e36-1/validation-p2.md`). Prémisses relues au code à `9d5a142df` :
  - **V2-1 (MEDIUM), retenu** : `262:		} else if (e.key === 'ArrowUp') {` / `264:			highlightIndex = Math.max(highlightIndex - 1, 0);`
    — n'ouvre pas la liste ; `261` `Math.min(highlightIndex + 1, filtered.length - 1)` rend `-1` sur une liste vide. Les deux
    états orphelins existent. Remède : table de D1 **close par construction** (E0, E1, E2, puis tout brouillon empêche
    l'envoi ; E3, E4, E5 partitionnés sur « blanc » et « `filtered` vide » seuls, sans condition sur `open` ni sur l'index) ;
    *flèche haut* qui ouvre la liste sur le dernier élément depuis `-1` ; AC1 p, q ; M13, M14 (double, avec M15 ; M14 seule
    équivalente, dit).
  - **V2-2 (MEDIUM), retenu** : `$effect` `:186-191` et bouton × `:370-371` ne touchent ni `open` ni `highlightIndex`.
    Remède : énumération de **toutes** les transitions de l'index (grep cité en D1) et invariant « index ≥ 0 ⇔ désigné » ;
    recalcul de l'index sur tout changement de la suite des identifiants de `filtered` ou de `value` (× , valeur du parent,
    comptes arrivés, liste rétrécie, frappe) ; `$derived` réassignable recommandé, `$effect` en repli ; AC1 r ; M15.
  - **V2-3 (LOW), retenu** : `396` `hover:bg-accent` lu ; le survol passe à `hover:bg-muted`, D1 le dit, capture d'AC7
    pointeur hors liste et sur une option ; M16 contrôlée par la capture.
  - **V2-4 (LOW)** : #1168, ouverte par l'orchestrateur, citée hors périmètre.
  - **V2-5 (LOW), retenu** : `AccountAutocomplete.test.ts:171-203` (`onSelect` sans réinjection) lu ; montage avec valeur
    réinjectée pour c, e, k, n, p, q, r ; valeur de départ d'AC1 c à `null`.
  - **V2-6 (LOW), retenu** : un seul point d'effacement du message (dérivé de `draft === noMatchDraft`) ; AC1 h en quatre
    variantes (frappe, `blur`, ×, choix) ; M6 redéfinie, tuée par les quatre.
  - **V2-7 (LOW), retenu** : `invoice-revenue-account.spec.ts:115` (lu :
    `115:	const accountFields = page.locator('input[aria-autocomplete="list"]');`) au lieu de `:118` ;
    `JournalEntryForm.svelte:290` (lu : `(document.activeElement as HTMLElement | null)?.blur();`) au lieu de `:289`.
  - **V2-8 (LOW)** : ligne du `sprint-status.yaml` corrigée (validation P1 et P2 remédiées, #1165) ; clé
    `36-1-compte-produit-entree` et nom de fiche gardés tels quels (écart consigné au premier rapport) ; fichier d'epic annoté
    par l'orchestrateur.
  - **V2-9 (LOW), retenu** : la note de version (`refs #1165`) dit la première flèche bas, la flèche haut et le survol pour les
    sept champs.
