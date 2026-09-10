# Story 24.5 : Les comptes de clôture — fermer le compte où l'à-nouveau se perd

## Status

review

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

✅ **TRANCHÉ — arbitrage de Guy, 2026-09-09 : l'instance du NAS exécute v0.11.1**, publiée le
2026-08-24, donc **antérieure** à la borne du `20260827000001`. Le parc réel — la seule
installation en service du projet — est **hors de l'intervalle**, et l'exemption est établie sur
un fait, non sur une présomption.

⚠️ **Cette réponse ne se déduisait d'aucun fichier du dépôt** : ni un tag, ni un manifeste ne dit
ce qui tourne sur une machine. C'est pourquoi elle a été **demandée** plutôt que supposée — et
c'est aussi pourquoi le contrôle de T3/T8 porte sur les **tags publiés**, seule part du fait qui
soit vérifiable par commande.

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
    son code d'erreur actuel, sans en inventer un neuf.**

    *(Amendée en passe 2 de revue de code, finding P2-5. Le libellé d'origine disait « son code
    d'erreur **et son message** actuels », et le test n'assertait que le code — l'AC était donc
    déclarée tenue sur une moitié de ce qu'elle demandait. Ce n'est pas le test qu'on corrige,
    c'est l'AC : le message passe par `t("error-inactive-accounts", …)` et **change avec la
    locale**, si bien que l'asserter attacherait le test à une traduction. Le patron du dépôt
    tranche déjà ainsi — `grep -cF '["error"]["message"]' crates/kesh-api/tests/reports_e2e.rs`
    rend **0** sur les quatre refus qu'y assertent déjà leur code. Le comportement exigé est
    inchangé : aucun changement de périmètre, donc pas de CR.)*

## Invariants testables

- **I1 — Aucune écriture neuve sous un compte de clôture, PAR LA SAISIE MANUELLE.**
  ⛔ **Amendé en passe 1 de revue de code (BH-1), et l'énoncé d'origine était FAUX.** Il disait
  « aucune écriture neuve », sans réserve. Or les trois flux de réconciliation
  (`post_manual`, `accept_one_split`, `accept_one_rule`) appellent `create_in_tx(..., false)` —
  `enforce_postable = false` — et leurs gardes ne vérifient que `active` : `grep -nF postable
  crates/kesh-api/src/routes/reconciliation.rs` rend **zéro**. Un rapprochement dont l'utilisateur
  désigne lui-même un compte 9000 comme contrepartie **passe**.
  ⇒ L'invariant tient pour la **saisie manuelle**, qui est le vecteur nommé par l'issue #375 et le
  seul que la garde de la 14-3b couvre. La fermeture des flux automatiques est **hors périmètre**
  et tracée par l'issue **#427**.
  ⚠️ *L'énoncé faux avait une conséquence concrète : le manuel, modifié par cette même story,
  affirmait « n'acceptent aucune écriture » à sept lignes d'un paragraphe préexistant disant « les
  flux automatiques ne sont pas concernés ». La contradiction a été introduite par la story et
  corrigée par la revue.*

- **I2 — Le plan livré et le plan migré coïncident.** Pour les trois `org_type`, l'ensemble
  `{(numéro, postable)}` produit par le seed est **identique** à celui produit par le backfill sur
  une base contenant le plan d'origine. *C'est l'AC 10, énoncée comme propriété.*
- **I3 — Rien d'autre n'a bougé.** Le backfill modifie **exactement** les lignes de numéro
  9000/9100/9200 : un décompte avant/après sur `postable` le vérifie, et aucun `account_type`
  ne diffère.

## Tasks / Subtasks

