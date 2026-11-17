# Story v0190-correction : le client que montre le champ est celui qui part (#1162)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 (Sonnet 5.5), worktree `kesh-v0190c`, branche `story/v0190-correction`, partie de
`epic/v0190-correction` (= `main` = v0.18.0, `8b50fd195`), par le workflow `bmad-create-story`. Release **0.19.0** (publiée le
2026-11-20, coupure le 2026-11-19 au soir). **Frontend seul : aucune migration, aucun changement serveur, aucune clé i18n.** Voie de développement :
voie 2, clé `v0190c`. Ordre de priorité de la release : rang 3 ; Guy a accepté qu'elle glisse à la 0.20.0. **Scindée le 2026-11-17** (règle de splitting :
six modules) : la partie #1097 (nom de la société modifiable) est la fiche sœur `v0190-nom-societe.md`. Aucun fichier de code n'est commun aux deux.*

## Story

En tant que titulaire qui monte sa première facture client, je veux que le champ « client » ne montre jamais un client différent de celui que la facture enregistrera,
afin de ne pas émettre une facture à la mauvaise personne.

## Ce qui se lit au code (`8b50fd195`)

`frontend/src/lib/components/invoices/ContactPicker.svelte` (`selected`, `onSelect`, `placeholder` ; **aucune** clé i18n : « Rechercher un contact… », « Chargement… », « Aucun contact » sont écrits en français dans le composant) :

