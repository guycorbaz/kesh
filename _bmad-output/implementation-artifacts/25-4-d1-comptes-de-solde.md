# Story 25.4-d1 : Les comptes de solde — escompte, frais bancaires, perte sur débiteur

Status: done

**Issue : [#384]** — ⛔ la PR porte `refs #384` : la **25-4-d2**, qui écrit le solde, la fermera.

**Première des deux stories de la 25-4-d** (« solder le reste »). Empilée sur la 25-4-e (branche
`story/25-4-d1-comptes-de-solde`).

## Arbitrages (Project Lead, 2026-10-01)

1. Quatre natures d'écart : **escompte accordé**, **frais bancaires** retenus par la banque du client, **perte sur
   débiteur**, et **reste d'arrondi** (#490, sur le compte de différences d'arrondi déjà en place).
2. L'utilisateur **choisit la nature** au moment de solder ; **aucun seuil**.
3. **Un réglage par nature** dans *Paramètres → Facturation*, avec un compte par défaut des plans livrés, désigné
   d'office à la création d'une société.
4. **TVA corrigée au prorata des taux** de la facture pour l'escompte et la perte ; pas pour les frais ni l'arrondi.
5. Bouton **« Solder le reste »** sur la fiche, annulable par contre-passation.

Et, sur recommandation acceptée le 2026-10-01 :
- **le rapport TVA est étendu** pour retrancher la TVA des soldes (d2) ;
- **comptes par défaut** : ajouter **3805 *Pertes sur créances*** aux trois plans ; frais bancaires sur le **6900
  *Charges financières*** existant ; escompte sur le **3800 *Déductions sur ventes*** (PME, indépendant) — **rien**
  dans le plan des associations, dont le 3800 est « Autres produits » ;
- toute facture **validée non soldée** peut être soldée, réglée en partie ou pas du tout (d2).

## Découpage

- **d1 (celle-ci)** : les comptes et les réglages — le marqueur de plan, 3805, la désignation d'office, les trois
  colonnes de réglage, leur API, leur écran, le manuel admin.
- **d2** : l'écriture de solde (nature, ventilation HT/TVA au prorata, journal OD), le bouton et son dialogue,
  l'annulation, le rapport TVA, le manuel utilisateur ; ferme #384 et #490.

## Story

En tant que comptable,
je veux que chaque nature d'écart ait son compte, proposé par le plan et modifiable dans les paramètres,
afin que solder le reste d'une facture impute l'écart au bon compte sans que j'aie à le choisir à chaque fois.

## Les faits, vérifiés dans le code (inventaire du 2026-10-01)

- **Les plans** (`crates/kesh-core/assets/charts/{pme,independant,association}.json`, 85/85/82 entrées) : 3800
  (parent `30`, Revenue : « Déductions sur ventes » / « Déductions sur prestations » / **« Autres produits »**), 6900
  « Charges financières » (Expense, parent `6`), 6940 marqué `roundingDifference`. **Aucun compte de pertes sur
  créances.** Aucun de ces comptes ne porte de rôle.
- **Le marqueur `roundingDifference`** (Story 25-4-c3-a2) est le patron : `ChartEntry.rounding_difference`
  (`chart_of_accounts/mod.rs:206-216`), sa validation (`:317-355` : au plus un, charge ou produit, imputable, sans
  rôle), `rounding_difference_number` (`:363-368`), et sa désignation à l'onboarding par `rounding_account_from_chart`
  (`company_invoice_settings.rs:266-306`), appelée par les deux `insert_with_defaults*` (`:465`, `:593`).
- **Les réglages** : chaque colonne ajoutée touche une vingtaine de sites (inventaire, rubrique 7) — entité (deux
  structs), `COLUMNS`, `settings_snapshot_json`, les **deux `SELECT` `cis.`-préfixés** des `insert_with_defaults*`,
  `UPDATE`, `is_no_op_change`, les deux `INSERT IGNORE`, route (réponse, requête `double_option`, validation
  `validate_account_of` seulement si la valeur change), export CSV, types frontend, écran, montages de test.