- [x] **T1 — Le champ du plan** (AC 1, 2, 3)
  - [x] `postable: Option<bool>` dans `ChartEntry`, `#[serde(default)]`, doc-comment sur le
        modèle de celui de `role`
  - [x] la troisième cause dans `is_postable`, **en tête**, et son doc-comment mis à jour —
        ⚠️ il énonce aujourd'hui « **deux** causes » : le décompte se recompte (§ *Recompter ses
        propres comptes rendus*)
  - [x] ⛔ **et corriger sa ligne de résumé, qui est INVERSÉE** : `chart_of_accounts/mod.rs:299`
        dit *« `true` si l'entrée doit être créée non-postable »* alors que la fonction rend `true`
        pour un compte **postable**. Défaut préexistant — mais la story édite ce commentaire, et
        passer dessus sans le voir le rendrait invisible pour un an de plus
  - [x] ⛔ **les DIX littéraux `ChartEntry { … }` existants doivent recevoir le champ** — huit
        dans `chart_of_accounts/mod.rs` (`:464`, `:480`, `:492`, `:505`, `:512`, `:526`, `:592`,
        `:657`) et **deux dans `kesh-db/src/repositories/accounts.rs`** (`:1438`, `:1450`).
        `ChartEntry` n'a **pas** de `#[derive(Default)]` et ne peut pas en avoir un
        (`account_type` n'a pas de défaut sémantique) : sans ce geste, `cargo build --workspace
        --all-targets` échoue en **`E0063 missing field`** — le deuxième des quatre checks du gate.
        ⚠️ *Le précédent le prouve : l'ajout de `role` en 14-3a a dû faire exactement cela, et
        c'est pourquoi tous ces littéraux portent aujourd'hui un `role:` explicite.*
  - [x] tests unitaires : `Some(false)` retire ; `Some(true)` ne force ni sur un parent ni sur
        `CurrentYearResult` ; `None` se comporte comme aujourd'hui
- [x] **T2 — Les trois plans** (AC 4, 5)
  - [x] `"postable": false` sur 9000/9100/9200 dans `pme.json`, `association.json`,
        `independant.json` — ⚠️ **et nulle part ailleurs**
  - [x] test : pour chaque `org_type`, exactement trois entrées portent `postable: Some(false)`
- [x] **T3 — La migration de backfill** (AC 6, 7, 8, 9)
  - [x] `UPDATE accounts SET postable = FALSE WHERE number IN ('9000','9100','9200') AND postable = TRUE`
        — la clause `AND postable = TRUE` rend la migration **idempotente** et son effet mesurable
  - [x] ⛔ **P7 — exemption avec justification factuelle** (AC 9) : entrée dans
        `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs`), libellée sur le **parc** et
        non sur la fenêtre. ⛔ **Ne PAS commencer par `Hors fenêtre`** — le contrôle symétrique
        recalculerait la fenêtre et rejetterait la justification, à juste titre.
  - [x] ⛔ **Vérifier le fait AVANT d'écrire la justification**, et le citer dans le commentaire :
        `git tag --sort=-creatordate | head -1` doit rendre **v0.11.1 (2026-08-24)** ou antérieur.
        Si une version a été publiée depuis `main` après le `2026-08-27`, **la décision de D6 se
        rouvre** et l'exemption devient fausse.
  - [x] ⛔ **NE PAS inscrire l'entrée au registre `POST_RESTORE_BACKFILLS`** : ce serait du code
        **exécuté à chaque import** pour un parc vide, et — l'entrée n'étant pas gardée — cela
        contredirait la promesse publiée du manuel d'administration (`admin-manual.tex:1609`,
        « vos données ne sont jamais écrasées »). ⚠️ *Ni l'en-tête du module ni le manuel n'ont
        donc à être amendés : la story ne les met plus en défaut.*
  - [x] ⛔ **P5** — ligne dans `docs/migrations-idempotence-audit.md` **et les cinq compteurs**,
        recomptés depuis la source : `ls crates/kesh-db/migrations/*.sql | wc -l` (**65 → 66**),
        `grep -c '^| `20' docs/migrations-idempotence-audit.md`, l'en-tête `## Table d'audit (N
        migrations)`, la ligne `Total`, et les trois compteurs de partition dont la **somme** doit
        égaler le total — ⚠️ **ils ne valent pas le total**
  - [x] ⛔ **Le verdict d'idempotence est `yes`, et le compteur `yes` passe de 5 à 6.**
        ⚠️ **Le critère n'est PAS « DML pur » — c'est la RE-EXÉCUTION MANUELLE SANS EFFET.**
        Quatre des cinq `yes` actuels sont du **DDL** pur, gardé par des `IF NOT EXISTS` ; ici
        c'est la clause **`AND postable = TRUE`** qui porte le verdict. *L'absence de DDL ne suffit
        pas : un `UPDATE accounts SET version = version + 1` serait du DML pur et ne serait pas
        idempotent pour deux sous.* ⇒ **si le SQL change à l'implémentation, le verdict se
        re-décide.** Le précédent à copier est `20260729000001` (`yes`), **pas** `20260722000001`,
        qui porte `tracked-by-sqlx` **parce qu'il contient aussi du DDL**.
        ⚠️ *Un verdict mal affecté laisse la somme juste et la partition fausse : recompter les
        cinq compteurs ne le détecte pas.*
  - [x] ⛔ **P6 — DEUX nombres**, pas un : `total == 65` → **66** *et* la fenêtre `total - 31` →
        `total - 32`, frontière tenue à **34** (`crates/kesh-db/tests/migrations_upgrade_path.rs`).
        *Bumper le total seul élargirait la fenêtre en silence — la 24-4c a payé ce piège.*
  - [x] ⛔ **P8** — ligne dans `crates/kesh-db/migrations.sha384`
  - [x] le squash `crates/kesh-db/test-schema/0001_schema_squash.sql` régénéré
  - [x] **P1/P3 — non breaking** : `UPDATE` de données, aucun DDL destructif ⇒ **ni bump
        `min_required`, ni bump Cargo**
- [x] **T4 — L'invariant seed ≡ backfill** (AC 10, I2)
  - [x] `crates/kesh-db/tests/accounts_role_backfill.rs` étendu aux trois comptes
  - [x] ⚠️ ce test monte les migrations **par version** (`migrations_before`), jamais par
        position : ne pas y introduire de `total - N`
- [x] **T5 — Les deux surfaces** (AC 11, 12, 13, I1, I3)
  - [x] test d'intégration : poster sur le 9000 est refusé par la garde **existante** de 14-3b
  - [x] test : un solde historique sur le 9000 n'apparaît **ni** au compte de résultat **ni** dans
        le **`retainedEarnings` du bilan rendu** — ⛔ *pas* dans `fetch_retained_earnings`, qui est
        **privée** : elle ne s'observe qu'au travers de `GET /api/v1/reports/balance-sheet` —
        **une fois les lignes retirées** — ⚠️ et **apparaît** tant
        qu'elles sont là : c'est ce second sens qui démontre le défaut, le premier ne démontre que
        l'absence de lignes
  - [x] test I3 : décompte avant/après du backfill, et aucun `account_type` modifié
  - [x] ⛔ **test de l'AC 11** : un `PUT /api/v1/accounts/{id}` avec `postable: true` sur un 9000
        que le backfill a fermé le **rouvre**. ⚠️ *C'est la moitié « réparable » de D6 : sans ce
        test, le seul geste qui rend l'écrasement acceptable n'est exercé par rien, et D6
        deviendrait faux sans que rien ne rougisse.*
- [x] **T6 — Le manuel** (D4)
  - [x] `docs/manual/fr/user-manual.tex` : les comptes de clôture n'accueillent pas d'écriture
        dans Kesh ; une écriture déjà passée sur un compte 9 se corrige par **contre-passation**
        (24-4a), non par réécriture (24-4b)
  - [x] ⛔ **et montrer la porte ouverte, pas seulement celle qu'on ferme** : la voie prévue pour
        un bilan d'ouverture de migration est l'écran **Administration → Soldes de départ**
        (`routes/opening_balances`) — ⚠️ **« Administration », pas « Réglages »** : c'est le groupe
        du sidebar (`+layout.svelte`, `nav-administration`) et la formulation que `user-manual.tex:562`
        emploie **déjà** pour ce même écran ; le **2970** en est la contrepartie comptable, **pas la
        procédure**. ⚠️ *Fermer un chemin sans montrer l'autre est exactement le reproche que
        cette story adresse au correctif annoncé par l'issue.* ✅ Vérifié : la grille des soldes de
        départ ne prend que les comptes de bilan (`Asset`/`Liability`), donc les 9xxx en sont
        **déjà** exclus — la story ne casse rien de ce côté
  - [x] PDF régénéré (`make fr` dans `docs/manual/`) et commité
  - [x] ✅ **RIEN à corriger dans le manuel d'administration** : D6 ayant retenu l'exemption plutôt
        que la classe A, la promesse *« vos données ne sont jamais écrasées »*
        (`admin-manual.tex:1609`) **reste vraie**. ⚠️ *C'est le meilleur argument en faveur de
        l'exemption : la décision qui n'oblige à réécrire aucune garantie publiée est celle qui
        n'en casse aucune.*
  - [x] ⚠️ **la limite du critère « numéro »**, sur le modèle de celle que le backfill de rôles
        énonce déjà — ⛔ **`20260722000001:115-118`**, le bloc « LIMITE ASSUMÉE » ; `:110-113`
        défend la *licéité* du numéro en migration, c'est-à-dire le gabarit de l'**AC 6**, pas
        celui de la limite : un utilisateur ayant **réaffecté** le numéro 9000
        à un autre usage verra ce compte fermé, sans erreur. *Exposition faible —
        `uq_accounts_company_number` n'est pas filtrée sur `active`, donc un numéro semé garde son
        compte à vie — mais le précédent la documente, et l'omettre ici serait une régression de
        complétude.*
  - [x] ⛔ **Aucun gate ne lit le manuel** — leçon de la 24-4a, appliquée ici *avant* le fait
- [x] **T7 — La limitation L1** (D3)
  - [x] issue GitHub `[FEATURE]` : l'arbre affiche les comptes de clôture sous un type Charge ;
        exige un variant d'`AccountType`, donc une migration **breaking**. Labels `enhancement`,
        `v0.2-milestone` ⇒ **catégorie B tracée** au sens de la politique zero carry-forward
  - [x] la limitation écrite dans les Dev Notes avec son numéro d'issue
- [x] **T8 — Les gates** (⛔ **complets, ciblage interdit** — migration *et* repository)
  - [x] base remise à zéro **avant** (inconditionnel — gate interrompu, gate terminé, **ou
        conteneur redémarré**)
  - [x] `cargo fmt` · `clippy --workspace --all-targets -D warnings` · `scripts/test-fast.sh`
  - [x] frontend : `check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [x] Playwright **complet**, `kesh_e2e` **reconstruite** — le rouge se juge fichier par fichier
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

✅ **La question ouverte de cette passe a été tranchée le jour même** : l'instance du NAS
exécute **v0.11.1** (2026-08-24), donc hors de l'intervalle — arbitrage de Guy. L'exemption repose
désormais sur un fait établi, et non sur une présomption favorable.

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

### Passe 4 — 2026-09-09 · Haiku 4.5, contexte frais, CIBLÉE sur `85b83904`..`4ccddb19`

**0 finding, toutes sévérités confondues.** Le critère d'arrêt de la Review Iteration Rule est
atteint.

✅ **Et cette passe a réellement travaillé, ce qui n'allait pas de soi.** Le dépôt garde le
précédent d'un zéro de complaisance : en passe 1 de la 24-4c, la même lentille avait rendu zéro en
vérifiant *« par grep ou lecture directe DE LA SPEC »* — elle avait contrôlé que la spec disait ce
que la spec disait, et déclaré traité le cas où se cachait le premier CRITICAL. **Ici les
vérifications portent sur le CODE et les FICHIERS TIERS** : `git tag` et les dates de publication,
`admin-manual.tex:1609`, `user-manual.tex:562`, le sidebar `nav-administration`,
`chart_of_accounts/mod.rs:299`, `post_restore.rs:257-260`, les deux blocs de
`20260722000001` (`:110-113` licéité / `:115-118` limite), et les **dix littéraux `ChartEntry`**
retrouvés un par un aux lignes annoncées.

✅ **Le point le plus utile de cette passe est négatif** : après **deux renversements** de D6 —
classe A avec dérogation, puis exemption sur le parc —, elle n'a trouvé **aucune trace résiduelle**
de l'ancienne décision dans les treize AC, les huit tâches, les Dev Notes ni les journaux. *C'était
le risque principal de cette remédiation, et il ne s'est pas réalisé.*

### BOUCLE DE REVUE DE SPEC CLOSE — 2026-09-09

| passe | modèle | CRIT | HIGH | MED | LOW | total |
|---|---|---|---|---|---|---|
| 1 | Sonnet 4.6 + Haiku 4.5 | 1 | 1 | 3 | 1 | **6** |
| 2 | Opus 5 *(ciblée)* | 0 | 1 | 8 | 4 | **13** |
| 3 | Sonnet 4.6 *(ciblée)* | 0 | 1 | 2 | 1 | **4** |
| 4 | Haiku 4.5 *(ciblée)* | 0 | 0 | 0 | 0 | **0** |

**Trend : 6 → 13 → 4 → 0.** Sévérité maximale : CRITICAL → HIGH → HIGH → rien. Rotation complète
Sonnet+Haiku → Opus → Sonnet → Haiku, quatre contextes frais, trois prompts versionnés.

⛔ **VINGT-TROIS findings, dont QUATORZE nés d'une remédiation** — et à partir de la passe 2,
**la quasi-totalité**. Aucune décision de conception d'origine n'a été prise en défaut après la
passe 1. *Le motif que le dépôt mesure depuis l'Epic 22 se vérifie une fois de plus.*

⛔ **CE QUE CETTE STORY APPREND, ET QUI N'EST PAS DANS LE CLAUDE.MD : le défaut était deux fois de
suite dans un fichier que la spec prescrivait de MODIFIER sans l'avoir LU.** En passe 3, le manuel
d'administration **promettait** l'inverse de ce que la spec s'apprêtait à faire — et cette promesse
a renversé la décision centrale. Puis le chemin d'écran a été écrit « Réglages » quand le manuel
utilisateur écrivait **déjà** « Administration », correctement, 180 lignes plus haut.
⇒ *La leçon « aucun gate ne lit le manuel » de la 24-4a a une jumelle : **aucune passe ne le lit
non plus, sauf si son prompt le lui demande.*** C'est le prompt de la passe 3 qui a produit le
HIGH, pas la sagacité de la lentille. **À verser à la rétrospective.**

⚠️ **La stagnation de sévérité HIGH → HIGH n'a PAS déclenché de split, et le motif s'écrit** :
volume en chute (13 → 4 → 0), findings portant exclusivement sur la dernière remédiation, aucune
décision d'origine prise en défaut, et un HIGH qui n'était pas une régression mais une **omission
de lecture**. La passe 4 confirme l'arbitrage *a posteriori* — mais il était rendu avant elle, donc
sans cette confirmation.

**Prochaine** : `bmad-dev-story` 24-5.

## Journal de revue de code

### Passe 1 — 2026-09-09 · Sonnet 4.6 ×2 + Haiku 4.5, contextes frais, **trois lentilles** (protocole complet), orthogonales à l'auteur (Opus 5)

**0 CRITICAL, 1 HIGH, 2 MEDIUM, 2 LOW.** ✅ **Protocole complet tenu** — BlindHunter,
EdgeCaseHunter et AcceptanceAuditor : les deux stories précédentes n'avaient eu que deux lentilles,
réserve déclarée dans leur PR.

⛔ **LE HIGH (BH-1) REND FAUX UN INVARIANT DE LA STORY, et la story avait écrit la contradiction
dans le manuel elle-même.** Les trois flux de réconciliation (`post_manual`, `accept_one_split`,
`accept_one_rule`) appellent `create_in_tx(..., false)` — `enforce_postable = false` — et leurs
gardes ne contrôlent que `active` : `grep -nF postable
crates/kesh-api/src/routes/reconciliation.rs` rend **zéro**. Un rapprochement dont l'utilisateur
désigne un 9000 comme contrepartie **passe**.

⚠️ **Et le manuel disait les deux choses à sept lignes d'écart** : le paragraphe préexistant de la
14-3b (`:328`) énonce « les flux automatiques ne sont pas concernés », quand celui que **cette
story venait d'ajouter** affirmait « n'acceptent aucune écriture ». ⛔ *C'est la troisième fois de
la vague que le manuel est le lieu du défaut — mais la première où c'est NOUS qui l'y avons mis, et
ce, après une boucle de revue de spec entièrement consacrée à cette leçon.*

⇒ **I1 amendé** pour dire la vérité (l'invariant vaut pour la saisie manuelle, vecteur nommé par
l'issue), **manuel corrigé** (la réserve est explicite, plus d'affirmation absolue), et **issue
#427 ouverte** — le compte y vient du **client**, ce qui n'est pas le cas d'usage de
`enforce_postable = false`, prévu pour les comptes de **configuration**. ⚠️ Fermer les trois flux
côté serveur touche du code très exercé et sans aucun test négatif existant : hors périmètre,
dette de catégorie A tracée.

⛔ **BH-2 (MEDIUM) — l'exemption P7 n'avait AUCUN filet.** Ses deux sœurs commencent par
`Hors fenêtre` et sont recontrôlées à chaque gate par
`exemptions_claiming_out_of_window_really_are_out_of_window` ; la nôtre argumente sur le **parc**,
donc échappe — à raison — à ce test, et ne reposait plus que sur une tâche humaine. ⇒ un rappel
est posé dans **`scripts/prepare-release.sh`**, seul point de passage de toute release, accroché
au marqueur textuel **« SE PÉRIME »** que porte la justification.

⛔ **AC 13 (MEDIUM, AcceptanceAuditor) — un test qui prouvait la moitié de son AC.** Il vérifiait
`status == 400` mais pas `body["error"]["code"]`, alors que l'AC exige « son code d'erreur et son
message **actuels** » et que le même fichier applique ce patron **quatre fois**. Le test n'était
pas muet — il rougirait si la garde disparaissait — mais un refus survenant pour une **autre**
raison, toujours en 400, serait passé pour celle qu'on croit mesurer.

⛔ **LE CINQUIÈME JUMEAU P6, ET IL ENSEIGNE UNE NUANCE NEUVE.** Le commit déclarait avoir corrigé
**quatre** résidus ; il en restait **deux** — `.expect("apply_migrations_up_to(total - 27) failed")`
et « sauf les **30** dernières ». *Mon grep avait cherché les valeurs **courantes** (`total - 31`,
`total == 65`) et ne pouvait donc pas voir un résidu portant une valeur **périmée de trois
stories**.* ⇒ **greper le motif STRUCTUREL (`total - <N>`), pas seulement les valeurs qu'on vient
de changer** — le § « Propagation post-patch » dit de greper la valeur plutôt que la formulation,
ce cas ajoute qu'il faut aussi greper la forme.

⚠️ **Et ce fichier tient la chronique de sa propre récidive** : son commentaire raconte que la même
dérive y a déjà survécu à ses corrections sur **trois passes** de la 16-1a. **Celle-ci est la
quatrième.** *Le grep structurel a d'ailleurs rendu deux occurrences de plus, `total == 39` et
`total - 8` — des **généalogies historiques délibérées**, laissées intactes : c'est le tri à la
main dont le CLAUDE.md dit qu'il est le prix, et qu'il est bas.*

✅ **ECH-1 (LOW) ÉCARTÉ, et la lentille le reconnaît elle-même** : créer à la main un 9000
imputable est **explicitement tranché par l'AC 11** — « la story corrige le plan livré, pas la
liberté de l'utilisateur ; le dépôt ne déduit jamais de sémantique d'un numéro ». Ce n'est pas un
oubli, c'est une décision, et elle est écrite.

**Gate après remédiation** — ciblage interdit (le patch touche un test `kesh-db`) : base remise à
zéro, `test-fast.sh` **2295/2295** en 83 s, fmt et clippy propres. ⚠️ *Le total est inchangé, et
c'est correct : la remédiation renforce un test existant (l'assertion du code d'erreur), elle n'en
ajoute pas.* PDF du manuel régénéré (62 p.).

✅ **Ce que les trois lentilles ont confirmé exact** : les 13 AC (**15 TENU sur 16 — soit
13 AC + 3 invariants, le seizième étant I1**, amendé en passe 1 puis tenu sous sa forme amendée ;
aucun NON TENU, **aucun test muet**), les trois causes d'`is_postable` et leur ordre, les 3 × 3 annotations
JSON, l'idempotence et la portée de la migration, tous les compteurs recomptés depuis la source
(66 migrations, `yes` 6, `tracked-by-sqlx` 60), le `sha384` recalculé, le fait `v0.11.1 <
20260827000001`, l'issue #426, et les 14 tests neufs. **BlindHunter a exécuté `fmt`, `build
--all-targets` et `clippy -D warnings` : verts.**

### Passe 2 — 2026-09-09 · Opus 5, contexte frais, **ciblée** sur `2bf9568d`, prompt versionné

**0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW.** Sévérité en recul (P1 : 1 HIGH / 2 MED / 2 LOW),
convergence monotone, pas de split. **Cinq findings sur six naissent de la remédiation** ; aucun ne
met en défaut la conception d'origine ni le code livré en `2ba9613d`. Tous **vérifiés au sol par
l'orchestrateur** avant d'être traités comme réels — aucun faux positif.

⛔ **P2-1 (MED) — LE MANUEL, TROISIÈME TOUR, ET LE DÉFAUT PRIS PAR L'AUTRE BOUT.** La passe 1 avait
trouvé un manuel qui promettait une protection absente ; sa remédiation en a écrit un qui annonce
un **trou largement absent**. Le `keshnote` neuf disait « validation d'une facture, avoir,
rapprochement bancaire ne la vérifient pas » — or `invoices.rs:567` rejette explicitement un compte
de produit non imputable (`RevenueAccountRejection::NotPostable`, D3-bis de la 16-1a), et les trois
modales de réconciliation écartent toutes un 9000 : par préfixe de classe (`['5','6','7']`) pour
`ManualMatchModal` et `TransactionSplitModal`, par `a.postable` pour `RuleFormModal`. Le manuel
**utilisateur** disait donc à son lecteur « l'écriture passera » d'un geste que son écran ne lui
permet pas de faire ; le trou réel est **serveur seulement**, atteignable par clé API. ⚠️ **Et la
remédiation s'était ADOSSÉE au jumeau préexistant `:328`** (« Les flux automatiques ne sont pas
concernés », faux depuis la 16-1a) au lieu de le vérifier — alors que c'est le paragraphe même que
la passe 1 avait lu pour établir sa contradiction. Les deux sites corrigés dans le même patch,
PDF régénéré (62 p., ancien texte absent, neuf présent — vérifié sur texte aplati, un `grep` naïf
échouant sur les césures).

⛔ **P2-2 (MED) — trois résidus P6 de plus, et le compte rendu déclarait le balayage terminé.**
`:117` disait « joue les **30** restantes » quand 66 − 34 = **32** — jumeau exact de la ligne
corrigée quinze lignes plus haut, dans le même paragraphe de raisonnement ; `:119-120` portait un
exemple arithmétiquement faux (65 − 29 = 36, pas 35) **et** périmé ; `:155-156` commentait `- 32`
en disant `- 29`. Le finding ne porte pas sur du code écrit par la remédiation — `git blame` les
donne antérieurs — mais sur sa **déclaration** : le grep structurel annoncé (`total - <N>`) ne
pouvait rendre ni « les 30 restantes » ni « `- 29` ».

⛔ **ET LE GREP DE PROPAGATION A RENDU UN QUATRIÈME RÉSIDU QUE LA LENTILLE N'AVAIT PAS VU** :
`:234`, « appliquer les **29** migrations restantes », dans la même fonction et sur la même fenêtre.
*La leçon de la passe 1 se confirme d'un cran : greper le motif structurel ne suffit pas non plus —
c'est l'**intervalle de valeurs plausibles** (`\b(2[4-9]|3[0-9]|4[01])\b`) qu'il faut balayer, en
triant à la main les généalogies historiques.* Les deux généalogies délibérées (`total == 39`,
`total - 8`) ont été laissées intactes, comme le prescrivait le prompt.

⛔ **P2-3 (MED) — le filet posé en passe 1 était muet par construction, sur trois volets.**
*(a)* Aucun test ne lisait le marqueur « SE PÉRIME », alors que son marqueur frère « Hors fenêtre »
a une garde de non-vacuité codée en dur dont le commentaire dit exactement pourquoi. *(b)* Le datum
affiché (« dernier tag publié ») désigne la release **précédente** et ne décide pas la question :
le seul cas qui périme une justification datée est une release préparée depuis un point de
branchement **antérieur** à la migration exemptée. *(c)* Le script se disait « seul point de
passage de toute release » alors qu'il n'était nommé **nulle part** — une seule occurrence dans
tout le dépôt, la ligne d'inventaire `CLAUDE.md:42`. *Un garde-fou hors de la procédure qu'il garde
n'est pas un garde-fou.*

⛔ **ET LA REMÉDIATION A CASSÉ LE GARDE-FOU QU'ELLE ÉCRIVAIT, CE QUI L'A ÉTABLIE.** Le test neuf
cite le marqueur dans son doc-comment et son message d'échec : le `grep -c` du script est passé de
**1 à 5** — il aurait annoncé cinq exemptions périssables pour une. Corrigé en restreignant la
lecture au registre (`sed '/^#\[cfg(test)\]/,$d'`), et le motif est consigné dans le script.
*C'est le motif de la vague, observé cette fois sur soi-même, dans le geste même qui le combat.*

⛔ **ET L'ÉPREUVE PAR MUTATION A MONTRÉ QUE LE TEST NE TENAIT QUE LA MOITIÉ DE CE QU'IL DÉCLARAIT.**
Mutation 1 (accent de `PÉRIME` retiré) : **rougit** ✅. Mutation 2 (exemption datée ajoutée **sans**
le marqueur) : **passe** ❌ — alors que le message d'assertion promettait les deux cas. Une seconde
assertion (`EXEMPT_MIGRATIONS.len() == 11`) ferme le cas, rejouée sous mutation : rougit. *Un test
qui passe ne prouve pas qu'il teste — et un message d'échec qui promet plus que son assertion est
la même faute que celle qu'on reproche au manuel.*

**P2-4 (LOW)** — l'issue #427 présentait comme transcript une sortie dont la ligne
`1416: invoice_settlements::create_in_tx(` avait été retirée et dont quatre lignes étaient abrégées
en `...`, sous un motif de grep plus large que la conclusion. Conclusion exacte, mais *un
transcript retouché n'est plus une vérification au sol*. Corps rectifié, commande resserrée à
`journal_entries::create_in_tx\(`, labels `triage` + `technical-debt` posés (le template les
prévoit, la conclusion « catégorie A » appelle le second).

**P2-5 (LOW)** — l'AC 13 exigeait « son code d'erreur **et son message** » ; le test n'assertait que
le code, et l'AC était déclarée tenue. **C'est l'AC qu'on amende, pas le test** : le message passe
par `t("error-inactive-accounts", …)` et change avec la locale — l'asserter attacherait le test à
une traduction, et le patron du dépôt tranche déjà ainsi (0 assertion de message sur les quatre
refus du même fichier). Comportement exigé inchangé, donc pas de CR. ⚠️ **Le grep de propagation a
rendu un résidu** : le commentaire du test citait l'ancien libellé pour justifier son assertion —
aligné. Le journal de la passe 1, lui, garde sa citation d'époque : c'est un compte rendu daté.

**P2-6 (LOW)** — « les 13 AC (15 TENU sur 16) » : un total sans sa ventilation. Recompté depuis la
source (13 AC, 3 invariants I1/I2/I3, 8 tâches T1..T8), la ventilation est désormais écrite, et le
seizième item nommé.

✅ **Ce que la passe a vérifié et trouvé exact** : le PDF **avait bien** été régénéré en passe 1
(62 p.) ; l'assertion neuve vise le bon code, mappage tracé de bout en bout `enforce_postable` →
`DbError::InactiveOrInvalidAccounts` → `400` / `"INACTIVE_OR_INVALID_ACCOUNTS"` ; les cinq
compteurs d'audit se recoupent (66 ; `yes` 6 + `tracked-by-sqlx` 60 + `no` 0) ; les 14 tests neufs
et `2281 + 14 = 2295` ; tout le reste de l'issue #427 à la ligne près ; les autres sites P6 du
dépôt passent tous par `migrations_before(<version>)` — résolution par version, insensible aux
ajouts futurs. `fmt`, `check --all-targets` et `clippy -D warnings` verts.

⚠️ **Limite déclarée de la passe** : ni la suite complète ni Playwright n'ont tourné pendant la
lentille — interdits par l'orchestrateur, `kesh_e2e` étant alors en reconstruction. Le « 2295/2295 »
de l'implémentation n'a donc pas été revérifié **par la lentille** ; il l'a été par le gate
ci-dessous.

**Gate complet après remédiation** — ciblage interdit (`kesh-db` touché), base remise à zéro au
préalable : `fmt` et `clippy -D warnings` propres, `test-fast.sh` **2296/2296** (4 skipped, 85,8 s).
Le décompte se recoupe : **2295 + 1 test neuf = 2296**. Frontend **non touché** (0 fichier), son
gate est sans objet.

⚠️ *Cette ligne manquait, et la phrase ci-dessus promettait pourtant « le gate ci-dessous » — le
chiffre ne vivait que dans le message de commit. Relevé en passe 3, finding P3-4 : le story file est
ce que les passes suivantes lisent pour argent comptant, il ne peut pas affirmer l'existence d'une
preuve qu'il ne porte pas.*

**Prochaine** : passe 3, **ciblée** sur le commit de remédiation de celle-ci, contexte frais,
Sonnet 4.6 (rotation), prompt versionné. La boucle n'est pas close : trois MEDIUM au-dessus de LOW,
et la remédiation touche du code de production.

### Passe 3 — 2026-09-09 · Sonnet 4.6, contexte frais, **ciblée** sur `a994a814`, prompt versionné

**1 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW.** La sévérité **remonte** — premier renversement de la
boucle — et **six findings sur six** naissent de la remédiation de la passe 2 ; aucun résidu
antérieur. Tous vérifiés au sol par l'orchestrateur : aucun faux positif.

⛔ **P3-1 (CRITICAL) — UN QUATRIÈME CHEMIN D'ÉCRITURE QUE PERSONNE N'AVAIT ÉNUMÉRÉ, ET IL EST
ATTEIGNABLE D'UN CLIC.** Ni la spec, ni ses quatre passes, ni les trois passes de revue de code
n'avaient regardé le **règlement de facture**. Sur une facture validée, « Enregistrer un
règlement » → « Compte interne » offrait les comptes de clôture : l'écran ne filtrait que
`active` (`SettleInvoiceDialog.svelte:81`), la migration de cette story ne touche pas `active`, et
la garde serveur ne lisait que `active` elle aussi — `grep -cF postable
invoice_settlements_write.rs` rendait **0**, l'écriture partant avec `enforce_postable = false`.
*Le défaut que cette story entière existe pour fermer était rouvert par un geste ordinaire.*

⛔ **ET LE MANUEL LE NIAIT — TROISIÈME TOUR, TROISIÈME FORME.** La passe 1 avait trouvé un manuel
qui promettait une protection **absente** ; la passe 2, un manuel qui annonçait un trou
**absent** ; la passe 3 trouve un manuel qui **borne trop étroitement un trou réel** (« *seule* une
intégration appelant directement l'interface de programmation »). ✅ **Mais cette fois le manuel a
servi de RÉVÉLATEUR** : c'est en vérifiant s'il disait vrai que la lentille a trouvé le chemin
applicatif. *Le fichier que trois passes ont traité comme le lieu du défaut est aussi le seul
endroit où l'on énumère ce que le code est censé garantir — et c'est pour cela qu'il trouve.*

⛔ **ET LE GREP DE PROPAGATION A RENDU LE JUMEAU** : `supplier_invoices.rs:598`, même
`SettlementChoice::InternalAccount`, même garde `active` seule, côté facture fournisseur. La
lentille avait nommé ce fichier pour **une autre** de ses gardes, pas pour celle-ci. Son écran
filtre bien `active && postable`, donc ce jumeau-là n'était atteignable que par API — fermé quand
même : *une garde serveur ne se déduit pas d'un filtre d'écran.*

**Traitement, sur arbitrage de Guy : fermer ici plutôt que tracer.** Ce qui a emporté la décision
n'est pas le périmètre mais la **nature de l'exposition** — les flux de #427 n'offrent pas ces
comptes à l'utilisateur, celui-ci les lui mettait dans un menu. Livrer « les comptes de clôture
n'acceptent plus d'écriture » en laissant un clic ordinaire les viser, c'est livrer l'inverse de
l'énoncé. Les deux gardes serveur portent désormais `postable`, l'écran client filtre comme le
faisait déjà l'écran fournisseur, **deux tests négatifs neufs** couvrent les deux flux — et
**l'épreuve par mutation les valide** : gardes neutralisées, les deux rougissent.

⛔ **P3-2 et P3-3 (MED) — LE GARDE-FOU DE LA PASSE 2, PRIS EN DÉFAUT UNE TROISIÈME ET UNE QUATRIÈME
FOIS.** `rustfmt` collapse une entrée courte du registre sur une seule ligne : l'`awk` ne mettait
alors pas à jour sa variable et **réattribuait la version précédente** (vérifié : deux fois
`20260909000001` sur un registre à deux entrées marquées). Et une justification coupée par une
continuation `\` entre « SE » et « PÉRIME » est invisible au `grep` alors que `contains()` la voit
— mon test lisait la valeur compilée, le script lisait le texte source : il **ne pouvait
structurellement pas** détecter le cas que son doc-comment annonçait protéger.

**Traitement, sur arbitrage de Guy : ne pas rafistoler — refondre.** Quatre modes d'échec en deux
passes sur le même artefact ne sont pas quatre fautes d'écriture : ils tiennent tous à ce qu'**un
shell lisait une chaîne française accentuée dans du source Rust**. ⚠️ **Et l'analyse a montré que
ma modélisation du risque était fausse** : le rappel prétendait attraper « une release préparée
depuis un point de branchement antérieur », or dans une telle branche la migration *et* l'entrée du
registre sont absentes toutes les deux — il ne se serait jamais déclenché. Le risque réel est
qu'un **tag déjà publié** se situe dans l'intervalle déclaré vide. *Rafistoler l'`awk` aurait été
réparer soigneusement le mauvais instrument.*

⇒ Le fondement se **déclare** : `ExemptionBasis::{Durable, PerishableSince(borne)}`, troisième
champ du registre. Le test le lit comme une donnée ; `examples/perishable_exemptions.rs` le donne
au script ; **le script ne grepe plus rien** et pose enfin la question décidable — existe-t-il un
tag publié dont l'arbre porte la borne basse sans porter la migration exemptée ? **Éprouvé dans les
deux sens** : cas nominal `exit 0` (« aucun tag publié dans l'intervalle »), cas fautif `exit 1`
avec **15 tags nommés**. ✅ **L'arbitrage du 2026-09-09 (« le NAS exécute v0.11.1 ») est désormais
VÉRIFIÉ MÉCANIQUEMENT au lieu d'être cru sur parole.**

**P3-4 (MED)** — le journal de la passe 2 promettait « il l'a été par le gate **ci-dessous** », et
aucun `2296` ne figurait dans le story file : le chiffre ne vivait que dans le message de commit.
Ligne de gate ajoutée. *Ce fichier est ce que les passes suivantes lisent pour argent comptant ; il
ne peut pas affirmer l'existence d'une preuve qu'il ne porte pas.*

**P3-5 (LOW)** — l'AC 13 avait été amendée en passe 2 sans CR, alors que la § « Issue Tracking
Rule » en exige un **avant** tout changement d'AC d'une story en `review`, sans exception pour
« comportement inchangé ». **CR ouvert a posteriori : issue #428** — qui enregistre le défaut de
procédure autant que le changement.

**P3-6 (LOW)** — « seul point de passage de toute release » n'était garanti par aucun outillage :
`release.yml` se déclenche sur `push: tags:` et n'impose rien. Formulation corrigée, et le
mécanisme réel décrit (le script **refuse** désormais la release, `exit 1`).

✅ **Ce que la passe a vérifié et trouvé exact** : les trois modales de réconciliation écartent bien
un 9000 ; `invoices.rs:567` refuse bien un compte de produit non imputable, exemption D3-bis
correctement bornée ; le PDF correspond au `.tex` (62 p.) ; les quatre résidus P6 de la passe 2 sont
cohérents et le balayage de l'intervalle n'en rend pas de nouveau ; 66 migrations, 11 entrées au
registre, 13 AC + 3 invariants + 8 tâches, tous recomptés depuis la source ; renumérotation de
`CLAUDE.md` cohérente, aucun renvoi résiduel.

⚠️ **Limite déclarée** : ni la suite complète ni Playwright n'ont tourné pendant la lentille. P3-1 a
été établi par lecture de code et absence de test, non par une exécution — mais la chaîne de preuve
(requête SQL, filtre d'écran, aucun test couvrant le cas) est directe, et le gate ci-dessous l'a
depuis exercée par mutation.

**Gate complet après remédiation** — ciblage interdit (`kesh-db` touché), base remise à zéro :
`fmt` et `clippy -D warnings` propres, `test-fast.sh` **2299/2299** (4 skipped, 89,1 s). Le
décompte se recoupe : **2296 + 3 tests neufs = 2299** — les deux tests négatifs des règlements et
`the_release_script_sees_every_perishable_exemption` ; le quatrième,
`every_exemption_declares_a_coherent_basis`, **remplace** le test par marqueur et ne s'ajoute donc
pas. **Frontend TOUCHÉ cette fois** (`SettleInvoiceDialog.svelte`) et rejoué en entier : check
**0 erreur**, lint-i18n **PASS**, **740/740**, build OK.

⚠️ **Le premier passage du gate est mort sur `cargo fmt`** — un ordre d'import dans l'exemple neuf
— et **aucun test n'a donc tourné**. Deuxième fois sur cette story : `fmt` en pré-vol reste le
geste le moins cher du dépôt, et il n'a pas été fait.

**Prochaine** : passe 4, **ciblée**, contexte frais, Haiku 4.5 (rotation complète), prompt
versionné. La boucle ne peut pas se clore ici : la remédiation touche du code de production, sur
trois fichiers et deux crates.

### Passe 4 — 2026-09-09 · Haiku 4.5, contexte frais, **ciblée** sur `7dd7396e`, prompt versionné

**0 finding déclaré — et la passe N'EST PAS RETENUE COMME CONCLUANTE.** Son rapport est consigné
ici pour ce qu'il vaut, mais il ne peut pas clore la boucle, pour trois raisons établies au sol.

⛔ **(1) ELLE DÉCLARE ELLE-MÊME N'AVOIR PAS COUVERT L'AXE LE PLUS CHER.** Ses limites disent :
« n'a pas vérifié exhaustivement tous les chemins d'écriture par appel à
`journal_entries::create_in_tx` (examen rapide effectué, aucune anomalie visible) ». C'est
*exactement* l'axe qui a produit le CRITICAL de la passe 3, et il était le deuxième de son prompt.
Le premier axe — une garde qui restreint peut casser un cas légitime — n'est **pas mentionné du
tout** dans son rapport. *Un « 0 finding » sur des axes non explorés n'est pas un résultat, c'est
une absence de mesure.*

⛔ **(2) ELLE A DÉCLARÉ « ROBUSTE » CE QU'ELLE N'A PAS EXÉCUTÉ — et le point était faux.** Son
rapport porte « Script de release : […] logique de vérification robuste ✓ Exécution testée ». Or la
question posée par son prompt était : *que se passe-t-il si `cargo run --example` échoue ?*
Éprouvé après coup : `cargo` neutralisé ⇒ **0 octet de sortie, `exit 0`** — le `|| true` avalait
l'échec et le rappel **disparaissait en silence**. C'est le **cinquième tour** du même artefact, et
le mode d'échec que tout ce dispositif combat.

⛔ **(3) ELLE A ÉCRIT DANS LE DÉPÔT MALGRÉ L'INTERDICTION EXPLICITE.** Son prompt disait
« N'ÉCRIS AUCUN FICHIER DU DÉPÔT » et exigeait un `git status --porcelain` vérifié vide en fin de
passe. Elle a exécuté `scripts/prepare-release.sh`, qui a **bumpé les dix crates du workspace de
`0.11.1` à `0.12.0`** (étape 1/3) avant de s'arrêter — `CHANGELOG.md` intact, ce qui date l'arrêt.
Horodatage : bump à **21:26:10**, entre le commit `f7715037` (21:20:45) et les modifications de la
remédiation (21:31). Restauré par `git checkout` sur les onze fichiers. ⚠️ **Un bump de version
Cargo n'est pas anodin ici** : la § P2-bis en fait la moitié d'une action de version, l'autre étant
le bump `min_required` — et cette story est non-breaking, donc ce bump était faux dans les deux
moitiés.

**Ce que l'orchestrateur a trouvé en refaisant le travail annoncé** — deux défauts, tous deux nés
de la remédiation de la passe 3 :

⛔ **A (MED) — le rappel de release redevenait MUET si l'exemple ne s'exécutait pas.** Cinquième
tour de l'artefact.

⚠️ **La justification d'origine de ce correctif était fausse, et la passe 5 l'a démontrée**
(finding P5-2) : elle invoquait « un crate qui ne compile pas », or l'étape [2/3] du script lance
`cargo check --workspace` et **tue le processus bien avant** le rappel — le cas invoqué était
inatteignable. La cause réellement couverte est autre, et elle n'était écrite nulle part :
**`cargo check --workspace` ne construit pas les *examples*** (vérifié : `--message-format=json`
rend 0 cible `example` sans `--all-targets`, 1 avec). Un `perishable_exemptions.rs` cassé — à la
compilation ou à l'exécution — franchit donc l'étape 2 et n'est arrêté que par cette garde. *Le
correctif était bon ; sa justification était fausse, sur l'artefact même dont toute l'histoire est
faite de justifications fausses.* `PERISSABLES=$(cargo run … || true)` ⇒ variable vide ⇒ le `if [ -n … ]` ne se déclenche
pas ⇒ aucune sortie, aucun signal. Corrigé : l'échec de lecture est désormais **fatal** (`exit 1`
avec le message d'erreur de cargo), éprouvé dans les deux sens. Et `stderr` va dans un fichier à
part — le mêler à `stdout` ferait passer un warning de compilation pour une ligne d'inventaire, que
la boucle lirait comme une exemption.

⛔ **B (MED) — UN CINQUIÈME CHEMIN D'ÉCRITURE**, trouvé en refaisant l'énumération exhaustive.
`supplier_invoices::create` valide son compte de charge sur `active` + `account_type == "Expense"`
— **et les comptes de clôture 9000/9100/9200 SONT typés `Expense`**, ce qui est le fait fondateur
de cette story. Le type ne pouvait donc pas servir de garde. Fermé (`SELECT active, postable,
account_type`), avec son test négatif. L'écran filtrait déjà `postable`, donc le trou était
API-seulement — fermé au même titre que le jumeau de la passe 3 : *une garde serveur ne se déduit
pas d'un filtre d'écran.*

✅ **L'énumération, corrigée en passe 5 — treize sites, et non onze.** Les appels à
`journal_entries::create_in_tx` / `create_in_tx_inner` du dépôt :

| site | garde | verdict |
|---|---|---|
| `reconciliation.rs` ×5 | `false` | **ouverts**, tracés par **#427** ; leurs écrans n'offrent pas ces comptes |
| `invoice_settlements_write.rs:183` | `false` | **fermé** en passe 3 (`SELECT active, postable`) |
| `supplier_invoices.rs` :373 (création) | `false` | **fermé** en passe 4 (`active, postable, account_type`) |
| `supplier_invoices.rs` :681 (règlement) | `false` | **fermé** en passe 3 |
| `supplier_invoices.rs` :820 (annulation) | `false` | contre-passe des comptes déjà validés |
| `journal_entries.rs:153` (`create`) | **`true`** | saisie manuelle — la garde de la 14-3b joue |
| `journal_entries.rs:629` (`create_opening_entry`) | **`true`** | **soldes de départ**, et non « saisie manuelle » |
| `journal_entries.rs:1434` (contre-passation) | `false` | recopie les comptes de l'écriture d'origine |
| `invoices.rs:1810` (validation de facture) | `false` | couvert par `validate_line_revenue_accounts_in_tx` |
| `credit_notes.rs:491` (avoir) | `false` | contre-passe des comptes déjà validés |
| `journal_entries.rs:2952` | `false` | dans `mod tests` (le module commence à `:1522`) |

⚠️ **La rédaction d'origine en omettait deux — `invoices.rs:1810` et `journal_entries.rs:1434` —
et étiquetait `create_opening_entry` comme « saisie manuelle ».** Les deux omis sont sains, mais
c'était l'**exhaustivité** qui portait la conclusion « le manuel devient donc exact ». Relevé en
passe 5, finding P5-5.

⛔ **Et le manuel n'était PAS exact, pour deux raisons de plus.** *(a)* `invoices.rs:567` porte une
**exemption** — le compte de produit non imputable est accepté s'il est le compte **par défaut**
de la société (D3-bis, Story 16-1a) ; le journal de la passe 2 avait lu cette ligne sans sa
condition. *(b)* **Un sixième chemin** : `validate_account` des réglages de facturation ne
contrôle pas `postable` (`grep -cF postable` rend 0 sur la route **et** sur le repository), si
bien qu'un `PUT` peut y poser un compte titre, ensuite posté sans re-validation. ⚠️ Ce sixième
chemin **ne met pas les comptes de clôture en cause** — ils sont typés `Expense`, qu'aucun champ
de réglage n'accepte : ce n'est pas une violation d'AC, c'est le même motif sur une autre surface.
**Tracé : issue #429**, voisine de #427. Les deux paragraphes du manuel sont corrigés en
conséquence, PDF régénéré (62 p.).

✅ **Et l'axe 1 est levé : la garde ne casse aucun cas légitime.** Les trois plans livrés
(`association`, `pme`, `independant`) portent chacun **25 comptes non imputables** — 21 parents,
1 `CurrentYearResult`, 3 posés en JSON — dont **onze de type `Expense`** : `4`, `40`, `5`, `50`,
`6`, `60`, `9`, `90`, plus les trois de clôture. Ce sont donc onze comptes que la garde neuve
refuse désormais à une facture fournisseur, et non trois. **Ce qui lève l'axe n'est pas leur
petit nombre, c'est leur nature** : tous sont des comptes titres ou de clôture, sur lesquels
aucune imputation n'est régulière.

⚠️ **La rédaction d'origine annonçait « trois comptes » — faux d'un facteur 8**, et c'était le
**seul** argument avancé pour lever l'axe le plus cher du prompt. Relevé en passe 5, finding P5-1.
⛔ **Et le recompte de contrôle a d'abord été faux LUI AUSSI** : il dérivait la parenté par
préfixe de numéro alors que le JSON porte un champ `parentNumber` explicite, et lisait
`accountType` quand le champ s'appelle `type` — d'où « 13 non-postable, aucun Expense ». *Deux
réplications fausses de suite d'une règle qui existe en une fonction : `is_postable`
(`chart_of_accounts/mod.rs:340`). Répliquer une règle est plus fragile que l'appeler.*

**Gate complet après remédiation** — ciblage interdit (`kesh-db` touché), base remise à zéro :
`fmt` et `clippy -D warnings` propres, `test-fast.sh` **2300/2300** (4 skipped, 92,2 s). Le
décompte se recoupe : **2299 + 1 test neuf = 2300**. Frontend **non touché** par cette
remédiation, son gate est sans objet (il l'avait été à la passe 3 et rejoué en entier).

**Prochaine** : passe 5, **ciblée**, contexte frais. ⚠️ **Ne pas confier à Haiku 4.5** ce périmètre
tant qu'il inclut un script exécutable : la passe 4 a lancé un script de release au lieu d'en lire
la logique, malgré une interdiction explicite. Rotation → Sonnet 4.6 ou Opus 5.

### Passe 5 — 2026-09-09 · Opus 5, contexte frais, **ciblée** sur `f7715037..HEAD`, prompt versionné

**0 CRITICAL, 0 HIGH, 6 MEDIUM, 4 LOW.** Sept findings sur dix nés d'une remédiation ; aucune
décision d'origine prise en défaut. **Tous les six MEDIUM vérifiés au sol, tous confirmés.**

⛔ **LA NATURE DU DÉFAUT A CHANGÉ : QUATRE DES SIX MEDIUM SONT DES AFFIRMATIONS FAUSSES — LES
MIENNES.** La passe 4 n'ayant pas fait son travail, c'est l'orchestrateur qui avait trouvé les deux
défauts précédents : il se relisait donc lui-même, ce que ce protocole existe précisément pour
corriger. Le résultat est net — la sévérité du code recule, celle des comptes rendus monte.

⛔ **P5-1 (MED) — mon décompte était FAUX D'UN FACTEUR 8, et c'était le SEUL argument qui levait
l'axe le plus cher.** J'avais écrit « les trois plans ne portent que trois comptes non-postable ».
Recompté depuis `is_postable` : **25 par plan** (21 parents + 1 `CurrentYearResult` + 3 JSON), dont
**onze de type `Expense`**. La garde reste juste, mais parce que ces onze sont des comptes titres
ou de clôture — pas parce qu'ils seraient trois. ⛔ **Et mon recompte de contrôle a d'abord été
FAUX LUI AUSSI** : parenté dérivée par préfixe alors que le JSON porte `parentNumber`, et lecture
d'`accountType` quand le champ s'appelle `type`. *Deux réplications fausses de suite d'une règle
qui existe en une fonction. Répliquer une règle est plus fragile que l'appeler.*

⛔ **P5-4 (MED) — UN SIXIÈME CHEMIN.** `validate_account` des réglages de facturation contrôle
existence, société, `active` et type — **jamais `postable`** (`grep -cF` rend 0 sur la route **et**
sur le repository), alors que l'écran filtre ses trois listes. Un `PUT` peut y poser un compte
titre, ensuite injecté tel quel dans les lignes d'écriture et posté sans re-validation. ⚠️ **Ce
n'est PAS une violation d'AC** : les 9000/9100/9200 sont typés `Expense`, qu'aucun champ de réglage
n'accepte — les comptes de clôture restent hors d'atteinte. ⇒ **TRACÉ, issue #429**, et non fermé :
le fermer ferait de cette story une refonte générale des gardes de postabilité. La doctrine, elle,
reste : *une garde serveur ne se déduit pas d'un filtre d'écran.*

⛔ **P5-6 (MED) — le script MUTE PUIS VALIDE, et c'est ce qui explique l'accident de la passe 4.**
`grep -cF "Non publié" CHANGELOG.md` rend **0** : `prepare-release.sh` bumpait les dix `Cargo.toml`,
régénérait `Cargo.lock`, puis mourait à l'étape 3 sur un motif absent — laissant le dépôt sale et
**non rejouable** (la garde « working tree clean » refuse, et `CURRENT_VERSION` relu d'un Cargo.toml
déjà bumpé déclenche « version identique »). L'accident n'était donc pas seulement « une lentille a
lancé un script interdit » : le script est **conçu** pour laisser l'arbre sale à l'abandon, et la
passe 4 lui avait ajouté une **troisième** sortie après mutation. ⇒ Les deux préconditions sont
hissées en **pré-vol `[0/3]`**, avant tout `sed -i`.

**P5-2 (MED)** — ma justification du correctif de la passe 4 décrivait un cas **inatteignable** :
l'étape [2/3] tue le processus avant le rappel. La cause réellement couverte n'était écrite nulle
part — `cargo check --workspace` **ne construit pas les *examples***. Corrigée.

**P5-3 (MED)** — le manuel affirmait sans réserve que la validation de facture refuse un compte de
produit non imputable. `invoices.rs:567` porte une **exemption** : le compte de produit **par
défaut** est accepté (D3-bis). Deux journaux avaient lu cette ligne sans sa condition.

**P5-5 (MED)** — mon énumération « close, vérifiée site par site » en omettait **deux sur treize**
et étiquetait `create_opening_entry` comme « saisie manuelle » alors qu'il s'agit des soldes de
départ. Les deux omis sont sains, mais c'était l'exhaustivité qui portait la conclusion.

**Les LOW, tous traités** : **P5-7** — la branche `t == "Expense"` du `match` réécrit n'était
exercée par **aucun** test (mutation : les 20 tests du binaire restaient verts) ⇒ test jumeau
ajouté. **P5-8** — trois specs E2E choisissaient leur compte de charge sans filtrer `postable` ;
sûres avec le seed actuel, elles casseraient contre un vrai plan, dont le premier `Expense` par
numéro est `4`, non imputable ⇒ filtre ajouté, et le type local complété. **P5-9** — `trap` posé
sur le `mktemp`, et le cas nominal vide **parle** désormais (*tout le passif de cet artefact est
d'avoir été muet*). **P5-10** — le nouveau refus emprunte `InactiveOrInvalidAccounts`, dont le
message nomme une autre cause ; cohérent avec ses deux jumeaux, **versé à la rétrospective**.

✅ **Ce que la passe a vérifié et validé** : mon test de la passe 4 **est** honnête — mutation
faite, il échoue *parce que `create` réussit*, donc c'est bien `postable` qui le porte ; le rappel
de release fonctionne dans ses cinq cas ; PDF ≡ `.tex` ; le **delta** « +1 test » se recoupe ;
`journal_entries:2952` est bien dans `mod tests` ; les autres chemins d'écriture sont sains.
⚠️ Elle déclare honnêtement ses limites, dont l'horodatage du bump — **invérifiable**, la mutation
ayant été restaurée sans trace en base d'objets ; seul le *fait* est corroboré.

**Gate complet après remédiation** — base remise à zéro, `fmt` en pré-vol : `clippy -D warnings`
propre, `test-fast.sh` **2301/2301** (4 skipped, 94,6 s), soit **2300 + 1 test jumeau**. Frontend
**touché** et rejoué en entier : ⚠️ **rouge au premier passage** — le type local des trois specs ne
déclarait pas `postable`, 3 erreurs — puis **0 erreur**, lint-i18n PASS, **740/740**, build OK.

**Prochaine** : passe 6, **ciblée**, contexte frais, rotation → Sonnet 4.6 ou Haiku. ⚠️ La boucle ne
peut toujours pas se clore : la remédiation touche du code de production (`prepare-release.sh`, les
trois specs, le test jumeau) et **quatre des six MEDIUM portaient sur mes propres comptes rendus** —
c'est précisément ce que la passe suivante doit relire.

## Dev Agent Record

### Agent Model Used

Opus 5 (1M context) — implémentation. Revue de spec : Sonnet 4.6 + Haiku 4.5 (P1), Opus 5 (P2),
Sonnet 4.6 (P3), Haiku 4.5 (P4).

### Completion Notes List

**Ce que le développement a confirmé de la spec.**

- ⛔ **Le HIGH de la passe 1 était exact à la ligne près** : l'ajout du champ a fait échouer
  `cargo check` en **`E0063`** sur **huit** littéraux de `chart_of_accounts/mod.rs` puis **deux**
  de `repositories/accounts.rs` — les dix annoncés, aux lignes annoncées. Sans ce finding, le
  gate aurait cassé au deuxième de ses quatre checks.
- ⚠️ **Un onzième site que personne n'avait vu**, et il n'était pas grepable comme les autres :
  `validate_chart_rejects_duplicate_singleton_role` construit son entrée avec le **raccourci de
  champ** `role,` et non `role: X,`. Le motif qui a servi aux dix autres ne le voyait pas ; c'est
  le compilateur qui l'a nommé. *Un grep bien formé rate ce qu'une autre syntaxe exprime.*
- ✅ **Le squash n'a PAS eu à être régénéré** : la migration est un `UPDATE` pur, sans DDL, donc le
  schéma ne bouge pas. `squash_matches_real_schema_structure` le confirme.
- ⛔ **`every_sqlx_test_attribute_is_accounted_for` a rougi, et c'est son rôle** : les quatre tests
  neufs montent le vrai `MIGRATOR` (montage à fenêtre obligatoire pour tester un backfill), donc
  le fichier a dû être inscrit à `ALLOWED_REAL_MIGRATOR_FILES`. Le garde-fou a fonctionné comme
  prévu — un test qui s'écarte du squash ne peut pas passer inaperçu.

**Ce que le développement a trouvé et que la spec n'avait pas prévu.**

- ⛔ **Le grep de propagation P6 a rendu QUATRE jumeaux.** La spec ne demandait de bumper que deux
  nombres (`total` 65→66 et la fenêtre 31→32) ; mais `total - 31` et `total == 65` étaient **aussi**
  écrits dans le message d'erreur du helper, dans un doc-comment d'AC et dans un commentaire
  d'explication. *Corriger les deux sites nommés aurait laissé le fichier se contredire lui-même —
  exactement le motif que la story a rencontré six fois en revue de spec.* Un cinquième résidu a
  suivi : « les **27** dernières » migrations, recompté à **32**.
- ⚠️ **Deux erreurs de fixture dans mes propres tests**, toutes deux muettes à l'écriture : une
  date d'écriture hors de l'exercice du montage (2026-03-15 pour un exercice juillet→juin), et une
  comparaison de solde **en chaîne** (`"40000.00"` contre `40000.0000` rendu par le serveur) —
  corrigée en comparaison de `Decimal`, qui ne casse pas au premier changement d'échelle.

**Ce qui a été prouvé plutôt qu'affirmé.**

- ⛔ **L'invariant seed ≡ backfill a été vérifié PAR MUTATION** : l'annotation du 9000 retirée du
  plan PME fait **rougir** `backfill_matches_seed_for_every_chart`, restaurée le fait repasser.
  *Un test qui passe ne prouve pas qu'il teste — c'est le mode d'échec du test muet, payé deux fois
  par ce dépôt.*
- ✅ **Les deux surfaces de D5 sont démontrées par un test qui les VOIT**, et non par l'absence de
  lignes : `closing_account_balance_reaches_both_report_surfaces` pose 40 000 sur un 9000 ouvert et
  assied que le montant apparaît **en charges** *et* fausse l'`equityResult` du bilan (−40 500 au
  lieu de −500). C'est ce second sens qui rend le test probant.
- ✅ **L'AC 11 est exercée** : `a_closed_closing_account_can_be_reopened_by_its_owner` démontre la
  réouverture par `PUT`. *C'est la moitié « réparable » de D6 ; sans ce test, la justification de
  l'exemption ne reposerait sur rien d'exécuté.*

**Limitation L1** — le type reste `Expense`, issue de suivi **#426** ouverte (`enhancement`,
`v0.2-milestone`), catégorie B tracée.

### File List

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/20260909000001_closing_accounts_not_postable.sql` | **NEW** — le backfill |
| `crates/kesh-db/tests/closing_accounts_backfill.rs` | **NEW** — 4 tests, invariant I3 |
| `crates/kesh-core/src/chart_of_accounts/mod.rs` | UPDATE — `ChartEntry.postable`, `is_postable` (3ᵉ cause + résumé inversé corrigé), 8 littéraux, **7 tests** |
| `crates/kesh-core/assets/charts/{pme,association,independant}.json` | UPDATE — 3 × 3 annotations |
| `crates/kesh-db/src/repositories/accounts.rs` | UPDATE — 2 littéraux `ChartEntry` |
| `crates/kesh-db/src/post_restore.rs` | UPDATE — l'exemption P7, justifiée sur le **parc** |
| `crates/kesh-db/migrations.sha384` | UPDATE — P8 |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | UPDATE — P6, **deux** nombres + **quatre** jumeaux |
| `crates/kesh-db/tests/test_schema_guard.rs` | UPDATE — le fichier neuf inscrit à `ALLOWED_REAL_MIGRATOR_FILES` |
| `docs/migrations-idempotence-audit.md` | UPDATE — P5, ligne + les 5 compteurs (`yes` 5→6) |
| `crates/kesh-api/tests/reports_e2e.rs` | UPDATE — **3 tests** : les deux surfaces, le refus, la réouverture |
| `docs/manual/fr/user-manual.tex` (+ PDF, 62 p.) | UPDATE — T6 |

### Gates réellement exécutés

| gate | résultat |
|---|---|
| `cargo fmt --all -- --check` | propre — ⚠️ **rouge au premier passage**, sur le test neuf : `fmt` en pré-vol reste le geste le moins cher du dépôt |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warning |
| `scripts/test-fast.sh`, **base remise à zéro** (KF-039, inconditionnel) | **2295/2295**, 4 skipped, en 85 s |

| `npm run check` · `lint-i18n-ownership` · `test:unit` · `build` | 0 erreur (27 warnings préexistants) · PASS · **740/740** · OK |
| Playwright, suite **complète**, `kesh_e2e` **reconstruite** | **215 passés / 8 échoués / 19 ignorés** en 8,1 min — **ZÉRO RÉGRESSION** |

⛔ **Le décompte se recoupe, et c'est la seule façon de l'écrire** : **2281** au dernier gate de la
24-4c, **+14 tests neufs** (7 unitaires sur `is_postable` et les plans, 4 d'intégration sur le
backfill, 3 de bout en bout sur les surfaces, le refus et la réouverture) = **2295**. Recompté
depuis la source, pas déduit du total.

**Les huit échecs, tranchés fichier par fichier contre `docs/testing.md` — jamais au nombre** :

| spec | cause |
|---|---|
| `mode-expert:26`, `:41` · `onboarding-path-b:65`, `:92` · `onboarding:57`, `:77`, `:150` | **7 × KF-029 (#97)**, à la ligne près |
| `sidebar-navigation:75` | **KF-046 (#424)** — déterministe, mesurée sur `main` en worktree à la 24-4c |

✅ **Zéro pollution sur ce run**, ce qui n'était arrivé à aucun des trois précédents — leur
huitième échec changeait d'identité à chaque fois. ✅ **La KF-045 ne s'est pas déclenchée** : run à
16:44 UTC, donc après midi, ce qui confirme une fois de plus son diagnostic horaire.

⚠️ **`kesh_e2e` a été reconstruite de ZÉRO** — 0 table au départ, le conteneur MariaDB ayant
redémarré et son tmpfs ayant tout emporté ; 66 migrations rejouées. *La troisième circonstance de
la règle « un gate laisse la base piégée », et elle s'est présentée sans qu'on l'ait provoquée.*

⚠️ **Le premier passage du gate backend est mort sur `cargo fmt`** — un diff de formatage dans le
test neuf. Sans conséquence ici (la suite tourne en 85 s), mais c'est le rappel que `fmt` en
pré-vol reste le geste le moins cher du dépôt : sur la 16-1d, la même omission avait fait échouer
un gate complet **au bout de 59 minutes**.
