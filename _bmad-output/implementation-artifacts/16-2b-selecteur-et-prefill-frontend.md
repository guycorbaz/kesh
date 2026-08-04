# Story 16.2b : Compte de produit sur la fiche produit — sélecteur et pré-remplissage

## Status

ready-for-dev

## Story

**As a** fiduciaire ou indépendant qui facture des natures de prestations distinctes,
**I want** choisir le compte de produit d'un article **une fois** sur sa fiche, et le retrouver pré-rempli sur la ligne quand je monte une facture depuis le catalogue,
**so that** l'imputation comptable ne soit plus ressaisie à chaque facture — où elle est répétitive et donc oubliée.

Issue : **#144**. Sous-story de l'Epic 16, cible **v0.9.0**. **Née du split de 16-2** (4 passes de `validate`, cf. `16-2-compte-produit-catalogue.md`).

⚠️ **Doit partir dans la MÊME PR que 16-2a**, qui livre la colonne, l'API et la validation. Cette story n'a rien à lire sans elle.

---

## Contexte — le pont existe déjà, et 16-1 y a laissé l'ancre

`InvoiceForm.svelte` propose deux boutons, « Ligne libre » et « **Depuis catalogue** » (`:862-871`). Le second ouvre `ProductPicker.svelte`, dont le choix appelle `onProductSelect(p)` (`:420-430`) et **recopie** `p.name` → `description`, `p.unitPrice`, `p.vatRate`.

Ce site porte déjà, écrite par 16-1, l'ancre de cette story :

```ts
// AC9-bis — site 3/5. Idem : 16-2 y branchera le compte du produit.
revenueAccountId: null,
```

