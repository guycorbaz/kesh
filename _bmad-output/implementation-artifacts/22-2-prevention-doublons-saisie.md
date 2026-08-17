# Story 22.2 : Prévenir les doublons à la saisie d'un contact — UMBRELLA, DÉCOUPÉE

## Status

split

⛔ **NE PAS IMPLÉMENTER CE DOCUMENT.** Son corps a été **retiré** — décisions, critères d'acceptation, tâches et décompte des preuves vivent désormais dans les deux sous-stories ci-dessous, et **elles seules font foi**.

⚠️ **Pourquoi le corps a été retiré plutôt que laissé en place** : le vestige `17-2` est resté `ready-for-dev` avec un corps complet après son propre découpage, et la Story 22-4 a dû le démêler quatre passes durant. Un document qui décrit un travail déjà réparti ailleurs ne se périme pas bruyamment — il se contente de diverger.

## Le découpage

| Sous-story | Périmètre | Gate |
|---|---|---|
| **[22-2a](22-2a-socle-appariement-contacts.md)** — le socle | le **module pur** et lui seul : normalisation du terme, seuil, classement, exclusion de soi, vérification de l'IDE. Aucun `.svelte`, aucune clé i18n, aucun appel réseau. | `vitest`, quelques secondes |
| **[22-2b](22-2b-surface-prevention-doublons.md)** — la surface | les deux sondes, la temporisation, le balisage et `aria-live`, l'i18n ×4, l'extension backend de `push_where_clauses`, l'E2E, le manuel. | composant + E2E + gate backend complet |

**Ordre SÉRIE : 22-2a d'abord.** La 22-2b importe le module de la 22-2a ; commencée avant, elle réécrirait la logique qu'elle est censée consommer, et le découpage n'aurait servi à rien.

⚠️ **Les deux moitiés se mergent dans UNE SEULE PR**, qui porte `closes #301` — la 22-2a seule ne livre rien de visible, la 22-2b seule ne compile pas.

## Pourquoi ce découpage

Quatre passes de `bmad-create-story validate`. Trend `9 → 11 → 20 → 6`, sévérité maximale `1C → 0C → 2C → 3C`. Le critère de non-convergence de la § *Règle de splitting préventif* s'est déclenché **deux passes consécutives**. Arbitrage de Guy le 2026-08-17, question **Q5**.

**Le diagnostic n'est pas « la story est trop large »** — elle touche quatre modules, sous le seuil de cinq. C'est que **le mécanisme et sa surface ne se relisent pas avec la même lentille**, et que chaque tour de correctifs réinjectait des défauts au sommet de l'échelle : la passe 2 a trouvé les défauts des patches de la passe 1, la passe 4 ceux du patch de la passe 3. Les trois `CRITICAL` de la dernière passe tenaient tous en trois lignes de logique pure — un ordre d'instructions inversé, un compteur mal soustrait, une preuve écrite dans le mauvais langage — et **aucun n'avait besoin d'un navigateur, d'une base ou d'un serveur pour être attrapé**. Ils vivent désormais dans la 22-2a, dont le gate coûte quelques secondes.

## Ce que ce document conserve, et pourquoi

**Son Change Log intégral** — les quatre passes de revue, leurs trends, leurs convergences, leurs réfutations. C'est la **source de toutes les décisions** reproduites dans les deux sous-stories, et le seul endroit où l'on peut lire *pourquoi* chacune est écrite comme elle l'est. Les deux sous-stories y renvoient nommément.

C'est aussi, accessoirement, le dossier le plus instructif du dépôt sur la limite d'une revue par lecture : trois passes n'ont trouvé que des défauts de rédaction ; la quatrième a exécuté la requête et montré que le mécanisme ne faisait pas le travail.

## Change Log

### Passe 1 de `bmad-create-story validate` — 2026-08-17, Sonnet ×3, contexte frais

