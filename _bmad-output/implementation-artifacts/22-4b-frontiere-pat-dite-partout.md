# Story 22.4b : La frontière du jeton, dite partout où elle est écrite

## Status

backlog

⚠️ **Née du découpage de la story 22-4**, le 2026-08-13, après quatre passes de `bmad-create-story validate`. Elle reçoit **la frontière documentaire** ; la story **22-4a** reçoit le mécanisme. Les deux se mergent dans **une seule PR** — cf. § *Découpage* de 22-4a.

## Story

**As a** intégrateur qui écrit un client de l'API Kesh, ou administrateur qui lit le manuel,
**I want** que la documentation dise la frontière réelle du jeton, et non celle d'avant,
**so that** je ne construise pas une intégration sur une route qui répond désormais `403`, et que je ne croie pas ouverte une faille qui est fermée.

Complète **#167** (KF-036) hors du code. La PR qui porte les deux moitiés porte `closes #167`.

## Pourquoi cette story existe séparément, et ce que ça change à sa méthode

La story 22-4 n'a pas convergé en quatre passes, et le diagnostic est précis : **son critère documentaire énumérait.** Chaque passe, chaque lentille y trouvait un site de plus — d'abord le guide d'intégration entier, puis son tableau des codes d'erreur, puis deux lignes du manuel, puis sept énoncés d'autres specs. Une clause qui énumère se lit comme close, et **une énumération sur un corpus mouvant ne peut jamais l'être**.

⚠️ **D'où le renversement de méthode : cette story pose un CRITÈRE DÉCIDABLE, et ne traite l'inventaire que comme un instantané.** Le critère survit à l'ajout d'un document ; la liste, non. Elle est donnée parce qu'elle a été vérifiée et qu'elle fait gagner du temps — pas comme définition du périmètre.

**Et la revue se fait au file-by-file**, comme la § *Règle de splitting préventif* du `CLAUDE.md` le prescrit pour une sous-story de rollout — pas en passes adversariales globales, qui sont précisément ce qui a échoué.

## Décisions

**D-a — Le critère : est réécrit tout énoncé qui, au PRÉSENT, décrit un contrat ou une limitation que la story 22-4a rend faux.**

Trois questions, dans cet ordre, et la réponse est décidable :

1. **L'énoncé est-il au présent, et affirme-t-il quelque chose sur ce qu'un jeton peut atteindre ou sur le code rendu ?** Sinon — c'est un compte rendu de ce qui a été fait, une ligne de tâche cochée, une note de revue — **on n'y touche pas.**
2. **Est-il dans un document que quelqu'un lit pour AGIR ?** Manuel, guide d'intégration, CHANGELOG : il est **réécrit**.
3. **Est-il dans une spécification `done` ?** Alors c'est une **trace** : elle reçoit une mention de clôture, elle **n'est pas réécrite** — sauf s'il s'agit d'un **énoncé de frontière** (une décision `DC`), auquel cas la frontière se réécrit, cf. D-b.

⚠️ **La distinction trace / énoncé n'est pas cosmétique.** Réécrire un Dev Agent Record effacerait la décision de l'époque et rendrait l'historique du dépôt menteur dans l'autre sens. Les traces disent *ce qui a été décidé alors* ; les énoncés de frontière disent *ce qui est vrai maintenant*.

**D-b — DC6 devient une CONJONCTION, pas un remplacement.**

La story 22-4 prescrivait : « elle disait *un PAT ne gère pas les clés* ; elle dira *un PAT n'atteint aucune route `require_admin_role`* ». **C'était un remplacement, et il perdait la moitié de la frontière.**

⚠️ Les routes de gestion de clés vivent dans `comptable_routes` (`lib.rs:575`, `:579`) et ne sont **pas** des routes `require_admin_role`. Après un remplacement, DC6 ne dirait plus rien d'elles — et deux énoncés de `17-2b` (`:172`, `:220`) invoqueraient « DC6 backend = filet » pour une frontière qui ne les couvre plus, tandis que `17-2c:102` (« DC6 : JWT cookie only ») deviendrait orphelin. La story 22-4 affirmait pourtant, sous la mention « vérifié, pas supposé », que ces phrases restaient vraies. *(Relevé en passe 4.)*