*(« site 3/5 » renvoie aux **cinq** constructions de `LineState` d'`InvoiceForm.svelte`, numérotées par 16-1b sous `AC9-bis — site N/5`. Seuls les sites **2** — `addFreeLine` — et **3** — `onProductSelect` — concernent cette story ; les trois autres recopient une valeur déjà décidée.)*

**Le pré-remplissage est donc une recopie de plus, à un endroit déjà écrit pour la recevoir.** Il ne demande aucune colonne `product_id`, aucune relation persistante, aucun changement du contrat des lignes. `ProductPicker` transmet l'objet **complet** (`:19` `onSelect: (p: ProductResponse) => void` ; `:59-60`, `pick` reçoit le `ProductResponse` et le passe tel quel), la chaîne est donc intacte dès que `ProductResponse` porte le champ — ce que **16-2a** livre.

---

## Décisions

- **D-B1 — Un compte devenu inutilisable est pré-rempli quand même, puis signalé au niveau de la LIGNE.** Si l'article référence un compte archivé, retypé ou devenu non imputable, `onProductSelect` recopie la valeur telle quelle ; le sélecteur de ligne l'affiche, le marque `markInvalid` et bloque l'enregistrement en nommant la ligne. **Motif** : c'est l'arbitrage rendu par Guy en 16-1b — *afficher + signaler + bloquer* — et la machinerie existe (`isAccountUnusable`, `AccountAutocomplete markInvalid`, `allowClear`). Ne pas pré-remplir, et retomber en silence sur le défaut société, **cacherait** que la fiche produit est à corriger.

- **D-B2 — La fiche produit, elle, ne signale RIEN.** `markInvalid` n'est **pas** activé sur le sélecteur du catalogue. Un compte devenu inutilisable ne se découvre pas en éditant un article — il se découvre **au moment où l'on s'en sert**. C'est le corollaire de **D4** de 16-2a (la validation ne se déclenche que si le compte change) : la fiche accepte une valeur devenue invalide sans broncher, et le signal est rendu une fois, au bon endroit.

- **D-B3 — `addFreeLine` reste à `null`.** Une ligne libre ne vient d'aucun article. Son commentaire d'ancrage est corrigé lui aussi, faute de quoi il annoncerait un travail déjà fait.

---

## Acceptance Criteria

- **AC-B1 — Sélecteur sur la fiche produit.** Le formulaire de `frontend/src/routes/(app)/products/+page.svelte` porte un sélecteur de compte avec `requiredAccountType="Revenue"` et `allowClear`, hydraté depuis la réponse serveur à l'édition, et **toujours** envoyé à la soumission (valeur ou `null`), jamais omis — cf. **D5** de 16-2a. Le champ est **facultatif**.

  Il porte un `<label for>` et sa clé i18n : **les quatre champs existants en portent un** (`+page.svelte:586`, `:598`, `:609`, `:622`), ce serait le seul sans.

  ⚠️ **`openCreate()` DOIT remettre le compte à `null` — sinon un article en hérite d'un autre, en silence.** Les champs du formulaire sont des `$state` déclarés **au niveau de la page** (`:94-98`), pas dans le dialogue : ils **survivent** à sa fermeture. C'est pourquoi `openCreate()` (`:249-258`) remet **explicitement** à zéro les quatre existants. Un cinquième champ ajouté sans son reset produit ceci : éditer un article portant le compte 3200, fermer, cliquer « Nouveau produit » — et le sélecteur affiche **3200**. **Et rien ne le signale**, puisque **D-B2** interdit tout marqueur sur cette fiche. L'article créé part avec un compte hérité d'un autre, ce qui se propage à **toutes ses factures futures**.

  Le sélecteur **n'active pas `markInvalid`** (**D-B2**) — son défaut est `false`, la décision se tient donc par absence de code, et c'est écrit ici pour qu'une régression qui l'activerait soit visible en revue.

- **AC-B2 — Le flag `includeArchived` est obligatoire, ET prouvé.** Les comptes sont chargés par **`fetchAccounts(true)`** : sans le flag, un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'afficher son libellé.

  ⚠️ **Le décrire ne le prouve pas.** 16-1b a **mesuré** que l'AC qui se présentait comme le garde-fou de ce piège n'en était pas un : le mock rend la liste complète quel que soit l'argument, donc un test fonctionnel reste vert sous la mutation. **Seule** une assertion `expect(fetchAccounts).toHaveBeenCalledWith(true)` l'attrape. Le piège s'est **rejoué** en revue de code de 16-1b, fermé alors par un E2E archivant un compte après coup.

- **AC-B3 — Pré-remplissage.** `onProductSelect` (`InvoiceForm.svelte:420-430`) pose `revenueAccountId: p.defaultRevenueAccountId ?? null`. Le site `addFreeLine` (`:407-418`) **reste à `null`** (D-B3).

  Les **deux** commentaires d'ancrage sont remplacés par la description du comportement livré. ⚠️ Les chercher **par leurs numéros de ligne** (`:413-414` et `:426`) : un `grep -F "16-2 y branchera"` ne retrouve que **celui de `:426`**, la phrase d'`addFreeLine` étant scindée sur deux lignes.

- **AC-B4 — Un compte inutilisable est signalé au niveau de la ligne** (D-B1). Une facture montée depuis un article dont le compte a été archivé affiche le libellé, le marqueur d'invalidité, et refuse l'enregistrement en nommant la ligne. **Aucun code nouveau n'est attendu** : l'AC vérifie que le pré-remplissage passe par la machinerie de 16-1, et **le test qui le prouve est le livrable**.

- **AC-B5 — Types et client API.** `ProductResponse`, `CreateProductRequest`, `UpdateProductRequest` (`frontend/src/lib/features/products/products.types.ts`) portent `defaultRevenueAccountId`. Le doc-comment de `ProductPicker.svelte:1-4`, qui énumère le snapshot recopié (« name/unitPrice/vatRate »), est mis à jour — il devient faux dès que le compte s'y ajoute.

- **AC-B6 — i18n.** Les libellés **énumérés ici** existent dans les **4** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`) — un critère portant sur un ensemble non énuméré n'est ni cochable ni réfutable :
  1. l'**étiquette** du sélecteur de la fiche produit ;
  2. son **texte d'aide** disant que laisser vide fait suivre le compte par défaut de la société.

  Préfixe `product-` : la page catalogue vit dans `src/routes/(app)/products/`, **hors du périmètre** de `lint-i18n-ownership` (qui ne parcourt que `src/lib/features`, `:17`). ⚠️ Tout libellé qui finirait consommé depuis un composant partagé sous `features/` doit porter l'un des **six** préfixes globaux (`error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`) — le passer **en prop** depuis la page évite la question. C'est l'erreur exacte de 16-1b, où une clé a dû être renommée après un lint rouge.

- **AC-B7 — Documentation synchronisée.**
  - **CHANGELOG** : la fonctionnalité **et** l'avertissement aux clients d'API de **D5** de 16-2a (le `PUT` full-replace efface un `defaultRevenueAccountId` omis). Un avertissement analogue existe pour les factures — s'y accorder, ne pas le dupliquer.
  - **Manuel utilisateur** — ⚠️ **il promet DÉJÀ ce champ.** `docs/manual/fr/user-manual.tex:553` documente « **Compte produit par défaut** : par exemple 3000 » **depuis la Story 11-0**, pour un code où il n'existe pas ; et `:583` décrit la chaîne de repli à **deux** maillons (ligne → société), que **AC-B3** fait passer à **trois** (ligne → **article** → société). Le manuel est donc à **corriger**, pas à compléter : dire que le champ est **facultatif**, décrire le pré-remplissage, dire ce que signifie le laisser vide. **`make fr` puis commiter les PDF.** *(Obligation reprise de 16-1b AC17.)*
  - **README** : vérifier la feuille de route — l'Epic 16 reste « 🚧 En cours » tant que #144 et #151 ne sont pas tous deux livrés — et la section « Fonctionnalités ». Tracer la vérification même si la conclusion est « rien à changer ».

- **AC-B8 — Discrimination prouvée par mutation.** **Quatre** mutations exécutées et consignées, avec **le rayon d'effet réellement attendu** — pas une liste minorée :
  1. `onProductSelect` repose `revenueAccountId: null` → **les tests du pré-remplissage** rougissent, l'unitaire **et** l'E2E — **plus le test d'AC-B4**, dont le prérequis est justement que la ligne porte le compte de l'article. ⚠️ **Trois tests, pas deux** : sous cette mutation la ligne vaut `null`, état **valide** qui suit le défaut société, donc aucun marqueur n'apparaît et rien n'est refusé. Annoncer deux tests ferait conclure à un montage cassé au troisième rouge.
  2. `fetchAccounts(true)` → `fetchAccounts()` **sur la fiche produit** → l'assertion `toHaveBeenCalledWith(true)` **et l'E2E « rouvrir la fiche produit après archivage »** (T-B4, scénario 3) rougissent. ⚠️ **Sans ce troisième scénario E2E, aucun test de bout en bout n'attrape cette mutation** : les deux autres passent par le formulaire de facture, dont le `fetchAccounts(true)` (`InvoiceForm.svelte:192`) est **un appel distinct**, déjà en place et non touché par cette story. Muter celui du catalogue ne peut rien y changer.
  3. **`openCreate()` ne remet pas le compte à `null`** → le test « éditer un article avec compte, puis Nouveau produit » rougit. Sans lui, l'héritage silencieux décrit en AC-B1 n'est discriminé par rien.
  4. le champ est retiré du **payload HTTP** envoyé par la fiche produit → seul un test qui traverse réellement HTTP l'attrape. *(Mutation la plus instructive de 16-1b : ni Vitest ni les tests Rust ne voient une clé qui disparaît entre les deux.)*

  **Un test attendu qui ne rougit pas invalide le montage.**

- **AC-B9 — Gate.** Frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) **et** E2E Playwright (`npm run test:e2e` ; **pré-requis** : MariaDB démarré, seed CI appliqué, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` sur Ubuntu 26.04+ — sans lui l'installation échoue en donnant l'impression que l'E2E n'est pas exécutable) **et** `cargo test --workspace`, les locales vivant dans un crate Rust. État final, exit 0, non présumé d'un run antérieur.

