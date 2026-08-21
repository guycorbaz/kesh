# Story 23.3b : la garde contre les libellés en dur, et les huit sites qu'elle révèle

Status: review

> **Base** : branchée sur `main` **après le merge de la 23-3** (`046efa51`). Tous les faits, lignes
> et décomptes de ce document ont été **vérifiés au grep sur cette base** le 2026-08-20.
> ⚠️ **Les numéros de ligne dateront** : chaque référence porte aussi un **motif de grep** —
> **le motif fait foi, pas la ligne.**
>
> **Catalogues** : les quatre locales `fr-CH` (source), `de-CH`, `it-CH`, `en-CH`, dans
> `crates/kesh-i18n/locales/<locale>/messages.ftl`.

## Pourquoi cette story existe, et pourquoi avant la 23-4

⚠️ **Une fonction de libellé qui n'appelle jamais `i18nMsg` est invisible de TOUT l'appareil de
contrôle de cet epic.** Le moissonneur (`i18n-harvest.js`), l'allowlist (`i18n-dette-connue.ts`) et
les trois gardes existantes — `loader.rs::parity_between_locales`, `i18n-keys.test.ts`,
`i18n-un-repli-par-cle.test.ts` — ne relèvent que les **sites `i18nMsg`**. Ce qui n'appelle pas
n'existe pour aucune d'elles : c'est l'angle mort que l'issue
[#255](https://github.com/guycorbaz/kesh/issues/255) décrit sans le couvrir.

Le défaut a été trouvé **en production**, en passe 4 de revue de la 23-3 :
`supplierInvoiceStatusLabel()` retournait `'Ouverte'` / `'Payée'` / `'Annulée'` en dur **sous un
en-tête de colonne que la story venait de traduire** — un germanophone lisait « Status » au-dessus de
« Payée », sur chaque ligne et en badge sur chaque fiche. Corrigé dans la 23-3.

⚠️ **Ce n'est pas un accident : c'est un patron.** Le doc-comment *« Libellé FR du statut d'un X
(fallback i18n côté composant) »* est copié **trois fois** — `payment-batch-helpers.ts:19`,
`credit-note-helpers.ts:22`, `supplier-invoice-helpers.ts:19` *(celui-ci corrigé en 23-3)* — et un
**quatrième** occurrence du patron porte le même défaut **sans commentaire** — c'est le **site 5 du
tableau** (`invoices/[id]:602`). ⚠️ **Ce dernier
est le plus instructif : un grep du doc-comment ne l'aurait pas trouvé.**

**L'urgence est un problème d'ordonnancement, pas de gravité actuelle.** ⚠️ Sur `payment-batches` et
`credit-notes`, le défaut est **LATENT** : leurs en-têtes de colonne ne sont pas encore traduits
(`payment-batches-col-status` et `credit-notes-col-status` sont **à l'allowlist**, absents des quatre
catalogues — vérifié). Personne ne voit aujourd'hui « Status » au-dessus de « Généré ». **C'est la
traduction qui l'activera** — exactement comme pour `supplier-invoices-col-total`. Or ces deux
domaines sont ceux des stories **23-4 et 23-5**, et 49 de leurs clés attendent à l'allowlist
(30 + 19, recompté).

## Story

**En tant que** développeur d'un rollout de traduction de l'epic 23,
**je veux** qu'une garde refuse une fonction de libellé qui n'appelle pas `i18nMsg`,
**afin que** le patron cesse d'être recopié — et que traduire un domaine ne fige plus des valeurs
françaises sous des en-têtes traduits.

## ⚠️ Ce que la garde couvre, et ce qu'elle NE PEUT PAS couvrir

**Ce paragraphe est le cœur de la story. Il a été établi par mesure, pas par intuition.**

La garde vise **un patron précis** : *une fonction dont la valeur est affichée retourne un littéral
sans passer par `i18nMsg`*. Elle le détecte **par le nom** de la fonction (`*Label`, `*Text`,
`*Display`), puis par l'inspection de son corps.

| forme du défaut | la garde la voit ? | pourquoi |
|---|---|---|
| fonction `*Label` → `return '<littéral>'` | ✅ **oui** — c'est son objet | sites **1, 2, 3, 5, 7** — **cinq** |
| valeur affichée sans aucun appel (`{data.company.orgType}`) | ❌ **non** | il n'y a **ni fonction ni littéral** — le défaut est l'*absence* d'appel |
| littéraux dans un **tableau de données** (`navGroups`) | ❌ **non** | pas un corps de fonction ; et **indiscernable** d'une table de replis légitime sans analyse de flot |
| **nœud de texte** de markup (`<Dialog.Title>Valider la facture</Dialog.Title>`) | ❌ **non** | ni fonction, ni littéral JS |
| `const xLabel = $derived.by(() => …)` | ❌ **non** *(si T4 ne cherche que le mot-clé `function`)* | **3 occurrences aujourd'hui, aucune fautive** — mais un futur `const statusLabel = $derived.by(() => x ? 'Ouvert' : 'Fermé')` passerait. ⚠️ **Élargir le motif de déclaration si le coût est trivial** |

⚠️ **Les sites 4, 6 et 8 sont donc corrigés À LA MAIN, hors portée de la garde, et le motif est
écrit ci-dessus.** Ne pas élargir la garde pour les attraper : la mesure ci-dessous dit ce que ça
coûterait.

**Mesure de l'élargissement « par le contenu »** *(détecter tout littéral « qui ressemble à du
français »)* : **1035** littéraux bruts, dont 860 sont des replis légitimes en 2ᵉ argument
d'`i18nMsg` → **175 restants à trier**, dont ~160 sans rapport avec ce défaut (messages d'erreur,
prose de commentaire `<!-- -->`). ⚠️ **Et cet angle rate quand même `Brouillon` et 7 des 9 libellés
de navigation**, qui n'ont ni accent ni mot-outil. **Il coûte 160 exemptions pour ne rien gagner** —
c'est-à-dire une garde qu'on désactivera au premier gate bruyant. **Proscrit comme critère
bloquant.**

## Critères d'acceptation

1. **AC1 — la garde existe, s'appelle `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`, et lit
   les SOURCES.** Elle relève les **déclarations de fonction** dont le nom finit par `Label`, `Text`
   ou `Display` dans `src/`, et échoue si l'une retourne un littéral sans `i18nMsg` sur le chemin.
   ⚠️ **Elle lit les sources, jamais les catalogues** — une garde qui s'appuie sur les catalogues
   s'éteint au moment où la traduction est livrée, c'est-à-dire quand le risque devient réel.
   ⚠️ **Une garde codée en dur sur une liste de chemins ne satisfait PAS cet AC** : elle
   n'empêcherait aucune récidive, qui est l'objet même de la story.