Trois lentilles orthogonales (BlindHunter, EdgeCaseHunter, AcceptanceAuditor), lancées en parallèle sur contexte frais, ground-truth obligatoire. Bruts : 1 + 5 + 4 = 10 findings. **Après dédoublonnage : 1 CRITICAL / 2 HIGH / 6 MEDIUM / 0 LOW — 9 findings > LOW, 9 correctifs appliqués, 0 écarté, 0 faux positif.**

**Le finding du jour est une CONVERGENCE de deux lentilles indépendantes** : AA-1 (`CRITICAL`) et ECH-1 (`HIGH`) décrivent le même défaut — **la sonde IDE n'excluait pas le contact en cours d'édition**. `openEdit` pré-remplit `formIde` avec l'IDE du contact lui-même (`+page.svelte:256`), si bien que l'avertissement **franc** — celui auquel on demande à l'utilisateur de faire confiance — se serait déclenché sur sa propre fiche, à l'ouverture, sur le chemin le plus banal du formulaire. J'avais écrit la garde pour la sonde nom (AC7) et **je ne l'avais pas propagée à la sonde IDE** : les deux sondes ne partagent pas leur code, et un raisonnement ne se propage pas tout seul. La sévérité `CRITICAL` de l'AcceptanceAuditor est retenue plutôt que le `HIGH` de l'EdgeCaseHunter — pas par prudence, mais parce que le défaut touche le signal fort et se déclenche **sans action de l'utilisateur**.

Les huit autres, par ordre de sévérité :

