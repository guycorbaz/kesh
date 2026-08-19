# Story 23.1b : Le pilote — vingt clés, et la terminologie qu'elles engagent

## Status

review

Seconde moitié du split de la **23-1**, arbitré par Guy le 2026-08-19. La première est la
**23-1a**, qui pose les deux gardes et leurs allowlists. **Cette story-ci ne peut pas démarrer
avant que la 23-1a soit mergée** : elle décrémente des allowlists qui n'existent pas encore.

⚠️ **Tout ce qui se pèse en mots est ici.** Vingt libellés `fr-CH` à relire, soixante traductions à
écrire dans trois langues, trois termes de glossaire à promouvoir, un registre d'adresse à
respecter. Aucune ligne de garde, aucun extracteur : c'est ce partage qui rend les deux moitiés
relisables séparément.

## Story

**As a** personne qui prépare les cinq rollouts de l'Epic 23,
**I want** que le premier domaine soit traité de bout en bout, terminologie comprise,
**so that** les stories suivantes copient un geste éprouvé au lieu de le réinventer chacune.

## Périmètre

**Dedans** : le moissonneur de replis, les 20 clés du domaine `contacts` entrées dans les quatre
locales, le renommage de `delete` et sa ligne de `KNOWN_VIOLATIONS`, la promotion de trois termes
en partie A du glossaire, le registre d'adresse.