**La formulation retenue, en deux membres :**

> **DC6** — un PAT ne gère pas les clés (`ensure_not_pat`, `comptable_routes`) **et** n'atteint aucune route `require_admin_role` (couche, story 22-4a).

**D-c — Une seule graphie de la chaîne de contrôle, et elle est sans balisage.**

La chaîne qui sert de preuve est **`n'atteint aucune route require_admin_role`**, écrite **sans apostrophes inverses**.

⚠️ **Sans cette décision, la clause de preuve est insatisfaisable AVEC le travail fait.** La story 22-4 prescrivait la formulation avec balisage — `` `require_admin_role` `` — et grepait la chaîne **sans**, en `-F`. Un développeur appliquant D4 à la lettre obtenait `0, 0, 0`. La passe 3 avait réparé une clause satisfaisable **sans** travail ; elle en avait produit une insatisfaisable **avec**. *(Relevé en passe 4 par deux lentilles indépendamment.)*

Le reste de la phrase peut porter du balisage ; c'est **ce fragment-là** qui est la chaîne-pivot, et il s'écrit en texte nu.

**D-d — Le CHANGELOG écrit sous `### Sécurité`, dans une section `## [Unreleased]` à créer.**

`CHANGELOG.md` est strictement par release, la plus récente — `## [0.9.0]` — étant **publiée**. Il n'existe pas de section `[Unreleased]` : elle est créée. Et l'intertitre est **`### Sécurité`**, non `### Ajouté` : la story **retire** un accès à des jetons existants, c'est un durcissement, et c'est le mot que cherche un lecteur qui audite.

## Acceptance Criteria

**AC-a — Aucun document que l'on lit pour agir n'affirme encore la faille ouverte, ni n'enseigne à l'exploiter.**

*Preuve* — deux contrôles, tous deux fail-loud :
- `grep -nF "KF-036" docs/api-external.md docs/manual/fr/admin-manual.tex` ne rend **plus aucun** énoncé présentant la limitation comme **ouverte** — au passé, ou rien ;
- `grep -cF "API_KEY_ADMIN_FORBIDDEN" docs/api-external.md` rend **au moins 1** : le nouveau code figure dans le tableau des codes d'erreur.

⚠️ **Le tableau des codes d'erreur est la condition d'entrée, et il manquait à la story mère.** `api-external.md:262` dit à l'intégrateur, en propres termes, de **se fier au champ `code`** — et `:267` est la table qui les énumère. Livrer un `403` neuf sans l'y inscrire, c'est reproduire dans le nouvel état le défaut de documentation que la story existe pour réparer. *(Relevé en passe 4 par les deux lentilles Opus.)*

**AC-b — La frontière DC6 est réécrite en conjonction à ses quatre énoncés.**

*Preuve* : `grep -oF "n'atteint aucune route require_admin_role"` rend **1** dans `17-2-api-pat-integrations.md`, **2** dans `17-2a-api-pat-backend.md`, **1** dans `17-2c-api-pat-doc.md` — nommés fichier par fichier, avec leur attendu.

⚠️ **Nommer chaque fichier avec son attendu, et non lire une suite de nombres.** Un `grep -c` sur un glob rend ses résultats dans un ordre **non garanti** : lu positionnellement, le critère crie au loup dans le cas nominal, et le multi-ensemble `{1,2,1}` reste identique si le `2` et un `1` s'échangent de fichier — un faux vert. Et `grep -o … | wc -l` compte les **occurrences**, là où `grep -c` compte les **lignes** : la seconde forme fige la mise en page de l'amendement. *(Relevé en passe 4.)*

**AC-c — Les traces reçoivent une mention de clôture, et ne sont pas réécrites.**

