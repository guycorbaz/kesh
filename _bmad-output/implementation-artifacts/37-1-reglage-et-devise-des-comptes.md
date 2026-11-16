# Story 37-1 : réglage multi-devises de la société et devise des comptes

Status: review

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/37-multi-devises`, numéros de ligne de `67efc51f`. Epic 37
(`_bmad-output/planning-artifacts/epic-37-multi-devises.md`), registre C-37-1 à C-37-8. **Migration non breaking**.
Révisée le 2026-11-14 pour la revue de Beat (CR-2 : R1, R3, R6 — colonnes seulement —, R7, R10), qui prime sur le
registre antérieur ; puis pour sa revue CR-3 (S2 : comptes de différence de change propres à un compte en devise ;
S3 : liste des comptes que l'avertissement d'archivage archiverait ; S4 : R10 et l'arbitrage A1 retirés), qui prime
sur CR-2 ; puis pour sa revue 2 (CR-4 : **T1** — l'assistant de démarrage demande les dates du premier exercice,
AC-11 rétablie avec ce contenu ; **T3** — la devise d'une ligne se lit sur le compte, le verrou de l'AC-4 en devient
la garantie ; **T4** — sens du genre d'ouverture et son défaut à la création), qui primait sur tout ce qui
précède ; puis pour sa revue 3 (CR-5), qui prime sur tout : **U1** — la table des cours de la société s'appelle
`exchange_rates` (37-2), et la création d'un exercice multi-devises (la bascule vers multi-devises, retirée par CR-7)
et une devise donnée à un compte (création, ou changement sur un compte sans ligne) y copient les cours déjà récupérés des devises des comptes (A24 ; AC-3, AC-3 bis, AC-4,
AC-6) ; **U4** — changer le genre de clôture propose de l'appliquer à
l'ouverture de l'exercice suivant (AC-3 bis, renvoi à 37-2 AC-5) ; **U5** — les arbitrages approuvés de cette fiche
sont des AC (Décisions).*

## Révision CR-7 (2026-11-16 — A34, A35, A36 ; prime sur tout ce qui suit)

**Modèle retenu (A34 : le multi-devises, une fois activé, reste activé).**

- `companies.multi_currency_enabled` est **irréversible** : il passe de `FALSE` à `TRUE`, jamais l'inverse (seule la
  réinitialisation de la société, AC-5, le remet à `FALSE`, en vidant du même coup comptes, exercices et écritures).
  L'écran demande une **confirmation** avant d'activer (dialogue : « irréversible », AC-7). Un `PUT
  …/currency-settings` qui porte `multiCurrencyEnabled: false` quand le mode est actif → 409
  **`MULTI_CURRENCY_IRREVERSIBLE`** (code neuf, `details: {}`, quatre locales ; il prend le rang de l'ancien
  `MULTI_CURRENCY_IN_USE`, juste après le no-op). Les trois comptes restent modifiables, jamais vidables (`missing`).
- `fiscal_years.multi_currency` est **gardée** (migration de l'AC-1 et liste close de l'AC-6 inchangées) mais ne
  s'écrit plus qu'en deux endroits : à la **création** (réglage de la société, AC-3 bis) et à l'**activation** (tous
  les exercices **ouverts** → `TRUE`, AC-3) ; elle ne repasse jamais à `FALSE` et aucune route ne la change ensuite.
  *Pourquoi la colonne plutôt qu'une règle de date* : elle est la seule trace des exercices déjà clos à l'activation
  (une règle « fin postérieure à l'activation » exigerait une date d'activation et un calcul à chaque contrôle), et elle
  garde fermé aux devises un exercice clos avant l'activation puis rouvert.
- Conséquence (invariant I de `fiscal_years::close` : aucun exercice ouvert avant un exercice clos ; et
  `fiscal_years::create` refuse un exercice qui précède un exercice **clos**, `LaterFiscalYearClosed`,
  `repositories/fiscal_years.rs:363-380`) : à l'activation, les exercices clos précèdent tous les ouverts ; tout
  exercice créé ensuite est multi-devises et ne se place jamais avant un exercice clos. **D-CR7-2 (KISS, passe CR-7
  P2 — remplace la « limite assumée » d'un exercice créé dans un trou devant un exercice clos avant l'activation puis
  rouvert)** : `fiscal_years::create` refuse aussi, dans une société multi-devises, un exercice qui précède un exercice
  **mono-devise** (400 `LATER_FISCAL_YEAR_MONO_CURRENCY`, garde voisine de `LaterFiscalYearClosed`, AC-3 bis). Les
  exercices mono-devise forment donc toujours un préfixe de l'ordre chronologique : aucun exercice multi-devises ne
  précède un exercice mono-devise, et l'exercice suivant d'un exercice multi-devises l'est aussi. La garde
  `laterFiscalYear` de 37-3a n'a plus d'objet.

**Retiré** (le texte qui suit a été réécrit ; ce qui en subsiste dans les Change Logs est historique) :

- la route `PUT /api/v1/fiscal-years/{id}/currency-mode` (`fiscal_years::set_currency_mode`, `force`, audit
  `fiscal_year.currency_mode_updated`, son entrée au registre des routes, son rejeu et son test de victime) ;
- les codes `MULTI_CURRENCY_IN_USE`, `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` ;
- la désactivation de la société (comptes remis à `NULL`, exercices repassés à `FALSE`) et l'arbitrage A27 ;
- l'archivage forcé et l'extraction d'`archive_in_tx` ;
- le refus de `reactivate` d'un compte en devise en mono-devise (aucun compte en devise n'existe en mono-devise) ;
- la colonne « Devises », l'action de bascule et le dialogue d'archivage de l'écran *Exercices* ;
- la copie des cours « au passage d'un exercice en multi-devises » (37-2 AC-3 bis) ;
- les arbitrages A4 et A5, sauf « l'activation passe tous les exercices ouverts en multi-devises », gardé.

**Registre des routes** : la 37-1 compte **+1 route, +1 tracée, total +1, `sans_ecriture` +1, `tout.len()` +1** ; le
point (vi) gagne **+2** routes rejouées (`currency-settings`, `accounts::create_account`) : « vingt-sept ».
**A35** (sous-comptes) : aucun travail. **A36** (KISS) : appliqué ci-dessus.

## Story

En tant que comptable d'une société qui encaisse et paie en USD et en EUR, je veux passer ma comptabilité en mode
multi-devises, désigner mes comptes de différences de change et donner une devise à certains comptes du plan, afin d'y
passer ensuite des écritures en devise (37-3), sans rien changer pour qui reste en mono-devise.

## Décisions qui s'appliquent

D1, D2 (fichier d'epic) ; C-37-2 (réglage sur `companies`, sans FK, quatre gardes), C-37-5 (verrous), C-37-9 (409 partout,
verrou des deux côtés, réglage réservé à l'Administrateur). **CR-2 (revue de Beat, prime)** : R1 (une sauvegarde
antérieure à l'Epic 37 se réimporte : liste close des tables et colonnes de l'epic, AC-6) ; R3 (mode mono-devise ou
multi-devises **par exercice**, posé à la création et à l'activation, jamais changé ensuite depuis CR-7, AC-3 bis) ; R6 (colonnes
`opening_rate_kind`/`closing_rate_kind` seulement, AC-1 ; route et écran en 37-2) ; R7 (compte de conversion en
CHF désigné à l'activation, AC-3). **CR-3 (revue de Beat, prime sur CR-2)** : S2 (comptes de différence de
change **propres** à un compte en devise, le réglage de la société restant le défaut : AC-1, AC-4 bis, AC-6, AC-7) ;
S3 (liste des comptes de l'avertissement d'archivage — *retiré par CR-7*) ; S4 (R10 et l'arbitrage A1 **retirés** :
pas de constante). **CR-4 (revue 2 de Beat, prime sur tout)** : T1 (l'assistant demande les dates du premier
exercice, AC-11) ; T3 (la ligne d'écriture ne stocke pas de devise : le verrou de la devise d'un compte, AC-4, porte
celle de ses lignes) ; T4 (genre d'ouverture = clôture de l'exercice précédent, défaut repris à la création, AC-1,
AC-3 bis). **CR-5 (revue 3 de Beat, prime sur tout)** : U1 (copie des cours récupérés dans `exchange_rates` de 37-2 à
la création d'un exercice multi-devises et quand un compte reçoit une devise (la bascule vers multi-devises est
retirée, CR-7) — pas à
l'activation, vide par construction : appels ajoutés par 37-2 AC-3 bis, AC-3 bis et AC-4 ci-dessous, l'AC-3 disant qu'il n'y en a aucun ; liste close de l'AC-6 sous le nom neuf) ; U4 (remplace A22 : AC-3 bis, la
route vit en 37-2 AC-5). **U5 — arbitrages approuvés par Beat (CR-5), écrits comme AC de cette fiche** : A4 (*retiré par CR-7* : la ligne en devise
datée avant un exercice mono-devise n'a plus d'objet ; reste le refus sur un exercice mono-devise, AC-3 bis, puce « dans
un exercice mono-devise », 37-3a AC-4 rang 2) ; A5 (*réduit par CR-7* à « l'activation passe tous les exercices
ouverts en multi-devises » : AC-3, AC-3 bis ; `force` et la bascule sont retirés) ; A6 (compte de conversion
de bilan, non archivé, à créer, p. ex. 1090 : AC-3, AC-10) ; A15 (les comptes propres couvrent gains et pertes, le
compte de conversion reste un réglage de la société : AC-4 bis) ; A24 (copie limitée aux devises des comptes de la
société, déclenchée aussi quand un compte reçoit une devise : AC-3, AC-3 bis, AC-4) ; A27 (*retiré par CR-7* : plus de
désactivation) ; A18 et A19 (fin du premier exercice saisie dans
l'assistant, nom « Exercice AAAA » ou « Exercice AAAA/AAAA » : AC-11). A22 est remplacé par U4 ; A1 était déjà retiré
(S4). **CR-7 (revue de Guy, prime sur tout)** : A34 (multi-devises sans retour, révision ci-dessus — retire de R3 la
bascule par exercice, de S3 l'avertissement, d'A4/A5 la bascule et `force`, A27) ; A35 (aucun travail) ; A36 (KISS).

## Exigences

*(B26, 2026-11-15 : tirées des critères d'acceptation ci-dessous, sans en créer ni en modifier.)*

**Fonctionnelles**

- FR-37-1-1 — La migration ajoute, sans écrire de données, les réglages de la société, le mode et les genres de cours de l'exercice, et la devise et les comptes de différence du compte, sans clé étrangère sur les comptes de différence ; le schéma refuse une devise hors `[A-Z]{3}` ou égale à `CHF` (la devise principale s'écrit `NULL`), des comptes propres dépareillés ou posés sur un compte CHF (AC-1).
- FR-37-1-2 — `GET /companies/current` expose `multiCurrencyEnabled`, les trois comptes FX et `baseCurrency: "CHF"` ; les exercices (`GET /fiscal-years`, détail) exposent `multiCurrency`, `openingRateKind`, `closingRateKind`, toujours présents ; l'outil MCP `kesh_exercices` gagne `multi_devises`, `genre_ouverture` et `genre_cloture` (types scellés, mono-devise compris) (AC-2).
- FR-37-1-3 — `PUT /companies/current/currency-settings` écrit le réglage avec verrou optimiste (409) : l'activation exige les trois comptes (gains, pertes, conversion) et passe tous les exercices ouverts en multi-devises ; chaque compte **changé** passe les gardes — de la société, imputable, actif, sans devise, `Expense`/`Revenue`, ni rôle `DefaultRevenue` ni désigné compte de produit par défaut ; le compte de conversion, lui, est de type `Asset`/`Liability` et n'est visé ni par le refus du rôle ni par celui du compte de produit — sous peine de 400 `FX_ACCOUNT_INVALID` (liste close des `reason`) ; l'activation est irréversible : `multiCurrencyEnabled: false` sur une société multi-devises est refusé (409 `MULTI_CURRENCY_IRREVERSIBLE`) ; un no-op ne change pas la `version` et n'est pas audité ; aucune copie de cours à l'activation (AC-3).
- FR-37-1-4 — Chaque exercice est mono-devise ou multi-devises : il prend le réglage de la société à sa création (trois chemins) et passe multi-devises à l'activation s'il est ouvert ; aucune route ne change ensuite son mode (CR-7) ; dans une société multi-devises, la création d'un exercice avant un exercice mono-devise est refusée (400 `LATER_FISCAL_YEAR_MONO_CURRENCY`, D-CR7-2) ; un exercice mono-devise (clos avant l'activation) n'accepte aucune ligne sur un compte en devise (37-3a). Son genre d'ouverture reprend le genre de clôture de l'exercice qui le précède sans trou (`day` sinon), et son genre de clôture est `day` (AC-3 bis).
- FR-37-1-5 — La devise d'un compte est facultative (absent ou `null` = CHF ; à la mise à jour : absent = inchangée, `null` = CHF), normalisée (`CHF` → `NULL` avant tout contrôle), refusée en mono-devise (409 `MULTI_CURRENCY_DISABLED`), hors ISO 4217 (400 `CURRENCY_UNKNOWN`), et se verrouille dès la première ligne, sans exception ni confirmation (409 `ACCOUNT_CURRENCY_LOCKED`) ; un compte que les chemins automatiques débitent en CHF (rôle, désignation de facturation, compte bancaire, article, règle de réconciliation) ne reçoit pas de devise (409 `ACCOUNT_CURRENCY_IN_USE`), et un compte en devise n'est pas désignable à ces sites (400 `ACCOUNT_IN_FOREIGN_CURRENCY`) ; `AccountResponse` porte `currency` et `currencyLocked` (AC-4).
- FR-37-1-6 — Un compte en devise suit les comptes de différence de la société ou porte les siens (paire gains/pertes, à trois états à l'API), résolus par une seule fonction serveur et son miroir TypeScript et revalidés à l'usage ; les comptes propres changés passent les gardes de l'AC-3 (400 `FX_ACCOUNT_INVALID`, dont `missing` pour un seul champ et `unexpected` sur un compte qui reste ou devient CHF) ; le compte de conversion reste un réglage de la société seul (AC-4 bis).
- FR-37-1-7 — La réinitialisation de la société remet à zéro les quatre colonnes du réglage (AC-5).
- FR-37-1-8 — Un `.keshbackup` de la 0.17.6 se réimporte, l'import ne tolérant que la liste close des absences de l'Epic 37 (toute autre table absente reste refusée, fenêtre d'import inchangée) ; l'aller-retour conserve réglage, devises, comptes propres et mode des exercices, et les exports CSV de la société, des comptes et des exercices portent les colonnes neuves (AC-6).
- FR-37-1-9 — L'écran *Paramètres → Comptabilité* règle le mode et les trois comptes — l'activation demande une confirmation (irréversible) et la case ne se décoche plus ensuite —, éditable pour l'Administrateur, en lecture seule pour les autres rôles ; le *Plan comptable* offre, en multi-devises, la devise d'un compte (désactivée avec l'explication si verrouillée), sa colonne, et pour un compte en devise le choix « par défaut / propres » des comptes de différence ; en mono-devise, aucun changement visible (AC-7).
- FR-37-1-10 — `kesh_plan_comptable` donne la devise d'un compte (`null` pour le CHF) (AC-8).
- FR-37-1-11 — L'assistant de démarrage fixe les dates du premier exercice : défaut 1er janvier de l'année courante, fin la veille du même jour un an plus tard ; fin ≤ début ou date hors bornes → 400, rien de créé ; nom « Exercice AAAA » ou « Exercice AAAA/AAAA » ; finalisation rejouée sans effet ; exercice créé mono-devise ; aucune borne de durée (six ou dix-huit mois admis) ; dates non modifiables ensuite ; à l'écran de la dernière étape, une section « Premier exercice » (début et fin en forme suisse, fin recalculée tant qu'elle n'est pas retouchée, boutons désactivés si fin ≤ début) dont les deux boutons envoient les dates (AC-11).
- FR-37-1-12 — Un compte désigné gains ou pertes — par la société ou par un compte en devise, archivé compris — ou conversion — par la société seule — ne peut recevoir de devise, être archivé, changer de type (hors `Expense`/`Revenue`, ou `Asset`/`Liability` pour la conversion), passer non imputable, ni recevoir un sous-compte ; les comptes de gains et de pertes ne peuvent en outre recevoir le rôle `DefaultRevenue` ni être désignés compte de produit par défaut (réglages de facturation, article) ; le refus de désignation « produit par défaut » ne vise pas le compte de conversion, et le rôle `DefaultRevenue` lui est déjà fermé par son type (`Asset`/`Liability`) (409 `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `details.designation` `gain`|`loss`|`transit`, plus `holderAccountId` et `holderNumber` pour une désignation propre) ; l'archivage du compte en devise qui désigne reste admis (AC-4, AC-4 bis).

**Non fonctionnelles**

