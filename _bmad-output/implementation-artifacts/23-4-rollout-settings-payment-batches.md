# Story 23.4 : le rollout `settings` + `payment-batches` + `onboarding`, et les deux clés qui portent deux sens

Status: review

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
périmètre. ⚠️ **Il en manquait un troisième, que seule la passe 3 a trouvé** : `onboarding-next`,
invisible du moissonneur parce que **déjà traduit**. Le compte du périmètre est donc de **TROIS**
divergences, pas deux.

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

### ⚠️⚠️ `onboarding-next` — le TROISIÈME conflit du périmètre, et il est DÉJÀ CONSOMMÉ

| site | repli écrit | ce qui s'affiche |
|---|---|---|
| `onboarding/+page.svelte:359` | `'Continuer'` | **Continuer** |
| `onboarding/+page.svelte:393` — bouton d'enregistrement du compte bancaire | `'Enregistrer'` | ⚠️ **Continuer** |

⚠️ **Ce n'est pas un défaut latent : il est ACTIF aujourd'hui, dans les quatre langues.** La clé est
déjà au catalogue (`Continuer` / `Weiter` / `Continua` / `Continue`), donc le repli `'Enregistrer'`
est **mort** : le bouton qui doit enregistrer les coordonnées bancaires dit « Continuer ».

⚠️ **Et voici pourquoi mon relevé ne l'a pas vu — c'est un angle mort de MÉTHODE, pas un oubli.**
Le moissonneur **ne relève que les clés ABSENTES des catalogues**. La 23-3 l'a écrit noir sur blanc
dans le docstring de la garde qu'elle a livrée : *« il ne voit QUE les clés absentes des catalogues.
Une fois la traduction livrée, il cesserait de les voir et se tairait. »* Or **`onboarding` est
partiellement traduit depuis la 23-1b** — ce que T6 dit en toutes lettres. J'ai fondé le périmètre
sur un outil dont je citais moi-même la limite deux paragraphes plus loin.

**Conséquence sur la méthode, et elle vaut pour les rollouts suivants** : sur un domaine
**partiellement traduit**, le relevé de référence n'est pas le moissonneur mais **la garde
« une clé, un repli »**, qui lit les sources sans filtrer sur le catalogue.

**Scission** : `onboarding-next` (« Continuer ») reste ; le site `:393` prend **`onboarding-save`**
(« Enregistrer »). ⚠️ **Cette clé est neuve et s'ajoute au décompte** — cf. AC1.

### ⚠️ Le quatrième conflit est HORS PÉRIMÈTRE, et il faut le laisser

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

## ⚠️ Dérogation à la règle de splitting préventif — arbitrage de Guy, 2026-08-21

La § *Règle de splitting préventif* de `CLAUDE.md` traite une **remontée de sévérité** entre deux
passes comme un signal de non-convergence. Le trend de `validate` est
`1C/2H/3M` → `0C/1H/2M` → **`1C/7H/7M`** : le critère est atteint.

**Arbitrage rendu : pas de split.** Motif, et il est vérifiable : **les quinze findings de la
passe 3 sont des OMISSIONS PAR RAPPORT À UN PATRON EXISTANT**, pas des défauts nés de la complexité
de cette story. Leur correctif consiste à recopier des critères déjà rédigés dans la 23-3 et la
23-3b — assertion anti-fusion, AC de relecture, AC5-bis, contrôle d'homonymie à deux portées, borne
`sitesTotal`. Une story trop large produit des findings **nouveaux et enchevêtrés** ; celle-ci a
produit des findings **connus et isolables**.

⚠️ **Le CRITICAL lui-même ne plaide pas pour le split** : il vient d'un angle mort d'outillage
— le moissonneur aveugle aux domaines déjà traduits — qui aurait frappé la story quelle que soit
sa taille, et qui frappera les rollouts suivants si la leçon n'est pas écrite. Elle l'est
désormais, au § *Ce que le relevé a trouvé*.

**Risque accepté** : si une passe de `bmad-code-review` remonte à son tour une sévérité égale ou
supérieure sur cette story, cette dérogation devra être rouverte.

## Critères d'acceptation

1. **AC1 — le périmètre est écrit dans les QUATRE catalogues, et DEUX grandeurs distinctes le
   décrivent.** ⚠️ **Les confondre est le premier piège de cette story, et il vient de sa propre
   spec** *(relevé en passe 1 de `validate`, où la ventilation contredisait AC2)* :

   | grandeur | valeur | pourquoi elle diffère |
   |---|---|---|
   | **clés écrites au catalogue** | **96** | les **trois** clés scindées par AC2 en produisent six |
   | **décrément de l'allowlist** | **93** → **166 − 93 = 73** | les trois clés NEUVES n'y ont jamais figuré |

   ⚠️ **96 et non 95** : la passe 3 a trouvé une **troisième** divergence (`onboarding-next`), dont
   la scission ajoute une clé. ⚠️ **`onboarding-save` n'entre PAS dans le décrément d'allowlist** —
   la clé mère y est, la fille non.

   Ventilation des 93 entrées d'allowlist, relevée au moissonneur : `payment-batches` **30**,
   `settings` **55** *(dont **28** pour le seul écran `settings/projects`, et 25 via le relais `msg`
   de l'écran des modèles d'e-mail)*, `onboarding` **4**, plus les **4 `nav-*`** de l'inventaire —
   que le moissonneur ne voit pas, leurs clés étant portées par une table de données.

   ⚠️ **Recompter les DEUX depuis la source**, et ne jamais ajuster l'une pour la faire coïncider
   avec l'autre : c'est ce qui pousserait à ne scinder qu'à moitié, et à réintroduire le défaut
   qu'AC2 existe pour fermer. *(Une lentille de la passe 1 proposait 71 pour l'allowlist : **c'est
   faux**, et l'erreur est instructive — elle décomptait des clés qui n'y ont jamais été inscrites.)*
