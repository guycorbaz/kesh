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
`grep -rn '9000\|9100\|9200' crates/ --include=*.rs` rend **zéro site applicatif** : ces trois
comptes n'ont, dans Kesh, aucun usage. Ils sont offerts à la saisie et à rien d'autre.

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
   deux CRITICAL de la 24-4a.* ✅ Le gabarit **exact** est deux lignes plus bas
   (`:177`, `UPDATE accounts SET postable = FALSE WHERE role = 'CurrentYearResult';`), qui ne
   filtre ni sur `active` ni sur la société.
8. Le backfill **ne touche ni `account_type`, ni les écritures, ni `version`** — cf. D4 et le
   précédent de `create` (`accounts.rs:191` : *« pas de bump de version : postable est ici une
   conséquence structurelle »*).
9. **L'invariant « seed ≡ backfill » tient** : le test dédié
   (`crates/kesh-db/tests/accounts_role_backfill.rs`) compare les deux sources et doit rester
   vert **en couvrant les trois comptes neufs**.
   ⛔ **C'est le filet principal de la story**, et son en-tête le dit : *« rien n'oblige
   structurellement ces deux sources à concorder ; le symptôme n'apparaîtrait qu'en production,
   chez un utilisateur migré. »*
10. La création manuelle d'un compte reste libre : un utilisateur qui **crée lui-même** un compte
   9000 le crée postable. ⚠️ **Assumé, et ce n'est pas une faille** — la story corrige le plan
   **livré**, pas la liberté de l'utilisateur ; le dépôt ne déduit jamais de sémantique d'un
   numéro (14-3a).
11. Un test de bout en bout démontre le défaut **fermé sur les deux surfaces de D5** : après
    migration, poster sur le 9000 est refusé, et le montant n'apparaît ni au compte de résultat
    ni dans le report à nouveau du bilan.
12. La postabilité est déjà appliquée à la saisie (Story 14-3b) : **aucun nouveau refus n'est à
    écrire**. Un test vérifie que le refus existant se déclenche bien sur ces comptes — ⚠️ **avec
    son code d'erreur et son message actuels, sans en inventer un neuf.**

## Invariants testables

- **I1 — Aucune écriture neuve sous un compte de clôture.** Après la migration, aucune ligne de
  `journal_entry_lines` créée postérieurement ne pointe sur un compte de numéro 9000/9100/9200.
- **I2 — Le plan livré et le plan migré coïncident.** Pour les trois `org_type`, l'ensemble
  `{(numéro, postable)}` produit par le seed est **identique** à celui produit par le backfill sur
  une base contenant le plan d'origine. *C'est l'AC 9, énoncée comme propriété.*
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
  - [ ] tests unitaires : `Some(false)` retire ; `Some(true)` ne force ni sur un parent ni sur
        `CurrentYearResult` ; `None` se comporte comme aujourd'hui
- [ ] **T2 — Les trois plans** (AC 4, 5)
  - [ ] `"postable": false` sur 9000/9100/9200 dans `pme.json`, `association.json`,
        `independant.json` — ⚠️ **et nulle part ailleurs**
  - [ ] test : pour chaque `org_type`, exactement trois entrées portent `postable: Some(false)`
- [ ] **T3 — La migration de backfill** (AC 6, 7, 8)
  - [ ] `UPDATE accounts SET postable = FALSE WHERE number IN ('9000','9100','9200') AND postable = TRUE`
        — la clause `AND postable = TRUE` rend la migration **idempotente** et son effet mesurable
  - [ ] ⛔ **P7 — triage obligatoire** : c'est un backfill de **données**, il DOIT être inscrit à
        `POST_RESTORE_BACKFILLS` ou exempté avec justification écrite
        (`crates/kesh-db/src/post_restore.rs`) ; le test `every_data_backfill_migration_is_triaged`
        échoue sinon en nommant le fichier. ⚠️ Si l'argument retenu est « hors fenêtre », la
        justification **DOIT commencer par la chaîne `Hors fenêtre`**, sinon elle échappe au
        contrôle symétrique
  - [ ] ⛔ **P5** — ligne dans `docs/migrations-idempotence-audit.md` **et les cinq compteurs**,
        recomptés depuis la source : `ls crates/kesh-db/migrations/*.sql | wc -l` (**65 → 66**),
        `grep -c '^| `20' docs/migrations-idempotence-audit.md`, l'en-tête `## Table d'audit (N
        migrations)`, la ligne `Total`, et les trois compteurs de partition dont la **somme** doit
        égaler le total — ⚠️ **ils ne valent pas le total**
  - [ ] ⛔ **P6 — DEUX nombres**, pas un : `total == 65` → **66** *et* la fenêtre `total - 31` →
        `total - 32`, frontière tenue à **34** (`crates/kesh-db/tests/migrations_upgrade_path.rs`).
        *Bumper le total seul élargirait la fenêtre en silence — la 24-4c a payé ce piège.*
  - [ ] ⛔ **P8** — ligne dans `crates/kesh-db/migrations.sha384`
  - [ ] le squash `crates/kesh-db/test-schema/0001_schema_squash.sql` régénéré
  - [ ] **P1/P3 — non breaking** : `UPDATE` de données, aucun DDL destructif ⇒ **ni bump
        `min_required`, ni bump Cargo**
- [ ] **T4 — L'invariant seed ≡ backfill** (AC 9, I2)
  - [ ] `crates/kesh-db/tests/accounts_role_backfill.rs` étendu aux trois comptes
  - [ ] ⚠️ ce test monte les migrations **par version** (`migrations_before`), jamais par
        position : ne pas y introduire de `total - N`
- [ ] **T5 — Les deux surfaces** (AC 11, 12, I1, I3)
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

Modules touchés : `kesh-core/chart_of_accounts`, `kesh-db` (migration + le test d'invariant),
`docs/manual`. **Trois**, sous le seuil de cinq. Aucun frontend, **aucune clé i18n** — la garde de
14-3b et son message existent déjà (AC 12).

### Fichiers à toucher

| fichier | nature |
|---|---|
| `crates/kesh-core/src/chart_of_accounts/mod.rs` | UPDATE — `ChartEntry.postable`, `is_postable`, tests |
| `crates/kesh-core/assets/charts/{pme,association,independant}.json` | UPDATE — 3 × 3 annotations |
| `crates/kesh-db/migrations/<date>_closing_accounts_not_postable.sql` | NEW — le backfill |
| `crates/kesh-db/migrations.sha384` · `test-schema/0001_schema_squash.sql` | UPDATE — P8 + squash |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | UPDATE — P6, **deux** nombres |
| `crates/kesh-db/src/post_restore.rs` | UPDATE — P7, registre ou exemption justifiée |
| `docs/migrations-idempotence-audit.md` | UPDATE — P5, ligne + **cinq** compteurs |
| `crates/kesh-db/tests/accounts_role_backfill.rs` | UPDATE — l'invariant seed ≡ backfill |
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

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