- NFR-37-1-1 — La migration est non breaking : DDL seul, aucune écriture de données (AC-1).
- NFR-37-1-2 — RBAC : le `PUT` du réglage est réservé à l'Administrateur en session (cookie, ou `Authorization: Bearer` après `POST /auth/login`), refusé au Comptable (403) et à une clé d'API (`API_KEY_ADMIN_FORBIDDEN`) (AC-3).
- NFR-37-1-3 — Aucune régression mono-devise : un client qui ne connaît pas `currency` ne reçoit pas de 400, et le JSON des comptes est identique en mono-devise et pour un compte « par défaut » (`currency`, `currencyLocked`, `fxGainAccountId`, `fxLossAccountId` absents quand vides, C-37-9) (AC-4, AC-4 bis).
- NFR-37-1-4 — Sauvegarde et restauration : l'aller-retour d'une sauvegarde 37-1 ne perd ni le réglage, ni les devises, ni les comptes propres, ni le mode des exercices ; une colonne de l'epic ne peut devenir obligatoire sans faire rougir un test (AC-5, AC-6).
- NFR-37-1-5 — Sorties MCP : `kesh_plan_comptable` re-mesurée avec le champ `devise` ; `kesh_exercices`, non paginé, mesuré par un test dédié, le nombre d'exercices qui tient sous `MCP_ENVOI_OCTETS_MAX` écrit au Change Log ; jamais de `String` dans `sortie!` (AC-2, AC-8).
- NFR-37-1-6 — Chaînes neuves dans les quatre locales, `lint-i18n-ownership` vert (AC-9).
- NFR-37-1-7 — Manuel utilisateur et manuel d'administration (activation par session, jamais par clé d'API ; matrice MCP de `kesh_exercices`) à jour (`.tex` seulement ; PDF non commités, régénérés à la release, contrôle du rendu possible en local) ; Note de version (AC-10).
- NFR-37-1-8 — Audit : `company.currency_settings_updated` (avant/après, exercices changés), comptes propres dans l'instantané d'un compte ; action à `ACTIONS`, libellés dans les quatre locales, route au registre (`Traced, SansEcritureAuJournal`) (AC-3, AC-4 bis).
- NFR-37-1-9 — Concurrence : verrous dans l'ordre global du dépôt (`companies` → exercices → réglages → `accounts`), lectures verrouillantes sans lecture simple avant ; `currency-settings` et `accounts::create` rejoués sur interblocage, un test de victime par route, et un pour `accounts::update` (enveloppe existante conservée, cas d'un compte CHF rendu en devise avec comptes propres) (AC-3, AC-3 bis, AC-4, AC-4 bis).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000001_multi_currency_settings.sql`, non breaking (DDL seul, aucune écriture de
   données) :
   - `companies` : `multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE`, `fx_gain_account_id BIGINT NULL`,
     `fx_loss_account_id BIGINT NULL`, `fx_transit_account_id BIGINT NULL` (compte de conversion, R7), **sans clé
     étrangère** (commentaire SQL qui dit pourquoi : cycle `companies` ↔ `accounts`, réinitialisation ; intégrité par le
     dépôt, C-37-2) ;
   - `fiscal_years` (R3, R6) : `multi_currency BOOLEAN NOT NULL DEFAULT FALSE` (mode de l'exercice, AC-3 bis),
     `opening_rate_kind ENUM('day','month') NOT NULL DEFAULT 'day'` et `closing_rate_kind ENUM('day','month') NOT NULL
     DEFAULT 'day'` (genre du cours d'ouverture et de clôture de l'exercice, R6 : lus par 37-3b et 37-5, réglés à
     l'écran *Cours de change* de 37-2). **Sens (T4, CR-4 : cours d'ouverture = cours de clôture de l'exercice
     précédent, pour que les valeurs CHF concordent d'un exercice à l'autre)** : ouverture `day` = cours journalier de
     la **veille** de `start_date` (le jour de clôture de l'exercice précédent) ; ouverture `month` = cours mensuel du
     **dernier mois de l'exercice précédent**, c'est-à-dire du mois de la veille de `start_date` (et non plus du premier
     mois du nouvel exercice) ; clôture `day` = cours journalier de `end_date` ; clôture `month` = cours mensuel du mois
     de `end_date`. Rappel : les soldes de bilan sont cumulés d'un exercice à l'autre, sans écriture de report ; le
     cours d'ouverture ne sert qu'aux soldes d'ouverture d'une société neuve (37-3b) et à l'affichage (37-2). **Décision CR-2** : les trois colonnes d'exercice sont dans **cette** migration,
     non dans celle de 37-2 — l'entité `FiscalYear` et ses quinze listes de colonnes écrites à la main (Tâches) ne se
     touchent qu'une fois, et le test de montée de `fiscal_years` est unique ; 37-2 n'écrit que la route et l'écran qui
     les règlent. Les exercices existants prennent `FALSE`, `'day'`, `'day'` (une base 0.17.6 est mono-devise) ;
   - `accounts` : `currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NULL` (NULL = devise principale),
     `CHECK (currency IS NULL OR BINARY currency REGEXP '^[A-Z]{3}$')` et
     `CHECK (currency IS NULL OR BINARY currency <> BINARY 'CHF')` (la devise principale s'écrit NULL, une seule
     graphie). ⚠️ La table est en `utf8mb4_unicode_ci` (`20260411000001_accounts.sql:23`) : sans collation binaire,
     `REGEXP` ignore la casse et `usd` passerait. Test : un `INSERT` de `usd` est refusé par le schéma.
   - `accounts` (**CR-3, S2**) : `fx_gain_account_id BIGINT NULL`, `fx_loss_account_id BIGINT NULL` — comptes de
     différence de change **propres** au compte en devise ; `NULL` = suivre le réglage de la société (`companies`,
     ci-dessus), lu **à l'usage** : un changement du réglage de la société s'applique donc d'office aux comptes « par
     défaut ». **Sans clé étrangère** (commentaire SQL, même raison que `companies` — C-37-2 : relation logique vers
     `accounts.id` de la même société, intégrité par le dépôt ; la remise à zéro vide `accounts` entière, l'import
     s'exécute sans contrôle de clé, `backup.rs:689-736`). Contraintes nommées, filet du dépôt :
     `chk_accounts_fx_own_pair` `CHECK ((fx_gain_account_id IS NULL) = (fx_loss_account_id IS NULL))` (les deux, ou
     aucun : « par défaut » ou « propres ») et `chk_accounts_fx_own_foreign` `CHECK (fx_gain_account_id IS NULL OR
     currency IS NOT NULL)` (posables seulement sur un compte en devise). Les deux peuvent désigner le même compte.
     **Deux index** `idx_accounts_fx_gain (company_id, fx_gain_account_id)` et `idx_accounts_fx_loss (company_id,
     fx_loss_account_id)` (37-1-CR3-P1-1 : sans eux, la recherche du compte qui désigne, AC-4, balaie et verrouille
     toutes les lignes `accounts` de la société). Test : un `INSERT` qui viole chacune est refusé par **sa** contrainte ; les comptes existants prennent `NULL`.
   - Les colonnes des lignes d'écriture **ne sont pas** dans cette migration (37-3).
   - Inscrite à `migrations.sha384`, au tableau de `docs/migrations-idempotence-audit.md` (compteurs recomptés depuis la source, P5, et
     recomptés après chaque reprise de `main`),
     squash régénéré (`scripts/regen-test-schema.sh`), P6 inspecté, P7 : aucune donnée écrite (le test de triage passe
     sans entrée).
   - `kesh_version_min_required` inchangé (0.17.6).
   - Test de montée sur base peuplée (`migrations_upgrade_path.rs`, précédent de la 34-11a1) : comptes, société et
     exercices existants prennent les défauts.
2. **AC-2 — Lecture du réglage.** `GET /api/v1/companies/current` expose `multiCurrencyEnabled`, `fxGainAccountId`,
   `fxLossAccountId`, `fxTransitAccountId`, `baseCurrency: "CHF"` (constante, C-37-2 ; la devise principale ne change
   pas). `GET /api/v1/fiscal-years` et le détail d'un exercice (`FiscalYearResponse`, `routes/fiscal_years.rs:51`, et
   `services/exercices.rs`) exposent `multiCurrency`, `openingRateKind`, `closingRateKind` — toujours présents, comme
   les champs du réglage de `CompanyJson` (ce ne sont pas des champs en devise, C-37-9). L'outil MCP `kesh_exercices`
   **est étendu** (V10, CR-6 — l'ancienne justification, « sa sortie ne lit pas ces champs », était circulaire : c'est
   précisément la sortie qu'il s'agit d'étendre) : `sortie! struct Exercice` (`mcp/outils/exercices.rs:23-29`) gagne
   `multi_devises: bool`, `genre_ouverture` et `genre_cloture` (énumération des genres de l'AC-1, `RateKindChoice` / nom
   au développement, rendue en type **scellé** comme `statut: kesh_db::entities::FiscalYearStatus`, `:28` — jamais
   `String`, qui ne compile pas, `mcp/sortie.rs:92` ; **l'énumération s'inscrit à `champ_enum!`**, `mcp/sortie.rs:131-143`
   — sans cela `sortie!` ne compile pas : elle doit être sans champ (`x as isize`), `Serialize`, et ses valeurs écrites
   `day`/`month`, celles du HTTP), posés à la construction (`:69-75`) depuis le `FiscalYear` que
   rend déjà `services::exercices::lister` ; présents en mono-devise aussi (ce ne sont pas des champs en devise, comme
   au HTTP) ; la description de l'outil (`:42-46`) les nomme. **Tests** : `mcp_outils_e2e.rs` — un exercice
   multi-devises au genre `month`/`day` et un mono-devise rendent les trois champs ; `mcp_contrat` et
   `mcp_gardes_lexicales` verts. **Mesure de taille** : l'outil n'est pas paginé (aucune `LIMITE`, absent de
   `bornes_attendues`, et son `SansArgument` refuse tout `limit`, `deny_unknown_fields`) : `tailles_mesurees`
   (`mcp_outils_e2e.rs:1345`) ne s'applique donc **pas** telle quelle — test dédié, même méthode (différence de taille
   entre un et deux exercices, trois champs compris) ; le nombre d'exercices qui tient sous `MCP_ENVOI_OCTETS_MAX`
   (`services/envois_mcp.rs:55`, 24 576 octets) est écrit au Change Log du développement, **avec la conséquence** :
   au-delà, `texte_et_valeur` (`mcp/sortie.rs:451-466`, refus à `:463-464`) rend `TropVolumineux` sans recours (aucune pagination) — état
   existant, aggravé de trois champs ; pagination hors périmètre, dite au Change Log.
3. **AC-3 — Écriture du réglage.** `PUT /api/v1/companies/current/currency-settings`
   `{ multiCurrencyEnabled, fxGainAccountId, fxLossAccountId, fxTransitAccountId, version }` (verrou optimiste comme
   les autres `PUT` de la société) :
   - **Administrateur seul**, comme les trois `PUT /companies/current/*` et `PUT /company/invoice-settings`
     (`lib.rs:277-278, 446-461`) : route dans `admin_routes`, **au-dessus** du marqueur « TOUTE ROUTE S'AJOUTE
     AU-DESSUS DE CETTE LIGNE » (`require_admin_role` + `require_not_pat`) ; une clé d'API reçoit
     `API_KEY_ADMIN_FORBIDDEN` ; la route s'ajoute à la liste de `admin_pat_denied_e2e.rs` (:85). **Comment il
     s'active (V6, CR-6 — remplace « il s'active à l'écran »)** : par une **session** d'Administrateur, jamais par une
     clé d'API — `POST /api/v1/auth/login` (`lib.rs:1353`), dont la réponse porte `accessToken` (`routes/auth.rs:184-186`),
     puis le `PUT` en `Authorization: Bearer <accessToken>` (lu par `require_auth` quand aucun cookie n'est présent,
     `middleware/auth.rs:83-95`) ; l'écran ne fait pas autre chose (cookie de la même session). Le refus vise la seule
     clé d'API (`require_not_pat`, `middleware/rbac.rs:58-66` : `api_key_id.is_some()`), non le client HTTP : une
     recette **par l'API seule** (37-5 AC-7, V7) active le mode par une session Bearer. **Test d'API** : Administrateur
     connecté par `POST /auth/login`, `PUT` en Bearer, **sans** cookie → 200 et `multiCurrencyEnabled` vrai ; même
     `PUT` par une clé d'API de l'Administrateur → `API_KEY_ADMIN_FORBIDDEN` ;
   - fonction dédiée `companies::update_currency_settings` (ne passe **pas** par `CompanyUpdate`/`update_in_tx`, qui
     sont en remplacement complet : chaque route e-mail, coordonnées, adresse et `company_update_of` de l'assistant
     devrait sinon recopier trois champs de plus) ;
   - activer exige **les trois comptes** (gains, pertes, conversion ; le même compte est admis pour les gains et les
     pertes) ; tant que le mode est actif, aucun des trois ne peut être vidé (400 `FX_ACCOUNT_INVALID`,
     `reason: "missing"`) ; `multiCurrencyEnabled: false` avec un identifiant non nul → 400 `FX_ACCOUNT_INVALID`,
     `reason: "unexpected"` (une société mono-devise n'a pas de comptes de différence) ; **no-op** (rien ne
     change) : court-circuit sans bump de `version` ni audit, comme KF-004 ; **irréversible (CR-7, A34)** :
     `multiCurrencyEnabled: false` sur une société multi-devises → 409 **`MULTI_CURRENCY_IRREVERSIBLE`** (code neuf,
     `details: {}`, quatre locales ; aucune désactivation n'existe) ;
   - **compte de conversion** (R7, `fx_transit_account_id`) : compte **de bilan** en CHF par lequel passe une
     conversion entre deux devises (37-3b, assistant *Conversion* : il revient à 0 après les deux écritures) ; mêmes
     gardes que les comptes de gains et de pertes (de la société, imputable, actif, **sans devise**, lu `FOR UPDATE`,
     motif C4, revalidation à l'usage par la route de conversion de 37-3b — la réévaluation de 37-5 n'emploie que les
     comptes de gains et de pertes —, verrous de l'AC-4 sous `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`),
     **sauf le type** : `Asset` ou `Liability` (à créer, p. ex. 1090 *Transferts de fonds* — aucun plan livré n'a de
     compte 109x, grep des `*.json` de plans du 2026-11-14), jamais `Expense`/`Revenue`
     (`reason: "accountType"`) ; il ne peut donc pas être le compte de gains ou de pertes. La contrainte « ni au rôle
     `DefaultRevenue`, ni désigné `default_revenue_account_id` » ne le concerne pas (un compte de bilan n'est pas un
     compte de produit) ; la liste close des `reason` est inchangée ;
   - **exercices à l'activation** (R3) : passer le réglage à `true` passe **tous les exercices ouverts** de la société
     en multi-devises (`fiscal_years.multi_currency = TRUE`) dans la même transaction — sans quoi l'exercice initial de
     l'assistant, créé mono-devise (la société l'était), le resterait ; les exercices clos ne changent pas et restent
     mono-devise pour toujours, même rouverts (CR-7). Les identifiants des exercices changés vont au détail d'audit.
     Aucune route ne change ensuite le mode d'un exercice (AC-3 bis) ;
   - **cours de la société à l'activation (U1, A24, CR-5)** : l'activation **ne copie aucun cours** et
     `companies::update_currency_settings` n'appelle pas la copie. La copie dans `exchange_rates` ne porte que sur les
     **devises des comptes de la société** (A24) ; or une société qui passe en multi-devises n'en a aucune : tant
     qu'elle est mono-devise, aucun compte — archivé compris — ne porte de devise (`MULTI_CURRENCY_DISABLED`, AC-4 ; et
     elle ne redevient jamais mono-devise, CR-7). Les cours déjà récupérés se copient **quand un compte reçoit une
     devise** (AC-4) et à la création d'un exercice multi-devises (AC-3 bis) — fonction et règles de **37-2 AC-3 bis**
     (la table n'existe pas avant 37-2) ;
   - **ordre des refus** (une seule raison) : forme du corps (`missing`/`unexpected`, 400, avant la transaction) →
     version (409, jugée sous le `FOR UPDATE` de `companies`) → no-op → `MULTI_CURRENCY_IRREVERSIBLE` →
     `FX_ACCOUNT_INVALID` (compte changé, gain puis perte puis conversion) ;
   - chaque compte : de la société, imputable, actif, de type `Expense` ou `Revenue`, **sans devise** ; **ni** au rôle
     `DefaultRevenue`, **ni** désigné `default_revenue_account_id` (réglages de facturation ou articles) — les gains de
     change ne vont pas au chiffre d'affaires (CO 959b) ; lu `FOR UPDATE` ; sinon 400 `FX_ACCOUNT_INVALID` (champ et
     `reason` en `details`, p. ex. `reason: "revenueAccount"` ; **liste close des `reason`**, reprise par la 37-5 : `missing`, `unexpected`, `unknown`
     (introuvable ou d'une autre société), `archived`, `notPostable`, `accountType`, `foreignCurrency`,
     `revenueAccount`). Les autres désignations (arrondi 6940, frais bancaires
     6900) restent admises. **Motif C4** du précédent (`validate_account_of`, `crates/kesh-api/src/routes/company_invoice_settings.rs:219,231` ;
     `DESIGNATED_ACCOUNT_COLUMNS` vit, lui, dans `crates/kesh-db/src/repositories/company_invoice_settings.rs:46-57`) :
     ces contrôles ne portent que sur un compte **changé** ; un compte inchangé devenu invalide n'empêche pas
     d'enregistrer l'autre (la revalidation à l'usage — la réévaluation, 37-5, C-37-9 — reste le filet). ⚠️
     `validate_account_of` vit dans `crates/kesh-api/src/routes/company_invoice_settings.rs` (:200), non dans le
     dépôt du même nom ; le contrôle de la 37-1 est écrit dans `companies::update_currency_settings`, sous verrou ;
   - **verrous, dans l'ordre global du dépôt** (`companies` → réglages → `accounts` → `bank_accounts`,
     `accounts.rs:446-448`) : ligne `companies` `FOR UPDATE` (version jugée dessus, et non sur une lecture sans verrou
     comme `update_in_tx` :849), puis — quand le geste active — les exercices de la société `FOR UPDATE`
     dans l'ordre chronologique (CR-2 ; ordre `companies` → exercices → `accounts` de `create_in_tx_inner`, qui prend
     l'exercice `FOR UPDATE` avant de lire les comptes, 37-3a AC-4 : les prendre après les comptes formerait un cycle),
     puis les réglages de facturation `LOCK IN SHARE MODE` (contrôle « désigné
     `default_revenue_account_id` »), puis les comptes désignés `FOR UPDATE`, puis les articles qui les désignent
     (`products.default_revenue_account_id`, lecture verrouillante) ;
     ⚠️ **tout test d'existence de ligne d'écriture de l'AC-4 est une lecture verrouillante**
     (`… LOCK IN SHARE MODE`) et **aucune lecture simple ne le précède dans la transaction** : sous REPEATABLE READ,
     une lecture simple fige l'instantané *avant* l'attente des verrous et ne verrait pas une ligne ou un compte
     commités entre-temps (c'est pourquoi la lecture des comptes bancaires de la 34-11a2 est elle-même verrouillante,
     `accounts.rs:617-620`) ;
   - **rejeu sur interblocage** (37-1-CR2-P1-1) : l'ordre « `companies` → exercices » ci-dessus n'est **pas** celui des
     écritures — `create_in_tx_inner` lit `companies` sans verrou, prend l'exercice `FOR UPDATE`
     (`journal_entries.rs:326-336`), puis son `INSERT` prend `companies` en partage par `fk_journal_entries_company`
     (exercice → société). Une activation, qui tient `companies` `FOR UPDATE` puis demande les
     exercices, ferme donc un cycle avec toute saisie d'écriture en cours (angle mort (iv) du registre des routes) :
     le handler est sous `kesh_db::retry::retry_on_deadlock("companies::update_currency_settings", …)`, comme
     `create_fiscal_year`, et la route est **rejouée** — la 37-1 rend donc **deux** routes `SansEcritureAuJournal`
     rejouées (celle-ci et `accounts::create_account`, AC-4 bis, 37-1-CR3-P2-1 ; `currency-mode` retirée, CR-7) :
     `audit_route_registry.rs` (vi) passe de « vingt-cinq » à « vingt-sept » (décompte, liste et « vingt-trois handlers
     `SansEcritureAuJournal` de `src/routes/` » recomptés au code, en deltas : + 2), et un test de victime
     d'interblocage par route dans `rejeu_interblocage_e2e.rs` (patron
     `fiscal_year_reopen_is_replayed_when_it_is_the_deadlock_victim`). **Autres cycles, couverts par le même rejeu**
     (37-1-CR2-P2-2, à écrire au Change Log du développement) : la validation d'une facture, l'avoir et la saisie
     fournisseur prennent les réglages de facturation `FOR UPDATE` **avant** l'exercice (`invoices.rs:1974`,
     `credit_notes.rs:312`, `supplier_invoices.rs:1484`), l'activation les réglages en partage **après** les
     exercices — routes rejouées des deux côtés *(CR-7 : le cycle de la bascule `currency-mode` sous `force` avec les
     écritures et la conversion de 37-3b disparaît avec la route)* ;
   - audit `company.currency_settings_updated` (avant/après, ajouté à `ACTIONS` de `audit_labels.rs`, liste triée) ;
     libellés d'audit dans les quatre locales ;
   - **réponse** : 200 avec le `CompanyJson` à jour (nouvelle `version`, comme les trois autres `PUT` de la société) ;
     tests : Comptable → 403, clé d'API → `API_KEY_ADMIN_FORBIDDEN`, Administrateur en session Bearer sans cookie →
     200 (V6) ; `multiCurrencyEnabled: false` après activation → 409 `MULTI_CURRENCY_IRREVERSIBLE`, réglage et
     exercices inchangés (CR-7) ; le même `false` avec un identifiant non nul → 400 `FX_ACCOUNT_INVALID`
     `unexpected` (la forme du corps passe avant) ;
   - **registre des routes** : le `PUT` entre dans `LIB_ROUTES` (`tests/audit_route_registry.rs`) avec le couple
     `Traced, SansEcritureAuJournal` (le quatrième champ du tuple est la partition de rejeu : il écrit au journal
     d'audit, non au journal comptable) ; **deux** tests comptent, et le second est à recompter aussi :
     `the_registry_partition_is_what_the_story_declares` (:992) : **+1 route et +1 tracée**, total du registre
     (`LIB_ROUTES` + `TEST_ENDPOINT_ROUTES`, `:1069-1073`) **+1** ; `the_replay_partition_is_what_the_story_declares`
     (:1450) : `sans_ecriture` **+1** (:1480), `tout.len()` **+1** (:1502 ; `rejouees` et `exemptees` inchangés) ;
     messages d'assertion et commentaires recomptés, dont l'en-tête du module (`//!`, :33-35). **Chiffres en deltas**
     (état au 2026-11-14 : 146 routes, 139 tracées, total 149, `sans_ecriture` 119) : les 37-2 et 37-5 ajoutent aussi
     leurs routes, l'ordre d'intégration décide du chiffre absolu, **recompté au code, depuis la source, après chaque reprise de `main`**
     (`LIB_ROUTES`, partitions, phrase du manuel). La phrase du manuel
     d'administration (`admin-manual.tex:3217`, « N des M routes… », « N + 4 + 3 = M ») suit le chiffre recompté (`.tex`
     seulement, PDF non commité), garde `le_manuel_dit_le_compte_du_registre`. *(Alignement de l'orchestrateur sur 37-2-P4-IF3.)*
     **CR-7** : la route de bascule que CR-2 ajoutait est retirée — la 37-1 compte donc **+1 route, +1 tracée,
     total +1, `sans_ecriture` +1, `tout.len()` +1**, toujours en deltas.
3. **AC-3 bis — Mode d'un exercice (R3, révisé CR-7).** Chaque exercice est **mono-devise** ou **multi-devises**
   (`fiscal_years.multi_currency`, AC-1), posé à sa création ou à l'activation, jamais changé ensuite :
   - **à la création**, l'exercice prend `companies.multi_currency_enabled`, lu **dans la transaction qui l'insère**
     (`INSERT … SELECT … FROM companies`, ou lecture `LOCK IN SHARE MODE` en tête de transaction, avant les verrous
     d'exercice : ordre `companies` → exercices). **Inventaire clos des chemins qui créent un exercice** (grep
     `INSERT INTO fiscal_years`, relevé au code le 2026-11-14) : `fiscal_years::create` (`repositories/fiscal_years.rs:331`,
     `INSERT` dans `insert_fiscal_year_in_tx` :1360 ; route `POST /api/v1/fiscal-years`, `routes/fiscal_years.rs:259`) ;
     `create_if_absent_in_tx` (:456, `INSERT … SELECT … FROM dual` :470 ; **l'exercice initial de l'assistant**,
     `routes/onboarding.rs:1053`, dont la transaction tient déjà `companies` `FOR UPDATE` par `company_select!` — la
     colonne s'y lit sans verrou de plus) ; `create_for_seed` (:412 ; `kesh-seed/src/lib.rs:286`, `pieges.rs:379`, et
     deux tests de `journal_entries.rs`). Les `INSERT` bruts des tests (`test_fixtures.rs:118`, `invoices.rs:4334`,
     `accounts.rs:1499`, `journal_entries.rs:4022`, `:4220`, `:4353`) prennent le défaut de la colonne (`FALSE`) : admis,
     aucun changement. L'import d'une sauvegarde recopie la colonne (ou la met à son défaut, AC-6) ; la remise à zéro
     recrée l'exercice par l'assistant. Inventaire écrit au Change Log. Test : société multi-devises → exercice créé
     multi-devises par chacun des trois chemins ; société mono-devise → mono-devise ;
   - **genre d'ouverture à la création (T4, CR-4)** : un exercice qui **suit** un exercice Kesh — celui de la société
     dont `end_date` est la veille de son `start_date` — prend par défaut `opening_rate_kind` = `closing_rate_kind` de
     celui-ci ; sinon (premier exercice, ou exercice précédé d'un trou) `'day'`, le défaut de la colonne ;
     `closing_rate_kind` prend toujours `'day'`. Lu dans la transaction qui insère, par **un** fragment SQL partagé des
     trois chemins ci-dessus (DRY ; p. ex. `COALESCE((SELECT closing_rate_kind FROM fiscal_years WHERE company_id = ?
     AND end_date = ? - INTERVAL 1 DAY), 'day')` dans l'`INSERT … SELECT`) : `fiscal_years::create` et `create_for_seed` passent par `insert_fiscal_year_in_tx` (aujourd'hui
     `INSERT … VALUES`, `repositories/fiscal_years.rs:1360` : il devient `INSERT … SELECT … FROM companies`, qui lit aussi
     `multi_currency_enabled`) ; sous REPEATABLE READ, InnoDB pose des verrous **partagés** sur les lignes que lit
     l'`INSERT … SELECT` — la ligne `companies` et l'exercice précédent : ce n'est pas une lecture ordinaire. Le
     précédent n'est pas renvoyé par `find_overlapping` (bornes disjointes), mais peut être tenu par son verrou de
     parcours (dépendant du plan, `repositories/fiscal_years.rs:371-372`) ; un `PUT /rate-kinds` (37-2) concurrent est
     donc attendu ou vu commité, et l'un des deux défauts est retenu, sans conséquence. Dans `create`, la garde
     D-CR7-2 (ci-dessous) a besoin du réglage de la société **avant** l'`INSERT` : `create` lit donc
     `companies.multi_currency_enabled` `LOCK IN SHARE MODE` **en tête de transaction**, avant les verrous d'exercice
     des pré-contrôles (ordre « `companies` → exercices », celui de l'activation de l'AC-3 : aucun cycle entre elles —
     l'une attend l'autre) ; l'`INSERT … SELECT … FROM companies` relit la même ligne, tenue en partage jusqu'au
     `COMMIT` : même valeur. Le rejeu existant (`retry_on_deadlock("fiscal_years::create", …)`,
     `routes/fiscal_years.rs:286`) reste en place pour ses autres cycles ; `create_for_seed`, sans rejeu, ne
     s'exécute que dans une société neuve sans concurrence ;
     `create_if_absent_in_tx` (assistant) n'insère que sans aucun exercice (`WHERE NOT EXISTS`, :472), donc toujours
     `'day'` ; `create_for_seed` applique le même fragment. **Changement ultérieur du genre de clôture (U4, CR-5 —
     remplace A22, « ne se propage pas »)** : quand le genre de clôture d'un exercice change alors que l'exercice
     **suivant** existe (celui qui commence le lendemain de sa clôture), un dialogue propose d'appliquer le genre
     correspondant au **cours d'ouverture** de l'exercice suivant, case **cochée par défaut** — exercice suivant
     **clos** : il n'est pas touché, la case est désactivée avec la raison (37-2 AC-5, AC-6) (seul le genre de **clôture** change la question : celui d'ouverture seul n'y touche pas) ; aucune écriture
     d'ouverture n'est modifiée — il n'en existe aucune entre deux exercices Kesh (soldes cumulés, et soldes
     d'ouverture saisissables sur le seul premier exercice). L'AC (route, champ `applyToNextOpening`, refus, audit,
     tests) vit avec la route, en **37-2 AC-5** (*Genres d'ouverture et de clôture*), et le dialogue en **37-2 AC-6** ;
     cette fiche ne pose que le défaut à la création. Tests : précédent en clôture `month` → nouvel exercice en
     ouverture `month` ; précédent `day` → `day` ;
     exercice séparé du précédent par un trou → `day` ; premier exercice → `day` ; création des trois chemins ;
     **cours de la société (U1, A24, CR-5)** : un exercice créé multi-devises par `fiscal_years::create` reçoit, dans la
     transaction qui l'insère, la copie des cours déjà récupérés **des devises des comptes de la société** (archivés
     compris) pour ses unités utiles (37-2 AC-3 bis, appel ajouté par 37-2) ; `copied` s'écrit à côté du snapshot de
     `fiscal_year.created` (objet JSON, `snapshot_json`) ; sans compte en devise, rien n'est copié ;
   - **aucune bascule (CR-7, A34)** : aucune route ne change le mode d'un exercice après sa création, hors
     l'activation de la société (AC-3 : exercices ouverts → multi-devises). Un exercice clos à l'activation reste
     mono-devise, même rouvert ; tout exercice créé ensuite est multi-devises. L'écran *Exercices* ne change pas ;
   - **aucun exercice avant un exercice mono-devise (D-CR7-2, KISS)** : dans `fiscal_years::create`, après la garde
     de l'invariant I (`find_later_closed_in_tx`, `repositories/fiscal_years.rs:363-380` — un exercice clos parle
     d'abord), si la société est multi-devises (lecture de tête, ci-dessus), le plus proche exercice **postérieur
     mono-devise** (`start_date > ?` et `multi_currency = FALSE`, requête propre : `FIND_LATER_CLOSED_SQL`, partagé par
     d'autres appelants, n'est pas touché) refuse la création → `DbError::LaterFiscalYearMonoCurrency { fiscal_year_id,
     fiscal_year_name }`, mappée dans `map_create_error` (`routes/fiscal_years.rs:146`) en **400
     `LATER_FISCAL_YEAR_MONO_CURRENCY`** — code voisin de `LATER_FISCAL_YEAR_CLOSED`, mêmes `details` `{ fiscalYearId,
     fiscalYearName }`, variante `AppError` voisine de `FiscalYearBeforeClosedYear` (`errors.rs:358`), réponse bâtie par le
     gabarit `later_fiscal_year_closed_response` (`errors.rs:1438`) **généralisé d'un paramètre `code`** (DRY : un seul
     gabarit, pas une copie dont les `details` divergeraient ; son doc-comment suit), clé
     `error-fiscal-year-create-later-mono-currency` dans les quatre locales (« L'exercice « {name} », postérieur, est
     mono-devise (clos avant l'activation du multi-devises) : aucun exercice ne peut être créé avant lui. »). Pas le même
     code : l'exercice visé peut être ouvert, et le remède de `LATER_FISCAL_YEAR_CLOSED` (rouvrir) n'y fait rien. Lecture
     simple suffisante : sous le partage de `companies`, la société ne change pas de mode, et la colonne d'un exercice
     existant ne change plus (AC-3). Les deux autres chemins de création ne sont pas concernés (assistant : aucun
     exercice ; `create_for_seed` : société neuve, mono-devise). Une société mono-devise n'a que des exercices
     mono-devise : la garde ne s'y applique pas (création avant un exercice ouvert admise, comme aujourd'hui) ;
   - **dans un exercice mono-devise** (clos avant l'activation, puis rouvert), un compte en devise **n'est pas
     utilisable** : toute ligne sur un compte en devise, avec ou sans montant en devise, est refusée (409
     `MULTI_CURRENCY_DISABLED`, `details.fiscalYearId`, jugé sur l'exercice de l'écriture, 37-3a AC-4, rang 2). Le
     réglage de la société reste la condition de tout ce qui est en devise (devise d'un compte, écrans, récupération
     des cours) : un exercice multi-devises n'existe que dans une société multi-devises ;
   - **tests** : activation → exercices ouverts multi-devises, clos inchangés (AC-3) ; exercice clos à l'activation
     puis rouvert → reste mono-devise ; exercice créé après l'activation → multi-devises ; création d'un exercice
     avant un exercice clos à l'activation → `LaterFiscalYearClosed`, et, une fois ce dernier rouvert, 400
     `LATER_FISCAL_YEAR_MONO_CURRENCY` nommant l'exercice rouvert, rien de créé ni d'audité (D-CR7-2) ; exercice à la
     fois clos et mono-devise → `LATER_FISCAL_YEAR_CLOSED` (ordre) ; société mono-devise, création avant un exercice
     ouvert → admise, mono-devise ; société multi-devises, création avant un exercice multi-devises ouvert → admise.
4. **AC-4 — Devise d'un compte.** `CreateAccountRequest.currency` est facultatif (`null` ou absent = CHF).
   `UpdateAccountRequest.currency` est `Option<Option<String>>` désérialisé par `present_or_null` (`routes/accounts.rs:60`) :
   **absent = devise inchangée**, `null` = CHF, valeur = devise. L'absence se résout **dans le dépôt**, sous le
   `FOR UPDATE` du compte : `AccountUpdate.currency` est lui aussi à trois états (inchangé / CHF / devise), jamais
   rempli par une lecture de la route. ⚠️ **Pas de 400 sur l'absence, à la différence de `role`** :
   le motif d'un full-replace est d'éviter l'effacement silencieux d'une donnée (`routes/accounts.rs:69-76`) ; ici
   « absent = inchangé » n'efface rien, et un 400 casserait en mono-devise tout client qui ne connaît pas le champ
   (la recette par l'API de l'epic, l'écran de la version précédente), ce que la story s'interdit (« sans rien changer
   pour qui reste en mono-devise », précédent : la `confirm_account_retype`, `routes/accounts.rs:94-109`). Code ISO
   4217 à trois lettres, `trim` puis **majuscules ASCII** (`to_ascii_uppercase`, jamais Unicode) avant validation ;
   toute valeur non ASCII → 400 `CURRENCY_UNKNOWN`. `AccountResponse` expose `currency` et
   `currencyLocked: bool` (le compte porte au moins une ligne d'écriture ; calculé pour la liste en **une** requête,
   bornée à la société — `SELECT a.id FROM accounts a WHERE a.company_id = ? AND EXISTS (SELECT 1 FROM
   journal_entry_lines l WHERE l.account_id = a.id)`, par l'index `idx_jel_account` (celui de la contrainte
   `fk_jel_account`), plutôt qu'un `DISTINCT` sur
   toutes les lignes —, jamais une par compte ; la forme de `letterings::letterable_account_ids`) ; les réponses à un
   seul compte (création, modification, détail) le portent aussi. **Sérialisation (C-37-9 : le JSON des routes HTTP
   reste identique en mono-devise)** : `currency` porte `#[serde(skip_serializing_if = "Option::is_none")]` (absent
   pour un compte CHF) et `currencyLocked` `#[serde(skip_serializing_if = "std::ops::Not::not")]` (absent = `false`) ;
   type TS `currency?: string; currencyLocked?: boolean`, lu `account.currency ?? null`. Test : la réponse d'un
   compte CHF sans ligne n'a aucune des deux clés. Les champs du réglage de `CompanyJson` (AC-2) ne sont pas des
   champs en devise : toujours présents.
   - refus **409** `MULTI_CURRENCY_DISABLED` si la société est en mono-devise et qu'une devise est donnée (409 partout,
     décision d'epic) ;
   - code inconnu de la liste ISO 4217 embarquée (`kesh-core`) → 400 `CURRENCY_UNKNOWN` ; `CHF` donné → normalisé à
     `NULL` **avant** tout autre contrôle : en mono-devise, `"CHF"` (ou `"chf"`) est la devise principale, jamais un
     `MULTI_CURRENCY_DISABLED` (le refus ne vise qu'une devise **autre** que CHF). La liste ne valide qu'**à l'écriture** : la lecture décode la colonne en `String`, de sorte qu'un code
     retiré plus tard de la liste ne fasse jamais échouer la lecture d'un compte ;
   - changer la devise d'un compte qui porte **au moins une ligne d'écriture** → 409 `ACCOUNT_CURRENCY_LOCKED`
     (jamais de confirmation possible, à la différence du changement de type). ⚠️ **Depuis CR-4 (T3), ce verrou porte
     la devise de toutes les lignes passées** : `journal_entry_lines` ne stocke plus de devise (37-3a AC-1), la devise
     d'une ligne est celle de son compte, lue par jointure ; changer la devise d'un compte qui porte une ligne
     réinterpréterait donc en silence toutes ses lignes. Le verrou n'a ni exception ni confirmation, archivés compris ;
     le seul retour est la suppression de toutes ses lignes. Inventaire au Change Log du développement (grep de
     `UPDATE accounts` et de `INSERT INTO accounts`) : `accounts.currency` ne s'écrit que par `create` et `update`
     (gardés), par les créations en masse des plans livrés (`NULL`, compte neuf sans ligne) et par l'import et la
     remise à zéro, qui remplacent comptes et lignes ensemble. Un `PUT` qui renvoie la devise en place
     est un no-op sur ce champ et ne déclenche pas ce refus (`currency` entre dans `is_no_op_change`) ;
   - **un compte que les chemins automatiques débitent en CHF ne peut recevoir de devise** → 409
     `ACCOUNT_CURRENCY_IN_USE` (le site en `details`) : compte qui porte un rôle (`AccountRole`, les dix) ; désigné
     dans les réglages de facturation (`DESIGNATED_ACCOUNT_COLUMNS`, `company_invoice_settings.rs:46-57`) ; lié à un
     compte bancaire (`bank_accounts.journal_account_id`) ; compte de produit par défaut d'un article
     (`products.default_revenue_account_id`) ; contrepartie d'une règle de réconciliation
     (`reconciliation_rules.counterparty_account_id`) (le compte par défaut des factures fournisseur,
     `default_payable_account_id`, est déjà l'une des dix colonnes de `DESIGNATED_ACCOUNT_COLUMNS`). *(Note de dépendance
     future, sans effet sur 37 : l'Epic 38 (38-5) portera les colonnes désignées à onze, avec
     `default_vat_acquisition_account_id`.)* **Inversement**,
     lier à un compte bancaire, désigner dans les réglages de facturation, choisir comme compte d'un article ou d'une
     règle de réconciliation, ou poser un rôle sur un compte **en devise** → 400 `ACCOUNT_IN_FOREIGN_CURRENCY`.
     *(Note de dépendance future, sans effet sur 37 : l'Epic 38 (38-4, O3) lèvera `ACCOUNT_IN_FOREIGN_CURRENCY` pour
     `reconciliation_rules.counterparty_account_id` ; `ACCOUNT_CURRENCY_IN_USE` y restera.)*
     **C'est la 37-1 qui pose ces refus des deux côtés, pour ces sites de configuration** ; les comptes choisis ligne
     par ligne (`invoice_lines`/`credit_note_lines.revenue_account_id`, `supplier_invoice_lines.expense_account_id`,
     `supplier_invoices`/`invoice_settlements.settlement_account_id`) ne sont **pas** gardés par la 37-1 : la 37-3a (AC-5) les
     refuse à l'enregistrement de la pièce (400 `ACCOUNT_IN_FOREIGN_CURRENCY`) ; un brouillon antérieur dont le compte
     est devenu en devise est refusé par le contrôle central à la validation (`ACCOUNT_REQUIRES_CURRENCY`) — décision
     écrite au Change Log. *(Alignement de l'orchestrateur sur 37-3a-P4-IF1.)* La 37-3a (AC-5) s'appuie
     sur les refus de la 37-1.
     Les liens se comptent **quel que soit l'état** de l'objet qui désigne (compte bancaire archivé, article archivé,
     règle supprimée logiquement compris) : aucune réactivation de ces objets n'a de garde de devise. Limite assumée,
     écrite au manuel : un compte lié à un compte bancaire archivé ne reçoit plus de devise.
     Inventaire **clos** des sites, lu sur les FK `REFERENCES accounts` des migrations — **dix-neuf** à ce jour
     (`grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql`), dont `accounts.parent_id` et
     `journal_entry_lines.account_id` hors configuration, soit dix-sept colonnes : les dix des réglages, l'article, la
     règle, et les cinq colonnes de pièce laissées à la 37-3a — et sur les liens sans FK
     (`bank_accounts.journal_account_id`), écrit au Change Log (méthode « inventorier les sites non résolus ») ;
   - **ordre des refus** de `update` (une seule raison, la plus bloquante) : le pré-vol existant de la route reste en
     tête (`find_by_id_in_company` sur le pool → 404, `routes/accounts.rs:290` ; puis `name`, `role` et `postable`
     → 400), le contrôle `CURRENCY_UNKNOWN` se place après lui, dans la route ; avant la transaction, forme
     (`CURRENCY_UNKNOWN`, `check_role_account_type`) ; puis compte introuvable/archivé → version → no-op →
     `MULTI_CURRENCY_DISABLED` → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` → `ACCOUNT_CURRENCY_LOCKED` →
     `ACCOUNT_CURRENCY_IN_USE` → `ACCOUNT_IN_FOREIGN_CURRENCY` → `FX_ACCOUNT_INVALID` des comptes propres (AC-4 bis,
     S2 : forme `missing` avant la transaction, puis `unexpected` sur l'état résultant, puis gain puis perte, comptes
     **changés** seulement) → refus existants dans leur ordre actuel (rôle singleton
     déjà porté, `AccountLinkedToBankAccount`, `AccountHasEntries`). Jugés sur l'**état résultant** du `PUT` : devise
     posée **et** rôle posé dans la même requête → `ACCOUNT_IN_FOREIGN_CURRENCY` (comme `create`) ; devise posée sur
     un compte qui garde son rôle → `ACCOUNT_CURRENCY_IN_USE` ; rôle retiré et devise posée dans la même requête →
     admis (si aucun autre site ne désigne le compte). Pour `create` : `CURRENCY_UNKNOWN` (avant la transaction) →
     `MULTI_CURRENCY_DISABLED` → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` (parent désigné) → `ACCOUNT_IN_FOREIGN_CURRENCY`
     (rôle posé en même temps) → `FX_ACCOUNT_INVALID` des comptes propres (AC-4 bis) ;
   - un compte désigné gain/perte — **par la société ou par un compte en devise** (comptes propres, AC-4 bis, S2 ;
     désignations d'un compte archivé comprises, comme les liens de l'AC-4) — ne peut recevoir de devise, ni être archivé, ni changer de type hors
     `Expense`/`Revenue`, ni recevoir le rôle `DefaultRevenue`, ni passer à `postable = false`, ni recevoir un
     sous-compte (`accounts::create` :197-207 passe le parent à non imputable sans garde) → 409
     `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`. **Symétrique de l'AC-3** (les gains de change ne vont pas au chiffre
     d'affaires) : il ne peut pas non plus être **désigné** compte de produit par défaut — `default_revenue_account_id`
     des réglages de facturation (`company_invoice_settings::update`, qui lit alors `companies` `LOCK IN SHARE MODE`
     **avant** son `INSERT IGNORE` / `FOR UPDATE` des réglages, ordre global) ni d'un article (`products::create` /
     `update`, `companies` lu en partage avant le compte) — → 409 `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, seulement si la
     valeur **change** (motif C4) ; sans quoi la garde de l'AC-3 se contourne en désignant après coup, et la
     revalidation de la 37-5 ne le verrait pas. Pour une désignation **propre** (S2), `details` porte en plus
     `holderAccountId` et `holderNumber` (le compte en devise qui le désigne, le premier par identifiant s'il y en a
     plusieurs) ; sans eux, la désignation est celle de la société. La recherche des comptes qui le désignent
     (**deux** recherches, chacune par son index, nommé — `SELECT id, number FROM accounts FORCE INDEX
     (idx_accounts_fx_gain) WHERE company_id = ? AND fx_gain_account_id = ? ORDER BY id LIMIT 1 LOCK IN SHARE MODE`,
     puis la même sur `fx_loss_account_id` et `idx_accounts_fx_loss` (37-1-CR3-P2-3 : l'empreinte des verrous dépend
     du plan, et `ORDER BY id LIMIT 1` peut faire préférer un parcours de la clé primaire, qui verrouillerait chaque
     ligne lue) —, le plus
     petit identifiant des deux gardé — **jamais** un `OR` sur les deux colonnes) se fait **après** le `FOR UPDATE` du
     compte visé, en lecture verrouillante (AC-3) ; 37-1-CR3-P1-1 : un balayage sans index verrouille en partage toutes
     les lignes de la société sous REPEATABLE READ, et deux archivages concurrents de comptes **différents** (chacun
     tenant son compte `FOR UPDATE`, puis demandant les lignes de l'autre en partage) s'interbloquent, or
     `accounts::archive_account` n'est pas rejoué (registre : seule `accounts::update_account` l'est) ; elle ne peut croiser une désignation concurrente, qui tient `companies` en exclusif
     (AC-4 bis, verrous). À l'archivage, ce refus est **sans recours** — à la différence du précédent
     #486 (`AccountDesignatedInSettings`, confirmable) : `confirm_designated` ne le lève pas. Ordre des refus
     d'`archive_guarded` : sous-comptes actifs → gain/perte → désignation de facturation → version ;
   - **compte de conversion** (R7) : mêmes refus, même code (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `details.designation`
     = `"gain"`, `"loss"` ou `"transit"` — champ ajouté pour les trois), sauf le type : il ne peut pas passer hors
     `Asset`/`Liability` ; le refus de désignation « produit par défaut » ne le vise pas. Le compte de conversion ne
     reçoit ni devise, ni archivage, ni sous-compte, ni `postable = false` ;
   - **verrous** : `accounts::create`, `update` et `archive_guarded` lisent la ligne `companies`
     `LOCK IN SHARE MODE` **avant** le `FOR UPDATE` du compte (même ordre que l'AC-3, pas d'interblocage) ;
     `multi_currency_enabled` et les trois colonnes FX se lisent sous ce verrou (`reactivate` n'est plus touché, CR-7 :
     son seul contrôle de devise est retiré). ⚠️ **Cela change deux fonctions** :
     `update` lit aujourd'hui le compte `FOR UPDATE` *en premier* (`FIND_BY_ID_FOR_UPDATE_SQL`), et `archive_guarded`
     (:859) ne prend **aucun** `FOR UPDATE` sur le compte (lecture simple de `company_id`). Elles lisent donc d'abord `company_id` (immuable) **hors transaction**, sur le
     pool (voir ⛔ ci-dessous), puis, dans la transaction, `companies` en partage, puis les réglages en partage quand le geste touche à une désignation
     (archivage, retypage, `postable`, rôle `DefaultRevenue`, **devise posée**), puis `accounts` `FOR UPDATE` : une lecture simple en tête figerait l'instantané REPEATABLE READ avant l'attente
     (AC-3). ⛔ **`company_id` ne se lit pas `LOCK IN SHARE MODE` sur la ligne du compte** : ce verrou partagé suivi du
     `FOR UPDATE` de la même ligne est une promotion S → X, et deux gestes concurrents sur le même compte
     (deux archivages, un `PUT` et un archivage) s'interbloquent (1213), chacun tenant le partage que l'autre attend
     (la même promotion est décrite à `company_invoice_settings.rs:385-387`). `company_id` est **immuable** : il se lit
     **sur le pool, avant `begin()`** (une lecture hors transaction ne fige pas l'instantané de la transaction ; la
     doc d'`archive_guarded` veut que la société soit celle du compte, lue par la fonction, revue P1 A L-5 — l'appelant
     ne la passe donc pas), puis la transaction prend `companies` en partage, les réglages, le compte `FOR UPDATE`
     et **compare** le `company_id` relu sous verrou à celui de la lecture hors transaction (écart → `NotFound`).
     Le test d'existence de ligne d'écriture se fait **après** le `FOR UPDATE` du compte, en lecture
     verrouillante. La 37-3a lit la devise des comptes `LOCK IN SHARE MODE`
     (`validate_lines_accounts_in_tx` :135 lit aujourd'hui sans verrou). **Les sites de configuration** : les
     validateurs actuels lisent le compte **hors transaction**, sur le pool — `validate_account_of`
     (`routes/company_invoice_settings.rs:211`, `accounts::find_by_id(&state.pool, …)`), `validate_revenue_account`
     (`routes/products.rs:351`), `validate_counterparty_account` (`routes/reconciliation_rules.rs:215`) : un `LOCK IN
     SHARE MODE` y serait relâché aussitôt. Les refus `ACCOUNT_IN_FOREIGN_CURRENCY` et `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`
     de ces sites se font donc **dans la transaction d'écriture du dépôt** : `company_invoice_settings::update` (compte
     changé lu `LOCK IN SHARE MODE` après le `FOR UPDATE` des réglages) ; `products::create` / `update` et
     `reconciliation_rules::create_in_tx` / `update_in_tx` (compte lu `LOCK IN SHARE MODE` **avant** tout verrou ou
     écriture de la ligne d'article ou de règle) ; les routes bancaires par `type_in_share_mode` (34-11a2, `currency`
     ajouté à son `SELECT`) ; **l'étape bancaire de l'assistant** (`routes/onboarding.rs:659-696`) lie un compte par
     `validate_journal_account_id` sur le pool puis `upsert_primary_in_tx`, sans relecture verrouillante : le refus
     `ACCOUNT_IN_FOREIGN_CURRENCY` y est posé dans `validate_journal_account_id` (pré-vol, partagé avec les routes
     bancaires), et la course est une **limite assumée** écrite au Change Log (la société d'un assistant en cours
     n'a ni exercice ni compte en devise : la remise à zéro en repart du stub). Côté `update` d'un compte (devise ou rôle posé) : réglages lus en partage **avant** le
     `FOR UPDATE` du compte (ci-dessus — les lire après formerait un cycle avec `company_invoice_settings::update`, qui
     tient les réglages en `FOR UPDATE` puis lit le compte), puis, **sous** le `FOR UPDATE`, `bank_accounts`,
     `products` et `reconciliation_rules` en lecture verrouillante — de sorte que de deux gestes concurrents l'un
     attend l'autre et voit sa valeur commitée, sans cycle. L'ordre de pose de ces verrous par
     site est écrit au Change Log. Un test de course par paire : activation contre création en devise, désignation
     contre archivage/retypage, changement de devise contre première ligne, **changement de devise contre
     désignation (réglages de facturation) et contre lien à un compte bancaire**. ⚠️ Toute ligne d'écriture de ces
     tests (et de l'E2E) se passe sur un compte **encore CHF**, puis on tente de lui donner une devise : une ligne CHF
     sur un compte déjà en devise, acceptée tant que la 37-3a n'est pas là, sera refusée par son contrôle central
     (`ACCOUNT_REQUIRES_CURRENCY`) et casserait le test ;
   - un compte de groupe (non imputable) peut porter une devise (informatif) ; un enfant n'hérite de rien ;
   - **cours de la société quand un compte reçoit une devise (A24, CR-5)** : `accounts::create` et `accounts::update`
     qui **posent** une devise (compte CHF → devise, ou devise changée sur un compte sans ligne ; pas un no-op)
     copient, dans leur transaction et après leurs verrous, les cours déjà récupérés **de cette devise** pour les
     unités utiles de chaque exercice **multi-devises ouvert** de la société (fonction et règles de **37-2 AC-3
     bis**, qui ajoute l'appel ; la restriction à la seule devise posée y est écrite en prose, « de cette devise
     seule », sans paramètre à la signature de la copie, renvoi signalé à 37-2 ; insertion seulement,
     `source = 'afc'`, `updated_by` `NULL`) ; `copied` s'écrit au détail d'audit de l'action : à côté du snapshot de
     `account.created` (`accounts.rs:227`), à côté de `{ before, after }` pour `accounts::update`, que l'action soit
     `account.updated` ou `account.retyped` (`accounts.rs:792-804`). Les exercices se lisent
     par une lecture **simple**, jamais `LOCK IN SHARE MODE` : une écriture prend l'exercice `FOR UPDATE` puis le
     compte en partage (37-3a AC-4) ; les verrouiller après le `FOR UPDATE` du compte fermerait un cycle. Une course
     avec la création d'un exercice multi-devises, où chacun ne voit pas l'autre non commité, peut laisser des cours
     non copiés : le filet est `lookup` 404 → `POST /fetch` → copie (37-2 AC-3 bis). La sous-requête `accounts` de
     la copie (37-2 AC-3 bis) lit en partage, sous REPEATABLE READ, les comptes en devise de la société : deux poses
     concurrentes de devise sur deux comptes, chacune tenant son compte `FOR UPDATE`, peuvent s'interbloquer ; le
     cycle est couvert par les enveloppes `retry_app_on_deadlock` de `accounts::update` (`routes/accounts.rs:337`) et
     de `accounts::create` (AC-4 bis, *Rejeu*). Aucun ordre de refus ne change.
     Tests (portés par 37-2, la table n'existant pas ici) : un compte reçoit USD → seules les lignes USD sont copiées,
     aucune EUR ; devise changée d'USD à EUR sur un compte sans ligne → lignes EUR ; `PUT` no-op → aucune copie ;
     aucun exercice multi-devises ouvert → rien.
4. **AC-4 bis — Comptes de différence de change propres à un compte en devise (S2, CR-3).** Le réglage de la
   société (AC-3) reste le **défaut**. Sur chaque compte **en devise**, l'utilisateur choisit : *comptes par défaut*
   (`accounts.fx_gain_account_id` et `fx_loss_account_id` à `NULL` : ils suivent le réglage de la société, y compris
   quand il change) ou **comptes propres** (les deux posés ; p. ex. un sous-compte « frais de change cachés PayPal »,
   un autre pour Stripe). Toute ligne d'écart (37-3b), conversion (37-3b) ou réévaluation (37-5) qui touche ce compte
   prend ses comptes propres. Le compte de **conversion** reste un réglage de la société seul. Lien avec la
   comptabilité analytique (projets) : **hors périmètre**, au registre (C-37-8).
   - **Résolution unique** (nom au développement, p. ex. `kesh_db::repositories::accounts::fx_accounts_for(company:
     &Company, account: &Account) -> Option<FxAccounts>`, fonction **pure**, sans I/O) : pour un compte en devise, ses
     comptes propres s'il en a, sinon `companies.fx_gain_account_id` / `fx_loss_account_id` ; `FxAccounts { gain,
     loss, origin: Own | Company }` ; `None` pour un compte CHF, ou si la société n'en désigne pas (état hérité). **Seule
     fonction** qui choisit les comptes de différence côté serveur ; la réévaluation (37-5) l'appelle, **puis** la
     revalidation à l'usage ci-dessous, et, depuis V3 (CR-6), la route de conversion de 37-3b aussi (ligne de
     différence calculée par le serveur sur demande (A31, `completeDifference`) quand le corps ne la porte pas ; paire
     contrôlée quand le corps en porte une sans la demande ; 400 s'il porte une ligne **et** la demande, 37-3b AC-2 bis). Côté écran, **une** fonction TypeScript miroir
     (`fxAccountsFor(account, company)`, `features/accounts/`), seule lue par le formulaire de compte, la ligne d'écart
     et l'assistant *Conversion* (37-3b) et l'assistant TVA (37-3b, S5) ; tests unitaires des deux (propres, défaut,
     défaut après changement du réglage, compte CHF).
   - **Revalidation à l'usage** (C-37-2 (3), étendue) : fonction du dépôt partagée (p. ex.
     `accounts::validate_fx_accounts_in_tx(conn, company_id, &FxAccounts)`), qui relit les deux comptes résolus
     `LOCK IN SHARE MODE` et applique les gardes de l'AC-3 (de la société, imputable, actif, `Expense`/`Revenue`, sans
     devise, ni rôle `DefaultRevenue` ni désigné `default_revenue_account_id`) → 400 `FX_ACCOUNT_INVALID` (`field` :
     `fxGainAccountId` | `fxLossAccountId`, `reason` de la liste close de l'AC-3, plus `holderAccountId` quand
     `origin = Own`). Un compte de la société invalide mais **non employé** (tous les comptes touchés ont leurs comptes
     propres) n'arrête rien. 37-5 l'appelle sous ses verrous, et, depuis V3 (CR-6), la route de conversion de 37-3b (AC-2 bis) ; elle est créée
     ici, avec ses tests.
   - **API** : `CreateAccountRequest` prend `fxGainAccountId?`, `fxLossAccountId?` (absents = par défaut) ;
     `UpdateAccountRequest` les prend en `Option<Option<i64>>` par `present_or_null` (comme `currency`) : **absents =
     inchangés**, `null` = par défaut, valeur = compte propre ; les deux champs vont ensemble — l'un présent sans
     l'autre, ou l'un `null` et l'autre posé → 400 `FX_ACCOUNT_INVALID`, `reason: "missing"`, avant la transaction.
     `AccountUpdate`/`NewAccount` portent le choix en **un** champ à trois états (inchangé / par défaut / propres
     `{ gain, loss }`), résolu sous le `FOR UPDATE` du compte. `AccountResponse` expose `fxGainAccountId` et
     `fxLossAccountId` avec `#[serde(skip_serializing_if = "Option::is_none")]` (JSON identique en mono-devise et pour
     un compte « par défaut », C-37-9 ; type TS `fxGainAccountId?: number`).
   - **Gardes à la désignation** (comptes **changés** seulement, motif C4 de l'AC-3) : chaque compte propre passe les
     gardes de l'AC-3 (de la société, imputable, actif, `Expense`/`Revenue`, sans devise, ni rôle `DefaultRevenue` ni
     désigné `default_revenue_account_id`) → 400 `FX_ACCOUNT_INVALID` (`field`, `reason`, liste close inchangée) ;
     des comptes propres sur un compte qui **reste ou devient CHF** (état résultant du `PUT` : `currency` `null`, ou
     absente sur un compte CHF) → 400 `FX_ACCOUNT_INVALID`, `reason: "unexpected"` (même sens qu'à l'AC-3 : un
     identifiant là où aucun n'est admis) — retirer la devise d'un compte exige donc d'envoyer aussi `null` pour ses
     comptes propres. La société en mono-devise ne peut avoir de compte en devise : `MULTI_CURRENCY_DISABLED` passe
     avant (ordre de l'AC-4).
   - **Gardes inverses** : un compte désigné par un compte en devise est gardé comme un compte désigné par la société
     (AC-4, `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `details.designation` `gain`|`loss` et `holderAccountId`) : archivage,
     retypage hors `Expense`/`Revenue`, devise posée, `postable = false`, sous-compte, rôle `DefaultRevenue`,
     désignation `default_revenue_account_id` (réglages de facturation, article). L'archivage du compte **en devise**
     qui désigne reste admis (il garde ses désignations, inertes tant qu'il est archivé, de nouveau gardées s'il est
     réactivé : elles comptent quel que soit son état). Le mode multi-devises ne se désactive pas (CR-7).
   - **Verrous** : un `POST` ou un `PUT` de compte qui **porte** `fxGainAccountId`/`fxLossAccountId` prend la ligne
     `companies` **`FOR UPDATE`** comme **premier** verrou (au lieu du partage de l'AC-4 ; jamais un partage suivi d'un
     exclusif, promotion S → X) : la désignation propre est ainsi sérialisée contre toutes les gardes inverses, qui
     prennent `companies` en partage avant le compte visé, et contre la réévaluation (37-5, `companies` en partage en
     tête) — aucun cycle entre deux comptes (désignant en `FOR UPDATE`, désigné en partage). Les comptes propres
     désignés se lisent ensuite `LOCK IN SHARE MODE`, après le `FOR UPDATE` du compte en devise. Le formulaire
     n'envoie ces champs que **s'ils changent** (absents = inchangés), pour ne pas sérialiser chaque modification de
     compte. Argument écrit au Change Log ; test de course : désignation propre contre archivage du compte désigné
     (l'un attend l'autre ; l'archivage commité d'abord fait refuser la désignation en `reason: "archived"`, la
     désignation commitée d'abord fait refuser l'archivage en `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) ; **deux archivages
     concurrents de deux comptes différents aboutissent tous deux, sans 1213** (37-1-CR3-P1-1).
   - **Rejeu** (37-1-CR3-P1-2, précisé par 37-1-CR3-P2-2) : le `PUT` qui porte les champs prend `companies`
     `FOR UPDATE` puis le compte visé `FOR UPDATE`. Le cycle se ferme avec une écriture — aucune écriture ne prend
     `companies` avant les comptes (37-3a, D-CR7-1 : plus de verrou de `companies` pour une ligne en devise) —, qui
     tient ce compte en partage (lecture de sa devise) puis, à son `INSERT`, demande `companies` en partage
     (`fk_journal_entries_company`) : cas d'un compte **encore CHF** auquel le même `PUT` donne une devise et ses
     comptes propres, l'écriture portant une ligne CHF sur ce compte (avec ou sans ligne en devise sur un autre).
     (Une ligne qui porte `foreign` sur un compte encore CHF, ou une ligne CHF sur un compte déjà en devise, est
     refusée avant tout `INSERT` : ni l'une ni l'autre ne ferme de cycle.) Cycle couvert par l'enveloppe existante de `accounts::update_account`
     (`retry_app_on_deadlock("accounts::update", …)`, `routes/accounts.rs:337`), qui reste en place ; test de victime
     d'interblocage dans `rejeu_interblocage_e2e.rs`, monté sur ce cas.
     **`create_account`** (37-1-CR3-P2-1) : un `POST` qui porte les champs **et** un `parentId` tient `companies`
     `FOR UPDATE`, puis passe le parent à non imputable (`UPDATE accounts SET postable = FALSE`, `accounts.rs:204-211`,
     verrou exclusif sur la ligne du parent) ; une écriture CHF sur ce parent (imputable jusque-là) le tient en partage
     puis demande `companies` en partage à son `INSERT` : cycle, et `create_account` n'a **aucune** enveloppe
     aujourd'hui (`routes/accounts.rs:267`, `accounts::create(&state.pool, …)` appelé nu). Le handler passe donc sous
     `crate::retry::retry_app_on_deadlock("accounts::create", …)` (le dépôt ouvre sa propre transaction, `:156` ;
     `NewAccount` cloné à chaque tentative) ; registre : la route reste `Traced, SansEcritureAuJournal` (partition et
     totaux inchangés), elle entre à la liste (vi) (AC-3 : « vingt-sept ») ; test de victime d'interblocage dans
     `rejeu_interblocage_e2e.rs`.
   - **Audit et no-op** : `account_snapshot_json` et `is_no_op_change` incluent les deux colonnes (un `PUT` qui ne
     change que les comptes propres écrit et audite `account.updated`).
   - **Tests** : désignation propre (création et modification), retour au défaut (`null`), résolution après changement
     du réglage de la société (défaut suivi, propres inchangés), même compte pour gains et pertes, chaque `reason`
     (dont `unexpected` sur un compte CHF et sur un compte rendu CHF dans le même `PUT`, `missing` pour un seul champ),
     chaque garde inverse avec `holderAccountId`, désignation d'un compte archivé qui désigne comptée, `CHECK` en SQL
     brut, réponse d'un compte « par défaut » sans les deux clés ; rejeu de `update_account` (compte CHF rendu en
     devise avec comptes propres) et de `create_account` (comptes propres et `parentId`) contre une écriture CHF, chacun
     victime d'interblocage puis rejoué (37-1-CR3-P2-1, -2).
5. **AC-5 — Réinitialisation de la société.** Seul chemin qui vide les comptes en gardant la société :
   `kesh_seed::reset_demo` (`onboarding/reset`) — `accounts` est dans `reset_cleared_tables`, `companies` est
   préservée (`backup.rs:136-144`) et remise à zéro en place par `companies::reset_to_stub_in_tx`
   (`companies.rs:214`). Aucun autre `DELETE FROM accounts` hors tests (grep vérifié) ; l'import complet remplace
   `companies` entière. `reset_to_stub_in_tx` écrit `multi_currency_enabled = FALSE, fx_gain_account_id = NULL,
   fx_loss_account_id = NULL, fx_transit_account_id = NULL` (les exercices, applicatifs, sont vidés puis recréés
   par l'assistant). Test : les quatre colonnes entrent dans `PERTURBED` (**pas** dans `NOT_COMPARED`) de
   `reset_restores_the_stub_columns_in_place` (`onboarding_audit_e2e.rs:2342`) et sont perturbées par son `UPDATE`
   (:2344).
6. **AC-6 — Sauvegarde.** Un `.keshbackup` écrit par la 0.17.6 se réimporte (colonnes à leur défaut) — **et le
   reste dans la release** (R1, CR-2 : l'annotation CR-1 qui le bornait à cette story est caduque). **Liste close des
   absences admises** (R1) : une sauvegarde antérieure à l'Epic 37 est forcément mono-devise ; l'import tolère
   l'absence des **seules** tables et colonnes ajoutées par l'Epic 37, déclarées dans une constante de
   `kesh-db::backup` (nom au développement, p. ex. `ABSENCES_ADMISES_EPIC_35: &[Absence]`, `Absence::Table(&str)` |
   `Absence::Colonne(&str, &str)`, chaque entrée avec la version de la migration qui l'ajoute). Cette story crée le
   mécanisme et y inscrit ses colonnes (`companies` × 4, `accounts` × 3 — `currency`, et `fx_gain_account_id`,
   `fx_loss_account_id` de CR-3, S2 —, `fiscal_years` × 3) ; 37-2 y inscrit la table `exchange_rates` (U1, CR-5 ;
   ex-`exchange_rate_overrides` ; la table système `_kesh_fetched_exchange_rates` ne passe pas par l'import), 37-3a les **trois** colonnes
   de `journal_entry_lines` (`amount_currency`, `exchange_rate`, `rate_source` ; aucune colonne de devise, CR-4, T3). Au code :
   - **tables** : la boucle de couverture de `parse_and_verify_avec_inventaire`
     (`crates/kesh-api/src/admin_backup/import.rs:250-253`, « table '…' absente du manifeste ») saute une table de la
     liste absente du manifeste ; toute autre table absente reste refusée (400, test conservé). La restauration la vide
     sans rien y insérer (`restore_body`, `backup.rs:796-803`, saute déjà une table absente de la carte après son
     `DELETE`) : elle est « traitée comme vide ». La réciproque (table hors inventaire → 400) ne change pas ;
   - **fenêtre d'import** : `creates_application_table` (`crates/kesh-db/src/post_restore.rs`, appelée par
     `last_table_creating_migration`) **ignore** les tables de la liste — une table que l'import tolère absente ne
     referme pas la fenêtre ; `registry_entries_are_within_import_window` reste vert, et le registre de rejeu
     (`POST_RESTORE_BACKFILLS`, entrée `20261106000001`) **n'est pas touché** par l'epic ;
   - **colonnes** : déjà tolérées par la règle générique — `missing_required_columns` (`import.rs:625`) ne refuse
     qu'une colonne `is_required()` (ni `NULL` ni `DEFAULT`) ; toutes les colonnes de l'epic en ont un. La liste les
     nomme pour les **garder** : un test vérifie que chaque colonne listée existe au schéma et n'est **pas**
     `is_required()` (une future modification en `NOT NULL` sans défaut rougit), que chaque table listée est dans
     `TABLES_TO_TRUNCATE`, et que chaque entrée est ajoutée par une migration de l'epic (version ≥ `20261114000001`,
     `MIGRATOR`, dont le SQL crée la table ou ajoute la colonne) — la liste ne peut pas servir à autre chose ;
   - **test d'import d'une sauvegarde 0.17.6** (`admin_full_import_e2e.rs`, sur le patron des sauvegardes 0.17.4 et
     0.17.5 de ce fichier, `:5563`, `:5638`, simulées par l'export courant retouché) : export d'une société
     mono-devise, puis retrait des colonnes et des tables de la liste au manifeste et aux NDJSON, `keshVersion` 0.17.6 →
     import 200, réglage et exercices à leur défaut (mono-devise), comptes sans devise (37-2 étend le test à
     `exchange_rates` absente — aucun cours retenu —, 37-3a aux colonnes des lignes — aucune ligne en devise) ;
     et une table **hors liste** retirée → 400 (le refus d'origine tient). La phase 5 de
     `scripts/recette-montee.sh` (import du `.keshbackup` de l'image 0.17.6, exige un 200, `:651`) se joue normalement
     à la release ; son `--diff-attendu` y liste `migrations` (toujours, `_sqlx_migrations`), `companies`, `accounts`
     et `fiscal_years`, plus les tables des autres stories (l'empreinte lit
     `SELECT *` : les colonnes neuves sont un écart attendu, `scripts/recette-montee.sh:414-440`).
   L'aller-retour d'une sauvegarde 37-1 conserve réglage, devises et mode des exercices ; l'export CSV des comptes (`serialize_accounts_csv`,
   `csv_tables.rs:265`, en-tête écrit à la main) porte `currency`, `fx_gain_account_id` et `fx_loss_account_id` (S2) ;
   l'aller-retour conserve les comptes propres (test) ; `company.csv` (`serialize_company_csv`, :163)
   porte les quatre colonnes ; l'export CSV des exercices (`serialize_fiscal_years_csv`, `csv_tables.rs:230`, écrit à
   la main ; `exports/global.rs:124`) porte les trois colonnes d'exercice, avec son cas de test (le test qui compare
   les en-têtes au schéma rougit sinon). Le `.keshbackup` lui-même n'énumère rien à la main (`export_table` lit les colonnes à
   `information_schema`) : les colonnes neuves y passent seules, le test d'aller-retour le prouve. Le test qui
   compare les en-têtes des CSV aux colonnes du schéma (`csv_tables.rs`, ~:2395, `entete(|w| serialize_*_csv(&[], w))`)
   rougit tant que les deux sérialiseurs ne suivent pas ; ses tests unitaires (`serialize_accounts_csv_*`,
   `serialize_company_csv_*`) portent un cas avec devise et avec réglage actif.
7. **AC-7 — Écrans.**
   - *Paramètres → Comptabilité* : une case « Comptabilité multi-devises » et, cochée, trois sélecteurs (compte de gains
     de change, compte de pertes de change, compte de conversion — comptes de bilan pour ce dernier ; mêmes filtres que
     l'AC-3), enregistrer, erreurs affichées ; le texte dit que la devise principale reste le CHF, que le réglage est le
     défaut des exercices créés et passe les exercices ouverts en multi-devises (AC-3), qu'une conversion entre deux
     devises passe par le compte de conversion (R7), et que l'activation est **définitive** (CR-7). **Confirmation**
     (CR-7) : enregistrer la case cochée sur une société mono-devise ouvre un dialogue — « Le multi-devises ne pourra
     plus être désactivé ; les exercices ouverts passent en multi-devises » — avec *Activer* et *Annuler* ; seul
     *Activer* envoie le `PUT`. Une fois actif, la case est cochée et désactivée (les trois sélecteurs restent
     modifiables). Vitest : dialogue affiché, *Annuler* n'envoie rien, case désactivée après activation. Section éditable
     pour `isAdmin` seulement (`settings/+page.svelte:228`), lecture seule pour les autres rôles.
   - *Plan comptable* : en multi-devises, un champ « Devise » (sélecteur ISO 4217, vide = CHF) à la création et à la
     modification, désactivé d'avance avec l'explication si `currencyLocked` ; une colonne « Devise » dans la liste
     (vide pour CHF). **Comptes de différence de change** (S2, AC-4 bis), pour un compte **en devise** seulement
     (champ « Devise » non vide) : un choix « Comptes de différence de change : **par défaut** / **propres** » —
     « par défaut » rappelle les comptes de la société (« 6840 Différences de change / 6840 », lus par
     `fxAccountsFor`), « propres » ouvre deux sélecteurs (gains, pertes ; mêmes filtres que ceux des paramètres :
     comptes `Expense`/`Revenue` en CHF, imputables, actifs) ; envoyés seulement s'ils changent ; erreurs
     `FX_ACCOUNT_INVALID` affichées sur le sélecteur (`field`). Passer la devise à vide repasse le choix à « par
     défaut » (et envoie `null`). Vitest : le choix n'apparaît que pour un compte en devise, « par défaut » n'envoie
     rien quand rien ne change, « propres » envoie les deux. En mono-devise : **aucun** changement visible.
   - Captures d'écran des états touchés (recette visuelle, thème clair), hors du dépôt.
8. **AC-8 — MCP et services.** `kesh_plan_comptable` donne la devise d'un compte. Le service
   (`services/plan_comptable.rs`) rend déjà l'`Account` entier : le changement est dans la sortie de l'outil,
   `sortie! struct Compte` (`crates/kesh-api/src/mcp/outils/plan_comptable.rs:31-43`), champ `devise` (noms français
   de la sortie) de type `Option<…>` admis par `sortie!` — `TexteNonFiable` ou un type scellé dédié, **jamais
   `String`, qui ne compile pas** (`mcp/sortie.rs:92`) ; `null` pour le CHF (la macro ne pose aucun
   `skip_serializing_if` : la clé est toujours présente). La description de l'outil mentionne la devise. Les tailles
   mesurées (`LIMITE_DEFAUT` 40, `LIMITE_MAX_MCP` 70 ; `mcp_outils_e2e.rs:183`) sont re-mesurées avec le champ
   (précédent C-e331-27) ; le test de contrat du protocole MCP de la CI est relu. Test de l'outil avec un compte en
   devise et un compte CHF. Les comptes de différence propres (AC-4 bis) ne sont **pas** ajoutés à la sortie MCP
   (hors périmètre, écrit au Change Log du développement). `kesh_exercices` est étendu (AC-2, V10). **Société (V11,
   CR-6)** : aucun outil MCP n'expose la société — état existant, aucun module `societe` parmi les outils
   (`mcp/outils/mod.rs:9-20`) — ; le réglage multi-devises et ses trois comptes ne sont donc lus par aucun outil :
   **hors périmètre**, dit ici et au Change Log du développement (un client lit `GET /companies/current`, AC-2).
9. **AC-9 — i18n.** Toutes les chaînes neuves dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`),
   `lint-i18n-ownership` vert.
10. **AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : section « Comptabilité multi-devises » —
    activer, comptes de différences de change, devise d'un compte et son verrou — **dit que le verrou est définitif**
    (une écriture sur un compte en devise ne se défait que par sa suppression) et que **l'activation du multi-devises
    est définitive** (CR-7 : confirmation demandée, aucun retour en mono-devise) ; `.tex` seulement, PDF non commité
    (régénéré à la release ; rendu contrôlable en local par `pdftotext | tr`). Il dit aussi que les exercices **ouverts** à l'activation et tous ceux créés ensuite sont
    multi-devises, et qu'un exercice clos avant l'activation reste mono-devise, même rouvert (CR-7), et qu'aucun exercice ne
    peut être créé avant lui (D-CR7-2, AC-3 bis), le **compte de conversion** (R7 : un compte de bilan en CHF à créer, p. ex.
    1090, qui revient à 0 après une conversion), les **comptes de différence propres** d'un compte en devise (S2 :
    par défaut ou propres, p. ex. des sous-comptes de frais de change cachés pour PayPal et pour Stripe ; ce qu'ils
    changent à la ligne d'écart, à la conversion et à la réévaluation ; gardes inverses) et la réimportation des
    sauvegardes antérieures (R1, mono-devise), le **verrou de la devise d'un compte** comme ce qui fixe la devise de
    ses écritures (T3), le **genre du cours d'ouverture** (cours de clôture de l'exercice précédent, repris par défaut,
    T4) et les **dates du premier exercice** choisies à l'assistant, non modifiables ensuite (T1, AC-11). Aucun plan livré n'a de compte « Différences de change » (`pme.json:76-78` : 6900, 6940,
    6950) : le manuel dit quoi choisir ou créer (p. ex. 6840, **sous le groupe 6** et non sous 6800 *Amortissements*,
    compte imputable que la création d'un sous-compte rendrait non imputable, `accounts::create` :197-207). Si le manuel cite la route
    `PUT /api/v1/companies/current/currency-settings`, ajouter l'entrée à `MULTI_SOCIETE_ASSUME` de
    `textes_coherents.rs` (motif `\bcompanies\b`, précédent :1014-1017). **Le manuel dit aussi (V6, CR-6) que le
    mode s'active par une session d'Administrateur — écran, ou client HTTP en `Authorization: Bearer` après
    `POST /auth/login` — et jamais par une clé d'API** ; même phrase au manuel d'administration, là où il dit les
    routes refusées à une clé d'API — le point **Administration** (`admin-manual.tex:3183`, non `:3217`, qui est le
    décompte du journal d'audit) : le réglage multi-devises s'ajoute à son énumération des routes réservées, et sa
    phrase « Ces opérations se font dans l'interface web », que V6 rend fausse, devient « par une session
    d'Administrateur — l'interface web, ou un client HTTP en `Authorization: Bearer` —, jamais par une clé » (37-1-CR6-P2-2)
    — et à la Note de version. **Matrice MCP du manuel d'administration** (`admin-manual.tex:3447`, V10) : la ligne
    `kesh\_exercices` énumère ce que l'outil lit, « (identifiant, nom, dates, statut) » ; elle gagne le mode
    multi-devises et les genres de cours d'ouverture et de clôture — `mcp_matrice_manuel.rs` ne lit que les colonnes
    outil et portée, aucun test ne verrait l'énumération périmée (37-1-CR6-P2-1). `.tex` seulement, PDF régénérés à la release.
11. **AC-11 — Dates du premier exercice dans l'assistant de démarrage (T1, CR-4 ; remplace la version « retirée »
    de CR-3 et l'ancien arbitrage A1).** Aujourd'hui, `finalize` (`routes/onboarding.rs:879`, sans corps) crée
    l'exercice initial du 1er janvier au 31 décembre de l'année courante (`:1043-1046`, `Utc::now()`), par
    `create_if_absent_in_tx` (`:1053` ; `repositories/fiscal_years.rs:456`, `INSERT … WHERE NOT EXISTS`). ⚠️ **Les dates
    d'un exercice ne se modifient plus ensuite** — relevé au code le 2026-11-14 : `PUT /fiscal-years/{id}` ne change
    que le nom (`UpdateFiscalYearRequest { name }`, `routes/fiscal_years.rs:86-90`, doc-comment « Seul le `name` est
    mutable. Les dates et le statut ne peuvent pas être modifiés ») et aucune route ne supprime un exercice ; un
    exercice voulu ailleurs chevaucherait l'exercice créé (`error-fiscal-year-overlap`). La revue T1 disait « fin
    modifiable dans *Exercices* » : **prémisse fausse au code**, d'où le choix de la fiche — la fin se modifie **dans
    l'assistant**, avant la création (question posée à Beat, Change Log).
    - **Corps** : `finalize` prend `body: Option<Json<FinalizeRequest>>` (précédent : `logout`,
      `routes/auth.rs:357`), `FinalizeRequest { fiscal_year_start_date: Option<NaiveDate>, fiscal_year_end_date:
      Option<NaiveDate> }` (`camelCase`). Corps absent ou champs absents : début = 1er janvier de l'année courante (comme
      aujourd'hui), fin = **veille du même jour un an plus tard** (le « même jour un an plus tard » d'un début au 29 février est le
      **1er mars** suivant, donc fin au **28 février** ; Rust : `with_year(y + 1)` rend `None` un 29 février, à traiter
      par `from_ymd_opt(y + 1, 3, 1)` — jamais `checked_add_months(12)`, qui ramène au 28 février et donnerait le 27 ;
      la fonction du serveur et son miroir TypeScript de l'écran, `new Date(y + 1, m, d)` qui donne le 1er mars,
      rendent la même fin, et les **mêmes cas limites** sont testés de chaque côté (test API et Vitest, ci-dessous ;
      le miroir formate sa date par les champs locaux, comme `todayLocal`, jamais par `toISOString()`, que relève
      `local-date-guard.test.ts`) ; sans début fourni : 31 décembre — le comportement actuel, clients de l'API et tests
      existants inchangés). Avant la transaction : `borner` sur chaque date fournie (`dates_recues.rs:55`, comme
      `create_fiscal_year`, 400), fin ≤ début → 400 `error-fiscal-year-dates-invalid` (même message que
      `map_create_error`) ; aucune borne de durée (S4 : six ou dix-huit mois admis). Un corps illisible est refusé par
      l'extracteur (statut d'axum, comme `logout`). **Nom** : « Exercice AAAA » si début et fin sont dans la même
      année, sinon « Exercice AAAA/AAAA » (années de début et de fin) — au plus 50 caractères (`VARCHAR(50)`).
      **Rejeu et idempotence** : les dates se calculent une fois, hors de la fermeture rejouée de
      `retry_app_on_deadlock` (passées par référence à `finalize_inner`) ; une finalisation déjà faite (`step_completed
      == 8`) ignore le corps et rend l'état, comme aujourd'hui. Le chemin de démonstration (`seed_demo`) n'est pas
      touché. Le registre des routes ne change pas (route existante, même partition).
    - **Multi-devises** : l'assistant ne le règle **pas** ; il se règle plus tard dans *Paramètres → Comptabilité*
      (AC-7), et son activation passe l'exercice initial, ouvert, en multi-devises (AC-3). L'exercice initial est donc
      créé mono-devise, genres d'ouverture et de clôture `day` (AC-3 bis : aucun exercice précédent).
    - **Écran** (`frontend/src/routes/onboarding/+page.svelte`, dernière étape, compte bancaire, `stepCompleted === 6`,
      :478 — les **deux** boutons, « Enregistrer » et « Passer », appellent `finalize`, :293 et :310) : une section
      « Premier exercice » au-dessus du formulaire bancaire, deux champs de date (forme suisse JJ.MM.AAAA, comme les
      écrans comptables, #880) — *Début* (défaut : 1er janvier de l'année courante, date du poste par `todayLocal`, #889) et
      *Fin* (calculée : veille du même jour un an plus tard, recalculée à chaque changement du début tant qu'elle n'a
      pas été retouchée ; modifiable) — et une phrase : « Les dates d'un exercice ne se modifient plus une fois
      créé. » Contrôle à l'écran : fin > début, sinon les deux boutons sont désactivés avec le motif. `finalize(body)`
      (`features/onboarding/onboarding.api.ts:95`, `onboarding.svelte.ts:153`) envoie `{ fiscalYearStartDate,
      fiscalYearEndDate }` (AAAA-MM-JJ) ; un refus 400 s'affiche par `reportFinalizeError` (:237, motif du serveur).
      i18n quatre locales, `lint-i18n-ownership`.
    - **Inventaire clos des sites** (relevé le 2026-11-14) : route `finalize` / `finalize_inner`
      (`routes/onboarding.rs:879-1060`) ; DTO neuf ; `onboarding.api.ts:95-96` et `onboarding.svelte.ts:153-162` ;
      `routes/onboarding/+page.svelte` (:230-320 et l'étape :478) ; tests d'API qui appellent `finalize`
      (`onboarding_e2e.rs` 3, `onboarding_path_b_e2e.rs` 4, `onboarding_audit_e2e.rs` 11, `fiscal_years_e2e.rs` 21,
      `onboarding_bank_account_e2e.rs` 1, `admin_backup_e2e.rs` 2, `audit_route_registry.rs` 5 — lignes qui citent le
      mot ; appels sans corps, donc inchangés) ; E2E de l'assistant (`onboarding-path-b.spec.ts`, `onboarding-bank-account.spec.ts`,
      `onboarding-motif.spec.ts`) et Vitest de l'étape (`routes/onboarding/onboarding-bank-step.test.ts`,
      `onboarding-motif.test.ts` : ils montent l'écran de la dernière étape, qui gagne la section « Premier exercice »
      et dont `finalize` reçoit un corps — leurs attentes sont relues) ; manuel utilisateur (`user-manual.tex:217`, « il crée votre premier exercice,
      l'année civile en cours », et `:1004`, « Si votre premier exercice Kesh ne doit pas commencer au 1er janvier …
      créez d'abord l'exercice voulu », tous deux faux après cette AC ; `:2465` relu) ; guide d'API si `finalize` y
      figure (grep au développement).
    - **Tests** : API — sans corps → 01.01–31.12 de l'année courante, nom « Exercice AAAA » ; début 01.04.2026 seul →
      fin 31.03.2027, nom « Exercice 2026/2027 » ; début 01.07 et fin fournie à dix-huit mois → admis ; début
      29.02.2028 → fin 28.02.2029 ; début 01.03.2027 → fin 29.02.2028 ; fin ≤ début → 400, rien de créé, étape inchangée (7) ; date hors bornes → 400 ;
      finalisation rejouée avec d'autres dates après succès → état rendu, exercice inchangé ; audit
      `fiscal_year.created` aux dates choisies. Vitest de l'écran (défaut, recalcul de la fin, fin retouchée gardée,
      boutons désactivés si fin ≤ début, corps envoyé par les deux boutons ; cas limites partagés avec le serveur :
      début 29.02.2028 → fin 28.02.2029, début 01.03.2027 → fin 29.02.2028). E2E : assistant mené avec un début au
      1er avril → *Exercices* liste 01.04.AAAA–31.03.AAAA+1. Manuel (AC-10) : les deux passages ci-dessus récrits.
    Les tests d'exercices décalés (1er janvier, 1er juillet, 1er avril — T2) et de durée autre que douze mois (six et
    dix-huit mois) des cours et de la réévaluation restent ceux de 37-2, 37-3b et 37-5.
    **Limite assumée** : le jour par défaut est celui du serveur (`Utc::now()`, comme aujourd'hui) pour un client
    d'API sans corps, celui de l'écran la date du poste (`todayLocal`, `shared/utils/local-date.ts:13`, Zurich pour un poste réglé
    sur la Suisse) : la nuit du 31 décembre, les deux années peuvent
    différer ; l'écran envoie toujours ses dates.

## Tâches

- [x] Migration + `migrations.sha384` + audit d'idempotence + squash (AC-1).
- [x] `kesh-core` : module `currency` — liste ISO 4217 (codes actifs, trois lettres), `CurrencyCode` validé,
      `BASE_CURRENCY = "CHF"` ; tests (AC-4).
- [x] `kesh-db` : `Company` + **les quatre listes de colonnes écrites à la main** qui la construisent par
      `query_as::<_, Company>` — `FIND_BY_ID_SQL`/`LIST_SQL` (`companies.rs:24,29`), la macro `company_select!` de
      `routes/onboarding.rs:1273-1287` (`COMPANY_SELECT_FOR_UPDATE`, `COMPANY_BY_ID_FOR_UPDATE`, lues :1292, :1376,
      :1482) et la requête de `kesh-seed/src/lib.rs:210-214` (`seed_demo`). ⚠️ `FromRow` : une colonne manquante
      n'échoue **pas** à la compilation mais à l'exécution (assistant et chargement de la démonstration en 500) ;
      les quatre colonnes de `companies` de l'AC-1 entrent dans les quatre ; `Account`/`NewAccount`/
      `AccountUpdate` + macro `columns!` ; `companies::update_currency_settings` (gardes C-37-2, verrous AC-3) ;
      `accounts::create/update/archive_guarded` (verrous, gardes AC-4) ; `is_no_op_change` (:530-537) et
      `account_snapshot_json` (:57) avec `currency` ; `bulk_create` (:1146) et `bulk_create_from_chart_in_tx` (:1256)
      lient `currency` (leurs `INSERT` l'ignoreraient sinon en silence ; plans livrés et assistant : `NULL`) ;
      sites de construction inventoriés par l'échec de compilation (`NewAccount` : une cinquantaine de littéraux
      `NewAccount {` (52 occurrences du motif, définition comprise), dans les tests de `kesh-db` et de `kesh-api`
      surtout, hors `NewAccount::new` qui prend `currency: None` ; `AccountUpdate` : deux sites de production —
      `routes/accounts.rs:315` et la normalisation de `accounts::update` (`accounts.rs:663`, `..changes` : elle
      recopie `currency` sans erreur de compilation, à garder ainsi) — et vingt-quatre littéraux de tests (vingt et un dans
      `accounts.rs`, un dans `credit_card_repository.rs`, `letterings.rs` et `journal_entry_reversal_e2e.rs`) ;
      `kesh-seed` n'en construit aucun, vérifié) ;
      gardes croisées de l'AC-4 dans les transactions du dépôt des réglages de facturation, comptes bancaires,
      articles et règles de réconciliation (les colonnes de pièce, factures fournisseur comprises, sont à la 37-3a) ; `reset_to_stub_in_tx` (AC-5) ; `test_fixtures.rs`, seed.
- [x] `kesh-api` : `CompanyJson` (`routes/companies.rs:31`, miroir écrit à la main) ; `serialize_accounts_csv` et
      `serialize_company_csv` (AC-6).
- [x] `kesh-api` : routes, DTO, codes d'erreur (`errors.rs` + i18n), audit (`audit_labels.rs`), MCP (AC-2 à AC-8).
- [x] Frontend : `settings` (section Comptabilité), `accounts` (champ, colonne), types, API, Vitest (AC-7).
- [x] `kesh-api` : registre des routes d'audit et phrase du manuel d'administration (AC-3).
- [x] Manuel (AC-10, `.tex` seulement), section *Note de version* de la fiche — pas d'écriture au `CHANGELOG.md` (nouveau champ `currency` du compte, absent = inchangé ; réglage
      réservé à l'Administrateur, par une session — écran ou client HTTP en Bearer —, hors clé d'API, V6).
- [x] Tests : dépôt (gardes, verrou, reset), API (codes), sauvegarde (AC-6), E2E Playwright (créer d'abord un compte de
      bilan pour la conversion — aucun plan livré n'en a, AC-3 —, activer le mode avec les trois comptes, créer un
      compte USD, verrou : écriture passée par l'API sur un compte **CHF**, puis devise refusée à l'écran — AC-4 ;
      dialogue de confirmation de l'activation, case désactivée ensuite — AC-7, CR-7).
- [x] CR-2 — `kesh-db` : `FiscalYear` (`entities/fiscal_year.rs:73`) gagne `multi_currency`, `opening_rate_kind`,
      `closing_rate_kind` (énumération `RateKindChoice`/nom au développement) ; **quinze listes de colonnes écrites à la
      main** à compléter (`FromRow` échoue à l'exécution, non à la compilation) — quatorze constantes et requêtes de
      `repositories/fiscal_years.rs` (`FIND_BY_ID_SQL` :145, `LIST_BY_COMPANY_SQL` :150, `LOCK_IN_COMPANY_SQL` :162,
      `EARLIER_OPEN_SQL` :186, `NEXT_AFTER_SQL` :207, `OPEN_COVERING_DATE_SQL` :731, `FIND_LATER_CLOSED_SQL` :890…,
      recomptées par `grep -rn "SELECT id, company_id, name, start_date, end_date, status" crates --include=*.rs` : 14 + 1 de
      production et la copie de test de `kesh-db/tests/reversal_next_fiscal_year.rs:554` (texte de verrou, sans
      `FromRow`) ; le motif sans `SELECT` en rend 21, avec cinq `INSERT` de `kesh-report/tests`) et une de
      `repositories/closing_checks.rs` ; `snapshot_json` des exercices ; les trois chemins de création (AC-3 bis) ;
      garde D-CR7-2 de `fiscal_years::create` (lecture de tête de `companies` en partage, exercice postérieur
      mono-devise → `DbError::LaterFiscalYearMonoCurrency`, AC-3 bis) ;
      `update_currency_settings` étendu (compte de conversion, exercices à l'activation, refus
      `MULTI_CURRENCY_IRREVERSIBLE` — CR-7) ; liste close des absences admises et son
      emploi dans `parse_and_verify_avec_inventaire` et `creates_application_table` (AC-6).
- [x] CR-2 — tests manquants (37-1-CR2-P1-3) : activation → exercices **ouverts** multi-devises, clos inchangés,
      identifiants au détail d'audit ; `false` après activation → 409 `MULTI_CURRENCY_IRREVERSIBLE` (CR-7) ; compte de
      conversion : `reason: "accountType"` pour `Expense`/`Revenue`, `details.designation = "transit"` à l'archivage, au
      retypage, à la devise et au sous-compte ; refus de l'activation sans l'un des trois comptes ; rejeu sur
      interblocage de `currency-settings` (AC-3).
- [x] CR-2 — `kesh-api` : code `MULTI_CURRENCY_IRREVERSIBLE` (409, quatre locales, CR-7), code
      `LATER_FISCAL_YEAR_MONO_CURRENCY` (400, `map_create_error`, variante `AppError`, gabarit de réponse généralisé d'un
      `code`, quatre locales, D-CR7-2) et ses tests (AC-3 bis),
      `FiscalYearResponse` ;
      frontend : troisième sélecteur des paramètres et dialogue de confirmation de l'activation (AC-7, CR-7) ; test
      d'import d'une sauvegarde 0.17.6 (AC-6). *(CR-7 : route `currency-mode`, ses codes, son audit et l'écran
      *Exercices* retirés.)*
      *(CR-4 : la phrase « l'assistant n'est pas touché » ne vaut plus ; voir la tâche T1 ci-dessous.)*
- [x] CR-4 (T1) — dates du premier exercice dans l'assistant (AC-11) : `FinalizeRequest`, calcul de la fin et du
      nom, `borner`, refus avant la transaction, écran de la dernière étape, `onboarding.api.ts` /
      `onboarding.svelte.ts`, i18n, tests d'API, Vitest, E2E, manuel (`user-manual.tex:217`, `:1004`).
- [x] CR-4 (T4) — défaut du genre d'ouverture à la création (AC-3 bis) : fragment SQL partagé des trois chemins de
      création, tests ; sens des genres écrit au doc-comment de `FiscalYear` (AC-1).
- [x] CR-5 — rien à développer ici pour U1, A24 et U4 : la copie des cours (appels dans `fiscal_years::create`,
      `accounts::create` et `accounts::update` ; **aucun** dans
      `update_currency_settings`, vide par construction) et `applyToNextOpening` sont des tâches de **37-2** (AC-3 bis,
      AC-5, AC-6) ; cette story laisse ces fonctions prêtes à recevoir l'appel (transaction ouverte, verrous pris,
      détail d'audit extensible).
- [x] CR-6 — **V6** : test d'API de l'activation en session Bearer (`POST /auth/login`, `PUT
      /companies/current/currency-settings` avec `Authorization: Bearer`, sans cookie → 200) à côté du refus par clé
      d'API ; manuel d'administration et Note de version : « s'active par une session, jamais par une clé d'API » (non « à
      l'écran »), point **Administration** du manuel d'administration (`:3183`) amendé (AC-10) ; **V10** :
      `kesh_exercices` — `multi_devises`, `genre_ouverture`, `genre_cloture` (type scellé),
      description, ligne de la matrice MCP du manuel d'administration (`:3447`, AC-10), test `mcp_outils_e2e.rs`, mesure de taille d'un exercice écrite au Change Log, `mcp_contrat` et
      `mcp_gardes_lexicales` verts ; **V11** : « aucun outil n'expose la société » écrit au Change Log du développement.
- [x] CR-3 (S2) — comptes de différence propres (AC-4 bis) : colonnes et deux `CHECK` dans la migration de l'AC-1 ;
      `Account`, `NewAccount`, `AccountUpdate` (un champ à trois états), macro `columns!`, `bulk_create` et
      `bulk_create_from_chart_in_tx` (`NULL`) ; `fx_accounts_for` (pure) et `validate_fx_accounts_in_tx` (dépôt) ;
      gardes à la désignation et gardes inverses (`holderAccountId`) dans `create`, `update`, `archive_guarded`,
      `company_invoice_settings::update`, `products::create/update` ; `companies` `FOR UPDATE` en tête
      d'un geste qui porte les champs ; enveloppe de rejeu de `create_account` (37-1-CR3-P2-1) ; recherches du
      compte désignant par `FORCE INDEX` ; DTO, `AccountResponse`, CSV des comptes, liste close des absences admises
      (`accounts` × 3) ; frontend : `fxAccountsFor`, choix « par défaut / propres » du formulaire de compte ; tests de
      l'AC-4 bis.

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md` — `CLAUDE.md` § *Travailler à plusieurs développeurs*.)*

- **Added** — nouveau champ `currency` du compte (absent = inchangé) ; réglage multi-devises réservé à l'Administrateur, activé par une session (écran ou client HTTP en `Authorization: Bearer`), jamais par une clé d'API ; l'activation est définitive ; `kesh_exercices` (MCP) gagne `multi_devises`, `genre_ouverture` et `genre_cloture`.
- **Added** *(complété à la revue de code P2, 37-1-CR-P2-6)* — champs `fxGainAccountId`, `fxLossAccountId` (comptes de différence propres d'un compte en devise) et `currencyLocked` du compte ; corps facultatif de `POST /onboarding/finalize` (`fiscalYearStartDate`, `fiscalYearEndDate`, dates du premier exercice) ; `kesh_plan_comptable` (MCP) gagne `devise` ; codes neufs `MULTI_CURRENCY_DISABLED`, `MULTI_CURRENCY_IRREVERSIBLE`, `CURRENCY_UNKNOWN`, `FX_ACCOUNT_INVALID`, `ACCOUNT_CURRENCY_LOCKED`, `ACCOUNT_CURRENCY_IN_USE`, `ACCOUNT_IN_FOREIGN_CURRENCY`, `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `LATER_FISCAL_YEAR_MONO_CURRENCY`.
- **Changed** — `kesh_plan_comptable` (MCP) : page par défaut 40 → **38** comptes, maximum 70 → **68** (C-37-1-3) ; un `limit` de 69 ou 70 est désormais refusé.
- **Migration** `20261114000001_multi_currency_settings`, non breaking (`kesh_version_min_required` inchangé) : la sauvegarde d'avant la mise à jour reste exigée (le retour à la version précédente en dépend).
- **Compteurs** : recomptés depuis la source après chaque reprise de `main` (`LIB_ROUTES`, totaux de l'audit des migrations, phrase du manuel d'administration).

## Notes de développement

- Comptes : `crates/kesh-db/src/repositories/accounts.rs` (`COLUMNS` :34, `create` :153, `update` :622, garde
  `AccountHasEntries` :726-735, `archive_guarded` :859, `reactivate` :1026) ; `crates/kesh-api/src/routes/accounts.rs`
  (`CreateAccountRequest` :36, `present_or_null` :60, `UpdateAccountRequest` :77, `AccountResponse` :139).
- Société : `crates/kesh-db/src/entities/company.rs` (`Company` :137), `repositories/companies.rs` (`update_in_tx`
  :842, `reset_to_stub_in_tx` :214) ; `routes/companies.rs` (`get_current` :448, `PUT` par champ :112/:250/:344 —
  suivre leur forme ; :497 est `lock_company_books`, un `POST` ouvert au Comptable, pas un précédent de réglage) ;
  écran `frontend/src/routes/(app)/settings/+page.svelte` (section Comptabilité :881-890).
- Précédent de garde d'archivage d'un compte désigné : #486 (Story 33-8, `compte-designe-archive`) — confirmable, à
  la différence du refus gain/perte de l'AC-4.
- Marqueur de plan comptable pour un compte de différences de change (sur le modèle de `roundingDifference`) : hors
  périmètre, à noter au registre pour plus tard.
- La liste ISO 4217 est **embarquée** (pas de dépendance réseau) ; une devise absente des cours AFC reste admise (cours
  manuel, 37-3). Elle ne porte que des **monnaies** : les codes réservés (`XXX`, `XTS`), les métaux (`XAU`, `XAG`,
  `XPT`, `XPD`), les droits de tirage (`XDR`) et les unités de fonds en sont exclus ; `CHF` y figure (normalisé à
  `NULL` par l'AC-4).
- `CHECK` : collation binaire de la colonne **et** `BINARY` dans les `CHECK` (AC-1) — `20261110000001` ne met
  `BINARY` que sur son `IN`, son `REGEXP` ne porte que sur des chiffres : ne pas le recopier tel quel.

## Dev Agent Record

*(Développement du 2026-11-16, worktree `kesh-s37-1`, branche `story/37-1-reglage-et-devise-des-comptes`, base `19bf12fb`.
Mené par phases successives — A1 schéma et plomberie, A2 réglage de la société, B devise des comptes et gardes, C
assistant, D écrans, E manuels, F E2E, captures et gate complet —, chacune remise à l'orchestrateur qui a commité.
Journaux, captures et notes dans `kesh-gate-logs/s37-1/` (hors dépôt).)*

### Commits

`6781cf75` (story en cours) · `2cb3e214` (écrans, AC-7) · `4e9f9a4c` (migration, entités, exercices, sauvegarde, MCP) ·
`883b3a8a` (réglage, AC-3) · `09e71fd0` (manuels, AC-10) · `3f423eda` (devise des comptes, AC-4, AC-4 bis) · `478ca803`
(assistant, AC-11 ; guide d'API) · `e597054a` (E2E ; acteur d'audit inventorié — **dernier commit de code**).

### Gates réellement exécutés au dernier commit de code (`e597054a`)

- `cargo fmt --all -- --check` rc 0 ; `cargo clippy --workspace --all-targets -- -D warnings` rc 0 ; `cargo build
  --workspace --all-targets` rc 0 (`final2-*.log`).
- Base de gate `kesh_s37_1` remise à zéro avant (DROP/CREATE, migrations, seed).
- `scripts/test-fast.sh --ci` (nextest complet du workspace, sous les verrous) : **5547 exécutés, 5547 passés, 0 échec,
  13 ignorés**, 807,9 s (`final2-nextest.log`). Le passage précédent (`final-nextest.log`, 5546/5547) avait rougi sur
  `audit_acteur_garde` : le site d'audit de `companies::update_currency_settings` n'était pas inventorié — défaut de la
  story, corrigé dans la garde (`AdminSansCle`, la route passe `require_not_pat`), seul changement entre les deux passages.
- Frontend (au contenu frontend final ; le correctif suivant ne touche que `crates/kesh-api/tests/`) : `npm run check`
  0 erreur, 29 avertissements (fichiers non touchés) ; `lint-i18n-ownership` vert ; `test:unit` **244 fichiers, 3790
  tests passés** ; `npm run build` réussi (`final-front-*.log`).
- **E2E complet** (après le gate backend, base `kesh_e2e_s37_1` remise à zéro, recette de `docs/testing.md`, port 3371) :
  **426 passés, 12 échecs, 18 ignorés**, 34 min (`final-e2e.log`). Échecs : **7 KF-029 (#97)** attendus
  (`mode-expert` ×2, `onboarding-path-b` ×2, `onboarding` ×3) ; **5 hors liste**, sur des specs et des pages que la
  branche ne touche pas (diff contre `main`) :
  - `date-filtre-frappe.spec.ts:87` ×4 (cas « jusqu'à/au/fin ») — dépend de la locale de l'hôte : la station tourne en
    `LANG=C.UTF-8`, Chromium ordonne alors les champs date en MM/JJ malgré `locale: 'fr-CH'` ; rejouée avec
    `LANG=fr_CH.UTF-8`, les sept tests de la spec passent (`final-e2e-rejeu-lang-fr.log`) ;
  - `opening-balances.spec.ts:156` — course entre le toast « Écriture d'ouverture générée » et le bouton *Compléter*
    (le survol suspend le toast) ; échoue seule 3/3, mais **passe** dans le run de ses voisines au même code
    (`f-e2e-iter3-voisines.log`).
  Non rejoués sur `main` faute d'un second worktree : **à confirmer par l'orchestrateur** (issues demandées ci-dessous).
- La spec neuve `multi-currency.spec.ts` : 4/4, seule, avec ses voisines (25/25) et dans le run complet.

### Tests ajoutés (recomptés au diff `19bf12fb..e597054a`)

- Rust : **95** attributs `#[test]`/`#[tokio::test]`/`#[sqlx::test]` ajoutés, aucun retiré (hors 2 doctests de
  `kesh-core::currency`). Binaires neufs : `kesh-db/tests/currency_settings_repository.rs`,
  `kesh-db/tests/account_currency_repository.rs` (dont sept tests de course, par `attendre_une_requete_en_cours`),
  `kesh-api/tests/currency_settings_e2e.rs`, `kesh-api/tests/account_currency_e2e.rs` ; tests ajoutés aussi à
  `fiscal_years_repository`, `migrations_upgrade_path`, `fiscal_years_e2e`, `accounts_e2e`, `companies_e2e`,
  `mcp_outils_e2e`, `admin_full_import_e2e`, `rejeu_interblocage_e2e` (+3 victimes : `currency-settings`,
  `accounts::update`, `accounts::create`), tests lib (`currency`, `exercice`, `backup`, `csv_tables`, `onboarding`,
  `company_invoice_settings`).
- Frontend : **76** appels `it(`/`test(` ajoutés, dont **71 Vitest** et **5 E2E** (4 dans `multi-currency.spec.ts`, 1
  dans `onboarding-bank-account.spec.ts`).

### Fichiers

133 fichiers modifiés ou créés (`git diff --stat 19bf12fb..e597054a`). Principaux : migration
`crates/kesh-db/migrations/20261114000001_multi_currency_settings.sql`, `migrations.sha384`, squash
`test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md` ; `kesh-core/src/currency.rs`,
`kesh-core/src/accounting/exercice.rs` ; `kesh-db` : entités `account`, `company`, `fiscal_year`, `errors.rs`,
dépôts `accounts`, `companies`, `fiscal_years`, `company_invoice_settings`, `products`, `reconciliation_rules`,
`closing_checks`, `backup.rs`, `post_restore.rs`, `test_fixtures.rs` ; `kesh-seed/src/lib.rs` ; `kesh-api` :
`errors.rs`, `lib.rs`, `audit_labels.rs`, routes `accounts`, `companies`, `fiscal_years`, `onboarding`,
`bank_accounts`, `admin_backup/import.rs`, `exports/csv_tables.rs`, MCP `sortie.rs`, `outils/exercices.rs`,
`outils/plan_comptable.rs` ; quatre `messages.ftl` ; frontend : `features/accounts/{fx-accounts.ts,
AccountCurrencyFields.svelte}`, `features/settings/CurrencySettingsSection.svelte`, `features/onboarding/
premier-exercice.ts`, `shared/utils/iso4217.ts`, pages `settings`, `accounts`, `onboarding`, types et API ; manuels
`docs/manual/fr/{user,admin}-manual.tex`, `docs/api-external.md`, `docs/user-guide/fr/getting-started.md`.

### Étape 0 — prémisses vérifiées au code, écarts à la fiche

- `FiscalYear` : 14 listes de colonnes dans `fiscal_years.rs` (13 de production + `REFERENCE_FIGEE`, texte de test), 1
  dans `closing_checks.rs`, plus la copie de test de `reversal_next_fiscal_year.rs:555` — la fiche disait « 14 + 1 de
  production ». `Company` : 4 listes (9 sites `query_as::<_, Company>`), plus 5 littéraux `Company {` corrigés.
- `INSERT INTO fiscal_years` de production : `create_if_absent_in_tx` et `insert_fiscal_year_in_tx` (passés à `INSERT …
  SELECT … FROM companies c`, fragment partagé `NEW_FY_DERIVED_COLUMNS`) ; tests : `test_fixtures.rs:118`,
  `accounts.rs:1575`, `journal_entries.rs:4065/4263/4411`, `invoices.rs:4402` (défaut de colonne, admis).
- `NewAccount {` : 52 sites ; `AccountUpdate {` : 28 ; `bulk_create` lie `new.currency` (appelants : `None`) plutôt
  qu'un `NULL` en dur ; `bulk_create_from_chart_in_tx` écrit `NULL`.
- `REFERENCES accounts` : **19** (recompté) — `parent_id`, `journal_entry_lines.account_id`, les dix colonnes des
  réglages de facturation, `products.default_revenue_account_id`, `reconciliation_rules.counterparty_account_id`, les
  cinq colonnes de pièce laissées à la 37-3a ; lien sans FK : `bank_accounts.journal_account_id`.
- Écritures de `accounts.currency` : `create` et `update` (gardés), `bulk_create` (`None`), chargement de plan
  (`NULL`), import et remise à zéro (comptes et lignes remplacés ensemble).
- P6 : `migrations_upgrade_path` (compteur 91 → 92, soustracteur 57 → 58, frontière 34) ; neuf tests de montée qui
  sèment par le dépôt courant sur une base antérieure → helper `common::colonnes_multi_devises`. P7 : aucune donnée
  écrite. P5 : 92 migrations (en-tête et `Total`), 11 `yes`, 81 `tracked-by-sqlx`, 0 `no`. Non breaking :
  `kesh_version_min_required` inchangé.
- Date du jour de l'assistant : `state.horloge.aujourd_hui()` (Zurich, depuis la 35-5a0), non `Utc::now()` comme le dit
  l'AC-11 ; la limite assumée reste vraie avec Zurich pour référence.
- Libellé du second bouton de l'assistant : « Configurer plus tard », non « Passer ».
- Aucun composant de saisie JJ.MM.AAAA dans les écrans comptables (#880 ne vise que l'affichage) : champs `type=date`
  et une ligne de résumé en forme suisse (`formatSwissDate`).
- Société inexistante à la création d'un exercice : `NotFound` (lecture de tête de `companies`) au lieu de
  `ForeignKeyViolation` ; test renommé `missing_company_is_not_found`.

### Choix tranchés (à reporter au registre de l'epic)

- **C-37-1-1 — collation** : `accounts.currency` en `ascii_general_ci` (et non `ascii_bin`), les deux `CHECK` gardant
  `BINARY` (la casse reste refusée, test `usd`). Motif : le serveur marque une colonne `_bin` du drapeau `BINARY` et
  `sqlx-mysql` 0.8.6 refuse alors de la décoder en `String` ; le contournement par `CAST` à la lecture aurait piégé
  toute lecture future (jointures de la 37-3). Écart à la lettre de l'AC-1 ; réversible tant que non publié.
- **C-37-1-2 — numéros des comptes dans le manuel** : 6840 et 1090 ne sont pas écrits (garde G4-bis de
  `textes_coherents` : tout nombre de quatre chiffres du manuel doit être un compte d'un plan livré) ; le manuel dit où
  les créer (classe 6 sous le groupe 6, non sous 6800 ; actif « Transferts de fonds » sous le groupe 10).
- **C-37-1-3 — bornes de `kesh_plan_comptable`** : 70/40 → **68/38** comptes (334 o par compte avec `devise` ; page
  par défaut 13 729 o pour 13 312, page maximale 23 779 o pour 23 552), précédent C-32-9-44. Aucune borne citée aux
  manuels ni au guide d'API.
- **C-37-1-4 — verrous des comptes** : `accounts::create`/`update` prennent les réglages de facturation en partage à
  chaque geste (type et rôle inconnus avant le `FOR UPDATE`), sans cycle ; `ACCOUNT_IN_FOREIGN_CURRENCY` aux sites de
  configuration au seul changement de valeur (motif C4) ; parent pris `FOR UPDATE` avant l'`INSERT` d'un sous-compte.
- **C-37-1-5 — `details` des codes** : `ACCOUNT_CURRENCY_IN_USE {site, role|setting|bankAccountId|productId|ruleId}` ;
  `ACCOUNT_IN_FOREIGN_CURRENCY {accountId|null, currency}` ; `CURRENCY_UNKNOWN {value}` ; `MULTI_CURRENCY_DISABLED` et
  `ACCOUNT_CURRENCY_LOCKED` `{}` (la variante admet `fiscalYearId` pour la 37-3a) ; audit du réglage `{ before, after,
  fiscalYearIds }`.
- **C-37-1-6 — liste ISO 4217** : 152 monnaies actives côté serveur (ANG exclu, remplacé par XCG ; VED inclus), même
  liste au frontend, tenue égale par un test Vitest qui lit `currency.rs`.
- **C-37-1-7 — écran** : la colonne « Devise » du plan comptable est un badge (la page est une liste) ; les sélecteurs
  de gains/pertes excluent aussi le rôle `DefaultRevenue` ; le contrôle de l'assistant refuse aussi une date vide ou
  invalide.

### Ordre de pose des verrous par site (AC-4)

- `update` : `company_id` sur le pool → `companies` en partage (`FOR UPDATE`, premier verrou, si le `PUT` porte des
  comptes propres) → réglages en partage → compte `FOR UPDATE` (company_id comparé) → désignations propres (deux
  recherches par index, `FORCE INDEX`) → ligne d'écriture → comptes bancaires, articles, règles, comptes propres en
  lecture verrouillante.
- `create` : `companies` en partage (ou `FOR UPDATE`) → réglages en partage → parent `FOR UPDATE` → désignations du
  parent → comptes propres en partage.
- `archive_guarded` : `company_id` sur le pool → `companies` en partage → réglages en partage → compte `FOR UPDATE` →
  sous-comptes → désignations propres → désignation de facturation → version.
- Réglages de facturation : `companies` en partage → `INSERT IGNORE` → réglages `FOR UPDATE` → comptes changés en
  partage. Articles : `companies` en partage → compte en partage → article. Règles : sentinelle du projet → compte en
  partage → règle. Routes bancaires : sentinelle → réglages → `type_in_share_mode` (devise) → `bank_accounts`.
  Assistant (étape bancaire) : pré-vol sur le pool seulement — **limite assumée** (société d'un assistant en cours sans
  exercice ni compte en devise).
- `update_currency_settings` : `companies` `FOR UPDATE` → exercices `FOR UPDATE` (ordre `start_date, id`, à
  l'activation) → réglages en partage → comptes `FOR UPDATE` → articles en partage. Autres cycles couverts par le rejeu
  (37-1-CR2-P2-2) : validation de facture, avoir et saisie fournisseur prennent les réglages `FOR UPDATE` avant
  l'exercice ; l'activation les prend en partage après — routes rejouées des deux côtés.

### Registre et compteurs (recomptés depuis la source)

`LIB_ROUTES` 146 → 147, tracées 139 → 140, total du registre 149 → 150, `sans_ecriture` 119 → 120, `tout.len()`
149 → 150 ; point (vi) « vingt-cinq » → « vingt-sept » (25 handlers `SansEcritureAuJournal` sous enveloppe, plus
`seed_demo` et `reset`) ; manuel d'administration « 140 des 147 routes », « 140 + 4 + 3 = 147 ».
`admin_pat_denied_e2e` répartition (15, 38) → (15, 39). `sitesTotal` 3050 → 3093 ; `i18n-libelle-en-dur` 92 → 94
candidates, `conforme` 78 → 80 ; `CLES_RELEVEES` 434 → 441 ; `COMPTE_CHAMPS` (dates reçues) 68 → 70 ;
`MULTI_SOCIETE_TOTAL` 22 → 23.

### MCP

`kesh_exercices` : 294 o par exercice au nom de 50 caractères → **82 exercices** sous `MCP_ENVOI_OCTETS_MAX` (24 576
o) ; au-delà, `TropVolumineux` sans recours (aucune pagination) — état existant, aggravé de trois champs ; pagination
hors périmètre. V11 : aucun outil MCP n'expose la société (réglage lu par `GET /companies/current`). Les comptes de
différence propres ne sont pas dans la sortie MCP (hors périmètre).

### Hors périmètre et limites

- La copie des cours (`copied`) et `applyToNextOpening` sont des tâches de la 37-2 : `fiscal_years::create`,
  `accounts::create` et `accounts::update` sont prêts à recevoir l'appel (transaction ouverte, verrous pris, détail
  d'audit extensible) ; `update_currency_settings` n'en reçoit aucun (vide par construction).
- Refus des colonnes de pièce (lignes de facture, d'avoir, de facture fournisseur, règlements) : 37-3a.

### Demandes à l'orchestrateur (issues, non ouvertes ici)

1. `accounts::archive_account` n'est pas rejoué et prend maintenant `companies` et les réglages en partage : un flux qui
   tient un compte puis demande `companies` en exclusif (p. ex. `journal_entries::update` avec projets) pourrait
   l'interbloquer en 500 — raisonné, non mesuré.
2. Assistant : si `finalize` échoue après un enregistrement bancaire ou « Configurer plus tard » réussi, la page reste
   vide (étape 7 sans rendu) — défaut préexistant.
3. E2E `date-filtre-frappe.spec.ts` : dépend de la locale système de l'hôte (`LANG=C.UTF-8` → champs date en MM/JJ).
4. E2E `opening-balances.spec.ts:156` : course entre le toast et le bouton *Compléter* — à confirmer sur `main`.

### Écarts aux consignes de travail

Sept éditions de fichiers par commande shell plutôt que par l'outil d'édition, déclarées par les phases A1 (script
python sur `onboarding_audit_e2e.rs`, `sed -i` d'une ligne sur `entities/mod.rs`, `cat >>` en fin de
`fiscal_years_repository.rs`, `echo >>` sur `migrations.sha384`, valeur corrigée ensuite) et E (quatre remplacements
python de chaînes uniques dans `user-manual.tex`) ; résultats relus au diff. Squash régénéré avec le `mariadb-dump` du
conteneur (client de l'hôte : SSL exigé, bruit de version) ; diff final du squash : les colonnes de la migration.

### Captures (recette visuelle, thème clair)

Quatorze captures dans `kesh-gate-logs/s37-1/captures/` : plan comptable en mono-devise (liste et création sans champ
Devise), Paramètres → Comptabilité (case décochée, cochée avec les trois sélecteurs, dialogue de confirmation, actif
avec case désactivée, lecture seule pour un Comptable), création avec champ Devise (CHF, USD avec comptes « par
défaut », « propres »), badges de devise dans la liste, champ Devise verrouillé, assistant (premier exercice par
défaut ; fin = début, boutons désactivés). Les champs `type=date` y paraissent en MM/JJ/AAAA (locale de la station).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 9 MEDIUM, 5 LOW ; tous remédiés.
- 2026-11-14 — validation P2 (Sonnet) : 0 CRITICAL, 1 HIGH, 5 MEDIUM, 4 LOW dans la fiche, plus 3 inter-fiches (1 MEDIUM,
  2 LOW) ; remédiés dans la fiche sauf IF-1 (37-3a, `FOR SHARE` → `LOCK IN SHARE MODE`), IF-2 (37-3a AC-5, « ici »)
  et IF-3 (37-2, décompte du registre des routes). Les cinq choix de conception de cette passe — `currency` absent d'un
  `PUT` de compte = inchangé (et non 400, P2-4) ; lectures verrouillantes avant tout `COUNT` (P2-1) ; verrous des
  sites de configuration (P2-2) ; ordre des refus (P2-5) ; liste des comptes en devise dans `MULTI_CURRENCY_IN_USE`
  (P2-6) — sont de l'orchestrateur de la passe, à confirmer par la P3.
- 2026-11-14 — validation P3 (Opus) : 0 CRITICAL, 0 HIGH, 8 MEDIUM, 7 LOW ; remédiés dans la fiche sauf IF-1 (37-5,
  revalidation des comptes de différences de change : filet de la désignation « produit par défaut », LOW). Choix
  P2 confirmés : P2-4, P2-1, P2-6 ; P2-2 corrigé (P3-1 : validateurs hors transaction ; P3-2 : place de la lecture des
  réglages dans `update`, sans quoi cycle avec `company_invoice_settings::update`) ; P2-5 complété (P3-5). Décisions de
  cette passe : désignation « produit par défaut » d'un compte gain/perte refusée des deux côtés (P3-8) ; liens d'un
  objet archivé comptés (P3-12) ; sortie MCP `devise` toujours présente, `null` pour le CHF (P3-4).
- 2026-11-14 — validation P4 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW dans la fiche, plus 2 inter-fiches
  (2 MEDIUM) ; remédiés dans la fiche sauf IF-1 (37-2, second test du registre des routes : `sans_ecriture` +1,
  `exemptees` +1, `tout.len()` +2) et IF-2 (37-5, idem : `sans_ecriture` +1, `rejouees` +1, `tout.len()` +2).
  Points : second test de partition du registre (`the_replay_partition_…`, quatrième champ `SansEcritureAuJournal`)
  oublié (P4-1) ; lecture verrouillante de `company_id` = promotion S → X et interblocage entre deux gestes sur le
  même compte, remplacée par une lecture hors transaction (immuable) comparée sous verrou (P4-2, choix de la
  passe, à confirmer par la suivante) ; `"CHF"` en mono-devise (P4-3) ; réponse du `PUT` et test 403 (P4-4) ; sites
  de construction de `NewAccount`/`AccountUpdate` (P4-5) ; chemins des renvois `company_invoice_settings.rs` (P4-6) ;
  liste close des `reason` de `FX_ACCOUNT_INVALID` (P4-7) ; étape bancaire de l'assistant (P4-8, limite assumée).
- 2026-11-14 — validation P5 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW dans la fiche, plus 1 inter-fiches (LOW) ;
  remédiés dans la fiche sauf 37-1-P5-IF1 (37-3b, type TS `currency?` d'`AccountResponse`). Points : listes de
  colonnes de `Company` écrites à la main hors `companies.rs` (onboarding, `kesh-seed`), échec à l'exécution et non
  à la compilation (P5-1) ; `AccountResponse` aligné sur C-37-9, clés omises pour un compte CHF (P5-2) ; sites de
  construction de `NewAccount`/`AccountUpdate` recomptés (P5-3) ; pré-vol 404 de la route en tête de l'ordre des
  refus de `update` (P5-4) ; nom de l'index des lignes (P5-5). Choix P4-2 (`company_id` lu sur le pool puis
  comparé sous verrou) confirmé.
- 2026-11-14 — validation P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW (P6-1 : littéraux de tests
  d'`AccountUpdate` recomptés, 21 dans `accounts.rs` et non 20, soit 24 et non 23 ; remédié) ; 0 inter-fiches.
  Vérifiés au code : `idx_jel_account`, `routes/accounts.rs:290`/`:315`, `accounts.rs:663`, les quatre listes de colonnes
  de `Company`, 52 `NewAccount {`.
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : AC-6 annotée — la réimportation d'une sauvegarde 0.17.6
  ne vaut que jusqu'à 37-2, dont la table applicative `exchange_rate_overrides` referme la fenêtre d'import et qui
  inverse ce test en refus. Aucun autre AC touché.
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R1** — une sauvegarde antérieure à l'Epic 37 se
  réimporte (mono-devise) : liste close des tables et colonnes de l'epic dans `kesh-db::backup`, sautée par la boucle de
  couverture de `parse_and_verify_avec_inventaire` et ignorée par `creates_application_table` (la fenêtre d'import ne se
  referme pas, le registre de rejeu reste tel quel), colonnes gardées non `is_required()` par un test, test d'import
  d'une sauvegarde 0.17.6 ; l'annotation CR-1 de l'AC-6 est caduque. **R3** — mode par exercice :
  `fiscal_years.multi_currency` (défaut tiré de la société par les trois chemins de création inventoriés, exercice
  initial de l'assistant compris), activation qui passe les exercices ouverts en multi-devises, désactivation qui les
  passe tous en mono-devise, route `PUT /fiscal-years/{id}/currency-mode` (refus `FISCAL_YEAR_HAS_FOREIGN_BALANCES`,
  avertissement `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` et archivage sous `force`), écran *Exercices*. **R6** (colonnes
  seulement) — `opening_rate_kind`, `closing_rate_kind` sur `fiscal_years`, dans **cette** migration (décision : une
  seule retouche de `FiscalYear` et de ses quinze listes de colonnes ; route et écran en 37-2). **R7** — compte de
  conversion `fx_transit_account_id` (bilan, CHF, mêmes gardes que gains/pertes sauf le type, obligatoire pour
  activer, `details.designation`). **R10** — constantes de l'exercice initial de l'assistant (AC-11). Registre des
  routes : +2 routes. Prémisses relues au code : `import.rs:250-253`, `:625` ; `backup.rs:796-803` ;
  `post_restore.rs:939`, `:968`, `:988` ; `fiscal_years.rs:331`, `:412`, `:456`, `:470`, `:1360`, listes de colonnes ;
  `journal_entries.rs:332-336` ; `onboarding.rs:1044-1046`, `:1053` ; `routes/fiscal_years.rs:51`, `:286` ;
  `audit_route_registry.rs:537-539` ; `csv_tables.rs:230`. AC touchés : en-tête, Décisions, AC-1, AC-2, AC-3, AC-3 bis
  (neuf), AC-4 (compte de conversion), AC-5, AC-6, AC-7, AC-10, AC-11 (neuf), Tâches. Fiche à revalider (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; remédiés dans la fiche, aucun
  inter-fiches. CR2-P1-1 : cycle exercice → société des écritures contre l'activation (`companies` X puis exercices),
  rejeu des deux routes et registre (vi) ; CR2-P1-2 : archivage sous `force` (`archive_in_tx` extrait, imputables
  seuls) ; CR2-P1-3 : tests d'activation, de désactivation et du compte de conversion ; CR2-P1-4 : grep des listes de
  colonnes d'exercice (21 et non 15) ; CR2-P1-5 : `--diff-attendu` de la recette de montée. Prémisses relues au code :
  `import.rs:250-253`, `:625` ; `backup.rs:796-803` ; `post_restore.rs:968-1004` ; `fiscal_years.rs:331`, `:456`,
  `:1360` ; `journal_entries.rs:326-336` ; `accounts.rs:859` ; `audit_route_registry.rs` (vi) ; `onboarding.rs:1044`.
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW dans la fiche, plus 1 inter-fiches
  (LOW) ; remédiés dans la fiche sauf 37-1-CR2-P2-IF1 (fichier d'epic, arbitrage A6 : « suggestion 1091 » contre 1090
  des fiches 37-1 et 37-3b ; texte proposé : « suggestion 1090 »).
  Points : ordre 400 (corps) avant 404 (extracteur `Json`) de `currency-mode` (P2-1) ; cycles couverts par le rejeu
  — réglages `FOR UPDATE` avant l'exercice des validations de facture, avoir, saisie fournisseur, et cycle propre à
  `currency-mode` sous `force` (P2-2) ; commentaire de la constante repris de l'arbitrage A1 (P2-3) ; R6 à l'en-tête
  et aux Décisions (P2-4) ; revalidation du compte de conversion par 37-3b seule (P2-5) ; `migrations` au
  `--diff-attendu` de la recette (P2-6) ; E2E : compte de bilan à créer avant l'activation, bascule d'exercice
  (P2-7) ; contrat d'`archive_in_tx` (société comparée, version, verrous pris une fois, P2-8). Prémisses relues au
  code : `routes/fiscal_years.rs:286`, `:385-389` (`reopen`, extracteur `Json`) ; `accounts.rs:440-448`, `:859-946`
  (`archive_guarded` : aucun contrôle de solde) ; `fiscal_years.rs:162`, `:178`, `:331`, `:412`, `:456`, `:470`,
  `:1360`, 14 + 1 listes de colonnes ; `journal_entries.rs:281-336` ; `invoices.rs:698`, `:1974` ;
  `credit_notes.rs:312` ; `supplier_invoices.rs:1484` ; `post_restore.rs:938-1004` (fonctions de test) ;
  `audit_route_registry.rs:155`, `:302-308`, `:1450-1502` ; `rejeu_interblocage_e2e.rs:2497` ;
  `admin_pat_denied_e2e.rs:85` ; `onboarding.rs:1043-1053`, `:1273-1287` ; `scripts/recette-montee.sh:414-453`,
  `:651` ; plans livrés (aucun 109x). Axes non exercés : source journalière (aucun GET, la 37-1 n'y touche pas),
  manuels (PDF non lu), i18n au code.
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S2** — comptes de différence de change propres à un
  compte en devise (AC-4 bis, neuf) : `accounts.fx_gain_account_id` / `fx_loss_account_id` `NULL` (= réglage de la
  société, suivi à l'usage), sans clé étrangère (même raison que C-37-2), dans la migration de l'AC-1 avec deux
  `CHECK` (les deux ou aucun ; posables seulement sur un compte en devise) ; mêmes gardes que les comptes de la société,
  gardes inverses (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` + `holderAccountId`), résolution unique `fx_accounts_for` (pure)
  et revalidation à l'usage `validate_fx_accounts_in_tx` créées ici pour 37-5, miroir TS `fxAccountsFor` pour les écrans de 37-3b ;
  `companies` `FOR UPDATE` en tête d'un geste qui désigne (aucun cycle entre comptes) ; champ « par défaut / propres »
  du formulaire de compte (AC-7) ; liste close des absences admises étendue (`accounts` × 3, AC-6) ; CSV des comptes ;
  manuel (AC-10). **S3** — `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` porte la liste **complète** des comptes que `force`
  archiverait (`id`, `number`, `name`, `currency`), affichée par le dialogue ; `name` ajouté aussi à
  `FISCAL_YEAR_HAS_FOREIGN_BALANCES`. **S4** — R10 et l'arbitrage A1 retirés : AC-11 vidée (numéro gardé), tâche et
  Décisions alignées ; l'assistant n'est pas touché. S1, S5, S6 sans point de contact ici. Aucune route neuve.
  Prémisses relues au code : `chk_fiscal_years_dates` (`20260404000001_initial_schema.sql:51`, `end_date >
  start_date`, aucune borne de durée) ; `routes/onboarding.rs:1043-1046` ; `routes/fiscal_years.rs:264-279`
  (`borner` des dates, aucune durée) ; `backup.rs:689-736` (import sans contrôle de clé, motif repris de C-37-2).
  AC touchés : en-tête, Décisions, AC-1, AC-3 bis, AC-4, AC-4 bis (neuf), AC-6, AC-7, AC-8, AC-10, AC-11, Tâches.
  Fiche à revalider (cycle CR-3).
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche, aucun
  inter-fiches. CR3-P1-1 (MEDIUM) : la recherche du compte désignant (`… (fx_gain_account_id = ? OR fx_loss_account_id
  = ?) … LOCK IN SHARE MODE`, sans index) balaie toute la société en verrous partagés après le `FOR UPDATE` du compte
  visé : interblocage de deux archivages de comptes différents (`archive_account` non rejoué) ; deux index et deux
  recherches indexées, test de non-interblocage. CR3-P1-2 (LOW) : cycle `companies` X contre écriture (compte en partage
  puis `fk_journal_entries_company`) du `PUT` qui désigne, couvert par le rejeu existant de `accounts::update_account` ;
  écrit, avec son test de victime. Prémisses relues au code : `20260404000001_initial_schema.sql:51` ;
  `routes/onboarding.rs:1043-1046` ; `routes/fiscal_years.rs:262-280` ; `backup.rs:686-692` ; `import.rs:248-254` ;
  `accounts.rs:195-208` ; `routes/accounts.rs:337` ; `journal_entries.rs:326-336`, `:428` ;
  `audit_route_registry.rs:444-447`, `:292-296` ; `post_restore.rs:988-1004` (dans `mod tests`). Axes exercés : AC-1,
  AC-3 bis, AC-4 bis (verrous, gardes, ordre des refus), AC-6, AC-11 retirée, contacts 37-3a/37-3b/37-5 par grep.
  Non exercés : source journalière (aucun GET, hors sujet), manuels (PDF non lu), i18n au code, MCP au code.
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche, aucun
  inter-fiches. CR3-P2-1 (MEDIUM, défaut de la remédiation P1) : « `create_account` ne verrouille aucun compte déjà lu
  par une écriture » est faux — un `POST` qui porte les comptes propres **et** un `parentId` tient `companies`
  `FOR UPDATE` puis écrit le parent (`accounts.rs:204-211`, `UPDATE accounts SET postable = FALSE`), qu'une écriture
  CHF tient en partage avant de demander `companies` à son `INSERT` ; `create_account` n'a aucune enveloppe
  (`routes/accounts.rs:267`) : rejouée, liste (vi) « vingt-huit » (AC-3), test de victime. CR3-P2-2 (LOW) : le cycle
  du `PUT` (CR3-P1-2) ne se ferme qu'avec une écriture sans ligne en devise sur un compte encore CHF que le `PUT` passe
  en devise ; texte précisé pour que le test de victime soit montable. CR3-P2-3 (LOW) : `FORCE INDEX` sur les deux
  recherches du compte désignant (l'empreinte des verrous dépend du plan). Prémisses relues au code :
  `accounts.rs:153-156`, `:204-211` ; `routes/accounts.rs:267`, `:325-345` ; `audit_route_registry.rs:155`,
  `:292-310`, `:444-447`, `:1450-1502` ; `journal_entries.rs:423-427` ; 37-3a AC-4 (concurrence, `companies` pris en
  partage avant les comptes seulement si une ligne porte une devise) ; `20260411000001_accounts.sql`
  (index existants : PK, `uq_accounts_company_number`, FK) ; `20260507200001_bank_account_journal_link.sql:28`
  (`bank_accounts.journal_account_id` indexé : la lecture verrouillante de l'AC-4 ne balaie pas) ;
  `csv_tables.rs:163`, `:230`, `:265` ; `mcp/outils/plan_comptable.rs:31-43`, `mcp/sortie.rs:88-95`. Axes exercés :
  défauts de la remédiation CR-3 P1 (index, recherches, rejeu), cycles de verrous des gestes qui prennent `companies`
  en exclusif (`update_currency_settings` : déjà rejouée), contacts 37-3a/37-5 (`holderAccountId`, aucune autre fiche
  ne reprend la recherche en `OR`), CSV, MCP. Non exercés : source journalière (aucun GET, la 37-1 n'y touche pas),
  manuels (PDF non lu), i18n au code, E2E au code.
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; aucune remédiation, aucun
  inter-fiches. Diff `d9c0018e..040185ac` relu phrase à phrase. Prémisses relues au code : `accounts.rs:156`
  (`pool.begin()` propre au dépôt), `:205-211` (`UPDATE accounts SET postable = FALSE` sur le parent),
  `routes/accounts.rs:267` (`accounts::create` appelé nu), `:337` (enveloppe `accounts::update`), `NewAccount` dérive
  `Clone`, `retry.rs:40` (`retry_app_on_deadlock`), `audit_route_registry.rs:155`, `:302-309` (« vingt-cinq » ; les
  « vingt-trois handlers » à enveloppe passent à vingt-six, cohérent avec + 3), `:444-445` ; précédent de `FORCE INDEX`
  (`bank_imports.rs:474-484`, `refresh_tokens.rs:206`) ; 37-3a AC-4 (place de `companies`, cycle résiduel) ; 37-5:223
  (compte `Rejouee`, autre partition, non touché) ; aucune autre fiche ne cite `create_account`, `parentId` ni la
  liste (vi). Axes exercés : vérité au code des phrases ajoutées, cohérence des décomptes (vi) et de la partition,
  nom des index vs AC-1, renvois P2-1/-2/-3. Non exercés : exécution réelle du cycle (non montable en lecture seule),
  plan de requête MariaDB, manuels, i18n.
- 2026-11-14 — validation CR-4 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW, 0 inter-fiches ; remédié.
  CR4-P3-1 (LOW, AC-11) : résidu du symptôme de CR4-P2-3 — « date locale Europe/Zurich » subsistait à la description
  de l'écran (*Début*) : alignée sur « date du poste par `todayLocal` ». Relus et justes : `fiscal_years.rs:371-372`
  (verrou de borne, dépendant du plan), `routes/fiscal_years.rs:286` (`retry_on_deadlock("fiscal_years::create")`),
  `initial_schema.sql:48` (`fk_fiscal_years_company`), `local-date.ts:13` (`todayLocal`, champs locaux), arithmétique
  01.03.2027 → 29.02.2028, renvois à l'AC-3 et à la lecture `LOCK IN SHARE MODE`, contacts 37-2 (Zurich propre à
  ses AC). Axes exercés : vérité au code et cohérence du diff CR-4 P2, propagation du symptôme (grep). Non exercés :
  plan de requête MariaDB, exécution, manuels, i18n.
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW, 0 inter-fiches ; remédiés.
  CR4-P2-1 (MEDIUM, AC-3 bis) : « sous-requête ordinaire », « précédent non verrouillé » inexacts (l'`INSERT … SELECT`
  pose des verrous partagés sous REPEATABLE READ ; verrou de parcours de `find_overlapping`, `fiscal_years.rs:371-372`),
  et le choix de l'`INSERT … SELECT … FROM companies` dans `insert_fiscal_year_in_tx` (:1359) lit `companies` après
  les verrous d'exercice, contre l'ordre écrit pour la variante `LOCK IN SHARE MODE` : précisé (même ordre que la FK
  `fk_fiscal_years_company` aujourd'hui, `initial_schema.sql:48` ; cycle avec l'AC-3 absorbé par les rejeux,
  `routes/fiscal_years.rs:286`). CR4-P2-2 (LOW, AC-11) : « un test commun » irréalisable entre Rust et Vitest →
  mêmes cas limites des deux côtés, cas 01.03.2027 → 29.02.2028 ajouté aux tests API, formatage local du miroir.
  CR4-P2-3 (LOW, AC-11) : « date locale Europe/Zurich » → date du poste (`todayLocal`, `local-date.ts:13`).
  Prémisses relues : `repositories/fiscal_years.rs:331-391`, `:412-436`, `:456-480`, `:807-827`, `:1355-1378`,
  `routes/onboarding.rs:1043-1053` (`Utc::now()`), `local-date.ts`, `local-date-guard.test.ts`, les deux Vitest de
  l'étape (existent, mockent `finalize`), 37-2 `PUT /rate-kinds`. Axes exercés : vérité au code du diff CR-4 P1,
  arithmétique des dates (chrono/JS), ordre des verrous vs AC-3/AC-3 bis, contacts 37-2. Non exercés : plan de requête
  MariaDB réel, exécution, manuels, i18n.
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW dans la fiche, plus 1 inter-fiches (LOW) ;
  remédiés dans la fiche sauf 37-1-CR4-P1-IF1 (37-2, voir le rapport). CR4-P1-1 (MEDIUM) : « début + 1 an − 1 jour »
  et le test « 29.02.2028 → 28.02.2029 » se contredisent (`checked_add_months` de chrono donne 28.02.2029 puis 27.02.2029) :
  règle écrite (1er mars − 1 jour), cas limites partagés serveur/écran. CR4-P1-2 (LOW) : inventaire de l'AC-11 sans les
  Vitest de l'étape ; limite UTC/Zurich écrite. CR4-P1-3 (LOW) : prémisse inexacte « précédent verrouillé par les
  pré-contrôles » et `insert_fiscal_year_in_tx` en `INSERT … VALUES` (à convertir) ; corrigées. Prémisses relues au code :
  `routes/onboarding.rs:879-1060` (`finalize` sans corps, `step_completed` 7/8, `:1043-1053`), `routes/auth.rs:357` et
  `frontend/src/lib/app/stores/auth.svelte.ts:204-210` (précédent : le client envoie `{}`), `api-client.ts:239`
  (`Content-Type` JSON posé même sans corps : le client de l'écran envoie donc toujours un corps), `routes/fiscal_years.rs:86-90`,
  `:107-129`, `:259-300`, `repositories/fiscal_years.rs:331-391`, `:407-500`, `:807-827`, `:1360`, `dates_recues.rs:55`,
  `user-manual.tex:217`, `:1004` ; décomptes de `finalize` par fichier de tests (3, 4, 11, 21, 1, 2, 5 : exacts) ;
  grep `UPDATE accounts` / `INSERT INTO accounts` (inventaire de l'AC-4 exact). Axes exercés : T1, T3 (aucun reste de
  `journal_entry_lines.currency` dans cette fiche), T4, AC-6 (listes closes), manuel (`.tex` seulement), contacts 37-2/37-3b.
  Non exercés : source journalière (aucun GET, hors sujet), PDF des manuels, i18n au code, MCP au code.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T1** — AC-11 rétablie : l'assistant demande les
  dates du premier exercice (début, défaut 1er janvier de l'année courante ; fin, défaut veille du même jour un an
  plus tard), corps facultatif de `finalize` (absent = comportement actuel), nom « Exercice AAAA » ou « AAAA/AAAA »,
  refus 400 avant la transaction, écran de la dernière étape (deux boutons), inventaire clos des sites (route, DTO,
  écran, client, tests d'API et E2E, manuel) ; le multi-devises reste réglé plus tard dans les paramètres.
  ⚠️ **Prémisse de T1 fausse au code** : la fin n'est **pas** « modifiable dans *Exercices* » — le `PUT` d'un
  exercice ne change que le nom (`routes/fiscal_years.rs:86-90`) et aucune route ne supprime un exercice ; la fiche
  rend donc la fin modifiable **dans l'assistant** (question à Beat). **T3** — AC-4 : le verrou de la devise d'un
  compte porte désormais celle de ses lignes (aucune colonne de devise sur la ligne, 37-3a) ; AC-6 : liste close à
  trois colonnes de `journal_entry_lines`. **T4** — AC-1 : sens des genres (ouverture `month` = dernier mois de
  l'exercice précédent) ; AC-3 bis : un exercice qui suit un exercice Kesh (fin = veille du début) reprend son genre
  de clôture comme genre d'ouverture, par un fragment SQL partagé des trois chemins de création ; sans propagation
  ultérieure. AC-10 : manuel complété (T1, T3, T4). T2 : tests portés par 37-2, 37-3b, 37-5 (rappel à l'AC-11). T5,
  T6 sans point de contact. Prémisses relues au code : `routes/onboarding.rs:879-1060` (`finalize` sans corps,
  `:1043-1046`, `:1053`), `:220-320` et `:478` de `routes/onboarding/+page.svelte`, `onboarding.api.ts:95-96`,
  `routes/auth.rs:357` (`Option<Json<…>>`, axum 0.8), `routes/fiscal_years.rs:78-90`, `:259-300`,
  `repositories/fiscal_years.rs:456-500`, `dates_recues.rs:55`, `user-manual.tex:217`, `:1004`, `:2465` ; décomptes
  de `finalize` par fichier de `crates/kesh-api/tests` (`grep -c`). AC touchés : en-tête, Décisions, AC-1, AC-3 bis,
  AC-4, AC-6, AC-10, AC-11, Tâches. Fiche à revalider (cycle CR-4).
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — copie des cours déjà récupérés dans
  `exchange_rates` à l'activation (AC-3), à la création d'un exercice multi-devises et à la bascule vers
  multi-devises (AC-3 bis) : règles et appels portés par 37-2 AC-3 bis (la table n'existe qu'à 37-2) ; la
  désactivation ne vide pas `exchange_rates` ; liste close des absences admises sous le nom neuf `exchange_rates` (AC-6).
  **U4** — « un changement ultérieur du genre de clôture ne se propage pas » (A22) remplacé : dialogue qui propose
  d'appliquer le genre à l'ouverture de l'exercice suivant, case cochée par défaut, aucune écriture modifiée (il n'en
  existe aucune entre deux exercices Kesh) ; l'AC vit avec la route `PUT /fiscal-years/{id}/rate-kinds`, en 37-2 AC-5
  et AC-6 ; renvoi en AC-3 bis. **U5** — vérifié que les arbitrages A2–A22 (hors A11, A22) sont écrits comme AC ou
  règles de leur fiche : A4, A5, A6, A15, A18, A19 ici (inventaire aux Décisions) ; A2, A7, A9, A10, A16 en 37-2 ;
  A3, A20, A21 en 37-3a ; A8, A12, A13, A14, A21 en 37-3b ; A17 en 37-3a et 37-3b. **U6** — « en CHF » du solde
  contrôlé à la bascule écrit « en devise principale, aujourd'hui CHF » (AC-3 bis) ; les autres « en CHF » de la fiche
  désignent un compte sans devise (exemple concret) et restent. AC touchés : en-tête, Décisions, AC-3, AC-3 bis,
  AC-6, Tâches. Aucune migration ni route neuve. Fiche à revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW dans la fiche, plus 2 inter-fiches
  (37-2 : 1 MEDIUM, 1 LOW) ; remédiés dans la fiche sauf 37-1-CR5-P1-IF1 et -IF2 (37-2, voir le rapport).
  CR5-P1-1 (MEDIUM) : A24 non appliqué (copie « des unités utiles » de toutes les devises ; aucun déclencheur quand un
  compte reçoit une devise) : copie limitée aux devises des comptes (AC-3 bis), nouvelle puce d'AC-4 (lecture simple
  des exercices, ordre des verrous des écritures, filet `POST /fetch`), tests portés par 37-2. CR5-P1-2 (MEDIUM) :
  la copie à l'activation est vide par construction (aucun compte en devise tant que la société est mono-devise,
  AC-4 et désactivation de l'AC-3) : retirée d'AC-3, tâche CR-5 corrigée. CR5-P1-3 (LOW) : U4 sans « s'il est
  ouvert » ni « seul le genre de clôture » (37-2 AC-5) : aligné. CR5-P1-4 (LOW) : `copied` de `fiscal_year.created`
  non spécifié : écrit à côté du snapshot. CR5-P1-5 (LOW, U6) : « ne connaissant que le CHF » et « solde CHF » du
  solde contrôlé à la bascule : « devise principale ». Relu au code : `repositories/fiscal_years.rs:331-391`
  (`create`, transaction propre, `insert_fiscal_year_in_tx` :1355, `snapshot_json` :279), `accounts.rs:57-72`
  (`account_snapshot_json`), `routes/accounts.rs:337`, `routes/fiscal_years.rs:286`, `opening_balances.rs:240`, `:379`
  (`find_first_by_company`). Aucun reste de `exchange_rate_overrides`, `_kesh_exchange_rates` ni `republished_*` hors
  historique et mentions « ex- » (grep, six fiches). Axes exercés : U1 (sites sauvegarde, rejeu, `TABLES_TO_TRUNCATE`,
  CSV, remise à zéro — portés par 37-2 AC-1/AC-7, cohérents avec AC-5 et AC-6 ici), U4, U5 (A4, A5, A6, A15, A18, A19
  présents), U6, A24, A27, A28 (sans objet ici). Non exercés : source journalière (aucun GET, la 37-1 n'y touche pas),
  manuels (PDF non lu), i18n au code, MCP au code, exécution des verrous.
- 2026-11-14 — validation CR-5 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW dans la fiche, plus 1
  inter-fiches (37-2, LOW) ; remédiés dans la fiche. CR5-P2-1 : l'en-tête disait « la première devise donnée à un
  compte », l'AC-4 copie aussi au changement de devise d'un compte sans ligne : aligné. CR5-P2-2 : Décisions (U1)
  rangeait l'AC-3 parmi les sites d'appel, qu'elle exclut : reformulé. CR5-P2-3 : l'AC-3 bis (U4) ne proposait le
  dialogue que pour un exercice suivant ouvert ; 37-2 AC-6 l'affiche aussi, case désactivée, pour un suivant clos :
  aligné. CR5-P2-4 : `copied` « à côté du snapshot de `account.updated` » — `update` écrit `{ before, after }` et peut
  auditer `account.retyped` (`accounts.rs:792-804`) : précisé. CR5-P2-5 : renvoi faux, « 37-2 AC-3 bis ajoute un
  paramètre `currencies` » — la signature y est `copy_fetched_in_tx(conn, company_id, units)` (37-2 :376), la
  restriction à une devise n'y est qu'en prose : reformulé, IF1 signalé. CR5-P2-6 : la sous-requête `accounts` de la
  copie (37-2) lit en partage les comptes en devise ; deux poses concurrentes sur deux comptes s'interbloquent, cycle
  non écrit mais couvert par les deux enveloppes de rejeu (`routes/accounts.rs:337`, AC-4 bis) : écrit. Inter-fiches
  non appliqué : 37-1-CR5-P2-IF1 (37-2 AC-3 bis) — écrire le paramètre de devises de `copy_fetched_in_tx` (appel des
  routes de comptes : la seule devise posée, ce qui rend vrai le test « aucune EUR ») et le cycle de CR5-P2-6 avec son
  test de victime. Relu au code : `accounts.rs:57`, `:227`, `:792-815`, `fiscal_years.rs:279`, `:389`,
  `routes/accounts.rs:337`. Axes exercés : diff `d59a1de9..3b058509` de la fiche, phrase à phrase ; A24, A27, U1, U4,
  U6 contre le registre de l'epic (:262, :314-318) ; points de contact 37-2 AC-3 bis, AC-5, AC-6 et 37-3a AC-4
  (ordre des verrous) ; noms d'audit au code. Non exercés : manuels, i18n, exécution des verrous (analyse seule),
  autres fiches que 37-2 et 37-3a.
- 2026-11-14 — révision CR-6 (couverture API et MCP, Opus) : **V6** — AC-3 : le mode multi-devises s'active par une
  session d'Administrateur (`POST /auth/login`, `accessToken` en `Authorization: Bearer`), jamais par une clé d'API —
  non « à l'écran » ; test d'API en Bearer sans cookie, tâche, CHANGELOG. **V10** — AC-2 : `kesh_exercices` gagne
  `multi_devises`, `genre_ouverture`, `genre_cloture` (type scellé comme `statut`), description, test, mesure de taille
  (outil non paginé) ; la justification « hors périmètre » antérieure était circulaire. **V11** — AC-8 : aucun outil
  MCP n'expose la société (existant), hors périmètre dit. **V3** (renvoi) — AC-4 bis : la route de conversion de 37-3b
  passe désormais par `fx_accounts_for` puis `validate_fx_accounts_in_tx`. Prémisses relues au code : `lib.rs:518`
  (`require_not_pat` sur `admin_routes`), `:1353` (`/auth/login`) ; `routes/auth.rs:184-186` (`LoginResponse`,
  `accessToken`) ; `middleware/auth.rs:83-95` (cookie puis Bearer) ; `middleware/rbac.rs:58-66` ;
  `mcp/outils/exercices.rs:23-29`, `:42-46`, `:69-75` ; `mcp/outils/mod.rs:9-20` (aucun outil société) ;
  `tests/mcp_outils_e2e.rs:99` (`bornes_attendues`, sans `kesh_exercices`), `:312` (portée `books:read`), `:1345` ; `services/envois_mcp.rs:55`. AC
  touchés : AC-2, AC-3, AC-4 bis, AC-8, Tâches. Fiche à revalider (cycle CR-6).
- 2026-11-14 — validation CR-6 P1 (Sonnet) : 4 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW) ; remédiés dans la
  fiche, aucun inter-fiches. **CR6-P1-1** (MEDIUM) — AC-2 : le type scellé de `kesh_exercices` exige l'inscription à
  `champ_enum!` (`mcp/sortie.rs:131-143`), énumération sans champ, valeurs `day`/`month`. **-2** (LOW) — AC-2 :
  `tailles_mesurees` ne s'applique pas (pas de `limit`, `SansArgument` en `deny_unknown_fields`) → test dédié ; la
  conséquence `TropVolumineux` (`sortie.rs:463`) écrite. **-3** (LOW) — AC-4 bis : la route de conversion (V3) ajoutée
  aux appelants de `validate_fx_accounts_in_tx` et son cycle avec la bascule `force` aux « autres cycles » rejoués.
  **-4** (LOW) — V6 n'avait qu'une tâche : l'exigence passe à l'AC-10 (manuel utilisateur, d'administration,
  CHANGELOG). Prémisses relues au code et justes : `lib.rs:518`, `:1353` ; `routes/auth.rs:186` ; `middleware/auth.rs:83-95` ;
  `rbac.rs:58-66` ; `mcp/outils/exercices.rs:23-29`, `:42`, `:69-75` ; `mcp/outils/mod.rs` (aucun outil société) ;
  `mcp_outils_e2e.rs:312`, `:1345` ; `envois_mcp.rs:55`. Axes exercés : prémisses de la révision, V6, V10, V11, renvoi V3,
  cohérence avec 37-3b AC-2 bis, choix (b) et (c) vus depuis 37-1 (corrects : la paire résolue fait échouer fermé).
  Non exercés : contrats V1/V2/V3 de 37-3a et 37-3b en détail (choix (a), (d), (e)), 37-4, 37-5, exécution des tests.
- 2026-11-14 — validation CR-6 P2 (Opus) : 3 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW) ; remédiés dans la fiche,
  aucun inter-fiches. **CR6-P2-1** (MEDIUM) — AC-10 : V10 étend `kesh_exercices`, mais la matrice MCP du manuel
  d'administration (`admin-manual.tex:3447`, « les exercices (identifiant, nom, dates, statut) ») restait inchangée,
  et `mcp_matrice_manuel.rs` ne lit que les colonnes outil et portée → ligne à compléter, tâche. **-2** (MEDIUM) —
  AC-10 (remédiation P1-4) : renvoi `admin-manual.tex:3217` faux (décompte du journal d'audit) ; le lieu est le point
  *Administration* (`:3183`), dont la phrase « Ces opérations se font dans l'interface web » contredit V6 → amendée,
  réglage ajouté à l'énumération. **-3** (LOW) — AC-3 (remédiation P1-3) : « ces comptes » faisait prendre
  `FOR UPDATE` les comptes de différence par la bascule, qui n'archive que les comptes en devise (CHF exclus) →
  reformulé ; renvoi `sortie.rs:463` précisé (`:451-466`). Prémisses relues au code : `mcp/sortie.rs:121-143`
  (`champ_enum!`, invocation globale à `:132`, précédents locaux `postes_ouverts.rs:75`, `commun.rs:203`), `:451-466` ;
  `mcp/outils/exercices.rs:17-29`, `:62-75` (`SansArgument` en `deny_unknown_fields`) ; `mcp_outils_e2e.rs:99`, `:1344-1356` ;
  `envois_mcp.rs:54` ; `middleware/auth.rs:77-100` ; `entities/fiscal_year.rs:17-19` (`FiscalYearStatus` en
  `PascalCase` : le genre en `day`/`month` est une sérialisation propre, sans conflit) ; `admin-manual.tex:3183`,
  `:3217`, `:3447` ; 37-3b AC-2 bis (verrous de V3, cohérents). Axes exercés : remédiation P1 (P1-1 à P1-4), V6, V10,
  V11, MCP (`champ_enum!`, test de taille, contrat, matrice du manuel), RBAC Bearer, cycle avec la conversion V3.
  Non exercés : AC-1, AC-3 bis à AC-7, AC-11 hors CR-6 ; contrats V1/V2 de 37-3a ; exécution des tests.
- 2026-11-14 — arbitrages A31/A32 et reports IF1/IF2 (orchestrateur, Sonnet) : AC-4 bis — « sur demande (A31) » ajouté
  à la phrase sur la ligne de différence calculée par la route de conversion (37-3b AC-2 bis, `completeDifference`).
- 2026-11-14 — validation CR-6 P3 (Sonnet) : 1 finding (0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW) ; remédié dans la fiche,
  aucun inter-fiches. **37-1-CR6-P3-1** (LOW) — AC-3 (cycle avec la conversion) et AC-4 bis : la remédiation P2-3 disait
  que la conversion lit « le compte en devise de l'écriture à compléter », « sur le seul compte en devise » ; 37-3b
  AC-2 bis lit en partage **les comptes des lignes** de l'écriture à compléter (reconnaissance de la ligne de
  différence, avant la forme exigée), puis la paire à l'usage ; et la phrase d'AC-4 bis, après A31, taisait le cas du
  corps qui porte une ligne sans la demande (paire contrôlée) et celui des deux ensemble (400) → alignés sur 37-3b.
  Prémisses relues au code : `mcp/sortie.rs:60-84` (`bool` est `ChampSur`), `:121-143`, `:451-466` ;
  `mcp/outils/exercices.rs:17-29`, `:42-46`, `:69-75` ; `mcp_outils_e2e.rs:1344-1356` ; `envois_mcp.rs:54-55` ;
  `middleware/auth.rs:77-100` (Bearer en repli) ; `routes/auth.rs:186` (`access_token` au corps du login) ;
  `admin-manual.tex:3183`, `:3447` (aucun autre site ne répète « identifiant, nom, dates, statut » hors la description
  de l'outil, `exercices.rs:43`, que la tâche V10 vise déjà) ; 37-3b AC-2 bis (verrous, A31). Axes exercés : diff P2
  `ff3ea962..f46de99c` et révision, V6, V10, V11, MCP (`champ_enum!`, taille, matrice du manuel), RBAC Bearer, cycle
  avec la conversion V3 sous A31. Non exercés : AC-1, AC-3 bis à AC-7, AC-11 ; A32 (37-3a/3b, sans effet sur cette
  fiche) ; exécution des tests.
- 2026-11-15 — exigences FR/NFR ajoutées (B26 ; orchestrateur, Sonnet) : 11 FR, 7 NFR.
- 2026-11-15 — validation ciblée des exigences (Opus) : 11 MEDIUM, 5 LOW, tous corrigés. MEDIUM : FR-3 (gardes,
  `FX_ACCOUNT_INVALID`, `MULTI_CURRENCY_IN_USE`, exercices à l'activation) ; FR-4 (bascule refusée **vers mono-devise**
  seulement, aussi sur une ligne dans l'exercice ; avertissement et `force` ; `MULTI_CURRENCY_DISABLED`,
  `FISCAL_YEAR_CLOSED` ; genre d'ouverture T4 ; écran *Exercices*) ; FR-5 (refus de la devise d'un compte) ; FR-6
  (gardes à la désignation, revalidation) ; FR-9 (écran *Plan comptable* omis) ; FR-11 (défauts et refus 400) ;
  NFR-2 (RBAC de `currency-mode`, session Bearer) ; NFR-3 (JSON identique en mono-devise, C-37-9) ; gardes inverses
  `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` sans exigence → **FR-37-1-12** ; audit sans exigence → **NFR-37-1-8** ; verrous et
  rejeu sans exigence → **NFR-37-1-9**. LOW : FR-1 (contrôles du schéma), FR-2 (noms des champs MCP), NFR-4 (« reset
  sans perte »), NFR-5 (« reste sous les tailles » : re-mesurées), NFR-7 (manuel d'administration, CHANGELOG) ; FR-8
  précisé. Décompte : 12 FR, 9 NFR (+1, +2).
- 2026-11-15 — validation ciblée P2 des exigences (Sonnet) : 1 MEDIUM, 6 LOW, tous corrigés. MEDIUM : FR-12 (le refus du rôle `DefaultRevenue` et du compte de produit par défaut ne vise pas le compte de conversion, AC-4). LOW : FR-3 (même ambiguïté), FR-4 (genre de clôture `day`, no-op, `FISCAL_YEAR_CLOSED` des deux sens), FR-6 (conversion réglage de la société seul), FR-11 (écran de l'assistant, absence de borne de durée), FR-12 (archivage du compte désignant admis), NFR-9 (test de victime de `accounts::update`). Décompte : 12 FR, 9 NFR (inchangé).
- 2026-11-15 — validation ciblée P3 des exigences (Opus) : 0 MEDIUM, 3 LOW, tous corrigés. LOW : FR-4 (ordre des refus de la bascule ramené à celui de l'AC-3 bis — `MULTI_CURRENCY_DISABLED` → `FISCAL_YEAR_CLOSED` → no-op ; ensemble de l'avertissement et de `force` précisé : imputables, sans ligne après la fin, sans sous-compte actif, sans plafond) ; FR-12 (compte de conversion désigné par la société seule, AC-4 bis ; le correctif P2 disait que le refus du rôle `DefaultRevenue` ne le vise pas, alors que l'AC-4 lui applique « mêmes refus » sauf le type et la désignation « produit par défaut » — récrit : le rôle lui est fermé par son type, `accepts_account_type`) ; FR-12 (`details.designation` à trois valeurs, `holderNumber`). Récapitulatif de l'epic concordant. Décompte : 12 FR, 9 NFR (inchangé).
- 2026-11-15 — notes de dépendance de l'Epic 38 (orchestrateur, Sonnet) : AC-4 / C-37-9, levée de `ACCOUNT_IN_FOREIGN_CURRENCY` pour la règle de réconciliation (38-4, O3) et colonnes désignées portées à onze (38-5) ; aucun changement de comportement de 37.
- 2026-11-16 — révision CR-7 (A34 multi-devises sans retour, Opus) : section « Révision CR-7 » en tête (modèle :
  réglage de la société irréversible, refus 409 `MULTI_CURRENCY_IRREVERSIBLE` neuf ; `fiscal_years.multi_currency`
  gardée, posée à la création et à l'activation seulement, protège un exercice clos avant l'activation puis rouvert).
  Retirés : route `PUT /fiscal-years/{id}/currency-mode` (`set_currency_mode`, `force`, audit
  `fiscal_year.currency_mode_updated`, rejeu, registre), codes `MULTI_CURRENCY_IN_USE`,
  `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`, désactivation (comptes à `NULL`,
  exercices à `FALSE`), `COUNT` des comptes en devise, `archive_in_tx`, garde de `reactivate`, écran *Exercices*
  (colonne, action, dialogue) et leurs tests, A27. Ajoutés : confirmation de l'activation à l'écran (AC-7), test du
  refus de désactivation (AC-3). Registre des routes : +1 route (au lieu de +2), point (vi) « vingt-sept » (+2). FR-3,
  FR-4, FR-9, FR-12, NFR-2, NFR-8, NFR-9 réécrites ; AC-3, AC-3 bis, AC-4, AC-4 bis, AC-7, AC-10, Tâches. Schéma, liste
  close des absences (AC-6) et fenêtre d'import inchangés. Décompte : 12 FR, 9 NFR (inchangé).
- 2026-11-16 — validation CR-7 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW. MEDIUM 37-1-CR7-P1-1 : la limite
  assumée disait « créé… puis rouvert », ordre impossible au code (`fiscal_years::create` refuse un exercice avant un
  exercice clos, `LaterFiscalYearClosed`, `fiscal_years.rs:363-380`) et taisait sa conséquence (solde en devise porté
  dans un exercice mono-devise que 37-5 ne réévalue pas) — récrite, test et phrase du manuel (AC-10) ajoutés.
  -P1-2 : AC-4 bis, *Rejeu*, s'appuyait sur le verrou partagé de `companies` des écritures en devise de 37-3a, retiré
  par D-CR7-1 — récrit (aucune écriture ne prend `companies` avant les comptes ; le cycle des écritures à ligne CHF
  sur le compte rendu en devise reste couvert par l'enveloppe de `accounts::update`). -P1-3 : AC-4 bis, *Rejeu*,
  inscrivait `create_account` à la liste (vi) « vingt-huit » (reste de CR-3), contre « vingt-sept » de l'AC-3 — corrigé.
  LOW -P1-4 : Décisions et en-tête (S3, A4, A5, A27, R3, U1) citaient encore `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`,
  `force`, la désactivation et la bascule comme en vigueur — annotés « retiré par CR-7 ». -P1-5 : « jamais mono-devise »
  sans la réinitialisation de la société (AC-5) — précisé. -P1-6 : test du `false` avec identifiant non nul (400
  `unexpected` avant 409) ajouté à l'AC-3. KISS (signalé, non modifié) : la lecture de `multi_currency_enabled` sous
  `companies` en partage (AC-4) n'est plus un filet nécessaire (réglage monotone) mais l'est pour les colonnes FX ;
  le mécanisme `fiscal_years.multi_currency` ne protège plus qu'un seul cas (exercice clos avant l'activation, rouvert).
  Axes exercés : résidus (grep des neuf termes), AC-3, AC-3 bis, AC-4, AC-4 bis, AC-6, AC-7, AC-10, registre des
  routes, protection d'un exercice antérieur, D-CR7-1, code lu (`fiscal_years.rs` create/reopen). Non exercés : code
  applicatif au-delà de `fiscal_years.rs`, PDF, i18n.
- 2026-11-16 — validation CR-7 P2 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés. MEDIUM 37-1-CR7-P2-1
  (D-CR7-2, KISS) : la « limite assumée » écrite en P1 (exercice créé dans un trou devant un exercice clos avant
  l'activation puis rouvert), son test et sa phrase au manuel sont retirés ; `fiscal_years::create` refuse, dans une
  société multi-devises, un exercice qui précède un exercice mono-devise — 400 `LATER_FISCAL_YEAR_MONO_CURRENCY` (code
  voisin de `LATER_FISCAL_YEAR_CLOSED`, mêmes `details`, `DbError::LaterFiscalYearMonoCurrency`, `map_create_error`
  `routes/fiscal_years.rs:146`, réponse comme `later_fiscal_year_closed_response` `errors.rs:1438` ; pas le même code :
  l'exercice visé peut être ouvert, « rouvrir » n'y ferait rien), après la garde de l'invariant I
  (`fiscal_years.rs:363-380`) et par une requête propre (`FIND_LATER_CLOSED_SQL` a d'autres appelants, `:878-905`).
  Prémisse vérifiée au code : la garde a besoin du mode de la société **avant** l'`INSERT` — `create` le lit donc
  `LOCK IN SHARE MODE` en tête de transaction ; la note qui disait le partage de `companies` pris après les verrous
  d'exercice et le cycle avec l'activation « absorbé par les deux rejeux » est récrite (ordre `companies` → exercices,
  plus de cycle entre elles). Révision CR-7, FR-4, AC-3 bis (puce et tests : refus, ordre clos/mono, société
  mono-devise inchangée), AC-10 et Tâches mis à jour. LOW 37-1-CR7-P2-2 (Epic, mission 37-1) : A5 annoté « retiré par
  CR-7 » ; « Points à faire valider » de la section CR-7 récrits (D-CR7-1 appliqué, D-CR7-2, D-CR7-3). Axes exercés :
  D-CR7-1 (renvois de 37-1 aux verrous des écritures : AC-3 *rejeu*, AC-4, AC-4 bis *Rejeu*, cohérents), D-CR7-2 (code
  lu : `fiscal_years::create` :331-395, `find_later_closed_in_tx` :865-893, `errors.rs` DbError :725-736 et réponse
  :1438-1458, `routes/fiscal_years.rs` :143-153, `create_if_absent_in_tx` :438-456), résidus du modèle réversible
  (grep : désactivation, bascule, `force`, `currency-mode`, `archive_in_tx`, codes retirés, « limite » — seuls restent
  la liste « Retiré », les annotations datées et les Change Logs), KISS (aucun mécanisme restant propre à la
  réversibilité trouvé ; lecture de `multi_currency_enabled` sous le partage de `companies` gardée, le partage restant
  nécessaire aux colonnes FX), renvois 37-2/37-3a/37-5 sur D-CR7-2. Non exercés : PDF et manuels, i18n, code au-delà
  des sites cités, registre des routes (inchangé par D-CR7-2 : aucune route). Décompte : 12 FR, 9 NFR (inchangé ;
  FR-4 complétée).
- 2026-11-16 — validation CR-7 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié. LOW 37-1-CR7-P3-1 (AC-3 bis, D-CR7-2 et tâche CR-2) : « réponse construite comme `later_fiscal_year_closed_response` » laissait croire à une copie du gabarit, dont le code `LATER_FISCAL_YEAR_CLOSED` est écrit en dur (`errors.rs:1438-1450`) et dont le doc-comment dit « un seul gabarit » ; la variante `AppError` (comme `FiscalYearBeforeClosedYear`, `errors.rs:358`) manquait aux tâches — gabarit généralisé d'un paramètre `code`, variante nommée. Vérifié au code : `fiscal_years::create` :331-395 (garde de l'invariant I après les pré-contrôles, la garde D-CR7-2 s'y insère à sa suite), `map_create_error` :143-153, rejeu :286, `create_if_absent_in_tx` (aucun exercice), clé `error-fiscal-year-create-later-closed` (messages.ftl:1087, la nouvelle clé en suit le nommage). Raisonnement contrôlé : préfixe mono-devise (invariant I à l'activation, aucune bascule, refus de création avant un mono), ordre de verrouillage companies → exercices sans cycle avec l'activation, lecture simple de l'exercice postérieur sûre sous le partage de `companies`, ordre des deux refus testé, société mono-devise inchangée, limite assumée retirée (grep : ne restent que la liste « Retiré » et des Change Logs datés). Axes exercés : AC-3 bis, FR-4, AC-10, tâches, NFR-9, résidus. Non exercés : PDF, i18n, reste de la fiche. Décompte : 12 FR, 9 NFR (inchangé).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : PDF des manuels non commités (`.tex` seulement, régénération à la release ; 4 remplacements) ; CHANGELOG remplacé par la section *Note de version* (4 remplacements, section ajoutée) ; compteurs « recomptés depuis la source après chaque reprise de `main` » (2 ajouts). Fond des AC inchangé.
- 2026-11-16 — développement (dev-story, Opus 5.5, par phases A1, A2, B, C, D, E, F) : AC-1 à AC-11 livrées ; gate complet au dernier commit de code `e597054a` — nextest 5547/5547 (13 ignorés), Vitest 3790 (244 fichiers), E2E 426 passés, 12 échecs (7 KF-029, 5 hors liste sur des specs non touchées, à confirmer sur `main`), 18 ignorés ; choix C-37-1-1 à C-37-1-7 (collation `ascii_general_ci`, comptes du manuel sans numéro, bornes MCP 68/38…) ; détail au Dev Agent Record. Statut → `review`.
- 2026-11-16 — revue de code P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW (1 remédié, 2 tracés). 37-1-CR-P1-1 (LOW, `routes/accounts.rs`, remédié) : la devise refusée en `CURRENCY_UNKNOWN` était reprise sans borne dans le message et les `details` ; tronquée à 32 caractères, test `unknown_currency_echo_is_bounded`. 37-1-CR-P1-2 (LOW, non remédié, à tracer) : `PUT /accounts/{id}/archive` n'est pas rejoué sur interblocage alors qu'`archive_guarded` prend désormais `companies` et les réglages en partage (demande n° 1 du Dev Agent Record, raisonnée et non mesurée) ; le rejouer change la partition `Rejouee` du registre (27 → 28), le manuel d'administration et exige un test de victime. 37-1-CR-P1-3 (LOW, non remédié) : `finalize` valide les dates du corps avant la lecture de l'étape, de sorte qu'une finalisation déjà faite rejouée avec des dates invalides rend 400 au lieu de l'état. Écarts du dev : C-37-1-1 (collation `ascii_general_ci`) acceptée sur décision de l'orchestrateur ; C-37-1-2 (6840 et 1090 non cités) acceptée ; C-37-1-3 à 7 jugés sans objection. Recomptés à la source (périmètre `19bf12fb..e597054a`) : 95 attributs de test Rust ajoutés et 0 retiré, 76 appels `it(`/`test(` frontend (71 + 5 E2E), 92 migrations, 11 `yes` et 81 `tracked-by-sqlx`. Gate ciblé exécuté : fmt, clippy workspace (`-D warnings`), nextest `kesh-api` lib 687/687 (5 ignorés) et binaires `account_currency_e2e`, `audit_route_registry`, `textes_coherents`, `dates_recues_garde` 57/57 ; aucun patch ne touche `kesh-db`, gate complet non rejoué.
- 2026-11-16 — revue de code P2 (Opus ; couches Blind Hunter, Edge Case Hunter, Acceptance Auditor en parallèle, contexte frais) : 0 CRITICAL, 0 HIGH, **3 MEDIUM**, 7 LOW (9 remédiés, 1 tracé), plus 37-1-CR-P1-3 remédié ; 3 rejetés. 37-1-CR-P2-1 (MEDIUM, `repositories/accounts.rs` `check_own_fx_accounts_in_tx`, relevé par les trois couches) : un compte CHF qui reçoit une devise dans le même `PUT` pouvait se nommer lui-même gain ou perte (lu avant l'`UPDATE`, encore CHF) ; refusé en `foreignCurrency` sur l'état résultant, l'écran ne propose plus le compte modifié (`AccountCurrencyFields`, prop `accountId`) ; tests `comptes_propres_un_compte_ne_se_designe_pas_lui_meme` et Vitest (mutation du filtre vérifiée : rouge). 37-1-CR-P2-7 (MEDIUM, `accounts::reactivate`) : réactiver le sous-compte archivé d'un compte désigné (repassé imputable puis désigné) lui rendait un sous-compte actif ; `reactivate` lit la société sur le pool, prend `companies` en partage en premier verrou, le parent en partage, et refuse en `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` ; test `reactivation_sous_un_compte_de_difference_refusee` (société et propre). 37-1-CR-P2-8 (MEDIUM, `accounts::create`) : un compte en devise créé sous un parent qu'il nomme compte propre rendait ce compte désigné non imputable ; refusé en `notPostable` ; test `creation_sous_son_propre_compte_de_difference_refusee`. LOW remédiés : P2-3 message propre de `unexpected` pour les comptes propres (`error-fx-account-unexpected-own`, quatre locales) ; P2-4 la section *Comptabilité multi-devises* gardait mal les choix non enregistrés quand une autre section enregistrait (rechargement seulement si le réglage a changé ou si rien n'est modifié ; deux Vitest) ; P2-5 message du compte de conversion (`error-account-is-fx-transit-account`, quatre locales) ; P2-6 Note de version complétée (champs, corps de `finalize`, codes, bornes MCP 38/68, migration) ; P2-9 manuel utilisateur : la devise affichée à côté du numéro (et non une colonne), le compte de différence désigné sorti de la liste d'`ACCOUNT_CURRENCY_IN_USE` (son code est `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) ; P2-11 doc de `check_fx_accounts_in_tx` (comptes propres lus en partage, non `FOR UPDATE`). LOW tracé (P2-10, non remédié) : deux tests de course ne discriminent pas la clause qu'ils semblent prouver — `course_changement_de_devise_contre_designation_de_facturation` reste vert sans le partage anticipé des réglages dans `update`, `course_changement_de_devise_contre_premiere_ligne` / `_contre_lien_bancaire` sans `LOCK IN SHARE MODE` (ils prouvent l'attente du `FOR UPDATE` du compte) ; aucun défaut de production. 37-1-CR-P1-3 remédié : le refus des dates de `finalize` est rendu après la lecture de l'étape sous verrou (`RefusPremierExercice`), une finalisation déjà faite ignore le corps même invalide ; assertions ajoutées à `path_b_finalize_refuse_les_dates_invalides_sans_rien_creer`. Rejetés : rejeu absent de `accounts::update` (faux : `retry_app_on_deadlock("accounts::update", …)`, `routes/accounts.rs:509`) ; corps illisible de `finalize` replié sur le défaut (faux sous axum 0.8, `crates/kesh-api/Cargo.toml:17`) ; relance de `finalize` sans dates (les deux appels passent `finalizeBody()`, `+page.svelte:342, 360`). 37-1-CR-P1-2 (`archive_account` non rejoué) : aucun cycle concret trouvé, laissé à l'issue. Axes non exercés : E2E Playwright, captures (non reprises), traductions de/it/en relues seulement pour les clés ajoutées, MCP et exports seulement par l'Acceptance Auditor. Gates (au contenu commité) : base `kesh_s37_1` remise à zéro (DROP/CREATE, migrations, seed) ; `cargo fmt --check` vert ; `cargo build --workspace --all-targets` vert ; `scripts/test-fast.sh --ci` (clippy `-D warnings` + nextest complet) **5551 exécutés, 5551 passés, 13 ignorés** ; frontend `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert, `test:unit` 244 fichiers / 3793 tests, `npm run build` vert ; **E2E complet non rejoué** (à faire au dernier commit de code). Tests ajoutés (périmètre `822fe8b5..` ce commit) : 3 Rust (`#[sqlx::test]`), 3 Vitest, assertions ajoutées à 3 tests existants. Trois MEDIUM : une passe P3 est due.
- 2026-11-16 — revue de code P3 ciblée (Sonnet, une lentille sur `a7a1d70a`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW (1 remédié, 1 tracé). 37-1-CR-P3-1 (LOW, `accounts.rs` `guard_currency_update`, remédié) : les comptes propres posés dans une société encore mono-devise rendaient le message « propre … sur un compte en devise » (P2-3, discriminé par la présence du détenteur) au lieu de celui du mode ; refusé `unexpected` sans détenteur ; test `comptes_propres_en_mono_devise_sans_detenteur`. 37-1-CR-P3-2 (LOW, non remédié, à tracer) : `POST /accounts` en société multi-devises, comptes propres sans devise, rend encore le message du mode (la création n'a pas de détenteur pour discriminer ; une raison neuve changerait la liste close `details.reason`) ; et `PUT /accounts/{id}/reactivate` n'est pas rejoué sur interblocage alors qu'il prend `companies` en partage (même famille que 37-1-CR-P1-2 ; aucun cycle concret trouvé : `companies` S, parent S, désignations S, puis le compte X — aucun autre geste ne tient l'un en attendant l'autre). Axes exercés : ordre des verrous de `reactivate` contre create/update/archive_guarded/réglages, état résultant des comptes propres, `finalize` (refus après lecture de l'étape), rechargement de la section réglages, i18n quatre locales (clés ajoutées), manuel `.tex`, propagation du symptôme `unexpected`. Non exercés : E2E Playwright, MCP, concurrence mesurée. Gates (au contenu commité) : base `kesh_s37_1` remise à zéro ; fmt et clippy `-D warnings` verts ; `scripts/test-fast.sh --ci` **5552 exécutés, 5552 passés, 13 ignorés** (un premier run rouge — `accounts_e2e::account_response_porte_devise_verrou_et_comptes_propres` — a révélé que le garde mono-devise s'appliquait aussi aux comptes propres inchangés ; restreint aux comptes propres posés) ; Vitest ciblé (2 fichiers de P2) 28/28, frontend non modifié ; E2E Playwright non rejoué. Tests ajoutés (périmètre `a7a1d70a..` ce commit) : 1 Rust. Aucun finding au-dessus de LOW : boucle close sur cette passe, la remédiation touchant pourtant la production (une garde de 8 lignes), relue par le gate complet seulement.
