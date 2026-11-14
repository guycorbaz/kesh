# Story 37-5 : réévaluation des comptes en devise à la fin de l'exercice

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/37-multi-devises`. Epic 37, D5 corrigé par D10, D9, C-37-7
(amendé, CR-1). Suit 37-4.
**Aucune migration attendue** (`rate_source = 'revaluation'` posé par 37-3a ; `closing_rate_kind` posé par 37-1).
Révisée le 2026-11-14 pour la revue de Beat (CR-2 : R3, R6, R9), qui prime sur le registre antérieur : **une écriture
par compte réévalué** (R9), cours de clôture selon le genre choisi pour l'exercice (R6), refus en exercice
mono-devise (R3). Révisée de nouveau pour la revue CR-3 de Beat, qui prime sur CR-2 : **S2** — la contrepartie de
chaque écriture prend les comptes de différence **propres** du compte réévalué s'il en a (37-1 AC-4 bis) ; **S4** —
exercice de durée quelconque (aucune constante de douze mois : le mois et le jour de clôture sont ceux de `end_date`).
Révisée enfin pour la revue 2 de Beat (CR-4) : **T3** — la ligne ne porte aucune colonne de devise, la devise est celle
du compte ; **T2** — exercice au 1er janvier ajouté aux tests. Révisée en dernier pour la revue 3 de Beat (CR-5), qui
prime sur tout : **U1** — le cours de clôture se lit dans `exchange_rates` de 37-2 (le **cours retenu par la
société**, une seule valeur), et sa **source** (valeur brute : `afc`, `manual` ou autre, A30) est affichée et auditée ; une correction de l'AFC non
reprise n'y change rien (AC-1, AC-2, AC-6, AC-7).*

## Story

En tant que comptable en multi-devises, je veux qu'à la fin de l'exercice Kesh me propose la réévaluation de mes
comptes de bilan en devise au cours de clôture choisi pour l'exercice — cours journalier OFDF de la date de clôture
(à défaut, cours mensuel AFC du dernier mois, signalé) ou cours mensuel AFC du dernier mois (R6) —, et la passe, après
mon contrôle, en **une écriture par compte réévalué** (R9), afin que mon bilan présente ces comptes au cours de
clôture et que l'écart aille aux différences de change.

## Critères d'acceptation

1. **AC-1 — Aperçu.** `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview { rates?: {USD: "0.8846", …},
   accountIds?: [...] }` → par compte de **bilan** (`Asset`, `Liability`) en devise, **imputable** (`postable = TRUE` ;
   les comptes de groupe ne sont jamais proposés) : **devise du compte** (`currency`, lue sur le compte, T3 : c'est elle qui
   désigne le cours de la ligne et les devises cochées de l'AC-1), solde en devise et solde CHF **cumulés depuis l'origine** jusqu'à
   la date de fin d'exercice (toutes années confondues, sans borne basse), cours retenu, CHF réévalué = round(solde
   devise × cours), écart, compte de contrepartie — **par compte** (S2, CR-3) : gains ou pertes résolus par
   `fx_accounts_for(société, compte)` (37-1 AC-4 bis), soit les comptes **propres** du compte réévalué, soit ceux de
   la société ; l'aperçu porte par compte `counterpartAccountId` et `fxAccountsOrigin` (`own` | `company`), que l'écran
   affiche. **Durée de l'exercice** (S4, CR-3) : quelconque — « dernier mois » et « date de fin » sont toujours ceux
   de `end_date` (un exercice de six ou de dix-huit mois, ou qui finit un 14, n'a rien de particulier ; aucun calcul
   ne suppose douze mois).
   - **Cours par défaut** (D10, D9, **R6**, U1), par devise, résolu par la lecture du cours effectif de 37-2
     (`find_company_rate_in_tx`, AC-5 de 37-2 — le **cours retenu par la société** dans `exchange_rates`, une seule
     valeur, avec sa `source` : `afc` pour une copie d'une valeur publiée ou une valeur reprise par ↑, `manual` pour un
     cours saisi, ou toute autre valeur brute (`RateOrigin::Other`, p. ex. `ecb`, A30), **traitée comme `manual`** (`sourceDate` `null`, genre et comparaison inchangés) mais affichée et auditée telle quelle ; jamais la dernière valeur publiée de `_kesh_fetched_exchange_rates`, si bien qu'une correction de
     l'AFC que la société n'a pas reprise ne change pas le cours proposé), selon le **genre
     de clôture de l'exercice** (`fiscal_years.closing_rate_kind`, 37-1 AC-1, réglé à l'écran *Cours de change*, 37-2
     AC-6) :
     - genre **`day`** (défaut) : (1) le cours effectif **journalier** de la **date de fin d'exercice** ; (2) à défaut,
       le cours effectif **mensuel** du mois de cette date (le dernier mois de l'exercice), **signalé** comme repli ;
       (3) à défaut, `RATE_UNAVAILABLE` ;
     - genre **`month`** : le cours effectif **mensuel** du dernier mois de l'exercice, choisi et non un repli (aucun
       signalement, aucun bouton « Récupérer le cours du jour ») ; à défaut, `RATE_UNAVAILABLE`.
     Le genre se lit avec l'exercice (handler, `find_by_id_in_company`, avant la transaction, comme les cours) ; sous
     verrou, la passation relit l'exercice et refuse si le genre a changé entre-temps (409
     `OPTIMISTIC_LOCK_CONFLICT`, « rechargez l'aperçu » — un cours ne se résout jamais sous le verrou, AC-2).
     Surchargeable par devise dans la requête (`rates`), comme avant.
     La réponse porte, par devise, `{ currency, rate, rateKind: "day"|"month"|"request", rateDate, defaultRate,
     defaultSource, overridden, sourceDate }` (U1, CR-5 — remplace `afcRate` : `defaultRate` et `defaultSource` =
     le cours retenu par la société et sa source (valeur brute : `"afc"`, `"manual"` ou autre, A30) de la **résolution par défaut**, tous deux
     `null` s'il n'y en a pas ; `sourceDate` = date de publication d'un cours journalier, lue sur la valeur publiée
     (`find_fetched_rate_in_tx`, 37-2 AC-5, pour l'affichage seulement), **seulement quand `defaultSource` vaut `"afc"`** :
     `null` pour un cours `manual` ou de toute autre source (A30 : traitée comme `manual`) (la date de publication d'une autre valeur ne qualifie pas un cours saisi), et `null`
     si elle n'est pas connue (valeur publiée absente, p. ex. après une restauration), comme pour un cours mensuel (37-2 : `sourceDate` `null` hors `day`) ; `rateDate`, `defaultRate` et `defaultSource` sont toujours ceux
     de la résolution par défaut de la devise — jour, sinon mois —, que l'écran affiche sous le champ, AC-6) ;
     l'aperçu porte en outre `closingRateKind` (`day` | `month`), d'après lequel l'écran distingue un repli d'un choix.
     **Genre d'un cours reçu dans `rates`** — règle **unique**, la même à l'aperçu et à la passation (AC-2, audit) : égal
     au cours par défaut résolu par le serveur → genre de ce défaut (`day` ou `month`) ; différent, ou sans défaut →
     `request` (comparaison **numérique**, zéros terminaux ignorés ; `overridden` vaut `true` si et seulement si le genre est `request` : sens **propre à l'aperçu** — 37-2 n'a plus de champ de ce nom depuis CR-5 ; l'écran affiche « cours saisi » quand le genre est `day` ou `month` et que `defaultSource` vaut `"manual"`). Sans cette règle à l'aperçu, un cours prérempli renvoyé ferait disparaître le signalement du repli et
     masquerait le cours du jour récupéré. **L'écran n'envoie à l'aperçu que les cours modifiés par l'utilisateur** ;
     après une récupération, un champ non modifié reprend le nouveau cours par défaut (la passation, elle, reçoit tous
     les cours affichés **des devises d'au moins un compte coché** — toute autre clé serait refusée,
     `FX_RATE_INVALID`, AC-4 —, AC-2). **Toute** relecture de l'aperçu (case, cours modifié, récupération) porte la
     sélection courante (`accountIds`) et les cours modifiés de ces seules devises, et seule la dernière réponse est
     retenue ; à sélection vide, aucune relecture (AC-4). Quand `closingRateKind` vaut `day`, l'écran **signale** un
     `rateKind: "month"` (« cours du jour de clôture indisponible : cours mensuel de <mois> ») et propose « Récupérer
     le cours du JJ.MM.AAAA » (`POST /exchange-rates/fetch { date }`, 37-2), puis relit l'aperçu (règle de relecture ci-dessus : pas de relecture à sélection vide, le nouveau cours est alors repris à la prochaine case cochée) ; un
     `404 RATE_NOT_PUBLISHED` ou un `503 RATE_SOURCE_UNAVAILABLE` s'affiche (message traduit) et laisse l'aperçu tel
     quel, repli mensuel compris. ⚠️ **Date de fin d'exercice
     non passée** : 37-2 ne récupère un cours journalier que pour un jour **strictement antérieur** à aujourd'hui
     (Europe/Zurich ; sinon 400) — le bouton n'est alors **pas proposé** et, si le repli mensuel est pris ou qu'aucun
     cours n'existe, l'écran dit « date de clôture pas encore passée : cours du jour non publié » ; le cours par défaut
     suit la même résolution (un cours journalier **saisi** par la société pour ce jour, D9, reste le premier ; sinon
     le cours effectif mensuel du mois de cette date — celui du mois courant ou du mois suivant s'il a été copié, un mois
     postérieur seulement s'il a été saisi), sinon
     `RATE_UNAVAILABLE` et saisie ; « Récupérer le mois » n'est pas proposé non plus pour un mois postérieur au
     mois suivant le mois courant (37-2 AC-5 : le mois suivant est admis à toute date, non publié → 404
     `RATE_NOT_PUBLISHED` ; R4 amendé, alignement de l'orchestrateur). Le serveur ne lit pas la date du jour : seul l'écran en dépend. Le cours journalier
     d'un jour sans publication (week-end) est celui de la publication antérieure (37-2, *Cours journaliers*) : un
     exercice qui finit un dimanche prend le cours du vendredi, `sourceDate` l'affiche.
   - **Règle de calcul en débit net**, quel que soit le type du compte : `devise_net` = Σ `amount_currency` au débit −
     Σ au crédit sur **toutes** les lignes du compte (T3, CR-4 : elles sont toutes dans la devise du compte, la ligne
     n'en portant aucune ; `balances_as_of` de 37-4 AC-1 quater) ; `chf_net` = Σ débit − Σ crédit ; **D = round(devise_net ×
     cours, 0.01, MidpointAwayFromZero) − chf_net**. D > 0 : débit du compte réévalué, crédit du compte de gains ;
     D < 0 : crédit du compte réévalué, débit du compte de pertes ; D = 0 : rien. Ce sens vaut pour tout signe de solde
     (un actif à solde créditeur dont le cours monte donne une perte ; un passif à solde débiteur suit la même règle).
     Les gains et pertes affichés sont Σ(D > 0) et Σ|D < 0|. Les soldes affichés gardent le signe naturel du type.
     `balances_as_of` (37-4 AC-1 quater, dans `kesh-db`) rend directement le **débit net brut** (`chf_net`,
     `devise_net`), sans signe par type : aucune conversion de signe ; le signe naturel ne sert qu'à l'affichage. Les
     cinq cas de signe (AC-7) se testent sur le calcul pur de `kesh-core`. *(Alignement de l'orchestrateur sur
     37-4-P2-1, 2026-11-14.)*
   - Un compte à solde en devise nul mais à solde CHF résiduel ≠ 0 **est proposé** (D = −chf_net) : il n'est pas écarté
     par un `HAVING balance != 0` de rapport.
   - Comptes présélectionnés : tous ceux de bilan en devise, actifs et imputables, **dans la limite de 498** (AC-4) : au-delà,
     l'aperçu présélectionne les 498 premiers par numéro de compte et le signale par un champ de la réponse (nom au
     développement, testé à l'AC-7), que l'écran affiche (la passation en refuserait davantage, AC-4). Un compte **archivé** à solde ≠ 0 est
     listé avec `reason: ACCOUNT_ARCHIVED`, non sélectionnable, avec avertissement (sinon l'écriture entière serait
     refusée en `InactiveOrInvalidAccounts`, `validate_lines_accounts_in_tx`). Comptes `Revenue`/`Expense` jamais
     proposés.
   - Aucun cours du genre de clôture (au genre `day` : ni journalier ni mensuel ; au genre `month` : pas de mensuel ;
     ni saisi ni AFC) sans surcharge → la devise est signalée
     `RATE_UNAVAILABLE` (`details` de 37-2, forme `month` : le mois de la date de fin), ses comptes exclus jusqu'à
     saisie ou récupération.
   - **Cours résolus avant toute transaction**, par lecture de la table des cours (37-2 : `lookup` est en lecture
     seule, sans appel sortant). Un cours absent rend la devise `RATE_UNAVAILABLE` dans l'aperçu ; l'écran propose
     « Récupérer le cours du JJ.MM.AAAA » (**genre `day` seulement**, date de fin passée : au genre `month`, le cours
     du jour ne serait pas retenu), puis « Récupérer le mois » (mois publié) (`POST /exchange-rates/fetch { date }`
     puis `{ year, month }`, 37-2, audités à part) ou la saisie du cours. Le serveur de 37-5
     **ne récupère jamais** de cours lui-même ; aucune récupération n'a lieu sous le verrou de l'exercice.
     L'aperçu **n'écrit rien**. *(Alignement de l'orchestrateur sur la remédiation P1 de 37-2, 2026-11-14.)*
   - **Avertissements de l'aperçu** (sans refus) : l'aperçu signale, sans écrire, ce que la passation refuserait —
     `EARLIER_FISCAL_YEAR_OPEN`, `LATER_FISCAL_YEAR_CLOSED`, `PERIOD_LOCKED`, `FX_ACCOUNT_INVALID` — dans un champ
     `blockers[]` (code et détails), dont `MULTI_CURRENCY_DISABLED` `reason: "laterFiscalYear"` (37-3a : un exercice
     postérieur mono-devise) ; l'écran désactive « Passer les écritures » tant qu'il n'est pas vide
     (`FX_ACCOUNT_INVALID` : sur les comptes sélectionnés seulement, l'aperçu relu à chaque case, AC-4).
   - **Une ligne d'aperçu par compte = une écriture (R9)** : l'aperçu présente, par compte retenu, l'écriture qui sera
     passée (compte réévalué, contrepartie, montant, libellé de l'AC-2) ; les totaux des gains et des pertes restent
     affichés.
2. **AC-2 — Passation.** `POST …/fx-revaluation { rates, accountIds }` (pas de `version` : `FiscalYear` n'en a pas ;
   l'idempotence vient du rejeu, AC-3, et du verrou de l'exercice). Le serveur ne fait jamais confiance à l'aperçu : il
   recalcule.
   - **Fonction de dépôt unique** `fx_revaluation::post(pool, user_id, actor_api_key_id, company_id, fiscal_year_id,
     rates, account_ids)` (`actor_api_key_id: Option<i64>`, propagé depuis `CurrentUser` par le handler : c'est
     l'argument qu'exige `NewAuditLogEntry::for_actor`, `entities/audit_log.rs:195`, pour l'audit ci-dessous), qui ouvre **une transaction** ; c'est le **handler** qui l'appelle dans
     `kesh_db::retry::retry_on_deadlock("fx_revaluation::post", …)`, comme `close_fiscal_year`
     (`routes/fiscal_years.rs:361`) — le registre des routes (ci-dessous) contrôle l'enveloppe au handler. Elle
     appelle ensuite `create_in_tx(…, enforce_postable = true)` **une fois par compte réévalué** (R9), dans l'ordre des
     numéros de compte, **dans la même transaction** : toutes les écritures passent, ou aucune (un refus de l'une
     annule les autres) ; l'audit `journal_entry.created` de chacune est écrit par `create_in_tx_inner`. Une `rates` qui omet une devise sélectionnée : cours par défaut de l'AC-1, **selon le
     genre de clôture** lu avec l'exercice (`day` : effectif journalier, sinon effectif mensuel ; `month` : effectif
     mensuel seul), lu **avant** la transaction, `RATE_UNAVAILABLE` (404) s'il manque. Les devises des comptes sélectionnés se lisent
     avant la transaction par une lecture ordinaire sur le pool (elles ne servent qu'à choisir les cours à résoudre et à
     contrôler les clés de `rates`) ; **dans** la transaction, après l'étape (4), toute devise d'un compte admissible
     sélectionné qui n'a pas de cours résolu (devise du compte changée entre-temps) → rollback, 404 `RATE_UNAVAILABLE`
     (`details` de 37-2) : un cours n'est jamais supposé. Ce refus se place, dans l'ordre de l'AC-4, juste après
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
     `LOCK IN SHARE MODE` (jamais `FOR SHARE`, erreur de syntaxe sur MariaDB 10.11 ; ce partage bloque aussi toute
     désignation de comptes propres concurrente, qui prend `companies` en exclusif, 37-1 AC-4 bis : les désignations
     lues ensuite restent celles du calcul) **avant tout exercice** : c'est
     l'ordre global `companies` → exercice des chemins Pattern 5 (`repositories/invoices.rs:698`, sentinelle puis `create_in_tx`) et
     de 37-3a AC-4 (`companies` en tête de l'étape 0 de `create_in_tx_inner`) ; prise après l'exercice, elle formerait
     le cycle exercice → `companies` contre `companies` → exercice que 37-3a écarte (aucun flux ne prend un exercice
     puis `companies` **en exclusif** : ni `close`, ni `reopen`, ni `lock_books`, `repositories/companies.rs:1002-1020` ; la
     contre-passation d'une écriture en devise de 37-3a prend l'exercice puis `companies` en partagé, compatible) ; (1) chaque antérieur
     verrouillé par clé primaire (`LOCK_EARLIER_BY_ID_SQL`) dans l'ordre chronologique, puis
     l'exercice `FOR UPDATE` (`LOCK_IN_COMPANY_SQL`) ; (2) relecture **verrouillante** du plus ancien antérieur ouvert (`earlier_open_locking_sql`,
     même texte que `close`) → `EARLIER_FISCAL_YEAR_OPEN` ; (3) les désignations propres des comptes
     sélectionnés (`SELECT id, number, currency, fx_gain_account_id, fx_loss_account_id FROM accounts WHERE
     company_id = ? AND id IN (…) ORDER BY number LOCK IN SHARE MODE`, S2 ; `number` et `currency` servent à
     `fx_accounts_for` et à l'ordre des refus), d'où, par `fx_accounts_for`, les paires de comptes de gains et de pertes
     **employées** — **par les comptes sélectionnés admissibles** (la liste de l'étape (4), CHF, archivés, de groupe et
     de résultat exclus, D nul ou non) : les verrous de cette étape couvrent tous les comptes sélectionnés, l'évaluation
     des refus (AC-4) ne retient que les admissibles, après la lecture de l'étape (4) — `validate_fx_accounts_in_tx`
     (37-1), qui rend son refus dès qu'elle le constate, est donc appelée ici pour la paire de **chaque** compte
     sélectionné (verrous), son résultat **gardé par compte sans être propagé** (une erreur de base, elle, se propage
     aussitôt) ; après l'étape (4), le premier `FX_ACCOUNT_INVALID` d'un compte **admissible**, dans l'ordre des
     numéros, est rendu à son rang de l'AC-4 (ainsi qu'un `None` de `fx_accounts_for`, AC-4) ; la projection lue
     ici est celle qu'exige la signature de `fx_accounts_for` au développement (un `Account` complet si elle le
     prend) ; puis réglages de facturation et articles qui les désignent (`default_revenue_account_id`, contrôle
     de l'AC-4), puis ces comptes de gains et de pertes, lus `LOCK IN SHARE MODE` par `validate_fx_accounts_in_tx`
     (37-1 ; ordre `companies` → réglages → `accounts` de 37-1 AC-3 ; ce flux prend les comptes sélectionnés et les comptes
     désignés tous en partage, aucun en exclusif : pas de cycle entre eux) ; (4) seulement
     ensuite, les soldes cumulés (`balances_as_of`, 37-4) et la liste des comptes admissibles. L'ordre des **verrous**
     n'est pas l'ordre des **refus** (AC-4) : tout se lit d'abord, puis s'évalue dans l'ordre de l'AC-4. Le test des deux
     passations concurrentes (AC-7) fait attendre la seconde sur le verrou de l'exercice pendant que la première
     commite, et constate une seule écriture.
   - **Source du cours** (pour l'audit ; U1, CR-5) : par devise, la **source de la ligne `exchange_rates`** de la
     résolution par défaut (`defaultSource` de l'AC-1, valeur brute : `afc`, `manual` ou autre, A30) quand le cours retenu **égale**
     `defaultRate` (genre `day` ou `month` selon la règle unique de l'AC-1), qu'il vienne de la table ou de `rates`
     (l'écran renvoie les cours préremplis) ; `manual` sinon (genre `request`) — y compris un cours saisi par la société
     (D9, `defaultSource` `manual`), et une devise sans cours retenu (`defaultRate` `null`). La source d'un cours `afc`
     se garde ainsi **pour toujours** à l'audit de la réévaluation, même après une correction de l'AFC ou une
     restauration (U1). Limite écrite (37-5-CR5-P1-3) : si le cours retenu change entre l'aperçu et la passation (↑ ou saisie d'un autre utilisateur), le cours renvoyé par l'écran diffère du nouveau défaut : genre `request` et source `manual` à l'audit — le cours posé est bien celui que l'utilisateur a vu ; test de dépôt. L'audit porte aussi le **genre** du cours retenu par devise (`day` | `month` | `request`),
     selon la règle unique de l'AC-1 (cours reçu égal au défaut : genre de ce défaut ; sinon `request`) : comme l'écran
     renvoie les cours préremplis, le genre serait sans cela toujours `request` et ne dirait rien. Ces lectures (cours
     retenus par la société, du jour et du mois de fin, avec leur source) se font **avant la transaction**, avec celles des cours (37-2 : aucune lecture de cours
     sous le verrou de l'exercice), et non à la volée sous les verrous.
   - **Une écriture par compte réévalué (R9, CR-2 — remplace l'écriture unique)**, comme les livres 2024 de Beat
     (« Gain de change PAYPAL ONLINE USD 0.91193 ») : chaque compte d'écart non nul reçoit **sa** pièce journal `OD`,
     datée du dernier jour de l'exercice, de **deux lignes** — la ligne du compte réévalué et sa contrepartie CHF (au
     crédit du compte de gains si D > 0, au débit du compte de pertes si D < 0, pour |D|). Chaque écriture ne porte
     ainsi qu'**une** devise (`FX_MULTIPLE_CURRENCIES` de 37-3a ne peut pas se lever). Libellé **en français**
     construit par **une fonction unique** (`kesh-db`, appelée par `post` **et** par l'aperçu, qui montre le libellé qui sera passé — DRY ; test : aperçu et écriture passée portent le même texte), comme la contre-passation (`crates/kesh-db/src/repositories/journal_entries.rs:2975`) :
     « Gain de change <nom du compte> <DEVISE> <cours> au JJ.MM.AAAA (non réalisé) » ou « Perte de change … » (cours
     par unité, sans zéros terminaux ; le nom du compte est tronqué pour que le libellé reste sous les 500 caractères
     de `MAX_DESCRIPTION_LEN`, privé à `routes/journal_entries.rs` : un test sur le libellé produit, nom de compte
     maximal compris, suffit).
   - Ligne du compte réévalué : `ForeignAmount { declared_currency: None, source: Revaluation, amount: 0, rate }`
     (T3, CR-4 : la ligne **ne porte pas de colonne de devise** — sa devise est celle du compte réévalué, lue par
     jointure ; le contrôle central de 37-3a la vérifie **par la devise du compte** : la ligne `foreign` sur un compte
     CHF serait refusée `FX_ACCOUNT_IN_CHF`, et `FX_CURRENCY_MISMATCH` ne peut naître, aucune devise n'étant annoncée ; ces refus du rang 3 de 37-3a sont
     inatteignables ici, la devise du compte étant lue sous verrou à l'étape (3) — filet, non refus métier ;
     le cours retenu est celui de la devise du compte, lue au `SELECT … LOCK IN SHARE MODE` de l'étape (3)), `rate_source =
     'revaluation'`, `amount_currency = 0`, `exchange_rate` = cours retenu (> 0, par unité), CHF = |D|. **Cette ligne
     est exemptée de l'égalité CHF = montant × cours et de `FX_LINE_ROUNDS_TO_ZERO`** (le CHF est pris tel quel, > 0) ;
     la saisie manuelle de 37-3a (AC-6) la refuse, c'est la route de 37-5 qui la construit.
   - Contrepartie : **une** ligne CHF par écriture, sur le compte de gains ou de pertes **du compte réévalué**
     (`fx_accounts_for` : ses comptes propres, sinon ceux de la société — S2, CR-3 ; p. ex. PayPal USD sur son
     sous-compte de frais de change cachés, Wise USD sur 6840) ; le même compte peut servir aux deux, et plusieurs
     comptes réévalués peuvent partager une contrepartie : chaque écriture n'en porte qu'une ligne, aucune agrégation
     ni compensation entre comptes — la règle « une seule ligne au net » de l'écriture unique disparaît.
   - Aucun écart non nul → 200 sans écriture (`{ entryIds: [] }`) ; sinon 200 `{ entryIds: [...] }` (dans l'ordre des
     numéros de compte).
   - **Audit** : outre le `journal_entry.created` de chaque écriture, **une** ligne d'audit propre
     `fx_revaluation.posted` qui **liste les écritures** passées — `{ entries: [{ entryId, accountId, currency, rate,
     delta, counterpartAccountId, fxAccountsOrigin }], rates, source par devise (valeur brute : afc|manual|autre, A30), genre par devise (day|month|request), closingRateKind,
     accountIds }` — écrite **dans la même transaction** (la source
     du cours, perdue sur la ligne puisque `rate_source = 'revaluation'`, y est conservée), entité `fiscal_year`
     (déjà au registre des types, `audit_labels.rs:40`), identifiant = l'exercice. **Aucune** ligne d'audit quand
     aucun écart n'est passé (`entryIds: []` : rien n'a changé). Libellé d'audit dans les quatre locales. Limite
     connue : l'acteur d'une clé d'API n'apparaît pas à l'audit de la création d'écriture (`NewAuditLogEntry::user`),
     comme pour toute écriture existante ; `fx_revaluation.posted` l'écrit par `NewAuditLogEntry::for_actor`.
   - **Registres** : action `fx_revaluation.posted` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) et sa clé `audit-log-action-fx-revaluation-posted` dans
     les quatre locales ; les deux routes entrent à `LIB_ROUTES` (`tests/audit_route_registry.rs`) — aperçu
     `NoMatter("calcule sans persistance — aucune mutation"), SansEcritureAuJournal` (comme `bank_imports::preview`,
     `:540`), passation `Traced, Rejouee` ; les décomptes de `the_registry_partition_is_what_the_story_declares` (+2 routes, **total du registre +2**) et la
     phrase du manuel d'administration (`admin-manual.tex:3217`, gardée par `le_manuel_dit_le_compte_du_registre`)
     suivent, **cumulés** avec ceux des 37-1, 37-2 et 37-3b (route de conversion) (37-5 : +2 routes, dont +1 tracée et +1 sans mutation : l'assertion
     `no_matter` passe de 3 à 4, et la phrase du manuel change **en chiffres et en toutes lettres** — « N + 4 + 4 = M » et
     « quatre autres routes qui n'écrivent rien », que la garde, qui ne lit que les chiffres, ne contrôle pas). Les décomptes de
     `the_replay_partition_is_what_the_story_declares` (`audit_route_registry.rs:1450`) suivent aussi : l'aperçu
     `SansEcritureAuJournal` fait `sans_ecriture` +1, la passation `Rejouee` fait `rejouees` +1, `tout.len()` +2
     (:1502), en deltas cumulés avec ceux des 37-1, 37-2 et 37-3b ; la passation appelle une enveloppe
     (`every_replayed_route_calls_an_envelope`, `audit_route_registry.rs:1290`), dont l'assertion
     `assert_eq!(examinees, 27, "les vingt-sept routes Rejouee …")` (:1324-1327) suit aussi, +1 en delta (avec la
     route de conversion de 37-3b), chiffre et toutes lettres. *(Alignement de l'orchestrateur sur 37-1-P4-IF-2.)*
3. **AC-3 — Rejouable, compte par compte (R9).** Un second passage, sans mouvement entre-temps, ne passe rien ; après
   une écriture tardive en devise sur un compte, il ne passe qu'**une** écriture, le résidu de **ce** compte (l'écart se
   calcule par compte sur le cumul, les autres comptes ont D = 0) ; la **suppression** de l'écriture
   d'un seul compte, puis un nouveau passage, ne repasse que ce compte. ⚠️ **La contre-passation ne corrige pas une
   réévaluation de l'exercice** : l'écriture inverse est datée du **jour** (dans l'exercice ouvert qui le couvre) ou du
   **premier jour de l'exercice suivant** (`ReversalDating::Today` / `NextFiscalYearStart`,
   `journal_entries.rs:2931-2961`). Cas ordinaire — la fin de l'exercice est passée, ou l'inverse est au début de
   l'exercice suivant — : elle tombe **hors** du cumul arrêté à la fin de l'exercice, qui continue de compter
   l'original ; un nouveau passage ne repasse rien, et l'effet se déplace dans l'exercice qui la porte (le cumul de cet
   exercice voit les deux). Cas d'un exercice **en cours** (réévaluation passée avant sa fin, AC-1 ; `Today` ≤ date de
   fin, l'exercice du jour **est** celui de la réévaluation) : l'inverse tombe **dans** le cumul, annule l'original, et
   un nouveau passage repasse l'écart du compte. La correction d'une réévaluation de l'exercice est la suppression
   (exercice ouvert) ou un nouveau passage au résidu. Pas d'extourne au premier jour de l'exercice suivant. **Le report à nouveau des
   soldes en devise se fait par le cumul** (`opening.rs:17`, Kesh ne passe aucune écriture de clôture) : aucune écriture
   de report n'est créée. Une réévaluation se corrige par **suppression** de l'écriture, ou par un
   nouveau passage (résidu) ; le `PUT` de la saisie manuelle la refuse (37-3a AC-7 : 409 `ENTRY_IS_FX_REVALUATION` ;
   et 37-3a AC-6 refuse `rateSource: "revaluation"` dans toute requête d'écriture).
4. **AC-4 — Gardes.** Ordre des refus, **en deux temps** (une seule raison, la plus bloquante d'abord ; l'état
   **clos** de l'exercice parle avant les comptes et les cours ; les refus de forme (400) et le cours par défaut manquant (AC-1),
   **avant la transaction**, précèdent `EARLIER_FISCAL_YEAR_OPEN`, qui ne se lit que sous verrou).
   **Avant la transaction** (handler) : exercice inexistant ou d'une autre société → 404 (le handler appelle d'abord
   `fiscal_years::find_by_id_in_company`) ; société **ou exercice** mono-devise → **409** `MULTI_CURRENCY_DISABLED`
   (409 partout, décision d'epic ; R3 : `fiscal_years.multi_currency` de 37-1, `details: { fiscalYearId }` pour
   l'exercice) ; exercice clos (lecture non verrouillante, refaite dans la transaction) → **400** `FISCAL_YEAR_CLOSED`
   (`DbError::FiscalYearClosed`, `errors.rs:4516` — le code et le statut d'une écriture dans un exercice clos, non le 409
   `ILLEGAL_STATE_TRANSITION` de `close`) ; forme de
   la requête (bornes ci-dessous) → 400 `FX_RATE_INVALID`, ou 400 `FX_REVALUATION_ACCOUNT_INVALID` sur `accountIds` ; aucun cours par
   défaut (AC-1, selon le genre de clôture : ni journalier ni mensuel au genre `day`, pas de mensuel au genre
   `month`) pour une devise sélectionnée sans surcharge → 404 `RATE_UNAVAILABLE`.
   **Dans la transaction**, après tous les verrous de l'AC-2 et **dans le même ordre qu'avant la transaction** (sauf le
   genre de clôture changé et le `RATE_UNAVAILABLE` d'une devise changée entre-temps, qui ne peuvent naître qu'ici et
   viennent après les comptes, ci-dessous) :
   réglage relu sous verrou (étape (0)) et mode de l'exercice relu dans sa ligne `FOR UPDATE` (étape (1)) : société ou
   exercice mono-devise → 409 `MULTI_CURRENCY_DISABLED` ; exercice clos →
   `FISCAL_YEAR_CLOSED` ; **exercice antérieur encore ouvert → `EARLIER_FISCAL_YEAR_OPEN`** (comme K1 : le cumul lu ne
   serait plus stable) ; comptes de différences de change **employés** (résolus par `fx_accounts_for` pour chaque compte sélectionné
   **admissible**, S2 : un compte sélectionné non admissible, CHF ou archivé par exemple, n'emploie rien et n'est
   refusé que par `FX_REVALUATION_ACCOUNT_INVALID` plus bas ; l'admissibilité se lit à l'étape (4), avant cette
   évaluation) **revalidés** par `validate_fx_accounts_in_tx` (37-1 AC-4 bis : société, actif,
   imputable, de type `Expense`/`Revenue`, en CHF, ni au rôle `DefaultRevenue`, ni désigné
   `default_revenue_account_id` des réglages de facturation ou d'un article — 37-1 P3-8) → 400 `FX_ACCOUNT_INVALID` (C-37-2 ; même code et même statut que
   37-1, `details` de 37-1, plus `holderAccountId` pour un compte propre ; le premier fautif dans l'ordre des
   numéros de compte réévalués, gain avant perte ; un compte de la société invalide qu'**aucun** compte sélectionné
   n'emploie ne bloque rien ; un compte admissible que ni ses comptes propres ni la société ne dotent de comptes de
   différence — `fx_accounts_for` rend `None`, état hérité — → 400 `FX_ACCOUNT_INVALID`, `reason: "missing"`, `field`
   `fxGainAccountId`, `holderAccountId` = ce compte, même à D = 0 : à l'aperçu, un blocker `FX_ACCOUNT_INVALID`, et
   `counterpartAccountId` `null` sur la ligne — la ligne reste **sélectionnable** (le compte est admissible) : la
   décocher lève le refus. Les blockers `FX_ACCOUNT_INVALID` de l'aperçu portent sur les comptes **sélectionnés** de
   la requête (`accountIds`, à défaut la présélection), comme la passation ; l'écran relit l'aperçu, avec
   `accountIds` **et** les cours modifiés (`rates`, limités aux devises d'au moins un compte coché : la requête
   refuse toute autre clé, `FX_RATE_INVALID`, ci-dessous), à chaque case cochée ou décochée — dernière réponse
   seule retenue —, et ne désactive « Passer les écritures » que sur les blockers de cette sélection ; **aucune
   relecture à sélection vide** (`accountIds` vide serait refusé, `reason: "empty"`) : le bouton reste désactivé,
   sans blocker) ; `accountIds` hors de la liste admissible (lue à l'étape (4) de l'AC-2) → 400
   `FX_REVALUATION_ACCOUNT_INVALID`, ensuite ; puis genre de clôture changé depuis la lecture d'avant la transaction →
   409 `OPTIMISTIC_LOCK_CONFLICT` (AC-1 ; après les refus d'état et de comptes : le motif réel prime sur
   « rechargez », comme l'étape 3-bis d'`update_in_tx` avant la version) ; puis 404 `RATE_UNAVAILABLE` d'une devise
   sans cours résolu (AC-2) ; puis, par `create_in_tx_inner`, dans l'ordre de ses étapes : exercice
   postérieur clos → `LATER_FISCAL_YEAR_CLOSED` (étape 1-a, garde `journal_entries.rs:368`, repository ; état hérité
   seulement, puisque `close` impose l'ordre) ; `DATE_OUTSIDE_FISCAL_YEAR` ; livres verrouillés à cette date
   (`books_locked_through`, seuil **inclusif** : `entry_date <= locked_through`, étape 1-bis) → `PERIOD_LOCKED` ;
   enfin exercice postérieur mono-devise → 409 `MULTI_CURRENCY_DISABLED` `reason: "laterFiscalYear"` (garde de 37-3a,
   rang 2 de son AC-4, après la validation des comptes de l'étape 2 de `create_in_tx_inner`, donc **après**
   `PERIOD_LOCKED` ; rendue ici sur les lignes en devise de l'écriture). L'aperçu
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
     en devise (37-3a AC-3) et d'un cours saisi (37-2 AC-5) et se contrôlent par **la même fonction** de `kesh-core`
     (DRY), **créée par 37-2** (AC-2, bornes d'un cours seul), qui rend une erreur de **type propre** (p. ex.
     `RateBoundsError { Scale, TooLarge, NotPositive }`, non une variante de `CoreError` liée à un code — arbitrage de
     l'orchestrateur, CR-1 P2) ; 37-5 traduit **toute** variante en 400 `FX_RATE_INVALID`, `details: { currency }`
     (la devise fautive), code **créé par 37-2** (variante `AppError` et traduction) et réutilisé ici. **Le produit** devise_net × cours (devise_net est
     un cumul, non borné par ligne) se calcule en entiers `i128` par multiplication **contrôlée** (`i128::checked_mul` ; un dépassement →
     `FX_VALUE_OUT_OF_RANGE`), jamais par `Decimal::checked_mul` (37-3a AC-3 : réduction d'échelle silencieuse) ; un
     CHF réévalué ou un |D| > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE`, sans écriture. **Forme du refus** : la variante de 37-3a
     (`FxValueOutOfRange { line }`) porte un rang de ligne que 37-5 n'a pas ; 37-5 rend le même code et le même statut par
     sa **propre variante** de `DbError` (`FxRevaluationValueOutOfRange`, `kesh-db` ; bras dans
     `kesh-api/src/errors.rs`), que l'aperçu et la passation rendent tous deux (le calcul pur de `kesh-core` signale le
     dépassement, le code de `kesh-db` qui l'appelle le traduit), `details: { currency, accountId }` pour un compte
     fautif, sans toucher
     `map_core_error`. Depuis R9 (une écriture par compte), la contrepartie d'une écriture vaut |D| de son seul compte,
     déjà borné à 10¹³ : il n'y a plus de **somme** de contrepartie à borner (le refus d'une somme de contrepartie hors
     bornes, `details: {}`, de l'écriture unique disparaît ;
     Σ gains et Σ pertes ne sont plus qu'affichés, calculés en `Decimal` sans écriture). Au plus 100 devises et **498**
     comptes par requête — borne **conservée** (taille de la transaction, de l'aperçu et du détail d'audit
     `fx_revaluation.posted`) ; elle ne vient plus de `MAX_LINES_PER_ENTRY` (chaque écriture a deux lignes) : plus de 498
     `accountIds` → 400 `FX_REVALUATION_ACCOUNT_INVALID`, `reason: "tooMany"` ; plus de 100 clés dans `rates` → 400
     `FX_RATE_INVALID`, `details: { reason: "tooMany" }` (sans devise fautive).