- le texte affiché est un état propre, `:29` `let query = $state('');`, rempli une fois depuis `selected` par un `$effect` (`:44` `let initialSyncDone = $state(false);` … `:50` `if (!initialSyncDone && query === '') {` / `:51` `query = selected.name;`) ;
- la frappe remplace `query` sans toucher à `selected` (`:73` `function onInput(e: Event) {`) et lance la recherche après 300 ms (`:77` `debounceHandle = setTimeout(() => runSearch(query.trim()), 300);`) ;
- **le défaut** : `:110` `function onBlur() {` / `:112` `setTimeout(() => (open = false), 150);` — le départ du champ ferme la liste et ne résout rien ; `query` garde le texte tapé, `selected` (donc l'ancien client) reste celui que le parent envoie ;
- *Entrée* ne fait quelque chose que si la liste est ouverte avec une surbrillance : `:98` `} else if (e.key === 'Enter' && highlighted >= 0 && results[highlighted]) {` ; sinon l'événement suit son cours et le navigateur soumet le formulaire (soumission implicite, sans `blur`) ;
- **la surbrillance n'est pas une désignation** : `runSearch` pose `highlighted = r.items.length > 0 ? 0 : -1` (`:63`) quelle que soit la saisie, et le focus lance `runSearch('')` (`:128` `onfocus={() => {` / `:130` `if (results.length === 0) runSearch('');`) : *Entrée* dans un champ sans saisie choisit le premier contact de la liste ; *Entrée* pendant l'anti-rebond choisit dans les résultats de la recherche **précédente** (même défaut que #1165 pour `AccountAutocomplete`, que la 36-1 a fermé) ;
- *flèche bas* depuis `-1` va à `1` (`:90` `highlighted = Math.min(results.length - 1, Math.max(0, highlighted) + 1);`) : le premier contact est sauté.

Les trois consommateurs (`git grep -n "<ContactPicker" -- frontend/src`) : `frontend/src/lib/components/invoices/InvoiceForm.svelte:790` `<ContactPicker selected={selectedContact} onSelect={onContactSelect} />`
**dans un `<form>`** (`:773` `<form onsubmit={onSubmit} class="space-y-6">`, bouton `:1044` `type="submit"`) ; `frontend/src/routes/(app)/invoices/+page.svelte:338` et
`frontend/src/routes/(app)/invoices/due-dates/+page.svelte:517`, deux **filtres** (`selected={contactFilter}`), hors formulaire.

Ce qui part : `InvoiceForm.svelte:665` `contactId: selectedContact!.id,` ; validation `:538` `if (!selectedContact) return 'Veuillez sélectionner un contact';`. Le parent remet le client à `null` ou à un autre après un 409
(`:745` `selectedContact = c;`, `:748` et `:751` `selectedContact = null;`) : le champ doit suivre.

**La règle de la 36-1, reprise** (`_bmad-output/implementation-artifacts/36-1-entree-compte-produit.md`, D1) : `AccountAutocomplete` montre le brouillon pendant la frappe (`const query = $derived(draft ?? displayLabel);`), le jette au `blur` (`commitDraft()`),
et n'envoie jamais le formulaire par *Entrée* tant qu'un brouillon est affiché ; une surbrillance ne vaut choix que si elle a été désignée. **S'applique ici**, avec trois différences : (i) un contact ne s'efface pas (pas d'`onClear`, la facture exige un client) :
le brouillon jeté **rend le client retenu, ou le vide s'il n'y en a pas** ; (ii) la liste vient du serveur, avec anti-rebond : *Entrée* pendant la recherche en cours ne choisit rien ; (iii) le brouillon blanc + *Entrée* est jeté comme à la 36-1 (E5), mais le champ réaffiche le client retenu (D2).

## Décisions

- **D1 — le brouillon se jette au départ du champ** (C-v0190c-1). `draft: string | null` (`null` hors frappe) ; `query = $derived(draft ?? selected?.name ?? '')` — le `$effect` à `initialSyncDone` disparaît ; `pick` fait
  `onSelect(c); draft = null; open = false;` ; **`blur` : `draft = null`**, anti-rebond annulé, `searchSeq++` (une réponse tardive est ignorée), `pending` remis à faux, puis fermeture de la liste (`pick` fait de même, D2) ; **un changement de `selected?.id` jette le brouillon** (`$effect` sur l'identifiant seul, `untrack` du brouillon : après un 409, le client rechargé s'affiche) ;
  frappe : `draft = valeur`. **Au `blur` et au choix, `loading` revient aussi à faux** (V2-3 ; sinon une recherche en vol dont la réponse est ignorée laisserait « Chargement… » affiché à vie au focus suivant) et **`results` est vidé**, de sorte que le focus d'un
  champ non touché relance `runSearch('')` (`:130` `results.length === 0`) et n'affiche jamais les résultats d'une recherche abandonnée. **Un changement de `selected?.id` purge aussi la désignation et les résultats périmés** (V2-5) : le même `$effect` remet
  `highlighted = -1`, `open = false`, `pending = false`, `loading = false`, annule l'anti-rebond et vide `results` (course au chargement d'une facture, `InvoiceForm.svelte:380-384` : le champ ne doit pas rester ouvert sur la liste d'un brouillon jeté, dont *Entrée* choisirait l'option 0). **Option écartée** : résoudre le texte par une recherche au `blur` (« le premier contact qui correspond ») — un choix que l'écran n'a pas montré, contraire à la règle du `CLAUDE.md` § *Un appariement automatique propose,
  il ne crée jamais* (le faux rattachement est muet). **Option écartée** : un message — le client retenu réaffiché dit déjà ce qui part, comme pour le compte.
- **D2 — *Entrée*** (C-v0190c-2). Ordre : (1) liste ouverte **et** surbrillance désignée **et** recherche courante terminée → l'élément est choisi (`preventDefault`, rien ne part) ; (2) sinon, **un brouillon est affiché** (`draft !== null`) →
  `preventDefault` toujours, la liste s'ouvre ; (3) sinon (champ non touché) l'événement suit son cours : le formulaire part avec le client que le champ montre. *Entrée* maintenue (`e.repeat`) et composition IME (`e.isComposing`, `e.keyCode === 229`) :
  mêmes règles que `AccountAutocomplete.svelte` (à lire, ne pas réinventer). **Brouillon blanc** (C-v0190c-11, validation P1 V1-6) : *Entrée* avec `draft.trim() === ''` jette le brouillon, ferme la liste, laisse le champ réafficher le client retenu
  et empêche l'envoi ; l'*Entrée* suivante envoie (patron E5 d'`AccountAutocomplete.svelte:398-402`, « troisième différence » avec la 36-1 : la règle reprise a donc **trois** différences, non deux).
  **« Recherche courante terminée »** (C-v0190c-12, validation P1 V1-1) : `searchSeq` n'avance aujourd'hui qu'au départ de `runSearch` (`ContactPicker.svelte:57`), 300 ms après la frappe : une réponse partie **avant** la dernière frappe
  serait donc acceptée (`:61`) et désignerait un contact de l'ancienne recherche. Règle : **chaque frappe invalide toute recherche en vol** (`searchSeq++` dans `onInput`, avant l'armement de l'anti-rebond) ; `pending` est posé à la frappe et ne se lève
  qu'à la réponse de la recherche lancée pour le brouillon **actuel** (la séquence courante) ; **au `blur` et au choix, l'anti-rebond est annulé et `pending` revient à faux** (sinon il resterait vrai au focus suivant et *Entrée* serait empêché ou, pire,
  la désignation ignorée). **Un échec de `listContacts` lève `pending` comme une réponse** (V2-4) : liste « Aucun contact », *Entrée* toujours empêchée tant que le brouillon est affiché (règle 2), mais plus de `pending` fantôme. **La frappe efface la désignation** (V2-2) :
  `onInput` pose `highlighted = -1` en même temps que `searchSeq++` et `pending = true` ; pendant l'anti-rebond et la recherche, aucune option ne porte `CLASSE_DESIGNEE` (l'écran ne montre pas une désignation que la touche ignore), et une flèche pendant `pending` ne désigne
  rien dans l'ancienne liste (elle ne fait qu'ouvrir la liste, sans désignation, tant que `pending` est vrai). **`mousedown` empêche le défaut** (V2-7) : `e.preventDefault()` sur le `mousedown` de la liste (`role="listbox"`, barre de défilement comprise) et de chaque option, comme
  `AccountAutocomplete` (`onmousedown={(e) => { e.preventDefault(); handleSelect(acc); }}`), pour que le champ garde le focus : sans lui, le `blur` immédiat de D1 jetterait le brouillon et fermerait la liste au clic sur la barre de défilement (`limit: 50`, `max-h-64`).
- **D3 — la surbrillance ne vaut choix que désignée** (C-v0190c-3). `highlighted` vaut `-1` à l'ouverture, au focus, au `blur`, à *Échap* et au choix ; il passe à `0` quand le **brouillon** est non blanc (`draft !== null && draft.trim() !== ''`, **non** le texte affiché :
  `query` vaut le nom du client sur un champ non touché ; patron `indexParDefaut`, `AccountAutocomplete.svelte:277-278`) et la liste non vide, ou par une flèche ; **cette désignation par la saisie se calcule à l'arrivée de la réponse, sur le `draft` courant** (V2-4 : pas
  celui du départ de `runSearch`) : une réponse arrivant après un `blur` (recherche déjà partie, que l'annulation du minuteur n'arrête pas) ne désigne rien sur un champ non touché ; elle est de plus ignorée, car `blur` fait avancer `searchSeq` ;
  *flèche bas* depuis `-1` va à `0` ; *flèche haut* depuis `-1` va au dernier. **Flèches liste fermée** (C-v0190c-13, V1-7) : flèche bas / haut **ouvrent** la liste et désignent le premier / le dernier (patron `AccountAutocomplete.svelte:328-338`) ; la garde
  `ContactPicker.svelte:87` `if (!open) return;` ne s'applique plus à elles. Rendu : `ContactPicker` emploie `hover:bg-muted` / `bg-muted` (`:159`, `:161`), classes que le thème ne génère pas (`/usr/bin/grep -nE "muted|accent" frontend/src/app.css` ne rend que
  `--color-text-muted`, `:32`, et un commentaire `:41`) ; **sous-ligne du numéro IDE** (V2-6) : `text-text-muted` (`#64748b`) sur `bg-primary` (`#1e40af`) donne ≈ 1,8:1, le numéro disparaît sur l'option désignée ; la sous-ligne
  prend `text-primary-foreground` quand `i === highlighted`, `text-text-muted` sinon, avec un **contraste d'au moins 4,5:1** (calcul WCAG des deux jetons de `app.css`, écrit dans le test) ; **réemployer** `CLASSE_DESIGNEE` / `CLASSE_SURVOL` de `frontend/src/lib/features/journal-entries/account-autocomplete-classes.ts` (présent : `git cat-file -e HEAD:…` vérifié) ;
  l'import traverse `lib/features/journal-entries` depuis `lib/components/invoices` : couplage transverse **assumé**, le déplacement des constantes est hors périmètre.
