# Story 23.3b : la garde contre les libellés en dur, et les cinq sites qu'elle révèle

Status: ready-for-dev

## Pourquoi cette story existe, et pourquoi maintenant

⚠️ **Une fonction de libellé qui n'appelle jamais `i18nMsg` est invisible de TOUT l'appareil de
contrôle de cet epic.** Le moissonneur (`i18n-harvest.js`), l'allowlist (`i18n-dette-connue.ts`) et
les **deux** gardes (`i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`) ne relèvent que les
**sites `i18nMsg`**. Ce qui n'appelle pas n'existe pour aucun d'eux — c'est l'angle mort nommé par
l'issue [#255](https://github.com/guycorbaz/kesh/issues/255), qui le décrit sans le couvrir.

Le défaut a été trouvé **en production**, en passe 4 de revue de la story 23-3 :
`supplierInvoiceStatusLabel()` retournait `'Ouverte'` / `'Payée'` / `'Annulée'` en dur, **sous un
en-tête de colonne que la story venait de traduire**. Un germanophone lisait « Status » au-dessus de
« Payée », sur chaque ligne de la liste et en badge sur chaque fiche. Corrigé dans la 23-3.

⚠️ **La passe 5 a cherché le PATRON plutôt que la chaîne, et l'a retrouvé quatre fois.** Le même
doc-comment est copié mot pour mot — *« Libellé FR du statut d'un X (fallback i18n côté composant) »* —
et **aucun composant ne fait l'i18n**. Deux de ces sites sont dans `payment-batches` et
`credit-notes`, c'est-à-dire **les deux prochains domaines de la rollout** (23-4 et 23-5), dont 49
clés sont encore à l'allowlist.

**C'est ce qui rend la story urgente** : si la 23-4 traduit `payment-batches` en l'état, elle
produira le défaut à l'identique — en-tête « Statut » en allemand au-dessus de « Généré / Confirmé /
Annulé ». La traduction *active* le défaut, exactement comme pour `supplier-invoices-col-total`.

## Story

**En tant que** développeur d'un rollout de traduction de l'epic 23,
**je veux** qu'une garde refuse une fonction de libellé qui n'appelle pas `i18nMsg`,
**afin que** le patron cesse d'être recopié — et que traduire un domaine ne fige plus des valeurs
françaises sous des en-têtes traduits.

## Critères d'acceptation

1. **AC1 — la garde existe et lit les SOURCES.** Un test de `frontend/src/lib/shared/` relève, dans
   `src/`, les fonctions dont le rôle est de rendre un libellé affiché, et échoue si l'une d'elles
   retourne un littéral sans passer par `i18nMsg`. ⚠️ **Elle lit les sources, jamais les catalogues** —
   même raison que `i18n-un-repli-par-cle.test.ts` : une garde qui s'appuie sur les catalogues
   s'éteint au moment où la traduction est livrée, c'est-à-dire quand le risque devient réel.
2. **AC2 — la garde rougit sous mutation, et la mutation est CONSIGNÉE.** Remettre en dur l'un des
   sites corrigés fait échouer le test ; la preuve est notée dans le Dev Agent Record avec la sortie
   réelle. Une garde non éprouvée par mutation est un test muet.
3. **AC3 — borne EXACTE, pas un minimum.** Le décompte des fonctions relevées est une valeur exacte
   assortie de la consigne « un écart se recompte, il ne s'ajuste pas ». *(La 23-3 a livré une borne
   `>= 90` là où sa voisine posait un total exact — deux disciplines opposées dans un même commit.)*
