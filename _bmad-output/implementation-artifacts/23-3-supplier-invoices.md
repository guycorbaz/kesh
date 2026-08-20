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
- [x] **T4 — Relecture des 116 libellés `fr-CH`** moissonnés (AC6), consignée — § *AC6* ci-dessous. ⚠️ *Cochée sans consignation à l'implémentation ; le tableau date de la passe 2.*
- [x] **T5 — `fr-CH`** : les 115 clés.
- [x] **T6 — `de-CH`, `it-CH`, `en-CH`** : les 115 × 3, avec contrôle d'homonymie langue par langue (AC7) — § *AC7* ci-dessous. ⚠️ *Cochée sans le contrôle à l'implémentation ; exécuté en passe 2, il a trouvé une homonymie en `de-CH`.*
- [x] **T7 — Allowlist** : 275 → 166 (AC2).
- [x] **T8 — Gates complets, E2E comprise** (AC9), et PR en `refs #316` (AC10).

### Review Findings — passe 2 de `bmad-code-review` (2026-08-20, Haiku ×3)

- [x] [Review][Patch] **H1 n'a été corrigé qu'aux deux tiers — `restauré` et `règlement` restent en partie B** [`docs/i18n-glossaire.md:158-172`] — la passe 1 a promu six termes ; **deux autres termes de partie B sont employés et figés par cette même story sans monter en partie A** : `restauré` (`imported-supplier-invoices-doc-gone:1531`, `-source-doc-gone:1562`) et `règlement` (`supplier-invoices-pay-date:1607`). ⚠️ Les traductions employées sont **justes** (`wiederhergestellt`/`ripristinato`/`restored` ; `Zahlungsdatum`/`Data di pagamento`/`Payment date`, conformes à la réserve « préférer *paiement* ») — **c'est la protection qui manque, pas la traduction**. La règle d'immuabilité ne couvre que la partie A, et **la 23-4 touche `payment-batches`** : le rollout suivant peut écrire `Begleichung` sans que rien ne rougisse. C'est le mode d'échec de H1, à l'identique.
- [x] [Review][Patch] **AC6 et AC7 déclarés tenus, mais rien n'est consigné** [`23-3-supplier-invoices.md:147,149`] — `AC7` exige un contrôle d'homonymie langue par langue **« consigné en tableau »** et `AC6` une relecture du `fr-CH` **« consignée »** ; T4 et T6 sont cochées et **le story file ne contient ni l'un ni l'autre tableau**. ⚠️ **La preuve empirique que le contrôle n'a pas eu lieu est dans la passe 1 elle-même** : H2 (`Aufwandkonto`/`conto costi` contre les formes attestées) et H3 (accord de `QR-facture`) sont exactement ce que ces deux contrôles devaient attraper — ils ont été trouvés par la revue, pas par eux. Troisième récidive du motif « case cochée sans le travail » sur cette story.
- [x] [Review][Patch] **« TROIS conflits de repli » chapeaute un tableau de QUATRE lignes** [`23-3-supplier-invoices.md:155`] — le total contredit sa propre ventilation, et l'écart est propagé à **trois sites** : le titre du §, le Change Log (`:327`) et la ligne 23-3 de `sprint-status.yaml`. Quatre clés portaient deux replis et **quatre** ont été scindées (d'où les « quatre clés neuves » du même Change Log). Trancher : soit « QUATRE conflits », soit dire explicitement que `-field-reference` et `-field-project` comptent pour un seul conflit de même nature.
- [x] [Review][Patch] **« couverture des 103 statiques » est périmé — il y en a 105** [`23-3-supplier-invoices.md:200`] — les deux clés de couverture ajoutées en passe 1 (`-col-qty`, `-col-vat`, présentes et vérifiées dans les quatre locales) portent les statiques de 103 à 105, ce que le périmètre (`:25`) et T4 (`:147`) disent déjà. ⚠️ **La ligne voisine (`:203`) porte sa réserve de périmètre, celle-ci non** — un tableau intitulé « Preuves d'exécution » sous-déclare donc sa propre couverture. Recompté : 116 clés du domaine dans **chacune** des quatre locales.
- [x] ~~[Review][Patch] **`conto di costo` (3 sites) — il y en a 2**~~ ⚠️ **PATCH ANNULÉ en passe 3 — il y en a bien 3** ; le `grep` de la passe 2 était sensible à la casse. [`23-3-supplier-invoices.md:235`] — recompté : `Aufwandskonto` compte bien **3** sites attestants hors story en `de-CH` (`vat-purchase-charge-account`, `-same-account`, `-recoverable-conflict`), mais `conto di costo` n'en a que **2** en `it-CH` (pas d'équivalent de `vat-purchase-charge-account`). Le correctif reste juste ; c'est son compte rendu qui est faux.
- [x] [Review][Patch] **L'angle mort de la garde « une clé, un repli » n'est écrit nulle part** [`frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts:46,50`] — `site.arg?.kind !== 'literal'` et `repli.kind !== 'literal'` écartent **les clés construites par gabarit et les replis non littéraux**, donc toute la famille `imported-supplier-invoices-error-${…}` (`import/+page.svelte:68`). ⚠️ **Aucune divergence n'est possible aujourd'hui** — un seul site construit cette clé, vérifié — mais la garde porte trois preuves écrites et **aucune ne dit ce qu'elle ne voit pas**. Même famille que l'angle mort #255 (chaîne en dur), et le doc-comment est le seul endroit où un rollout suivant le lirait.