5. **AC-5 — RBAC.** Les deux routes vont dans `comptable_routes` (`lib.rs:524`, `require_comptable_role`), comme
   `…/close` (`lib.rs:821`) : Consultation reçoit 403 **y compris sur l'aperçu** (c'est un POST). Pas d'`ensure_not_pat`
   (comme `close`, contrairement à `reopen`), sauf décision contraire de l'utilisateur ; une clé en lecture seule est
   refusée sur le POST d'aperçu comme sur tout POST.
6. **AC-6 — Écran** : depuis *Exercices* (exercice ouvert), action « Réévaluer les devises », **affichée seulement en
   multi-devises** (D1 : en mono-devise, rien n'apparaît) **et aux rôles Comptable et Administrateur** (AC-5) : tableau de l'aperçu,
   cours modifiables par devise (**valable pour cette passation seulement** : le champ n'écrit aucun cours saisi, qui
   se fixe à l'écran *Cours de change*, 37-2 ; sous le champ, le cours retenu par la société (`defaultRate`) quand le
   champ en diffère, sans ↑ ; **source** affichée (U1) : « AFC » (`defaultSource: "afc"`), « cours saisi »
   (`"manual"`), « cours saisi (<source brute>) » pour toute autre source (A30, p. ex. « cours saisi (ecb) », comme
   37-3b AC-2 ; Vitest) ou « manuel » (cours modifié ici, `request`) — la même que l'audit ; **genre** affiché : « cours du JJ.MM.AAAA » pour
   un cours journalier, avec la date de publication si elle diffère, « cours mensuel de <mois> » — signalé comme repli
   au genre de clôture `day` seulement, sans signalement au genre `month` (R6) —, « cours saisi » quand le cours vient
   de la société, D9 ; bouton « Récupérer le cours du JJ.MM.AAAA » quand le genre de clôture est `day`, que le repli
   mensuel est pris ou qu'aucun cours n'existe, et que la date de fin est passée (AC-1) ; au genre `month`, seul
   « Récupérer le mois » est proposé, pour un mois publié — Administrateur et Comptable), cases par compte (comptes archivés grisés avec
   avertissement ; au-delà de 498 comptes admissibles, l'avis de présélection tronquée de l'AC-1, et pas plus de 498 cases
   cochées), l'écriture prévue par compte (R9) avec son compte de contrepartie et son origine (« comptes propres » ou
   « réglage de la société », S2 ; libellés dans les quatre locales), total des gains et des pertes, **« Passer les écritures »** (R9), liens
   vers les écritures passées ; le genre de clôture de l'exercice est rappelé (« cours de clôture : cours journalier du
   JJ.MM.AAAA » ou « cours mensuel de <mois> », avec un lien vers l'écran *Cours de change* qui le règle, R6) ; l'action
   « Réévaluer les devises » n'est offerte que pour un exercice **multi-devises** (R3) ; captures d'écran.