**Dehors, et c'est la 23-1a** : les deux gardes, l'extracteur, les motifs dynamiques, les bornes,
les allowlists elles-mêmes — cette story ne fait qu'en **retirer 20 lignes**.
**Dehors aussi** : les 265 autres clés statiques de [#316] (rollouts 23-2 à 23-6), les 5 entrées Fluent à variables de
`TransactionSplitModal` (23-5), [#255], [#314], [#242].

⚠️ **Les identifiants de décisions et de critères sont ceux de la 23-1 d'origine.** Ceux qui
manquent ici vivent dans la 23-1a — en particulier **D5-bis** (cf. 23-1a, exclusion des `.test.*`) et
**AC7-bis** (cf. 23-1a, l'extracteur robuste), dont le moissonneur de D6 réutilise le lecteur de littéral.

## Chiffres de référence

Recomptés le 2026-08-19 et confirmés par trois passes de revue indépendantes. **Le tableau complet
et les commandes de recompte sont dans la 23-1a, § *Chiffres de référence*** ; ne sont repris ici
que ceux dont cette story se sert.

| | valeur |
|---|---|
| clés du pilote `contacts` | **20** — 12 sous `lib/features/contacts`, 8 sous `routes/(app)/contacts` |
| clés à repli moissonnable (tout le dépôt) | **274 des 285** — 245 atteintes directement, +29 via les relais ; les 5 restantes ont un repli **interpolé** (ligne suivante), les 6 dernières viennent de l'inventaire (23-1a § D4-ter). ⚠️ **245 est la sortie d'un moissonneur AVEUGLE AUX RELAIS**, pas une cible |
| clés sans repli littéral | 5, toutes dans `TransactionSplitModal` — **hors périmètre**, portées par la 23-5 |
| clés à repli **divergent** | **7** — valeur de contrôle datée, le moissonneur la calcule |
| partie B du glossaire | **15** entrées, dont **3** promues par cette story → **12** ensuite |

## Décisions

**D6 — Le moissonneur PROPOSE, il n'écrit jamais dans les catalogues.**
⚠️ **Il exclut les fichiers dont le nom contient `.test.`**, comme la garde B (23-1a § D5-bis).
Sans cette clause — citée en renvoi jusqu'ici, mais jamais imposée au script —, il moissonnerait
`i18n.svelte.test.ts`, qui demande `une-cle` et `compteur` : deux clés **fictives**, absentes des
quatre catalogues, dont `compteur` porte **deux replis divergents** à lui seul. Il produirait donc
`une-cle = mon repli` dans le fragment `.ftl` et annoncerait **8** clés à repli divergent au lieu
de 7. *(L'identifiant `D5-bis` avait traversé le découpage, sa substance non. Relevé en passe 1 du
split.)*
⚠️ **Il reconnaît aussi les relais locaux** (23-1a § D4-bis) : sans quoi il ignore les 29 clés
qu'ils portent, dont 25 de `routes/(app)/settings`.
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
catalogue sans relecture, c'est faire entrer **285** approximations dans le produit.** Chaque story
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
`contact-error-person-name` ; `personne de contact` → les **cinq** `contact-persons-*` dont le libellé emploie le mot
« personne » — `-load-error`, `-title` et `-empty` disant « personne(s) **de contact** », `-add-error`
et `-delete-error` disant seulement « la personne » — plus `contact-error-person-name`,
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

**D8 — Le glossaire est figé POUR LE RESTE DE CETTE STORY, et le pilote en promeut TROIS termes.**
*(Il n'est pas « figé » au sens absolu : le commit de spécification de cette story y a ajouté `personne de contact`. Il l'est à partir de maintenant — la story n'y touche plus que pour la promotion d'AC11-bis.)*
`docs/i18n-glossaire.md` existe (kickoff du 2026-08-19). Sa **partie A est contraignante** : **52** équivalences,
chacune nommant la clé qui l'atteste — 48 relevées au kickoff, plus « projet analytique »
(arbitrage du 2026-08-19) et les **trois** que cette story y promeut.
Sa **partie B compte désormais **12** termes** sans précédent : 16 au kickoff, 15 après
l'arbitrage sur « analytique », **12** après la promotion livrée ici.

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
story (`bb24d94c`, `i18n-glossaire.md:122`) — la rédaction précédente ordonnait de l'ajouter, ce
qui aurait produit un doublon dans un document que cette même décision déclare « INPUT figé ». La
story ne fait donc **qu'en promouvoir trois de B vers A** (AC11-bis) ; après quoi la partie B
comptera **12** entrées, et le « douze » de `i18n-glossaire.md` doit suivre. *Relevé en passe 3 :
une valeur modifiée par l'édition même de cette spec, dont les compteurs n'avaient pas été
recomptés.*

**D8-bis — Le découpage par dossier FUIT, et huit clés du pilote sont partagées avec un dossier
hors périmètre.** *(**Décision normative**, contrôlée par AC11-ter. La rédaction précédente la
disait « constat de cadrage, non exigence » — c'était vrai de la version qui ne comptait que
deux clés partagées ; elle porte désormais une obligation de relecture, et une obligation
qu'aucun critère ne contrôle n'est pas tenue.)*

⚠️ **La rédaction précédente affirmait l'inverse** — « 2 seulement sont partagées, et les deux sont
à l'intérieur du pilote » — et c'était faux, parce que le recensement ne voyait pas les **relais**
(cf. 23-1a § D4-bis). Réfuté en passe 1 du split, au sol.

`frontend/src/routes/onboarding/+page.svelte` déclare `function msg(key, fallback) { return
i18nMsg(key, fallback) }` puis appelle, à l'étape « Coordonnées de votre organisation », **les huit
clés `field-*` du pilote** : `field-first-name`, `field-last-name`, `field-address`, `field-street`,
`field-building`, `field-postal-code`, `field-city`, `field-country`. Un **troisième dossier**,
absent de cette story comme du découpage de l'epic.

**Ce que cela engage, et qui doit être décidé plutôt que subi** : une fois ces huit clés traduites,
l'écran d'onboarding — aujourd'hui en français pour tout le monde — se met à afficher **les libellés
choisis pour le carnet d'adresses**, sans qu'aucune revue ne les ait regardés dans ce contexte. ⚠️ **Aucun verdict n'est
pré-annoncé ici, et c'est délibéré** : une rédaction antérieure écrivait « en l'espèce ils
conviennent », donnant la conclusion avant l'exercice — un contrôle dont la sortie est écrite
d'avance ne peut pas rougir. L'hypothèse à confirmer est que ces étiquettes valent dans les deux
écrans ; le cas le plus tendu est `field-building`, dont le repli `N°` est une abréviation dont
l'équivalent varie (`Nr.` / `N.` / `No.`).
T5 doit relire les huit libellés **dans les deux contextes** avant de les figer.

**D9 — Registre d'adresse, mesuré et non supposé.**
`de-CH` **vouvoie** (Sie-Form, **115 messages** — 117 *lignes*), `en-CH` reste à l'impératif neutre.
`it-CH` **tutoie** : 31 impératifs à la 2ᵉ personne du singulier contre **11 messages** au registre
de courtoisie (2 impératifs « Aggiungete » + 10 lignes en `vostro`/`vostra`). Les 20 clés du pilote
suivent le registre **majoritaire**, soit le tutoiement.
⚠️ *La rédaction précédente écrivait « 31 contre 1 » — chiffre que la passe 1 avait déjà réfuté
dans le glossaire, en corrigeant la prose sans toucher ni la ligne de tableau au-dessus, ni cette
décision qui la cite. Onze messages sur 1216 ne sont plus « une anomalie ponctuelle » ; la décision
de tutoyer survit, son ordre de grandeur non.*

## Acceptance Criteria

10. **AC10** — `frontend/scripts/harvest-i18n-fallbacks.mjs` existe, rend un fragment `.ftl`
    trié sur la sortie standard, **ne modifie aucun fichier**, **ne traite que les clés absentes
    des quatre catalogues**, et liste séparément sur la sortie d'erreur (a) les clés sans repli
    littéral — **5**, non 13, l'écart venant du périmètre — et (b) les **7 clés dont le repli
    diffère selon le site d'appel**, avec leurs variantes. ⚠️ **Ces deux nombres sont des valeurs
    de contrôle datées, pas des cibles** : le moissonneur les **calcule** et publie son décompte ;
    un écart se recompte, il ne s'ajuste pas.
    ⚠️ **TROIS clauses de D6 sont exigées ICI, faute de quoi elles ne sont pas contrôlées** :
    **(c) le moissonneur recense les 7 relais locaux** (23-1a § D4-bis) et collecte les littéraux
    passés à chacun. ⚠️ **Sans cette clause, un moissonneur aveugle aux relais rend `5` et `7` —
    EXACTEMENT les deux valeurs que cet AC contrôlait** : elles ne discriminent pas. Il rendrait
    245 entrées au lieu de **274**, perdant 29 clés dont 21 de `settings/email-templates`.
    **La valeur de contrôle qui discrimine est donc `274`.**
    **(d) le moissonneur porte SON PROPRE TEST vitest** — `frontend/src/lib/shared/i18n-harvest.test.ts`,
    donc dans le périmètre de `test:unit` —, à trois fixtures reprenant les trois défauts déjà payés :
    (i) un fichier `*.test.ts` demandant `une-cle`, absent de la sortie ; (ii) une même clé demandée
    avec deux replis différents, présente dans la liste des conflits ; (iii) un repli entre
    guillemets doubles contenant une apostrophe (`"Date d'exécution"`), lu entier.
    ⚠️ *Sans ce test, **rien n'exécute le moissonneur** : `vite.config.ts:59` borne vitest à
    `src/**/*.test.ts`, et `frontend/scripts/` n'est dans aucun gate. Toute la substance d'AC10
    n'aurait pour preuve que deux nombres recopiés à la main.*
    ⚠️ **Les deux clauses déjà écrites** :
    (a) le moissonneur **exclut les fichiers `.test.*`** — sans quoi il entre `une-cle` au catalogue
    et annonce 8 conflits au lieu de 7 ; (b) il emploie **le lecteur de littéral d'AC7-bis** pour la
    clé **comme pour le repli** — sans quoi il rate `payment-batches-col-date`, dont les deux replis
    sont entre guillemets doubles, et n'en annonce que 6.
11-ter. **AC11-ter** — Les **huit libellés `field-*`** ont été relus **dans les deux contextes**
    (carnet d'adresses et étape « Coordonnées » de l'onboarding, D8-bis — **trois** contextes pour
    `field-first-name` et `field-last-name`, aussi demandés par `ContactPersonsManager`), et la
    relecture est consignée au Dev Agent Record **sous forme d'un tableau de huit lignes** donnant,
    par clé : le repli lu à chaque site, le libellé `fr-CH` retenu et les trois traductions.
    ⚠️ **Chaque repli déclaré doit se retrouver au `grep -nF` dans le fichier cité** — sans quoi le
    critère n'est qu'une case à cocher, et un relecteur ne peut pas distinguer « j'ai relu » de
    « j'ai écrit que j'ai relu ». ⚠️ *Aucun test ne couvre l'écran d'onboarding en langue non
    française : ses spécifications E2E n'ont que des sélecteurs français. La relecture humaine est
    le seul contrôle, d'où cet AC.*
11-sexies. **AC11-sexies** — Les **20 libellés `fr-CH`** ont été relus avant d'être figés —
    capitale initiale, ponctuation finale, formulation de catalogue et non de repli — et **tout
    écart au repli moissonné est consigné** au Dev Agent Record avec sa raison. ⚠️ *Un repli
    recopié verbatim est un choix, pas un défaut ; c'est de n'avoir pas regardé qui en est un.
    Sans ce critère, `contact-persons-hint` entre au catalogue en « à titre informatif » — sans
    majuscule ni point — et `field-building` en « N° » : c'est le patron que les 265 clés des
    rollouts recopieraient, et c'est le contraire exact de ce que D6 prescrit.*
11-quater. **AC11-quater** — `docs/i18n-glossaire.md` est **cohérent après promotion** : partie B à
    **12** entrées, **son préambule recompté — « 352 clés », « 1056 messages » (`:5-6`), la valeur
    `317`/`951` datant d'avant le recompte des relais** —, et le paragraphe qui l'accompagne réécrit
    **au passé** (il est aujourd'hui au
    futur : « trois de ces termes sont tranchés… les treize autres resteront ouverts »).
11-quinquies. **AC11-quinquies** — Le commentaire de `lint-i18n-ownership.js:103` ne cite plus
    `delete` parmi les « clés génériques » : la clé renommée porte désormais un domaine, elle n'est
    plus générique. *(Retrait, non remplacement.)*

11. **AC11** — Les **20 clés du domaine `contacts`** existent dans les **quatre** locales, sont
    retirées de l'allowlist de dette, et leurs libellés `de-CH` / `it-CH` / `en-CH` respectent la
    partie A du glossaire et le registre de D9 — **et leurs libellés `fr-CH` ont été relus**
    (AC11-sexies). La clé `delete` est entrée sous le nom `contact-persons-delete` (D7-bis), son
    site d'appel mis à jour. ⚠️ **Clause négative, vérifiable d'un grep** : la chaîne `^delete *=`
    n'apparaît **dans aucun** des quatre `messages.ftl`.

11-bis. **AC11-bis** — `docs/i18n-glossaire.md` est mis à jour : **`localité`, `prénom` et
    `personne de contact` passent de la partie B à la partie A**, chacun avec la clé du pilote qui
    l'atteste désormais. *Sans cet AC, D8 promet une promotion qu'aucun autre critère n'exige.*

13. **AC13** — Les deux gates complets passent : backend (`cargo test --workspace`) et frontend
    (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`).

## Tasks / Subtasks

- [x] **T4 — Moissonneur** (AC10)
  - [x] Recensement des 7 relais + **valeur de contrôle 274** (AC10 c)
  - [x] `i18n-harvest.test.ts` à trois fixtures, dans le périmètre de `test:unit` (AC10 d)
  - [x] Import du module partagé `i18n-literal-reader` (23-1a § D1-bis)
  - [x] Exclusion des `.test.*` et lecteur de littéral d'AC7-bis pour la clé ET le repli (AC10)
  - [x] `frontend/scripts/harvest-i18n-fallbacks.mjs`, sortie standard uniquement
  - [x] Périmètre restreint aux clés absentes des 4 catalogues
  - [x] Sortie d'erreur : 5 clés sans repli littéral **+ 7 clés à repli divergent** (valeurs de contrôle datées — le script les calcule)
  - [x] **Même lecteur de littéral que la garde** pour le repli comme pour la clé (AC7-bis)

- [x] **T5 — Pilote `contacts`** (AC11, AC11-bis, AC11-ter, AC11-quater, AC11-quinquies, AC11-sexies)
  - [x] ⚠️ **L'ORDRE EST NORMATIF : le renommage de `delete` PRÉCÈDE la moisson.** Moissonner
        d'abord ferait entrer `delete = Supprimer` au catalogue — et **tous les gates resteraient
        verts** : parité satisfaite, plus rien ne demande la clé donc la garde B se tait, sa ligne
        d'allowlist est retirée par cette story, le contrôle des orphelines est borné à
        `contact-duplicate-*`, et le lint ne détecte pas les entrées mortes. La clause négative
        d'AC11 (`^delete *=` absente des quatre `.ftl`) est le seul filet.
  - [x] Renommer `delete` → `contact-persons-delete` dans `ContactPersonsManager.svelte:118` (D7-bis)
  - [x] **Substituer la ligne correspondante de `KNOWN_VIOLATIONS`** (`lint-i18n-ownership.js:112`) — sans quoi `npm run lint-i18n-ownership` rougit au gate AC13
  - [x] Moisson des 20 replis, **relecture** des libellés `fr-CH` avant de les figer
  - [x] Traduction `de-CH` / `it-CH` / `en-CH` sur la partie A du glossaire, registre D9
  - [x] Retrait des 20 clés de l'allowlist de dette
  - [x] **Promouvoir `localité`, `prénom` et `personne de contact` en partie A** de `docs/i18n-glossaire.md`, avec la clé qui les atteste
  - [x] **Recompter la partie B (12 entrées après retrait des 3 promus)** et **réécrire le paragraphe qui l'accompagne** — il est au futur (« trois de ces termes sont tranchés… les treize autres resteront ouverts ») et doit passer au passé, la promotion étant faite. ⚠️ *Ne pas chercher la chaîne « douze autres » : elle a déjà été corrigée en « treize » à la passe 3, et la tâche pointait encore dessus.*
  - [x] **Mettre à jour le commentaire de `lint-i18n-ownership.js:103`**, qui cite `delete` parmi les « clés génériques » — **retirer la mention**, la clé renommée portant désormais un domaine (§ *Propagation post-patch*)
  - [x] **Relire les huit libellés `field-*` DANS LES DEUX CONTEXTES** — carnet d'adresses et étape « Coordonnées » de l'onboarding (D8-bis)

- [x] **T6 — Gates** (AC13)
  - [x] Gate backend complet, gate frontend complet, avant tout push

## Dev Notes

### Ce que cette story ne doit PAS faire

- **Ne pas toucher aux gardes ni à l'extracteur** — ils sont livrés par la 23-1a. Si un patch touche
  `i18n-keys.test.ts` autrement que pour en retirer 20 lignes d'allowlist, c'est qu'il déborde.
- **Ne pas traduire au-delà des 20 clés du pilote** — les 29 clés révélées par les relais (23-1a § D4-bis) appartiennent aux rollouts, pas à cette story.
- **Ne pas « corriger » les 5 clés interpolées** de `TransactionSplitModal` : entrées Fluent à
  variables, portées par la 23-5.
- **Ne pas ajouter de clé à une seule locale** : la garde A de la 23-1a le refuserait, et ce serait
  creuser [#283] dans la story qui vient le borner.

### Ce que le dev doit lire avant d'écrire

| Fichier | Pourquoi |
|---|---|
| `_bmad-output/implementation-artifacts/23-1a-mecanisme-gardes-i18n.md` | les gardes que cette story alimente, et les chiffres de référence complets |
| `docs/i18n-glossaire.md` | terminologie contraignante (partie A) et registre mesuré |
| `frontend/scripts/lint-i18n-ownership.js:101-118` | les neuf clés sœurs déjà inscrites, et la ligne `:112` à substituer |
| `frontend/src/lib/features/contacts/ContactPersonsManager.svelte` | les 12 clés du pilote côté feature, et le site de `delete` |
| `frontend/src/routes/(app)/contacts/+page.svelte` | les **8 autres** clés du pilote — 40 % des sites porteurs |
| `frontend/src/routes/onboarding/+page.svelte` | le **troisième consommateur** des huit `field-*`, via son relais `msg()` (D8-bis) |

### References

- [Source: `_bmad-output/planning-artifacts/epic-23-dette-i18n.md`] — plan d'epic
- [Source: `_bmad-output/implementation-artifacts/23-1a-mecanisme-gardes-i18n.md`] — la moitié mécanisme, et l'historique complet des trois passes de `validate`
- [Source: `docs/i18n-glossaire.md`] — terminologie contraignante, registre mesuré
- [#316] : https://github.com/guycorbaz/kesh/issues/316
- [#283] : https://github.com/guycorbaz/kesh/issues/283

## Change Log

### Split — 2026-08-19, arbitrage de Guy

Née du split de la 23-1 après trois passes de `validate` (`2C/1H/8M/3L → 0C/4H/6M/2L →
0C/2H/10M/6L`, plafond stagnant à HIGH). **L'historique détaillé des trois passes est au Change Log
de la 23-1a** — les deux moitiés partant de la même spec, le dupliquer le ferait diverger.

**Ce qui revient spécifiquement à cette moitié**, des findings des trois passes :

- **sept** clés à repli divergent et non six (`payment-batches-col-date` manquait — troisième
  récidive de la classe de caractères négative, cette fois sur le **repli** et non sur la clé) ;
- la clé `delete`, qui allait entrer au catalogue **sans domaine**, et dont le renommage impose une
  **substitution** dans `KNOWN_VIOLATIONS` — le contraire de ce que la première rédaction affirmait ;
- la partie B du glossaire, qui compte **16** entrées et non 15, `personne de contact` y ayant été
  ajouté par le commit de spécification lui-même : la story **promeut**, elle n'ajoute pas ;
- l'attestation des trois termes, qui disait « six » `contact-persons-*` pour **cinq** ;
- le registre italien, à **11 messages** de courtoisie et non un.

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

**Le détail complet des MEDIUM et LOW est au Change Log de la 23-1a**, qui porte l'historique de la boucle.

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
qui ne résolvent pas** — **54 sur 1514 à l'estimation, 33 sur 1493 une fois le lecteur réel écrit**
(cf. 23-1a ; les valeurs vivent dans `ATTENDU` de `i18n-keys.test.ts`) —, ensemble clos, comptable,
décidable. Chaque site y est soit
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

**L'ordre normatif de `T5` a été suivi, et il servait à quelque chose.** Le renommage de `delete`
en `contact-persons-delete` a précédé la moisson. Effet observé immédiatement : la garde B a rougi
sur **deux** tests — la clé renommée demandée sans être connue, l'ancienne connue sans être demandée
—, ce qui est exactement le comportement attendu du contrôle symétrique. Moissonner d'abord aurait
fait entrer `delete = Supprimer` au catalogue **tous gates verts**, comme la spec l'annonçait.

**La convention du catalogue m'a évité une réécriture fautive.** J'allais capitaliser
`contact-persons-hint = à titre informatif`. Recompté avant d'agir : **29 clés du catalogue
commencent par une minuscule**, dont `onboarding-field-ide-hint = optionnel, format CHE-…` — un
*hint*, exactement le même cas. Le second chiffre avancé ici — « les messages d'erreur sans point final sont
78 contre 153 avec » — a été **retiré : il n'était pas remesurable**, aucune définition de « message
d'erreur » n'étant donnée, et cinq définitions plausibles essayées en passe 3 donnent des totaux
compris entre 152 et 190, jamais 231. La décision ne repose pas dessus : elle repose sur les **29
minuscules**, recomptées et exactes. *Les vingt replis sont donc entrés VERBATIM, et c'est une décision relue, pas une omission.*

**Deux conventions suisses posées faute de précédent** : le fichier `de-CH` ne contenait **aucun
`ß`** — l'entrée `field-street = Strasse` ne l'introduit pas ; et `NPA` est retenu pour `it-CH`,
usage de la Poste suisse, contre le `CAP` italien.

**Un choix de terminologie s'écarte des trois autres locales, délibérément** : `contact-persons-role`
rend `Fonction` par `Funktion` / `Funzione` mais par **`Position`** en anglais — le glossaire réserve
« role » aux rôles RBAC, et confondre les deux concepts au catalogue serait une dette de vocabulaire.

### Relecture des huit libellés partagés — AC11-ter

⚠️ **Consignée sous une forme qui se vérifie**, et non comme une case cochée : chaque repli ci-dessous
se retrouve au `grep -nF` dans le fichier cité. Un tableau qu'on peut contrôler distingue « j'ai
relu » de « j'ai écrit que j'ai relu ».

| clé | carnet d'adresses | onboarding | CRM personnes | fr-CH retenu | de-CH | it-CH | en-CH |
|---|---|---|---|---|---|---|---|
| `field-first-name` | Prénom | Prénom | Prénom | Prénom | Vorname | Nome | First name |
| `field-last-name` | Nom | Nom | Nom | Nom | Name | Cognome | Last name |
| `field-address` | Adresse | Adresse | — | Adresse | Adresse | Indirizzo | Address |
| `field-street` | Rue | Rue | — | Rue | Strasse | Via | Street |
| `field-building` | N° | N° | — | N° | Nr. | N. | No. |
| `field-postal-code` | NPA | NPA | — | NPA | PLZ | NPA | Postal code |
| `field-city` | Localité | Localité | — | Localité | Ort | Località | Town/city |
| `field-country` | Pays | Pays | — | Pays | Land | Paese | Country |

**Verdict de la relecture** : les huit libellés sont **identiques dans tous leurs contextes** — le
détecteur de conflits du moissonneur le confirme, aucun `field-*` ne figure parmi les 7 clés à repli
divergent. Ce sont des étiquettes de champ d'adresse, et elles valent dans les trois écrans.
⚠️ **Le cas le plus tendu était `field-building`**, dont le repli `N°` est une abréviation française
dont l'équivalent varie : `Nr.` en allemand, `N.` en italien, `No.` en anglais. C'est le seul des
huit où la traduction n'est pas mécanique.

⚠️ **Conséquence assumée et transitoire** : l'écran d'onboarding devient **partiellement traduit**.
Ses huit étiquettes d'adresse le sont désormais ; ses quatre messages propres
(`onboarding-address-npa-city-required`, `-already-finalized`, `-finalize-incomplete`,
`-field-name-hint`) restent au repli français jusqu'à la **23-4**. Un germanophone verra donc, sur le
même formulaire, huit libellés traduits et quatre messages en français.

### Completion Notes List

- **Moissonneur** — logique dans `src/lib/shared/i18n-harvest.js` (importable), enveloppe CLI dans
  `scripts/harvest-i18n-fallbacks.mjs`. ⚠️ **Le module existe parce que `vite.config.ts` borne vitest
  à `src/**`** : un script de `scripts/` n'est exécuté par aucun gate, et toute la substance d'AC10
  n'aurait eu pour preuve que des nombres recopiés à la main. Il rend, **contre les catalogues d'AVANT la story**, exactement les
  valeurs de contrôle de la spec : **274 clés moissonnées, 5 sans repli littéral, 7 à repli divergent**.
- **Sept preuves au commit de dev** (`i18n-harvest.test.ts`) — **dix à `HEAD`**, les passes de revue
  en ayant ajouté trois. Dont les trois exigées par AC10 (d) — fichier de test
  ignoré, conflit de repli détecté, repli entre guillemets doubles contenant une apostrophe lu entier.
- **Renommage** `delete` → `contact-persons-delete` : site d'appel, ligne de `KNOWN_VIOLATIONS`
  **substituée** (non ajoutée), et `delete` **retiré de la liste des clés génériques** du commentaire
  `:103` — cinq lignes l'évoquant y ont en revanche été **ajoutées**, pour expliquer le retrait.
- **20 clés entrées dans les quatre locales**, retirées de l'allowlist (295 → 275).
- **Glossaire** : `localité`, `prénom` et `personne de contact` promus en partie A avec la clé qui
  les atteste ; partie B recomptée à **12** ; paragraphe passé au passé ; préambule aligné.

### File List

| fichier | |
|---|---|
| `frontend/src/lib/shared/i18n-harvest.js` | **neuf** — logique du moissonneur, importable |
| `frontend/src/lib/shared/i18n-harvest.test.ts` | **dix preuves à `HEAD`** (7 au commit de dev) |
| `frontend/scripts/harvest-i18n-fallbacks.mjs` | **neuf** — enveloppe CLI |
| `frontend/src/lib/features/contacts/ContactPersonsManager.svelte` | `delete` → `contact-persons-delete` |
| `frontend/scripts/lint-i18n-ownership.js` | ligne substituée + commentaire `:103` |
| `frontend/src/lib/shared/i18n-dette-connue.ts` | 295 → 275 entrées |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` | +20 clés chacune |
| `docs/i18n-glossaire.md` | 3 termes promus, partie B à 12 |
| `_bmad-output/implementation-artifacts/23-1b-…md`, `sprint-status.yaml` | statut, record |

### Passe 1 de `bmad-code-review` — 2026-08-19, Sonnet ×3, contextes frais

**0 CRITICAL · 1 HIGH · 4 MEDIUM · 2 LOW.** ⚠️ **Presque tous les findings portent sur le CONTENU
des traductions**, pas sur le code — ce qui est le juste centre de gravité pour cette moitié, et ce
que la consigne de revue demandait explicitement de regarder.

**Le HIGH est un faux ami INTERNE.** `contact-error-person-name` rendait « personne » par
**`individual`** en anglais — or le glossaire réserve ce mot à la personne **physique**, par
opposition à la personne morale (`legal entity`). Les dix-neuf autres clés du lot disent `person` ou
`contact person`. Le mot portait donc deux sens dans le même catalogue, ambiguïté qu'aucune des
trois autres locales n'avait. Corrigé en `contact person`.

**Un MEDIUM que je n'aurais pas trouvé seul** : `field-city = City`. `Ort`, `Localité` et `Località`
sont le terme **large** — lieu, localité —, choisi contre le cognat étroit `Stadt` / `Ville` /
`Città` **parce que beaucoup de communes suisses ne sont pas des villes**. `City` est précisément ce
cognat étroit que les trois autres locales avaient écarté. Rendu par `Town/city`, forme usuelle des
formulaires d'adresse.

**Deux MEDIUM sur le moissonneur, de la même famille que ceux de la story sœur** :
- **le filtre d'exclusion des `.test.*` vivait dans le script, donc hors de tout gate**, alors que
  le docstring du test revendiquait l'« exclusion des tests » parmi ce qu'il prouve. Déplacé dans le
  module sous `dansLePerimetreDeFichier`, et testé sur six noms.
- **le fragment `.ftl` ne garantissait pas sa propre syntaxe** : un repli à retour à la ligne ou à
  accolade non appariée y entrait tel quel. ⚠️ **La conséquence n'est pas locale** — `loader.rs:71`
  propage l'erreur de `FluentResource::try_new` sans tri, donc **une seule entrée cassée empêche le
  chargement de TOUTE la locale**. Vérifié par la lentille avec le **vrai parseur Fluent**.
  ⚠️ **Et le correctif naïf aurait cassé six replis légitimes** : `Facture #{$id} enregistrée.`,
  `{$n} facture(s) importée(s).` portent de vrais placeables que le frontend interpole. `estFtlSain`
  n'écarte donc que les accolades **non appariées**, jamais les placeables valides.

**MEDIUM de comptes rendus** : le glossaire annonçait « 60 messages » sans dire que le décompte
porte sur les **trois langues cibles**, les libellés `fr-CH` étant la source — alors que le même
diff affirme les avoir *relus*, donc traités comme un travail à part entière. Précisé.

**LOW** : mon commentaire allemand annonçait la Sie-Form, mais **aucune des vingt chaînes ne porte
de forme d'adresse directe** — ce sont des passifs et des infinitifs de bouton ; et il affirmait que
le fichier « ne contient aucun ß » **tout en en introduisant deux**, en citant « Straße » comme
contre-exemple. Reformulé. Le nom du fichier de test (`i18n-harvest.test.ts` contre `harvest.test.ts`
dans l'AC) est aligné sur le code, plus cohérent avec les autres modules `i18n-*`.

**Ce que la revue a CONFIRMÉ compte autant** : les **8** critères tenus, les deux gates **rejoués et non
recopiés**, le tableau de relecture d'`AC11-ter` vérifié **ligne à ligne au `grep -nF`**, les vingt
libellés `fr-CH` confirmés entrés **verbatim**, et le moissonneur qui reproduit **274 / 5 / 7** une
fois les locales remises à leur état d'avant la story — la contre-preuve que je n'avais pas faite.

### Passe 2 de `bmad-code-review` — 2026-08-19, Haiku ×3, diff aplati

**0 CRITICAL · 1 HIGH · 1 MEDIUM · 1 LOW.** Le trend descend : `1 HIGH / 4 MED / 2 LOW` → `1 HIGH /
1 MED / 1 LOW`. Et **`AcceptanceAuditor` n'a rien trouvé de neuf** — les **8** critères tenus, les gates
rejoués, la terminologie des vingt clés confirmée conforme au glossaire après les deux correctifs de
la passe 1.

⚠️ **Le HIGH est encore une régression du patch de la passe précédente** — cinquième fois de suite
sur ce dossier, en comptant les quatre passes de la story sœur. Aucune passe n'a jamais trouvé de
défaut dans la conception d'origine.

**Le HIGH : un maillon prouvé ne prouve pas la chaîne.** La passe 1 avait déplacé le filtre des
`.test.*` du script vers le module, pour le rendre testable — bon geste, à demi fait : la
**garantie** reposait toujours sur la politesse de l'appelant. Mesuré, pas supposé :

```
# le filtre retiré du script, ligne 37
Tests  8 passed (8)          ← VERT. Le garde-fou ne gardait rien.
```

C'est le mode d'échec que `contacts-i18n-realpath.test.ts` documente déjà dans ce dépôt, et que
cette story avait **déjà payé une fois** — `une-cle = mon repli` était entré au catalogue. Corrigé
en faisant appliquer le périmètre par `moissonner` **lui-même**.

⚠️ **La lentille avait classé cela CRITICAL en affirmant que le moissonneur incluait les `.test.*`.
C'était faux** : elle l'avait appelé en court-circuitant l'énumération de fichiers, donc en sautant
l'étape qui filtre. Le grep ground-truth a réfuté l'observation ; c'est en creusant sa piste qu'on
trouve le vrai défaut, plus grave et ailleurs.

**Le MEDIUM : la passe 1 n'avait gardé qu'un côté du signe `=`.** `estFtlSain` contrôlait la
**valeur** ; la **clé** entrait telle quelle. Une clé vide, numérique, ou pointée produit une ligne
que le parseur rejette — et `loader.rs` propageant l'erreur **sans tri**, elle emporte **toute la
locale**. Exactement le raisonnement qui avait motivé `estFtlSain`, appliqué à sa moitié manquante.
⚠️ Le `.` n'est pas exclu par étourderie : en Fluent il introduit un **attribut**, si bien que
`foo.bar = x` serait accepté **et faux**.

**Les deux correctifs sont vérifiés par mutation**, et c'est la seule preuve qui vaille ici :

| mutation | avant | après |
|---|---|---|
| périmètre retiré de `moissonner` | — | **1 failed** / 9 passed |
| contrôle de clé retiré | — | **1 failed** / 9 passed |

**LOW** : ajout de l'assertion sur le second repli à placeable cité par le Change Log de la passe 1
(`{$n} facture(s) importée(s).`), le premier seul étant couvert.

**Recomptés depuis la source par `EdgeCaseHunter`, et tous justes** : 20 clés × 4 locales = 80
vérifications, allowlist 295 → 275 (delta exact de 20), glossaire 52 / 12, moissonneur **274 / 5 / 7 contre les catalogues d'avant la story**.

⚠️ **Le périmètre fait partie du nombre.** Lancé sur le dépôt à `HEAD`, le moissonneur rend
**254 / 5 / 7** — les vingt clés du pilote étant désormais au catalogue, il ne les moissonne plus.
La règle générale est `274 − (clés déjà résorbées)` : un auteur de rollout obtiendra 234, puis 141,
et ce n'est **pas** une régression du moissonneur. `AC10` désigne 274 comme valeur de contrôle ; elle
ne vaut que contre les catalogues du kickoff.
Les décomptes — lieu du défaut sur les stories précédentes de ce dossier — tiennent cette fois.

### Passe 3 de `bmad-code-review` — 2026-08-19, Opus ×2, contextes frais

**0 CRITICAL · 3 HIGH · 7 MEDIUM · 5 LOW.** ⚠️ **La sévérité REMONTE** (`1H/4M/2L` → `1H/1M/1L` →
`3H/7M/5L`). Mais il faut voir *ce que* la passe a trouvé : **aucun finding ne porte sur le
comportement du code livré**. Les deux lentilles n'étaient pas orthogonales aux précédentes par le
modèle — elles l'étaient par **l'angle** : les deux premières passes regardaient le code et les
traductions, celle-ci regarde les **comptes rendus** et la **robustesse**. La remontée mesure un
angle jamais braqué, pas un délitement.

**Le HIGH le plus grave : le glossaire était réfuté par la clé qu'il cite comme preuve.**
`docs/i18n-glossaire.md:82` donnait `city` pour `en-CH`, en nommant `field-city` comme attestation —
or le catalogue porte `Town/city` **depuis le correctif de la passe 1**, et le commentaire qui
l'explique est trois lignes plus haut dans ce même fichier. Le correctif avait été appliqué au
catalogue et **non propagé** au document.

⚠️ **La conséquence dépasse cette story.** L'epic pose la partie A du glossaire comme **« non
négociable en story de rollout »**. Un rollout 23-2 à 23-6 suivant le glossaire aurait écrit `city`
là où le pilote a écrit `Town/city` — **la divergence terminologique que le glossaire existe pour
empêcher, semée par le pilote lui-même**. C'est la réalisation du risque `R3` de l'epic.

**Le second HIGH explique le premier.** Les passes 1 et 2 déclaraient « les 7 critères tenus » — il y
en a **8**. Un décompte de couverture d'audit n'est pas décoratif : c'est la **liste de ce qui a été
regardé**. Le critère non compté était `AC11-quater`, « le glossaire est cohérent après promotion » —
le seul en défaut. *Compter faux ses critères, c'est en laisser un hors champ sans le savoir.*

**Le troisième HIGH : un nombre juste, un périmètre absent.** « Le moissonneur rend, **sur le dépôt**,
274 / 5 / 7 » — sur le dépôt il rend **254**. 274 est exact contre les catalogues d'**avant** la
story ; les vingt clés du pilote y étant désormais entrées, il ne les moissonne plus. `AC10` désigne
274 comme valeur de contrôle : un auteur de rollout aurait lu une régression du moissonneur là où il
n'y a que la story qui travaille. La règle `274 − (clés résorbées)` est maintenant écrite.

**Deux MEDIUM sur le code, et le premier est encore une régression de la passe précédente** —
sixième d'affilée :

- **le silence rouvert.** L'invariant « écarté du fragment ⇒ recensé dans `aEchapper` » tenait
  exactement avant la passe 2, qui a ajouté un **second motif d'écartement sans canal de
  signalement**. Une clé mal formée disparaissait de **tous** les canaux, et l'en-tête la comptait
  encore : « 2 clés » pour un fragment d'une ligne. Le commentaire que la passe 2 avait écrit à cette
  ligne — *« cf. `aEchapper` »* — était faux pour la moitié des cas qu'il commentait.
- **le vide passait la garde.** `estFtlSain('')` rendait `true`, alors que Fluent rejette `cle = `
  (`ExpectedMessageField`). La garde qui existe pour empêcher qu'une ligne emporte la locale
  laissait passer **exactement la ligne qui l'emporte**.

**Et un LOW qui a coûté trois rédactions**, ce qui en dit long : compter les accolades ne suffit pas —
`a {} b`, `JSON {"a": 1}`, `code {1,2}` sont **appariés et rejetés** par Fluent. La première
rédaction ne validait que l'**amorce** du placeable ; la deuxième, qui validait le contenu entier,
**rejetait une entrée légitime de `fr-CH`** (`{"{"}` est l'échappement Fluent d'une accolade, et le
catalogue s'en sert) ; la troisième traverse les littéraux de chaîne caractère par caractère.
⚠️ **Trouvé en passant les 5001 entrées des quatre catalogues à la garde, pas en raisonnant.**

⚠️ **Un correctif de la lentille était à moitié faux, et le gate l'a dit.** Elle qualifiait le
`?? chemin` du basename de « code mort suggérant une défense qui n'existe pas ». Inatteignable à
l'exécution : **vrai**. Inutile : **faux** — `.pop()` est typé `string | undefined` et `npm run check`
refusait sans lui. Le retirer sec faisait rougir le gate. L'indexation par la longueur dit la même
chose sans mentir sur une défense inexistante.

**Les quatre gardes neuves sont vérifiées par mutation** — chacune tue un test : basename remplacé
par le chemin entier, vide accepté, placeable non validé, `aEchapper` privé du motif de clé.

**Ce que la passe a CONFIRMÉ, recompté depuis la source** : 655 tests et 29 `kesh-i18n` **rejoués et
exacts** ; allowlist 295 → 275 dont le `diff` rend **exactement** les 20 clés du pilote ; 20 × 4 = 80
lignes ajoutées, **0 supprimée** ; 57 clés de parité inchangées ; glossaire 52 / 12 ; les **vingt
replis comparés octet à octet** et identiques ; les **29 minuscules** exactes ; **aucun `ß`** en
`de-CH` ; les 8 lignes du tableau `AC11-ter` vérifiées au `grep` ; `individual` **entièrement**
propagé. Et — le geste que la story fait le mieux — **aucune déclaration d'exécution des E2E nulle
part**, avec le paragraphe qui dit pourquoi.

### Gates — exécutés, non déclarés

| gate | résultat |
|---|---|
| backend complet (`test-fast.sh`, base **remise à zéro d'abord**) | **2219/2219**, 4 ignorés, 89,7 s |
| `cargo test -p kesh-i18n` | 29/29 — **garde A verte avec les 80 entrées neuves**, écart de parité toujours à 57 |
| `npm run check` | **0 erreur** |
| `npm run lint-i18n-ownership` | PASS — *avec la ligne substituée* |
| `npm run test:unit` | **655/655** sur 70 fichiers |
| `npm run build` | vert |
| clause négative d'AC11 | `^delete *=` **absente des quatre** `.ftl` |

⚠️ **E2E non exécutée.** La story touche un `.svelte` de production (une clé renommée) et l'écran
d'onboarding change d'affichage en langue non française — mais **aucune spécification E2E ne
sélectionne sur ces libellés** ni ne s'exécute en locale non française : la suite ne pourrait rien
en dire. C'est écrit plutôt que sous-entendu.