*Preuve* : chaque site de la catégorie *trace* de l'inventaire porte une mention citant **« story 22-4 »** et **`#167`**, et son texte d'origine est intact — vérifiable au `git diff`, qui ne doit montrer que des **ajouts** sur ces lignes.

**AC-d — Le CHANGELOG dit, dans les mots de l'utilisateur, qu'un jeton créé par un Admin perd des accès qu'il avait.**

*Preuve* : une section `## [Unreleased]` existe, porte un intertitre `### Sécurité`, et son texte nomme la perte d'accès des jetons Admin **existants** — sans quoi la première intégration qui tombe en `403` produit un ticket de support.

**AC-e — L'issue #167 se ferme réellement.**

*Preuve* : le message de la PR porte le mot-clé `closes #167`, et après merge :

```sh
gh issue view 167 --json state --jq .state    # CLOSED
```

⚠️ **Ce n'est pas une formalité, c'est un mode d'échec documenté du dépôt.** Le `CLAUDE.md` en porte le précédent : la Story 16-3b avait sept commits en `refs #151` et une PR disant « Ferme #151 » **en prose, pas en mot-clé** ; l'issue est restée ouverte après le merge et a dû être fermée à la main. Rien ne rougit, rien ne prévient. La story 22-4 annonçait « Ferme #167 » en prose et n'avait aucune tâche portant le mot-clé. *(Relevé en passe 4.)*
Décider aussi du sort du label `v0.2-milestone` porté par l'issue.

## Inventaire vérifié — un INSTANTANÉ du 2026-08-13, que D-a supersède

⚠️ **Cette liste n'est pas le périmètre.** Le périmètre est le critère **D-a**. La liste a été vérifiée ligne à ligne et fait gagner du temps ; si un site s'y trouve à tort ou si un autre y manque, **c'est le critère qui tranche**, pas elle.

### Documentation utilisateur — réécrite (D-a question 2)

`docs/api-external.md` :

| Ligne | Ce qui devient faux, ou incomplet |
|---|---|
| `:235` | **une instruction d'usage** : « pour changer un mot de passe via l'API, utilisez `PUT /api/v1/users/:id/reset-password` » — rendra `403` |
| `:222` | l'avertissement « une clé créée par un Administrateur hérite des pouvoirs d'Administrateur », avec renvoi à KF-036 |
| `:246` | la ligne « Auto-propagation des clés Administrateur » du tableau des limitations |
| `:267` | **le tableau des codes d'erreur** — il manque `API_KEY_ADMIN_FORBIDDEN` (cf. AC-a) |
| `:229` et `:231` | la recommandation de `PUT /api/v1/users/:id` en remplacement complet — passage devenu inatteignable par PAT |
| `:205`, `:209-210` | les mutations `/vat-rates` données comme disponibles, la note ¹ ne les réservant qu'au **rôle** Administrateur |
| `:195` | « toute route `/api/v1/*` de l'UI est consommable » — devient **faux** pour 25 couples |
| `:73` | « la permission effective est l'intersection du rôle du créateur et de la portée » — devient **incomplet** : les routes admin sont fermées quelle que soit cette intersection |

`docs/manual/fr/admin-manual.tex` :

| Ligne | Ce qui devient faux, ou incomplet |
|---|---|
| `:1765` | le bloc `keshwarning` « Moindre privilège » — décrit l'héritage des pouvoirs Admin comme un fait courant, avec renvoi à KF-036 |
| `:1756` | « la gestion des clés est impossible via l'API elle-même » — à **élargir** à toute l'administration |
| `:1757` | la même formule d'intersection que `api-external.md:73`, **dans l'itemize que T-b édite déjà** |

⚠️ **`:1757` est l'exemple type du symptôme non grepé.** La story mère relevait la formule à `api-external.md:73` et ne la cherchait pas ailleurs — elle était deux lignes au-dessus d'un passage qu'elle éditait déjà, et une troisième fois dans `17-2:28`. C'est la § *Propagation post-patch* du `CLAUDE.md`, prise en défaut. *(Relevé en passe 4.)*