7. **AC-7 — Tests** : deux exercices (achat EUR en N-1, réévaluation N-1, mouvements N, réévaluation N) : solde CHF au
   31.12.N = round(solde EUR × cours) exactement ; cours par défaut = cours **journalier** de la date de fin
   (`rateKind: "day"`), servi par la source factice de 37-2 et **distinct** du cours mensuel ; sans cours journalier,
   cours mensuel du mois de fin, `rateKind: "month"` ; ni l'un ni l'autre → `RATE_UNAVAILABLE` ; un cours saisi par
   la société (D9, ligne `manual`) pour le jour est le cours du jour, un cours saisi pour le mois ne l'emporte **pas**
   sur un cours journalier présent (l'ordre est jour puis mois, chacun effectif) ; source `afc` d'une ligne `afc` du
   jour, `manual` d'une ligne `manual` (même si elle égale la valeur publiée) ; `sourceDate` `null` pour une ligne `manual` ; **A30** : ligne de la société de `source` `ecb` (posée par SQL, `RateOrigin::Other("ecb")`) → cours proposé comme pour `manual`, `sourceDate` `null`, `defaultSource` `"ecb"` rendu tel quel à l'aperçu et à l'audit ; **U1** : correction de l'AFC du jour de
   clôture non reprise → cours proposé = l'ancien cours retenu, source `afc` ; après ↑ (37-2) → la valeur corrigée,
   source `afc` ; restauration d'un `.keshbackup` sur une installation neuve → aperçu au cours restauré, sans aucune
   récupération, source restaurée (la source de cours factice de 37-2, tâche et déclenchement à la connexion neutralisés : la preuve est l'absence d'appel sortant ; `sourceDate` `null` tant que la valeur publiée n'est pas relue) ; exercice **se terminant le 30 juin** (un dimanche en 2024) → cours
   journalier du 30.06 = publication du 28.06 (`sourceDate`) ; rejeu sans écriture ; compte
   décoché ou de résultat : aucune ligne ; cours surchargé figé sur la ligne ; refus des gardes. S'y ajoutent :
   - **les cinq cas de signe** : actif USD débiteur, cours en hausse (gain) puis en baisse (perte) ; actif USD à solde
     créditeur, cours en hausse (perte) ; passif USD créditeur, cours en hausse (perte) ; passif USD à solde débiteur, cours en hausse (gain) ;
     devise à 0 avec CHF résiduel ≠ 0 ;
   - compte de groupe, archivé, non imputable ; plusieurs devises (USD et EUR) dans la même **passation** (une écriture
     par compte, R9) ;
   - **une écriture par compte (R9)** : trois comptes à écart non nul (USD actif, USD passif, EUR) → trois écritures
     de deux lignes, une devise chacune, libellés « Gain de change … » / « Perte de change … » ; un compte à D = 0 →
     pas d'écriture ; compte de gains = compte de pertes → une ligne de contrepartie par écriture, aucune compensation
     entre comptes ; atomicité : une erreur survenue après la deuxième écriture (provoquée au test de dépôt ; mécanisme
     au développement, écrit au Change Log) → aucune des trois n'existe après (transaction unique) ; audit
     `fx_revaluation.posted` qui liste les trois `entryId` ;
   - **genre de clôture (R6)** : exercice au genre `month` → cours mensuel du dernier mois, `rateKind: "month"` sans
     signalement de repli ni bouton du jour ; genre `month` sans cours mensuel, un cours journalier présent →
     `RATE_UNAVAILABLE` (le jour n'est pas retenu), passation sans `rates` → 404, écran sans bouton du jour (Vitest) ;
     genre changé entre l'aperçu et la passation → 409 ; exercices **au 1er janvier** (T2, CR-4 : exercice civil
     2024, fin 31.12.2024, un mardi : cours journalier du 31.12.2024 ; genre `month` : décembre 2024), **au 1er
     juillet** (fin 30.06.2024, un dimanche : cours journalier publié le 28.06 ; genre `month` : juin 2024) et **au
     1er avril** (fin 31.03.2024, dimanche de Pâques : publication antérieure, `sourceDate` ; genre `month` : mars 2024) ;
   - **comptes de différence propres (S2, CR-3)** : PayPal USD à comptes propres (gains 6841, pertes 6842) et Wise
     USD « par défaut » (6840) dans la même passation → l'écriture de PayPal a sa contrepartie sur 6841 ou 6842, celle
     de Wise sur 6840 ; aperçu et audit portent `counterpartAccountId` et `fxAccountsOrigin` ; compte propre devenu
     archivé (posé par SQL) → `FX_ACCOUNT_INVALID` avec `holderAccountId`, rien de passé, à la passation **et** en
     `blockers[]` de l'aperçu ; compte sélectionné non admissible (CHF, archivé) dont les comptes propres seraient
     invalides → seul `FX_REVALUATION_ACCOUNT_INVALID` `notEligible` ; compte à D = 0 dont les comptes propres sont
     invalides → refusé (comptes employés) ; ce même compte **décoché** → aperçu (avec `accountIds`) sans blocker et
     passation admise, et l'écran (Vitest) réactive « Passer les écritures » après la relecture, sans perdre les cours modifiés, la dernière case décochée ne déclenchant aucune relecture ; une récupération de cours après ce décochage relit l'aperçu avec la même sélection (le bouton reste actif) ; la passation n'envoie pas le cours d'une devise dont aucun compte n'est coché ; ni comptes propres ni
     comptes de la société (`None`) → `reason: "missing"`, ligne cochable ; premier fautif dans l'ordre des numéros de compte, gain avant perte ; compte de la société
     invalide (par SQL) alors que tous les comptes sélectionnés ont des comptes propres → passation admise ; une
     désignation propre concurrente attend la fin de la passation (verrou `companies`) ;
   - **devise portée par le compte (T3, CR-4)** : la ligne de réévaluation est posée sans devise annoncée et relue
     avec la devise du compte (jointure), pour une devise autre que USD au moins ; un compte EUR dont le cumul contient
     une écriture **EUR–EUR** (T3 : chaque ligne a son cours) est réévalué sur `devise_net` de toutes ses lignes ;
     aucun `FX_CURRENCY_MISMATCH` ni `FX_ACCOUNT_IN_CHF` ne sort d'une passation ordinaire ;
   - **durée de l'exercice (S4, CR-3)** : exercice de **six mois** (01.07.2024–31.12.2024) et de **dix-huit mois**
     (01.07.2023–31.12.2024) → cours journalier du 31.12.2024 (genre `day`) ou cours mensuel de décembre 2024 (genre
     `month`), écart sur le cumul depuis l'origine (achat EUR antérieur à l'exercice compris), une écriture par compte
     datée du 31.12.2024 ; exercice qui finit un 14 (15.03.2024–14.03.2025) → cours journalier du 14.03.2025, repli
     mensuel de mars 2025 ;
   - **exercice mono-devise (R3)** : aperçu et passation → 409 `MULTI_CURRENCY_DISABLED` `details.fiscalYearId`, avant
     la transaction et sous verrou ; exercice postérieur mono-devise → `blockers[]` à l'aperçu **et** 409 `reason: "laterFiscalYear"` à la passation (état posé par SQL, la bascule de 37-1 l'interdisant avec un solde ≠ 0) ;
   - cours invalide (0, négatif, 11 décimales, > 10⁶, devise sans compte) ; au-delà des bornes ;
   - `PERIOD_LOCKED` exactement à la borne (inclusive) ; exercice postérieur clos (test de dépôt, comme
     `opening_complement_repository.rs:963`) ; exercice antérieur ouvert ; exercice clos ; mono-devise ;
   - `FX_ACCOUNT_INVALID` (compte devenu archivé, d'une autre société, en devise, ou compte de gains désigné produit
     par défaut) — états que 37-1 interdit par
     l'API (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) : posés **par SQL** dans le test, comme un état hérité ;
   - l'ordre des refus de l'AC-4 (exercice clos **et** cours manquant → `FISCAL_YEAR_CLOSED`) ; source `afc` d'un
     cours prérempli renvoyé tel quel, `manual` d'un cours modifié ; aucun `fx_revaluation.posted` sans écriture ;
     `blockers[]` de l'aperçu ;
   - RBAC (Consultation 403) et société (exercice d'une autre société 404) ;
   - audit : `journal_entry.created` avec `foreign` au snapshot (`currency` lue sur le compte, `amount`, `rate`,
     `rateSource`, 37-3a AC-8), et `fx_revaluation.posted` ; la ligne relue porte la devise du compte (T3) ;
   - le résidu après écriture tardive ; une réévaluation **supprimée**, puis un nouveau passage (le résidu
     rétablit l'écart) ; une réévaluation **contre-passée** (écriture inverse hors de l'exercice) : le nouveau passage ne
     passe rien, l'original reste au cumul ; contre-passée **le jour même dans un exercice en cours** (date de fin
     future, inverse datée du jour dans le même exercice) : le nouveau passage repasse l'écart du compte ; 498 comptes acceptés, 499 refusés (`reason: "tooMany"`), 101 devises refusées ;
     499 comptes admissibles : l'aperçu en présélectionne 498 et le signale ; libellé ≤ 500 caractères avec un nom de
     compte maximal ; produit devise × cours hors bornes, |D| > 10¹³ → `FX_VALUE_OUT_OF_RANGE` ; `accountIds` vide, en
     doublon, étranger à la société et inexistant → `FX_REVALUATION_ACCOUNT_INVALID`, réponses identiques pour les deux
     derniers ; l'audit `fx_revaluation.posted` d'une passation par clé d'API porte la clé (`for_actor`) ; exercice antérieur ouvert **et** cours manquant → `RATE_UNAVAILABLE` (ordre en deux
     temps) ; deux passations concurrentes : l'une passe, l'autre ne passe rien ;
   - genre, à l'aperçu **et** à l'audit : cours reçu égal au défaut journalier → `day`, égal au défaut mensuel →
     `month`, différent → `request` ; source `manual` d'un cours journalier saisi (ligne `manual`), même égal au cours
     mensuel retenu ; cours de `rates` → `FX_RATE_INVALID` pour chaque variante de l'erreur des bornes de 37-2 ;
   - fin d'exercice non passée (date ≥ aujourd'hui, Europe/Zurich) : Vitest à horloge fixée — pas de bouton du jour
     ni « Récupérer le mois » pour un mois futur, message affiché ; côté serveur, un cours journalier saisi pour ce
     jour est retenu (`day`), sinon repli mensuel ou `RATE_UNAVAILABLE` ;
   - l'aperçu ne crée aucune écriture ; E2E de l'écran, dont le parcours repli mensuel → « Récupérer le cours du
     JJ.MM.AAAA » (source factice de 37-2) → aperçu relu, cours journalier (`rateKind: "day"`, le champ non modifié
     ayant repris le défaut) ; Vitest : l'aperçu ne reçoit que les cours modifiés, un `404 RATE_NOT_PUBLISHED` du
     `POST /fetch` laisse le repli affiché.
