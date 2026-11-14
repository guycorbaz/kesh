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
     `balances_as_of` (35-4 AC-1 quater, dans `kesh-db`) rend directement le **débit net brut** (`chf_net`,
     `devise_net`), sans signe par type : aucune conversion de signe ; le signe naturel ne sert qu'à l'affichage. Les
     cinq cas de signe (AC-7) se testent sur le calcul pur de `kesh-core`. *(Alignement de l'orchestrateur sur
     35-4-P2-1, 2026-11-14.)*
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
   - **Avertissements de l'aperçu** (sans refus) : l'aperçu signale, sans écrire, ce que la passation refuserait —
     `EARLIER_FISCAL_YEAR_OPEN`, `LATER_FISCAL_YEAR_CLOSED`, `PERIOD_LOCKED`, `FX_ACCOUNT_INVALID` — dans un champ
     `blockers[]` (code et détails) ; l'écran désactive « Passer l'écriture » tant qu'il n'est pas vide.
2. **AC-2 — Passation.** `POST …/fx-revaluation { rates, accountIds }` (pas de `version` : `FiscalYear` n'en a pas ;
   l'idempotence vient du rejeu, AC-3, et du verrou de l'exercice). Le serveur ne fait jamais confiance à l'aperçu : il
   recalcule.
   - **Fonction de dépôt unique** `fx_revaluation::post(pool, user_id, actor_api_key_id, company_id, fiscal_year_id,
     rates, account_ids)` (`actor_api_key_id: Option<i64>`, propagé depuis `CurrentUser` par le handler : c'est
     l'argument qu'exige `NewAuditLogEntry::for_actor`, `entities/audit_log.rs:195`, pour l'audit ci-dessous), qui ouvre **une transaction** ; c'est le **handler** qui l'appelle dans
     `kesh_db::retry::retry_on_deadlock("fx_revaluation::post", …)`, comme `close_fiscal_year`
     (`routes/fiscal_years.rs:361`) — le registre des routes (ci-dessous) contrôle l'enveloppe au handler. Elle
     appelle ensuite `create_in_tx(…, enforce_postable = true)` ; l'audit `journal_entry.created` est écrit par
     `create_in_tx_inner`. Une `rates` qui omet une devise sélectionnée : cours AFC lu dans la table des cours
     **avant** la transaction, `RATE_UNAVAILABLE` (404) s'il manque.
   - **Verrous et instantané** (le même défaut que la règle de placement de `bank_accounts.rs:905-918`) : sous
     `REPEATABLE READ` (niveau par défaut), la vue cohérente naît à la **première lecture non verrouillante** de la
     transaction. Les soldes doivent donc se lire **après** tous les verrous, et **aucune lecture ordinaire ne
     précède** ces verrous — sinon la seconde de deux passations concurrentes, débloquée après le `COMMIT` de la
     première, relirait les soldes d'avant et passerait l'écart **une seconde fois**. Ordre des
     verrous d'exercice, celui de `fiscal_years::close` (`fiscal_years.rs:1116-1190`, antérieurs puis exercice, pour ne pas croiser
     une réouverture) — **mais pas ses étapes (a) et (b)**, qui sont des lectures ordinaires
     (`START_DATE_IN_COMPANY_SQL`, `LIST_EARLIER_SQL`, `fiscal_years.rs:166-175`) et fixeraient l'instantané avant tout
     verrou (`close` le dit de sa propre étape (d) : « la vue fixée à l'étape (a) » ; elle n'y perd rien, n'ayant aucun
     solde à lire). `post` lit donc la date de début de l'exercice et la liste des antérieurs **avant `begin`**, sur le
     pool (une date de début ne change pas : seul le nom d'un exercice se modifie), puis, dans la transaction :
     (0) la ligne `companies` (`multi_currency_enabled`, `fx_gain_account_id`, `fx_loss_account_id`) lue
     `LOCK IN SHARE MODE` (jamais `FOR SHARE`, erreur de syntaxe sur MariaDB 10.11) **avant tout exercice** : c'est
     l'ordre global `companies` → exercice des chemins Pattern 5 (`invoices.rs:698`, sentinelle puis `create_in_tx`) et
     de 35-3a AC-4 (`companies` en tête de l'étape 0 de `create_in_tx_inner`) ; prise après l'exercice, elle formerait
     le cycle exercice → `companies` contre `companies` → exercice que 35-3a écarte (aucun flux ne prend un exercice
     puis `companies` **en exclusif** : ni `close`, ni `reopen`, ni `lock_books`, `companies.rs:1020` ; la
     contre-passation d'une écriture en devise de 35-3a prend l'exercice puis `companies` en partagé, compatible) ; (1) chaque antérieur
     verrouillé par clé primaire (`LOCK_EARLIER_BY_ID_SQL`) dans l'ordre chronologique, puis
     l'exercice `FOR UPDATE` (`LOCK_IN_COMPANY_SQL`) ; (2) relecture **verrouillante** du plus ancien antérieur ouvert (`earlier_open_locking_sql`,
     même texte que `close`) → `EARLIER_FISCAL_YEAR_OPEN` ; (3) réglages de facturation et articles qui désignent les
     comptes de différences de change (`default_revenue_account_id`, contrôle de l'AC-4), puis comptes de gains et de
     pertes, lus `LOCK IN SHARE MODE` (ordre `companies` → réglages → `accounts` de 35-1 AC-3) ; (4) seulement
     ensuite, les soldes cumulés (`balances_as_of`, 35-4) et la liste des comptes admissibles. L'ordre des **verrous**
     n'est pas l'ordre des **refus** (AC-4) : tout se lit d'abord, puis s'évalue dans l'ordre de l'AC-4. Le test des deux
     passations concurrentes (AC-7) fait attendre la seconde sur le verrou de l'exercice pendant que la première
     commite, et constate une seule écriture.
   - **Source du cours** (pour l'audit) : par devise, `afc` si le cours retenu **égale** `rate_per_unit` de la table
     pour (devise, mois de fin d'exercice), qu'il vienne de la table ou de `rates` (l'écran renvoie les cours
     préremplis) ; `manual` sinon, y compris pour une devise non publiée.
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
     `{ entryId, rates, source par devise (afc|manual), accountIds }` écrite **dans la même transaction** (la source
     du cours, perdue sur la ligne puisque `rate_source = 'revaluation'`, y est conservée), entité `fiscal_year`
     (déjà au registre des types, `audit_labels.rs:40`), identifiant = l'exercice. **Aucune** ligne d'audit quand
     aucun écart n'est passé (`entryId: null` : rien n'a changé). Libellé d'audit dans les quatre locales. Limite
     connue : l'acteur d'une clé d'API n'apparaît pas à l'audit de la création d'écriture (`NewAuditLogEntry::user`),
     comme pour toute écriture existante ; `fx_revaluation.posted` l'écrit par `NewAuditLogEntry::for_actor`.
   - **Registres** : action `fx_revaluation.posted` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) et sa clé `audit-log-action-fx-revaluation-posted` dans
     les quatre locales ; les deux routes entrent à `LIB_ROUTES` (`tests/audit_route_registry.rs`) — aperçu
     `NoMatter("calcule sans persistance — aucune mutation"), SansEcritureAuJournal` (comme `bank_imports::preview`,
     `:540`), passation `Traced, Rejouee` ; les décomptes de `the_registry_partition_is_what_the_story_declares` et la
     phrase du manuel d'administration (`admin-manual.tex:3217`, gardée par `le_manuel_dit_le_compte_du_registre`)
     suivent, **cumulés** avec ceux des 35-1 et 35-2 (+2 routes, dont +1 tracée et +1 sans mutation : l'assertion
     `no_matter` passe de 3 à 4, et la phrase du manuel change **en chiffres et en toutes lettres** — « N + 4 + 4 = M » et
     « quatre autres routes qui n'écrivent rien », que la garde, qui ne lit que les chiffres, ne contrôle pas). Les décomptes de
     `the_replay_partition_is_what_the_story_declares` (`audit_route_registry.rs:1450`) suivent aussi : l'aperçu
     `SansEcritureAuJournal` fait `sans_ecriture` +1, la passation `Rejouee` fait `rejouees` +1, `tout.len()` +2
     (:1502), en deltas cumulés avec ceux des 35-1 et 35-2 ; la passation appelle une enveloppe
     (`every_replayed_route_calls_an_envelope`). *(Alignement de l'orchestrateur sur 35-1-P4-IF-2.)*
3. **AC-3 — Rejouable.** Un second passage, sans mouvement entre-temps, ne passe rien ; après une écriture tardive en
   devise, il ne passe que le résidu. Pas d'extourne au premier jour de l'exercice suivant. **Le report à nouveau des
   soldes en devise se fait par le cumul** (`opening.rs:17`, Kesh ne passe aucune écriture de clôture) : aucune écriture
   de report n'est créée. Une réévaluation se corrige par **suppression ou contre-passation** de l'écriture, ou par un
   nouveau passage (résidu) ; le `PUT` de la saisie manuelle la refuse (35-3a AC-7 : 409 `ENTRY_IS_FX_REVALUATION` ;
   et 35-3a AC-6 refuse `rateSource: "revaluation"` dans toute requête d'écriture).
4. **AC-4 — Gardes.** Ordre des refus, **en deux temps** (une seule raison, la plus bloquante d'abord ; l'état
   **clos** de l'exercice parle avant les comptes et les cours ; les refus de forme (400) et le cours AFC manquant,
   **avant la transaction**, précèdent `EARLIER_FISCAL_YEAR_OPEN`, qui ne se lit que sous verrou).
   **Avant la transaction** (handler) : exercice inexistant ou d'une autre société → 404 (le handler appelle d'abord
   `fiscal_years::find_by_id_in_company`) ; mono-devise → **409** `MULTI_CURRENCY_DISABLED` (409 partout, décision
   d'epic) ; exercice clos (lecture non verrouillante, refaite dans la transaction) → **400** `FISCAL_YEAR_CLOSED`
   (`DbError::FiscalYearClosed`, `errors.rs:4516` — le code et le statut d'une écriture dans un exercice clos, non le 409
   `ILLEGAL_STATE_TRANSITION` de `close`) ; forme de
   la requête (bornes ci-dessous) → 400 `FX_RATE_INVALID`, ou 400 `FX_REVALUATION_ACCOUNT_INVALID` sur `accountIds` ; cours AFC manquant pour une
   devise sélectionnée sans surcharge → 404 `RATE_UNAVAILABLE`.
   **Dans la transaction**, après tous les verrous de l'AC-2 et **dans le même ordre qu'avant la transaction** :
   réglage relu sous verrou (étape (0)) : mono-devise → 409 `MULTI_CURRENCY_DISABLED` ; exercice clos →
   `FISCAL_YEAR_CLOSED` ; **exercice antérieur encore ouvert → `EARLIER_FISCAL_YEAR_OPEN`** (comme K1 : le cumul lu ne
   serait plus stable) ; comptes de différences de change **revalidés** (société, actif,
   imputable, de type `Expense`/`Revenue`, en CHF, ni au rôle `DefaultRevenue`, ni désigné
   `default_revenue_account_id` des réglages de facturation ou d'un article — 35-1 P3-8) → 400 `FX_ACCOUNT_INVALID` (C-35-2 ; même code et même statut que
   35-1, `details` de 35-1) ; `accountIds` hors de la liste admissible (lue à l'étape (4) de l'AC-2) → 400
   `FX_REVALUATION_ACCOUNT_INVALID`, ensuite ; puis, par `create_in_tx_inner`, exercice
   postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (garde `journal_entries.rs:368`, repository ; état hérité seulement,
   puisque `close` impose l'ordre) ; `DATE_OUTSIDE_FISCAL_YEAR` ; livres verrouillés à cette date
   (`books_locked_through`, seuil **inclusif** : `entry_date <= locked_through`) → `PERIOD_LOCKED`. L'aperçu
   refuse de même (404, mono-devise, exercice clos) et **signale** les autres (`blockers[]`, AC-1). `accountIds` : un seul code, 400 `FX_REVALUATION_ACCOUNT_INVALID`, `details: { reason, accountIds }` (les
   identifiants fautifs **tels que reçus**, rien de lu en base) ; liste vide ou doublons → `reason: "empty"` /
   `"duplicate"`, avant la transaction ; comptes d'une autre société ou hors de la liste admissible (groupe, archivé,
   résultat, sans devise) → `reason: "notEligible"`, dans la transaction (ci-dessus), sans fuite d'existence (même
   réponse pour un compte inexistant et pour un compte d'une autre société) ; libellé dans les quatre locales.
   - **Bornes de la requête** : chaque cours est un décimal > 0, ≤ 10 décimales, ≤ 10⁶, exprimé **par unité** de
     monnaie (1 JPY, pas 100 JPY), et sa clé est la devise d'au moins un compte sélectionné ; sinon 400
     `FX_RATE_INVALID`. Les bornes d'échelle (≤ 10 décimales, zéros terminaux ignorés) et ≤ 10⁶ sont celles du
     cours d'une ligne en devise (35-3a AC-3, étape 1) et se contrôlent par **la même fonction** de `kesh-core` (DRY),
     exposée seule (cours sans montant) ; le signe (> 0) est contrôlé ici (35-3a le contrôle à l'étape 2) ; seul le code
     rendu diffère (`FX_RATE_INVALID`, la devise fautive en `details`). **Le produit** devise_net × cours (devise_net est
     un cumul, non borné par ligne) se calcule en entiers `i128` par multiplication **contrôlée** (`i128::checked_mul` ; un dépassement →
     `FX_VALUE_OUT_OF_RANGE`), jamais par `Decimal::checked_mul` (35-3a AC-3 : réduction d'échelle silencieuse) ; un
     CHF réévalué ou un |D| > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE`, sans écriture. **De même Σ gains et Σ pertes** (et le
     net d'un compte unique) : les lignes de contrepartie agrégent jusqu'à 498 écarts, et 498 × 10¹³ dépasse la colonne
     `debit`/`credit` `DECIMAL(19,4)` (< 10¹⁵, `0001_schema_squash.sql:848-849`) — une erreur SQL, donc un 500, si rien
     ne la borne ; une somme > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE`, sans écriture. Au plus 100 devises et **498** comptes par requête (l'écriture reste sous
     `MAX_LINES_PER_ENTRY` = 500, `routes/journal_entries.rs`, avec les deux lignes de contrepartie).
5. **AC-5 — RBAC.** Les deux routes vont dans `comptable_routes` (`lib.rs:524`, `require_comptable_role`), comme
   `…/close` (`lib.rs:821`) : Consultation reçoit 403 **y compris sur l'aperçu** (c'est un POST). Pas d'`ensure_not_pat`
   (comme `close`, contrairement à `reopen`), sauf décision contraire de l'utilisateur ; une clé en lecture seule est
   refusée sur le POST d'aperçu comme sur tout POST.
6. **AC-6 — Écran** : depuis *Exercices* (exercice ouvert), action « Réévaluer les devises », **affichée seulement en
   multi-devises** (D1 : en mono-devise, rien n'apparaît) **et aux rôles Comptable et Administrateur** (AC-5) : tableau de l'aperçu,
   cours modifiables par devise (source affichée : AFC ou manuel), cases par compte (comptes archivés grisés avec
   avertissement), total des gains et des pertes, « Passer l'écriture », lien vers l'écriture passée ; captures d'écran.
7. **AC-7 — Tests** : deux exercices (achat EUR en N-1, réévaluation N-1, mouvements N, réévaluation N) : solde CHF au
   31.12.N = round(solde EUR × cours) exactement ; exercice **se terminant le 30 juin** → cours de juin ; rejeu sans écriture ; compte
   décoché ou de résultat : aucune ligne ; cours surchargé figé sur la ligne ; refus des gardes. S'y ajoutent :
   - **les cinq cas de signe** : actif USD débiteur, cours en hausse (gain) puis en baisse (perte) ; actif USD à solde
     créditeur, cours en hausse (perte) ; passif USD créditeur, cours en hausse (perte) ; passif USD à solde débiteur, cours en hausse (gain) ;
     devise à 0 avec CHF résiduel ≠ 0 ;
   - compte de groupe, archivé, non imputable ; plusieurs devises (USD et EUR) dans la même écriture ;
   - compte de gains = compte de pertes (une ligne au net ; net nul, aucune ligne de contrepartie) ;
   - cours invalide (0, négatif, 11 décimales, > 10⁶, devise sans compte) ; au-delà des bornes ;
   - `PERIOD_LOCKED` exactement à la borne (inclusive) ; exercice postérieur clos (test de dépôt, comme
     `opening_complement_repository.rs:963`) ; exercice antérieur ouvert ; exercice clos ; mono-devise ;
   - `FX_ACCOUNT_INVALID` (compte devenu archivé, d'une autre société, en devise, ou compte de gains désigné produit
     par défaut) — états que 35-1 interdit par
     l'API (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) : posés **par SQL** dans le test, comme un état hérité ;
   - l'ordre des refus de l'AC-4 (exercice clos **et** cours manquant → `FISCAL_YEAR_CLOSED`) ; source `afc` d'un
     cours prérempli renvoyé tel quel, `manual` d'un cours modifié ; aucun `fx_revaluation.posted` sans écriture ;
     `blockers[]` de l'aperçu ;
   - RBAC (Consultation 403) et société (exercice d'une autre société 404) ;
   - audit : `journal_entry.created` avec les quatre colonnes de devise au snapshot, et `fx_revaluation.posted` ;
   - le résidu après écriture tardive ; une réévaluation supprimée ou contre-passée, puis un nouveau passage (le résidu
     rétablit l'écart) ; 498 comptes acceptés, 499 refusés, 101 devises refusées, libellé ≤ 500 caractères ; produit
     devise × cours hors bornes ; Σ des gains (ou des pertes) > 10¹³ → `FX_VALUE_OUT_OF_RANGE` ; `accountIds` vide, en
     doublon, étranger à la société et inexistant → `FX_REVALUATION_ACCOUNT_INVALID`, réponses identiques pour les deux
     derniers ; l'audit `fx_revaluation.posted` d'une passation par clé d'API porte la clé (`for_actor`) ; exercice antérieur ouvert **et** cours manquant → `RATE_UNAVAILABLE` (ordre en deux
     temps) ; deux passations concurrentes : l'une passe, l'autre ne passe rien ;
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
      `fx_revaluation.posted`). Soldes lus par `kesh_db::repositories::account_balances::balances_as_of(&mut conn,
      company_id, fin_d_exercice, Some(ids))` (35-4 AC-1 quater : dans `kesh-db`, car `kesh-db` ne peut pas dépendre de
      `kesh-report` ; borne `<=` incluse ; débit net brut ; aucune conversion de signe).
- [ ] `kesh-api` : routes (aperçu, passation) dans `comptable_routes`, gardes, résolution des cours avant la
      transaction, i18n des erreurs neuves (`FX_RATE_INVALID`, `FX_REVALUATION_ACCOUNT_INVALID` ; `FX_ACCOUNT_INVALID` et
      `RATE_UNAVAILABLE` viennent de 35-1 et 35-2, réemployés) et du libellé d'audit
      (quatre locales) ; registres de l'AC-2 (`ACTIONS`, `LIB_ROUTES`, décomptes, phrase du manuel d'administration).
- [ ] Frontend : écran depuis les exercices, Vitest, E2E.
- [ ] Manuel, CHANGELOG.

## Notes de développement

- Kesh ne passe aucune écriture de clôture (`crates/kesh-report/src/opening.rs:17`) : les soldes de bilan sont
  cumulés depuis l'origine ; d'où le calcul « jusqu'à la date », toutes années confondues (risque relevé à la table
  ronde). Les fonctions actuelles de `kesh-report` prennent `&MySqlPool` (`balance_sheet.rs`,
  `fetch_cumulative_section`) : elles ne conviennent pas telles quelles sous le verrou ;
  `fetch_cumulative_section` reste dans `kesh-report`, la réévaluation n'en dépend pas.
- Exercices : `crates/kesh-db/src/repositories/fiscal_years.rs` (`close` :1116), `routes/fiscal_years.rs` (:352) ;
  garde « exercice postérieur clos » `crates/kesh-db/src/repositories/journal_entries.rs:368` (étape 1-a de
  `create_in_tx_inner`, lecture non verrouillante) ; `validate_lines_accounts_in_tx` (:135).
- Les textes de verrouillage de `close` sont **privés** à `fiscal_years.rs` (`LOCK_EARLIER_BY_ID_SQL` :178,
  `LOCK_IN_COMPANY_SQL` :162, `earlier_open_locking_sql` :201) : les rendre `pub(crate)`, ou mieux extraire dans
  `fiscal_years` une fonction `*_in_tx` des étapes (b') à (d) partagée par `close` et `fx_revaluation::post` (DRY ;
  un texte copié divergerait). Le commentaire de `LOCK_IN_COMPANY_SQL` (:155-160) rappelle que des tests de 15-12a
  reconnaissent l'étape (c) à ce texte : le réemployer ne change pas leurs motifs.
- `FiscalYear` n'a pas de `version` ; la lecture des cours (35-2) est sans réseau ; la récupération est un geste séparé de l'écran.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 7 MEDIUM, 5 LOW ; tous remédiés dans la fiche ; P1-1 (côté
  35-3a) et l'exigence 35-4 de P1-2 portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 8 LOW ; remédiés dans la fiche sauf 35-5-P2-IF1
  (MEDIUM, 35-3a), 35-5-P2-IF2 (LOW, 35-3a/35-3b) et 35-5-P2-IF3 (LOW, 35-2), inter-fiches.
- 2026-11-14 — validation P3 (Sonnet) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf 35-5-P3-IF1
  (LOW, 35-3a), inter-fiches.
- 2026-11-14 — validation P4 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW ; remédiés dans la fiche sauf 35-5-P4-IF1
  (LOW, 35-2 : la lecture du cours par 35-5 se fait avant la transaction, non « sous ses verrous »), inter-fiches.
  Choix de cette passe, à confirmer : `companies` verrouillée **avant** les exercices (P4-1, ordre de 35-3a et de
  Pattern 5) ; refus en transaction alignés sur l'ordre d'avant la transaction, mono-devise d'abord (P4-5) ; code neuf
  `FX_REVALUATION_ACCOUNT_INVALID` pour `accountIds` (P4-4).
