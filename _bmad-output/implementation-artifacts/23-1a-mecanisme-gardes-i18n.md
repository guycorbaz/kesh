# Story 23.1a : Le mécanisme — deux gardes, et rien qui se traduise

## Status

review

Première moitié du split de la **23-1**, arbitré par Guy le 2026-08-19 après trois passes de
`bmad-create-story validate` (`CRITICAL → HIGH → HIGH` : plafond de sévérité stagnant, § *Règle de
splitting préventif*). La seconde moitié est la **23-1b**, qui consomme ce que celle-ci produit.

⚠️ **Cette story ne traduit RIEN et ne touche à AUCUN catalogue.** Elle pose les deux gardes, leur
extracteur et leurs allowlists — les allowlists naissent donc **pleines** : **352** clés, dont les 20
du pilote que la 23-1b retirera. C'est délibéré, et c'est ce qui la rend relisable : aucun
arbitrage terminologique, aucun libellé, aucune locale à peser.

## Story

**As a** personne qui utilise Kesh en allemand, en italien ou en anglais,
**I want** qu'un oubli de traduction devienne un test rouge au lieu d'un silence,
**so that** la dette cesse de se creuser pendant qu'on la résorbe.

Adresse [#316] et [#283] — dont elle ne ferme aucune : elle les **borne**.

## Périmètre

**Dedans** : la garde A (parité inter-locales, Rust), la garde B (existence des clés demandées,
vitest), l'extracteur et son test, les 8 préfixes dynamiques, les deux allowlists, les bornes
anti-test-muet.

**Dehors, et c'est la 23-1b** : le moissonneur de replis, les 20 clés du pilote `contacts`, le
renommage de `delete`, le glossaire et ses trois termes à promouvoir, le registre d'adresse.
**Dehors aussi** : les rollouts 23-2 à 23-6, [#255], [#314], le sélecteur de langue [#242], **et
toute traduction des manuels LaTeX**.

⚠️ **Dehors également, et il faut le redire parce que le découpage l'avait fait tomber** : le
backend **`kesh-qrbill`** et sa table `I18N_KEYS` / `DEFAULT_EN` (`crates/kesh-qrbill/src/types.rs:216`)
sont un **troisième catalogue, apparié par POSITION** — les deux gardes ne le couvrent pas, et
l'y étendre serait un autre chantier. *(La D2 ci-dessous mentionne `kesh-qrbill` à propos des PDF :
c'est un fait différent, qui ne dispense pas de cet avertissement. Clause présente dans la spec
d'origine, perdue par les DEUX moitiés au découpage, restaurée en passe 1 du split.)*

⚠️ **Les identifiants de décisions et de critères sont ceux de la 23-1 d'origine**, conservés tels
quels dans les deux moitiés — les renuméroter casserait les renvois internes que trois passes de
revue ont vérifiés. Ceux qui manquent ici vivent dans la 23-1b.

## Contexte — le défaut a deux chemins, pas un

C'est le point que toute rédaction rapide de cette story rate, et il commande tout le reste.

`I18nBundle::all_messages(locale)` (`crates/kesh-i18n/src/loader.rs:130-143`) **charge d'abord
toutes les clés `fr-CH` comme base**, puis écrase avec celles de la locale demandée. Le
frontend reçoit donc, par `GET /api/v1/i18n/messages`, un catalogue **déjà replié**. D'où deux
chemins distincts pour un même symptôme visible :

| | où la clé manque | ce que reçoit le frontend | quel repli s'affiche |
|---|---|---|---|
| **[#283]** — 57 clés | en `de-CH` / `it-CH` / `en-CH`, présente en `fr-CH` | **le texte français**, via le repli du loader | celui du **backend** — le 2ᵉ argument d'`i18nMsg` n'est jamais atteint |
| **[#316]** — 285 clés | **partout** | rien | le **littéral en dur** du fichier appelant |

⚠️ **Conséquence directe sur la conception des gardes** : aucun test passant par `format()` ou
par `all_messages()` ne peut voir [#283]. Ces deux fonctions rendent du français **exactement
comme si la traduction existait**. Le seul observable honnête est **l'ensemble des clés déclarées
dans chaque fichier `.ftl`** — ce que `I18nBundle` conserve déjà dans son champ `keys`
(`loader.rs:34`), peuplé par `fluent_syntax::parser` au chargement.

C'est aussi pourquoi les deux précédents du dépôt — `client_number_labels_are_translated_in_all_four_locales`
(16-3b) et `duplicate_probe_labels_are_translated_in_all_four_locales` (22-2b) — recourent à
`assert_ne!(msg, fr)` : faute d'accès aux ensembles de clés, ils déduisent l'absence du fait que
la locale rend le même texte que le français. **Cette ruse ne se généralise pas** : « Total »,
« CHF », « Journal », « Kesh » sont légitimement identiques en quatre langues, et un test
généralisé sur ce critère produirait des faux positifs en série.

## Un constat de la spécification — il y a 10 clés de plus qu'annoncé

L'énumération des motifs dynamiques, exigée par le point 3, a été **faite** pendant la
spécification plutôt que supposée. ⚠️ **Et elle a d'abord été faite FAUX** — cf. l'encadré qui
suit le tableau. Le compte exact est de **8 préfixes sur 10 sites d'appel** (`journal-` et
`vat-category-` sont demandés depuis deux fichiers chacun) :

| motif | valeurs énumérées | présentes au catalogue |
|---|---|---|
| `journal-${j.toLowerCase()}` | 5 (achats, ventes, banque, caisse, od) | ✅ 5 |
| `account-type-${t.toLowerCase()}` | 4 | ✅ 4 |
| `due-dates-filter-${ps}` | 4 | ✅ 4 |
| `reports-filename-${type}` | 5 | ✅ 7 déclarées |
| `vat-category-${category}` / `${r.category}` | données de la table `vat_rates` | ✅ 5 |
| `reminders-error-${entry[0]}` | 15 + `-unknown` | ✅ 17 |
| **`imported-supplier-invoices-error-${entry[0]}`** | **10** | ❌ **0** |
| `bank-import-info-${info.replace(…)}` | 2 (`bank_csv_profile_auto_matched`, `bank_csv_multiple_profile_matches`) | ✅ 2, dans les 4 locales |

**Le dernier motif est entièrement absent des quatre catalogues.** Ses dix valeurs
(`unsupported-file-type`, `file-too-large`, `symlink-rejected`, `duplicate`, `no-qr-code-found`,
`invalid-spc-payload`, `invalid-iban`, `pdf-render-error`, `file-read-error`, `field-too-long`,
carte à `supplier-invoices/import/+page.svelte:55-66`) ne figurent dans **aucun** décompte
antérieur, puisqu'elles ne sont jamais écrites comme littéraux.

**Le total de l'epic passe donc de 307 à 317**, et la story 23-3 de 99 à 109. Le plan d'epic est
corrigé en conséquence. ⚠️ **Et il est passé à 346 en passe 1 du split** : les relais de D4-bis
ont révélé **29 clés de plus** et un dossier entier. Le présent § garde son récit d'origine, la
correction ultérieure étant au Change Log. ⚠️ *C'est exactement ce que le point 3 existe pour empêcher : un motif
dynamique que la garde ignorerait serait un trou **dans la garde elle-même**.*

### ⚠️ Le motif `bank-import-info-*` avait été manqué — et la cause est instructive

**La première rédaction de cette spec annonçait « 8 motifs » en n'en listant que 7**, `bank-import-info-*`
(`BankImportUpload.svelte:547`) étant absent du tableau comme du décompte. Relevé en passe 1 de
`validate`, vérifié : `grep -rnE 'i18nMsg\(\s*`[^$]*\$\{' frontend/src` rend **10 sites, 8 préfixes**.

**La cause n'est pas l'inattention, c'est la méthode d'extraction** — et elle contamine la garde
que cette story spécifie. Le motif employé pour recenser les clés était :

```
i18nMsg\(\s*(['"`])([^'"`]*)\1
```

Le corps de la clé y est défini comme « tout sauf une quote ». Or le site manqué s'écrit
`` i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info) `` : **l'interpolation contient
elle-même des apostrophes**, donc la classe `[^'"`]*` s'arrête avant la fin du gabarit et l'appel
n'est jamais reconnu. Le recensement rendait « 8 littéraux » — coïncidence numérique avec les
8 préfixes réels — dont deux étaient les deux sites de `vat-category-`, et il manquait une famille
entière.

⚠️ **Et le gabarit n'est pas la seule forme qui casse une extraction naïve** : l'appel lui-même
s'écrit parfois **sur plusieurs lignes** — `supplier-invoices/import/+page.svelte:85-88` place
`i18nMsg(`, sa clé et son repli sur trois lignes distinctes. Une extraction ligne à ligne les
manque toutes.

**Conséquence pour l'implémentation, et c'est un AC** : l'extraction de la garde B ne doit pas
reposer sur une classe de caractères négative, ni sur un balayage ligne à ligne. Elle doit apparier l'appel `i18nMsg(` puis **lire le
littéral en respectant l'échappement et l'imbrication** — ou, à défaut, recenser séparément **tous**
les gabarits `` ` ``…`${`…`}`…`` `` et exiger que chacun soit déclaré dans `MOTIFS_DYNAMIQUES`.
Une garde qui hérite du défaut de méthode de sa propre spec ne verrait pas le prochain
`bank-import-info-*`.

## Décisions

**D1 — Garde A compare des ENSEMBLES DE CLÉS, jamais des textes rendus.**
Un test dans `crates/kesh-i18n/src/loader.rs` (module `tests`, accès au champ privé `keys`) ou,
si l'on préfère un test d'intégration, derrière un accesseur `pub fn keys(&self, locale: &Locale) -> &[String]`
ajouté à `I18nBundle`. **Le critère est l'égalité des quatre ensembles**, dans les deux sens :
une clé présente seulement en `de-CH` est un défaut au même titre (aujourd'hui il y en a **0**,
mesuré — la garde le maintient).
⚠️ **Ne PAS généraliser `assert_ne!(msg, fr)`** — motif à la § *Contexte*.

**D1-bis — Le lecteur de littéral vit dans un MODULE PARTAGÉ, pas dans le fichier de garde.**
`frontend/src/lib/shared/i18n-literal-reader.js` — ESM neutre, importable par vitest **comme** par
`node scripts/…`. La garde B l'importe, son test d'AC7-bis l'importe, et le moissonneur de la 23-1b
l'importe.
⚠️ **Sans ce module, `AC10` de la 23-1b est irréalisable** : il exige « le **même** lecteur » alors
que la garde vivrait dans un `.test.ts` — qu'un script `node` ne peut pas importer — et que les Dev
Notes de la 23-1b interdisent d'y toucher. La seule issue serait la recopie, donc une **cinquième**
récidive de la classe de caractères négative, sans test pour la révéler.
⚠️ **Ce module est la SEULE exception à l'interdiction faite par AC7** au fichier de garde
d'importer du code de production : c'est un utilitaire de lecture, sans clé ni catalogue.

**D2 — Garde B généralise `duplicate-i18n-keys.test.ts`, elle ne le double pas — MAIS le fichier
porte DEUX contrôles, et le second n'est repris par aucune garde générale.**
⚠️ Outre l'existence des clés, il vérifie que *« le catalogue n'a pas de clé du domaine que
PERSONNE ne demande »* — le contrôle des **orphelines**, vert aujourd'hui (5 clés au catalogue,
5 demandées). **Il ne se généralise PAS** : le catalogue sert aussi `kesh-qrbill`, `kesh-report`
et les PDF, si bien qu'une clé sans demandeur côté frontend n'est pas orpheline pour autant —
`reports-filename-*` en donne le contre-exemple immédiat, avec **7** clés déclarées pour **5**
valeurs de `ReportType`. Le contrôle est donc **conservé, borné à son domaine** : la garde B
reprend l'assertion pour les préfixes explicitement déclarés « à couverture close »
(`contact-duplicate-*` pour commencer), et cette liste s'étend story par story.
⚠️ *Sans cette décision, AC12 promettait « le contrôle qu'il exerçait ne doit pas disparaître » et
livrait sa suppression. Relevé en passe 3.*
Le fichier `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` (22-2b) est le
prototype : même façon de lire les `.ftl`, même parcours de `src`. La story le **remplace** par
un test de portée générale — `frontend/src/lib/shared/i18n-keys.test.ts` — et supprime l'ancien,
dont la borne `DOMAINE = /^contact-duplicate-/` n'a plus de raison d'être **pour son premier
contrôle**. **Un seul test, pas deux** — mais qui porte les **deux** assertions, la seconde restant
bornée par domaine (cf. l'encadré ci-dessus).

**D2-bis — `contacts-i18n-realpath.test.ts` N'EST PAS supprimé.**
Le seul test retiré est `duplicate-i18n-keys.test.ts` (D2). ⚠️ L'autre test i18n de la 22-2b,
`frontend/src/routes/(app)/contacts/contacts-i18n-realpath.test.ts`, prouve **le chemin de rendu**
— le ternaire singulier/pluriel, avec le dictionnaire réel et sans mock d'`i18nMsg` — et non
l'existence des clés. Les deux gardes de cette story ne le recouvrent en rien : une clé peut
exister dans les quatre locales **et** être choisie à tort par le code qui la sélectionne.

**D3 — Les allowlists sont DÉCROISSANTES PAR CONSTRUCTION.**
Deux fichiers de données, `frontend/src/lib/shared/i18n-dette-connue.ts` (clés jamais entrées au
catalogue) et `crates/kesh-i18n/dette-parite-connue.txt` (les 57 de [#283]), triés, une clé par
ligne, chacun ouvert par un commentaire qui dit *ce que c'est* et *quelle story le vide*.
⚠️ **Chaque garde échoue AUSSI quand son allowlist contient une clé désormais présente** — sans
quoi l'allowlist se fossilise et la garde devient décorative. C'est la moitié qui compte : elle
force chaque story de rollout à retirer ses lignes.
⚠️ **Une entrée « plus demandée » a DEUX causes, et le message d'échec doit nommer les deux** :
la feature a été retirée (**retirer la ligne**), ou **l'extracteur ne la voit plus** (**réparer
l'extracteur**). *La seconde se vérifie d'abord* — c'est la cause dominante de ce dossier, où
l'extracteur a raté successivement 29 clés puis 6 autres. Le geste que suggère spontanément le
message d'échec est de supprimer la ligne : ce serait effacer la dette et sortir de couverture le
code qui la demande.
⚠️ **Et le contrôle est SYMÉTRIQUE dans l'autre sens aussi** : une entrée qui n'est **plus demandée
par aucun code** (feature retirée plutôt que traduite) doit faire échouer la garde de la même
façon. Sans cela, « décroissante par construction » n'est vrai que d'un côté : l'allowlist
retiendrait indéfiniment des clés mortes, et son décompte cesserait de mesurer la dette réelle.

**D4 — Les motifs dynamiques sont DÉCLARÉS, avec leurs valeurs, et bornés — 8 préfixes, 10 sites.**
Une table `MOTIFS_DYNAMIQUES` dans la garde B : `{ motif, valeurs[] }`. La garde vérifie chaque
`motif.replace(valeur)` comme une clé ordinaire.
⚠️ **Les valeurs sont écrites EN DUR dans le test, pas lues depuis le code de production.** Lire
la carte de production ferait qu'une carte vidée par erreur rendrait le test vert à vide — le
mode d'échec du **test muet**, déjà payé sur `backfill_skips_archived_accounts` (16-1a). En
contrepartie, chaque motif porte une assertion de **cardinalité attendue** (`expect(...).toBe(...)`,
la garde B étant en vitest) qui rougit dès que la carte de production évolue sans le test — **sauf
pour les deux ensembles ouverts signalés plus bas**.
⚠️ **Cette clause est exigible, pas seulement souhaitée** : AC7 impose que le fichier de garde
**n'importe aucun module de production**, ce qui se vérifie d'un coup d'œil sur ses `import` — au
même titre qu'AC2 pour `format()`.
⚠️ **`vat-category-*` est un ANGLE MORT ASSUMÉ, et l'appeler autrement serait mentir.** Ses
valeurs ne sont ni un littéral ni une énumération fermée : c'est une **colonne libre**. La
migration `20260613000001_vat_rates_crud.sql:12-16` l'écrit noir sur blanc — *« **PAS de contrainte
`CHECK IN` (liste fermée)** : décision projet (Story 11-1) — les autorités peuvent introduire de
NOUVELLES catégories officielles sans migration de schéma »* — et `validate_category`
(`kesh-api/src/routes/vat.rs:243-252`) ne contrôle que « non vide » et « ≤ 32 caractères ». Un
administrateur qui crée un taux `category = "covid-temporaire"` **étend l'espace des clés sans
toucher au code**, donc sans qu'aucune assertion de cardinalité ne puisse rougir.

La déclaration porte donc les **cinq catégories seedées**, et la table indique explicitement, en
commentaire, que **les catégories créées par un administrateur ne sont couvertes par aucune
garde** — leur libellé retombera sur le repli, qui est ici la catégorie brute. *Prétendre que la
cardinalité protège ce motif serait exactement le genre d'assurance fausse que les deux gardes
existent pour supprimer.*

⚠️ **`bank-import-info-*` est le SECOND angle mort, et il est pire que le premier.** Ses valeurs
ne sont pas énumérées côté frontend du tout : elles sont poussées par le **backend Rust** —
`warnings.push("bank_csv_profile_auto_matched")` (`kesh-api/src/routes/bank_imports.rs:1668`) et
`bank_csv_multiple_profile_matches` (`:693`) —, le type frontend n'étant qu'un `informational:
string[]` (`bank-import.types.ts:49`). **Il n'y a donc aucune « carte de production » à confronter**,
et l'assertion de cardinalité y est verte à jamais quoi qu'il arrive côté Rust. Un troisième code
informationnel s'afficherait en `snake_case` brut, dans les quatre langues, sans qu'aucune garde ne
bouge — le mode d'échec du **test muet**, sur le préfixe même dont l'oubli était le CRITICAL de la
passe 1. *La promesse de D4 (« l'assertion rougit dès que la carte évolue ») ne vaut que pour les
six motifs dont l'énumération vit dans `frontend/src` ; elle est FAUSSE pour ces deux-là.*

⚠️ **Les valeurs se déclarent APRÈS transformation.** Le gabarit est
`` `bank-import-info-${info.replace(/_/g, '-')}` `` : les valeurs à écrire sont
`bank-csv-profile-auto-matched` et `bank-csv-multiple-profile-matches`, **en tirets**. Les déclarer
en `snake_case` ferait chercher une clé inexistante, donc signaler une dette imaginaire — et le
réflexe de l'inscrire à l'allowlist masquerait durablement deux clés qui sont **traduites dans les
quatre locales**.

⚠️ **La garde compare la LISTE des sites**, pas leur nombre — cf. AC7 —, contre la liste de
référence ci-dessous, recomptée le 2026-08-19 :

| fichier | préfixe |
|---|---|
| `lib/features/bank-import/BankImportUpload.svelte:547` | `bank-import-info-` |
| `lib/features/journal-entries/JournalEntryForm.svelte:279` | `journal-` |
| `lib/features/journal-entries/VatPurchaseAssistant.svelte:62` | `vat-category-` |
| `lib/features/reminders/reminder-error-label.ts:38` | `reminders-error-` |
| `lib/features/reports/reports.api.ts:250` | `reports-filename-` |
| `routes/(app)/accounts/+page.svelte:37` | `account-type-` |
| `routes/(app)/invoices/due-dates/+page.svelte:344` | `due-dates-filter-` |
| `routes/(app)/journal-entries/+page.svelte:355` | `journal-` |
| `routes/(app)/settings/vat-rates/+page.svelte:47` | `vat-category-` |
| `routes/(app)/supplier-invoices/import/+page.svelte:68` | `imported-supplier-invoices-error-` |

*Un simple compte laisserait passer un ajout compensant un retrait ; et le dev ne doit pas
**produire** cette liste par l'extraction qui a déjà échoué deux fois, mais la **confronter**.*

**D4-ter — L'INVERSION : on n'énumère plus les formes qui marchent, on inventorie les sites qui ne
résolvent pas.**

⚠️ **Cette décision remplace la méthode des passes précédentes, et il faut dire pourquoi.** Cinq
fois de suite, cette spec a énuméré une *forme d'appel* — littéral direct, gabarit dynamique, relais,
appel multi-ligne, clé portée par une table — et **cinq fois la passe suivante en a trouvé une
sixième** : ternaire à l'intérieur du premier argument (`contacts/+page.svelte:646`), gabarit affecté
à une variable (`AccountingTooltip.svelte:40-41`), clé fabriquée par une fonction
(`accountRoleKey`), clé lue dans une colonne libre (`vat-rates/+page.svelte:52`). *Une énumération
de formes est ouverte par nature : elle ne peut pas prouver sa propre complétude.*

**La question robuste est le complément** : non pas « quelles formes la clé prend-elle ? », mais
**« quels sites d'appel ne résolvent PAS en littéral ? »**. Cet ensemble est **clos, comptable et
décidable** — mesuré par le lecteur réel : **33 sites sur 1493**.

**La garde procède donc ainsi** :

1. elle parcourt **tous** les appels à `i18nMsg` et aux relais — **1493 sites** ;
2. tout site dont le premier argument est un **littéral** ou un **gabarit** est résolu comme
   aujourd'hui (D4, D4-bis) ;
3. tout site dont le premier argument est **autre chose** entre à l'**inventaire des sites non
   résolus**, avec une **assertion de cardinalité (`33`)** ;
4. chaque entrée de l'inventaire est **soit résolue** — ses clés déclarées en dur, comme les valeurs
   des motifs dynamiques — **soit écrite comme angle mort**, avec sa raison.

⚠️ **C'est l'assertion de cardinalité de l'étape 3 qui porte la garantie**, et elle est d'une autre
nature que celles des passes précédentes : elle ne dépend d'aucune énumération de formes. Un site
d'une **sixième** forme inconnue y tombe automatiquement et fait rougir le test, au lieu de
disparaître en silence. **C'est la seule assertion de ce document qu'une forme imprévue ne puisse
pas contourner.**

⚠️ **Elle corrige aussi un défaut de conception d'`AC7`** : celui-ci faisait comparer la liste des
10 sites dynamiques à la table de D4 — **mais les deux côtés sortaient du même extracteur**, donc
l'assertion était verte par construction. L'inventaire, lui, se compare au nombre total d'appels,
que l'extracteur ne choisit pas.

⚠️ **Les chiffres `33` et `1493` sont ceux du LECTEUR RÉEL, après la passe 1 de revue de code.** La
spécification annonçait 54 et 1514 : ces valeurs venaient d'une mesure d'estimation par expression
régulière, qui comptait aussi les **définitions** des relais et des mentions en commentaire. *La
spec a été corrigée sur l'outil, et non l'inverse — c'est le sens de la § « Recompter ses propres
comptes rendus ».* La revue de code les a ensuite affinés une seconde fois — de 36 sur 1496 à
**33 sur 1493** —, en masquant les commentaires (trois docstrings citaient `i18nMsg` en exemple) et
en rendant visibles les appels membres.

**Ce que l'inventaire a rendu à sa première exécution** — sites énumérables, donc **résolus** :

| site | famille | clés | manquantes |
|---|---|---|---|
| `routes/(app)/+layout.svelte:154` | `nav-*` (table `i18nKey`) | 22 | **4** |
| `routes/(app)/reports/+page.svelte:628` | onglets de rapport (`labelKey`) | 8 | **2** |
| `lib/features/invoices/PaymentStatusBadge.svelte:24` | `payment-status-*` (`LABEL_KEY[…]`) | 3 | 0 |
| `accounts/+page.svelte:62`, `opening-balances:133`, `BalanceSheetView:205` | `account-role-*` (`accountRoleKey()`) | 10 | 0 |
| `contacts/+page.svelte:646, :732, :829` | `contact-type-*` (ternaire) | 2 | 0 |
| `BankProfileForm.svelte:256` | `bank-import-profile-labels-*` (ternaire) | 12 | 0 |
| `AccountingTooltip.svelte:49, :51` | `tooltip-*-natural` / `-technical` (gabarit en variable) | 8 | 0 |

**Six clés manquantes révélées** — `nav-credit-notes`, `nav-email-templates`, `nav-projects`,
`nav-supplier-invoices-import`, `reports-project-expenses`, `reports-project-return`. ⚠️ **Les
quatre premières sont des entrées de la BARRE DE NAVIGATION PRINCIPALE** : elles s'affichent en
français dans les trois autres langues, sur tous les écrans, depuis toujours. **Clés statiques
manquantes : 279 → 285. Total de l'epic : 346 → 352.**

⚠️ **Un site non énumérable reste un angle mort, mais DÉCLARÉ** : `vat-rates/+page.svelte:52`
(`i18nMsg(r.label, r.label)`) lit la clé dans `vat_rates.label`, **colonne libre sans `CHECK`** — le
**troisième** ensemble ouvert, de même nature que `vat-category-*` et `bank-import-info-*`.

**D5 — Les deux gardes se bornent contre le passage à vide.**
Chacune assert un **minimum de clés effectivement collectées**. *Le principe vient du prototype
22-2b, dont le `expect(demandees.size).toBeGreaterThanOrEqual(5)` bornait la collecte du seul
domaine `contact-duplicate-*` ; la garde générale le reprend à son échelle.*

| borne | valeur | mesuré |
|---|---|---|
| garde A — clés par locale | `>= 1200` | 1216 |
| garde B — littéraux demandés | `>= 1050` | **1151 statiques** (relais compris, D4-bis) |
| garde B — clés issues de fichiers `.ts` | `>= 5` | 5 |

⚠️ **Le compte a changé en passe 1 de la 23-1b, et c'est le chiffre corrigé qui fait foi** :
**1151 littéraux statiques distincts** sont demandés — `i18nMsg` **et les sept relais de D4-bis**
réunis —, contre 1010 quand on ignorait les relais. Sur ces 1151, **872 existent** et **279
manquent**. *(La relation « 1010 = 1002 statiques + 8 gabarits », établie en passe 2, portait sur
le seul périmètre `i18nMsg(` ; elle était exacte et incomplète.)*

⚠️ **Une borne globale ne voit pas une perte PARTIELLE, et c'est mesuré** : **1146** de ces 1151
clés viennent de `.svelte` et **5 seulement** de `.ts`. Un filtre d'extension réduit par erreur à
`/\.svelte$/` ferait tomber le total de 1151 à 1146 — **toujours au-dessus de la borne globale**.
D'où la troisième ligne du tableau.
⚠️ **Et cette borne `.ts` est posée à la valeur MESURÉE (5), non à « mesuré moins une marge ».**
Motif : deux des cinq clés `.ts` viennent d'appels **multi-lignes en syntaxe TypeScript pure**
(`notify.ts:103-110`, un ternaire sans balisage Svelte). Un extracteur dont le support multi-ligne
serait calqué sur la seule forme `.svelte` les perdrait — le compte tomberait à 3, et une borne
posée à `>= 3` serait **verte sur la perte même qu'elle existe pour attraper**.
⚠️ **Cette borne `.ts` compte les fichiers de PRODUCTION**, les `.test.*` étant hors collecte
(cf. D5-bis) — sans quoi la garde, elle-même écrite dans un `.ts`, pourrait se compter et
survivre à la perte de tout ce qu'elle doit surveiller.

**D4-bis — L'extracteur reconnaît les RELAIS LOCAUX, sans quoi il rate 29 clés et un dossier entier.**
Sept fichiers déclarent une fonction qui **transmet** ses arguments à `i18nMsg` :

```ts
function msg(key: string, fallback: string): string { return i18nMsg(key, fallback); }
// …ou à TROIS paramètres — `routes/(app)/+page.svelte:11` :
function msg(key: string, fallback: string, args?: Record<string, string | number>): string {
	return i18nMsg(key, fallback, args);
}
```
⚠️ **La forme canonique est `return i18nMsg(<param>, <param>[, <param>])`.** Un motif à deux
paramètres n'en recense que **6** ; l'assertion rougit à 6, et le réflexe le moins cher est
d'ajuster le nombre attendu plutôt que le motif — ce qui rendrait invisibles les clés de
`routes/(app)/+page.svelte`.

Le littéral ne se trouve alors **pas** au site `i18nMsg(`, mais au site `msg(`. Un extracteur qui
ne cherche que `i18nMsg(` ne voit **rien** de ce que ces fichiers demandent.

⚠️ **Ce n'est pas une hypothèse : c'est ce qui s'est produit.** Le recensement de cette spec —
et donc les décomptes de trois passes de revue — ignorait **29 clés manquantes**, un **dossier
entier** (`routes/onboarding`) absent du découpage de l'epic, et **25 clés de plus** pour
`routes/(app)/settings` (l'écran des modèles d'e-mail à lui seul). Trouvé en passe 1 de la 23-1b,
par la lentille qui cherchait l'intégrité du découpage.

**La garde procède donc en deux temps** : (1) elle **recense les relais** — une fonction dont le
corps se réduit à `return i18nMsg(<param>, <param>)` — et (2) elle collecte les littéraux passés
à `i18nMsg` **et à chacun d'eux**.
⚠️ **Une assertion de cardinalité porte sur le nombre de relais trouvés (`7`)** : un huitième
relais ajouté sans que le test le sache rendrait ses clés invisibles, en silence, exactement comme
aujourd'hui. C'est le même garde-fou que pour les 10 sites dynamiques, et pour la même raison.

⚠️ **C'est le TROISIÈME angle mort de la garde**, après `vat-category-*` et `bank-import-info-*` —
et le seul des trois qui soit **refermable** : les deux autres tiennent à des valeurs produites
hors du frontend, celui-ci n'est qu'une forme d'appel.

**D5-bis — Les fichiers `.test.*` sont HORS COLLECTE, et ce n'est pas un détail de commodité.**
La garde B balaie `frontend/src` **à l'exception des fichiers dont le nom contient `.test.`** —
comme le fait déjà le prototype 22-2b.
⚠️ **Sans cette exclusion, la garde rougirait dès sa première exécution, et pour de mauvaises
raisons** : `i18n.svelte.test.ts` appelle `i18nMsg('compteur', …)` et `i18nMsg('une-cle', …)`,
**clés fictives absentes des quatre catalogues** et qui doivent le rester — les traduire serait
absurde, les inscrire à l'allowlist de dette encore plus, puisqu'elles ne sont pas de la dette.
⚠️ **AC6 dit « tout `frontend/src` » et cette exclusion en est la seule exception** : elle est
écrite ici pour qu'un lecteur ne la prenne pas pour un oubli, ni ne l'élargisse aux `.svelte` de
test.

**D10 — Où les gardes s'exécutent, et pourquoi ça suffit.** *(Constat de cadrage : aucun AC ne
contrôle « aucun nouveau script npm, aucune nouvelle étape de CI » — AC13 vérifie que les gates
passent, pas qu'aucun script n'a été ajouté. Déclaré tel plutôt que laissé implicite, comme D8-bis
dans la moitié sœur.)*
Garde A dans `cargo test -p kesh-i18n` (donc dans le gate backend et en CI). Garde B dans
`npm run test:unit` (donc dans le gate frontend et en CI). **Aucun nouveau script npm, aucune
nouvelle étape de CI** : les deux gates existants les portent. `lint-i18n-ownership` reste tel
quel — il répond à une autre question (l'appartenance d'un namespace à un dossier).
⚠️ **UNE exception, et elle est imposée par la 23-1b (D7-bis, T5) — pas par cette story** :
renommer `delete` en
`contact-persons-delete` **crée une violation d'appartenance**. Le lint compare le *namespace* de
la clé (`getNamespace` → `contact`) au nom du *dossier* (`contacts`) : l'écart singulier/pluriel de
l'issue #30 fait que **les neuf clés sœurs du même composant sont déjà inscrites** à
`KNOWN_VIOLATIONS` (`lint-i18n-ownership.js:104-118`). La ligne `…:delete` devient donc
`…:contact-persons-delete` — **une substitution, pas un ajout**.
⚠️ *Une lentille de passe 2 avait annoncé l'inverse — que le lint « détecterait une exception
morte » et échouerait. **Réfuté au sol** : `KNOWN_VIOLATIONS` n'est consulté que par
`.has(violationKey)` (`:189`), aucune entrée morte n'est détectée. Le défaut réel est symétrique de
celui annoncé, et il n'aurait pas été trouvé sans lire le script.*

## Acceptance Criteria

1. **AC1** — Un test de `kesh-i18n` échoue si les quatre `messages.ftl` ne déclarent pas le
   **même ensemble** de clés, **allowlist de parité déduite**. Le message d'échec **nomme** les
   clés et la locale.

2. **AC2** — Ce test compare des **ensembles de clés**, sans jamais appeler `format()` ni
   `all_messages()`. *(Vérifiable : le corps du test ne mentionne ni l'une ni l'autre.)*

3. **AC3** — Le même test échoue si l'allowlist de parité contient une clé **présente dans au
   moins une** des trois locales cibles, **ou absente de `fr-CH`** (contrôle symétrique, D3).
   ⚠️ **« Au moins une », pas « les quatre »** : une story de rollout qui traduirait en `de-CH` et
   `it-CH` en oubliant `en-CH`, et laisserait la ligne d'allowlist, resterait invisible des deux
   contrôles — la garde verte sur le défaut même qu'elle doit rendre bruyant.

4. **AC4** — Un test vitest échoue si une clé passée à `i18nMsg()`, **à l'un des relais
   (AC7-quater) ou résolue depuis l'inventaire (AC7-quinquies)**, **n'existe dans
   aucun** des quatre catalogues, allowlist de dette déduite. Le message nomme la clé **et le
   fichier** qui la demande.

5. **AC5** — Le même test échoue si l'allowlist de dette contient une clé désormais présente
   **ou une clé que plus rien ne demande** (contrôle symétrique, D3). ⚠️ **« Demandées » est
   l'union des littéraux statiques ET des expansions de `MOTIFS_DYNAMIQUES`** — sans quoi ce
   contrôle chasserait les 10 clés `imported-supplier-invoices-error-*` que l'AC8 impose à cette
   même allowlist, et le raccourci consisterait à les retirer, effaçant la traçabilité vers 23-3.

6. **AC6** — Le test vitest balaie **tout** `frontend/src` — `src/routes/` compris, d'où
   **226 des 285** clés manquantes sont demandées **exclusivement** (228 depuis au moins un fichier
   de `routes/`) —, à **une seule exception écrite** : les fichiers dont le nom
   contient `.test.` (D5-bis), qui demandent des clés fictives (`compteur`, `une-cle`) devant
   rester absentes des catalogues.

7. **AC7** — Les **8 préfixes dynamiques** (sur **10 sites d'appel**) sont déclarés avec leurs
   valeurs et contrôlés comme des clés ordinaires ; chaque préfixe porte une assertion de
   cardinalité, et la garde **compare la LISTE des sites** (fichier + préfixe) à la **liste de
   référence donnée en D4** — et non leur seul nombre : un compte dit « 9 au lieu de 10 » sans dire
   lequel, et un ajout compensant un retrait passerait inaperçu. **Le fichier de garde n'importe
   aucun module de production** : les valeurs et les cardinalités y sont écrites en dur (D4), ce
   qui se vérifie sur ses `import`.

7-quater. **AC7-quater** — La garde **recense les relais locaux** (D4-bis) et collecte les
   littéraux passés à chacun, en plus de ceux passés à `i18nMsg`. Une assertion de cardinalité
   porte sur le **nombre de relais trouvés (7)**. *(Sans quoi 29 clés manquantes et le dossier
   `routes/onboarding` restent invisibles — c'était l'état de cette spec jusqu'à la passe 1 de la
   23-1b.)*
7-bis. **AC7-bis** — L'extracteur de la garde B porte **son propre test**, qui le confronte aux
   **trois** formes réelles connues pour casser une extraction naïve :
   (a) un gabarit dont **l'interpolation contient des apostrophes** —
   `` i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info) ``
   (`BankImportUpload.svelte:547`), que ne traverse aucune classe `[^'"`]*` ;
   (b) un appel **réparti sur plusieurs lignes en balisage Svelte** —
   `supplier-invoices/import/+page.svelte:85-88` — que ne voit aucun balayage ligne à ligne ;
   (c) un appel **multi-ligne en syntaxe TypeScript pure** — `notify.ts:103-110`, un ternaire —
   pour que le support multi-ligne ne soit pas calqué sur la seule forme `.svelte`.
   ⚠️ **La robustesse porte sur le PREMIER argument, la clé.** Le contenu du repli n'est ni lu ni
   requis par la garde B — c'est le **moissonneur** (23-1b, D6/AC10) qui lit le second argument, et
   qui doit alors employer le même lecteur de littéral. *(L'entrée « Passe 3 » du Change Log dit
   « AC7-bis vaut pour les deux arguments d'`i18nMsg` » : c'était vrai de la story unifiée, où le
   moissonneur vivait dans le même périmètre. Après le split, la clause se lit ici pour la clé, et
   dans la 23-1b pour le repli.)*
   Sans ce test, la garde hérite du défaut de méthode qui a fait manquer un motif entier à la
   première rédaction de cette spec.

7-quinquies. **AC7-quinquies** — La garde tient l'**inventaire des sites non résolus** (D4-ter) :
   tout appel à `i18nMsg` ou à un relais dont le premier argument n'est ni littéral ni gabarit y
   figure, avec une **assertion de cardinalité (`33`)** et une **assertion sur le nombre total de
   sites d'appel (`1493`)**. Chaque entrée est soit **résolue** (clés déclarées en dur), soit
   **commentée comme angle mort**. ⚠️ *C'est la seule assertion du document qu'une forme d'appel
   imprévue ne puisse pas contourner : elle ne repose sur aucune énumération de formes.*
7-ter. **AC7-ter** — Les **trois** ensembles ouverts portent, en commentaire, le fait qu'aucune
   garde ne les borne : `vat-category-*` (colonne libre, sans `CHECK`, catégories créées par un
   administrateur), **`vat_rates.label`** (`vat-rates/+page.svelte:52` — la clé elle-même vient de
   la colonne libre) et **`bank-import-info-*`** (valeurs poussées par
   `crates/kesh-api/src/routes/bank_imports.rs`, aucune carte frontend à confronter — le fichier
   Rust est nommé comme source de vérité). Leurs valeurs sont déclarées **après transformation**
   (`bank-csv-profile-auto-matched`, en tirets).

8. **AC8** — Les **10 clés `imported-supplier-invoices-error-*`** révélées par cette énumération
   figurent dans l'allowlist de dette avec le commentaire qui dit **quelle story les videra**
   (23-3).

9. **AC9** — Chaque garde porte une **borne de collecte minimale** (D5) qui la fait échouer si
   son motif d'extraction cesse de trouver quoi que ce soit, **et la garde B porte une seconde
   borne par extension** (**≥ 5** clés collectées depuis des `.ts`, la valeur mesurée) — une perte
   totale de la couverture `.ts` ne coûterait que 5 clés sur 1151 et resterait au-dessus de la
   borne globale. ⚠️ **≥ 5 et non ≥ 3** : deux des cinq clés `.ts` viennent d'appels multi-lignes en
   TypeScript pur, dont la perte ramènerait le compte à 3 — une borne à 3 serait **verte sur la
   perte même qu'elle existe pour attraper**.

12. **AC12** — `duplicate-i18n-keys.test.ts` est **supprimé**, sa fonction étant reprise par la
    garde générale. *(Le contrôle qu'il exerçait ne doit pas disparaître : les **cinq** clés
    `contact-duplicate-heading`, `-others-count`, `-others-count-one`, `-ide-active`,
    `-ide-archived` restent couvertes — leur **existence** par la garde B, leur **présence dans
    les quatre locales** par la garde A.)*

12-ter. **AC12-ter** — Le **second** contrôle du fichier supprimé — « le catalogue n'a pas de clé
    du domaine que personne ne demande » — est **repris dans la garde B, borné aux préfixes
    déclarés à couverture close** (`contact-duplicate-*` pour commencer, D2). Il ne se généralise
    pas : le catalogue sert aussi les PDF et les rapports, et `reports-filename-*` déclare 7 clés
    pour 5 valeurs de `ReportType`. La liste vit dans le fichier de garde sous le nom
    **`PREFIXES_A_COUVERTURE_CLOSE`**, avec un commentaire disant ce qu'y inscrire : un préfixe
    dont **toutes** les clés du catalogue sont demandées par le frontend. Elle s'étend story par
    story, jamais par défaut. ⚠️ **La garde échoue si cette liste est vide, ou si les clés du
    catalogue qu'elle couvre sont moins de 5** — c'est la seule collection de la story sans borne
    anti-test-muet, et la pression est vers le petit. ⚠️ **Les nommer plutôt que les
    compter est délibéré** : la première rédaction disait « quatre », valeur juste jusqu'à la
    passe 4 de la 22-2b qui a ajouté `-others-count-one`. Un nombre se démode en silence, une
    liste se confronte.

12-bis. **AC12-bis** — `frontend/src/routes/(app)/contacts/contacts-i18n-realpath.test.ts`
    **existe toujours et passe**. Il prouve le *chemin de rendu* (D2-bis) et non l'existence des
    clés ; aucune des deux gardes ne le remplace, et rien d'autre que cet AC n'empêche de le
    supprimer au passage de T2.

13. **AC13** — Les deux gates complets passent : backend (`cargo test --workspace`) et frontend
    (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`).

## Tasks / Subtasks

- [x] **T1 — Garde A, parité inter-locales** (AC1, AC2, AC3, AC9)
  - [x] `frontend/src/lib/shared/i18n-literal-reader.js` — module partagé, importable par vitest ET par `node` (D1-bis)
  - [x] Accesseur `keys()` sur `I18nBundle` si le test est hors module, sinon usage direct du champ privé
  - [x] Comparaison des quatre ensembles dans les deux sens, message d'échec nommant clés et locale
  - [x] `crates/kesh-i18n/dette-parite-connue.txt` — les 57 clés, triées, en-tête explicatif
  - [x] Assertions « allowlist obsolète » **dans les deux sens** + borne de collecte (`>= 1200` par locale)

- [x] **T2 — Garde B, existence des clés demandées** (AC4, AC5, AC6, AC9, AC12, AC12-bis, AC12-ter)
  - [x] `frontend/src/lib/shared/i18n-keys.test.ts` — parcours de tout `src` **hors fichiers `.test.*`** (D5-bis), lecture des 4 `.ftl`
  - [x] `frontend/src/lib/shared/i18n-dette-connue.ts` — **285 + 10** clés, triées, en-tête qui nomme les stories de résorption
  - [x] Assertions « allowlist obsolète » **dans les deux sens** + borne globale (`>= 1050`) + borne `.ts` (`>= 5`) — valeurs du tableau de D5, et non celles d'avant le recompte des relais
  - [x] Suppression de `duplicate-i18n-keys.test.ts` — **ses DEUX contrôles repris** (existence + orphelines bornées, AC12-ter) — **et conservation de `contacts-i18n-realpath.test.ts`**

- [x] **T3 — Motifs dynamiques, relais et inventaire** (AC7, AC7-bis, AC7-ter, AC7-quater, AC7-quinquies, AC8)
  - [x] Table `MOTIFS_DYNAMIQUES` : **8 préfixes**, valeurs en dur, `bank-import-info-*` compris
  - [x] Assertion de cardinalité par préfixe **+ assertion sur le nombre de sites (10)**
  - [x] Extracteur robuste **aux apostrophes dans l'interpolation, aux appels multi-lignes Svelte ET TypeScript**, avec son test à trois fixtures (AC7-bis)
  - [x] **Recensement des 7 relais locaux** et collecte de leurs littéraux, avec assertion de cardinalité (AC7-quater, D4-bis)
  - [x] Comparaison de la **liste** des 10 sites contre la table de référence de D4
  - [x] Aucun `import` de module de production dans le fichier de garde (AC7) — **hors `i18n-literal-reader` (D1-bis)**
  - [x] **Inventaire des sites non résolus** : 33 sites, assertion de cardinalité + assertion sur les 1493 sites d'appel (AC7-quinquies)
  - [x] Résolution des 7 familles énumérables, dont les **6 clés manquantes** (`nav-*` ×4, `reports-project-*` ×2) inscrites à l'allowlist
  - [x] Commentaires d'angle mort sur **`vat-category-*` ET `bank-import-info-*`** (AC7-ter), valeurs déclarées **après transformation**
  - [x] Les 10 `imported-supplier-invoices-error-*` en allowlist, commentaire « résorbées par 23-3 »

- [x] **T6 — Gates** (AC13)
  - [x] Gate backend complet, gate frontend complet, avant tout push

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne traduire AUCUNE clé.** La borne de cette moitié est zéro, pas vingt : les 20 clés du pilote
  sont le travail de la 23-1b. *(Cette ligne disait « ne pas traduire au-delà des 20 clés du
  pilote » — vraie de la story unifiée, fausse ici, et contredisant le bandeau d'ouverture de sa
  propre page. Relevée en passe 1 du split.)*
- **Ne pas toucher `KNOWN_VIOLATIONS`** de `lint-i18n-ownership.js` : aucune tâche de cette moitié
  (T1, T2, T3, T6) n'a de raison d'y toucher. *La substitution de la ligne `…:delete` →
  `…:contact-persons-delete` est le travail de **T5, dans la 23-1b**, imposée par son D7-bis.*
- **Ne pas toucher au moissonneur ni aux replis** : ils sont le périmètre de la 23-1b.
- **Ne pas ajouter de clé au catalogue sans l'ajouter aux QUATRE locales** : ce serait creuser
  [#283] dans la story qui vient la borner, et la garde A le refuserait de toute façon.

### Ce que le dev doit lire avant d'écrire

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-i18n/src/loader.rs:130-143` | `all_messages` et son repli `fr-CH` — la raison d'être de D1 |
| `crates/kesh-i18n/src/loader.rs:34` | le champ `keys`, seul observable honnête |
| `crates/kesh-i18n/src/loader.rs:261` et `:296` | les deux gardes bornées existantes (16-3b, 22-2b) et leur ruse `assert_ne!` |
| `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` | le prototype à généraliser |
| `frontend/src/lib/shared/utils/i18n.svelte.ts:14-23` | `i18nMsg`, son repli, son interpolation `{ $var }` |
| `frontend/scripts/lint-i18n-ownership.js` | ce que l'autre lint fait — et ne fait pas |
| `docs/i18n-glossaire.md` | terminologie contraignante (partie A) et registre mesuré |

### Chiffres de référence — recomptés le 2026-08-19, à re-vérifier avant de les réécrire

| | valeur |
|---|---|
| clés `fr-CH` / `de-CH` / `en-CH` / `it-CH` | 1273 / 1216 / 1216 / 1216 |
| clés de [#283] | 57, **même ensemble** sur les trois locales, 0 clé en trop |
| littéraux demandés (`i18nMsg` **+ les 7 relais**) | **1151** statiques distincts, dont **866 existent** et **285 manquent** ; s'y ajoutent **8 gabarits dynamiques** sur 10 sites |
| clés statiques manquantes | **285**, sur **14** dossiers — au sens `lib/features/<domaine>` et `routes/(app)/<section>`, un niveau sous la racine fonctionnelle (relais compris, D4-bis) |
| préfixes dynamiques / sites d'appel | **8** / **10** |
| clés révélées par les motifs dynamiques | **+10** (`imported-supplier-invoices-error-*`) |
| **total de l'epic** | **352** (285 statiques + 10 de la famille dynamique + 57 de parité) |
| replis moissonnables | *hors périmètre — cf. 23-1b, qui les recompte* |

Commande de recompte, à exécuter et non à croire :

```sh
cd crates/kesh-i18n/locales
LC_ALL=C comm -23 <(grep -oE '^[a-zA-Z][A-Za-z0-9_-]* *=' fr-CH/messages.ftl | LC_ALL=C sort -u) \
                  <(grep -oE '^[a-zA-Z][A-Za-z0-9_-]* *=' de-CH/messages.ftl | LC_ALL=C sort -u) | wc -l
```

⚠️ **La classe de caractères est celle du parseur, et ce n'est pas un détail de style.** Un
`[a-z0-9-]` rate `email-templates-type-invoice_send` et `…_reminder` — les deux seules clés à tiret
bas — et rend **1271** là où le tableau ci-dessus annonce **1273**. Le delta de 57 reste juste par
accident (les deux clés existent dans les quatre locales), mais une commande « à exécuter et non à
croire » qui contredit le tableau de sa propre section fait conclure à une dérive inexistante.
*C'est le jumeau, côté shell, de la classe négative qu'AC7-bis interdit côté extraction : troisième
famille du même défaut dans ce document.*

⚠️ **`LC_ALL=C` n'est pas cosmétique** : sans lui, `comm` avertit sur stderr que ses entrées ne
sont pas triées (la collation `fr_FR.UTF-8` ordonne les tirets autrement que `sort` ne le suppose)
et son résultat n'est plus garanti. Il rend 57 ici **par chance de collation**, ce qui est
précisément le genre de commande « à exécuter et non à croire » qui trompe sur un autre poste.

⚠️ **Le décompte du registre de D9 — qui vit dans la 23-1b — porte sur des MESSAGES, pas sur des
lignes** : 115 messages
`de-CH` emploient « Sie » ; un `grep -c` en rend **117**, les messages multi-lignes comptant deux
fois. *(Écart relevé en passe 1 de `validate` et **réfuté** après recompte : le chiffre de la spec
était le bon, l'unité manquait.)*

### References

- [Source: `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`] — plan d'epic, découpage, risques
- [Source: `docs/i18n-glossaire.md`] — terminologie contraignante, registre mesuré
- [Source: `crates/kesh-i18n/src/loader.rs`] — repli `fr-CH`, champ `keys`, gardes bornées existantes
- [Source: `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts`] — prototype de la garde B
- [Source: `CLAUDE.md` § *Recompter ses propres comptes rendus*] — tout décompte de cette story se recompte
- [Source: `CLAUDE.md` § *Migration breaking policy* P6] — précédent du **test muet**, motif de D4 et D5
- [#316] : https://github.com/guycorbaz/kesh/issues/316
- [#283] : https://github.com/guycorbaz/kesh/issues/283

## Change Log

### Création — 2026-08-19, Opus 5

Spec créée après le kickoff de l'epic. **Trois choses ont été établies pendant la spécification
plutôt que supposées**, et chacune a changé le contenu :

1. **Le défaut a deux chemins et non un** (`all_messages` replie sur `fr-CH`) — d'où D1, et le
   rejet explicite de la généralisation d'`assert_ne!(msg, fr)`.
2. **L'énumération des 8 motifs dynamiques révèle 10 clés manquantes de plus** — le total de
   l'epic passe de 307 à 317, la 23-3 de 99 à 109.
3. **Les 5 clés sans repli littéral sont toutes dans un seul composant** et demandent des entrées
   Fluent à variables — sorties du périmètre de la story-zéro, portées par la 23-5.

**Et une affirmation de cette même spec a été réfutée par sa propre vérification**, avant
relecture extérieure : D8 déclarait « aucun terme de la partie B dans le pilote (vérifié) »
alors que trois y figurent, sur 10 des 20 clés. Corrigé sur place, avec la mention de l'erreur —
*une correction n'est pas exempte du défaut qu'elle corrige.*

### Passe 1 de `bmad-create-story validate` — 2026-08-19, Sonnet ×3, contextes frais

**Trend : 2 CRITICAL · 1 HIGH · 8 MEDIUM · 3 LOW** (BlindHunter 0/0/2/0 — EdgeCaseHunter 2/0/5/1 —
AcceptanceAuditor 0/1/3/2 ; l'AC12 a convergé sur les **trois** lentilles).

**Les deux CRITICAL portent sur le même endroit — le § qui se vantait d'avoir énuméré plutôt que
supposé.**

- **ECH-1** — le motif `bank-import-info-*` manquait, et **la cause était la méthode d'extraction
  de la spec elle-même** : une classe `[^'"`]*` ne peut pas traverser un gabarit dont
  l'interpolation contient des apostrophes. Le recensement rendait « 8 littéraux » là où il y a
  **8 préfixes sur 10 sites**, la coïncidence numérique masquant la famille absente. Corrigé, et
  transformé en exigence : **AC7-bis** impose que l'extracteur de la garde résiste à ce cas, faute
  de quoi la garde hériterait du défaut qui l'a fait naître.
- **ECH-2** — `vat-category-*` était présenté comme borné par ses « cinq catégories seedées ».
  Vérifié dans la source : la colonne n'a **aucune contrainte `CHECK`**, par décision explicite de
  la Story 11-1, et `validate_category` n'impose que « non vide, ≤ 32 ». Une catégorie créée par un
  administrateur échappe donc à toute assertion de cardinalité. Requalifié en **angle mort assumé
  et écrit** (AC7-ter) plutôt qu'en garantie.

**Le HIGH est une contradiction interne** : D7 concluait « aucune décision terminologique n'est
prise dans la story-zéro » quand D8, six lignes plus bas, disait le contraire et le disait
*explicitement à propos de D7*. J'avais corrigé le site signalé sans remonter à sa source — le
mode d'échec que la § *Propagation post-patch* du `CLAUDE.md` décrit. D7 réécrit.

**MEDIUM retenus** : AC12 disait « quatre » clés `contact-duplicate-*` pour **cinq** (les trois
lentilles l'ont vu ; désormais **nommées** plutôt que comptées) ; D2-bis et la promotion des trois
termes au glossaire n'étaient exigés par aucun AC (**AC12-bis** ici, **AC11-bis** dans la 23-1b) ; la borne
anti-test-muet ne voyait pas une perte de la seule extension `.ts` (5 clés sur 1002 — seconde
borne ajoutée) ; les allowlists n'étaient décroissantes que dans un sens (contrôle symétrique
ajouté) ; le moissonneur n'avait pas de périmètre écrit (13 replis interpolés au lieu de 5) et ne
disait rien des **6 clés à repli divergent** ; la clé `delete` allait entrer au catalogue **sans
domaine** (**D7-bis** : renommée `contact-persons-delete`) ; et le glossaire annonçait une
occurrence « unique » du pluriel de courtoisie italien pour **2 « Aggiungete » et 10 « vostro »**.

**Un LOW réfuté, et dit comme tel** : l'écart « 115 vs 117 » sur le registre allemand n'en est pas
un — 115 **messages** contiennent « Sie », 117 **lignes**. Le chiffre était juste, l'unité
manquait ; elle a été ajoutée. L'autre LOW (absence de `LC_ALL=C` dans la commande de recompte) est
retenu et corrigé.

### Passe 2 de `bmad-create-story validate` — 2026-08-19, Haiku ×3, contextes frais

**Trend : 0 CRITICAL · 4 HIGH · 6 MEDIUM · 2 LOW**, braquées en priorité sur ce que la passe 1
venait d'écrire. ⚠️ **La sévérité maximale descend de CRITICAL à HIGH** — la boucle converge au
sens de la § *Règle de splitting préventif*, qui exige un découpage sur `N+1 ≥ N`, pas sur la durée.

**Trois findings de passe 2 portent sur des patches de passe 1**, ce qui confirme le motif de
l'Epic 22 : la sévérité se déplace vers ce qu'on vient d'écrire.

- **La borne `.ts` (D5)** : mon insertion avait placé le nouveau paragraphe juste avant la phrase
  qui parlait du prototype 22-2b, faisant lire « ≥ 3 » et « ≥ 5 » comme deux valeurs d'une même
  borne. Ce sont deux bornes distinctes ; D5 est désormais un tableau.
- **1010 contre 1002** : les deux chiffres étaient justes et leur relation n'était écrite nulle
  part — **1010 littéraux distincts = 1002 statiques + 8 gabarits dynamiques**. Dit une fois, à
  l'endroit qui borne.
- **Le renommage de `delete` (D7-bis)** contredisait D10 (« `KNOWN_VIOLATIONS` n'est pas
  touchée »). Le renommage **crée** une violation d'appartenance, l'écart singulier/pluriel de
  l'issue #30 mettant déjà les neuf clés sœurs à l'allowlist du lint : la ligne se **substitue**.

**Deux angles morts réels trouvés dans le mécanisme** : les fichiers `.test.*` demandent des clés
**fictives** (`compteur`, `une-cle`) que la garde aurait signalées comme dette dès sa première
exécution — d'où **D5-bis**, qui écrit l'exclusion au lieu de la sous-entendre ; et l'appel
`i18nMsg(` de `supplier-invoices/import` s'étend **sur plusieurs lignes**, forme qu'aucun balayage
ligne à ligne ne voit — ajoutée au test d'extracteur d'**AC7-bis**.

**Trois affirmations RÉFUTÉES au sol, et c'est le mode d'échec Haiku que le `CLAUDE.md` documente** :

| affirmation | vérification |
|---|---|
| « le lint échoue sur une exception morte » | `KNOWN_VIOLATIONS` n'est lu que par `.has()` (`:189`) — aucune détection d'entrée morte. Le vrai défaut est **symétrique** de celui annoncé |
| « `fr-CH` compte 1271 clés » | **1273**, par `grep -cE` et par parseur |
| « 846 clés statiques » | **1002** |

**MEDIUM retenus** : les titres de T2 et T3 ne nommaient pas les AC-bis/-ter (traçabilité illisible
au titre seul) ; l'assertion sur les motifs comptait les sites au lieu de comparer leur **liste**
(un ajout compensant un retrait passait inaperçu) ; D8 attestait trois termes sans donner les
**20 clés du pilote**, désormais nommées ; D8-bis est requalifiée en **constat de cadrage** — elle
justifie le découpage de l'epic, elle n'exige rien de cette story.

### Passe 3 de `bmad-create-story validate` — 2026-08-19, Opus ×3, contextes frais

**Trend : 0 CRITICAL · 2 HIGH · 10 MEDIUM (dédoublonnés) · 6 LOW.** BlindHunter 0/2/4/2,
EdgeCaseHunter 0/1/6/2, AcceptanceAuditor 0/0/6/6. **Les deux HIGH ont convergé sur les trois
lentilles.**

**HIGH-1 — sept clés à repli divergent, pas six.** `payment-batches-col-date` manquait
(« Exécution » sur un en-tête de colonne, « Date d'exécution » sur la fiche). **Et c'est la
troisième récidive de la même cause** : c'est le seul conflit dont les deux replis sont entre
**guillemets doubles**, parce qu'ils contiennent une apostrophe — une classe de caractères négative
s'y arrête. Le défaut attrapé en passe 1 sur l'extraction de la **clé** s'était simplement déplacé
sur l'extraction du **repli**. AC7-bis vaut désormais pour les deux arguments d'`i18nMsg`.
⚠️ *La vérification que j'ai écrite pour contrôler ce finding reproduisait elle-même le défaut et
rendait 6 : il a fallu un vrai lecteur de littéral pour obtenir 7. Quatrième occurrence, dans
l'outil de contrôle cette fois.*

**HIGH-2 — `bank-import-info-*` est un second angle mort, et pire que le premier.** Ses valeurs
sont poussées par le **backend Rust** (`bank_imports.rs:693` et `:1668`) ; le frontend n'a qu'un
`informational: string[]`. **Il n'existe donc aucune carte de production à confronter**, et
l'assertion de cardinalité que D4 promettait y est verte à jamais. Un troisième code informationnel
s'afficherait en `snake_case` brut dans les quatre langues sans qu'aucune garde ne bouge — le test
muet, sur le préfixe même dont l'oubli était le CRITICAL de la passe 1. AC7-ter couvre désormais
**les deux** ensembles ouverts, et les valeurs se déclarent **après transformation**.

**MEDIUM retenus, par famille.**
*Mécanisme* : le fichier supprimé portait **deux** contrôles et AC12 n'en reprenait qu'un — celui
des **orphelines** disparaissait, alors que l'AC promettait sa conservation (D2 amendée, AC12-ter) ;
AC7 exigeait une « liste attendue de 10 sites » que la spec ne donnait nulle part (table de
référence ajoutée en D4) ; D4 prescrivait encore le simple comptage qu'AC7 déclarait insuffisant —
contradiction créée par le patch de passe 2 ; la clause « valeurs en dur, jamais lues depuis la
production » n'était exigible par aucun AC ; l'allowlist de parité était indexée par clé et non par
locale, laissant passer une traduction partielle ; et « demandées » n'était pas défini comme
l'union des littéraux **et** des expansions, si bien qu'AC5 aurait chassé les 10 clés qu'AC8 impose.
*Comptes rendus* : la partie B du glossaire compte **16** entrées et non 15 — `personne de contact`
y ayant été ajouté par le commit de création de cette spec, l'instruction « à ajouter » aurait
produit un doublon ; le « douze autres » devient treize ; l'attestation disait « six »
`contact-persons-*` pour **cinq** ; et `it-CH` compte **11 messages** au registre de courtoisie, pas
un — chiffre que la passe 1 avait réfuté dans la prose du glossaire **sans toucher la ligne de
tableau six lignes plus haut**, ni la décision D9 qui la cite.

**LOW retenus** : la commande de recompte employait `[a-z0-9-]` et rendait **1271** au lieu de 1273,
ratant les deux seules clés à tiret bas — *jumeau shell de la classe négative qu'AC7-bis interdit* ;
trois citations imprécises (`:84-89` → `:85-88`, l'attribution des 8 replis interpolés, « 6 appels »
pour 4) ; et « 197 sous `routes/` » est vrai **exclusivement** (199 au moins partiellement).

⚠️ **Ce que ce trend dit, et qui appelle un arbitrage.** La sévérité maximale fait
`CRITICAL → HIGH → HIGH` : le plafond **stagne**, ce que la § *Règle de splitting préventif* traite
comme un signal de non-convergence (`N+1 ≥ N`). Le volume au-dessus de LOW ne décroît pas non plus
(11 → 10 → 12). **Et le motif est stable sur les trois passes : la majorité des findings portent sur
ce que la passe précédente vient d'écrire**, pas sur la spec d'origine.

### Split — 2026-08-19, arbitrage de Guy

Trois passes, trois modèles, trend `2C/1H/8M/3L → 0C/4H/6M/2L → 0C/2H/10M/6L`. **Le plafond de
sévérité stagne à HIGH** et le volume au-dessus de LOW ne décroît pas — le critère `N+1 ≥ N` de la
§ *Règle de splitting préventif* est atteint.

**Le diagnostic n'est pas « la story est trop large » au sens des modules touchés** — elle en touche
quatre. C'est que **les findings se répartissent en deux familles qui ne se relisent pas avec la
même lentille** : le *mécanisme* (extracteur, motifs, ensembles ouverts, allowlists, orphelines) et
les *comptes rendus* (glossaire, attestations, décomptes, commandes de recompte). Sur la passe 3,
presque tous les MEDIUM de la seconde famille étaient des résidus laissés par les patches des
passes 1 et 2.

C'est exactement le motif du split 22-2a / 22-2b, dont la rétrospective de l'Epic 22 dit qu'il a
marché pour cette raison. **Cette moitié-ci ne contient aucun libellé, aucune locale, aucun terme de
glossaire** : ce qui s'y relit est du code et des ensembles.

### Passe 1 de `validate` sur le SPLIT — 2026-08-19, Sonnet ×6, contextes frais

**Trois lentilles par moitié, braquées sur l'intégrité du découpage.**
23-1a : 0 C · **2 H** · 2 M · 4 L. 23-1b : 0 C · **2 H** · 4 M · 2 L.

⚠️ **UN FINDING CHANGE TOUS LES CHIFFRES DE L'EPIC — et il invalide trois passes de recomptes.**
Sept fichiers déclarent un **relais** : `function msg(key, fallback) { return i18nMsg(key, fallback) }`.
Le littéral s'y trouve au site `msg(`, **jamais** au site `i18nMsg(`. Toute extraction cherchant
`i18nMsg(` — la mienne, celle des trois passes précédentes, et **la garde telle qu'elle était
spécifiée** — les ignore intégralement.

Recompté depuis la source : **+29 clés manquantes** (250 → **279**), **+1 dossier entier**
(`routes/onboarding`, qui n'apparaissait dans aucun découpage), et `routes/(app)/settings` qui
passe de 30 à **55** — l'écran des modèles d'e-mail à lui seul. **Total de l'epic : 317 → 346.**
D'où la décision **D4-bis** et le critère **AC7-quater**, avec assertion de cardinalité sur les
7 relais. *C'est le troisième angle mort de la garde, après `vat-category-*` et
`bank-import-info-*` — et le seul **refermable**, les deux autres tenant à des valeurs produites
hors du frontend.*

**Le second HIGH de chaque moitié porte sur le découpage lui-même :**

- **23-1b** — la **substance** de `D5-bis` (exclusion des `.test.*`) n'avait pas traversé la
  frontière : l'identifiant était cité, la règle n'était imposée nulle part au moissonneur, qui
  est un **script distinct** et non une modification de la garde. Il aurait moissonné
  `i18n.svelte.test.ts`, entré `une-cle = mon repli` au catalogue et annoncé **8** replis
  divergents au lieu de 7.
- **23-1a** — **six renvois orphelins** pointaient vers `D7-bis`, `D8-bis`, `D9` et `T5`, qui
  vivent chez la sœur ; l'un d'eux **dans le corps normatif d'AC6**, où un relecteur ne pouvait pas
  savoir que `D8-bis` est un *constat* et non une exigence.

**Et une affirmation de la 23-1b était fausse** : `D8-bis` déclarait « 2 clés partagées entre
dossiers, toutes deux dans le pilote ». Il y en a **8**, partagées avec `routes/onboarding` — un
troisième dossier hors périmètre, qui affichera les libellés choisis pour le carnet d'adresses dès
que la 23-1b sera mergée. T5 doit désormais relire ces huit libellés **dans les deux contextes**.

**MEDIUM et LOW retenus** : la clause « Dehors, explicitement » excluant `kesh-qrbill` et sa table
`I18N_KEYS` — **appariée par position** — ainsi que les manuels LaTeX avait été perdue par **les
deux** moitiés, restaurée ici ; deux bullets de Dev Notes contredisaient le bandeau d'ouverture de
la 23-1a (« ne pas traduire au-delà des 20 clés » alors que sa borne est **zéro**) ; l'exception
sur `KNOWN_VIOLATIONS` réservait à la 23-1a un travail qui appartient à T5 ; `D10` n'était exigible
par aucun critère sans se déclarer constat ; les chiffres du moissonneur étaient recopiés dans le
tableau de la 23-1a sans y servir aucun critère ; T5 pointait vers une chaîne « douze autres »
**corrigée en « treize » à la passe 3** ; et la borne `.ts >= 3` était défaite par le cas même
qu'elle vise — deux des cinq clés `.ts` viennent d'appels **multi-lignes en TypeScript pur**
(`notify.ts:103-110`), forme qu'aucune fixture ne testait : leur perte ramenait le compte à 3,
toujours vert. Borne portée à **5**, troisième fixture ajoutée.

**Un chiffre de MA consigne réfuté par les lentilles** : je leur annonçais « 17 critères », il y en
a **18** — recomptés depuis la source par deux méthodes convergentes avant de me contredire. Le
chiffre faux ne vivait que dans mes prompts, aucun artefact versionné n'était touché.

### Passe 2 de `validate` sur le split — 2026-08-19, Haiku ×6, contextes frais

**23-1a : 0 C · 2 H · 4 M · 3 L. 23-1b : 0 C · 2 H · 6 M · 2 L.** Aucun CRITICAL, et **tous les
chiffres neufs du recompte des relais ont été confirmés au sol** par plusieurs lentilles
indépendantes — 7 relais, 1151 littéraux, 279 manquantes, 14 dossiers, 346 au total.

**Les deux HIGH de la 23-1a ont convergé, et c'est encore un défaut de propagation** : les bornes
révisées en passe 1 du split (`>= 1050` et `.ts >= 5`) n'avaient été portées **que dans le tableau
de D5**. `AC9` disait toujours `≥ 3`, la sous-tâche de `T2` disait `>= 900` et `>= 3`. **Un
développeur lit le critère et la case à cocher, pas la décision** — il aurait donc posé exactement
la borne dont la passe précédente venait de démontrer qu'elle est *verte sur la perte qu'elle doit
attraper*. Quatrième récidive du même geste dans ce dossier.

**Les deux HIGH de la 23-1b sont le même diagnostic, vu d'un autre angle** : `D6` prescrivait
l'exclusion des `.test.*` et le lecteur de littéral, `AC10` ne les exigeait pas ; `D8-bis` se
déclarait « constat, non exigence » tout en imposant une relecture à `T5` ; et trois sous-tâches
neuves de `T5` n'étaient rattachées à aucun critère. **Mes corrections des passes précédentes ont
enrichi les décisions et les tâches, jamais les critères d'acceptation.** D'où `AC11-ter`,
`AC11-quater`, `AC11-quinquies`, et les deux clauses ajoutées à `AC10`.

**MEDIUM et LOW retenus** : `sprint-status.yaml` portait encore « 317 clés » à trois endroits hors
récit historique ; deux résidus de l'ancien compte (« 245 sur 250 », « 250 approximations ») dans
la 23-1b ; `D8` s'intitulait « INPUT figé » à propos d'un fichier que le commit de spécification de
cette story avait modifié ; le Change Log de la 23-1a citait `AC11-bis` sans dire qu'il vit chez la
sœur ; les premières mentions de `D5-bis` et `AC7-bis` n'étaient pas accompagnées d'un « cf.
23-1a » ; `AC12-ter` parlait de préfixes « à couverture close, pour commencer » sans nommer la
liste ni dire comment elle s'étend — elle s'appelle désormais `PREFIXES_A_COUVERTURE_CLOSE` ; et la
sous-tâche sur le commentaire du lint ne disait pas s'il fallait **retirer** ou **remplacer** la
mention de `delete` (c'est un retrait).

⚠️ **Trois findings d'une lentille Haiku réfutés au sol, et ils tenaient à DEUX lectures** :

| affirmation | vérification |
|---|---|
| « AC8 annonce 10 clés, il y en a 11 » puis, par ricochet, « le total est 347, pas 346 » et « T2 devrait dire 279 + 11 » | `imported-supplier-invoices-error-unknown` est un **littéral statique**, donc **déjà compté** dans les 279. Le « +10 » désigne les dix valeurs de la carte, que seule l'énumération révèle. L'ajouter serait un **double compte** |
| « D1 affirme faussement qu'il y a 0 clés manquantes, il y en a 57 » | D1 parle des clés présentes **seulement en `de-CH`** — le sens inverse, mesuré à **0** (`comm -13`). Les 57 sont l'autre sens, et D1 ne les nie nulle part |

*C'est le mode d'échec Haiku que le `CLAUDE.md` documente : une lecture rapide d'un énoncé, propagée
en cascade sur plusieurs findings. Le garde-fou ground-truth l'a écarté en deux commandes.*

### Passe 3 de `validate` sur le split — 2026-08-19, Opus ×6, contextes frais

**23-1a : 1 CRITICAL · 1 HIGH · 6 MEDIUM · 5 LOW. 23-1b : 0 C · 3 H · 6 M · 6 L.** Les deux moitiés
ont convergé sur le même défaut de fond, et il commande tout le reste.

#### Le CRITICAL — la garde ne voyait aucune clé passée par INDIRECTION

Six clés absentes des quatre catalogues échappaient à la garde **et** à tous les décomptes, dont
**quatre entrées de la barre de navigation principale** (`nav-credit-notes`, `nav-email-templates`,
`nav-projects`, `nav-supplier-invoices-import`) et deux onglets de rapport. Elles vivent dans des
**tables de données** (`{ i18nKey: 'nav-credit-notes', … }`) et n'atteignent `i18nMsg` que par
`item.i18nKey`.

⚠️ **Et le critère censé fermer la faille la certifiait** : `AC7` faisait comparer la liste des sites
à la table de `D4` — **les deux côtés sortant du même extracteur**. L'assertion était verte par
construction.

#### Ce que six passes ont établi, et la décision qui en sort

Cette spec a énuméré **cinq fois** une forme d'appel — littéral, gabarit, relais, multi-ligne,
table — et **cinq fois la passe suivante en a trouvé une sixième** : ternaire dans le premier
argument, gabarit affecté à une variable, clé fabriquée par une fonction, clé lue dans une colonne
libre. *Une énumération de formes est ouverte par nature.*

**D4-ter inverse la méthode** : on n'énumère plus les formes qui marchent, on **inventorie les sites
qui ne résolvent pas** — **54 sur 1514** à l'estimation, **36 sur 1496** une fois le lecteur réel écrit (cf. § D4-ter), ensemble clos, comptable, décidable. Chaque site y est soit
résolu (clés déclarées en dur), soit écrit comme angle mort. **C'est la seule assertion du document
qu'une forme imprévue ne puisse pas contourner.** L'inventaire, à sa première exécution, a résolu
sept familles et rendu les six clés manquantes : **279 → 285 clés statiques, 346 → 352 pour l'epic.**

#### Les trois HIGH de la 23-1b — le même motif, cinquième récidive

- **Les 20 libellés `fr-CH` — l'objet déclaré de la story — n'étaient exigés par aucun critère** :
  `AC11` ne parlait que de `de-CH`/`it-CH`/`en-CH`. Un dev pouvait entrer « à titre informatif » et
  « N° » tels quels et rendre la story verte, fixant le patron que 265 clés recopieraient. D'où
  **AC11-sexies**.
- **`AC10` n'exigeait pas la clause « relais » de `D6`** — et ses deux valeurs de contrôle (5 et 7)
  sont **identiques avec et sans relais** : elles ne discriminaient pas. Un moissonneur aveugle rend
  245 entrées au lieu de **274** en cochant l'AC. La valeur discriminante est désormais écrite.
- **`AC10` exigeait « le même lecteur de littéral » que la garde**, alors que la garde vit dans un
  `.test.ts` qu'un script `node` ne peut pas importer, et que les Dev Notes interdisaient d'y
  toucher. D'où **D1-bis** : un module partagé, `i18n-literal-reader.js`.

#### Deux chemins verts vers un livrable défaillant, refermés

- **L'ordre de `T5` n'était pas normatif** : moissonner avant de renommer fait entrer
  `delete = Supprimer` au catalogue **avec tous les gates verts** — parité satisfaite, garde B
  muette (plus rien ne demande la clé), allowlist retirée par la story, orphelines bornées ailleurs,
  lint aveugle aux entrées mortes. L'ordre est désormais imposé, avec une clause négative qui se
  vérifie d'un grep.
- **Rien n'exécutait le moissonneur** : `vite.config.ts:59` borne vitest à `src/**/*.test.ts`. Toute
  la substance d'`AC10` n'avait pour preuve que deux nombres recopiés à la main. Il porte désormais
  son propre test, à trois fixtures reprenant les trois défauts déjà payés.
- **`AC11-ter` était une case à cocher**, et `D8-bis` **annonçait son verdict avant l'exercice**
  (« en l'espèce ils conviennent »). La consignation devient un tableau de huit lignes dont chaque
  repli se retrouve au `grep -nF`.

#### Corrections de comptes rendus

`245` était la sortie d'un moissonneur aveugle aux relais, ré-ancrée sur un dénominateur corrigé —
la valeur juste est **274** ; le préambule du glossaire annonçait encore 317 clés et 951 messages ;
`sprint-status.yaml` affirmait un total juste (346) à partir d'opérandes fausses (250 + 10 + 57 =
317) ; la ligne de la story 23-4 portait 60 clés au lieu de **89** ; et la forme canonique de relais
de `D4-bis` excluait l'un des sept (celui à trois paramètres).

**Le diagnostic de méthode, formulé par une lentille et retenu tel quel** : *le geste correctif n'est
pas « relire les décisions », c'est **remonter du critère vers la décision** — pour chaque ⚠️ et
chaque case à cocher, demander quel critère la contrôle.* J'avais corrigé les cinq récidives
précédentes dans le mauvais sens.

## Dev Agent Record

### Agent Model Used

Opus 5 (1M context) — implémentation du 2026-08-19.

### Debug Log References

**Le lecteur de littéral a été confronté au dépôt AVANT d'écrire la moindre garde**, et il a
corrigé la spec : celle-ci annonçait **54 sites non résolus sur 1514**, valeurs issues d'une mesure
par expression régulière qui comptait aussi les **définitions** de relais et des mentions en
commentaire. Le lecteur réel rend **36 sur 1496**. *La spec a été alignée sur l'outil, pas
l'inverse.*

**Les deux gardes ont été MUTÉES pour prouver qu'elles mordent**, et non seulement exécutées :

| mutation | attendu | obtenu |
|---|---|---|
| garde A — retirer `invoice-error-already-validated` de la dette de parité | rouge | rouge, nommant la clé **et** les trois locales |
| garde A — inscrire `error-invalid-credentials`, déjà traduite | rouge | rouge, « désormais traduite — retirer de la dette » |
| garde B — retirer `nav-credit-notes` de la dette | rouge | rouge |
| garde B — inscrire `error-invalid-credentials`, déjà au catalogue | rouge | rouge |
| garde B — retirer le préfixe `due-dates-filter-` de `MOTIFS_DYNAMIQUES` | rouge | rouge |
| garde B — **créer un site d'indirection neuf** (`i18nMsg(k, 'repli')`) | rouge | rouge, **2 tests sur 7** (« collecte » et « inventaire ») |

⚠️ **La dernière mutation est celle qui compte** : elle simule la sixième forme d'appel qu'aucune
énumération n'avait prévue. L'assertion d'inventaire l'attrape sans qu'on lui ait rien appris — c'est
la garantie de `D4-ter`, vérifiée plutôt que postulée.

**Un piège prévu par la revue s'est produit tel quel** : à la première exécution, la garde B a
rougi sur les **6 clés de l'inventaire** (`nav-*`, `reports-project-*`), inscrites à la dette mais
introuvables parmi les clés demandées. C'est exactement le retournement que la passe 3 annonçait —
*« le contrôle symétrique pousserait la dette hors de l'allowlist »*. Le correctif n'a pas été de
retirer les lignes, mais de déclarer `FAMILLES_RESOLUES` : l'étape 4 de `D4-ter`.

### Completion Notes List

- **Garde A** (`loader.rs`, `parity_between_locales`) — compare les **ensembles de clés** des quatre
  catalogues via le champ privé `keys`, sans jamais appeler `format()` ni `all_messages()`, qui
  replient sur `fr-CH` et ne peuvent pas distinguer « traduit » de « replié ». Échoue dans les deux
  sens, avec borne anti-test-muet à 1200 clés par locale.
- **Garde B** (`i18n-keys.test.ts`, 7 tests) — existence des clés demandées, allowlist bidirectionnelle,
  8 préfixes dynamiques avec cardinalité, **liste** des 10 sites de gabarit confrontée à une table de
  référence, **assertion de cardinalité sur les 7 relais**, inventaire des **33** sites non résolus, orphelines bornées à `PREFIXES_A_COUVERTURE_CLOSE`.
- **Lecteur partagé** (`i18n-literal-reader.js`) — automate caractère par caractère, importable par
  vitest **et** par `node` : c'est ce que la 23-1b consommera pour son moissonneur (D1-bis). Son test
  (12 cas) inscrit les **trois formes qui ont réellement cassé** pendant les passes de revue.
- **Allowlists** — 295 entrées côté frontend (279 littéraux + 6 de l'inventaire + 10 expansions),
  57 côté Rust. Générées **depuis la source**, jamais saisies.
- `duplicate-i18n-keys.test.ts` supprimé, **ses deux contrôles repris** ; `contacts-i18n-realpath.test.ts`
  conservé et vert.

### File List

| fichier | |
|---|---|
| `frontend/src/lib/shared/i18n-literal-reader.js` | **neuf** — lecteur partagé (D1-bis) |
| `frontend/src/lib/shared/i18n-literal-reader.test.ts` | **neuf** — 12 preuves, dont les 3 formes qui ont cassé |
| `frontend/src/lib/shared/i18n-keys.test.ts` | **neuf** — garde B, 7 tests |
| `frontend/src/lib/shared/i18n-dette-connue.ts` | **neuf** — 295 entrées |
| `frontend/src/lib/features/contacts/duplicate-i18n-keys.test.ts` | **supprimé** — remplacé par la garde B |
| `crates/kesh-i18n/src/loader.rs` | garde A ajoutée au module `tests` |
| `crates/kesh-i18n/dette-parite-connue.txt` | **neuf** — 57 clés de l'issue #283 |
| `_bmad-output/implementation-artifacts/23-1a-mecanisme-gardes-i18n.md` | chiffres alignés sur l'outil, tâches, Dev Agent Record |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | statut |

### Passe 1 de `bmad-code-review` — 2026-08-19, Sonnet ×3, contextes frais

**0 CRITICAL · 4 HIGH · 5 MEDIUM · 5 LOW.** Les trois lentilles ont trouvé des défauts réels dans
le livré, et **aucun n'était visible depuis les tests, qui passaient tous**.

**Les deux HIGH d'acceptation me visent directement.**

- **`AC7-quater` n'était pas satisfait** : aucune assertion de cardinalité sur les 7 relais. ⚠️ **Et
  j'avais coché la case de la tâche correspondante.** La collecte des relais était bien faite —
  `findCallSites` appelle `findRelays` — mais le garde-fou, lui, n'existait pas. Une case à moitié
  vraie est pire qu'une case fausse : elle survit à la relecture. *C'est exactement ce que la
  § « ne déclarer que ce qui a tourné » interdit.* L'assertion existe désormais.
- **`AC7-ter` n'était satisfait qu'aux deux tiers** : le **troisième** ensemble ouvert —
  `vat-rates/+page.svelte:52`, où la clé vient de la colonne libre `vat_rates.label` — n'était nommé
  nulle part. La tâche T3 elle-même n'en citait que deux : l'implémentation a suivi la case, pas le
  critère.

**Les deux HIGH techniques portent sur le lecteur, et le premier est un défaut de correction.**

- **`kind` était déduit d'une relecture de la valeur** (`value.includes('${')`) au lieu du parsing.
  Conséquence démontrée : `'a${b}c'` entre guillemets **simples** — qui ne peut pas être un gabarit
  en JavaScript — était classé dynamique, tout comme un `\${` **échappé**, volontairement inerte.
  L'automate connaissait pourtant la vérité : il maintient un compteur d'interpolation qu'il
  n'utilisait pas.
- **Les commentaires étaient comptés comme sites d'appel** : trois docstrings du dépôt écrivent
  `i18nMsg(clé, repli)` en exemple. Trois des trente-six « sites non résolus » étaient de la prose.
  ⚠️ **La conséquence grave n'est pas le bruit, c'est la compensation silencieuse** : un commentaire
  qui disparaît pendant qu'un vrai site apparaît laisse le compte inchangé. C'est le mode d'échec
  « un ajout compensant un retrait » que j'avais nommé pour refuser le comptage des sites de
  gabarit, et reproduit sur l'inventaire.

**Le correctif du masquage a lui-même eu un défaut, trouvé en le vérifiant** : un fichier `.svelte`
n'est pas du JavaScript. Son balisage contient de la prose, et le français y met des apostrophes —
`l'exercice`, `d'abord`. Traitées comme ouvertures de chaîne, elles avalaient des dizaines de lignes
et **un commentaire sur trois restait démasqué**. Le critère retenu est net : une chaîne JavaScript
n'est jamais précédée d'une lettre.

**MEDIUM retenus** : une regex à accolade non appariée ou à quote (`/['"]/g`, idiome courant)
désynchronisait la lecture d'un gabarit — le commentaire promettait cette protection, le code ne la
décrivait pas ; `(?<![\w.])` rendait un appel membre `obj.i18nMsg(…)` **totalement invisible**, pas
même « non résolu » ; `findRelays` n'acceptait qu'une forme unique, si bien qu'un relais en fonction
fléchée, ou précédé d'une garde, emportait tous ses sites dans le silence ; et le Dev Agent Record
annonçait « +19 » là où le net est **+17** (le prototype supprimé en retirait 2).

**Effet sur les chiffres** : après masquage des commentaires et prise en compte des appels membres,
l'inventaire passe de **36 sur 1496** à **33 sur 1493**. Les valeurs de la spec ont suivi.

**Une observation hors périmètre a été réfutée** : une lentille signalait le retour du fichier
supprimé dans l'arbre de travail. `git status` est propre et le fichier n'existe pas.

### Gates — exécutés, non déclarés

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | vert |
| `cargo clippy --workspace --all-targets -D warnings` | vert, 0 avertissement |
| backend complet (`test-fast.sh`, base **remise à zéro d'abord**) | **2219/2219**, 4 ignorés, 138 s — *+1 test : la garde A* |
| `npm run check` | **0 erreur** (27 avertissements préexistants) |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | **633/633** sur 69 fichiers — *+19 tests neufs à l'implémentation, +7 en revue, **−2** par la suppression du prototype : **net +24** depuis `98026213` (609)* |
| `npm run build` | vert |

⚠️ **E2E non exécutée** : cette story ne touche aucune surface utilisateur — ni `.svelte` de
production, ni route, ni API. Les quatre fichiers livrés sont deux tests, un module de lecture et
une liste de données.
