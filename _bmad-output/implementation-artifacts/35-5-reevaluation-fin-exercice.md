# Story 35-5 : réévaluation des comptes en devise à la fin de l'exercice

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, D5 corrigé par D10, D9, C-35-7
(amendé, CR-1). Suit 35-4.
**Aucune migration attendue** (`rate_source = 'revaluation'` posé par 35-3a).*

## Story

En tant que comptable en multi-devises, je veux qu'à la fin de l'exercice Kesh me propose la réévaluation de mes
comptes de bilan en devise au cours journalier OFDF de la date de clôture (à défaut, au cours mensuel AFC du mois de
clôture, signalé), et la passe en une écriture après mon contrôle, afin
que mon bilan présente ces comptes au cours de clôture et que l'écart aille aux différences de change.

## Critères d'acceptation

1. **AC-1 — Aperçu.** `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview { rates?: {USD: "0.8846", …},
   accountIds?: [...] }` → par compte de **bilan** (`Asset`, `Liability`) en devise, **imputable** (`postable = TRUE` ;
   les comptes de groupe ne sont jamais proposés) : solde en devise et solde CHF **cumulés depuis l'origine** jusqu'à
   la date de fin d'exercice (toutes années confondues, sans borne basse), cours retenu, CHF réévalué = round(solde
   devise × cours), écart, compte de contrepartie.
   - **Cours par défaut** (D10, D9), par devise, résolu par la lecture du cours effectif de 35-2
     (`find_effective_rate_in_tx`, AC-5 de 35-2 — cours saisi par la société, sinon cours AFC brut) : (1) le cours
     effectif **journalier** de la **date de fin d'exercice** ; (2) à défaut, le cours effectif **mensuel** du mois de
     cette date ; (3) à défaut, `RATE_UNAVAILABLE`. Surchargeable par devise dans la requête (`rates`), comme avant.
     La réponse porte, par devise, `{ currency, rate, rateKind: "day"|"month"|"request", rateDate, afcRate, overridden,
     sourceDate }` (`afcRate` = le cours AFC brut du genre de la **résolution par défaut**, `null` s'il n'y en a pas ;
     `sourceDate` = date de publication d'un cours journalier, cf. 35-2 ; `rateDate` et `afcRate` sont toujours ceux de
     la résolution par défaut de la devise — jour, sinon mois —, que l'écran affiche sous le champ comme ailleurs, D9).
     **Genre d'un cours reçu dans `rates`** — règle **unique**, la même à l'aperçu et à la passation (AC-2, audit) : égal
     au cours par défaut résolu par le serveur → genre de ce défaut (`day` ou `month`) ; différent, ou sans défaut →
     `request` (comparaison **numérique**, zéros terminaux ignorés ; `overridden` vaut `true` si et seulement si le genre est `request`). Sans cette règle à l'aperçu, un cours prérempli renvoyé ferait disparaître le signalement du repli et
     masquerait le cours du jour récupéré. **L'écran n'envoie à l'aperçu que les cours modifiés par l'utilisateur** ;
     après une récupération, un champ non modifié reprend le nouveau cours par défaut (la passation, elle, reçoit tous
     les cours affichés, AC-2). L'écran **signale** un
     `rateKind: "month"` (« cours du jour de clôture indisponible : cours mensuel de <mois> ») et propose « Récupérer
     le cours du JJ.MM.AAAA » (`POST /exchange-rates/fetch { date }`, 35-2), puis relit l'aperçu ; un
     `404 RATE_NOT_PUBLISHED` ou un `503 RATE_SOURCE_UNAVAILABLE` s'affiche (message traduit) et laisse l'aperçu tel
     quel, repli mensuel compris. ⚠️ **Date de fin d'exercice
     non passée** : 35-2 ne récupère un cours journalier que pour un jour **strictement antérieur** à aujourd'hui
     (Europe/Zurich ; sinon 400) — le bouton n'est alors **pas proposé** et, si le repli mensuel est pris ou qu'aucun
     cours n'existe, l'écran dit « date de clôture pas encore passée : cours du jour non publié » ; le cours par défaut
     suit la même résolution (un cours journalier **saisi** par la société pour ce jour, D9, reste le premier ; sinon
     le cours effectif mensuel du mois de cette date, qui pour un mois futur n'existe que s'il a été saisi), sinon
     `RATE_UNAVAILABLE` et saisie ; « Récupérer le mois » n'est pas proposé non plus pour un mois postérieur au mois
     courant (35-2 le refuse en 400). Le serveur ne lit pas la date du jour : seul l'écran en dépend. Le cours journalier
     d'un jour sans publication (week-end) est celui de la publication antérieure (35-2, *Cours journaliers*) : un
     exercice qui finit un dimanche prend le cours du vendredi, `sourceDate` l'affiche.
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
   - Comptes présélectionnés : tous ceux de bilan en devise, actifs et imputables, **dans la limite de 498** (AC-4) : au-delà,
     l'aperçu présélectionne les 498 premiers par numéro de compte et le signale par un champ de la réponse (nom au
     développement, testé à l'AC-7), que l'écran affiche (la passation en refuserait davantage, AC-4). Un compte **archivé** à solde ≠ 0 est
     listé avec `reason: ACCOUNT_ARCHIVED`, non sélectionnable, avec avertissement (sinon l'écriture entière serait
     refusée en `InactiveOrInvalidAccounts`, `validate_lines_accounts_in_tx`). Comptes `Revenue`/`Expense` jamais
     proposés.
   - Aucun cours (ni journalier ni mensuel, ni saisi ni AFC) sans surcharge → la devise est signalée
     `RATE_UNAVAILABLE` (`details` de 35-2, forme `month` : le mois de la date de fin), ses comptes exclus jusqu'à
     saisie ou récupération.
   - **Cours résolus avant toute transaction**, par lecture de la table des cours (35-2 : `lookup` est en lecture
     seule, sans appel sortant). Un cours absent rend la devise `RATE_UNAVAILABLE` dans l'aperçu ; l'écran propose
     « Récupérer le cours du JJ.MM.AAAA », puis « Récupérer le mois » (`POST /exchange-rates/fetch { date }` puis
     `{ year, month }`, 35-2, audités à part) ou la saisie du cours. Le serveur de 35-5
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
     `create_in_tx_inner`. Une `rates` qui omet une devise sélectionnée : cours par défaut de l'AC-1 (effectif
     journalier, sinon effectif mensuel) lu **avant** la transaction, `RATE_UNAVAILABLE` (404) s'il manque. Les devises des comptes sélectionnés se lisent
     avant la transaction par une lecture ordinaire sur le pool (elles ne servent qu'à choisir les cours à résoudre et à
     contrôler les clés de `rates`) ; **dans** la transaction, après l'étape (4), toute devise d'un compte admissible
     sélectionné qui n'a pas de cours résolu (devise du compte changée entre-temps) → rollback, 404 `RATE_UNAVAILABLE`
     (`details` de 35-2) : un cours n'est jamais supposé. Ce refus se place, dans l'ordre de l'AC-4, juste après
     `FX_REVALUATION_ACCOUNT_INVALID` (un compte devenu CHF est d'abord `notEligible`).
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
     puis `companies` **en exclusif** : ni `close`, ni `reopen`, ni `lock_books`, `repositories/companies.rs:1002-1020` ; la
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
   - **Source du cours** (pour l'audit) : par devise, `afc` si le cours retenu **égale** l'`afcRate` de l'AC-1 — le
     cours AFC **brut** du genre de la résolution par défaut : du jour de fin d'exercice si un cours effectif
     journalier existe, sinon du mois de fin —, qu'il vienne de la table ou de `rates` (l'écran renvoie les cours
     préremplis) ; `manual` sinon — y compris un cours saisi par la société (D9) qui diffère, un cours journalier saisi
     sans cours AFC du jour (`afcRate` `null` : l'écran affiche « AFC : non publié », l'audit dit `manual`), et une
     devise non publiée. L'audit porte aussi le **genre** du cours retenu par devise (`day` | `month` | `request`),
     selon la règle unique de l'AC-1 (cours reçu égal au défaut : genre de ce défaut ; sinon `request`) : comme l'écran
     renvoie les cours préremplis, le genre serait sans cela toujours `request` et ne dirait rien. Ces lectures (cours effectif par défaut et cours AFC bruts
     du jour et du mois de fin) se font **avant la transaction**, avec celles des cours (35-2 : aucune lecture de cours
     sous le verrou de l'exercice), et non à la volée sous les verrous.
   - Une **seule** écriture journal `OD`, datée du dernier jour de l'exercice, libellé **en français** construit dans `post`, comme la
     contre-passation (`crates/kesh-db/src/repositories/journal_entries.rs:2975`) : « Réévaluation des comptes en devise au JJ.MM.AAAA (écarts non
     réalisés) » (longueur fixe, très inférieure aux 500 caractères de `MAX_DESCRIPTION_LEN`, privé à `routes/journal_entries.rs` : un test sur
     le libellé produit suffit, aucune constante à exporter).
   - Une ligne par compte d'écart non nul : `ForeignAmount { currency, source: Revaluation, amount: 0, rate }` (`currency` =
     devise **du compte**, que le contrôle central de 35-3a, `FX_CURRENCY_MISMATCH`, compare à celle du compte), `rate_source =
     'revaluation'`, `amount_currency = 0`, `exchange_rate` = cours retenu (> 0, par unité), CHF = |D|. **Cette ligne
     est exemptée de l'égalité CHF = montant × cours et de `FX_LINE_ROUNDS_TO_ZERO`** (le CHF est pris tel quel, > 0) ;
     la saisie manuelle de 35-3a (AC-6) la refuse, c'est la route de 35-5 qui la construit.
   - Lignes de contrepartie CHF : une par compte de contrepartie et par sens, agrégée. **Si le compte de gains et le
     compte de pertes sont le même, une seule ligne au net** (|Σ gains − Σ pertes|, du côté du solde) ; **aucune ligne
     si le net est nul** (les lignes des comptes réévalués s'équilibrent alors entre elles) ; à tester.
   - Aucun écart non nul → 200 sans écriture (`{ entryId: null }`).
   - **Audit** : outre `journal_entry.created`, une ligne d'audit propre `fx_revaluation.posted`
     `{ entryId, rates, source par devise (afc|manual), genre par devise (day|month|request), accountIds }` écrite **dans la même transaction** (la source
     du cours, perdue sur la ligne puisque `rate_source = 'revaluation'`, y est conservée), entité `fiscal_year`
     (déjà au registre des types, `audit_labels.rs:40`), identifiant = l'exercice. **Aucune** ligne d'audit quand
     aucun écart n'est passé (`entryId: null` : rien n'a changé). Libellé d'audit dans les quatre locales. Limite
     connue : l'acteur d'une clé d'API n'apparaît pas à l'audit de la création d'écriture (`NewAuditLogEntry::user`),
     comme pour toute écriture existante ; `fx_revaluation.posted` l'écrit par `NewAuditLogEntry::for_actor`.
   - **Registres** : action `fx_revaluation.posted` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) et sa clé `audit-log-action-fx-revaluation-posted` dans
     les quatre locales ; les deux routes entrent à `LIB_ROUTES` (`tests/audit_route_registry.rs`) — aperçu
     `NoMatter("calcule sans persistance — aucune mutation"), SansEcritureAuJournal` (comme `bank_imports::preview`,
     `:540`), passation `Traced, Rejouee` ; les décomptes de `the_registry_partition_is_what_the_story_declares` (+2 routes, **total du registre +2**) et la
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
   **clos** de l'exercice parle avant les comptes et les cours ; les refus de forme (400) et le cours par défaut manquant (AC-1),
   **avant la transaction**, précèdent `EARLIER_FISCAL_YEAR_OPEN`, qui ne se lit que sous verrou).
   **Avant la transaction** (handler) : exercice inexistant ou d'une autre société → 404 (le handler appelle d'abord
   `fiscal_years::find_by_id_in_company`) ; mono-devise → **409** `MULTI_CURRENCY_DISABLED` (409 partout, décision
   d'epic) ; exercice clos (lecture non verrouillante, refaite dans la transaction) → **400** `FISCAL_YEAR_CLOSED`
   (`DbError::FiscalYearClosed`, `errors.rs:4516` — le code et le statut d'une écriture dans un exercice clos, non le 409
   `ILLEGAL_STATE_TRANSITION` de `close`) ; forme de
   la requête (bornes ci-dessous) → 400 `FX_RATE_INVALID`, ou 400 `FX_REVALUATION_ACCOUNT_INVALID` sur `accountIds` ; aucun cours par
   défaut (AC-1 : ni journalier ni mensuel) pour une devise sélectionnée sans surcharge → 404 `RATE_UNAVAILABLE`.
   **Dans la transaction**, après tous les verrous de l'AC-2 et **dans le même ordre qu'avant la transaction** (sauf le
   `RATE_UNAVAILABLE` d'une devise changée entre-temps, qui ne peut naître qu'ici et vient après les comptes, ci-dessous) :
   réglage relu sous verrou (étape (0)) : mono-devise → 409 `MULTI_CURRENCY_DISABLED` ; exercice clos →
   `FISCAL_YEAR_CLOSED` ; **exercice antérieur encore ouvert → `EARLIER_FISCAL_YEAR_OPEN`** (comme K1 : le cumul lu ne
   serait plus stable) ; comptes de différences de change **revalidés** (société, actif,
   imputable, de type `Expense`/`Revenue`, en CHF, ni au rôle `DefaultRevenue`, ni désigné
   `default_revenue_account_id` des réglages de facturation ou d'un article — 35-1 P3-8) → 400 `FX_ACCOUNT_INVALID` (C-35-2 ; même code et même statut que
   35-1, `details` de 35-1) ; `accountIds` hors de la liste admissible (lue à l'étape (4) de l'AC-2) → 400
   `FX_REVALUATION_ACCOUNT_INVALID`, ensuite ; puis 404 `RATE_UNAVAILABLE` d'une devise sans cours résolu (AC-2) ; puis, par `create_in_tx_inner`, exercice
   postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (garde `journal_entries.rs:368`, repository ; état hérité seulement,
   puisque `close` impose l'ordre) ; `DATE_OUTSIDE_FISCAL_YEAR` ; livres verrouillés à cette date
   (`books_locked_through`, seuil **inclusif** : `entry_date <= locked_through`) → `PERIOD_LOCKED`. L'aperçu
   refuse de même (404, mono-devise, exercice clos, formes de la requête : 400 `FX_RATE_INVALID` /
   `FX_REVALUATION_ACCOUNT_INVALID`, `FX_VALUE_OUT_OF_RANGE` ; un `accountIds` fourni hors de la liste admissible y est
   refusé de même, `reason: "notEligible"`) et **signale** les autres (`blockers[]`, AC-1). `accountIds` : un seul code, 400 `FX_REVALUATION_ACCOUNT_INVALID`, `details: { reason, accountIds }` (les
   identifiants fautifs **tels que reçus**, rien de lu en base) ; liste vide ou doublons → `reason: "empty"` /
   `"duplicate"`, plus de 498 identifiants → `reason: "tooMany"`, avant la transaction ; comptes d'une autre société ou hors de la liste admissible (groupe, archivé,
   résultat, sans devise) → `reason: "notEligible"`, dans la transaction (ci-dessus), sans fuite d'existence (même
   réponse pour un compte inexistant et pour un compte d'une autre société) ; libellé dans les quatre locales.
   - **Bornes de la requête** : chaque cours est un décimal > 0, ≤ 10 décimales, ≤ 10⁶, exprimé **par unité** de
     monnaie (1 JPY, pas 100 JPY), et sa clé est la devise d'au moins un compte sélectionné ; sinon 400
     `FX_RATE_INVALID`. Ces bornes (signe, échelle — zéros terminaux ignorés —, ≤ 10⁶) sont celles du cours d'une ligne
     en devise (35-3a AC-3) et d'un cours saisi (35-2 AC-5) et se contrôlent par **la même fonction** de `kesh-core`
     (DRY), **créée par 35-2** (AC-2, bornes d'un cours seul), qui rend une erreur de **type propre** (p. ex.
     `RateBoundsError { Scale, TooLarge, NotPositive }`, non une variante de `CoreError` liée à un code — arbitrage de
     l'orchestrateur, CR-1 P2) ; 35-5 traduit **toute** variante en 400 `FX_RATE_INVALID`, `details: { currency }`
     (la devise fautive), code **créé par 35-2** (variante `AppError` et traduction) et réutilisé ici. **Le produit** devise_net × cours (devise_net est
     un cumul, non borné par ligne) se calcule en entiers `i128` par multiplication **contrôlée** (`i128::checked_mul` ; un dépassement →
     `FX_VALUE_OUT_OF_RANGE`), jamais par `Decimal::checked_mul` (35-3a AC-3 : réduction d'échelle silencieuse) ; un
     CHF réévalué ou un |D| > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE`, sans écriture. **Forme du refus** : la variante de 35-3a
     (`FxValueOutOfRange { line }`) porte un rang de ligne que 35-5 n'a pas ; 35-5 rend le même code et le même statut par
     sa **propre variante** de `DbError` (`FxRevaluationValueOutOfRange`, `kesh-db` ; bras dans
     `kesh-api/src/errors.rs`), que l'aperçu et la passation rendent tous deux (le calcul pur de `kesh-core` signale le
     dépassement, le code de `kesh-db` qui l'appelle le traduit), `details: { currency, accountId }` pour un compte
     fautif, `details: {}` pour une somme de contrepartie (elle mêle les devises : ni compte ni devise), sans toucher
     `map_core_error`. **De même Σ gains et Σ pertes** (et le
     net d'un compte unique) : les lignes de contrepartie agrégent jusqu'à 498 écarts, et 498 × 10¹³ dépasse la colonne
     `debit`/`credit` `DECIMAL(19,4)` (< 10¹⁵, `crates/kesh-db/test-schema/0001_schema_squash.sql:848-849`) — une erreur SQL, donc un 500, si rien
     ne la borne ; une somme > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE`, sans écriture. Au plus 100 devises et **498** comptes par requête (l'écriture reste sous
     `MAX_LINES_PER_ENTRY` = 500, `routes/journal_entries.rs`, avec les deux lignes de contrepartie) : plus de 498
     `accountIds` → 400 `FX_REVALUATION_ACCOUNT_INVALID`, `reason: "tooMany"` ; plus de 100 clés dans `rates` → 400
     `FX_RATE_INVALID`, `details: { reason: "tooMany" }` (sans devise fautive).
5. **AC-5 — RBAC.** Les deux routes vont dans `comptable_routes` (`lib.rs:524`, `require_comptable_role`), comme
   `…/close` (`lib.rs:821`) : Consultation reçoit 403 **y compris sur l'aperçu** (c'est un POST). Pas d'`ensure_not_pat`
   (comme `close`, contrairement à `reopen`), sauf décision contraire de l'utilisateur ; une clé en lecture seule est
   refusée sur le POST d'aperçu comme sur tout POST.
6. **AC-6 — Écran** : depuis *Exercices* (exercice ouvert), action « Réévaluer les devises », **affichée seulement en
   multi-devises** (D1 : en mono-devise, rien n'apparaît) **et aux rôles Comptable et Administrateur** (AC-5) : tableau de l'aperçu,
   cours modifiables par devise (**valable pour cette passation seulement** : le champ n'écrit aucun cours saisi, qui
   se fixe à l'écran *Cours de change*, 35-2 ; « AFC : … » affiché sous le champ, sans ↑ ; source affichée : AFC ou
   manuel ; **genre** affiché : « cours du JJ.MM.AAAA » pour
   un cours journalier, avec la date de publication si elle diffère, « cours mensuel de <mois> » signalé comme repli,
   « cours saisi » quand le cours vient de la société, D9 ; bouton « Récupérer le cours du JJ.MM.AAAA » quand le repli
   mensuel est pris ou qu'aucun cours n'existe, et que la date de fin est passée (AC-1) — Administrateur et
   Comptable), cases par compte (comptes archivés grisés avec
   avertissement ; au-delà de 498 comptes admissibles, l'avis de présélection tronquée de l'AC-1, et pas plus de 498 cases
   cochées), total des gains et des pertes, « Passer l'écriture », lien vers l'écriture passée ; captures d'écran.
7. **AC-7 — Tests** : deux exercices (achat EUR en N-1, réévaluation N-1, mouvements N, réévaluation N) : solde CHF au
   31.12.N = round(solde EUR × cours) exactement ; cours par défaut = cours **journalier** de la date de fin
   (`rateKind: "day"`), servi par la source factice de 35-2 et **distinct** du cours mensuel ; sans cours journalier,
   cours mensuel du mois de fin, `rateKind: "month"` ; ni l'un ni l'autre → `RATE_UNAVAILABLE` ; un cours saisi par
   la société (D9) pour le jour l'emporte sur le cours AFC du jour, un cours saisi pour le mois ne l'emporte **pas**
   sur un cours journalier présent (l'ordre est jour puis mois, chacun effectif) ; source `afc` d'un cours journalier
   AFC, `manual` d'un cours saisi qui diffère ; exercice **se terminant le 30 juin** (un dimanche en 2024) → cours
   journalier du 30.06 = publication du 28.06 (`sourceDate`) ; rejeu sans écriture ; compte
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
     rétablit l'écart) ; 498 comptes acceptés, 499 refusés (`reason: "tooMany"`), 101 devises refusées ;
     499 comptes admissibles : l'aperçu en présélectionne 498 et le signale ; libellé ≤ 500 caractères ; produit
     devise × cours hors bornes ; Σ des gains (ou des pertes) > 10¹³ → `FX_VALUE_OUT_OF_RANGE` ; `accountIds` vide, en
     doublon, étranger à la société et inexistant → `FX_REVALUATION_ACCOUNT_INVALID`, réponses identiques pour les deux
     derniers ; l'audit `fx_revaluation.posted` d'une passation par clé d'API porte la clé (`for_actor`) ; exercice antérieur ouvert **et** cours manquant → `RATE_UNAVAILABLE` (ordre en deux
     temps) ; deux passations concurrentes : l'une passe, l'autre ne passe rien ;
   - genre, à l'aperçu **et** à l'audit : cours reçu égal au défaut journalier → `day`, égal au défaut mensuel →
     `month`, différent → `request` ; source `manual` d'un cours journalier saisi sans cours AFC du jour, même égal au
     cours AFC mensuel ; cours de `rates` → `FX_RATE_INVALID` pour chaque variante de l'erreur des bornes de 35-2 ;
   - fin d'exercice non passée (date ≥ aujourd'hui, Europe/Zurich) : Vitest à horloge fixée — pas de bouton du jour
     ni « Récupérer le mois » pour un mois futur, message affiché ; côté serveur, un cours journalier saisi pour ce
     jour est retenu (`day`), sinon repli mensuel ou `RATE_UNAVAILABLE` ;
   - l'aperçu ne crée aucune écriture ; E2E de l'écran, dont le parcours repli mensuel → « Récupérer le cours du
     JJ.MM.AAAA » (source factice de 35-2) → aperçu relu, cours journalier (`rateKind: "day"`, le champ non modifié
     ayant repris le défaut) ; Vitest : l'aperçu ne reçoit que les cours modifiés, un `404 RATE_NOT_PUBLISHED` du
     `POST /fetch` laisse le repli affiché.
8. **AC-8 — Manuel** : réévaluation, cours journalier OFDF de la date de clôture (D10 : la pratique du cours de
   clôture, et les cours que portaient les livres de Beat), repli sur le cours mensuel AFC du mois de clôture (fixé
   d'avance, non une moyenne), cours saisi par la société (D9), récupération du cours du jour, gains et pertes non réalisés
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
      transaction (cours effectif journalier puis mensuel, 35-2), i18n des erreurs neuves (`FX_REVALUATION_ACCOUNT_INVALID` ;
      `FX_ACCOUNT_INVALID`, `RATE_UNAVAILABLE` et `FX_RATE_INVALID` viennent de 35-1 et 35-2, réemployés) et du libellé d'audit
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
- Cours de la recette (fichier d'epic, D10) : USD 0.91193 et EUR 0.94947 au 31.12.2024, cours journaliers ; les cours
  mensuels de décembre 2024 (USD 0.8846, EUR 0.9470) sont le repli. Aucun cours officiel n'est versionné (fixtures
  synthétiques, 35-2).

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
- 2026-11-14 — validation P5 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW ; remédiés dans la fiche ; aucun finding
  inter-fiches. (MEDIUM : variante propre de `FX_VALUE_OUT_OF_RANGE` sans rang de ligne ; devises des comptes lues avant
  la transaction et revérifiées dedans. LOW : `currency` de `ForeignAmount`, chemins cités, plafond de 498 à l'aperçu,
  formes de la requête à l'aperçu, total du registre +2, libellé et `MAX_DESCRIPTION_LEN`.)
- 2026-11-14 — validation P6 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : code et `reason` des plafonds 498 comptes / 100 devises, sur lesquels s'appuie la
  présélection ajoutée en P5 — `reason: "tooMany"`. LOW : `details` d'une somme de contrepartie sans devise, variante
  `DbError` rendue aussi par l'aperçu ; place du `RATE_UNAVAILABLE` en transaction dans l'ordre de l'AC-4 ; champ de
  réponse de la présélection tronquée et son test ; `notEligible` à l'aperçu.)
- 2026-11-14 — validation P7 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (LOW : « même ordre qu'avant la transaction » contredit par le `RATE_UNAVAILABLE` en transaction
  placé après les comptes ; avis de présélection tronquée et plafond de 498 cases absents de l'écran, AC-6.)
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : cours par défaut = cours **effectif journalier** de la
  date de fin d'exercice (D10), sinon cours effectif mensuel du mois de fin, signalé (`rateKind: day|month|request`
  dans l'aperçu), sinon `RATE_UNAVAILABLE` ; récupération du jour par `POST /exchange-rates/fetch { date }` (35-2) ;
  source `afc` comparée au cours AFC brut du genre retenu ; genre conservé à l'audit `fx_revaluation.posted` ;
  `FX_RATE_INVALID` et la fonction des bornes d'un cours seul désormais créés par 35-2 ; écran et manuel (genre du
  cours, repli, cours saisi) ; tests (jour, repli, cours saisi, exercice finissant un dimanche). AC touchés : en-tête,
  Story, AC-1, AC-2, AC-4, AC-6, AC-7, AC-8, Tâches, Notes.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : fin d'exercice non passée, où 35-2 refuse le cours journalier, et le bouton du jour
  proposé à tort ; genre d'audit toujours `request` puisque l'écran renvoie les cours préremplis. LOW : `rateDate` et
  `afcRate` d'un cours `request` ; lectures des cours AFC bruts avant la transaction ; champ de cours de l'écran
  par passation, sans cours saisi ; tests du genre, de la date non passée et du parcours E2E de récupération.)
- 2026-11-14 — validation CR-1 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : genre `request` à l'aperçu pour tout cours de `rates`, alors que l'écran renvoie
  les cours préremplis — repli et cours du jour récupéré masqués : règle unique de genre à l'aperçu et à l'audit,
  l'aperçu ne reçoit que les cours modifiés ; arbitrage de l'orchestrateur sur la fonction des bornes de 35-2 —
  erreur de type propre, signe compris, toute variante → `FX_RATE_INVALID`. LOW : source `afc` alignée sur
  l'`afcRate` affiché ; date de fin non passée — cours journalier saisi, mois futur, « Récupérer le mois » ;
  libellé du bouton et échec de la récupération ; tests Vitest à horloge fixée. Vérifié à la source le 2026-11-14 :
  `xmldaily?d=20251231` rend le 31.12.2025, `d=20251226` (férié) le 24.12.2025.)
- 2026-11-14 — validation CR-1 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche ; aucun
  finding inter-fiches. (LOW : sens de `overridden` et mode de comparaison du cours reçu au défaut non écrits avec la
  règle unique de genre. Vérifié aux fiches : `RateBoundsError`, `FX_RATE_INVALID` `details: { currency }`, fetch jour
  strictement antérieur / mois ≤ courant, `RATE_NOT_PUBLISHED`, `RATE_SOURCE_UNAVAILABLE` — cohérents avec 35-2 et 35-3a.
  Axes non exercés : relecture hors diff, code réel non encore écrit.)
