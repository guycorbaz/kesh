# Story 37-5 : réévaluation des comptes en devise à la fin de l'exercice

Status: review

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
reprise n'y change rien (AC-1, AC-2, AC-6, AC-7). Révisée pour CR-8 (A37–A39, précision de Beat du 2026-11-16) : un
compte en devise non réévalué **empêche la clôture** (contrôle **K9** bloquant, AC-9) ; l'avis de K9 montre l'aperçu et
son bouton « Réévaluer » passe les écritures montrées ; l'aperçu montre les réévaluations déjà passées, l'ancienne et la
nouvelle valeur cumulée, avec des liens (AC-1, AC-6).*

## Story

En tant que comptable en multi-devises, je veux qu'à la fin de l'exercice Kesh me propose la réévaluation de mes
comptes de bilan en devise au cours de clôture choisi pour l'exercice — cours journalier OFDF de la date de clôture
(à défaut, cours mensuel AFC du dernier mois, signalé) ou cours mensuel AFC du dernier mois (R6) —, et la passe, après
mon contrôle, en **une écriture par compte réévalué** (R9), afin que mon bilan présente ces comptes au cours de
clôture et que l'écart aille aux différences de change — et que Kesh refuse de clôturer un exercice dont un compte en
devise n'est pas réévalué (A37).

## Exigences

