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
et n'envoie jamais le formulaire par *Entrée* tant qu'un brouillon est affiché ; une surbrillance ne vaut choix que si elle a été désignée. **S'applique ici**, avec deux différences : (i) un contact ne s'efface pas (pas d'`onClear`, la facture exige un client) :
le brouillon jeté **rend le client retenu, ou le vide s'il n'y en a pas** ; (ii) la liste vient du serveur, avec anti-rebond : *Entrée* pendant la recherche en cours ne choisit rien.

## Décisions

- **D1 — le brouillon se jette au départ du champ** (C-v0190c-1). `draft: string | null` (`null` hors frappe) ; `query = $derived(draft ?? selected?.name ?? '')` — le `$effect` à `initialSyncDone` disparaît ; `pick` fait
  `onSelect(c); draft = null; open = false;` ; **`blur` : `draft = null`** puis fermeture de la liste ; **un changement de `selected?.id` jette le brouillon** (`$effect` sur l'identifiant seul, `untrack` du brouillon : après un 409, le client rechargé s'affiche) ;
  frappe : `draft = valeur`. **Option écartée** : résoudre le texte par une recherche au `blur` (« le premier contact qui correspond ») — un choix que l'écran n'a pas montré, contraire à la règle du `CLAUDE.md` § *Un appariement automatique propose,
  il ne crée jamais* (le faux rattachement est muet). **Option écartée** : un message — le client retenu réaffiché dit déjà ce qui part, comme pour le compte.
- **D2 — *Entrée*** (C-v0190c-2). Ordre : (1) liste ouverte **et** surbrillance désignée **et** recherche courante terminée → l'élément est choisi (`preventDefault`, rien ne part) ; (2) sinon, **un brouillon est affiché** (`draft !== null`) →
  `preventDefault` toujours, la liste s'ouvre ; (3) sinon (champ non touché) l'événement suit son cours : le formulaire part avec le client que le champ montre. *Entrée* maintenue (`e.repeat`) et composition IME (`e.isComposing`, `e.keyCode === 229`) :
  mêmes règles que `AccountAutocomplete.svelte` (à lire, ne pas réinventer). « Recherche courante terminée » : un indicateur `pending` posé à la frappe, levé quand `runSearch` de **cette** séquence (`searchSeq`) rend la main.
- **D3 — la surbrillance ne vaut choix que désignée** (C-v0190c-3). `highlighted` vaut `-1` à l'ouverture, au focus, au `blur`, à *Échap* et au choix ; il passe à `0` quand la saisie est **non blanche** et la liste non vide, ou par une flèche ;
  *flèche bas* depuis `-1` va à `0` ; *flèche haut* depuis `-1` va au dernier. Rendu : `ContactPicker` emploie `hover:bg-muted` / `bg-muted` (`:159`, `:161`), classes que le thème ne génère pas (`/usr/bin/grep -nE "muted|accent" frontend/src/app.css` ne rend que
  `--color-text-muted`, `:32`, et un commentaire `:41`) : **réemployer** `CLASSE_DESIGNEE` / `CLASSE_SURVOL` de `frontend/src/lib/features/journal-entries/account-autocomplete-classes.ts` (présent : `git cat-file -e HEAD:…` vérifié).
- **Hors périmètre** : les deux filtres de liste n'ont pas de bouton d'effacement du client — **issue #1228**.

## Critères d'acceptation

1. **AC1 — le composant** (`frontend/src/lib/components/invoices/ContactPicker.test.ts`, neuf ; mock de `listContacts` ; l'événement constaté par la valeur rendue de `fireEvent.keyDown`, `false` quand `preventDefault` a été appelé ; les cas qui constatent
   l'affichage après un choix réinjectent `selected` comme un parent, par un `onSelect` espion qui appelle `rerender`) :
   - a. `selected = Dupont`, taper « zzz », `blur` → le champ montre « Dupont » ; `onSelect` jamais appelé ;
   - b. `selected = null`, taper « zzz », `blur` → le champ est vide ;
   - c. `selected = Dupont`, taper « Mar », *Entrée* (liste fermée par *Échap*, ou aucun résultat) → événement empêché ; `onSelect` jamais appelé ; le texte tapé reste ; la liste est affichée ;
   - d. *Entrée* **pendant l'anti-rebond** (temps simulé, 100 ms après la frappe, anciens résultats présents) → événement empêché, `onSelect` jamais appelé ; la recherche terminée, *Entrée* choisit le premier résultat de **la nouvelle** liste ;
   - e. focus sans saisie, liste affichée, *Entrée* → rien n'est choisi, événement **non** empêché (aucun brouillon : le formulaire part avec le client affiché) ; *flèche bas* → le **premier** contact est désigné, *Entrée* le choisit ;
   - f. choix à la souris (`mousedown`) puis `blur` → le champ montre le contact choisi (parent réinjecté) ;
   - g. `selected` change de client pendant qu'un brouillon est affiché → le champ montre le nouveau client ; `selected` passe à `null` → champ vide ;
   - h. composition IME et touche maintenue : mêmes issues que `AccountAutocomplete` (un cas chacun).
2. **AC2 — la facture** (`InvoiceForm.test.ts`, un test) : facture existante au client X ; taper « zzz » dans le champ client puis cliquer *Enregistrer* → `updateInvoice` reçoit le `contactId` de X et le champ montre X. Les tests existants du formulaire restent verts (montage du client initial, `:380-384`).
3. **AC3 — les filtres** : les deux pages de liste restent vertes ; un filtre posé se réaffiche tel quel après un texte tapé non choisi (AC1 a, même composant).
4. **AC4 — E2E** (`frontend/tests/e2e/invoices.spec.ts`, un test) : nouvelle facture, client choisi dans la liste ; taper un autre texte dans le champ client, *Entrée* → l'URL reste `/invoices/new` ; quitter le champ → il montre le client choisi ; *Créer la facture* → la facture porte ce client (liste des factures).
5. **AC5 — aucune clé i18n neuve, aucune migration, aucun changement serveur.**
6. **AC6 — le manuel** (`docs/manual/fr/user-manual.tex`, § création d'une facture, `:1162` *Factures → Nouvelle facture. Renseigner :*) : une phrase — le client se choisit **dans la liste** ; un texte tapé et non choisi est abandonné en quittant le champ, qui réaffiche le client retenu. `.tex` seulement.

## Tâches

1. `ContactPicker.svelte` : D1, D2, D3 (lire d'abord `AccountAutocomplete.svelte`, `handleKeydown` et `commitDraft`, à la tête du worktree).
2. `ContactPicker.test.ts` (AC1) ; test de `InvoiceForm.test.ts` (AC2) ; E2E (AC4) et captures ; manuel (AC6) ; section `## Note de version` ci-dessous (assemblée par l'orchestrateur, la story n'écrit pas `CHANGELOG.md`).

## Gates

Ceux de `e35-dev-commun.md` : gate ciblé (`npm run check`, Vitest de `ContactPicker`, `InvoiceForm`, pages de liste) en cours de développement ; `test-fast --ci` et `gate-ci` au passage en `review` ; `gate-ci` et E2E complet à la clôture, au dernier commit de code.

## Recette visuelle (captures, thème clair, base de démonstration, `kesh-gate-logs/v0190c/`)

Champ client avec un texte tapé non choisi ; le même champ après avoir quitté le champ (client retenu réaffiché) ; liste avec une surbrillance désignée par flèche.

## Dev Notes

- Taille : un composant, trois fichiers de test, un manuel ; petite.
- Le détecteur du symptôme (`git grep -n -l -E 'aria-autocomplete|role="combobox"|role="listbox"' -- frontend/src`) : avant correction il rend `ContactPicker` (sonde positive) et `AccountAutocomplete` ; après, plus aucun composant sans la règle.
- Issue liée : **#1228** (filtres sans effacement). Aucune donnée réelle dans les fixtures (`Dupont`).

## Note de version

**Corrigé**
- Le champ « client » d'une facture ne montre plus un client que la facture n'enregistrera pas : un texte tapé sans choisir un client de la liste est abandonné en quittant le champ (le client retenu s'affiche), et la touche *Entrée* ne fait plus partir la facture tant qu'un texte tapé n'est pas résolu, ni ne choisit un client que vous n'avez pas désigné (refs #1162).
