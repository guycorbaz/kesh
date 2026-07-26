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
- l'**enregistrement du formulaire est bloqué** tant que cette ligne précise n'a pas reçu un choix explicite (soit un compte valide, soit un retour à « défaut société ») ;
- les **autres lignes ne sont pas bloquées** par l'invalidité d'une ligne voisine.

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

**Définition opérationnelle du marqueur d'invalidité (AC2)** : la valeur est présente dans `accounts` mais **absente** de la liste filtrée `active && postable`, ou son `account_type` n'est pas `Revenue`.

---

## Acceptance Criteria

### A. Composant

- **AC1** — `AccountAutocomplete` accepte `allowClear?: boolean` (défaut `false`) : quand `true`, un bouton d'effacement accessible (`aria-label` traduit) appelle `onSelect(null)` et vide le champ (D8).
- **AC2** — `AccountAutocomplete` accepte de quoi signaler une valeur invalide (D7) : quand la valeur liée est **présente dans `accounts` mais absente de la liste filtrée `active && postable`** (ou d'un `account_type` ≠ `Revenue`), le libellé reste affiché **et** un marqueur d'invalidité textuel apparaît. Rendu accessible : pas de signal porté par la seule couleur. Prérequis : la liste complète est fournie par l'appelant (D11 / AC5).
- **AC3** — L'import `i18nMsg` du composant passe de `$lib/features/onboarding/onboarding.svelte` (`:3`) à l'emplacement canonique `$lib/shared/utils/i18n.svelte`, conformément à la doc-comment de `shared/utils/i18n.svelte.ts:1-7`. Comportement inchangé.
- **AC4** — Le composant **n'est pas déplacé** (D6). Les 4 importeurs (`JournalEntryForm.svelte:22`, `VatPurchaseAssistant.svelte:14`, `TransactionSplitModal.svelte:22`, `ManualMatchModal.svelte:24`) et `AccountAutocomplete.test.ts` gardent leur chemin d'import actuel.

### B. Formulaire de facture

- **AC5** — `InvoiceForm.svelte` charge la liste des comptes de la société via **`fetchAccounts(true)`** (`accounts.api.ts:10`) — le flag `includeArchived` est **obligatoire**, cf. D11 : sans lui, un compte archivé persisté n'est pas dans le tableau, le `$effect` ne résout pas son libellé et le champ s'affiche vide, ce qui invalide toute la décision D7. Plomberie nette : aucun chargement de comptes n'existe dans ce formulaire aujourd'hui. Le formulaire expose ensuite un sélecteur de compte **par ligne**, optionnel.
- **AC5-bis** — Test de non-régression de D11 : un compte **archivé** référencé par une ligne de brouillon s'affiche avec son libellé (et son marqueur d'invalidité), et **n'apparaît pas** dans les propositions du dropdown. C'est le test qui échoue si quelqu'un « simplifie » `fetchAccounts(true)` en `fetchAccounts()`.
- **AC6** — D9 : une ligne sans compte affiche en placeholder le **compte par défaut de la société** (numéro + libellé), issu des réglages déjà chargés par `getInvoiceSettings`. La ligne reste à `NULL` — le placeholder n'est jamais persisté.
- **AC7** — D10 : l'échec du chargement de la liste des comptes ne bloque pas la saisie ; le mode dégradé `loadError` existant est utilisé.
- **AC8** — D7 : une ligne dont le compte persisté est invalide affiche le libellé **et** le marqueur d'invalidité ; l'enregistrement du formulaire est **bloqué** tant que cette ligne n'a pas reçu un choix explicite (compte valide ou retour à « défaut société ») ; les autres lignes ne sont pas bloquées.
- **AC9** — Ajout, suppression et réordonnancement de lignes conservent l'association ligne ↔ compte. (`InvoiceForm` clé son `{#each}` par `_uiKey` et non par index — vérifié en passe 1, aucun bug de réordonnancement pré-existant à corriger, mais la nouvelle donnée doit suivre la même clé.)
- **AC10** — Types TS et client API alignés : `revenueAccountId?: number | null` optionnel en création et modification, restitué en lecture (`frontend/src/lib/features/invoices/invoices.types.ts` et `invoices.api.ts`).

### C. i18n

