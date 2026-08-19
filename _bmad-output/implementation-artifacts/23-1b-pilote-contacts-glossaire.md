# Story 23.1b : Le pilote — vingt clés, et la terminologie qu'elles engagent

## Status

ready-for-dev

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
**Dehors aussi** : les 297 autres clés (rollouts 23-2 à 23-6), les 5 entrées Fluent à variables de
`TransactionSplitModal` (23-5), [#255], [#314], [#242].

⚠️ **Les identifiants de décisions et de critères sont ceux de la 23-1 d'origine.** Ceux qui
manquent ici vivent dans la 23-1a — en particulier **D5-bis** (exclusion des `.test.*`) et
**AC7-bis** (l'extracteur robuste), dont le moissonneur de D6 réutilise le lecteur de littéral.

## Chiffres de référence

Recomptés le 2026-08-19 et confirmés par trois passes de revue indépendantes. **Le tableau complet
et les commandes de recompte sont dans la 23-1a, § *Chiffres de référence*** ; ne sont repris ici
que ceux dont cette story se sert.

| | valeur |
|---|---|
| clés du pilote `contacts` | **20** — 12 sous `lib/features/contacts`, 8 sous `routes/(app)/contacts` |
| clés à repli moissonnable (tout le dépôt) | **à recompter avec les relais** — 245 sur 250 avant leur prise en compte, cf. 23-1a § D4-bis |
| clés sans repli littéral | 5, toutes dans `TransactionSplitModal` — **hors périmètre**, portées par la 23-5 |
| clés à repli **divergent** | **7** — valeur de contrôle datée, le moissonneur la calcule |
| partie B du glossaire | **16** entrées, dont **3** promues par cette story → **13** ensuite |

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

**D8-bis — Le découpage par dossier FUIT, et huit clés du pilote sont partagées avec un dossier
hors périmètre.** *(Constat de cadrage, non exigence : aucun AC ne le contrôle.)*

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
choisis pour le carnet d'adresses**, sans qu'aucune revue ne les ait regardés dans ce contexte. En
l'espèce ils conviennent (« Localité », « Rue », « NPA », « Prénom » sont des étiquettes de champ
d'adresse, identiques dans les deux écrans), **mais c'est un constat à vérifier, pas une évidence** :
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

11. **AC11** — Les **20 clés du domaine `contacts`** existent dans les **quatre** locales, sont
    retirées de l'allowlist de dette, et leurs libellés `de-CH` / `it-CH` / `en-CH` respectent la
    partie A du glossaire et le registre de D9. La clé `delete` est entrée sous le nom
    `contact-persons-delete` (D7-bis), son site d'appel mis à jour.

11-bis. **AC11-bis** — `docs/i18n-glossaire.md` est mis à jour : **`localité`, `prénom` et
    `personne de contact` passent de la partie B à la partie A**, chacun avec la clé du pilote qui
    l'atteste désormais. *Sans cet AC, D8 promet une promotion que rien n'exige et que les 13
    autres critères laisseraient passer.*

13. **AC13** — Les deux gates complets passent : backend (`cargo test --workspace`) et frontend
    (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`).

## Tasks / Subtasks

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
  - [ ] **Promouvoir `localité`, `prénom` et `personne de contact` en partie A** de `docs/i18n-glossaire.md`, avec la clé qui les atteste
  - [ ] **Recompter la partie B (13 entrées après retrait des 3 promus)** et **réécrire le paragraphe qui l'accompagne** — il est au futur (« trois de ces termes sont tranchés… les treize autres resteront ouverts ») et doit passer au passé, la promotion étant faite. ⚠️ *Ne pas chercher la chaîne « douze autres » : elle a déjà été corrigée en « treize » à la passe 3, et la tâche pointait encore dessus.*
  - [ ] **Mettre à jour le commentaire de `lint-i18n-ownership.js:103`**, qui cite `delete` parmi les « clés génériques » — la substitution de la ligne `:112` le laisse périmé (§ *Propagation post-patch*)
  - [ ] **Relire les huit libellés `field-*` DANS LES DEUX CONTEXTES** — carnet d'adresses et étape « Coordonnées » de l'onboarding (D8-bis)

- [ ] **T6 — Gates** (AC13)
  - [ ] Gate backend complet, gate frontend complet, avant tout push

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
| `frontend/scripts/lint-i18n-ownership.js:104-118` | les neuf clés sœurs déjà inscrites, et la ligne `:112` à substituer |
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