2. **AC2 — la garde rougit sur les CINQ sites de son patron AVANT tout correctif**, et la sortie du
   run **brute** est collée dans le Dev Agent Record. ⚠️ **Le balayage rend HUIT hits, pas cinq** —
   mesuré en passe 2 de `validate`, script exécuté. Les cinq violations réelles, **plus trois
   candidats à écarter par CRITÈRE** *(jamais par allowlist)* : `account-label.ts:35`
   `invoiceRevenueAccountLabel` → `'—'`, `account-label.ts:66` `creditNoteRevenueAccountLabel` →
   `'—'`, et `settings/opening-balances/+page.svelte:131` `roleLabel` → `''`. **Coller les huit, puis
   la ventilation** — c'est ce que réclame AC3. ⚠️ **C'est la seule preuve qu'elle les voit** ; sans
   elle, une garde vide passe tous les autres AC.
3. **AC2-bis — elle rougit encore sous mutation APRÈS les correctifs** : remettre en dur l'un des
   sites corrigés fait échouer le test, sortie citée. Une garde non éprouvée par mutation est un
   test muet.
4. **AC3 — borne EXACTE sur une grandeur STABLE.** Le nombre déclaré est celui des **fonctions
   candidates relevées** par le balayage (conformes + allowlistées + en violation), pas celui des
   violations — un compteur de violations tombe à zéro et n'atteste plus rien. Le Dev Agent Record
   porte la **ventilation**, et la somme doit égaler le total. Consigne inscrite dans le fichier :
   « un écart se recompte, il ne s'ajuste pas ».
5. **AC4 — allowlist justifiée, motif par entrée.** Les quatre entrées connues d'avance sont données
   au § *Faux amis* ci-dessous. ⚠️ **Aucune entrée ne peut être ajoutée au seul motif qu'un site
   n'était pas prévu** — cf. AC5-bis.
6. **AC5 — les huit sites du tableau sont corrigés**, chacun avec ses clés dans les quatre locales
   *(clés neuves OU réutilisées — le site 4 réutilise `onboarding-org-*`, le site 7 des clés déjà
   traduites : la réutilisation est la bonne réponse quand l'équivalence existe)*.
7. **AC5-bis — tout site relevé par la garde et absent du tableau est soit corrigé ici, soit
   consigné dans le Dev Agent Record avec sa raison ET une issue GitHub** — jamais inscrit à
   l'allowlist. ⚠️ Le tableau est un **plancher**, pas un inventaire clos : la garde balaie tout
   `src/`, c'est sa raison d'être.
8. **AC6 — les termes non attestés sont tranchés PAR GUY, puis promus en partie A du glossaire** avec
   leur clé attestante. Portée : les trois statuts **et** les six codes de `failedItemLabel`.
9. **AC7 — tout test qui verrouille le français est corrigé, unitaire ET E2E.** **Quatre** verrous
   connus au 2026-08-20 : trois blocs unitaires *(sites 1, 2, 3)* **et** `fiscal-years.spec.ts:270`,
   qui cible le dialogue du site 8 **par son libellé**. ⚠️ **Un sélecteur E2E se bascule sur
   `data-testid`, il ne se « répare » PAS en y réécrivant la chaîne française** — ce serait remettre
   le verrou en place sous couvert de correctif. ⚠️ **Cette liste n'est pas close** : la balayer avec
   les DEUX greps donnés ci-dessous.
10. **AC8 — `sitesTotal` d'`i18n-keys.test.ts` est recompté depuis la source et déclaré dans le Dev
    Agent Record**, jamais ajusté en silence. Base au départ : **1502**.
11. **AC9 — gates complets verts, E2E comprise**, PR en `refs #316`, et le corps de la PR mentionne
    que le **site 8** déborde le périmètre de l'issue #255 *(le site 7, lui, la FERME)*.

## Tasks

- [x] **T1 — Arbitrage de Guy — RENDU le 2026-08-20**, cf. § *Les termes* ci-dessous. AC6.
      ⚠️ **Il ne reste rien à inventer** : les neuf termes sont soit attestés au catalogue, soit tranchés. ⚠️ **Si un terme NEUF apparaît en cours de route** — un
      dixième que le relevé ne couvre pas —, le proposer au Dev Agent Record et **S'ARRÊTER**. Ne jamais
      inventer : ces valeurs sont figées dans quatre catalogues **et** promues en partie A.
- [x] **T2 — Exporter `corpsDeFonction`** depuis `i18n-literal-reader.js` (+ son test) — elle existe
      (`:374`) mais **n'est pas exportée**. AC1. ⚠️ **Elle ne suffit PAS à elle seule** : elle prend en
      entrée la position d'une accolade **déjà connue**. Localiser la déclaration (nom + vraie accolade
      de corps) reste à écrire. ⚠️ **Piège** : chercher « le premier `{` après le nom » casse sur un
      type inline — `function fooLabel(opts: { x: number }): string {` — il faut le `{` qui suit la
      **fermeture de la signature**. Aucune des 32 déclarations actuelles n'a ce cas ; **le verrouiller
      par un test** plutôt que compter dessus.
- [x] **T3 — Étendre `masquerCommentaires` aux commentaires `<!-- -->`** (+ son test) — elle ne masque
      aujourd'hui que `//` et `/* */`, ce qui laisse **21** blocs de prose française passer pour des
      littéraux dans les `.svelte`. AC1. ⚠️ **Vérifié en passe 2, à recoller au Record** : **zéro** appel `i18nMsg` vit dans un commentaire
      HTML, et **zéro** commentaire HTML vit dans un bloc `<script>` — l'extension ne peut donc ni
      retirer un site compté, ni toucher un corps de fonction candidat. `sitesTotal` ne bouge pas.
- [x] **T4 — La garde**, sa borne exacte, son allowlist — AC1, AC3, AC4. ⚠️ **Avant les correctifs**,
      et sa sortie rouge sur les cinq sites du patron est collée au Record — AC2.
