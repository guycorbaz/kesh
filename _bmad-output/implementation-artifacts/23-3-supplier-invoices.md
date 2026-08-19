# Story 23.3 : `supplier-invoices` — 109 clés, et un défaut que la traduction aurait activé

## Status

draft

Deuxième rollout de l'Epic 23, et le plus gros : **109 clés** sur les 265 restantes de [#316].

## Story

**En tant qu'** utilisateur de Kesh dans une autre langue que le français,
**je veux** que le domaine des factures fournisseurs — liste, détail, import de QR-factures,
complétion — s'affiche dans ma langue,
**afin de** ne pas voir un quart de l'application en français sans que rien ne me le signale.

## Périmètre

**Dedans** : les **99 clés statiques** moissonnables de `supplier-invoices` et
`imported-supplier-invoices`, plus les **10 clés de la famille dynamique**
`imported-supplier-invoices-error-*`, écrites dans les **quatre** locales ; le retrait des
109 entrées correspondantes de `frontend/src/lib/shared/i18n-dette-connue.ts`.

⚠️ **Différence majeure avec la 23-2** : là, `fr-CH` était la **source** et ne bougeait pas. Ici les
clés manquent des **quatre** catalogues — le français doit donc être écrit lui aussi, moissonné
depuis les replis en dur du code, **puis relu** avant d'être figé.

**Dehors** : les 156 clés des rollouts 23-4 à 23-6 ; le comportement applicatif, **à une exception
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

⚠️ `imported-supplier-invoices-error-unknown`, lui, **est** un littéral et compte déjà parmi les 99.

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

1. **AC1** — les **109** clés existent dans les **quatre** locales, valeur non vide.
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
10. **AC10** — [#316] n'est **pas** fermée (il restera 156 clés) ; la PR porte `refs #316`.

## Tasks

- [ ] **T1 — Arbitrage** des termes de partie B (AC5). **Bloquant.**
- [ ] **T2 — Scission de `supplier-invoices-col-total`** et son test (AC3), avant toute traduction.
- [ ] **T3 — Arbitrage des deux autres conflits** (AC4).
- [ ] **T4 — Relecture des 99 libellés `fr-CH`** moissonnés (AC6), consignée.
- [ ] **T5 — `fr-CH`** : les 109 clés.
- [ ] **T6 — `de-CH`, `it-CH`, `en-CH`** : les 109 × 3, avec contrôle d'homonymie langue par langue (AC7).
- [ ] **T7 — Allowlist** : 275 → 166 (AC2).
- [ ] **T8 — Gates complets, E2E comprise** (AC9), et PR en `refs #316` (AC10).

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-19 | création | spec initiale. ⚠️ **Le moissonneur a révélé un défaut applicatif latent** : `supplier-invoices-col-total` sert deux grandeurs différentes (TTC et HT) sur trois sites, et **c'est l'acte de traduire qui l'activerait**. |