| # | Sév. | Défaut | Correctif |
|---|---|---|---|
| AA-2 | HIGH | AC3 (« ignorer l'avertissement crée bien le contact ») **contredit formellement** la preuve 4 d'AC2 (« le 409 reste levé ») — sur le signal franc, le contact ne PEUT pas être créé. Le test E2E prescrit était impossible à écrire. | AC3 recentrée sur *l'interface ne bloque pas* ; la preuve E2E est restreinte au signal nuancé |
| AA-3 | HIGH | T3 et T4 **ne portaient aucune sous-tâche de test** alors que T4 seule porte AC1, AC2, AC3, AC5 et AC7. Un dev cochait T4 après le seul balisage, sans qu'aucune preuve existe — et **aucun test de cette page n'existe** dans le dépôt (`find` → seul `contact-helpers.test.ts`). | sous-tâches de test ajoutées à T3 et T4, fichier `+page.test.ts` nommé ; le décompte annoncé ici (« 8 ») était **faux dès son écriture** et a été refait en passe 2 — cf. § *Décompte des preuves* |
| BH-1 | MED | Le patron cité pour la branche `escaped.is_empty()` **ne l'exerce pas** : `test_search_handles_special_chars` cherche `"100%"`, et `%` n'est pas un opérateur BOOLEAN MODE (`util/search.rs:41`) — le terme survit et prend la branche `else`. **Aucun test du dépôt n'exerce cette branche.** | terme concret prescrit (`***`), et la fausse piste explicitement démontée |
| ECH-3 | MED | L'état des sondes n'était remis à zéro nulle part : le dialogue se ferme depuis **3 sites** et n'est jamais démonté. Une réponse armée avant « Annuler » s'affichait sur la fiche suivante. | **D14** — remise à zéro dans `openCreate()`/`openEdit()`, pas sur la fermeture |
| ECH-4 | MED | Rien n'imposait **deux** paires `(timer, compteur)`. Une paire factorisée fait qu'une réponse tardive de l'une invalide la fraîcheur de l'autre — un des deux avertissements ne s'affiche jamais, en silence. | **D13** + preuve 3 d'AC4, le test croisé |
| ECH-5 | MED | AC1 affirmait « archivés exclus » **sans aucune preuve**, alors que les deux appels ne diffèrent que par ce booléen et se lisent côte à côte. | preuve 3 d'AC1, symétrique de la preuve 2 d'AC2 |
| AA-4 | MED | AC5 attachait « aucune requête » à **deux** déclencheurs, dont l'un (« aucun contact proche ») ne peut être constaté qu'**après** avoir cherché — indécidable à la lettre. | AC5 scindée en clauses (a) et (b) |
| ECH-2 | MED | Une session expirée pendant la frappe redirige en **plein document** et efface la saisie ; les sondes rendent ce risque préexistant bien plus atteignable. | `try/catch` prescrit en T4 + risque **accepté explicitement** en Dev Notes |

**Ce que la passe n'a PAS trouvé, et qui compte autant** : aucune des dizaines de citations `fichier:ligne` de la spec n'était fausse — les trois lentilles les ont vérifiées indépendamment, y compris la mesure en direct de `innodb_ft_min_token_size` et l'asymétrie des deux contraintes d'unicité. Le ground-truth de la spec tient ; ce qui a cédé, c'est sa **couverture** — des raisonnements justes appliqués à une sonde et pas à l'autre.

⚠️ **Passe 2 requise** par la § *Review Iteration Rule* (1 CRITICAL + 2 HIGH). Rotation : **Haiku**, contexte frais — avec le garde-fou du `CLAUDE.md` § *Haiku-specific guardrails*, tout `CRITICAL`/`HIGH` affirmant une absence ou une présence sera ground-truthé au `grep -nF` avant d'être traité.

### Passe 2 de `bmad-create-story validate` — 2026-08-17, Haiku 4.5 ×3, contexte frais

Bruts : 3 + 8 + 3 = 14. Un doublon (BH2-2 ≡ AA2-2), une fusion (ECH2-6 + ECH2-7 traitent le même sujet), **un finding réfuté**. **Retenus : 0 CRITICAL / 1 HIGH / 9 MEDIUM / 1 LOW — 11 findings, 11 correctifs appliqués.**

**Trend : `1 CRIT / 2 HIGH / 6 MED` → `0 CRIT / 1 HIGH / 9 MED`.** La sévérité maximale **décroît** (CRITICAL → HIGH), donc le critère de non-convergence de la § *Règle de splitting préventif* n'est **pas** déclenché — mais il faut dire aussi ce qui monte : **les MEDIUM passent de 6 à 9**, et ce n'est pas du bruit.

**Le fil rouge de cette passe, et il est à ma charge : ce sont les correctifs de la passe 1 qui ont produit l'essentiel de ses findings.** Deux familles, toutes deux nommées dans le `CLAUDE.md` :

- **Trois décomptes faux, tous introduits par mes propres patches de passe 1** — le titre d'AC2 annonçait « quatre assertions » après que j'y en eus ajouté une cinquième ; celui des pièges muets « six » pour sept lignes ; le Change Log de la passe 1 « 8 tests » quand la tâche en dénombrait dix. C'est mot pour mot la § *Recompter ses propres comptes rendus* : **le compte rendu devient le lieu du défaut**. La réponse n'est pas de corriger trois nombres mais de **supprimer les compteurs épars** : un § *Décompte des preuves* unique porte désormais le total. ⚠️ **Cette phrase se terminait initialement par « et aucun autre passage n'énonce de total » — c'était FAUX**, six sites en énonçaient encore. La réforme avait corrigé les valeurs sans supprimer les sites, puis déclaré les sites supprimés : le mode d'échec de la § *Recompter ses propres comptes rendus*, appliqué au correctif censé le fermer. Relevé en passe 3, et la règle exacte est désormais écrite au § *Décompte des preuves*.
- **Quatre décisions ajoutées en passe 1 sans leur preuve** — D14 (remise à zéro à l'ouverture), le `try/catch` des sondes, le seuil dans le sens descendant, les transitions de validité de l'IDE. C'est la § *Propagation post-patch* : un patch qui pose une obligation sans poser le test qui la tient n'est pas terminé. Chacune a maintenant sa preuve **et sa mutation**.

Le seul `HIGH` relève de la même famille : **T6 ne prescrivait aucune assertion** — on pouvait cocher la tâche sur un fichier E2E qui monte le décor et n'affirme rien. C'est le mode d'échec du test muet, un cran au-delà de celui contre lequel la ligne du `testMatch` met en garde. T6 porte désormais ses deux assertions nommées, dont la preuve 4 d'AC2 (le `409`), qui n'était **rattachée à aucune tâche**.

Le reste, par famille : le seuil de trois caractères ne jouait qu'à la montée (AC5) ; les cas limites de composition du terme pour une `Personne` en cours de frappe n'étaient pas couverts, alors que l'un des deux champs est presque toujours vide à ce moment (T2) ; AC7 ne disait pas sur quel type de contact porte sa preuve — précisé, avec la raison pour laquelle **un seul type suffit ici** (la garde est `c.id !== editing?.id`, qui ne consulte ni le type ni le terme).

**Un finding RÉFUTÉ, au grep** : ECH2-8 affirmait qu'un collage échapperait à la temporisation faute d'événement clavier. Vérifié — `grep -nF "oninput"` : les deux champs de recherche du dépôt sont liés à **`oninput`** (`ContactPicker.svelte:127`, `+page.svelte:463`), qui se déclenche sur un collage ; le `onkeydown:133` de `ContactPicker` ne pilote que la navigation dans la liste. Le patron que la spec fait copier traite donc le cas par construction. Le finding est écarté — mais il a produit une garde utile : T3 dit maintenant explicitement de brancher sur `oninput` et **jamais** sur `onkeydown`, parce qu'un dev qui l'inverserait resterait muet sur le geste le plus courant de tous.

**Ce que la passe 2 n'a PAS trouvé** : aucune fausseté factuelle, aucune régression des correctifs de la passe 1, aucune citation `fichier:ligne` erronée — les trois lentilles Haiku ont re-vérifié indépendamment les ancres du code, et **aucune hallucination du mode d'échec documenté au `CLAUDE.md`** n'est survenue. Le mécanisme spécifié n'a été mis en défaut par aucune des six lentilles des deux passes ; ce qui cède, passe après passe, c'est la **tenue du document**.

⚠️ **Passe 3 requise** par la § *Review Iteration Rule* (1 HIGH > LOW). Rotation : **Opus**, contexte frais.

### Passe 3 de `bmad-create-story validate` — 2026-08-17, Opus ×3, contexte frais

**⛔ AUCUN CORRECTIF N'A ÉTÉ APPLIQUÉ. La boucle est SUSPENDUE et attend un arbitrage de Guy.** Ce qui suit explique pourquoi il serait faux de patcher et de relancer une passe 4.

Bruts : 5 + 11 + 12 = 28. Après dédoublonnage (trois convergences) : **2 CRITICAL / 5 HIGH / 13 MEDIUM / 5 LOW — 20 findings > LOW.**

**Trend : `9` → `11` → `20`, et la sévérité maximale REMONTE : `1 CRIT / 2 HIGH` → `0 CRIT / 1 HIGH` → `2 CRIT / 5 HIGH`.**
Le second critère de la § *Règle de splitting préventif* est **franchement déclenché** — « une passe `N+1` remonte une sévérité **égale ou supérieure** à la passe `N` ». Ici `HIGH → CRITICAL`.

**Mais le critère décrit mal ce qui s'est passé, et le diagnostic exact importe plus que le critère.** Les passes 1 et 2 ont **lu** la spécification. La passe 3 a **exécuté la requête**. Ce n'est pas une story trop large qui refuse de converger : c'est une revue qui atteint enfin la couche où le défaut habite. Les vingt findings ne sont pas vingt défauts de rédaction — trois d'entre eux disent que **le mécanisme choisi ne fait pas le travail**.

#### Les trois faits d'exécution, reproduits par l'orchestrateur sur une base jetable

Fixture : sept contacts, `MATCH … AGAINST` copié à l'octet depuis `push_where_clauses` (branche `else`), tri et fenêtre copiés depuis `list_by_company_paginated`.

**(1) ⛔ `CRITICAL` — le doublon exact est ÉVINCÉ de la fenêtre.** Carnet contenant `Jean Bernard`, `Jean Dupont`, `Jean Favre`, `Jean Martin`, `Jean Rochat`, `Jean Zwahlen`. L'utilisateur saisit la `Personne` « Jean Zwahlen » :

```
SELECT id,name FROM contacts WHERE company_id=1 AND active=TRUE
 AND MATCH(name) AGAINST('Jean Zwahlen*' IN BOOLEAN MODE) ORDER BY name ASC LIMIT 5;
→ Jean Bernard · Jean Dupont · Jean Favre · Jean Martin · Jean Rochat
→ total_reel = 6        ⚠️ « Jean Zwahlen » EST ABSENT
```

Trois faits se composent : `escape_boolean_ft` ne préfixe **aucun** token de `+` (sémantique **OU inclusif** — `search.rs:88-92` le dit en toutes lettres) ; le tri est **alphabétique** et il n'existe **aucun** tri par pertinence (`ContactSortBy` n'offre que `Name | CreatedAt | UpdatedAt`) ; la fenêtre est `limit: 5`, que la spec justifiait d'un « 5 suffit largement ». Résultat : **le dispositif est muet sur le seul cas pour lequel il existe**, et bavard sur cinq contacts sans rapport — c'est-à-dire qu'il réalise simultanément les deux échecs que la story se donnait pour but d'éviter. Les raisons sociales suisses commençant très souvent par un mot générique (`Garage`, `Fiduciaire`, `Sàrl`), ce n'est pas un cas de laboratoire.
⚠️ **Aucune des 22 preuves ne peut voir ce défaut** : les 12 preuves de composant passent par un `vi.mock` de l'API — la requête réelle n'est jamais exécutée.

**(2) `HIGH` — un nom composé d'un trait d'union rend la sonde totalement muette.**

```
AGAINST('CoopVaud*')  → 0        AGAINST('Coop*') → 1   (contrôle)
```

`-` est l'un des 10 opérateurs `BOOLEAN MODE`, et `escape_boolean_ft` le **supprime** au lieu de le remplacer par une espace. Retaper `Coop-Vaud` à l'identique produit `CoopVaud*`, qui ne matche aucun des deux tokens indexés. **Le signal de doublon le plus fort qui existe — le nom retapé au caractère près — ne remonte rien.** En Suisse, les noms de famille composés (`Müller-Weber`) rendent le cas massif.

**(3) `MEDIUM` — D7 est FAUSSE, et de ma main.**

```
AGAINST('Du*') → 1        AGAINST('Du') → 0
```

`innodb_ft_min_token_size = 3` gouverne les tokens **exacts** ; il ne gouverne **pas** les recherches par préfixe — or `push_where_clauses:205` appose inconditionnellement `*`. D7 affirmait « en dessous de trois caractères, `MATCH … AGAINST` ne rend **rien** », et en tirait que le seuil est « d'abord une contrainte du moteur ». **C'est renversé** : le seuil de trois caractères est *uniquement* une politique d'ergonomie — défendable, mais révisable, et D7 retirait cette latitude au décideur en la présentant comme forcée.
⚠️ **C'est exactement le mode d'échec que cette story hérite de la 22-1 et cite dans ses propres Dev Notes** : j'ai **mesuré la variable** puis **déduit** sa conséquence sans l'exécuter. Une mesure juste ne rend pas vraie l'inférence qu'on en tire.

#### Ce que les trois lentilles ont trouvé par ailleurs

Deux familles, et aucune n'est cosmétique :

- **Des preuves qui ne peuvent pas tomber sous la mutation qu'elles nomment.** `AA3-1` (`CRITICAL`) et `AA3-2` : les preuves d'archivage d'AC1 et AC2 sont formulées **fonctionnellement**, alors que `includeArchived` est un paramètre de query-string traité **par le serveur** — sous `vi.mock`, la mutation `includeArchived: true → false` laisse le test **vert**. Seule une assertion **sur l'argument** l'attrape, et la spec cite pourtant le doc-comment de `products-page.test.ts` qui énonce précisément cette leçon. `AA3-3` : **les deux assertions E2E de T6 sont vertes sur `main` aujourd'hui**, avant qu'une ligne de la story soit écrite — créer un contact fonctionne, un IDE dupliqué donne déjà `409` ; la mutation « n'implémenter aucune sonde » les laisse passer. `AA3-4` : les exigences d'**affichage** d'AC1 (nom, localité, numéro de client) et d'AC2 (« le message dit que le porteur est archivé ») n'ont **aucune** preuve — or D11 troque une dimension de requête contre cet affichage, et D12 en fait le seul recours de l'utilisateur.
- **Des prémisses tenues pour acquises faute d'exécution.** `BH3-4` : la requête réelle **n'utilise pas** l'index FULLTEXT — le `OR` avec les `LIKE` force un `range` sur `idx_contacts_company_name` (mesuré : 1,26 + 2,36 ms contre 0,09 ms en `MATCH` seul, sur 3016 contacts, **quatre** requêtes par pause de frappe, le `COUNT(*)` compris que `limit: 5` n'économise pas). D3 fondait tout sur « le carnet se cherche déjà par index FULLTEXT ». **#301 demandait explicitement d'instruire le coût par frappe** ; la spec y répondait par une prémisse fausse et aucun chiffre. `BH3-5` : **archiver ne libère pas l'IDE** (contrainte plate, D12) — le nettoyage prescrit par T6, copié de la 22-1 où il fonctionne pour le numéro de client, ne fonctionne pas ici ; et un IDE ne se tamponne pas à l'horloge, son dernier chiffre étant un checksum modulo 11. `AA3-8` : D10 et le § *Forme exacte des deux appels* donnent **deux expressions différentes** de la même condition, et celle de D10 relit le champ à l'arrivée de la réponse — `normalizeIdeForApi('')` rendant `null`, `c.ideNumber === null` désignerait **n'importe quel contact sans IDE**.

