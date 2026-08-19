# Story 23.1 : Le socle — deux gardes, parce qu'il y a deux silences

## Status

ready-for-dev

Story-zéro de l'**Epic 23 « Dette i18n »** (plan : `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`).
Elle ne résorbe presque rien — **20 clés sur 317**. Elle rend le reste *mesurable*, et surtout
elle rend impossible d'en ajouter d'autres sans le savoir.

## Story

**As a** personne qui utilise Kesh en allemand, en italien ou en anglais,
**I want** que le logiciel cesse de me servir du français en se taisant,
**so that** un oubli de traduction devienne un test rouge et non un état permanent.

Et, du côté de qui développe : **I want** qu'une clé écrite dans un `.svelte` et absente du
catalogue fasse échouer `npm run test:unit`, **so that** la dette cesse de se creuser pendant
qu'on la résorbe.

Adresse [#316] et [#283] — dont elle ne ferme aucune : elle les **borne**.

## Contexte — le défaut a deux chemins, pas un

C'est le point que toute rédaction rapide de cette story rate, et il commande tout le reste.

`I18nBundle::all_messages(locale)` (`crates/kesh-i18n/src/loader.rs:130-143`) **charge d'abord
toutes les clés `fr-CH` comme base**, puis écrase avec celles de la locale demandée. Le
frontend reçoit donc, par `GET /api/v1/i18n/messages`, un catalogue **déjà replié**. D'où deux
chemins distincts pour un même symptôme visible :

| | où la clé manque | ce que reçoit le frontend | quel repli s'affiche |
|---|---|---|---|
| **[#283]** — 57 clés | en `de-CH` / `it-CH` / `en-CH`, présente en `fr-CH` | **le texte français**, via le repli du loader | celui du **backend** — le 2ᵉ argument d'`i18nMsg` n'est jamais atteint |
| **[#316]** — 250 clés | **partout** | rien | le **littéral en dur** du `.svelte` |

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

## Périmètre

### Dedans

1. **Garde A — parité inter-locales**, dans `kesh-i18n` (Rust).
2. **Garde B — existence des clés demandées**, en vitest, portée `frontend/src` **entier**.
3. **Les 8 préfixes dynamiques** (10 sites d'appel), traités par énumération déclarée.
4. **Le moissonneur de replis**, versionné et exécutable.
5. **Le domaine pilote `contacts`** — 20 clés entrées aux quatre locales.

### Dehors, explicitement

- Les **297 autres clés** — elles sont le travail des stories 23-2 à 23-6.
- **[#255]** (chaînes en dur sans appel à `i18nMsg`). ⚠️ **C'est l'angle mort assumé des deux
  gardes** : ce qui n'est jamais demandé ne peut pas être trouvé manquant. La page `/invoices`
  en est le cas — **4 appels** `i18nMsg` pour toute la page (6 mentions du symbole, import compris).
- **[#314]**, le sélecteur de langue **[#242]**, et toute traduction des manuels LaTeX.
- **Le backend `kesh-qrbill`** et sa table `I18N_KEYS` / `DEFAULT_EN` (`types.rs:216`) : c'est un
  troisième catalogue, apparié par **position**, hors sujet ici.

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
corrigé en conséquence. ⚠️ *C'est exactement ce que le point 3 existe pour empêcher : un motif
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

**D5 — Les deux gardes se bornent contre le passage à vide.**
Chacune assert un **minimum de clés effectivement collectées**. *Le principe vient du prototype
22-2b, dont le `expect(demandees.size).toBeGreaterThanOrEqual(5)` bornait la collecte du seul
domaine `contact-duplicate-*` ; la garde générale le reprend à son échelle.*

| borne | valeur | mesuré |
|---|---|---|
| garde A — clés par locale | `>= 1200` | 1216 |
| garde B — littéraux demandés | `>= 900` | **1002 statiques** |
| garde B — clés issues de fichiers `.ts` | `>= 3` | 5 |

⚠️ **Les deux nombres de la spec ne se contredisent pas, et il faut le dire une fois pour
toutes** : `i18nMsg()` est appelé avec **1010 littéraux distincts**, dont **8 sont des gabarits
dynamiques** ; il reste donc **1002 clés statiques**, et c'est cette valeur que borne la garde B.
*(Écart relevé en passe 2 — les deux chiffres étaient justes, leur relation n'était écrite nulle
part.)*

⚠️ **Une borne globale ne voit pas une perte PARTIELLE, et c'est mesuré** : **997** de ces 1002
clés viennent de `.svelte` et **5 seulement** de `.ts`. Un filtre d'extension réduit par erreur à
`/\.svelte$/` ferait tomber le total à 997 — **toujours ≥ 900**, donc toujours vert. D'où la
troisième ligne du tableau.
⚠️ **Cette borne `.ts` compte les fichiers de PRODUCTION**, les `.test.*` étant hors collecte
(cf. D5-bis) — sans quoi la garde, elle-même écrite dans un `.ts`, pourrait se compter et
survivre à la perte de tout ce qu'elle doit surveiller.

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

**D6 — Le moissonneur PROPOSE, il n'écrit jamais dans les catalogues.**
`frontend/scripts/harvest-i18n-fallbacks.mjs` — il lit `src`, extrait les couples
(clé, repli littéral), et rend un fragment `.ftl` **trié, sur la sortie standard**. Il ne touche
à aucun `messages.ftl`.
⚠️ **Il ne traite QUE les clés absentes des quatre catalogues**, et cette restriction doit être
écrite dans le script : sans elle, il moissonne les ~1000 clés demandées et sa sortie d'erreur
annonce **13** replis interpolés au lieu des **5** que cette spec nomme — les 8 autres appartenant
à des clés **déjà traduites** : `ManualMatchModal` 1, **`TransactionSplitModal` 2**
(`reconciliation-split-error-imbalance` et `-balance-indicator` — ce composant en porte donc 7 en
tout, dont 5 seulement sont manquantes), `invoices/[id]` 1, `reports` 1, `fiscal-years` 3.
⚠️ **Il DÉTECTE les conflits de repli plutôt que d'en choisir un.** **Sept** clés manquantes sont
demandées avec **deux textes de repli différents** selon le site :

| clé | variantes |
|---|---|
| `credit-notes-title` | « Avoirs » / « Avoir » |
| `payment-batches-col-total` | « Total » / « Montant » |
| **`payment-batches-col-date`** | **« Exécution » / « Date d'exécution »** |
| `supplier-invoices-col-total` | « TTC » / « Total HT » |
| `supplier-invoices-field-reference` | « Référence » / « Référence (optionnel) » |
| `supplier-invoices-field-project` | « Projet analytique » / « Projet analytique (optionnel) » |
| `imported-supplier-invoices-reload-failed` | deux phrases |

Un moissonneur qui garde le dernier vu fige silencieusement le mauvais libellé ; ces cas vont sur
la sortie d'erreur, avec leurs variantes, pour arbitrage humain dans la story de rollout concernée.

⚠️ **La septième — `payment-batches-col-date` — a manqué à la première rédaction, et pour la
TROISIÈME fois par la même cause.** C'est le seul conflit dont les **deux replis sont entre
guillemets doubles**, parce qu'ils contiennent une apostrophe (`"Date d'exécution"`) : une classe
de caractères négative s'y arrête. Le défaut attrapé en passe 1 sur l'extraction de la **clé** se
rejoue ici sur l'extraction du **repli**. **AC7-bis vaut donc pour les DEUX arguments d'`i18nMsg`**,
et le moissonneur emploie le même lecteur de littéral que la garde — pas une expression régulière.

⚠️ **Le nombre « sept » est une valeur de contrôle datée du 2026-08-19, pas une cible.** Le
moissonneur **calcule** la liste et publie son décompte ; si les deux divergent, c'est la spec
qu'on recompte, jamais la sortie qu'on ajuste.
⚠️ **Motif** : un repli est écrit dans le feu de l'action, souvent sans majuscule, sans point
final, parfois avec la formulation d'un développeur pressé. **Le laisser devenir un libellé de
catalogue sans relecture, c'est faire entrer 250 approximations dans le produit.** Chaque story
de rollout relit ce qu'elle fait entrer.
⚠️ **Cinq clés n'ont PAS de repli littéral** — les cinq de `TransactionSplitModal.svelte`, dont
le repli est interpolé (`` `Ligne ${i + 1} : compte requis` ``). Le moissonneur les **liste à
part**, sur la sortie d'erreur, comme « à écrire à la main, entrée Fluent à variable ». Elles
sont traitées en **23-5**, pas ici.

**D7 — Le pilote est `contacts`, et il est choisi pour une raison.**
20 clés (12 sous `lib/features/contacts`, 8 sous `routes/(app)/contacts`). C'est le domaine que
la 22-2b vient de travailler, dont la garde bornée existe déjà, et dont le vocabulaire est
**presque** entièrement couvert par la partie A du glossaire — **trois termes exceptés, que cette
story tranche : cf. D8**.
⚠️ *La rédaction initiale de D7 concluait « donc aucune décision terminologique n'est prise dans
la story-zéro ». C'était faux, D8 le disait déjà six lignes plus bas, et les deux phrases ont
cohabité un temps dans ce document — une correction appliquée au site signalé et pas à sa source.
Relevé en passe 1 de `validate`.*

**Les 20 clés du pilote, nommées** — pour que l'attestation des trois termes de D8 soit
vérifiable sans les rechercher :

| dossier | clés |
|---|---|
| `lib/features/contacts` (12) | `contact-persons-add`, `contact-persons-add-error`, `contact-persons-delete-error`, `contact-persons-empty`, `contact-persons-hint`, `contact-persons-load-error`, `contact-persons-name-required`, `contact-persons-role`, `contact-persons-title`, **`delete`** (cf. D7-bis), `field-first-name`, `field-last-name` |
| `routes/(app)/contacts` (8) | `contact-error-address-npa-city`, `contact-error-person-name`, `field-address`, `field-building`, `field-city`, `field-country`, `field-postal-code`, `field-street` |

**Attestation des trois termes de la partie B** : `localité` → `field-city`,
`contact-error-address-npa-city` ; `prénom` → `field-first-name`, `contact-persons-name-required`,
`contact-error-person-name` ; `personne de contact` → les **cinq** `contact-persons-*` dont le libellé porte le mot
(`-load-error`, `-add-error`, `-delete-error`, `-title`, `-empty`) plus `contact-error-person-name`,
soit **6 clés** — le compte du glossaire. L'union des trois groupes fait **10 clés distinctes sur
20**, `contact-error-person-name` appartenant à deux d'entre eux. *(« Six » avait été écrit sous la
mention « recompté » en passe 2 : le total restait juste, la ventilation non.)*

**D7-bis — `delete` est renommée `contact-persons-delete` avant d'entrer au catalogue.**
`ContactPersonsManager.svelte:118` demande `i18nMsg('delete', 'Supprimer')` — une clé **sans
domaine**, aujourd'hui absente des quatre catalogues et donc inoffensive.
⚠️ **La faire entrer telle quelle est irréversible en pratique** : une fois `delete` au catalogue,
n'importe quelle feature pourra l'appeler, la garde B ne signalera jamais rien, et un libellé
traduit dans le contexte « supprimer une personne de contact » sera silencieusement resservi
ailleurs — y compris là où l'allemand ou l'italien demanderaient un autre mot. Les neuf autres
clés du même composant portent déjà le préfixe `contact-persons-`.
*La clé n'est demandée que depuis ce seul site (vérifié), le renommage tient en une ligne.*

**D8 — Le glossaire est un INPUT figé, et le pilote en tranche TROIS termes.**
`docs/i18n-glossaire.md` existe (kickoff du 2026-08-19). Sa **partie A est contraignante** : 48
équivalences relevées dans les 1216 clés déjà alignées, chacune nommant la clé qui l'atteste.
Sa **partie B attend l'arbitrage de Guy** — **16** termes sans précédent.

⚠️ **Correction d'une affirmation de la première rédaction de cette spec.** Elle disait
« aucun terme de la partie B n'apparaît dans les 20 clés du pilote (vérifié) ». **C'est faux, et
la vérification l'a montré** : `localité` (`field-city`, `contact-error-address-npa-city`),
`prénom` (`field-first-name`, `contact-persons-name-required`, `contact-error-person-name`) et
**`personne de contact`** — ce dernier n'étant même pas *dans* la partie B — y figurent, sur
**10 des 20 clés** — recompté. *Une affirmation portant le mot « vérifié » et qui ne l'était pas est
précisément ce que la § « Recompter ses propres comptes rendus » du `CLAUDE.md` vise.*

**Ce que cela change, et ce que cela ne change pas.** Les trois termes sont **sans enjeu
sémantique** — un champ d'adresse, un champ d'état civil, un libellé d'annuaire :
`Ort` / `località` / `city`, `Vorname` / `nome` / `first name`, `Kontaktperson` / `persona di
contatto` / `contact person`. La story les fixe et les **remonte en partie A** avec la clé qui
les atteste désormais. **L'arbitrage structurant — « analytique », centres de coûts contre
projets — reste intact et hors du pilote** : la story n'est donc pas bloquée, mais elle n'est pas
non plus neutre terminologiquement, ce qui était l'affirmation d'origine.

⚠️ **`personne de contact` A DÉJÀ ÉTÉ AJOUTÉ** à la partie B, au commit de spécification de cette
story (`bb24d94c`, `i18n-glossaire.md:118`) — la rédaction précédente ordonnait de l'ajouter, ce
qui aurait produit un doublon dans un document que cette même décision déclare « INPUT figé ». La
story ne fait donc **qu'en promouvoir trois de B vers A** (AC11-bis) ; après quoi la partie B
comptera **13** entrées, et le « douze » de `i18n-glossaire.md` doit suivre. *Relevé en passe 3 :
une valeur modifiée par l'édition même de cette spec, dont les compteurs n'avaient pas été
recomptés.*

**D8-bis — Le découpage par dossier ne fuit pas, et c'est mesuré.** *(Constat de cadrage, non
exigence : aucun AC ne le contrôle, et c'est délibéré — il justifie le découpage de l'epic, il ne
décrit rien que cette story doive produire.)*
Sur les 250 clés manquantes, **2 seulement** sont demandées depuis plus d'un dossier —
`field-first-name` et `field-last-name`, et **les deux sont à l'intérieur du pilote**
(`features/contacts` et `routes/(app)/contacts`). Aucune clé n'est donc partagée entre deux
stories de rollout : chaque story peut vider sa part de l'allowlist sans coordination.

**D9 — Registre d'adresse, mesuré et non supposé.**
`de-CH` **vouvoie** (Sie-Form, **115 messages** — 117 *lignes*), `en-CH` reste à l'impératif neutre.
`it-CH` **tutoie** : 31 impératifs à la 2ᵉ personne du singulier contre **11 messages** au registre
de courtoisie (2 impératifs « Aggiungete » + 10 lignes en `vostro`/`vostra`). Les 20 clés du pilote
suivent le registre **majoritaire**, soit le tutoiement.
⚠️ *La rédaction précédente écrivait « 31 contre 1 » — chiffre que la passe 1 avait déjà réfuté
dans le glossaire, en corrigeant la prose sans toucher ni la ligne de tableau au-dessus, ni cette
décision qui la cite. Onze messages sur 1216 ne sont plus « une anomalie ponctuelle » ; la décision
de tutoyer survit, son ordre de grandeur non.*

**D10 — Où les gardes s'exécutent, et pourquoi ça suffit.**
Garde A dans `cargo test -p kesh-i18n` (donc dans le gate backend et en CI). Garde B dans
`npm run test:unit` (donc dans le gate frontend et en CI). **Aucun nouveau script npm, aucune
nouvelle étape de CI** : les deux gates existants les portent. `lint-i18n-ownership` reste tel
quel — il répond à une autre question (l'appartenance d'un namespace à un dossier).
⚠️ **UNE exception, et elle est imposée par D7-bis** : renommer `delete` en
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
4. **AC4** — Un test vitest échoue si une clé littérale passée à `i18nMsg()` **n'existe dans
   aucun** des quatre catalogues, allowlist de dette déduite. Le message nomme la clé **et le
   fichier** qui la demande.
5. **AC5** — Le même test échoue si l'allowlist de dette contient une clé désormais présente
   **ou une clé que plus rien ne demande** (contrôle symétrique, D3). ⚠️ **« Demandées » est
   l'union des littéraux statiques ET des expansions de `MOTIFS_DYNAMIQUES`** — sans quoi ce
   contrôle chasserait les 10 clés `imported-supplier-invoices-error-*` que l'AC8 impose à cette
   même allowlist, et le raccourci consisterait à les retirer, effaçant la traçabilité vers 23-3.
6. **AC6** — Le test vitest balaie **tout** `frontend/src` — `src/routes/` compris, d'où
   **197 des 250** clés manquantes sont demandées **exclusivement** (199 le sont depuis au moins un
   fichier de `routes/` ; les 2 restantes, `field-first-name` et `field-last-name`, sont partagées
   avec `lib/features/contacts`, cf. D8-bis) —, à **une seule exception écrite** : les fichiers dont le nom
   contient `.test.` (D5-bis), qui demandent des clés fictives (`compteur`, `une-cle`) devant
   rester absentes des catalogues.
7. **AC7** — Les **8 préfixes dynamiques** (sur **10 sites d'appel**) sont déclarés avec leurs
   valeurs et contrôlés comme des clés ordinaires ; chaque préfixe porte une assertion de
   cardinalité, et la garde **compare la LISTE des sites** (fichier + préfixe) à la **liste de
   référence donnée en D4** — et non leur seul nombre : un compte dit « 9 au lieu de 10 » sans dire
   lequel, et un ajout compensant un retrait passerait inaperçu. **Le fichier de garde n'importe
   aucun module de production** : les valeurs et les cardinalités y sont écrites en dur (D4), ce
   qui se vérifie sur ses `import`.
7-bis. **AC7-bis** — L'extracteur de la garde B porte **son propre test**, qui le confronte aux
   deux formes réelles connues pour casser une extraction naïve :
   (a) un gabarit dont **l'interpolation contient des apostrophes** —
   `` i18nMsg(`bank-import-info-${info.replace(/_/g, '-')}`, info) ``
   (`BankImportUpload.svelte:547`), que ne traverse aucune classe `[^'"`]*` ;
   (b) un appel **réparti sur plusieurs lignes** — `supplier-invoices/import/+page.svelte:85-88`,
   que ne voit aucun balayage ligne à ligne.
   Sans ce test, la garde hérite du défaut de méthode qui a fait manquer un motif entier à la
   première rédaction de cette spec.
7-ter. **AC7-ter** — Les **deux** ensembles ouverts portent, en commentaire, le fait qu'aucune
   garde ne les borne : `vat-category-*` (colonne libre, sans `CHECK`, catégories créées par un
   administrateur) et **`bank-import-info-*`** (valeurs poussées par
   `crates/kesh-api/src/routes/bank_imports.rs`, aucune carte frontend à confronter — le fichier
   Rust est nommé comme source de vérité). Leurs valeurs sont déclarées **après transformation**
   (`bank-csv-profile-auto-matched`, en tirets).
8. **AC8** — Les **10 clés `imported-supplier-invoices-error-*`** révélées par cette énumération
   figurent dans l'allowlist de dette avec le commentaire qui dit **quelle story les videra**
   (23-3).
9. **AC9** — Chaque garde porte une **borne de collecte minimale** (D5) qui la fait échouer si
   son motif d'extraction cesse de trouver quoi que ce soit, **et la garde B porte une seconde
   borne par extension** (≥ 3 clés collectées depuis des `.ts`) — une perte totale de la
   couverture `.ts` ne coûterait que 5 clés sur 1002 et resterait au-dessus de la borne globale.
10. **AC10** — `frontend/scripts/harvest-i18n-fallbacks.mjs` existe, rend un fragment `.ftl`
    trié sur la sortie standard, **ne modifie aucun fichier**, **ne traite que les clés absentes
    des quatre catalogues**, et liste séparément sur la sortie d'erreur (a) les clés sans repli
    littéral — **5**, non 13, l'écart venant du périmètre — et (b) les **7 clés dont le repli
    diffère selon le site d'appel**, avec leurs variantes. ⚠️ **Ces deux nombres sont des valeurs
    de contrôle datées, pas des cibles** : le moissonneur les **calcule** et publie son décompte ;
    un écart se recompte, il ne s'ajuste pas.
11. **AC11** — Les **20 clés du domaine `contacts`** existent dans les **quatre** locales, sont
    retirées de l'allowlist de dette, et leurs libellés `de-CH` / `it-CH` / `en-CH` respectent la
    partie A du glossaire et le registre de D9. La clé `delete` est entrée sous le nom
    `contact-persons-delete` (D7-bis), son site d'appel mis à jour.
11-bis. **AC11-bis** — `docs/i18n-glossaire.md` est mis à jour : **`localité`, `prénom` et
    `personne de contact` passent de la partie B à la partie A**, chacun avec la clé du pilote qui
    l'atteste désormais. *Sans cet AC, D8 promet une promotion que rien n'exige et que les 13
    autres critères laisseraient passer.*
12. **AC12** — `duplicate-i18n-keys.test.ts` est **supprimé**, sa fonction étant reprise par la
    garde générale. *(Le contrôle qu'il exerçait ne doit pas disparaître : les **cinq** clés
    `contact-duplicate-heading`, `-others-count`, `-others-count-one`, `-ide-active`,
    `-ide-archived` restent couvertes — leur **existence** par la garde B, leur **présence dans
    les quatre locales** par la garde A.)*
12-ter. **AC12-ter** — Le **second** contrôle du fichier supprimé — « le catalogue n'a pas de clé
    du domaine que personne ne demande » — est **repris dans la garde B, borné aux préfixes
    déclarés à couverture close** (`contact-duplicate-*` pour commencer, D2). Il ne se généralise
    pas : le catalogue sert aussi les PDF et les rapports, et `reports-filename-*` déclare 7 clés
    pour 5 valeurs de `ReportType`. ⚠️ **Les nommer plutôt que les
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

- [ ] **T1 — Garde A, parité inter-locales** (AC1, AC2, AC3, AC9)
  - [ ] Accesseur `keys()` sur `I18nBundle` si le test est hors module, sinon usage direct du champ privé
  - [ ] Comparaison des quatre ensembles dans les deux sens, message d'échec nommant clés et locale
  - [ ] `crates/kesh-i18n/dette-parite-connue.txt` — les 57 clés, triées, en-tête explicatif
  - [ ] Assertions « allowlist obsolète » **dans les deux sens** + borne de collecte (`>= 1200` par locale)
- [ ] **T2 — Garde B, existence des clés demandées** (AC4, AC5, AC6, AC9, AC12, AC12-bis, AC12-ter)
  - [ ] `frontend/src/lib/shared/i18n-keys.test.ts` — parcours de tout `src` **hors fichiers `.test.*`** (D5-bis), lecture des 4 `.ftl`
  - [ ] `frontend/src/lib/shared/i18n-dette-connue.ts` — 250 + 10 clés, triées, en-tête qui nomme les stories de résorption
  - [ ] Assertions « allowlist obsolète » **dans les deux sens** + borne globale (`>= 900`) + borne `.ts` (`>= 3`)
  - [ ] Suppression de `duplicate-i18n-keys.test.ts` — **ses DEUX contrôles repris** (existence + orphelines bornées, AC12-ter) — **et conservation de `contacts-i18n-realpath.test.ts`**
- [ ] **T3 — Motifs dynamiques** (AC7, AC7-bis, AC7-ter, AC8)
  - [ ] Table `MOTIFS_DYNAMIQUES` : **8 préfixes**, valeurs en dur, `bank-import-info-*` compris
  - [ ] Assertion de cardinalité par préfixe **+ assertion sur le nombre de sites (10)**
  - [ ] Extracteur robuste **aux apostrophes dans l'interpolation ET aux appels multi-lignes**, avec son test (AC7-bis)
  - [ ] Comparaison de la **liste** des 10 sites contre la table de référence de D4
  - [ ] Aucun `import` de module de production dans le fichier de garde (AC7)
  - [ ] Commentaires d'angle mort sur **`vat-category-*` ET `bank-import-info-*`** (AC7-ter), valeurs déclarées **après transformation**
  - [ ] Les 10 `imported-supplier-invoices-error-*` en allowlist, commentaire « résorbées par 23-3 »
- [ ] **T4 — Moissonneur** (AC10)
  - [ ] `frontend/scripts/harvest-i18n-fallbacks.mjs`, sortie standard uniquement
  - [ ] Périmètre restreint aux clés absentes des 4 catalogues
  - [ ] Sortie d'erreur : 5 clés sans repli littéral **+ 7 clés à repli divergent** (valeurs de contrôle datées — le script les calcule)
  - [ ] **Même lecteur de littéral que la garde** pour le repli comme pour la clé (AC7-bis)
- [ ] **T5 — Pilote `contacts`** (AC11, AC11-bis)
  - [ ] Renommer `delete` → `contact-persons-delete` dans `ContactPersonsManager.svelte:118` (D7-bis)
  - [ ] **Substituer la ligne correspondante de `KNOWN_VIOLATIONS`** (`lint-i18n-ownership.js:112`) — sans quoi `npm run lint-i18n-ownership` rougit au gate AC13
  - [ ] Moisson des 20 replis, **relecture** des libellés `fr-CH` avant de les figer
  - [ ] Traduction `de-CH` / `it-CH` / `en-CH` sur la partie A du glossaire, registre D9
  - [ ] Retrait des 20 clés de l'allowlist de dette
  - [ ] **Promouvoir `localité`, `prénom` et `personne de contact` en partie A** de `docs/i18n-glossaire.md`, avec la clé qui les atteste — et **recompter la partie B (13 après promotion)** ainsi que sa ligne « douze autres »
- [ ] **T6 — Gates** (AC13)
  - [ ] Gate backend complet, gate frontend complet, avant tout push

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas traduire au-delà des 20 clés du pilote.** Une story-zéro qui déborde n'est plus une
  story-zéro, et les rollouts perdent leur mesure.
- **Ne pas toucher `KNOWN_VIOLATIONS`** de `lint-i18n-ownership.js` — **à UNE ligne près** : celle
  de `ContactPersonsManager.svelte:delete`, que le renommage de D7-bis fait devenir
  `…:contact-persons-delete`. C'est une substitution imposée par D7-bis, pas une dérogation à la
  règle. *(Cette phrase disait « autre question, autre garde » sans exception : elle contredisait
  D10 et T5 depuis la passe 2, et n'a été rattrapée que par le grep du symptôme.)*
- **Ne pas « corriger » les 5 clés interpolées** de `TransactionSplitModal` — elles demandent des
  entrées Fluent à variables, et c'est la 23-5 qui les porte.
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
| littéraux demandés par `i18nMsg()` | **1010** distincts = **1002 statiques** + **8 gabarits dynamiques** ; 752 des statiques existent |
| clés statiques manquantes | **250**, sur 13 dossiers, dont 197 sous `src/routes/` |
| préfixes dynamiques / sites d'appel | **8** / **10** |
| clés révélées par les motifs dynamiques | **+10** (`imported-supplier-invoices-error-*`) |
| **total de l'epic** | **317** |
| replis moissonnables | 245 ; 5 interpolés |

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

⚠️ **Le décompte du registre de D9 porte sur des MESSAGES, pas sur des lignes** : 115 messages
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
termes au glossaire n'étaient exigés par aucun AC (**AC12-bis** et **AC11-bis** ajoutés) ; la borne
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
