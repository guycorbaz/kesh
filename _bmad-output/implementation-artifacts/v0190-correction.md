# Story v0190-correction : le client que montre le champ est celui qui part (#1162), et le nom de la société se modifie dans *Paramètres → Organisation* (#1097)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 (Sonnet 5.5), worktree `kesh-v0190c`, branche `story/v0190-correction`, partie de
`epic/v0190-correction` (= `main` = v0.18.0, `8b50fd195`), par le workflow `bmad-create-story`. Release **0.19.0** (publiée le
2026-11-20, coupure le 2026-11-19 au soir). **Aucune migration** : `companies.name` existe (`VARCHAR(255) NOT NULL`,
`crates/kesh-db/migrations/20260404000001_initial_schema.sql`, ligne `6:    name VARCHAR(255) NOT NULL,`), `CompanyUpdate`
porte déjà `name` et `companies::update_in_tx` l'écrit déjà (`crates/kesh-db/src/repositories/companies.rs`, `SET name = ?, first_name = ?, …`).
Aucun changement du schéma de test, de `docs/migrations-idempotence-audit.md` ni de `post_restore.rs`. Voie de développement : **voie 2**,
clé `v0190c`. Ordre de priorité de la release : rang 3 (après l'Epic 43 et la 42-1) ; **Guy a accepté que la story glisse à la 0.20.0**.
Elle se compose de **deux parties indépendantes** (A : #1162 ; B : #1097) qui ne partagent aucun fichier de code : B se détache sans
rien changer à A (cf. *Découpage de repli*).*

## Story

En tant que titulaire qui monte sa première facture client, je veux que le champ « client » ne montre jamais un client différent de
celui que la facture enregistrera, afin de ne pas émettre une facture à la mauvaise personne (#1162) ; et, en tant
qu'administrateur dont le nom saisi à l'assistant est refusé par la QR-facture, je veux corriger le nom de la société dans
*Paramètres → Organisation*, afin de ne pas rester sans remède devant un PDF qui se refuse (#1097).

## Ce qui se lit au code (`8b50fd195`)

### A — `ContactPicker` (#1162)

`frontend/src/lib/components/invoices/ContactPicker.svelte` (`selected`, `onSelect`, `placeholder` ; **aucune** clé i18n : ses textes
« Rechercher un contact… », « Chargement… », « Aucun contact » sont écrits en français dans le composant) :

- le texte affiché est un état propre, `:29` `let query = $state('');`, rempli une fois depuis `selected` par un `$effect`
  (`:44` `let initialSyncDone = $state(false);` … `:50` `if (!initialSyncDone && query === '') {` / `:51` `query = selected.name;`) ;
- la frappe remplace `query` sans toucher à `selected` (`:73` `function onInput(e: Event) {`) et lance la recherche après
  300 ms (`:77` `debounceHandle = setTimeout(() => runSearch(query.trim()), 300);`) ;
- **le défaut** : `:110` `function onBlur() {` / `:112` `setTimeout(() => (open = false), 150);` — le départ du champ ferme la
  liste et ne résout rien ; `query` garde le texte tapé, `selected` (donc l'ancien client) reste celui que le parent envoie ;
- *Entrée* ne fait quelque chose que si la liste est ouverte avec une surbrillance : `:98` `} else if (e.key === 'Enter' && highlighted >= 0 && results[highlighted]) {` ; sinon l'événement
  suit son cours et le navigateur soumet le formulaire (soumission implicite, sans `blur`) ;
- **la surbrillance n'est pas une désignation** : `runSearch` pose `highlighted = r.items.length > 0 ? 0 : -1` (`:63`) quelle que
  soit la saisie, et le focus lance `runSearch('')` (`:128` `onfocus={() => {` / `:130` `if (results.length === 0) runSearch('');`) :
  *Entrée* dans un champ sans saisie choisit le premier contact de la liste ; *Entrée* pendant l'anti-rebond choisit dans les
  résultats de la recherche **précédente** (même défaut que #1165 pour `AccountAutocomplete`, que la 36-1 a fermé) ;
- *flèche bas* depuis `-1` va à `1` (`:90` `highlighted = Math.min(results.length - 1, Math.max(0, highlighted) + 1);`) : le premier
  contact est sauté.

Les trois consommateurs (`git grep -n "<ContactPicker" -- frontend/src`) :
`frontend/src/lib/components/invoices/InvoiceForm.svelte:790` `<ContactPicker selected={selectedContact} onSelect={onContactSelect} />`
**dans un `<form>`** (`:773` `<form onsubmit={onSubmit} class="space-y-6">`, bouton `:1044` `type="submit"`) ;
`frontend/src/routes/(app)/invoices/+page.svelte:338` et `frontend/src/routes/(app)/invoices/due-dates/+page.svelte:517`, deux
**filtres** (`selected={contactFilter}`), hors formulaire.

Ce qui part : `InvoiceForm.svelte:665` `contactId: selectedContact!.id,` ; la validation `:538` `if (!selectedContact) return 'Veuillez sélectionner un contact';`.
Le parent remet le client à `null` ou à un autre après un 409 (`:745` `selectedContact = c;`, `:748` et `:751` `selectedContact = null;`) :
le champ doit suivre.

**La règle de la 36-1, reprise** (`_bmad-output/implementation-artifacts/36-1-entree-compte-produit.md`, D1) :
`AccountAutocomplete` montre le brouillon pendant la frappe (`const query = $derived(draft ?? displayLabel);`), le jette au `blur`
(`commitDraft()`), et n'envoie jamais le formulaire par *Entrée* tant qu'un brouillon est affiché ; une surbrillance ne vaut choix
que si elle a été désignée (saisie non blanche ou flèche). **S'applique ici**, avec deux différences : (i) un contact ne s'efface
pas (le parent n'a pas d'`onClear`, la facture exige un client) : le brouillon jeté **rend le client retenu, ou le vide s'il n'y en a pas** ;
(ii) la liste vient du serveur, avec anti-rebond : *Entrée* pendant la recherche en cours ne choisit rien.

### B — le nom de la société (#1097)

- Écran : `frontend/src/routes/(app)/settings/+page.svelte:640` `<dd>{data.company.name}</dd>` (section *Organisation*, sans édition) ; modèle à
  suivre : le bloc e-mail — état `:230-236` (`emailEditing`, `emailBase`, `emailVersion`, `emailConflict`), `startEmailEdit`, `saveEmail` (appelle
  `updateCompanyEmail`, puis `adoptCompany`, et sur `OPTIMISTIC_LOCK_CONFLICT` `reloadAfterConflict`, patron de la 34-6) ; `const EMAIL_FIELDS = ['email'] as const;`
  `:62`, `isAdmin` `:229`. API : `frontend/src/lib/features/settings/settings.api.ts:39` `updateCompanyAddress` (modèle), types
  `settings.types.ts:58-62` `UpdateCompanyAddressRequest`.
- Serveur : `crates/kesh-api/src/routes/companies.rs` — `update_company_email` (`:112`), `update_company_contact_details`, `update_company_address`
  (`:344`) reconstruisent chacune un `CompanyUpdate` complet avec `name: company.name.clone(),` ; aucune n'écrit `name`. Patron de
  la trace : `NewAuditLogEntry::user(… "company.updated", "company", …)` avec `details_json` `{before, after}` **limité aux champs touchés**,
  écrite seulement si `updated.version != company.version` (no-op KF-004). Routes : `crates/kesh-api/src/lib.rs:458-473`, bloc `admin_routes`
  (`require_not_pat` : session seulement).
- Les contraintes de la QR-facture, déjà publiques dans `kesh-qrbill` : `validation.rs:10` `pub const NAME_MAX: usize = 70;` (non ré-exportée
  à la racine : `lib.rs:30` n'exporte que `champ_vide_pour_qr, first_qr_refused_char, is_iso_3166_alpha2, validate` ; `pub mod validation;` `lib.rs:18`
  la rend accessible par `kesh_qrbill::validation::NAME_MAX`) ; `validation.rs:244` `pub fn champ_vide_pour_qr` ; `:213` `pub fn first_qr_refused_char`.
  Le PDF les applique au nom : `validation.rs:317-321` (`champ_vide_pour_qr(&addr.name)` → `FieldEmpty`, `check_len(…, NAME_MAX)`, `check_charset`),
  et `crates/kesh-api/src/routes/invoice_pdf_service.rs:281` `name: company.name.clone(),`, `:419` `creditor_name: company.name.clone(),`.
- Le modèle de validation : `address_input.rs` `validate_required_for_qr` (message qui nomme le champ et le caractère, français littéral, sans clé).
  L'assistant ne contrôle que « non vide » : `crates/kesh-api/src/routes/onboarding.rs:535` `if name.is_empty() {` (et recompose `name` en
  « prénom nom » pour une personne physique, `:533`).
- **Les trois refus du PDF disent que le nom « ne se modifie pas encore depuis l'interface »** — `invoice_pdf_service.rs:624`, `:704`, `:737` (replis français)
  et les mêmes phrases aux quatre `messages.ftl` : `invoice-pdf-error-charset-company-name`, `invoice-pdf-error-empty-company-name`,
  `invoice-pdf-error-too-long-company-name` (fr-CH `:999`, `:1011`, `:1018` ; de-CH `:937`, `:949`, `:956` ; it-CH et en-CH idem). Deux gardes les
  **imposent** aujourd'hui : `invoice_pdf_service.rs:1737` `if r.cle.ends_with("-company-name") {` / `!texte.contains("Organi")` et
  `crates/kesh-api/tests/invoice_pdf_e2e.rs:698` (`assert!(!m.contains("organisation"), "{m}");` après le commentaire « le nom ne se modifie pas »).
  La story les **inverse** : la phrase renvoie désormais à *Paramètres → Organisation*, comme celles de la rue et du NPA.
- Manuel : `docs/manual/fr/user-manual.tex`, § *Vos coordonnées sur la facture* (`:1221`), dernière phrase du paragraphe « Corriger votre adresse » :
  « Le nom et l'IDE ne se modifient pas ici. » ; `docs/manual/fr/admin-manual.tex:2667` (paragraphe « Pays des adresses », qui cite `PUT /api/v1/companies/current/address`).
  `docs/api-external.md` : aucune des routes `PUT /companies/current/*` n'y figure (`git grep -n "companies/current" -- docs/api-external.md` vide) et elles refusent les clés d'API :
  **inchangé**.
- Gardes qui énumèrent les routes d'administration : `crates/kesh-api/tests/admin_pat_denied_e2e.rs:83-85` (liste `("PUT", "/api/v1/companies/current/address")`),
  `crates/kesh-api/tests/textes_coherents.rs:1355` (occurrence assumée du chemin dans `admin-manual.tex`, liste fermée `MULTI_SOCIETE_ASSUME`).

## Décisions

- **D1 — A : le brouillon se jette au départ du champ, *Entrée* ne fait jamais partir un brouillon** (C-v0190c-1). `ContactPicker` reçoit la règle de
  la 36-1, adaptée : `draft: string | null` (`null` hors frappe) ; `query = $derived(draft ?? selected?.name ?? '')` — le `$effect` à
  `initialSyncDone` disparaît (le parent qui change `selected` change l'affichage, sauf pendant une frappe) ; `pick` fait `onSelect(c); draft = null; open = false;` ;
  **`blur` : `draft = null`** (le champ réaffiche le client retenu, ou rien) puis fermeture de la liste ; **un changement de `selected?.id` jette le brouillon** (`$effect`
  sur l'identifiant seul, `untrack` du brouillon : après un 409 le client rechargé s'affiche) ; **frappe** : `draft = valeur`. **Option écartée** :
  résoudre le texte par une recherche au `blur` (« le premier contact qui correspond ») — un choix que l'écran n'a pas montré, contraire à la règle « un
  appariement automatique propose, il ne crée ni ne tranche » (règle du `CLAUDE.md`, § *Un appariement automatique propose, il ne crée jamais*, lue dans la version de `origin/docs/nettoyage-claude-md`) ; et le faux
  rattachement d'une facture est muet. **Option écartée** : signaler par un message — le client retenu réaffiché dit déjà ce qui part, comme pour le compte.
- **D2 — A : *Entrée*** (C-v0190c-2). Ordre : (1) liste ouverte **et** surbrillance désignée **et** recherche courante terminée → l'élément est choisi
  (`preventDefault`, rien ne part) ; (2) sinon, **un brouillon est affiché** (`draft !== null`) → `preventDefault` toujours, la liste s'ouvre (« Aucun contact » ou les résultats, à l'écran) ;
  (3) sinon (aucun brouillon : champ non touché) l'événement suit son cours : le formulaire part avec le client que le champ montre. *Entrée* maintenue (`e.repeat`) et composition
  IME (`e.isComposing`, `e.keyCode === 229`) : mêmes règles que `AccountAutocomplete.svelte` (à lire, ne pas réinventer). « Recherche courante terminée » : un indicateur
  `pending` posé à la frappe et levé quand `runSearch` de **cette** séquence (`searchSeq`) rend la main ; pendant l'anti-rebond, *Entrée* ne choisit pas dans l'ancienne liste.
- **D3 — A : la surbrillance ne vaut choix que désignée** (C-v0190c-3). `highlighted` vaut `-1` à l'ouverture, au focus, au `blur`, à *Échap* et au choix ; il passe à `0` quand la
  saisie est **non blanche** et la liste non vide, ou par une flèche ; *flèche bas* depuis `-1` va à `0` (et non `1`) ; *flèche haut* depuis `-1` va au dernier. Rend visible la désignation comme
  la 36-1 le fait pour le compte **seulement si** les jetons existent : `ContactPicker` emploie aujourd'hui `hover:bg-muted` / `bg-muted` (`:159`, `:161`), classes que le thème ne
  génère pas (`/usr/bin/grep -nE "muted|accent" frontend/src/app.css` ne rend que `--color-text-muted`, `:32`, et un commentaire `:41`) — **réemployer les deux constantes** `CLASSE_DESIGNEE` / `CLASSE_SURVOL`
  de `frontend/src/lib/features/journal-entries/account-autocomplete-classes.ts` (à vérifier présent à la tête du worktree : `git cat-file -e HEAD:frontend/src/lib/features/journal-entries/account-autocomplete-classes.ts` ;
  s'il est absent, ne pas copier le patron : traiter le rendu en LOW, issue). **Hors périmètre** : les deux filtres ne gagnent pas de bouton d'effacement (le filtre « Tous les contacts » ne se rétablit pas
  aujourd'hui : issue à ouvrir si aucune ne la porte, I1).
- **D4 — B : une route dédiée `PUT /api/v1/companies/current/name`** (C-v0190c-4), sur le patron exact d'`update_company_email` : `UpdateCompanyNameRequest { name: String, version: i32 }`,
  même routeur Admin-only (`lib.rs`, après la route `address`). Rend `CompanyJson`. **Option écartée** : étendre `address` ou `contact-details` (blocs sans rapport, et le
  full-replace efface par omission). Le client ne reçoit rien de neuf : `CompanyJson.name` existe.
- **D5 — B : les contrôles** (C-v0190c-5), dans une fonction unique de `companies.rs` (`validate_company_name_for_qr`, DRY, testée à part) appelée **avant** la lecture de la société :
  rogner (`trim`) ; **vide** au sens de la QR-facture (`kesh_qrbill::champ_vide_pour_qr`, pas `trim().is_empty()` : l'assistant la reprend déjà pour l'adresse, 0.17.5 M-1) → refus ;
  **plus de 70 caractères** (`chars().count() > kesh_qrbill::validation::NAME_MAX`) → refus qui dit le maximum et le nombre saisi ; **caractère que la QR-facture n'admet pas**
  (`kesh_qrbill::first_qr_refused_char`) → refus qui nomme le caractère et son point de code (`escape_debug`, `U+{:04X}`, comme `address_input.rs`). `400 VALIDATION_ERROR`, français
  littéral comme la route d'adresse (pas de clé i18n neuve côté serveur ; le message se lit quelle que soit la langue de l'interface, ce que le manuel dit déjà de l'adresse, #710).
  Ce sont **les contrôles de l'assistant** (non vide) **plus** ceux de la QR-facture : une société qui n'a jamais eu de QR-facture reçoit le même refus à la correction qu'à la facture.
- **D6 — B : `first_name` / `last_name` ne bougent pas** (C-v0190c-6). Pour une personne physique, l'assistant recompose `name` en « prénom nom » ; l'écran ne montre que `name`, et le
  PDF n'emploie que `name` (`invoice_pdf_service.rs:281`, `:419`). La route reporte `first_name` / `last_name` à l'identique (full-replace) : corriger le nom ne réécrit pas l'état civil, qui
  n'a pas d'écran. **Conséquence signalée, non traitée** : le nom corrigé peut différer de « prénom nom » ; aucun écran ni document ne lit les deux ensemble (`git grep -n "first_name" -- crates/kesh-api/src/routes/invoice_pdf_service.rs`
  ne rend que le montage de test `:980`). Question Q1 au rapport.
- **D7 — B : audit et verrou** (C-v0190c-7). Transaction + `companies::update_in_tx` + `audit_log::insert_in_tx` comme les trois routes jumelles : verrou optimiste par `version`
  (`409 OPTIMISTIC_LOCK_CONFLICT`), no-op sans `version` ni audit (KF-004), `NewAuditLogEntry::user(…, "company.updated", "company", …)` avec
  `{"before": {"name": …}, "after": {"name": …}}` — **aucune nouvelle action** (`audit_labels.rs:106` `"company.updated"` existe). Rien n'est réécrit dans les factures déjà émises : leur PDF figé ne
  change pas (le manuel le dit déjà pour l'adresse) ; les brouillons et les factures non figées prennent le nouveau nom.
- **D8 — B : l'écran** (C-v0190c-8). Section *Organisation* : le nom se lit comme aujourd'hui, avec le bouton **Modifier** (administrateurs, `isAdmin`) ; l'édition est le bloc
  e-mail transposé (`nameEditing`, `nameValue`, `nameBase`, `nameVersion`, `nameConflict`, `NAME_FIELDS = ['name'] as const`, `nameValuesOf`), mêmes `adoptCompany`,
  `reloadAfterConflict`, `CompanyConflictNotice` (testid `settings-company-name-conflict`, `field="name"`). `maxlength={70}` sur le champ. Contrôle immédiat côté client : vide → message
  `settings-company-name-required` ; **les refus de longueur et de jeu de caractères sont ceux du serveur** (le message s'affiche tel quel, comme pour l'adresse). **Option écartée** : recopier le jeu
  de caractères de la QR-facture dans le frontend (deux copies qui divergent : le refus qui fait autorité est celui du serveur).
- **D9 — B : les trois refus du PDF renvoient à l'écran** (C-v0190c-9). Les trois replis Rust et les douze phrases des catalogues (3 clés × 4 locales) perdent « ne se modifie pas encore depuis
  l'interface » et disent « Corrigez-le dans Paramètres → Organisation » (même tournure que `invoice-pdf-error-too-long-company-street`, `invoice_pdf_service.rs:739-743`, à lire et recopier par locale). Les deux gardes
  qui exigeaient l'inverse (`invoice_pdf_service.rs:1737`, `invoice_pdf_e2e.rs:698`) s'inversent : le texte **contient** « Organi ». Le test de cohérence de `textes_coherents.rs`
  qui énumère les 42 clés (`invoice_pdf_service.rs:1751` `assert_eq!(cles.len(), 42, "{cles:?}");`) ne change pas de nombre : aucune clé n'est ajoutée ni retirée.
- **D10 — le manuel** (C-v0190c-10). `user-manual.tex` : § *Vos coordonnées sur la facture* — remplacer « Le nom et l'IDE ne se modifient pas ici. » par une phrase qui dit que le **nom** se corrige là
  (bouton *Modifier* en face de « Nom », administrateurs ; 70 caractères au plus ; jeu de caractères de la QR-facture ; un PDF déjà figé ne change pas) et que seul l'**IDE** ne s'y modifie pas ;
  § *Créer une facture* (`:1162` *Factures → Nouvelle facture. Renseigner :*) — une phrase : le client se choisit **dans la liste** ; un texte tapé qui n'est pas choisi est abandonné en quittant le champ, et le champ
  réaffiche alors le client retenu. `admin-manual.tex:2667` : `PUT /api/v1/companies/current/name` à côté de la route d'adresse (un seul `\texttt{}` ; **la garde `MULTI_SOCIETE_ASSUME` de
  `textes_coherents.rs:1355` s'applique au chemin `companies` : y ajouter l'occurrence assumée, ou écrire la route sans le segment que la garde détecte — lire la garde d'abord**). Les PDF ne
  se commitent pas (règle des fichiers communs).

## Critères d'acceptation

### A — #1162

1. **AC1 — le composant** (`frontend/src/lib/components/invoices/ContactPicker.test.ts`, neuf ; mock de `listContacts` ; l'événement constaté par la valeur rendue de `fireEvent.keyDown`, `false` quand
   `preventDefault` a été appelé ; les cas qui constatent l'affichage après un choix réinjectent `selected` comme un parent, par un `onSelect` espion qui appelle `rerender`) :
   - a. `selected = Dupont`, taper « zzz », quitter le champ (`blur`) → le champ montre « Dupont » ; `onSelect` jamais appelé ;
   - b. `selected = null`, taper « zzz », `blur` → le champ est vide ;
   - c. `selected = Dupont`, taper « Mar », *Entrée* (liste fermée par *Échap*, ou aucun résultat) → événement empêché ; `onSelect` jamais appelé ; le texte tapé reste ; la liste « Aucun contact » ou les résultats est affichée ;
   - d. *Entrée* **pendant l'anti-rebond** (temps simulé, 100 ms après la frappe, anciens résultats présents) → événement empêché, `onSelect` jamais appelé ; la recherche terminée, *Entrée* choisit le premier résultat de **la nouvelle** liste ;
   - e. focus sans saisie, liste affichée, *Entrée* → rien n'est choisi (aucune surbrillance), événement **non** empêché si aucun brouillon (le formulaire part avec le client affiché) ; *flèche bas* → le **premier** contact est désigné, *Entrée* le choisit ;
   - f. choix à la souris (`mousedown`) puis `blur` → le champ montre le contact choisi (parent réinjecté) ;
   - g. `selected` change de client pendant qu'un brouillon est affiché (rechargement après conflit) → le champ montre le nouveau client ; `selected` passe à `null` → le champ est vide ;
   - h. composition IME et touche maintenue : mêmes issues que `AccountAutocomplete` (un cas chacun).
2. **AC2 — la facture** (`InvoiceForm.test.ts`, un test) : facture existante au client X ; taper « zzz » dans le champ client puis cliquer *Enregistrer* → `updateInvoice` reçoit `contactId` de X et le champ montre X. Les tests existants du formulaire restent verts (le montage du client initial, `:380-384`, passe par `selected`).
3. **AC3 — les filtres** : les deux pages de liste (`invoices/+page.svelte`, `due-dates/+page.svelte`) restent vertes ; un filtre posé se réaffiche tel quel après un texte tapé non choisi (couvert par AC1 a, le composant étant le même).
4. **AC4 — E2E** (`frontend/tests/e2e/invoices.spec.ts`, un test) : nouvelle facture, client choisi dans la liste ; taper un autre texte dans le champ client, *Entrée* → l'URL reste `/invoices/new` ; quitter le champ → il montre le client choisi ; *Créer la facture* → la facture créée porte ce client (liste des factures). Captures (clair seulement) : texte tapé non choisi, puis champ revenu au client retenu.
5. **AC5 — aucune clé i18n neuve, aucune migration, aucun changement serveur** pour A.

### B — #1097

6. **AC6 — la route** (`crates/kesh-api/tests/company_address_e2e.rs`, section neuve « Story v0190-correction », harnais existant de ce fichier — pas de copie ailleurs) :
   - a. `PUT /companies/current/name` `{"name":"Nom corrigé SA","version":v}` → `200`, `company.name` relu égal, `version` + 1, `first_name`/`last_name`/adresse/e-mail/téléphone/site inchangés ;
   - b. rogné (`"  Nom  "` → `"Nom"`) ; vide, blanc, et fait du seul U+00AD → `400 VALIDATION_ERROR`, rien d'écrit ; 70 caractères acceptés, 71 refusés (le message porte `70` et `71`) ; un caractère hors jeu (« 张 », U+0A, U+2028) refusé, message qui nomme le caractère et son point de code ; `&`, `é`, U+2019 acceptés ;
   - c. version périmée → `409 OPTIMISTIC_LOCK_CONFLICT` sans écriture ; **un nom invalide à version périmée est un 400** (la validation passe d'abord, comme `an_invalid_address_at_a_stale_version_is_a_400`) ;
   - d. nom identique → no-op : `version` inchangée, **aucune** entrée d'audit ;
   - e. audit : une entrée `company.updated`, `details_json` = `{"before":{"name":ancien},"after":{"name":nouveau}}` **sans autre clé**, acteur `user` (`common::audit_actor`, comme `company_address_e2e.rs:883`) ;
   - f. **bout en bout** : un nom de 71 caractères posé en base fait refuser le PDF d'une facture validée (message : « Paramètres → Organisation ») ; après la correction par la route, le PDF se produit (patron de `correcting_the_country_restores_the_qr_invoice`, `company_address_e2e.rs:570`) ;
   - g. un rôle non administrateur (Comptable) → `403` ; une clé d'API → refusée (`admin_pat_denied_e2e.rs` : ajouter `("PUT", "/api/v1/companies/current/name")` à la liste `:83-85`).
7. **AC7 — les contrôles, à part** : tests unitaires de `validate_company_name_for_qr` (`companies.rs`, `mod tests`) pour les bornes (0, 1, 70, 71 caractères, U+00AD seul, le premier caractère refusé d'une chaîne qui en porte deux).
8. **AC8 — les refus du PDF** : les trois messages (replis et catalogues, quatre locales) renvoient à *Paramètres → Organisation* ; `invoice_pdf_service.rs:1737` et `invoice_pdf_e2e.rs:698` inversés ; `assert_eq!(cles.len(), 42)` inchangé ; la garde i18n et les tests de textes (`cargo test -p kesh-api --test textes_coherents`) verts.
9. **AC9 — l'écran** (`settings-page.test.ts`, patron des blocs e-mail et adresse) : le bouton *Modifier* du nom n'apparaît que pour un administrateur ; l'enregistrement appelle `updateCompanyName({ name, version })` puis affiche le nom reçu et le message de succès ; nom vide → refus local, aucun appel ; refus du serveur (400) → message affiché, saisie conservée ; `OPTIMISTIC_LOCK_CONFLICT` → même chemin que le bloc e-mail (relecture et avis). `settings.api.test.ts` : un test de l'appel `PUT` (URL, corps).
10. **AC10 — E2E** (`frontend/tests/e2e/company-address.spec.ts`, un test, ou un fichier `company-name.spec.ts` si la restauration l'exige — la société est partagée : **rendre le nom tel qu'on l'a trouvé**, comme `restoreAddress`) : *Paramètres → Organisation → Modifier* le nom, enregistrer, recharger la page : le nom corrigé est affiché ; un nom de 71 caractères est refusé à l'écran avec le message du serveur. Captures (clair seulement) : bouton *Modifier* en face de « Nom », champ en édition, refus.
11. **AC11 — les compteurs et les gardes** : `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts:1040`) **recompté** depuis la source après l'ajout des clés ; `npm run lint-i18n-ownership` vert (clés `settings-company-name-*` sous le préfixe `settings-` de la page) ; catalogues des quatre locales **dans le même ordre**, les clés neuves à côté de `settings-company-email-saved` ; le test de manuels (`crates/kesh-api/tests/manuel.rs`) et `textes_coherents` verts.
12. **AC12 — le manuel** : D10, `.tex` seulement ; contrôle par `git grep` (pas de PDF).

## Tâches

**A (frontend seul)**
1. `ContactPicker.svelte` : D1, D2, D3 (lire d'abord `AccountAutocomplete.svelte`, `handleKeydown` et `commitDraft`, à la tête du worktree).
2. `ContactPicker.test.ts` (AC1) ; test de `InvoiceForm.test.ts` (AC2) ; test E2E (AC4) et captures.

**B (backend, i18n, écran)**
3. `companies.rs` : `UpdateCompanyNameRequest`, `validate_company_name_for_qr`, `update_company_name` (D4, D5, D6, D7) ; route dans `lib.rs` ; tests AC6, AC7 ; `admin_pat_denied_e2e.rs`.
4. `invoice_pdf_service.rs` (trois replis) et quatre `messages.ftl` (douze phrases) ; inversion des deux gardes (D9, AC8).
5. `settings.api.ts` + `settings.types.ts` (`UpdateCompanyNameRequest`) ; `+page.svelte` (bloc nom, D8) ; clés `settings-company-name-saved`, `-required`, `-conflict`, `-conflict-reload-failed` dans les quatre catalogues ; tests AC9, E2E AC10 ; `sitesTotal` (AC11).
6. Manuels (D10, AC12) ; section **`## Note de version`** ci-dessous (l'orchestrateur l'assemble ; la story n'écrit pas `CHANGELOG.md`).

## Gates

Ceux de `e35-dev-commun.md` : gate ciblé en cours de développement (A : `npm run check`, Vitest de `ContactPicker`, `InvoiceForm`, `settings` ; B : `cargo nextest run -E 'binary(company_address_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e) | binary(admin_pat_denied_e2e)'`, `fmt`,
`clippy` workspace) ; `test-fast --ci` et `gate-ci` au passage en `review` ; `gate-ci` **et** E2E complet à la clôture, au dernier commit de code. Pas de `kesh-db` touché : pas de gate complet forcé.

## Recette visuelle (captures, thème clair, base de démonstration, `kesh-gate-logs/v0190c/`)

A : champ client avec un texte tapé non choisi ; le même champ après avoir quitté le champ (client retenu réaffiché) ; liste avec une surbrillance désignée par flèche. B : *Organisation* avec le bouton
*Modifier* en face du nom ; édition ; refus de 71 caractères ; refus du PDF d'une facture (message renvoyant à *Paramètres → Organisation*).

## Dev Notes

- **Taille** : A ≈ 1 composant, 3 fichiers de test ; B ≈ 1 route, 1 page, 4 catalogues, 6 fichiers de test, 2 manuels. Au total six modules (kesh-api routes, kesh-api PDF, kesh-i18n, frontend invoices, frontend settings, manuels) : **au-dessus de la limite de cinq** de la règle de découpage — d'où le **découpage de repli** ci-dessous, à décider avant le développement si le temps presse.
- **Découpage de repli (proposé, non écrit)** : `v0190-correction` = A seule (#1162, le plus grave : Guy émet sa première facture client) ; `v0190-nom-societe` = B seule (#1097), glissant à la 0.20.0. Aucun fichier de code n'est partagé ; seuls `user-manual.tex` (deux paragraphes distincts), `sprint-status.yaml` et les catalogues (clés de B seulement) sont communs.
- **Issues à ouvrir si le développement les confirme** (« on trace toujours ») : I1, les filtres de liste sans effacement du client ; I2, **l'assistant n'applique pas les contrôles de la QR-facture au nom** (`onboarding.rs:535` ne refuse que le vide — origine de #1097 : un nom à 71 caractères ou à caractère refusé s'y enregistre encore) ; I3, `first_name`/`last_name` sans écran (D6).
- **Les gardes du dépôt à relire avant d'écrire** : G19 de `textes_coherents.rs` (URL des manuels), `MULTI_SOCIETE_ASSUME`, `manuel.rs` ; le détecteur du symptôme de #1162 (`git grep -n -l -E 'aria-autocomplete|role="combobox"|role="listbox"' -- frontend/src`) ne doit plus rendre qu'un composant sans la règle (`ContactPicker`, une fois corrigé, et `AccountAutocomplete`, déjà corrigé) ; sonde positive : avant correction, `ContactPicker` est trouvé.
- Aucune donnée réelle dans les fixtures (noms de démonstration, `Dupont`, « Démo SA »).

## Note de version

**Corrigé**
- Le champ « client » d'une facture ne montre plus un client que la facture n'enregistrera pas : un texte tapé sans choisir un client de la liste est abandonné en quittant le champ (le client retenu s'affiche), et la touche *Entrée* ne fait plus partir la facture tant qu'un texte tapé n'est pas résolu ni ne choisit un client que vous n'avez pas désigné (refs #1162).
- Le **nom de la société** se corrige dans *Paramètres → Organisation* (bouton *Modifier*, administrateurs) : jusqu'à 70 caractères, jeu de caractères de la QR-facture ; le refus du PDF d'une facture dont le nom est vide, trop long ou refusé renvoie désormais à cet écran. Aucun changement de contrat de l'API des clés d'API ; une route d'administration est ajoutée, `PUT /api/v1/companies/current/name` (session seulement) (refs #1097).
