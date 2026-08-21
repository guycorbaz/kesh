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

1. **AC1 — le périmètre est écrit dans les QUATRE catalogues, et DEUX grandeurs distinctes le
   décrivent.** ⚠️ **Les confondre est le premier piège de cette story, et il vient de sa propre
   spec** *(relevé en passe 1 de `validate`, où la ventilation contredisait AC2)* :

   | grandeur | valeur | pourquoi elle diffère |
   |---|---|---|
   | **clés écrites au catalogue** | **95** | les deux clés scindées par AC2 en produisent **quatre** |
   | **décrément de l'allowlist** | **93** → **166 − 93 = 73** | les deux clés NEUVES n'y ont jamais figuré |

   Ventilation des 93 entrées d'allowlist, relevée au moissonneur : `payment-batches` **30**,
   `settings` **55** *(dont **28** pour le seul écran `settings/projects`, et 25 via le relais `msg`
   de l'écran des modèles d'e-mail)*, `onboarding` **4**, plus les **4 `nav-*`** de l'inventaire —
   que le moissonneur ne voit pas, leurs clés étant portées par une table de données.

   ⚠️ **Recompter les DEUX depuis la source**, et ne jamais ajuster l'une pour la faire coïncider
   avec l'autre : c'est ce qui pousserait à ne scinder qu'à moitié, et à réintroduire le défaut
   qu'AC2 existe pour fermer. *(Une lentille de la passe 1 proposait 71 pour l'allowlist : **c'est
   faux**, et l'erreur est instructive — elle décomptait des clés qui n'y ont jamais été inscrites.)*
2. **AC2 — `payment-batches-col-total` est SCINDÉE** en une clé de lot et une clé de ligne, et
   `payment-batches-col-date` en un libellé de colonne et un libellé de fiche. ⚠️ **Aucune valeur
   unique ne doit être imposée à des sites qui affichent des grandeurs ou des registres différents.**
3. **AC3 — la garde « une clé, un repli » couvre `payment-batches-`**, et sa borne `CLES_RELEVEES`
   est **recomptée**, jamais ajustée. ⚠️ **Elle doit rougir sur les deux divergences AVANT leur
   correction** : la sortie brute du run rouge est collée au Dev Agent Record. Sans cette preuve,
   une extension vide passe tous les autres AC.
4. **AC4 — le verbe « générer » disparaît du domaine** au profit de « créer », sur les trois sites
   du tableau ci-dessus. Contrôle, **exécuté par T4 et sa sortie collée** :
   `grep -rn "[Gg]énér" frontend/src/routes/\(app\)/payment-batches/`.
   ⚠️ **Le critère porte sur les LIBELLÉS AFFICHÉS, pas sur les commentaires de code.** Le grep
   remonte aujourd'hui un commentaire de `[id]/+page.svelte` qui documente pourquoi un test E2E
   attendait `/Généré/i` — il parle bien de la création d'un lot, **au passé**, et c'est une
   explication qu'une revue antérieure a payée. **Ne pas la réécrire pour faire disparaître le
   bruit.**
5. **AC5 — le seul terme non attesté est tranché PAR GUY avant écriture** — `Référence message` —,
   et les termes **relevés mais non promus** montent en partie A avec leur clé attestante.
   ⚠️ **`lot` en fait partie** : la 23-3b l'a employé et figé (`Bereits in einem erstellten Stapel`)
   **sans le promouvoir** — le défaut que le glossaire documente sur lui-même, ouvert depuis une
   story. ⚠️ **Promouvoir n'est pas arbitrer** : `lot` et `NPA` sont relevés, ils ne demandent
   aucune décision.
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

- [x] **T1 — Arbitrage de Guy sur `Référence message` — RENDU le 2026-08-21, et promotion des termes relevés** — AC5.
      ⚠️ **Bloquant pour ce seul terme**, et à faire AVANT toute écriture au catalogue : la valeur
      sera figée dans quatre catalogues. ⚠️ **La liste est passée de QUATRE termes à UN en passe 1
      de `validate`** : `virement` était déjà en partie A, `NPA` attesté dans les quatre locales par
      `field-postal-code`, `lot` relevé et seulement à promouvoir. Ne jamais inventer ; si un terme
      neuf apparaît en cours de route, le proposer au Record et **s'arrêter**.
