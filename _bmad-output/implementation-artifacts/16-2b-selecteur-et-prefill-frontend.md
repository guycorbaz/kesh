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

*(« site 3/5 » renvoie aux **cinq** constructions de `LineState` d'`InvoiceForm.svelte`, numérotées par 16-1b sous `AC9-bis — site N/5`. Seuls les sites **2** — `addFreeLine` — et **3** — `onProductSelect` — concernent cette story. Les trois autres sont hors de ce geste, mais pas pour la même raison : les sites **1** (`:118`) et **4** (`:610`) **recopient** une valeur déjà décidée, tandis que le site **5** (`:133`) ouvre une facture **neuve** et pose `null` à neuf — il ne recopie rien.)*

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

  ⚠️ **Le `<label for>` exige une prop `id` qui N'EXISTE PAS — l'ajouter fait partie du travail.** `AccountAutocomplete` expose onze props (`accounts`, `value`, `loadError`, `disabled`, `onSelect`, `allowClear`, `markInvalid`, `requiredAccountType`, `postableExemptAccountId`, `placeholder`, `ariaLabel`) et **aucune `id`** ; son `<Input>` (`:272-289`) est appelé sans spread, rien ne peut passer au travers. Un `<label for="…">` écrit dans la page ne se lierait donc **à rien** — et le champ n'aurait d'autre nom accessible que le repli d'`AccountAutocomplete`, `i18nMsg('journal-entry-form-col-account', 'Compte')` : sur une fiche produit, un libellé venant d'une clé **`journal-entry-*`**.

  **Tranché : ajouter une prop `id?: string` opt-in** — défaut `undefined`, comportement des **cinq** écrans consommateurs strictement inchangé, exactement ce que les Dev Notes autorisent. ⚠️ **Ne pas se contenter de recopier le patron de `VatPurchaseAssistant.svelte:156-158`** : son `for="{uid}-charge"` ne désigne aucun élément du DOM, et `svelte-check` ne le voit pas — la règle `a11y_label_has_associated_control` se satisfait de la présence de l'attribut, elle ne résout pas les `id` à travers un composant. Le copier livrerait un label orphelin sous un gate vert.

  **Le texte d'aide, lui, est rendu PAR LA PAGE** — un `<p class="text-xs">` sous le champ. Ni prop, ni `placeholder` : un placeholder disparaît à la saisie, ce n'est pas un texte d'aide.

  ⚠️ **`openCreate()` DOIT remettre le compte à `null` — sinon un article en hérite d'un autre, en silence.** Les champs du formulaire sont des `$state` déclarés **au niveau de la page** (`:94-98`), pas dans le dialogue : ils **survivent** à sa fermeture. C'est pourquoi `openCreate()` (`:249-258`) remet **explicitement** à zéro les quatre existants. Un cinquième champ ajouté sans son reset produit ceci : éditer un article portant le compte 3200, fermer, cliquer « Nouveau produit » — et le sélecteur affiche **3200**. **Et rien ne le signale**, puisque **D-B2** interdit tout marqueur sur cette fiche. L'article créé part avec un compte hérité d'un autre, ce qui se propage à **toutes ses factures futures**.

  Le sélecteur **n'active pas `markInvalid`** (**D-B2**) — son défaut est `false`, la décision se tient donc par absence de code, et c'est écrit ici pour qu'une régression qui l'activerait soit visible en revue.

- **AC-B2 — Le flag `includeArchived` est obligatoire, ET prouvé.** Les comptes sont chargés par **`fetchAccounts(true)`** : sans le flag, un compte archivé n'est plus résoluble et le champ **paraît vide** au lieu d'afficher son libellé.

  ⚠️ **Le décrire ne le prouve pas.** 16-1b a **mesuré** que l'AC qui se présentait comme le garde-fou de ce piège n'en était pas un : le mock rend la liste complète quel que soit l'argument, donc un test fonctionnel reste vert sous la mutation. **Seule** une assertion `expect(fetchAccounts).toHaveBeenCalledWith(true)` l'attrape. Le piège s'est **rejoué** en revue de code de 16-1b, fermé alors par un E2E archivant un compte après coup.

- **AC-B3 — Pré-remplissage.** `onProductSelect` (`InvoiceForm.svelte:420-430`) pose `revenueAccountId: p.defaultRevenueAccountId ?? null`. Le site `addFreeLine` (`:407-418`) **reste à `null`** (D-B3).

  Les **deux** commentaires d'ancrage sont remplacés par la description du comportement livré. ⚠️ Les chercher **par leurs numéros de ligne** (`:413-414` et `:426`) : un `grep -F "16-2 y branchera"` ne retrouve que **celui de `:426`**, la phrase d'`addFreeLine` étant scindée sur deux lignes.

- **AC-B4 — Un compte inutilisable est signalé au niveau de la ligne** (D-B1). Une facture montée depuis un article dont le compte a été archivé affiche le libellé, le marqueur d'invalidité, et refuse l'enregistrement en nommant la ligne. **Aucun code nouveau n'est attendu** : l'AC vérifie que le pré-remplissage passe par la machinerie de 16-1, et **le test qui le prouve est le livrable**.

- **AC-B5 — Types et client API.** `ProductResponse`, `CreateProductRequest`, `UpdateProductRequest` (`frontend/src/lib/features/products/products.types.ts`) portent `defaultRevenueAccountId`.

  ⚠️ **Non optionnel — `number | null`, SANS le `?`**, contrairement à `description?: string | null` (`:32`, `:40`). L'obligation « toujours envoyé, jamais omis » d'AC-B1 est ainsi portée par **le compilateur**, et non par la forme actuelle du code. Aujourd'hui un **littéral unique** sert les deux verbes (`+page.svelte:323-330` → `updateProduct` ou `createProduct`), donc un oubli disparaîtrait des deux payloads et le scénario E2E n°1 l'attraperait ; mais si les deux payloads divergent un jour, l'effacement silencieux que **D5** de 16-2a ne documente que pour les clients d'API tiers se rouvrirait **sur l'interface de Kesh elle-même** — et **D-B2** garantit que rien ne le signalerait. Le doc-comment de `ProductPicker.svelte:1-4`, qui énumère le snapshot recopié (« name/unitPrice/vatRate »), est mis à jour — il devient faux dès que le compte s'y ajoute.

- **AC-B6 — i18n.** Les libellés **énumérés ici** existent dans les **4** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`) — un critère portant sur un ensemble non énuméré n'est ni cochable ni réfutable :
  1. l'**étiquette** du sélecteur de la fiche produit ;
  2. son **texte d'aide** disant que laisser vide fait suivre le compte par défaut de la société.

  **Les deux clés sont nommées ici**, faute de quoi l'AC n'est pas cochable : **`product-form-revenue-account`** (étiquette) et **`product-form-revenue-account-help`** (texte d'aide).

  ⚠️ **Préfixe `product-form-`, pas `product-revenue-account-`.** Les six clés voisines de cette même fiche le portent (`product-form-name`, `-description`, `-price`, `-vat-rate`, `-create-title`, `-edit-title`) : ce serait les seules hors famille. Et surtout, **`product-revenue-account-*` est la famille que 16-2a D10 revendique** pour le message de rejet backend — mêmes quatre `messages.ftl`, **même PR**. Deux moitiés de la même PR écrivant la même famille sans se citer, c'est une collision qui n'attend qu'un nom identique.

  Préfixe `product-` : la page catalogue vit dans `src/routes/(app)/products/`, **hors du périmètre** de `lint-i18n-ownership` (qui ne parcourt que `src/lib/features`, `:17`). ⚠️ **Mais le piège est concret, pas théorique** : `AccountAutocomplete` vit sous **`src/lib/features/journal-entries/`** — donc **DANS** le périmètre du lint. Une clé `product-*` qu'il résoudrait lui-même ferait rougir `npm run lint-i18n-ownership`, puisque `product` n'est pas l'un des **six** préfixes globaux (`error`, `tooltip`, `common`, `mode`, `shortcut`, `demo`).

  **D'où l'exigence réelle : les deux libellés sont RÉSOLUS DANS `+page.svelte`** — l'étiquette dans son `<label>`, le texte d'aide dans un `<p>` sous le champ. **Aucune clé `product-*` ne doit être lue depuis un composant de `features/`.** *(La passe 2 écrivait « passés en prop » : formulation inapplicable, aucune prop d'`AccountAutocomplete` n'accueille un texte d'aide — cf. AC-B1.)* C'est l'erreur exacte de 16-1b, où une clé a dû être renommée après un lint rouge.

- **AC-B7 — Documentation synchronisée.**
  - **CHANGELOG** : la fonctionnalité **et** l'avertissement aux clients d'API de **D5** de 16-2a (le `PUT` full-replace efface un `defaultRevenueAccountId` omis). Un avertissement analogue existe pour les factures — s'y accorder, ne pas le dupliquer.
  - **Manuel utilisateur** — ⚠️ **il promet DÉJÀ ce champ.** `docs/manual/fr/user-manual.tex:553` documente « **Compte produit par défaut** : par exemple 3000 » **depuis la Story 11-0**, pour un code où il n'existe pas ; et **deux** sites décrivent la chaîne de repli à **deux** maillons (ligne → société), que **AC-B3** fait passer à **trois** (ligne → **article** → société) : `:583` et **`:603`** — ce dernier affirmant que « les lignes laissées vides continuent de suivre le compte par défaut de la société ». Le manuel est donc à **corriger**, pas à compléter : dire que le champ est **facultatif**, décrire le pré-remplissage, dire ce que signifie le laisser vide. **`make fr` puis commiter les PDF.** *(Obligation reprise de 16-1b AC17.)*
  - **README** : vérifier la feuille de route — l'Epic 16 reste « 🚧 En cours » tant que #144 et #151 ne sont pas tous deux livrés — et la section « Fonctionnalités ». Tracer la vérification même si la conclusion est « rien à changer ».

- **AC-B8 — Discrimination prouvée par mutation.** **Quatre** mutations exécutées et consignées, avec **le rayon d'effet réellement attendu** — pas une liste minorée :
  1. `onProductSelect` repose `revenueAccountId: null` → **les tests du pré-remplissage** rougissent, l'unitaire **et** l'E2E — **plus le test d'AC-B4**, dont le prérequis est justement que la ligne porte le compte de l'article. ⚠️ **Trois tests, pas deux** : sous cette mutation la ligne vaut `null`, état **valide** qui suit le défaut société, donc aucun marqueur n'apparaît et rien n'est refusé. Annoncer deux tests ferait conclure à un montage cassé au troisième rouge.
  2. `fetchAccounts(true)` → `fetchAccounts()` **sur la fiche produit** → l'assertion `toHaveBeenCalledWith(true)` **et l'E2E « rouvrir la fiche produit après archivage »** (T-B4, scénario 3) rougissent. ⚠️ **Sans ce troisième scénario E2E, aucun test de bout en bout n'attrape cette mutation** : les deux autres passent par le formulaire de facture, dont le `fetchAccounts(true)` (`InvoiceForm.svelte:192`) est **un appel distinct**, déjà en place et non touché par cette story. Muter celui du catalogue ne peut rien y changer.
  3. **`openCreate()` ne remet pas le compte à `null`** → le test « éditer un article avec compte, puis Nouveau produit » rougit. Sans lui, l'héritage silencieux décrit en AC-B1 n'est discriminé par rien.
  4. le champ est retiré du **payload HTTP** envoyé par la fiche produit → **le scénario E2E n°1 rougit, et lui seul**. ⚠️ Ce « lui seul » n'est vrai que parce que les scénarios 2 et 3 assignent leur compte **par l'API** (T-B4) : assignés par l'interface, ils passeraient par le **même** littéral de payload (`+page.svelte:323-330`, qui sert création **et** modification) et rougiraient aussi — **trois** rouges pour une mutation qui en annonce un, et le dev conclurait à un montage qui fuit. *(Mutation la plus instructive de 16-1b : ni Vitest ni les tests Rust ne voient une clé qui disparaît entre les deux.)*

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
  - [ ] **Prop `id?: string` opt-in** sur `AccountAutocomplete`, rendue sur son `<Input>` — défaut `undefined`, les cinq écrans inchangés (AC-B1).
  - [ ] Clés **`product-form-revenue-account`** et **`product-form-revenue-account-help`** dans les **4** locales, **résolues dans `+page.svelte`** (étiquette dans le `<label for>`, aide dans un `<p>` sous le champ) — jamais lues depuis `AccountAutocomplete`, qui vit sous `features/` et ferait rougir le lint (AC-B6).
  - [ ] Passer **`loadError`**, comme le font `InvoiceForm.svelte:823` et `VatPurchaseAssistant.svelte:162`. Sans lui, un `fetchAccounts` en échec laisse le champ **vide et muet** — le symptôme même qu'AC-B2 prévient, par une autre cause, et que **D-B2** garantit de ne pas signaler.
- [ ] **T-B3 — Pré-remplissage** (AC-B3, AC-B4)
  - [ ] `onProductSelect` pose le compte de l'article ; `addFreeLine` reste à `null`.
  - [ ] Remplacer les **deux** commentaires d'ancrage, repérés par leurs numéros de ligne.
- [ ] **T-B4 — Tests et preuve** (AC-B4, AC-B8)
  - [ ] Unitaire : le pré-remplissage pose bien le compte de l'article ; l'assertion `toHaveBeenCalledWith(true)`.
  - [ ] **« éditer un article avec compte → Nouveau produit → le sélecteur est vide »** (mutation 3) — **en E2E, pas en unitaire** : le scénario enchaîne deux ouvertures du dialogue sur une page SvelteKit complète, ce qu'un test de composant isolé ne monte pas.
  - [ ] **E2E** (suffixe `.spec.ts` **obligatoire**) — **quatre exécutions Playwright au total** : les trois scénarios numérotés ci-dessous, **plus** le test de reset d'`openCreate()` de la puce précédente. La numérotation « scénario N » à laquelle renvoie AC-B8 désigne cette liste-ci.
    1. « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte ». ⚠️ **Le produit doit être créé PAR L'INTERFACE**, pas semé par fixture : c'est ce scénario, et lui seul, qui exerce le **payload HTTP** de la fiche et discrimine donc la **mutation 4**. Un produit semé directement en base contournerait le payload, et la mutation passerait sans qu'aucun test ne bouge ;
    2. cas **AC-B4** — assigner un compte **par l'API** (pas par l'interface), l'**archiver ensuite**, monter la facture, constater marqueur et refus ;
    3. **« assigner un compte PAR L'API → l'archiver → ROUVRIR la fiche produit en édition → le libellé du compte s'affiche encore »**. ⚠️ **Sans ce scénario, la mutation 2 n'est attrapée par aucun E2E** : les deux premiers passent par le formulaire de facture, dont le `fetchAccounts(true)` est un appel **distinct** de celui du catalogue. Patron à reprendre : `frontend/tests/e2e/invoice-revenue-account.spec.ts:197-207` — archiver par l'API, puis `page.goto` vers l'écran, puis asserter le libellé. C'est lui qui a fermé ce piège en 16-1b.
  - [ ] ⚠️ Le preset E2E `with-data` crée **déjà** un produit `'CI Product'` au compte `NULL` (`test_fixtures.rs:482-483`, appelé par `routes/test_endpoints.rs:203`) — en tenir compte dans les montages.
  - [ ] Les **quatre** mutations d'AC-B8, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [ ] **T-B5 — Documentation** (AC-B7)
  - [ ] CHANGELOG (fonctionnalité + avertissement `PUT`).
  - [ ] `user-manual.tex` : corriger `:553` (champ **facultatif**) et `:583` (repli à trois maillons), décrire le pré-remplissage. **`make fr`**, commiter les PDF.
  - [ ] **`:603` est à CORRIGER lui aussi, pas seulement à relire.** Le grep du symptôme — **`grep -on "par défaut de la société" docs/manual/fr/user-manual.tex`** — rend **trois** sites : `:583`, **`:603`**, `:611`. ⚠️ **Ne pas restreindre le motif à « compte par défaut de la société »** : il n'en rend que **deux** et rate précisément `:583`, rédigé « compte **de produit** par défaut de la société » — l'un des deux sites à corriger. *(Motif corrigé en passe 3 : c'est le mode d'échec que la § « Propagation post-patch » et le garde-fou P7 documentent déjà deux fois — un motif sous-inclusif rend le garde-fou muet.)* Or `:603` (« Variante ventilée ») affirme que **« les lignes laissées vides continuent de suivre le compte par défaut de la société »** — phrase qui devient **fausse** avec cette story : une ligne montée depuis un article suit d'abord le compte de l'**article**. `:611` en revanche reste **exact** (la mention « (défaut) » désigne toujours une ligne à `revenueAccountId = null`) : le lire, et **tracer le verdict « inchangé »** au Dev Agent Record — c'est ce qu'AC-B7 exige déjà du README.

    *(La passe 1 avait rangé `:603` parmi les sites « à tracer ». C'était insuffisant : greper le symptôme ne suffit pas si l'on ne lit pas ce que le site dit vraiment.)*
  - [ ] README — feuille de route et « Fonctionnalités » vérifiées.
- [ ] **T-B6 — Gate** (AC-B9) — frontend + E2E + `cargo test --workspace`, état final, exit 0.

---

## Dev Notes

### Dérogation à la règle de splitting préventif

**Le second critère de la § *Règle de splitting préventif* est formellement déclenché** : entre la passe 2 et la passe 3 de `validate`, la sévérité maximale reste `MEDIUM` et le compteur stagne à **3**. **Dérogation accordée par Guy le 2026-08-04**, boucle poursuivie en passe 4 sans split.

**Justification.** Le critère vise la *non-convergence réelle* — le signe qu'une story est trop large pour tenir dans un mental-model adversarial fiable. Ce n'est pas ce qui se passe ici :

- **La conception n'a jamais été prise en défaut.** Sur les trois passes, aucun finding n'a porté sur les décisions `D-B1`–`D-B3`, ni sur le cadrage, ni sur la frontière avec 16-2a — vérifiée nette dans les deux sens par deux lentilles indépendantes.
- **Les trois MEDIUM de la passe 3 sont des défauts de REMÉDIATION**, tous introduits par les patches des passes 1 et 2 : une prescription inapplicable (« passés en prop »), un motif de grep sous-inclusif, et un rayon de mutation mal borné. Splitter ne les aurait pas évités — ils naissent du geste de correction, pas de la taille du document.
- **Le document reste petit** : ~200 lignes, 3 décisions, 9 critères, 6 tâches. Le précédent qui a motivé la règle (7-1) étalait 7 modules ; celui qui a motivé son amendement (14-2, 14-4) a convergé en 5 passes sans split.

**Risque accepté.** Le mode d'échec mesuré sur la Story 16-1a — *la remédiation devient la première source des findings suivants*, 12 sur 28 aux passes 3-7 — est **exactement** ce qui s'observe ici. La passe 4 peut donc produire de nouveaux MEDIUM portant sur les patches de la passe 3. **Critère d'arrêt maintenu** : plafond de 8 passes de la § *Review Iteration Rule*, et arbitrage à reprendre si la passe 4 stagne à son tour.

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

**2026-08-04 — Passe 2 de `validate`** (Haiku 4.5, deux lentilles, contexte frais, **diff aplati** conformément à la § *Haiku-specific guardrails*). **0 CRITICAL, 0 HIGH, 3 MEDIUM, 1 LOW** — après réfutation. Une lentille a rendu **0 finding** avec son énumération de contrôles.

⚠️ **Le CRITICAL et le HIGH annoncés ont tous deux été RÉFUTÉS comme tels — et ont tous deux conduit à un défaut réel.** C'est le cas de figure que la § *Haiku-specific guardrails* existe pour traiter, avec une nuance qui mérite d'être notée : **la vérification ne s'arrête pas à écarter le finding, elle regarde ce qu'il visait.**

- **Le « CRITICAL » (clés i18n non nommées)** s'appuyait sur un `grep` montrant qu'une clé n'existe pas encore — ce qui est vrai de *toute* clé d'une story non implémentée, donc ne prouve rien —, et il se contredisait avec son propre MEDIUM (l'un disait « risque de lint rouge », l'autre « le lint ne verra jamais cette clé »). **Mais la vérification a établi un fait que ni lui ni la passe 1 n'avaient : `AccountAutocomplete` vit sous `src/lib/features/journal-entries/`, donc DANS le périmètre du lint.** Le piège d'AC-B6 était donc concret, et la story se contentait de le *suggérer* (« le passer en prop évite la question »). C'est désormais une **obligation**, et les deux clés sont **nommées**.
- **Le « HIGH » (mutation 4 sans tueur)** est faux : le scénario E2E n°1 traverse HTTP de bout en bout. **Mais il désignait une ambiguïté réelle** — « fiche produit avec compte » ne disait pas si le produit est créé *par l'interface* ou semé par fixture. Semé, le payload n'est jamais exercé et la mutation 4 passe sans qu'aucun test ne bouge. Le scénario 1 précise maintenant **« créé par l'interface »**.

**Le MEDIUM exact, et il corrige la passe 1 :** `user-manual.tex:603` affirme que « **les lignes laissées vides continuent de suivre le compte par défaut de la société** » — phrase qui devient **fausse** avec cette story. La passe 1 avait rangé ce site parmi ceux « à lire et tracer ». C'était insuffisant : **greper le symptôme ne suffit pas si l'on ne lit pas ce que le site dit vraiment.** `:603` passe en correction obligatoire ; `:611`, vérifié, reste exact et son verdict « inchangé » est à tracer.

Dernier point : le test de la mutation 3 est requalifié en **E2E** — il enchaîne deux ouvertures du dialogue sur une page SvelteKit complète, ce qu'un test de composant isolé ne monte pas.

**Trend : `2H/2M/2L` → `0H/3M/1L`.** Sévérité décroissante, garde-fou de splitting non déclenché. **Boucle NON convergée** — 3 MEDIUM imposent une **passe 3**, sur le seul modèle pas encore employé ici : **Opus**, contexte frais.

---

**2026-08-04 — Passe 3 de `validate`** (Opus 5, deux lentilles, contexte frais). **0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW** après déduplication — les deux lentilles **convergent** sur trois points. **Les trois MEDIUM visent tous des corrections écrites par les passes 1 et 2.** La story elle-même n'a pas été prise en défaut ; sa remédiation, si.

**MEDIUM-1 — « passés en prop » était INAPPLICABLE.** La passe 2 avait érigé en obligation que les deux libellés soient « passés en prop » au sélecteur. Or `AccountAutocomplete` expose **onze** props et **aucune `id`**, **aucune `label`**, **aucune `helpText`** ; son `<Input>` (`:272-289`) est appelé sans spread, rien ne passe au travers. Le `<label for>` qu'AC-B1 exige se serait donc lié **à rien**, et le champ n'aurait eu d'autre nom accessible que le repli du composant — `i18nMsg('journal-entry-form-col-account', …)`, une clé **`journal-entry-*`** sur une fiche produit. Pire : le patron du dépôt qu'un dev copierait est **déjà cassé** (`VatPurchaseAssistant.svelte:156-158`, `for="{uid}-charge"` ne désigne aucun élément) et `svelte-check` ne le voit pas. **Tranché** : prop `id?: string` **opt-in** — ce que les Dev Notes autorisaient déjà — et texte d'aide rendu **par la page**, pas par une prop.

**MEDIUM-2 — la commande de propagation ratait le site qu'elle devait trouver.** Les passes 1 et 2 prescrivaient `grep -on "compte par défaut de la société"` en annonçant **trois** sites. Elle en rend **deux** : `:583` est rédigé « compte **de produit** par défaut de la société ». Le motif ratait donc précisément l'un des deux sites **à corriger**. C'est le mode d'échec que la § *Propagation post-patch* et le garde-fou **P7** documentent déjà deux fois — *un motif sous-inclusif rend le garde-fou muet* —, et il s'est reproduit dans le geste même censé l'appliquer.

**MEDIUM-3 — le rayon de la mutation 4, par l'autre bout.** T-B4 affirmait que le scénario 1 « et lui seul » exerce le payload. Mais les scénarios 2 et 3 disaient « assigner un compte » **sans dire par quel canal** — et par l'interface, ils passent par le **même** littéral (`+page.svelte:323-330`, qui sert création *et* modification) : trois rouges pour une mutation qui en annonce un. C'est la **symétrie exacte** du MEDIUM corrigé en passe 1 sur la mutation 1, retombé par l'autre bout. Canal fixé : **par l'API** pour les scénarios 2 et 3, qui n'ont rien à prouver sur le payload.

Les six LOW : prop **`loadError`** ajoutée (sans elle, un `fetchAccounts` en échec laisse le champ vide **et muet**, D-B2 garantissant qu'aucun marqueur ne viendra) ; clés renommées **`product-form-revenue-account*`**, alignées sur les six voisines et sans recouvrement avec la famille que 16-2a D10 revendique **dans la même PR** ; typage **non optionnel** pour que « jamais omis » soit porté par le compilateur ; décompte des exécutions Playwright porté à **quatre** ; et la justification du cadrage corrigée — le site 5/5 ne « recopie » rien, il ouvre une facture neuve.

**Vérifié et jugé sain, à ne pas refaire** : traçabilité complète dans les deux sens (aucun AC orphelin, aucune tâche sans AC, aucune décision non invoquée) ; mutations 1, 2 et 3 **exactes**, rayon ni minoré ni majoré ; tous les décomptes recomptés à la source (5 sites `LineState`, 4 `<label for>`, 6 préfixes, 5 écrans, 4 locales) ; frontière avec 16-2a nette dans les deux sens, l'avertissement CHANGELOG de **D5** étant attribué à 16-2b **par les deux stories** ; et le code `PRODUCT_REVENUE_ACCOUNT_INVALID` de 16-2a **déjà couvert sans AC nouveau** — le `catch` de `submitForm` (`+page.svelte:357-359`) affiche tout message non reconnu, et le dropdown filtre déjà `active && postable && requiredAccountType`, rendant le chemin quasi inatteignable par l'interface.

⚠️ **Trend : `2H/2M/2L` → `0H/3M/1L` → `0H/3M/6L`. Le compteur MEDIUM stagne à 3, et la sévérité maximale reste MEDIUM d'une passe à l'autre** — c'est la formulation littérale du second critère de la § *Règle de splitting préventif*. **Arbitrage requis avant toute passe 4** : la nature des findings plaide contre le split (les trois MEDIUM sont des défauts de *remédiation*, pas de conception, et la story n'a jamais été prise en défaut sur son fond), mais le garde-fou est formellement déclenché et la décision revient au Project Lead.

---

**2026-08-04 — Passe 4 de `validate`** (Sonnet, deux lentilles, contexte frais, sous **dérogation** au garde-fou de splitting arbitrée par Guy). **0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW** — **BOUCLE CONVERGÉE**, critère d'arrêt de la § *Review Iteration Rule* atteint.

**La dérogation était le bon appel, et c'est maintenant mesuré.** Le pari était que la stagnation à 3 MEDIUM tenait à la *remédiation* et non à la taille de la story ; la passe 4, lancée sur les patches de la passe 3, fait tomber le compteur MEDIUM à **zéro**. Une story trop large n'aurait pas convergé d'un coup.

**Les six patches de la passe 3 sont validés un par un, contre le code réel** — c'était le mandat, et deux points portaient un risque réel que la passe lève :
- **la prop `id?: string`** n'entre en collision avec **rien** : `uid`/`invalidMsgId` (`AccountAutocomplete.svelte:82-83`) ne servent qu'à l'`aria-describedby` du message d'invalidité, et les cinq consommateurs passent tous leurs props nommément — l'ajout opt-in ne casse personne ;
- **le canal « par l'API »** imposé aux scénarios E2E 2 et 3 s'appuie sur `create_product` (`routes/products.rs:273`) et `update_product` (`:295`), dont **16-2a AC-A3 prescrit** qu'ils acceptent `defaultRevenueAccountId`. La dépendance a bien un objet.

Les rayons des **quatre** mutations sont désormais **exacts, ni minorés ni majorés** — vérifié un à un contre le code, y compris celui de la mutation 4, devenu « scénario 1 seul » précisément parce que les scénarios 2 et 3 contournent le littéral de payload.

**Les trois LOW sont trois ancres fausses, toutes introduites par la passe 3** — `VatPurchaseAssistant.svelte:161` → **`:162`**, `products.types.ts:39` → **`:40`** (`:39` est `name: string`), et `VatPurchaseAssistant.svelte:153-155` → **`:156-158`**, citée à deux endroits. Aucune n'affecte le fond : la démonstration du label orphelin a été revérifiée indépendamment et tient. Mais c'est la quatrième passe d'affilée où **la remédiation apporte ses propres défauts**, fussent-ils cosmétiques — à verser à la rétrospective de l'Epic 16.

**Trend complet : `2H/2M/2L` → `0H/3M/1L` → `0H/3M/6L` → `0H/0M/3L`.** Rotation **Sonnet → Haiku → Opus → Sonnet**, plafond de 8 passes non atteint.

**Statut : `ready-for-dev`, VALIDÉE.** Prête pour `bmad-dev-story`. ⚠️ **Même PR que 16-2a**, obligation écrite des deux côtés.