⚠️ **Régénérer le PDF** (`latexmk -xelatex` dans `docs/manual/fr/`) et le commiter : la convention du dépôt versionne les PDF.

### Énoncés de frontière DC6 — réécrits en conjonction (D-b)

`17-2:57` · `17-2a:35` (décision) et `:58` (tâche) · `17-2c:34` — **quatre énoncés, trois documents.**

⚠️ **`17-2` est à amender AVEC les autres, contrairement à ce que la story mère a d'abord dit.** Il est resté `ready-for-dev`, vestige de découpage — mais `17-2a:206` le désigne comme « spec parente convergée, **source des DC1-DC6** ». Un lecteur qui suit cette piste y trouverait sinon l'ancienne frontière, avec l'autorité de la source.

⚠️ **`17-2:28` porte la formule d'intersection** (DC3). Elle devient incomplète pour la même raison que `api-external.md:73` — à traiter au même passage.

### Traces — mention de clôture, sans réécriture (D-a question 3, AC-c)

| Site | Ce qu'il porte |
|---|---|
| `17-2a:114` | entrée `[Review][Defer]` — la décision Project Lead d'accepter L3 en catégorie B |
| `17-2a:255` | la limitation **L3** dans les Completion Notes |
| `17-2c:35` | L3 / KF-036 dans les décisions de la story doc |
| `17-2c:129` | ⚠️ **affirme au PRÉSENT que les routes admin sont atteignables par un PAT**, et prescrit d'en faire un avertissement |
| `17-3:49` et `:158` · `17-3a:27` et `:54` · `17-3c:28` et `:103` · `14-2:402` | sept énoncés de contrat donnant `API_KEY_MANAGEMENT_FORBIDDEN` pour `full-export`, `full-import` et `reopen` — le **code change** (22-4a, D2) |

⚠️ **`17-2c:129` avait été rangé « hors sujet » par erreur de tri.** La passe 3 avait classé les sept résultats de son grep en « trois entrées L3 + quatre Dev Notes hors sujet » ; celui-là énonce la faille au présent et appartient exactement à la catégorie que la story dit devoir traiter. *(Relevé en passe 4.)*

⚠️ **Les sept énoncés de `17-3*`/`14-2` n'avaient jamais été recensés.** La story mère avait relevé les **trois tests** et **la fixture frontend** que le nouveau code invalide, mais pas les spécifications qui décrivent le même contrat : DC7 de l'Epic 17-3 devient faux tel qu'écrit. *(Relevé en passe 4.)*

### Vérifié NON impacté — à ne pas rejouer

`17-2b:172` et `:220` (« DC6 backend = filet » pour la page de gestion des clés) restent **vrais** sous la conjonction de **D-b** — et deviendraient faux sous un remplacement · `website/` ne mentionne ni PAT, ni clé API, ni KF-036 · `README.md` ne porte aucune affirmation sur les permissions · `docs/manual/fr/user-manual.tex:239-244` décrit les clés sans énoncer l'héritage Admin ni la formule d'intersection · `marketing-brochure.tex` ne parle pas de PAT · les manuels `de/`, `en/`, `it/` ne contiennent qu'un `README.md` · `frontend/tests/e2e/api-keys.spec.ts` n'exerce que `/accounts` et `/contacts` · les Dev Agent Records, listes de tâches cochées et notes de revue des specs `17-2*` sont des **traces d'implémentation** : hors critère D-a.

## Tasks / Subtasks

