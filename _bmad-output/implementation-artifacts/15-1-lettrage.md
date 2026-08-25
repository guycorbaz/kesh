# Story 15.1 : Lettrage — savoir ce qui reste ouvert

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni décisions, ni critères, ni tâches.** Elle
ne garde que les pointeurs vers ses trois moitiés et l'historique des passes qui ont conduit
au découpage. *(La définition du statut `split` l'impose, et le précédent de la Story 17-2 —
démêlé quatre passes durant — dit ce que coûte un corps complet laissé derrière un split.)*

## Les trois sous-stories

| | fiche | ce qu'elle porte |
|---|---|---|
| **15-1a** | `15-1a-socle-lettrage.md` | **Socle** : la marque, sa portée société, son unicité sous concurrence, et son **cycle de vie face aux chemins d'écriture existants** — c'est là que tombent les deux HIGH de la passe 3 |
| **15-1b** | `15-1b-vue-lignes-ouvertes.md` | **La vue « ce qui reste ouvert »** et sa relation aux dispositifs qui lisent déjà `paid_at` — balance âgée, relances, et les deux écritures d'une facture fournisseur |
| **15-1c** | `15-1c-proposition-ecran.md` | **Le moteur de proposition et l'écran**, avec la frontière énoncée pour l'utilisateur |

⚠️ **L'ordre n'est pas indifférent** : 15-1a est un **socle** que les deux autres supposent.
15-1b lit la marque que 15-1a pose ; 15-1c la pose depuis un écran.

## Pourquoi le découpage

La spécification a subi **trois passes de `validate` sans converger** :

| passe | modèle | sévérité maximale |
|---|---|---|
| relecture d'auteur (hors protocole) | Opus | HIGH |
| passe 1 | Sonnet | HIGH |
| passe 2 | Haiku | MEDIUM |
| **passe 3** | **Opus** | **HIGH** ⛔ |

La remontée `MEDIUM → HIGH` déclenche le **critère de non-convergence** de la § *Règle de
splitting préventif* — qui vise la sévérité qui stagne ou régresse, et non la durée. Décision
prise par le Project Lead le 2026-08-25.

⚠️ **Ce que les trois passes ont appris, et qui vaut au-delà de cette story** : les passes 1
et 2 ont relu la spec **contre elle-même et contre le schéma**. Aucune n'a suivi les **chemins
de code qui écrivent, effacent ou soldent** les lignes que le lettrage prétend porter — et
les quatre HIGH de la passe 3 étaient tous là.

## Décisions restées ouvertes au moment du split

Elles sont reportées **dans la sous-story qui les porte**, avec leurs conduites possibles :

| décision | fiche |
|---|---|
| Le porteur de la marque : colonne `lettering_code` ou table `letterings` | 15-1a |
| Le sort de la marque quand une écriture est **modifiée** | 15-1a |
| Ce que lettrer change pour la **balance âgée** et les **relances** | 15-1b |
| Ce que la vue **prétend égaler** — le solde du compte, ou autre chose | 15-1b |
| La portée de la garantie d'immuabilité sur exercice clos | 15-1b |
| La **tolérance de montant** face aux frais bancaires | 15-1c |
| La **fenêtre de dates**, qui écarte aujourd'hui la contre-passation | 15-1c |

## Change Log

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **LA SÉVÉRITÉ REMONTE : `MEDIUM` (passe 2) → `HIGH` (passe 3).** 4 HIGH, 4 MEDIUM,
3 LOW. **Le critère de non-convergence de la § *Règle de splitting préventif* est
DÉCLENCHÉ** — il vise précisément la sévérité qui stagne ou régresse, et non la durée. Le
découpage est proposé plus bas ; **l'arbitrage revient au Project Lead**, pas à
l'orchestrateur.

**Pourquoi cette passe trouve ce que deux autres n'ont pas vu**, et la leçon vaut au-delà de
cette story : les passes 1 et 2 ont relu la spec **contre elle-même et contre le schéma**.
Aucune n'a suivi les **chemins de code qui écrivent, effacent ou soldent** les lignes que le
lettrage prétend porter. Tout est là.

**Deux findings mettent en cause les patches des passes précédentes — corrigés dans ce
commit, car ils ne demandent aucun arbitrage :**

⛔ **P3-5 (MEDIUM) — le moyen d'unicité inscrit en passe 2 était impossible ET cassait AC1.**
La passe 2 proposait une contrainte `UNIQUE (company_id, lettering_code)`. Or
`journal_entry_lines` **ne porte aucun `company_id`** (vérifié : zéro occurrence dans les
migrations ; le scoping passe par jointure, `journal_entries.rs:1226` le documente). Et même
portée par jointure, cette contrainte **interdirait ce qu'AC1 exige** — deux lignes lettrées
ensemble portent le *même* code. C'est une régression franche du patch précédent, exactement
le motif que la § A9 du `CLAUDE.md` a codifié ce matin.