**Réfutés par vérification ground-truth — 7 dismiss** *(garde-fou Haiku du `CLAUDE.md`)* : « `652 / 658`, 6 tests skippés » — `652` **n'apparaît nulle part** dans ce fichier ; il est sur la ligne **23-1b** de `sprint-status.yaml`, confusion de ligne classique ; « clé préexistante inexpliquée » — `:201` l'explique et porte déjà un ⚠️, `2865c2b6` étant le tip de la 23-2 ; « replis divergents de `common-loading` » — ⚠️ **ce dismiss était FAUX, rectifié en passe 3** : il y a **9** sites, pas 8, et le neuvième (`InvoiceForm.svelte:888`) porte `'Chargement...'` en trois points ASCII là où les huit autres ont l'ellipse `…`. *La cause est instructive : le `grep` avait bien trouvé la ligne, c'est le `sed` de mise en forme qui l'a masquée en n'affichant que le DERNIER `i18nMsg` de la ligne.* Conséquence réelle **nulle** — `common-loading` est présent dans les quatre catalogues, donc aucun repli n'est jamais rendu — et **hors périmètre** (domaine `invoices`) ; relevé, non corrigé ; « `.d.ts` non gardés » — **aucun** `.d.ts` du dépôt n'appelle `i18nMsg` ; « `readFileSync` échoue silencieusement » — il lève ; et deux LOW de lecture (`:14` lue comme une addition alors qu'elle énumère des décomptes recomptés ; timing des promotions, que `:290` date explicitement de la passe 1).

## Dev Agent Record

### ⚠️ QUATRE conflits de repli, non deux — et chacun a demandé un arbitrage différent

La spec en annonçait deux. Le test écrit pour `AC3` en a fait rougir **quatre** — une clé par
ligne du tableau, chacune portant deux replis :

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

### AC6 — relecture des 116 libellés `fr-CH`, consignée

*(Écrite en **passe 2**. T4 était cochée « consignée » et **rien n'était consigné** : le tableau
ci-dessous est le contrôle réellement mené, pas sa déclaration. Les 116 libellés du domaine ont été
relus contre le code appelant, et **14 d'entre eux ont produit un verdict** — les voici. ⚠️ **Ce
tableau consigne les SIGNALEMENTS, pas un balayage exhaustif de 116 lignes** : la première rédaction
disait « relus un à un », ce qui réclamait plus que ce qu'elle montrait — même motif que les cases
cochées sans le travail que cette story a déjà payé deux fois. Ce qui n'y figure pas n'a pas été
jugé individuellement.)*

| libellé relu | verdict | suite donnée |
|---|---|---|
| `-scan-too-large` = « Image trop volumineuse (**max 15 Mo**) » | ✅ **exact** — `+page.svelte:77` teste `file.size > 15 * 1024 * 1024` | aucune |
| `-err-currency` = « CHF uniquement **en v0.4** » | ⚠️ **FAUX** — le workspace est en **0.10.0** | mention de version **retirée des trois cibles** ; français signalé (cf. ci-dessous) |
| `-save` = « **Valider** la facture » | ⚠️ trompeur — le code **crée** (`complete_import`, étape 7) | français verbatim ; les trois cibles suivent l'acte réel |
| `QR-facture` accordée au masculin (4 libellés) | ⚠️ faux — le terme est **féminin** | corrigé au catalogue et aux replis *(passe 1)* |
| `-mismatch` = « écart à corriger », `-target` = « cible QR » | ✅ minuscule initiale — **convention du dépôt** pour les fragments (29 clés, cf. 23-1b) | aucune |
| `-report-accepted` = « {$n} facture(s) importée(s) » | ⚠️ pluriel par « (s) » là où Fluent sait sélectionner | **hors périmètre** — le français est verbatim ; les trois cibles reprennent la même forme |
| `-field-*` avec « (optionnel) » vs libellés de détail nus | ✅ deux étiquettes, pas deux formulations | scindées, pas aplaties |
| `compte de charge`, `exercice`, `écriture` | ✅ attestés au catalogue (23-2) | relevés, non réinventés |

⚠️ **« v0.4 » — ce que la relecture a coûté, et ce qu'elle a épargné.** Le libellé français annonçait
une version que le produit a dépassée de six mineures, et **cette story venait de le recopier dans
trois langues neuves** : un défaut du français multiplié par quatre, ce qui est exactement le mode
d'échec que l'epic existe pour empêcher. Les trois cibles disent désormais le **fait** (« nur CHF »,
« solo CHF », « CHF only ») et non une promesse datée. Le français reste **verbatim**, comme pour
`Valider` — à corriger par une story qui le touche légitimement.

⚠️ **Le symptôme existe HORS périmètre, sur huit libellés, et n'a pas été touché** :
`bank-import-warnings-unsupported-currency` et `bank-accounts-confirm-archive` annoncent **« v0.1 »**
dans les quatre locales. Domaines `bank-import` et `bank-accounts` — ni l'un ni l'autre n'est un
rollout de cette story. **Relevé au grep `\bv0\.[0-9]\b` sur les quatre catalogues**, à porter en
issue plutôt qu'à corriger ici.

### AC7 — contrôle d'homonymie, langue par langue

*(Écrit en **passe 2**, réécrit en **passe 3** : sa première version avait la bonne méthode et
**la mauvaise portée**.)*

⚠️ **La portée est le tout de ce contrôle, et la passe 2 s'était trompée dessus.** Elle comparait les
116 clés du domaine **entre elles**, ce qui ne peut par construction jamais voir une collision avec
les **1289 autres entrées** du catalogue. Le contrôle est donc mené à **deux portées**, et c'est la
seconde qui trouve quelque chose.

**Portée 1 — à l'intérieur du domaine** *(a : un libellé cible servant plusieurs `fr-CH` ; b : un
`fr-CH` rendu de plusieurs façons)* :

