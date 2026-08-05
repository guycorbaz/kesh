# Story 16.2b : Compte de produit sur la fiche produit — sélecteur et pré-remplissage

## Status

review

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

- [x] **T-B1 — Types et client API** (AC-B5)
  - [x] Champ dans les trois interfaces de `products.types.ts`.
  - [x] Mettre à jour le doc-comment de `ProductPicker.svelte:1-4`. *(Site trouvé par le grep de propagation, pas par les lentilles.)*
- [x] **T-B2 — Fiche produit** (AC-B1, AC-B2, AC-B6)
  - [x] Sélecteur, `<label for>` + clé i18n, hydratation à l'édition, envoi systématique. **Pas de `markInvalid`** (D-B2).
  - [x] **`openCreate()` (`:249-258`) remet le compte à `null`**, comme il le fait déjà pour les quatre autres champs — les `$state` vivent au niveau de la page (`:94-98`) et survivent à la fermeture du dialogue.
  - [x] `fetchAccounts(true)` — **et son assertion** `toHaveBeenCalledWith(true)`.
  - [x] **Prop `id?: string` opt-in** sur `AccountAutocomplete`, rendue sur son `<Input>` — défaut `undefined`, les cinq écrans inchangés (AC-B1).
  - [x] Clés **`product-form-revenue-account`** et **`product-form-revenue-account-help`** dans les **4** locales, **résolues dans `+page.svelte`** (étiquette dans le `<label for>`, aide dans un `<p>` sous le champ) — jamais lues depuis `AccountAutocomplete`, qui vit sous `features/` et ferait rougir le lint (AC-B6).
  - [x] Passer **`loadError`**, comme le font `InvoiceForm.svelte:823` et `VatPurchaseAssistant.svelte:162`. Sans lui, un `fetchAccounts` en échec laisse le champ **vide et muet** — le symptôme même qu'AC-B2 prévient, par une autre cause, et que **D-B2** garantit de ne pas signaler.
- [x] **T-B3 — Pré-remplissage** (AC-B3, AC-B4)
  - [x] `onProductSelect` pose le compte de l'article ; `addFreeLine` reste à `null`.
  - [x] Remplacer les **deux** commentaires d'ancrage, repérés par leurs numéros de ligne.
- [x] **T-B4 — Tests et preuve** (AC-B4, AC-B8)
  - [x] Unitaire : le pré-remplissage pose bien le compte de l'article ; l'assertion `toHaveBeenCalledWith(true)`.
  - [x] **« éditer un article avec compte → Nouveau produit → le sélecteur est vide »** (mutation 3) — **en E2E, pas en unitaire** : le scénario enchaîne deux ouvertures du dialogue sur une page SvelteKit complète, ce qu'un test de composant isolé ne monte pas.
  - [x] **E2E** (suffixe `.spec.ts` **obligatoire**) — **quatre exécutions Playwright au total** : les trois scénarios numérotés ci-dessous, **plus** le test de reset d'`openCreate()` de la puce précédente. La numérotation « scénario N » à laquelle renvoie AC-B8 désigne cette liste-ci.
    1. « fiche produit avec compte → facture depuis catalogue → la ligne porte le compte ». ⚠️ **Le produit doit être créé PAR L'INTERFACE**, pas semé par fixture : c'est ce scénario, et lui seul, qui exerce le **payload HTTP** de la fiche et discrimine donc la **mutation 4**. Un produit semé directement en base contournerait le payload, et la mutation passerait sans qu'aucun test ne bouge ;
    2. cas **AC-B4** — assigner un compte **par l'API** (pas par l'interface), l'**archiver ensuite**, monter la facture, constater marqueur et refus ;
    3. **« assigner un compte PAR L'API → l'archiver → ROUVRIR la fiche produit en édition → le libellé du compte s'affiche encore »**. ⚠️ **Sans ce scénario, la mutation 2 n'est attrapée par aucun E2E** : les deux premiers passent par le formulaire de facture, dont le `fetchAccounts(true)` est un appel **distinct** de celui du catalogue. Patron à reprendre : `frontend/tests/e2e/invoice-revenue-account.spec.ts:197-207` — archiver par l'API, puis `page.goto` vers l'écran, puis asserter le libellé. C'est lui qui a fermé ce piège en 16-1b.
  - [x] ⚠️ Le preset E2E `with-data` crée **déjà** un produit `'CI Product'` au compte `NULL` (`test_fixtures.rs:482-483`, appelé par `routes/test_endpoints.rs:203`) — en tenir compte dans les montages.
  - [x] Les **quatre** mutations d'AC-B8, exécutées, consignées avec leur sortie, fichiers restaurés à l'identique.