## Acceptance Criteria

**AC 1 — Le marqueur de plan.** `ChartEntry.write_off_nature: Option<WriteOffNature>` (`#[serde(default)]`, JSON
`writeOffNature`), `enum WriteOffNature { Discount, BankFees, BadDebt }` (`"discount" | "bankFees" | "badDebt"`).
`validate_chart` : **au plus une entrée par nature** (et une seule nature par entrée, que le type `Option` garantit), charge ou produit, imputable, sans rôle — les règles du marqueur
d'arrondi, **factorisées** dans un helper commun (et non recopiées), **paramétré par le libellé** : les messages du
marqueur d'arrondi restent **mot pour mot** ceux d'aujourd'hui, dont dépendent quatre tests existants
(`chart_of_accounts/mod.rs:994, :1008, :1019, :1038`, qui doivent passer sans modification). ⛔ **Une entrée ne porte
pas deux marqueurs** (`roundingDifference` et une nature, ou deux usages d'un même compte) : refus, sans quoi un même
compte recevrait deux réglages distincts. `write_off_account_number(entries, nature)`. Tests unitaires : doublon,
mauvais type, non imputable, avec rôle, double marqueur → refus.

**AC 2 — Les plans livrés.**
- **3805** ajouté aux trois plans : parent `30`, Revenue, « Pertes sur créances » / « Verluste aus Forderungen » /
  « Perdite su crediti » / « Bad debt losses », marqué `badDebt`.
- **6900** marqué `bankFees` dans les trois plans.
- **3800** marqué `discount` dans **pme** et **independant** seulement.
- ⚠️ **Aucun test ne compte aujourd'hui les entrées exactes des plans** (seulement des bornes, `mod.rs:425, :439, :458`,
  et une comparaison dynamique, `accounts_role_backfill.rs:148-150` ; validation P1) : **en ajouter un**, qui fige
  86/86/83 — l'état **après** l'ajout du 3805, contre 85/85/82 aujourd'hui — et les marqueurs attendus de chaque plan — l'ajout du 3805 ne serait vu par rien d'autre.

**AC 3 — La migration** (non breaking) : `company_invoice_settings.default_discount_account_id`,
`default_bank_fees_account_id`, `default_bad_debt_account_id`, `BIGINT NULL`, FK `ON DELETE RESTRICT` vers `accounts`
(patron `20260930000001`). **DDL seul, aucune donnée écrite : triage P7 sans objet.** Garde-fous : P5 (audit, 73), P6 (`migrations_upgrade_path.rs` 72 → 73, 38 → 39), squash,
`migrations.sha384`, export de souveraineté, sauvegarde antérieure → `NULL`.

