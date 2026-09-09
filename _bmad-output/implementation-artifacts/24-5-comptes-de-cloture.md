# Story 24.5 : Les comptes de clôture — fermer le compte où l'à-nouveau se perd

## Status

ready-for-dev

## Story

**As a** personne qui tient les livres et arrive d'un autre logiciel,
**I want** que les comptes de clôture 9000/9100/9200 n'acceptent aucune écriture,
**so that** mon bilan d'ouverture ne parte pas en charges sans que rien ne m'avertisse.

## ⛔ Le correctif annoncé par l'issue ne ferme PAS le défaut décrit par l'issue

L'issue #375 conclut : *« Correctif : une ligne dans trois fichiers JSON, plus un backfill. »*
Cette ligne serait le **type** du compte. **Changer le type ne ferme rien** — il déplace le
montant, il ne l'empêche pas d'entrer :

| type donné à 9000 | où l'à-nouveau atterrit |
|---|---|
| `Expense` (aujourd'hui) | en **charges** — le défaut de l'issue |
| `Liability` | au **bilan**, en passif — faux autrement, mais visible |
| un type neuf (`Equity`/`Closing`) | **nulle part** : ni bilan ni résultat, le montant **disparaît des deux états** |

⛔ **La troisième ligne est la plus grave, et c'est celle vers laquelle l'intuition pousse.** Un
type « hors états » rendrait le défaut *plus* silencieux qu'il ne l'est : aujourd'hui le montant
est au mauvais endroit, il serait alors nulle part. *Un compte qui n'apparaît dans aucun état
n'est pas un compte corrigé, c'est un trou.*

**Ce qui ferme le chemin, c'est la POSTABILITÉ**, et le dépôt a déjà tranché ce cas exact.

## D1 — Le gabarit existe : `2979 Résultat de l'exercice`, et il dit pourquoi

`is_postable` (`crates/kesh-core/src/chart_of_accounts/mod.rs:314`) rend `false` pour deux
causes seulement, **toutes deux chart-agnostiques — aucun numéro codé en dur** :

```rust
pub fn is_postable(entry: &ChartEntry, parent_numbers: &HashSet<&str>) -> bool {
    if parent_numbers.contains(entry.number.as_str()) {
        return false;                                  // compte de regroupement
    }
    entry.role != Some(AccountRole::CurrentYearResult) // l'appli CALCULE ce solde
}
```

La seconde cause est notre cas, mot pour mot : *« en modèle temps réel virtuel (Story 14-1),
l'application **calcule** le résultat de l'exercice à chaque rendu ; y poster serait un
double-comptage garanti. »*

⛔ **Les comptes 9000/9100/9200 matérialisent exactement ce que Kesh calcule** — le bilan
d'ouverture, le compte de résultat, le bilan de clôture. Dans un logiciel qui passe des écritures
de clôture, ils reçoivent le virement des soldes ; **Kesh n'en passe aucune**, et
`grep -rnE "['\"](9000|9100|9200)['\"]" crates/ --include=*.rs` rend **zéro site applicatif** : ces
trois comptes n'ont, dans Kesh, aucun usage.

⚠️ **Le motif porte les guillemets, et ce n'est pas un détail de forme** : `grep -rn '9000'` sans
eux rend dix lignes de `post_restore.rs` — des sous-chaînes de numéros de version de migration
(`20260729000001`). *Un grep qui rend zéro sur un symptôme doit éveiller le soupçon ; un grep qui
rend du bruit fait conclure à tort qu'il y a des sites.* Ils sont offerts à la saisie et à rien d'autre.

⚠️ **`RetainedEarnings` reste postable, et la distinction est instructive** : le doc-comment
d'`is_postable` le dit — *« un utilisateur qui migre depuis un autre logiciel doit pouvoir poser
son report à nouveau d'ouverture »*. Le report à nouveau est une **donnée** que l'utilisateur
apporte ; le bilan d'ouverture est un **état** que Kesh dérive. C'est la ligne de partage, et
c'est elle qui autorise à fermer les trois comptes sans fermer la porte au migrant : **son
à-nouveau a déjà un compte légitime**, le `2970`.

## D2 — Le marqueur va dans le PLAN, pas dans le code

`is_postable` est chart-agnostique **par décision** (« aucun numéro codé en dur », cf. le
préambule de la migration `20260722000001` : *« Tout code applicatif qui écrirait
`WHERE number = '1100'` est un piège silencieux »*). Y coder `9000` défairait la doctrine que la
Story 14-3a a posée.

⇒ **`ChartEntry` reçoit un champ optionnel `postable: Option<bool>`**, et le gabarit est à trois
lignes de là : `role` est déjà `#[serde(default)] Option<AccountRole>`, avec le commentaire
*« un plan sans annotation reste valide (non-breaking) »*. Même patron, même raison.

```rust
#[serde(default)]
pub postable: Option<bool>,   // None = comportement actuel
```

`is_postable` gagne une troisième cause, en **tête** des deux autres :

```rust
if entry.postable == Some(false) { return false; }
```

⚠️ **Un `Some(true)` ne doit PAS forcer la postabilité** — il ne doit pas pouvoir rouvrir un
compte de regroupement ni le résultat de l'exercice. Le champ ne sait que **retirer**. Un test
l'exerce sur une entrée `postable: true` qui est aussi parente.

## D3 — Le type reste `Expense`, et c'est un choix, pas un oubli

**L'enum `AccountType` n'a que quatre variants** — `Asset`, `Liability`, `Revenue`, `Expense`
(`kesh-db/src/entities/account.rs:12` et `kesh-core/.../mod.rs:16`, **dupliqués** par l'orphan
rule). Il n'existe **pas** d'`Equity` : les fonds propres sont modélisés en `Liability`
(classe 28, vérifié au sol).

Ajouter un variant coûterait : deux enums, le `CHECK` `chk_accounts_type` (VARCHAR(20) + CHECK,
**pas** un ENUM MySQL), les `match` de `income_statement`/`balance_sheet`/`trial_balance`, le
frontend, les quatre locales — et ce serait **breaking au sens P1** (un binaire antérieur ne sait
pas décoder la valeur neuve : `FromStr` rend `Err`), donc bump `min_required` **et** bump Cargo
du workspace entier (P2-bis).

⛔ **Et cela n'apporterait rien une fois les comptes non postables** : `income_statement` et
`balance_sheet` font tous deux un `INNER JOIN journal_entry_lines`. **Sans ligne, le compte
n'apparaît nulle part** — son type devient inerte.

⇒ Le type reste `Expense`. **Limitation L1, documentée**, avec son issue de suivi (T7) : l'arbre
affiché montre « 9 Clôture » sous un type Charge, ce qui est visuellement faux et sans effet sur
les chiffres. *L'issue #375 classe elle-même la cohérence de type en « accessoirement ».*

## D4 — Le backfill ne corrige QUE l'avenir, et il faut le dire

`postable = FALSE` **n'efface aucune ligne existante**. Un utilisateur qui a déjà passé son
à-nouveau par le 9000 garde son montant en charges après la migration : la garde ferme le
chemin, elle ne répare pas le passé.

⛔ **Et c'est la seule conduite admissible.** Déplacer d'office des écritures existantes serait
une correction **invisible dans les livres**, exactement ce que l'art. 958f CO interdit et ce que
toute cette vague combat. La voie de correction existe désormais et elle est apparente : la
**contre-passation** livrée par la 24-4a. C'est au manuel de l'enseigner (T6), pas à la migration
de l'exécuter.

⚠️ **Conséquence assumée** : un compte non postable portant des écritures est un état que le
dépôt connaît déjà — c'est le cas de l'issue #271, où un compte de configuration devenu non
postable disparaissait des sélecteurs. Le correctif d'alors (`account-options.ts`) réintroduit la
valeur configurée ; **rien à faire ici**, aucun réglage ne pointe sur un compte 9.

## D6 — Le triage P7 n'est PAS une case à cocher : aucune des trois issues n'est propre

⛔ **C'est le point le plus facile à croire réglé, et le dépôt a déjà écrit noir sur blanc que ce
statement-là n'est pas gardé.** Le commentaire de `RETIRED_BACKFILLS` porte sur le gabarit même
que l'AC 7 cite (`post_restore.rs:257-260`) :

> *« Sur ses 12 statements, les 10 `UPDATE` de rôle sont gardés `role IS NULL`, mais les 2
> `UPDATE` de `postable` ne portent **AUCUNE** garde — rejoués sur une base à jour, ils
> écraseraient un `postable` posé à la main (`PUT /api/v1/accounts/{id}`, sémantique
> full-replace). »*

Les trois issues, examinées :

| issue | verdict |
|---|---|
| exemption **`Hors fenêtre`** | ⛔ **fausse** — la migration est la dernière du dépôt, donc **postérieure** à la dernière création de table : elle est **dans** la fenêtre. Le test `exemptions_claiming_out_of_window_really_are_out_of_window` recalcule et **rejetterait** la justification |
| **classe B** (sentinelle) | ⛔ **indisponible** — la classe B exige que **le DDL et le backfill soient dans le même fichier** ; notre migration est un `UPDATE` **pur**, il n'y a aucune colonne à créer, donc aucune sentinelle valide |
| **classe A** (inconditionnel) | ⚠️ **le critère n'est pas satisfait** — `AND postable = TRUE` porte l'**idempotence**, pas une garde d'**intention** |

⛔ **Et le rejeu est NÉCESSAIRE.** Une archive prise entre **`20260827000001`** — dernière
migration créatrice de table, donc **borne basse de la fenêtre** — et cette migration porte
`postable = TRUE` sur le 9000 et le **restaure**. Sans rejeu, le défaut **rouvre définitivement** :
`_sqlx_migrations` n'étant pas restaurée, la migration reste marquée appliquée et ne repassera
jamais. C'est le mode d'échec exact que P7 existe pour empêcher.

⚠️ **La borne est celle de la fenêtre, PAS la date de la colonne.** `postable` existe depuis
juillet, mais une archive de juillet est **refusée en 400** au contrôle de couverture — c'est le
motif même par lequel les deux anciennes entrées ont été exemptées. *Invoquer juillet ici
utiliserait une fenêtre que le paragraphe précédent vient de recalculer autrement.* L'intervalle
réel est donc étroit — il exclut **v0.11.1**, la version publiée — mais il n'est pas vide, et c'est
tout ce que la nécessité du rejeu demande.

### Les gardes d'intention examinées, et pourquoi aucune n'est retenue

| voie | verdict |
|---|---|
| `AND version = 1` | ⛔ **faux négatif silencieux** — un compte seulement **renommé** porte `version > 1` sans qu'on ait touché `postable` ; le rejeu le sauterait. *Le mode d'échec qu'on ferme, réintroduit par sa propre garde.* |
| garde par **`audit_log`** | ⚠️ **constructible et correcte en principe** — `audit_log` est restaurée (`backup.rs`, `TABLES_TO_TRUNCATE`) et `account_snapshot_json` (`accounts.rs:46-60`) y écrit `postable` depuis la 14-3a, donc l'archive **porte** la trace d'une réouverture délibérée. **Écartée sur le coût** : elle ferait dépendre une migration de données du **contenu** d'une table d'audit — table dont la sémantique évolue, que le dépôt ne requête nulle part à cette fin, et qui n'a **aucun précédent** de ce type. Le remède serait plus lourd que le mal. |
| **backfill hors migration** (au boot et en fin d'import, gabarit `backfill_client_number_canonical`, 22-1) | ⛔ **écartée** — elle s'exécuterait à **chaque démarrage** pour fermer trois comptes une fois, et déplacerait hors de toute trace un geste que le registre, lui, rapporte. |

⇒ **Décision : EXEMPTION, avec une justification factuelle et vérifiable — et NON une classe A
par dérogation.**

⛔ **Ce qui a fait changer la décision : la classe A casserait une PROMESSE FAITE À L'EXPLOITANT,
pas seulement un commentaire.** `admin-manual.tex:1609` énonce, comme première des trois
propriétés du rejeu :

> *« **Vos données ne sont jamais écrasées.** Si la sauvegarde contenait déjà l'information, elle
> fait foi : un rôle de compte ou un réglage que vous aviez ajusté à la main est conservé tel
> quel. »*

Le manuel, l'en-tête du module et le critère de P7 disent donc **la même chose**, à trois endroits
indépendants. Une classe A non gardée ne contredirait pas un détail d'implémentation : elle
rendrait **fausse une garantie publiée**.

⛔ **Et le rejeu protégerait un parc qui N'EXISTE PAS.** L'intervalle où une archive porterait
`postable = TRUE` est `[20260827000001 … cette migration)`. Or la dernière version **publiée** est
**v0.11.1, du 2026-08-24** — donc **antérieure** à la borne basse. Cet intervalle ne contient
**aucun binaire distribué** : uniquement des builds de développement.

⇒ **L'exemption est donc l'issue juste, et sa justification est factuelle** : *« aucune version
publiée ne se situe entre `20260827000001` et cette migration ; l'intervalle ne contient que des
binaires de développement non distribués. »*

⚠️ **Elle ne commence PAS par `Hors fenêtre`, et c'est délibéré** : la migration **est** dans la
fenêtre, et le marqueur textuel déclencherait le contrôle symétrique
(`exemptions_claiming_out_of_window_really_are_out_of_window`) qui la **rejetterait à juste titre**.
L'argument n'est pas la fenêtre, c'est le **parc**.

⛔ **CETTE JUSTIFICATION SE PÉRIME, donc elle se contrôle — deux fois.** Une exemption fausse
désactive le rejeu *définitivement et en silence* ; celle-ci est vraie **aujourd'hui** et cesserait
de l'être si une version était publiée depuis `main` avant le merge de cette story :

```sh
git tag --sort=-creatordate | head -1        # doit rester <= v0.11.1 (2026-08-24)
git log -1 --format=%ci $(git tag --sort=-creatordate | head -1)
```

⇒ **Le contrôle est une tâche (T3), et il se refait au gate de clôture (T8)**, pas seulement à
l'écriture de la spec.

⚠️ **Question ouverte, à trancher avant l'implémentation** : l'instance qui tourne sur le NAS de
l'auteur exécute-t-elle un binaire **postérieur au 2026-08-27** ? Si oui, elle est le seul membre
du parc concerné, et la décision se rouvre — l'exemption deviendrait fausse pour la seule
installation réelle du projet. *Cette question ne se déduit d'aucun fichier du dépôt.*

## D5 — Le défaut a DEUX surfaces, l'issue n'en nomme qu'une

L'issue cite `income_statement.rs:76-83`. Il y en a une seconde, et elle touche le **bilan** :

```rust
// balance_sheet.rs:339-346 — fetch_retained_earnings
WHERE a.company_id = ? AND a.account_type IN ('Revenue', 'Expense') AND je.entry_date < ?
```

⇒ Un solde sur le 9000 entre **aussi** dans le report à nouveau du bilan. Le montant est donc
compté **deux fois de deux façons** : en charges au compte de résultat, et dans le report calculé
au bilan. ⚠️ *Un test d'AC écrit contre le seul compte de résultat laisserait la seconde surface
non vérifiée* — et elle est celle qui touche l'état que le réviseur ouvre en premier.

⚠️ **Troisième surface, sans défaut mais à connaître** : `trial_balance.rs` liste les comptes
mouvementés quel que soit leur type. La balance des comptes montre donc le 9000 fautif — c'est
un **symptôme visible**, pas un bug, et c'est par là qu'un utilisateur peut se rendre compte.

## Critères d'acceptation

1. `ChartEntry` porte `postable: Option<bool>` en `#[serde(default)]` — **un plan sans
   annotation reste valide et se comporte comme aujourd'hui** (non-breaking, gabarit `role`).
2. `is_postable` rend `false` dès que l'entrée porte `postable: Some(false)`, **en plus** des
   deux causes existantes, qu'elle ne modifie ni ne remplace.
3. `postable: Some(true)` **ne force rien** : une entrée parente ou portant
   `AccountRole::CurrentYearResult` reste non postable. Un test l'exerce sur les deux cas.
4. Les trois plans (`pme`, `association`, `independant`) annotent `"postable": false` sur
   **9000**, **9100** et **9200**.
   ⛔ **Sur ces trois-là et pas cinq** : `9` et `90` sont **déjà** non postables — ils sont
   parents, donc pris par la première cause. *L'issue nomme `9` mais omet `90` ; ni l'un ni
   l'autre n'a besoin de l'annotation, et l'ajouter masquerait la cause réelle.*
5. Au seed d'une société neuve (les trois `org_type`), les comptes 9000/9100/9200 sont créés avec
   `postable = FALSE`, et **aucun autre compte du plan ne change de postabilité**.
6. Une migration de backfill passe `postable = FALSE` sur les comptes existants dont le
   **numéro** est 9000, 9100 ou 9200 — **toutes sociétés confondues**.
   ⛔ **Ici le numéro EST le critère, et c'est licite** : une migration corrige un plan
   **déjà écrit** en base, où le numéro est la seule donnée qui identifie l'intention d'origine.
   La règle « aucun numéro codé en dur » vise le **code applicatif**, et la migration
   `20260722000001` procède déjà exactement ainsi (`UPDATE … WHERE number IN (…)`).
7. ⛔ **Le backfill ne porte PAS de clause `AND active = TRUE`**, contrairement aux dix `UPDATE`
   de rôle du gabarit (`20260722000001:129-138`). ⚠️ **Ne pas copier le gabarit sans lire ce qu'il
   fait** : la clause y protège l'**unicité des rôles singleton** — un compte archivé ne doit pas
   squatter un rôle. Ici il n'y a aucun rôle, et un compte 9000 archivé puis **réactivé** doit
   rester non postable. *Le gabarit voisin dont le modèle diffère est la faute qui a produit les
   deux CRITICAL de la 24-4a.* ✅ Le gabarit à copier **pour ce point précis** est deux lignes plus bas
   (`:177`, `UPDATE accounts SET postable = FALSE WHERE role = 'CurrentYearResult';`), qui ne
   filtre ni sur `active` ni sur la société.
   ⚠️ **Exact sur la clause, PAS sur la garde** : ce même statement est celui dont
   `post_restore.rs:257-260` dit qu'il ne porte **aucune** garde d'intention — cf. **D6**, qui en
   tire les conséquences pour le registre. *Un gabarit se copie sur le point pour lequel on
   l'invoque, jamais en bloc.*
8. Le backfill **ne touche ni `account_type`, ni les écritures, ni `version`** — cf. D4 et le
   précédent de `create` (`accounts.rs:191` : *« pas de bump de version : postable est ici une
   conséquence structurelle »*).
9. ⛔ **Le backfill est EXEMPTÉ** (`EXEMPT_MIGRATIONS`), avec la justification factuelle de D6 —
   *aucune version publiée ne se situe dans l'intervalle* —, et **non** inscrit en classe A.
   ⚠️ **La justification ne commence PAS par `Hors fenêtre`** : la migration est dans la fenêtre,
   et ce marqueur déclencherait un contrôle qui la rejetterait. ⛔ **Le fait se vérifie par
   commande avant le merge** (T3) **et se re-vérifie au gate de clôture** (T8) : une exemption
   fausse désactive le rejeu *définitivement et en silence*.

10. **L'invariant « seed ≡ backfill » tient** : le test dédié
   (`crates/kesh-db/tests/accounts_role_backfill.rs`) compare les deux sources et doit rester
   vert **en couvrant les trois comptes neufs**.
   ⛔ **C'est le filet principal de la story**, et son en-tête le dit : *« rien n'oblige
   structurellement ces deux sources à concorder ; le symptôme n'apparaîtrait qu'en production,
   chez un utilisateur migré. »*
11. La liberté de l'utilisateur reste entière, **à la création comme à la modification** : il peut
   créer lui-même un compte 9000 postable, et **rouvrir** par `PUT` un compte que le backfill a
   fermé — `effective_postable` (`accounts.rs:126`) ne force `false` que pour un parent ou le
   rôle `CurrentYearResult`, jamais pour un numéro.
   ⚠️ **Assumé, et ce n'est pas une faille** — la story corrige le plan **livré**, pas la liberté
   de l'utilisateur ; le dépôt ne déduit jamais de sémantique d'un numéro (14-3a).
   ⛔ **Mais c'est cette liberté qui rend D6 nécessaire** : un compte rouvert délibérément est
   indiscernable, après restauration, d'un compte jamais fermé. *Énoncer la liberté sans énoncer
   sa conséquence sur le rejeu, c'est laisser le développeur la découvrir au pire endroit.*
12. Un test de bout en bout démontre le défaut **fermé sur les deux surfaces de D5** : après
    migration, poster sur le 9000 est refusé, et le montant n'apparaît ni au compte de résultat
    ni dans le report à nouveau du bilan.
13. La postabilité est déjà appliquée à la saisie (Story 14-3b) : **aucun nouveau refus n'est à
    écrire**. Un test vérifie que le refus existant se déclenche bien sur ces comptes — ⚠️ **avec
    son code d'erreur et son message actuels, sans en inventer un neuf.**

## Invariants testables

- **I1 — Aucune écriture neuve sous un compte de clôture.** Après la migration, aucune ligne de
  `journal_entry_lines` créée postérieurement ne pointe sur un compte de numéro 9000/9100/9200.
- **I2 — Le plan livré et le plan migré coïncident.** Pour les trois `org_type`, l'ensemble
  `{(numéro, postable)}` produit par le seed est **identique** à celui produit par le backfill sur
  une base contenant le plan d'origine. *C'est l'AC 10, énoncée comme propriété.*
- **I3 — Rien d'autre n'a bougé.** Le backfill modifie **exactement** les lignes de numéro
  9000/9100/9200 : un décompte avant/après sur `postable` le vérifie, et aucun `account_type`
  ne diffère.

## Tasks / Subtasks

- [ ] **T1 — Le champ du plan** (AC 1, 2, 3)
  - [ ] `postable: Option<bool>` dans `ChartEntry`, `#[serde(default)]`, doc-comment sur le
        modèle de celui de `role`
  - [ ] la troisième cause dans `is_postable`, **en tête**, et son doc-comment mis à jour —
        ⚠️ il énonce aujourd'hui « **deux** causes » : le décompte se recompte (§ *Recompter ses
        propres comptes rendus*)
  - [ ] ⛔ **et corriger sa ligne de résumé, qui est INVERSÉE** : `chart_of_accounts/mod.rs:299`
        dit *« `true` si l'entrée doit être créée non-postable »* alors que la fonction rend `true`
        pour un compte **postable**. Défaut préexistant — mais la story édite ce commentaire, et
        passer dessus sans le voir le rendrait invisible pour un an de plus
  - [ ] ⛔ **les DIX littéraux `ChartEntry { … }` existants doivent recevoir le champ** — huit
        dans `chart_of_accounts/mod.rs` (`:464`, `:480`, `:492`, `:505`, `:512`, `:526`, `:592`,
        `:657`) et **deux dans `kesh-db/src/repositories/accounts.rs`** (`:1438`, `:1450`).
        `ChartEntry` n'a **pas** de `#[derive(Default)]` et ne peut pas en avoir un
        (`account_type` n'a pas de défaut sémantique) : sans ce geste, `cargo build --workspace
        --all-targets` échoue en **`E0063 missing field`** — le deuxième des quatre checks du gate.
        ⚠️ *Le précédent le prouve : l'ajout de `role` en 14-3a a dû faire exactement cela, et
        c'est pourquoi tous ces littéraux portent aujourd'hui un `role:` explicite.*
  - [ ] tests unitaires : `Some(false)` retire ; `Some(true)` ne force ni sur un parent ni sur
        `CurrentYearResult` ; `None` se comporte comme aujourd'hui
- [ ] **T2 — Les trois plans** (AC 4, 5)
  - [ ] `"postable": false` sur 9000/9100/9200 dans `pme.json`, `association.json`,
        `independant.json` — ⚠️ **et nulle part ailleurs**
  - [ ] test : pour chaque `org_type`, exactement trois entrées portent `postable: Some(false)`
- [ ] **T3 — La migration de backfill** (AC 6, 7, 8, 9)
  - [ ] `UPDATE accounts SET postable = FALSE WHERE number IN ('9000','9100','9200') AND postable = TRUE`
        — la clause `AND postable = TRUE` rend la migration **idempotente** et son effet mesurable
  - [ ] ⛔ **P7 — exemption avec justification factuelle** (AC 9) : entrée dans
        `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`), libellée sur le **parc** et
        non sur la fenêtre. ⛔ **Ne PAS commencer par `Hors fenêtre`** — le contrôle symétrique
        recalculerait la fenêtre et rejetterait la justification, à juste titre.
  - [ ] ⛔ **Vérifier le fait AVANT d'écrire la justification**, et le citer dans le commentaire :
        `git tag --sort=-creatordate | head -1` doit rendre **v0.11.1 (2026-08-24)** ou antérieur.
        Si une version a été publiée depuis `main` après le `2026-08-27`, **la décision de D6 se
        rouvre** et l'exemption devient fausse.
  - [ ] ⛔ **NE PAS inscrire l'entrée au registre `POST_RESTORE_BACKFILLS`** : ce serait du code
        **exécuté à chaque import** pour un parc vide, et — l'entrée n'étant pas gardée — cela
        contredirait la promesse publiée du manuel d'administration (`admin-manual.tex:1609`,
        « vos données ne sont jamais écrasées »). ⚠️ *Ni l'en-tête du module ni le manuel n'ont
        donc à être amendés : la story ne les met plus en défaut.*
  - [ ] ⛔ **P5** — ligne dans `docs/migrations-idempotence-audit.md` **et les cinq compteurs**,
        recomptés depuis la source : `ls crates/kesh-db/migrations/*.sql | wc -l` (**65 → 66**),
        `grep -c '^| `20' docs/migrations-idempotence-audit.md`, l'en-tête `## Table d'audit (N
        migrations)`, la ligne `Total`, et les trois compteurs de partition dont la **somme** doit
        égaler le total — ⚠️ **ils ne valent pas le total**
  - [ ] ⛔ **Le verdict d'idempotence est `yes`, et le compteur `yes` passe de 5 à 6.**
        ⚠️ **Le critère n'est PAS « DML pur » — c'est la RE-EXÉCUTION MANUELLE SANS EFFET.**
        Quatre des cinq `yes` actuels sont du **DDL** pur, gardé par des `IF NOT EXISTS` ; ici
        c'est la clause **`AND postable = TRUE`** qui porte le verdict. *L'absence de DDL ne suffit
        pas : un `UPDATE accounts SET version = version + 1` serait du DML pur et ne serait pas
        idempotent pour deux sous.* ⇒ **si le SQL change à l'implémentation, le verdict se
        re-décide.** Le précédent à copier est `20260729000001` (`yes`), **pas** `20260722000001`,
        qui porte `tracked-by-sqlx` **parce qu'il contient aussi du DDL**.
        ⚠️ *Un verdict mal affecté laisse la somme juste et la partition fausse : recompter les
        cinq compteurs ne le détecte pas.*
  - [ ] ⛔ **P6 — DEUX nombres**, pas un : `total == 65` → **66** *et* la fenêtre `total - 31` →
        `total - 32`, frontière tenue à **34** (`crates/kesh-db/tests/migrations_upgrade_path.rs`).
        *Bumper le total seul élargirait la fenêtre en silence — la 24-4c a payé ce piège.*
  - [ ] ⛔ **P8** — ligne dans `crates/kesh-db/migrations.sha384`
  - [ ] le squash `crates/kesh-db/test-schema/0001_schema_squash.sql` régénéré
  - [ ] **P1/P3 — non breaking** : `UPDATE` de données, aucun DDL destructif ⇒ **ni bump
        `min_required`, ni bump Cargo**
- [ ] **T4 — L'invariant seed ≡ backfill** (AC 10, I2)
  - [ ] `crates/kesh-db/tests/accounts_role_backfill.rs` étendu aux trois comptes
  - [ ] ⚠️ ce test monte les migrations **par version** (`migrations_before`), jamais par
        position : ne pas y introduire de `total - N`
- [ ] **T5 — Les deux surfaces** (AC 11, 12, 13, I1, I3)
  - [ ] test d'intégration : poster sur le 9000 est refusé par la garde **existante** de 14-3b
  - [ ] test : un solde historique sur le 9000 n'apparaît **ni** au compte de résultat **ni** dans
        le **`retainedEarnings` du bilan rendu** — ⛔ *pas* dans `fetch_retained_earnings`, qui est
        **privée** : elle ne s'observe qu'au travers de `GET /api/v1/reports/balance-sheet` —
        **une fois les lignes retirées** — ⚠️ et **apparaît** tant
        qu'elles sont là : c'est ce second sens qui démontre le défaut, le premier ne démontre que
        l'absence de lignes
  - [ ] test I3 : décompte avant/après du backfill, et aucun `account_type` modifié
  - [ ] ⛔ **test de l'AC 11** : un `PUT /api/v1/accounts/{id}` avec `postable: true` sur un 9000
        que le backfill a fermé le **rouvre**. ⚠️ *C'est la moitié « réparable » de D6 : sans ce
        test, le seul geste qui rend l'écrasement acceptable n'est exercé par rien, et D6
        deviendrait faux sans que rien ne rougisse.*
- [ ] **T6 — Le manuel** (D4)
  - [ ] `docs/manual/fr/user-manual.tex` : les comptes de clôture n'accueillent pas d'écriture
        dans Kesh ; une écriture déjà passée sur un compte 9 se corrige par **contre-passation**
        (24-4a), non par réécriture (24-4b)
  - [ ] ⛔ **et montrer la porte ouverte, pas seulement celle qu'on ferme** : la voie prévue pour
        un bilan d'ouverture de migration est l'écran **Administration → Soldes de départ**
        (`routes/opening_balances`) — ⚠️ **« Administration », pas « Réglages »** : c'est le groupe
        du sidebar (`+layout.svelte`, `nav-administration`) et la formulation que `user-manual.tex:562`
        emploie **déjà** pour ce même écran ; le **2970** en est la contrepartie comptable, **pas la
        procédure**. ⚠️ *Fermer un chemin sans montrer l'autre est exactement le reproche que
        cette story adresse au correctif annoncé par l'issue.* ✅ Vérifié : la grille des soldes de
        départ ne prend que les comptes de bilan (`Asset`/`Liability`), donc les 9xxx en sont
        **déjà** exclus — la story ne casse rien de ce côté
  - [ ] PDF régénéré (`make fr` dans `docs/manual/`) et commité
  - [ ] ✅ **RIEN à corriger dans le manuel d'administration** : D6 ayant retenu l'exemption plutôt
        que la classe A, la promesse *« vos données ne sont jamais écrasées »*
        (`admin-manual.tex:1609`) **reste vraie**. ⚠️ *C'est le meilleur argument en faveur de
        l'exemption : la décision qui n'oblige à réécrire aucune garantie publiée est celle qui
        n'en casse aucune.*
  - [ ] ⚠️ **la limite du critère « numéro »**, sur le modèle de celle que le backfill de rôles
        énonce déjà — ⛔ **`20260722000001:115-118`**, le bloc « LIMITE ASSUMÉE » ; `:110-113`
        défend la *licéité* du numéro en migration, c'est-à-dire le gabarit de l'**AC 6**, pas
        celui de la limite : un utilisateur ayant **réaffecté** le numéro 9000
        à un autre usage verra ce compte fermé, sans erreur. *Exposition faible —
        `uq_accounts_company_number` n'est pas filtrée sur `active`, donc un numéro semé garde son
        compte à vie — mais le précédent la documente, et l'omettre ici serait une régression de
        complétude.*
  - [ ] ⛔ **Aucun gate ne lit le manuel** — leçon de la 24-4a, appliquée ici *avant* le fait
- [ ] **T7 — La limitation L1** (D3)
  - [ ] issue GitHub `[FEATURE]` : l'arbre affiche les comptes de clôture sous un type Charge ;
        exige un variant d'`AccountType`, donc une migration **breaking**. Labels `enhancement`,
        `v0.2-milestone` ⇒ **catégorie B tracée** au sens de la politique zero carry-forward
  - [ ] la limitation écrite dans les Dev Notes avec son numéro d'issue
- [ ] **T8 — Les gates** (⛔ **complets, ciblage interdit** — migration *et* repository)
  - [ ] base remise à zéro **avant** (inconditionnel — gate interrompu, gate terminé, **ou
        conteneur redémarré**)
  - [ ] `cargo fmt` · `clippy --workspace --all-targets -D warnings` · `scripts/test-fast.sh`
  - [ ] frontend : `check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [ ] Playwright **complet**, `kesh_e2e` **reconstruite** — le rouge se juge fichier par fichier
        contre `docs/testing.md` § « Les échecs attendus », **jamais au nombre**

## Hors périmètre

- **Le variant d'`AccountType`** — limitation L1, issue de suivi (T7).
- **La correction des écritures déjà passées** sur un compte 9 : elle appartient à l'utilisateur,
  par contre-passation (D4).
- **Les autres incohérences de type du plan** signalées en passant par l'issue #375 : `6950`
  (Revenue sous un parent Expense) et `8010`/`8900` (Expense sous `80` typé Revenue).
  ⚠️ **L'issue les décrit de travers** — elle dit « le parent `8` » alors que le parent direct de
  `8010` et `8900` est **`80`**, lui-même sous `8`. Sans effet sur les chiffres (les rapports
  ignorent la hiérarchie) ; à verser à l'issue L1, qui porte déjà le sujet du typage.
- **Toute notion de clôture d'exercice par écritures** : Kesh calcule en temps réel virtuel
  (14-1), et cette story ne remet pas ce modèle en cause — elle en tire la conséquence.

## Dev Notes

### Règle de splitting — examinée, non déclenchée

Modules touchés : `kesh-core/chart_of_accounts`, `kesh-db` (migration, registre de rejeu, test
d'invariant), **`kesh-api`** et `docs/manual`. **Quatre**, sous le seuil de cinq. Aucun frontend,
**aucune clé i18n** — la garde de 14-3b et son message existent déjà (AC 13).

⛔ **Pourquoi `kesh-api` et non `kesh-report`, et le motif n'est pas celui qu'on croit.**
`crates/kesh-report/tests/` **existe** et monte le schéma (six fichiers, `#[sqlx::test]`) : passer
du test unitaire au test d'intégration ne fait donc pas sortir de `kesh-report`. La vraie raison
est que **`fetch_retained_earnings` est privée** (`balance_sheet.rs:333`, pas de `pub`) : le report
à nouveau ne s'observe qu'**au travers du rapport rendu**, donc par la route HTTP.

⚠️ **Le décompte a été recompté, et il était faux à trois.** *Un décompte qu'on écrit sans le
refaire est le défaut que le § « Recompter ses propres comptes rendus » vise ; il ne devient pas
vrai parce qu'il reste sous le seuil.*

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-core/src/chart_of_accounts/mod.rs` | UPDATE — `ChartEntry.postable`, `is_postable`, **8 littéraux de test** |
| `crates/kesh-db/src/repositories/accounts.rs` | UPDATE — **2 littéraux `ChartEntry`** (`:1438`, `:1450`), sans quoi le build casse |
| `crates/kesh-core/assets/charts/{pme,association,independant}.json` | UPDATE — 3 × 3 annotations |
| `crates/kesh-db/migrations/<date>_closing_accounts_not_postable.sql` | NEW — le backfill |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE — P8 + squash |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | UPDATE — P6, **deux** nombres |
| `docs/migrations-idempotence-audit.md` | UPDATE — P5, ligne + **cinq** compteurs |
| `crates/kesh-db/src/post_restore.rs` | UPDATE — P7, **une entrée d'exemption** justifiée sur le parc (D6) ; ni registre, ni extrait `.sql`, ni en-tête à amender |
| `crates/kesh-db/tests/accounts_role_backfill.rs` | UPDATE — l'invariant seed ≡ backfill |
| `crates/kesh-api/tests/reports_e2e.rs` | UPDATE — le test de bout en bout de l'AC 12 (les tests de `income_statement.rs` ne montent **aucune** base) |
| `docs/manual/fr/user-manual.tex` (+ PDF) | UPDATE — T6, la conduite à tenir (D4) |

### Pièges vérifiés au sol

- ⛔ **`bulk_create_from_chart` binde `postable`** (`accounts.rs:842`) depuis `is_postable` : le
  seed suivra donc **automatiquement** le champ neuf. Rien d'autre à câbler côté seed.
- ⛔ **`accounts.account_type` est un `VARCHAR(20)` + `CHECK chk_accounts_type`**, pas un ENUM
  MySQL — c'est ce qui rendrait un variant neuf coûteux mais faisable ; sans objet ici (D3).
- ⚠️ **L'enum `AccountType` existe en DOUBLE** (`kesh-core` et `kesh-db`), par l'orphan rule, et
  le doc-comment de `kesh-db/src/entities/account.rs:84` le documente. Ne pas tenter de fusionner.
- ⚠️ **`income_statement.rs:65` porte un `_ => unreachable!()`** sur `AccountType`. Sans effet
  aujourd'hui (deux appelants, tous deux Revenue/Expense), mais c'est l'anti-pattern que le
  CLAUDE.md proscrit § *Pattern batch*. **Hors périmètre**, à signaler si une passe le relève.
- ⚠️ **Le compte `2206 Décompte TVA` existe** dans les trois plans : ne pas le confondre avec les
  comptes de clôture au moment d'écrire les fixtures.
- ⛔ **`association.json` compte 81 entrées, les deux autres 84** — un test qui affirmerait « le
  même nombre de comptes dans les trois plans » serait faux.

### Limitations

- **L1 — le type des comptes de clôture reste `Expense`** (D3). Sans effet sur les chiffres une
  fois les comptes non postables ; l'arbre affiché reste visuellement faux. Issue de suivi à
  ouvrir en T7, labels `enhancement` + `v0.2-milestone` ⇒ catégorie **B** tracée.

### References

- [Source: GitHub issue #375] — le défaut, le scénario de la persona, le correctif annoncé
- [Source: `crates/kesh-core/src/chart_of_accounts/mod.rs:299-327`] — `is_postable` et sa doctrine
- [Source: `crates/kesh-db/migrations/20260722000001_accounts_role_postable.sql:1-40`] — le
  précédent de backfill par numéro, et la règle « aucun numéro dans le code applicatif »
- [Source: `crates/kesh-db/tests/accounts_role_backfill.rs:1-45`] — l'invariant seed ≡ backfill
- [Source: `crates/kesh-report/src/income_statement.rs:60-90`] — la première surface
- [Source: `crates/kesh-report/src/balance_sheet.rs:333-356`] — la seconde surface
- [Source: `_bmad-output/planning-artifacts/epic-24-vague1-livres-justes.md`] — la vague et son
  critère de clôture
- [Source: CLAUDE.md § *Migration breaking policy*] — P1, P3, P5, P6, P7, P8

## Journal de revue

### Passe 1 — 2026-09-09 · Sonnet 4.6 + Haiku 4.5, contextes frais, orthogonales à l'auteur (Opus 5)

**1 CRITICAL, 1 HIGH, 3 MEDIUM, 1 LOW** retenus — et les deux lentilles **convergent** sur le
CRITICAL, Haiku l'ayant atteint par l'axe du triage et Sonnet par celui des décisions.

⛔ **Le CRITICAL était écrit dans le dépôt depuis juillet, sur le gabarit même que la spec
citait.** `post_restore.rs:257-260` dit du backfill de rôles : *« les 2 `UPDATE` de `postable` ne
portent AUCUNE garde — rejoués sur une base à jour, ils écraseraient un `postable` posé à la
main. »* La spec traitait P7 comme une case à cocher (« inscrire ou exempter ») alors qu'**aucune
des trois issues n'était propre** : l'exemption « hors fenêtre » est **fausse** (la migration est
la dernière du dépôt, donc dans la fenêtre), la classe B est **indisponible** (`UPDATE` pur, donc
aucune sentinelle constructible), et la classe A ne satisfait pas son propre critère. ⇒ **D6**.

⚠️ **La décision prise ici a été RENVERSÉE en passe 3, et la trace se garde.** D6 tranchait alors
pour la **classe A**, sur l'asymétrie des coûts. La passe 3 a établi deux faits qui l'ont défaite :
le manuel d'administration **promet** que les données ne sont jamais écrasées, et **aucune version
publiée** ne se trouve dans l'intervalle que le rejeu aurait protégé. *L'analyse des trois issues
reste juste ; c'est la conclusion qu'on en tirait qui était trop chère.*

⚠️ **La garde `AND version = 1` a été examinée et écartée dans la remédiation** : un utilisateur
ayant seulement **renommé** son compte porte `version > 1` sans avoir touché `postable`, et le
rejeu le sauterait — un faux négatif **silencieux**, c'est-à-dire le mode d'échec qu'on ferme,
réintroduit par sa propre garde.

⛔ **Le HIGH est mécanique et aurait cassé le gate au deuxième check** : `ChartEntry` n'a pas de
`#[derive(Default)]`, et **dix littéraux** le construisent champ par champ — huit dans
`chart_of_accounts/mod.rs`, **deux dans `kesh-db/src/repositories/accounts.rs`**, fichier que la
table « Fichiers à toucher » ne nommait pas. Sans eux, `cargo build --workspace --all-targets`
rend `E0063 missing field`.

⚠️ **Deux MEDIUM portaient sur des comptes rendus de la spec, pas sur sa conception** : le
décompte de modules disait « trois » en omettant `kesh-api`, que l'AC 12 exige — ⚠️ *le motif
alors avancé, « les tests d'`income_statement.rs` ne montent aucune base », était un
non-séquitur : `kesh-report/tests/` existe et monte le schéma. La vraie raison, établie en passe 2,
est que `fetch_retained_earnings` est **privée*** ; et le
verdict d'idempotence de la ligne d'audit n'était pas spécifié, avec un gabarit qui **induisait en
erreur** — `20260722000001` porte `tracked-by-sqlx` **parce qu'il contient du DDL**, quand notre
migration relève du `yes` de `20260729000001` — ⚠️ *non parce qu'elle est du DML pur, motif
énoncé alors et corrigé en passe 2, mais parce que sa clause `AND postable = TRUE` la rend
rejouable sans effet.* *Un verdict mal affecté laisse la somme
juste et la partition fausse : recompter les cinq compteurs ne le détecte pas.*

✅ **Ce que la propagation post-patch a rattrapé, et que les deux lentilles n'avaient pas vu** :
le grep du symptôme sur tout le document a trouvé un **jumeau** — l'ancienne ligne de la table des
fichiers disait encore « registre **ou exemption justifiée** », ce que D6 venait de réfuter — et
deux renvois devenus faux par la renumérotation. *C'est le motif que le § « Propagation
post-patch » vise, attrapé par le geste et non par une lentille.*

⚠️ **Réserve de la lentille Sonnet, à porter** : elle n'a pas pu exécuter le gate (pas de MariaDB
dans son environnement), et n'a pas vérifié exhaustivement que **aucun réglage ne pointe sur un
compte 9** (D4) — jugé plausible, non prouvé. À traiter en passe 2 ou à l'implémentation.

**Prochaine** : passe 2 **ciblée** sur le seul commit de remédiation, contexte frais, modèle
différent, prompt versionné.

### Passe 2 — 2026-09-09 · Opus 5, contexte frais, CIBLÉE sur `99dce529`, prompt versionné

**0 CRITICAL, 1 HIGH, 8 MEDIUM, 4 LOW — et ONZE findings sur TREIZE naissent de la remédiation
de la passe 1.** Aucune décision antérieure à cette remédiation n'a été prise en défaut. La
sévérité recule (CRITICAL → HIGH) : convergence lente mais monotone, donc **pas de split**.

⛔ **Le HIGH ne porte pas sur une erreur de raisonnement mais sur une RÈGLE ENFREINTE SANS
DÉROGATION DÉCLARÉE.** D6 écrivait « le critère n'est pas satisfait » puis tranchait quand même
pour la classe A — or P7 dit « classe A **uniquement si** ». Le patch avait vu le problème,
l'avait bien analysé, **et avait omis de dire qu'il sortait de la règle**. ⚠️ Second volet, que
personne n'avait vu : l'**en-tête du module** (`post_restore.rs:38-40`) définit la classe A comme
*« auto-gardée… tous ses statements sont gardés »* — après la story, il **mentirait sur son propre
registre**. *C'est le motif de la 24-4c — corriger la thèse au site nommé et laisser ses
applications ailleurs — transposé du story file au code.*

⛔ **Deux voies de garde n'avaient PAS été examinées**, et l'énumération se donnait pour close.
La plus sérieuse : `audit_log` **est restaurée** et `account_snapshot_json` y écrit `postable`
depuis la 14-3a — une garde d'intention y est donc **constructible**, contrairement au proxy
`version` qu'on avait écarté à juste titre. Elle est désormais examinée **et rejetée par écrit**,
sur le coût. *« Une hypothèse éliminée par raisonnement n'est pas une hypothèse testée » vaut
aussi pour celles qu'on n'énonce pas.*

⛔ **Le critère du verdict d'idempotence était faux, et il l'était dans la puce qui met en garde
contre les verdicts mal affectés.** La spec écrivait « DML pur, sans DDL ⇒ `yes` » ; or **quatre
des cinq `yes` du dépôt sont du DDL pur**, gardés par `IF NOT EXISTS`. Le vrai critère est la
**re-exécution manuelle sans effet**. Le verdict retenu restait juste, mais pour la mauvaise
raison — et il aurait cessé de l'être si le SQL changeait à l'implémentation.

⚠️ **Trois autres findings sont des renvois que la remédiation a CRÉÉS** : un numéro de ligne
(`:110-113` défend la licéité du numéro, la limite est à `:115-118`), une exigence sur
`admin-manual.tex` absente de la table des fichiers, et une étiquette `(D4)` sur une tâche à qui
D6 venait de confier sa réserve.

⛔ **Et l'AC 11 n'était couverte par AUCUNE tâche** — la seule des treize. La remédiation l'avait
transformée d'assertion de doctrine en affirmation vérifiable, **et l'avait rendue portante pour
D6** : c'est la moitié « réparable » de l'asymétrie. *Le seul geste qui rend l'écrasement
acceptable n'était exercé par rien : D6 pouvait devenir faux sans que rien ne rougisse.*

⚠️ **L'asymétrie de D6 a été qualifiée plutôt que défendue.** « Bruyant » veut dire « à la
prochaine tentative de saisie » — or le 9000 sert **une fois l'an**. Restauration en mars,
symptôme en janvier. Le rapport de rejeu ne nomme aucun compte et son `rows_affected` est déclaré
« informatif ». L'arbitrage reste le bon ; sa justification était plus faible qu'écrite.

✅ **Ce que la passe 2 a tranché et que la passe 1 avait laissé ouvert** : **D4 est prouvée, plus
seulement plausible** — aucun réglage ne pointe sur un compte de la classe 9 (les colonnes de réglage pointant un compte
ont été recensées une à une, `company_dunning_settings` n'en porte aucune, et les trois écrans
passent par `account-options.ts`, le correctif de #271). ⚠️ *Le décompte « neuf FK » d'abord écrit
ici était faux et mal nommé — `bank_accounts.journal_account_id` est documenté comme n'ayant
volontairement pas de FK au niveau base.*

✅ **Le grep de propagation a de nouveau payé, et sur le compte rendu lui-même** : la table des
fichiers prescrivait encore un `.sql` neuf sous `src/post_restore/`, et **le journal de la passe 1
portait le non-séquitur de P2-8 dans le paragraphe même où il se félicitait d'avoir attrapé un
jumeau.**

**Prochaine** : passe 3 ciblée sur le seul commit de remédiation, contexte frais, modèle
différent, prompt versionné.

### Passe 3 — 2026-09-09 · Sonnet 4.6, contexte frais, CIBLÉE sur `3740f456`, prompt versionné

**0 CRITICAL, 1 HIGH, 2 MEDIUM, 1 LOW** — le volume s'effondre (13 → 4), et **trois findings sur
quatre naissent du texte que la passe 2 venait d'écrire**. Aucune décision antérieure à cette
remédiation n'est prise en défaut.

⛔ **LE HIGH A RENVERSÉ LA DÉCISION, et il l'a fait avec un fichier que deux passes n'avaient pas
ouvert : le manuel.** `admin-manual.tex:1609` promet, comme **première** des trois propriétés du
rejeu : *« Vos données ne sont jamais écrasées. […] un réglage que vous aviez ajusté à la main est
conservé tel quel. »* La classe A ne cassait donc pas un commentaire de module — elle rendait
**fausse une garantie publiée**. Le manuel, l'en-tête du module et le critère de P7 disaient la
même chose à trois endroits indépendants ; la dérogation devait les contredire tous les trois.

⛔ **Et le second fait a achevé la décision** : le rejeu protégeait **un parc vide**. L'intervalle
concerné est `[20260827000001 … cette migration)`, or la dernière version publiée est **v0.11.1 du
2026-08-24** — antérieure à la borne basse. *On s'apprêtait à enfreindre une règle et à démentir
une promesse pour couvrir des binaires qui n'existent pas.*

⇒ **D6 refondue : exemption avec justification factuelle sur le PARC**, et non classe A par
dérogation. ✅ **Trois findings tombent par la racine** : plus de dérogation à mettre en forme
(P3-3 sans objet), plus d'en-tête de module à amender, plus de promesse du manuel à réécrire.
*La décision qui n'oblige à réécrire aucune garantie publiée est celle qui n'en casse aucune.*

⚠️ **Mais la justification se périme, donc elle se contrôle** — deux fois, en T3 et au gate de
clôture. Une exemption fausse désactive le rejeu *définitivement et en silence* ; c'est ce que le
CLAUDE.md dit de l'exemption : « l'issue la moins coûteuse, donc celle qu'il faut contrôler ».

⛔ **Une question reste ouverte et n'est déductible d'AUCUN fichier** : l'instance du NAS
exécute-t-elle un binaire postérieur au 2026-08-27 ? Si oui, elle est le seul membre du parc
concerné et la décision se rouvre. *Portée à l'arbitrage du Project Lead.*

⚠️ **Le MEDIUM restant est de la même famille que le HIGH** : la remédiation de la passe 2 écrivait
« Réglages → Soldes de départ » quand le sidebar range cet écran sous **Administration** — et le
manuel utilisateur l'écrit **déjà correctement** 180 lignes plus haut. *Deux fois de suite, le
défaut était dans un fichier que la spec prescrivait de modifier sans l'avoir lu.*

**Trend** : 6 → 13 → 4. Sévérité maximale : CRITICAL → HIGH → HIGH.
⚠️ **La sévérité maximale STAGNE, et la règle de splitting préventif en fait un signal.** Elle
n'est pas retenue ici, et le motif s'écrit : le **volume** s'effondre, les findings portent
**exclusivement** sur la dernière remédiation, aucune décision de conception d'origine n'a été
prise en défaut en trois passes, et le HIGH de la passe 3 n'est pas une régression mais une
**omission de lecture** — le manuel. *C'est le cas que l'amendement de la rétro Epic 14 vise :
une convergence lente n'est pas une non-convergence.* **Arbitrage porté au Project Lead.**

**Prochaine** : passe 4, contexte frais, modèle différent, prompt versionné.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