- **AC11** — Toutes les chaînes **nouvelles de cette story** sont ajoutées aux **4 catalogues** `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`. Aucun libellé codé en dur dans le composant ou le formulaire — tout passe par `i18nMsg(key, fallback)`.
- **AC11-bis** — **Parité pré-existante hors périmètre.** Mesuré en passe 2 : `fr-CH` compte **1225** clés, `de-CH` / `en-CH` / `it-CH` en comptent **1168** — **57 clés absentes** des trois locales non-françaises, dont plusieurs de facturation (`invoice-error-configuration-required`, `invoice-validate-button`, `invoice-status-validated-label`, `error-fiscal-year-invalid`…). C'est une **dette antérieure**, sans lien avec cette story. AC11 porte sur les clés **nouvelles** uniquement ; ne pas transformer 16-1b en chantier de traduction. Conformément à la § « Issue Tracking Rule », ouvrir une **issue GitHub** (`known_failure.yml`, labels `known-failure` + `technical-debt`) pour l'écart de parité, et la référencer ici. Ne pas la corriger dans cette story.
- **AC12** — `npm run lint-i18n-ownership` **PASS**. Les nouvelles clés consommées depuis `lib/components/invoices/` sont **hors périmètre du lint** (il ne parcourt que `src/lib/features`) ; celles ajoutées à `AccountAutocomplete`, qui reste dans `features/journal-entries/`, doivent soit utiliser un namespace global (`common-*`, `error-*`, cf. `GLOBAL_NAMESPACES` `:16`), soit être ajoutées à `KNOWN_VIOLATIONS`. **Préférer le namespace global** — ne pas allonger la liste de dette #30.

### D. Tests

- **AC13** — Tests du composant : `allowClear` absent ⇒ comportement strictement identique à aujourd'hui (garde-fou de non-régression pour les 4 consommateurs) ; `allowClear` présent ⇒ le bouton remet à `null` ; valeur invalide ⇒ libellé affiché **et** marqueur présent.
- **AC14** — Tests du formulaire : sélection d'un compte par ligne ; repli affiché avec le compte par défaut (D9) ; blocage d'enregistrement sur ligne invalide sans blocage des autres (D7/AC8) ; ajout / suppression / réordonnancement (AC9) ; échec de chargement des comptes non bloquant (AC7).
- **AC15** — Test E2E Playwright : créer une facture à 2 lignes sur 2 comptes différents, la valider, vérifier que l'écriture générée porte 2 lignes de crédit produit distinctes. (Pré-requis : MariaDB + seed CI + `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+.)
- **AC16** — Gate « Test Locally First » frontend complet vert (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`) **et** backend vert (les `.ftl` sont dans un crate Rust : `cargo test --workspace` doit passer).

### E. Doc-sync

- **AC17** — `docs/manual/fr/user-manual.tex` : la description des lignes de facture (`:574`) mentionne le sélecteur de compte de produit et le repli sur le compte par défaut ; l'exemple d'écriture (`:591`) reste valide ou est complété d'un cas ventilé. PDF régénéré (`make fr` dans `docs/manual/`) et commité (convention projet : les PDF sont versionnés).
- **AC18** — README section « Fonctionnalités » : la ventilation par ligne est mentionnée si elle y a sa place. `CHANGELOG.md` `[Non publié]` : entrée orientée utilisateur couvrant **16-1a + 16-1b** (la capacité n'est utilisable qu'une fois les deux livrées).

---

## Tasks / Subtasks

- [ ] **T1** — `AccountAutocomplete` : prop `allowClear` + bouton d'effacement accessible (AC1) ; marqueur d'invalidité (AC2) ; import `i18nMsg` canonique (AC3). Aucun déplacement de fichier (AC4).
- [ ] **T2** — Types TS + client API `revenueAccountId` (AC10).
- [ ] **T3** — `InvoiceForm` : chargement de la liste des comptes via `fetchAccounts(true)` (D11) + mode dégradé (AC5, AC5-bis, AC7).
- [ ] **T4** — `InvoiceForm` : sélecteur par ligne + placeholder « défaut société » (AC5, AC6, AC9).
- [ ] **T5** — Blocage d'enregistrement sur ligne invalide, ligne par ligne (AC8).
- [ ] **T6** — i18n : nouvelles clés dans les 4 `.ftl`, namespace global de préférence (AC11, AC12).
- [ ] **T7** — Tests composant + formulaire (AC13, AC14).
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

---

## Dev Agent Record

### Agent Model Used

_(à compléter par `dev-story`)_

### Debug Log References

### Completion Notes List

### File List
