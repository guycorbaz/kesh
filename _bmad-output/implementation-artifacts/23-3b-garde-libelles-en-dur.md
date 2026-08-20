# Story 23.3b : la garde contre les libellés en dur, et les huit sites qu'elle révèle

Status: ready-for-dev

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
**quatrième** site porte le même patron **sans commentaire** (`invoices/[id]:602`). ⚠️ **Ce dernier
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
   run est collée dans le Dev Agent Record. ⚠️ **C'est la seule preuve qu'elle les voit** ; sans
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
6. **AC5 — les huit sites du tableau sont corrigés**, chacun avec ses clés dans les quatre locales.
7. **AC5-bis — tout site relevé par la garde et absent du tableau est soit corrigé ici, soit
   consigné dans le Dev Agent Record avec sa raison ET une issue GitHub** — jamais inscrit à
   l'allowlist. ⚠️ Le tableau est un **plancher**, pas un inventaire clos : la garde balaie tout
   `src/`, c'est sa raison d'être.
8. **AC6 — les termes non attestés sont tranchés PAR GUY, puis promus en partie A du glossaire** avec
   leur clé attestante. Portée : les trois statuts **et** les six codes de `failedItemLabel`.
9. **AC7 — tout test qui verrouille le français est corrigé.** Trois blocs connus au 2026-08-20 —
   ⚠️ **cette liste n'est pas close**, la balayer au grep donné ci-dessous.
10. **AC8 — `sitesTotal` d'`i18n-keys.test.ts` est recompté depuis la source et déclaré dans le Dev
    Agent Record**, jamais ajusté en silence. Base au départ : **1502**.
11. **AC9 — gates complets verts, E2E comprise**, PR en `refs #316`, et le corps de la PR mentionne
    que le site 6 déborde le périmètre de l'issue #255.

## Tasks

- [ ] **T1 — Arbitrage de Guy** sur les trois statuts non attestés **et** les six codes de
      `failedItemLabel` — AC6. **BLOQUANT.** ⚠️ **Conduite à tenir** : proposer les valeurs `de`/`it`/`en`
      dans le Dev Agent Record, puis **S'ARRÊTER et attendre**. Ne **jamais** inventer : ces valeurs
      seront figées dans quatre catalogues **et** promues en partie A du glossaire, zone déclarée
      « non négociable en rollout ». Précédents d'arbitrage : `analytique`, `bascule`, `fattura fornitore`.
- [ ] **T2 — Exporter `corpsDeFonction`** depuis `i18n-literal-reader.js` (+ son test) — elle existe
      (`:374`) mais **n'est pas exportée**, et c'est la seule brique manquante. AC1.
- [ ] **T3 — Étendre `masquerCommentaires` aux commentaires `<!-- -->`** (+ son test) — elle ne masque
      aujourd'hui que `//` et `/* */`, ce qui laisse **21** blocs de prose française passer pour des
      littéraux dans les `.svelte`. AC1. ⚠️ Vérifier que l'extension ne déplace pas `sitesTotal`.
- [ ] **T4 — La garde**, sa borne exacte, son allowlist — AC1, AC3, AC4. ⚠️ **Avant les correctifs**,
      et sa sortie rouge sur les cinq sites du patron est collée au Record — AC2.
