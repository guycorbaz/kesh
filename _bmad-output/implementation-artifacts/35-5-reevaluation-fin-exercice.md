# Story 35-5 : réévaluation des comptes en devise à la fin de l'exercice

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, D5, C-35-7. Suit 35-4.
**Aucune migration attendue** (`rate_source = 'revaluation'` posé par 35-3a).*

## Story

En tant que comptable en multi-devises, je veux qu'à la fin de l'exercice Kesh me propose la réévaluation de mes
comptes de bilan en devise au cours mensuel du mois de clôture, et la passe en une écriture après mon contrôle, afin
que mon bilan présente ces comptes au cours de clôture et que l'écart aille aux différences de change.

## Critères d'acceptation

1. **AC-1 — Aperçu.** `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview { rates?: {USD: "0.8846", …},
   accountIds?: [...] }` → par compte de **bilan** (`Asset`, `Liability`) en devise, **imputable** (`postable = TRUE` ;
   les comptes de groupe ne sont jamais proposés) : solde en devise et solde CHF **cumulés depuis l'origine** jusqu'à
   la date de fin d'exercice (toutes années confondues, sans borne basse), cours retenu (défaut : cours AFC du **mois
   de la date de fin d'exercice**, `lookup` 35-2 ; surchargeable par devise), CHF réévalué = round(solde devise ×
   cours), écart, compte de contrepartie.
   - **Règle de calcul en débit net**, quel que soit le type du compte : `devise_net` = Σ `amount_currency` au débit −
     Σ au crédit sur les lignes de la devise du compte ; `chf_net` = Σ débit − Σ crédit ; **D = round(devise_net ×
     cours, 0.01, MidpointAwayFromZero) − chf_net**. D > 0 : débit du compte réévalué, crédit du compte de gains ;
     D < 0 : crédit du compte réévalué, débit du compte de pertes ; D = 0 : rien. Ce sens vaut pour tout signe de solde
     (un actif à solde créditeur dont le cours monte donne une perte ; un passif à solde débiteur suit la même règle).
     Les gains et pertes affichés sont Σ(D > 0) et Σ|D < 0|. Les soldes affichés gardent le signe naturel du type.
   - Un compte à solde en devise nul mais à solde CHF résiduel ≠ 0 **est proposé** (D = −chf_net) : il n'est pas écarté
     par un `HAVING balance != 0` de rapport.
   - Comptes présélectionnés : tous ceux de bilan en devise, actifs et imputables. Un compte **archivé** à solde ≠ 0 est
     listé avec `reason: ACCOUNT_ARCHIVED`, non sélectionnable, avec avertissement (sinon l'écriture entière serait
     refusée en `InactiveOrInvalidAccounts`, `validate_lines_accounts_in_tx`). Comptes `Revenue`/`Expense` jamais
     proposés.
   - Un cours manquant sans surcharge → la devise est signalée `RATE_UNAVAILABLE`, ses comptes exclus jusqu'à saisie.
   - **Cours résolus avant toute transaction**, par lecture de la table des cours (35-2 : `lookup` est en lecture
     seule, sans appel sortant). Un cours absent rend la devise `RATE_UNAVAILABLE` dans l'aperçu ; l'écran propose
     « Récupérer le mois » (`POST /exchange-rates/fetch`, 35-2, audité à part) ou la saisie du cours. Le serveur de 35-5
     **ne récupère jamais** de cours lui-même ; aucune récupération n'a lieu sous le verrou de l'exercice.
     L'aperçu **n'écrit rien**. *(Alignement de l'orchestrateur sur la remédiation P1 de 35-2, 2026-11-14.)*
2. **AC-2 — Passation.** `POST …/fx-revaluation { rates, accountIds }` (pas de `version` : `FiscalYear` n'en a pas ;
   l'idempotence vient du rejeu, AC-3, et du verrou de l'exercice). Le serveur ne fait jamais confiance à l'aperçu : il
   recalcule.
   - **Fonction de dépôt unique** `fx_revaluation::post(pool, user_id, company_id, fiscal_year_id, rates,
     account_ids)`, dans **une transaction** rejouée par `retry_on_deadlock("fx_revaluation::post", …)` comme
     `fiscal_years::close`. Elle verrouille l'exercice (`FOR UPDATE`), lit **dans cette transaction** les soldes cumulés
     et les comptes de gains et de pertes, puis appelle `create_in_tx(…, enforce_postable = true)` ; l'audit
     `journal_entry.created` est écrit par `create_in_tx_inner`. Une `rates` qui omet une devise sélectionnée : cours
     AFC lu dans la table des cours **avant** la transaction, `RATE_UNAVAILABLE` (404) s'il manque.
   - Une **seule** écriture journal `OD`, datée du dernier jour de l'exercice, libellé **en français**, comme la
     contre-passation (`journal_entries.rs:2975`) : « Réévaluation des comptes en devise au JJ.MM.AAAA (écarts non
     réalisés) » (≤ 500 caractères, `MAX_DESCRIPTION_LEN`).
   - Une ligne par compte d'écart non nul : `ForeignAmount { source: Revaluation, amount: 0 }`, `rate_source =
     'revaluation'`, `amount_currency = 0`, `exchange_rate` = cours retenu (> 0, par unité), CHF = |D|. **Cette ligne
     est exemptée de l'égalité CHF = montant × cours et de `FX_LINE_ROUNDS_TO_ZERO`** (le CHF est pris tel quel, > 0) ;
     la saisie manuelle de 35-3a (AC-6) la refuse, c'est la route de 35-5 qui la construit.
   - Lignes de contrepartie CHF : une par compte de contrepartie et par sens, agrégée. **Si le compte de gains et le
     compte de pertes sont le même, une seule ligne au net** (|Σ gains − Σ pertes|, du côté du solde) ; **aucune ligne
     si le net est nul** (les lignes des comptes réévalués s'équilibrent alors entre elles) ; à tester.
   - Aucun écart non nul → 200 sans écriture (`{ entryId: null }`).
   - **Audit** : outre `journal_entry.created`, une ligne d'audit propre `fx_revaluation.posted`
     `{ rates, source par devise (afc|manual), accountIds }` écrite **dans la même transaction** (la source du cours,
     perdue sur la ligne puisque `rate_source = 'revaluation'`, y est conservée) ; libellé d'audit dans les quatre
     locales. Limite connue : l'acteur d'une clé d'API n'apparaît pas à l'audit de la création d'écriture
     (`NewAuditLogEntry::user`), comme pour toute écriture existante.
3. **AC-3 — Rejouable.** Un second passage, sans mouvement entre-temps, ne passe rien ; après une écriture tardive en
   devise, il ne passe que le résidu. Pas d'extourne au premier jour de l'exercice suivant. **Le report à nouveau des
   soldes en devise se fait par le cumul** (`opening.rs:17`, Kesh ne passe aucune écriture de clôture) : aucune écriture
   de report n'est créée. Une réévaluation se corrige par **suppression ou contre-passation** de l'écriture, ou par un
   nouveau passage (résidu) ; le `PUT` de la saisie manuelle la refuse (35-3a AC-6, `revaluation` réservé à 35-5).
4. **AC-4 — Gardes.** Ordre des refus : exercice inexistant ou d'une autre société → 404 (le handler appelle d'abord
   `fiscal_years::find_by_id_in_company`) ; mono-devise → **409** `MULTI_CURRENCY_DISABLED` (409 partout, décision
   d'epic) ; comptes de différences de change **revalidés dans la transaction** (société, actif, imputable, de type
   `Expense`/`Revenue`, en CHF) → `FX_ACCOUNT_INVALID` (C-35-2) ; exercice clos → `FISCAL_YEAR_CLOSED` ; **exercice
   antérieur encore ouvert → `EARLIER_FISCAL_YEAR_OPEN`** (comme K1 : le cumul lu ne serait plus stable) ; exercice
   postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (garde `journal_entries.rs:368`, repository ; état hérité seulement,
   puisque `close` impose l'ordre) ; `DATE_OUTSIDE_FISCAL_YEAR` ; livres verrouillés à cette date
   (`books_locked_through`, seuil **inclusif** : `entry_date <= locked_through`) → `PERIOD_LOCKED`. Ces refus valent
   aussi pour l'aperçu (404, mono-devise, exercice clos). `accountIds` : doublons, comptes d'une autre société ou hors
   de la liste admissible (groupe, archivé, résultat, sans devise) → 400, sans fuite d'existence.
   - **Bornes de la requête** : chaque cours est un décimal > 0, ≤ 10 décimales, ≤ 10⁶, exprimé **par unité** de
     monnaie (1 JPY, pas 100 JPY), et sa clé est la devise d'au moins un compte sélectionné ; sinon 400
     `FX_RATE_INVALID`. Au plus 100 devises et **498** comptes par requête (l'écriture reste sous
     `MAX_LINES_PER_ENTRY` = 500, `routes/journal_entries.rs`, avec les deux lignes de contrepartie).
5. **AC-5 — RBAC.** Les deux routes vont dans `comptable_routes` (`lib.rs:524`, `require_comptable_role`), comme
   `…/close` (`lib.rs:821`) : Consultation reçoit 403 **y compris sur l'aperçu** (c'est un POST). Pas d'`ensure_not_pat`
   (comme `close`, contrairement à `reopen`), sauf décision contraire de l'utilisateur ; une clé en lecture seule est
   refusée sur le POST d'aperçu comme sur tout POST.
6. **AC-6 — Écran** : depuis *Exercices* (exercice ouvert), action « Réévaluer les devises » : tableau de l'aperçu,
   cours modifiables par devise (source affichée : AFC ou manuel), cases par compte (comptes archivés grisés avec
   avertissement), total des gains et des pertes, « Passer l'écriture », lien vers l'écriture passée ; captures d'écran.
7. **AC-7 — Tests** : deux exercices (achat EUR en N-1, réévaluation N-1, mouvements N, réévaluation N) : solde CHF au
   31.12.N = round(solde EUR × cours) exactement ; exercice clos en juin → cours de juin ; rejeu sans écriture ; compte
   décoché ou de résultat : aucune ligne ; cours surchargé figé sur la ligne ; refus des gardes. S'y ajoutent :
   - **les cinq cas de signe** : actif USD débiteur, cours en hausse (gain) puis en baisse (perte) ; actif USD à solde
     créditeur, cours en hausse (perte) ; passif USD créditeur, cours en hausse (perte) ; passif USD à solde débiteur ;
     devise à 0 avec CHF résiduel ≠ 0 ;
   - compte de groupe, archivé, non imputable ; plusieurs devises (USD et EUR) dans la même écriture ;
   - compte de gains = compte de pertes (une ligne au net ; net nul, aucune ligne de contrepartie) ;
   - cours invalide (0, négatif, 11 décimales, > 10⁶, devise sans compte) ; au-delà des bornes ;
   - `PERIOD_LOCKED` exactement à la borne (inclusive) ; exercice postérieur clos (test de dépôt, comme
     `opening_complement_repository.rs:963`) ; exercice antérieur ouvert ; exercice clos ; mono-devise ;
   - `FX_ACCOUNT_INVALID` (compte devenu archivé, d'une autre société, ou en devise) ;
   - RBAC (Consultation 403) et société (exercice d'une autre société 404) ;
   - audit : `journal_entry.created` avec les quatre colonnes de devise au snapshot, et `fx_revaluation.posted` ;
   - le résidu après écriture tardive ; deux passations concurrentes : l'une passe, l'autre ne passe rien ;
   - l'aperçu ne crée aucune écriture ; E2E de l'écran.
8. **AC-8 — Manuel** : réévaluation, cours du mois de clôture (choix assumé, D5), gains et pertes non réalisés
   comptabilisés (CO 960b, choix assumé ; la neutralisation par provision n'est pas offerte, C-35-8), postes non
   monétaires à décocher (stocks, immobilisations, capital en devise), la source du cours conservée à l'audit et non
   sur la ligne ; i18n ; PDF.

**Hors périmètre, tracé** : un contrôle de clôture informatif « comptes en devise à réévaluer » (K9 de
`closing_checks.rs`, dont l'ensemble K1–K8 est clos) — à ouvrir en issue avec la PR (C-35-8).

## Tâches

- [ ] `kesh-core` : calcul pur de la réévaluation en débit net (entrées : soldes, cours ; sortie : lignes) et tests.
- [ ] `kesh-db` : `fx_revaluation::post` (transaction, verrou d'exercice, soldes lus dans la transaction, audit
      `fx_revaluation.posted`). `kesh-report` : soldes cumulés à deux devises à une date, **avec exécuteur ou
      transaction**, sans borne basse ni `HAVING` (exigence portée par 35-4 AC-1).
- [ ] `kesh-api` : routes (aperçu, passation) dans `comptable_routes`, gardes, résolution des cours avant la
      transaction, i18n des erreurs et du libellé d'audit (quatre locales).
- [ ] Frontend : écran depuis les exercices, Vitest, E2E.
- [ ] Manuel, CHANGELOG.

## Notes de développement

- Kesh ne passe aucune écriture de clôture (`crates/kesh-report/src/opening.rs:17`) : les soldes de bilan sont
  cumulés depuis l'origine ; d'où le calcul « jusqu'à la date », toutes années confondues (risque relevé à la table
  ronde). Les fonctions actuelles de `kesh-report` prennent `&MySqlPool` (`balance_sheet.rs`,
  `fetch_cumulative_section`) : elles ne conviennent pas telles quelles sous le verrou.
- Exercices : `crates/kesh-db/src/repositories/fiscal_years.rs` (`close` :1116), `routes/fiscal_years.rs` (:352) ;
  garde « exercice postérieur clos » `crates/kesh-db/src/repositories/journal_entries.rs:368` (étape 1-a de
  `create_in_tx_inner`, lecture non verrouillante) ; `validate_lines_accounts_in_tx` (:135).
- `FiscalYear` n'a pas de `version` ; la lecture des cours (35-2) est sans réseau ; la récupération est un geste séparé de l'écran.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 7 MEDIUM, 5 LOW ; tous remédiés dans la fiche ; P1-1 (côté
  35-3a) et l'exigence 35-4 de P1-2 portés par l'orchestrateur.