2. **AC2 — les TROIS clés à double sens sont SCINDÉES**, et **la fusion ne peut pas revenir.**
   ⚠️ **Un test le couvre**, sur le modèle littéral de la 23-3 (*« les deux totaux restent DEUX
   clés »*) : la garde générique « aucune clé ne porte deux replis » resterait **verte** si
   quelqu'un refusionnait — une clé unique à repli unique ne diverge pas. La preuve du run rouge
   d'AC3 est une preuve *d'aller*, jamais *de retour*. L'assertion porte les **six** sens attendus :

   | clé | repli attendu |
   |---|---|
   | `payment-batches-col-total` | `Total` |
   | `payment-batches-line-amount` | `Montant` |
   | `payment-batches-col-date` | `Exécution` |
   | `payment-batches-detail-date` | `Date d'exécution` |
   | `onboarding-next` | `Continuer` |
   | `onboarding-save` | `Enregistrer` |
3. **AC3 — la garde « une clé, un repli » couvre `payment-batches-` ET `onboarding-`**, et sa borne
   `CLES_RELEVEES` est **recomptée**, jamais ajustée. ⚠️ **Elle doit rougir sur les TROIS divergences
   AVANT leur correction** : la sortie brute du run rouge est collée au Dev Agent Record. Sans cette
   preuve, une extension vide passe tous les autres AC.
   ⚠️ **`onboarding-` n'était pas prévu, et c'est le domaine qui portait le défaut ACTIF** : une
   garde étendue au seul domaine où l'on savait déjà quoi chercher n'aurait rien appris.
   ⚠️ **AC5-bis (repris de la 23-3b, mot pour mot)** : *tout site relevé par la garde et absent du
   tableau est soit corrigé ici, soit consigné au Dev Agent Record avec sa raison ET une issue
   GitHub — jamais inscrit à une allowlist.* **Le tableau est un plancher, pas un inventaire clos** :
   la 23-3 annonçait deux conflits et son test en a fait rougir **quatre**. ⚠️ Le relevé de la
   passe 3 en montre déjà d'autres hors périmètre — `dunning-edit`, `dunning-conflict`,
   `error-unexpected`, `loading` : les tracer, ne pas les traiter ici.
4. **AC4 — le verbe « générer » disparaît du domaine** au profit de « créer », sur les trois sites
   du tableau ci-dessus. Contrôle, **exécuté par T4 et sa sortie collée** :
   `grep -rn "[Gg]énér" frontend/src/routes/\(app\)/payment-batches/`.
   ⚠️ **Le critère porte sur les LIBELLÉS AFFICHÉS, pas sur les commentaires de code.** Le grep
   remonte aujourd'hui un commentaire de `[id]/+page.svelte` qui documente pourquoi un test E2E
   attendait `/Généré/i` — il parle bien de la création d'un lot, **au passé**, et c'est une
   explication qu'une revue antérieure a payée. **Ne pas la réécrire pour faire disparaître le
   bruit.**
5. **AC5-zéro — les replis `fr-CH` sont RELUS AVANT d'être figés, et la relecture est consignée.**
   ⚠️ **Ils sont écrits par des développeurs dans le feu du code, pas rédigés** — et cette story en
   fige 96 tout en changeant le français en quatre endroits (AC4, T1). La 23-3 avait cet AC ; il a
   trouvé qu'un message annonçait une limitation « en v0.4 » sur un produit en 0.10.0, **et que la
   story venait de le recopier dans trois langues neuves** : un défaut du français multiplié par
   quatre, ce que l'epic existe pour empêcher. ⚠️ **La relecture vient AVANT l'écriture, pas après**
   — sinon un français faux part dans quatre langues avant d'être vu.
   ⚠️ **Greper le SYMPTÔME, pas une liste** : confronter les 96 valeurs aux parties A **et** B du
   glossaire. La 23-3 avait promu six termes sur huit parce qu'elle relisait sa propre liste au lieu
   de balayer ce que le catalogue employait.
6. **AC5 — le seul terme non attesté est tranché PAR GUY avant écriture** — `Référence message` —,
   et les termes **relevés mais non promus** montent en partie A avec leur clé attestante.
   ⚠️ **`lot` en fait partie** : la 23-3b l'a employé et figé (`Bereits in einem erstellten Stapel`)
   **sans le promouvoir** — le défaut que le glossaire documente sur lui-même, ouvert depuis une
   story. ⚠️ **Promouvoir n'est pas arbitrer** : `lot` et `NPA` sont relevés, ils ne demandent
   aucune décision.
7. **AC6 — les DEUX bornes des gardes sont recomptées et DÉCLARÉES**, jamais ajustées en silence :
   `CANDIDATES_ATTENDUES` de `i18n-libelle-en-dur.test.ts` *(si le rollout crée ou supprime une
   fonction de libellé)*, et **`sitesTotal` de `i18n-keys.test.ts`, base actuelle 1525**.
   ⚠️ **`sitesTotal` VA rougir**, et c'est voulu : router les libellés en dur d'AC6-bis crée autant
   de sites d'appel. Précédent exact en 23-3 — `1497 → 1502`, « deux *Chargement…*, un *Qté*, un
   *TVA* ». La borne exacte existe pour qu'un écart se **recompte**, pas s'ajuste.