- [ ] **T5 — `payment-batches`** : `paymentBatchStatusLabel` (3 cas) et `failedItemLabel` (6 codes) — AC5, AC7.
- [ ] **T6 — `credit-notes`** : `creditNoteStatusLabel` (3 cas) — AC5, AC7.
- [ ] **T7 — `invoices`** : `statusLabel` de la liste (site 7, **ferme #255**) et de la fiche (site 5) — AC5.
- [ ] **T8 — Correctifs hors portée de la garde** : `settings` (site 4), la barre de navigation
      (site 6), le titre de dialogue (site 8) — AC5.
- [ ] **T9 — Glossaire** : promotion des termes tranchés — AC6.
- [ ] **T10 — Recompter et déclarer `sitesTotal`** — AC8.
- [ ] **T11 — Retirer les clés `nav-*` du périmètre de la 23-4** *(l'epic en prévoyait 4 ; cette
      story en couvre 9)* — sinon double travail ou conflit.
- [ ] **T12 — Gates complets, E2E comprise** — AC9.

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
| 5 | `invoices/[id]/+page.svelte:602-605` `statusLabel` | `:755` | — | non |
| 6 | `+layout.svelte:57-150` *(motif : `label: '`)* — **3 groupes + 6 entrées = 9 libellés** | `:340` `<span>{group.label}</span>` | — *(chrome permanente)* | non — ⚠️ **mais voir le piège E2E** |
| 7 | `invoices/+page.svelte:251` `statusLabel` | `:399` | — | non |
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
| **`Généré`** | ⚠️ **NON attesté** — vérifié : zéro occurrence dans les 4 locales | **arbitrage T1** |
| **`Confirmé`** | ⚠️ **NON attesté** | **arbitrage T1** |
| **`Émis`** | ⚠️ **NON attesté**. ⚠️ Ne pas le confondre avec « validé », qui est l'acte d'immuabilité | **arbitrage T1** |
| **les 6 codes de `failedItemLabel`** | ⚠️ **à relever ET à faire arbitrer** — 6 × 4 = **24 valeurs** | s'inspirer de `reminders-error-*` (15 codes) et `imported-supplier-invoices-error-*` (10), déjà traduits. ⚠️ Ce sont des **messages techniques** : dire au § glossaire s'ils montent en partie A ou non |

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

⚠️ **Le patron y est CORRECT** — ce sont des tables de replis passées en 2ᵉ argument d'`i18nMsg` :

| entrée | fichier | consommée en |
|---|---|---|
| `LABEL_FALLBACK` | `lib/features/invoices/PaymentStatusBadge.svelte:18` | `:24`, 2ᵉ arg |
| `FILTER_FALLBACK_FR` | `routes/(app)/invoices/due-dates/+page.svelte:42` | 2ᵉ arg |
| `TYPE_FALLBACK` | `routes/(app)/accounts/+page.svelte` | `:37`, 2ᵉ arg |
| `ROLE_FALLBACK` | `routes/(app)/accounts/+page.svelte:40` | `:63`, 2ᵉ arg |

⚠️ **`readFallback` rend `null` sur ces quatre-là** (le 2ᵉ argument est `TABLE[clé]`, pas un
littéral) : l'outillage ne sait pas les reconnaître seul. **Critère de discrimination à tenir** — *un
`Record` de littéraux est légitime si et seulement si chacun de ses accès apparaît en 2ᵉ argument
d'`i18nMsg` ou d'un relais.* Sans ce critère écrit, le dev exemptera au nom, et `navGroups` (site 6,
même forme) deviendrait légitime par accident.

Ajouter aussi la classe des littéraux **non français** (`'—'` de `account-label.ts:66`, `'N/A'`) aux
**critères**, pas à l'allowlist.

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
- Grep de contrôle pour AC7 :
  `grep -rnE "toBe\('[A-ZÀ-Ü]|toContain\('[a-zà-ü]" frontend/src/lib/features/*/*helpers.test.ts`

### Ce qui a été balayé et trouvé PROPRE — ne pas re-balayer

`reconciliation`, `reports`, `onboarding`, `contacts`, `accounts`, `vat-rates`, `api-keys`,
`opening-balances`, `reminders`, `imported-supplier-invoices`, `journal-entries`, `bank-accounts`,
`fiscal-years`, `products`, `due-dates`. *(Balayage par le nom : 32 déclarations `*Label|*Text|*Display`,
aucune fautive hors des sites listés.)*

### References

- Angle mort : [issue #255](https://github.com/guycorbaz/kesh/issues/255)
- Précédent : `_bmad-output/implementation-artifacts/23-3-supplier-invoices.md`, § *Review Findings —
  passe 4* (le défaut d'origine) et § *passe 5* (la chasse au patron)
- Glossaire : `docs/i18n-glossaire.md` — partie A non négociable en rollout
- Outillage : `frontend/src/lib/shared/i18n-literal-reader.js`, `i18n-harvest.js`
- Montage à copier : `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts`
- Correctif de référence : `frontend/src/lib/features/supplier-invoices/supplier-invoice-helpers.ts`
  et son test, qui emprunte le chemin réel du dictionnaire

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-20 | **passe 1** de `validate` | **2 CRITICAL · 6 HIGH · 6 MEDIUM · 3 LOW** (Opus ×2). ⚠️ **La spec ne survit pas à sa première passe, et deux findings la refondent.** **CRITICAL-1** : la branche avait été coupée de `main` **avant** le merge de la 23-3 — quatre références pointaient dans le vide et le défaut fondateur était encore dans l'arbre. Corrigé : #325 mergée, branche rebasée sur `046efa51`. **CRITICAL-2, mesuré** : « la garde rougit sur les cinq sites » était **irréalisable** — trois formes du défaut (valeur sans appel, tableau de données, nœud de markup) sont **hors de portée par construction**. La garde est donc **réduite à ce qu'elle peut prouver** et les trois sites restants passent en correctifs manuels **avec motif écrit**. L'élargissement « par le contenu » a été **chiffré** : 175 sites à trier pour rater quand même `Brouillon` et 7 des 9 libellés de nav — proscrit. **HIGH le plus embarrassant** : le tableau cochait ✅ « en-tête traduit » pour deux sites dont **les clés n'existent dans aucune locale** — elles sont à l'allowlist. Le défaut y est **latent**, pas actif ; ma propre prose le disait deux paragraphes plus haut. Autres faits corrigés : `masquerCommentaires` ne masque **pas** les `<!-- -->`, `corpsDeFonction` **n'est pas exportée**, un **3ᵉ** verrou de test (`toContain`), **quatre** tables de replis et non deux, doc-comment copié **trois** fois et non quatre, `NavItem` **déjà correct** (c'est le type de groupe qui manque). Sites 5 → **7**. |
| 2026-08-20 | création | spec initiale, à partir de la passe 5 de revue de la 23-3. |
