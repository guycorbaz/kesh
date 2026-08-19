# Story 23.3 : `supplier-invoices` — 115 clés, et un défaut que la traduction aurait activé

## Status

review

Deuxième rollout de l'Epic 23, et le plus gros : **115 clés** sur les **275** que l'allowlist portait
à la clôture de la 23-2.

⚠️ **Ce chiffre a dû être corrigé** : la spec annonçait « 109 sur 265 ». **265 n'a jamais été
recompté depuis la source** — `git show 2865c2b6:…/i18n-dette-connue.ts | grep -c` rend **275**, et
c'est aussi ce qu'annonce le dernier commentaire de [#316]. Le défaut est exactement celui que la
§ *Recompter ses propres comptes rendus* vise, commis sur la seule ligne de ce fichier qui n'avait
pas été recomptée — alors que 103, 10, 160+4+2 et 339 = 113 × 3 l'ont tous été.

## Story

**En tant qu'** utilisateur de Kesh dans une autre langue que le français,
**je veux** que le domaine des factures fournisseurs — liste, détail, import de QR-factures,
complétion — s'affiche dans ma langue,
**afin de** ne pas voir un quart de l'application en français sans que rien ne me le signale.

## Périmètre

**Dedans** : les **105 clés statiques** de `supplier-invoices` et
`imported-supplier-invoices`, plus les **10 clés de la famille dynamique**
`imported-supplier-invoices-error-*`, écrites dans les **quatre** locales ; le retrait des
109 entrées correspondantes de `frontend/src/lib/shared/i18n-dette-connue.ts`.

⚠️ **Différence majeure avec la 23-2** : là, `fr-CH` était la **source** et ne bougeait pas. Ici les
clés manquent des **quatre** catalogues — le français doit donc être écrit lui aussi, moissonné
depuis les replis en dur du code, **puis relu** avant d'être figé.

**Dehors** : les **166** clés des rollouts 23-4 à 23-6 ; le comportement applicatif, **à une exception
documentée ci-dessous**.

## Dev Notes

### ⚠️ Le moissonneur a trouvé un DÉFAUT, pas seulement une dette

Trois clés de ce domaine ont **deux replis divergents** selon leur site d'appel. Deux sont des
variantes de formulation sans conséquence. **La troisième est un bug.**

```
supplier-invoices-col-total
    « TTC »        supplier-invoices/+page.svelte:446, supplier-invoices/[id]/+page.svelte:161
    « Total HT »   supplier-invoices/[id]/+page.svelte:192
```

Les deux libellés désignent des grandeurs **différentes**, et la source de vérité les distingue
explicitement :

| | valeur affichée | ce que dit le code |
|---|---|---|
| liste + détail (`:446`, `:161`) | `invoice.totalAmount` | `supplier_invoice.rs:29` — « **TTC** (Σ HT + Σ TVA), montant dû au créancier » |
| tableau des lignes (`:192`) | `line.lineTotal` | `supplier_invoice.rs:89` — « `line_total` = `quantity × unit_price` (**HT**) » |

**Chaque repli est donc juste à son site.** Le défaut est qu'ils partagent **une seule clé**.

⚠️ **Et le bug est LATENT aujourd'hui : c'est l'acte de traduire qui l'activerait.** Tant que la clé
manque des quatre catalogues, `i18nMsg` retombe sur le repli **du site appelant**, et chaque écran
affiche le bon libellé — par accident. Dès qu'une valeur unique entre au catalogue, **elle s'impose
aux trois sites** et l'un des deux groupes devient faux. Traduire sans scinder ferait afficher
« TTC » au-dessus d'une colonne de montants **hors taxe**, sur une facture fournisseur.

**Correctif retenu** : scinder en deux clés — `supplier-invoices-col-total` reste le total **TTC**
de la facture, et le tableau des lignes reçoit `supplier-invoices-line-total`. C'est une
modification de code applicatif, **assumée et hors du périmètre « traduction seule »** : la story ne
peut pas livrer sa traduction sans elle.

⚠️ **Ce cas justifie à lui seul que le moissonneur SIGNALE les conflits au lieu de choisir.** S'il
avait retenu silencieusement le premier repli venu, ce défaut serait entré au catalogue sans que
personne ne le voie — et n'aurait été découvert qu'en lisant une facture dans la mauvaise langue.

### Les deux autres conflits, à trancher à la relecture

| clé | replis | arbitrage |
|---|---|---|
| `imported-supplier-invoices-reload-failed` | « La liste n'a pas pu être rechargée — actualisez la page. » / « Import effectué, mais la liste n'a pas pu être rechargée — actualisez la page. » | ⚠️ Les deux **portent un sens différent** — le second dit que l'import a réussi. À scinder aussi, ou à généraliser en un libellé vrai dans les deux cas |
| `supplier-invoices-field-project` | « Projet analytique (optionnel) » / « Projet analytique » | Simple variante ; retenir la forme **sans** la mention, le caractère facultatif se marquant à l'interface |

### La famille dynamique — 10 clés qui ne se moissonnent pas

`supplier-invoices/import/+page.svelte:55-66` porte une carte `errorCode → [suffixe, libellé]`. Les
clés sont **construites** (`imported-supplier-invoices-error-${entry[0]}`), donc invisibles à toute
extraction de littéral. Elles se relèvent **dans la carte**, pas dans le code d'appel.

⚠️ `imported-supplier-invoices-error-unknown`, lui, **est** un littéral et compte déjà parmi les 105.

### Terminologie — ce que le glossaire impose déjà

Ce domaine emploie plusieurs termes tranchés ou attestés. **Les relever, ne pas les réinventer** :

| terme | statut |
|---|---|
| QR-facture | partie B — **terminologie officielle SIX**, `QR-Rechnung` / `fattura QR` / `QR-bill`, à retenir tel quel |
| justificatif | partie B — `Beleg` (CO art. 957a) / documento giustificativo / supporting document |
| écarter / compléter | partie B — le couple de la file d'import |
| image | partie B — scan de QR-facture |
| exercice, écriture comptable, compte de charge | attestés au catalogue (cf. 23-2) |

⚠️ **Quatre de ces cinq termes sont en partie B** — donc **non attestés** et contraignants une fois
tranchés. Leur arbitrage est un **préalable**, comme « analytique » l'a été pour la 23-1b et
« bascule » pour la 23-2.

### Les quatre homonymies déjà connues — les chercher ici aussi

La 23-2 a montré qu'elles ne se voient pas dans le français : `Actif` (statut / classe de bilan),
`Valider` (enregistrer / valider comptablement), `Libellé` (description / étiquette),
`bascule` (interrupteur / changement de taux). ⚠️ **Et elle a montré pire** — le contrôle mené sur
une seule langue en laisse passer : le raisonnement fautif y était présent dans les trois, l'anglais
s'en est tiré par évidence, **l'allemand par coïncidence**, l'italien a payé. Le contrôle se fait
donc **langue par langue**, sur le catalogue **complet** de chacune.

## Acceptance Criteria

1. **AC1** — les **115** clés existent dans les **quatre** locales, valeur non vide.
2. **AC2** — les 109 entrées correspondantes sont retirées de `i18n-dette-connue.ts`, qui passe de
   **275** à **166**. La garde B passe.
3. **AC3** — `supplier-invoices-col-total` est **scindée** : le tableau des lignes emploie
   `supplier-invoices-line-total`. ⚠️ **Un test le couvre** — sans quoi rien n'empêche la fusion de
   revenir.
4. **AC4** — les deux autres conflits sont tranchés, et `imported-supplier-invoices-reload-failed`
   **ne prétend plus deux choses différentes**.
5. **AC5** — les termes de partie B employés ici sont **tranchés et promus** en partie A, chacun
   avec la clé qui l'atteste.
6. **AC6** — les libellés `fr-CH` moissonnés sont **relus avant d'être figés**, et la relecture est
   consignée. ⚠️ Ils sont écrits par des développeurs dans le feu du code, pas rédigés.
7. **AC7** — contrôle d'homonymie **langue par langue** sur le catalogue complet, consigné en
   tableau. Pas de contrôle mené sur l'anglais seul.
8. **AC8** — les placeables (`{$id}`, `{$actual}`, `{$expected}`, `{$code}`…) sont **identiques**
   dans les quatre locales, et aucun nom de variable n'est traduit.
9. **AC9** — gates complets verts, **exécutés et non recopiés**, E2E comprise : la story **touche du
   code applicatif** (AC3), contrairement à la 23-2.
10. **AC10** — [#316] n'est **pas** fermée ; la PR porte `refs #316`.
    ⚠️ **Il restera 166 clés.** L'allowlist perd **109** entrées, pas 115 : **six** des 115 clés
    écrites sont **neuves** — quatre nées de la scission des conflits, deux de la couverture des
    chaînes en dur (`-col-qty`, `-col-vat`) —, et n'ont donc jamais figuré à la dette.
    `275 − 109 = 166`, ce que le fichier confirme. *(Une lentille de revue a proposé 162 = 275 − 113 :
    le piège est de soustraire les clés livrées d'une liste qui ne les contenait pas toutes.)*

## Tasks

- [x] **T1 — Arbitrage** des termes de partie B (AC5). **Bloquant.**
- [x] **T2 — Scission de `supplier-invoices-col-total`** et son test (AC3), avant toute traduction.
- [x] **T3 — Arbitrage des deux autres conflits** (AC4).
- [x] **T4 — Relecture des 105 libellés `fr-CH`** moissonnés (AC6), consignée.
- [x] **T5 — `fr-CH`** : les 115 clés.
- [x] **T6 — `de-CH`, `it-CH`, `en-CH`** : les 115 × 3, avec contrôle d'homonymie langue par langue (AC7).
- [x] **T7 — Allowlist** : 275 → 166 (AC2).
- [x] **T8 — Gates complets, E2E comprise** (AC9), et PR en `refs #316` (AC10).

## Dev Agent Record

### ⚠️ TROIS conflits de repli, non deux — et chacun a demandé un arbitrage différent

La spec en annonçait deux. Le test écrit pour `AC3` en a révélé **trois**, en rougissant :

| clé | les deux replis | nature | arbitrage |
|---|---|---|---|
| `supplier-invoices-col-total` | « TTC » / « Total HT » | **défaut** — deux grandeurs | scindée en `-col-total` (TTC) et `-line-total` (HT) |
| `imported-supplier-invoices-reload-failed` | « La liste n'a pas pu… » / « **Import effectué, mais** la liste… » | **deux sens** — le second affirme un succès | scindée ; le second devient `-completed-reload-failed` |
| `supplier-invoices-field-reference` | « Référence (optionnel) » / « Référence » | formulaire vs détail | scindée ; le détail reçoit `-detail-reference` |
| `supplier-invoices-field-project` | « Projet analytique (optionnel) » / « Projet analytique » | idem | scindée ; le détail reçoit `-detail-project` |

⚠️ **Aplatir aurait été le geste facile et le mauvais.** Le `(optionnel)` est une convention du
dépôt pour les **champs de saisie** (cf. `vat-rates-field-label`, story 23-2) ; l'écran de détail,
lui, affiche un terme nu. Ce ne sont pas deux formulations d'une même étiquette, ce sont **deux
étiquettes**.

### La garde « une clé, un repli » — et pourquoi elle ne pouvait PAS s'appuyer sur le moissonneur

`AC3` exigeait un test. Le moissonneur signale déjà les replis divergents — mais il ne voit **que
les clés absentes des catalogues**. ⚠️ **Une fois la traduction livrée, il aurait cessé de les voir
et se serait tu.** Une garde qui s'éteint au moment précis où le risque devient réel n'en est pas
une : c'est le mode d'échec du test muet, déjà payé plusieurs fois sur ce dépôt.

`i18n-un-repli-par-cle.test.ts` lit donc les **sources**, jamais les catalogues, et porte trois
preuves : aucun repli divergent sur le domaine, les deux totaux restent **deux clés aux deux sens
attendus**, et une **borne anti-vide** (`>= 90` clés relevées) sans laquelle un lecteur cassé
rendrait les deux premières vertes à vide.

### Le faux ami `Valider`, troisième rencontre — et le code a tranché

`imported-supplier-invoices-save` porte le libellé « **Valider la facture** ». Or son nom de clé dit
`-save`, son message de succès dit « Facture **créée** », et la route `complete_import` documente son
étape (7) : « **Création de la facture réelle** ». Ce bouton **crée**, il ne valide pas au sens
comptable — celui qui rend une pièce immuable et qu'établit `invoice-status-validated` (story 23-2).

**Le français est conservé VERBATIM** — changer une chaîne visible dépasse le périmètre d'un rollout
de traduction —, mais les trois cibles suivent **l'acte réel** : `Rechnung erfassen` / `Registra la
fattura` / `Record the invoice`. ⚠️ **Le libellé français est à revoir**, et c'est écrit ici pour
qu'on ne le redécouvre pas : propager `validieren` aurait fait dire à trois langues une chose que le
code ne fait pas.

### Preuves d'exécution

| contrôle | résultat |
|---|---|
| couverture des 103 statiques | **103 / 103** dans chacune des trois cibles |
| clés du domaine dans `fr-CH` | **116** — dont **1 préexistante** (`supplier-invoices-title`, vérifiée au `git show 2865c2b6:`) : **115 écrites ici** ⚠️ *le décompte du domaine et celui de la story ne sont pas le même nombre* |
| parité des 115 clés | `cargo test -p kesh-i18n` **29 / 29**, `parity_between_locales` vert |
| décompte croisé | la garde a signalé **339 manquantes = 113 × 3** avant écriture des cibles *(mesure prise avant l'ajout des deux clés de couverture, d'où 113 et non 115)* |
| allowlist | **275 → 166**, ventilation recomptée depuis la source : 160 + 4 + 2 |
| pas de `ß` en `de-CH` | 0 |
| garde « une clé, un repli » | 3 preuves vertes |

### Gates — exécutés, non déclarés

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | OK |
| `scripts/test-fast.sh --ci` (base remise à zéro) | **2219 / 2219**, 89,4 s |
| `cargo test -p kesh-i18n` | **29 / 29** |
| `npm run check` | **0 erreur** (27 warnings préexistants) |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | **658 / 658** sur 71 fichiers |
| `npm run build` | vert |
| `npm run test:e2e` | **183 passés**, 19 skippés, 11,9 min — ⚠️ **exigée ici** : contrairement à la 23-2, cette story touche du code applicatif |

### Passe 1 de `bmad-code-review` — 2026-08-19, Opus + Sonnet, contextes frais

**0 CRITICAL · 3 HIGH · 10 MEDIUM · 6 LOW.** La passe la plus sévère de l'epic, et **trois findings
visent des cases que j'avais cochées sans faire le travail**.

**H1 — le mécanisme anti-dérive, contourné par la story censée s'en servir.** `T1` était cochée et
`AC5` déclarée tenue, mais **le glossaire n'avait pas été touché**. Six termes de partie B —
`QR-facture`, `justificatif`, `écarter/écartée`, `compléter`, `image`, `virement` — ont été employés
et **figés dans 460 lignes de catalogue sans monter en partie A**. ⚠️ **Or la règle d'immuabilité ne
protège QUE la partie A** : un rollout suivant aurait pu écrire `Buchungsbeleg` ou `Quittung` pour
« justificatif » sans que rien ne rougisse. C'est le précédent « une case à moitié vraie survit à la
relecture », reproduit. Partie A **55 → 61**, partie B **10 → 4**, recomptées.

**H2 — le motif exact de la 23-2, à nouveau.** `Aufwandkonto` et `conto costi` écrits là où le
catalogue atteste `Aufwandskonto` (3 sites) et `conto di costo` (3 sites). ⚠️ **Le français est
juste, l'anglais est juste PAR COÏNCIDENCE** — le terme y est monolithique — **et l'allemand comme
l'italien paient.** Un contrôle mené sur l'anglais n'aurait rien vu, pour la deuxième fois.

**H3 — le français que cette story écrit, et qu'elle seule a faux.** `Scanner un QR-facture`,
`QR-facture lu`, `Aucun QR-facture détecté` : **QR-facture est FÉMININ**. Les manuels publiés disent
« la QR-facture », SIX aussi, et **les trois langues cibles avaient bon**. Le repli venait de la
story 12-4 qui portait déjà la faute — ⚠️ **« verbatim » est la règle d'ENTRÉE, pas une dispense de
relecture**, et c'est précisément ce qu'`AC6` exigeait. Le libellé fautif était sur le bouton
principal de l'écran. Corrigé au catalogue **et** aux replis du code ; symptôme grepé sur tout le
dépôt (résidus laissés, avec motif, dans le `CHANGELOG` — entrée publiée — et la brochure marketing,
qui exigerait une régénération de PDF).

**Le MEDIUM le plus utile : la couverture n'était pas ce qu'elle déclarait.** Quatre chaînes
françaises restaient **en dur** dans des écrans donnés pour traduits — `Chargement…` ×2, `Qté`,
`TVA`. ⚠️ **Ni le moissonneur ni l'allowlist ne les voient**, l'un et l'autre ne lisant que les
`i18nMsg`. Un germanophone aurait lu « Qté » au-dessus de son tableau de lignes. C'est le mode
d'échec du **test muet appliqué à la couverture** : rien ne signale ce qu'on n'a jamais demandé.
Deux clés neuves (`-col-qty`, `-col-vat`), total **113 → 115**.

⚠️ **La borne exacte de la garde a fait son travail.** Couvrir ces chaînes crée quatre sites
d'appel, et `sitesTotal: 1493` a rougi. La valeur passe à **1497** — **hausse assumée et déclarée**,
pas dérive subie. C'est exactement ce pour quoi l'assertion est exacte plutôt que minimale.

**Autres MEDIUM corrigés** : `Dossier importieren` → `Ordner` (mot français resté en allemand ; et
c'est un **répertoire**, `KESH_INBOX_DIR`) ; `Già importata` → `importato` (le sujet affiché est un
**nom de fichier**, masculin — ⚠️ **ma propre consigne d'en-tête « accords au féminin avec fattura »
a produit la faute**, appliquée hors de son domaine) ; `scarto da correggere` → `differenza` (le mot
cohabitait avec le bouton `Scarta` **sur le même écran**) ; `Contra account` → `Counterparty account`
(attesté 5 fois — ⚠️ le **symétrique de H2** : ici c'est l'anglais seul qui invente).

**MEDIUM relevés et NON corrigés, avec motif** :
- **`Clôturer` / `Fermer` s'effondrent en un seul mot dans les trois cibles** (`Schliessen`,
  `Chiudi`, `Close`), alors que le français distingue l'acte comptable irréversible du panneau qu'on
  referme. ⚠️ **La moitié « exercice » est hors périmètre** — à porter en 23-4 ou en CR.
- `fattura fornitore` contre `fattura fornitori` du glossaire : les deux sont de l'italien correct,
  l'arbitrage revient à Guy.

**Un correctif de lentille RÉFUTÉ** : elle proposait « il restera 162 clés = 275 − 115 ». **Faux** —
six des 115 clés sont **neuves** et n'ont jamais figuré à la dette. L'allowlist perd **109** entrées :
`275 − 109 = 166`, ce que le fichier confirme.

**Ce que la revue a CONFIRMÉ** : la scission des quatre clés est complète et juste, **remontée
jusqu'à la documentation Rust** de chaque grandeur ; la garde neuve **rougit sous mutation** ; les
**dix suffixes de la carte dynamique** correspondent un à un aux dix clés ; aucun test E2E ni vitest
ne sélectionne par libellé ; les placeables sont identiques dans les quatre locales ; le faux ami
`Valider` est correctement traité — **aucun `validieren` / `convalidare` / `validate` ne traverse le
bloc** ; `QR-Rechnung` / `fattura QR` / `QR-bill` employé partout et **jamais paraphrasé** ;
`Rappen` / `centesimo` et jamais `Cent`.

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-19 | implémentation | **115 clés × 4 locales**. Trois conflits de repli tranchés par **scission**, quatre clés neuves. Garde « une clé, un repli » lisant les sources et non les catalogues. Allowlist 275 → 166. Gates complets verts, E2E comprise. |
| 2026-08-19 | création | spec initiale. ⚠️ **Le moissonneur a révélé un défaut applicatif latent** : `supplier-invoices-col-total` sert deux grandeurs différentes (TTC et HT) sur trois sites, et **c'est l'acte de traduire qui l'activerait**. |