- **Hors périmètre** : les deux filtres de liste n'ont pas de bouton d'effacement **du seul client** (la page des échéances n'en a aucun) — **issue #1228**. **Constats voisins, non traités ici** : `ProductPicker.svelte:108` emploie aussi `hover:bg-muted`, classe que le thème ne génère pas — **issue #1231** ;
  `InvoiceForm.svelte:789` `<label for="invoice-contact">` ne désigne aucun champ (`ContactPicker` ne porte pas d'`id`) — **issue #1232**. La page des factures a, elle, un chemin de remise à `null` : le bouton *Réinitialiser*
  (`invoices/+page.svelte:375-380`, `contactFilter = null;`) ; après lui, le champ gardait l'ancien nom (`ContactPicker.svelte:46-48`), même classe de défaut, fermée par D1 (AC1 g, AC3).

## Critères d'acceptation

1. **AC1 — le composant** (`frontend/src/lib/components/invoices/ContactPicker.test.ts`, neuf ; mock de `listContacts` ; l'événement constaté par la valeur rendue de `fireEvent.keyDown`, `false` quand `preventDefault` a été appelé ; les cas qui constatent
   l'affichage après un choix réinjectent `selected` comme un parent, par un `onSelect` espion qui appelle `rerender`) :
   - a. `selected = Dupont`, taper « zzz », `blur` → le champ montre « Dupont » ; `onSelect` jamais appelé ;
   - b. `selected = null`, taper « zzz », `blur` → le champ est vide ;
   - c. `selected = Dupont`, taper « Mar », *Entrée* (liste fermée par *Échap*, ou aucun résultat) → événement empêché ; `onSelect` jamais appelé ; le texte tapé reste ; la liste est affichée ;
   - d. *Entrée* **pendant l'anti-rebond** (temps simulé, 100 ms après la frappe, anciens résultats présents) → événement empêché, `onSelect` jamais appelé ; la recherche terminée, *Entrée* choisit le premier résultat de **la nouvelle** liste ;
   - d-bis. (V1-1) recherche « M » **en vol** (promesse non résolue), frappe « a » (anti-rebond armé), la réponse « M » arrive **pendant l'anti-rebond** → *Entrée* empêchée, `onSelect` jamais appelé, aucune option désignée (aucune ne porte `CLASSE_DESIGNEE`) ;
   - d-ter. (V1-1) frappe, `blur` pendant l'anti-rebond, focus, flèche bas, *Entrée* → le contact désigné est choisi (`pending` est revenu à faux au `blur`) ;
   - d-quater. (V2-2) « Ma » + réponse (option 0 désignée), frappe « r » → **pendant l'anti-rebond et la recherche, aucune option ne porte `CLASSE_DESIGNEE`** ; flèche bas à ce moment ne désigne rien dans l'ancienne liste ; *Entrée* empêchée ;
   - d-quinquies. (V2-4) (i) `blur` **pendant qu'une recherche est en vol**, réponse arrivée après le `blur`, focus, *Entrée* → rien n'est choisi, aucune option désignée ; (ii) `listContacts` rejette → `pending` faux, liste « Aucun contact », *Entrée* empêchée tant que le brouillon est affiché ;
   - d-sexies. (V2-3) frappe pendant qu'une recherche est en vol, `blur` avant les 300 ms, focus → la liste n'affiche pas « Chargement… » et relance la recherche du champ non touché ;
   - e. focus sans saisie, liste affichée, *Entrée* → rien n'est choisi, événement **non** empêché (aucun brouillon : le formulaire part avec le client affiché) ; *flèche bas* → le **premier** contact est désigné, *Entrée* le choisit ; la réponse de la recherche du focus, arrivant sur un champ **non touché** dont le nom est non blanc, ne désigne rien (V1-5) ;
   - e-bis. (V1-7) liste fermée par *Échap* : flèche bas rouvre la liste et désigne le premier ; flèche haut, le dernier ;
   - e-ter. (V1-6) brouillon blanc (champ vidé au clavier) + *Entrée* → événement empêché, brouillon jeté, liste fermée, le champ réaffiche le client retenu ; l'*Entrée* suivante n'est pas empêchée ;
   - f. choix à la souris (`mousedown`) puis `blur` → le champ montre le contact choisi (parent réinjecté) ;
   - f-bis. (V1-11) la désignation **rendue** : l'option désignée porte `CLASSE_DESIGNEE`, les autres `CLASSE_SURVOL`, aucune ne porte `bg-muted` (la garde de jetons ne lit que les constantes) ;
   - f-ter. (V2-6) un contact à numéro IDE : sur l'option désignée la sous-ligne porte `text-primary-foreground` (et non `text-text-muted`), sur les autres `text-text-muted` ; le contraste calculé de `--color-primary-foreground` sur `--color-primary` (`app.css`) est d'au moins 4,5:1 ;
   - f-quater. (V2-7) `mousedown` sur le `role="listbox"` (hors option) et sur une option : événement empêché, le brouillon n'est pas jeté, la liste reste ouverte ;
   - g. `selected` change de client pendant qu'un brouillon est affiché → le champ montre le nouveau client ; `selected` passe à `null` → champ vide (chemin du bouton *Réinitialiser* de la liste des factures) ; **(V2-5)** liste ouverte avec l'option 0 désignée, `selected` change → la liste est fermée, aucune option désignée, *Entrée* juste après n'est pas un choix ;
   - h. composition IME et touche maintenue : mêmes issues que `AccountAutocomplete` (un cas chacun).
2. **AC2 — la facture** (`InvoiceForm.test.ts`, un test) : facture existante au client X ; `fireEvent.input` « zzz » dans le champ client, **`fireEvent.blur`** (en jsdom le clic ne déplace pas le focus : sans `blur` explicite le champ montrerait « zzz » même avec le correctif),
   puis `fireEvent.click` sur *Enregistrer* (patron `InvoiceForm.test.ts:336-340`) → `updateInvoice` reçoit le `contactId` de X et le champ montre X. **L'assertion qui prouve le rouge est l'affichage de X** : le `contactId` de X part déjà aujourd'hui (`selectedContact` n'est jamais touché par la frappe).
   Le mock `vi.mock('$lib/features/contacts/contacts.api', …)` (`:39-41`) reçoit `listContacts` (la frappe lance `runSearch`). Les tests existants du formulaire restent verts (le montage du client initial est `InvoiceForm.svelte:380-384`, non le fichier de test).
3. **AC3 — les filtres** : les deux pages de liste restent vertes ; un filtre posé se réaffiche tel quel après un texte tapé non choisi (AC1 a, même composant) ; le chemin *Réinitialiser* de la page des factures (`contactFilter = null`, AC1 g au composant) est le seul chemin de remise à `null` : **un test de page est exigé** (V2-8), dans le patron existant de `invoices/+page.test.ts` : *Réinitialiser* remet le champ client à vide (filet du parent, en plus d'AC1 g).
4. **AC4 — E2E** (`frontend/tests/e2e/invoices.spec.ts`, un test) : nouvelle facture, client choisi dans la liste ; **les lignes (description, quantité, prix, comme `invoices.spec.ts:486-492`) sont remplies AVANT le texte tapé dans le champ client**, de sorte que le formulaire soit valide à l'*Entrée*
   (sinon `InvoiceForm.svelte:538` `Veuillez sélectionner un contact` refuserait l'envoi et l'URL resterait `/invoices/new` avec l'ancien code aussi : le test serait vert avec le défaut, V2-1) ; **aucune requête `POST /api/v1/invoices` n'est partie** (compteur sur `page.on('request')`, plus robuste que l'URL seule) ; taper un texte **qui ne correspond à aucun contact** (suffixe unique, `zzz-<uniq>`) dans le champ client ; **attendre que la liste affiche « Aucun contact »** avant d'appuyer sur *Entrée* (l'ancien code laisse alors partir le formulaire : l'URL change, le test rougit sans le correctif — avec une *Entrée* immédiate l'ancien code rechoisit le client déjà choisi et le test passerait avec le défaut) → l'URL reste `/invoices/new` ;
   quitter le champ → il montre le client choisi ; *Créer la facture* → la facture porte ce client (liste des factures). Variante facultative : *Entrée* immédiate (pendant l'anti-rebond) avec un client choisi qui n'est **pas** le premier de la liste précédente.
5. **AC5 — aucune clé i18n neuve, aucune migration, aucun changement serveur.**
6. **AC6 — le manuel** (`docs/manual/fr/user-manual.tex`, § création d'une facture, `:1162` *Factures → Nouvelle facture. Renseigner :*) : **la parenthèse « (ou création à la volée) » de `:1167` est fausse** (`\item \textbf{Client} : sélection dans le carnet d'adresses (ou création à la volée).` ; aucune création depuis ce champ :
   `git grep -n -i -E "volée|nouveau contact|createContact" -- frontend/src/lib/components/invoices/ 'frontend/src/routes/(app)/invoices/'` ne rend rien) : **elle est remplacée** par la règle — le client se choisit **dans la liste** ; un texte tapé et non choisi est abandonné en quittant le champ, qui réaffiche le client retenu ;
   un client absent se crée d'abord dans *Contacts*. Contrôle : `git grep -n "à la volée" -- docs/manual/fr/user-manual.tex` ne rend plus que `:663` et `:689` (les rapports). `.tex` seulement ; **le PDF ne se commite pas dans la branche** : il se régénère à la release, et le contrôle aplati
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'ou création à la volée'` → `0`) se fait alors, à la charge de l'orchestrateur.

## Tâches

1. `ContactPicker.svelte` : D1, D2 (dont brouillon blanc, invalidation par la frappe, `pending` remis à faux au `blur` et au choix), D3 (lire d'abord `AccountAutocomplete.svelte`, `handleKeydown`, `handleEnter`, `commitDraft` et `indexParDefaut`, à la tête du worktree).
2. `ContactPicker.test.ts` (AC1, dont d-bis à d-sexies, e-bis, e-ter, f-bis à f-quater) ; test de `InvoiceForm.test.ts` (AC2, `listContacts` au mock) ; test de page de `invoices/+page.test.ts` (AC3) ; E2E (AC4) — **à sa première exécution, constater le rouge sur le code d'origine** (composant non corrigé, lignes remplies avant la frappe) et l'écrire au Dev Agent Record ; captures (la liste avec surbrillance désignée porte un contact à numéro IDE) ; manuel (AC6, `.tex` seul) ; section `## Note de version` ci-dessous (assemblée par l'orchestrateur, la story n'écrit pas `CHANGELOG.md`).

## Jalon de coupure

**Mercredi 2026-11-18, 18 h** : si la story n'est pas close à ce moment-là (code et tests de l'AC1 à AC3 écrits, gate ciblé vert), elle **sort de la 0.19.0** et passe à la 0.20.0 (décision de l'orchestrateur du 2026-11-17 ; Guy a accepté qu'elle glisse). La coupure officielle reste le jeudi 2026-11-19 au soir pour ce qui est clos.

## Gates

Ceux de `e35-dev-commun.md` : gate ciblé (`npm run check`, Vitest de `ContactPicker`, `InvoiceForm`, pages de liste) en cours de développement ; `test-fast --ci` et `gate-ci` au passage en `review` ; `gate-ci` et E2E complet à la clôture, au dernier commit de code.

## Recette visuelle (captures, thème clair, base de démonstration, `kesh-gate-logs/v0190c/`)

Champ client avec un texte tapé non choisi ; le même champ après avoir quitté le champ (client retenu réaffiché) ; liste avec une surbrillance désignée par flèche.

## Dev Notes

- Taille : un composant, trois fichiers de test, un manuel ; petite.
- Le détecteur du symptôme, restreint aux composants : `git grep -n -l -E 'aria-autocomplete|role="combobox"|role="listbox"' -- 'frontend/src/*.svelte'` rend **deux** fichiers, `ContactPicker.svelte` (sonde positive) et `AccountAutocomplete.svelte` (sans la restriction, huit fichiers dont six de test ou de constantes) ;
  et `git grep -n -l -E "onSelect" -- 'frontend/src/*.svelte'` ne montre aucun autre sélecteur à texte tapé (`ProductPicker` est une boîte de dialogue à clic). **Liste fermée** : les deux composants à texte tapé et choix dans une liste sont `ContactPicker` et `AccountAutocomplete` ; la règle (brouillon jeté au `blur`, aucune surbrillance qui vaille choix sans désignation) s'applique aux deux, et à tout composant qui s'y ajouterait.
- Issues liées : **#1228** (filtres sans effacement du seul client), **#1231** (`ProductPicker`, `hover:bg-muted`), **#1232** (`<label for>` sans champ). Aucune donnée réelle dans les fixtures (`Dupont`).
- **Registre des choix** : `_bmad-output/implementation-artifacts/e0190-choix-autonomes-v0190c.md` (C-v0190c-1 à 13).

## Note de version

**Corrigé**
- Le champ « client » d'une facture ne montre plus un client que la facture n'enregistrera pas : un texte tapé sans choisir un client de la liste est abandonné en quittant le champ (le client retenu s'affiche), et la touche *Entrée* ne fait plus partir la facture tant qu'un texte tapé n'est pas résolu, ni ne choisit un client que vous n'avez pas désigné (refs #1162).