- [x] **T5 — `payment-batches`** : `paymentBatchStatusLabel` (3 cas) et `failedItemLabel` (6 codes) — AC5, AC7.
- [x] **T6 — `credit-notes`** : `creditNoteStatusLabel` (3 cas) — AC5, AC7.
- [x] **T7 — `invoices`** : `statusLabel` de la liste (site 7, **ferme #255**) et de la fiche (site 5) — AC5.
- [x] **T8 — Correctifs hors portée de la garde** : `settings` (site 4), la barre de navigation
      (site 6), le titre de dialogue (site 8) — **AC5 ET AC7**. ⚠️ **T8 est la SEULE tâche à toucher
      des libellés que des tests E2E ciblent par leur texte** — le site 6 (entrées de menu) et le
      site 8 (`fiscal-years.spec.ts:270`, `getByRole('dialog', { name: 'Valider la facture' })`).
      **Basculer ces sélecteurs sur `data-testid` AVANT de changer les libellés**, jamais après :
      dans l'autre ordre on découvre un rouge et la réparation la moins coûteuse est d'y réécrire la
      chaîne traduite — ce qui remet le verrou en place sous couvert de correctif.
- [x] **T9 — Glossaire** : promotion des termes tranchés — AC6.
- [x] **T10 — Recompter et déclarer `sitesTotal`** — AC8.
- [x] **T10-bis — Éprouver la garde par MUTATION**, après T5-T8 — AC2-bis. ⚠️ **Muter un des CINQ
      sites du patron** (1, 2, 3, 5, 7) : muter un site hors portée (4, 6, 8) ne ferait rien rougir
      **par construction**, et se lirait à tort comme un échec de la garde.
- [x] **T10-ter — Confronter le relevé de la garde au tableau** — AC5-bis. Tout site remonté hors
      des huit est corrigé ou tracé en issue, **jamais exempté**. À faire dès le premier run de T4.
- [x] **T11 — Retirer les clés `nav-*` du périmètre de la 23-4** *(l'epic en prévoyait 4 ; cette
      story en couvre 9)* — **AC5** : sans ce retrait, la 23-4 refait le travail ou entre en conflit.
- [x] **T12 — Gates complets, E2E comprise** — AC9.

## Dev Notes

### Les huit sites — vérifiés jusqu'au site de RENDU

⚠️ **Chaque site a été vérifié jusqu'à l'écran** : une fonction jamais appelée ne serait pas un
défaut. Les appelants sont donnés — ne pas les redécouvrir.

| # | définition *(motif de grep)* | rendu | en-tête voisin traduit ? | verrou de test |
|---|---|---|---|---|
| 1 | `payment-batch-helpers.ts:20-31` `paymentBatchStatusLabel` | `payment-batches/+page.svelte:234`, `[id]:84` | ⏳ **non** — `payment-batches-col-status` **à l'allowlist** ; la 23-4 l'activera | ⚠️ `payment-batch-helpers.test.ts:20-22` (`toBe`) |
| 2 | `payment-batch-helpers.ts:34-51` `failedItemLabel` | `payment-batches/+page.svelte:200` | idem | ⚠️ **`payment-batch-helpers.test.ts:28-29` (`toContain`)** — plus insidieux qu'un `toBe` : il survit à une traduction partielle |
| 3 | `credit-note-helpers.ts:23-34` `creditNoteStatusLabel` | `credit-notes/+page.svelte:65`, `[id]:178` | ⏳ **non** — `credit-notes-col-status` **à l'allowlist** ; la 23-5 l'activera | ⚠️ `credit-note-helpers.test.ts:22-24` (`toBe`) |
| 4 | `settings/+page.svelte:184-185` *(motif : `data.company.orgType`)* | la même ligne | ✅ `settings-field-org-type` **existe et est traduit** | non |
| 5 | `invoices/[id]/+page.svelte:602-606` `statusLabel` | `:755` | ❌ **sans objet** — badge de fiche, pas de colonne | non |
| 6 | `+layout.svelte:57-150` *(motif : `label: '`)* — **3 groupes + 6 entrées = 9 libellés** | `:340` `<span>{group.label}</span>` | — *(chrome permanente)* | non — ⚠️ **mais voir le piège E2E** |
| 7 | `invoices/+page.svelte:251` `statusLabel` | `:399` | ⏳ **non** — c'est l'objet même de #255 | non |
| 8 | `invoices/[id]/+page.svelte:949` *(motif : `<Dialog.Title>Valider la facture`)* | le dialogue de validation | — | ⚠️ `fiscal-years.spec.ts:270` cible ce dialogue **par son nom** |

**Le site 4 est le plus embarrassant à l'usage, et le seul défaut ACTIF aujourd'hui** : l'utilisateur
choisit son type d'organisation en langue localisée pendant l'onboarding (« Indépendant », « PME »),
puis retrouve dans les réglages le **code brut du backend** — `Independant`, `Association`, `Pme` —
dans **toutes** les langues, français compris. ⚠️ Les clés `onboarding-org-*` existent déjà et sont
câblées côté onboarding : **les réutiliser, ne pas en créer**. C'est légitime — vocabulaire
transverse, et `routes/**` est hors du périmètre du lint d'appartenance.

**Le site 6 est le plus visible** : la barre de navigation est sur **chaque page**. ⚠️ Paradoxe à
signaler dans la PR : `/reconciliation` et `/reports` **sont déjà entièrement traduites**, mais
l'entrée de menu qui y mène reste en français.
⚠️ **Correction de fait** : c'est le **type de groupe**, déclaré *inline* dans le `Array<{…}>` de
`:57`, qui ne porte que `label: string`. Le type **`NavItem` (`:17-19`) est déjà correct** — il a la
variante `{ i18nKey, fallback, href }`, employée par 20 entrées et consommée par `getItemLabel:152`.
Les 6 entrées en dur n'ont donc **qu'à changer de variante** ; seul le type de groupe doit gagner une
paire `i18nKey`/`fallback` et un `getGroupLabel` symétrique.

**Le site 7 ferme l'issue #255** — trois lignes, et les clés `invoice-status-{draft,validated,cancelled}`
**existent déjà, traduites dans les quatre locales**. Coût quasi nul. ⚠️ **Il est dans le périmètre
précisément parce que la garde rougira dessus** : l'exempter reviendrait à inscrire un défaut avéré
dans une liste réservée aux cas légitimes. Le site 8, lui, déborde #255 (qui ne vise que la liste) —
le **dire dans la PR**, ne pas modifier l'issue.

### Les termes : ce qui est attesté, et ce qui demande ton arbitrage

⚠️ **Relever avant d'inventer.** Les valeurs seront figées dans quatre catalogues et promues en
partie A.

| terme | statut | équivalence |
|---|---|---|
| `Brouillon` | ✅ **attesté** | `invoice-status-draft` → `Entwurf` / `Bozza` / `Draft` |
| `Annulé(e)` | ✅ **attesté** | `invoice-status-cancelled` → `Storniert` / `Annullata` / `Cancelled`. ⚠️ **Accord** : un *lot* est masculin, une *facture* féminine |
| `Validée` | ✅ **attesté** | `invoice-status-validated` |
| `Ouverte` / `Payée` | ✅ **partie A** | `offen`/`aperta`/`open` · `bezahlt`/`pagata`/`paid` |
| ~~`Généré`~~ → **`Créé`** | ⚠️ **LE FRANÇAIS CHANGE** — arbitrage de Guy, 2026-08-20 | `Erstellt` / `Creato` / `Created` — **relevé** sur `imported-supplier-invoices-completed` |
| **`Confirmé`** | tranché | `Bestätigt` / `Confermato` / `Confirmed` — **relevé** sur `demo-reset-confirm-ok` |
| **`Émis`** *(fém. : une note de crédit)* | tranché. ⚠️ Ne pas le confondre avec « validé », qui est l'acte d'immuabilité | `ausgestellt` / `emessa` / `issued` — **relevé** sur `credit-note-revenue-account-archived` |
| **les 6 codes de `failedItemLabel`** | tranchés — 4 **relevés**, 2 décidés | `SUPPLIER_INVOICE_NOT_FOUND` ← `reminders-error-invoice-not-found` · `INVALID_IBAN` / `INVALID_QR_IBAN` ← `imported-supplier-invoices-error-invalid-iban`, **sans** « du créancier » · `SUPPLIER_INVOICE_NOT_OPEN` ← *ouverte*, **partie A** · `NO_PAYMENT_COORDINATES` → **`Keine Zahlungsverbindung`** (terme bancaire, **pas** le calque `Zahlungskoordinaten`) · `ALREADY_IN_GENERATED_BATCH` → cf. ⚠️ ci-dessous. ⚠️ **Messages techniques : ils ne montent PAS en partie A** — le glossaire porte des termes métier, pas des libellés d'erreur |

⚠️ **« Généré » → « Créé » entraîne un SECOND changement de français, et il faut le voir venir.**
`ALREADY_IN_GENERATED_BATCH` dit aujourd'hui « Déjà dans un lot **en cours** » — une troisième
formulation pour un statut qui s'appelle `generated` dans le code et s'affichera « Créé » à l'écran.
**L'utilisateur ne peut pas faire le lien entre les deux.** Le message devient donc « Déjà dans un
lot **créé** », qui reprend le mot exact de la colonne Statut. ⚠️ **Ce n'est pas une extension de
périmètre** : c'est la conséquence directe de l'arbitrage, et la laisser de côté rendrait le
correctif incohérent avec lui-même.

⚠️ **Le verbe « créer » devient uniforme dans le domaine** : `Créer une facture` (tranché la veille,
usage Bexio), `Facture créée.`, et maintenant `Créé` pour un lot. Les trois cibles suivent le même
verbe — `erstellen`/`erstellt`, `creare`/`creata`, `create`/`created`. C'est le bénéfice caché de
l'arbitrage : un mot de moins à retenir pour l'utilisateur, et une équivalence de plus au relevé.

### Sur quoi la garde s'appuie — ce qui existe et ce qui manque

`i18n-literal-reader.js` expose `readLiteral:41`, `masquerCommentaires:258`, `findRelays:340`,
`findCallSites:420`, `readFallback:460` ; `i18n-harvest.js` expose `dansLePerimetreDeFichier:31`.

⚠️ **Ce lecteur a été durci par sept passes de revue cumulées** (gabarits à apostrophes, échappements
Fluent, commentaires, relais). **Ne pas en écrire un second.** Mais deux briques manquent, et c'est
l'objet de T2 et T3 :

- **`corpsDeFonction` existe (`:374`) mais n'est PAS exportée.** C'est l'appariement d'accolades dont
  la garde a besoin, et son doc-comment raconte trois passes de durcissement. **L'exporter coûte une
  ligne ; la réécrire coûte 40 lignes et trois régressions déjà payées.**
- **`masquerCommentaires` ne masque PAS les `<!-- -->`** — vérifié, elle ne traite que `//` et
  `/* */`. Sans T3, la garde récolte **21 faux positifs de prose** dans les `.svelte`, c'est-à-dire
  là où vivent les sites 4, 6 et 8.

⚠️ **`dansLePerimetreDeFichier` accepte bien `src/routes/**`** (elle ne filtre que sur l'extension et
`.test.`) — contrairement à `lint-i18n-ownership`, qui ne balaie que `src/lib/features/`. Le
réutiliser est donc sûr. **Montage complet à copier** : `i18n-un-repli-par-cle.test.ts` (117 lignes).

### Faux amis : les quatre entrées d'allowlist connues d'avance

⚠️ **Le patron y est CORRECT** — ce sont des tables de replis passées en 2ᵉ argument d'`i18nMsg`.
⚠️ **Ne pas confondre avec les trois classes de littéraux ci-dessous** : ici ce sont des **structures
nommées**, qui vont à l'allowlist entrée par entrée ; là ce sont des **formes de littéral**, qui
doivent être des **critères** — une forme ne s'énumère pas, sinon la liste enfle sans fin.

| entrée | fichier | consommée en |
|---|---|---|
| `LABEL_FALLBACK` | `lib/features/invoices/PaymentStatusBadge.svelte:18` | `:24`, 2ᵉ arg |
| `FILTER_FALLBACK_FR` | `routes/(app)/invoices/due-dates/+page.svelte:42` | 2ᵉ arg |
| `TYPE_FALLBACK` | `routes/(app)/accounts/+page.svelte` | `:37`, 2ᵉ arg |
| `ROLE_FALLBACK` | `routes/(app)/accounts/+page.svelte:40` | `:62` *(motif : `ROLE_FALLBACK[r] ??`)*, 2ᵉ arg |

⚠️ **`readFallback` rend `null` sur ces quatre-là** (le 2ᵉ argument est `TABLE[clé]`, pas un
littéral) : l'outillage ne sait pas les reconnaître seul. **Critère de discrimination à tenir** — *un
`Record` de littéraux est légitime si et seulement si chacun de ses accès apparaît en 2ᵉ argument
d'`i18nMsg` ou d'un relais.* Sans ce critère écrit, le dev exemptera au nom, et `navGroups` (site 6,
même forme) deviendrait légitime par accident.

**Trois classes de littéraux légitimes doivent être des CRITÈRES, jamais des entrées d'allowlist** —
sinon la liste enfle à chaque cas et perd son sens :

1. **le cadratin et assimilés** — `'—'`, `'N/A'`. ⚠️ **DEUX** occurrences dans `account-label.ts`
   (`:35` **et** `:66`), pas une : ce sont deux fonctions jumelles à 31 lignes d'écart ;
2. **la chaîne vide** `''` — `settings/opening-balances/+page.svelte:131`. ⚠️ **Ce cas n'était prévu
   nulle part** : une chaîne vide n'est ni du français ni un cadratin ;
3. **les gabarits interpolés** — un `return `${a} — ${b}`` n'est pas un libellé en dur. *(Le lecteur
   les distingue déjà : `readLiteral` rend `kind: 'template'`.)*