*(B26, 2026-11-15 : tirées des critères d'acceptation ci-dessous, sans en créer ni en modifier.)*

**Fonctionnelles**

- FR-37-5-1 — `POST /fiscal-years/{id}/fx-revaluation/preview` donne, par compte de bilan (`Asset`, `Liability`) imputable en devise, la devise du compte, les soldes en devise et CHF cumulés depuis l'origine jusqu'à la date de fin (exercice de durée quelconque), le cours, le CHF réévalué, l'écart D = round(devise_net × cours) − chf_net en débit net, le numéro et le nom du compte, les réévaluations déjà passées dans l'exercice (`previousRevaluationTotal`, `previousEntryIds`) et la valeur cumulée après passage (`revaluationTotalAfter`, CR-8 A39), la contrepartie résolue par compte (`fx_accounts_for` : comptes propres ou de la société, `counterpartAccountId`, `fxAccountsOrigin`) et l'écriture prévue ; il n'écrit rien, présélectionne au plus 498 comptes (troncature signalée), liste un compte archivé non sélectionnable, propose aussi un compte à solde en devise nul et CHF résiduel (D = −chf_net), et signale sans refuser, dans `blockers[]`, seulement `EARLIER_FISCAL_YEAR_OPEN`, `LATER_FISCAL_YEAR_CLOSED`, `PERIOD_LOCKED`, `FX_ACCOUNT_INVALID` (sur les comptes sélectionnés), `RATE_UNAVAILABLE` (une fois par devise sans cours d'un compte de `accountIds`, 37-5-CR-P1-5) et `FX_VALUE_OUT_OF_RANGE` (un par compte sélectionné hors bornes, 37-5-CR-P2-D2) — six codes depuis la revue de code ; il refuse comme la passation l'exercice inconnu ou d'une autre société (404), le mono-devise (409), l'exercice clos (400), les formes de requête et `notEligible` (AC-1, AC-4) ; un compte sélectionné hors bornes y est signalé (`blockers[]`), non refusé (37-5-CR-P2-D2).
- FR-37-5-2 — Le cours par défaut, par devise, est le cours **retenu par la société** (jamais la dernière valeur publiée non reprise, U1), selon le genre de clôture de l'exercice : genre `day`, le cours journalier de la date de fin, à défaut le cours mensuel du mois de fin signalé comme repli ; genre `month`, le cours mensuel du dernier mois seul ; sinon `RATE_UNAVAILABLE` — à l'aperçu, la devise est signalée et ses comptes exclus jusqu'à saisie ou récupération (ni refus ni `blockers[]`), à la passation, 404. Il est surchargeable par devise (`rates`) ; le genre d'un cours reçu (`day`, `month`, `request`) suit une règle unique à l'aperçu, à la passation et à l'audit ; l'aperçu porte, par devise, le cours, son genre et sa date, le cours et la source de la résolution par défaut (source brute, une autre que `afc` traitée comme `manual`, A30), `overridden` et `sourceDate` (cours `afc` seulement), et `closingRateKind` (AC-1, AC-2, AC-7).
- FR-37-5-3 — `POST …/fx-revaluation` recalcule sans se fier à l'aperçu et passe, dans **une seule transaction** (toutes ou aucune), une écriture `OD` de deux lignes par compte à écart non nul, datée du dernier jour de l'exercice (ligne `revaluation` à montant en devise 0 et CHF = |D|, exemptée de l'égalité CHF = montant × cours et de `FX_LINE_ROUNDS_TO_ZERO` ; D > 0 : débit du compte réévalué, crédit du compte de gains ; D < 0 : crédit du compte réévalué, débit du compte de pertes — gains et pertes du compte réévalué, quel que soit le signe du solde ; aucune compensation entre comptes), libellé français partagé avec l'aperçu ; verrous pris avant toute lecture des soldes, si bien que deux passations concurrentes n'en passent qu'une ; aucun écart → 200 `{ entryIds: [] }`, sinon les identifiants dans l'ordre des numéros de compte. Les livres importés repassent leur réévaluation par cette route (V8) (AC-2).
- FR-37-5-4 — La passation est rejouable compte par compte : un second passage sans mouvement ne passe rien, après un mouvement tardif seul le résidu du compte touché ; une réévaluation se corrige par suppression ou nouveau passage (le `PUT` la refuse, la contre-passation ne corrige que dans un exercice en cours) ; aucune écriture de report (AC-3).
- FR-37-5-5 — Les gardes refusent en deux temps, une seule raison, la plus bloquante d'abord. Avant la transaction : exercice inconnu ou d'une autre société (404), exercice mono-devise (409 `MULTI_CURRENCY_DISABLED`, lecture simple, jamais relue sous verrou : le mode ne change plus, CR-7), exercice clos (400 `FISCAL_YEAR_CLOSED`), forme (400 `FX_RATE_INVALID`, `FX_REVALUATION_ACCOUNT_INVALID` `empty`/`duplicate`/`tooMany`), cours par défaut manquant (404 `RATE_UNAVAILABLE`). Sous verrou : clos, `EARLIER_FISCAL_YEAR_OPEN`, `FX_ACCOUNT_INVALID` des comptes employés par les comptes admissibles, `notEligible` (sans fuite d'existence), genre de clôture changé (409 `OPTIMISTIC_LOCK_CONFLICT`), `RATE_UNAVAILABLE` d'une devise changée, puis `LATER_FISCAL_YEAR_CLOSED`, `DATE_OUTSIDE_FISCAL_YEAR`, `PERIOD_LOCKED`. Bornes : cours > 0, ≤ 10 décimales, ≤ 10⁶, par unité, clé = devise d'au moins un compte sélectionné (sinon 400 `FX_RATE_INVALID`) ; au plus 100 devises et 498 comptes ; produit en `i128` contrôlé, CHF réévalué ou |D| > 10¹³ → 400 `FX_VALUE_OUT_OF_RANGE` (AC-4).
- FR-37-5-6 — L'action « Réévaluer les devises » n'apparaît que pour un exercice ouvert et multi-devises, aux rôles Comptable et Administrateur ; l'écran montre cours, source et genre, montre l'écriture prévue par compte avec sa contrepartie et son origine, les totaux des gains et des pertes, propose la récupération du cours (jour ou mois selon le genre et la date ; aucun bouton du jour tant que la date de fin n'est pas passée), affiche sous le champ le cours retenu par la société quand il en diffère, sans ↑, ne relit l'aperçu qu'avec la sélection et les cours modifiés (aucune relecture à sélection vide), envoie à la passation les cours affichés des seules devises d'au moins un compte coché et désactive « Passer les écritures » tant que `blockers[]` n'est pas vide ; un cours modifié à l'écran ne vaut que pour la passation, il n'écrit aucun cours saisi ; par compte, l'aperçu affiche numéro et nom (lien vers le grand livre du compte), solde en devise, CHF actuel, cours, CHF cible, réévaluations déjà passées (valeur cumulée, liens vers ces écritures), nouvelle valeur cumulée et écart — les écritures sont toujours ajoutées, jamais modifiées (AC-1, AC-4, AC-5, AC-6).
- FR-37-5-7 — Contrôle de clôture **K9** `FOREIGN_ACCOUNTS_NOT_REVALUED`, sévérité `blocking`, neuvième et dernier (`checks[8]`) : en attention dès qu'un compte de l'aperçu a un écart non nul au cours par défaut, ou qu'une devise d'un compte à solde non nul n'a pas de cours de clôture ; calcul par la fonction de l'aperçu (DRY) ; `details` = les comptes concernés ; et `POST /fiscal-years/{id}/close` refuse dans ce cas (409 `FOREIGN_ACCOUNTS_NOT_REVALUED`, mêmes `details`), après `EARLIER_FISCAL_YEAR_OPEN`, soldes lus après les verrous (AC-9).
- FR-37-5-8 — À l'écran *Exercices*, l'avis de K9 en attention montre directement l'aperçu (cours par défaut, présélection) et son bouton « Réévaluer » passe les écritures montrées, sans écran intermédiaire ; K9 OK : ni avis, ni aperçu, ni bouton ; le dialogue de clôture désactive « Clôturer » quand K9 est en attention (AC-9).

**Non fonctionnelles**

- NFR-37-5-1 — Les deux routes sont dans `comptable_routes` : Consultation reçoit 403, aperçu compris ; ouvertes aux clés d'API (pas d'`ensure_not_pat`), une clé en lecture seule refusée sur l'aperçu comme sur tout POST (AC-5).
- NFR-37-5-2 — Aucun outil MCP pour l'aperçu ; les écritures passées se lisent par `kesh_ecriture(s)` (AC-5).
- NFR-37-5-3 — Audit : outre le `journal_entry.created` de chaque écriture, une ligne `fx_revaluation.posted` (entité `fiscal_year`) écrite dans la même transaction, qui liste les écritures et porte cours, source et genre par devise et `closingRateKind`, acteur par `for_actor` (clé d'API comprise) ; aucune ligne sans écriture ; registres `ACTIONS` et `LIB_ROUTES` (aperçu `NoMatter`, `SansEcritureAuJournal` ; passation `Traced`, `Rejouee`, appelée par le handler sous `retry_on_deadlock`) et quatre locales suivent (AC-2).
- NFR-37-5-4 — Exactitude : solde CHF = round(solde devise × cours) sur deux exercices ; cinq cas de signe ; recette de bout en bout par l'API seule, en `Authorization: Bearer`, sans écran ni cookie (V7) (AC-1, AC-7).
- NFR-37-5-5 — Le serveur de 37-5 ne récupère jamais de cours : cours résolus avant la transaction, aucune lecture ni récupération de cours sous le verrou ; aucun appel réseau en test (source factice de 37-2) (AC-1, AC-7).
- NFR-37-5-6 — Manuel à jour (AC-8).
- NFR-37-5-7 — K9 ne coûte rien hors multi-devises : exercice mono-devise → OK sans autre lecture ; exercice multi-devises sans compte de bilan en devise → OK après une seule requête `EXISTS`, sans résolution de cours ni lecture de soldes ; au panneau, aucune lecture verrouillante (AC-9).

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
     Le genre se lit avec l'exercice (handler, `find_by_id_in_company`, avant la transaction, comme les cours, résolus
     par la fonction de `kesh-db` de l'AC-9, *Une seule fonction*) ; sous
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
     `blockers[]` (code et détails) (CR-7 : `laterFiscalYear` retiré, aucun exercice mono-devise ne suit un exercice
     multi-devises — les exercices clos à l'activation précèdent les ouverts, et la création d'un exercice avant un
     exercice mono-devise est refusée, D-CR7-2, 37-1) ; l'écran désactive « Passer les écritures » tant qu'il n'est pas vide
     (`FX_ACCOUNT_INVALID` : sur les comptes sélectionnés seulement, l'aperçu relu à chaque case, AC-4).
   - **Une ligne d'aperçu par compte = une écriture (R9)** : l'aperçu présente, par compte retenu, l'écriture qui sera
     passée (compte réévalué, contrepartie, montant, libellé de l'AC-2) ; les totaux des gains et des pertes restent
     affichés.
   - **Ce qui est déjà passé (A39, CR-8)** — champs **ajoutés** à chaque ligne de la réponse (contrat additif ; aucun
     champ existant ne change) : `accountNumber`, `accountName` (s'ils n'y sont pas déjà sous un autre nom) ;
     `previousRevaluationTotal` — Σ débit − Σ crédit, en CHF, des lignes de réévaluation (`rate_source =
     'revaluation'`) du compte **datées dans l'exercice** (`start_date` ≤ date ≤ `end_date`), chaîne décimale,
     `"0.00"` s'il n'y en a pas ; `previousEntryIds` — les écritures de ces lignes, par date puis `id` (au plus 20, comme
     `MAX_LISTED` des contrôles de clôture ; la somme porte sur toutes) ; `revaluationTotalAfter` =
     `previousRevaluationTotal` + D (valeur cumulée après passage). Une requête de `kesh-db`, lue par l'aperçu seulement (la
     passation et K9 n'en ont pas besoin). Le
     « CHF actuel » est `chf_net`, le « CHF cible » le CHF réévalué. Une réévaluation ne se **modifie** jamais : une
     nouvelle passation **ajoute** l'écriture du résidu (AC-3) ; « mise à jour » ne s'affiche que comme ancienne →
     nouvelle valeur cumulée, pour qu'une valeur excessive se voie avant de passer.
2. **AC-2 — Passation.** `POST …/fx-revaluation { rates, accountIds }` (pas de `version` : `FiscalYear` n'en a pas ;
   l'idempotence vient du rejeu, AC-3, et du verrou de l'exercice). Le serveur ne fait jamais confiance à l'aperçu : il
   recalcule.
   - **Livres tenus ailleurs, importés par l'API (V8, CR-6)** : la réévaluation des livres d'origine ne s'importe
     **pas** en écritures — sa ligne sur le compte en devise (montant en devise 0, CHF non nul) n'a de forme admise que
     `revaluation`, refusée par l'API (37-3a AC-6) ; elle se **repasse** par cette route, à la fin de l'exercice
     importé, avec les cours de clôture des livres d'origine dans `rates` — pour la recette de Beat, `{ "USD":
     "0.91193", "EUR": "0.94947" }` au 31.12.2024 (cours journaliers, fichier d'epic D10) — et `accountIds` = les
     comptes que les livres d'origine réévaluaient. Le CHF obtenu égale celui des livres d'origine dès que les autres
     écritures y ont été reprises à leur CHF (`chf` ou `debit`/`credit`, 37-3a AC-6, V1) : `round(solde en devise ×
     cours) − solde CHF`. Genre des cours reçus : la règle unique de l'AC-1 (*Genre d'un cours reçu dans `rates`*).
     Écrit au manuel (AC-8) et au guide d'API (37-3b AC-6). CR-8 : un cours de `rates` différent du cours retenu par
     la société laisse K9 en attention et la clôture refusée (AC-9) — pour clôturer l'exercice importé, d'abord
     **récupérer le cours du jour de clôture** s'il égale celui des livres d'origine (le cas de la recette de Beat : cours
     journaliers OFDF), sinon saisir ces cours de clôture d'origine comme cours saisis (37-2) ; la recette V7 ne clôt
     pas et n'en dépend pas.
   - **Fonction de dépôt unique** `fx_revaluation::post(pool, user_id, actor_api_key_id, company_id, fiscal_year_id,
     rates, account_ids)` (`actor_api_key_id: Option<i64>`, propagé depuis `CurrentUser` par le handler : c'est
     l'argument qu'exige `NewAuditLogEntry::for_actor`, `entities/audit_log.rs:238` (`par`, :263, depuis l'Epic 35), pour l'audit ci-dessous), qui ouvre **une transaction** ; c'est le **handler** qui l'appelle dans
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
     (0) la ligne `companies` (`fx_gain_account_id`, `fx_loss_account_id` — **pas** `multi_currency_enabled` : le mode
     ne se relit pas sous verrou, D-CR7-3) lue
     `LOCK IN SHARE MODE` (jamais `FOR SHARE`, erreur de syntaxe sur MariaDB 10.11 ; ce partage fige les comptes de
     différence de la société contre un `PUT /companies/current/currency-settings` concurrent et bloque toute
     désignation de comptes propres concurrente, qui prennent `companies` en exclusif, 37-1 AC-3 et AC-4 bis : les
     désignations lues ensuite restent celles du calcul) **avant tout exercice** : c'est
     l'ordre global `companies` → exercice des chemins Pattern 5 (`repositories/invoices.rs:698`, sentinelle puis `create_in_tx`) ;
     prise après l'exercice, elle formerait un cycle avec ces chemins ; prise en tête, elle n'en forme aucun, car aucun
     flux ne prend un exercice puis `companies` **en exclusif** (ni `close`, ni `reopen`, ni `lock_books`,
     `repositories/companies.rs:1002-1020` ; 37-3a ne prend plus de verrou sur `companies` pour une ligne en devise,
     D-CR7-1 : `create_in_tx_inner`, appelée ensuite, n'en ajoute aucun pour ces lignes, qui ne portent aucun projet — son étape 0 ne prend la sentinelle `companies` `FOR UPDATE` que pour une ligne à projet, `projects.rs:101`, ce qui ferait monter ce partage de S à X, cas de 37-3b AC-2 bis) ; (1) chaque antérieur
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
   `fiscal_years::find_by_id_in_company`) ; exercice mono-devise → **409** `MULTI_CURRENCY_DISABLED`
   (409 partout, décision d'epic ; R3 : `fiscal_years.multi_currency` de 37-1, `details: { fiscalYearId }` ; une
   société mono-devise n'a que des exercices mono-devise, si bien qu'aucune branche « société » n'est jugée, comme en
   37-3a, D-CR7-1) — **lecture simple**, faite ici seulement : le réglage est irréversible et
   `fiscal_years.multi_currency` ne change plus après la création ou l'activation (CR-7), aucune course n'est possible
   et le mode n'est pas relu sous verrou (D-CR7-3) ; exercice clos (lecture non verrouillante, refaite dans la transaction) → **400** `FISCAL_YEAR_CLOSED`
   (`DbError::FiscalYearClosed`, `errors.rs:4516` — le code et le statut d'une écriture dans un exercice clos, non le 409
   `ILLEGAL_STATE_TRANSITION` de `close`) ; forme de
   la requête (bornes ci-dessous) → 400 `FX_RATE_INVALID`, ou 400 `FX_REVALUATION_ACCOUNT_INVALID` sur `accountIds` ; aucun cours par
   défaut (AC-1, selon le genre de clôture : ni journalier ni mensuel au genre `day`, pas de mensuel au genre
   `month`) pour une devise sélectionnée sans surcharge → 404 `RATE_UNAVAILABLE`.
   **Dans la transaction**, après tous les verrous de l'AC-2 et **dans le même ordre qu'avant la transaction** (sauf le
   mode, jugé avant la transaction seulement, et le genre de clôture changé et le `RATE_UNAVAILABLE` d'une devise
   changée entre-temps, qui ne peuvent naître qu'ici et viennent après les comptes, ci-dessous) :
   exercice clos, relu dans sa ligne `FOR UPDATE` (étape (1)) → `FISCAL_YEAR_CLOSED` ; **exercice antérieur encore ouvert → `EARLIER_FISCAL_YEAR_OPEN`** (comme K1 : le cumul lu ne
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
   (`books_locked_through`, seuil **inclusif** : `entry_date <= locked_through`, étape 1-bis) → `PERIOD_LOCKED`
   (CR-7 : la garde `laterFiscalYear` de 37-3a est retirée). L'aperçu
   refuse de même (404, mono-devise, exercice clos, formes de la requête : 400 `FX_RATE_INVALID` /
   `FX_REVALUATION_ACCOUNT_INVALID` ; `FX_VALUE_OUT_OF_RANGE` y est un blocker par compte, non un refus, depuis
   37-5-CR-P2-D2 ; un `accountIds` fourni hors de la liste admissible y est
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
   **MCP (V12, CR-6)** : aucun outil pour l'aperçu — un calcul à la demande (proposition d'écritures), hors du
   périmètre du serveur MCP de l'Epic 32, qui ne rend que des lectures des livres (dit au Change Log du développement) ; les écritures passées se lisent par `kesh_ecritures`
   et `kesh_ecriture` (`mcp/outils/ecritures.rs:198`, `:272`), dont les lignes portent `foreign` avec la source
   `revaluation` (37-3a AC-8) ; l'action d'audit `fx_revaluation.posted` n'est lue par aucun outil (existant : aucun
   outil d'audit). Test : `kesh_ecriture` d'une écriture de réévaluation rend la ligne du compte en devise à montant 0
   et source `revaluation` (dans `mcp_outils_e2e.rs`, fixture de 37-3a AC-8, et, de bout en bout, dans le test d'API de
   l'AC-7).
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
   vers les écritures passées ; **par compte (A39, CR-8)** : numéro et nom du compte, **lien vers son grand livre**
   (`/reports?tab=general-ledger&from=<début>&to=<fin>&accountId=<id>`, lien profond existant de l'écran des rapports,
   construit par l'**existant** `ledgerHref(accountId, from, to)`, `reports.api.ts` — DRY),
   solde en devise, CHF actuel, cours, CHF cible, **réévaluations déjà passées** (valeur cumulée
   `previousRevaluationTotal`, chaque écriture de `previousEntryIds` en lien vers `/journal-entries/{id}`, affichage —
   le `PUT` d'une réévaluation étant refusé, 409 `ENTRY_IS_FX_REVALUATION`, elle se corrige par suppression, AC-3),
   **nouvelle valeur cumulée** (`revaluationTotalAfter`) et **écart** (D) ; ce tableau est **un seul composant**, repris
   par l'avis de K9 (AC-9, DRY) ; le genre de clôture de l'exercice est rappelé (« cours de clôture : cours journalier du
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
     la transaction (aucune relecture sous verrou, D-CR7-3) ; état posé par l'API (exercice clos avant l'activation, activation, réouverture ; CR-7 : le
     seul exercice mono-devise possible sous une société multi-devises) ;
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
   - **recette par l'API seule (V7, CR-6)** — test d'intégration HTTP de bout en bout (`crates/kesh-api/tests/`, nom
     au développement, p. ex. `fx_recette_api_e2e.rs`), **sans écran et sans cookie** : chaque appel en
     `Authorization: Bearer` d'une session (`POST /api/v1/auth/login`, `accessToken`) — l'Administrateur pour
     l'activation (37-1 AC-3, V6), un Comptable pour le reste ; source de cours factice de 37-2 (`AppState::new_for_tests`,
     aucun appel sortant). Étapes, chacune contrôlée par sa réponse : comptes de gains, de pertes et de conversion
     (`POST /accounts`) ; **activation** (`PUT /companies/current/currency-settings`) ; comptes USD et EUR de bilan
     (`POST /accounts` avec `currency`), l'un à comptes propres ; exercice 2024 (`POST /fiscal-years`), multi-devises (créé
     après l'activation, CR-7) ; `POST /exchange-rates/fetch` des mois et du
     31.12.2024 utiles (source factice) ; **ouverture** (`POST /opening-balances`) d'un compte USD par `{ amount, chf,
     side }` sans `rate` ni `debit`/`credit` (V1, V2) ; **écritures** (`POST /journal-entries`) — une au cours retenu
     sans `rateSource` (relue `afc`), une par `chf` sans `rate` ni `rateSource` **égal à `round(montant × cours
     retenu, 2)`** (relue `afc`, au cours retenu exact et non au cours déduit : A32, le cas des livres tenus au cours
     AFC), une par `chf` sans `rate` ni `rateSource` **différent** de ce produit (relue `manual`, au cours déduit —
     37-5-CR6-P3-2 ; envoyée `afc`, elle serait refusée `FX_RATE_NOT_AFC`, 37-3a AC-4, 4) — ces deux CHF et le cours
     déduit attendu sont des **littéraux** fixés à l'écriture du test d'après le cours que la source factice de 37-2
     rend pour le mois, non des produits calculés par le test (37-5-CR6-P4-3), une sans `debit`/`credit` (`side`),
     une écriture ordinaire avec sa ligne d'écart CHF sur les comptes du compte en devise (V4) ; **conversion** EUR →
     USD (`POST /journal-entries/conversions`) : l'écriture 1 (EUR au crédit, compte de conversion au débit) et
     l'écriture 2 (USD au débit net, compte de conversion au crédit du **même** CHF qu'à l'écriture 1), montants
     choisis pour une différence de change **non nulle** (sans quoi le premier envoi passerait en 201 : 37-5-CR6-P4-1),
     **sans** ligne de différence, corps dans l'ordre écriture 1 puis écriture 2 — **d'abord sans**
     `completeDifference` → 400 `ENTRY_UNBALANCED` (`details.entry` = 2), rien de créé (A31 : le serveur ne calcule
     jamais la ligne d'office), **puis avec `completeDifference: true`** → 201, ligne de différence ajoutée par le
     serveur (37-3b AC-2 bis, V3, A31), relue dans la réponse (37-5-CR6-P3-1), compte de conversion à 0 ;
     **réévaluation** au 31.12.2024 (`POST /fiscal-years/{id}/fx-revaluation/preview`, puis
     la passation) avec `rates` `{ "USD": "0.91193", "EUR": "0.94947" }` (V8) ; **rapports** (`GET
     /reports/trial-balance`, `/reports/balance-sheet`, `/reports/general-ledger`, 37-4) : soldes en devise et en CHF
     de chaque compte en devise calculés à la main, solde CHF au 31.12.2024 = round(solde en devise × cours de clôture)
     exactement, compte de conversion à 0 ; **lecture MCP** (V12) : le serveur MCP est désactivé par défaut et ses clés ne
     sont **pas** des clés d'API (`require_auth` ne les reconnaît pas, `lib.rs:366-368` ; elles ne passent pas par
     `/api/v1`) — le test l'active en session d'Administrateur (`PUT /api/v1/admin/mcp/settings` : corps complet
     `{ enabled, addressFilterEnabled, allowedNetworks, trustedProxies, allowedOrigins, version }`,
     `routes/mcp_server_settings.rs:95-102`, la `version` relue par le `GET` ; le filtre d'adresse doit admettre le pair
     du banc, `127.0.0.1`, sans quoi `/mcp` répond comme une route absente, `mcp/transport.rs:90-96` — 37-5-CR6-P2-3),
     crée une clé MCP de portée `books:read` **dispensée de validation** (`POST /api/v1/admin/mcp/keys` avec
     `"sendValidation": "exempt"`, jeton rendu à la création ; le défaut est `required`, `routes/mcp_keys.rs:273-276`,
     `entities/mcp_key.rs:256-260`, sous lequel un outil de données rend « envoi n° X en attente » et non les lignes —
     37-5-CR6-P2-1) et appelle `kesh_ecritures` puis
     `kesh_ecriture` par le transport MCP (comme `mcp_outils_e2e.rs`, `banc(&pool).appeler(&jeton, …)`,
     `tests/common/mcp_pieges.rs:76`) : la réponse porte les données (non un envoi en attente), et la ligne du
     compte en devise rend `foreign` à montant 0 et source `revaluation`. Le Comptable de l'étape précédente est créé
     par l'Administrateur (`POST /api/v1/users`). Les **deux** cours de `rates` (USD 0.91193, EUR 0.94947) sont choisis
     **différents** de ceux que la source factice de 37-2 rend pour le 31.12.2024 (à fixer à l'écriture du test) :
     genre `request` et source `manual` à l'audit **pour chaque devise**, attendus ainsi (37-5-CR6-P2-2). Le test n'emploie aucun calcul de cours ni de CHF côté client : tout montant CHF qu'il envoie
     est une donnée (celle des livres d'origine), jamais un produit qu'il aurait calculé ;
   - **ce qui est déjà passé (A39, CR-8)** : sans réévaluation antérieure, `previousRevaluationTotal` `"0.00"`,
     `previousEntryIds` `[]`, `revaluationTotalAfter` = D ; après une passation puis une écriture tardive,
     `previousRevaluationTotal` = D₁, `previousEntryIds` = [l'écriture de D₁], `revaluationTotalAfter` = D₁ + D₂ ; une
     réévaluation d'un autre exercice n'y entre pas ; deux passations sur le même compte → deux écritures, aucune
     modifiée ; Vitest : colonnes, liens vers le grand livre (paramètres `tab`, `from`, `to`, `accountId`) et vers chaque
     écriture ;
   - l'aperçu ne crée aucune écriture ; E2E de l'écran, dont le parcours repli mensuel → « Récupérer le cours du
     JJ.MM.AAAA » (source factice de 37-2) → aperçu relu, cours journalier (`rateKind: "day"`, le champ non modifié
     ayant repris le défaut) ; Vitest : l'aperçu ne reçoit que les cours modifiés, un `404 RATE_NOT_PUBLISHED` du
     `POST /fetch` laisse le repli affiché.
8. **AC-8 — Manuel** : réévaluation, cours journalier OFDF de la date de clôture (D10 : la pratique du cours de
   clôture, et les cours que portaient les livres de Beat), contrepartie sur les comptes de différence propres du
   compte réévalué s'il en a (S2), exercice de durée quelconque (S4), repli sur le cours mensuel AFC du mois de clôture (la méthode de
   calcul du cours mensuel ne se décrit qu'après confirmation, R4 amendé), choix du genre de clôture par exercice (R6), une écriture
   par compte réévalué (R9), correction par suppression ou nouveau passage, la contre-passation ne corrigeant que dans un exercice en cours (AC-3), refus en
   exercice mono-devise (R3 ; CR-7 : un exercice clos avant l'activation puis rouvert), cours saisi par la société (D9), récupération du cours du jour, gains et pertes non réalisés
   comptabilisés (CO 960b, choix assumé ; la neutralisation par provision n'est pas offerte, C-37-8), postes non
   monétaires (stocks, immobilisations, capital en devise : à décocher à la passation, mais ils bloquent la clôture
   tant que leur écart n'est pas nul — les tenir sur un compte en CHF, A37 et A41 : aucune marque « non monétaire »
   n'existe, AC-9), la source du cours (« AFC » ou « cours saisi ») conservée à l'audit et non
   sur la ligne, une correction de l'AFC non reprise (↑) ne changeant pas le cours proposé (U1), la reprise de livres
   tenus ailleurs dont la réévaluation se **repasse** par cette fonction avec les cours de clôture d'origine, non par
   des écritures importées (V8, AC-2) ; **CR-8** : l'aperçu enrichi (réévaluations déjà passées, ancienne et nouvelle
   valeur cumulée, liens, écritures toujours ajoutées), le contrôle K9 qui **bloque la clôture**, son avis qui montre
   l'aperçu et son bouton « Réévaluer », et le cours d'une passation à un autre cours que celui de la société, à
   récupérer (cours du jour de clôture) ou à saisir à l'écran *Cours de change* pour que K9 l'accepte (AC-9) ; i18n ; manuel en `.tex` seulement (PDF non commité,
   régénéré à la release).
9. **AC-9 — Contrôle de clôture K9 bloquant (A37, A38 ; CR-8, précision de Beat du 2026-11-16).**
   - **Le mécanisme lu au code.** Le panneau des contrôles ne bloque rien par lui-même (`closing_checks.rs:24` :
     « Rien ne **bloque** : la clôture garde son unique refus existant (K1, `fiscal_years::close`) ») : K1 est `blocking`
     parce que `fiscal_years::close` refuse, à son étape (d), sur la relecture verrouillante du plus ancien antérieur
     ouvert (`fiscal_years.rs:1175-1190`, `DbError::EarlierFiscalYearOpen`, 409 `EARLIER_FISCAL_YEAR_OPEN`,
     `kesh-api/src/errors.rs:4200-4221`), et l'écran désactive « Clôturer » (`+page.svelte:515`). ⚠️ Kesh n'a **pas** de
     contrôle d'équilibre CHF bloquant à la clôture : l'équilibre se vérifie à l'insertion, et K6 (écritures
     incohérentes) est `anomaly`, informatif. « Bloquant comme l'équilibre » (A37) se réalise donc **comme K1** : une
     sévérité `blocking` au panneau **et** un refus réel dans `close`.
   - **Le contrôle.** Code `FOREIGN_ACCOUNTS_NOT_REVALUED`, sévérité `blocking`, rendu **en dernier** (`checks[8]`, K9 :
     les index de K1 à K8 ne bougent pas, règle de K8, `closing_checks.rs:5-7`) ; nouvelle variante de `CheckCode`, de
     `CheckDetails` et du type `ClosingCheckCode` du frontend. Statut `attention` dès qu'une ligne de l'aperçu (AC-1)
     a, **au cours par défaut**, un écart D ≠ 0, ou qu'un compte à solde non nul (devise ou CHF) a une devise **sans
     cours de clôture** (`RATE_UNAVAILABLE` de l'AC-1 : écart inconnu). Lignes examinées : toutes celles de l'aperçu
     **sans `accountIds`** — présélection et comptes archivés compris, sans le plafond de 498 (un compte archivé à
     écart non nul se réactive pour être réévalué, manuel). `count` = nombre de comptes concernés (jamais borné) ;
     `details: { accounts: [{ accountId, number, name, currency, delta, reason }] }`, `reason` `NOT_REVALUED` (`delta`
     = D) ou `RATE_UNAVAILABLE` (`delta` `null`), au plus `MAX_LISTED` comptes, par numéro.
   - **Une seule fonction (DRY).** K9, le refus de `close` et l'aperçu appellent **la même** fonction de `kesh-db` qui,
     sur une connexion et des cours résolus, rend les lignes de l'aperçu (soldes par `balances_as_of`, D par le calcul
     pur de `kesh-core`) ; les cours par défaut se résolvent par **la même** fonction que l'aperçu (genre de clôture,
     cours retenu par la société, AC-1). Aucun second calcul du résidu. **Cette résolution vit dans `kesh-db`**
     (37-5-CR8-P2-1 : K9 et `close` y sont, et `kesh-db` ne dépend pas de `kesh-api`) : une fonction (nom au
     développement, p. ex. `fx_revaluation::resolve_default_rates(conn, company_id, &fiscal_year, currencies)`) qui,
     sur une connexion, applique la règle du genre de l'AC-1 (`day` : jour puis mois ; `month` : mois seul) par
     `find_company_rate_in_tx` (37-2) et rend, par devise, cours, genre, date et source, ou l'absence ; les handlers de
     l'aperçu et de la passation l'appellent sur le pool avant la transaction, K9 dans l'instantané, `close` avant
     `begin`.
   - **Sans lecture coûteuse (NFR-37-5-7).** Exercice mono-devise (`fiscal_years.multi_currency` faux, lu avec
     l'exercice par `FISCAL_YEAR_SQL`) → `ok`, aucune autre lecture ; exercice multi-devises sans compte de bilan en
     devise (une requête `EXISTS` sur `accounts`, constante du module inscrite à `ALL_SQL`, **même prédicat que l'aperçu** :
     `Asset`/`Liability`, devise autre que CHF, imputable, archivés compris — 37-5-CR8-P1-7) → `ok`, sans résolution de
     cours ni soldes. Au panneau, tout se lit dans l'instantané `ReadSnapshot`, sans verrou ; les fonctions externes
     appelées (résolution des cours, lignes de l'aperçu, `balances_as_of`) s'ajoutent à la liste de l'angle mort de
     la garde T-a2 (doc du module), vérifiées sans verrou en revue.
   - **Le cours par défaut fait foi.** Une passation à un autre cours (genre `request` : cours modifié à l'écran, ou
     `rates` de l'API, V8) laisse K9 en attention au cours retenu par la société. Pour clôturer à ce cours, le
     **saisir** à l'écran *Cours de change* (cours saisi, 37-2, D9) : il devient le cours par défaut, la passation
     suivante n'ajoute que le résidu, K9 passe `ok`. Écrit au manuel (AC-8) et à l'écran, sous le champ d'un cours
     modifié (« ce cours ne vaut que pour cette passation : la clôture vérifie au cours retenu par la société »).
     **Premier remède, avant toute saisie** (37-5-CR8-P1-3) : quand le cours passé est le cours **publié** du jour
     de clôture, *Récupérer le cours du JJ.MM.AAAA* (37-2) le copie dans `exchange_rates` (`afc`) et en fait le cours
     par défaut, sans saisie. **Livres de Beat au 31.12.2024** (genre `day`, cours journaliers OFDF 0.91193 et
     0.94947, D10) : K9 passe `ok` **sans saisie dès que ces deux cours journaliers sont dans `exchange_rates`** —
     copiés à la récupération du jour de clôture, ou déjà récupérés quand l'exercice multi-devises s'est créé
     (37-2 AC-3 bis : la copie suit une valeur publiée déjà connue) ; sinon la résolution
     retombe sur le cours mensuel (0.8846 / 0.9470), K9 reste en attention après une passation à 0.91193
     (`request`), et il faut récupérer le jour (ou le saisir). La recette V7/V8 ne clôt pas et n'en dépend pas.
   - **Un compte en devise qu'on ne veut pas réévaluer bloque la clôture (A37, 37-5-CR8-P1-2).** K9 juge **tous** les
     comptes de l'aperçu : décocher un compte (poste non monétaire — stock, immobilisation, capital en devise — que
     l'écran de l'AC-6 permet de décocher à la passation) ne le sort pas de K9, et l'exercice ne se clôt pas tant que
     son écart n'est pas nul. Conséquence **assumée** de la décision de Beat (A37, qui ne prévoit pas
     d'exception) ; le manuel (AC-8) dit de tenir un tel poste sur un compte **en CHF**. **Tranché par Beat (A41,
     epic 37, section CR-8, 2026-11-16, option a)** : un poste non monétaire se tient sur un compte en CHF, aucune
     marque « non monétaire » n'est ajoutée aux comptes (KISS) — ni colonne, ni case, ni exception dans K9.
   - **Le refus à la clôture.** **Tout est dans `fiscal_years::close`** (37-5-CR8-P1-1) : sa signature **ne change pas**
     (`close(pool, user_id, company_id, id)` compte 64 appels de tests, dont deux dans le `mod tests` de
     `journal_entries.rs`, qui n'auraient plus compilé, et un appelant qui oublierait de passer les cours contournerait K9 ; le handler
     `close_fiscal_year`, `routes/fiscal_years.rs:352`, ne change que par le bras d'erreur). **Avant `begin`**, sur le
     pool et donc hors de toute transaction (aucune lecture de cours sous verrou, NFR-37-5-5), `close` lit : (a)
     **l'exercice entier** (la ligne `FiscalYear`, au lieu de la seule date de début : `multi_currency`,
     `closing_rate_kind`, `end_date`), (b) les antérieurs, comme `post` (AC-2, *Verrous et instantané* ; fonction
     partagée des étapes (b')–(d), Notes), pour que la vue naisse à la lecture des soldes, après les verrous ; puis,
     seulement si l'exercice est multi-devises, la requête `EXISTS` de K9 (NFR-37-5-7) et, si elle trouve un compte,
     les cours par défaut des devises de ces comptes (la lecture de devises et la résolution de l'aperçu) — mono-devise
     ou sans compte en devise : aucune autre lecture, aucun cours. Les lectures de `close` se font à **chaque**
     tentative de `retry_on_deadlock`, toujours avant `begin`. Dans `close`, après (d), étape **(d-bis)** : genre de clôture relu sur la ligne verrouillée de Y,
     changé → 409 `OPTIMISTIC_LOCK_CONFLICT` ; **(d-bis) se juge sur la ligne verrouillée** (37-5-CR8-P2-3) : si
     son `multi_currency` est vrai, la fonction partagée lit les comptes **après les verrous**, même quand la lecture
     d'avant `begin` disait mono-devise ou ne trouvait aucun compte en devise (ces lectures ne servent qu'à choisir
     les cours à résoudre) — un compte en devise apparu entre-temps (activation concurrente, compte créé et mouvementé
     pendant la clôture), sans cours résolu, rend `RATE_UNAVAILABLE` et refuse : le contrôle échoue fermé ; ligne
     verrouillée mono-devise → aucune lecture ; lignes de l'aperçu par la fonction partagée → attention → rollback,
     `DbError::ForeignAccountsNotRevalued` → **409 `FOREIGN_ACCOUNTS_NOT_REVALUED`**, `details` = ceux de K9 (même forme,
     même liste) ; une devise devenue sans cours résolu → même refus, `reason` `RATE_UNAVAILABLE`. Ordre : 404, déjà
     clos (`ILLEGAL_STATE_TRANSITION`), `EARLIER_FISCAL_YEAR_OPEN` (le cumul ne serait pas stable), genre changé, puis
     K9. Message `error-fiscal-year-close-fx-not-revalued` (quatre locales), bras dans `errors.rs` à côté
     d'`EarlierFiscalYearOpen` ; aucun audit d'un refus ; route inchangée au registre (déjà `Traced`, `Rejouee`). Limite
     écrite : un cours retenu changé entre la résolution et la transaction est vu au cours lu par `close` avant `begin`
     (comme 37-5-CR5-P1-3). Un calcul hors bornes (`FX_VALUE_OUT_OF_RANGE`, > 10¹³) de la fonction partagée
     (37-5-CR8-P1-9) : `close` le rend tel quel (400 `FX_VALUE_OUT_OF_RANGE`), l'aperçu en fait un blocker du compte sélectionné (37-5-CR-P2-D2) ; au panneau, il ne doit pas
     faire perdre les huit autres contrôles — K9 est alors `attention`, le compte dans `details.accounts`, `reason`
     `NOT_REVALUED`, `delta` `null`.
   - **L'avis à l'écran (A38, précision de Beat).** Dans le panneau *Préparer la clôture*, K9 en attention montre
     **directement l'aperçu** (le tableau de l'AC-6, composant partagé : comptes, anciennes et nouvelles valeurs,
     écarts, liens), lu sans `accountIds` ni `rates` (cours par défaut, présélection), **avant** tout clic ; le bouton
     **« Réévaluer »** passe **les écritures montrées** (`{ rates: cours affichés des devises cochées, accountIds:
     présélection }`), sans écran intermédiaire, désactivé tant que `blockers[]` n'est pas vide ou que rien n'est
     sélectionnable ; une devise sans cours y est signalée avec un lien vers *Cours de change* ; **par devise, la
     ligne du cours de l'AC-6** (même composant, en lecture seule : cours, source, genre, et le **repli mensuel
     signalé** au genre `day`), avec « Récupérer le cours du JJ.MM.AAAA » aux mêmes conditions qu'à l'AC-6 (genre
     `day`, repli ou absence, date de fin passée), puis relecture de l'aperçu et des contrôles (37-5-CR8-P2-2 : sans
     elle, « Réévaluer » passerait d'un clic au cours mensuel de repli sans que le repli se voie — pour Beat, 0.8846
     au lieu du cours du jour 0.91193 — alors que l'AC-6 le signale et propose la récupération) ; les `blockers[]` de l'aperçu s'affichent
     sous le tableau avec le message du serveur (période verrouillée, exercice antérieur ouvert…), comme à l'écran de
     l'AC-6 — le bouton désactivé dit pourquoi (37-5-CR8-P1-5) ; un compte **archivé** à écart non nul, non
     sélectionnable, est signalé avec son remède (le réactiver : K9 le compte aussi, AC-9 *Le contrôle*) ; après
     succès, les contrôles sont relus. **Pas de lien « Modifier les cours ou la sélection »** (KISS, A36,
     37-5-CR8-P1-6) : changer un cours ou une sélection ne peut pas faire passer K9 (seule une passation au cours par
     défaut de tous les comptes le fait) ; l'écran de l'AC-6 reste atteignable par l'action de la ligne de
     l'exercice. Consultation (aperçu en 403, AC-5) : la liste de `details`, sans aperçu ni bouton. **K9 `ok` : rien n'est
     présenté** — ni avis, ni aperçu, ni bouton, et aucune requête d'aperçu n'est envoyée ; la **ligne** K9 reste
     dans la liste, « Rien à signaler », comme les huit autres (le serveur rend toujours neuf contrôles, mono-devise
     compris ; 37-5-CR8-P2-4). Limite écrite (37-5-CR8-P2-6) : au-delà de 498 comptes en devise, l'avis passe la
     présélection (les 498 premiers par numéro, écart nul compris) et K9 peut rester en attention ; les autres se
     cochent à l'écran de l'AC-6. Dialogue de clôture : K9 en
     attention → le point est listé et « Clôturer » est **désactivé** (« Réévaluez d'abord les comptes en devise »),
     le lien existant ouvre le panneau ; contrôles en chargement ou en échec → bouton actif, le serveur fait foi : un
     409 `FOREIGN_ACCOUNTS_NOT_REVALUED` s'affiche (message du serveur) et ferme le dialogue, comme
     `EARLIER_FISCAL_YEAR_OPEN` (`+page.svelte:335`). Le commentaire « Il ne bloque rien » (`+page.svelte`, dialogue)
     et `fiscal-years-checks-intro` (« Ces contrôles ne bloquent rien », quatre locales) se corrigent : deux contrôles
     bloquent, K1 et K9. Libellés K9 (`fiscal-years-checks-k9-title`, `-k9-help`, motifs, bouton), quatre locales.
   - **Sites qui comptent huit contrôles → neuf** (inventaire du 2026-11-16, `grep -rnE "(huit|sept|eight)
     (contrôles|checks)"` et `toHaveCount(8)` / `toHaveLength(8)` / `len(), 8`) : `closing_checks.rs:4`, `:123`,
     `:566` (doc ; plus la table du module, `Severity` « Seul `blocking` (K1) » → K1 et K9, « Rien ne bloque »
     `:24` et « rien ne bloque ici » `:170`) ; `routes/fiscal_years.rs:233` (doc, dit encore « sept ») ; `crates/kesh-db/tests/closing_checks.rs:1679` ;
     `crates/kesh-api/tests/closing_checks_e2e.rs:236-250` (liste des codes) ;
     `frontend/src/lib/features/fiscal-years/closing-checks.test-fixtures.ts:24` ; **omis à l'inventaire d'origine**
     (37-5-CR8-P1-8) : `FiscalYearChecks.svelte:2-6` (en-tête : « huit », « Il ne bloque rien non plus ») et `:135`
     (texte de repli de `fiscal-years-checks-intro`), `fiscal-years.api.ts:49` (« Ils ne bloquent rien »),
     `closing-checks-labels.ts` et `fiscal-years.types.ts` (code K9 ; et `:48`, « Les huit codes ») ; **omis encore**
     (37-5-CR8-P2-4) : `FiscalYearChecks.test.ts:48` (titre « huit lignes ») et `:56` (« Rien à signaler »
     `toHaveLength(7)` → 8, K9 `ok` compris), `fiscal-years-page.test.ts:447` (titre « les huit lignes ») ;
     `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts:461` ;
     `frontend/tests/e2e/fiscal-years.spec.ts:499-500`, `:545` ; `supplier-invoice-draft-cycle.spec.ts:630` ;
     `fiscal-year-closing-journey.spec.ts:165` ; manuel utilisateur `user-manual.tex:920` (« huit »), `:922` (« ne
     bloque rien », « son seul refus »), `:925` (« le seul contrôle qui bloque »), `:954` (« sans bloquer la
     clôture ») et une entrée K9 dans la liste `:924-933` ; manuel d'administration `admin-manual.tex:1866` ; brochure
     `marketing-brochure.tex:437` (« Huit contrôles … en lecture seule ») ; `README.md:48` (« huit contrôles en lecture
     seule ») ; `docs/api-external.md:257` (« la réponse ne bloque rien … garde son seul refus, `409 EARLIER_FISCAL_YEAR_OPEN` »),
     **`:266`** (« porte **toujours huit** contrôles *(sept jusqu'à la v0.17.0)* » : neuf, K9 en dernier, et sa
     description) et **`:1077`** (tableau des codes : une ligne `409 FOREIGN_ACCOUNTS_NOT_REVALUED` pour `POST
     /fiscal-years/{id}/close`). `.tex`
     seulement, PDF à la release ; `README.md:233` (v0.13.4, « sept contrôles ») est historique et reste.
   - **Tests.** `kesh-db` (`closing_checks.rs`) : neuf contrôles, `checks[8]` K9 `blocking`, `checks[7]` reste K8 ;
     exercice mono-devise (clos avant l'activation puis rouvert, CR-7) avec un compte en devise → `ok` ; multi-devises
     sans compte en devise et sans aucun cours en base → `ok` ; écart non nul → `attention`, `details` ; devise sans
     cours → `RATE_UNAVAILABLE` ; passation au cours par défaut → `ok` ; passation à un cours `request` → `attention` ;
     écriture tardive → `attention` ; compte archivé à écart non nul → `attention` ; devise sans cours mais compte à
     solde nul (devise et CHF) → `ok` ; **recette de Beat** (37-5-CR8-P1-3) : livres importés, passation à 0.91193
     avec le cours journalier du 31.12.2024 présent dans `exchange_rates` → `ok` sans saisie, et sans lui (repli
     mensuel) → `attention` ; compte décoché à la passation, écart restant → `attention` (A37) ; calcul hors bornes →
     `attention` sans perdre les huit autres. Clôture : 409
     `FOREIGN_ACCOUNTS_NOT_REVALUED` avec `details`, exercice resté ouvert ; antérieur ouvert **et** non réévalué →
     `EARLIER_FISCAL_YEAR_OPEN` ; déjà clos → `ILLEGAL_STATE_TRANSITION` ; genre changé → 409
     `OPTIMISTIC_LOCK_CONFLICT` ; après passation → 200 ; exercice mono-devise → 200 sans cours en base ; **clôture et
     passation concurrentes** (37-5-CR8-P1-4 : la seconde attend le verrou de l'exercice ; si la passation commite
     d'abord, la clôture relit les soldes après les verrous et passe ; si la clôture commite d'abord, la passation
     est refusée `FISCAL_YEAR_CLOSED`) ; **écriture tardive commitée entre le début de `close` et ses verrous** →
     refus K9 (la vue naît après les verrous) ; **compte en devise créé et mouvementé après la lecture d'avant
     `begin`** (lecture `EXISTS` vide, posée par le test entre les deux temps) → refus `RATE_UNAVAILABLE`
     (37-5-CR8-P2-3) ; les tests de clôture existants (mono-devise) **et leurs 64 appels de `fiscal_years::close`**
     (recomptés le 2026-11-16 : `grep -rn "fiscal_years::close(" crates` en rend 65, dont la route ; deux des 64 dans le
     `mod tests` de `journal_entries.rs`) inchangés (signature conservée). `kesh-api` : `closing_checks_e2e.rs` (code K9), corps du 409 et message
     dans les quatre locales. Vitest : fixtures à neuf contrôles ; K9 en attention → aperçu affiché avant tout clic
     (colonnes, liens), « Réévaluer » envoie les comptes et cours montrés, désactivé avec un `blockers[]` ; repli
     mensuel au genre `day` → signalé dans l'avis, « Récupérer le cours du JJ.MM.AAAA » proposé, puis aperçu et
     contrôles relus (37-5-CR8-P2-2) ; **K9 `ok` →
     aucun avis, aucun aperçu, aucune requête d'aperçu, aucun bouton** ; Consultation → liste sans bouton ; dialogue
     de clôture : « Clôturer » désactivé sous K9 en attention, 409 traité. E2E (parcours de 37-5) : contrôle bloquant
     (avis avec l'aperçu) → « Réévaluer » → contrôle `ok`, **aucun aperçu affiché** → clôture permise ; puis, sur un
     autre exercice, écriture tardive → l'avis montre la valeur cumulée passée et le lien vers sa première écriture.

**Dépendance de l'Epic 38** : 38-10 (Epic 38) passera de K9 à K10 (CR-8 : K9 est pris ici, l'Epic 37 sortant avant
l'Epic 38).

## Tâches

- [x] `kesh-core` : calcul pur de la réévaluation en débit net (entrées : soldes, cours ; sortie : lignes) et tests.
- [x] `kesh-db` : `fx_revaluation::post` (transaction, verrou d'exercice, soldes lus dans la transaction, **une écriture
      par compte** dans la même transaction — R9 —, audit `fx_revaluation.posted` qui les liste, état clos et genre de
      clôture de l'exercice relus sous verrou — R6 ; le mode, R3, se juge avant la transaction seulement, D-CR7-3). Soldes lus par `kesh_db::repositories::account_balances::balances_as_of(&mut conn,
      company_id, fin_d_exercice, Some(ids))` (37-4 AC-1 quater : dans `kesh-db`, car `kesh-db` ne peut pas dépendre de
      `kesh-report` ; borne `<=` incluse ; débit net brut ; aucune conversion de signe).
- [x] `kesh-api` : routes (aperçu, passation) dans `comptable_routes`, gardes, appel de la résolution des cours avant la
      transaction (fonction de `kesh-db`, AC-9 *Une seule fonction* : cours effectif journalier puis mensuel, ou mensuel seul ; 37-2), i18n des erreurs neuves (`FX_REVALUATION_ACCOUNT_INVALID` ;
      `FX_ACCOUNT_INVALID`, `RATE_UNAVAILABLE` et `FX_RATE_INVALID` viennent de 37-1 et 37-2, réemployés) et du libellé d'audit
      (quatre locales) ; registres de l'AC-2 (`ACTIONS`, `LIB_ROUTES`, décomptes, phrase du manuel d'administration) — tous recomptés depuis la source après chaque reprise de `main`.
- [x] Frontend : écran depuis les exercices (contrepartie et origine par compte, clés i18n neuves dans les quatre
      locales), Vitest, E2E.
- [x] CR-8 : champs `previousRevaluationTotal`, `previousEntryIds`, `revaluationTotalAfter` (et numéro, nom) de
      l'aperçu ; fonction partagée des lignes de l'aperçu (aperçu, passation, K9, clôture) ; K9 dans `closing_checks`
      et refus (d-bis) de `fiscal_years::close`, qui lit l'exercice et résout les cours elle-même avant `begin`
      (signature inchangée) ; erreur
      `FOREIGN_ACCOUNTS_NOT_REVALUED` (quatre locales) ; avis de K9 avec l'aperçu et « Réévaluer », rien quand K9 est
      `ok`, « Clôturer » désactivé ; inventaire « huit » → « neuf » de l'AC-9 ; tests de l'AC-9.
- [x] Manuel (`.tex` seulement), section *Note de version* de la fiche (pas d'écriture au `CHANGELOG.md`).
- [x] CR-6 : **V7** *(ouverture et conversion ajoutées à la fusion de la 37-3b)* — test d'intégration HTTP de bout en bout de la recette par l'API (AC-7, sessions Bearer, source
      factice, de l'activation aux rapports) ; **V8** — réévaluation de livres importés par cette route avec `rates`
      (AC-2), manuel (AC-8), guide d'API (37-3b AC-6) ; **V12** — test `kesh_ecriture` d'une écriture de réévaluation
      (AC-5), « aucun outil pour l'aperçu » écrit au Change Log du développement.

### Review Findings

*Revue de code P1 (Opus 5.5, 2026-11-17) — couches Blind Hunter, Edge Case Hunter, Acceptance Auditor ; triage et
correctifs par l'orchestrateur de la passe. Prompt : `37-5-review-prompt-p1.md`.*

- [x] [Review][Patch] 37-5-CR-P1-1 (MEDIUM) — le champ de cours d'une devise **sans cours retenu** était éditable
  mais inerte (ses comptes non cochables, `previewRequest` limité aux devises cochées, champ remis à vide à la
  relecture) : plus de champ pour une telle devise, la valeur « — » et le renvoi à *Cours de change* (AC-1) ; manuel
  utilisateur aligné [`frontend/src/lib/features/fiscal-years/RevaluationPanel.svelte`, `docs/manual/fr/user-manual.tex:1226-1228`]
- [x] [Review][Patch] 37-5-CR-P1-2 (MEDIUM) — « Passer les écritures » restait actif pendant une relecture : un
  clic juste après un changement de cours ou de case passait des cours et une sélection que l'aperçu affiché ne
  montrait pas ; bouton désactivé et `submit` refusé tant qu'une relecture est en cours [`RevaluationPanel.svelte`]
- [x] [Review][Patch] 37-5-CR-P1-3 (LOW) — l'avis de K9 resté monté après une passation (compte archivé ou sans
  cours) gardait l'aperçu d'avant : relu dans les deux modes [`RevaluationPanel.svelte`]
- [x] [Review][Patch] 37-5-CR-P1-4 (LOW) — une relecture échouée après une passation ne se voyait pas (erreur
  affichée seulement sans aperçu) : affichée au-dessus de l'aperçu périmé [`RevaluationPanel.svelte`]
- [x] [Review][Patch] 37-5-CR-P1-5 (MEDIUM) — l'aperçu acceptait dans `accountIds` un compte dont la devise n'a
  pas de cours (`selected` et non `selectable`) sans rien dans `blockers[]`, alors que la passation rend 404 :
  blocker `RATE_UNAVAILABLE` (`details { currency, month }`) une fois par devise [`crates/kesh-api/src/routes/fx_revaluation.rs`]
- [x] [Review][Patch] 37-5-CR-P1-6 (MEDIUM) — cas de l'AC-7 absents : deux exercices (N-1 réévalué et clos, N,
  CHF exact au 31.12.N, réévaluation de N-1 hors des « déjà passées » de N — second exercice réel), écriture
  EUR–EUR dans le cumul, `journal_entry.created` portant `foreign` au snapshot, correction AFC non reprise puis ↑,
  clôture concurrente qui commite d'abord (`FISCAL_YEAR_CLOSED`), exercice mono-devise **posé par l'API**
  (clôture, activation, réouverture) ; commentaire de `k9_mono_devise` rendu exact. Reste la restauration d'un
  `.keshbackup` (U1) : dette ci-dessous [`crates/kesh-db/tests/fx_revaluation.rs`, `crates/kesh-api/tests/fx_revaluation_e2e.rs`]
- [x] [Review][Patch] 37-5-CR-P1-7 (MEDIUM, compte rendu) — compteurs du registre des routes faux au Dev Agent
  Record (valeurs d'avant la fusion de la 37-3b) : corrigés par annotation datée, recomptés au test [fiche, *Completion Notes*]
- [x] [Review][Patch] 37-5-CR-P1-8 (LOW) — passation d'un compte non admissible (archivé, de résultat) dont la
  devise n'a pas de cours : 404 `RATE_UNAVAILABLE` au lieu de `notEligible` ; seules les devises des comptes
  admissibles exigent un cours [`routes/fx_revaluation.rs`]
- [x] [Review][Patch] 37-5-CR-P1-9 (LOW) — l'aperçu jugeait `notEligible` avant les clés de `rates`, la passation
  l'inverse : même ordre, même règle de clés (`find_accounts`) [`routes/fx_revaluation.rs`]
- [x] [Review][Patch] 37-5-CR-P1-10 (LOW) — guide d'API : ordre des refus faux (`notEligible` placé avant la
  transaction, `FISCAL_YEAR_CLOSED` relu et `RATE_UNAVAILABLE` sous verrou omis), blocker `RATE_UNAVAILABLE`,
  liste de K9 bornée à 20 sans le nombre au 409 [`docs/api-external.md`]
- [x] [Review][Patch] 37-5-CR-P1-11 (LOW) — `blocker_de` avalait un corps d'erreur illisible sans trace :
  `tracing::error!` [`routes/fx_revaluation.rs`]
- [x] [Review][Patch] 37-5-CR-P1-12 (LOW) — doc de l'aperçu « lectures ordinaires » alors que la revalidation des
  comptes de différence lit `LOCK IN SHARE MODE` (autocommit) : dit [`routes/fx_revaluation.rs`]
- [x] [Review][Defer] 37-5-CR-P1-D1 (MEDIUM) — un compte de bilan en devise **mouvementé** qui devient non
  imputable (sous-compte créé, `postable=false`) sort du prédicat des candidats : ni aperçu, ni K9, ni refus de la
  clôture — dette documentée ci-dessous, issue demandée [`fx_revaluation.rs:106` `CANDIDATE_PREDICATE`, `accounts.rs` `create`]
- [x] [Review][Defer] 37-5-CR-P1-D2 (LOW) — un seul compte présélectionné hors bornes (> 10¹³) fait refuser tout
  l'aperçu sans `accountIds` : l'écran ne se charge pas — issue demandée [`routes/fx_revaluation.rs` `out_of_range`]

**Dette (37-5-CR-P1-D1, 37-5-CR-P1-6 reste).** *D1* : le prédicat des candidats (`postable = TRUE`) est celui de
l'AC-9 ; un compte en devise ne devient non imputable avec un solde que par la création d'un sous-compte ou un `PUT`
`postable=false`, chemins antérieurs à la 37-5 qui ne regardent pas les mouvements. Remède à trancher (lister le
compte non imputable à solde comme un archivé et le compter à K9, ou refuser le passage à non imputable d'un compte
en devise mouvementé) : issue à ouvrir par l'orchestrateur, propriétaire Beat, à planifier dans l'Epic 37 avant la
release. *U1, restauration* : l'aperçu après restauration d'un `.keshbackup` sur une installation neuve (cours
restauré, aucun appel sortant) n'a pas de test propre ; la table `exchange_rates` est couverte par la sauvegarde de
37-2 et l'aperçu ne lit que cette table — issue à ouvrir, même propriétaire.

**Règle retenue pour D1 (revue de code P2, 37-5-CR-P2-D1).** Le prédicat des candidats ne retient plus `postable` :
un compte de bilan en devise **actif ou archivé, imputable ou non**, qui porte un solde (en devise ou en CHF) reste
candidat. Il figure à l'aperçu, non sélectionnable (`reason` `ACCOUNT_NOT_POSTABLE` s'il est actif, `ACCOUNT_ARCHIVED`
sinon), la passation le refuse (`notEligible`), **K9 le compte** (motif `NOT_POSTABLE`, avec son écart) et la clôture
est refusée en 409 avec ce motif ; le texte dit de le rendre imputable pour le réévaluer. Un compte non imputable
**sans solde** n'est pas rendu. Tests : `kesh-db/tests/fx_revaluation.rs`
`compte_mouvemente_devenu_non_imputable_bloque_la_cloture` (aperçu, K9, passation refusée, clôture refusée puis
permise), `kesh-api/tests/fx_revaluation_e2e.rs` `compte_non_imputable_a_solde_bloque_l_apercu_et_la_cloture`,
Vitest `RevaluationPanel.test.ts` et `FiscalYearChecks.k9.test.ts`.

**Amendement de la revue de code P3 (37-5-CR-P3-1).** « Rendez-le imputable » était **impossible** pour le chemin
même que nommaient le manuel et le commentaire du test : un compte qui a des sous-comptes actifs ne redevient pas
imputable (`effective_postable`), ni ne s'archive ; un compte en devise mouvementé sous lequel on crée un sous-compte
gardait donc un solde propre (les soldes ne remontent pas des sous-comptes) que rien ne réévaluait, et la clôture
restait refusée sans recours. Règle ajoutée : **un compte d'actif ou de passif en devise qui porte des écritures ne
reçoit pas de sous-compte** — `POST /accounts` rend `409 ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` (`details { parentAccountId,
parentNumber, currency }`, quatre locales), même prédicat que les candidats (`CANDIDATE_PREDICATE`), lecture des lignes
verrouillante sous le `FOR UPDATE` du parent. Ne reste, pour un compte non imputable à solde, que le `PUT`
`postable = false` d'une feuille, que l'utilisateur défait. Un parent en devise **sans** écriture reçoit un sous-compte :
sans solde propre, il n'est ni listé ni compté par K9, et seul l'enfant se réévalue. **Limite** : des données antérieures
à la règle (base de développement, sauvegarde restaurée) peuvent porter un tel parent ; aucune version publiée ne porte
le multi-devises, il n'y en a pas en production.

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Changed** — exports CSV des rapports (journal, grand livre, balance, bilan…) : une cellule de texte qui commence par `=`, `+`, `-` ou `@` est précédée d'une apostrophe, et tabulation et retours à la ligne y deviennent une espace ; les montants ne changent pas.
- **Changed** — `GET …/closing-checks` : le motif `NOT_POSTABLE` s'ajoute à K9 (compte à solde devenu non imputable) ; la ligne d'aperçu porte `ACCOUNT_NOT_POSTABLE` ; `blockers[]` de l'aperçu peut porter `RATE_UNAVAILABLE` et `FX_VALUE_OUT_OF_RANGE`.

- **Added** — réévaluation de fin d'exercice : aperçu puis passation des écritures de différence de change (action d'audit `fx_revaluation.posted`, erreur `FX_REVALUATION_ACCOUNT_INVALID`), depuis l'écran des exercices et par l'API.
- **Added** — neuvième contrôle de clôture, **bloquant** : un compte de bilan en devise non réévalué au cours de clôture empêche la clôture (`FOREIGN_ACCOUNTS_NOT_REVALUED`) ; son avis montre l'aperçu et passe les écritures d'un clic ; l'aperçu montre les réévaluations déjà passées et la nouvelle valeur cumulée.
- **Changed** — `POST /fiscal-years/{id}/close` refuse désormais aussi en 409 `FOREIGN_ACCOUNTS_NOT_REVALUED` ; `GET …/closing-checks` rend neuf contrôles (code neuf en dernier, changement de contrat additif) ; réponse de `…/fx-revaluation/preview` enrichie (champs ajoutés).
- **Compteurs** : recomptés depuis la source après chaque reprise de `main` (`ACTIONS`, `LIB_ROUTES`, partitions du registre, phrase du manuel d'administration, catalogues i18n).

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
  reconnaissent l'étape (c) à ce texte : le réemployer ne change pas leurs motifs. CR-8 : `close` lit à son tour (a)
  et (b) sur le pool avant `begin` (AC-9), si bien que la fonction partagée sert telle quelle aux deux ; les tests de
  course de la clôture (13 b1 à b3, rejeu) se relancent au gate complet (exception `kesh-db`).
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
- 2026-11-14 — révision CR-6 (couverture API et MCP, Opus) : **V7** — AC-7 : test d'intégration HTTP de bout en bout
  de la recette par l'API seule (sessions Bearer : Administrateur pour l'activation, Comptable pour le reste ; source
  factice ; comptes, activation, exercice, `POST /exchange-rates/fetch`, ouverture, écritures V1/V2/V4, conversion sans
  ligne de différence V3, réévaluation avec `rates`, rapports, `kesh_ecritures`) ; **V8** — AC-2 et AC-8 : la
  réévaluation de livres importés se repasse par `POST /fiscal-years/{id}/fx-revaluation` avec `rates` (USD 0.91193,
  EUR 0.94947 au 31.12.2024), non par des écritures `revaluation` (refusées par l'API) ; **V12** — AC-5 : MCP, aucun
  outil pour l'aperçu, écritures passées lues par `kesh_ecritures` / `kesh_ecriture` (source `revaluation`), test ;
  tâche CR-6. Prémisses relues au code : `lib.rs:524-525` (`comptable_routes`, `POST /accounts`), `:813` (`POST
  /fiscal-years`, Comptable), `:582` (`/opening-balances`), `:1246`, `:1259`, `:1272` (rapports), `:1353`
  (`/auth/login`) ; `mcp/outils/ecritures.rs:198`, `:272` ; `mcp/outils/mod.rs:9-20` (aucun outil d'audit). AC
  touchés : AC-2, AC-5, AC-7, AC-8, Tâches. Fiche à revalider (cycle CR-6).
- 2026-11-14 — validation CR-6 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche sauf <aucun inter-fiches>. MEDIUM 37-5-CR6-P1-1 : la lecture MCP du test V7 (AC-7) parlait d'une « clé d'API en lecture seule » — une clé MCP n'est pas une clé d'API (`lib.rs:366-368`), le serveur est désactivé par défaut : étapes d'activation (`PUT /admin/mcp/settings`) et de clé (`POST /admin/mcp/keys`, `books:read`) ajoutées. LOW -P1-2 : phrase MCP d'AC-5 redondante, lieu du test V12 absent (`mcp_outils_e2e.rs`). LOW -P1-3 : création du Comptable (`POST /users`) et genre attendu du cours de `rates` (`request`) non dits. Relu au code : `lib.rs:525`, `:582`, `:813`, `:1246`, `:1259`, `:1272`, `:1353` ; `mcp/outils/ecritures.rs:198`, `:272` ; `mcp/outils/mod.rs` (aucun outil d'audit) ; cohérence avec 37-1 AC-3 (V6, Bearer), 37-3a AC-6 (V1, V2, V4, V8), 37-3b AC-2 bis (V3) et epic l. 8, 55 (0.91193, 0.94947). Non exercés : relecture complète hors diff CR-6, compilation, rendu.
- 2026-11-14 — validation CR-6 P2 (Opus) : 3 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW) ; remédiés dans la fiche,
  aucun inter-fiches. **CR6-P2-1** (MEDIUM) — AC-7 (remédiation P1-1) : la clé MCP créée par
  `POST /admin/mcp/keys` sans `sendValidation` prend le défaut `required` (`routes/mcp_keys.rs:273-276`,
  `entities/mcp_key.rs:256-260`) : `kesh_ecritures` rendrait « envoi n° X en attente », non la ligne `revaluation` —
  l'assertion échouait par construction → clé `"sendValidation": "exempt"`, réponse de données vérifiée. **-2** (LOW) —
  AC-7 (P1-3) : seul le cours USD était dit différent de la source factice ; l'EUR (0.94947) aussi, genre `request` et
  source `manual` par devise. **-3** (LOW) — AC-7 (P1-1) : `PUT /admin/mcp/settings` est en remplacement complet avec
  `version` (`routes/mcp_server_settings.rs:95-102`), et le filtre d'adresse doit admettre le pair du banc
  (`mcp/transport.rs:90-96`, sinon route absente) → dits. Prémisses relues au code : `lib.rs:365-384` (routes MCP
  d'administration, clé MCP non reconnue par `require_auth`), `:260` (`/users`) ; `routes/mcp_keys.rs:97-107`,
  `:266-276` ; `mcp/transport.rs:71-96` (réglages lus en base à chaque requête : le banc voit l'activation faite par
  le routeur du test) ; `tests/common/mcp_pieges.rs:60-110` (`banc`, `cle_de`) ; `middleware/auth.rs:83-100`.
  Comptabilité (V8) : le CHF repassé égale celui des livres d'origine, réévaluations infra-annuelles d'origine
  comprises (leur cumul est l'écart au 31.12), sous la condition écrite (autres écritures reprises à leur CHF) ;
  gain/perte au sens de `round(solde × cours) − solde CHF`. Axes exercés : remédiation P1 (P1-1 à P1-3), V7, V8, V12,
  MCP (portée, activation, validation avant envoi, transport), RBAC Bearer, cohérence avec 37-1 AC-3 (V6) et 37-3b
  AC-2 bis (V3 dans la recette). Non exercés : AC-1 à AC-4 et AC-6 hors CR-6 ; contrats V1/V2 de 37-3a en détail ;
  exécution des tests, rendu.
- 2026-11-14 — validation CR-6 P3 (Sonnet) : 2 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW) ; remédiés dans la fiche,
  aucun inter-fiches. **37-5-CR6-P3-1** (MEDIUM) — AC-7 : l'étape de conversion de la recette disait « sans ligne de
  différence, ajoutée par le serveur » ; depuis A31 le serveur ne la calcule que sur `completeDifference: true`, sinon
  `ENTRY_UNBALANCED` — l'étape échouait par construction → conversion jouée d'abord sans le champ (refus, rien de créé),
  puis avec (201), formes des deux écritures dites (37-3b AC-2 bis : écriture à compléter = USD au débit net, compte de
  conversion au crédit net). **-2** (LOW) — AC-7 : la ligne « par `chf` sans `rate` (relue `manual`) » ne tient plus
  sous A32 quand le CHF égale `round(montant × cours retenu, 2)` (relue `afc`, cours exact) → deux lignes, l'une égale
  (`afc`), l'autre différente (`manual`). Prémisses relues au code : `routes/mcp_server_settings.rs:95-102`,
  `mcp/transport.rs:85-98` (P2-3 confirmé) ; 37-3a AC-4 (4) *Cours déduit*, AC-6 ; 37-3b AC-2 bis (A31, forme exigée,
  ordre des refus). V8, V12, AC-2 : non touchés par A31/A32 (la réévaluation lit les soldes, non la provenance des
  lignes). Axes exercés : diff P2 `ff3ea962..f46de99c` et révision, A31/A32 dans la recette V7, V8, V12, MCP (activation,
  clé exempte, portée), RBAC Bearer, cohérence avec 37-1 AC-3 et 37-3a/3b. Non exercés : AC-1 à AC-4 et AC-6 hors CR-6,
  les lignes d'ouverture par `chf` (provenance d'ouverture, 37-3b AC-1), exécution des tests, rendu.
- 2026-11-14 — validation CR-6 P4 ciblée (Opus) : 3 findings (0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW) ; remédiés dans la
  fiche ; un inter-fiches signalé, non appliqué. **37-5-CR6-P4-1** (LOW) — AC-7, conversion : le refus
  `ENTRY_UNBALANCED` (`details.entry` = 2) du premier envoi suppose une différence non nulle, un compte de conversion du
  même CHF aux deux écritures (sinon `FX_CONVERSION_TRANSIT_UNBALANCED`, plus tôt dans l'ordre de 37-3b AC-2 bis) et le
  corps dans l'ordre 1, 2 — écrits. **-2** (LOW) — AC-7 : la ligne au CHF **différent** du produit ne part `manual` que
  sans `rateSource` (envoyée `afc` → `FX_RATE_NOT_AFC`, 37-3a AC-4, 4, *Cours déduit*) → « sans `rate` ni
  `rateSource` » aux deux lignes. **-3** (LOW) — AC-7 : « égal à `round(montant × cours retenu, 2)` » se lisait comme un
  calcul du test, contre « aucun calcul de cours ni de CHF côté client » de la même étape → littéraux fixés à
  l'écriture du test. **Inter-fiches (epic)** : V3 de `epic-37-multi-devises.md` (« ligne de différence calculée par le
  serveur … si le corps ne la porte pas », « un client de l'API qui l'omet la délègue au serveur ») contredit A31,
  écrit quelques lignes plus bas — à aligner par l'orchestrateur. Prémisses relues : diff `7537d144..7d80c9bd` de la
  fiche ; A31, A32 (epic, section CR-6) ; 37-3a AC-4 (4) *Cours déduit* et *Ligne sans provenance* ; 37-3b AC-2 bis
  (demande, écriture à compléter, forme, ordre des refus, tests A31) ; `ENTRY_UNBALANCED` au code
  (`errors.rs:1873`). Axes exercés : les deux phrases changées de la recette V7 (A32 écritures, A31 conversion), leur
  cohérence avec 37-3a/3b et l'epic, ordre des refus de la conversion dans les deux modes, la règle « aucun calcul côté
  client » de la même étape. Non exercés : reste de la fiche (hors périmètre), appariement au `PUT` (la recette n'en
  joue aucun), exécution des tests, rendu.
- 2026-11-15 — exigences FR/NFR ajoutées (B26 ; orchestrateur, Sonnet) : 6 FR, 6 NFR.
- 2026-11-15 — validation ciblée des exigences (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW ; tous remédiés dans la section Exigences, sans exigence ajoutée (6 FR, 6 NFR, total de l'epic inchangé). MEDIUM 37-5-B26-1 : FR-37-5-2 « journalier, à défaut mensuel » est faux au genre de clôture `month` (AC-1 : « le cours effectif mensuel du dernier mois de l'exercice, choisi et non un repli » ; AC-7 : genre `month` avec un journal présent → `RATE_UNAVAILABLE`) et omettait le cours retenu par la société (U1) — récrite. MEDIUM -B26-2 : FR-37-5-5 plaçait `EARLIER_FISCAL_YEAR_OPEN` juste après l'exercice clos, alors que les refus de forme et `RATE_UNAVAILABLE` le précèdent (AC-4 : « avant la transaction, précèdent `EARLIER_FISCAL_YEAR_OPEN` » ; AC-7 : « exercice antérieur ouvert et cours manquant → `RATE_UNAVAILABLE` »), et taisait les refus sous verrou et les bornes — récrite en deux temps. MEDIUM -B26-3 : FR-37-5-3 sans l'atomicité (transaction unique, AC-2 « toutes les écritures passent, ou aucune »), ni l'ordre verrous → soldes (deux passations concurrentes : une seule écriture), ni la forme de l'écriture — complétée. LOW -B26-4 : FR-37-5-1 sans cumul depuis l'origine, écart en débit net, archivés, `blockers[]`, présélection de 498, « n'écrit rien ». LOW -B26-5 : FR-37-5-4 sans le résidu ni la correction (suppression, `PUT` refusé, contre-passation). LOW -B26-6 : NFR-37-5-3 doublonnait FR-37-5-3 (« ne fait pas confiance ») et réduisait l'audit à la source — contenu, transaction, absence sans écriture, `for_actor`, registres. LOW -B26-7 : NFR-37-5-5 ne disait que le test, non la règle de production (le serveur ne récupère jamais de cours, aucune lecture de cours sous verrou). LOW -B26-8 : FR-37-5-6 sans « exercice ouvert » ni la conduite de l'écran. Axes exercés : chaque exigence contre l'AC entier cité (AC-1 à AC-8) ; points majeurs (contrat d'API, refus et ordre, gardes comptables, verrous, audit, RBAC, MCP, sauvegarde : restauration couverte par U1 dans FR-37-5-2) ; récapitulatif de l'epic. Non exercés : les AC eux-mêmes (hors périmètre), le code, les fiches 37-1/37-2/37-3a citées.
- 2026-11-15 — validation ciblée P2 des exigences (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW, remédiés dans la section Exigences, sans exigence ajoutée (6 FR, 6 NFR). MEDIUM 37-5-B26-P2-1 : FR-37-5-1 disait que l'aperçu « signale sans refuser ce que la passation refuserait », trop large au regard de l'AC-4 (l'aperçu refuse 404, mono-devise, clos, formes, `notEligible`, `FX_VALUE_OUT_OF_RANGE` ; il ne signale en `blockers[]` que les cinq codes de l'AC-1) — borné. LOW -B26-P2-2 : FR-37-5-6 sans écriture prévue par compte, totaux, bouton désactivé tant que `blockers[]` non vide, cours modifié valable pour la passation seule (AC-6) — ajoutés. LOW -B26-P2-3 : FR-37-5-5 sans la clé de `rates` = devise d'un compte sélectionné (AC-4) — ajoutée. LOW -B26-P2-4 : compte à devise nulle et CHF résiduel proposé (AC-1) — ajouté à FR-37-5-1. LOW -B26-P2-5 : recette par l'API seule (V7, AC-7) sans exigence — portée par NFR-37-5-4. Axes exercés : chaque exigence contre l'AC entier cité (AC-1 à AC-8 lus en entier), ordre des refus en deux temps, RBAC, MCP, audit et registres, récapitulatif de l'epic. Non exercés : les AC eux-mêmes, le code.
- 2026-11-15 — validation ciblée P3 des exigences (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW ; remédiés dans la section Exigences, sans exigence ajoutée (6 FR, 6 NFR). LOW 37-5-B26-P3-1 : la devise sans cours à l'aperçu (AC-1 : « la devise est signalée `RATE_UNAVAILABLE` … ses comptes exclus jusqu'à saisie ou récupération ») n'était ni refus ni `blockers[]` dans FR-37-5-1, et FR-37-5-2 ne disait que « sinon `RATE_UNAVAILABLE` » — précisé (aperçu : signalée, comptes exclus ; passation : 404). LOW -P3-2 : contrat de réponse de l'aperçu par devise (`rate`, `rateKind`, `rateDate`, `defaultRate`, `defaultSource` brut A30, `overridden`, `sourceDate` pour `afc` seul, `closingRateKind`, AC-1) sans exigence — ajouté à FR-37-5-2. LOW -P3-3 : FR-37-5-3 sans le sens des deux lignes (AC-1 *Règle de calcul* : D > 0 débit du compte, crédit gains ; D < 0 l'inverse, tout signe de solde), sans l'exemption de la ligne `revaluation` (AC-2 : égalité CHF = montant × cours et `FX_LINE_ROUNDS_TO_ZERO`), ni l'ordre de `entryIds` — ajoutés. LOW -P3-4 : FR-37-5-6 sans les cours envoyés à la passation (devises d'au moins un compte coché, AC-1, AC-4), le cours de la société sous le champ sans ↑ ni l'absence du bouton du jour avant la date de fin (AC-1, AC-6) — ajoutés. LOW -P3-5 : NFR-37-5-1 sans la clé d'API (AC-5 : pas d'`ensure_not_pat`, clé en lecture seule refusée sur l'aperçu) ; NFR-37-5-3 sans la classe des deux routes au registre ni l'enveloppe `retry_on_deadlock` (AC-2 *Registres*, *Fonction de dépôt unique*) — ajoutés. LOW -P3-6 : récapitulatif de l'epic « refus comme la passation pour le reste » trop large (la devise sans cours n'est pas refusée à l'aperçu) — reformulé. Vérifiés vrais : FR-37-5-4 (AC-3), FR-37-5-5 (ordre en deux temps, AC-4 l. 296-349, bornes l. 350-373), NFR-37-5-2 (AC-5 MCP), NFR-37-5-4 (AC-7), NFR-37-5-5 (AC-1, AC-2 : cours lus avant la transaction), NFR-37-5-6 ; sauvegarde (restauration, U1) portée par FR-37-5-2. Axes exercés : chaque exigence contre l'AC entier cité (AC-1 à AC-8 relus) ; points majeurs (contrat d'API, refus et ordre, garde comptable du sens, verrous, audit, RBAC, MCP, sauvegarde) ; récapitulatif de l'epic. Non exercés : les AC eux-mêmes, Tâches, code. Aucun finding au-dessus de LOW : la boucle des exigences de 37-5 peut se clore.
- 2026-11-15 — notes de dépendance de l'Epic 38 (orchestrateur, Sonnet) : le contrôle de clôture « comptes en devise à réévaluer » réservé en K9 devient K10 (38-10 livre K9).
- 2026-11-16 — révision CR-7 (A34 multi-devises sans retour, Opus) : blocker et refus `MULTI_CURRENCY_DISABLED`
  `laterFiscalYear` retirés (aperçu, ordre des refus de l'AC-4, test « exercice postérieur mono-devise ») ; refus d'un
  exercice mono-devise gardé (exercice clos avant l'activation puis rouvert) ; recette par l'API : exercice
  multi-devises parce que créé après l'activation (route `currency-mode` retirée). FR-1 et FR-5 réécrites
  (`blockers[]` : quatre codes). Décompte : 6 FR, 6 NFR (inchangé).
- 2026-11-16 — validation CR-7 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW. MEDIUM 37-5-CR7-P1-1 : l'AC-2 (étape (0)) s'appuyait sur le verrou partagé de `companies` de 37-3a dans `create_in_tx_inner` et sur la contre-passation « prend `companies` en partagé », retirés par D-CR7-1 — texte réduit à l'ordre de Pattern 5. LOW -P1-2 : test « exercice mono-devise » sans moyen de poser l'état (CR-7 : clos avant l'activation, puis rouvert) — précisé. LOW -P1-3 : manuel (AC-8) « exercice mono-devise » sans le seul cas possible — précisé. LOW -P1-4 (KISS, non modifié, signalé) : le mode de la société et de l'exercice relu « sous verrou » (AC-2 étape (0)-(1), AC-4, FR-5, Tâches) est un filet sans course possible depuis CR-7 (réglage irréversible, `fiscal_years.multi_currency` ne change plus) ; gardé, la ligne `companies` étant de toute façon lue en partage pour les désignations. Axes exercés : résidus (grep des neuf termes : aucun hors Change Logs datés), blockers/ordre des refus, FR-1/FR-5, recette V7, tests, verrous, récapitulatif de l'epic. Non exercés : code.
- 2026-11-16 — validation CR-7 P2 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW ; remédiés dans la fiche (D-CR7-2, D-CR7-3 appliquées). MEDIUM 37-5-CR7-P2-1 (D-CR7-3, KISS ; reprend P1-4) : le mode de la société et de l'exercice relu sous verrou (AC-2 étape (0) `multi_currency_enabled`, AC-4 « dans la transaction », FR-37-5-5, Tâches, test « avant la transaction et sous verrou ») n'existait que pour une bascule retirée par CR-7 — retiré ; le 409 `MULTI_CURRENCY_DISABLED` reste une lecture simple avant la transaction ; l'étape (0) garde `companies` en partage pour les seuls comptes de différence. LOW -P2-2 (D-CR7-2) : « aucun exercice mono-devise ne suit un exercice multi-devises » (AC-1, `blockers[]`) était faux sous la limite assumée de 37-1 (exercice créé dans un trou devant un exercice rouvert) ; il tient par le refus de D-CR7-2, renvoi écrit ; 37-5 ne portait ni la limite, ni son test, ni sa phrase au manuel. LOW -P2-3 (KISS) : branche « société mono-devise » du 409 redondante (une société mono-devise n'a que des exercices mono-devise) — jugé sur l'exercice seul, `details: { fiscalYearId }`, comme 37-3a (D-CR7-1). LOW -P2-4 : étape (0), la parenthèse « aucun flux ne prend un exercice puis `companies` en exclusif » justifiait l'ordre choisi et non le cycle qu'elle suivait, et le partage omettait le `PUT currency-settings` (37-1 AC-3, `companies` `FOR UPDATE`) — reformulé. Vérifié : 37-1 AC-3 (l. 261, `companies` `FOR UPDATE`), AC-4 bis (l. 592-596, désignation propre `FOR UPDATE`, « contre la réévaluation (37-5, `companies` en partage en tête) ») ; 37-3a D-CR7-1 (l. 23, 256, 418) ; `repositories/invoices.rs:698` (sentinelle Pattern 5) ; `fiscal_years.rs:376` (`LaterFiscalYearClosed`) ; récapitulatif de l'epic (l. 451) sans « sous verrou » du mode. Axes exercés : remédiation P1 (diff `882d865b..e99364d2`), D-CR7-2, D-CR7-3, résidus (grep `multi_currency_enabled`, « société ou exercice », « trou », « limite assumée », mode relu), verrous de l'AC-2, ordre des refus de l'AC-4, FR-37-5-1/-5, tests de l'AC-7, manuel AC-8. Non exercés : code (non écrit), AC-3, AC-5, AC-6 hors mode, recette V7 en détail. Décompte : 6 FR, 6 NFR (inchangé). Un MEDIUM : passe suivante requise.
- 2026-11-16 — validation CR-7 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié. LOW 37-5-CR7-P3-1 (AC-2, étape (0)) : « `create_in_tx_inner`, appelée ensuite, n'en ajoute aucun » était vrai pour les lignes de réévaluation (aucun projet) mais contredisait 37-3b AC-2 bis (étape 0 : sentinelle `companies` `FOR UPDATE` pour une ligne à projet, `journal_entries.rs:308` → `projects.rs:101`) — précisé. Vérifié au code : D-CR7-3 (409 jugé sur l'exercice, lecture simple avant la transaction ; aucun résidu « société ou exercice », « sous verrou » du mode, `multi_currency_enabled` hors étape (0) niée, FR-5, AC-4, AC-7, Tâches) ; renvoi D-CR7-2 (AC-1) ; `PUT currency-settings` `companies` `FOR UPDATE` (37-1 AC-3). Non exercés : code, AC-3, AC-5, AC-6, manuel.
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : PDF du manuel non commité, `.tex` seulement (1 remplacement) ; CHANGELOG remplacé par la section *Note de version* (1 remplacement, section ajoutée) ; compteurs « recomptés depuis la source après chaque reprise de `main` » (1 ajout, plus la Note de version). Fond des AC inchangé.
- 2026-11-16 — révision CR-8 (contrôle de clôture bloquant, Opus) : **A37** — AC-9 neuf : contrôle **K9**
  `FOREIGN_ACCOUNTS_NOT_REVALUED`, `blocking`, en dernier (`checks[8]`), attention dès qu'une ligne de l'aperçu a un
  écart non nul au cours par défaut ou une devise sans cours ; calcul par la fonction de l'aperçu (DRY) ; OK sans
  lecture en mono-devise, après un seul `EXISTS` sans compte en devise. Mécanisme lu au code : le panneau ne bloque
  rien (`closing_checks.rs:24`), K1 bloque par le refus de `fiscal_years::close` à l'étape (d)
  (`fiscal_years.rs:1175-1190`) et le bouton désactivé (`+page.svelte:515`) ; aucun contrôle d'équilibre CHF ne bloque
  la clôture (K6 informatif) — K9 suit donc K1 : refus (d-bis) 409 `FOREIGN_ACCOUNTS_NOT_REVALUED` dans `close`, cours
  résolus par le handler avant la transaction, (a) et (b) lues avant `begin` comme `post`. Un cours `request` laisse K9
  en attention : le saisir à *Cours de change* (V8 annoté). **A38** et **précision de Beat du 2026-11-16** (qui prime
  sur « le bouton ouvre l'aperçu » de la demande) : l'avis de K9 **montre l'aperçu** avant tout clic et « Réévaluer »
  passe les écritures montrées, sans écran intermédiaire ; K9 `ok` → rien n'est présenté (ni avis, ni aperçu, ni
  bouton, ni requête) ; « Clôturer » désactivé sous K9 en attention. **A39** — AC-1 et AC-6 : `accountNumber`,
  `accountName`, `previousRevaluationTotal`, `previousEntryIds`, `revaluationTotalAfter` (additifs), liens vers le
  grand livre et vers les écritures, écritures toujours ajoutées. Inventaire des sites « huit contrôles » (AC-9 :
  code, tests Rust, Vitest, E2E, deux manuels, brochure, README, guide d'API ; doc de route qui disait « sept »).
  « Hors périmètre, tracé … K10 » retiré ; note « 38-10 passera de K9 à K10 ». Exigences : 6 → 8 FR (FR-7, FR-8
  ajoutées ; FR-1, FR-6 complétées), 6 → 7 NFR (NFR-7). AC touchés : en-tête, Story, AC-1, AC-2 (V8), AC-6, AC-7,
  AC-8, AC-9, Tâches, Note de version, Notes. Fiche à revalider (cycle CR-8).
- 2026-11-16 — validation CR-8 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 5 LOW ; tous remédiés dans la fiche
  (37-5-CR8-P1-1 à -9) ; la décision de fond de -2 (comptes non monétaires, A37) est à confirmer par Beat ; aucun
  inter-fiche (38-10 passe à K10, déjà noté ; ligne de 37-5 du récapitulatif de l'epic complétée : brochure, README,
  guide d'API). Axes exercés : prémisses au code (`closing_checks.rs`, `fiscal_years::close`, handler, page des
  exercices, `ledgerHref`, tests appelants de `close`), blocage et ordre des refus, instantané et verrous, K9 sans
  lecture coûteuse, comptabilité du résidu, cas de Beat, écran, inventaire (grep recompté), tests, KISS. Non exercés :
  code de 37-1 à 37-4 (non écrit), E2E.
- 2026-11-16 — validation CR-8 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; tous remédiés dans la fiche.
  MEDIUM **37-5-CR8-P2-1** : la résolution des cours par défaut était placée au handler (AC-1, Tâches `kesh-api`),
  alors que K9 et, depuis P1-1, `close` (tous deux dans `kesh-db`, qui ne dépend pas de `kesh-api`) doivent appeler
  « la même fonction » — elle vit dans `kesh-db` (AC-1, AC-9, Tâches). MEDIUM **-2** : l'avis de K9 passait d'un clic au
  cours mensuel de repli sans le signaler ni proposer la récupération du jour (AC-6 le fait) — ligne de cours de l'AC-6
  en lecture seule et « Récupérer le cours du JJ.MM.AAAA » dans l'avis, test Vitest. LOW **-3** : (d-bis) se fiait aux
  lectures d'avant `begin` (mode, `EXISTS`) — jugé sur la ligne verrouillée, comptes lus après les verrous, échec
  fermé en `RATE_UNAVAILABLE`, test ; **-4** : inventaire (`FiscalYearChecks.test.ts:48`, `:56`,
  `fiscal-years-page.test.ts:447`, `fiscal-years.types.ts:48`) et ligne K9 « Rien à signaler » quand elle est `ok` ;
  **-5** : décompte des appels de `close` (64 de tests, dont deux dans le `mod tests` de `journal_entries.rs`, + la
  route ; « une soixantaine » / « ~65 ») ; **-6** : limite au-delà de 498 comptes écrite. Vérifié au code :
  `fiscal_years::close` (:1116-1222, (a)/(b) aujourd'hui **dans** la transaction — la fiche les sort, voulu),
  handler `close_fiscal_year` (:352-370), `closing_checks.rs` (:4, :5-7, :24, :123, :170, :566, `FISCAL_YEAR_SQL`
  :439, `ALL_SQL` :550, `MAX_LISTED` :435), `+page.svelte` (:335, :515, :674), `ledgerHref` (`reports.api.ts:473`),
  `user-manual.tex` (:920, :922, :925, :954), `docs/api-external.md` (:257, :266), 37-2 AC-3 bis (copie à la création
  d'un exercice multi-devises, unités de Clôture). Cas de Beat : K9 `ok` sans saisie avec les deux cours journaliers
  dans `exchange_rates`, juste. Inter-fiches signalés : epic C-37-7 (« décochables (postes non monétaires) ») ne dit
  pas qu'un compte décoché bloque la clôture (A37) ; hors périmètre de la passe, `audit_log.rs:195` a dérivé
  (`for_actor` à :238 depuis l'Epic 35, qui ajoute `NewAuditLogEntry::par`/`Acteur`, 35-8a1) — à relire à la prochaine
  passe complète. Axes non exercés : code de 37-1 à 37-4 (non écrit), E2E, reste de la fiche hors diff P1.
- 2026-11-16 — validation CR-8 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche.
  LOW **37-5-CR8-P3-1** : la référence `entities/audit_log.rs:195` de l'AC-2 (signalée périmée par la P2) n'avait pas
  été corrigée — `for_actor` est à `:238`, `NewAuditLogEntry::par` à `:263` (Epic 35, relus). LOW **-2** : A41 (Beat,
  option a) appliqué — « À confirmer par Beat » de l'AC-9 (*Un compte en devise qu'on ne veut pas réévaluer*) remplacé
  par le renvoi à A41 (poste non monétaire tenu sur un compte en CHF, aucune marque « non monétaire », K9 sans
  exception) ; AC-8 (manuel) dit déjà de tenir ces postes en CHF, renvoi à A41 ajouté. Remédiation P2 relue au code,
  sans défaut : résolution des cours dans `kesh-db` (`find_company_rate_in_tx` existe, `kesh-db` ne dépend pas de
  `kesh-api`) ; `fiscal_years::close` (:1116-1222) lit aujourd'hui (a)/(b) dans la transaction, la fiche les sort
  avant `begin`, voulu, signature conservée ; (d-bis) jugé sur la ligne verrouillée, échec fermé ; 65 occurrences de
  `fiscal_years::close(` (64 + la route), recomptées ; cas de Beat au 31.12.2024 juste ; K9 `ok` sans avis, ligne
  « Rien à signaler » ; inventaire P2 sans résidu ; Tâches cohérentes. Inter-fiches : aucun (epic A41 déjà écrit, C-37-7
  lu sous cette réserve). Axes exercés : diff P2 `47abae1e..5f295c8f`, blocage et ordre des refus, verrous, comptabilité
  du résidu, écran, inventaire, tests, KISS, A41. Non exercés : code de 37-1 à 37-4 (non écrit), E2E, manuels `.tex`
  (contrôlés à la passe précédente). Aucun finding au-dessus de LOW, remédiation sans code de production : boucle CR-8
  de 37-5 clôturable.
- 2026-11-16 — **étape 0 du développement** (Opus 5.5), table de `kesh-gate-logs/etape0/37-5.md` revérifiée contre la
  base de la branche (`31910288`, partie de `story/37-4-rapports-en-devise`) : tient, lignes relues — `fiscal_years::close`
  `repositories/fiscal_years.rs:1382`, `LOCK_IN_COMPANY_SQL` :163, `START_DATE_IN_COMPANY_SQL` :167, `LIST_EARLIER_SQL`
  :172, `LOCK_EARLIER_BY_ID_SQL` :179, `earlier_open_locking_sql` :202 ; `closing_checks.rs` « Rien ne bloque » :24,
  « Toujours huit » :123, `CheckCode` :150, `MAX_LISTED` :435, `FISCAL_YEAR_SQL` :439, `ALL_SQL` :550 ;
  `close_fiscal_year` `routes/fiscal_years.rs:515`, « sept contrôles » :254 ; `comptable_routes` `lib.rs:573`, `/close`
  :894 ; `fx_accounts_for` `accounts.rs:1773`, `validate_fx_accounts_in_tx` :1994 ; `find_company_rate_in_tx`
  `exchange_rates.rs:365`, `find_fetched_rate_in_tx` :402 ; `balances_as_of` `account_balances.rs:109` (37-4) ;
  `for_actor` `audit_log.rs:238`, `par` :263 ; registre : partitions :1063 et :1529, enveloppes :1369, `examinees, 27`
  :1404. Un écart : le bras `EarlierFiscalYearOpen` de `kesh-api/src/errors.rs` est à :4845 (37-3a et 37-4 l'ont
  déplacé depuis l'étape 0, :4625). Prémisses de 37-3a (ligne `revaluation`, exemption, `ENTRY_IS_FX_REVALUATION`) et de
  37-4 (`balances_as_of`) : présentes.
- 2026-11-16 — **développement** (Opus 5.5, worktree `kesh-s37-5`). Détail au Dev Agent Record. Fusion de
  `epic/37-multi-devises` à `dd01a3df` (37-4 terminée) par `git merge --no-ff`, sur consigne de l'orchestrateur : sans
  conflit, deux fichiers d'artefacts seulement (fiche 37-4, `sprint-status.yaml`) ; `sitesTotal`, registre des routes et
  audit des migrations recomptés après la fusion (aucun changement venu d'elle). Aucun outil MCP pour l'aperçu (V12) :
  un calcul à la demande, hors du périmètre en lecture des livres du serveur MCP ; les écritures passées se lisent par
  `kesh_ecritures` et `kesh_ecriture` (test `une_ecriture_de_reevaluation_dans_kesh_ecriture`, et recette V7).
- 2026-11-17 — **fusion de `epic/37-multi-devises` à `03f4599d`** (37-3b terminée), sur consigne de l'orchestrateur.
  Conflits résolus en gardant les deux intentions : `accounts.rs` — les deux stories avaient chacune ajouté un
  `fx_accounts_for_settings` de signature différente ; gardé celui de la 37-3b (`&Account`, route de conversion), qui
  délègue désormais à `fx_accounts_for_columns` (la règle seule, sur colonnes, celle de la 37-5) : une seule règle,
  trois enveloppes ; registre des routes — 37-3b +1 (conversion), 37-5 +2 : 155 routes, tracées 147, rejouées 29,
  enveloppes examinées 29, total 159, `sans_ecriture` 127, `no_matter` 4 ; phrase du manuel d'administration « 147 des
  155 », « 147 + 4 + 4 = 155 » ; `sitesTotal` **3350** relevé par la garde (3300 de la 37-3b + 50) ; candidates des
  libellés en dur **116** (`ecartee` 18, `conforme` 98) ; manuel utilisateur : les deux sous-sections gardées
  (réévaluation, puis écritures en devise). Audit des migrations : aucune migration de la 37-5 ; 94 fichiers et 94 lignes d'audit après la fusion
  (recomptés). La
  recette V7 gagne l'ouverture et la conversion (routes de la 37-3b).
- 2026-11-17 — revue de code P1 (Opus 5.5) : 0 CRITICAL, 0 HIGH, 6 MEDIUM, 8 LOW après dédoublonnage (28 constats
  bruts : Blind Hunter 3 MEDIUM + 9 LOW, Edge Case Hunter 3 MEDIUM + 7 LOW, Acceptance Auditor 3 MEDIUM + 3 LOW) ;
  12 corrigés (CR-P1-1 à 12), 2 reportés en dette avec issue demandée (D1 MEDIUM, D2 LOW), 7 écartés — le cours
  `request` qui laisse K9 en attention (voulu, l. 1075 et manuel), l'écart sous le centime (la cible est au centime,
  l'écart ramène le CHF dessus), l'absence de décochage dans l'avis (A36), la fenêtre des cours lus hors verrou
  (NFR-37-5-5), deux connexions du pool à l'aperçu, les clés de `rates` non contrôlées (bornées par `cles_dans`),
  les deux E2E dépendants (`mode: 'serial'`, C-37-5-5). Écarts C-37-5-1 à 6 jugés justifiés. Tests ajoutés (de
  `HEAD` avant la passe au commit de la passe) : `kesh-db/tests/fx_revaluation.rs` +5, `kesh-api/tests/fx_revaluation_e2e.rs`
  +2, `RevaluationPanel.test.ts` +3 (un test existant complété). Mutations : sans le blocker `RATE_UNAVAILABLE`, le
  test e2e rougit (`left: 0, right: 1`) ; sans les gardes de l'écran (champ, relecture, relecture de l'avis), trois
  Vitest rougissent. Gates : voir *Tests ajoutés et gates*, entrée P1. Boucle à poursuivre (MEDIUM rendus).
- 2026-11-17 — échappement commun des cellules texte des CSV de rapports (décision de Beat, ajoutée à la phase 2
  de la revue de code P1) : la fonction d'échappement des cellules CSV quitte `kesh-api` (`util`) pour
  `kesh_core::csv::csv_sanitize`, comportement identique, ses tests avec elle ; `kesh-api` la réexporte (sites
  d'appel inchangés), `kesh-report` l'applique à chaque cellule texte de ses CSV (libellés d'écriture, numéros et
  noms de compte, contreparties, nom d'exercice, noms de contact et de projet) — jamais aux montants. Tests : deux
  tests de `kesh-report` (journal ; bilan, balance et grand livre : `=`, `+`, `-`, `@`, tabulation et retour chariot
  en tête, montants négatifs restés des nombres) ; l'assertion d'un nom de compte à saut de ligne passe de la cellule
  sur deux lignes à une espace. Mutation : sans l'échappement dans `kesh-report`, les deux tests rougissent. Gate
  complet : l'entrée « Revue de code P1 » de *Tests ajoutés et gates* (5872 passés).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`).

### Completion Notes List

- **Calcul pur** (`kesh-core/src/accounting/fx_revaluation.rs`) : écart en débit net par produit `i128` contrôlé,
  arrondi au centime loin de zéro, bornes 10¹³ ; règle unique du genre d'un cours reçu (`received_rate_kind`) ; côté
  (gain, perte) ; totaux. 13 tests (dont les cinq cas de signe).
- **Dépôt** (`kesh-db/src/repositories/fx_revaluation.rs`) : `resolve_default_rates` (une seule fonction : aperçu,
  passation, K9, clôture) ; `lines_for_fiscal_year` et `build_lines` (une seule fonction des lignes : aperçu, K9,
  clôture ; la passation emploie `build_lines` sur ses comptes verrouillés) ; `not_revalued` (K9 et refus de la
  clôture, mêmes `details`) ; `previous_revaluations` (A39) ; `revaluation_label` (libellé partagé par l'aperçu et la
  passation) ; `fx_account_verdicts` (comptes de différence employés, la paire de la société revalidée une fois) ;
  `post` (verrous `companies` → antérieurs → exercice → comptes en partage → comptes de différence, puis soldes ;
  évaluation dans l'ordre de l'AC-4 ; une écriture `OD` de deux lignes par compte ; audit `fx_revaluation.posted` par
  `NewAuditLogEntry::par`, aucun sans écriture).
- **Clôture** : `fiscal_years::close` lit (a) l'exercice entier et (b) les antérieurs **avant `begin`**, puis, en
  multi-devises, la requête `EXISTS` de K9 et les cours par défaut ; étapes (b')–(d) extraites en
  `lock_with_earlier_in_tx`, partagée avec `post` ; étape (d-bis) : genre changé → 409 `OPTIMISTIC_LOCK_CONFLICT`, K9
  sur la ligne verrouillée → 409 `FOREIGN_ACCOUNTS_NOT_REVALUED`. Signature inchangée ; les 105 tests de clôture et de
  course existants (13 b1 à b3 compris) passent tels quels.
- **K9** (`closing_checks.rs`) : neuvième contrôle, `blocking`, `FX_CANDIDATES_EXIST_SQL` dans `ALL_SQL` (garde T-a2
  verte ; test qui lie son prédicat à `CANDIDATE_PREDICATE`), mono-devise sans lecture, fonctions externes inscrites à
  l'angle mort de T-a2.
- **API** : `routes/fx_revaluation.rs` (aperçu, passation, `retry_on_deadlock("fx_revaluation::post")`), erreurs
  `FX_REVALUATION_ACCOUNT_INVALID`, `FX_VALUE_OUT_OF_RANGE` (variante propre sans rang de ligne),
  `FOREIGN_ACCOUNTS_NOT_REVALUED`, `FX_RATE_INVALID {reason: tooMany}` ; libellés et action d'audit dans les quatre
  locales ; registre des routes : 153 → 155 routes, tracées 146 → 147, `no_matter` 3 → 4, examinées et rejouées
  28 → 29, `sans_ecriture` 126 → 127, total 157 → 159 ; phrase du manuel d'administration « 147 des 155 » et
  « 147 + 4 + 4 = 155 », « quatre autres routes ». *(Corrigé le 2026-11-17, revue de code P1, 37-5-CR-P1-7 : ce
  paragraphe portait les valeurs d'avant la fusion de la 37-3b — 152 → 154, 145 → 146, 27 → 28, 156 → 158, « 146
  des 154 » — ; recomptées dans `crates/kesh-api/tests/audit_route_registry.rs` (`155`, `traced, 147`,
  `examinees, 29`, `rejouees, 29`, `sans_ecriture, 127`, `tout.len()` 159) et `admin-manual.tex:3257`.)*
- **Écran** : un seul composant `RevaluationPanel.svelte` (mode `full` pour l'action « Réévaluer les devises », mode
  `notice` pour l'avis de K9) et sa logique pure `revaluation.helpers.ts` ; avis de K9 dans `FiscalYearChecks.svelte`
  (aperçu avant tout clic pour Comptable et Administrateur, liste seule pour Consultation, rien quand K9 est `ok`) ;
  « Clôturer » désactivé sous K9, 409 traité comme `EARLIER_FISCAL_YEAR_OPEN`.
- **Inventaire « huit » → « neuf »** : `closing_checks.rs`, `routes/fiscal_years.rs` (disait « sept »), tests Rust,
  `closing-checks.test-fixtures.ts`, `FiscalYearChecks.svelte` (en-tête, texte de repli), `fiscal-years.api.ts`,
  `fiscal-years.types.ts`, Vitest (`FiscalYearChecks.test.ts`, `fiscal-years-page.test.ts`), trois specs E2E, manuel
  utilisateur (préparation de la clôture, entrée K9, clôture), manuel d'administration (:1876), brochure, `README.md:48`,
  `docs/api-external.md` (note ¹⁰, codes, note ¹⁰ᵇ des routes neuves, V8) ; `fiscal-years-checks-intro` dans les quatre
  locales. `README.md:233` (v0.13.4) est historique et reste.
- **Manuel** : section « La réévaluation de fin d'exercice » (`sec:reevaluation-devises`, AC-8), `.tex` seulement.
- **Choix du développement** (pas de registre `epic-37-choix-autonomes.md` dans la branche ; écrits ici) :
  C-37-5-1 — champ additif `defaultKind` dans `rates[]` de l'aperçu : sans lui, l'écran ne distinguait un repli mensuel
  d'un cours du jour qu'en comparant des dates, faux pour un exercice qui finit un 1er du mois ;
  C-37-5-2 — sans `accountIds`, les clés de `rates` admises à l'aperçu sont les devises des comptes **admissibles** (et
  non de la seule présélection, qui exclut les devises sans cours) : sinon un cours saisi pour une devise sans cours
  serait refusé à la première lecture ; la présélection se calcule ensuite, cours reçus compris ;
  C-37-5-3 — `blockers[]` porte un `FX_ACCOUNT_INVALID` **par compte sélectionné fautif** (la passation, elle, rend le
  premier), et chaque blocker porte le message traduit du serveur, construit par le rendu de l'erreur (DRY) ;
  C-37-5-4 — mécanisme de l'atomicité (AC-7) : un déclencheur `BEFORE INSERT` posé dans la base éphémère du test refuse
  la ligne `revaluation` du troisième compte (`SIGNAL`) ; aucune des trois écritures ne subsiste ;
  C-37-5-5 — l'E2E de l'avis de K9 se joue sur le **même** exercice que l'écran (écriture tardive après la passation),
  non « sur un autre exercice » : il montre la valeur cumulée passée et le lien vers sa première écriture, puis lève K9
  et clôt ;
  C-37-5-6 — à `D = 0`, la contrepartie affichée est le compte de gains.
- **Recette V7** (`fx_recette_api_e2e.rs`) : complète depuis la fusion de la 37-3b — activation, comptes à comptes
  propres, exercice créé après l'activation, cours factices, ouverture par `{ amount, chf, side }`, quatre formes
  d'écriture, conversion EUR → USD refusée sans `completeDifference` (400 `ENTRY_UNBALANCED`, `details.entry` = 2)
  puis complétée par le serveur, réévaluation V8 à 0.91193 / 0.94947 en `request`/`manual`, balance et grand livre au
  CHF exact, compte de conversion à 0, lecture MCP. Avant la fusion, l'ouverture et la conversion manquaient (routes de
  la 37-3b).
- **Captures** (thème clair) : `kesh-gate-logs/s37-5/captures/` (prises par la spec E2E).

### File List

- *Revue de code P1 (échappement commun des CSV) et P2 :* `crates/kesh-core/src/csv.rs` (neuf), `crates/kesh-core/src/lib.rs`, `crates/kesh-report/src/csv.rs`, `crates/kesh-api/src/util.rs` ; `frontend/src/lib/features/fiscal-years/` (`RevaluationPanel.svelte`, tests) et `closing-checks-labels.ts`.
- `crates/kesh-core/src/accounting/fx_revaluation.rs` (neuf), `crates/kesh-core/src/accounting/mod.rs`
- `crates/kesh-db/src/repositories/fx_revaluation.rs` (neuf), `repositories/mod.rs`, `repositories/fiscal_years.rs`,
  `repositories/closing_checks.rs`, `repositories/accounts.rs` (`fx_accounts_for_settings`), `src/errors.rs`
- `crates/kesh-db/tests/fx_revaluation.rs` (neuf), `crates/kesh-db/tests/closing_checks.rs`
- `crates/kesh-api/src/routes/fx_revaluation.rs` (neuf), `routes/mod.rs`, `routes/fiscal_years.rs`, `src/lib.rs`,
  `src/errors.rs`, `src/audit_labels.rs`
- `crates/kesh-api/tests/fx_revaluation_e2e.rs` (neuf), `fx_recette_api_e2e.rs` (neuf), `closing_checks_e2e.rs`,
  `mcp_outils_e2e.rs`, `audit_route_registry.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/fiscal-years/` : `RevaluationPanel.svelte` (neuf), `revaluation.helpers.ts` (neuf),
  `revaluation.test-fixtures.ts` (neuf), `RevaluationPanel.test.ts` (neuf), `revaluation.helpers.test.ts` (neuf),
  `FiscalYearChecks.k9.test.ts` (neuf), `FiscalYearChecks.svelte`, `FiscalYearChecks.test.ts`,
  `closing-checks-labels.ts`, `closing-checks.test-fixtures.ts`, `fiscal-years.api.ts`, `fiscal-years.types.ts`
- `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`, `fiscal-years-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts`
- `frontend/tests/e2e/fx-revaluation.spec.ts` (neuf), `fiscal-years.spec.ts`, `fiscal-year-closing-journey.spec.ts`,
  `supplier-invoice-draft-cycle.spec.ts`
- `docs/manual/fr/user-manual.tex`, `admin-manual.tex`, `marketing-brochure.tex`, `docs/api-external.md`, `README.md`
- `_bmad-output/implementation-artifacts/37-5-reevaluation-fin-exercice.md`, `sprint-status.yaml`

### Tests ajoutés et gates

- **Tests ajoutés** (recomptés au code, périmètre `31910288` → `d818fad6`, revue comprise : aucune) : Rust **55** —
  `kesh-core` `fx_revaluation.rs` 13 (+ 2 doctests), `kesh-db` `repositories/fx_revaluation.rs` 4,
  `tests/fx_revaluation.rs` 26, `repositories/closing_checks.rs` +1 (2 → 3), `kesh-api` `routes/fx_revaluation.rs` 2,
  `tests/fx_revaluation_e2e.rs` 7, `tests/fx_recette_api_e2e.rs` 1, `tests/mcp_outils_e2e.rs` +1 (31 → 32) ; Vitest
  **30** — `revaluation.helpers.test.ts` 11, `RevaluationPanel.test.ts` 11, `FiscalYearChecks.k9.test.ts` 4,
  `fiscal-years-page.test.ts` +4 (29 → 33) ; E2E **2** (`fx-revaluation.spec.ts`). Tests existants adaptés (huit →
  neuf, compteurs des gardes).
- **Gates complets au dernier commit de code `d818fad6`** (fusion de la 37-3b comprise), base `kesh_s37_5` sur
  `kesh-mariadb-dev3` remise à zéro (94 migrations, seed) : `heavy.sh scripts/test-fast.sh --ci` — fmt, clippy
  `-D warnings`, nextest **5863 passés, 13 ignorés** (`gate-backend-3.log`) ; frontend `check` 0 erreur,
  `lint-i18n-ownership` PASS, Vitest **4104 / 263 fichiers**, `build` vert (`gate-frontend-3.log`).
- **E2E complet au même commit** (`LANG=fr_CH.UTF-8`, port 3376, base `kesh_e2e_s37_5` neuve) : **443 passés,
  11 échecs, 17 ignorés** (`e2e-3.log`, 43,2 min). Les onze : les sept de la KF-029 ; `backup-restore-new-install.spec.ts:262`
  et `:381`, rouges aussi aux E2E de la 37-2, de la 37-3a et de la 37-4 ; `opening-balances.spec.ts:156`, déjà relevée
  par la 37-1 et la 37-4 (`issues-a-ouvrir-epic-37.md` n° 4 ; rejouée seule avant la fusion,
  `e2e-rejeu-opening.log` : rouge, délai du clic) ; `bank-import.spec.ts:130`, **verte rejouée seule**
  (`e2e-rejeu-bank-import.log`, 7/7 : pollution). Les deux tests de `fx-revaluation.spec.ts` passent à chaque
  exécution (`e2e-1.log`, `e2e-2.log`, `e2e-3.log`).
- Avant la fusion de la 37-3b : gate backend 5836/5836 (`gate-backend-2.log`), Vitest 3940, E2E 434–435 passés aux
  mêmes échecs connus (`e2e-1.log`, `e2e-2.log`).
- **Captures** (thème clair, prises par la spec au dernier E2E) : `kesh-gate-logs/s37-5/captures/` —
  `37-5-ecran-repli-mensuel.png`, `37-5-ecran-cours-du-jour.png`, `37-5-ecran-apres-passation.png`,
  `37-5-k9-avis-apercu.png`, `37-5-k9-leve.png`.
- Journaux : `kesh-gate-logs/s37-5/`.
- **Revue de code P1 (2026-11-17)**, journaux `kesh-gate-logs/s37-5/review-p1/` : gate ciblé (`cible-1.log`,
  binaires `fx_revaluation`, `fx_revaluation_e2e`, `fx_recette_api_e2e`, `closing_checks`, `closing_checks_e2e` :
  66/66 ; `textes-1.log`, `textes_coherents` et `audit_route_registry` : 40/40) ; **gate complet** sur l'arbre qui
  porte les correctifs de la P1 **et** l'échappement commun des cellules texte des CSV de rapports (commit suivant),
  base `kesh_s37_5` remise à zéro (94 migrations, seed) : `heavy.sh scripts/test-fast.sh --ci` — fmt, clippy
  `-D warnings`, nextest **5872 passés, 13 ignorés** (`gate-backend-p1.log`) ; frontend `check` 0 erreur,
  `lint-i18n-ownership` PASS, Vitest complet **4107 / 263 fichiers** (`front-vitest.log`), `build` vert
  (`front-build.log`). **E2E complet non rejoué** à cette passe : à rejouer au dernier commit de code de la story.
- 2026-11-16 — revue de code P2 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 9 LOW après dédoublonnage (Blind Hunter
  3 MEDIUM + 5 LOW, Edge Case Hunter 6 LOW, Acceptance Auditor 1 MEDIUM + 6 LOW) ; D1 et D2 de la P1 corrigées.
  Corrigés : **D1** (MEDIUM, règle retenue dans « Règle retenue pour D1 » ci-dessus : prédicat sans `postable`,
  motif `NOT_POSTABLE` à K9, ligne `ACCOUNT_NOT_POSTABLE`) ; **D2** (LOW : blocker `FX_VALUE_OUT_OF_RANGE` par compte
  sélectionné, plus de refus de tout l'aperçu) ; **CR-P2-1** (MEDIUM : à sélection vide, « Récupérer le cours »
  recharge l'aperçu entier) ; **CR-P2-2** (MEDIUM, spec : `blockers[]` porte six codes, FR-37-5-1, AC-1 et l'epic
  amendés) ; **CR-P2-3** (LOW : « Réévaluer » désactivé pendant le chargement de l'aperçu) ; **CR-P2-4** (LOW : aperçu
  qui réécrivait « actif » au lieu d'`is_eligible`) ; **CR-P2-5** (LOW : commentaire sur l'audit d'un cours de compte non
  admissible) ; **CR-P2-6** (LOW : guide d'API, `accountIds` vide pour `tooMany`) ; **CR-P2-7** (LOW : File List et Notes
  de version, échappement des CSV). Écartés ou laissés : deux connexions du pool à l'aperçu (déjà écarté en P1,
  `companies::find_by_id` prend le pool), clic avalé sur « Passer les écritures » juste après une saisie (aucun envoi
  faux), ordre de `notEligible` et de `FX_VALUE_OUT_OF_RANGE` entre aperçu et passation par l'API directe seulement,
  erreur de relecture dans `postError`, cas `FX_ACCOUNT_INVALID` de l'AC-7 non rejoués au dépôt (`DefaultRevenue`, gain
  avant perte : portés par la 37-1), cours saisi qui laisse K9 en attention (voulu, P1). **U1** (aperçu après
  restauration d'un `.keshbackup`) **non écrit** : il exige le montage complet de l'import, hors « quelques lignes » ;
  reste en dette. Tests ajoutés (de `HEAD` avant la passe) : `kesh-db/tests/fx_revaluation.rs` +1, `kesh-api/tests/fx_revaluation_e2e.rs` +2,
  Vitest +2 (`RevaluationPanel.test.ts` +2 ; `FiscalYearChecks.k9.test.ts` complété). Aucune mutation rejouée à cette
  passe. Gates (base `kesh_s37_5` remise à zéro, 94 migrations, seed) : `cargo fmt` ; `clippy --workspace
  --all-targets -D warnings` vert ; `heavy.sh scripts/test-fast.sh --ci` **5875 passés, 13 ignorés**
  (`review-p2/gate-backend-1.log`) ; frontend `check` 0 erreur, `lint-i18n-ownership` PASS, Vitest complet **4109 / 263
  fichiers**, `build` vert (`review-p2/`) ; `sitesTotal` 3350 → 3352 relevé par la garde. E2E complet non rejoué (à
  la clôture). Boucle à poursuivre : la P2 a rendu des MEDIUM.
- 2026-11-17 — revue de code P3 (Opus) : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 1 LOW. Passe braquée sur la remédiation P2
  (`2a370c4e`), complète sur le rayon de la règle D1 (prédicat, aperçu, K9, clôture, passation, création de compte,
  textes). **CR-P3-1** (HIGH) : un compte en devise mouvementé devenu parent bloquait la clôture sans recours — règle
  ajoutée, sous-compte refusé (amendement ci-dessus) ; manuel (`user-manual.tex`, § *La devise d'un compte* et K9),
  guide d'API (refus de `POST /accounts` et table des codes). **CR-P3-2** (MEDIUM, spec) : FR-37-5-1, AC-4 et la note
  de K9 disaient encore que l'aperçu **refuse** `FX_VALUE_OUT_OF_RANGE`, ce que la D2 de la P2 a remplacé par un blocker
  par compte : amendés. **CR-P3-3** (LOW, test) : aucun test du compte parent — ajouté (parent sans solde propre non
  listé, enfant seul réévalué, K9 `ok` ; compte CHF mouvementé qui reçoit toujours un sous-compte). Vérifiés sans
  finding : prédicat unique (`CANDIDATE_PREDICATE` employé par `candidate_currencies`, `lines_for_fiscal_year` et
  `FX_CANDIDATES_EXIST_SQL`, gardé par test), archivé / résultat / CHF hors prédicat ou non admissibles, aucun double
  compte (`balances_as_of` ne lit que les lignes du compte), `is_eligible` aux trois sites, blocker
  `FX_VALUE_OUT_OF_RANGE` par compte sélectionné, rechargement de « Récupérer le cours » à sélection vide, « Réévaluer »
  désactivé pendant le chargement, six codes de `blockers[]` à l'epic et au guide. Tests ajoutés (de `HEAD` avant la
  passe) : `kesh-db/tests/fx_revaluation.rs` +1 (`sous_compte_sous_un_compte_de_bilan_en_devise`),
  `kesh-api/tests/account_currency_e2e.rs` +1 (`sous_compte_refuse_sous_un_compte_en_devise_mouvemente`) ; mutation
  (garde retirée de `accounts::create`) rejouée : les deux tests rougissent (`review-p3/mutation-1.log`), garde
  rétablie. Gates (journaux `kesh-gate-logs/s37-5/review-p3/`) : `cargo fmt` ; `clippy --workspace --all-targets
  -D warnings` vert (`clippy-1.log`) ; ciblé `fx_revaluation`, `fx_revaluation_e2e`, `account_currency_e2e`,
  `account_currency_repository` **79/79** (`cible-1.log`) ; base `kesh_s37_5` remise à zéro (94 migrations, seed),
  `heavy.sh scripts/test-fast.sh --ci` **5877 passés, 13 ignorés** (`gate-backend-1.log`). Frontend non touché (seuls
  les catalogues `.ftl` changent) : `lint-i18n-ownership` PASS, Vitest `i18n-keys.test.ts` 11/11 ; Vitest complet et
  `build` non rejoués. E2E complet non rejoué (à la clôture). Axes non exercés : E2E Playwright, concurrence de la
  création d'un sous-compte avec une écriture sur le parent (raisonnée, non testée). Boucle à poursuivre : la P3 a
  rendu un HIGH et un MEDIUM.
- 2026-11-17 — revue de code P4 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié. LOW 37-5-CR-P4-1 (`accounts.rs`, doc) : le correctif P3 avait inséré `refuse_foreign_parent_with_entries_in_tx` entre la doc de `has_journal_line_locked` et sa fonction — la doc de l'une préfixait l'autre ; remise en place, aucune ligne de code changée. Vérifié au code : ordre des verrous (parent `FOR UPDATE`, puis lignes `LOCK IN SHARE MODE`, même ordre que `update` l.776 ; le cycle avec une écriture concurrente se résout en interblocage rejoué par `retry_app_on_deadlock("accounts::create")`, `routes/accounts.rs:415`) ; `parent_id` immuable au `PUT` (aucun chemin de reparentage), import de plan, seed, onboarding sans devise (hors règle, sans objet) ; compte de résultat ou CHF libre (`CANDIDATE_PREDICATE`, test CHF) ; message des 4 locales (`$number`, `$currency`), manuel et guide d'API conformes au code. `heavy.sh scripts/test-fast.sh --ci` **5877 passés, 13 ignorés** (`review-p4/gate-backend-1.log`). Non exercés : E2E Playwright, concurrence réelle (raisonnée), compte de produit en devise non testé (même prédicat).
- 2026-11-17 — **boucle de revue close** (orchestrateur) : P1 (Opus) 6 MEDIUM → P2 (Sonnet) 3 MEDIUM (D1) → P3 (Opus) 1 HIGH, 1 MEDIUM → P4 ciblée (Sonnet) 1 LOW, commentaires seuls. Clôture : Vitest complet et build, E2E complet au dernier commit.