4. **AC4 — allowlist justifiée, pas une échappatoire.** Les cas légitimes (fonction rendant une clé
   et non un texte, table de replis passée en 2ᵉ argument d'`i18nMsg`, valeur non affichée) sont
   inscrits dans une liste **avec un motif écrit par entrée**. ⚠️ L'exemption est l'issue la moins
   coûteuse, donc celle qu'il faut contrôler.
5. **AC5 — les cinq sites relevés sont corrigés**, chacun avec ses clés dans les **quatre** locales.
6. **AC6 — les termes non attestés sont tranchés et promus en partie A du glossaire**, avec leur clé
   attestante. ⚠️ **`Généré`, `Confirmé` et `Émis` ne sont attestés NULLE PART** (vérifié au grep) —
   ils demandent un arbitrage, comme `analytique` et `bascule` avant eux.
7. **AC7 — les tests qui VERROUILLENT le français sont corrigés.** Deux tests unitaires assèrent
   aujourd'hui `toBe('Généré')` / `toBe('Brouillon')` : ils rendent le défaut vert. Chacun reçoit,
   comme dans la 23-3, une assertion qui emprunte **le chemin réel** (dictionnaire servi par l'API).
8. **AC8 — l'allowlist et les compteurs ne régressent pas** : `i18n-keys.test.ts` reste vert, son
   `sitesTotal` recompté et déclaré, jamais ajusté en silence.
9. **AC9 — gates complets verts, E2E comprise** (la story touche du code applicatif), et PR en
   `refs #316`.

## Tasks

- [ ] **T1 — Arbitrage des trois termes non attestés** (`Généré`, `Confirmé`, `Émis`) — AC6. **Bloquant.**
- [ ] **T2 — La garde** et sa mutation consignée — AC1, AC2, AC3, AC4. **Avant les correctifs** : elle
      doit d'abord rougir sur les cinq sites, ce qui prouve qu'elle les voit.
- [ ] **T3 — `payment-batches`** : `paymentBatchStatusLabel` (3 cas) et `failedItemLabel` (6 codes) — AC5, AC7.
- [ ] **T4 — `credit-notes`** : `creditNoteStatusLabel` (3 cas) — AC5, AC7.
- [ ] **T5 — `settings`** : le type d'organisation — AC5.
- [ ] **T6 — la barre de navigation** : groupes et entrées — AC5.
- [ ] **T7 — `invoices/[id]`** : `statusLabel` et le titre de dialogue en dur — AC5.
- [ ] **T8 — Glossaire** : promotion des termes tranchés — AC6.
- [ ] **T9 — Gates complets, E2E comprise** — AC9.

## Dev Notes

### Les cinq sites, relevés au grep et vérifiés jusqu'au site de RENDU

⚠️ **Chaque site a été vérifié jusqu'à l'écran** : une fonction jamais appelée ne serait pas un
défaut. Les appelants sont donnés — ne pas les redécouvrir.

| # | définition | rendu à l'écran | l'en-tête voisin est-il traduit ? | un test verrouille-t-il le français ? |
|---|---|---|---|---|
| 1 | `lib/features/payment-batches/payment-batch-helpers.ts:20-31` (`paymentBatchStatusLabel`) et `:34-49` (`failedItemLabel`) | `routes/(app)/payment-batches/+page.svelte:200,234` et `[id]/+page.svelte:84` | ✅ `payment-batches-col-status` | ⚠️ **OUI** — `payment-batch-helpers.test.ts:20-22` |
| 2 | `lib/features/credit-notes/credit-note-helpers.ts:23-31` (`creditNoteStatusLabel`) | `routes/(app)/credit-notes/+page.svelte:65` et `[id]/+page.svelte:178` | ✅ `credit-notes-col-status` | ⚠️ **OUI** — `credit-note-helpers.test.ts:22-24` |
| 3 | `routes/(app)/settings/+page.svelte:184-185` | la même ligne — `{data.company.orgType}` | ✅ `settings-field-org-type` | non |
| 4 | `routes/(app)/+layout.svelte:57-131` (groupes + 6 entrées) | `:340` — `<span>{group.label}</span>` | — *(chrome permanente)* | non |
| 5 | `routes/(app)/invoices/[id]/+page.svelte:602-605` (`statusLabel`) et `:949` (`<Dialog.Title>`) | `:755` et le dialogue de validation | — | non |

**Le site 3 est le plus embarrassant à l'usage** : l'utilisateur choisit son type d'organisation en
langue localisée pendant l'onboarding (« Indépendant », « PME »), puis retrouve dans les réglages le
**code brut du backend** — `Independant`, `Association`, `Pme` — dans **toutes** les langues, français
compris. Les clés existent déjà (`onboarding-org-*`) et sont câblées côté onboarding : il s'agit de
les réutiliser, pas d'en créer.

**Le site 4 est le plus visible** : la barre de navigation est sur **chaque page**. ⚠️ Et il porte un
paradoxe à signaler dans la PR : `/reconciliation` et `/reports` **sont déjà entièrement traduites**,
mais l'entrée de menu qui y mène reste en français. La dette est partiellement documentée en
commentaires `FINDING-7` / `FINDING-12` — **mais les libellés de GROUPE (`Quotidien`, `Mensuel`,
`Administration`) n'y sont pas mentionnés**, ils ne sont donc dans le périmètre d'aucune dette
déclarée. ⚠️ Le type `NavItem` (`:57`) n'a même pas de variante i18n possible pour un groupe :
`label: string` seul. **La structure du type devra changer**, ce n'est pas qu'un remplacement de
valeurs. *(L'epic 23 prévoyait déjà 4 clés `nav-*` en 23-4 — cette story les couvre ; à retirer du
périmètre de la 23-4 pour éviter le double travail.)*

**Le site 5 est un résidu de l'issue #255**, qui ne couvre que `invoices/+page.svelte` (la liste), pas
`[id]` (la fiche) — vérifié en lisant le corps de l'issue. Soit élargir #255, soit le dire dans la PR.

### Les termes : ce qui est attesté, et ce qui ne l'est pas

⚠️ **Relever avant d'inventer** — c'est la règle du glossaire, et elle vaut ici plus qu'ailleurs
puisque les valeurs vont être **figées dans quatre catalogues**.

| terme | statut | équivalence |
|---|---|---|
| `Brouillon` | ✅ **attesté** | `invoice-status-draft` → `Entwurf` / `Bozza` / `Draft` |
| `Annulé(e)` | ✅ **attesté** | `invoice-status-cancelled` → `Storniert` / `Annullata` / `Cancelled`. ⚠️ **Accord** : un *lot* est masculin (« Annulé »), une *facture* féminine (« Annulée ») — le français distingue, les cibles moins |
| `Ouverte` / `Payée` | ✅ **partie A** du glossaire *(23-3)* | `offen`/`aperta`/`open` · `bezahlt`/`pagata`/`paid` |
| **`Généré`** | ⚠️ **NON attesté** | arbitrage requis → partie A avec sa clé |
| **`Confirmé`** | ⚠️ **NON attesté** | arbitrage requis |
| **`Émis`** | ⚠️ **NON attesté** | arbitrage requis. ⚠️ Ne pas le confondre avec « validé », qui est l'acte d'immuabilité |
| les 6 codes de `failedItemLabel` | à relever | s'inspirer de `reminders-error-*` (15 codes) et `imported-supplier-invoices-error-*` (10), déjà traduits |

### Sur quoi la garde peut s'appuyer — ne rien réécrire

`frontend/src/lib/shared/i18n-literal-reader.js` **existe déjà** et expose ce qu'il faut :
`findCallSites(source)`, `readFallback(source, afterFirstArg)`, `readLiteral(source, i)`,
`masquerCommentaires(source)`, `findRelays(source)`. Et `i18n-harvest.js` expose
`dansLePerimetreDeFichier(nom)` — le filtre de balayage, **testé**, qui écarte déjà les `.test.*`.

⚠️ **Ce lecteur a été durci par SEPT passes de revue cumulées** sur les stories 23-1a/23-1b : gabarits
à apostrophes, échappements Fluent, commentaires, relais. **Ne pas en réécrire un second** — c'est
exactement le « reinventing wheels » que le processus cherche à éviter. `i18n-un-repli-par-cle.test.ts`
montre le montage complet en 40 lignes : le reprendre.

**Piste de détection, à éprouver plutôt qu'à suivre aveuglément** : la signature du défaut est un
`return '<littéral>'` (ou une valeur de `Record`/`switch`) **atteignable depuis une fonction dont la
valeur est rendue**, sans `i18nMsg` sur le chemin. Deux angles complémentaires :
- **par le nom** — `*Label`, `*Text`, `*Display` : ciblé, peu de faux positifs, mais contournable en
  renommant ;
- **par le contenu** — un littéral qui « ressemble à du français ». ⚠️ **Bruyant** : commentaires
  (déjà masqués par `masquerCommentaires`), clés d'API, valeurs d'énumération backend (`'open'`,
  `'paid'`), messages de test.

⚠️ **Le second angle seul produira des faux positifs, et le premier seul laissera passer.** Le choix
est un arbitrage à documenter dans le Dev Agent Record, pas à trancher en silence. **Ce qui n'est pas
négociable, c'est que la garde rougisse sur les cinq sites AVANT qu'ils soient corrigés** (T2 avant
T3-T7) : c'est la seule preuve qu'elle les voit.

### Ce qui a été balayé et trouvé PROPRE — ne pas re-balayer

`reconciliation`, `reports`, `onboarding`, `contacts`, `accounts`, `vat-rates`, `api-keys`,
`opening-balances`, `reminders`, `imported-supplier-invoices`, `journal-entries`, `bank-accounts`,
`fiscal-years`, `products`, `due-dates`.

⚠️ **Deux faux amis relevés au passage, à ne pas « corriger »** : `FILTER_FALLBACK_FR` et
`LABEL_FALLBACK` (`invoices/due-dates`, `PaymentStatusBadge.svelte`) sont des tables de **replis
passées en 2ᵉ argument d'`i18nMsg`** — le patron est **correct**. Une garde qui les signale a un faux
positif ; c'est le cas d'école de l'allowlist justifiée d'AC4.

### Pièges du dépôt qui s'appliquent ici

- ⚠️ **`lint-i18n-ownership` ne balaie que `src/lib/features/`**, jamais `src/routes/**` — les sites 3,
  4 et 5 y échappent par construction. Ne pas compter dessus.
- ⚠️ **Le namespace doit correspondre au dossier** : un module posé dans `features/X/` ne peut employer
  que des clés `X-*`. *(La 23-3 l'a appris en déplaçant un module : le lint a refusé, à raison.)*
- ⚠️ **Le gate E2E exige `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`**, sans quoi des tests échouent pour
  une raison qui ne ressemble pas à un problème de configuration — cf. `docs/testing.md`, corrigé au
  gate de la 23-3.
- ⚠️ **Ne déclarer que ce qui a tourné.** Le Dev Agent Record ne porte « gate vert » que pour un gate
  réellement exécuté.

### References

- Angle mort : [issue #255](https://github.com/guycorbaz/kesh/issues/255) — le décrit, ne le couvre pas
- Précédent complet : `_bmad-output/implementation-artifacts/23-3-supplier-invoices.md`, § *Review
  Findings — passe 4* (le défaut d'origine) et § *passe 5* (la chasse au patron)
- Glossaire, contrainte de l'epic : `docs/i18n-glossaire.md` — partie A non négociable en rollout
- Outillage à réutiliser : `frontend/src/lib/shared/i18n-literal-reader.js`, `i18n-harvest.js`
- Montage de garde à copier : `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts`
- Correctif de référence : `frontend/src/lib/features/supplier-invoices/supplier-invoice-helpers.ts`
  (story 23-3, passe 4) et son test, qui emprunte le chemin réel du dictionnaire

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-20 | création | spec initiale. ⚠️ **La story naît d'un défaut trouvé en production** (statut de facture en français dans les quatre langues, story 23-3 passe 4), dont la passe 5 a montré qu'il était **un patron copié quatre fois** et non un accident. Deux des cinq sites sont dans les **prochains domaines de la rollout** : les traduire en l'état figerait le défaut. **Trois termes ne sont attestés nulle part** et demandent un arbitrage. |
