# Story 16.1b : Sélecteur de compte de produit par ligne — formulaire de facture

## Status

ready-for-dev

## Story

**As a** indépendant / PME / fiduciaire qui saisit ses factures dans Kesh,
**I want** choisir le **compte de produit** de chaque ligne directement dans le formulaire de facture, et voir clairement quand une ligne suit le **compte par défaut de la société**,
**so that** je ventile ma facturation au moment où je la saisis, sans avoir à passer par l'API ni à reclasser après coup.

Issue : **#152**. Rattaché au CR **#265**.

**Dépend de 16-1a** (colonne `invoice_lines.revenue_account_id`, champ API `revenueAccountId`, validation et ventilation backend). 16-1b est purement la surface utilisateur.

---

## Contexte

### Ce qui existe aujourd'hui

- **`InvoiceForm.svelte`** (`frontend/src/lib/components/invoices/InvoiceForm.svelte`) — noter `lib/components/`, **pas** `lib/features/`. Charge déjà contacts, produits, taux de TVA, projets et réglages de facturation (`:14-37`, `:203`). **Ne charge aucune liste de comptes** — c'est du plomberie nette à ajouter.
- **`AccountAutocomplete.svelte`** (`frontend/src/lib/features/journal-entries/`) — composant de sélection de compte existant :
  - `Props` : `accounts`, `value: number | null`, `loadError?`, `disabled?`, `onSelect: (id: number | null) => void` (`:6-14`) ;
  - **résout le libellé sur la liste COMPLÈTE** `accounts.find((a) => a.id === value)` (`:22-30`) — pas sur la liste filtrée ;
  - ne propose au dropdown que `accounts.filter((a) => a.active && a.postable)` (`:32-34`, commentaire 14-3b) ;
  - `handleInput` (`:52-60`) n'appelle `onSelect` **que** en mode `loadError` — taper du texte sans sélectionner ne modifie donc jamais la valeur liée ;
  - **aucune affordance de remise à `null`** hors mode `loadError` ;
  - importe `i18nMsg` depuis `$lib/features/onboarding/onboarding.svelte` (`:3`), alors que `frontend/src/lib/shared/utils/i18n.svelte.ts:1-7` documente explicitement que l'emplacement canonique est `$lib/shared/utils/i18n.svelte` et qu'il ne faut **pas** importer depuis onboarding (couplage transverse).
- **4 importeurs réels** d'`AccountAutocomplete` (+ son test) :
  - `features/journal-entries/JournalEntryForm.svelte:22`
  - `features/journal-entries/VatPurchaseAssistant.svelte:14`
  - `features/reconciliation/TransactionSplitModal.svelte:22`
  - `features/reconciliation/ManualMatchModal.svelte:24`
  - test : `features/journal-entries/AccountAutocomplete.test.ts:14`