---

## Tasks / Subtasks

- [ ] **T-B1 — Types et client API** (AC-B5)
  - [ ] Champ dans les trois interfaces de `products.types.ts`.
  - [ ] Mettre à jour le doc-comment de `ProductPicker.svelte:1-4`. *(Site trouvé par le grep de propagation, pas par les lentilles.)*
- [ ] **T-B2 — Fiche produit** (AC-B1, AC-B2, AC-B6)
  - [ ] Sélecteur, `<label for>` + clé i18n, hydratation à l'édition, envoi systématique. **Pas de `markInvalid`** (D-B2).
  - [ ] **`openCreate()` (`:249-258`) remet le compte à `null`**, comme il le fait déjà pour les quatre autres champs — les `$state` vivent au niveau de la page (`:94-98`) et survivent à la fermeture du dialogue.
  - [ ] `fetchAccounts(true)` — **et son assertion** `toHaveBeenCalledWith(true)`.
  - [ ] Clés i18n dans les **4** locales.
- [ ] **T-B3 — Pré-remplissage** (AC-B3, AC-B4)
  - [ ] `onProductSelect` pose le compte de l'article ; `addFreeLine` reste à `null`.
  - [ ] Remplacer les **deux** commentaires d'ancrage, repérés par leurs numéros de ligne.