8. **AC6-bis — les libellés français EN DUR des écrans du périmètre sont routés vers une clé.**
   ⚠️ **Aucune garde ne les voit** : le préambule de `i18n-libelle-en-dur.test.ts` le dit lui-même —
   *« nœud de texte de balisage → ❌ non »*. AC6 exige que cette garde reste verte : **elle le
   restera sans rien prouver.** Relevé de la passe 3, cinq sites, **et la clé existe déjà dans les
   quatre locales pour quatre d'entre eux** :

   | site | littéral | clé disponible |
   |---|---|---|
   | `payment-batches/+page.svelte:208` | `Chargement…` | `common-loading` |
   | `payment-batches/[id]/+page.svelte:78` | `Chargement…` | `common-loading` |
   | `onboarding/+page.svelte:218` | `Chargement...` *(points ASCII)* | `common-loading` |
   | `onboarding/+page.svelte:384` | ` (optionnel)` | à router |
   | `settings/+page.svelte:200` | `<title>Paramètres - Kesh</title>` | `settings-title`, **et la ligne 204 l'utilise déjà** |

   ⚠️ **Le dernier est le jumeau exact du `Qté` de la 23-3** : deux sites, un traduit, l'autre en
   dur, la clé disponible. Un germanophone lit aujourd'hui « Paramètres - Kesh » dans son onglet
   au-dessus d'un `<h1>` qui dit « Einstellungen ».