### Pièges du dépôt qui s'appliquent ici

- ⚠️ **T8 change le texte de CHAQUE entrée de menu.** Inventorier les sélecteurs E2E qui s'appuient
  sur ce texte **avant** d'appliquer T8, et les basculer sur `data-testid`. Ne **jamais** figer un
  sélecteur sur un libellé traduit.
- ⚠️ **Le site 8 est ciblé par `fiscal-years.spec.ts:270`** via `getByRole('dialog', { name: 'Valider
  la facture' })`. Le toucher casse ce test.
- ⚠️ **Le gate E2E exige `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`** — sans eux, des tests échouent
  pour une raison qui ne ressemble pas à un problème de configuration. Cf. `docs/testing.md`.
- ⚠️ **Le namespace doit correspondre au dossier** pour tout module de `src/lib/features/`.
- ⚠️ **Ne déclarer que ce qui a tourné.**
- Greps de contrôle pour AC7 — **les deux** :
  `grep -rnE "toBe\('[A-ZÀ-Ü]|toContain\('[a-zà-ü]" frontend/src/lib/features/*/*helpers.test.ts`
  `grep -rnE "getByRole\(.*name: '[A-ZÀ-Ü]|getByText\('[A-ZÀ-Ü]" frontend/tests/e2e/`
  ⚠️ **Le premier rend ~6 fois plus de bruit que de signal** (mesuré : 3 vrais verrous, ~18 faux —
  formats de n° IDE, paramètres de requête, et les assertions **multi-locales** du site corrigé en
  23-3, qui sont légitimes). Trier, ne pas se décourager.