- [x] **T-B5 — Documentation** (AC-B7)
  - [x] CHANGELOG (fonctionnalité + avertissement `PUT`).
  - [x] `user-manual.tex` : corriger `:553` (champ **facultatif**) et `:583` (repli à trois maillons), décrire le pré-remplissage. **`make fr`**, commiter les PDF.
  - [x] **`:603` est à CORRIGER lui aussi, pas seulement à relire.** Le grep du symptôme — **`grep -on "par défaut de la société" docs/manual/fr/user-manual.tex`** — rend **trois** sites : `:583`, **`:603`**, `:611`. ⚠️ **Ne pas restreindre le motif à « compte par défaut de la société »** : il n'en rend que **deux** et rate précisément `:583`, rédigé « compte **de produit** par défaut de la société » — l'un des deux sites à corriger. *(Motif corrigé en passe 3 : c'est le mode d'échec que la § « Propagation post-patch » et le garde-fou P7 documentent déjà deux fois — un motif sous-inclusif rend le garde-fou muet.)* Or `:603` (« Variante ventilée ») affirme que **« les lignes laissées vides continuent de suivre le compte par défaut de la société »** — phrase qui devient **fausse** avec cette story : une ligne montée depuis un article suit d'abord le compte de l'**article**. `:611` en revanche reste **exact** (la mention « (défaut) » désigne toujours une ligne à `revenueAccountId = null`) : le lire, et **tracer le verdict « inchangé »** au Dev Agent Record — c'est ce qu'AC-B7 exige déjà du README.

    *(La passe 1 avait rangé `:603` parmi les sites « à tracer ». C'était insuffisant : greper le symptôme ne suffit pas si l'on ne lit pas ce que le site dit vraiment.)*
  - [x] README — feuille de route et « Fonctionnalités » vérifiées.
- [x] **T-B6 — Gate** (AC-B9) — frontend + E2E + `cargo test --workspace`, état final, exit 0.

### Review Findings

`bmad-code-review` **passe 1** — 2026-08-04, Opus 5, trois couches (Blind Hunter aveugle, Edge Case Hunter, Acceptance Auditor). Diff `main...HEAD`, 16-2a **et** 16-2b confondues ; les findings backend sont dans le story file de 16-2a.

**Décisions à rendre**

- [x] [Review][Decision] **(RÉSOLU — arbitrage de Guy, 2026-08-05 : ACCEPTÉE telle quelle, consignée en limitation **L1** ci-dessous, catégorie B ; issue GitHub de remédiation à ouvrir)** **D-B1 envoie corriger une fiche que D-B2 garantit muette — la boucle n'a pas de sortie** — `blind`, **MEDIUM**. D-B1 justifie de bloquer la facture *pour envoyer corriger la fiche produit* ; D-B2 garantit que la fiche **n'affichera aucun marqueur** ; et D4 (16-2a) fait que la valeur invalide, tant qu'elle n'est pas *changée*, n'est jamais revalidée. L'utilisateur voit sa facture refusée, ouvre la fiche de l'article, y trouve un champ **identique à celui d'un article sain**, l'enregistre sans y toucher (200), refait une facture, et se fait refuser à nouveau. Aucun écran ne liste les articles dont le compte est devenu inutilisable. Les trois décisions sont arbitrées et cohérentes prises deux à deux ; c'est leur composition qui enferme. Guy a arbitré D-B1 en 16-1b — à lui de dire si la conséquence est acceptée telle quelle, ou si un signal minimal sur la fiche (sans `markInvalid`) est dû.

**Patches**