- [ ] **T-a — Le guide d'intégration** (AC-a). Les huit sites d'`api-external.md`. ⚠️ **Commencer par `:267`**, le tableau des codes : c'est la condition d'entrée d'AC-a et le seul site dont l'omission casse le contrat programmatique.
- [ ] **T-b — Le manuel administrateur** (AC-a). `:1765`, `:1756`, `:1757`. Puis **régénérer le PDF** et le commiter.
- [ ] **T-c — Les quatre énoncés de DC6** (AC-b). Réécriture en **conjonction** (D-b), chaîne-pivot sans balisage (D-c).
- [ ] **T-d — Les traces** (AC-c). Mention de clôture sur les quatre entrées de limitation et les sept énoncés de contrat. **Aucune réécriture** — le `git diff` ne doit montrer que des ajouts sur ces lignes.
- [ ] **T-e — Le CHANGELOG** (AC-d). Section `## [Unreleased]` à créer, intertitre `### Sécurité` (D-d).
- [ ] **T-f — La fermeture de l'issue** (AC-e). `closes #167` dans le message de la PR, contrôle `gh issue view` après merge, et arbitrage du label `v0.2-milestone`.
- [ ] **T-g — Le grep de propagation, une fois les patches faits.** Rejouer le critère **D-a** sur tout le dépôt et lister ce qu'il atteint encore : `grep -rn "KF-036\|API_KEY_MANAGEMENT_FORBIDDEN\|permission effective\|hérite des pouvoirs"` sur `docs/`, `website/`, `README.md`, `CHANGELOG.md` et `_bmad-output/`. **Trier à la main** — la commande rend des traces légitimes, c'est le prix et il est bas.

## Dev Notes

### La méthode de revue de cette story

**File-by-file, pas en passes adversariales.** La story mère a consommé quatre passes sans que son critère documentaire converge, chaque passe ajoutant des sites. Le geste utile n'est pas une cinquième lentille sur le même texte : c'est d'ouvrir chaque fichier de l'inventaire, d'appliquer **D-a** à chacun de ses énoncés, et de cocher.

### Ce que la story mère a appris, et qui vaut au-delà d'elle

⚠️ **Une clause de preuve doit être discriminante, sinon elle ne prouve rien.** Deux fois de suite, la story 22-4 a produit une clause fautive sur le même critère : d'abord un `grep -rn "require_admin_role"` qui rendait **déjà sept résultats** avant tout amendement — satisfaisable sans travail —, puis une chaîne balisée grepée sans balisage — insatisfaisable **avec** le travail. Le contrôle d'une clause de preuve est de la **rejouer sur le dépôt tel qu'il est** et de regarder ce qu'elle rend.

⚠️ **Une énumération ne clôt pas un périmètre mouvant.** C'est le motif du découpage. Chaque fois qu'un critère documentaire est écrit comme une liste, il faut se demander ce qui décide de l'appartenance à la liste — et écrire **cela**.

### References

- Issue **#167** (KF-036).
- Story **22-4a** — le mécanisme, **à merger dans la même PR**.
- Stories **17-2**, **17-2a**, **17-2b**, **17-2c** — DC1-DC6, limitation L3.
- Stories **17-3**, **17-3a**, **17-3c**, **14-2** — DC7 et les énoncés de contrat portant l'ancien code.
- `CLAUDE.md` — § *Synchroniser TOUTES les docs*, § *Issue Tracking Rule*, § *Propagation post-patch*, § *Règle de splitting préventif*.

## Change Log

**2026-08-13 — créée par découpage de la story 22-4**, après quatre passes de `validate`. Elle reçoit la frontière documentaire, qui est la partie **qui n'a pas convergé** : les quatre passes y ont trouvé, cumulativement, le guide d'intégration entier, son tableau des codes d'erreur, trois lignes du manuel, quatre énoncés de DC6, quatre traces de limitation et sept énoncés de contrat dans d'autres specs.

**Le changement de méthode est le fond du découpage** : un critère décidable (**D-a**) remplace l'énumération, et l'inventaire devient un instantané que le critère supersède. Les findings de passe 4 sont intégrés : DC6 en conjonction et non en remplacement, le tableau des codes d'erreur, `:195` et `:73`, `admin-manual:1757` et `17-2:28`, `17-2c:129` sorti du « hors sujet », les sept énoncés de `17-3*`/`14-2`, la graphie unique de la chaîne-pivot, `grep -o` plutôt que `grep -c`, la section et l'intertitre du CHANGELOG, et `closes #167`.

**Aucun gate exécuté** : la story n'a pas de code.