9. **AC7 — aucun test ne verrouille le français.** Les **deux** greps de contrôle de la 23-3b, plus
   celui qu'elle a dû inventer en cours de route :
   `grep -rnE "toBe\('[A-ZÀ-Ü]|toContain\('[a-zà-ü]" frontend/src/lib/features/*/*helpers.test.ts`
   `grep -rnE "getByRole\(.*name: '[A-ZÀ-Ü]|getByText\('[A-ZÀ-Ü]" frontend/tests/e2e/`
   `grep -rnE "toContainText\(/[A-ZÀ-Üa-zà-ü]|has-text\(\"[A-ZÀ-Ü]" frontend/tests/e2e/`
   `grep -rnE "toHaveText\('[A-ZÀ-Üa-zà-ü]|toContainText\('[A-ZÀ-Üa-zà-ü]|name: /[A-ZÀ-Üa-zà-ü]" frontend/tests/e2e/`
   ⚠️ **Le troisième est né d'un verrou que les deux premiers ne pouvaient pas voir** — une
   assertion de *contenu* sur un sélecteur déjà stable. ⚠️ **Le QUATRIÈME est né en passe 3, et il
   trouve NEUF verrous réels sur des clés de ce périmètre** : `toHaveText('Défaut')` et
   `toHaveText('Personnalisé')` dans `email-templates.spec.ts` (7 occurrences), et
   `toContainText('Projets')` dans `projects.spec.ts:30`. Les trois premiers greps ne connaissaient
   ni `toHaveText('…')` ni `toContainText('…')` **avec quote** — seulement avec regex.
   ⚠️ **Conclure du silence d'un grep est le défaut, pas le grep** : T10 confronte son relevé aux
   93 clés, il ne déduit rien d'une sortie vide. ⚠️ **Et la liste n'est toujours pas close** :
   cf. [KF-043 (#326)](https://github.com/guycorbaz/kesh/issues/326), la suite E2E ne tourne qu'en
   français.
10. **AC8 — gates complets verts, E2E comprise**, PR en `refs #316`. ⚠️ **#316 reste OUVERTE** —
   elle ne se ferme qu'à la 23-6, qui vide l'allowlist.

## Tasks

- [x] **T1 — Arbitrage de Guy sur `Référence message` — RENDU le 2026-08-21, et promotion des termes relevés** — AC5.
      ⚠️ **Bloquant pour ce seul terme**, et à faire AVANT toute écriture au catalogue : la valeur
      sera figée dans quatre catalogues. ⚠️ **La liste est passée de QUATRE termes à UN en passe 1
      de `validate`** : `virement` était déjà en partie A, `NPA` attesté dans les quatre locales par
      `field-postal-code`, `lot` relevé et seulement à promouvoir. Ne jamais inventer ; si un terme
      neuf apparaît en cours de route, le proposer au Record et **s'arrêter**.
- [x] **T2 — Étendre la garde « une clé, un repli » à `payment-batches-`** — AC3, et **coller sa
      sortie ROUGE** : c'est la seule preuve qu'elle voit les deux divergences. ⚠️ **Cette tâche
      vient AVANT le split, et la numérotation le dit désormais** : dans l'autre ordre, les
      divergences ont disparu du code, la garde ne peut plus rougir, et la preuve qu'AC3 exige
      devient impossible à produire honnêtement. *(L'ordre était contredit par la numérotation en
      passe 1 de `validate` — un développeur suivant les numéros aurait cassé AC3 sans le voir.)*
- [x] **T3 — Scinder les deux clés à double sens** — AC2. ⚠️ **AVANT d'écrire quoi que ce soit aux
      catalogues** : entrer `payment-batches-col-total` telle quelle figerait le défaut.
      **Les quatre clés d'arrivée, nommées** *(la clé de date ne l'était pas — passe 2)* :

      | clé | sites | repli |
      |---|---|---|
      | `payment-batches-col-total` *(inchangée)* | liste `:222`, fiche `:98` | `'Total'` |
      | **`payment-batches-line-amount`** *(neuve)* | fiche `:122` | `'Montant'` |
      | `payment-batches-col-date` *(inchangée)* | liste `:220` | `'Exécution'` |
      | **`payment-batches-detail-date`** *(neuve)* | fiche `:96` | `"Date d'exécution"` |

      ⚠️ **Ce sont les deux clés NEUVES qui font l'écart 93 → 95** d'AC1. Contrôle après split :
      `grep -rn "payment-batches-col-total\|payment-batches-col-date" frontend/src/` doit rendre
      **trois** sites, pas cinq.
- [x] **T4 — `payment-batches` : 32 clés × 4 locales** *(30 d'allowlist + les 2 issues du split)* —
      AC1, **AC4**, **AC6**. ⚠️ **Exécuter le grep d'AC4 et COLLER sa sortie** — un critère qu'aucune
      tâche n'exécute est une affirmation invérifiable. ⚠️ **AC6** : vérifier si le rollout crée ou
      supprime une fonction `*Label`/`*Text`/`*Display` ; si oui, recompter `CANDIDATES_ATTENDUES`
      et le **déclarer**, sinon écrire qu'aucune ne l'a été. Le gate le vérifie mécaniquement, mais
      rien n'oblige à le dire — et c'est le dire qui manque.
- [x] **T5 — `settings` : 55 clés × 4 locales** — AC1. ⚠️ **25 passent par le relais `msg`** de
      l'écran des modèles d'e-mail : le littéral vit au site `msg(`, pas au site `i18nMsg(`.
      `findRelays` les voit ; un grep naïf de `i18nMsg(` ne les verrait pas.
      ⚠️ **« settings » est un DOSSIER, pas un préfixe de clé, et cette confusion a une victime
      documentée** : une lentille de la passe 2 a compté par préfixe, trouvé **87** clés au lieu de
      93, et conclu à une erreur d'arithmétique de la spec. Elle avait tort — mais elle prouve le
      piège. **Ventilation réelle des 55, relevée au moissonneur :**

      | préfixe | clés |
      |---|---:|
      | `projects-*` *(écran des projets analytiques)* | **28** |
      | `email-templates-*` *(**20 via relais**, 0 directe — recompté en passe 3)* | **20** |
      | `settings-*` | 1 |
      | `saving-*`, `save-*`, `cancel-*`, `creating-*`, `create-*`, `closing-*` | **6** |

      ⚠️ **Les six dernières sont des clés GÉNÉRIQUES**, sans préfixe de domaine — un développeur
      qui cherche « les 55 clés de settings » par préfixe en ratera exactement six. Elles sont
      propres à ce dossier aujourd'hui *(vérifié : tous leurs sites y sont)*, mais leur nom ne le
      dit pas : **candidates naturelles à une homonymie future**, à surveiller au contrôle d'AC7.

      ⚠️ **28 des 55 — plus de la moitié — sont l'écran `settings/projects`**, que la première
      rédaction ne nommait nulle part. Il porte un vocabulaire de **hiérarchie** (« Projet parent »,
      « — Aucun (projet racine) ») dont le précédent est au catalogue et **n'est pas un calque** :
      cf. le § *Les termes*.
- [x] **T6 — `onboarding` : 4 clés × 4 locales** — AC1. ⚠️ Le dossier est **partiellement traduit
      depuis la 23-1b** (8 libellés faits, 4 messages restants) : cette tâche referme l'écart.
- [x] **T7 — Les 4 `nav-*` de l'inventaire** — AC1 : `nav-credit-notes`, `nav-email-templates`,
      `nav-projects`, `nav-supplier-invoices-import`. ⚠️ **Ce sont des clés DÉJÀ CÂBLÉES dont la
      traduction manque** — à ne pas confondre avec les 9 libellés de navigation que la 23-3b a
      sortis du français en dur. Les deux ensembles sont **disjoints**, vérifié.
- [x] **T8 — Décrémenter l'allowlist de 93** — AC1, borne recomptée depuis le fichier.
- [x] **T9 — Glossaire** : promotion des termes tranchés, dont **`lot`** — AC5.
- [x] **T9-bis — Relecture des replis français ET contrôle d'HOMONYMIE, langue par langue** — AC7.
      ⚠️ **La 23-3 avait ces deux contrôles, et ils ont produit KF-041 et KF-042** — deux homonymies
      qu'aucune garde automatique ne peut voir. La première rédaction de cette spec ne les demandait
      pas. Pour chaque locale cible, relever les valeurs qui apparaissent **deux fois** :

      ```sh
      for l in de-CH it-CH en-CH; do
        grep -hE "^(payment-batches-|projects-|email-templates-|settings-|onboarding-|nav-|save|cancel|creat|clos)" \
          crates/kesh-i18n/locales/$l/messages.ftl |
          sed 's/^[^=]*= //' | sort | uniq -d
      done
      ```

      ⚠️ **`sav` et non `save` dans la classe ci-dessus** : `^save` n'atteint pas `saving`, et le
      couple à risque est précisément celui-là (`Speichern` / `Speichern…`). *Un préfixe de contrôle
      se teste sur les clés qu'il prétend couvrir avant d'être écrit — c'est « greper la valeur, pas
      la formulation » appliqué au grep lui-même.*

      ⚠️ **DEUX PORTÉES, et c'est la seconde qui trouve quelque chose.** La commande ci-dessus est la
      portée 1 — les clés du périmètre entre elles. **Elle ne peut pas voir une collision avec les
      ~1400 autres clés du catalogue**, et c'est exactement la faute que la passe 3 de la 23-3 a
      classée HIGH : *« la bonne méthode et la MAUVAISE PORTÉE — 116 clés comparées entre elles,
      aveugles aux 1289 autres »*. **KF-041 et KF-042 sont nées de la seconde portée.** Cas concret
      ici : cette story écrira `payment-batches-form-close = 'Fermer'`, et `fiscal-year-status-closed`
      dit déjà « Clôturé » → `Geschlossen` — **KF-041 en personne**, qu'aucun préfixe du périmètre
      ne peut atteindre. Confronter donc chaque libellé écrit au **catalogue entier** de sa locale.

      ⚠️ **Deux clés de sens différents qui aboutissent au même mot cible sont un défaut**, même si
      chaque traduction est correcte isolément — c'est précisément ce que KF-041 (« Clôturer » et
      « Fermer » confondus) et KF-042 (le faux ami « Valider ») décrivent. **Les six clés génériques
      de T5 sont les premières candidates.** Consigner le tableau, pas seulement la conclusion.
- [x] **T10 — Balayer les verrous de français**, les trois greps — AC7.
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
| `NPA` | ✅ **attesté**, corrigé en passe 1 | `field-postal-code` → **`NPA` / `PLZ` / `NPA` / `Postal code`** dans les quatre locales. ⚠️ **La première rédaction le donnait pour « à trancher, aucun attesté au catalogue » : c'était FAUX**, et l'envoyer à l'arbitrage aurait au mieux fait perdre du temps, au pire fait trancher une forme différente de celle déjà en service |
| `virement` | ✅ **partie A** | `Überweisung` (`Banküberweisung`) / `bonifico` / `bank transfer`, attesté par `supplier-invoices-pay-transfer` (23-3). ⚠️ **La première rédaction le donnait pour « à vérifier »** — il était **déjà en partie A du glossaire**, qu'il suffisait d'ouvrir |
| **`projet parent` / `projet racine`** | ⚠️ **relevé en passe 1, absent de la première rédaction** | L'écran `settings/projects` fait **28 des 55 clés de `settings`** et n'était nommé nulle part. ⚠️ **Son idiome est attesté et ce n'est PAS un calque** : `accounts-parent-archived` rend « le compte parent » par **`das übergeordnete Konto`**, jamais *Eltern-Konto*. Suivre cet idiome |
| ~~`Référence message`~~ → **`MsgId`** | ⚠️ **TRANCHÉ PAR GUY, 2026-08-21 — le français CHANGE** | *« garde-le verbatim, comme EndToEndId »*. `MsgId` est le nom exact du champ dans pain.001, et `pain001/mod.rs:28` le nomme dans la **même famille** que `EndToEndId` et `PmtInfId` : `const MAX_ID: usize = 35; // MsgId, PmtInfId, EndToEndId`. **Valeur identique dans les quatre locales.** Précédent visible à l'écran : `EndToEndId` est affiché verbatim deux lignes plus bas, dans le même tableau (`[id]/+page.svelte:121`) |

### ⚠️ T1 est RENDUE — `Référence message` devient `MsgId` (arbitrage de Guy, 2026-08-21)

*« Garde-le verbatim, comme EndToEndId. »* Les trois identifiants de pain.001 forment une famille
que le code nomme ensemble ; l'un d'eux est déjà affiché tel quel à l'écran. Les traduire à moitié —
`EndToEndId` verbatim et `MsgId` en « Référence message » — reviendrait à faire dire deux choses
différentes à une même famille de champs, dans le même écran.

**Le français change donc**, comme il a changé pour « Généré » → « Créé » : `payment-batches-msg-id`
porte **`MsgId`** dans les quatre locales.

⚠️ **La clé i18n est CONSERVÉE**, avec la même valeur partout — elle n'est pas supprimée au profit
d'un texte en dur. Motif : l'objet même de cet epic est de faire passer les libellés par `i18nMsg`,
et écrire `MsgId` en dur créerait un libellé de plus hors de tout appareil de contrôle. Le jour où
une locale voudrait diverger, la clé est là. *(Si l'intention était au contraire de retirer la clé
et d'écrire `MsgId` en dur comme `EndToEndId`, c'est un mot à dire — le geste est trivial, mais il
va dans le sens inverse de l'epic.)*

⚠️ **Trois des quatre termes que la première rédaction envoyait à l'arbitrage n'en demandaient
aucun** — `virement` était en partie A, `NPA` au catalogue dans les quatre locales, `lot` relevé et
seulement à promouvoir. **Il n'en reste qu'UN.** La leçon vaut plus que la correction : cette spec
répète « relever avant d'inventer » et ne l'avait pas appliqué à sa propre table de termes. Ouvrir
le glossaire coûte une commande ; un arbitrage inutile coûte un aller-retour, et un arbitrage rendu
en ignorant un précédent coûte une incohérence de vocabulaire que rien ne rattrape.

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

### Ce qui est HORS PÉRIMÈTRE et TRACÉ — ne pas le traiter ici

⚠️ **`settings/invoicing` est en français en dur, et invisible de tout compteur de l'epic.**
306 lignes, **5 appels i18n**, **0 clé `invoicing-` à l'allowlist**. Ni le moissonneur, ni
l'allowlist, ni la garde des libellés en dur ne le voient — la garde le dit elle-même : *« nœud de
texte de balisage → ❌ non »*. **Conséquence** : la 23-6 clôt l'epic par le **vidage de l'allowlist**,
ce qui vaudra déclaration que la dette i18n est éteinte — alors que cet écran restera français dans
les trois autres langues. **[KF-044 (#328)](https://github.com/guycorbaz/kesh/issues/328)** existe
pour que le sujet survive à cette clôture. ⚠️ **Ne PAS l'ajouter au périmètre de cette story** :
il a été relevé et validé en trois passes, et l'élargir contredirait la règle de splitting.

⚠️ **Autres divergences de replis relevées hors périmètre**, à tracer et non à traiter :
`dunning-edit`, `dunning-conflict`, `dunning-form-error`, `dunning-form-error-delay`,
`error-unexpected` (4 replis distincts), `loading`. Elles confirment qu'AC5-bis est nécessaire :
**le tableau est un plancher.**

### Ce qui a été balayé et trouvé PROPRE — ne pas re-balayer

⚠️ **Deux pistes de la passe 3 ont été RÉFUTÉES au sol, et les fermer a de la valeur** :
**aucune** des 155 clés moissonnées n'a de sites dans plus d'un domaine — les six clés génériques
sont vérifiées nominativement, leurs 17 sites sont tous dans `settings/` —, et le moissonneur **ne
filtre pas** par dossier (`filter(([, parTexte]) => parTexte.size > 1)`), contrairement à ce que la
consigne de la lentille supposait. Le périmètre est étanche.

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

`claude-opus-5[1m]` — implémentation du 2026-08-21.

### Debug Log References

#### T2 / AC3 — la garde AVANT tout correctif, sortie BRUTE

```
     × aucune clé ne porte deux replis différents
AssertionError: expected [ …(3) ] to deeply equal []
+   "payment-batches-col-date → « Exécution » / « Date d'exécution »",
+   "payment-batches-col-total → « Total » / « Montant »",
+   "onboarding-next → « Continuer » / « Enregistrer »",
```

⚠️ **Les TROIS y sont, dont celle que le moissonneur ne pouvait pas voir.** C'est la démonstration
de l'angle mort relevé en passe 3 : `onboarding-next` était **déjà au catalogue**, donc invisible
d'un outil qui ne relève que les clés absentes.

#### AC1 — les deux grandeurs, recomptées séparément

| grandeur | mesuré |
|---|---|
| clés écrites au catalogue | **96** |
| décrément de l'allowlist | **93** — `166 → 73`, ventilé : settings 55, payment-batches 30, onboarding 4, nav 4 |
| parité par locale | **1525** dans les quatre |

Les trois clés neuves (`payment-batches-line-amount`, `-detail-date`, `onboarding-save`) expliquent
l'écart : **elles n'ont jamais figuré à l'allowlist.**

#### AC6 — les deux bornes ont rougi, et c'était leur travail

| borne | avant | après | cause recomptée |
|---|---|---|---|
| `sitesTotal` | 1525 | **1529** | +4 appels neufs : 2 `Chargement…`, 1 `Chargement...`, 1 `<title>` |
| `CLES_RELEVEES` | 187 | **186** | −1 : `onboarding-field-qr-iban` n'est plus appelée |
| `CANDIDATES_ATTENDUES` | 41 | **41** | aucune fonction de libellé créée ni supprimée |

⚠️ **La BAISSE se recompte comme la hausse** : `bank-accounts-labels-qr-iban` a remplacé le couple
« libellé traduit + `(optionnel)` en dur ». Une clé sort du relevé sans que rien ne se perde — une
baisse *inexpliquée*, elle, serait une clé qui a cessé d'être traduite.

#### AC6-bis — cinq libellés en dur qu'aucune garde ne voit

Les quatre premiers routés vers `common-loading` / `loading` / `settings-title` ; le cinquième
— ` (optionnel)` accolé à un libellé traduit — **réutilise `bank-accounts-labels-qr-iban`**, déjà
traduite pour ce champ exact, plutôt que de créer une clé. ⚠️ La convention du dépôt met
« (optionnel) » **dans** le libellé (`account-field-parent-optional`, `vat-rates-field-to`).

#### T9-bis — contrôle d'homonymie aux DEUX portées

⚠️ **La portée 2 est la seule qui trouve quelque chose**, et elle a trouvé :

| couple | fr | de | verdict |
|---|---|---|---|
| `payment-batches-form-close` / `fiscal-year-close-button` | Fermer / **Clôturer** | `Schliessen` / `Schliessen` | ⚠️ **KF-041 (#323) confirmée** — ma clé est correcte, l'autre ment. Commentaire mesuré porté à l'issue |
| `creating` / `opening-balances-generating` | Création… / **Génération…** | `Wird erstellt…` | deux français pour un allemand — mineur, et « générer » est le mot que la 23-3b a écarté |
| `payment-batches-confirm-title` / `invoice-mark-paid-confirm` | règlement / paiement | `Zahlung bestätigen` | **légitime** : le glossaire rend les deux par `Zahlung` |
| `nav-supplier-invoices-import` / `imported-supplier-invoices-title` | identiques | identiques | **légitime** — menu et titre de page |

⚠️ **`closing` a été tranché DANS L'AUTRE SENS** — `Abschluss…`, jamais `Schliessen…` : le correctif
de KF-041 a désormais son précédent au catalogue.

#### T10 — les quatre greps, et le quatrième trouve HUIT verrous

Les trois greps historiques : rien sur le périmètre. **Le quatrième — né en passe 3 — rend huit
verrous réels** : sept `toHaveText('Défaut'|'Personnalisé')` dans `email-templates.spec.ts` et un
`toContainText('Projets')` dans `projects.spec.ts`. ⚠️ **Tous sur des sélecteurs déjà stables** :
le `data-testid` était en place et ne protégeait rien. Basculés sur `data-variant` (le CODE) et sur
`toBeVisible`. ⚠️ **Ils seraient restés verts en français et n'auraient cassé qu'en allemand** —
que la suite n'exécute pas (KF-043, #326).

### Completion Notes List

- **L'angle mort de la passe 3 s'est vérifié à l'exécution** : la garde a bien rougi sur les trois
  divergences, dont celle que le moissonneur ne voyait pas. **Sur un domaine partiellement traduit,
  le relevé de référence est la garde.**
- **Aucun terme inventé** : `lot`, `NPA`, `projet parent`, `clôture`, `modèle`, `objet` sont tous
  **relevés** sur une clé attestante. Partie A du glossaire **70 → 76**. ⚠️ Aucun n'est un
  arbitrage — ils manquaient, ce qui les laissait libres de dériver.
- **KF-041 confirmée par la mesure** et documentée dans l'issue avec le précédent de son correctif.
- **Trois chiffres de la spec revus à l'exécution** : 95 → **96** clés écrites (la 3ᵉ scission),
  187 → **186** pour `CLES_RELEVEES`, 1525 → **1529** pour `sitesTotal`.

### File List

**Modifié — outillage et gardes**
- `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts` — `PREFIXES` + `payment-batches-`/`onboarding-`, borne 186, assertion anti-fusion des six sens
- `frontend/src/lib/shared/i18n-keys.test.ts` — `sitesTotal` 1529
- `frontend/src/lib/shared/i18n-dette-connue.ts` — 166 → 73

**Modifié — les trois scissions et les cinq libellés en dur**
- `frontend/src/routes/(app)/payment-batches/+page.svelte`, `[id]/+page.svelte`
- `frontend/src/routes/onboarding/+page.svelte`
- `frontend/src/routes/(app)/settings/+page.svelte`, `settings/email-templates/+page.svelte`

**Modifié — tests**
- `frontend/tests/e2e/email-templates.spec.ts`, `projects.spec.ts`

**Modifié — catalogues et documentation**
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — 96 clés × 4
- `docs/i18n-glossaire.md` — partie A 70 → 76
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

| date | passe | résultat |
|---|---|---|
| 2026-08-21 | **passe 3** de `validate` | **1 CRITICAL · 7 HIGH · 7 MEDIUM · 2 LOW** (Opus ×2, 105 vérifications au sol). ⚠️ **La sévérité REMONTE** (`1C/2H/3M` → `0C/1H/2M` → `1C/7H/7M`) — arbitrage porté à Guy, cf. ci-dessous. ⚠️ **Le CRITICAL est un angle mort de MA MÉTHODE, pas un oubli** : `onboarding-next` porte deux replis (« Continuer » / « Enregistrer ») et la clé est **DÉJÀ au catalogue** — le bouton d'enregistrement du compte bancaire affiche « Continuer », **dans les quatre langues, aujourd'hui**. Mon relevé ne pouvait pas le voir : **le moissonneur ne relève que les clés ABSENTES des catalogues**, et la 23-3 l'avait écrit noir sur blanc dans le docstring de la garde que ma spec cite en References. **Sur un domaine partiellement traduit, le relevé de référence est la GARDE, pas le moissonneur.** Trois divergences, pas deux ; 96 clés écrites, pas 95. ⚠️ **Les deux lentilles CONVERGENT sur cet angle mort** par des chemins indépendants. **HIGH retenus** : l'assertion anti-fusion de la 23-3 était perdue (la garde générique reste verte si l'on refusionne — une clé unique à repli unique ne diverge pas) ; la garde n'était étendue qu'à `payment-batches-` alors que **c'est `onboarding-` qui portait le défaut actif** ; **cinq libellés français EN DUR** dans les écrans que la story déclare traduits, dont `<title>Paramètres - Kesh</title>` avec `settings-title` **déjà utilisée quatre lignes plus bas** — jumeau exact du `Qté` de la 23-3 ; la borne **`sitesTotal`** n'était nommée nulle part alors qu'elle **va rougir** ; les trois greps d'AC7 étaient aveugles à `toHaveText('…')` et `toContainText('…')` **avec quote** — **neuf verrous réels** sur des clés du périmètre ; et aucun AC ne portait la **relecture du français**, que la 23-3 avait et qui lui a évité de recopier « en v0.4 » dans trois langues neuves. **MEDIUM** : le contrôle d'homonymie était en **portée 1** — la faute exacte que la passe 3 de la 23-3 a classée HIGH, et `payment-batches-form-close = 'Fermer'` contre `fiscal-year-status-closed = Geschlossen` **étend KF-041** ; le grep de T9-bis ratait `saving`, l'une des clés qu'il désigne ; les décomptes de relais se contredisaient (« 25 + 3 » pour une ligne à 20 — recompté : 20 et 0, 31 sur le périmètre) ; AC5-bis manquait. ⚠️ **DEUX pistes RÉFUTÉES au sol**, et les fermer a de la valeur : aucune clé partagée entre domaines, et le moissonneur ne filtre pas par dossier. **KF-044 (#328) ouverte** : `settings/invoicing`, 306 lignes et 5 appels i18n, **invisible de tout compteur** — la 23-6 déclarerait la dette éteinte alors qu'il resterait français. |
| 2026-08-21 | **passe 2** de `validate` | **0 CRITICAL retenu · 1 HIGH · 2 MEDIUM** (Haiku ×2). ⚠️ **QUATRE des cinq CRITICAL annoncés sont RÉFUTÉS au sol**, garde-fou Haiku de `CLAUDE.md` appliqué. **Le plus instructif est le dernier** : une lentille affirmait que l'allowlist ne contient que **87** clés du périmètre et non 93, avec une ventilation détaillée à l'appui. **Mesuré : 93, et 0 clé absente** — `payment-batches` 30/30, `settings` 55/55, `onboarding` 4/4, plus 4 `nav-*`. ⚠️ **Son erreur EST le finding** : elle a compté par **préfixe de clé** là où « settings » est un **DOSSIER**, et a raté six clés **génériques** (`save-`, `cancel-`, `create-`, `saving-`, `creating-`, `closing-`, une chacune). 28 + 20 + 1 = 49, plus 6 = 55. La ventilation par préfixe est désormais écrite dans T5, avec le piège — **une lentille s'y est trompée, un développeur s'y trompera**. Les deux autres CRITICAL réfutés visaient des consignes **déjà présentes** : l'ordre T2-avant-T3 est écrit dans T2 même, et AC1 distingue déjà 95 et 93 dans un tableau. ⚠️ **Signal de méthode** : la lentille qui a produit 3 CRITICAL et 7 HIGH n'a fait que **6 appels d'outils** ; celle qui en a fait 47 a produit un seul CRITICAL, faux mais argumenté et fécond. **Le volume de findings n'est pas une mesure de rigueur.** **Retenu** : les deux clés NEUVES du split n'étaient pas nommées — une seule l'était —, et la spec ne demandait **ni relecture des replis, ni contrôle d'homonymie langue par langue**, alors que la 23-3 les avait et que **KF-041 et KF-042 en sont nées**. T9-bis les ajoute, avec la commande et le tableau à consigner. |
| 2026-08-21 | **passe 1** de `validate` | **1 CRITICAL · 2 HIGH · 3 MEDIUM · 0 LOW** (Sonnet ×2, contextes frais). ⚠️ **La lentille des FAITS rend UN seul écart sur vingt affirmations vérifiées** — tous les décomptes, les trois conflits de replis, la portée de la garde, les trois sites « Générer », le statut de `lot` et les quatre affirmations sur la 23-3b se recomptent exactement comme annoncé. **Le moissonneur exécuté sur la base réelle a donc tenu ses promesses.** ⚠️ **Mais la lentille de COHÉRENCE a trouvé une contradiction INTERNE, et c'est le CRITICAL** : AC1 annonçait 93 clés quand AC2 impose de scinder deux clés en quatre — un développeur recomptant honnêtement aurait trouvé un écart, et la sortie la moins coûteuse aurait été de **ne scinder qu'à moitié**, réintroduisant le défaut qu'AC2 existe pour fermer. Deux grandeurs distinctes sont désormais écrites : **95 clés au catalogue, 93 de décrément d'allowlist**. ⚠️ **Le correctif chiffré de la lentille était FAUX** (elle proposait 71 pour l'allowlist) : elle décomptait des clés neuves qui n'y ont jamais figuré — réfuté au sol. **DEUX HIGH, tous deux sur ma table des termes** : `virement` était **déjà en partie A du glossaire** et `NPA` attesté dans les quatre locales par `field-postal-code`, alors que la spec les envoyait à l'arbitrage. ⚠️ **Cette spec répète « relever avant d'inventer » et ne l'avait pas appliqué à sa propre table de termes** — la liste bloquante passe de QUATRE termes à **UN**. Et l'écran `settings/projects`, **28 des 55 clés de `settings`**, n'était nommé nulle part : son idiome de hiérarchie est attesté et **n'est pas un calque** (`übergeordnetes`, jamais *Eltern-*). **MEDIUM** : la numérotation des tâches contredisait leur ordre d'exécution — T3 devait précéder T2, un développeur suivant les numéros aurait rendu AC3 impossible à prouver ; AC4 et AC6 n'étaient portés par aucune tâche ; et le grep d'AC4 était ambigu sur un commentaire de code qu'une revue antérieure avait payé. |
| 2026-08-21 | création | Spec écrite après **exécution du moissonneur sur la base réelle**, pas depuis le plan d'epic. Périmètre confirmé à **93** (30 + 55 + 4 + 4). ⚠️ **Trois replis divergents trouvés, deux dans le périmètre** — dont `payment-batches-col-total`, qui porte **deux grandeurs** (total du lot / montant d'une ligne) : jumeau exact du défaut de la 23-3, latent, et **que la traduction activerait**. ⚠️ **La garde « une clé, un repli » ne couvre pas ce domaine** (bornée à `supplier-invoices-`) : son extension entre au livrable. ⚠️ **Conséquence non vue de l'arbitrage de la 23-3b** : trois sites disent encore « Générer un lot » pour un objet dont le statut s'affiche « Créé ». ⚠️ **`lot` est employé et figé depuis la 23-3b sans être en partie A** du glossaire. |