- [ ] **T-B4 — Tests et preuve** (AC-B4, AC-B8)
  - [ ] Unitaire : le pré-remplissage pose bien le compte de l'article ; l'assertion `toHaveBeenCalledWith(true)` ; **« éditer un article avec compte → Nouveau produit → le sélecteur est vide »** (mutation 3).
  - [ ] **E2E** (suffixe `.spec.ts` **obligatoire**), **TROIS** scénarios :
    1. « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte » ;
    2. cas **AC-B4** — assigner un compte, l'**archiver ensuite**, monter la facture, constater marqueur et refus ;
    3. **« assigner un compte → l'archiver → ROUVRIR la fiche produit en édition → le libellé du compte s'affiche encore »**. ⚠️ **Sans ce scénario, la mutation 2 n'est attrapée par aucun E2E** : les deux premiers passent par le formulaire de facture, dont le `fetchAccounts(true)` est un appel **distinct** de celui du catalogue. Patron à reprendre : `frontend/tests/e2e/invoice-revenue-account.spec.ts:197-207` — archiver par l'API, puis `page.goto` vers l'écran, puis asserter le libellé. C'est lui qui a fermé ce piège en 16-1b.
  - [ ] ⚠️ Le preset E2E `with-data` crée **déjà** un produit `'CI Product'` au compte `NULL` (`test_fixtures.rs:482-483`, appelé par `routes/test_endpoints.rs:203`) — en tenir compte dans les montages.
  - [ ] Les **quatre** mutations d'AC-B8, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T-B5 — Documentation** (AC-B7)
  - [ ] CHANGELOG (fonctionnalité + avertissement `PUT`).
  - [ ] `user-manual.tex` : corriger `:553` (champ **facultatif**) et `:583` (repli à trois maillons), décrire le pré-remplissage. **`make fr`**, commiter les PDF.
  - [ ] **Greper le SYMPTÔME, pas seulement les deux sites nommés** (§ *Propagation post-patch*) : `grep -n "par défaut de la société\|compte par défaut" docs/manual/fr/*.tex` rend **trois** sites — `:583`, mais aussi **`:603`** (« Variante ventilée ») et **`:611`** (mention « (défaut) » sur la facture validée). Les lire et **tracer le verdict** dans le Dev Agent Record, même si la conclusion est « inchangé » — c'est ce qu'AC-B7 exige déjà du README, et qui manquait ici.
  - [ ] README — feuille de route et « Fonctionnalités » vérifiées.
- [ ] **T-B6 — Gate** (AC-B9) — frontend + E2E + `cargo test --workspace`, état final, exit 0.

---

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas toucher au backend** — colonne, DTO, validation, export CSV et migration sont en **16-2a**.
- **Ne pas activer `markInvalid` sur la fiche produit** (D-B2). Le signal est rendu au niveau de la ligne, une fois.
- **Ne pas dupliquer** `isAccountUnusable` ni les helpers de libellé de 16-1b : ils sont la source unique de vérité de leur verdict.
- **Ne pas modifier `AccountAutocomplete`** autrement que par une **prop opt-in** dont le défaut préserve le comportement — il est partagé par **cinq** écrans (`InvoiceForm`, `JournalEntryForm`, `VatPurchaseAssistant`, `ManualMatchModal`, `TransactionSplitModal`).

### References

- Story **16-2a** — colonne, API, validation. **Même PR obligatoire.**
- Story parente **16-2** — archivée, 4 passes de `validate`.
- Story **16-1b** — `AccountAutocomplete` et ses props opt-in, `account-label.ts`, `account-validity.ts`, l'arbitrage *afficher + signaler + bloquer*, et la mesure du piège `fetchAccounts(true)`.
- `CLAUDE.md` § « Test Locally First » (frontend, E2E) et § « Synchroniser TOUTES les docs ».

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