- [ ] **T2 — Étendre la garde « une clé, un repli » à `payment-batches-`** — AC3, et **coller sa
      sortie ROUGE** : c'est la seule preuve qu'elle voit les deux divergences. ⚠️ **Cette tâche
      vient AVANT le split, et la numérotation le dit désormais** : dans l'autre ordre, les
      divergences ont disparu du code, la garde ne peut plus rougir, et la preuve qu'AC3 exige
      devient impossible à produire honnêtement. *(L'ordre était contredit par la numérotation en
      passe 1 de `validate` — un développeur suivant les numéros aurait cassé AC3 sans le voir.)*
- [ ] **T3 — Scinder les deux clés à double sens** — AC2. ⚠️ **AVANT d'écrire quoi que ce soit aux
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
- [ ] **T4 — `payment-batches` : 32 clés × 4 locales** *(30 d'allowlist + les 2 issues du split)* —
      AC1, **AC4**, **AC6**. ⚠️ **Exécuter le grep d'AC4 et COLLER sa sortie** — un critère qu'aucune
      tâche n'exécute est une affirmation invérifiable. ⚠️ **AC6** : vérifier si le rollout crée ou
      supprime une fonction `*Label`/`*Text`/`*Display` ; si oui, recompter `CANDIDATES_ATTENDUES`
      et le **déclarer**, sinon écrire qu'aucune ne l'a été. Le gate le vérifie mécaniquement, mais
      rien n'oblige à le dire — et c'est le dire qui manque.
- [ ] **T5 — `settings` : 55 clés × 4 locales** — AC1. ⚠️ **25 passent par le relais `msg`** de
      l'écran des modèles d'e-mail : le littéral vit au site `msg(`, pas au site `i18nMsg(`.
      `findRelays` les voit ; un grep naïf de `i18nMsg(` ne les verrait pas.
      ⚠️ **« settings » est un DOSSIER, pas un préfixe de clé, et cette confusion a une victime
      documentée** : une lentille de la passe 2 a compté par préfixe, trouvé **87** clés au lieu de
      93, et conclu à une erreur d'arithmétique de la spec. Elle avait tort — mais elle prouve le
      piège. **Ventilation réelle des 55, relevée au moissonneur :**

      | préfixe | clés |
      |---|---:|
      | `projects-*` *(écran des projets analytiques)* | **28** |
      | `email-templates-*` *(les 25 via relais + 3 directes)* | **20** |
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
- [ ] **T6 — `onboarding` : 4 clés × 4 locales** — AC1. ⚠️ Le dossier est **partiellement traduit
      depuis la 23-1b** (8 libellés faits, 4 messages restants) : cette tâche referme l'écart.
- [ ] **T7 — Les 4 `nav-*` de l'inventaire** — AC1 : `nav-credit-notes`, `nav-email-templates`,
      `nav-projects`, `nav-supplier-invoices-import`. ⚠️ **Ce sont des clés DÉJÀ CÂBLÉES dont la
      traduction manque** — à ne pas confondre avec les 9 libellés de navigation que la 23-3b a
      sortis du français en dur. Les deux ensembles sont **disjoints**, vérifié.
- [ ] **T8 — Décrémenter l'allowlist de 93** — AC1, borne recomptée depuis le fichier.
- [ ] **T9 — Glossaire** : promotion des termes tranchés, dont **`lot`** — AC5.
- [ ] **T9-bis — Relecture des replis français ET contrôle d'HOMONYMIE, langue par langue** — AC7.
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

      ⚠️ **Deux clés de sens différents qui aboutissent au même mot cible sont un défaut**, même si
      chaque traduction est correcte isolément — c'est précisément ce que KF-041 (« Clôturer » et
      « Fermer » confondus) et KF-042 (le faux ami « Valider ») décrivent. **Les six clés génériques
      de T5 sont les premières candidates.** Consigner le tableau, pas seulement la conclusion.
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
| 2026-08-21 | **passe 2** de `validate` | **0 CRITICAL retenu · 1 HIGH · 2 MEDIUM** (Haiku ×2). ⚠️ **QUATRE des cinq CRITICAL annoncés sont RÉFUTÉS au sol**, garde-fou Haiku de `CLAUDE.md` appliqué. **Le plus instructif est le dernier** : une lentille affirmait que l'allowlist ne contient que **87** clés du périmètre et non 93, avec une ventilation détaillée à l'appui. **Mesuré : 93, et 0 clé absente** — `payment-batches` 30/30, `settings` 55/55, `onboarding` 4/4, plus 4 `nav-*`. ⚠️ **Son erreur EST le finding** : elle a compté par **préfixe de clé** là où « settings » est un **DOSSIER**, et a raté six clés **génériques** (`save-`, `cancel-`, `create-`, `saving-`, `creating-`, `closing-`, une chacune). 28 + 20 + 1 = 49, plus 6 = 55. La ventilation par préfixe est désormais écrite dans T5, avec le piège — **une lentille s'y est trompée, un développeur s'y trompera**. Les deux autres CRITICAL réfutés visaient des consignes **déjà présentes** : l'ordre T2-avant-T3 est écrit dans T2 même, et AC1 distingue déjà 95 et 93 dans un tableau. ⚠️ **Signal de méthode** : la lentille qui a produit 3 CRITICAL et 7 HIGH n'a fait que **6 appels d'outils** ; celle qui en a fait 47 a produit un seul CRITICAL, faux mais argumenté et fécond. **Le volume de findings n'est pas une mesure de rigueur.** **Retenu** : les deux clés NEUVES du split n'étaient pas nommées — une seule l'était —, et la spec ne demandait **ni relecture des replis, ni contrôle d'homonymie langue par langue**, alors que la 23-3 les avait et que **KF-041 et KF-042 en sont nées**. T9-bis les ajoute, avec la commande et le tableau à consigner. |
| 2026-08-21 | **passe 1** de `validate` | **1 CRITICAL · 2 HIGH · 3 MEDIUM · 0 LOW** (Sonnet ×2, contextes frais). ⚠️ **La lentille des FAITS rend UN seul écart sur vingt affirmations vérifiées** — tous les décomptes, les trois conflits de replis, la portée de la garde, les trois sites « Générer », le statut de `lot` et les quatre affirmations sur la 23-3b se recomptent exactement comme annoncé. **Le moissonneur exécuté sur la base réelle a donc tenu ses promesses.** ⚠️ **Mais la lentille de COHÉRENCE a trouvé une contradiction INTERNE, et c'est le CRITICAL** : AC1 annonçait 93 clés quand AC2 impose de scinder deux clés en quatre — un développeur recomptant honnêtement aurait trouvé un écart, et la sortie la moins coûteuse aurait été de **ne scinder qu'à moitié**, réintroduisant le défaut qu'AC2 existe pour fermer. Deux grandeurs distinctes sont désormais écrites : **95 clés au catalogue, 93 de décrément d'allowlist**. ⚠️ **Le correctif chiffré de la lentille était FAUX** (elle proposait 71 pour l'allowlist) : elle décomptait des clés neuves qui n'y ont jamais figuré — réfuté au sol. **DEUX HIGH, tous deux sur ma table des termes** : `virement` était **déjà en partie A du glossaire** et `NPA` attesté dans les quatre locales par `field-postal-code`, alors que la spec les envoyait à l'arbitrage. ⚠️ **Cette spec répète « relever avant d'inventer » et ne l'avait pas appliqué à sa propre table de termes** — la liste bloquante passe de QUATRE termes à **UN**. Et l'écran `settings/projects`, **28 des 55 clés de `settings`**, n'était nommé nulle part : son idiome de hiérarchie est attesté et **n'est pas un calque** (`übergeordnetes`, jamais *Eltern-*). **MEDIUM** : la numérotation des tâches contredisait leur ordre d'exécution — T3 devait précéder T2, un développeur suivant les numéros aurait rendu AC3 impossible à prouver ; AC4 et AC6 n'étaient portés par aucune tâche ; et le grep d'AC4 était ambigu sur un commentaire de code qu'une revue antérieure avait payé. |
| 2026-08-21 | création | Spec écrite après **exécution du moissonneur sur la base réelle**, pas depuis le plan d'epic. Périmètre confirmé à **93** (30 + 55 + 4 + 4). ⚠️ **Trois replis divergents trouvés, deux dans le périmètre** — dont `payment-batches-col-total`, qui porte **deux grandeurs** (total du lot / montant d'une ligne) : jumeau exact du défaut de la 23-3, latent, et **que la traduction activerait**. ⚠️ **La garde « une clé, un repli » ne couvre pas ce domaine** (bornée à `supplier-invoices-`) : son extension entre au livrable. ⚠️ **Conséquence non vue de l'arbitrage de la 23-3b** : trois sites disent encore « Générer un lot » pour un objet dont le statut s'affiche « Créé ». ⚠️ **`lot` est employé et figé depuis la 23-3b sans être en partie A** du glossaire. |
