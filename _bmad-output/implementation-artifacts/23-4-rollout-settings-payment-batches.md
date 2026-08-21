# Story 23.4 : le rollout `settings` + `payment-batches` + `onboarding`, et les deux clés qui portent deux sens

Status: ready-for-dev

> **Base** : branchée sur `main` **après le merge de la 23-3b** (`9ca57d49`, PR #327). Tous les
> faits, décomptes et sites de ce document ont été **relevés au moissonneur sur cette base** le
> 2026-08-21 — pas recopiés du plan d'epic.
> ⚠️ **Les numéros de ligne dateront** ; les **clés** et les **motifs de grep** font foi.
>
> **Catalogues** : `fr-CH` (source), `de-CH`, `it-CH`, `en-CH` dans
> `crates/kesh-i18n/locales/<locale>/messages.ftl`. Allowlist : `frontend/src/lib/shared/i18n-dette-connue.ts`.

## Pourquoi cette story vient maintenant, et pas avant

⚠️ **Elle ne pouvait pas démarrer avant le merge de la 23-3b**, et ce n'est pas une précaution de
principe : `payment-batches` et `settings` sont **ses domaines**, et la 23-3b y a corrigé des
libellés qui n'appelaient pas `i18nMsg`. Traduire ces domaines avant elle aurait **activé** le
défaut au lieu de le corriger — un germanophone aurait lu « Status » au-dessus de « Généré ».

C'est le précédent exact de la 23-1b, qui ne pouvait pas démarrer avant la 23-1a, et celui du
`CRITICAL-1` de la passe 1 de la 23-3b, où une branche coupée trop tôt faisait pointer quatre
références dans le vide.

## Story

**En tant que** utilisateur germanophone, italophone ou anglophone de Kesh,
**je veux** que les écrans de paiements fournisseurs, de réglages et d'onboarding parlent ma langue,
**afin de** ne plus lire du français au milieu d'une interface traduite.

## ⚠️ Ce que le relevé a trouvé, et que le plan d'epic ne pouvait pas savoir

**Le moissonneur a été exécuté sur la base réelle** (`i18n-harvest.js`, même outil que les rollouts
précédents). Il confirme le périmètre — et il **signale trois conflits**, dont deux dans ce
périmètre.

### ⚠️ `payment-batches-col-total` porte DEUX GRANDEURS — jumeau exact du défaut de la 23-3

| site | valeur affichée | repli |
|---|---|---|
| `payment-batches/+page.svelte:222` | `batch.totalAmount` — **total du lot** | `'Total'` |
| `payment-batches/[id]/+page.svelte:98` | `batch.totalAmount` — **total du lot** | `'Total'` |
| `payment-batches/[id]/+page.svelte:122` | `item.amount` — **montant d'une LIGNE** | `'Montant'` |

⚠️ **C'est le défaut de `supplier-invoices-col-total` à l'identique** (« TTC » sur le total de
facture contre « Total HT » sur une ligne), celui qui a fait naître la garde « une clé, un repli ».
**Le défaut est LATENT** : tant que la clé manque des quatre catalogues, `i18nMsg` retombe sur le
repli **du site appelant** et chaque écran affiche le bon libellé, par accident. **Entrer une valeur
unique au catalogue l'impose aux trois sites** — et « Total » se retrouverait au-dessus d'une
colonne de montants de lignes, ou « Montant » comme total du lot.

**Ne pas aplatir : scinder**, comme la 23-3 → `payment-batches-col-total` (le lot) et
`payment-batches-line-amount` (la ligne).

### ⚠️ `payment-batches-col-date` porte deux REGISTRES

| site | repli |
|---|---|
| `payment-batches/+page.svelte:220` — en-tête de colonne | `'Exécution'` |
| `payment-batches/[id]/+page.svelte:96` — étiquette de fiche | `"Date d'exécution"` |

Même grandeur (`requestedExecutionDate`), deux longueurs. Précédent de la 23-3 : `field-reference`
puis `field-project` opposaient un libellé de **formulaire** à un libellé de **détail**, et ont été
**scindés, pas aplatis**. Même traitement — une colonne étroite ne porte pas la même chaîne qu'une
étiquette de fiche.

### ⚠️ Le troisième conflit est HORS PÉRIMÈTRE, et il faut le laisser

`credit-notes-title` → « Avoirs » / « Avoir » (pluriel de liste contre singulier de fiche). Domaine
de la **23-5**. Le relever ici, ne pas le traiter — et surtout **ne pas entrer la clé au catalogue
« en passant »**, ce qui figerait l'un des deux sens.

## ⚠️ La conséquence de l'arbitrage de la 23-3b que personne n'a encore vue

La 23-3b a tranché : le statut d'un lot n'est plus « **Généré** » mais « **Créé** », et le verbe
*créer* devient uniforme dans le domaine. **Trois sites de ce périmètre disent encore
« générer »** :

| clé | repli actuel | devient |
|---|---|---|
| `payment-batches-new` | « Générer un lot de virements » | « **Créer** un lot de virements » |
| `payment-batches-generate` | « Générer le lot » | « **Créer** le lot » |
| `payment-batches-created` | « Lot généré » | « Lot **créé** » |

⚠️ **Sans ce changement, l'interface demande de « Générer un lot » pour produire un lot dont le
statut s'affiche « Créé ».** L'utilisateur ne peut pas faire le lien entre les deux — c'est
exactement le raisonnement qui a fait corriger `ALREADY_IN_GENERATED_BATCH` en 23-3b. **Ce n'est pas
une extension de périmètre : c'est la conséquence directe d'un arbitrage déjà rendu**, et la laisser
de côté rendrait le domaine incohérent avec lui-même.

## ⚠️ La garde « une clé, un repli » NE COUVRE PAS ce domaine

`i18n-un-repli-par-cle.test.ts:51` porte `const PREFIXES = ['supplier-invoices-',
'imported-supplier-invoices-']`. **Elle est bornée au domaine de la 23-3** : les deux divergences
ci-dessus ne la font pas rougir, et c'est le moissonneur qui les a trouvées.

**Étendre la garde à `payment-batches-` fait partie du livrable** — sans quoi le rollout suivant
reproduira le défaut, et la garde restera un dispositif à domaine unique alors qu'elle prétend
protéger l'epic.

## Critères d'acceptation

1. **AC1 — les 93 clés du périmètre existent dans les QUATRE catalogues**, et l'allowlist décroît
   d'autant : **166 → 73**, recompté depuis le fichier. ⚠️ La ventilation relevée au moissonneur :
   `payment-batches` **30**, `settings` **55** *(dont 25 via le relais `msg` de l'écran des modèles
   d'e-mail)*, `onboarding` **4**, plus les **4 `nav-*`** de l'inventaire — que le moissonneur ne
   voit pas, leurs clés étant portées par une table de données et non par un littéral.
2. **AC2 — `payment-batches-col-total` est SCINDÉE** en une clé de lot et une clé de ligne, et
   `payment-batches-col-date` en un libellé de colonne et un libellé de fiche. ⚠️ **Aucune valeur
   unique ne doit être imposée à des sites qui affichent des grandeurs ou des registres différents.**
3. **AC3 — la garde « une clé, un repli » couvre `payment-batches-`**, et sa borne `CLES_RELEVEES`
   est **recomptée**, jamais ajustée. ⚠️ **Elle doit rougir sur les deux divergences AVANT leur
   correction** : la sortie brute du run rouge est collée au Dev Agent Record. Sans cette preuve,
   une extension vide passe tous les autres AC.
4. **AC4 — le verbe « générer » disparaît du domaine** au profit de « créer », sur les trois sites
   du tableau ci-dessus. Contrôle : `grep -rn "[Gg]énér" frontend/src/routes/\(app\)/payment-batches/`
   ne rend plus que des occurrences sans rapport avec la création d'un lot.
5. **AC5 — les termes non attestés sont tranchés PAR GUY avant écriture**, puis promus en partie A
   du glossaire avec leur clé attestante. ⚠️ **`lot` en fait partie** : la 23-3b l'a déjà employé et
   figé (`Bereits in einem erstellten Stapel`) **sans le promouvoir** — c'est le défaut que le
   glossaire documente lui-même, et il est ouvert depuis une story.
6. **AC6 — la garde des libellés en dur reste verte** (`i18n-libelle-en-dur.test.ts`), et sa borne
   `CANDIDATES_ATTENDUES` est recomptée si le rollout crée ou supprime une fonction de libellé.
7. **AC7 — aucun test ne verrouille le français.** Les **deux** greps de contrôle de la 23-3b, plus
   celui qu'elle a dû inventer en cours de route :
   `grep -rnE "toBe\('[A-ZÀ-Ü]|toContain\('[a-zà-ü]" frontend/src/lib/features/*/*helpers.test.ts`
   `grep -rnE "getByRole\(.*name: '[A-ZÀ-Ü]|getByText\('[A-ZÀ-Ü]" frontend/tests/e2e/`
   `grep -rnE "toContainText\(/[A-ZÀ-Üa-zà-ü]|has-text\(\"[A-ZÀ-Ü]" frontend/tests/e2e/`
   ⚠️ **Le troisième est né d'un verrou que les deux premiers ne pouvaient pas voir** — une
   assertion de *contenu* sur un sélecteur déjà stable. ⚠️ **Et cette liste n'est toujours pas
   close** : cf. [KF-043 (#326)](https://github.com/guycorbaz/kesh/issues/326), la suite E2E ne
   tourne qu'en français.
8. **AC8 — gates complets verts, E2E comprise**, PR en `refs #316`. ⚠️ **#316 reste OUVERTE** —
   elle ne se ferme qu'à la 23-6, qui vide l'allowlist.

## Tasks

- [ ] **T1 — Arbitrage de Guy sur les termes non attestés** — AC5. ⚠️ **Bloquant, et à faire
      AVANT toute écriture au catalogue** : ces valeurs seront figées dans quatre catalogues et
      promues en partie A. Ne jamais inventer ; si un terme neuf apparaît en cours de route, le
      proposer au Record et **s'arrêter**. Liste de départ au § *Les termes*.
- [ ] **T2 — Scinder les deux clés à double sens** — AC2. ⚠️ **AVANT d'écrire quoi que ce soit aux
      catalogues** : entrer `payment-batches-col-total` telle quelle figerait le défaut.
- [ ] **T3 — Étendre la garde « une clé, un repli » à `payment-batches-`** — AC3. ⚠️ **AVANT T2**,
      et coller sa sortie ROUGE : c'est la seule preuve qu'elle voit les deux divergences.
- [ ] **T4 — `payment-batches` : 30 clés × 4 locales** — AC1, AC4 *(le verbe « créer »)*.
- [ ] **T5 — `settings` : 55 clés × 4 locales** — AC1. ⚠️ **25 passent par le relais `msg`** de
      l'écran des modèles d'e-mail : le littéral vit au site `msg(`, pas au site `i18nMsg(`.
      `findRelays` les voit ; un grep naïf de `i18nMsg(` ne les verrait pas.
- [ ] **T6 — `onboarding` : 4 clés × 4 locales** — AC1. ⚠️ Le dossier est **partiellement traduit
      depuis la 23-1b** (8 libellés faits, 4 messages restants) : cette tâche referme l'écart.
- [ ] **T7 — Les 4 `nav-*` de l'inventaire** — AC1 : `nav-credit-notes`, `nav-email-templates`,
      `nav-projects`, `nav-supplier-invoices-import`. ⚠️ **Ce sont des clés DÉJÀ CÂBLÉES dont la
      traduction manque** — à ne pas confondre avec les 9 libellés de navigation que la 23-3b a
      sortis du français en dur. Les deux ensembles sont **disjoints**, vérifié.
- [ ] **T8 — Décrémenter l'allowlist de 93** — AC1, borne recomptée depuis le fichier.
- [ ] **T9 — Glossaire** : promotion des termes tranchés, dont **`lot`** — AC5.
- [ ] **T10 — Balayer les verrous de français**, les trois greps — AC7.
- [ ] **T11 — Gates complets, E2E comprise, et différentiel contre `main`** — AC8. ⚠️ **Le
      différentiel se lit sur `fichier › titre`, jamais sur `fichier:ligne`** : un commentaire
      ajouté décale les lignes et fabrique des régressions imaginaires. Précédent 23-3b.

## Dev Notes

### Les termes — ce qui est attesté, et ce qui demande un arbitrage

| terme | statut | remarque |
|---|---|---|
| `règlement` | ✅ **partie A** | `Zahlung` / `pagamento` / `payment` |
| `localité` | ✅ **partie A** | `Ort` / `località` / `Town/city` |
| `créé`, `confirmé`, `annulé` | ✅ **partie A** | promus par la 23-3b |
| **`lot`** | ⚠️ **employé, figé, JAMAIS promu** | `Stapel` / `lotto` / `batch`, relevé sur `reminders-batch-cap` **et déjà écrit au catalogue par la 23-3b**. À promouvoir — c'est le défaut que le glossaire documente : *un terme que le rollout emploie et fige va en partie A* |
| **`NPA`** | ⚠️ **à trancher** | Code postal suisse. `PLZ` en allemand, `NPA` en italien suisse, `Postcode`/`ZIP` en anglais — **aucun attesté au catalogue** |
| **`Référence message`** | ⚠️ **à trancher** | `msgId` de pain.001, terme ISO 20022. Traduire ou garder ? Précédent : `EndToEndId` est laissé **verbatim** dans la même table |
| **`virement`** | ⚠️ **à vérifier** | employé par `payment-batches-new` ; contrôler s'il est attesté avant de le tenir pour acquis |

⚠️ **Relever avant d'inventer**, et **distinguer relevé de dérivé** : la 23-3b a dû corriger deux
entrées de partie A qui citaient comme *attestantes* des clés portant l'**infinitif** là où la
valeur écrite était un **participe**. La colonne « précédent » du glossaire doit dire laquelle des
deux choses elle fait.

### Ce que la 23-3b a changé dans ces fichiers, et qu'il ne faut pas défaire

- `payment-batch-helpers.ts` — `paymentBatchStatusLabel` et `failedItemLabel` passent désormais par
  `i18nMsg` (9 clés déjà au catalogue). **Ne pas les recompter dans les 30.**
- `settings/+page.svelte` — `orgTypeLabel` réutilise les clés `onboarding-org-*`.
- `+layout.svelte` — `getGroupLabel` et 6 entrées passées en variante `i18nKey`.
- `payment-batches/[id]/+page.svelte` — le badge de statut porte `data-status`, sur lequel l'E2E
  asserte. ⚠️ **Ne pas le retirer** : l'assertion redeviendrait textuelle.

### Pièges du dépôt qui s'appliquent ici

- ⚠️ **Le gate E2E exige `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`** — sans eux, des tests échouent
  d'une façon qui ne ressemble pas à un problème de configuration. Recette complète dans
  `docs/testing.md`.
- ⚠️ **`pkill -f "target/debug/kesh-api"` tue le shell qui le porte** — utiliser `pkill -x kesh-api`.
- ⚠️ **Remettre la base de gate à zéro avant CHAQUE gate complet**, sans se demander comment le run
  précédent s'est terminé (KF-039).
- ⚠️ **Le namespace doit correspondre au dossier** pour `src/lib/features/` — `routes/**` est hors
  du périmètre de `lint-i18n-ownership`, ce qui **ne dispense pas** de la convention.
- ⚠️ **Une migration appliquée ne se modifie plus** — sans objet ici, aucune migration.

### Ce qui a été balayé et trouvé PROPRE — ne pas re-balayer

Les 5 clés **sans repli littéral** relevées au moissonneur sont **toutes** dans
`lib/features/reconciliation` (`TransactionSplitModal.svelte`) : domaine de la **23-5**, hors
périmètre. Le moissonneur ne rend **aucun** repli à échapper (`aEchapper` vide) sur tout le dépôt.

### References

- Plan d'epic : `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`, ligne 23-4
- Story précédente : `_bmad-output/implementation-artifacts/23-3b-garde-libelles-en-dur.md` —
  § *Review Findings* des trois passes, et l'arbitrage « générer → créer »
- Patron de rollout : `_bmad-output/implementation-artifacts/23-3-supplier-invoices.md`
- Glossaire : `docs/i18n-glossaire.md` — partie A non négociable en rollout
- Outillage : `frontend/src/lib/shared/i18n-harvest.js`, `i18n-literal-reader.js`
- Gardes à tenir vertes : `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`,
  `i18n-libelle-en-dur.test.ts`, `loader.rs::parity_between_locales`
- Angle mort connu : [KF-043 (#326)](https://github.com/guycorbaz/kesh/issues/326)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-21 | création | Spec écrite après **exécution du moissonneur sur la base réelle**, pas depuis le plan d'epic. Périmètre confirmé à **93** (30 + 55 + 4 + 4). ⚠️ **Trois replis divergents trouvés, deux dans le périmètre** — dont `payment-batches-col-total`, qui porte **deux grandeurs** (total du lot / montant d'une ligne) : jumeau exact du défaut de la 23-3, latent, et **que la traduction activerait**. ⚠️ **La garde « une clé, un repli » ne couvre pas ce domaine** (bornée à `supplier-invoices-`) : son extension entre au livrable. ⚠️ **Conséquence non vue de l'arbitrage de la 23-3b** : trois sites disent encore « Générer un lot » pour un objet dont le statut s'affiche « Créé ». ⚠️ **`lot` est employé et figé depuis la 23-3b sans être en partie A** du glossaire. |