**AC 4 — La désignation d'office.** Les deux `insert_with_defaults*` désignent, pour chaque nature, l'entrée marquée
du plan de la forme juridique (un helper générique `write_off_account_from_chart(tx, company_id, nature)`, sur le
patron de `rounding_account_from_chart`, idéalement le même code paramétré) : `None` sans erreur si le plan n'en
marque pas (association pour l'escompte). **Rien chez les sociétés existantes.**

**AC 5 — Le réglage, API et écran.** À tous les sites de l'inventaire : entité, `COLUMNS`, `settings_snapshot_json`,
les deux `SELECT` `cis.`, `UPDATE`, `is_no_op_change`, les `INSERT IGNORE`, route (`double_option` : absent préservé,
`null` effacé ; validation **charge ou produit, actif, imputable, de la société**, seulement si la valeur change),
export CSV, types frontend, montages de test. Écran : une section **« Solde du reste »** dans *Paramètres →
Facturation* avec trois sélecteurs (escompte, frais bancaires, perte sur débiteur) et une aide ; libellés dans les 4
locales, `sitesTotal` recompté, `data-testid`.

**AC 6 — Le manuel admin.** `admin-manual.tex`, paramètres de facturation : les trois comptes, leurs défauts par
plan, l'absence de défaut d'escompte pour les associations. PDF régénéré, contrôlé aplati. CHANGELOG `[0.12.1]`
*Added* (#384), en annonçant que le solde lui-même suit (d2).

**AC 7 — Tests.** Chacun aurait échoué avant le patch : validation du marqueur (cas d'AC 1) ; les plans livrés se
valident et marquent ce qui est attendu ; une société créée depuis chaque plan a ses comptes désignés (association :
escompte `NULL`) ; une société existante n'est pas touchée ; API : posé, préservé, effacé, refus d'un compte actif de
bilan, archivé, non imputable, d'une autre société ; Vitest de l'écran ; sauvegarde sans les colonnes.

## Tasks / Subtasks

- [x] **T1 — le marqueur** (AC 1).
- [x] **T2 — les plans** (AC 2).
- [x] **T3 — la migration** (AC 3).
- [x] **T4 — la désignation** (AC 4).
- [x] **T5 — le réglage** (AC 5).
- [x] **T6 — textes** (AC 6).
- [x] **T7 — tests** (AC 7).
- [x] **T8 — gates** : backend complet (migration), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Un rôle de compte** pour ces natures : un marqueur de plan et un réglage, comme le compte d'arrondi.
- ⛔ **Un numéro de compte dans le code** : il vit dans les plans.
- ⛔ **Recopier trois fois la validation et la désignation** du marqueur d'arrondi : un helper paramétré.
- ⛔ **Toucher les sociétés existantes** : elles désignent leurs comptes elles-mêmes.

### Modules

`kesh-core` (marqueur, plans), `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n` (+ `docs`) — cinq modules de code, au
seuil sans le dépasser.

⚠️ **La charge, elle, est triple** (validation P1) : la seule colonne d'arrondi touche 68 occurrences dans 16
fichiers (`grep -rn "default_rounding_account_id\|defaultRoundingAccountId" crates/ frontend/src | wc -l`) ; trois
colonnes en font **une soixantaine de sites mécaniques**. Les faire en **un seul passage par fichier** (les trois
champs ensemble), et vérifier chaque site par le compilateur et par le test du `SELECT` `cis.` (chemin idempotent).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Mon `prettier` lancé à la main a reformaté tout `invoices.types.ts` (235 lignes de bruit, la
  configuration du dépôt n'étant pas celle qu'il a prise). Vu au `git diff --numstat` avant le
  commit ; fichier rétabli, seuls les trois champs réappliqués, gate frontend **relancé** sur
  l'état réel.
- Les montages de test qui appelaient `update` avec `None` pour les trois nouveaux champs les
  auraient **effacés** — la société de test les reçoit d'office du plan PME — et changé le sens
  des tests de no-op. Ils reprennent la valeur de la société, comme le compte d'arrondi.

### Completion Notes List

- **T1** — `WriteOffNature { Discount, BankFees, BadDebt }` (JSON `discount`/`bankFees`/`badDebt`),
  champ `ChartEntry.write_off_nature`. La validation du marqueur d'arrondi est **factorisée** dans
  `validate_designated_account`, paramétré par un libellé (singulier, pluriel) ; les messages de
  l'arrondi sont produits mot pour mot (`ROUNDING_LABEL`) et ses quatre tests passent sans
  modification. Double marqueur refusé. `write_off_account_number(entries, nature)`.
- **T2** — 3805 *Pertes sur créances* (Revenue, parent `30`, quatre langues) marqué `badDebt` dans
  les trois plans ; 6900 marqué `bankFees` dans les trois ; 3800 marqué `discount` dans PME et
  indépendant. Plans à 86/86/83, figés par `shipped_charts_have_exact_counts_and_write_off_markers`.
- **T3** — `20261001000003_invoice_settings_write_off_accounts.sql` (trois `BIGINT NULL`, FK
  `ON DELETE RESTRICT`, DDL seul). Audit à 73 lignes (65 `tracked-by-sqlx` + 8 `yes`, recomptés
  depuis le tableau) ; `migrations_upgrade_path.rs` 72 → 73 et 38 → 39, frontière 34 inchangée, résidus
  grepés (`\b(72|38|37)\b` : seules les généalogies restent) ; les autres sites P6 résolvent par
  version ; squash régénéré par le script ; `migrations.sha384` complété ; export CSV.
- **T4** — `rounding_account_from_chart` devient `chart_designated_accounts` : le plan est lu **une
  fois**, et une seule requête paramétrée (`designated_account_id`) cherche chaque compte marqué
  (arrondi, puis les natures dans l'ordre de `WriteOffNature::ALL` — ordre de verrouillage fixe).
  Les deux `insert_with_defaults*` posent les quatre colonnes ; absence → `None` sans erreur.
- **T5** — entité (deux structs), `COLUMNS`, snapshot d'audit, `is_no_op_change`, `UPDATE`, les deux
  `SELECT cis.`, les deux `INSERT IGNORE`. Route : `double_option` pour les trois champs ; la
  résolution « absent préservé, validé s'il change » est factorisée dans
  `resolve_designated_account`, partagé avec le compte d'arrondi. Écran : section *Solde du reste*,
  trois sélecteurs générés d'une seule liste (`writeOffFields`), filtrés charge/produit actifs
  imputables, compte choisi gardé visible (#271). Cinq clés dans les quatre locales ; `sitesTotal`
  1769 → 1775 (41 → 47 sites dans la page : cinq clés et le « — Sélectionner — » de la boucle).
- **T6** — `admin-manual.tex` § *Comptes du solde du reste*, PDF régénéré et contrôlé aplati ;
  CHANGELOG `[0.12.1]` *Added* (#384), qui annonce que le bouton suit.
- **T7** — périmètre `HEAD` (835130d9) → arbre de travail, recompté aux deux bornes :
  **11 tests Rust neufs** (7 `kesh-core` : 36 → 43 ; 4 `company_invoice_settings_repository` :
  18 → 22), **4 étendus** (les deux tests d'API du compte d'arrondi, généralisés aux quatre comptes
  désignés et renommés ; la sauvegarde sans les colonnes ; la finalisation d'onboarding de bout en
  bout), **2 Vitest** (5 → 7), **1 spec E2E** (`invoice-settings-write-off-accounts.spec.ts`).
  Trois contre-épreuves faites et restaurées : validation des natures neutralisée → 5 tests
  `kesh-core` rouges ; escompte non posé à l'`INSERT` → 4 tests rouges (dépôt et onboarding) ;
  compte de pertes non envoyé par l'écran → le Vitest rouge.
- **Gates** — base remise à zéro ; `scripts/test-fast.sh` (fmt + clippy + nextest) **vert, 2588/2588** ;
  frontend `check` (0 erreur, 27 avertissements préexistants, identiques sur `HEAD`),
  `lint-i18n-ownership`, `test:unit` **853/853**, `build` : verts.
- **E2E complet** — base `kesh_e2e` reconstruite, montage de `docs/testing.md` (SMTP, inbox,
  documents ; `/health` → `smtpConfigured:true`), run à 17:15 UTC : **231 passés, 8 échecs, 19
  ignorés**. Sept sont les **KF-029** de la liste nominative (`mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) ; run postérieur à 12:00 UTC, donc
  pas de KF-045. Le huitième était **la spec neuve de cette story** : elle lisait les options avant
  l'arrivée des comptes (requête de `onMount`) — défaut du test, non de l'écran. Corrigée (attente
  d'une option de compte), **rejouée seule : verte** ; contre-épreuve faite (frais bancaires non
  envoyés, frontend reconstruit → `toHaveValue` rouge au rechargement ; restauré → vert). La suite
  complète n'a pas été relancée après cette correction, qui ne touche que la spec.

### File List

- `CHANGELOG.md`
- `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/fiscal_years_e2e.rs`
- `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`
- `crates/kesh-core/assets/charts/association.json`
- `crates/kesh-core/assets/charts/independant.json`
- `crates/kesh-core/assets/charts/pme.json`
- `crates/kesh-core/src/chart_of_accounts/mod.rs`
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/migrations/20261001000003_invoice_settings_write_off_accounts.sql` (nouveau)
- `crates/kesh-db/src/entities/company_invoice_settings.rs`
- `crates/kesh-db/src/repositories/accounts.rs`
- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/test-schema/0001_schema_squash.sql`
- `crates/kesh-db/tests/company_invoice_settings_repository.rs`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/migrations-idempotence-audit.md`
- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`
- `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
- `frontend/tests/e2e/invoice-settings-write-off-accounts.spec.ts` (nouveau)

## Change Log

- **2026-10-01** — Créée après l'inventaire et les arbitrages (Q1–Q3 acceptées) : marqueur `writeOffNature`, 3805 dans
  les trois plans, 6900 et 3800 marqués, trois réglages désignés d'office, écran, manuel admin. Découpage d1 / d2.
- **2026-10-01** — Validation P1 (Sonnet) : 4 MED, 1 LOW, retenus. Le test de comptage des plans annoncé n'existait pas
  → à créer ; double marqueur interdit (test) ; messages du marqueur d'arrondi gardés mot pour mot (quatre tests en
  dépendent) ; charge chiffrée (une soixantaine de sites) ; P7 sans objet écrit.
- **2026-10-01** — Validation P2 ciblée (Haiku) : 1 MED rendu, **reclassé LOW** — les comptes 85/85/82 (« Les faits »,
  l'état actuel) et 86/86/83 (AC 2, l'état visé) ne se contredisent pas ; précisé quand même, comme « une nature par
  entrée ». **Boucle close** : 4 MED/1 LOW → 0 au-dessus de LOW ; Sonnet → Haiku ; remédiation sur la fiche seule.

- **2026-10-01** — Implémentée (T1–T8) : marqueur `writeOffNature` et validation factorisée avec
  celle de l'arrondi, 3805 et marqueurs dans les trois plans, migration n° 73, désignation d'office
  par un helper commun, réglage (API, écran, i18n), manuel admin, CHANGELOG. 11 tests Rust neufs,
  4 étendus, 2 Vitest, 1 spec E2E. Gates : backend 2588/2588, frontend 853/853, E2E 231/8 (7 KF-029,
  1 spec neuve corrigée et rejouée seule). Statut → review.
- **2026-10-01** — Revue de code P1 (Sonnet, lentilles A et C, prompt versionné) : **0 au-dessus de LOW**,
  3 LOW. Appliqués : le double marqueur est contrôlé **avant** les validations par marqueur (une
  entrée doublement marquée et de mauvais type recevait le message de type) — test
  `validate_chart_names_the_double_marker_before_the_type` ; « sur ce compte » du manuel admin,
  devenu lointain de son antécédent, nomme le compte de différences d'arrondi (PDF régénéré, contrôlé
  aplati). Laissé : le libellé « différences d'arrondi » en dur du message de double marqueur, exact
  tant qu'il n'existe que deux marqueurs. Gate ciblé `kesh-core` vert (44 tests), clippy propre.
- **2026-10-01** — Revue de code P2 ciblée (Haiku, sur la seule remédiation `b093bab1`, prompt versionné) :
  **0 finding** ; axes déclarés tous exercés, et l'affirmation vérifiable (« aucun autre *ce compte*
  ambigu ») recontrôlée par l'orchestrateur au `grep`. **Boucle close** : P1 3 LOW (2 appliqués) → P2 0 ;
  Sonnet → Haiku. Gate complet au dernier commit, base remise à zéro : **2589/2589**. Statut → done.

[#384]: https://github.com/guycorbaz/kesh/issues/384