| cible | (a) | (b) | verdict |
|---|---|---|---|
| `de-CH` | **1** — `Rechnung erfassen` servait *« Valider la facture »* **et** *« Enregistrer une facture »* | 0 | ⚠️ **corrigé** — cf. ci-dessous |
| `it-CH` | 0 | 0 | ✅ — distingue déjà `Registra **la** fattura` / `Registra **una** fattura` |
| `en-CH` | 0 | 0 | ✅ — distingue déjà `Record **the** invoice` / `Record **an** invoice` |

⚠️ **Le correctif de la passe 2 était à moitié faux, et c'est la passe 3 qui l'a vu.** Elle avait
écrit `Die Rechnung erfassen` — ce qui **résout la collision de chaîne** (le contrôle repasse au
vert) tout en manquant les deux choses qui comptent : c'était le **seul bouton du domaine à porter un
article**, quand les douze autres sont des infinitifs nus (`Vervollständigen`, `Verwerfen`,
`Rechnung stornieren`, `Rechnung bezahlen`, `Speichern`…) ; et **l'opposition n'était pas rendue**,
un infinitif-objet nu étant en allemand la forme non marquée et non un indéfini.

⚠️ **Le français n'oppose PAS un article, il oppose DEUX VERBES** — « **Valider** la facture » contre
« **Enregistrer** une facture ». L'italien et l'anglais ont choisi un verbe unique plus un article,
ce qui leur est idiomatique ; l'allemand devait donc suivre le **français**, conformément à
l'arbitrage du 2026-08-19. D'où **`Rechnung übernehmen`** — le terme usuel pour reprendre une pièce
importée dans le système, absent de tous les catalogues *(vérifié : aucune collision créée)*, et
fidèle au registre de l'infinitif nu.