**2026-08-03 — Story née du split de 16-2**, arbitré par Guy après deux déclenchements du garde-fou de la dérogation (passes 3 et 4 de `validate`). Le contenu repris est dans son **état corrigé de la passe 4** : il incorpore notamment la preuve exigée pour `fetchAccounts(true)` (que la parente décrivait sans la prouver), la correction du motif de recherche des commentaires d'ancrage (un `grep -F` n'en retrouve qu'un des deux), le manuel utilisateur passé de puce conditionnelle à obligation, et le fait — réfuté en passe 4 — que le preset E2E `with-data` crée déjà un produit.

**2026-08-04 — Passe 1 de `validate`** (Sonnet, trois lentilles indépendantes en sous-agents, contexte frais chacune). **2 HIGH, 2 MEDIUM, 2 LOW**, tous remédiés. **Aucune ancre du document n'a été prise en défaut** — les trois lentilles ont vérifié une par une les cinq sites `LineState`, les quatre `<label for>`, les six préfixes du lint i18n, les cinq écrans consommant `AccountAutocomplete`, les deux ancres du manuel, et jusqu'à l'affirmation méta selon laquelle `grep -F "16-2 y branchera"` ne retrouve qu'un des deux commentaires. Tout tient. **Les défauts sont ailleurs : dans la preuve.**

**HIGH-1 — une mutation dont aucun E2E ne pouvait rougir** *(convergence de DEUX lentilles indépendantes)*. La mutation 2 annonçait que « l'E2E à compte archivé » tomberait avec elle. Or les deux scénarios E2E prescrits passent tous deux par le **formulaire de facture**, dont le `fetchAccounts(true)` (`InvoiceForm.svelte:192`) est un appel **distinct**, déjà en place et non touché par cette story. Muter celui du **catalogue** ne pouvait rien y changer : seule l'assertion unitaire serait tombée. Le dev, appliquant la règle « un test attendu qui ne rougit pas invalide le montage », aurait dû inventer en cours de route un test que le document ne bornait pas. Un **troisième scénario E2E** est désormais prescrit — archiver puis **rouvrir la fiche produit** —, sur le patron qui a fermé ce même piège en 16-1b (`invoice-revenue-account.spec.ts:197-207`).

**HIGH-2 — un article pouvait hériter du compte d'un autre, en silence.** Les champs du formulaire sont des `$state` déclarés **au niveau de la page** (`+page.svelte:94-98`), pas dans le dialogue : ils survivent à sa fermeture, et c'est pourquoi `openCreate()` (`:249-258`) remet **explicitement** à zéro les quatre existants. Un cinquième champ ajouté sans son reset, et l'on édite un article au compte 3200, on ferme, on clique « Nouveau produit » — le sélecteur affiche 3200. **Rien ne le signale**, puisque D-B2 interdit tout marqueur sur cette fiche, et l'article part avec un compte hérité qui se propagera à toutes ses factures. Reset prescrit, avec sa mutation dédiée.

**Les deux MEDIUM portent sur la même faiblesse que la passe 2 de 16-2a avait trouvée chez sa sœur** : une liste de tests visés **minorée**. La mutation 1 tue **trois** tests et non deux — le test d'AC-B4 en dépend, son prérequis étant justement que la ligne porte le compte de l'article — et annoncer deux rouges ferait conclure à un montage cassé au troisième. Second MEDIUM : **D-B2 n'était portée par aucun critère**, seulement par un interdit en Dev Notes ; elle est désormais écrite dans AC-B1, pour qu'une régression qui activerait `markInvalid` soit visible.

Les LOW : ancre `ProductPicker.svelte:59` → **`:59-60`** (la ligne citée est la signature, l'appel est à la suivante) ; et le **grep du symptôme** sur le manuel, qui rend **trois** sites (`:583`, `:603`, `:611`) là où la story n'en nommait que deux — les deux autres sont à lire et leur verdict à tracer, comme AC-B7 l'exige déjà du README.

**Statut : `ready-for-dev`. Boucle NON convergée** — 2 HIGH imposent une **passe 2** (§ *Review Iteration Rule*), sur un LLM autre que Sonnet et en contexte frais.