**Et une convergence qui change la question posée à Guy.** `BH3-3` ≡ `AA3-9`, vérifié par l'orchestrateur : `'CHE109322551' LIKE '%CHE-109.322.551%'` rend **0**. L'IDE est stocké **normalisé** sur 12 caractères ; le `LIKE` porte sur le terme **brut**. La « conséquence visible » que Q1 soumettait à l'arbitrage — « chercher `CHE-109.322.551` remontera son contact » — **est fausse**, et c'est précisément la forme qu'on lit sur une facture papier, le cas d'usage invoqué. La symétrie avancée avec la 16-3b est fausse elle aussi : `client_number` est stocké **tel que saisi**, séparateurs compris, ce qui est la raison pour laquelle le `LIKE` fragmentaire y fonctionne.

#### Ce que la passe 3 n'a PAS mis en défaut

Les ancres `fichier:ligne` — encore une fois, et cette fois re-vérifiées par trois lentilles Opus indépendantes. Le scoping multi-tenant. La garde D6 (compteur de génération plutôt qu'`AbortSignal`). D12, D13, D14. L'asymétrie des deux contraintes d'unicité. L'insensibilité aux accents du FULLTEXT (`Dubarde Sarl*` remonte bien `Dubarde SàRL` — le cas nommé par #301 est traité **par la collation**, pas par le mécanisme). **Ce qui a cédé, c'est ce que deux passes de lecture ne pouvaient pas voir : le comportement effectif de la requête.**

#### Pourquoi la boucle s'arrête ici

Patcher vingt findings et relancer une passe 4 reviendrait à corriger la forme des preuves d'un mécanisme dont on vient d'établir **qu'il ne fait pas le travail**. Les trois faits d'exécution ne s'amendent pas par une reformulation : ils appellent une **décision de conception**, et cette décision n'appartient pas à l'orchestrateur — c'est la lettre de la § *Tech debt management* (« l'arbitrage de sévérité est fait par le Project Lead au moment de la découverte ») et de la § *Règle de splitting préventif*.

⚠️ **Statut ramené à `draft`.** `ready-for-dev` serait une affirmation fausse tant que le mécanisme n'est pas tranché, et un agent de développement ne doit pas prendre cette story en l'état.

**Trois questions sont posées à Guy** — cf. § *Questions ouvertes*, Q1 amendée et Q3/Q4 neuves.

### Remédiation de la passe 3 — 2026-08-17, après arbitrage de Guy

**Les trois questions ont été tranchées, et les vingt findings appliqués.** Statut rétabli à `ready-for-dev`.

| Question | Arbitrage de Guy | Matérialisé par |
|---|---|---|
| **Q3** — l'appariement par nom | **fenêtre large + classement côté client** ; la correction à la source part en story distincte | **D15** — `limit: 20`, classement et coupe dans le module pur de T2, « et N autres » via `total` |
| **Q4** — le trait d'union | **normaliser le terme côté client** ; corriger `escape_boolean_ft` en story distincte, groupée avec Q3(b) | **D16** — les dix opérateurs remplacés par une espace, jamais supprimés |
| **Q1** — l'IDE cherchable | **étendre `push_where_clauses`** | T1 débloquée, avec le bénéfice de bord **rectifié** (seule la saisie sans séparateurs remonte) |

Trois décisions naissent de la remédiation : **D15** et **D16** ci-dessus, plus **D17** — *une proposition informe, elle ne navigue pas* : `openEdit` réassigne dix-neuf champs, donc un clic mal spécifié effacerait la saisie en cours sans confirmation. Et **D7 a été RÉÉCRITE** : le seuil de trois caractères est une politique d'ergonomie, pas une contrainte du moteur, avec **D7-bis** qui le porte sur le **plus long token** plutôt que sur la longueur du terme.

**Le décompte des preuves passe de 22 à 32**, recompté depuis les AC. L'ajout le plus important n'est pas le nombre mais **sa nature** : la preuve 6 d'AC1 est un `#[sqlx::test]`, et c'est la **seule preuve de toute la story qui exécute la requête réelle**. Les dix-sept preuves de composant passent par un `vi.mock` de l'API et sont, par construction, aveugles au défaut que D15 répare — c'est précisément pourquoi trois passes de revue ne l'avaient pas vu.

Le tableau des pièges muets passe de sept à **neuf**, avec les deux familles qu'a révélées cette passe : *une preuve d'archivage écrite fonctionnellement* (verte sous la mutation, le mock ignorant ses arguments) et *une assertion E2E sans la visibilité du signal* (verte sur `main`, avant qu'une ligne soit écrite).

⚠️ **Passe 4 due**, rotation **Sonnet**, contexte frais. Elle porte sur une spécification dont le **mécanisme** vient de changer : ce n'est plus la même story qu'aux passes 1 à 3, et le trend ne se lit pas comme une simple continuation.

### Passe 4 de `bmad-create-story validate` — 2026-08-17, Sonnet ×3, contexte frais

Bruts : 2 + 3 + 3 = 8. **Deux convergences.** Retenus : **3 CRITICAL / 1 HIGH / 2 MEDIUM / 0 LOW — 6 findings, 6 correctifs appliqués, 0 écarté.**

**Trend : `9` → `11` → `20` → `6`. Le NOMBRE s'effondre. La SÉVÉRITÉ, non : `1C` → `0C` → `2C` → `3C`.**

⚠️ **Le critère de non-convergence est donc déclenché pour la DEUXIÈME passe consécutive** — « une passe `N+1` remonte une sévérité **égale** ou supérieure ». `CRITICAL → CRITICAL`, et le compte de `CRITICAL` monte de 2 à 3.

**Et cette fois, l'explication de la passe 3 ne tient plus.** En passe 3 j'ai soutenu — à juste titre — que la remontée de sévérité venait d'un **changement de lentille** : on était passé de la lecture à l'exécution. La passe 4 emploie **exactement les mêmes lentilles que la passe 1** (Sonnet ×3, mêmes trois angles), et trouve **trois `CRITICAL`**. Ce n'est plus un effet d'instrument.

#### Les trois CRITICAL, et ce qu'ils ont en commun

**Ils sont TOUS LES TROIS dans le mécanisme écrit par le patch de remédiation de la passe 3** — un patch volumineux, rédigé d'un seul geste, et que rien n'avait relu avant cette passe.

1. **L'ordre `seuil → normalisation` était inversé** (convergence de deux lentilles, vérifiée par exécution). D7-bis mesurait le seuil sur le terme **brut**, D16 normalisait ensuite : `term.split(/\s+/)` ne connaît que l'espace, or D16 crée **dix autres frontières de token**. Le défaut casse dans **les deux sens** — `Yo-An` (5 caractères bruts) passe le seuil puis devient deux tokens de 2, jamais indexés ⇒ **silence garanti sur un doublon exact** ; `C++` passe le seuil puis devient `C` ⇒ `AGAINST('C*')` remonte **tout contact commençant par C**. Un simple **ordre d'instructions**, invisible en revue de diff.
2. **« et N autres » comptait le contact exclu** (convergence de deux lentilles). `rn.total` est le `COUNT(*)` **serveur**, qui ne connaît pas la notion de « soi » ; l'exclusion `c.id !== editing?.id` n'a lieu qu'au **client**. Corriger un caractère du nom d'une fiche existante — le scénario même d'AC7 — affichait donc **« et 1 autre » au-dessus d'une liste vide**, en désignant la fiche qu'on modifie. La preuve d'AC7 restait verte : elle ne regardait que la liste.
3. **La preuve décisive du classement était écrite dans le mauvais langage.** J'avais confié à un `#[sqlx::test]` le soin de « classer selon D15 » — **incohérent** : `rank()` est une fonction TypeScript, qu'un test Rust ne peut pas appeler. La mutation « `rank()` = identité » laissait les **32 preuves vertes** et faisait réapparaître en production le défaut exact qui avait motivé D15. Les deux preuves sont désormais **séparées et non interchangeables** : la 7 (vitest) voit le classement livré au navigateur, la 8 (`#[sqlx::test]`) voit la requête réelle ; aucune ne voit ce que voit l'autre.

Le `HIGH` est du même bois : la preuve de la fenêtre SQL était **orpheline** — rattachée à T1 par le tableau de synthèse, mais exclue par le libellé de T1 (« et **seulement** cela »). Les deux `MEDIUM` : « puis proximité » n'était **pas un algorithme** (deux implémentations conformes à la lettre rendaient des résultats différents, donc rien n'était testable), et **D17 n'avait ni preuve ni mutation** — dixième piège muet.

#### Ce que la passe 4 dit du PROCESSUS, et c'est le vrai enseignement

Le tableau se lit d'un coup d'œil :

| | ce que la passe a trouvé | d'où ça venait |
|---|---|---|
| **P2** | dérives de décompte, décisions sans preuve | **les patches de P1** |
| **P3** | le mécanisme ne fait pas le travail | la spec d'origine *(exécution)* |
| **P4** | 3 `CRITICAL` | **le patch de remédiation de P3** |

**Chaque tour de correctifs réinjecte des défauts au sommet de l'échelle de sévérité.** Ce n'est pas un accident de rédaction : c'est le mode d'échec que le `CLAUDE.md` nomme deux fois — « un patch vient AVEC son test » et la § *Propagation post-patch* — et le constater une troisième fois sur la même story signifie que **la codification ne suffit pas à le contenir ici**.

Le diagnostic exact n'est **pas** « la story est trop large » au sens du décompte de modules : elle en touche quatre. C'est que **le patch de remédiation est trop large pour être tenu dans un seul mental-model fiable** — celui de la passe 3 réécrivait un mécanisme entier, trois décisions neuves et dix preuves, d'un seul geste.

⚠️ **Arbitrage soumis à Guy** — cf. § *Questions ouvertes*, **Q5**.

## Dev Agent Record

*(sans objet — ce document n'est pas implémenté. Cf. les Dev Agent Record de la 22-2a et de la 22-2b.)*