- **i18n** : les catalogues sont **côté backend**, `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`. Le frontend résout via `i18nMsg(key, fallback)` (`shared/utils/i18n.svelte.ts:14`), les messages étant chargés depuis `/api/v1/i18n/messages`. « i18n 4 locales » = éditer 4 fichiers `.ftl` dans un crate Rust.
- **`lint-i18n-ownership`** (`frontend/scripts/lint-i18n-ownership.js`) : ne parcourt **que** `src/lib/features` (`FEATURES_PATH` `:17`, `main()` `:225-226`). Ne signale **pas** les entrées obsolètes de `KNOWN_VIOLATIONS`. Les clés d'`AccountAutocomplete` y sont déjà listées comme violations connues (`:46-47`).
- **Manuel utilisateur FR** : `docs/manual/fr/user-manual.tex:574` (« Lignes de facture : un ou plusieurs produits avec quantité, prix, TVA ») et `:591` (exemple d'écriture « Débit 1100 Débiteurs / Crédit 3200 Honoraires »).

### Ce qui n'est PAS dans 16-1b

- Tout le backend → **16-1a**.
- Le pré-remplissage depuis la fiche produit → **16-2 (#144)**. Hypothèse de conception retenue (question ouverte n°3 levée) : le compte du produit ne fera que **pré-remplir** la ligne côté frontend, sans reprise rétroactive sur les factures existantes.
- Les coordonnées émetteur / n° client sur le PDF → **16-3 (#151)**.
- Aucun changement de la présentation du PDF de facture.

---

## Décisions de conception

### D6 (révisée) — NE PAS déplacer `AccountAutocomplete` ; l'étendre par props opt-in

**La version initiale prescrivait un déplacement vers `lib/components/accounts/`, au motif que `npm run lint-i18n-ownership` sanctionnerait l'import cross-feature. Ce motif est factuellement faux**, vérifié en passe 1 :

- le lint ne parcourt que `src/lib/features` (`lint-i18n-ownership.js:17`, `:225-226`) et ne contrôle **que** l'appariement clé i18n ↔ dossier de feature, jamais les imports de composants ;
- `features/reconciliation/ManualMatchModal.svelte:24` et `TransactionSplitModal.svelte:22` **importent déjà ce composant depuis `features/journal-entries/`** aujourd'hui, sans aucune violation ;
- déplacer le fichier le ferait **sortir** du périmètre du lint, laissant deux entrées mortes dans `KNOWN_VIOLATIONS` (`:46-47`) que le script ne signale pas.

**Décision** : `InvoiceForm.svelte` importe `AccountAutocomplete` depuis son emplacement actuel, exactement comme le fait déjà `reconciliation`. Le composant est **étendu**, pas déplacé.

**Motif** : le déplacement embarquerait 4 importeurs + un chemin de test + la feature `reconciliation`, entièrement étrangère à cette story, pour un bénéfice d'hygiène nul au regard du gate CI. Le composant doit de toute façon être **modifié** (cf. D7 et D8) — modifier n'est pas déplacer, et les deux risques ne se composent pas.

**Conséquence sur les nouvelles capacités** : toute extension du composant se fait par **props optionnelles dont le défaut préserve le comportement actuel**, pour que les 4 consommateurs existants soient strictement inchangés. Un test le vérifie (AC7).

*(Si le déplacement est souhaité pour raison d'architecture, il fera l'objet d'un `chore/` dédié, découplé de cette story.)*

### D7 (révisée) — Le compte persisté invalide s'affiche déjà ; ce qui manque, c'est le **signal** et le **blocage**

**La version initiale affirmait que le champ s'afficherait vide et qu'un simple enregistrement le nullifierait en silence — la dette #271. C'est faux pour ce composant**, et la Story 14-3b le disait déjà explicitement (`14-3b-consommateurs-roles.md:191` : « `AccountAutocomplete` (saisie d'écriture) **n'est pas concerné** (résout le libellé via la liste complète) »). Vérifié en passe 1 :

- le `$effect` (`AccountAutocomplete.svelte:22-30`) cherche dans `accounts`, la liste **complète**, pas dans `active` (le dérivé filtré `:32-34`) → un compte archivé ou non-postable **affiche bien son libellé** ;
- `handleInput` (`:52-60`) n'appelle `onSelect` qu'en mode `loadError` → taper du texte sans sélectionner ne nullifie **rien**.

**Le vrai manque, en deux points** :

1. **Aucun signal d'invalidité** — le compte s'affiche normalement, rien ne dit à l'utilisateur qu'il ne sera plus accepté au posting. La facture partira en `400` à la validation (16-1a AC8), au moment le moins pratique.
2. **Aucune affordance de retour à `null`** — hors mode `loadError`, il est **impossible** de revenir à « défaut société » une fois un compte choisi. Sur un champ obligatoire (saisie d'écriture) ça n'a jamais compté ; sur un champ **optionnel**, c'est bloquant.

**Décision** *(comportement arbitré par Guy le 2026-07-26 : afficher + signaler + bloquer la ligne)* :

- la valeur persistée reste **affichée** avec son libellé ;
- si le compte référencé est absent de la liste des comptes sélectionnables (archivé, non-postable, ou de type ≠ `Revenue`), la ligne porte un **marqueur d'invalidité visible** (message textuel, pas seulement une couleur — accessibilité) ;
- l'**enregistrement du formulaire est bloqué** tant qu'il reste au moins une ligne invalide — le blocage est **global**, la persistance étant atomique (`:359-369`), et le message nomme **toutes** les lignes fautives ;
- les **autres lignes ne sont ni signalées ni modifiées**, et débloquer une ligne n'exige aucune action sur les autres. *(Formulation précisée en passe 3 : « les autres lignes ne sont pas bloquées » décrivait un grain d'enregistrement par ligne qui n'existe pas dans ce formulaire — cf. AC8.)*

### D8 — Retour à « défaut société » : une affordance explicite, opt-in

Le champ est optionnel. Il faut pouvoir le vider.

**Décision** : une prop `allowClear?: boolean` (défaut `false`, donc les 4 consommateurs existants sont inchangés) ajoute un bouton d'effacement accessible (`aria-label` traduit) qui appelle `onSelect(null)` et remet `query` à vide. `InvoiceForm` la passe à `true`.

**Motif** : `onSelect` accepte déjà `number | null` dans la signature de `Props` (`:12`) — le contrat existe, seule l'affordance manque. Ne pas réinventer un second composant (le vrai fond de la D6 initiale, qui reste valide).

### D9 — Le repli est affiché, pas simulé

Quand une ligne n'a pas de compte, le champ affiche un placeholder nommant explicitement le **compte par défaut de la société** (numéro + libellé, chargé depuis les réglages de facturation déjà récupérés par `InvoiceForm` via `getInvoiceSettings`, `:28` / `:203`), et non un simple « — ».

**Motif** : le sens de `NULL` est « suivre le défaut », pas « aucun compte ». Un champ vide laisse croire à un oubli. Afficher le compte cible rend la ventilation lisible d'un coup d'œil et évite que l'utilisateur sélectionne explicitement le défaut « pour être sûr » — cas que 16-1a D3-bis doit précisément absorber.

**Ne pas matérialiser la valeur** : le placeholder est de l'affichage. La ligne reste à `NULL` en base (16-1a D2).

### D10 — Chargement de la liste des comptes : échec dégradé, pas bloquant

`InvoiceForm` ne charge aujourd'hui aucune liste de comptes. Il faut l'ajouter (API comptes de la société).

**Décision** : l'échec du chargement de la liste **ne bloque pas** la saisie de facture. Le composant a déjà un mode dégradé (`loadError` → saisie de l'ID numérique, `:52-60`) — le réutiliser. Une facture doit rester saisissable même si l'endpoint comptes est momentanément indisponible ; le champ est optionnel, son indisponibilité fait simplement retomber toutes les lignes sur le défaut société.

### D11 — La liste des comptes DOIT être chargée avec `fetchAccounts(true)` — sans quoi D7 s'effondre

**C'est la dépendance technique porteuse de toute la décision D7**, et elle est invisible si on ne la nomme pas.

`fetchAccounts(includeArchived = false)` (`frontend/src/lib/features/accounts/accounts.api.ts:10-14`) **exclut les comptes archivés par défaut**. Or D7 repose entièrement sur le fait que le `$effect` d'`AccountAutocomplete` (`:22-30`) résolve le libellé via `accounts.find((a) => a.id === value)`. Si le compte archivé **n'est pas dans le tableau `accounts`**, `find` renvoie `undefined`, `query` reste vide — et **le champ s'affiche vide**. C'est-à-dire exactement le symptôme #271 que D7 (révisée) affirme ne pas s'appliquer ici.

Autrement dit : la révision de D7 en passe 1 est correcte sur le **composant**, mais elle n'est vraie qu'à la condition que l'appelant fournisse la liste **complète**.

**Décision** : `InvoiceForm` appelle `fetchAccounts(true)`.

**Précédent dans le dépôt** — le projet a déjà rencontré et résolu exactement ce cas : `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte:33-43` appelle `fetchAccounts(true)` avec un commentaire qui en donne le motif (une écriture existante peut référencer un compte archivé), alors que l'écran de **création** `journal-entries/+page.svelte:100` appelle `fetchAccounts(false)`. Le formulaire de facture édite des brouillons porteurs de valeurs persistées : il est du côté « détail », pas du côté « création vierge ».

**Le filtre du dropdown reste inchangé** : `accounts.filter((a) => a.active && a.postable)` (`:32-34`) continue d'exclure les comptes archivés des propositions. Charger la liste complète sert **uniquement** à résoudre le libellé d'une valeur persistée — pas à la rendre sélectionnable.

**Définition opérationnelle du marqueur d'invalidité (AC2)** : la valeur est présente dans `accounts` mais **absente** de la liste filtrée `active && postable`, ou son `accountType` n'est pas `Revenue`.

**Exception obligatoire, miroir de 16-1a D3-bis** : le compte dont l'id est égal à `invoiceSettings.defaultRevenueAccountId` (déjà disponible dans le formulaire, `InvoiceForm.svelte:344`) n'est **jamais** marqué invalide au seul motif `postable` — les critères `active` et `accountType = Revenue` restent appliqués. Sans cette exception, le cas que D3-bis existe précisément pour absorber (un défaut société devenu non-postable, atteignable sans intention via `effective_postable`) serait re-créé une couche plus haut : backend `200 OK`, frontend marqueur + blocage. L'utilisateur serait enfermé, incapable d'enregistrer son brouillon. **Le frontend ne doit jamais bloquer un enregistrement que 16-1a accepte.**

---

## Acceptance Criteria

### A. Composant

- **AC1** — `AccountAutocomplete` accepte `allowClear?: boolean` (défaut `false`) : quand `true`, un bouton d'effacement accessible (`aria-label` traduit, respectant `disabled`) appelle `onSelect(null)` (D8). Le bouton **ne remet pas `query` à vide lui-même** — c'est le `$effect` existant (`:27-29`, `else if (value === null) query = ''`) qui s'en charge, **source unique de vérité** ; une double écriture divergerait si le parent refusait la mise à jour.
- **AC1-bis** — **Effacement au clavier (D8, cas réel du champ optionnel)** : quand `allowClear` est `true`, un champ dont le texte a été **entièrement effacé par l'utilisateur** (clavier, couper, `Suppr`) vaut effacement explicite — au `blur`, si `query.trim() === ''` et `value !== null`, le composant appelle `onSelect(null)`. Symétriquement, si `query` est non vide mais ne correspond à **aucune** sélection validée, le `blur` restaure le libellé de `value` (ou vide le champ si `value` est `null`). **Le champ ne peut jamais afficher un texte qui contredit la valeur liée.**
  **Le piège fermé** : aujourd'hui `handleInput` (`:49-60`) n'appelle `onSelect` qu'en mode `loadError`, et `handleBlur` (`:88-93`) ne restaure rien. Sur un champ optionnel, le geste naturel pour « revenir au défaut société » est *tout sélectionner + Suppr*, pas de chercher un bouton. Sans AC1-bis : l'utilisateur efface le texte → `value` **reste** l'ancien compte → le champ vide déclenche le placeholder de D9 « 3200 — Produits (défaut société) » → l'utilisateur enregistre convaincu d'avoir remis la ligne au défaut → la facture se poste sur l'ancien compte. **Écriture comptable fausse produite en silence, avec l'UI qui affirme activement le contraire.** Ni D8 ni D9 ne créent ce piège isolément — c'est leur combinaison.
  **Doit rester inactif quand `allowClear` est `false`**, sinon `JournalEntryForm` (champ obligatoire) se met à nullifier ses lignes au `blur` — exactement la dette #271 que D7 a écartée.
- **AC2** — `AccountAutocomplete` accepte **deux props optionnelles dont le défaut préserve strictement le comportement actuel** : `markInvalid?: boolean` (défaut `false`) et `requiredAccountType?: AccountType` (défaut `undefined` = aucun contrôle de type). Quand `markInvalid` est `true`, une valeur **présente dans `accounts` mais absente de la liste filtrée `active && postable`** — ou dont l'`accountType` diffère de `requiredAccountType` quand celui-ci est fourni — conserve son libellé **et** reçoit un marqueur textuel (pas de signal porté par la seule couleur). `InvoiceForm` passe `markInvalid` et `requiredAccountType="Revenue"` ; les 4 consommateurs existants ne passent ni l'une ni l'autre et sont **strictement inchangés**.
  **Pourquoi opt-in** : `JournalEntryForm` et les deux modales de rapprochement sélectionnent massivement des comptes `Expense` / `Asset` / `Liability` — c'est la norme, pas l'exception. Un marqueur inconditionnel afficherait « compte invalide » sur quasiment toutes les lignes d'écriture manuelle. Le critère `Revenue` est une règle métier **propre à la facture** ; la poser en dur dans un composant partagé par 4 écrans est une erreur de placement.
  **Nom du champ** : `accountType` en camelCase (`accounts.types.ts:50`), pas `account_type`.
- **AC3** — L'import `i18nMsg` du composant passe de `$lib/features/onboarding/onboarding.svelte` (`:3`) à l'emplacement canonique `$lib/shared/utils/i18n.svelte`, conformément à la doc-comment de `shared/utils/i18n.svelte.ts:1-7`. Comportement inchangé.
- **AC4** — Le composant **n'est pas déplacé** (D6). Les 4 importeurs (`JournalEntryForm.svelte:22`, `VatPurchaseAssistant.svelte:14`, `TransactionSplitModal.svelte:22`, `ManualMatchModal.svelte:24`) et `AccountAutocomplete.test.ts` gardent leur chemin d'import actuel.

### B. Formulaire de facture

- **AC5** — `InvoiceForm.svelte` charge la liste des comptes de la société via **`fetchAccounts(true)`** (`accounts.api.ts:10`) — le flag `includeArchived` est **obligatoire**, cf. D11 : sans lui, un compte archivé persisté n'est pas dans le tableau, le `$effect` ne résout pas son libellé et le champ s'affiche vide, ce qui invalide toute la décision D7. Plomberie nette : aucun chargement de comptes n'existe dans ce formulaire aujourd'hui. Le formulaire expose ensuite un sélecteur de compte **par ligne**, optionnel.
- **AC5-bis** — Test de non-régression de D11 : un compte **archivé** référencé par une ligne de brouillon s'affiche avec son libellé (et son marqueur d'invalidité), et **n'apparaît pas** dans les propositions du dropdown. C'est le test qui échoue si quelqu'un « simplifie » `fetchAccounts(true)` en `fetchAccounts()`.
- **AC6** — D9 : une ligne sans compte affiche en placeholder le **compte par défaut de la société** (numéro + libellé), issu des réglages déjà chargés par `getInvoiceSettings` (`InvoiceSettingsResponse.defaultRevenueAccountId`, `invoices.types.ts:84`). La ligne reste à `NULL` — le placeholder n'est jamais persisté.
- **AC6-bis** — **L'écran de détail affiche le compte** (`routes/(app)/invoices/[id]/+page.svelte:753-773`, aujourd'hui 5 colonnes sans compte) : numéro + libellé pour une ligne renseignée, compte par défaut de la société avec mention « (défaut) » pour une ligne à `NULL` — même règle qu'AC6, en lecture seule.
  **Pourquoi c'est nécessaire** : l'édition n'est offerte qu'aux brouillons (`:563-578`). Dès qu'une facture est validée — c'est-à-dire dès l'instant où la ventilation produit son effet comptable — l'utilisateur n'a **plus aucun moyen** de voir sur quels comptes ses lignes ont été ventilées, sinon d'ouvrir l'écriture et de la lire à l'envers. La donnée devient write-once, en contradiction directe avec le motif de D9, et c'est le geste de contrôle n°1 d'une fiduciaire qui reprend un dossier. L'écran de détail d'**avoir** (`routes/(app)/credit-notes/[id]/+page.svelte`) est traité à l'identique, **ou** l'écart est documenté en limitation avec sa story de remédiation (§ « Tech debt management »). Le **PDF reste inchangé** (information interne, pas destinée au client).
  **Plomberie requise, à ne pas sous-estimer** (même nature que D11/AC5-bis côté formulaire) : `InvoiceLineResponse` ne porte qu'un **id** de compte, pas de libellé dénormalisé — résoudre « numéro + libellé » exige la liste des comptes, et **aucun** des deux écrans de détail ne charge aujourd'hui `fetchAccounts` ni `getInvoiceSettings` (vérifié : zéro occurrence dans `invoices/[id]/+page.svelte`). Charger la liste avec **`fetchAccounts(true)`** (un compte archivé peut être référencé par une facture validée — même motif que D11), plus `getInvoiceSettings()` sur l'écran facture pour la mention « (défaut) ».
  **Côté avoir, la mention « (défaut) » ne s'applique JAMAIS** : 16-1a D2/AC9-bis matérialise le compte à la validation, donc toute ligne d'avoir porte un compte non-`NULL`. Ne pas y implémenter de repli. *(Nuance héritée du backfill, désormais porté par la story **16-1a-bis** : les avoirs **antérieurs au déploiement** dont le backfill n'a pas pu identifier le compte — ou qui n'ont pas encore été backfillés, 16-1a-bis pouvant être livrée après 16-1a — restent à `NULL`. L'écran doit donc tolérer un `null` côté avoir — afficher un tiret, **pas** la mention « (défaut) », qui serait un mensonge : aucun repli n'aura lieu, l'écriture est déjà passée.)*
  **Une facture et son avoir peuvent afficher DEUX comptes différents — ne pas le signaler comme une anomalie** *(hérité de `16-1a-bis` D-B7)* : pour un couple antérieur au déploiement dont le compte de produit par défaut de la société a changé entre la validation de la facture et l'émission de l'avoir, les deux pièces ont réellement mouvementé des comptes distincts. Le backfill enregistre ce résidu historique **fidèlement** ; il ne le répare pas (le corriger serait une écriture de reclassement, un acte comptable de l'utilisateur). L'UI **affiche** donc les deux valeurs telles quelles, **sans** marqueur d'incohérence, **sans** avertissement, et **sans** tenter de les harmoniser.
  **En-têtes du tableau facture — le fichier n'est PAS i18n** : les 5 `<th>` de `invoices/[id]/+page.svelte:754-759` sont en français **codé en dur** (« Description », « Quantité », « Prix unitaire », « TVA % », « Total »), alors que l'écran d'avoir utilise `i18nMsg` (`credit-notes/[id]/+page.svelte:116-120`). La colonne ajoutée ici suit la **convention du fichier qu'elle modifie** — en dur côté facture, `i18nMsg` côté avoir. AC11 (« aucun libellé codé en dur ») vise les chaînes **du sélecteur et du formulaire**, pas une mise en i18n rétroactive de cet écran : l'ouvrir imposerait de traduire les 5 en-têtes existants et de les ajouter aux 4 catalogues, hors périmètre. Écart tracé, non corrigé.
- **AC7** — D10 : l'échec du chargement de la liste des comptes ne bloque pas la saisie ; le mode dégradé `loadError` existant est utilisé.
- **AC8** — D7 : une ligne dont le compte persisté est invalide affiche le libellé **et** le marqueur. L'enregistrement de la facture — **atomique par nature** — est bloqué **globalement** tant qu'au moins une ligne est invalide ; le message nomme **toutes** les lignes concernées (« Lignes 2, 5 : compte de produit invalide »), à l'image de 16-1a D6 et **non** du `return` à la première erreur de `validateClient` (`:297`).
  « Les autres lignes ne sont pas bloquées » signifie que **seules les lignes fautives portent un marqueur** : les autres ne sont ni signalées ni modifiées, et débloquer une ligne n'exige aucune action sur les autres. Le grain « enregistrer les lignes saines et pas les autres » **n'existe pas** — la persistance est un `createInvoice(payload)` / `updateInvoice(id, req)` unique portant toutes les lignes (`:359-369`), et le `disabled` du bouton (`:617`) n'a aucune condition par ligne.
- **AC9** — Ajout et suppression de lignes conservent l'association ligne ↔ compte. `InvoiceForm` clé son `{#each}` par `_uiKey` (`:548`) et non par index ; la nouvelle donnée suit la même clé. **Le formulaire n'offre PAS de réordonnancement de lignes** — seulement `addFreeLine` (`:250`), `onProductSelect` (`:260`) et `removeLine` (`:270`) ; le commentaire `:86` anticipe une fonctionnalité jamais livrée. Ne pas en écrire de test, ne pas l'ajouter.
- **AC9-bis** — **Décompte de référence : 4 sites construisent une `LineState`** dans `InvoiceForm.svelte`. `initLines()` (`:107-113`) **et** `reloadFromServer()` (`:418-424`) doivent tous deux recopier `revenueAccountId` depuis la réponse serveur ; `addFreeLine` (`:251-257`) et `onProductSelect` (`:261-267`) l'initialisent à `null` (16-2 y branchera le pré-remplissage catalogue).
  **Pourquoi un décompte explicite** (même geste qu'AC5-ter côté 16-1a) : un oubli sur `reloadFromServer` n'est visible qu'**après un conflit de version**. Chemin complet : conflit optimiste → modale → « Recharger » (`:663`) → les lignes sont remappées sans `revenueAccountId` → toutes les ventilations retombent à `NULL` **en silence** → le ré-enregistrement les efface en base. `stripUiKey` (`:95-98`) étant un spread, rien ne signale l'absence du champ. C'est la nullification silencieuse de #271, simplement pas au site où la passe 1 l'avait cherchée.
- **AC10** — Types TS et clients API alignés, **des DEUX côtés** : `revenueAccountId?: number | null` optionnel en création et modification et restitué en lecture dans `frontend/src/lib/features/invoices/invoices.types.ts` (`InvoiceLineResponse`, `:14`) et `invoices.api.ts` ; **et** `CreditNoteLineResponse` (`frontend/src/lib/features/credit-notes/credit-notes.types.ts:8`) + `credit-notes.api.ts` en **lecture seule** — 16-1a D5 ajoute bien la colonne côté avoir, son pendant TS n'était couvert par aucun AC avant la passe 4.

### C. i18n

- **AC11** — Toutes les chaînes **nouvelles de cette story** sont ajoutées aux **4 catalogues** `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`. Aucun libellé codé en dur dans le composant ou le formulaire — tout passe par `i18nMsg(key, fallback)`.
- **AC11-bis** — **Parité pré-existante hors périmètre.** Mesuré en passe 2 : `fr-CH` compte **1225** clés, `de-CH` / `en-CH` / `it-CH` en comptent **1168** — **57 clés absentes** des trois locales non-françaises, dont plusieurs de facturation (`invoice-error-configuration-required`, `invoice-validate-button`, `invoice-status-validated-label`, `error-fiscal-year-invalid`…). C'est une **dette antérieure**, sans lien avec cette story. AC11 porte sur les clés **nouvelles** uniquement ; ne pas transformer 16-1b en chantier de traduction. Conformément à la § « Issue Tracking Rule », ouvrir une **issue GitHub** (`known_failure.yml`, labels `known-failure` + `technical-debt`) pour l'écart de parité, et la référencer ici. Ne pas la corriger dans cette story.
- **AC12** — `npm run lint-i18n-ownership` **PASS**. Les nouvelles clés consommées depuis `lib/components/invoices/` sont **hors périmètre du lint** (il ne parcourt que `src/lib/features`) ; celles ajoutées à `AccountAutocomplete`, qui reste dans `features/journal-entries/`, doivent soit utiliser un namespace global (`common-*`, `error-*`, cf. `GLOBAL_NAMESPACES` `:16`), soit être ajoutées à `KNOWN_VIOLATIONS`. **Préférer le namespace global** — ne pas allonger la liste de dette #30.

### D. Tests

- **AC13** — Tests du composant, **tous orientés non-régression des 4 consommateurs** : `allowClear` absent ⇒ comportement strictement identique à aujourd'hui, y compris l'effacement clavier qui ne doit **rien** nullifier ; `markInvalid` absent ⇒ **aucun** marqueur, **y compris sur un compte `Expense` non-postable** — le fixture doit en contenir un, faute de quoi le test passerait aussi avec un marqueur inconditionnel ; `allowClear` présent ⇒ le bouton **et** l'effacement clavier au `blur` appellent `onSelect(null)` une seule fois (AC1, AC1-bis) ; `markInvalid` présent ⇒ libellé affiché **et** marqueur (AC2).
- **AC14** — Tests du formulaire : sélection d'un compte par ligne ; repli affiché avec le compte par défaut (D9) ; **2 lignes invalides sur 4 → le message les nomme toutes les deux, les 2 lignes saines ne portent aucun marqueur, et corriger une seule laisse l'enregistrement bloqué avec un message ne nommant plus que l'autre** (AC8) ; `defaultRevenueAccountId` non-postable + une ligne le désignant explicitement → **aucun** marqueur, enregistrement possible (pendant frontend de l'AC19 backend) ; ajout / suppression (AC9) ; **après `reloadFromServer` — conflit `OPTIMISTIC_LOCK_CONFLICT` puis « Recharger » — les comptes des lignes rechargées sont ceux renvoyés par le serveur** (AC9-bis) ; échec de chargement des comptes non bloquant (AC7).
- **AC14-bis** — Tests des **écrans de détail** (AC6-bis, absents de la couverture jusqu'à la passe 4) : ligne avec compte renseigné → numéro + libellé affichés ; ligne de brouillon sans compte → défaut société avec mention « (défaut) » ; **compte archivé référencé → libellé affiché** (non-régression de D11 côté lecture, le piège se rejoue à l'identique ici) ; écran de détail d'avoir → aucune mention « (défaut) », **y compris pour une ligne d'avoir à `NULL`** (avoir antérieur au déploiement non backfillé ou non backfillable, cf. story **16-1a-bis**) qui doit afficher un **tiret** et non le compte par défaut.
- **AC15** — Test E2E Playwright : créer une facture à 2 lignes sur 2 comptes différents, la valider, vérifier que l'écriture générée porte 2 lignes de crédit produit distinctes. (Pré-requis : MariaDB + seed CI + `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+.)
- **AC16** — Gate « Test Locally First » frontend complet vert (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`) **et** backend vert (les `.ftl` sont dans un crate Rust : `cargo test --workspace` doit passer).

### E. Doc-sync

- **AC17** — `docs/manual/fr/user-manual.tex` : la description des lignes de facture (`:574`) mentionne le sélecteur de compte de produit et le repli sur le compte par défaut ; l'exemple d'écriture (`:591`) reste valide ou est complété d'un cas ventilé. PDF régénéré (`make fr` dans `docs/manual/`) et commité (convention projet : les PDF sont versionnés).
- **AC18** — README section « Fonctionnalités » : la ventilation par ligne est mentionnée si elle y a sa place. `CHANGELOG.md` `[Non publié]` : entrée orientée utilisateur couvrant **16-1a + 16-1b** (la capacité n'est utilisable qu'une fois les deux livrées).

---

## Tasks / Subtasks

- [x] **T1** — `AccountAutocomplete` : props opt-in `allowClear` / `markInvalid` / `requiredAccountType` (AC1, AC2) + bouton d'effacement + sémantique du champ vidé au clavier (AC1-bis) ; import `i18nMsg` canonique (AC3). Aucun déplacement de fichier (AC4).
- [x] **T2** — Types TS + client API `revenueAccountId` (AC10).
- [x] **T3** — `InvoiceForm` : chargement de la liste des comptes via `fetchAccounts(true)` (D11) + mode dégradé (AC5, AC5-bis, AC7).
- [x] **T4** — `InvoiceForm` : sélecteur par ligne + placeholder « défaut société » + propagation de `revenueAccountId` sur les **4** sites de construction de `LineState` (AC5, AC6, AC9, AC9-bis).
- [x] **T5** — Blocage global d'enregistrement dès qu'une ligne est invalide, message nommant toutes les lignes fautives (AC8).
- [x] **T5-bis** — Écrans de détail facture **et** avoir : chargement de `fetchAccounts(true)` + `getInvoiceSettings()` (facture seule), puis colonne compte de produit en lecture seule (AC6-bis, AC10 volet avoir).
- [x] **T6** — i18n : nouvelles clés dans les 4 `.ftl`, namespace global de préférence (AC11, AC12).
- [x] **T7** — Tests composant, formulaire et écrans de détail (AC13, AC14, AC14-bis).
- [ ] **T8** — Test E2E Playwright (AC15).
- [ ] **T9** — Doc-sync : manuel utilisateur + PDF, README, CHANGELOG (AC17, AC18).
- [ ] **T10** — Gate complet frontend + backend (AC16).

**Ordre conseillé** : T2 → T1 → T3 → T4 → T5 → T6 → T7 → T8 → T9 → T10.

---

## Dev Notes

### Ancres ground-truth (vérifiées en passe 1 de `validate`, 2026-07-26, commit `ef6cdf52`)

| Élément | Emplacement |
|---|---|
| `AccountAutocomplete` — `Props` | `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte:6-14` |
| … résolution du libellé sur la liste **complète** | `:22-30` |
| … filtre `active && postable` du dropdown | `:32-34` |
| … `handleInput` (n'appelle `onSelect` qu'en `loadError`) | `:52-60` |
| … import `i18nMsg` non canonique | `:3` |
| Emplacement i18n canonique (doc-comment) | `frontend/src/lib/shared/utils/i18n.svelte.ts:1-7`, `i18nMsg` `:14` |
| Importeurs (4) | `JournalEntryForm.svelte:22`, `VatPurchaseAssistant.svelte:14`, `TransactionSplitModal.svelte:22`, `ManualMatchModal.svelte:24` |
| Test du composant | `frontend/src/lib/features/journal-entries/AccountAutocomplete.test.ts:14` |
| `InvoiceForm` — imports et chargements existants | `frontend/src/lib/components/invoices/InvoiceForm.svelte:14-37`, `:203` |
| **`fetchAccounts(includeArchived = false)` — défaut EXCLUANT les archivés** | `frontend/src/lib/features/accounts/accounts.api.ts:10-14` |
| **Précédent `fetchAccounts(true)` + son motif commenté** | `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte:33-43` |
| … contre-exemple : écran de création | `frontend/src/routes/(app)/journal-entries/+page.svelte:100` |
| `InvoiceSettingsResponse.defaultRevenueAccountId` (pour D9) | `frontend/src/lib/features/invoices/invoices.types.ts:80-84` |
| Catalogues i18n (4 locales, **crate Rust**) | `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` |
| Clés existantes du composant | `fr-CH/messages.ftl:233`, `:246` |
| `lint-i18n-ownership` — périmètre et namespaces globaux | `frontend/scripts/lint-i18n-ownership.js:16-17`, `:225-226` |
| … `KNOWN_VIOLATIONS` du composant | `lint-i18n-ownership.js:46-47` |
| 14-3b L5 — `AccountAutocomplete` **non concerné** par #271 | `_bmad-output/implementation-artifacts/14-3b-consommateurs-roles.md:191` |
| Manuel utilisateur — lignes de facture / écriture | `docs/manual/fr/user-manual.tex:574`, `:591` |

### Pièges, par ordre de coût

0. **`fetchAccounts(true)` (D11)** — le piège n°1, parce qu'il est **silencieux** : `fetchAccounts()` compile, s'exécute, et marche parfaitement tant qu'aucun compte n'a été archivé. Le jour où un compte l'est, le champ de la ligne concernée s'affiche vide. AC5-bis est le seul garde-fou.
1. **Régression sur les 4 consommateurs existants (D6/AC4)** — toute extension du composant doit avoir un **défaut qui préserve le comportement actuel**. `JournalEntryForm`, `VatPurchaseAssistant`, `ManualMatchModal` et `TransactionSplitModal` ne doivent voir aucune différence. AC13 est le garde-fou.
2. **L'affordance de remise à `null` (D8)** — sans elle, le champ optionnel devient un aller simple. Facile à oublier parce que ça ne casse aucun test existant.
3. **i18n dans un crate Rust** — les `.ftl` sont côté backend. Un ajout de clé exige `cargo test --workspace`, pas seulement le gate frontend (AC16).
4. **Namespace i18n (AC12)** — une clé `invoice-*` consommée depuis `features/journal-entries/AccountAutocomplete.svelte` déclenche le lint. Utiliser un namespace global plutôt que d'allonger `KNOWN_VIOLATIONS`.
5. **PDF du manuel (AC17)** — la convention projet versionne les PDF ; régénérer et commiter, sinon le `.tex` et le `.pdf` divergent silencieusement.

### Propagation post-patch (§ CLAUDE.md)

Symptômes à greper après chaque patch de remédiation : `revenueAccountId`, `AccountAutocomplete`, `allowClear`, les clés i18n nouvelles (dans les **4** `.ftl` **et** les fallbacks Svelte), `default_revenue_account_id`.

### References

- Issue **#152**, CR **#265**. Dette **#271** (nullification silencieuse des `<select>` natifs de config — **ne concerne pas** `AccountAutocomplete`, cf. 14-3b L5) ; dette **#30** (`KNOWN_VIOLATIONS` du lint i18n).
- Story **16-1a** — socle backend dont dépend celle-ci.
- Stories antérieures : **14-3b** (filtre `postable` du composant, limitation L5), **3-2** (saisie d'écritures, origine du composant).

---

## Change Log

### Passe 1 de `validate` — 2026-07-26 (Sonnet, 3 lentilles : BlindHunter / EdgeCaseHunter / AcceptanceAuditor)

Issue du split de la story 16-1 (§ « Règle de splitting préventif », 8 modules > seuil de 5), arbitré par Guy. Comportement D7 arbitré par Guy : afficher + signaler + bloquer la ligne.

**Deux décisions de la spec initiale renversées, sur prémisses factuellement fausses** :

| Décision initiale | Ce que dit le code |
|---|---|
| D6 : déplacer `AccountAutocomplete`, car `lint-i18n-ownership` sanctionnerait l'import cross-feature | Le lint ne parcourt que `src/lib/features` (`:17`, `:225-226`) et ne contrôle **que** les clés i18n, jamais les imports. `reconciliation` importe déjà ce composant cross-feature **sans violation**. Le déplacement le ferait **sortir** du périmètre du lint. → **D6 révisée : ne pas déplacer, étendre par props opt-in.** |
| D7 : un compte persisté invalide s'afficherait **vide** et serait nullifié en silence (dette #271) | Le `$effect` résout le libellé sur la liste **complète** (`:22-30`), pas la liste filtrée ; `handleInput` n'appelle `onSelect` qu'en `loadError` (`:52-60`). **14-3b L5 le documentait déjà** : « `AccountAutocomplete` n'est pas concerné ». → **D7 révisée : le libellé s'affiche ; ce qui manque est le signal d'invalidité et l'affordance de retour à `null`.** |

**Sous-comptage corrigé** : la spec initiale nommait **2** importeurs du composant (`JournalEntryForm`, `VatPurchaseAssistant`) ; il y en a **4** — `TransactionSplitModal.svelte:22` et `ManualMatchModal.svelte:24` étaient omis. Un déplacement suivi à la lettre aurait cassé `npm run build`. Le sujet devient sans objet avec D6 révisée, mais l'écart est tracé.

**Décisions nouvelles** : D8 (affordance de retour à `null` — impossible aujourd'hui hors mode dégradé, bloquant pour un champ optionnel), D9 (le repli affiche le compte par défaut nommé, pas un tiret), D10 (échec de chargement des comptes non bloquant, réutilise le mode `loadError` existant).

**Corrections d'ancrage** : les catalogues i18n sont dans un **crate Rust** (`crates/kesh-i18n/locales/*/messages.ftl`), pas côté frontend — le gate d'AC16 inclut donc `cargo test --workspace`. `InvoiceForm` ne charge **aucune** liste de comptes aujourd'hui : AC5 est de la plomberie nette, pas un simple ajout de champ. AC12 reformulé : le lint ne couvre pas `lib/components/`, seules les clés ajoutées au composant (resté dans `features/`) sont concernées.

**Question ouverte n°3 levée par hypothèse** : le compte de la fiche produit (16-2) ne fera que **pré-remplir** la ligne côté frontend, sans reprise rétroactive sur les factures existantes.

### Passe 2 de `validate` — 2026-07-26 (Haiku 4.5, contexte frais)

**2 findings**, dont un trouvé par l'orchestrateur en vérifiant le travail du reviewer.

**[Orchestrateur] `fetchAccounts(true)` — la dépendance qui porte toute la décision D7.** Le reviewer avait conclu que « la disponibilité de la liste des comptes est acquise, `fetchAccounts()` existe déjà ». Vérification : `fetchAccounts(includeArchived = false)` (`accounts.api.ts:10`) **exclut les archivés par défaut**. Or D7 (révisée en passe 1) repose entièrement sur la résolution du libellé par `accounts.find((a) => a.id === value)` — si le compte archivé n'est pas dans le tableau, le champ **s'affiche vide**, exactement le symptôme #271 que D7 affirme ne pas s'appliquer. La révision de D7 est correcte sur le **composant**, mais n'est vraie qu'à la condition que l'appelant fournisse la liste complète. Le dépôt a déjà rencontré et résolu ce cas : `journal-entries/[id]/+page.svelte:33-43` utilise `fetchAccounts(true)` avec le motif en commentaire, alors que l'écran de création (`:100`) utilise `fetchAccounts(false)`. → **D11** ajoutée, **AC5** amendé, **AC5-bis** (test de non-régression) ajouté, **AC2** doté d'une définition opérationnelle du marqueur d'invalidité, piège n°0 des Dev Notes.

**[Haiku] Parité des catalogues i18n.** Mesure confirmée : `fr-CH` = **1225** clés, `de-CH` / `en-CH` / `it-CH` = **1168** → **57 clés manquantes**, dont plusieurs de facturation. Le reviewer en concluait que AC11 ne peut pas passer — **surévalué** : AC11 porte sur les chaînes **nouvelles**, qu'il suffit d'ajouter aux 4 fichiers. L'écart est une dette antérieure sans lien avec la story. → **AC11-bis** ajouté : périmètre clarifié + ouverture d'une issue GitHub (Issue Tracking Rule), sans corriger dans cette story.

**Confirmations utiles du reviewer** (pas des findings, mais elles valident la faisabilité) : `fetchAccounts` existe ; `InvoiceSettingsResponse.defaultRevenueAccountId` existe déjà (`invoices.types.ts:84`), donc D9 est réalisable sans appel supplémentaire ; le `$effect` remet bien `query` à `''` quand `value` devient `null`, donc le bouton d'effacement de D8 ne sera pas écrasé.

**Trend** : passe 1 (dans la story 16-1 unifiée) = 28 findings → passe 2 = 2 findings, aucun CRITICAL/HIGH. Convergence monotone.

### Passe 3 de `validate` — 2026-07-26 (Opus, contexte frais)

**7 findings : 2 HIGH, 4 MEDIUM, 1 LOW.** Tous vérifiés en ground-truth. **La sévérité remonte** par rapport à la passe 2 (0 HIGH) — arbitrage en fin de section.

| Finding | Verdict | Traitement |
|---|---|---|
| **HIGH — AC2 prescrivait le marqueur d'invalidité SANS prop d'activation**, et avec un critère (`Revenue`) propre à la facture, dans un composant partagé par 4 écrans. `JournalEntryForm` et les 2 modales de rapprochement sélectionnent massivement des comptes `Expense` / `Asset` / `Liability` : un marqueur au pied de la lettre d'AC2 afficherait « compte invalide » sur **quasiment toutes** les lignes d'écriture manuelle | Réel — erreur de placement, pas oubli de défaut. AC13 ne couvrait que `allowClear`, le marqueur n'avait aucun garde-fou | **AC2 réécrit** : deux props opt-in `markInvalid` (défaut `false`) et `requiredAccountType` (défaut `undefined`). **AC13 étendu** avec un fixture contenant un compte `Expense` non-postable — sans lui, le test passerait aussi avec un marqueur inconditionnel. Nom corrigé : `accountType` camelCase (`accounts.types.ts:50`) |
| **HIGH — D8 + D9 combinées créent un piège comptable silencieux.** Effacer le texte au clavier ne vide **pas** la valeur (`handleInput:49-60` n'appelle `onSelect` qu'en `loadError`, `handleBlur:88-93` ne restaure rien), mais vide le champ — ce qui déclenche le placeholder de D9 « défaut société ». L'utilisateur enregistre convaincu d'avoir remis la ligne au défaut ; la facture se poste sur l'ancien compte. **L'UI affirme activement le contraire de l'état réel** | Réel. Ni D8 ni D9 ne le créent isolément — c'est leur combinaison. Sur un champ devenu optionnel, *tout sélectionner + Suppr* est le geste naturel, pas le bouton | **AC1-bis** ajouté : au `blur`, `query` vide + `value ≠ null` ⇒ `onSelect(null)` ; `query` non vide sans sélection validée ⇒ restauration du libellé. Le champ ne peut jamais contredire la valeur liée. **Inactif si `allowClear = false`**, sinon `JournalEntryForm` nullifierait ses lignes au `blur` (dette #271). **AC1 corrigé** : le `$effect` (`:27-29`) reste la source unique de vérité pour vider `query` |
| MEDIUM — l'exemption D3-bis de 16-1a n'était pas répercutée : backend `200 OK`, frontend marqueur + blocage. Utilisateur enfermé sur un brouillon qu'il ne peut plus enregistrer | Réel. 16-1a écrit trois paragraphes pour interdire cette asymétrie, 16-1b la réintroduisait une couche plus haut | Exception ajoutée à la définition du marqueur (sous D11) et à AC2 ; test correspondant en AC14. Donnée déjà disponible : `InvoiceForm.svelte:344` |
| MEDIUM — **un des deux sites de mapping serveur → lignes était omis** : `reloadFromServer` (`:418-424`) en plus d'`initLines` (`:107-113`) | Réel, vérifié par `grep -nF "vatRate: l.vatRate,"` → 2 occurrences. Un oubli n'est visible qu'**après un conflit de version**, et efface les comptes en base au ré-enregistrement suivant — la nullification #271, pas au site où la passe 1 l'avait cherchée | **AC9-bis** ajouté avec décompte de référence des **4** sites de construction de `LineState` (même geste qu'AC5-ter côté 16-1a) + test après conflit optimiste en AC14 |
| MEDIUM — AC8 « les autres lignes ne sont pas bloquées » décrivait un grain **inexistant** : la persistance est un appel unique portant toutes les lignes (`:359-369`), le `disabled` du bouton (`:617`) n'a aucune condition par ligne, et `validateClient` (`:284-313`) s'arrête à la **première** erreur alors que 16-1a D6 impose de nommer **toutes** les lignes | Réel — AC14 demandait un test inécrivable tel quel | **AC8 réécrit** (blocage global, message multi-lignes) + bullets de D7 reformulés + AC14 précisé |
| MEDIUM — **angle mort UX** : l'écran de détail (`invoices/[id]/+page.svelte:753-773`) n'a aucune colonne de compte, et l'édition n'est offerte qu'aux brouillons (`:563-578`). Dès qu'une facture est validée — l'instant où la ventilation produit son effet — la donnée devient **write-once** | Réel, et en contradiction directe avec le motif de D9. C'est le geste de contrôle n°1 d'une fiduciaire | **AC6-bis** ajouté (détail facture + avoir, lecture seule). PDF explicitement inchangé — information interne |
| LOW — AC9/AC14 exigeaient un test de **réordonnancement** de lignes : la fonctionnalité n'existe pas (seuls `addFreeLine`, `onProductSelect`, `removeLine` ; le commentaire `:86` anticipe une capacité jamais livrée) | Réel | Retiré d'AC9 et AC14, avec la mention explicite « ne pas l'ajouter » pour couper le scope creep |

**Vérifié négatif (utile)** : **D8 + D11 ne créent aucune boucle ni écrasement** sur le chemin nominal. Le `$effect` (`:21-30`) dépend de `value` / `accounts` / `loadError` et n'écrit `query` que dans ce sens ; rien ne réécrit `value` → pas de cycle. Bouton → `onSelect(null)` → parent `value = null` → `else if (value === null) query = ''` : convergent et idempotent, y compris si le parent est asynchrone. Par ailleurs le contrat 16-1a ↔ 16-1b tient : `InvoiceForm.svelte:381-389` (`isApiError` → `errorMsg` + toast) affiche verbatim le message « Ligne 3 : … » de 16-1a D7 ; `#[serde(default)]` ↔ `revenueAccountId?: number | null` ↔ `stripUiKey` (spread) sont cohérents. Le cas « archivé » chez les 3 autres consommateurs est **réfuté** : ils chargent tous `fetchAccounts(false)` / `fetchAccounts()`, et `journal-entries/[id]` — seul appelant de `fetchAccounts(true)` — ne rend **aucun** `AccountAutocomplete`.

**Réserve pré-existante, hors périmètre** : les messages de validation du backend sont en français en dur (`routes/invoices.rs:370-410`) — un utilisateur DE/IT/EN les recevra en français. Convention du fichier, à ne pas ouvrir ici ; à rapprocher de l'écart de parité i18n d'AC11-bis.

**Trend et arbitrage de convergence** : passe 1 = 28 (story unifiée) → passe 2 = 2 → passe 3 = 7 dont 2 HIGH. La sévérité **remonte**, ce qui coche le second critère de la § « Règle de splitting préventif ». Ligne de fracture observée : F-1, F-2 et le nit `query` portent tous sur **l'extension du composant partagé** ; F-3 à F-7 portent sur **le formulaire**. Un split `16-1b-α` (composant, revu contre ses 4 consommateurs) / `16-1b-β` (formulaire, types, i18n, doc-sync) suivrait exactement cette ligne. **Arbitrage rendu par Guy le 2026-07-26 : PAS de re-split.** Motif retenu — la remontée vient d'un angle jamais regardé (le contrat du composant partagé avec ses 4 consommateurs), pas d'une story trop large : aucun finding des passes 1-2 n'a été remis en cause. Cet angle est désormais couvert par AC1 / AC1-bis / AC2 / AC13, et la passe 4 n'a plus rien trouvé au-dessus de MEDIUM dessus (ses 2 MEDIUM portaient sur AC6-bis, l'écran de détail). 16-1b reste à 5 modules, au seuil et non au-dessus. **Condition posée : si la passe 5 remonte encore un HIGH, le split en `16-1b-α` (composant) / `16-1b-β` (surface facture) devient non discutable.**

### Passe 4 de `validate` — 2026-07-26 (Sonnet, contexte frais)

**2 findings MEDIUM**, tous deux sur **AC6-bis** — l'AC introduit tardivement en passe 3 et jamais recoupé avec le reste du document. Rien au-dessus de MEDIUM.

| Finding | Verdict | Traitement |
|---|---|---|
| MEDIUM — AC6-bis exige une plomberie que ni AC10 ni T5-bis ne couvraient : `CreditNoteLineResponse` (`credit-notes.types.ts:8`) n'a pas le champ et n'était visé par aucun AC alors que 16-1a D5 ajoute bien la colonne côté avoir ; `InvoiceLineResponse` ne porte qu'un **id**, pas de libellé, et **aucun** des deux écrans de détail ne charge `fetchAccounts` ni `getInvoiceSettings` | Réel — exactement l'omission que D11/AC5-bis a été créé pour prévenir côté formulaire, non répercutée côté détail | **AC10** étendu au volet avoir ; **AC6-bis** doté de sa clause de plomberie (`fetchAccounts(true)` + `getInvoiceSettings`) ; **T5-bis** réécrit. Précision ajoutée : côté avoir la mention « (défaut) » ne s'applique **jamais**, le compte y étant toujours matérialisé (16-1a D2) |
| MEDIUM — AC6-bis n'avait **aucun test**, alors que le document le qualifie de « geste de contrôle n°1 d'une fiduciaire » et que toutes les autres décisions de la passe 3 ont le leur | Réel | **AC14-bis** ajouté (4 cas, dont le compte archivé qui rejoue le piège de D11 côté lecture) ; **T7** étendu |

**Vérifié négatif — deux inquiétudes de la passe 3 levées** : (1) **pas de course `mousedown` → `blur` sur AC1-bis** — `onmousedown` porte déjà `e.preventDefault()` (`AccountAutocomplete.svelte:121-123`), pattern combobox standard qui empêche le `blur` de se déclencher lors d'un clic sur un item ; la logique `onSelect(null)` ne peut donc pas s'exécuter dans cette séquence. (2) **AC1-bis n'exige aucun nouvel état** : recalculer le libellé depuis `value`/`accounts` au `blur` suffit. Confirmé aussi : les 4 consommateurs ne passent aucune des nouvelles props (défauts inchangés), l'écran de détail d'avoir existe bien, et les 10 ancres de la passe 3 pointent exactement sur le code décrit.

**Trend** : 28 (story unifiée) → 2 → 7 → 2, aucun HIGH en passe 4. Sévérité au-dessus de LOW → **passe 5 requise**.

### Passe 5 de `validate` — 2026-07-26 (Haiku 4.5, contexte frais + vérification orchestrateur)

**1 finding LOW.** Le reviewer a rendu « 0 finding » ; le LOW vient de la vérification d'orchestrateur. *(Sur 16-1a, la même vérification a en revanche retourné un HIGH que le reviewer avait manqué — cf. Change Log de 16-1a, passe 5 : un « rien trouvé » de Haiku n'est pas une preuve de convergence.)*

| Finding | Verdict | Traitement |
|---|---|---|
| LOW — AC6-bis ajoute une colonne à un tableau dont les **5 en-têtes existants sont en français codé en dur** (`invoices/[id]/+page.svelte:754-759`), alors que l'écran d'avoir, traité au même AC, utilise `i18nMsg` (`credit-notes/[id]/+page.svelte:116-120`). AC11 exige « aucun libellé codé en dur » sans dire lequel des deux régimes s'applique — le dev peut aussi bien i18n-iser la seule colonne nouvelle (incohérent avec ses 5 voisines) que rouvrir l'écran entier (hors périmètre, +5 clés × 4 catalogues) | Réel, ambiguïté de rédaction | Clause ajoutée à AC6-bis : **convention du fichier modifié** — en dur côté facture, `i18nMsg` côté avoir. Périmètre d'AC11 explicitement borné aux chaînes du sélecteur et du formulaire. Écart tracé, non corrigé |

**Répercussion de 16-1a** : le backfill étant **délibérément incomplet** (une écriture éditée n'est pas backfillable) — et depuis le 2026-07-26 **extrait dans la story `16-1a-bis`**, donc livrable séparément —, une ligne d'**avoir antérieur au déploiement** peut porter `revenue_account_id = NULL`. AC6-bis prescrivait « côté avoir le compte est toujours matérialisé » : nuance ajoutée — afficher un tiret, jamais la mention « (défaut) », qui affirmerait un repli qui n'aura pas lieu.

**Vérifié négatif** : les ancres de la passe 4 sont exactes — `invoices/[id]/+page.svelte:753-773` est bien le tableau des lignes à 5 colonnes sans compte, `:563-578` bien le bloc d'actions réservé aux brouillons, et l'écran d'avoir porte bien un tableau de lignes (`credit-notes/[id]/+page.svelte:113-124`, `{#each creditNote.lines as line (line.position)}`). Aucun renvoi vers un AC/D/T inexistant. Le contrat des props opt-in avec les 4 consommateurs tient.

**Trend** : 28 (story unifiée) → 2 → 7 → 2 → **1 LOW**. **Critère d'arrêt de la § « Review Iteration Rule » atteint pour 16-1b** : plus aucun finding au-dessus de LOW. Aucune passe supplémentaire requise sur cette story.

**Condition de split posée par Guy le 2026-07-26** (« si la passe 5 remonte encore un HIGH, le split `16-1b-α` / `16-1b-β` devient non discutable ») : **non déclenchée** — passe 5 = 1 LOW, aucun HIGH. 16-1b reste **non splittée** et est convergée. *(À noter pour l'ordonnancement : 16-1b dépendant de 16-1a, elle n'est implémentable qu'après elle, laquelle est encore en boucle de validation.)*

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `bmad-dev-story`, démarrée le 2026-07-30.

### Implementation Plan

Ordre suivi : **T2 → T1** (+ tests AC13 et les 2 clés globales de T6) → T3 → T4 → T5 → T5-bis → T6 → T7 → T8 → T9 → T10.

`T2` d'abord parce qu'il ne dépend de rien et débloque tout le reste ; `T1` ensuite parce que c'est le seul fichier partagé par 4 écrans étrangers à la story — le faire tôt laisse le maximum de gate en aval pour attraper une régression.

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
