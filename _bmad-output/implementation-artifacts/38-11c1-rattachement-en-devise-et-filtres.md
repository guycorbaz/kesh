# Story 38-11c1 : rattachement en devise, autre côté à deux branches, filtres et sens des règles

Status: done

*Fiche née le 2026-11-20 du découpage de l'ancienne 38-11c (validation P3, décision de l'orchestrateur, proposée à la
validation P1), branche `epic/38-paypal`, numéros de ligne relevés sur `origin/main` `fd04099f`. Epic 38, **phase 1
KISS**. **Dépend de** : 38-11b (colonnes `provider_type`, `counterparty_email`, `direction` ; comptes PayPal en
devise). **Suivie par** : 38-11c2 (revue et acceptation, qui s'appuie sur cette story), 38-11d (écrans). ⚠️
**Numérotation gardée** : les AC-1 à AC-3 sont ceux de l'ancienne 38-11c, sans changement de numéro ; l'AC-1b est
neuve (prédicat de l'autre côté, item 5 et 7 des décisions de la P3). Une mention « 38-11c AC-1 » à « AC-3 » dans une
autre fiche (38-11d, Epics 39 et 40) désigne cette story ; « 38-11c AC-4 » à « AC-11 », la 38-11c2.*

## Story

En tant que **Comptable** qui tient des comptes PayPal en devise, je veux rattacher une transaction en devise à une
écriture déjà passée, que les deux côtés d'une écriture qui touche deux de mes comptes en devises différentes
(conversion) se rattachent et se défassent comme un virement entre mes comptes, et que mes règles d'affectation
puissent se restreindre à un type PayPal, à un e-mail exact et à un sens, afin que le rapprochement PayPal (38-11c2)
ait de quoi travailler.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11c1-1` | Le rattachement à une écriture existante (v017-3a) vaut aussi pour une transaction **en devise** sur un compte bancaire lié à un compte du plan de cette devise, le mouvement se comptant en devise. | AC-1 |
| `FR-38-11c1-2` | L'autre côté d'une écriture (34-11b) est, au choix, une transaction de même devise et de montant opposé (virement), ou une transaction d'une **autre devise** et de signe opposé (conversion) : **un** texte à deux branches, pour les trois prestataires. | AC-1b |
| `FR-38-11c1-3` | Filtres facultatifs des règles d'affectation, combinés au critère existant : **type PayPal** (exact) et **e-mail exact** ; **sens** facultatif. | AC-2, AC-3 |
| `NFR-38-11c1-1` | Comportement d'un compte CHF inchangé : tests existants verts sans changement d'assertion. | AC-1, AC-1b, AC-3b |

## Critères d'acceptation

1. **AC-1 — Rattachement en devise** (`crates/kesh-db/src/repositories/reconciliation_link.rs`).
   - La condition 1 (`transaction_can_be_linked_to_entry`, `:535-537`, « montant non nul **et** CHF ») devient :
     montant non nul **et** `t.currency` = devise du compte du plan **actuel** du compte bancaire de `t` (`CHF` quand
     ce compte n'a pas de devise). Signature : la devise du compte du plan en second argument ; ses appelants
     (`find_entry_candidates` `:879`, `find_matched_elsewhere` `:905`, `count_already_booked_for_page` `:978`,
     `link_entry_in_tx` `:1174`, test `:1313`) la passent — un compte CHF garde exactement le comportement
     d'aujourd'hui (tests existants verts ; test `:1313-1321` gardé et complété : `EUR` sur compte EUR vrai, `EUR` sur
     compte CHF faux, `USD` sur compte EUR faux).
   - **D'où vient la devise** : lue avec le compte du plan au pré-vol, sur le pool (`link_preflight`,
     `routes/reconciliation.rs:6552`, rend le couple compte du plan, devise ; `LinkEntryRequest`
     (`reconciliation_link.rs:1060`) gagne `bank_ledger_currency` ; `link_preflight` est **partagé** avec les routes
     des règlements de la 29-8 (`get_link_candidates` `:6606`, `post_link_transaction` `:6697`), qui ignorent la devise
     rendue et restent CHF — quatre appelants à mettre à jour), jamais dans le geste, qui ne prend pas `bank_accounts`
     (doc de `link_entry_in_tx`). Lecture sûre hors verrou : la devise d'un compte qui porte une ligne ne change plus
     (`ACCOUNT_CURRENCY_LOCKED`, 37-1), et sans ligne aucune écriture n'a de mouvement à égaler. `get_proposals`
     (`:1161-1172`) passe celle du compte de la page, lue avec le compte bancaire de la page (`ba_check` ne porte que
     `journal_account_id` : une lecture de la devise de ce compte du plan s'ajoute, sur le pool, à côté de `ba_check`).
   - Le mouvement de la condition 3 se compte **dans la devise du compte** quand elle n'est pas CHF :
     `ENTRY_MOVEMENTS` (`:800-804`) devient une fonction du genre de mouvement — CHF : `SUM(l.debit) - SUM(l.credit)`
     (texte d'aujourd'hui) ; devise : `SUM(CASE WHEN l.debit > 0 THEN l.amount_currency ELSE -l.amount_currency END)`
     (une ligne de réévaluation, `amount_currency = 0`, `chk_jel_fx_amount`, n'y pèse rien ; une ligne d'un compte en
     devise porte son montant en devise, exigé par le contrôle central des lignes en devise de 37-3a,
     `ACCOUNT_REQUIRES_CURRENCY`) — appelée par **`entry_sql`** (`:629`, liste, geste, second côté) **et par
     `count_already_booked_for_page`** (`:1003-1011`, la lecture par page du signal « déjà comptabilisée », qui ne
     passe pas par `entry_sql`) : sans cela, une transaction EUR de la page serait comptée contre des mouvements CHF.
     Tests `the_restricted_entry_query_is_the_list_query_plus_the_entry`, `the_matched_elsewhere_query_is_the_list_query_plus_the_link`
     étendus aux deux genres ; `s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue`
     (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs:2443`) étendu à une page EUR. La somme des règlements
     de la page (`LINKABLE_SETTLEMENTS`) reste CHF (`transaction_can_be_linked`, `:90`).
   - Les routes `GET …/entry-candidates` (`routes/reconciliation.rs:6872`) et `POST …/link-entry` (`:6983`) en
     héritent sans changement de forme ; leurs docs Rust « hors CHF » (`:121`, `:6859-6860`) suivent (grep du symptôme
     « hors CHF » dans `routes/reconciliation.rs` et `reconciliation_link.rs`, au Change Log). `docs/api-external.md`
     (`:926`, `:944`, `:1293`) et le miroir `canLink` de l'écran (`ReconciliationProposals.svelte:285`) relèvent de
     38-11d.
   - ⚠️ Le rattachement des **règlements** de la 29-8 (`get_link_candidates` `:6606`, `post_link_transaction`
     `:6697`, `transaction_can_be_linked` du dépôt `:90`) **reste CHF** : les ventes des boutiques sont des
     écritures, non des règlements de facture. `get_proposals` (`:1091`, saut hors CHF `:1189-1196`) et
     `accept_one_rule` (`:4191`, refus hors CHF `:4385-4395`) restent CHF : le rapprochement PayPal a ses propres
     routes (38-11c2 AC-6, AC-7).
1b. **AC-1b — L'autre côté à deux branches** (même fichier ; décision de l'orchestrateur du 2026-11-20 : **cette
   story** porte le texte pour PayPal, Wise et Stripe ; 39-11c AC-3 et 40-7c AC-1 le réutilisent).
   - `other_side_predicate` (`:698-709`) devient, mot pour mot dans sa structure :
     `(bt.bank_account_id <> ? AND ((bt.currency = ? AND bt.amount = -(M)) OR (bt.currency <> ? AND SIGN(bt.amount) = -SIGN(M))) AND exclusions)`
     — `exclusions` : les deux `NOT EXISTS` des règlements d'aujourd'hui (`invoice_settlements`, facture fournisseur
     `paid`), inchangés. Première branche : même devise, montant opposé exact (virement entre ses comptes, retrait
     PayPal CHF vers la banque, versement Stripe reçu sur Wise) ; seconde : devise différente, signe opposé (les deux
     côtés d'une conversion à une écriture, 38-11c2 AC-8). ⛔ Aucun `AND bt.currency = …` isolé : il contredirait la
     seconde branche.
   - **Paramètres** : la devise de `t` (deux fois) suit `t.bank_account_id` ; elle est **commune à toute la page**
     (devise du compte du plan, comme `t.bank_account_id` ; après la condition 1, `t.currency` est cette devise). `M`
     vaut `m.movement` dans la liste, le geste et la page (`linkable_side`, `:739`) ; dans `transfer_other_side`
     (`:773`), `M` = `?` = `t.amount`, **lié deux fois**. L'ordre des liaisons change dans `query_entry_sql` (`:818`),
     `count_already_booked_for_page` (commentaire « Ordre des `?` ») et `transfer_other_side` : relu à la revue de
     code, chaque `bind` nommé. **Ordre des `?` du texte** : compte bancaire de `t`, devise de `t`, `M`, devise de `t`,
     `M` — les deux devises et les deux `M` sont **entrelacés**, non groupés ; dans `transfer_other_side` : `company_id`,
     `entry_id`, `t.id`, puis `t.bank_account_id`, `t.currency`, `t.amount`, `t.currency`, `t.amount` ; dans la liste, le
     geste et la page, `M` est `m.movement` (aucune liaison) : compte, devise, devise.
   - Toutes les dérivées en héritent sans code propre : condition 4 de la liste et du geste (`linkable_side`),
     second côté (`find_matched_elsewhere` `:896`, `HAS_LINK`), décompte de page, annulation
     (`transfer_other_side`, lue par `reconciliation_cancel::entry_cancel_outlook` et `cancel_in_tx`,
     `reconciliation_cancel.rs:338`, `:397`) : annuler un côté d'une conversion à une écriture **ne retire que son
     lien**, l'écriture reste ; annuler le second la contre-passe (34-11b).
   - **Le plafond « une transaction liée au plus »** (`linkable_side`, doc `:730-737`) reste inerte : chaque branche
     impose `SIGN(bt.amount) = -SIGN(M)` ; deux transactions liées sont de signes opposés (la seconde n'est entrée
     qu'en étant l'autre côté de la première), une troisième devrait être de signe opposé aux deux — impossible. La
     démonstration remplace celle de la doc (`bt₁.amount = −M = bt₂.amount`), qui ne vaut plus pour la seconde
     branche ; un test de comportement le garde quand même (ci-dessous).
   - **Inerte pour l'existant** : tant qu'aucun compte bancaire n'est en devise hors prestataire (C-37-9), toute
     transaction liée est en CHF et la seconde branche ne s'active jamais ; tests existants verts sans changement
     d'assertion, `the_settlement_exclusion_matches_the_classification` (`:1381`) compris. Doc Rust de
     `other_side_predicate` et de la règle (en-tête, condition 4, « montant opposé ») mise à jour.
   - **Tests** (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`, société multi-devises, deux comptes PayPal
     de 38-11b : CHF sur le compte bancaire A, EUR sur B ; écriture « crédit compte du plan CHF 100.00 / débit compte du
     plan EUR 95.00 EUR au CHF 100.00 » passée par `POST /api/v1/journal-entries`, cours manuel ; `t₁` = −100.00 CHF sur
     A, `t₂` = +95.00 EUR sur B, insérées par le dépôt) : (a) `t₁` rattachée par `link-entry` ; (b) `t₂` listée en
     second côté (`entry-candidates` ; `count_already_booked_for_page` sur la page de B) et rattachée ; (c) annulation de `t₁` →
     seul son lien retiré, écriture non contre-passée, `t₂` toujours rapprochée ; (d) `t₁` rattachable de nouveau ;
     (e) une troisième transaction refusée — +95.00 EUR sur B (même compte) et +95.00 EUR sur un compte bancaire C
     partageant le compte du plan EUR (`NotLinkable`) ; (f) annulation des deux côtés → la seconde contre-passe
     l'écriture ; (g) **le signe de la seconde branche** — écriture à trois lignes « débit compte du plan CHF 100.00 /
     débit compte du plan EUR 95.00 EUR / crédit un compte de produit CHF 200.00 », `t₁` = +100.00 CHF sur A rattachée,
     `t₂` = +95.00 EUR sur B **de même signe** : refusée (`NotLinkable`), absente du second côté, `transfer_other_side` de `t₁`
     vide (sans ce test, supprimer `SIGN(bt.amount) = -SIGN(M)` de la seconde branche laisse (a) à (f) verts, et la
     démonstration du plafond repose sur lui). Le test d'égalité de la 34-4 couvre un compte en devise et une paire de devises différentes. Un
     test unitaire du texte : les deux branches présentes, une seule occurrence de chaque `NOT EXISTS`.
2. **AC-2 — Filtres des règles** (`crates/kesh-reconciliation/src/rules.rs`, entité `ReconciliationRule`,
   `crates/kesh-db/src/entities/reconciliation_rule.rs` ; colonnes posées par la migration de 38-11b, **aucune valeur
   neuve de `ReconciliationMatchType`**). Une règle garde son critère (`match_type`, `match_value`, obligatoires) et
   gagne deux filtres facultatifs : `provider_type` — égalité, après `trim` et sans casse, avec
   `BankTransaction::provider_type` ; `counterparty_email` — égalité, tout blanc retiré et minuscules, avec
   `BankTransaction::counterparty_email`. `rule_matches` (`rules.rs:46-70`) = critère **et** chaque filtre posé **et**
   le sens (AC-3). Une transaction sans ces champs (CAMT, CSV, saisie) ne satisfait jamais un filtre posé : une règle
   filtrée ne vaut que pour les transactions d'un prestataire. Les huit tests existants restent verts sans changement
   d'assertion. Valeur stockée : la forme normalisée pour l'e-mail (un `@`, un caractère au moins de chaque côté,
   sinon 400 `VALIDATION_ERROR`), rognée pour le type (1 à 64 caractères).
3. **AC-3 — Sens et routes.** Colonne `direction` : `credit` → la règle ne vaut que pour un montant > 0, `debit` → < 0,
   `NULL` → les deux ; appliqué dans `rule_matches` pour tout critère (donc aussi au rapprochement CHF existant, où
   toutes les règles d'aujourd'hui ont `NULL` : comportement inchangé). Routes
   `crates/kesh-api/src/routes/reconciliation_rules.rs` (`post_create` `:259`, `patch` `:364`, `list` `:314`,
   `detail` `:342`) : `providerType`, `counterpartyEmail`, `direction` dans le corps et la réponse (`null` admis ; au
   `PATCH`, absent = inchangé, `null` = retiré ; `direction` autre que `credit`/`debit` → 400 `VALIDATION_ERROR`) ;
   `matchValue` reste obligatoire. Doublon (`uq_reconciliation_rules_match_active`, inchangé) → 409
   `RECONCILIATION_RULE_DUPLICATE` existant, **quels que soient les filtres et le sens** (limite KISS de 38-11b AC-1,
   testée). La trace d'audit des créations et modifications porte les filtres. Le compte de contrepartie d'une règle
   reste un compte **CHF** (garde de 37-1 inchangée) : un achat PayPal en devise s'écrit contre un compte de charge
   CHF, au cours (38-11c2 AC-8).
3b. **AC-3b — Non-régression.** Tests existants du rattachement, de l'annulation (34-11b), du signal « déjà
   comptabilisée » (34-4) et des règles verts sans changement d'assertion (seuls les littéraux de construction
   gagnent leurs champs ; nombre au Change Log). Aucun code d'erreur neuf (refus en `VALIDATION_ERROR` existant) :
   aucune clé i18n.

## Tâches

- [x] T1 — Condition 1, devise au pré-vol (quatre appelants de `link_preflight`), mouvement en devise (liste, geste,
  page), docs Rust (AC-1).
- [x] T2 — Prédicat de l'autre côté à deux branches, liaisons, doc du plafond, tests (a) à (f) (AC-1b).
- [x] T3 — Filtres et sens : `rule_matches`, dépôt et routes des règles (AC-2, AC-3).
- [x] T4 — Non-régression (AC-3b) ; gate complet.

## Stratégie de tests

- **Purs** (`rules.rs`) : règle filtrée par type, par e-mail, par sens, non filtrée ; transaction sans champs
  prestataire.
- **Dépôt et HTTP** : condition 1 (unitaire) ; rattachement EUR sur compte EUR par `entry-candidates` et `link-entry`,
  CHF inchangé ; signal « déjà comptabilisée » d'une page EUR ; tests (a) à (f) de l'AC-1b ; règles : création avec
  filtres et sens, `PATCH` absent / `null`, doublon de critère à filtres différents → 409.
- Gate **complet** (exception `kesh-db` : `reconciliation_link` et `reconciliation_rules` changent).

## Modules touchés (3)

`kesh-db` (`reconciliation_link`, `reconciliation_rules`, entité), `kesh-reconciliation` (`rules.rs`), `kesh-api`
(`reconciliation` pour le pré-vol, `reconciliation_rules`).

## Note de version

*Changé* — Une transaction en devise se rattache à une écriture existante sur un compte bancaire en devise ; les deux
côtés d'une conversion passée en une écriture se rattachent et se défont comme un virement entre vos comptes. Les
règles d'affectation acceptent deux filtres facultatifs, « type PayPal » et « e-mail exact », et un sens.

## Notes de développement

- ⚠️ Le prompt d'origine nommait `get_link_candidates` / `post_link_transaction` : ce sont les routes des
  **règlements de facture** (29-8). Les ventes des boutiques sont des écritures : la fonction juste est le
  rattachement à une écriture existante de la v017-3a (`find_entry_candidates` `:870`, `link_entry_in_tx` `:1156`).
- Si l'Epic 39 ou 40 arrive le premier dans `main` avec ce prédicat déjà écrit (leur repli), l'epic qui arrive le
  second reprend `main`, relit le texte et garde les tests de l'autre.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (worktree `kesh-wt38`, branche `epic/38-paypal`), 2026-11-20.

### Implementation Plan

- **AC-1** — `transaction_can_be_linked_to_entry(t, ledger_currency)` ; `bank_ledger_currency` (lecture sur le pool,
  `NotFound` si le compte manque) ; `link_preflight` rend `(compte, devise)` — quatre appelants de
  `routes/reconciliation.rs` (les deux routes 29-8 ignorent la devise), le cinquième (`reconciliation_group.rs:854`)
  jette le résultat, inchangé ; `LinkEntryRequest<'a>` gagne `bank_ledger_currency: &'a str` (reste `Copy`) ;
  `get_proposals` lit la devise à côté de `ba_check`. `ENTRY_MOVEMENTS` devient `entry_movements(MovementKind)`
  (`CHF_MOVEMENT_SUM`, texte d'avant ; `FOREIGN_MOVEMENT_SUM`, celui de la fiche), appelé par `entry_sql` et
  `count_already_booked_for_page`. Règlements groupés (`grouped_settlements.rs`, deux sites) : `BASE_CURRENCY`.
  Docs « hors CHF » des routes du rattachement à une écriture suivies (`:121`, doc de `get_entry_candidates`) ; celle
  de `get_link_candidates` (29-8) reste juste et est gardée.
- **AC-1b** — `other_side_predicate` au texte de la fiche ; liaisons nommées (`query_entry_sql` : compte, devise,
  devise ; page : idem ; `transfer_other_side` : compte, devise, montant, devise, montant) ; doc du plafond récrite
  (démonstration par les signes).
- **AC-2, AC-3** — `rule_matches` = `criterion_matches` ∧ `filters_match` (fonction publique pure) ;
  `normalize_email`, `DIRECTION_CREDIT/DEBIT` publics dans `kesh_reconciliation::rules`, réutilisés par la route
  (DRY) ; route : trois champs au corps (`Option`, double option au `PATCH`) et à la réponse, validation
  (`validate_provider_type` 1–64 rogné, `validate_counterparty_email` normalisé, un seul `@`, ≤ 254,
  `validate_direction` exact) ; trace d'audit des créations et modifications : les trois champs (avant/après).
- **Correction 38-11b P2-L1** — `paypal_imports::create` : `import_under_locks` (une tentative : verrous nommés +
  écriture) enveloppée par `retry_app_on_deadlock("paypal_imports::create")` ; registre point (vi) complété.

### Debug Log

- Premier passage des tests HTTP de l'AC-1b : `409 MULTI_CURRENCY_DISABLED` à la saisie de l'écriture — l'exercice
  du seed doit être `multi_currency = TRUE` (montage corrigé, comme `journal_entries_foreign_currency_e2e.rs:125`).
- Aucune migration, aucun code d'erreur neuf, aucune clé i18n.

### Completion Notes

- **Tests ajoutés : 17** (recomptés par `grep -cE '#\[(sqlx::)?test'` de `d8076fd9` au commit `ca34a3f4`) —
  `reconciliation_link.rs` 7 → 10 (condition 1 en devise, mouvement par genre, deux branches), `rules.rs` 8 → 12
  (type, e-mail, sens, tous filtres et règle non filtrée), `reconciliation_link_entry_e2e.rs` 59 → 63
  (`s38_11c1_rattachement_en_devise`, `s38_11c1_conversion_a_une_ecriture` = tests (a) à (f),
  `s38_11c1_le_signe_de_la_seconde_branche` = (g), `s38_11c1_la_lecture_par_page_eur_egale_les_fonctions_du_dialogue`),
  `reconciliation_rules_e2e.rs` 47 → 52 (création, refus, `PATCH` absent/`null`/valeur, doublon à filtres différents,
  sens et type au rapprochement), `paypal_import_e2e.rs` 10 → 11 (rejeu de l'import).
- **Non-régression (AC-3b)** — tests existants modifiés : littéraux et signatures seulement (quatre appels de
  `reconciliation_link_entry_e2e.rs` passent `CHF`, `transaction_can_be_linked_to_entry_is_a_non_zero_chf_amount`
  passe la devise) ; `the_restricted_entry_query_…` et `the_matched_elsewhere_query_…` bouclent sur les deux genres,
  assertions gardées. ⚠️ **Une assertion changée** : `the_linkable_side_and_the_cancel_side_share_one_predicate`
  vérifiait le texte même du prédicat (`starts_with("… AND bt.amount = -(?)")`, deux `?`) ; le texte change par
  l'AC-1b : `starts_with("(bt.bank_account_id <> ? ")` et cinq `?`. `the_settlement_exclusion_matches_the_classification`
  inchangé et vert. Aucun autre test existant touché.
- **Mutations** (`kesh-gate-logs/s38-11c1/mutations.py`, journal `mutations.log`) : 10 jouées, 9 rouges — M1 seconde
  branche sans `SIGN` (rougit **seul** le test (g)), M2 seconde branche retirée, M4 mouvement CHF pour un compte en
  devise, M5 condition 1 en CHF, M6 page en mouvement CHF, M7/M8/M9 sens/type/e-mail ignorés, M10 import PayPal sans
  enveloppe. **M3 survivante, équivalente** : retirer `bt.currency = ?` de la première branche ne change rien, car
  `bt.amount = −M` (M ≠ 0) implique le signe opposé — une transaction d'une autre devise qui satisfait la première
  branche mutée satisfait déjà la seconde. Le test unitaire `the_other_side_has_two_branches` garde le texte.
- **Gates au commit de code `ca34a3f4`** — gate ciblé (cible-1, avant le correctif du montage) puis
  `s38_11c1` 4/4 ; **gate backend complet** (base `kesh_s3811c1` remise à zéro : DROP/CREATE, migrations, seed),
  `scripts/test-fast.sh --ci` : fmt, clippy `-D warnings`, nextest **6220 exécutés, 6220 passés, 14 ignorés**
  (`kesh-gate-logs/s38-11c1/gate-backend.log`) ; **`scripts/gate-ci.sh`** sur une copie `git archive` de `ca34a3f4` :
  **6239 passés, 0 échoué, 18 ignorés** (`gate-ci.log`, cible supprimée après) ; **E2E complet** (frontend rebâti,
  base neuve `kesh_e2e_s3811c1`, `LANG=fr_CH.UTF-8`) : **477 passés, 10 échecs, 18 ignorés** (46 min, `e2e.log`) —
  les dix jugés un à un, tous attendus : les sept de la KF-029 (#97 : `mode-expert:26`, `:41`,
  `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`, `:150`) et `backup-restore-new-install:262`, `:381`,
  `opening-balances:156`. Frontend non modifié : `npm run check`, `lint-i18n-ownership`, `test:unit` non rejoués
  (seul `npm run build` l'a été, pour l'E2E).
- **Hors périmètre, laissé à 38-11d** (fiche) : `docs/api-external.md` (`:926`, `:944`, `:1293`), miroir `canLink`
  de l'écran ; les champs des règles ne sont pas décrits dans `api-external.md` (seul le code `ACCOUNT_NOT_POSTABLE`
  y nomme la route) — à écrire avec les écrans. Aucun frontend touché.
- Registre des routes : point (vi) à « trente-trois » **par ajout**, non recompté en entier (un balayage heuristique
  des handlers ne retrouve pas les 32 d'avant à l'identique : écart non élucidé ici, noté pour la revue).

### File List

- `crates/kesh-db/src/repositories/reconciliation_link.rs`
- `crates/kesh-db/src/repositories/grouped_settlements.rs`
- `crates/kesh-db/src/repositories/reconciliation_rules.rs`
- `crates/kesh-db/src/entities/reconciliation_rule.rs`
- `crates/kesh-reconciliation/src/rules.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/reconciliation_rules.rs`
- `crates/kesh-api/src/routes/paypal_imports.rs`
- `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs`
- `crates/kesh-api/tests/paypal_import_e2e.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `_bmad-output/implementation-artifacts/38-11c1-rattachement-en-devise-et-filtres.md`
- `_bmad-output/implementation-artifacts/38-11b-compte-paypal-et-import.md` (Change Log)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-11-20 — fiche née du découpage de l'ancienne 38-11c (AC-1 à AC-3, validées P1 et P2 dans celle-ci : voir le
  Change Log de 38-11c2) ; AC-1b neuve ; validation P3 (Claude Opus 5.5, contexte frais), rapport
  `kesh-gate-logs/epic-38-kiss/validation/38-11-P3.md`.
- 2026-11-20 — validation P4, ciblée sur l'AC-1b (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM,
  2 LOW, remédiés (rapport `kesh-gate-logs/epic-38-kiss/validation/38-11-P4-ciblee.md`). Test (g) du signe de la
  seconde branche, ordre entrelacé des liaisons écrit, origine de la devise de `get_proposals`.
- 2026-11-20 — développée (Claude Opus 5.5) : AC-1, AC-1b, AC-2, AC-3, AC-3b ; commits `8d241e52` (code), `ca34a3f4`
  (tests). Plus, hors fiche, la correction P2-L1 de la revue P2 de la 38-11b : import PayPal rejoué sur interblocage
  (`retry_app_on_deadlock`, opération `paypal_imports::create`), testé, noté au Change Log de la 38-11b. Grep du
  symptôme « hors CHF » : `routes/reconciliation.rs` (deux docs du rattachement à une écriture récrites, celle de la
  29-8 gardée), `reconciliation_link.rs` (aucune). Statut `review`.


- 2026-11-20 — revue de code P1 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL / 0 HIGH / 0 MEDIUM / 6 LOW ; boucle close.
  LOW versés à la dette : docs « montant opposé » de `reconciliation_cancel.rs:336-338, :777-778` ; lectures inutiles
  (`link_preflight` pour 29-8 et groupes, `LINKABLE_SETTLEMENTS` sur une page en devise, décompte non utilisé dans
  `get_proposals`) ; décompte « trente-trois » du registre non recompté ; test de rejeu dépendant du choix de victime
  InnoDB. Rapport : `kesh-gate-logs/s38-11c1/revue-p1-rapport.md`.