- [x] [Review][Patch] **(CORRIGÉ — `products-page.test.ts` créé, et la mutation 2 REJOUÉE sous Vitest : rayon mesuré exactement 1, l'assertion visée et elle seule)** **AC-B2 exige une assertion `toHaveBeenCalledWith(true)` sur la fiche produit — elle n'existe pas, et trois cases la déclarent faite** — `auditor`, **HIGH**. `grep -rn "toHaveBeenCalledWith(true)" frontend/src frontend/tests` rend **une** occurrence, `InvoiceForm.test.ts:161` — **préexistante sur `main`** (`git show main:…` la donne en `:154`, décalée de 7 lignes par ce diff) et portant sur le `fetchAccounts(true)` d'`InvoiceForm`, que la story qualifie elle-même d'appel « **distinct**, déjà en place et non touché ». `ls "frontend/src/routes/(app)/products/"` → `+page.svelte` seul : **aucun fichier de test n'existe pour la fiche produit**, et aucun n'est ajouté. Le `fetchAccounts(true)` du catalogue (`+page.svelte:137`) n'a donc **aucun** tueur unitaire. AC-B2 existe précisément parce que 16-1b avait **mesuré** qu'un test fonctionnel reste vert sous la mutation. T-B2 et T-B4 sont cochées, et le Dev Agent Record déclare la mutation 2 « ✅ conforme au rayon annoncé » avec **1** rouge là où AC-B8 en annonçait **2**. [`frontend/src/routes/(app)/products/+page.svelte:137`]
- [x] [Review][Patch] **(CORRIGÉ — les trois volets d'AC-B9 rejoués et consignés au Dev Agent Record avec leur verdict EXACT ; la suite E2E n'est PAS déclarée verte, cf. § *Gate* ci-dessous)** **T-B6 est cochée sans aucune trace de gate au Dev Agent Record** — `auditor`, **HIGH**. AC-B9 exige frontend complet **et** `npm run test:e2e` **et** `cargo test --workspace` (« les locales vivant dans un crate Rust »), sur l'état final. Le Dev Agent Record (`:170-203`) ne contient **aucune** section de gate ; les seules affirmations vivent dans des messages de commit, et chacune est plus étroite que l'AC : `285c3a4f` déclare « E2E Playwright **PAS ENCORE EXÉCUTÉS** », `c38e2dbe` déclare « les 4 E2E passent » — ce qui n'est pas `npm run test:e2e`, la § *Test Locally First* imposant la **suite complète** avant tout push — et omet `build`. Aucun `cargo test --workspace` n'est déclaré nulle part. Rejouer, puis n'écrire que ce qui a tourné. [`16-2b-selecteur-et-prefill-frontend.md:136`]
- [x] [Review][Patch] **(CORRIGÉ — les TROIS sites du symptôme, manuel `:583`/`:603` + CHANGELOG + README, PDF régénéré ; le premier grep de propagation avait été sous-inclusif, il cherchait la formulation du manuel)** **Le manuel décrit un repli qui n'existe pas : une ligne laissée vide ne peut pas suivre le compte de l'article** — `edge`, **MEDIUM**. `:583` énonce « **Laissé vide**, il suit d'abord le compte de produit de l'article si la ligne vient du catalogue […], puis à défaut le compte par défaut de la société », et `:603` répète la règle sur un exemple chiffré. Or il n'existe **aucun** lien persistant ligne → article (`grep -rn "product_id" crates/kesh-db/migrations/*.sql` → aucune sortie ; l'entité `invoice.rs:58-66` ne le porte pas) : le compte de l'article est **recopié une fois** au choix dans le catalogue (`InvoiceForm.svelte:436`), il n'est pas un repli. Une fois le champ vidé, l'article n'est plus consultable et la validation crédite le défaut société. T-B5 demandait bien de corriger `:583` et `:603` — la correction appliquée a remplacé une phrase fausse par une autre, en confondant « ligne **venue** du catalogue » (pré-remplie) et « ligne **laissée vide** » (défaut société). ⚠️ Le PDF a été régénéré : l'erreur est **publiée**. [`docs/manual/fr/user-manual.tex:583`, `:603`]
- [x] [Review][Patch] **(CORRIGÉ — le décompte avant/après est REMONTÉ dans `pickFirstProduct`, donc les deux tests en héritent ; le sélecteur du picker est ancré sur `[role="dialog"]` et le paramètre `container` sert enfin)** **Le test Vitest négatif ne distingue pas « ligne sans compte » de « aucune ligne ajoutée »** — `blind`, **MEDIUM**. Le test positif (`:527`) capture `const before = accountInputs(container).length` puis asserte `inputs.length === before + 1` ; le négatif (`:547`) n'a **ni `before` ni assertion de cardinalité** — il lit `inputs[inputs.length - 1].value === ''`. Si `ProductPicker` cesse d'émettre sa sélection, aucune ligne n'est ajoutée, la dernière ligne préexistante est vide, et le test **passe sur un pré-remplissage qui n'a pas eu lieu** : exactement le mode d'échec qu'il prétend couvrir. Le helper `pickFirstProduct` aggrave le montage — il déclare un paramètre `container` **jamais utilisé** et cible `document.querySelector('ul li button')`, non ancré. [`frontend/src/lib/components/invoices/InvoiceForm.test.ts:547`]
- [x] [Review][Patch] **(CORRIGÉ — inventaire rétabli, modifiés / créés / ajoutés en revue ; les trois en-têtes dupliqués à vide supprimés)** **Le `File List` est vide, et trois en-têtes de Dev Agent Record sont dupliqués à vide** — `auditor`, **MEDIUM**. `File List` (`:197`), puis `Debug Log References` (`:199`), `Completion Notes List` (`:201`) et `File List` (`:203`) une seconde fois, tous vides, sous les sections déjà remplies. Aucune revue n'a donc d'inventaire déclaré à confronter au diff pour la moitié frontend de la PR — par contraste, 16-2a liste ses douze fichiers. [`16-2b-selecteur-et-prefill-frontend.md:197`]
- [x] [Review][Patch] **(CORRIGÉ — le tableau porte désormais « 2 E2E » et un encadré nomme ce que la campagne N'A PAS couvert : Vitest n'avait jamais été rejoué sous mutation)** **Le rayon de la mutation 1 est consigné à 2 rouges là où AC-B8 en exige 3** — `auditor`, **MEDIUM**. Le Dev Agent Record (`:190`) porte « **2** : scénarios 1 et 2 » — les deux **E2E**. L'« unitaire » explicitement nommé par AC-B8 (« Trois tests, pas deux : […] annoncer deux tests ferait conclure à un montage cassé au troisième rouge ») n'apparaît ni au décompte ni au commentaire, alors que le diff livre bien un test Vitest qui rougirait (`InvoiceForm.test.ts:527`). La campagne ayant été menée sur `frontend/build`, Vitest n'a manifestement pas été rejoué — et la colonne « conforme au rayon annoncé » porte pourtant un `✅`. [`16-2b-selecteur-et-prefill-frontend.md:190`]
- [x] [Review][Patch] **(CORRIGÉ — les deux verdicts « inchangé » écrits, README `:212` et `user-manual.tex:611`)** **Les deux traces documentaires exigées par AC-B7 et T-B5 sont absentes** — `auditor`, **MEDIUM**. AC-B7 impose de **tracer** la vérification du README « même si la conclusion est *rien à changer* », et T-B5 impose de tracer le verdict « inchangé » sur `user-manual.tex:611`. Les seules occurrences de `README` et de `:611` dans le story file sont les **prescriptions** (`:91`, `:132`), jamais leur trace. *(Le **contenu**, lui, est correct — cf. réfutation ci-dessous.)* [`16-2b-selecteur-et-prefill-frontend.md:170`]
- [x] [Review][Patch] **(CORRIGÉ — le bandeau est asserté, ET le NUMÉRO de la ligne qu'il nomme : la 2e, celle venue du catalogue)** **AC-B4 exige « en nommant la ligne » — c'est le seul volet que le test n'asserte pas** — `auditor`, **LOW**. L'E2E asserte le libellé, le marqueur, et un `create-invoice-button` `disabled` — or ce `disabled` est la **disjonction de six conditions** (`InvoiceForm.svelte:887`) et ne discrimine pas la cause. Le message qui *nomme* la ligne (`invalidLinesMessage`, rendu `:754`, clé `invoice-lines-revenue-account-invalid`) n'est asserté nulle part. [`frontend/tests/e2e/product-revenue-account.spec.ts:206`]
- [x] [Review][Patch] **(CORRIGÉ — `notifyError` + clé `product-form-revenue-account-load-error` dans les 4 locales)** **L'échec de chargement du plan comptable est avalé sans un mot à l'utilisateur** — `blind`, **LOW**. `catch` sans liaison, sans journalisation, sans `notifyError` — alors que le fichier importe `notifyError` et s'en sert ailleurs. L'utilisateur trouve, à la place de l'autocomplétion, un champ de saisie d'**identifiant technique brut** sans aucune explication, et peut y taper un numéro de compte en croyant bien faire. [`frontend/src/routes/(app)/products/+page.svelte:139`]
- [x] [Review][Patch] **(CORRIGÉ — l'en-tête dit maintenant que le `ProductResponse` est transmis ENTIER, et INTERDIT d'y réintroduire un snapshot qui ferait disparaître le champ en silence)** **Le doc-comment de `ProductPicker` annonce un changement que le composant n'a pas reçu** — `blind`, **LOW**. L'en-tête déclare que le compte « a rejoint le snapshot », mais le diff du composant ne touche **aucune ligne de code** — le champ arrive parce que l'objet `ProductResponse` était déjà transmis entier. Le `?? null` d'`InvoiceForm.svelte:436` est alors soit mort, soit — si un refactor futur restreint le snapshot — le convertisseur qui fait retomber la ligne **en silence** sur le défaut société, ce que D-B1 déclare inacceptable trois lignes plus haut. [`frontend/src/lib/components/invoices/ProductPicker.svelte:1`]
- [x] [Review][Patch] **(CORRIGÉ — prop opt-in `describedBy` sur `AccountAutocomplete`, composée avec `invalidMsgId` par une espace ; le `<p>` d'aide reçoit son `id`)** **Le texte d'aide n'est relié au champ par aucun `aria-describedby`** — `edge`, **LOW**. La prop `id` ajoutée à `AccountAutocomplete` lie correctement le `<label for>`, mais la phrase qui porte le **sens de la valeur vide** (« les lignes créées depuis cet article suivent le compte par défaut de la société ») n'est accessible qu'à la vue. `grep -nF "aria-describedby"` ne rend aucune sortie ni dans le composant ni dans la page ; `invalidMsgId` existe mais ne sert qu'au message d'invalidité, non activé ici. [`frontend/src/routes/(app)/products/+page.svelte:671`]

**Réfutés en passe 1** (consignés pour qu'une passe suivante ne les rejoue pas)

- **« `CHANGELOG.md` et `README.md` manquent »** — réfuté. `git diff main...HEAD -- CHANGELOG.md README.md --stat` → `CHANGELOG.md | 7 +`, `README.md | 1 +`. Le CHANGELOG porte bien l'avertissement **D5** sur le `PUT` full-replace, et `README.md:212` conserve la ligne de feuille de route v0.9.0. Seule la **trace** de la vérification manque (patch ci-dessus). *(L'erreur venait de ma propre lecture d'un `--stat | tail -30` tronqué, corrigée par l'Acceptance Auditor.)*
- **« Les numéros de compte déterministes rendent les E2E non rejouables »** — réfuté par lecture du montage : `test.beforeAll` appelle `seedTestState('with-company')`, qui déclenche `truncate_all` (`routes/test_endpoints.rs:172`), lequel tronque bien `accounts` — assertion explicite dans `test_fixtures.rs:612`. Les slots 1 à 4 sont distincts au sein d'un run, et la table est vidée entre deux runs.
- **« Le sélecteur de la fiche produit propose les comptes archivés »** — réfuté : `AccountAutocomplete.svelte:183-186` filtre `a.active` avant de construire les propositions. `fetchAccounts(true)` sert à **afficher le libellé** de la valeur courante, ce qui est précisément l'objet d'AC-B2.

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

### Limitation assumée — L1 : la boucle D-B1 / D-B2 / D4 n'a pas de sortie dans l'application

*(Relevée en passe 1 de `bmad-code-review`, **arbitrée par Guy le 2026-08-05 : acceptée telle quelle**. Catégorie **B** au sens de la § *Tech debt management* — limitation documentée avec scope explicite et remédiation à planifier.)*

**Le mode d'échec, en quatre temps.** L'utilisateur monte une facture depuis un article dont le compte a été archivé ailleurs. **D-B1** : la ligne affiche le compte, le marque invalide, et l'enregistrement est refusé — pour l'envoyer corriger la fiche produit. Il ouvre cette fiche : **D-B2** garantit qu'elle n'affiche **aucun** marqueur, le champ y est indiscernable de celui d'un article sain. Il l'enregistre sans y toucher : **D4** (16-2a) ne revalide que si le compte **change**, donc 200, la valeur reste. Il refait une facture : même refus.

**Chaque décision est juste prise deux à deux ; c'est leur composition qui enferme.** Et aucun écran ne liste les articles dont le compte est devenu inutilisable, donc rien ne dit *lequel* corriger quand plusieurs sont en cause.

**Ce qui limite la portée réelle** : le déclencheur est l'archivage d'un compte **déjà référencé** par un article. C'est un événement rare — et la ligne de facture, elle, **nomme** le compte fautif par son numéro (`invoice-lines-revenue-account-invalid`), ce qui donne à l'utilisateur de quoi retrouver l'article à la main même sans écran dédié.

**Remédiation à planifier** — une vue « articles dont le compte de produit est devenu inutilisable », ou un signal minimal sur la fiche qui n'aille pas jusqu'à `markInvalid`. À trancher hors de cette PR : c'est une feature, pas un correctif.

**Tracée en [issue #286](https://github.com/guycorbaz/kesh/issues/286)** (`enhancement` + `technical-debt`), ouverte le 2026-08-05 — c'est elle qui fait foi, pas ce paragraphe (§ *Issue Tracking Rule*). Elle porte **les deux déclencheurs** : l'archivage du compte (L1) et son passage en **non imputable**, corollaire de l'arbitrage `postable` de 16-2a, qui aboutit à la même impasse par un autre chemin.

### References

- Story **16-2a** — colonne, API, validation. **Même PR obligatoire.**
- Story parente **16-2** — archivée, 4 passes de `validate`.
- Story **16-1b** — `AccountAutocomplete` et ses props opt-in, `account-label.ts`, `account-validity.ts`, l'arbitrage *afficher + signaler + bloquer*, et la mesure du piège `fetchAccounts(true)`.
- `CLAUDE.md` § « Test Locally First » (frontend, E2E) et § « Synchroniser TOUTES les docs ».

---

## Dev Agent Record

### Agent Model Used

Opus 5 (`bmad-dev-story`, 2026-08-04).

### Debug Log References

**Trois pièges du montage E2E, tous rencontrés — et deux d'entre eux produisaient des tests qui MENTAIENT.**

1. **`KESH_COOKIE_SECURE=false` est obligatoire en local HTTP.** Playwright rejette un cookie `Secure` sur `http://127.0.0.1` — `isLocalHostname()` (`network.js:62`) ne reconnaît que `"localhost"` et `"*.localhost"`. Sans la variable, **toute** la suite échoue en 401. Documenté au `CLAUDE.md` § *Test Locally First* → E2E, à la ligne de code près. *(Un diagnostic erroné — « SameSite » — avait été produit puis réfuté ; cf. issue #285.)*
2. **⚠️ LE BACKEND SERT `frontend/build`, PAS LES SOURCES.** Une mutation d'un `.svelte` **n'a aucun effet** sans `npm run build` préalable. Première campagne de mutations menée sans rebuild : **la mutation 1 laissait les 4 tests verts**, ce qui donnait à croire à des tests muets. Ils ne l'étaient pas — c'est le binaire testé qui était périmé. **Toute campagne de mutation E2E rebuild d'abord.**
3. **Les numéros de compte tirés au hasard rendaient la suite INSTABLE.** `accounts` porte `uq_accounts_company_number` : deux tirages qui se percutent font échouer la création. Observé : `4 passed`, puis `3 failed` sur le **même** code. Numéros rendus **déterministes** (un `slot` par test, 39101..39104). Trois runs consécutifs verts ensuite. *Un test intermittent est pire qu'un test rouge : on finit par le croire sur parole.*

### Completion Notes List

**Les QUATRE mutations d'AC-B8 sont exécutées, sur une ligne de base éprouvée stable (3 runs verts consécutifs).**

| Mutation | Rouges | Conforme au rayon annoncé |
|---|---|---|
| 1 — `onProductSelect` repose `null` | **2 E2E** : scénarios 1 et 2 | ⚠️ **partiel — voir ci-dessous** |
| 2 — `fetchAccounts(true)` → `()` | **1 E2E** : scénario 3 | ⚠️ **partiel — voir ci-dessous** |
| 3 — `openCreate()` sans reset | **1** : scénario 4 | ✅ |
| 4 — champ retiré du payload | **1** : scénario 1 | ✅ et lui seul — les scénarios 2 et 3 assignent par l'API, comme la passe 3 l'a imposé |

**Les décisions de la revue de spec sont validées par l'exécution**, et pas seulement par le raisonnement : le canal « par l'API » des scénarios 2 et 3 (passe 3) donne bien à la mutation 4 un tueur **unique** ; et le troisième scénario (passe 1) est bien le **seul** tueur E2E de la mutation 2.

#### ⚠️ Ce que cette campagne n'a PAS couvert — rectifié en passe 1 de revue

**La campagne s'est déroulée entièrement en E2E, sur `frontend/build`. Vitest n'a jamais été rejoué sous mutation**, et le tableau ci-dessus l'a longtemps tu. Deux conséquences, l'une de comptage, l'autre de fond :

- **Mutation 1 — rayon annoncé 3, mesuré 2.** AC-B8 énonce « **Trois tests, pas deux** : […] annoncer deux tests ferait conclure à un montage cassé au troisième rouge ». Les deux rouges consignés sont les deux **E2E** ; l'**unitaire** nommé par l'AC (`InvoiceForm.test.ts`, « la ligne créée porte le compte de l'article ») existe bien et rougirait, mais il n'a pas été exercé. La colonne portait pourtant un `✅`.
- **Mutation 2 — le tueur unitaire prescrit n'existait pas.** AC-B2 impose `expect(fetchAccounts).toHaveBeenCalledWith(true)` **sur la fiche produit**, précisément parce que 16-1b avait **mesuré** qu'un test fonctionnel reste vert sous cette mutation. L'unique occurrence de cette assertion dans le dépôt était celle d'`InvoiceForm.test.ts`, **préexistante sur `main`** et portant sur un appel que la story qualifie elle-même de « distinct ». Aucun fichier de test n'existait pour `routes/(app)/products/+page.svelte`. T-B2 et T-B4 étaient cochées.

**Rectification** — `frontend/src/routes/(app)/products/products-page.test.ts` est créé, et la mutation 2 a été **rejouée sous Vitest** :

```
$ sed -i 's/fetchAccounts(true)/fetchAccounts()/' "src/routes/(app)/products/+page.svelte"
$ npx vitest run "src/routes/(app)/products/products-page.test.ts"

  [
-   true,
+   undefined,
  ]
 ❯ src/routes/(app)/products/products-page.test.ts:98:29
     98|   expect(fetchAccountsMock).toHaveBeenCalledWith(true);

 Tests  1 failed | 2 passed (3)
```

**Rayon unitaire mesuré : exactement 1**, l'assertion visée et elle seule — conforme à ce qu'AC-B8 annonce pour la mutation 2. Fichier restauré à l'identique (`git diff` sur `+page.svelte` ne rend que les ajouts de la revue).

#### Gate d'AC-B9 — rejoué en passe 1 de revue, sur l'état final

*(Section absente de la version initiale de ce Record : T-B6 était cochée sans aucune trace. Ce qui suit est ce qui a **réellement tourné**, le 2026-08-05.)*

| Volet | Commande | Verdict |
|---|---|---|
| Backend | `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest), DB `kesh_gate2` | **`2115 tests run: 2115 passed, 4 skipped`**, 3154 s, **exit 0**, lu dans le log |
| Frontend | `check` / `lint-i18n-ownership` / `test:unit` / `build` | **4 × exit 0**, unitaires **510/510** sur 63 fichiers |
| E2E | `npm run test:e2e` (suite complète) | **170 passed, 42 failed, 19 skipped** — voir ci-dessous |

**Contrôle de composition du backend, pas seulement de total** : `main` était à **2102**, le run donne **2115**, soit **+13** — les tests livrés par 16-2a et 16-2b, plus `put_without_the_key_erases_an_existing_account` ajouté en revue. Aucun test n'a disparu en route, le mode d'échec qui avait coûté quatre suppressions par mégarde sur 16-2a.

⚠️ **Deux pièges d'exécution payés, à ne pas re-payer** :

1. **La base de gate est `kesh_gate2`, plus `kesh_gate`.** Cette dernière porte le schéma jusqu'à 16-1a mais **pas** `products.default_revenue_account_id`, et son `_sqlx_migrations` ne contient qu'une ligne, marquée `success = 0`. Un premier gate y est mort à 1479/2115 sur **un seul** échec, `Unknown column 'default_revenue_account_id' in 'field list'` — 38 minutes perdues. Les `#[sqlx::test]` sont **aveugles** à ce défaut : ils créent une base éphémère et y rejouent tout le `MIGRATOR`. Seul un test `--lib` passant par `test_pool()` pouvait le révéler. **Contrôle de 5 secondes** : `SELECT COUNT(*) FROM <db>._sqlx_migrations` doit égaler `ls crates/kesh-db/migrations/*.sql | wc -l`.
2. **`cargo fmt --check` a mordu d'emblée** sur le test ajouté en remédiation — 30 secondes, au lieu d'une heure si le pré-vol avait été placé après les tests.

#### ⚠️ La suite E2E n'est PAS déclarée verte — et ce n'est pas notre fait

**Les 4 scénarios de cette story passent**, y compris le volet « nommer la ligne » ajouté en passe 1 :

```
✓ 161 product-revenue-account.spec.ts:133 › la ligne porte le compte           (2.0s)
✓ 162 product-revenue-account.spec.ts:206 › marquée et enregistrement refusé   (1.8s)
✓ 163 product-revenue-account.spec.ts:301 › le libellé reste affiché           (1.7s)
✓ 164 product-revenue-account.spec.ts:349 › le sélecteur est vide              (1.6s)
```

**La baseline est rouge, et c'est démontré, pas supposé.** La même suite a été rejouée sur `main` — worktree dédié, **même backend, même base, même montage** :

| | passed | failed | skipped | total |
|---|---|---|---|---|
| `main` | 169 | **39** | 19 | 227 |
| branche | 170 | **42** | 19 | **231** |

Le delta de total est exactement **+4** : nos quatre scénarios. Les **39** rouges de `main` se retrouvent **tous** sur la branche.

**Les 3 rouges restants sont du flake, démontré par mesure** — ils ne sont pas reproductibles et **se déplacent** : rejoués isolément, `reconciliation-manual:66` et `users:117` passent ; `sidebar-navigation` fait échouer `:71` à un tirage et `:34` au suivant, **sans changement de code**. Et **`main` flake identiquement** sur cette spec — trois tirages : `4 passed`, `1 failed`, `4 passed`. Tracé en **[issue #287](https://github.com/guycorbaz/kesh/issues/287)**.

**Le reste du bruit est déjà tracé** : #96, #97, #107, #108, #124, #282. Cinq échecs relèvent en outre du **montage**, non du code : le dépôt documente que `invoice-send-email.spec.ts` exige un backend **avec** SMTP factice et `invoice-send-email-nosmtp.spec.ts` le **même sans** — deux runs séquentiels et deux configurations **opposées** (`docs/testing.md`), là où un montage unique a été employé. Le test l'énonce lui-même : `GET /_test/sent-emails → … — backend démarré sans SMTP factice ?`.

**Ce que cette section ne dit pas** : que la suite est verte. Elle ne l'est pas, et l'écrire serait précisément la faute que la § *Test Locally First* interdit. Ce qui est établi, c'est que **la branche n'ajoute aucun rouge** à une baseline déjà rouge, et que **ce qu'elle ajoute est vert**.

#### Traces documentaires exigées par AC-B7 et T-B5

*(Absentes de la version initiale de ce Record — AC-B7 impose de tracer « même si la conclusion est *rien à changer* ». Rétablies en passe 1 de revue.)*

- **`README.md` — feuille de route** : vérifiée, **inchangée**. `README.md:212` porte toujours `| v0.9.0 | **E16 Facturation avancée** (#152, #144, #151) … | 🚧 En cours |` — l'epic n'est pas clos, le statut reste juste.
- **`README.md` — section « Fonctionnalités »** : **modifiée** (une puce ajoutée), puis **re-corrigée en passe 1 de revue** (cf. ci-dessous).
- **`user-manual.tex:611`** : lu, verdict **inchangé** — la mention « (défaut) » y désigne une ligne à `revenueAccountId = null`, ce qui reste exact après cette story.

#### Correction de passe 1 — la « chaîne de repli à trois maillons » n'existe pas

Le manuel (`:583`, `:603`), le **CHANGELOG** et le **README** annonçaient tous trois un repli `ligne → article → société`. C'est faux : il n'existe **aucun** lien persistant entre une ligne et l'article dont elle vient (`grep -rn "product_id" crates/kesh-db/migrations/*.sql` → aucune sortie). Le compte de l'article est **recopié une fois**, au moment du choix dans le catalogue ; une ligne **vidée** retombe sur le défaut société, pas sur l'article.

⚠️ **L'erreur vient de la spec elle-même** — la story parente 16-2 l'écrit en toutes lettres (`16-2-compte-produit-catalogue.md:405` : « AC5 fait passer à trois ») — et elle s'est propagée aux **trois** livrables documentaires. Le premier grep de propagation ne l'avait pas vue : il cherchait la formulation du manuel, quand le CHANGELOG et le README écrivaient « trois maillons » et « ligne → article → société ». *C'est exactement le mode d'échec que la § « Propagation post-patch » du `CLAUDE.md` décrit — un motif sous-inclusif rend le garde-fou muet.* Les trois sites sont corrigés dans le même patch, et le PDF régénéré (`make fr`, 55 pages, `pdftotext | grep -c "recopie"` → 3).

### File List

*(Section laissée vide par le dev, et suivie de trois en-têtes dupliqués à vide — rétablie en passe 1 de revue. Sans elle, aucune revue n'avait d'inventaire déclaré à confronter au diff pour la moitié frontend de la PR.)*

**Story 16-2b — modifiés**

- `frontend/src/lib/features/products/products.types.ts` — champ dans les trois interfaces.
- `frontend/src/routes/(app)/products/+page.svelte` — sélecteur, `<label for>`, `fetchAccounts(true)`, reset d'`openCreate()`, envoi systématique.
- `frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte` — prop opt-in `id`.
- `frontend/src/lib/components/invoices/InvoiceForm.svelte` — pré-remplissage dans `onProductSelect`, commentaires d'ancrage des sites 2 et 3.
- `frontend/src/lib/components/invoices/ProductPicker.svelte` — doc-comment.
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — 2 clés × 4 locales.
- `CHANGELOG.md`, `README.md`, `docs/manual/fr/user-manual.tex` + `.pdf`.

**Story 16-2b — créés**

- `frontend/tests/e2e/product-revenue-account.spec.ts` — 4 scénarios.
- `frontend/src/lib/components/invoices/InvoiceForm.test.ts` — 2 tests ajoutés à un fichier préexistant.

**Ajoutés en passe 1 de revue**

- `frontend/src/routes/(app)/products/products-page.test.ts` — **créé** (tueur unitaire d'AC-B2 + signalement d'échec de chargement).
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — clé `product-form-revenue-account-load-error`.

## Change Log

**2026-08-05 — Passe 1 de `bmad-code-review`** (Opus 5, trois lentilles, diff `main...HEAD`). **1 MEDIUM décision + 2 HIGH + 5 MEDIUM + 4 LOW**, tous clos. **Boucle NON convergée — passe 2 due** (LLM ≠ Opus, contexte frais).

**Le finding de fond : AC-B2 exigeait un tueur unitaire qui n'existait pas, et trois cases le déclaraient fait.** `grep -rn "toHaveBeenCalledWith(true)"` ne rendait **qu'une** occurrence — **préexistante sur `main`** et portant sur le `fetchAccounts(true)` d'`InvoiceForm`, appel que la story qualifie elle-même de **distinct**. Aucun fichier de test n'existait pour la fiche produit. L'AC existait précisément parce que 16-1b avait **mesuré** qu'un test fonctionnel reste vert sous cette mutation : la seule protection prescrite était donc absente, et le Dev Agent Record annonçait la mutation « conforme ». `products-page.test.ts` créé, mutation **rejouée sous Vitest**, rayon mesuré **exactement 1**.

**La campagne de mutations s'était déroulée entièrement en E2E, sur `frontend/build` — Vitest n'avait jamais été rejoué.** Le tableau le taisait, et portait des `✅` sur deux lignes dont le rayon annoncé n'avait pas été atteint. Rectifié, avec la mention explicite de ce que la campagne **n'a pas** couvert.

**La « chaîne de repli à trois maillons » n'existe pas** : aucun lien persistant ne relie une ligne à l'article dont elle vient. L'erreur venait de la **spec parente**, s'était propagée aux **trois** livrables documentaires — manuel, CHANGELOG, README — et le PDF était **publié**. Les trois sites corrigés, PDF régénéré, errata posé à la source. Le premier grep de propagation l'avait manquée : il cherchait la formulation du manuel quand les deux autres écrivaient « trois maillons ». *Motif sous-inclusif, garde-fou muet — le mode d'échec que la § « Propagation post-patch » décrit.*

**Limitation L1 arbitrée par Guy : acceptée telle quelle**, catégorie B, tracée en **[issue #286](https://github.com/guycorbaz/kesh/issues/286)** avec le corollaire `postable` de 16-2a — deux déclencheurs, une seule sortie.

**Gate d'AC-B9 rejoué et consigné dans son verdict exact** : backend **2115/2115** exit 0, frontend **510/510** plus `check`/`lint`/`build` à 0, et la suite E2E **explicitement non déclarée verte**. La baseline est rouge et c'est **démontré** — même suite rejouée sur `main` à montage identique : **39** rouges contre 42, delta de total **+4** correspondant exactement à nos quatre scénarios, tous verts. Les 3 rouges restants sont du **flake mesuré** — ils se déplacent entre deux tirages sans changement de code, et `main` flake identiquement (`4 passed`, `1 failed`, `4 passed`). Tracé en **[issue #287](https://github.com/guycorbaz/kesh/issues/287)** ; le reste du bruit l'était déjà (#96, #97, #107, #108, #124, #282), et cinq échecs relèvent du montage — le dépôt documente **deux** configurations backend opposées pour les specs d'e-mail, là où une seule a été employée.

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
