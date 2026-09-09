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

⛔ **Et le rejeu est NÉCESSAIRE.** La colonne `postable` existe depuis juillet
(`20260722000001`) : une archive prise entre cette date et cette migration porte donc
`postable = TRUE` sur le 9000 et le **restaure**. Sans rejeu, le défaut **rouvre définitivement**
— `_sqlx_migrations` n'étant pas restaurée, la migration reste marquée appliquée et ne repassera
jamais. C'est le mode d'échec exact que P7 existe pour empêcher.

⇒ **Décision : classe A, et la réserve s'écrit au lieu de se taire.** Le rejeu peut refermer un
compte qu'un utilisateur avait délibérément rouvert. On l'accepte, sur l'asymétrie des coûts —
la même que le dépôt applique aux doublons de contacts :

- **l'écrasement est bruyant et réparable** — le compte est visible dans le plan comptable, et un
  `PUT` le rouvre en un geste ;
- **le non-rejeu est muet et comptablement faux** — l'à-nouveau repart en charges, et rien ne le
  signale.

⚠️ **`AND version = 1` a été examiné comme garde d'intention et écarté** : un utilisateur ayant
simplement **renommé** son compte porte `version > 1` sans avoir touché `postable`, et le rejeu le
sauterait — un faux négatif **silencieux**, c'est-à-dire le mode d'échec qu'on cherche à fermer,
réintroduit par sa propre garde.

⛔ **Cette réserve va au manuel d'administration** (T6) : elle décrit un comportement observable
par l'exploitant, et une réserve qui ne vit que dans un commentaire Rust n'atteint personne.

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
9. ⛔ **Le backfill est inscrit à `POST_RESTORE_BACKFILLS` en classe A** (`Unconditional`), et
   **ni exempté, ni classé B** — cf. D6, qui établit que les deux autres issues sont
   respectivement **fausse** (la migration est dans la fenêtre) et **indisponible** (pas de DDL,
   donc pas de sentinelle). ⚠️ **L'entrée porte, en commentaire, la réserve de D6** : le rejeu
   peut écraser un `postable` posé à la main. *Le dépôt a déjà écrit cette réserve pour le
   gabarit (`post_restore.rs:257-260`) ; ne pas la répéter ici la ferait disparaître avec lui.*
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
  - [ ] ⛔ **P7 — inscription en classe A au registre** (AC 9) : `POST_RESTORE_BACKFILLS`,
        `BackfillTrigger::Unconditional`, le `.sql` embarqué sous `src/post_restore/`.
        ⛔ **Ne PAS chercher une exemption ni une sentinelle** — D6 démontre que l'une est fausse
        et l'autre indisponible ; le test `every_data_backfill_migration_is_triaged` nommerait le
        fichier, et `exemptions_claiming_out_of_window_really_are_out_of_window` rejetterait la
        justification « hors fenêtre »
  - [ ] le commentaire de l'entrée porte **la réserve de D6** — le rejeu peut écraser un
        `postable` posé à la main — et **dit pourquoi on l'accepte**
  - [ ] ⛔ **P5** — ligne dans `docs/migrations-idempotence-audit.md` **et les cinq compteurs**,
        recomptés depuis la source : `ls crates/kesh-db/migrations/*.sql | wc -l` (**65 → 66**),
        `grep -c '^| `20' docs/migrations-idempotence-audit.md`, l'en-tête `## Table d'audit (N
        migrations)`, la ligne `Total`, et les trois compteurs de partition dont la **somme** doit
        égaler le total — ⚠️ **ils ne valent pas le total**
  - [ ] ⛔ **Le verdict d'idempotence est `yes`, et le compteur `yes` passe de 5 à 6.** Le critère
        du document est net : **DML pur, sans DDL** ⇒ `yes`. Le précédent à copier est
        `20260729000001_invoice_lines_revenue_account_backfill.sql` (« deux `UPDATE` de données
        uniquement, aucun DDL »), **pas** `20260722000001`, qui porte `tracked-by-sqlx` **parce
        qu'il contient aussi du DDL**. ⚠️ *Un verdict mal affecté laisse la somme juste et la
        partition fausse : recompter les cinq compteurs ne le détecte pas.*
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
- [ ] **T5 — Les deux surfaces** (AC 12, 13, I1, I3)
  - [ ] test d'intégration : poster sur le 9000 est refusé par la garde **existante** de 14-3b
  - [ ] test : un solde historique sur le 9000 n'apparaît **ni** au compte de résultat **ni** dans
        `fetch_retained_earnings` **une fois les lignes retirées** — ⚠️ et **apparaît** tant
        qu'elles sont là : c'est ce second sens qui démontre le défaut, le premier ne démontre que
        l'absence de lignes
  - [ ] test I3 : décompte avant/après du backfill, et aucun `account_type` modifié