### Ce qui a été balayé et trouvé PROPRE — ne pas re-balayer

`reconciliation`, `reports`, `onboarding`, `contacts`, `accounts`, `vat-rates`, `api-keys`,
`opening-balances`, `reminders`, `imported-supplier-invoices`, `journal-entries`, `bank-accounts`,
`fiscal-years`, `products`, `due-dates`. *(⚠️ **Correction de portée, passe 2** : les « 32 déclarations » sont le total **plein-arbre**, sites
fautifs compris — pas un décompte restreint à ces 15 domaines. **AC3 se recompte par un balayage
plein `src/` au moment de T4**, jamais repris d'ici. Quatre fichiers candidats hors de cette liste ont
été vérifiés conformes en passe 2 : `settings/email-templates`, `invoices/reminders`, le dispatcher
`getItemLabel` du layout, et `supplier-invoices/import`.)*

### References

- Angle mort : [issue #255](https://github.com/guycorbaz/kesh/issues/255)
- Précédent : `_bmad-output/implementation-artifacts/23-3-supplier-invoices.md`, § *Review Findings —
  passe 4* (le défaut d'origine) et § *passe 5* (la chasse au patron)
- Glossaire : `docs/i18n-glossaire.md` — partie A non négociable en rollout
- Outillage : `frontend/src/lib/shared/i18n-literal-reader.js`, `i18n-harvest.js`
- Montage à copier : `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts`
- Correctif de référence : `frontend/src/lib/features/supplier-invoices/supplier-invoice-helpers.ts`
  et son test, qui emprunte le chemin réel du dictionnaire

## Reprise — où en est le dossier, et par quoi commencer

*(Écrit à la fin de la séance du 2026-08-20, à la demande de Guy.)*

### L'état, en trois lignes

**`main` porte trois stories de l'epic 23** — le socle (23-1a/1b), la parité (23-2, qui a fermé #283),
et la 23-3 supplier-invoices (#325, `046efa51`). L'allowlist de dette est à **166 clés**. **#316 reste
ouverte**, et doit le rester jusqu'à la fin de l'epic.

**Cette story (23-3b) est en `ready-for-dev`**, spec passée par **trois passes de `validate`**
(`2C/6H/6M/3L` → `2C/4H/3M/6L` → `1C/0H/1M/2L`) et **T1 rendue**. ⚠️ **Six commits attendent sur la
branche, qui n'a JAMAIS été poussée** — le travail est dans `git`, pas sur le distant.

### Par quoi reprendre, dans l'ordre

1. **Décider si une passe 4 de `validate` a lieu.** La règle la prescrit *(il restait 1 CRITICAL en
   passe 3)*, le cycle appellerait **Opus**. ⚠️ **Mon avis, qui n'est pas une certitude** : la passe 3
   n'a plus rien trouvé sur les **faits** (42 vérifications, zéro finding) ni sur la **conception**, et
   son unique CRITICAL portait sur un lien manquant entre une tâche et un AC. Le rendement décroît.
2. **`bmad-dev-story`** sur cette spec. Rien ne bloque plus.
3. **Pousser et ouvrir la PR** en `refs #316`.
4. **Puis seulement la 23-4** — ⚠️ **jamais avant**, sinon elle fige le défaut : `payment-batches` et
   `credit-notes` sont ses domaines, et leurs valeurs de statut sont encore en français en dur.

### Ce qui a été tranché aujourd'hui, et qui se perdrait

- ⚠️ **Pas de relecture par un locuteur natif** — *« le jour où un germanophone ou un italophone
  utilisera Kesh, il annoncera les erreurs et nous prendrons les mesures à ce moment-là »*. Consigné
  au **préambule du glossaire**, avec ce que cela impose en retour : **le relevé devient la seule
  discipline qui borne la dérive**.
- **`imported-supplier-invoices-save` = « Créer une facture »** *(usage Bexio)*. Le libellé précédent
  disait « Valider la facture » alors que le code **crée** la pièce — et `invoice-validate-confirm-title`
  portait **déjà** ce texte pour la validation comptable, qui rend une facture immuable. **Deux actes
  distincts, dont un irréversible, sous un seul libellé français.**
- **Le statut de lot : « Généré » → « Créé »**, et le verbe « créer » devient **uniforme** dans le
  domaine. Cf. § *Les termes*.
- **Les six codes d'échec ne montent PAS en partie A** du glossaire : libellés d'erreur, pas termes
  métier.

### Ce qui reste en suspens

- **Deux décisions mineures sur la 23-3, déjà mergée** : renommer `-col-qty` / `-col-vat` en `-line-*`
  *(elles titrent des colonnes de LIGNES dans une famille qui sert la LISTE)*, et l'asymétrie
  « v0.4 » — le français annonce une limitation datée là où les trois cibles la donnent pour
  définitive.
- **Quatre bugs applicatifs reportés** par la revue de la 23-3, sans rapport avec la traduction, dont
  un `catch` sans branche `else` qui fait afficher **« Aucune facture fournisseur enregistrée »**
  quand le backend est injoignable. Un comptable en conclut que ses pièces ont disparu.
- **KF-041 (#323)** et **KF-042 (#324)**, ouvertes aujourd'hui : deux homonymies qu'aucune garde ne
  peut voir, hors périmètre de tout rollout en cours.

### Les pièges à ne pas redécouvrir

- ⚠️ **Le gate E2E exige `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`.** Sans eux, des tests échouent
  d'une façon qui **ne ressemble pas** à un problème de configuration — `docs/testing.md` a été
  corrigé aujourd'hui, avec la mesure.
- ⚠️ **`pkill -f "target/debug/kesh-api"` tue le shell qui le porte** — son propre motif figure dans
  sa ligne de commande. Utiliser `pkill -x kesh-api`.
- ⚠️ **Sur ce dossier, dix passes sur onze ont trouvé une régression du correctif précédent.** Ce
  n'est pas une figure de style : c'est le mode d'échec dominant, et il vise désormais les **comptes
  rendus** plus souvent que le code.

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]` (Claude Opus 5, 1M context) — implémentation du 2026-08-21.

### Debug Log References

#### T4 — la garde AVANT tout correctif : sortie BRUTE du run (AC2)

⚠️ **Premier run : la garde était MUETTE, et le gate VERT.** `accoladeDeCorps` cherchait
l'accolade du corps juste après la fermeture de la signature — or presque toutes les
déclarations du dépôt s'écrivent `function xLabel(…)\u200b: string {`, si bien que le
caractère suivant est `:` et non `{`. Elle rendait `null` sur toutes, le relevé était vide,
les violations aussi, et les deux tests du dépôt passaient. **Ce sont les cas synthétiques
qui l'ont attrapée, jamais le relevé** — c'est leur raison d'être : un test muet ne se
signale pas lui-même, et la § *Test Locally First* du dépôt en compte déjà trois précédents
(`backfill_skips_archived_accounts`, les mutations de 16-2a, `authedApiContext`).

Second run, après traversée de l'annotation de type de retour — **18 violations sur 5
fonctions**, qui sont exactement les sites 1, 2, 3, 5 et 7 du tableau :

```
 FAIL  src/lib/shared/i18n-libelle-en-dur.test.ts > libellés en dur — l'angle mort #255 > aucune fonction de libellé ne retourne un littéral sans passer par i18nMsg
AssertionError: expected [ …(18) ] to deeply equal []

+   "src/lib/features/credit-notes/credit-note-helpers.ts:26 creditNoteStatusLabel() → « Brouillon »",
+   "src/lib/features/credit-notes/credit-note-helpers.ts:28 creditNoteStatusLabel() → « Émis »",
+   "src/lib/features/credit-notes/credit-note-helpers.ts:30 creditNoteStatusLabel() → « Annulé »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:23 paymentBatchStatusLabel() → « Généré »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:25 paymentBatchStatusLabel() → « Confirmé »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:27 paymentBatchStatusLabel() → « Annulé »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:37 failedItemLabel() → « Facture introuvable »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:39 failedItemLabel() → « Facture non ouverte »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:41 failedItemLabel() → « Pas de coordonnées de paiement (IBAN/QR-IBAN) »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:43 failedItemLabel() → « Déjà dans un lot en cours »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:45 failedItemLabel() → « IBAN invalide »",
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:47 failedItemLabel() → « QR-IBAN invalide »",
+   "src/routes/(app)/invoices/+page.svelte:254 statusLabel() → « Brouillon »",
+   "src/routes/(app)/invoices/+page.svelte:256 statusLabel() → « Validée »",
+   "src/routes/(app)/invoices/+page.svelte:258 statusLabel() → « Annulée »",
+   "src/routes/(app)/invoices/[id]/+page.svelte:603 statusLabel() → « Brouillon »",
+   "src/routes/(app)/invoices/[id]/+page.svelte:604 statusLabel() → « Validée »",
+   "src/routes/(app)/invoices/[id]/+page.svelte:605 statusLabel() → « Annulée »",
```

#### T4 / AC3 — la ventilation, recomptée depuis le relevé

**39 déclarations candidates**, et la somme égale le total :

| classe | nombre |
|---|---|
| conformes — aucun retour littéral | **30** |
| écartées par CRITÈRE — cadratin ou chaîne vide | **4** |
| en violation | **5** |
| **total** | **39** |

Les **23 hits bruts** (littéraux retournés) se ventilent en **18 violations** et **5 écartés**
— `invoiceRevenueAccountLabel` en porte deux à elle seule.

⚠️ **La spec annonçait HUIT fonctions au balayage ; il y en a NEUF.** Les trois candidats
à écarter qu'elle nommait sont bien là — `invoiceRevenueAccountLabel` (`—`),
`creditNoteRevenueAccountLabel` (`—`), `roleLabel` (chaîne vide) —, mais un **quatrième**
s'y ajoute, de la même classe que le troisième :
`journal-entries/AccountAutocomplete.svelte:168 displayLabel() → « »`. Il est **écarté par le
critère de la chaîne vide**, pas par une exemption ; ce n'est donc pas un site fautif hors
tableau au sens d'AC5-bis, et **aucune issue n'est ouverte** — il n'y a pas de défaut. Le
chiffre écrit ici est le mesuré, pas celui de la spec.

#### T3 — les deux conditions qui garantissent que `sitesTotal` ne bouge pas

Vérifiées au dépôt, non supposées : **zéro** appel `i18nMsg` vit dans un commentaire de
balisage, et **zéro** commentaire de balisage vit dans un bloc `<script>`. L'extension ne
peut donc ni retirer un site compté ni toucher un corps de fonction candidat.

⚠️ **Le « 21 blocs » de la spec ne se retrouve pas** : le dépôt porte **235** commentaires de
balisage, dont **49** contiennent au moins un littéral lisible. Le nombre inscrit dans le
code est le mesuré.

#### T10-bis — la garde éprouvée par MUTATION (AC2-bis)

Mutation appliquée sur un des **cinq** sites du patron — `paymentBatchStatusLabel`, dont le
premier `return` redevient `'Créé'` sans `i18nMsg`. *(Muter un site hors portée — 4, 6 ou 8 —
n'aurait rien fait rougir **par construction**, et se serait lu à tort comme un échec.)*

```
     × aucune fonction de libellé ne retourne un littéral sans passer par i18nMsg 87ms
     × la ventilation se somme au total — conformes + écartées par critère + en violation 31ms
+   "src/lib/features/payment-batches/payment-batch-helpers.ts:42 paymentBatchStatusLabel() → « Créé »",
```

Elle tue **deux** tests, pas un : la garde elle-même et l'invariant de ventilation. Mutation
annulée, 16/16 verts.

#### T10-ter — le relevé confronté au tableau (AC5-bis)

Les **5 fonctions en violation** relevées par la garde sont **exactement** les sites 1, 2, 3, 5
et 7 du tableau — aucun site fautif hors tableau, donc **aucune issue à ouvrir**. Le seul écart
est le **quatrième candidat écarté par critère** (`displayLabel`, chaîne vide) décrit plus haut :
il n'est pas un site fautif, et il n'entre pas à l'allowlist — c'est précisément la distinction
qu'AC5-bis protège.

#### T11 — ⚠️ la tâche reposait sur une prémisse FAUSSE, vérifiée au sol

T11 prescrivait de « retirer les clés `nav-*` du périmètre de la 23-4 », en supposant que les
**9** libellés de navigation de cette story recouvraient les **4** `nav-*` que l'epic lui
attribue. **Il n'y a aucun recouvrement** :

| ensemble | clés | nature du défaut |
|---|---|---|
| les 4 de la 23-4 | `nav-credit-notes`, `nav-email-templates`, `nav-projects`, `nav-supplier-invoices-import` | clés **déjà câblées**, absentes des catalogues — dette de traduction. **Vérifié : 0/4 catalogues, inchangé par cette story** |
| les 9 de la 23-3b | 3 groupes + 6 entrées | **français en dur**, jamais routé vers `i18nMsg` |

**La 23-4 garde donc ses 93 clés.** Le geste utile n'était pas un retrait mais une **inscription** :
le plan d'epic porte désormais une ligne 23-3b et dit pourquoi les deux ensembles sont disjoints,
faute de quoi une story suivante aurait cherché à reprendre ces neuf libellés.

⚠️ **Trois des neuf étaient DÉJÀ TRADUITS dans les quatre catalogues, et jamais câblés** —
`nav-quotidien`, `nav-mensuel`, `nav-administration`. La traduction existait ; c'est le fil qui
manquait.

#### AC8 — `sitesTotal` recompté, et l'écart ventilé

**1502 → 1525**, soit **+23**, et chacun s'explique : 9 pour `payment-batch-helpers` (3 statuts +
6 codes), 3 pour `credit-note-helpers`, 3 + 4 pour les deux écrans de factures (dont le titre du
dialogue), 3 pour le type d'organisation, 1 pour `getGroupLabel`. Les sites **non résolus**
passent de 33 à **34** : `getGroupLabel` est le symétrique exact de `getItemLabel`, déjà de la
liste — la clé est portée par la donnée, pas par le site d'appel.

⚠️ **La borne a rougi la première, dans les deux cas.** C'est son travail.

#### ⚠️ L'allowlist de dette n'a PAS bougé — 166 avant, 166 après

Aucune des 18 clés neuves n'y figurait, et c'est **le fait le plus instructif de la story** :
cette dette-là n'était comptée nulle part. Tout l'appareil de mesure de l'epic — moissonneur,
allowlist, les trois gardes — ne relève que les sites `i18nMsg`. L'angle mort #255, en chiffres.

#### T12 / AC9 — les gates, et le DIFFÉRENTIEL E2E

Base de gate remise à zéro avant **chaque** run complet, inconditionnellement.

| gate | résultat |
|---|---|
| backend `scripts/test-fast.sh --ci` | **2219 tests, 2219 passés**, 4 skippés — fmt et clippy verts |
| `cargo test -p kesh-i18n` | **29/29**, dont `parity_between_locales` |
| frontend `npm run check` | **0 erreur** (27 warnings, tous préexistants) |
| `lint-i18n-ownership` | **PASS** |
| `npm run test:unit` | **687 tests, 72 fichiers** |
| `npm run build` | vert |
| **E2E branche** | **183 passés · 38 échoués · 19 skippés** |
| **E2E `main`** *(référence)* | **183 passés · 38 échoués · 19 skippés** |
| **différentiel** | ⚠️ **NUL** — listes d'échecs identiques |

⚠️ **Le différentiel n'était pas une formalité : le premier run rendait 182 contre 183.** Un
seul test d'écart, et il portait une régression réelle — cf. ci-dessous. Sans la mesure contre
`main`, l'écart se serait lu comme du bruit d'environnement, ce que la doctrine du dépôt
(« une quarantaine d'échecs sur `main` aussi ») invitait précisément à croire.

#### ⚠️ AC7 — il y avait CINQ verrous, pas quatre, et un SIXIÈME dormant

La spec en listait quatre en écrivant que sa liste n'était pas close. Elle avait raison.

**Le cinquième** — `payment-batches.spec.ts:146`, `toContainText(/Généré/i)`. ⚠️ **Aucun des
deux greps de contrôle d'AC7 ne pouvait l'atteindre** : ce n'est ni un `getByText` ni un
`getByRole({ name })`, mais une assertion de **contenu** posée sur un sélecteur **déjà stable**.
Le `data-testid` était en place et ne protégeait rien — c'est l'ASSERTION qui lisait la chaîne
traduite. Corrigé par `data-status={batch.status}` sur le composant, l'assertion portant
désormais sur le **code** (`generated`, `confirmed`).

**Le sixième** — `has-text("Administration")`, **5 occurrences dans 3 fichiers**, trouvé par le
grep du **symptôme** après le correctif du cinquième. ⚠️ **Il est vert aujourd'hui, et c'est ce
qui le rend dangereux** : le libellé est identique en français, en allemand et en anglais, si
bien qu'il ne casserait **qu'en italien** (`Amministrazione`). Un sélecteur vert dans trois
langues sur quatre ne se signale jamais. Basculé sur le `data-testid` du groupe, qui existait
déjà — aucun changement de production.

⚠️ **Un piège de comparaison, à ne pas reproduire** : `fiscal-years.spec.ts:291` contre `:288`
est **le même test**, décalé de trois lignes par un commentaire ajouté ici. Comparer les échecs
par `fichier:ligne` fabrique des régressions imaginaires — la comparaison finale est faite sur
`fichier › titre`.

**Verrous laissés en place, hors périmètre** *(libellés que cette story ne touche pas)* :
`journal-entries.spec.ts:66` (`/Écritures/` — le TITRE de page, non l'entrée de menu),
`homepage-settings.spec.ts:56` (`Utilisateurs`, un `<h2>` de `/settings`) et
`onboarding.spec.ts:161` (`Indépendant`, bouton d'onboarding). Les trois portent sur des clés
traduites **avant** cette story ; les corriger reviendrait à élargir le périmètre.

### Completion Notes List

- **La garde vaut plus que les correctifs**, et c'est elle qui a coûté le plus : son premier run
  était **vert et muet**. Les correctifs, eux, sont mécaniques une fois les termes relevés.
- **Aucun terme n'a été inventé.** Les valeurs viennent de clés attestantes nommées une à une ;
  les deux seules qui ne l'étaient pas — les entrées de menu « Écritures »/« Rapports » et
  « Importer » — ont été portées à Guy, qui a tranché le 2026-08-21. ⚠️ Après coup, « Importer des
  relevés » s'est révélé **entièrement relevé** sur `homepage-bank-empty-guided` dans les quatre
  locales : la question méritait d'être posée, la réponse était déjà au catalogue.
- **Glossaire : partie A 65 → 70**, recomptée depuis le tableau. Trois promotions exigées par AC6
  (`créé`, `confirmé`, `émis`) et **deux relevées en chemin** (`annulé(e)`, `relevé (bancaire)`) —
  toutes deux attestées au catalogue et pourtant absentes de la partie A, donc libres de dériver.
  Les **six codes d'échec n'y montent pas** : libellés d'erreur, pas termes métier.
- **Deux chiffres de la spec ne se retrouvent pas au dépôt**, et les mesurés ont été retenus :
  « 21 blocs `<!-- -->` » → **235** (dont 49 porteurs d'un littéral), et « le balayage rend HUIT
  hits » → **neuf fonctions**.

### File List

**Créé**
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` — la garde

**Modifié — outillage partagé**
- `frontend/src/lib/shared/i18n-literal-reader.js` — `corpsDeFonction` exportée (T2), `masquerCommentaires` étendue aux `<!-- -->` (T3)
- `frontend/src/lib/shared/i18n-literal-reader.test.ts` — 8 cas neufs
- `frontend/src/lib/shared/i18n-keys.test.ts` — bornes `sitesTotal` 1525 / non résolus 34

**Modifié — les huit sites**
- `frontend/src/lib/features/payment-batches/payment-batch-helpers.ts` *(+ son test)* — sites 1 et 2
- `frontend/src/lib/features/credit-notes/credit-note-helpers.ts` *(+ son test)* — site 3
- `frontend/src/routes/(app)/settings/+page.svelte` — site 4
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` — sites 5 et 8
- `frontend/src/routes/(app)/+layout.svelte` — site 6
- `frontend/src/routes/(app)/invoices/+page.svelte` — site 7 *(ferme #255)*
- `frontend/tests/e2e/fiscal-years.spec.ts` — sélecteur basculé sur `data-testid`

**Modifié — catalogues et documentation**
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — 18 clés × 4 locales
- `docs/i18n-glossaire.md` — partie A 65 → 70
- `_bmad-output/planning-artifacts/epic-23-dette-i18n.md` — ligne 23-3b, périmètre de la 23-4 clarifié
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-21 | **implémentation** (`bmad-dev-story`, Opus 5) | **T1-T12 livrées.** ⚠️ **La garde — le livrable central — était VERTE ET MUETTE à son premier run** : `accoladeDeCorps` cherchait l'accolade du corps juste après la signature, alors que presque toutes les déclarations s'écrivent `(…): string {`. Elle rendait `null` sur toutes, relevait zéro fonction et n'avait rien à signaler. **Attrapée par les cas synthétiques, jamais par le relevé du dépôt** — c'est leur raison d'être. Second run : **18 violations sur les 5 sites du patron**, sortie brute collée. Ventilation **41 = 37 + 4 + 0**, invariant sous test. Garde **éprouvée par mutation** : elle tue deux tests. **18 clés × 4 locales**, parité stricte à 1427. ⚠️ **TROIS chiffres de la spec ne se retrouvent pas au dépôt** et les mesurés ont été retenus : « 21 blocs `<!-- -->` » → **235**, « HUIT hits » → **neuf fonctions**, « QUATRE verrous » → **cinq**, plus un sixième dormant. ⚠️ **T11 reposait sur une prémisse FAUSSE** — aucun recouvrement entre les 9 libellés d'ici et les 4 `nav-*` de la 23-4 ; le geste utile était une inscription au plan d'epic, pas un retrait. ⚠️ **L'allowlist n'a pas bougé (166 → 166)** : cette dette n'était comptée nulle part, l'angle mort #255 en chiffres. **Différentiel E2E branche ↔ `main` NUL** (183/38/19 des deux côtés), après correction d'une régression réelle que le premier différentiel (182 contre 183) avait révélée. Glossaire partie A **65 → 70**. |
| 2026-08-20 | **passe 3** de `validate` | **1 CRITICAL · 0 HIGH · 1 MEDIUM · 2 LOW** (Haiku ×2, tâches délibérément mécaniques). ⚠️ **La lentille des FAITS rend ZÉRO finding** — 42 vérifications au `grep -nF`, tous les faits de la spec confirmés : après deux passes, le document ne ment plus sur le code. Et la lentille de COHÉRENCE rend **14 renvois « site N » sur 14 corrects** et **tous les décomptes cohérents** — la renumérotation tient. **Le CRITICAL est une couverture cassée, pas un fait faux** : `T8` touche le titre de dialogue, seul site dont un test E2E cible le libellé **par son texte**, et ne portait que `AC5` — jamais `AC7`. Un développeur aurait changé le libellé, vu `fiscal-years.spec.ts:270` rougir, et « réparé » en y réécrivant la chaîne : **le verrou remis en place sous couvert de correctif**. `T8` porte désormais les deux AC et impose de basculer le sélecteur sur `data-testid` **avant** de toucher au libellé. ⚠️ **Trois findings reclassés en LOW par la lentille elle-même**, qui conclut à chaque fois que le livrable final est cohérent — de l'archéologie de Change Log, pas des défauts. |
| 2026-08-20 | **passe 2** de `validate` | **2 CRITICAL · 4 HIGH · 3 MEDIUM · 6 LOW** (Sonnet ×2). ⚠️ **Les deux CRITICAL visent la RÉÉCRITURE de la passe 1** — dixième fois sur ce dossier qu'une passe trouve un défaut du correctif précédent. `AC9` renvoyait au **site 6** (la barre de navigation !) pour ce qui déborde #255, là où les Dev Notes disent **site 8** : l'erreur était **antérieure** à la renumérotation, qui ne l'a pas rattrapée parce que je n'ai corrigé que les renvois que je savais avoir décalés. Et le Change Log annonçait encore « Sites 5 → 7 » après la réinsertion de `credit-notes` — le tableau corrigé, son compte rendu non. ⚠️ **Le finding MESURÉ est le plus utile** : le balayage du patron réduit, **réellement exécuté**, rend **HUIT** hits et non cinq — cinq violations plus **trois candidats à écarter par critère**, dont un cas que rien ne prévoyait (`roleLabel` → **chaîne vide**). AC2 aurait échoué à la lettre. **Trois classes de littéraux légitimes** sont désormais des CRITÈRES et non des entrées d'allowlist, sans quoi la liste enfle et perd son sens. Autres corrections : une 5ᵉ forme hors de portée (`const xLabel = $derived.by`), la portée du « 32 » qui était plein-arbre et non par domaine, `corpsDeFonction` qui **ne suffit pas seule** (localiser la déclaration reste à écrire, avec le piège du type inline), `AC7` qui ignorait un **quatrième** verrou — un sélecteur E2E que ma propre section « Pièges » documentait deux paragraphes plus bas. **Un MEDIUM réfuté** : les deux usages de « 23-4 » désignent bien la même story, l'epic la définissant comme `settings` + `payment-batches` + `onboarding` + 4 `nav-*`. |
| 2026-08-20 | **passe 1** de `validate` | **2 CRITICAL · 6 HIGH · 6 MEDIUM · 3 LOW** (Opus ×2). ⚠️ **La spec ne survit pas à sa première passe, et deux findings la refondent.** **CRITICAL-1** : la branche avait été coupée de `main` **avant** le merge de la 23-3 — quatre références pointaient dans le vide et le défaut fondateur était encore dans l'arbre. Corrigé : #325 mergée, branche rebasée sur `046efa51`. **CRITICAL-2, mesuré** : « la garde rougit sur les cinq sites » était **irréalisable** — trois formes du défaut (valeur sans appel, tableau de données, nœud de markup) sont **hors de portée par construction**. La garde est donc **réduite à ce qu'elle peut prouver** et les trois sites restants passent en correctifs manuels **avec motif écrit**. L'élargissement « par le contenu » a été **chiffré** : 175 sites à trier pour rater quand même `Brouillon` et 7 des 9 libellés de nav — proscrit. **HIGH le plus embarrassant** : le tableau cochait ✅ « en-tête traduit » pour deux sites dont **les clés n'existent dans aucune locale** — elles sont à l'allowlist. Le défaut y est **latent**, pas actif ; ma propre prose le disait deux paragraphes plus haut. Autres faits corrigés : `masquerCommentaires` ne masque **pas** les `<!-- -->`, `corpsDeFonction` **n'est pas exportée**, un **3ᵉ** verrou de test (`toContain`), **quatre** tables de replis et non deux, doc-comment copié **trois** fois et non quatre, `NavItem` **déjà correct** (c'est le type de groupe qui manque). Sites 5 → **8** *(le tableau réécrit avait d'abord PERDU `credit-notes`, l'un des deux CRITICAL d'origine — réinséré avant la passe 2)*. |
| 2026-08-20 | création | spec initiale, à partir de la passe 5 de revue de la 23-3. |