**Portée 2 — chaque libellé du domaine contre le catalogue ENTIER de sa locale** (1405 clés). Neuf
libellés du domaine partagent leur forme cible avec une clé hors domaine dont le `fr-CH` diffère.
⚠️ **Dans les neuf cas, le membre du domaine est JUSTE** — la collision vient de l'autre côté, écrit
avant cette story. Deux d'entre elles sont nuisibles :

| forme cible | français effondrés | jugement |
|---|---|---|
| `Schliessen` / `Chiudi` / `Close` | **Clôturer** (`fiscal-year-close-button`) et **Fermer** (`supplier-invoices-form-close`, `api-keys-actions-close`) | ⚠️ **NUISIBLE** — la clôture d'exercice est un acte comptable **irréversible**, le panneau qu'on referme ne l'est pas. Hors périmètre : le membre du domaine (*Fermer*) est correct |
| `Speichern` / `Salva` / `Save` | **Enregistrer** (10 clés) et **Valider** (`journal-entry-form-submit`) | ⚠️ **NUISIBLE** — le faux ami `Valider`, **4ᵉ rencontre** sur cet epic. Hors périmètre : le membre du domaine (*Enregistrer*) est correct |
| `Betrag` ← Montant / Total · `Beschreibung` ← Description / Libellé · `Zurück` ← Retour / Précédent | | ⚠️ à surveiller — distinctions françaises réelles, mais aucune n'est un acte irréversible |
| `Menge` ← Qté / Quantité · `Status` ← Statut / État · `Zahlungsdatum` ← Date de paiement / Date de règlement · `Scadenza` ← Échéance / Date d'échéance | | ✅ bénin — synonymes français, et le glossaire a précisément tranché que les cibles disent *paiement* |

