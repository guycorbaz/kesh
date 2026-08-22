# Epic 23 — Dette i18n : le repli silencieux

**Statut** : in-progress, **régime allégé depuis le 2026-08-22** (kickoff 2026-08-19) — cf. § *Régime allégé* ci-dessous
**Issues GitHub** : [#316] (KF-040 — **285** clés demandées, absentes des **quatre** catalogues ; l'issue en annonce 258, chiffre d'avant les relais et l'inventaire) + [#283] (57 clés présentes en `fr-CH`, absentes des trois autres)
**Origine** : items de **catégorie A** de la rétrospective Epic 22, actions **A6**. Ils bloquent le kickoff de l'epic suivant au sens de la § *Tech debt management* du `CLAUDE.md`.
**Cible release** : v0.11 (à confirmer)
**Arbitrages de Guy (2026-08-19)** : (1) **tout résorber** et poser la garde générale, (2) traductions produites par l'orchestrateur **sur glossaire figé d'avance** (`docs/i18n-glossaire.md`), (3) **epic dédié**, story-zéro + rollouts — la règle de splitting préventif se déclenche (14 dossiers > 5 modules).

## ⚠️ Régime allégé — arbitrage du 2026-08-22

**Ce n'est pas un arrêt : on traduit, mais on ne fignole plus.** La priorité passe à ce que Kesh
**fonctionne en français** pour la première version de production ; les traductions se finiront
plus tard, quand cette version sera opérationnelle.

Ce qui change, concrètement :

| | Régime du kickoff | Régime allégé |
|---|---|---|
| Passes de revue | boucle adversariale jusqu'à 0 finding > LOW, modèle différent à chaque passe | une passe, ou aucune sur un rollout mécanique |
| Glossaire (`docs/i18n-glossaire.md`) | arbitrage de chaque terme avant traduction | on consulte, on n'arbitre plus |
| Compteurs exacts | recomptés et tenus à chaque story | tenus là où ils gardent quelque chose, pas par principe |
| Ordre | 23-7 avant 23-6 | **inchangé** — une clôture ne se prononce pas sur un inventaire incomplet |

⚠️ **Ce que le régime allégé ne touche PAS : les gardes.** Elles restent actives, et c'est le
point qui rend ce gel réversible. Leur fonction n'est pas de résorber la dette mais d'empêcher
qu'elle **grossisse** : un écran neuf qui écrit du français en dur rougit au gate, une clé qui
part servir deux sens rougit au gate. Les désactiver « puisqu'on ne traduit plus » ferait
exactement l'inverse de ce que l'arbitrage demande — on retrouverait, à la reprise, une dette
plus grosse qu'au moment de la décision, et sans inventaire.

⚠️ **Le statut du suivi reste `in-progress`, délibérément.** Aucun statut « suspendu » n'existe
dans `sprint-status.yaml`, et en inventer un casserait la découverte de story des workflows BMAD,
qui filtrent sur des valeurs connues. C'est le commentaire porté sur `epic-23` qui tient
l'arbitrage, pas le statut.

**Corollaire immédiat, et il a déjà servi.** La distinction qui décide de la priorité n'est pas
« traduit / pas traduit » mais **latent / actif** : tant qu'une clé manque des catalogues, chaque
site affiche son repli français et rien n'est visible ; dès qu'elle y figure, le catalogue gagne
et une clé servant deux sens **ment en français, aujourd'hui**. C'est ce qu'a montré [KF-045
(#330)] le jour même de cet arbitrage — six clés, dont dix `catch` de l'onboarding effondrés sur
« Erreur interne ». **Ce genre de défaut reste prioritaire malgré le régime allégé** : il ne
relève pas de la traduction, il relève du fonctionnement en français.

## Le défaut, en une phrase

`i18nMsg(clé, repli)` retombe **silencieusement** sur son second argument — du français en dur —
quand la clé manque au catalogue, et le `loader.rs` du crate `kesh-i18n` charge `fr-CH` comme
**base de repli des trois autres locales**. Un oubli de traduction ne produit donc ni erreur, ni
avertissement, ni clé brute à l'écran : il produit **du français correct**, servi à un
germanophone, avec tous les gates au vert. C'est le **test muet** transposé aux traductions.

## Chiffres — recomptés depuis la source le 2026-08-19

| | mesure |
|---|---|
| clés par catalogue | `fr-CH` **1273**, `de-CH` / `en-CH` / `it-CH` **1216** |
| **[#283]** clés en `fr-CH` absentes ailleurs | **57**, et le **même ensemble** sur les trois locales (union = intersection) ; **0** clé en trop |
| **[#316]** littéraux demandés et absents des 4 catalogues | **285 statiques** (relais et inventaire compris) ; le recensement initial, borné à `i18nMsg(`, en rendait 258 dont **8 littéraux dynamiques** (`journal-${j.toLowerCase()}`, `vat-category-${r.category}`…) → **250 clés statiques** |
| préfixes dynamiques réels / sites d'appel | **8** / **10** — ⚠️ les 8 littéraux ci-dessus ne couvrent que **7** préfixes ; le 8ᵉ, `bank-import-info-*`, avait échappé à l'extraction (passe 1 de `validate` de la 23-1) |
| replis moissonnables mécaniquement | **274** / 285 |
| clés sans repli littéral | **5**, toutes dans `TransactionSplitModal.svelte` — replis interpolés (`` `Ligne ${i + 1} : compte requis` ``) → entrées Fluent **à variables** |
| dossiers concernés | **14** pour [#316] (le 14ᵉ, `routes/onboarding`, n'est apparu qu'avec les relais) |
| clés révélées par l'énumération des motifs dynamiques | **+10** — la famille `imported-supplier-invoices-error-*` est absente des quatre catalogues (*trouvé à la spécification de la 23-1, cf. ci-dessous*) |
| clés révélées par les **relais locaux** (`msg(key, fallback) → i18nMsg`) | **+29**, et un **dossier entier** (`routes/onboarding`) — invisibles de toute extraction cherchant `i18nMsg(` ; trouvées en passe 1 de revue du split, cf. story 23-1a § D4-bis |
| clés révélées par l'**inventaire des sites non résolus** (D4-ter) | **+6** — 4 entrées du menu principal (`nav-*`) et 2 onglets de rapport, portées par des tables `i18nKey`/`labelKey` |
| **total à faire vivre** | **352 clés** → **295** entrées `fr-CH` + **1056** messages `de-CH` / `it-CH` / `en-CH` |

Commande de recompte, à rejouer et non à croire :

```sh
cd crates/kesh-i18n/locales
LC_ALL=C comm -23 <(grep -oE '^[a-zA-Z][A-Za-z0-9_-]* *=' fr-CH/messages.ftl | LC_ALL=C sort -u) \
                  <(grep -oE '^[a-zA-Z][A-Za-z0-9_-]* *=' de-CH/messages.ftl | LC_ALL=C sort -u) | wc -l
```

### Répartition de [#316] par dossier — elle donne le découpage

| clés | dossier |
|---:|---|
| 99 | `routes/(app)/supplier-invoices` (+ 10 clés de la famille dynamique `imported-supplier-invoices-error-*`) |
| 30 | `routes/(app)/payment-batches` |
| 55 | `routes/(app)/settings` (dont 25 via relais — l'écran des modèles d'e-mail) |
| 4 | `routes/onboarding` (via relais) |
| 20 | `lib/features/reconciliation` (15 + les 5 sans repli littéral) |
| 15 | `routes/(app)/credit-notes` |
| 14 | `lib/features/reports` |
| 12 | `lib/features/contacts` |
| 8 | `routes/(app)/contacts` |
| 7 | `routes/(app)/reports` |
| 6 | `routes/(app)/invoices` |
| 4 | `lib/components` |
| 3 | `lib/features/journal-entries` |
| 2 | `routes/(app)/bank-accounts` |
| **+6** | **tables `i18nKey` / `labelKey`** — `routes/(app)/+layout.svelte` (4, vers 23-4) et `routes/(app)/reports/+page.svelte` (2, vers 23-5) ; révélées par l'inventaire, elles n'appartiennent à aucun dossier au sens du recensement de littéraux |

## Pourquoi rien ne l'a vu — les trois contrôles regardent ailleurs

- **`npm run check`** (svelte-check) ne connaît pas les clés i18n : ce sont des littéraux de chaîne.
- **`npm run lint-i18n-ownership`** contrôle l'**appartenance** d'un namespace à un dossier
  (`keyBelongsToFeature`), jamais l'**existence** de la clé — et il ne balaie que
  `src/lib/features/`, alors que **226 des 285** manquantes sont demandées **exclusivement** depuis `src/routes/`
  (228 depuis au moins un fichier de `routes/`).
- **la suite E2E** tourne en **français**, où le repli est rigoureusement indiscernable de la traduction.

⚠️ **Et le repli a DEUX chemins, ce que la première rédaction de ce plan taisait.** `all_messages`
(`loader.rs:130-143`) charge `fr-CH` comme base avant d'écraser avec la locale demandée : pour les
57 clés de [#283], le frontend reçoit donc **du français depuis le backend**, et le second argument
d'`i18nMsg` n'est jamais atteint. Pour les 285 de [#316], c'est le littéral en dur du fichier appelant qui
s'affiche. Même symptôme, deux mécanismes — et **aucun test passant par `format()` ou
`all_messages()` ne peut voir le premier**. Détail et conséquences : story 23-1, § *Contexte*.

## La garde — deux niveaux, parce qu'il y a deux défauts

Le patron n'est pas à inventer : il est **déjà écrit deux fois** dans le dépôt, borné à un
domaine à chaque fois — `client_number_labels_are_translated_in_all_four_locales` (16-3b) et
`duplicate_probe_labels_are_translated_in_all_four_locales` (22-2b). L'epic 23 le généralise.

1. **Parité inter-locales — côté Rust, dans `kesh-i18n`.** L'ensemble des clés des quatre
   `messages.ftl` est **identique**. Ferme [#283] pour de bon.
   ⚠️ **Le test doit lire les FICHIERS, pas passer par `format()`** : le loader repliant sur
   `fr-CH`, `format()` rend un texte français pour une clé absente — il ne peut pas distinguer
   « traduit » de « replié ». C'est l'assertion `assert_ne!(msg, fr)` des deux précédents qui
   attrape le défaut réel, et elle ne se généralise pas telle quelle (deux locales peuvent
   légitimement partager un libellé : « Total », « CHF », « Journal »).
2. **Existence des clés demandées — côté frontend, en vitest.** Toute clé littérale passée à
   `i18nMsg()` existe au catalogue. Ferme [#316] pour de bon. Le test existant
   `duplicate-i18n-keys.test.ts` en est la version bornée à `contact-duplicate-*` : il est le
   point de départ, sa portée est à ouvrir.
3. **Les 8 préfixes dynamiques** (sur **10 sites d'appel**) ne sont pas des clés : ils se traitent
   par **énumération déclarée** (le motif + la liste de ses valeurs), sans quoi la garde les ignore
   en silence — exactement le défaut qu'elle prétend fermer.
   ⚠️ **Deux réserves établies en passe 1 de `validate` de la 23-1.** (a) Le recensement initial de ces
   motifs en avait **manqué un** (`bank-import-info-*`), parce qu'une extraction par classe de
   caractères négative ne traverse pas un gabarit dont l'interpolation contient des apostrophes —
   la garde doit résister à ce cas. (b) `vat-category-*` n'est **pas** énumérable : sa colonne n'a
   aucune contrainte `CHECK`, par décision explicite de la Story 11-1, donc les catégories créées
   par un administrateur restent un **angle mort assumé**.

⚠️ **Angle mort assumé de la garde, et il porte un numéro : [#255].** Une chaîne écrite **en
dur** dans un `.svelte`, sans passer par `i18nMsg()` du tout, n'est visible d'aucun des deux
niveaux — la page `/invoices` en est le cas (**6 appels** `i18nMsg` pour toute la page). [#255]
est le troisième item de catégorie A ; il n'est **pas** dans le périmètre de cet epic ⚠️ **Amendé le 2026-08-21 : il l'est devenu**, par la story 23-3b, intercalée après la 23-3. et
appelle un contrôle d'une autre nature (détection de littéraux affichés).

## Découpage — story-zéro puis rollouts

| Story | Objet | Clés |
|---|---|---|
| **23-1a** | **Mécanisme** : les deux gardes, l'extracteur et son test, les 8 préfixes dynamiques (10 sites), les deux allowlists — **décroissantes par construction** et **nées pleines**. Aucune traduction. | 0 |
| **23-1b** | **Pilote** : moissonneur de replis versionné, les 20 clés de `contacts` dans les 4 locales, glossaire. **Dépend du merge de la 23-1a.** | 20 |
| **23-2** | **[#283]** — les 57 clés en `de-CH` / `it-CH` / `en-CH`. La garde de **parité** devient inconditionnelle : son allowlist disparaît | 57 |
| **23-3** | `supplier-invoices` — le gros morceau, seul (99 statiques + **10** de la famille dynamique `imported-supplier-invoices-error-*`) | 109 |
| **23-3b** | **[#255]** — la **garde contre les libellés en dur**, et les 8 sites qu'elle révèle. ⚠️ **Ses clés ne viennent PAS du total ci-dessous** : un libellé qui n'appelle jamais `i18nMsg` n'était compté nulle part | 18 |
| **23-4** | `settings` + `payment-batches` + `onboarding` + les 4 `nav-*` de l'inventaire | 93 |
| **23-5** | `reconciliation` (dont les 5 entrées à variables) + `reports` (14 + 7) + `credit-notes` + les 2 `reports-project-*` de l'inventaire | 58 |
| **23-6** | Reliquat : `invoices`, `journal-entries`, `lib/components`, `bank-accounts` + **clôture** : allowlist vidée, garde inconditionnelle, [#316] fermée | 15 |
| **23-7** | **[KF-044 (#328)]** — `settings/invoicing`, écran entier en français **en dur**. ⚠️ **Invisible de TOUS les compteurs de l'epic** : 306 lignes pour 5 appels `i18nMsg`, aucune clé `invoicing-` à l'allowlist. Ses clés sont à **créer**, pas à traduire | ~50 |

⚠️ **La 23-1 a été DÉCOUPÉE le 2026-08-19** (arbitrage de Guy) après trois passes de `validate` au
plafond de sévérité stagnant — non pour son nombre de modules, mais parce que ses findings formaient
deux familles qui ne se relisent pas avec la même lentille : le mécanisme et les comptes rendus.
Patron 22-2a/22-2b. Les cinq stories de rollout, elles, restent entières : elles n'ont **que** la
seconde famille.

⚠️ **La 23-7 a été ajoutée le 2026-08-22, sur arbitrage de Guy, et elle NE PEUT PAS être fondue
dans la 23-6.** Deux motifs. Le premier tient à la règle de splitting : la 23-6 porte déjà **la
clôture** de l'epic — vider l'allowlist, rendre les gardes inconditionnelles, fermer [#316] —, et y
mêler un rollout complet mélange deux natures de travail que le dépôt sépare depuis l'Epic 7. Le
second est plus grave : **`settings/invoicing` n'est dans aucun compteur**. Ses clés ne sont pas à
l'allowlist parce que le code ne les demande pas encore — il affiche du français en dur. Le
décrément d'allowlist de la 23-6 tomberait donc à zéro **sans que cet écran soit traduit**, et la
clôture déclarerait la dette éteinte sur un écran qui ne l'est pas.

⚠️ **Ordre imposé : la 23-7 avant la 23-6.** Une clôture ne se prononce pas sur un inventaire connu
pour être incomplet.

⚠️ **La 23-3b s'est intercalée le 2026-08-21, et son périmètre ne retire rien à la 23-4.** La
tâche T11 de sa spec prescrivait de « retirer les clés `nav-*` du périmètre de la 23-4 » en
supposant un recouvrement — **il n'y en a aucun, vérifié au sol**. Les 4 `nav-*` de la 23-4
(`nav-credit-notes`, `nav-email-templates`, `nav-projects`, `nav-supplier-invoices-import`) sont
des clés **déjà câblées** dont la traduction manque ; les **9** libellés de navigation de la 23-3b
étaient du **français en dur**, jamais routé vers `i18nMsg` — 6 entrées passées en variante
`i18nKey` et 3 libellés de groupe dont les clés existaient au catalogue depuis longtemps **sans
avoir jamais été câblées**. Deux défauts distincts, deux ensembles disjoints. **La 23-4 garde ses
93 clés.**

⚠️ **Les 18 clés de la 23-3b ne s'ajoutent pas au total de l'epic, elles s'ajoutent À CÔTÉ** — et
c'est le fait le plus instructif de cette story : **l'allowlist de dette n'a pas bougé d'une
ligne** (166 avant, 166 après). Cette dette-là n'était comptée nulle part, puisque tout
l'appareil de mesure de l'epic — moissonneur, allowlist, les trois gardes — ne relève que les
sites `i18nMsg`. C'est l'angle mort #255, exprimé en chiffres.

**Chaque story de rollout est mécanique par construction** : entrer les clés au catalogue dans
les quatre locales, retirer d'autant l'allowlist, laisser la garde prouver le reste. La revue
s'y fait au fichier, pas en passes adversariales globales — conformément à la règle de splitting.

## Ce que la 23-7 a trouvé, et qu'aucune garde ne voyait

⚠️ **Douze clés `settings-invoicing-*` existaient déjà dans les quatre catalogues,
traduites, et le code n'en demandait aucune.** L'écran affichait du français en dur pendant
que ses traductions dormaient au catalogue, depuis la Story 5.2.

**C'est le miroir exact de #316**, et il faut voir pourquoi le dispositif de l'epic ne pouvait
pas le voir :

| | #316 (le défaut de l'epic) | Ce que la 23-7 a trouvé |
|---|---|---|
| Le code | demande une clé | n'appelle rien |
| Le catalogue | ne l'a pas | l'a, traduite ×4 |
| Ce que l'utilisateur voit | du français, servi à tous | du français, servi à tous |
| Le moissonneur | **la voit** (clé demandée, absente) | **aveugle** — il ne relève que les demandes |
| La parité `loader.rs` | ne la voit pas | **aveugle** — elle ne compare que les catalogues entre eux |

Les deux gardes centrales de l'epic partent donc du **code** ou des **catalogues comparés
entre eux** ; aucune ne part du catalogue pour demander *« qui appelle ceci ? »*. Le seul
dispositif du dépôt qui ferme ce sens est `PREFIXES_A_COUVERTURE_CLOSE` de
`i18n-keys.test.ts` — et il ne vaut **que par préfixe explicitement déclaré**, parce qu'une
clé sans demandeur côté frontend n'est pas orpheline pour autant : `kesh-qrbill` et
`kesh-report` lisent le même catalogue pour les PDF.

**`settings-invoicing-` y entre avec cette story.** Le préfixe n'est devenu clos qu'après
retrait de `settings-invoicing-format-too-long`, dernière orpheline et doublon de la clé
neuve `invoices-format-error-too-long`.

⚠️ **La leçon pour les stories restantes** : le compteur d'allowlist ne mesure qu'un sens de
la dette. Un écran peut en sortir sans être traduit — c'était l'argument de la 23-7 — et il
peut aussi être « traduit » au catalogue sans que rien ne l'affiche. **Les deux sens sont
muets, et pour des raisons différentes.**

## Hors périmètre, explicitement

- ~~**[#255]** (chaînes en dur sans `i18nMsg`) — même famille, autre mécanisme de détection.~~
  ⚠️ **PLUS VRAI depuis le 2026-08-21** : la story **23-3b** l'a traité (garde + 8 sites), et la
  ligne du tableau de découpage le dit. Cette entrée est restée ici après l'insertion de la 23-3b —
  **résidu de patch** : le symptôme n'avait pas été grepé sur le document. Un lecteur ouvrant la
  section la plus autoritaire du plan y lisait l'inverse de ce qui a été livré. *(Relevé en passe 3
  de revue de la 23-3b.)*
- **[#314]** (recherche d'un nom à trait d'union) — quatrième item de catégorie A, sans rapport.
- **Le sélecteur de langue dans l'interface** ([#242]) — cet epic rend les chaînes traduisibles,
  il ne change pas la manière dont la locale est choisie.
- **La relecture native** des traductions produites. Le glossaire fixe la terminologie et Guy
  relit un échantillon plus les messages d'erreur ; **aucune relecture par un locuteur natif
  n'est prévue**, et c'est un risque assumé, à écrire dans la rétrospective.

## Risques

| # | Risque | Parade |
|---|---|---|
| R1 | **La moisson des replis fait entrer 285 formulations françaises non relues.** Un repli écrit à la va-vite devient un libellé de catalogue. | Le moissonneur **propose**, il ne commite pas : chaque story de rollout relit ses entrées `fr-CH` avant de les figer. |
| R2 | **L'allowlist devient un cimetière.** Une allowlist de 352 lignes qui ne décroît pas rend la garde décorative. | Elle est **décroissante par construction** (un test échoue si elle contient une clé désormais présente) et l'epic se clôt sur son vidage. |
| R3 | **Terminologie divergente entre stories de rollout.** Six stories, six occasions de traduire « justificatif » différemment. | `docs/i18n-glossaire.md`, figé avant la première traduction ; partie A **non négociable** en story de rollout. |
| R4 | **Les entrées Fluent à variables** (les 5 de `TransactionSplitModal`) sont les seules non mécaniques — une erreur de nom de variable est silencieuse à la compilation. | Traitées dans la 23-5, avec un test qui **formate** chaque entrée avec ses arguments et vérifie qu'aucun placeholder ne survit au rendu. |

[#316]: https://github.com/guycorbaz/kesh/issues/316
[#283]: https://github.com/guycorbaz/kesh/issues/283
[#255]: https://github.com/guycorbaz/kesh/issues/255
[#314]: https://github.com/guycorbaz/kesh/issues/314
[#242]: https://github.com/guycorbaz/kesh/issues/242