- [ ] **T6 — Le manuel** (D4)
  - [ ] `docs/manual/fr/user-manual.tex` : les comptes de clôture n'accueillent pas d'écriture
        dans Kesh ; le report à nouveau se pose au **2970** ; une écriture déjà passée sur un
        compte 9 se corrige par **contre-passation** (24-4a), non par réécriture (24-4b)
  - [ ] PDF régénéré (`make fr` dans `docs/manual/`) et commité
  - [ ] **la réserve de D6 va au manuel d'ADMINISTRATION** (`docs/manual/fr/admin-manual.tex`) :
        restaurer une sauvegarde antérieure à cette version referme les comptes de clôture, y
        compris ceux que l'exploitant aurait rouverts délibérément
  - [ ] ⚠️ **la limite du critère « numéro »**, sur le modèle de celle que le backfill de rôles
        énonce déjà (`20260722000001:110-113`) : un utilisateur ayant **réaffecté** le numéro 9000
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
d'invariant), **`kesh-api` (le test de bout en bout de l'AC 12 vit dans
`crates/kesh-api/tests/reports_e2e.rs`)** et `docs/manual`. **Quatre**, sous le seuil de cinq.
Aucun frontend, **aucune clé i18n** — la garde de 14-3b et son message existent déjà (AC 13).

⚠️ **Le décompte a été recompté, et il était faux à trois** : `kesh-api` manquait, alors que
l'AC 12 l'exige. *Un décompte qu'on écrit sans le refaire est le défaut que le § « Recompter ses
propres comptes rendus » vise ; il ne devient pas vrai parce qu'il reste sous le seuil.*

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
| `crates/kesh-db/src/post_restore.rs` (+ `src/post_restore/<date>_*.sql`) | UPDATE + NEW — P7, entrée **classe A** et sa réserve (D6) |
| `crates/kesh-db/tests/accounts_role_backfill.rs` | UPDATE — l'invariant seed ≡ backfill |
| `crates/kesh-api/tests/reports_e2e.rs` | UPDATE — le test de bout en bout de l'AC 12 (les tests de `income_statement.rs` ne montent **aucune** base) |
| `docs/manual/fr/user-manual.tex` (+ PDF) | UPDATE — T6 |

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
aucune sentinelle constructible), et la classe A ne satisfait pas son propre critère. ⇒ **D6**,
qui tranche pour la classe A sur l'asymétrie des coûts — *l'écrasement est bruyant et réparable,
le non-rejeu est muet et comptablement faux* — et **écrit la réserve au lieu de la taire**.

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
décompte de modules disait « trois » en omettant `kesh-api`, où vit le seul fichier capable de
porter le test de bout en bout (les tests de `income_statement.rs` ne montent aucune base) ; et le
verdict d'idempotence de la ligne d'audit n'était pas spécifié, avec un gabarit qui **induisait en
erreur** — `20260722000001` porte `tracked-by-sqlx` **parce qu'il contient du DDL**, quand notre
migration, DML pur, relève du `yes` de `20260729000001`. *Un verdict mal affecté laisse la somme
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