⛔ **P3-11 (LOW) — la référence « corrigée » en passe 1 était encore fausse.** P1-4 avait
remplacé `1096-1206` par `1003-1328` : `accept_one_invoice` finit à **1313** (vérifié à
l'`awk` sur les accolades de colonne 1) ; la ligne 1328 tombe dans le doc-comment
d'`accept_one_split`. **Corriger une valeur fausse par une autre valeur fausse**, puis la
propager aux trois sites, est le mode d'échec que la § *Recompter ses propres comptes rendus*
décrit. Le chemin de crate manquait aussi — deux fichiers s'appellent `reconciliation.rs`.

**Les quatre HIGH, qui appellent des DÉCISIONS et non des précisions — non patchés :**

| | défaut | preuve au sol |
|---|---|---|
| **P3-1** | **Modifier une écriture détruit ses lignes** : `update` fait `DELETE FROM journal_entry_lines` puis réinsère (`journal_entries.rs:981`, dans `update` l. 805). Le code de lettrage est perdu, la contrepartie reste lettrée donc **réputée soldée**. AC12 n'a tranché que la *suppression* | le `DELETE` et la fonction, vérifiés |
| **P3-2** | **La garde d'AC12 est posée au mauvais endroit** : AC12 cite la route `journal_entries.rs:631`, mais `invoices::delete` efface aussi l'écriture via `journal_entries::delete_in_tx` (`invoices.rs:1338`), et **aucune de ses trois gardes ne regarde le lettrage** | le second appelant, vérifié |
| **P3-3** | **Lettrer ne calme ni la balance âgée ni les relances** : toutes deux ne lisent que `paid_at` (`aged_receivables.rs:127`, `dunning_eligibility.rs:87`) et `lettering` a **0 occurrence** dans tout le dépôt. On relancera un débiteur soldé — le défaut n'est pas muet, **il est adressé au client** | les deux SQL + le grep, vérifiés |
| **P3-4** | **Le § *Contexte* est FAUX sur les fournisseurs** : ils ont `pay()`, un `settlement_type IN ('bank_transfer','internal_account')` — donc **le règlement hors import existe déjà** — et un `settlement_journal_entry_id` dédié. Une facture fournisseur payée porte **DEUX** écritures ; AC3-bis, qui dit « un test par table », en masque une et **laisserait la ligne de règlement ouverte à jamais** | migration `20260628000001`, l. 44-68 |

**Quatre MEDIUM** : P3-5 (corrigé ci-dessus) ; **P3-6** — `paid_at` n'a aucune contrepartie
comptable côté client (`invoices.rs:1923` : « **Ne crée AUCUNE écriture comptable** »), donc
la vue ne peut pas « justifier le solde » comme le promet le *so that* ; **P3-7** — la
fenêtre de 30 jours **tue le cas de contre-passation qu'AC4 exige nommément**, l'écriture
d'annulation étant datée du jour ; **P3-8** — le canal `paid_at` n'a **aucune garde
d'exercice**, si bien qu'un dé-marquage rouvre une ligne sur un exercice clos, ce qu'AC7
interdit par l'autre canal.

**Trois LOW** : P3-9 (l'export CSV a un header figé sans garde d'exhaustivité pour
`journal_entry_lines`), P3-10 (l'exemption P7 demandée par T1 est **inutile** — le détecteur
ne trie que les migrations qui écrivent des données, un `ADD COLUMN` n'est jamais atteint),
P3-11 (corrigé).

**Pistes réfutées** : multi-devise et arrondis (non rouverts, rien ne les contredit),
l'allowlist i18n vide (**vraie**, `i18n-keys.test.ts:432`), le rejeu de backfill à l'import
(sans objet), l'exercice rouvert (`fiscal_years::reopen` existe et ne casse pas AC7 — il en
est la soupape), et D3 contre un verrou global (faux positif confirmé pour la deuxième fois).

**Verdict : la spec n'est pas prête, et le défaut n'est plus de la précision mais de la
DÉCISION.** Trois questions restent ouvertes, chacune produisant un résultat faux et
silencieux si elle est laissée au développeur : le sort du code face aux chemins d'écriture
existants (P3-1, P3-2) ; ce que le lettrage change pour la relance et la balance âgée
(P3-3) ; et ce que la vue prétend égaler, sachant que client et fournisseur **ne sont pas
symétriques** dans le code existant (P3-4, P3-6).

**Découpage proposé, à arbitrer** : **(a)** la colonne, sa portée et son cycle de vie face
aux chemins d'écriture existants — socle que les deux autres supposent ; **(b)** la vue « ce
qui reste ouvert » et sa relation aux dispositifs qui lisent déjà `paid_at` ; **(c)** le
moteur de proposition et l'écran.

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**La sévérité décroît : `HIGH → MEDIUM`.** Aucun HIGH, un MEDIUM retenu, quatre LOW dont
deux réfutés au sol. C'est une convergence **monotone**, pas une stagnation — le critère de
split de la § *Règle de splitting préventif* n'est donc pas déclenché.

**Ce que la passe cherchait en priorité** : les régressions de la passe 1, le motif mesuré du
dépôt étant que sept passes sur huit trouvent un défaut du patch précédent. Elle a vérifié
les quatre patches un par un et **n'en a trouvé aucune** — la séparation critères
structurels / filtres facture tient, la tolérance est bien retirée des deux côtés, et les six
décisions D1-D6 sont désormais toutes portées par au moins un critère.

| | défaut | gravité | remède |
|---|---|---|---|
| **P2-1** | **AC12 disait « à trancher à l'implémentation »** — deux développeurs auraient produit deux comportements opposés sur le même critère | MEDIUM | **tranché : la suppression est REFUSÉE**, le délettrage automatique contournerait AC7 par un chemin détourné |
| **P2-3** | **T6 omettait AC4 et AC13** — une liste de tests « prioritaires » incomplète se lit comme une couverture suffisante | LOW | les deux ajoutés |
| **P2-4** | **AC13 confondait deux exigences** : le texte visible et le `data-testid`. Un `data-testid` ne satisfait pas D6 — il est invisible | LOW | scindée en (1) et (2) |
| **P2-5** | **AC11 muette sur l'unicité sous CONCURRENCE** — deux lettrages simultanés pouvaient recevoir le même code | LOW | exigence ajoutée, moyen laissé libre |

**Deux findings RÉFUTÉS au sol, et la réfutation vaut d'être écrite :**

⚠️ **P2-2 (multi-devise) — réfuté.** La passe reprochait à la spec d'être muette sur la
comparaison de montants en devises différentes. Vérification : **`journal_entry_lines` ne
porte AUCUNE colonne de devise** — `debit` et `credit` sont des `DECIMAL(19,4)` nus
(`20260412000001_journal_entries.sql`) — et le PRD ne contient pas une occurrence de
« devise » ni de « currency ». Les seules colonnes `currency` du schéma sont dans les tables
d'**import** (`bank_transactions`, `imported_supplier_invoices`) : elles décrivent la donnée
entrante **avant** son écriture comptable. La comptabilité de Kesh est mono-devise ; deux
lignes d'écriture sont par construction dans la même unité. **Le cas n'existe pas.**

⚠️ **P2-6 (arrondis) — réfuté par le même relevé.** `DECIMAL(19,4)` est un type **exact**,
pas un flottant : l'égalité stricte entre deux montants stockés est exacte. Le scénario
avancé — « 1000.004 arrondi à 1000.01 » — suppose une précision que la colonne n'admet pas.

Ces deux réfutations coûtent le temps de la vérification et l'épargnent au développeur : un
patch appliqué sur un défaut inexistant aurait ajouté à la spec une règle de change qu'aucun
schéma ne permet d'implémenter.

La spec reste à **15 critères** (les remèdes amendent, ils n'ajoutent pas). **Verdict :
convergence en cours, passe 3 due** — un MEDIUM subsiste au moment où la passe a rendu son
rapport, et la § *Review Iteration Rule* impose de relancer tant qu'un finding dépasse LOW.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**Première passe au sens de la § *Review Iteration Rule*** : contexte frais et modèle
distinct de l'auteur de la spec (Opus). C'est ce que la relecture de la veille, menée par cet
auteur, ne pouvait pas offrir.

**Quatre findings, tous vérifiés au sol par l'orchestrateur avant application** — la
discipline grep ground-truth vaut pour tous les modèles, pas seulement pour Haiku.

| | défaut | gravité | remède |
|---|---|---|---|
| **P1-1** | **D5 reprenait les critères de la réconciliation en bloc**, dont `status = 'validated'` et `paid_at IS NULL` — deux filtres propres à `invoices`. Ils excluent **trois des quatre cas** que la story existe pour couvrir : fournisseurs (`supplier_invoices`), écritures manuelles (aucune facture), et facture annulée par avoir (`status = 'cancelled'`). Et `journal_entry_lines` ne porte **aucun** `invoice_id` : la jointure supposée n'existe pas | **HIGH** | D5 sépare les critères **structurels** (transposables) des **filtres facture** (hors sujet) ; **AC4** exige un test nommé par cas |
| **P1-2** | **D5 et AC6-bis se contredisaient sur le montant** : tolérance de 5 centimes en proposition, égalité stricte en validation. Le système aurait **proposé ce qu'il refuse** | **HIGH** | égalité stricte des deux côtés ; ⚠️ la conséquence (frais bancaires jamais lettrables) est **nommée et laissée à l'arbitrage** |
| **P1-3** | **D6 n'était nommée par aucun critère** — elle ne vivait que dans sa section et dans T5 | MEDIUM | **AC13** |
| **P1-4** | `accept_one_invoice` cité `1096-1206` ; la fonction va de **1003 à 1328**, et l'`UPDATE … paid_at` qu'elle invoque est **hors** de la plage citée | LOW | référence corrigée ici **et aux deux autres sites** qui la copiaient |

⚠️ **P1-3 est la sixième récidive du même geste** — une décision que ne porte aucun critère.
La relecture de la veille avait corrigé ce défaut **pour D1** et l'avait laissé **pour D6**,
dans le même fichier et le même passage. **Corriger un site n'est pas corriger le symptôme** :
c'est exactement ce que la § *Propagation post-patch* du `CLAUDE.md` demande de greper, et le
grep n'avait pas été fait sur « quelle décision n'est citée par aucun AC ».

**Ce que la passe dit de la relecture précédente** : ses quatre défauts tenaient tous, aucun
n'a été réfuté. Mais elle avait cherché ce qui **manquait** à la spec, non ce qu'elle
**affirmait à tort** — et les deux HIGH d'aujourd'hui sont l'un et l'autre des affirmations
fausses sur du code existant, qu'aucune relecture d'auteur ne remet en cause puisqu'elle les
a écrites.

**Un seul faux positif écarté** (D3 contre un verrou global de clôture) : la garde
`FiscalYearClosed` est posée par fonction, pas par déclencheur — D3 reste réalisable.

La spec passe de **14 à 15 critères**. **Verdict de la passe : non prête pour le
développement.** Une passe 2, modèle différent et contexte frais, est due.

### Relecture critique — 2026-08-25

⚠️ **Ce n'est PAS une passe de `bmad-create-story validate` au sens de la § *Review
Iteration Rule***, et le dire importe : elle a été menée **par l'auteur de la spec**, sans
contexte frais ni modèle distinct. Elle n'offre donc **aucune protection contre le biais
d'auteur** — c'est précisément ce que la rotation des modèles existe pour couvrir. **Une
passe adversariale reste due avant tout développement.**

Elle a néanmoins trouvé **quatre défauts, tous vérifiés au sol** :

| | défaut | gravité |
|---|---|---|
| **F1** | Le **code de lettrage** n'était engendré par rien : ni format, ni portée. ⚠️ Un compteur non scopé serait un défaut de multi-tenant — le dépôt en a déjà payé un (KF-002) → **AC11** | MEDIUM |
| **F2** | **Rien n'imposait l'égalité des montants.** D2 exclut le partiel : lettrer 1000 avec 300 prétendrait qu'une créance est soldée et **ferait mentir la vue qui est tout l'objet de la story** → **AC6-bis** | **HIGH** |
| **F3** | D4 définissait « ouvert » via `invoices.paid_at` seul, alors que **D1 met les fournisseurs dans le périmètre** et que `supplier_invoices` est une **table distincte** avec son propre `paid_at` (vérifié au sol). **La moitié du périmètre annoncé** aurait affiché des factures réglées comme ouvertes → **AC3-bis** | **HIGH** |
| **F4** | `delete_journal_entry` **existe** (`journal_entries.rs:631`). Supprimer une écriture lettrée laissait sa contrepartie **réputée soldée** : la facture quittait la vue des ouverts **en restant impayée** — défaut muet qui fausse le solde sans rien signaler → **AC12** | MEDIUM |

**Et un défaut de traçabilité** : **D1 n'était cité par aucun critère**. AC1 le portait sans
le nommer — or *un développeur lit le critère et la case à cocher, pas la décision*. C'est
la cinquième récidive de ce geste dans le dépôt ; AC1 porte désormais la mention.

⚠️ **Ce que cette récolte dit de la spec initiale** : ses six décisions étaient justes, et
les deux pièges qu'elle nommait le sont toujours. Ce qui manquait n'était pas du
raisonnement mais des **conditions de véracité** — l'égalité des montants, la seconde table
de factures, la portée du code, le sort d'une contrepartie supprimée. **Trois des quatre
défauts font mentir la vue**, c'est-à-dire attaquent exactement ce que la story existe pour
produire.