⚠️ **Ces deux collisions nuisibles ne sont PAS créées par la 23-3 et ne sont pas corrigées ici** —
elles vivent dans `fiscal-year` et `journal-entries`. **Tracées le 2026-08-20** : [KF-041] (#323)
pour `Clôturer`/`Fermer`, [KF-042] (#324) pour le faux ami `Valider`.

⚠️ **Pourquoi l'issue et pas la note.** `Clôturer`/`Fermer` était **déjà connu à l'implémentation**,
noté « à porter en 23-4 ou en CR » — et **rien n'a suivi**. Une note de story ne se relit qu'en
rouvrant la story ; l'Issue Tracking Rule fait de GitHub la source de vérité unique **précisément
pour ce mode d'échec**. Le contrôle de la passe 3 a redécouvert de zéro un défaut que la story avait
elle-même signalé quatre commits plus tôt.

### Preuves d'exécution

| contrôle | résultat |
|---|---|
| couverture des **105** statiques | **105 / 105** dans chacune des trois cibles *(103 à l'implémentation ; `-col-qty` et `-col-vat`, ajoutées en passe 1, portent le compte à 105)* |
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

⚠️ **Et ce correctif était lui-même incomplet — six termes promus sur huit.** `restauré`
(`-doc-gone`, `-source-doc-gone`) et `règlement` (`-pay-date`) étaient employés eux aussi et sont
restés en partie B une passe de plus, trouvés en **passe 2**. La cause remonte à la spec : son
tableau de préalables (§ *Termes à ne pas réinventer*) **énumérait quatre termes** de partie B là
où le catalogue en emploierait huit — une liste dressée de mémoire, jamais un grep du glossaire
contre les libellés. Partie A **61 → 63**, partie B **4 → 2**, recomptées depuis les tableaux.

**H2 — le motif exact de la 23-2, à nouveau.** `Aufwandkonto` et `conto costi` écrits là où le
catalogue atteste `Aufwandskonto` (**3** sites) et `conto di costo` (**3** sites). ⚠️ *La passe 2
avait « corrigé » ce compte en **2**, sur un `grep` **sensible à la casse** qui ratait
`vat-purchase-charge-account = **C**onto di costo` (it-CH:286). Le chiffre d'origine était juste ;
c'est sa correction qui était fausse. Recompté en passe 3 avec `grep -i`.* ⚠️ **Le français est
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
- ~~`fattura fornitore` contre `fattura fornitori`~~ — **tranché par Guy le 2026-08-19** :
  *« je ne parle pas italien : dans le doute, laisse le français »*. ⚠️ **Le critère est
  vérifiable, là où « quel italien est le meilleur » ne l'était pas** — et il tranche net :
  `Facture fournisseur` → `Fattura fornitore`, `Factures fournisseurs` → `Fatture fornitori`.
  **Les catalogues étaient déjà justes ; c'est la ligne du GLOSSAIRE qui était bancale**, donnant
  le lemme français au singulier et l'italien au pluriel — une forme hybride correspondant à ni
  l'un ni l'autre. Corrigée, et la règle « en cas de doute, suivre le français » est écrite au
  préambule du glossaire pour les rollouts suivants.

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

## Reprise — où en est la story, et par quoi commencer

*(Écrit à l'interruption de séance du 2026-08-19 au soir, à la demande de Guy.)*

**La story est implémentée et a passé UNE passe de revue.** Tous les gates sont verts, E2E comprise
(183 passés). L'arbre est propre, les quatre commits sont poussés.

**Par quoi reprendre, dans l'ordre :**

1. ⚠️ **Passe 2 de `bmad-code-review`** — la règle l'appelle, la passe 1 ayant rendu **3 HIGH**.
   La braquer sur **les six promotions au glossaire** : c'est la remédiation la plus lourde de la
   passe 1, donc la plus susceptible d'avoir son propre défaut. Sur ce dossier, **huit passes sur
   neuf ont trouvé une régression du patch précédent**. Modèle : Sonnet ou Haiku, contexte frais.
2. **PR de la 23-3**, en `refs #316` et **non** `closes` — il restera 166 clés.
3. **23-4** (93 clés : `settings`, `payment-batches`, `onboarding`, 4 `nav-*`).

**Ce qui attend un arbitrage de Guy, et que je n'ai pas tranché :**

- ⚠️ **Trois PR empilées.** #320 (socle, 23 commits), #322 (parité, `closes #283`, 29 commits **dont
  ceux de #320**), et la branche 23-3 par-dessus. Chaque PR contient les précédentes, et cela
  grossit tant que rien n'est mergé. Recommandation : **merger #320 d'abord**, puis rebaser.
- ~~**`Clôturer` / `Fermer`** s'effondrent en un seul mot dans les trois cibles~~ — ✅ **tracé en
  [KF-041] (#323)** le 2026-08-20, avec [KF-042] (#324) pour le faux ami `Valider` que le contrôle
  élargi a trouvé en même temps. Hors périmètre de la 23-3 : le membre de son domaine est correct.
- **Le libellé français `imported-supplier-invoices-save = « Valider la facture »`** est trompeur :
  le code **crée** la facture. Conservé verbatim ici (hors périmètre), les trois cibles suivent
  l'acte réel. À corriger dans une story qui touche légitimement le français.

**Résidus laissés avec motif** (accord de `QR-facture`, corrigé au catalogue et au code) : le
`CHANGELOG` — entrée **publiée** — et `docs/manual/fr/marketing-brochure.tex`, qui exigerait une
**régénération de PDF**. Ni l'un ni l'autre n'est un oubli.

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-20 | **passe 3** de `bmad-code-review` | **0 CRITICAL · 2 HIGH · 3 MEDIUM · 1 LOW** (Sonnet ×3, contextes frais). Trend : 3H/10M/6L → 2H/2M/2L → **2H/3M/1L** — ⚠️ **la sévérité ne redescend pas, et les six findings visent TOUS les correctifs de la passe 2**, aucun le code livré. **HIGH-1 : `Die Rechnung erfassen` était un demi-correctif** — il faisait repasser le contrôle au vert en résolvant la collision de chaîne, mais c'était le seul bouton du domaine à porter un article et l'opposition n'était pas rendue. **Le français oppose deux VERBES, pas un article** → `Rechnung übernehmen`. **HIGH-2 : le contrôle AC7 avait la bonne méthode et la mauvaise portée** — 116 clés comparées entre elles, aveugles aux 1289 autres du catalogue. Refait à deux portées : il trouve **deux collisions nuisibles** (`Schliessen`/`Chiudi`/`Close` ← *Clôturer* **et** *Fermer* ; `Speichern`/`Salva`/`Save` ← *Enregistrer* **et** *Valider*), dont aucune n'est créée par cette story. ⚠️ **Les deux MEDIUM sont des comptes rendus que la passe 2 croyait CORRIGER** : `conto di costo` en a bien **3** (son `grep` était sensible à la casse — le chiffre d'origine était juste, sa correction fausse), et le dismiss `common-loading` disait 8 sites pour **9**, un `sed` d'affichage ayant masqué la ligne divergente que le `grep` avait trouvée. **MEDIUM-3** : AC6 réclamait « relus un à un » pour 14 verdicts sur 116 — borné. |
| 2026-08-20 | **passe 2** de `bmad-code-review` | **0 CRITICAL · 2 HIGH · 2 MEDIUM · 2 LOW** (Haiku ×3, diff aplati de la story seule). Trend : 3H/10M/6L → **2H/2M/2L**. ⚠️ **Le HIGH est encore une régression du patch précédent — H1 n'avait promu que SIX termes sur HUIT** : `restauré` et `règlement` étaient employés eux aussi. Partie A **61 → 63**, partie B **4 → 2**, recomptées depuis les tableaux. ⚠️ **Le second HIGH est le motif de H1 appliqué aux contrôles** : `AC6` et `AC7` exigeaient une consignation, T4 et T6 étaient cochées, **rien n'était consigné**. Les deux contrôles ont été **réellement exécutés en passe 2** et **chacun a trouvé un défaut que personne n'avait vu** — `AC7` : `Rechnung erfassen` servait deux libellés français distincts en `de-CH`, là où l'italien et l'anglais distinguaient déjà par l'article (→ `Die Rechnung erfassen`) ; `AC6` : le libellé `-err-currency` annonçait **« v0.4 »** alors que le workspace est en **0.10.0**, et cette story venait de le recopier dans trois langues neuves — mention retirée des cibles, français signalé. **7 findings réfutés au grep** (garde-fou Haiku), dont « 652/658 » qui vient de la ligne **23-1b** du sprint-status. |
| 2026-08-19 | implémentation | **115 clés × 4 locales**. Quatre conflits de repli tranchés par **scission**, quatre clés neuves. Garde « une clé, un repli » lisant les sources et non les catalogues. Allowlist 275 → 166. Gates complets verts, E2E comprise. |
| 2026-08-19 | création | spec initiale. ⚠️ **Le moissonneur a révélé un défaut applicatif latent** : `supplier-invoices-col-total` sert deux grandeurs différentes (TTC et HT) sur trois sites, et **c'est l'acte de traduire qui l'activerait**. |
