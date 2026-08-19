# Epic 23 — Dette i18n : le repli silencieux

**Statut** : in-progress (kickoff 2026-08-19)
**Issues GitHub** : [#316] (KF-040 — 250 clés demandées, absentes des **quatre** catalogues) + [#283] (57 clés présentes en `fr-CH`, absentes des trois autres)
**Origine** : items de **catégorie A** de la rétrospective Epic 22, actions **A6**. Ils bloquent le kickoff de l'epic suivant au sens de la § *Tech debt management* du `CLAUDE.md`.
**Cible release** : v0.11 (à confirmer)
**Arbitrages de Guy (2026-08-19)** : (1) **tout résorber** et poser la garde générale, (2) traductions produites par l'orchestrateur **sur glossaire figé d'avance** (`docs/i18n-glossaire.md`), (3) **epic dédié**, story-zéro + rollouts — la règle de splitting préventif se déclenche (13 dossiers > 5 modules).

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
| **[#316]** littéraux demandés par `i18nMsg()` et absents des 4 catalogues | **258**, dont **8 littéraux dynamiques** (`journal-${j.toLowerCase()}`, `vat-category-${r.category}`…) → **250 clés statiques** |
| préfixes dynamiques réels / sites d'appel | **8** / **10** — ⚠️ les 8 littéraux ci-dessus ne couvrent que **7** préfixes ; le 8ᵉ, `bank-import-info-*`, avait échappé à l'extraction (passe 1 de `validate` de la 23-1) |
| replis moissonnables mécaniquement | **245** / 250 |
| clés sans repli littéral | **5**, toutes dans `TransactionSplitModal.svelte` — replis interpolés (`` `Ligne ${i + 1} : compte requis` ``) → entrées Fluent **à variables** |
| dossiers concernés | **13** pour [#316] |
| clés révélées par l'énumération des motifs dynamiques | **+10** — la famille `imported-supplier-invoices-error-*` est absente des quatre catalogues (*trouvé à la spécification de la 23-1, cf. ci-dessous*) |
| **total à faire vivre** | **317 clés** → **260** entrées `fr-CH` (moisson + énumération) + **951** messages `de-CH` / `it-CH` / `en-CH` |

Commande de recompte, à rejouer et non à croire :

```sh
cd crates/kesh-i18n/locales
LC_ALL=C comm -23 <(grep -oE '^[a-z0-9-]+ =' fr-CH/messages.ftl | LC_ALL=C sort -u) \
                  <(grep -oE '^[a-z0-9-]+ =' de-CH/messages.ftl | LC_ALL=C sort -u) | wc -l
```

### Répartition de [#316] par dossier — elle donne le découpage

| clés | dossier |
|---:|---|
| 99 | `routes/(app)/supplier-invoices` |
| 30 | `routes/(app)/payment-batches` |
| 30 | `routes/(app)/settings` |
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

## Pourquoi rien ne l'a vu — les trois contrôles regardent ailleurs

- **`npm run check`** (svelte-check) ne connaît pas les clés i18n : ce sont des littéraux de chaîne.
- **`npm run lint-i18n-ownership`** contrôle l'**appartenance** d'un namespace à un dossier
  (`keyBelongsToFeature`), jamais l'**existence** de la clé — et il ne balaie que
  `src/lib/features/`, alors que **197 des 250** manquantes vivent sous `src/routes/`.
- **la suite E2E** tourne en **français**, où le repli est rigoureusement indiscernable de la traduction.

⚠️ **Et le repli a DEUX chemins, ce que la première rédaction de ce plan taisait.** `all_messages`
(`loader.rs:130-143`) charge `fr-CH` comme base avant d'écraser avec la locale demandée : pour les
57 clés de [#283], le frontend reçoit donc **du français depuis le backend**, et le second argument
d'`i18nMsg` n'est jamais atteint. Pour les 250 de [#316], c'est le littéral en dur du `.svelte` qui
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
est le troisième item de catégorie A ; il n'est **pas** dans le périmètre de cet epic et
appelle un contrôle d'une autre nature (détection de littéraux affichés).

## Découpage — story-zéro puis rollouts

| Story | Objet | Clés |
|---|---|---|
| **23-1** | **Socle** : les deux gardes, avec allowlist explicite des clés connues, **décroissante seulement** ; moissonneur de replis versionné ; les 8 préfixes dynamiques (10 sites) ; glossaire figé ; domaine pilote `contacts` (12 + 8) | 20 |
| **23-2** | **[#283]** — les 57 clés en `de-CH` / `it-CH` / `en-CH`. La garde de **parité** devient inconditionnelle : son allowlist disparaît | 57 |
| **23-3** | `supplier-invoices` — le gros morceau, seul (99 statiques + **10** de la famille dynamique `imported-supplier-invoices-error-*`) | 109 |
| **23-4** | `settings` + `payment-batches` | 60 |
| **23-5** | `reconciliation` (dont les 5 entrées à variables) + `reports` (14 + 7) + `credit-notes` | 56 |
| **23-6** | Reliquat : `invoices`, `journal-entries`, `lib/components`, `bank-accounts` + **clôture** : allowlist vidée, garde inconditionnelle, [#316] fermée | 15 |

**Chaque story de rollout est mécanique par construction** : entrer les clés au catalogue dans
les quatre locales, retirer d'autant l'allowlist, laisser la garde prouver le reste. La revue
s'y fait au fichier, pas en passes adversariales globales — conformément à la règle de splitting.

## Hors périmètre, explicitement

- **[#255]** (chaînes en dur sans `i18nMsg`) — même famille, autre mécanisme de détection.
- **[#314]** (recherche d'un nom à trait d'union) — quatrième item de catégorie A, sans rapport.
- **Le sélecteur de langue dans l'interface** ([#242]) — cet epic rend les chaînes traduisibles,
  il ne change pas la manière dont la locale est choisie.
- **La relecture native** des traductions produites. Le glossaire fixe la terminologie et Guy
  relit un échantillon plus les messages d'erreur ; **aucune relecture par un locuteur natif
  n'est prévue**, et c'est un risque assumé, à écrire dans la rétrospective.

## Risques

| # | Risque | Parade |
|---|---|---|
| R1 | **La moisson des replis fait entrer 250 formulations françaises non relues.** Un repli écrit à la va-vite devient un libellé de catalogue. | Le moissonneur **propose**, il ne commite pas : chaque story de rollout relit ses entrées `fr-CH` avant de les figer. |
| R2 | **L'allowlist devient un cimetière.** Une allowlist de 317 lignes qui ne décroît pas rend la garde décorative. | Elle est **décroissante par construction** (un test échoue si elle contient une clé désormais présente) et l'epic se clôt sur son vidage. |
| R3 | **Terminologie divergente entre stories de rollout.** Six stories, six occasions de traduire « justificatif » différemment. | `docs/i18n-glossaire.md`, figé avant la première traduction ; partie A **non négociable** en story de rollout. |
| R4 | **Les entrées Fluent à variables** (les 5 de `TransactionSplitModal`) sont les seules non mécaniques — une erreur de nom de variable est silencieuse à la compilation. | Traitées dans la 23-5, avec un test qui **formate** chaque entrée avec ses arguments et vérifie qu'aucun placeholder ne survit au rendu. |

[#316]: https://github.com/guycorbaz/kesh/issues/316
[#283]: https://github.com/guycorbaz/kesh/issues/283
[#255]: https://github.com/guycorbaz/kesh/issues/255
[#314]: https://github.com/guycorbaz/kesh/issues/314
[#242]: https://github.com/guycorbaz/kesh/issues/242