8. **AC-8 — Manuel** : réévaluation, cours journalier OFDF de la date de clôture (D10 : la pratique du cours de
   clôture, et les cours que portaient les livres de Beat), contrepartie sur les comptes de différence propres du
   compte réévalué s'il en a (S2), exercice de durée quelconque (S4), repli sur le cours mensuel AFC du mois de clôture (la méthode de
   calcul du cours mensuel ne se décrit qu'après confirmation, R4 amendé), choix du genre de clôture par exercice (R6), une écriture
   par compte réévalué (R9), correction par suppression ou nouveau passage, la contre-passation ne corrigeant que dans un exercice en cours (AC-3), refus en
   exercice mono-devise (R3), cours saisi par la société (D9), récupération du cours du jour, gains et pertes non réalisés
   comptabilisés (CO 960b, choix assumé ; la neutralisation par provision n'est pas offerte, C-37-8), postes non
   monétaires à décocher (stocks, immobilisations, capital en devise), la source du cours (« AFC » ou « cours saisi ») conservée à l'audit et non
   sur la ligne, une correction de l'AFC non reprise (↑) ne changeant pas le cours proposé (U1) ; i18n ; PDF.

**Hors périmètre, tracé** : un contrôle de clôture informatif « comptes en devise à réévaluer » (K9 de
`closing_checks.rs`, dont l'ensemble K1–K8 est clos) — à ouvrir en issue avec la PR (C-37-8).

## Tâches

- [ ] `kesh-core` : calcul pur de la réévaluation en débit net (entrées : soldes, cours ; sortie : lignes) et tests.
- [ ] `kesh-db` : `fx_revaluation::post` (transaction, verrou d'exercice, soldes lus dans la transaction, **une écriture
      par compte** dans la même transaction — R9 —, audit `fx_revaluation.posted` qui les liste, mode et genre de
      clôture de l'exercice relus sous verrou — R3, R6). Soldes lus par `kesh_db::repositories::account_balances::balances_as_of(&mut conn,
      company_id, fin_d_exercice, Some(ids))` (37-4 AC-1 quater : dans `kesh-db`, car `kesh-db` ne peut pas dépendre de
      `kesh-report` ; borne `<=` incluse ; débit net brut ; aucune conversion de signe).
- [ ] `kesh-api` : routes (aperçu, passation) dans `comptable_routes`, gardes, résolution des cours avant la
      transaction (selon le genre de clôture : cours effectif journalier puis mensuel, ou mensuel seul ; 37-2), i18n des erreurs neuves (`FX_REVALUATION_ACCOUNT_INVALID` ;
      `FX_ACCOUNT_INVALID`, `RATE_UNAVAILABLE` et `FX_RATE_INVALID` viennent de 37-1 et 37-2, réemployés) et du libellé d'audit
      (quatre locales) ; registres de l'AC-2 (`ACTIONS`, `LIB_ROUTES`, décomptes, phrase du manuel d'administration).
- [ ] Frontend : écran depuis les exercices (contrepartie et origine par compte, clés i18n neuves dans les quatre
      locales), Vitest, E2E.
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
- `FiscalYear` n'a pas de `version` ; la lecture des cours (37-2) est sans réseau ; la récupération est un geste séparé de l'écran.
- Cours de la recette (fichier d'epic, D10) : USD 0.91193 et EUR 0.94947 au 31.12.2024, cours journaliers ; les cours
  mensuels de décembre 2024 (USD 0.8846, EUR 0.9470) sont le repli. Aucun cours officiel n'est versionné (fixtures
  synthétiques, 37-2).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 7 MEDIUM, 5 LOW ; tous remédiés dans la fiche ; P1-1 (côté
  37-3a) et l'exigence 37-4 de P1-2 portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 8 LOW ; remédiés dans la fiche sauf 37-5-P2-IF1
  (MEDIUM, 37-3a), 37-5-P2-IF2 (LOW, 37-3a/37-3b) et 37-5-P2-IF3 (LOW, 37-2), inter-fiches.
- 2026-11-14 — validation P3 (Sonnet) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf 37-5-P3-IF1
  (LOW, 37-3a), inter-fiches.
- 2026-11-14 — validation P4 (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 7 LOW ; remédiés dans la fiche sauf 37-5-P4-IF1
  (LOW, 37-2 : la lecture du cours par 37-5 se fait avant la transaction, non « sous ses verrous »), inter-fiches.
  Choix de cette passe, à confirmer : `companies` verrouillée **avant** les exercices (P4-1, ordre de 37-3a et de
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
  dans l'aperçu), sinon `RATE_UNAVAILABLE` ; récupération du jour par `POST /exchange-rates/fetch { date }` (37-2) ;
  source `afc` comparée au cours AFC brut du genre retenu ; genre conservé à l'audit `fx_revaluation.posted` ;
  `FX_RATE_INVALID` et la fonction des bornes d'un cours seul désormais créés par 37-2 ; écran et manuel (genre du
  cours, repli, cours saisi) ; tests (jour, repli, cours saisi, exercice finissant un dimanche). AC touchés : en-tête,
  Story, AC-1, AC-2, AC-4, AC-6, AC-7, AC-8, Tâches, Notes.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : fin d'exercice non passée, où 37-2 refuse le cours journalier, et le bouton du jour
  proposé à tort ; genre d'audit toujours `request` puisque l'écran renvoie les cours préremplis. LOW : `rateDate` et
  `afcRate` d'un cours `request` ; lectures des cours AFC bruts avant la transaction ; champ de cours de l'écran
  par passation, sans cours saisi ; tests du genre, de la date non passée et du parcours E2E de récupération.)
- 2026-11-14 — validation CR-1 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : genre `request` à l'aperçu pour tout cours de `rates`, alors que l'écran renvoie
  les cours préremplis — repli et cours du jour récupéré masqués : règle unique de genre à l'aperçu et à l'audit,
  l'aperçu ne reçoit que les cours modifiés ; arbitrage de l'orchestrateur sur la fonction des bornes de 37-2 —
  erreur de type propre, signe compris, toute variante → `FX_RATE_INVALID`. LOW : source `afc` alignée sur
  l'`afcRate` affiché ; date de fin non passée — cours journalier saisi, mois futur, « Récupérer le mois » ;
  libellé du bouton et échec de la récupération ; tests Vitest à horloge fixée. Vérifié à la source le 2026-11-14 :
  `xmldaily?d=20251231` rend le 31.12.2025, `d=20251226` (férié) le 24.12.2025.)
- 2026-11-14 — validation CR-1 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche ; aucun
  finding inter-fiches. (LOW : sens de `overridden` et mode de comparaison du cours reçu au défaut non écrits avec la
  règle unique de genre. Vérifié aux fiches : `RateBoundsError`, `FX_RATE_INVALID` `details: { currency }`, fetch jour
  strictement antérieur / mois ≤ courant, `RATE_NOT_PUBLISHED`, `RATE_SOURCE_UNAVAILABLE` — cohérents avec 37-2 et 37-3a.
  Axes non exercés : relecture hors diff, code réel non encore écrit.)
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R9** — une écriture par compte réévalué (deux lignes,
  une devise, libellé « Gain de change / Perte de change <compte> <DEVISE> <cours> au JJ.MM.AAAA (non réalisé) »),
  toutes passées dans une transaction, réponse `{ entryIds }`, audit `fx_revaluation.posted` qui liste les écritures,
  rejeu compte par compte, bouton « Passer les écritures » ; disparaissent la ligne de contrepartie au net, l'agrégation
  et la borne des sommes de contrepartie ; la borne de 498 comptes est conservée, re-justifiée. **R6** — cours de
  clôture selon `closing_rate_kind` de l'exercice (`day` : jour, repli mensuel signalé ; `month` : dernier mois, sans
  repli), `closingRateKind` dans l'aperçu, genre relu sous verrou (409 si changé) ; tests d'exercices au 1er juillet et
  au 1er avril. **R3** — refus 409 `MULTI_CURRENCY_DISABLED` sur un exercice mono-devise (avant et dans la
  transaction), `blockers[]` pour un exercice postérieur mono-devise (garde de 37-3a). **R4** — terminologie et borne
  du mois suivant. AC touchés : en-tête, Story, AC-1, AC-2, AC-3, AC-4, AC-6, AC-7, AC-8, Tâches. Fiche à revalider
  (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW ; remédiés dans la fiche sauf
  37-5-CR2-P1-IF1 (LOW, 37-3a : place de `MULTI_CURRENCY_DISABLED` `laterFiscalYear` dans l'ordre des refus de
  37-3a AC-4, que 37-5 AC-4 renvoie à 37-3a), inter-fiches. (MEDIUM : la contre-passation d'une réévaluation est datée
  du jour ou de l'exercice suivant, hors du cumul de fin d'exercice — « supprimée ou contre-passée, puis nouveau
  passage » était faux pour la contre-passation. LOW : refus `laterFiscalYear` absent de l'ordre de l'AC-4 et non testé
  à la passation ; libellé construit à deux endroits (aperçu, `post`) ; `invoices.rs:698` est
  `repositories/invoices.rs` ; `overridden` homonyme de celui de 37-2.) Axes non exercés : GET de la source
  journalière (déjà vérifiée en CR-1 P2).
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW ; remédiés dans la fiche sauf
  37-5-CR2-P2-IF1 (LOW, 37-3a AC-7 : « suppression ou contre-passation » citée de 37-5 AC-3) et 37-5-CR2-P2-IF2 (LOW,
  37-3b : assertion `examinees` de `every_replayed_route_calls_an_envelope`), inter-fiches. (MEDIUM : la
  contre-passation datée du jour tombe **dans** le cumul quand l'exercice est en cours — `ReversalDating::Today`,
  `Utc::now().date_naive()` et `find_open_covering_date`, `journal_entries.rs:2933-2938` —, l'affirmation « hors du
  cumul » de la remédiation CR-2 P1 ne valait que pour un exercice dont la fin est passée : AC-3, test, manuel ; genre
  `month` (R6) non propagé — bouton du jour proposé et repli signalé sans condition de genre, défaut d'une devise omise
  dit « journalier, sinon mensuel » : AC-1, AC-2, AC-4, AC-6, Tâches, test. LOW : `laterFiscalYear` placé avant
  `LATER_FISCAL_YEAR_CLOSED` / `PERIOD_LOCKED`, alors que `create_in_tx_inner` évalue les dates (étapes 1, 1-a, 1-bis)
  avant les comptes (étape 2) et que 37-3a la met au rang 2 de son AC-4 ; `OPTIMISTIC_LOCK_CONFLICT` du genre placé
  avant les refus d'état (motif réel avant « rechargez ») ; renvoi « `details: {}` ci-dessus » sans objet ; décomptes
  du registre cumulés sans 37-3b, assertion `examinees` (27, `audit_route_registry.rs:1325`) omise.) Vérifié à la
  source (trois GET) : `xmldaily?d=20240630` rend la publication du 28.06.2024 (validité 29.06–01.07) ;
  `d=20240331` celle du 28.03.2024 (Vendredi saint sans publication) ; `d=20271231` rend un JSON
  « Aucune donnée disponible » (date future). Axes non exercés : code réel de 37-1 à 37-4 (non écrit).
- 2026-11-14 — validation CR-2 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés (AC-8 : « moyenne du 25 au 24 »
  retiré, R4 amendé ne décrivant la méthode qu'après confirmation ; « et non par contre-passation » nuancé selon l'AC-3
  révisé) ; inter-fiche signalé : 37-3b l. 188 (« dès le 25, le mois suivant se récupère ») contredit son l. 190 et 37-2 AC-5.
  Vérifié au code : ordre `create_in_tx_inner` (1-a, 1-bis, puis étape 2 des comptes), `ReversalDating::Today`
  (`journal_entries.rs:2933-2938`), `examinees` = 27 (`audit_route_registry.rs:1325`), 37-3b +1 route (conversion). Axes non
  exercés : code réel de 37-1 à 37-4 (non écrit), reste de la fiche hors diff.
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S2** — une écriture par compte réévalué, contrepartie
  sur les comptes de différence **propres** du compte s'il en a, sinon ceux de la société, par `fx_accounts_for`
  (37-1 AC-4 bis) : AC-1 (aperçu : `counterpartAccountId`, `fxAccountsOrigin`), AC-2 (étape (3) : désignations propres
  lues en partage, puis comptes employés revalidés par `validate_fx_accounts_in_tx` ; le partage de `companies` de
  l'étape (0) bloque une désignation concurrente ; contrepartie par compte ; audit), AC-4 (revalidation des seuls
  comptes **employés**, `holderAccountId`), AC-6 (origine affichée), AC-7 (tests), AC-8. **S4** — exercice de durée
  quelconque : dernier mois et date de fin tirés de `end_date` (AC-1), tests d'exercices de six et de dix-huit mois
  et d'un exercice qui finit un 14 (AC-7). S1, S3, S5, S6 sans point de contact (le cours AFC lu reste le cours en
  vigueur, 37-2 ; une republication non adoptée ne change rien ici). Aucune migration ni route neuve. AC touchés :
  en-tête, AC-1, AC-2, AC-4, AC-6, AC-7, AC-8. Fiche à revalider (cycle CR-3).
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches (le cycle porte ici le nom CR-3, le prompt disait « CR-2 P1 » par recopie). (MEDIUM : le
  `SELECT` de l'étape (3) ne lisait ni `number` ni `currency`, alors que `fx_accounts_for` et l'ordre des refus les
  exigent, et les comptes « employés » incluaient des comptes sélectionnés non admissibles, d'où un
  `FX_ACCOUNT_INVALID` avant `notEligible` ; cas `None` de `fx_accounts_for` non traité (aperçu, passation).
  LOW : tests (aperçu en `blockers[]`, non admissible, D = 0, `None`, ordre des fautifs) ; clés i18n des libellés
  d'origine absentes des Tâches ; parenthèse de l'étape (0) coupant la phrase ; « aucun en exclusif » à borner à ce
  flux.) Vérifié : 37-1 AC-4 bis (verrous, `validate_fx_accounts_in_tx`, `reason` `missing`), epic C-37-7. Axes
  non exercés : GET de la source journalière (aucune date de la révision ne l'exigeait), code réel de 37-1 (non écrit).
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : `validate_fx_accounts_in_tx` (37-1) rend son refus dès qu'elle le constate, alors
  que l'étape (3) l'appelle avant que l'admissibilité de l'étape (4) soit connue — le défaut que la P1 corrigeait
  revenait à l'implémentation : résultat gardé par compte, rendu après (4) pour les seuls admissibles ; blockers
  `FX_ACCOUNT_INVALID` par compte depuis S2 sans lien écrit à la sélection, et ligne `None` dite « comme une ligne non
  sélectionnable » — un compte dormant décoché aurait laissé « Passer les écritures » désactivé : blockers sur la
  sélection, relecture de l'aperçu à chaque case, ligne cochable, test. LOW : `fx_accounts_for` prend un `Account`
  (37-1 AC-4 bis, signature d'exemple), que la projection de l'étape (3) ne suffit pas à construire.) A17 sans point
  de contact (37-3a, 37-3b). Vérifié : 37-1 AC-4 bis (signature, `None`, `holderAccountId`, verrous `companies`
  `FOR UPDATE` des désignations, pas de cycle avec 37-5), `companies.rs:1002-1020`, `invoices.rs:698` ; source
  journalière (un GET) : `xmldaily?d=20250314` rend la publication du 14.03.2025 (exercice qui finit un 14, AC-7).
  Axes non exercés : code réel de 37-1 à 37-4 (non écrit) ; registres d'audit et RBAC non touchés par la révision
  (vérifiés aux passes antérieures).
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW ; remédié dans la fiche ; aucun
  finding inter-fiches. (MEDIUM : la relecture de l'aperçu « avec `accountIds` » à chaque case, ajoutée en P2, perdait
  les cours modifiés (AC-6 : valables pour la passation) et, à sélection vide, rendait 400 `empty` ; `rates` d'une devise
  sans compte coché aurait rendu `FX_RATE_INVALID` : cours joints et limités aux devises cochées, pas de relecture à
  sélection vide, test.) Vérifié : lignes 286-296 (`empty`, `notEligible`), 37-1 AC-4 bis (signature, `None`).
  Axes non exercés : code réel de 37-1 à 37-4 (non écrit) ; passages hors diff d9c0018e..040185ac.
- 2026-11-14 — validation CR-3 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 0 LOW ; remédié dans la fiche ; aucun
  finding inter-fiches. (37-5-CR3-P4-1 MEDIUM : la règle de relecture (`accountIds` + `rates` des devises cochées,
  dernière réponse seule) n'était posée qu'au déclencheur « case » ; la relecture après récupération d'un cours (AC-1)
  ou après un cours modifié repartait sur la présélection, d'où le blocker du compte décoché et un bouton de nouveau
  désactivé, et pouvait croiser une relecture de case : règle étendue à **toute** relecture, AC-1, test.
  37-5-CR3-P4-2 MEDIUM : « la passation reçoit tous les cours affichés » (AC-1) contredisait la borne de l'AC-4 (clé =
  devise d'au moins un compte sélectionné) dès qu'une devise n'a plus aucun compte coché → `FX_RATE_INVALID` :
  limité aux devises cochées, test.) Vérifié : AC-4 « Bornes de la requête » (clé = devise d'un compte sélectionné),
  `reason: "empty"`, AC-1 « L'écran n'envoie à l'aperçu que les cours modifiés » ; aucune autre fiche ni l'epic ne
  nomment `accountIds` ou la route de réévaluation (grep). Axes non exercés : code réel de 37-1 à 37-4 (non écrit) ;
  passages hors diff 040185ac..53a885f4, sauf AC-1 (cours envoyés) relu pour cohérence.
- 2026-11-14 — validation CR-3 P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche ; aucun
  finding inter-fiches. (37-5-CR3-P5-1 LOW : « puis relit l'aperçu » après récupération (AC-1) restait inconditionnel
  face à « pas de relecture à sélection vide » : renvoi à la règle ajouté.) Vérifié : AC-1 (règle de relecture, cours
  envoyés), AC-2 (cours omis = défaut), AC-4 « Bornes de la requête » (clé = devise d'un compte sélectionné, `empty`),
  ligne de test du décochage. Axes non exercés : code réel de 37-1 à 37-4 (non écrit) ; passages hors diff
  53a885f4..644d2b74.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T3** — la ligne `revaluation` ne porte plus de
  colonne de devise (37-3a AC-1) : `ForeignAmount { declared_currency: None, … }`, devise lue sur le compte réévalué,
  contrôle central par la devise du compte (`FX_ACCOUNT_IN_CHF` pour un compte CHF ; `FX_CURRENCY_MISMATCH` sans
  objet), `devise_net` sur toutes les lignes du compte (AC-1) ; audit au snapshot `foreign` (devise lue sur le compte)
  (AC-7). **T2** — exercice au 1er janvier ajouté aux tests du genre de clôture, à côté du 1er juillet, du 1er avril
  et des durées de six et dix-huit mois (tests). T4 sans effet ici (cours de **clôture** ; la reprise du genre de
  clôture par l'exercice suivant est en 37-1 AC-3 bis). T1, T5, T6 sans point de contact. AC touchés : AC-1 (règle de
  calcul), AC-2 (ligne du compte réévalué), AC-7 (tests). Fiche à revalider (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW ; remédiés dans la fiche ; aucun
  finding inter-fiches. (37-5-CR4-P1-1 LOW : en-tête sans la révision CR-4. -2 LOW : ligne d'aperçu sans `currency`
  alors que T3 fait lire la devise sur le compte et que l'écran en a besoin pour les cours cochés. -3 LOW : refus du
  rang 3 de 37-3a (`FX_ACCOUNT_IN_CHF`, `FX_CURRENCY_MISMATCH`) non situés pour la ligne de réévaluation : inatteignables
  sous verrou, dits filet. -4 LOW : test T3 manquant — devise relue sur le compte, cumul avec écriture EUR–EUR.)
  Vérifié : 37-3a AC-1/AC-2/AC-4 rang 3/AC-8 (`declared_currency`, `account_currency`, snapshot `foreign`), 37-4 AC-1
  quater (`balances_as_of`, `Option<devise_net>`), epic C-37-3 amendé, A20/A21 ; aucun reste de colonne de devise de
  ligne dans la fiche (grep `currency`). Source journalière (un GET) : `xmldaily?d=20241231` rend la publication du
  31.12.2024 (EUR 0.94947), mardi. Axes non exercés : code réel de 37-1 à 37-4 (non écrit) ; T1, T4, T5, T6 sans point de
  contact ; registres d'audit et RBAC non touchés par la révision.
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — le cours de clôture par défaut se lit dans
  `exchange_rates` de 37-2 (cours retenu par la société, `find_company_rate_in_tx`, ex-`find_effective_rate_in_tx`),
  jamais dans la valeur publiée : une correction de l'AFC non reprise ne change pas le cours proposé (AC-1) ; l'aperçu
  rend `defaultRate` et `defaultSource` au lieu d'`afcRate` (AC-1) ; la source auditée par devise est celle de la ligne
  de la société (`afc` | `manual`) quand le cours retenu l'égale, `manual` sinon — gardée pour toujours, même après
  correction ou restauration (AC-2) ; écran : source affichée « AFC », « cours saisi » ou « manuel », la même que
  l'audit (AC-6) ; tests : correction non reprise, après ↑, restauration sans récupération, ligne `manual` égale à la
  valeur publiée (AC-7). U2 à U6 sans point de contact (aucune édition de ligne ici ; aucune phrase générale « débit
  et crédit en CHF »). Aucune migration ni route neuve. AC touchés : en-tête, AC-1, AC-2, AC-6, AC-7. Fiche à
  revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW ; remédiés dans la fiche ; aucun finding inter-fiches. (MEDIUM 37-5-CR5-P1-1 : `sourceDate` d'une valeur publiée affichée sur un cours `manual` — `null` sauf `defaultSource` `afc`, test ; fonction de lecture nommée. LOW : -2 AC-1, mois courant ou suivant d'une fin d'exercice non passée existent par copie, non seulement saisis ; -3 limite d'un cours retenu changé entre aperçu et passation (genre `request`, source `manual`) ; -4 test de restauration sans appel sortant, `sourceDate` `null`, et manuel AC-8. Vérifié : plus aucun reste de `exchange_rate_overrides`, `_kesh_exchange_rates`, `republished_*`, `find_effective_rate_in_tx` hors historiques des six fiches ; `lookup`/`find_company_rate_in_tx`/`find_fetched_rate_in_tx` de 37-2 AC-5 ; `for_actor` `audit_log.rs:195`, `"fiscal_year"` `audit_labels.rs:40`, `retry_on_deadlock` lus. A23–A29 : sans point de contact ici. Axes non exercés : GET de la source (aucun point de contact), code réel non écrit.)
- 2026-11-14 — validation CR-5 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche ; aucun finding inter-fiches. (LOW 37-5-CR5-P2-1 : `sourceDate` d'un défaut mensuel non dit `null` — explicité, renvoi à 37-2 AC-5 « `sourceDate` à `null` hors `day` ». Vérifié au diff `d59a1de9..3b058509` : `find_fetched_rate_in_tx` (37-2 AC-5, affichage seulement) ; copie du mois courant ou suivant sans borne de date (37-2 AC-3 bis), mois au-delà jamais publié (bornes du `POST /fetch`) ; `_kesh_fetched_exchange_rates` absente du `.keshbackup` (37-2, table système) et tâche et connexion lancées par `main.rs` seul, hors mode test ; limite P1-3 cohérente avec la règle unique de genre et de source. Alignement 37-2 (copie à l'activation) : sans point de contact ici. Axes non exercés : code réel non écrit, reste de la fiche hors diff.)
- 2026-11-14 — alignement A30 (orchestrateur, Sonnet) : une `source` de cours inconnue (`RateOrigin::Other(String)`, p. ex. `ecb`, 37-2) est affichée et auditée telle quelle (valeur brute, AC-1, AC-6) mais se traite comme `manual` : `sourceDate` `null`, même défaut ; test ajouté (ligne `ecb` → cours proposé, `sourceDate` `null`, `defaultSource` `"ecb"`). Noms repris de 37-2 (`RateOrigin`, `find_company_rate_in_tx`, `sourceDate`).
- 2026-11-14 — validation CR-5 P5 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche.
  MEDIUM 37-5-CR5-P5-1 (AC-6) la source affichée n'énumérait que « AFC » et « cours saisi » — « cours saisi (<source
  brute>) » pour toute autre source, comme 37-3b AC-2, Vitest. LOW -2 (AC-2, source à l'audit) « `defaultSource` :
  `afc` ou `manual` » contredisait la valeur brute de l'AC-1 et de l'audit — aligné. Vérifié : `sourceDate` `null`
  hors `afc`, test A30, audit `fx_revaluation.posted` à la valeur brute ; genre (`request` si différent) indépendant
  de la source. Non exercés : reste de la fiche hors diff, rendu.
- 2026-11-14 — validation CR-5 P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune modification. Phrases A30 du diff P5 vérifiées contre 37-2 (AC-1, AC-5, AC-6) ; aucun site restant. Non exercés : reste de la fiche hors diff, compilation, rendu.
