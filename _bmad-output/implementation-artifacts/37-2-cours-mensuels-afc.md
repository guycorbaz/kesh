# Story 37-2 : table des cours (mensuels AFC et journaliers), cours saisis par la société, récupération des cours officiels (AFC / OFDF)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/37-multi-devises`. Epic 37, C-37-1 (amendé par D9/D10, CR-1),
C-37-6, D9, D10. Suit 37-1. **Migration non breaking** : deux tables système `_kesh_*` et **une table applicative**
`exchange_rates` (ex-`exchange_rate_overrides`, U1). Révisée le 2026-11-14 pour la revue de Beat (CR-2 : R1, R2, R4, R5, R6), qui prime sur le
registre antérieur : la table applicative **ne referme plus** la fenêtre d'import (R1, liste close de 37-1 AC-6) ; la
récupération est ciblée sur les exercices multi-devises (R2) et suit le calendrier de publication (R4) ; les données
reprises sont filtrées strictement (R5) ; l'écran devient un tableau annuel par exercice (R6). Révisée de nouveau pour
la revue CR-3 de Beat, qui prime sur CR-2 et R4 amendé : **S1** — calendrier « quotidien jusqu'au premier cours du
mois suivant, puis pause de 24 jours, au plus tard jusqu'au 25 » (AC-4) ; **S4** — une colonne par mois de l'exercice,
quelle que soit sa durée (AC-5, AC-6) ; **S6** — un cours AFC **republié** autrement ne remplace pas le cours en
vigueur : il est gardé à part, signalé en rouge avec ↑ pour l'adopter, et annoncé à la connexion jusqu'à son
acquittement (AC-1, AC-3, AC-5, AC-6). Révisée enfin pour la revue 2 de Beat (CR-4), qui prime sur tout ce qui
précède : **T4** — le cours d'ouverture est le cours de **clôture** de l'exercice précédent : genre `month` = cours
mensuel du **dernier mois de l'exercice précédent** (mois de la veille de l'ouverture), non plus du premier mois
(AC-4, AC-5, AC-6) ; **T2** — tests d'exercices au 1er janvier, au 1er avril et au 1er juillet, plus six et
dix-huit mois (AC-4, AC-6). Révisée en dernier pour la revue 3 de Beat (CR-5), qui **prime sur tout ce qui précède**,
C-37-1, S6, A10, A11, A16 et les colonnes `republished_*` compris : **U1** — deux tables renommées et réparties :
`exchange_rates` (**applicative**, par société, sauvegardée ; ex-`exchange_rate_overrides`) porte **le cours retenu
par la société** et sa `source` (`afc` | `manual`), reçoit **automatiquement** une copie de chaque cours récupéré utile
à un exercice multi-devises de la société, pour les devises de ses comptes (A24), et garde la source d'un cours pour toujours ; `_kesh_fetched_exchange_rates`
(**système** ; ex-`_kesh_exchange_rates`) porte la **dernière** valeur publiée par la source, et une correction de
l'AFC la met à jour sans jamais toucher `exchange_rates` ; plus de colonnes `republished_*`, plus d'adoption globale ;
une seule valeur par case à l'écran, « AFC corrigé : x » en rouge avec ↑ ; avertissement et acquittement **par
société** (AC-1, AC-3, AC-3 bis, AC-4, AC-5, AC-6, AC-7, AC-9) ; **U3** — le ↑ d'un cours AFC corrigé reprend la valeur
pour cette société, **directement** (A23 ; le dialogue avec l'option « toutes les sociétés » est spécifié, masqué ; AC-6) ; **U4** — changer le genre de clôture
propose d'appliquer le genre correspondant à l'ouverture de l'exercice suivant (AC-5, AC-6 ; renvoi de 37-1 AC-3 bis) ;
**U5** — A2, A7, A9, A10, A16 sont des AC de cette fiche, A11 est retiré (AC-4, AC-5, AC-6).*

## Story

En tant que comptable en multi-devises, je veux que Kesh récupère seul, dès leur publication, les cours mensuels AFC
(publiés d'avance pour le mois, pour la TVA, par l'AFC, données de l'OFDF) des mois de mes exercices multi-devises,
pour toutes les devises, et les cours journaliers OFDF d'ouverture et de clôture de ces exercices, que ma société
**retienne** une copie de chacun avec sa source, que je voie les cours d'un exercice dans un tableau annuel, **saisisse
mon propre cours** pour une devise et un mois ou un jour — jamais écrasé par une récupération —, sois averti quand l'AFC
corrige un cours qu'elle a déjà publié et reprenne d'un clic la valeur publiée, et que je choisisse, par exercice, le genre du cours
d'ouverture et de clôture, afin que mes écritures en devise, mes soldes d'ouverture et ma réévaluation reçoivent le bon
cours sans saisie.

## Source (établie le 2026-11-14, fichier d'epic § *Source des cours*)

- `GET https://www.backend-rates.bazg.admin.ch/api/xmlavgmonth?j=<AAAA>&m=<M>` ;
- `<monatsmittelkurs xmlns="https://www.backend-rates.bazg.admin.ch/avgratesmonth" …><monat>AAAA-MM</monat><devise
  code="usd"><land_de>…</land_de><land_fr>…</land_fr><land_it>…</land_it><waehrung>1 USD</waehrung><kurs>0.8846</kurs>…`
  — UTF-8, espace de noms par défaut, 73 devises et 14 Ko en 2024-12 ; code en minuscules ; cours à quatre décimales ;
- **l'unité est un entier positif** en tête de `<waehrung>` : relevé 2024-12, 19 devises à `1`, 42 à `100`, 7 à
  `1000` (CLP, CRC, KZT, NGN, LKR, TZS, HUF), 5 à `10000` (IDR, KHR, COP, LBP, VND) — jamais « 1 ou 100 » seulement ;
  des cours très petits existent (`1 ARS` → `0.0009`) ;
- **calendrier de publication (R4, source ESTV *MWST Monatsmittelkurse*, lue par Beat le 2026-11-14)** : le cours du
  mois M est la moyenne des cours journaliers du 25 de M−2 au 24 de M−1, **publiée le 25 de M−1** (ou le jour
  ouvrable suivant), applicable dès le 1er de M ; l'heure n'est pas publiée. Le relevé du 2026-11-14 (`xmlavgmonth`
  sans paramètre rendait `2026-11`) montre le mois courant publié ; mais **un relevé du même jour le contredit pour le
  mois suivant** (validation CR-2 P2) : `j=2026&m=12` rendait dès le **14.11.2026** (en-tête `Date` du serveur) un
  `<monat>2026-12</monat>` complet (EUR 0.9295), `j=2027&m=1` la racine vide — le mois suivant peut donc être publié
  **avant le 25**, et la définition « du 25 de M−2 au 24 de M−1 » ne peut alors être exacte (le 24.11 n'était pas
  passé). **À confirmer par Beat (R4).** Kesh n'en dépend pas pour sa justesse (S1, CR-3, qui remplace R4 amendé) : la
  tâche automatique vérifie **une fois par jour** tant que le mois courant ou le mois suivant manque, puis, le mois
  suivant obtenu, **se met en pause 24 jours au plus, jamais au-delà du 25** du mois qu'elle vient d'obtenir, et
  reprend chaque jour pour le mois d'après (AC-4) ; la récupération à la demande accepte le mois suivant à
  toute date (non publié → `404 RATE_NOT_PUBLISHED`, AC-5) ;
- mois non publié : élément racine vide → « pas encore publié » ;
- la réponse peut porter un autre mois que celui demandé (sans paramètre : mois courant) : **le `<monat>` lu fait foi** ;
  un écart avec le mois demandé est une erreur (rien n'est écrit).
- **Terminologie** (fichier d'epic, précisée par R4) : ce n'est **pas la moyenne du mois** qu'il désigne — c'est la
  moyenne des cours du 25 au 24 qui précède (définition de R4, à confirmer : relevé ci-dessus), **publiée d'avance**
  et applicable tout le mois. Textes, libellés et
  manuels disent « cours mensuel AFC », jamais « cours moyen » ni « moyenne du mois » ; le manuel l'explique ainsi (le
  nom allemand de la page ESTV, *Monatsmittelkurse*, et la racine XML `monatsmittelkurs` restent des noms de la source).

### Cours journaliers (D10, relevés le 2026-11-14 par deux appels `GET`)

- `GET https://www.backend-rates.bazg.admin.ch/api/xmldaily?d=<AAAAMMJJ>` (le paramètre `locale` n'a pas d'effet sur
  les données) ;
- `<wechselkurse xmlns="https://www.backend-rates.bazg.admin.ch/xmldaily" …><datum>31.12.2024</datum><zeit>04:15:03</zeit>
  <gueltigkeit>01.01.2025,02.01.2025,03.01.2025</gueltigkeit><devise code="eur"><land_de>…</land_de><land_fr>…</land_fr>
  <land_it>…</land_it><land_en>…</land_en><waehrung>1 EUR</waehrung><kurs>0.94947</kurs></devise>…` — espace de noms
  **différent** de celui des cours mensuels, élément `land_en` en plus ; **cinq décimales** ; mêmes unités (relevé du
  30.06.2024 : 74 devises, 19 à `1`, 43 à `100`, 7 à `1000`, 5 à `10000`, 17 Ko) ; `d=20241231` rend USD 0.91193 et
  EUR 0.94947, les cours de la réévaluation des livres 2024 de Beat (D10) ;
- **un jour sans publication rend la dernière publication antérieure** : `d=20240630` (un dimanche) rend
  `<datum>28.06.2024</datum>` (le vendredi), `<gueltigkeit>29.06.2024,30.06.2024,01.07.2024</gueltigkeit>`. Le cours
  journalier **d'une date D** est donc, par définition dans Kesh, celui que rend `xmldaily?d=D` : la publication du
  jour D, ou la dernière qui la précède. `<gueltigkeit>` n'est pas lu (Kesh suit la convention des livres de Beat :
  la publication **datée** du 31.12, non celle « valable » le 31.12) ;
- **pas de publication : HTTP 404, corps JSON** — relevé le 2026-11-14 pour `d=20301231` : `404`
  `{"error":"Aucune donnée disponible","message":"Aucun taux de change trouvé pour la période demandée"}` (et non une
  racine vide, comme `xmlavgmonth`). Le client traite un **404** de `xmldaily` comme « pas encore publié »
  (`NotPublished`, → `404 RATE_NOT_PUBLISHED`, mémorisé 15 minutes), **jamais** comme `http_status:404` : `fetch_day` le
  rend par une variante propre de `SourceError` (p. ex. `SourceError::NotPublished`), que l'appelant traite comme la
  racine vide ; tout autre statut non 2xx de `xmldaily`, et **tout** non 2xx de `xmlavgmonth` (404 compris), reste
  une erreur (`http_status:<n>`). Relevé aussi : `d=20000103` rend `<datum>03.01.2000</datum>` et `d=20000101` rend
  `<datum>31.12.1999</datum>` (la borne 2000-01-01 de l'AC-5 est servie) ; les éléments `<land_en/>` peuvent être
  **vides** (ignorés) ; `<gueltigkeit>` peut ne porter qu'une date ;
- contrôle de la réponse : `<datum>` lu ≤ D et D − `<datum>` ≤ 7 jours, sinon erreur `date_mismatch` (rien n'est
  écrit) ; racine vide → « pas encore publié », comme pour `xmlavgmonth` (par prudence : le cas réel relevé est le 404) ; une date ≥ aujourd'hui (Europe/Zurich) n'est pas demandée (AC-5) : la publication du jour tombe
  tôt le matin (`<zeit>` 04:15:03 le 31.12.2024, 07:00:05 le 13.11.2026 : l'heure varie) et, avant, la source rendrait
  celle de la veille sous la date du jour. **Relevé le 2026-11-14** : `d=20261114` (le jour même, un samedi) et
  `d=20261115` (demain) rendent **tous deux** `<datum>13.11.2026</datum>`, sans erreur : une date future n'est jamais
  refusée par la source, et `date_mismatch` ne la voit pas (`<datum>` ≤ D) — seule la garde « strictement antérieur à
  aujourd'hui » (AC-5) empêche de stocker la publication du vendredi sous la date du lendemain.

## Exigences

*(B26, 2026-11-15 : tirées des critères d'acceptation ci-dessous, sans en créer ni en modifier.)*

**Fonctionnelles**

- FR-37-2-1 — La migration crée deux tables système — `_kesh_fetched_exchange_rates` (dernière valeur publiée par source, devise, genre et date) et `_kesh_exchange_rate_fetch` (état de la dernière récupération, codes d'erreur classés) — et la table applicative `exchange_rates` (cours retenu par la société et sa `source`, au format seul ; une source autre que `afc` se lit sans erreur et se traite comme `manual`) (AC-1).
- FR-37-2-2 — `parse_monthly_rates` et `parse_daily_rates` lisent le XML de l'AFC/OFDF par noms locaux, par une seule analyse des devises ; `rate_per_unit = published_rate / published_unit`, arrondi à 10 décimales ; une devise invalide ou hors bornes est écartée sans bloquer les autres ; une seule fonction publique des bornes d'un cours (> 0, ≤ 10 décimales, ≤ 10⁶) sert aussi le `PUT` d'un cours, 37-3a et 37-5 (AC-2).
- FR-37-2-3 — Une source de cours (`RateSource`, `fetch_month`, `fetch_day`) est portée par `AppState` ; une récupération écrit la valeur publiée par la règle (a) insérée, (b) confirmée, (c) corrigée et remplacée (`first_fetched_at` jamais avancé), en une transaction par mois ou par jour, et **ne modifie jamais une ligne existante d'`exchange_rates`**, ni `manual` ni `afc` (AC-3).
- FR-37-2-4 — Les cours publiés utiles d'un exercice multi-devises se copient dans `exchange_rates`, pour les seules devises des comptes de la société (archivés compris), par insertion seule (une ligne présente n'est jamais modifiée ; une course entre deux copies est rattrapée sur l'unicité, jamais en 500), sans audit propre : à chaque récupération, à la création d'un exercice multi-devises, au changement de genre et quand un compte reçoit une devise — jamais à l'activation ; la société ne lit que `exchange_rates` (AC-3 bis).
- FR-37-2-5 — La récupération automatique ne vise que les exercices multi-devises ouverts, mois et jours utiles seulement : quotidienne tant que le mois courant ou le mois suivant manque, puis en pause jusqu'au plus tôt de « date d'obtention + 24 jours » et du 25 du mois obtenu, sans reprendre avant le 1er de ce mois (la pause peut donc dépasser 24 jours, le réveil ne passe jamais le 25), et jusqu'au 1er du mois suivant si ce mois n'est couvert par aucun exercice multi-devises ouvert ; lancée aussi à la connexion (hors le seul mois suivant) ; chaque tour revérifie en plus le mois courant, hors plafond, sans jamais déclencher un tour ni changer le réveil ; ni tâche ni déclenchement en mode test ; dates en Europe/Zurich (AC-4).
- FR-37-2-6 — Le cours effectif est unique : le cours retenu par la société, avec sa source, lu par une seule fonction de dépôt ; `GET /exchange-rates`, `GET /exchange-rates/status`, `GET /exchange-rates/lookup` (lecture seule, 404 `RATE_UNAVAILABLE` à forme unique) et le tableau annuel `GET /fiscal-years/{id}/exchange-rates` le rendent ; `POST /exchange-rates/fetch` récupère un mois (≤ mois suivant) ou un jour (strictement passé), 404 `RATE_NOT_PUBLISHED`, 503 `RATE_SOURCE_UNAVAILABLE` sans bandeau dégradé, jamais 500 ni 502 ; toutes ces routes rendent 409 `MULTI_CURRENCY_DISABLED` en mono-devise, dans l'ordre des refus de l'AC-5 (AC-5).
- FR-37-2-7 — L'écran *Paramètres → Cours de change* montre un tableau annuel par exercice multi-devises, une colonne par mois quelle que soit la durée, plus Ouverture et Clôture : une valeur par case, modifiable, avec la mention de sa source et de la valeur publiée et le ↑ ; choix des deux genres et dialogue de l'exercice suivant ; « Récupérer les cours manquants » et « Revérifier les cours publiés » ; exercice clos et rôle Consultation en lecture seule (« Récupérer » reste offert sur un exercice clos) ; bandeau des corrections à acquitter (AC-6).
- FR-37-2-8 — `exchange_rates` est sauvegardée et restaurée avec la société, sources comprises, et sert dès l'import ; une sauvegarde antérieure sans la table se réimporte (liste close de 37-1 AC-6, fenêtre d'import inchangée) ; les tables `_kesh_*` ne sont pas touchées par l'import, sont vidées par `_test/seed` et `_test/reset`, et la remise à zéro de démonstration vide `exchange_rates` en gardant les valeurs publiées ; `exchange_rates` est exportée en CSV (cours à 10 décimales) ou, à défaut, hors export avec un motif écrit (AC-1, AC-7).
- FR-37-2-9 — Cours de la société et cours d'une ligne restent deux niveaux distincts ; saisie et reprise (↑) sont auditées, la copie ne l'est pas ; une ligne d'écriture est `afc` si son cours égale le cours `afc` retenu par la société, jamais la valeur publiée (AC-9).
- FR-37-2-10 — `PUT /exchange-rates/rates` saisit un cours `manual` par unité (400 `FX_RATE_INVALID` ; 409 `OPTIMISTIC_LOCK_CONFLICT`, deux créations simultanées comprises, jamais 500) ; `POST /exchange-rates/rates/reset` (↑) reprend en `afc` la valeur publiée que l'écran a montrée (409 si elle a changé), ou retire un cours saisi sans valeur publiée (une ligne `afc` sans valeur publiée : 404) ; la société de l'utilisateur seule (l'option « toutes les sociétés » est spécifiée, non livrée) ; ni l'un ni l'autre ne réécrit une écriture passée (AC-5).
- FR-37-2-11 — `PUT /fiscal-years/{id}/rate-kinds` règle les genres d'ouverture et de clôture (404, 409 `MULTI_CURRENCY_DISABLED`, 400 `FISCAL_YEAR_CLOSED`, no-op sans audit) et, avec `applyToNextOpening`, applique le genre de clôture changé à l'ouverture de l'exercice suivant s'il est ouvert (`applied: false` s'il est clos), sans modifier aucune écriture (AC-5, AC-6).
- FR-37-2-12 — Une correction de l'AFC ne change le cours retenu d'aucune société : elle s'annonce par société (`GET /exchange-rates/notices` : lignes `afc` qui diffèrent de la valeur publiée, non acquittées à cette valeur), s'acquitte par société et par valeur (`POST /exchange-rates/notices/acknowledge`, sans toucher `rate`, `source` ni `version`) et se reprend par le ↑ (AC-5, AC-6).

**Non fonctionnelles**

- NFR-37-2-1 — La migration est non breaking : DDL seul (AC-1).
- NFR-37-2-2 — L'arrondi est déterministe (`MidpointAwayFromZero`) : la valeur lue est celle qui est stockée (AC-2).
- NFR-37-2-3 — Appels sortants bornés : une récupération à la fois par unité (`rate_fetch_lock`), mémoire de 15 minutes des échecs et non-publications, au plus 36 unités par tour plus une revérification, au plus un tour de connexion par quart d'heure ; délais 5 s et 20 s, réponse ≤ 1 MiB (AC-3, AC-4, AC-5).
- NFR-37-2-4 — Aucun appel réseau en test ni en CI : source factice par défaut, pilotée en mode test par `POST /_test/exchange-rates/source`, tâche automatique non lancée en mode test ; `lookup` après import répond sans appel sortant (AC-3, AC-4, AC-7).
- NFR-37-2-5 — Audit : `exchange_rates.fetched` (entité `installation`, sur succès seulement), `exchange_rate.created`/`updated`/`reset`, `exchange_rates.corrections_acknowledged`, `fiscal_year.rate_kinds_updated` ; récupérations automatiques et copies non auditées ; routes mutantes au registre (`Traced, SansEcritureAuJournal`), rejouées sur interblocage (AC-4, AC-5, AC-9).
- NFR-37-2-6 — Quatre locales (écran, erreurs dont `FX_RATE_INVALID`, libellés d'audit et de type d'entité) ; manuel utilisateur et manuel d'administration (accès sortant HTTPS, `HTTPS_PROXY`, réimport des sauvegardes antérieures, phrase du registre des routes), `.tex` seulement, PDF non commités, régénérés à la release (AC-8).
- NFR-37-2-7 — RBAC : `POST /fetch`, `PUT …/rates`, `POST …/rates/reset`, `PUT /rate-kinds` et les deux routes de notices réservés à l'Administrateur et au Comptable (Consultation 403) ; `lookup` et le tableau annuel ouverts à tout rôle authentifié (AC-5).
- NFR-37-2-8 — Source tenue pour hostile : URL de production constante, sans variable d'environnement, HTTPS seul, sans redirection ni cookies, `User-Agent` sans donnée personnelle ; XML filtré avant toute écriture (DOCTYPE et entités non prédéfinies refusés, tailles, profondeur, grammaire des nombres, codes ISO 4217 hors `CHF`) ; erreurs stockées en codes classés, jamais le texte d'une erreur `reqwest` (AC-1, AC-2, AC-3).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000002_exchange_rates.sql` (non breaking, DDL seul) :
   - **`_kesh_fetched_exchange_rates`** (table **système**, U1, CR-5 ; ex-`_kesh_exchange_rates`) : les cours **tels
     que publiés** par la source, une ligne par (`source`, devise, genre, date), la **dernière** valeur publiée —
     `(source VARCHAR(12) CHARACTER SET ascii COLLATE ascii_bin NOT NULL, currency CHAR(3) CHARACTER SET ascii COLLATE
     ascii_bin NOT NULL, kind VARCHAR(5) CHARACTER SET ascii COLLATE ascii_bin NOT NULL, rate_date DATE NOT NULL,
     rate_per_unit DECIMAL(20,10) NOT NULL, published_unit INT NOT NULL, published_rate DECIMAL(19,6) NOT NULL,
     source_date DATE NULL, first_fetched_at DATETIME(3) NOT NULL, fetched_at DATETIME(3) NOT NULL, value_changed_at
     DATETIME(3) NOT NULL, PRIMARY KEY (source, currency, kind, rate_date), CHECK (BINARY source REGEXP '^[a-z]{2,12}$'),
     CHECK (BINARY kind IN ('month','day')), CHECK (kind <> 'month' OR DAY(rate_date) = 1), CHECK ((kind = 'day') =
     (source_date IS NOT NULL)), CHECK (rate_per_unit > 0), CHECK (published_unit > 0), CHECK (BINARY currency REGEXP
     '^[A-Z]{3}$'))`. `source` = `afc` seule aujourd'hui (OFDF/AFC, mensuels et journaliers) ; le `CHECK` ne porte que
     le **format** (A28, comme `exchange_rates.source`) : une autre source (p. ex. `ecb`) s'ajoute sans modifier le
     schéma ; `rate_date` = le 1er
     du mois pour `month`, la date **demandée** D pour `day` ; `source_date` = le `<datum>` lu pour `day` (≤ D, cf.
     *Cours journaliers*), `NULL` pour `month` ; `INT` et non `SMALLINT` (unités jusqu'à 10000 relevées) ;
     `published_rate DECIMAL(19,6)` porte les cinq décimales des cours journaliers ; `currency` a **la même définition**
     qu'`accounts.currency` (37-1 AC-1 ; depuis T3, CR-4, la devise d'une ligne d'écriture n'est plus stockée sur la
     ligne — 37-3a — et `accounts.currency` est la seule colonne de devise d'une écriture), et les `CHECK` portent
     `BINARY` comme dans 37-1 : la table, en `utf8mb4_unicode_ci` par défaut, confondrait sinon `usd` et `USD`.
     **Horodatages** (UTC) : `first_fetched_at` posée à l'insertion et **jamais** mise à jour (date d'obtention que lit le
     calendrier de l'AC-4, S1 ; distincte de `fetched_at`, A16) ; `fetched_at` = la dernière récupération qui a
     **confirmé** la ligne (AC-3) ; `value_changed_at` = la récupération qui a apporté la **valeur actuelle**
     (= `first_fetched_at` à l'insertion ; avancée par une correction de l'AFC, AC-3 (c) : l'écran et l'avertissement
     disent « corrigé le … »). **Plus aucune colonne `republished_*` ni `notice_acknowledged_at`** (S6 tel qu'écrit par
     CR-3 est remplacé par U1) : une correction de l'AFC **remplace** la valeur de cette table ; l'écart avec le cours
     retenu par une société se lit en comparant les deux tables, et son acquittement vit sur la ligne de la société
     (`exchange_rates`, ci-dessous) ;
   - **`exchange_rates`** (table **applicative**, U1, CR-5 ; ex-`exchange_rate_overrides`) : **le cours retenu par la
     société**, une ligne par (société, devise, genre, date), avec sa source — `id BIGINT AUTO_INCREMENT PRIMARY KEY`,
     `company_id BIGINT NOT NULL` (FK `companies`, `RESTRICT`), `currency` (même définition, `CHECK BINARY currency
     REGEXP '^[A-Z]{3}$' AND BINARY currency <> BINARY 'CHF'`), `kind` et `rate_date` (mêmes définitions et mêmes
     `CHECK` que ci-dessus), `rate DECIMAL(20,10) NOT NULL` (`CHECK rate > 0`, **par unité**, comme `rate_per_unit`),
     `source VARCHAR(12) CHARACTER SET ascii COLLATE ascii_bin NOT NULL` (`chk_exchange_rates_source` : `BINARY source
     REGEXP '^[a-z]{2,12}$'` — **format seul, aucune énumération** (A28, CR-5) ; `afc` : copie d'un cours récupéré, ou
     valeur publiée reprise par ↑ ; `manual` : cours saisi par la société ; les valeurs admises sont contrôlées **par le
     code** (`RateOrigin { Afc, Manual, Other(String) }` : la lecture d'une valeur inconnue est **tolérante** (A30) —
     `Other(source)`, affichée telle quelle, jamais une erreur interne ; seules `afc` et `manual` s'écrivent aujourd'hui ;
     **une ligne `Other` se traite partout comme `manual`**, hors l'affichage et l'audit de sa source, rendue telle
     quelle : elle n'est pas un cours `afc` — `FX_RATE_NOT_AFC` de 37-3a refuse une ligne d'écriture `afc` à ce cours,
     `expectedRate` à `null` —, son `PUT …/rates` la passe en `manual`, son ↑ (`reset`) en `afc` à la valeur publiée (sans
     valeur publiée : supprimée, comme une ligne `manual`, AC-5), l'écran *Cours de change* l'affiche comme une ligne
     `manual`, sa source entre parenthèses (AC-6), et aucun acquittement ni reprise ne la vise (`chk_exchange_rates_ack_afc`)),
     si bien qu'une source future, p. ex. `ecb`, s'ajoute sans DDL — élargir un `CHECK` d'énumération demanderait un
     `DROP CONSTRAINT` puis un `ADD CONSTRAINT` (P3 ne les liste pas). Dès cette version, la release qui écrit la première
     une source neuve n'a donc **plus à relever `kesh_version_min_required`** ni à livrer une migration breaking pour elle
     (A28, A30) ; le binaire de cette version lit la ligne sans erreur, et la traite comme `manual` (ci-dessus) — la
     release qui l'écrit vérifie que ce traitement reste juste pour la source neuve. Un binaire antérieur à cette
     version ne connaît pas la table du tout, et ne démarre de toute façon pas sur une base migrée (P1) ; un
     `.keshbackup` d'une version future portant une source neuve se lit de même **s'il s'importe** — même inventaire de
     tables (`parse_and_verify`, `admin_backup/import.rs:175-180`, qui passe `TABLES_TO_TRUNCATE` à `parse_and_verify_avec_inventaire`) et version minimale du manifeste non
     supérieure à ce binaire (`routes/admin.rs:235`)), `acknowledged_rate DECIMAL(20,10) NULL`
     et `acknowledged_at DATETIME(3) NULL` (UTC) — l'**acquittement par société** d'une correction de l'AFC : la valeur
     publiée (par unité) que la société a vue et acquittée (AC-5) ; `chk_exchange_rates_ack_pair` (les deux `NULL` ou les
     deux non `NULL`) et `chk_exchange_rates_ack_afc` (`acknowledged_rate IS NULL OR source = 'afc'`) —, `version INT NOT
     NULL DEFAULT 1`, `updated_by BIGINT NULL` (FK `users`, `RESTRICT`, comme `api_keys` ; **`NULL` = ligne écrite par
     Kesh**, la copie automatique de l'AC-3 bis : aucun utilisateur ne l'a posée, et `audit_log.user_id` est `NOT NULL`,
     C-37-6), `created_at`, `updated_at` `DATETIME(3) NOT NULL` ; `UNIQUE uq_exchange_rates_key (company_id, currency,
     kind, rate_date)`. **Relation** avec `_kesh_fetched_exchange_rates` : logique, par (`source`, `currency`, `kind`,
     `rate_date`) pour une ligne `afc` — **sans clé étrangère** (table système, vidée à volonté, absente de la
     sauvegarde) : une ligne `afc` garde sa valeur et sa source même quand la valeur publiée manque (installation
     restaurée, source qui ne l'offre plus). Volume (A24, CR-5) : les devises des **comptes** de la société × les unités
     utiles d'un exercice (AC-3 bis), soit quelques dizaines de lignes par exercice multi-devises (3 devises × 14
     unités ≈ 40), non plus 73 × 14 ≈ 1000, plus les cours saisis ou repris. Inscrite à
     `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:46`), **avant `companies` et `users`** (enfant des deux, sans
     enfant propre ; place proposée : après `mcp_server_settings`, autre table de réglages de société), absente de
     `RESET_PRESERVED_TABLES` (:136) : un cours retenu est une donnée de la société, vidée par la remise à zéro
     (`reset_partition_is_guarded_by_the_schema` reste vert : la table est aussi enfant de `companies`). Sauvegardée dans
     le `.keshbackup` par l'export générique (`export_table`, colonnes lues de `INFORMATION_SCHEMA`) ; export CSV
     utilisateur : ajoutée à `TABLES_EXPORTEES` (`crates/kesh-api/src/exports/global.rs:116`, garde
     `export_couvre_toutes_les_tables` :673) avec son sérialiseur dans `csv_tables.rs` (cours à 10 décimales, **jamais**
     par `fmt_decimal`, comme 37-3a AC-8), ou à `TABLES_HORS_EXPORT` avec un motif écrit — choix au développement,
     exportée de préférence ;
   - **Fenêtre d'import (R1, CR-2 — remplace la fermeture de CR-1)** : `exchange_rates` reste une table
     **applicative**, sauvegardée et restaurée ; mais une sauvegarde **antérieure** qui ne la porte pas se réimporte
     (elle est forcément mono-devise : aucun cours retenu). La table s'inscrit à la **liste close des absences
     admises** créée par 37-1 (AC-6) : la boucle de couverture de `parse_and_verify_avec_inventaire`
     (`crates/kesh-api/src/admin_backup/import.rs:250-253`) admet son absence, la restauration la laisse vide
     (`restore_body`, `backup.rs:796-803`), et `creates_application_table` (`post_restore.rs:988`) l'ignore, si bien
     que **la fenêtre ne bouge pas**. Rien de ce que CR-1 avait écrit ici ne se fait : `POST_RESTORE_BACKFILLS` garde
     son entrée `20261106000001_supplier_invoice_internal_number` (pas de passage à `RETIRED_BACKFILLS` ni à
     `EXEMPT_MIGRATIONS`, compteurs de `post_restore.rs` inchangés) ; `admin_full_import_e2e.rs` n'est **pas** touché
     (`full_import_report_mirrors_the_production_registry`, `entree_numero_interne` et les tests d'import 0.17.4 et
     0.17.5 restent tels quels) ; la phase 5 de `scripts/recette-montee.sh` (import du `.keshbackup` de l'image
     précédente, 200 exigé, `:651`) **se joue** à la release ; le test de 37-1 AC-6 (sauvegarde 0.17.6 réimportée)
     reste un test d'acceptation, et cette story y ajoute l'absence de la table (aucun cours retenu après l'import ;
     ils se recopient des cours récupérés quand un compte reçoit une devise ou qu'un exercice multi-devises se crée, AC-3 bis). R1 vaut pour la table sous son nom
     neuf : l'entrée de la liste close est `Absence::Table("exchange_rates")`, jamais l'ancien nom.
     Test ici : `registry_entries_are_within_import_window` (`post_restore.rs:939`) vert **avec** la table créée — c'est
     lui qui rougirait si la table manquait à la liste ;
   - `_kesh_exchange_rate_fetch` singleton : `id TINYINT UNSIGNED PRIMARY KEY` (`CHECK (id = 1)`),
     `triggered_by ENUM('auto','manual') NOT NULL` (**pas
     `trigger`** : mot réservé de MariaDB ; le champ JSON de l'API s'appelle `trigger`, lu depuis cette colonne),
     `last_run_at DATETIME(3) NOT NULL` (UTC), `last_status VARCHAR(16) NOT NULL` (`CHECK IN ('ok','error','not_published')`),
     `last_error VARCHAR(64) NULL` — **un code
     classé**, jamais le texte d'une erreur `reqwest` (qui porte l'URL) : `timeout`, `connect`, `http_status:<n>`,
     `too_large`, `malformed`, `month_mismatch`, `date_mismatch`, `no_valid_rate`, `invalid_rates:<n>` —,
     `last_kind VARCHAR(5) … NOT NULL` (`month` | `day`, même `CHECK`) et `last_rate_date DATE NOT NULL` (le 1er du mois
     ou le jour demandé). Le tour
     d'une récupération écrit **une** ligne : `ok` (avec `last_error = invalid_rates:<n>` si des devises ont été
     écartées), `not_published` ou `error` ; un tour automatique qui traite plusieurs unités (AC-4) écrit le **pire** des
     statuts (`error` > `not_published` > `ok`), avec `last_kind` et `last_rate_date` de l'unité en cause (à égalité, la plus récente). **Ligne créée
     paresseusement** (aucune écriture de données dans la migration, P7 sans objet ; le dump `--no-data` de
     `regen-test-schema.sh` n'a rien à réinjecter), écrite par `INSERT … ON DUPLICATE KEY UPDATE` ;
   - les deux tables `_kesh_*` inscrites à `TABLES_SYSTEME` (`crates/kesh-db/src/backup.rs` :949, lue par
     `backup_inventory_matches_schema` et `convention_du_prefixe_des_tables_systeme`), absentes de l'inventaire
     `.keshbackup` ; `exchange_rates` à `TABLES_TO_TRUNCATE` (ci-dessus) ; `migrations.sha384`, ligne et
     compteurs recomptés de `docs/migrations-idempotence-audit.md` (P5, depuis la source, après chaque reprise de `main`), squash régénéré, P6, P7 (DDL seul :
     `every_data_backfill_migration_is_triaged` n'a rien à trier pour ce fichier, et la fenêtre ne bouge pas, R1) ;
   - les colonnes `fiscal_years.opening_rate_kind` et `closing_rate_kind` (R6) sont créées par la migration de **37-1**
     (AC-1, décision CR-2) : cette story ne les ajoute pas, elle écrit la route et l'écran qui les règlent (AC-5, AC-6) ;
   - test de montée sur le modèle d'`installation_settings_adds_only_its_system_table`
     (`crates/kesh-db/tests/migrations_upgrade_path.rs:1766`), monté **par version**
     (`common::migrations_before(VERSION, LABEL)`, `tests/common/mod.rs:75`, P6) : deux tables système neuves, vides,
     (`_kesh_fetched_exchange_rates`, `_kesh_exchange_rate_fetch`) et **exactement une** table applicative ajoutée,
     `exchange_rates`, vide (l'assertion
     `application_tables(&pool).await == before` du modèle devient « `before` plus cette table ») ; aucune ligne
     existante modifiée.
2. **AC-2 — Analyse XML** dans `kesh-core` (pure, sans I/O) : `parse_monthly_rates(xml) -> Result<MonthlyRates>`
   (mois, liste `{currency, published_unit, published_rate, rate_per_unit}`), `rate_per_unit = published_rate /
   published_unit` (`Decimal`, exact pour les unités puissances de dix relevées), **arrondi à 10 décimales**
   (`round_dp_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)` — **pas** `round_dp`, qui arrondit
   l'équidistant au pair ; même règle que C-37-4 —, la précision de la colonne : la valeur lue par `FX_RATE_NOT_AFC`
   est celle qui est stockée) ; code mis en majuscules et validé (trois lettres). Parseur `quick-xml = "0.42"`
   (déjà au `Cargo.lock` par `kesh-import` et `kesh-payment`, dépendance neuve de `kesh-core`), lecture **par noms
   locaux** (la racine porte un `xmlns` par défaut), éléments inconnus ignorés (`land_de`, `land_fr`, `land_it`).
   - **Unité** : entier positif en tête de `<waehrung>` (1, 100, 1000, 10000 relevés — aucune liste fermée), le code
     qui suit doit égaler l'attribut `code` (casse ignorée).
   - **Tolérance par devise** : une devise invalide (cours ≤ 0, unité ou code illisible, code et `<waehrung>`
     discordants, doublon — les deux occurrences) est **écartée**, **comme toute devise que la base refuserait** :
     unité > 1 000 000, cours publié ≥ 10⁹ (borne de prudence, plus stricte que les 13 chiffres entiers de
     `DECIMAL(19,6)`) ou à plus de 6 décimales (que la colonne arrondirait en silence, rendant `published_rate /
     published_unit` différent du `rate_per_unit` stocké), `rate_per_unit` arrondi à 0 ou > 10⁶ (borne du cours d'une
     ligne en devise, 37-3a AC-3 : un cours de la table que la ligne refuserait ne servirait à rien ; plus stricte
     que la borne 10¹⁰ de `DECIMAL(20,10)`) — sans quoi un `INSERT` refusé annulerait la transaction du mois entier (500) au
     lieu d'écarter la seule devise fautive ; elle est rendue dans `MonthlyRates.rejected` et journalisée
     par l'appelant (`tracing::warn!`, `last_error = invalid_rates:<n>`) ; les autres s'écrivent. Une seule devise
     fautive (p. ex. un cours publié `0.0000`) ne bloque donc jamais USD ni EUR.
   - **Refus du mois entier** seulement si : XML mal formé, `<monat>` absent ou illisible, ou aucune devise valide
     (`NoValidRate`). Racine vide → `NotPublished`.
   - **Cours journaliers** (D10) : `parse_daily_rates(xml) -> Result<DailyRates>` (date lue `<datum>` au format
     `JJ.MM.AAAA`, mêmes listes `{currency, published_unit, published_rate, rate_per_unit}` et `rejected`) ; racine
     `wechselkurse`, lecture par noms locaux (espace de noms `…/xmldaily`), éléments inconnus ignorés (`zeit`,
     `gueltigkeit`, `land_*` dont `land_en`). **Une seule** analyse des `<devise>` (unité, cours, tolérance par devise,
     bornes, arrondi à 10 décimales) partagée par les deux fonctions (DRY) : seules la racine et la date diffèrent.
     Refus du jour entier : XML mal formé, `<datum>` absent ou illisible, aucune devise valide ; racine vide →
     `NotPublished`. Le contrôle `<datum>` ≤ D et D − `<datum>` ≤ 7 jours (`date_mismatch`) est celui de l'appelant
     (`kesh-api`, comme `month_mismatch`).
   - **Bornes d'un cours seul** (propriétaire : cette story, qui précède 37-3a et 37-5) : fonction publique de
     `kesh-core` (nom au développement, p. ex. `exchange_rates::check_rate_bounds`) — décimal > 0, au plus 10 décimales
     **après `normalize()`** (zéros terminaux ignorés), ≤ 10⁶. Elle rend une **erreur de type propre**, neutre (p. ex.
     `RateBoundsError { NotPositive, Scale, TooLarge }`), **jamais** une variante de `CoreError` liée à un code : chaque
     appelant la traduit — le `PUT` d'un cours saisi (AC-5) et 37-5 en `FX_RATE_INVALID`, 37-3a en
     `FX_VALUE_OUT_OF_RANGE { line }` (37-3a AC-3, 37-3a-CR1-P1-IF1), le parseur en devise écartée (filtre
     `rate_per_unit`) ; 37-3a l'appelle depuis son contrôle par ligne (AC-3) et 37-5 pour les cours de
     sa requête (AC-4) — **une seule** fonction.
   - **Filtrage strict (R5, CR-2)** — tout ce qui vient de la source est une donnée hostile jusqu'à preuve du
     contraire ; l'analyse (commune aux deux fonctions) valide, **avant** toute écriture :
     - **taille** : corps borné à 1 MiB (AC-3, avant l'analyse) ; au plus 500 éléments `<devise>` (73 et 74 relevés) ;
       texte d'un élément lu (`monat`, `datum`, `waehrung`, `kurs`) et attribut `code` bornés à 64 octets — au-delà,
       la devise est écartée (élément de `<devise>`) ou le document refusé (`monat`, `datum`) ;
     - **structure** : racine `monatsmittelkurs` (mensuel) ou `wechselkurse` (journalier) par nom local, sinon refus ;
       profondeur d'imbrication ≤ 3 (racine → `devise` → enfant), comptée par l'analyse itérative (aucune récursion),
       sinon refus ; un seul `<monat>` / `<datum>` ; UTF-8 valide ;
     - **DOCTYPE et entités** : un événement `DocType` (déclaration de type de document, sous-ensemble interne compris)
       → **refus du document** ; une référence d'entité (`Event::GeneralRef`, émis à part par quick-xml 0.42, cf.
       `kesh-import/src/camt053/mod.rs:244`, `:375`) n'est admise que si c'est une référence de caractère ou l'une des
       cinq entités prédéfinies de XML (`quick_xml::escape::resolve_xml_entity`) ; toute autre → refus du document
       (quick-xml ne développe aucune entité déclarée : pas d'expansion, pas d'accès externe ; le refus rend cela
       vérifiable). Instructions de traitement (`PI`) et `CDATA` dans un élément lu → refus ;
     - **nombres** : `kurs` suit la grammaire `^[0-9]{1,13}(\.[0-9]{1,6})?$` (après retrait des blancs ASCII de bord)
       **avant** toute conversion en `Decimal` — ni signe, ni exposant, ni séparateur, ni chiffres non ASCII ; l'unité de
       `<waehrung>` suit `^[1-9][0-9]{0,6} [A-Za-z]{3}$` ; hors grammaire → devise écartée ;
     - **codes** : trois lettres ASCII, présents dans la liste ISO 4217 embarquée de 37-1 (`kesh-core::currency`, qui
       exclut `XXX`, `XTS`, métaux et `XDR`), et différents de `CHF` ; sinon devise écartée (`rejected`) ;
     - **mois et dates** : `<monat>` = `AAAA-MM`, mois 01–12, année 2000–2099, et égal au mois demandé
       (`month_mismatch`) ; `<datum>` = `JJ.MM.AAAA`, date calendaire valide (`31.02.2024` refusé), mêmes bornes
       d'année **abaissées à 1999** (`d=20000101` rend `<datum>31.12.1999</datum>`, relevé le 2026-11-14 : sans
       cela, la borne 2000-01-01 du `POST /fetch` rendrait `malformed`), contrôle `date_mismatch` de l'appelant ;
     - un refus du document se classe `malformed` (`last_error`), jamais en texte libre.
     **Tests de réponses hostiles** (fixtures synthétiques, `kesh-core` et client contre le serveur local) : DOCTYPE
     avec entités imbriquées (« billion laughs ») et entité externe `SYSTEM "file:///etc/passwd"` → refus, sans
     expansion et en temps borné ; entité inconnue `&x;` dans `<kurs>` → refus ; nombres énormes (`kurs` de 10 000
     chiffres, `1e400`, `-0.5`, `0x10`, `1,5`, `NaN`, chiffres pleine chasse) → devise écartée, les autres écrites ;
     codes non ISO (`XXX`, `ZZZ`, `us`, `USDX`, `ÜSD`, `CHF`) → écartés ; XML imbriqué sur 10 000 niveaux → refus sans
     débordement de pile ; 501 `<devise>` → refus ; mois incohérent (`2024-13`, mois autre que demandé, deux `<monat>`)
     → refus, rien d'écrit ; `<datum>31.02.2024</datum>` → refus ; corps non UTF-8 → refus.
   - Fixtures versionnées **synthétiques** (forme du XML réel, `xmlns` compris, valeurs inventées — aucun cours officiel
     n'est versionné : conditions d'utilisation SIX), une mensuelle et une journalière, avec unités 1, 100, 1000 et
     10000, un cours à 0.0009 et, pour la journalière, un cours à cinq décimales ; tests du cas vide, des refus du mois
     ou du jour et des écarts par devise.
3. **AC-3 — Source des cours et client HTTP** (`kesh-api`, module `exchange_rates`).
   - **Trait `RateSource`** (`fn fetch_month(&self, year, month) -> Pin<Box<dyn Future<Output = Result<String,
     SourceError>> + Send + '_>>` **et** `fn fetch_day(&self, date: NaiveDate) -> …` de même forme (`xmldaily?d=AAAAMMJJ`,
     D10) : **objet-safe sans `async_trait`**, comme `Mailer` (type `MailFuture`, `mail/mod.rs:33`, trait `:69`) — un
     `async fn` dans un trait n'est pas utilisable en `dyn`), porté par `AppState` (`rate_source: Arc<dyn
     RateSource>`, précédent `mailer: Arc<dyn Mailer>` / `test_mock_mailer`, `lib.rs:84-98`, et `BackupEnv` de la
     sauvegarde automatique ; `AppState::new_for_tests` reçoit une source factice par défaut).
   - **Champs neufs d'`AppState`** : `rate_source`, `rate_fetch_lock: Arc<tokio::sync::Mutex<()>>`, `rate_fetch_memo`
     (la mémoire des 15 minutes de l'AC-5, `Arc<std::sync::Mutex<HashMap<(RateKind, NaiveDate), …>>>`, clé = genre et
     `rate_date` : le 1er du mois ou le jour), `rate_login_trigger` (`Arc<std::sync::Mutex<Option<Instant>>>` : le dernier
     déclenchement à la connexion, AC-4 ; vidé par `clear_transient_test_state`) et
     `test_rate_source: Option<MockRateSource>` (poignée du mode test, comme `test_mock_mailer`). **Sites de
     construction littérale à compléter** (inventaire au `grep -rn "AppState {"`, à relancer) : `main.rs:439`,
     `middleware/auth.rs:285`, `AppState::new_for_tests` (`lib.rs:119`), et les tests `password_recovery_e2e.rs:126`,
     `invoice_frozen_pdf_e2e.rs:101`, `invoice_send_email_e2e.rs:104`, `:1036`, `:1150`, `setup_admin_e2e.rs:82` ;
     l'échec de compilation les inventorie.
   - **Production** : `HttpRateSource`, URL de base **constante** `https://www.backend-rates.bazg.admin.ch`, **aucune
     variable d'environnement** ne la change. Les tests Rust construisent `HttpRateSource::with_base_url(…)` contre un
     serveur axum local en HTTP. ⚠️ **Visibilité** : `with_base_url`, `un_tour` (AC-4) et `MockRateSource` sont
     `#[doc(hidden)] pub` (précédent `AppState::new_for_tests`, `lib.rs:119`) — un `pub(crate)` ou un
     `#[cfg(test)]` est invisible des tests d'intégration de `crates/kesh-api/tests/` (précédent de l'Epic 31,
     HIGH en validation) ; ou bien ces tests vivent dans le `mod tests` du module, à déclarer alors ici.
   - **`KESH_TEST_MODE`** : `main.rs` substitue une **source factice** (XML synthétique en mémoire, aucun appel
     sortant ; la boucle n'est pas lancée en mode test, AC-4), pilotée, par la poignée `test_rate_source`, par un endpoint `POST /api/v1/_test/exchange-rates/source` (réponse
     servie : XML, vide, erreur, délai, **par genre** `month` | `day` ; ni la boucle ni le déclenchement à la connexion
     ne tournent en mode test (AC-4) ; **XML synthétique par défaut** tant que rien
     n'est piloté, **généré pour le mois ou le jour demandé** (`<monat>` = `(j, m)` de l'appel, `<datum>` = la date
     demandée : un XML figé rendrait `month_mismatch` ou `date_mismatch` pour tout autre ; un XML piloté est servi tel
     quel, ce qui permet de servir un `<datum>` antérieur pour un dimanche) ; cours journaliers par défaut **distincts**
     des mensuels (pour que les tests distinguent le repli de 37-5) ; valeurs du défaut fixées et écrites au module, que les specs
     E2E — 37-3b comprise — lisent pour leurs assertions) monté avec les autres routes `_test` ; il entre au registre (`TEST_ENDPOINT_ROUTES` : +1,
     détail des totaux à l'AC-5). La recette E2E de `CLAUDE.md` et de `docs/testing.md` reste inchangée.
   - `reqwest` en dépendance de production, `default-features = false`, `rustls-tls` ; `.use_rustls_tls()` explicite
     (la dev-dependency, `Cargo.toml:109`, garde les fonctionnalités par défaut — dont `default-tls`, native-tls — et
     ses `json`, `multipart`, `cookies` explicites, qui s'unifient dans les builds de test ; `cookies` n'active aucun
     magasin sans `.cookie_store(true)`) ; `redirect(Policy::none())` ; `https_only(true)` pour l'URL de production ; `connect_timeout` 5 s et délai
     total 20 s ; pas de magasin de cookies (la source pose `rates_service_session`) ; réponse limitée à 1 MiB
     (`Content-Length` contrôlé puis lecture par `chunk()` bornée) ; `User-Agent: Kesh/<version>` sans donnée
     personnelle. `Cargo.lock` change (l'entrée `reqwest` gagne les dépendances `rustls` ; crates déjà au lock, la liste exacte se
     constate au build et non ici) et se commite ; `rustls-tls` embarque `webpki-roots`, l'image a de toute façon `ca-certificates`
     (`Dockerfile:20`) ; le build Docker (`cargo build --release -p kesh-api`) n'a rien d'autre à changer.
   - **Une seule récupération à la fois** (`rate_fetch_lock`, `tokio::sync::Mutex` dans `AppState`, comme
     `backup_lock` ; **la boucle l'attend** — elle ne saute pas son tour —, un `POST /fetch` l'attend aussi). Le verrou
     se prend **par mois ou par jour récupéré** (le genre et la date forment l'unité de récupération, comme la clé de la mémoire), jamais pour un tour entier de la boucle (qui en traite plusieurs, AC-4) : l'attente
     d'un `POST /fetch` est alors bornée par **deux** requêtes sortantes au plus devant la sienne (le mois en cours de la
     boucle et un autre `POST /fetch` — le verrou tokio est équitable, premier arrivé premier servi), soit 3 × 20 s dans
     le pire cas, plus les écritures — borne valable pour **un** autre demandeur : avec plus de demandeurs simultanés
     (plusieurs onglets), elle s'allonge d'autant ; pour la tenir, un `POST /fetch` qui a **attendu** le verrou et
     trouve, une fois le verrou pris, le mois (ou le jour) écrit par une récupération **commencée après son arrivée** (`fetched_at`,
     pris par la récupération **avant** son appel sortant, postérieur à l'instant d'arrivée du `POST`) rend ce
     résultat (200, `count` des devises présentes, `rejected` et `corrected` à 0 — une correction trouvée par
     la récupération qui précède, la revérification automatique de l'AC-4 comprise, se voit en rouge à la relecture) sans rappeler la source, sans ligne d'audit et sans
     écriture de `_kesh_exchange_rate_fetch` (il n'a rien récupéré) ; il fait seulement la copie de l'AC-3 bis pour
     la société de l'utilisateur, si elle manque. Sous le
     verrou, le `POST /fetch` relit la mémoire de l'AC-5 avant d'appeler la source (un mois ou un jour **présent** se redemande,
     c'est la demande : la mémoire ne retient que `not_published` et les échecs) ; **un succès retire l'entrée** du
     mois (ou du jour) de la mémoire (sans quoi un échec mémorisé rendrait 503 pendant 15 minutes sur un mois que la boucle vient
     d'écrire). La boucle juge « manque » (AC-4) **sous le verrou**, non avant : sinon elle redemanderait un mois que
     le `POST /fetch` devant elle vient d'écrire. Le travail du `POST /fetch` (verrou, appel, transaction, état,
     audit) tourne dans un `tokio::spawn` attendu par le handler, comme « Sauvegarder maintenant »
     (`run_backup_now`, `routes/backup_schedule.rs:159`) : l'abandon du client (délai de 90 s, onglet fermé) n'annule
     pas une récupération entre le `COMMIT` des cours et l'audit. Le client du frontend appelle `POST /fetch` avec `timeoutMs: 90_000` (`CallOptions`, `api-client.ts:61` ; le
     défaut de 30 s, `:42`, et même 60 s ne couvrent pas ce pire cas : ils rendraient `TIMEOUT`, lu comme issue
     inconnue) ; écriture en **une transaction** par mois ou par jour récupéré : les valeurs publiées dans
     `_kesh_fetched_exchange_rates` (`source = 'afc'`), puis la **copie** dans `exchange_rates` (AC-3 bis) ; un échec
     n'écrit aucun cours. La transaction passe sous `kesh_db::retry::retry_on_deadlock` (la copie prend `companies` en
     partage par sa clé étrangère et lit `accounts` en partage par le filtre A24 : cycle possible avec une route
     d'exercice qui copie elle aussi dans `exchange_rates` (création d'un exercice multi-devises, `PUT /rate-kinds`), ou avec une route de compte
     qui tient le compte `FOR UPDATE` puis copie, AC-3 bis). **Règle d'écriture de la valeur publiée (U1, CR-5 —
     remplace la règle S6 de CR-3, qui gardait à part une valeur republiée)**, par devise valide, les lignes déjà
     présentes du mois ou du jour lues `FOR UPDATE` dans la transaction : (a) absente → insérée (`fetched_at` =
     `first_fetched_at` = `value_changed_at` = l'instant pris avant l'appel sortant) ; (b) présente, **même valeur
     publiée** (`published_unit` et `published_rate` égaux, comparés en valeur — jamais le `rate_per_unit` dérivé) →
     seul `fetched_at` avance (pour un jour, `source_date` reste celle de la valeur actuelle, même si le `<datum>` rendu
     diffère : seule la valeur compte) ; (c) présente, **valeur différente** (correction de l'AFC) → la ligne prend la
     **nouvelle** valeur (`rate_per_unit`, `published_unit`, `published_rate`, et pour un jour `source_date` = le
     `<datum>` rendu), `value_changed_at` = `fetched_at` = maintenant, `tracing::warn!` (devise, genre, date, ancienne
     et nouvelle valeur). `first_fetched_at` n'avance **jamais**. ⛔ **Une récupération ne modifie jamais une ligne
     existante de `exchange_rates`** — ni une ligne `manual` (D9 : un cours saisi n'est jamais remplacé), ni une ligne
     `afc` (U1 : une correction de l'AFC ne touche jamais le cours retenu par la société ; elle s'annonce, AC-5 et
     AC-6, et se reprend par ↑) ; elle n'y fait qu'**insérer** les lignes qui manquent (copie, AC-3 bis). Les lignes
     d'écriture déjà passées ne changent **jamais** (37-3a : elles portent leur cours). Tests : (a) ; (b), dont un jour
     rendu sous un autre `<datum>` à valeur égale (`source_date` inchangée) ; (c) — valeur remplacée, `value_changed_at`
     avancé, `first_fetched_at` inchangé —, puis retour de l'AFC à la valeur d'origine (nouvelle correction, (c) encore) ;
     un cours `manual` pour (USD, mois) reste, avec sa `version`, après un `POST /fetch` du même mois qui rend une autre
     valeur ; une ligne `afc` d'une société reste à sa valeur après une correction (c), et la correction s'annonce
     (AC-5, `notices`).
3. **AC-3 bis — Copie des cours récupérés dans `exchange_rates` (U1, CR-5).** Une société ne lit **que**
   `exchange_rates` (AC-5) ; la copie y apporte les cours publiés qui lui sont utiles.
   - **Unités utiles d'un exercice** : ses mois, du mois de `start_date` au mois de `end_date`, et ses unités
     d'Ouverture et de Clôture selon ses genres (`openingDate`, `closingDate` de la fonction pure de l'AC-5) — le même
     ensemble que `unites_utiles` de l'AC-4, **sans** la borne « strictement antérieur à aujourd'hui » ni celle du mois
     suivant (une valeur déjà publiée se copie quelle que soit sa date).
   - **Une seule fonction de dépôt** (nom au développement, p. ex. `exchange_rates::copy_fetched_in_tx(conn,
     company_id, units, currencies)`, `currencies` = `None` pour toutes les devises des comptes de la société, ou la
     seule devise qu'un compte reçoit — le paramètre que 37-1 AC-4 attend de cette story) : `INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source,
     updated_by, …) SELECT ?, f.currency, f.kind, f.rate_date, f.rate_per_unit, 'afc', NULL, … FROM
     _kesh_fetched_exchange_rates f WHERE f.source = 'afc' AND (f.kind, f.rate_date) IN (…) AND NOT EXISTS (SELECT 1
     FROM exchange_rates x WHERE x.company_id = ? AND x.currency = f.currency AND x.kind = f.kind AND x.rate_date =
     f.rate_date)` — les devises des **comptes de la société** seulement (A24, CR-5 : `AND f.currency IN (SELECT a.currency FROM
     accounts a WHERE a.company_id = ? AND a.currency IS NOT NULL)`, comptes archivés compris), non plus les 73 devises
     publiées ; une devise donnée plus tard à un compte déclenche sa propre copie (*Quand*, ci-dessous) ; **insertion
     seulement** : une ligne présente, `afc` ou `manual`, n'est jamais modifiée ; une course entre
     deux copies est rattrapée sur l'unicité (`uq_exchange_rates_key`, doublon ignoré, jamais une 500). `updated_by =
     NULL`, **aucune ligne d'audit** (copie automatique, C-37-6) ; le nombre de lignes copiées va au détail d'audit de
     l'action qui la déclenche, quand elle en a une (`copied`).
   - **Quand** :
     - à chaque récupération réussie d'une unité (AC-3 : nouvelle, confirmée ou corrigée), dans sa transaction, pour
       **chaque société dont un exercice multi-devises ouvert** a cette unité parmi ses unités utiles (exercices lus
       dans la transaction, sans verrou ; ordre des sociétés croissant) — et, pour un `POST /fetch`, **toujours** pour la
       société de l'utilisateur (exercice clos compris, A7 : ses cours se récupèrent à l'écran ; c'est aussi le filet
       d'une case vide : `lookup` 404 → `POST /fetch` → copie → relecture, 37-3b, 37-5) ;
     - à la **création** d'un exercice multi-devises (37-1 AC-3 bis, `fiscal_years::create` ; les deux autres chemins
       ne créent pas d'exercice multi-devises : l'assistant crée la société mono-devise, `create_for_seed` sert la
       démonstration) ;
     - au **changement de genre** d'ouverture ou de clôture (`PUT /fiscal-years/{id}/rate-kinds`, AC-5), pour les
       unités neuves de l'exercice et, si U4 s'applique, de l'exercice suivant ;
     - quand un **compte reçoit une devise** (A24 ; `accounts::create` avec une devise, `accounts::update` qui passe un
       compte de CHF ou d'une autre devise à celle-ci, 37-1 AC-4) : pour chaque exercice **multi-devises ouvert** de la
       société, copie des cours déjà récupérés de cette devise seule, sur les unités utiles de l'exercice. Une devise
       déjà portée par un autre compte n'a rien de neuf à copier (l'insertion seule le rend sans effet).
     **Pas à l'activation** du mode multi-devises (37-1 AC-3, CR5-P1-2) : la copie y serait vide par construction —
     tant que la société est mono-devise, aucun compte, archivé compris, ne porte de devise ;
     `companies::update_currency_settings` n'appelle pas la copie.
     Ces trois derniers appels se font **dans la transaction** de la route de 37-1 ou de cette story, après ses
     verrous (l'`INSERT` prend `companies` en partage par sa clé étrangère : `accounts::create`, `accounts::update` et
     `fiscal_years::create` la tiennent déjà — en partage, ou en exclusif pour un geste de compte qui porte des comptes
     propres —, 37-1 AC-4, AC-4 bis et AC-3 bis ; `PUT /rate-kinds` la prend là, après
     l'exercice `FOR UPDATE`, AC-5 *Rejeu*) ;
     le `SELECT` de l'`INSERT … SELECT` pose sous REPEATABLE READ des verrous **partagés** sur les lignes lues — celles
     de `_kesh_fetched_exchange_rates` et, par le filtre A24, celles d'`accounts` de la société : ce n'est **pas** une
     lecture sans verrou. Elle peut croiser une récupération, qui tient ses lignes `_kesh_fetched_exchange_rates`
     `FOR UPDATE` puis lit `accounts` et prend `companies` en partage : cycle avec une route d'exercice qui copie
     elle aussi (création d'un exercice multi-devises, `PUT /rate-kinds` : mêmes clés uniques de `exchange_rates`),
     ou avec une route de compte qui tient le compte `FOR UPDATE`. Les deux côtés sont
     rejoués : la récupération et les routes d'exercice sous `kesh_db::retry::retry_on_deadlock`, `accounts::create`
     et `accounts::update` sous `crate::retry::retry_app_on_deadlock` (37-1 AC-4 ; AC-5). Les
     routes de 37-1 sont développées avant cette table : **cette story ajoute l'appel** dans
     `fiscal_years::create`, `accounts::create` et `accounts::update`, et le
     détail `copied` à leur audit (renvoi écrit en 37-1 AC-3 bis et AC-4). *(CR-7 : plus de passage d'un exercice
     en multi-devises ni de désactivation — A27 retiré.)*
   - Rien d'autre n'écrit `exchange_rates` que cette copie, le `PUT` d'un cours saisi, le ↑ (AC-5), l'acquittement
     (colonnes `acknowledged_*` seulement), la restauration et la remise à zéro (AC-7).
   - **Tests** : récupération → lignes `afc` copiées pour la société A (exercice multi-devises ouvert qui couvre
     l'unité), non pour B (société mono-devise), ni pour C (exercice clos), sauf `POST /fetch` de C, ni pour D (société
     multi-devises dont un exercice clos avant l'activation a été rouvert : exercice mono-devise, CR-7 ; ni récupération
     ni copie pour lui, `GET /fiscal-years/{id}/exchange-rates` et `PUT …/rate-kinds` → 409 `MULTI_CURRENCY_DISABLED`
     `details.fiscalYearId`, absent du sélecteur de l'écran) ; ligne `manual` présente →
     inchangée ; devise publiée que **aucun compte** de la société ne porte (p. ex. SEK) → aucune ligne copiée ;
     un compte qui reçoit la devise SEK (création, puis modification d'un compte CHF) → les cours SEK déjà récupérés
     des unités utiles sont copiés, ceux d'une autre devise non ; devise changée d'USD à EUR sur un compte sans ligne →
     lignes EUR copiées ; `PUT` de compte no-op → aucune copie ; aucun exercice multi-devises ouvert → rien (tests
     délégués par 37-1 AC-4) ; compte archivé en JPY → JPY encore copié ;
     activation → aucune ligne copiée, `copied` absent de son audit ;
     correction (AC-3 (c)) → ligne `afc` inchangée ; ligne `source = 'ecb'` posée par SQL (A30) → lue par `find_company_rate_in_tx` (`RateOrigin::Other("ecb")`), `lookup` et grille rendent `source: "ecb"`, aucune erreur interne (ni 500) ; son `PUT …/rates` → `manual`, son `reset` → `afc`, son `reset` sans valeur publiée → supprimée ; devise sans compte à ligne `ecb` dans l'exercice → ligne de la grille sans `all=true` ; création d'un exercice multi-devises,
     changement de genre → cours déjà récupérés copiés (un mois, l'Ouverture `month` du mois
     précédant l'exercice, un jour de Clôture) ; deux copies concurrentes → une ligne, aucune erreur ; `updated_by`
     `NULL`, aucune ligne d'audit propre.
4. **AC-4 — Récupération automatique (R2, R4, S1 ; remplace la boucle permanente de 6 h).**
   - **Unités utiles (R2)** : Kesh ne récupère que pour les **exercices multi-devises ouverts**
     (`fiscal_years.multi_currency = TRUE`, `status = 'Open'`, 37-1 AC-3 bis), toutes sociétés de l'installation : les
     **mois** de chaque tel exercice, de son premier mois au plus petit de son dernier mois et du **mois suivant** le mois courant (R4 amendé, S1 : le mois suivant peut être publié avant le 25 ; non publié, il se retente chaque jour) ; et les **jours** d'ouverture et de clôture que
     son genre demande (R6, colonnes de 37-1) — la veille de `start_date` si `opening_rate_kind = 'day'`, `end_date` si
     `closing_rate_kind = 'day'` —, seulement s'ils sont **strictement antérieurs** à aujourd'hui (AC-5) ; et, si
     `opening_rate_kind = 'month'`, le **mois de la veille de `start_date`** (dernier mois de l'exercice précédent,
     T4, CR-4 : un mois **avant** celui de `start_date` quand l'exercice commence le 1er d'un mois, il s'ajoute alors
     aux mois de l'exercice ; quand il commence en cours de mois, p. ex. le 15.03, la veille tombe dans le même mois
     que `start_date`, qui est déjà le premier mois de l'exercice : aucune unité de plus, un ensemble sans doublon).
     Les unités **antérieures aux bornes de l'AC-5** (mois < 2000-01, jour < 2000-01-01 : un exercice commencé le
     01.01.2000 a pour veille le 31.12.1999) ne sont **pas** utiles — le `POST /fetch` les refuserait et la tâche les
     redemanderait chaque jour sans fin. Une unité
     **manque** s'il n'y a aucune ligne `_kesh_fetched_exchange_rates` (`source = 'afc'`) de ce genre et de cette
     date — jamais d'après `exchange_rates` (une installation restaurée a ses cours retenus, mais pas encore les valeurs
     publiées : la tâche les relit, et une correction survenue entre-temps s'annonce, AC-5). Aucun exercice
     multi-devises ouvert → aucun trafic sortant (mono-devise compris). Une fonction pure (`unites_utiles(exercices,
     aujourdhui)`, nom au développement) les calcule, testée pour des exercices civils (**1er janvier**, T2 : veille
     = 31.12 de l'année précédente, mois d'ouverture `month` = décembre de l'année précédente), au **1er juillet** et au
     **1er avril** (bornes, ordre, veille du 1er = 30.06 / 31.03, mois d'ouverture `month` = juin / mars, jour de
     clôture), et — S4, CR-3 : Kesh n'impose aucune
     durée d'exercice — pour un exercice de **six mois** (01.07.2024–31.12.2024) et de **dix-huit mois**
     (01.07.2023–31.12.2024), ainsi que pour un exercice qui ne commence pas le 1er (p. ex. 15.03.2024–14.03.2025 :
     mois de mars 2024 à mars 2025, treize mois ; veille d'ouverture 14.03.2024 ; en genre `month`, mois d'ouverture
     = mars 2024, **déjà** le premier mois : treize unités distinctes de mois, de mars 2024 à mars 2025, plus le mois
     suivant éventuel, pas de doublon) ; exercice commencé le 01.01.2000 : ni la veille ni son mois ne sont utiles.
   - **Calendrier (S1, CR-3 — remplace R4 amendé)** : une tâche `exchange_rates::spawn(state)` (modèle
     `auto_backup::spawn`, `auto_backup.rs:788` ; chaque tour dans son propre `tokio::spawn`, une panique journalisée
     n'arrête rien) ne tourne **pas** en boucle de 6 h permanente. Elle se réveille : (a) **au démarrage**, après un
     court délai fixe (60 s, constante) pour ne pas retarder le démarrage ; (b) **une fois par jour** (00:00,
     Europe/Zurich, même aide de fuseau) tant que le **mois courant ou le mois suivant** manque pour un exercice
     multi-devises qui le couvre — c'est-à-dire jusqu'à l'arrivée du **premier cours du mois suivant** ; (c) une fois le
     mois suivant `T` obtenu, elle **se met en pause** : réveil au **plus tôt** de « date d'obtention + 24 jours » et
     du **25 du mois `T`** (« au plus tard jusqu'au 25 »), mais **pas avant le 1er du mois `T`** (00:00) — avant ce
     1er, le mois d'après (`T + 1`) n'est pas encore le « mois suivant » et ne se demande pas (borne du `POST /fetch`,
     AC-5) ; au réveil, la vérification quotidienne (b) reprend pour `T + 1`. La **date d'obtention** est la date
     Europe/Zurich du plus petit `first_fetched_at` des lignes `month` de `T` (AC-1 : jamais avancée par une
     récupération ultérieure). Exemples : `T` = décembre obtenu le 14.11 → réveil le 08.12 ; obtenu le 25.11 → le
     19.12 ; obtenu le 02.11 → le 01.12 (borne du 1er). ⚠️ `T` n'est « mois suivant » qu'avant le 1er de `T` : il s'obtient
     donc au plus tard le dernier jour du mois qui précède, et « + 24 jours » tombe au plus tard le 24 de `T` — la
     borne du 25 ne mord jamais en pratique (un `T` obtenu le 1er de `T` ou après est le mois **courant**, et le mois
     suivant manque : régime (b)) ; elle est gardée parce que c'est la règle de Beat (S1) et testée comme une
     propriété (réveil ≤ 25 de `T`). Si le mois suivant n'est couvert par
     aucun exercice multi-devises ouvert (et le mois courant présent ou non couvert), la tâche dort jusqu'au 1er du
     mois suivant, 00:00. Un tour échoué ou `not_published` pendant (b) se retente au réveil du lendemain (le « jour
     ouvrable suivant » de la source est ainsi couvert). Un **jour** d'ouverture ou de clôture manquant ne réveille pas
     la tâche : il se récupère au premier tour qui suit le moment où il devient strictement antérieur à aujourd'hui —
     réveil quotidien (b), réveil de fin de pause (c), prochain démarrage ou prochaine connexion (qui lance un tour pour
     toute unité manquante autre que le seul mois suivant) ; le jour de clôture du 31.12 attend donc au plus la fin de
     la pause de janvier (au plus tard le 25.01) s'il n'y a ni connexion ni démarrage entre-temps — limite écrite au
     manuel ; test de `prochain_reveil` : un jour seul manquant ne change pas le réveil. Chaque tour traite **toutes**
     les unités utiles qui manquent (pas seulement le mois suivant : une restauration ou un exercice neuf peuvent en
     laisser plusieurs), **au plus 36 par tour** (constante, testée ; le reste au tour suivant), dans l'ordre
     chronologique. **Revérification du mois courant (A10, approuvé par Beat, CR-5)** :
     chaque tour automatique — tâche (démarrage, réveil quotidien, fin de pause) **et** déclenchement à la connexion —
     **revérifie aussi le mois courant** (Europe/Zurich), **un appel `xmlavgmonth` de plus**, pour découvrir seul une
     correction de l'AFC (U1 : la valeur différente rendue **met à jour `_kesh_fetched_exchange_rates`**, règle d'écriture
     de l'AC-3, cas (c), `tracing::warn!`, sans audit puisque le tour est automatique ; l'écart avec les lignes `afc` des
     sociétés est **détecté** et s'annonce par le bandeau de connexion de chacune, AC-5 et AC-6 ; **`exchange_rates`
     n'est pas modifiée** — seule la copie de l'AC-3 bis y insère ce qui manque). Précisions :
     (i) la revérification **s'ajoute** aux unités qui manquent, **hors du plafond de 36** ; (ii) elle ne vaut que si le
     mois courant est **présent** et couvert par un exercice multi-devises ouvert (R2) — s'il manque, c'est déjà une
     unité manquante, demandée une fois ; (iii) elle **ne déclenche jamais** un tour ni un réveil : un tour ne se lance
     que si une unité manque (le démarrage ou la connexion qui ne trouvent rien à récupérer n'appellent pas la source,
     pas même pour revérifier) et **`prochain_reveil` reste inchangé** — pendant la pause de (c), aucun tour, donc aucune
     revérification : une correction de l'AFC publiée alors se découvre au tour qui suit (fin de pause, ou démarrage
     ou connexion qui trouvent une unité manquante — pour la connexion, autre que le seul mois suivant) ou par « Revérifier les cours publiés » (AC-6) ; (iv) elle passe par
     `rate_fetch_lock` et la mémoire de 15 minutes comme toute récupération (un succès retire l'entrée ; un échec ou un
     `not_published` se mémorise 15 minutes et ne se redemande pas avant) ; borne du trafic : au plus un appel de
     revérification par tour, et au plus un tour de connexion par quart d'heure ; (v) son résultat entre dans le **pire
     statut** du tour (AC-1) : un échec de la revérification écrit `error` même si l'unité manquante a réussi. Les
     **autres** mois et jours déjà présents ne sont jamais redemandés par la tâche : leur correction se découvre par
     un `POST /fetch` explicite (AC-5, AC-6). Le calcul du prochain réveil est une
     fonction pure (`prochain_reveil(now, etat)`, `etat` = mois utile manquant ou non, et la date d'obtention du mois
     suivant), testée aux bords : mois utile manquant → lendemain 00:00 ; mois suivant obtenu le 14.11, le 25.11, le
     02.11 → 08.12, 19.12, 01.12 ; obtenu le 03.12 pour décembre (restauration en décembre) → mois courant présent,
     janvier manquant → lendemain ; propriété « réveil ≤ 25 de `T` » sur toutes les dates d'obtention d'une année ;
     mois suivant non couvert → 1er du mois suivant ; fin de mois, 31.12 → 01.01, changement d'heure (fin mars, fin
     octobre) ; **revérification du mois courant (A10)**, par `un_tour` contre la source locale : mois suivant manquant
     et mois courant présent → deux appels (le mois suivant, puis le mois courant) ; mois courant rendu avec une autre
     valeur → `_kesh_fetched_exchange_rates` prend la nouvelle valeur (cas (c) de l'AC-3), la ligne `afc` de la
     société reste à l'ancienne et apparaît dans `GET /exchange-rates/notices`, aucune ligne d'audit ; rien
     ne manque → **aucun appel**, revérification comprise ; mois courant non couvert par un exercice multi-devises
     ouvert, ou manquant → pas d'appel de revérification en plus ; revérification hors plafond (36 unités manquantes
     + 1 appel) ; échec de la revérification avec l'unité manquante réussie → `last_status = error` ;
     `prochain_reveil` identique avec ou sans revérification ; connexion avec une unité manquante autre que le mois
     suivant → tour avec revérification, avec seul le mois suivant manquant → aucun tour.
   - **À la connexion (R4)** : après un `POST /api/v1/auth/login` réussi (`routes/auth.rs:235`), une **tâche
     détachée** (`tokio::spawn`, jamais attendue : la connexion ne dépend jamais de la source) juge elle-même si une
     unité utile manque pour un exercice de la société de l'utilisateur — **le mois suivant excepté**, qui ne suffit pas à lancer
     un tour (seule la tâche quotidienne le fait redemander seul ; arbitrage de l'orchestrateur sur CR2-P4-3 : « une
     vérification par jour ») — et lance alors un
     tour — aucune lecture n'est
     ajoutée au chemin de la connexion —, au plus une fois toutes les 15 minutes pour
     l'installation (horodatage en mémoire d'`AppState`, comme `rate_fetch_memo`) ; le renouvellement de session
     (`refresh`) ne déclenche rien. Test : la réponse de connexion ne dépend ni du succès ni du délai de la source ; connexion avec seul le mois suivant
     manquant → aucun tour (aucune ligne `_kesh_exchange_rate_fetch`, aucun appel sortant).
   - Lancement de `main.rs` **seulement**, avec un commentaire de renvoi à la garde comme `main.rs:463-466`, et **ni la
     tâche ni le déclenchement à la connexion en `KESH_TEST_MODE`** (`if !config.test_mode`) : un tour tomberait au
     milieu de la suite E2E avec la réponse que la spec en cours aurait pilotée — les tours se testent en Rust
     (`un_tour`, AC-3) ; garde lexicale `exchange_rates::tests::lancement_par_main_seulement` (jeton
     `exchange_rates::spawn(` : une fois dans `main.rs`, zéro dans `lib.rs`), qui **réutilise** l'aide `code()`
     d'`auto_backup` (extraite ou partagée, refs #727) au lieu de la recopier (DRY).
   - « Mois courant », « jour » et « le 1er » s'entendent en **Europe/Zurich**, non en UTC (même principe que #889) :
     `chrono-tz` n'est aujourd'hui au `Cargo.lock` (0.10.4) que par `rxing` ; l'ajouter comme **dépendance directe** de
     `kesh-api` (`chrono-tz = "0.10"`) et poser **une** aide de fuseau (`aujourdhui_zurich(now)`, d'où se déduisent le
     mois courant et le mois suivant), partagée par la tâche, la connexion et les bornes du `POST /fetch`,
     testée aux bords (31 → 1er, UTC contre Zurich).
   - Un échec se journalise (`tracing::warn!`) ; un `not_published` du **mois suivant**, état ordinaire jusqu'à sa
     publication (S1 : vérifié chaque jour jusqu'à son arrivée), se journalise en `tracing::info!`, non en avertissement.
     Le tour s'écrit dans `_kesh_exchange_rate_fetch` (`triggered_by = 'auto'`,
     pire statut du tour, AC-1) ; jamais bloquant au démarrage ni à la connexion. Non auditée (C-37-6). **Seuls** la
     tâche et le déclenchement à la connexion écrivent `triggered_by = 'auto'`. La mémoire des 15 minutes (AC-5)
     s'applique aussi. Rien de ceci ne modifie une ligne existante de `exchange_rates` (seule la copie de l'AC-3 bis y
     insère les lignes qui manquent).
5. **AC-5 — API.**
   - **Cours effectif** (D9, U1 — CR-5 : **une seule valeur**) : pour une société, une devise, un genre et une date,
     le **cours retenu par la société**, c'est-à-dire `exchange_rates.rate` de sa ligne, avec sa `source` (`afc` |
     `manual`) — jamais `_kesh_fetched_exchange_rates`, qui ne sert qu'à la copie (AC-3 bis) et à l'**information**
     (valeur publiée affichée, écart signalé). Une seule fonction de dépôt le lit (nom au développement, p. ex.
     `exchange_rates::find_company_rate_in_tx(conn, company_id, currency, kind, rate_date) ->
     Result<Option<CompanyRate>, DbError>`, `CompanyRate { id, rate, source: RateOrigin { Afc, Manual, Other(String) } (A30), version,
     acknowledged_rate, updated_at }`, `&mut MySqlConnection`) : `lookup`, les `GET`, 37-3a (`FX_RATE_NOT_AFC`), 37-3b
     (par `lookup`) et 37-5 passent par elle. La lecture de la valeur publiée, pour l'affichage seulement,
     `find_fetched_rate_in_tx(conn, source, currency, kind, rate_date)` (nom au développement ; ex-`find_rate_in_tx`),
     n'est employée par aucun contrôle d'écriture.
   - `GET /api/v1/exchange-rates?month=AAAA-MM[&currency=USD]` (cours mensuels) **ou** `?date=AAAA-MM-JJ[&currency=USD]`
     (cours journaliers ; `month` et `date` ensemble ou ni l'un ni l'autre → 400) : toutes les devises qui ont une
     ligne `exchange_rates` de la société **ou** une valeur publiée pour ce mois ou ce jour (union), ou une seule ;
   - `GET /api/v1/exchange-rates/status` : état de `_kesh_exchange_rate_fetch` (`trigger`, date, statut, code
     d'erreur, genre et date du cours) ;
   - `POST /api/v1/exchange-rates/fetch { year, month }` (cours mensuel) **ou** `{ date: "AAAA-MM-JJ" }` (cours
     journalier, D10 ; les deux formes ensemble ou aucune → 400) — dans **`comptable_routes`** (Administrateur ou
     Comptable ; mois ≥ 2000-01 et ≤ **mois suivant** le mois courant en Europe/Zurich, **à toute date** (relevé
     CR-2 P2, *Source* : le mois suivant peut être publié avant le 25) ; jour ≥ 2000-01-01 et **strictement antérieur**
     au jour courant en Europe/Zurich, cf. *Cours journaliers* — même aide de fuseau que l'AC-4 ; sinon 400. Un mois
     suivant demandé avant sa publication rend `404 RATE_NOT_PUBLISHED`, comme tout mois non publié) : récupère ce mois ou ce jour, **auditée sous l'utilisateur**
     (`NewAuditLogEntry::from_current_user`, méthode du trait `AuditActor`, `audit.rs:24` : une clé d'API est attribuée comme telle ;
     `exchange_rates.fetched`, genre, mois ou jour, `<datum>` lu pour un jour, nombre de devises écrites, écartées,
     **corrigées** (`corrected` : valeur publiée différente de celle déjà connue, remplacée par la règle (c) de l'AC-3 ;
     remplace le `republished` de CR-3) et **copiées** (`copied`, toutes sociétés, AC-3 bis) — écrite **sur un succès
     seulement** : un
     404 ou un 503 n'écrit que `_kesh_exchange_rate_fetch`) ; écrit `_kesh_exchange_rate_fetch` avec
     `triggered_by = 'manual'` ; réponses `200 {kind, rateDate: "AAAA-MM-JJ", sourceDate|null, count, rejected, corrected}` (`count`, `rejected` et `corrected` sont des
     **nombres** ; les codes écartés vont au journal du serveur), `404 RATE_NOT_PUBLISHED`,
     `503 RATE_SOURCE_UNAVAILABLE` (code explicite, message sans URL ni trace — **jamais 502** : le frontend lit 502 et
     504 comme une issue inconnue venue d'un proxy, `frontend/src/lib/shared/utils/unknown-outcome.ts`). ⚠️ Mais
     **tout 503** allume aujourd'hui le bandeau « service dégradé » (`apiHealth.setDegraded()`,
     `api-client.ts:393-394`, avant toute lecture du corps), conçu pour une base injoignable : la fonction d'API du
     frontend qui appelle `POST /fetch` (seule entrée, partagée par cet écran, 37-3b et 37-5) passe une option neuve
     de `CallOptions` qui ne l'allume pas (`degradedOn503: false`, nom au développement), avec son test Vitest. ⚠️ Elle
     ne couvre que le **503** : un `TIMEOUT` ou une coupure du client allume le même bandeau par la branche `catch`
     (`api-client.ts:421-422`, `apiHealth.setDegraded()`), ce qui reste voulu (le serveur ne répond pas) — à dire dans le
     test, qui ne doit pas affirmer le contraire. **Tout**
     échec de récupération se rend ainsi — `timeout`, `connect`, `http_status`, `too_large`, mais aussi `malformed`,
     `month_mismatch`, `date_mismatch` et `no_valid_rate` (la source a répondu, inutilisablement) : jamais 500 ; le code classé va à
     `last_error`, pas à l'utilisateur. Un mois ou un jour `not_published` ou en échec n'est pas redemandé à la source avant 15
     minutes (`rate_fetch_memo`, clé `(genre, rate_date)`, alimentée aussi par la boucle) : la réponse d'origine est
     rendue, sans nouvelle ligne d'audit ni écriture de `_kesh_exchange_rate_fetch` ;
   - `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ[&kind=month|day]` (`kind` absent = `month`) —
     **en lecture seule**, sans appel sortant ni écriture ni audit, ouverte à tout rôle authentifié : **cours effectif**
     (cours retenu par la société, U1) du mois de la date (`month`) ou de la date elle-même (`day`) —
     `200 { currency, kind, rateDate: "AAAA-MM-JJ", effectiveRate, source: "afc"|"manual" (ou la valeur lue telle quelle, A30), publishedRate,
     publishedUnit, sourceDate }` (décimaux en chaînes ; `effectiveRate` et `source` = la ligne `exchange_rates` ;
     `publishedRate` (par unité), `publishedUnit` et `sourceDate` = la **dernière valeur publiée**
     (`_kesh_fetched_exchange_rates`), **pour information**, `null` si aucune — elle peut différer de `effectiveRate`
     après une correction de l'AFC non reprise, ou pour un cours `manual` ; `sourceDate` à `null` hors `day` ; forme lue
     par 37-3b et 37-5 ; **CR-5** : les champs `afcRate` et `overridden` de CR-1 n'existent plus — la provenance d'un
     cours est `source`, et une ligne d'écriture n'est `afc` que si son cours égale `effectiveRate` d'une ligne `source =
     'afc'`, 37-3a AC-4) —, ou `404 RATE_UNAVAILABLE` quand la société n'a **aucune** ligne `exchange_rates` pour cette
     unité (même si une valeur publiée existe : le `POST /fetch` qui suit la copie, AC-3 bis),
     `details: { currency: "USD", month: "AAAA-MM-01" }` pour `month` (jamais 1.0) —
     **forme unique** de ce code pour un cours mensuel, que le `DbError::RateUnavailable` de 37-3a rend à l'identique,
     et d'où l'écran tire `{year, month}` du `POST /fetch` ; pour `day`, la même forme **plus** `date: "AAAA-MM-JJ"`
     (`month` = le 1er du mois de cette date), d'où l'écran tire `{ date }` du `POST /fetch`. L'écran de saisie (37-3b) et la réévaluation (37-5)
     appellent `POST /fetch` sur ce 404 (rôles Administrateur et Comptable), puis relisent ; sans cours, saisie
     manuelle. Devise `CHF` (devise principale, sans cours) → 400 `Validation`, sans code dédié ; devise absente de la liste ISO 4217 de 37-1
     (`kesh-core::currency`) → 400 `CURRENCY_UNKNOWN` (code de 37-1) ; `month` ou `date` absents ou illisibles (les deux `GET`) → 400 ;
     `kind` hors `month|day` → 400. Un cours saisi sans valeur publiée rend 200 (`source: "manual"`, `publishedRate:
     null`) : aucun `POST /fetch` n'est alors nécessaire. **Forme `day`** : `DbError::RateUnavailable { currency, month }` (37-3a, 37-5) ne porte
     pas de date ; le handler de `lookup` pour `kind=day` construit donc **directement** l'`AppError` 404 (`details`
     avec `date`), sans passer par `DbError`, dont la variante reste à deux champs.
   - **Cours retenus par la société** (D9, U1, U3) — deux routes dans **`comptable_routes`** (Administrateur ou
     Comptable ; Consultation 403), **auditées** sous l'utilisateur (`from_current_user`) **dans la transaction** de
     l'écriture, ligne `exchange_rates` lue `FOR UPDATE` (ex-`PUT` et `DELETE /exchange-rates/overrides`, CR-1 ; la
     route `…/republications/adopt` de CR-3 est **retirée**, l'adoption globale n'existant plus, U1) :
     - `PUT /api/v1/exchange-rates/rates { currency, kind: "month"|"day", rateDate: "AAAA-MM-JJ", rate, version? }`
       **saisit** un cours : la ligne prend `rate` et `source = 'manual'` (créée si absente), `acknowledged_*` à `NULL`,
       `updated_by` = l'utilisateur : `rateDate` le 1er du mois pour `month` (sinon 400), 2000-01-01 ≤ `rateDate`
       ≤ 2099-12-31 (sinon 400) ; `rate` en chaîne décimale, **par unité** (1 JPY, jamais 100 JPY), contrôlé par la
       fonction des bornes d'un cours seul (AC-2) → 400 **`FX_RATE_INVALID`**, `details: { currency }` (code **créé
       ici**, avec sa variante `AppError` et sa traduction dans les quatre locales ; 37-5 le **réutilise** pour les
       cours de sa requête) ; `currency` comme aux `GET` (`CHF` → 400, inconnue → `CURRENCY_UNKNOWN`, lue telle
       quelle). Concurrence optimiste : `version` absent = création (une ligne déjà présente → 409
       `OPTIMISTIC_LOCK_CONFLICT` ; **y compris** deux créations simultanées : la violation de l'unicité `(company_id, currency, kind, rate_date)` est rattrapée et rendue 409, jamais 500) ; `version` présent = remplacement de cette version (ligne absente ou autre version
       → 409 `OPTIMISTIC_LOCK_CONFLICT`) — une ligne `afc` copiée par Kesh a elle aussi une `version`, que l'écran
       envoie ; `version` + 1 à chaque remplacement. Rend `200` avec la forme d'une case (`Cell`, ci-dessous). Un cours
       saisi **égal** à la valeur publiée est conservé tel quel, `manual` (l'écran n'en crée jamais : il envoie alors
       le ↑, AC-6, A2) ;
     - `POST /api/v1/exchange-rates/rates/reset { currency, kind, rateDate, version?, publishedRate? }` — le **↑**
       (« reprendre la valeur publiée ») : la ligne prend la **dernière valeur publiée** (`_kesh_fetched_exchange_rates`,
       `rate_per_unit`) avec **`source = 'afc'`**, `acknowledged_*` à `NULL`, `updated_by` = l'utilisateur — qu'elle
       soit `manual` ou d'une autre source que `afc` (cours saisi abandonné, A30) ou `afc` (correction de l'AFC reprise, U1) ; créée si absente (case vide
       dont la valeur est publiée). `publishedRate` = la valeur que l'écran a montrée : si la valeur publiée a changé
       entre-temps → 409 `OPTIMISTIC_LOCK_CONFLICT` (l'utilisateur reprend ce qu'il a vu, jamais une valeur plus neuve) ;
       `version` obligatoire si la ligne existe (absente ou autre version → 409), interdite sinon (400). **Sans valeur
       publiée** : ligne `manual` ou d'une autre source que `afc` (A30) → **supprimée** (« Retirer le cours saisi » : la case redevient vide) ; ligne `afc` →
       404 `NOT_FOUND` (installation restaurée avant toute récupération : rien à reprendre, l'écran ne propose pas ↑) ;
       aucune ligne → 404. **Portée (U3)** : la société de l'utilisateur seule ; l'option « toutes les sociétés sous
       votre gestion » est **spécifiée, non livrée** — un utilisateur n'appartient qu'à une société
       (`users.company_id BIGINT NOT NULL`, `20260419000002_users_company_id.sql:23` ; `CurrentUser.company_id: i64`,
       `middleware/auth.rs:41` ; aucune table de rattachement utilisateur–sociétés, relevé au code le 2026-11-14) : quand
       Kesh gérera plusieurs sociétés par utilisateur, un champ `scope: "company" | "managedCompanies"` appliquera la
       reprise, pour chaque société gérée dont un exercice multi-devises **ouvert** a cette unité parmi ses unités
       utiles (AC-3 bis), aux seules lignes `afc` qui diffèrent de la valeur publiée (jamais à une ligne `manual` ni d'une autre source, A30), dans
       une transaction, une ligne d'audit par société ; aujourd'hui le corps n'a pas ce champ (un champ inconnu est
       ignoré comme ailleurs). Issue à ouvrir avec la PR (C-37-8). Rend `200` avec la `Cell` à jour (`rate` `null` après
       une suppression) ;
     - audit : actions `exchange_rate.created`, `exchange_rate.updated` (le `PUT`) et `exchange_rate.reset` (le ↑, dont
       la suppression d'un cours saisi sans valeur publiée), type d'entité neuf `exchange_rate` (`ENTITY_TYPES`,
       `audit_labels.rs:24`, liste triée : entre `email_template` et `export` — remplace `exchange_rate_override`),
       identifiant = `exchange_rates.id`, détails `{ currency, kind, rateDate, before: { rate, source } | null, after: {
       rate, source } | null, publishedRate }` (cours en chaînes) ; clés
       `audit-log-action-exchange-rate-{created,updated,reset}` et `audit-log-entity-exchange-rate` dans les quatre
       locales ;
     - ⛔ un cours saisi ou repris **ne réécrit aucune écriture** : les lignes déjà passées gardent leur cours (37-3a) ;
       seul le cours **proposé** des saisies suivantes change (37-3b, 37-5) — et, pour une ligne `afc` reprise à une
       nouvelle valeur, le contrôle `FX_RATE_NOT_AFC` des lignes `afc` nouvelles ou modifiées (37-3a AC-4, A17).
   - **Tableau annuel d'un exercice (R6)** — `GET /api/v1/fiscal-years/{id}/exchange-rates[?all=true]`, en lecture
     seule (aucun appel sortant), tout rôle authentifié ; exercice d'une autre société → 404 ; exercice mono-devise →
     409 `MULTI_CURRENCY_DISABLED` (`details: { fiscalYearId }`). Réponse `{ fiscalYearId, startDate, endDate,
     openingRateKind, closingRateKind, openingDate, closingDate, months: ["AAAA-MM-01", …], rows: [{ currency, hasAccount, months:
     [Cell, …], opening: Cell, closing: Cell }] }` : `months` = les mois de l'exercice **dans l'ordre de l'exercice**
     (juillet à juin pour un exercice au 1er juillet), **du mois de `startDate` au mois de `endDate` inclus, quelle que
     soit la durée** (S4, CR-3 : aucune constante de douze mois — six colonnes pour un exercice de six mois, dix-huit
     pour un exercice de dix-huit mois, treize pour un exercice du 15.03 au 14.03 ; `chk_fiscal_years_dates` n'impose
     que `end_date > start_date`) ; `openingDate` = veille de `startDate` (genre `day`) ou 1er du mois de cette
     veille (`month`) — le **dernier mois de l'exercice précédent** (T4, CR-4 : le cours d'ouverture est le cours de
     clôture précédent). Pour un exercice qui commence le 1er d'un mois, ce mois précède le premier des `months` et
     n'en fait pas partie ; pour un exercice qui commence en cours de mois (15.03.2024), il **est** le premier des
     `months` (mars 2024) : `openingDate` égale alors la date de cette colonne, la case d'Ouverture et la première
     colonne sont **une seule case** (même devise, genre et date) ; **une seule fonction pure** de `kesh-core`
     calcule `openingDate` et `closingDate` pour ce `GET` et pour `unites_utiles` (AC-4) ; le cours proposé à
     l'ouverture (37-3b AC-1) se calcule **côté frontend** en TypeScript : un jumeau, non la même fonction. La parité
     tient par des **vecteurs communs** : une fixture JSON, `crates/kesh-core/tests/fixtures/fx_opening_dates.json`
     (créée ici, sur le modèle de `fx_rounding.json` de 37-3a ; tableau de `{ startDate, endDate, openingRateKind,
     closingRateKind, openingDate, closingDate }`, dates ISO), **lue par le test Rust** de la fonction de `kesh-core`
     (37-2) **et par le test Vitest** de la fonction TS (37-3b), même fichier, aucune copie (le jumeau TS ne calcule que `openingDate` : le Vitest ne contrôle que ce champ, `closingDate` n'est contrôlé que côté Rust) ; au moins les exercices au
     1er janvier, 1er avril, 1er juillet, au 15.03 (veille 14.03, mois mars), au 01.03.2024 (veille 29.02.2024, mois
     février 2024), de six et de dix-huit mois, au 01.01.2000 (veille 31.12.1999 : la date se calcule, c'est
     `unites_utiles` qui l'écarte, AC-4), chacun en genres `day` et `month` ; `closingDate` = `endDate` (`day`) ou 1er du dernier mois (`month`) ; **`Cell`** (U1, CR-5 — une seule valeur) =
     `{ kind, rateDate, effectiveRate, source, row: { id, version, updatedAt, updatedByKesh } | null, published: {
     rate, publishedUnit, publishedRate, sourceDate, fetchedAt, valueChangedAt } | null, corrected, acknowledged }` —
     `effectiveRate` et `source` de la ligne `exchange_rates` (`null` sans ligne : jamais un 404 par case) ;
     `published` = la dernière valeur publiée, **pour information** ; `corrected` = vrai si et seulement si la ligne est
     `afc` et que `published.rate` diffère de `effectiveRate` (correction de l'AFC non reprise) ; `acknowledged` = vrai
     si `corrected` et que `acknowledged_rate` égale `published.rate` (l'avertissement de connexion est acquitté pour
     cette valeur ; une correction suivante le rouvre) ; `updatedByKesh` = `updated_by IS NULL` (copie automatique).
     Les champs `afcRate`, `overridden`, `override` et `republished` de CR-1 et CR-3 n'existent plus. Test du `GET` sur
     un exercice de six et de dix-huit mois (nombre et ordre des colonnes, `openingDate`, `closingDate`). Lignes (`hasAccount` = au moins un compte de la société, archivé compris, porte la devise) : les
     devises des comptes de la société (archivés compris s'ils portent une ligne dans l'exercice) et celles qui ont
     une ligne `manual` ou d'une autre source que `afc` (A30) dans l'exercice, triées ; `all=true` ajoute toutes les devises qui ont une ligne
     `exchange_rates` ou une valeur publiée pour une unité de l'exercice. Les cases se lisent en **une** connexion, par
     lectures groupées par genre et plage de dates des deux tables — pas une requête par case ; la lecture groupée
     (nom au développement) est la **seule** autre lecture qui joint `exchange_rates` et
     `_kesh_fetched_exchange_rates`, et elle partage avec `find_company_rate_in_tx` la projection de la ligne (DRY).
   - **Genres d'ouverture et de clôture (R6)** — `PUT /api/v1/fiscal-years/{id}/rate-kinds { openingRateKind,
     closingRateKind }` (`"day"` | `"month"`, les deux obligatoires), dans **`comptable_routes`** (Administrateur ou
     Comptable ; Consultation 403), handler sous `retry_on_deadlock` comme les autres routes d'exercice ; écrit les
     colonnes de 37-1 sous `FOR UPDATE` de l'exercice ; refus, dans l'ordre : 401, 403 → 400 `VALIDATION_ERROR`
     (corps illisible, champ absent ou valeur hors `day|month` : rejet de l'extracteur `impl_validation_extractor`,
     `extractors.rs:18`, **avant** le handler — un corps typé ne peut se juger après le 404) → 404 (exercice d'une autre
     société) → 409 `MULTI_CURRENCY_DISABLED` (exercice mono-devise — une société mono-devise n'a que des exercices
     mono-devise, 37-1 AC-3 bis —, `details: { fiscalYearId }`) → 400 `FISCAL_YEAR_CLOSED` (exercice clos : ses genres sont figés), mode et statut relus sous le
     `FOR UPDATE` ; no-op → 200 sans audit ; audit `fiscal_year.rate_kinds_updated` (`{ before, after }`, `ACTIONS`
     triée, quatre locales). Le choix ne réécrit rien : il change le cours **proposé** à l'ouverture (37-3b) et à la
     réévaluation (37-5), et les jours que récupère la tâche automatique (AC-4) ; les unités neuves reçoivent la copie
     des cours déjà récupérés (AC-3 bis).
     **Exercice suivant (U4, CR-5 — remplace l'arbitrage A22, « le changement ne se propage pas »)** : le corps porte
     aussi `applyToNextOpening` (booléen, facultatif, **absent = `false`** pour un client de l'API ; l'écran l'envoie
     toujours, AC-6). L'**exercice suivant** est celui de la société qui commence le lendemain de `end_date` (lu sous
     `FOR UPDATE`, **après** l'exercice réglé : ordre chronologique, celui des autres routes d'exercice ; la lecture de
     `NEXT_AFTER_SQL`, `repositories/fiscal_years.rs:207`, sert de modèle, avec la condition `start_date = end_date + 1
     jour` — un exercice séparé par un trou n'est pas « suivant », comme en 37-1 AC-3 bis). Quand `closingRateKind`
     **change**, que l'exercice suivant existe et que `applyToNextOpening` est vrai : son `opening_rate_kind` prend la
     valeur de `closingRateKind` (même genre : le cours d'ouverture reste le cours de clôture de l'exercice réglé, T4),
     **s'il est ouvert** — clos, il n'est pas touché et la réponse le dit (`nextFiscalYear: { id, applied: false, reason:
     "closed" }`) ; son mode n'est pas relu (le genre n'est qu'un réglage ; l'exercice suivant d'un exercice
     multi-devises l'est aussi : aucun exercice ne se crée avant un exercice mono-devise, 37-1 AC-3 bis, D-CR7-2). `applyToNextOpening` vrai sans changement de `closingRateKind` ou sans exercice suivant : sans
     effet, sans erreur. **Aucune écriture n'est modifiée, et il n'y a rien à modifier** : entre deux exercices Kesh, les
     soldes de bilan sont **cumulés** d'un exercice à l'autre, sans écriture de report (`opening.rs`, 37-4 AC-1) ; les
     soldes d'ouverture ne se saisissent que sur le **premier** exercice de la société
     (`fiscal_years::find_first_by_company`, `routes/opening_balances.rs:240`, `:379`), qu'aucun exercice ne précède —
     l'exercice suivant d'un exercice Kesh n'en porte donc jamais, et le cours d'ouverture n'y sert qu'au cours
     **proposé** et à l'affichage (37-1 AC-1). Une seule ligne d'audit `fiscal_year.rate_kinds_updated` sur l'exercice
     réglé, détail `{ before, after, nextFiscalYear: { id, before, after } | null }` ; réponse `200 { fiscalYear,
     nextFiscalYear: { id, openingRateKind, applied, reason? } | null }` (exercices au format de `FiscalYearResponse`).
     Tests : clôture `day` → `month` avec exercice suivant ouvert et `applyToNextOpening` vrai → ouverture du suivant en
     `month`, une ligne d'audit avec les deux exercices ; même geste à `false` ou sans le champ → suivant inchangé ;
     exercice suivant clos → inchangé, `applied: false` ; exercice séparé par un trou → pas de suivant ; ouverture seule
     changée → rien au suivant ; aucune écriture modifiée (nombre et contenu des lignes de `journal_entries` identiques).
   - **Corrections de l'AFC et avertissement, par société (U1, CR-5 — remplace les trois routes globales de S6)** — une
     correction (règle (c) de l'AC-3) met à jour la valeur publiée ; elle ne change ni le cours retenu d'aucune
     société, ni `lookup`, ni aucune ligne passée. Elle s'**annonce** à chaque société dont une ligne **`afc`** diffère
     de la nouvelle valeur ; une ligne `manual` qui diffère ne s'annonce pas (elle montre « AFC : x » et ↑, AC-6). Deux
     routes, toutes deux par société (A11, « global à l'installation », est **retiré** : U3) :
     - `GET /api/v1/exchange-rates/notices` (Administrateur ou Comptable, **`comptable_routes`** ; Consultation 403 ;
       lecture seule) → `{ notices: [{ currency, kind, rateDate, effectiveRate, publishedRate, publishedUnit,
       valueChangedAt }] }` : les lignes `exchange_rates` de la société, `source = 'afc'`, dont la valeur publiée
       (`_kesh_fetched_exchange_rates`, même source, devise, genre et date) **diffère** de `rate` et n'est **pas
       acquittée** à cette valeur (`acknowledged_rate IS NULL OR acknowledged_rate <> f.rate_per_unit`, `f` = la ligne de `_kesh_fetched_exchange_rates`), triées par date
       puis devise ; liste vide sinon. Une ligne sans valeur publiée (installation restaurée) n'est pas une notice ;
     - `POST /api/v1/exchange-rates/notices/acknowledge { notices: [{ currency, kind, rateDate, publishedRate }] }`
       (mêmes rôles) : sous `FOR UPDATE` des lignes de la société, pose `acknowledged_rate = publishedRate`,
       `acknowledged_at` = maintenant sur chaque ligne encore `afc` dont la valeur publiée **égale encore** celle du
       corps (une valeur changée entre-temps n'est pas acquittée : la nouvelle s'annonce) ; ne change ni `rate`, ni
       `source`, ni `version` (l'acquittement n'est pas un cours, et un `PUT` ou un ↑ concurrents ne doivent pas
       rougir pour lui) ; 1 à 500 entrées, sinon 400 `Validation` ; réponse `200 { acknowledged: n }` ; audit
       `exchange_rates.corrections_acknowledged` (entité `company`, identifiant = la société, `{ notices: [...] }`, les
       seules acquittées ; aucune ligne d'audit si `n = 0`), action ajoutée à `ACTIONS` (liste triée), clé
       `audit-log-action-exchange-rates-corrections-acknowledged` dans les quatre locales ;
     - la reprise de la valeur corrigée est le ↑ (`POST …/rates/reset`, ci-dessus), par société (U3). Conséquence,
       écrite au manuel (testée par 37-3a) : les lignes d'écriture `afc` déjà passées gardent leur cours, et un `PUT` qui
       les laisse **inchangées** passe (A17 : seules les lignes `afc` nouvelles ou modifiées sont contrôlées) ; une
       ligne `afc` **nouvelle ou modifiée** à l'ancien cours est refusée `FX_RATE_NOT_AFC` (37-3a AC-4, 4 : comparaison
       au cours retenu par la société) — elle passe en `manual` ou prend le nouveau cours ;
     - concurrence avec une récupération : la règle de l'AC-3 écrit `_kesh_fetched_exchange_rates` sous `FOR UPDATE` ;
       le ↑ lit la valeur publiée sans verrou et la compare au `publishedRate` du corps (409 si elle a changé) ;
       l'acquittement fait de même : l'utilisateur n'acquitte ni ne reprend jamais une valeur qu'il n'a pas vue.
     - Tests : deux sociétés A et B, ligne `afc` USD décembre dans chacune ; correction → notice dans A et B ;
       acquittement dans A → notice absente de A, présente dans B ; ↑ dans B → ligne de B à la nouvelle valeur, ligne
       de A inchangée ; nouvelle correction → notice de retour dans A ; ligne `manual` qui diffère → aucune notice.
   - Toutes ces routes exigent le mode multi-devises de la société courante : **`409 MULTI_CURRENCY_DISABLED`**, même
     statut que dans la 37-1.
   - Nouvelles variantes `AppError` (`errors.rs`) et leur traduction côté frontend pour `RATE_NOT_PUBLISHED`,
     `RATE_SOURCE_UNAVAILABLE`, `RATE_UNAVAILABLE`. **Propriétaire unique de `RATE_UNAVAILABLE`** : cette story (elle
     précède 37-3a) crée la variante `DbError::RateUnavailable { currency, month }` (`crates/kesh-db/src/errors.rs`,
     `enum DbError` :668) **et** son bras de la correspondance `DbError` → `AppError` (`crates/kesh-api/src/errors.rs`,
     404, `details` ci-dessus) ; le handler de `lookup` s'en sert
     sur `find_company_rate_in_tx` → `None`. 37-3a et 37-5 la **réutilisent**, sans la redéfinir (37-5 la rend pour
     un cours mensuel, la forme `month` ; la forme `day` n'est rendue que par `lookup`).
   - **Formes des réponses** (camelCase, décimaux en chaînes, comme `lookup`) : `GET /exchange-rates` →
     `{ kind, rateDate: "AAAA-MM-JJ", rates: [{ currency, …Cell }] }` (chaque élément est une `Cell` du tableau annuel,
     plus la devise : une seule forme, DRY ; `published` à `null` pour une devise qui n'a qu'un cours saisi,
     `effectiveRate`/`source`/`row` à `null` pour une devise publiée que la société n'a pas retenue ; tri par devise ;
     liste vide si ni ligne de la société ni valeur publiée pour ce mois ou ce jour) ; `GET /exchange-rates/status` → `{ trigger,
     lastRunAt, status, error, kind, rateDate }`, tous `null` tant qu'aucune récupération n'a eu lieu (la ligne est
     créée paresseusement, AC-1).
     `fetchedAt` et `lastRunAt` sont **stockés en UTC** (`Utc::now().naive_utc()`, comme `auto_backup`) et **affichés en
     heure locale suisse** par l'écran (leçon de #889).
   - **Ordre des refus** (même ordre que 37-1/37-3a) : authentification et rôle (401, 403) → société en mono-devise
     (`409 MULTI_CURRENCY_DISABLED`) → paramètres illisibles ou hors bornes (400, dont `CHF`) → devise inconnue
     (`400 CURRENCY_UNKNOWN`) → cours absent (404) ; pour le `PUT` d'un cours saisi, après la devise inconnue :
     `400 FX_RATE_INVALID`, puis la version (`409 OPTIMISTIC_LOCK_CONFLICT`). La devise se lit **telle quelle**, sans normalisation : `usd` →
     `400 CURRENCY_UNKNOWN`. Le ↑ (`POST …/rates/reset`) suit le même ordre, puis : ligne ou valeur publiée absente
     (404), `version` ou `publishedRate` périmés (409). **Seul écart**, commun aux routes à corps JSON (`POST /fetch`,
     `PUT …/rates`, `POST …/rates/reset`, `POST …/notices/acknowledge`, `PUT /rate-kinds`) : un corps illisible est refusé par l'extracteur (`impl_validation_extractor`, 400
     `VALIDATION_ERROR`) avant le handler, donc avant le 409 ; les champs se lisent en chaînes (`rate`, `rateDate`,
     `currency`) et se jugent ensuite dans l'ordre ci-dessus.
   - **Audit et registres** : action `exchange_rates.fetched` ajoutée à `ACTIONS` (`crates/kesh-api/src/audit_labels.rs`,
     liste triée, garde `tests/audit_label_registry.rs`) ; type d'entité `installation` avec `AUDIT_ENTITY_ID_NONE`
     (précédent `admin.backup_run`, `routes/backup_schedule.rs:177-191`, `emit_audit` après coup — la table est
     globale et `audit_log.entity_id` est `NOT NULL`) ; clé `audit-log-action-exchange-rates-fetched` dans les quatre
     locales. `POST /fetch` et les routes des cours de la société (CR-1 : `PUT` et `DELETE /exchange-rates/overrides` ;
     depuis CR-5, `PUT /exchange-rates/rates` et `POST /exchange-rates/rates/reset` — décompte qui fait foi : le
     paragraphe CR-5 ci-dessous) entrent dans `LIB_ROUTES`
     (`tests/audit_route_registry.rs`, chacune `Traced, SansEcritureAuJournal` — aucune n'écrit au journal comptable ;
     précédent `company_invoice_settings::update_invoice_settings`, :387) :
     `the_registry_partition_is_what_the_story_declares` gagne **+3 routes et +3 tracées** (état au 2026-11-14 : 146 routes,
     139 tracées, total registre 149 ; **ces chiffres s'écrivent en deltas** — la 37-1 et la 37-5 ajoutent aussi leur
     route mutante, l'ordre d'intégration décide du chiffre absolu, à recompter au code), et le total registre **+4**
     avec l'endpoint `_test` (entrée `("post", "/exchange-rates/source", Exempt(…), SansEcritureAuJournal)` de
     `TEST_ENDPOINT_ROUTES`, sur le modèle de `/password-reset-token`, `audit_route_registry.rs:620` : il n'écrit
     rien au journal d'audit ; `Exemptee` signifie « écrit au journal et n'est pas rejouée », réservé à `full_import`, `/seed`,
     `/reset`) ; la phrase du manuel d'administration (`admin-manual.tex:3217`, « N des M routes à verbe
     mutant » et « N + 4 + 3 = M », gardée par `le_manuel_dit_le_compte_du_registre`) suit le chiffre recompté.
     Le second test, `the_replay_partition_is_what_the_story_declares` (`audit_route_registry.rs:1450`), compte aussi :
     `sans_ecriture` **+4** avant la révision CR-2 (le `POST /fetch`, le `PUT` et le `DELETE` des cours saisis, et
     l'endpoint `_test`), **+5 avec le `PUT /fiscal-years/{id}/rate-kinds`** (valeur qui fait foi, ci-dessous),
     `exemptees` et `rejouees` inchangés, `tout.len()` +5 (:1502) ; commentaires
     et `//!` du module recomptés, en deltas. *(Alignement de l'orchestrateur sur 37-1-P4-IF-1.)* **CR-2 (valeurs qui font foi, en deltas, annulant les « +3 / +4 » ci-dessus)** : le
     `PUT /fiscal-years/{id}/rate-kinds` s'y ajoute (`fiscal_years::update_rate_kinds`, `Traced,
     SansEcritureAuJournal`, comme les routes d'exercice `audit_route_registry.rs:537-539`) — la 37-2 compte donc
     **+4 routes, +4 tracées, total registre +5, `sans_ecriture` +5, `tout.len()` +5** ; le `GET` du tableau annuel
     n'est pas un verbe mutant. **CR-3 (valeurs qui font foi, en deltas, annulant les valeurs CR-2 ci-dessus)** : les
     deux `POST` des cours republiés (`…/notices/acknowledge`, `…/republications/adopt`, S6) s'y ajoutent, chacun
     `Traced, SansEcritureAuJournal` (ils écrivent au journal d'audit, jamais au journal comptable) — la 37-2 compte
     donc **+6 routes, +6 tracées, total registre +7, `sans_ecriture` +7, `tout.len()` +7** (`rejouees` et
     `exemptees` inchangés) ; le `GET /exchange-rates/notices` n'est pas un verbe mutant. Phrase du manuel
     d'administration recomptée en conséquence. **CR-5 (valeurs qui font foi, en deltas, annulant les valeurs CR-3
     ci-dessus)** : `DELETE /exchange-rates/overrides` et `POST …/republications/adopt` sont **retirées** (−2), `PUT
     /exchange-rates/overrides` devient `PUT /exchange-rates/rates` (même compte), `POST /exchange-rates/rates/reset`
     s'ajoute (+1, `exchange_rates::reset_rate`, `Traced, SansEcritureAuJournal`). Routes mutantes de la 37-2 : `POST
     /exchange-rates/fetch`, `PUT /exchange-rates/rates`, `POST /exchange-rates/rates/reset`, `POST
     /exchange-rates/notices/acknowledge`, `PUT /fiscal-years/{id}/rate-kinds` — la 37-2 compte donc **+5 routes, +5
     tracées, total registre +6 (avec l'endpoint `_test`), `sans_ecriture` +6, `tout.len()` +6** (`rejouees` et
     `exemptees` inchangés). **Rejeu** : les cinq routes écrivent `exchange_rates` (ou, pour `PUT /rate-kinds`, y
     copient) et prennent `companies` en partage par sa clé étrangère, en cycle possible entre elles, avec la
     récupération et la copie (mêmes clés uniques de `exchange_rates`) et, pour `PUT /rate-kinds` (exercice `FOR UPDATE`
     puis `companies` en partage), avec l'activation de 37-1 (`companies` `FOR UPDATE` puis exercices `FOR UPDATE`) :
     leur transaction passe sous
     `kesh_db::retry::retry_on_deadlock` ; elles entrent donc au décompte du point **(vi)** du module
     (`audit_route_registry.rs:155`, « vingt-cinq routes `SansEcritureAuJournal` sont rejouées », :302 ; 37-1 le porte à
     vingt-sept, CR-7) — **+5**, en deltas, recompté au code, avec un test de victime d'interblocage pour `PUT …/rates` (patron
     `fiscal_year_reopen_is_replayed_when_it_is_the_deadlock_victim` de `rejeu_interblocage_e2e.rs`, comme 37-1). Total
     de l'epic à recompter à l'intégration, en deltas.
6. **AC-6 — Écran** *Paramètres → Cours de change* (visible en multi-devises) — **tableau annuel par exercice (R6,
   CR-2 ; remplace les vues mensuelle et jour de CR-1)**. Choix de l'exercice parmi les exercices **multi-devises**
   (défaut : celui qui couvre aujourd'hui, sinon le plus récent) ; données du `GET /fiscal-years/{id}/exchange-rates`
   (AC-5). **Une ligne par devise** ; **une colonne par mois de l'exercice, du premier au dernier, dans l'ordre de
   l'exercice** (en-têtes « juil. 2024 » … « juin 2025 » pour un exercice au 1er juillet ; S4, CR-3 : un exercice
   raccourci ou allongé lors d'un changement de date de clôture donne moins ou plus de douze colonnes — l'écran lit
   `months` de la réponse, sans constante), puis **Ouverture** et **Clôture**. Chaque case montre **une seule
   valeur** (U1, CR-5 — corrige la maquette) : le **cours retenu par la société** (`effectiveRate`), modifiable ;
   dessous, la mention de sa source et de la valeur publiée (règles ci-dessous, *Mention et ↑*). Au-dessus du tableau, par exercice, les
   **deux choix** de R6 (Administrateur et Comptable ; `PUT /rate-kinds`, AC-5) : *Cours d'ouverture* — « cours
   journalier de la veille de l'ouverture (JJ.MM.AAAA) » (défaut) ou « cours mensuel du dernier mois de l'exercice
   précédent (<mois AAAA>) » (T4, CR-4 : le mois de la veille de l'ouverture — p. ex. « juin 2024 » pour un exercice
   au 1er juillet 2024 —, et non plus le premier mois ; une aide brève dit que le cours d'ouverture est le cours de
   clôture de l'exercice précédent, pour que les valeurs CHF concordent) ; *Cours de
   clôture* — « cours journalier de la date de clôture (JJ.MM.AAAA) » (défaut) ou « cours mensuel du dernier mois ».
   Les colonnes Ouverture et Clôture suivent le choix : genre `day`, la case du jour (avec la date de publication
   `sourceDate` quand elle diffère : « publié le 28.06.2024 ») ; genre `month`, la case du mois concerné — pour la
   Clôture, liée à la colonne du dernier mois (la même valeur, éditée à un seul endroit) ; pour l'Ouverture, une case
   **propre** quand l'exercice commence le 1er d'un mois (le mois de la veille n'est pas une colonne de l'exercice),
   qui est la même case (même devise, genre et date) que la colonne de ce mois dans le tableau de l'exercice qui le
   contient, s'il en est un, et, si l'exercice précédent finit la veille et a le genre `month`, que sa Clôture ; quand l'exercice commence en
   cours de mois (15.03.2024), le mois de la veille est son premier mois : l'Ouverture est **liée à la première
   colonne**, comme la Clôture à la dernière (une seule valeur, éditée à un seul endroit). En-tête de colonne : la date du cours. Bouton
   « Récupérer les cours manquants » (Administrateur et Comptable) : **un seul** `POST /exchange-rates/fetch` par
   **colonne** récupérable (mois ≤ mois suivant, AC-5, l'Ouverture en genre `month` comprise, T4 — une seule fois quand elle est liée à la première colonne ; jour
   d'Ouverture ou de Clôture en genre `day`, strictement passé ; jamais un mois < 2000-01 ni un jour < 2000-01-01,
   bornes de l'AC-5, dont la raison s'affiche au survol comme pour un jour non passé) dont au moins une ligne affichée n'a pas de valeur publiée (`published` à `null` : case vide, cours saisi sans valeur publiée, ou ligne `afc` d'une installation restaurée) ou — pour une devise **portée par un compte** de la société (`hasAccount`, AC-5) seulement — pas de cours retenu (`effectiveRate` à `null`) : la copie ne couvre que ces devises (A24), si bien qu'une devise ajoutée à l'écran sans compte resterait sans cours retenu après toute récupération et relancerait l'appel à chaque clic — une récupération
   écrit toutes les devises publiées du mois ou du jour : un appel par case doublerait appels sortants et lignes
   d'audit —, l'un après l'autre, par la fonction d'API commune (AC-5 : `timeoutMs` 90 s, sans bandeau dégradé), puis
   relecture (la récupération copie les cours publiés dans les lignes de la société qui en manquent, AC-3 bis) ; un échec s'affiche sur les cases de sa colonne sans arrêter les suivantes. « Ajouter une devise » ajoute une ligne (liste ISO 4217, hors CHF ; la ligne n'existe
   côté serveur qu'une fois un cours retenu, copié ou saisi) ; « Toutes les devises publiées » (`all=true`). État de la dernière
   récupération (automatique ou manuelle, d'après `trigger` ; genre, date, statut, code d'erreur traduit) ; mention
   de la source, de son calendrier (S1) et de son caractère non contraignant ; libellés « cours mensuel AFC » et
   « cours journalier », jamais « cours moyen ». Carte sur `settings/+page.svelte`. Défilement horizontal du seul
   tableau (colonne des devises figée) ; pas de défilement horizontal de la page. **Exercice clos** (A7 : la tâche
   automatique l'ignore, ses cours se récupèrent ici) : « Récupérer les cours manquants » reste offert, mais les champs
   et ↑ sont en lecture seule et les deux choix de genre sont désactivés (le serveur refuse `PUT /rate-kinds`,
   `FISCAL_YEAR_CLOSED`) ; Vitest. Lecture seule **de l'écran** : `PUT …/rates` et `POST …/rates/reset` ne
   connaissent pas l'exercice et restent admis par l'API (un cours saisi ou repris ne réécrit rien, AC-5). Le jour de Clôture
   (genre `day`) d'un exercice est aussi le jour d'Ouverture (veille) de l'exercice suivant **quand celui-ci commence
   le lendemain** : c'est **la même case**
   (même devise, genre et date), éditable depuis l'exercice ouvert, qui change aussi ce qu'affiche l'exercice clos
   (testé au serveur) ; de même, depuis T4, en genre `month` : le dernier mois d'un exercice est le mois d'Ouverture
   du suivant **quand celui-ci commence le lendemain** (un trou entre deux exercices rompt le partage ; testé au
   serveur et en Vitest).
   - **Cours éditable** (D9) : chaque cours effectif est un champ (Administrateur et Comptable ; lecture seule pour
     Consultation), validé à l'écran par les bornes de l'AC-2 (`AmountFormatOptions { maxDecimals: 10,
     maxIntegerDigits: 7 }`, comme 37-3b AC-2) ; une valeur changée s'enregistre à la validation du champ (`PUT
     …/rates`, avec la `version` de la ligne si elle existe, `afc` ou `manual`) ; un `409 OPTIMISTIC_LOCK_CONFLICT`
     recharge la vue avec un message.
     **Mention et ↑ (U1, CR-5 ; A2 approuvé)**, sous la valeur unique de la case — la valeur publiée n'est **jamais**
     une seconde valeur éditable :
     - ligne `afc`, valeur publiée **égale** → « **AFC ✓** », sans ↑ ;
     - ligne `afc`, valeur publiée **différente** (`corrected`, correction de l'AFC non reprise) → « **AFC corrigé :
       0.8850** (le 03.12.2024) » **en rouge** (date = `valueChangedAt`, heure locale suisse) et **↑** (libellé
       accessible « Reprendre la valeur corrigée »), qui reprend la valeur **directement** pour la société (A23 ; le
       dialogue U3 ci-dessous n'est pas rendu aujourd'hui) ; le rouge reste tant que
       la valeur n'est pas reprise (l'acquittement n'efface que l'avertissement de connexion) ; couleur par le jeton
       d'erreur existant du thème (recette visuelle : le rouge doit se voir, #640) ;
     - ligne `afc` **sans** valeur publiée (installation restaurée avant toute récupération) → « AFC (valeur publiée
       non relue) », sans ↑ ;
     - ligne `manual`, valeur publiée **différente** → « **AFC : 0.8846** » et **↑** (« Reprendre la valeur publiée »,
       direct, sans dialogue) ;
     - ligne `manual`, valeur publiée **égale** → « **AFC ✓ · saisi** » **avec** ↑ (A2 : un cours saisi égal à la
       valeur publiée reste retirable ; ↑ le repasse en `afc`) ;
     - ligne `manual` **sans** valeur publiée → « AFC : non publié » et ↑ (« Retirer le cours saisi » : la ligne est
       supprimée, AC-5) ;
     - ligne d'une **autre** source que `afc` (A30, p. ex. `ecb`) → comme une ligne `manual` dans les trois cas
       ci-dessus (mêmes ↑), la source brute entre parenthèses : « AFC : 0.8846 (ecb) », « AFC ✓ · saisi (ecb) »,
       « AFC : non publié (ecb) » ;
     - **aucune** ligne, valeur publiée présente → champ vide, « AFC : 0.8846 » et ↑ (« Reprendre la valeur publiée » :
       crée la ligne `afc`) ; aucune ligne ni valeur publiée → champ vide, « AFC : non publié ».
     Tout ↑ appelle `POST …/rates/reset` avec la `version` de la ligne et la valeur publiée affichée, puis relit le
     tableau (409 → rechargement avec message). Pour que « AFC ✓ » ne cache pas un cours saisi qu'on ne pourrait plus
     retirer (D9, A2), l'écran **ne crée jamais** une ligne `manual` égale à la valeur publiée : une valeur entrée égale
     à la valeur publiée envoie le ↑ (`reset`) si la ligne est `manual`, d'une autre source que `afc` (A30) ou absente, et rien si elle est déjà `afc` à
     cette valeur. Le cas résiduel — ligne `manual` égale posée par l'API, ou devenue égale après une correction de
     l'AFC — s'affiche « AFC ✓ · saisi » avec ↑ (testé). Un champ **vidé** ne supprime rien : la valeur affichée est
     restaurée, sans appel (seul ↑ retire un cours saisi) ; une valeur égale au cours affiché n'envoie rien. Une case
     de jour **non passé** (≥ aujourd'hui, Europe/Zurich) n'est pas récupérable (le serveur la refuse, AC-5) : la
     raison s'affiche au survol ; elle reste saisissable. Une copie automatique se distingue au survol d'une reprise par
     un utilisateur (`updatedByKesh`).
   - **Dialogue du ↑ d'une correction (U3, CR-5 — remplace A11 ; A23 : spécifié, non rendu aujourd'hui)** : tant que
     l'option « toutes les sociétés » est masquée, le ↑ d'une case « AFC corrigé » s'applique **directement** à la
     société de l'utilisateur, sans dialogue (`reset` avec la valeur affichée et la version) ; le dialogue ci-dessous
     n'apparaît que lorsque cette option devient disponible. Il (Administrateur et Comptable) dit la valeur retenue, la valeur corrigée et la date de la correction, et que la
     reprise ne touche aucune écriture passée — une ligne d'écriture `afc` à l'ancien cours reste admise tant qu'on ne
     la modifie pas, puis ne se modifie qu'en passant en `manual` ou au nouveau cours (A17). Choix : (•) « Corriger pour
     [nom de la société] » (défaut, la seule portée livrée) ; ( ) « Corriger pour toutes les sociétés sous votre
     gestion (exercices ouverts seulement) » — **spécifiée (AC-5, `POST …/rates/reset`, portée `managedCompanies`) mais
     masquée** : l'option n'est pas rendue tant qu'un utilisateur n'appartient qu'à une société (`users.company_id`,
     AC-5) ; le composant porte une condition unique (nom au développement, p. ex. `canManageSeveralCompanies`,
     constante `false` aujourd'hui) qui gouverne le **dialogue entier** (A23) et non la seule option, pour qu'il
     apparaisse sans réécrire l'écran ; issue à ouvrir avec la PR
     (C-37-8). Boutons « Corriger » et « Annuler ». Vitest : condition à `false` (état livré) → le ↑ d'une correction envoie `reset`
     directement (valeur affichée, version), sans dialogue, comme celui d'un cours saisi ; condition à `true` (test du
     composant seul) → dialogue ouvert par le ↑ d'une correction et non par celui d'un cours saisi, « Corriger » envoie
     `reset` avec la valeur affichée, « Annuler » n'envoie rien.
   - Bouton « **Revérifier les cours publiés** » (Administrateur et Comptable) : un `POST /exchange-rates/fetch` par
     colonne récupérable **déjà présente** de l'exercice, l'un après l'autre (même fonction d'API ; la tâche
     automatique ne revérifie que le **mois courant**, AC-4 (A10) : c'est par là que se découvre une correction de l'AFC
     sur les autres cours présents, et à tout moment sur le mois courant).
   - **Avertissement à la connexion (U1, par société)** : un bandeau (`ExchangeRateNoticesBanner.svelte`, sur le modèle de
     `BackupScheduleBanner.svelte` — monté dans `routes/(app)/+layout.svelte`, lu au montage, donc après chaque
     connexion, puis toutes les 30 minutes ; `role="status"`, jamais `alert`), pour l'Administrateur et le Comptable
     d'une société multi-devises seulement, tant que `GET /exchange-rates/notices` rend une liste non vide : « L'AFC a
     corrigé N cours que votre société a retenus » avec la liste (devise, mois ou jour, cours retenu → valeur corrigée),
     un lien vers l'écran *Cours de change* (exercice concerné) et un bouton « **Acquitter** » (`POST
     …/notices/acknowledge` avec la liste affichée, puis relecture) — l'acquittement vaut pour **la société** et pour
     **cette valeur** : une correction suivante avertit de nouveau. Un échec de lecture n'affiche aucun bandeau.
     L'écran de saisie des écritures (37-3b) ne les affiche pas. Vitest : bandeau absent pour Consultation, en
     mono-devise et sans notice ; présent sinon ; « Acquitter » envoie la liste affichée et le bandeau disparaît ; E2E :
     un `POST /fetch` piloté (source factice) qui rend une autre valeur pour un mois présent → case en rouge « AFC
     corrigé » et bandeau ; ↑ (direct, A23) → cours retenu changé, rouge disparu ; une autre correction →
     « Acquitter » → bandeau disparu, rouge encore là.
   - **Dialogue du genre de clôture (U4, CR-5 — remplace A22)** : changer *Cours de clôture* alors que l'exercice
     **suivant** existe (il commence le lendemain de la clôture ; l'écran le lit dans `GET /fiscal-years`) ouvre un
     dialogue : case « Appliquer aussi au cours d'ouverture de l'exercice suivant (<nom>) », **cochée par défaut**, et
     un texte : « Entre deux exercices Kesh, aucune écriture d'ouverture n'existe : les soldes de bilan sont reportés
     d'un exercice à l'autre. Ce choix ne modifie aucune écriture ; il change le cours proposé et affiché à
     l'ouverture de l'exercice suivant. » ; « Enregistrer » envoie `PUT /rate-kinds` avec `applyToNextOpening` = la case ;
     sans exercice suivant, aucun dialogue (`applyToNextOpening: false`) ; exercice suivant clos : case désactivée,
     avec la raison. Vitest : dialogue présent seulement avec un exercice suivant et un genre de clôture changé, case
     cochée par défaut, valeur envoyée ; réponse `applied: false` affichée.
   - **Tests des exercices (R6, T2 de CR-4)** : exercice **civil au 1er janvier** 2024 (douze colonnes, janvier →
     décembre ; Ouverture = cours journalier du 31.12.2023 ou, en genre `month`, décembre 2023 ; Clôture = 31.12.2024
     ou décembre 2024), exercice du **1er juillet** 2024 au 30 juin 2025 et exercice du **1er avril**
     2023 au 31 mars 2024 — colonnes dans l'ordre de l'exercice (juillet → juin, avril → mars), Ouverture = cours
     journalier du 30.06.2024 (un dimanche : publication du 28.06, `sourceDate` affiché) et du 31.03.2023, Clôture =
     cours journalier du 30.06.2025 et du 31.03.2024 (dimanche de Pâques : publication antérieure) ; avec les genres
     `month`, Ouverture = **juin 2024 / mars 2023** (T4 : dernier mois de l'exercice précédent, case propre, hors des
     colonnes) et Clôture = juin 2025 / mars 2024 — au serveur (`GET` du tableau,
     `unites_utiles` de l'AC-4) **et** en Vitest (en-têtes, liaison des colonnes) ; deux exercices consécutifs au
     1er juillet en genre `month` : la Clôture du premier (juin 2025) et l'Ouverture du second sont la même case.
     **Début en cours de mois (S4, T4)** : exercice 15.03.2024–14.03.2025 — treize colonnes, Ouverture = 14.03.2024
     (genre `day`) ou **mars 2024 liée à la première colonne** (genre `month`), serveur et Vitest. **Durées autres que
     douze mois (S4, CR-3)** : exercice de **six mois** (01.07.2024–31.12.2024 : six colonnes,
     juillet → décembre, Ouverture = 30.06.2024 ou juin 2024, Clôture = 31.12.2024 ou décembre 2024) et de **dix-huit
     mois** (01.07.2023–31.12.2024 : dix-huit colonnes, juillet 2023 → décembre 2024, Ouverture = 30.06.2023 ou **juin
     2023**), au serveur et en Vitest (en-têtes, défilement du seul tableau).
   - Vitest : ligne `afc` égale → « AFC ✓ » sans ↑ ; ligne `afc` corrigée → « AFC corrigé : … » en rouge et ↑ direct
     (sans dialogue, A23) ; ligne `afc` sans valeur publiée → sans ↑ ; ligne `manual` différente → « AFC : … » et ↑
     direct ; « AFC : non publié » et ↑ « Retirer le cours saisi » ; « AFC ✓ · saisi » et ↑ (A2) ; ligne `ecb` différente → « AFC : … (ecb) » et ↑ (A30) ; case vide à valeur
     publiée → ↑ ; valeur entrée égale à la valeur publiée → `reset` (ou rien si déjà `afc`) et jamais `PUT` ; ↑ →
     `reset` avec la bonne version et la valeur affichée ; 409 → rechargement ; changement d'un genre → `PUT
     /rate-kinds` puis relecture ; Consultation sans champ éditable ni choix ; cours JPY saisi par unité ; « Récupérer
     les cours manquants » poursuit après un échec et n'appelle qu'une fois par colonne (deux devises sans valeur
     publiée le même mois → un seul `POST /fetch`). Captures d'écran (exercice au 1er juillet avec un cours saisi et
     son ↑, une case « AFC corrigé » en rouge et son ↑ direct (le dialogue U3 n'est pas rendu, A23), le dialogue du genre de clôture, colonnes Ouverture et
     Clôture en genre `day` puis `month`, lecture seule).
7. **AC-7 — Restauration.** Les deux tables `_kesh_*` sont système : un import de sauvegarde ne les touche pas. Sur une
   installation neuve restaurée, elles sont vides : la boucle ou `POST /fetch` les complète ; aucun test ne suppose une
   valeur publiée présente sans l'avoir posée. **`exchange_rates`, applicative, est sauvegardée et restaurée** avec la
   société (U1, CR-5 — il n'y a plus de « cours perdus » à la restauration) : après une restauration, les cours retenus
   **et leur source** (`afc` ou `manual`) sont là et sont les cours effectifs, même avant toute récupération ; le
   contrôle `FX_RATE_NOT_AFC` de 37-3a s'appuie sur eux dès l'import ; les cases `afc` disent « valeur publiée non
   relue » jusqu'à la récupération suivante, qui peut révéler une correction survenue entre-temps (annoncée, AC-5).
   Test d'aller-retour : lignes `afc` et `manual` à 10 décimales, `source`, `version`, `updated_by` (`NULL` compris),
   `acknowledged_*` identiques ; après import sur une installation neuve, `lookup` rend 200 sans aucun appel sortant.
   Un `.keshbackup` antérieur à cette story **se réimporte** (R1, AC-1) : sans la table au manifeste, elle reste vide
   (test : sauvegarde simulée 0.17.6 de 37-1 AC-6, aucun cours retenu après l'import, cours d'avant l'import effacés
   comme toute donnée de la société remplacée). ⚠️ **Isolation des E2E** : système, elles échappent à `truncate_all`
   (`TABLES_TO_TRUNCATE`, `backup.rs:46`) que lancent `POST /_test/seed` et `/_test/reset`
   (`test_endpoints.rs:189`, `:325`) ; sans plus, une valeur publiée posée par une spec survit à la suivante, qui
   attendrait un `404 RATE_UNAVAILABLE` après sa propre récupération. (`exchange_rates` est dans `TABLES_TO_TRUNCATE` :
   vidée sans autre geste.) Ces
   deux routes vident donc aussi `_kesh_fetched_exchange_rates` et
   `_kesh_exchange_rate_fetch`, et `clear_transient_test_state` (`test_endpoints.rs:349`) vide `rate_fetch_memo` et
   remet la source factice à son défaut. Test Rust de ce vidage. Le commentaire de `test_fixtures.rs` (:365-373), qui liste
   les tables système hors `TABLES_TO_TRUNCATE`, gagne les deux tables neuves. La remise à zéro des données de
   démonstration (`kesh_seed::reset_demo`, `crates/kesh-seed/src/lib.rs:510`, audit `installation.reset`) ne vide
   que les tables applicatives : elle **laisse** les valeurs publiées, données officielles de l'installation, et
   **vide** `exchange_rates`, cours retenus de la société (copies comprises : elles se refont quand un compte
   reçoit une devise, AC-3 bis ; la réinitialisation remet aussi la société en mono-devise, 37-1 AC-5) — voulu, à dire au manuel. Aucun outil du serveur MCP ne lit la table des cours (une ligne porte son cours, 37-3a) : hors périmètre.
8. **AC-8 — i18n, manuels.** Quatre locales (écran, codes d'erreur dont `FX_RATE_INVALID`, libellés d'audit et de
   type d'entité) ; manuel utilisateur (tableau annuel, source, cours mensuel AFC — publié d'avance pour le mois ; la
   définition « moyenne du 25 au 24, publiée le 25 » de R4 n'est écrite qu'une fois confirmée par Beat (relevé
   contraire, *Source*) —, cours journalier, genres d'ouverture et de clôture de l'exercice (le cours d'ouverture est celui de la clôture de l'exercice
   précédent, T4 : pourquoi le genre « mois » d'ouverture prend le dernier mois de l'exercice précédent, et le
   genre d'ouverture repris par défaut à la création, et le dialogue qui propose de l'appliquer à l'exercice suivant
   quand le genre de clôture change, sans écriture modifiée et pourquoi, U4), **cours retenu par la société** et sa
   source — copie automatique des cours publiés, cours saisi, ↑ (U1) —,
   récupération automatique et son calendrier — quotidien jusqu'au cours du mois suivant, puis pause jusqu'au plus tôt de « obtention + 24 jours » et du 25 du mois obtenu, jamais avant le 1er de ce mois
   (S1) —, tableau d'un exercice de durée quelconque (S4), **correction d'un cours par l'AFC** :
   la société garde son cours, « AFC corrigé » en rouge, ↑ direct (cette société seule, U3, A23), avertissement à
   acquitter par société, effet sur les lignes `afc` passées (U1, A17) ; restauration : les cours retenus et leur
   source reviennent avec la sauvegarde) ;
   manuel d'administration : accès sortant HTTPS vers
   `www.backend-rates.bazg.admin.ch` requis pour un exercice multi-devises ouvert, aucun sinon, quand il a lieu
   (démarrage, une fois par jour tant que le mois courant ou le mois suivant manque, puis pause, S1 ; connexion ;
   chaque tour revérifie aussi le mois courant, un appel de plus, A10), mandataire par `HTTPS_PROXY` (lu par `reqwest`) ; un `.keshbackup` 0.17.6
   ou antérieur **se réimporte** (mono-devise, R1) ; phrase du registre des routes (AC-5, chiffres recomptés depuis la source après chaque reprise de `main`) ; `.tex` seulement, PDF non commités (régénérés à la release).
9. **AC-9 — Cours de la société et cours d'une ligne** (amendé par D9, puis par U1, CR-5). Deux niveaux, sans
   recouvrement :
   - **cours retenu par la société** pour (devise, mois) ou (devise, jour) : table applicative `exchange_rates`, une
     seule valeur et sa `source` — `afc` (copie d'une valeur publiée, ou valeur reprise par ↑) ou `manual` (cours
     saisi) ; jamais `_kesh_fetched_exchange_rates` (qui ne porte que les valeurs publiées, pour la copie et
     l'information) ; la saisie et la reprise sont auditées, la copie ne l'est pas (C-37-6) ; il devient le cours
     **proposé** ;
   - **cours d'une ligne** : il vit **sur la ligne d'écriture** (37-3a), audité avec l'écriture. Une ligne au cours
     retenu est `afc` si ce cours égale celui d'une ligne `exchange_rates` **`source = 'afc'`** du mois (`effectiveRate`
     et `source` de `lookup`), `manual` sinon (cours saisi par la société, cours retouché, ou aucun cours `afc`) —
     c'est le client qui la qualifie (37-3b) ; le contrôle `FX_RATE_NOT_AFC` de 37-3a compare à cette même ligne de la
     société, jamais à la valeur publiée (une correction de l'AFC non reprise ne change rien). Aucune nouvelle valeur
     de `rate_source`.

## Tâches

- [ ] Migration + inscriptions (AC-1), dont `exchange_rates` dans `TABLES_TO_TRUNCATE`, `TABLES_EXPORTEES`
      (ou exclusion motivée) ; table inscrite à la liste close des absences admises de 37-1 (R1 : fenêtre d'import
      inchangée, registre de rejeu et `admin_full_import_e2e.rs` non touchés, test de 37-1 AC-6 étendu à la table) ;
      `_kesh_fetched_exchange_rates` et `_kesh_exchange_rate_fetch` à `TABLES_SYSTEME`.
- [ ] `kesh-core::exchange_rates` (AC-2 : deux analyses, une seule analyse des `<devise>`, bornes d'un cours seul) ;
      `kesh-db::repositories::exchange_rates` (écriture des valeurs publiées en transaction, copie, lecture, état,
      cours retenus). **Noms fixés ici, repris tels quels par 37-3a et 37-5** (U1, CR-5) : la lecture du **cours
      retenu par la société** — `find_company_rate_in_tx(conn: &mut MySqlConnection, company_id: i64, currency: &str,
      kind: RateKind, rate_date: NaiveDate) -> Result<Option<CompanyRate>, DbError>` (`None` = aucune ligne ;
      `CompanyRate` porte `rate` et `source`), **seule** lecture de cours des contrôles et des propositions (`lookup`,
      les `GET`, 37-3a, 37-3b par `lookup`, 37-5) ; la lecture d'une **valeur publiée** —
      `find_fetched_rate_in_tx(conn, source, currency, kind, rate_date)` (ex-`find_rate_in_tx`), pour l'affichage et
      la copie seulement, jamais pour un contrôle. Toutes deux prennent une **connexion ou une transaction** (`&mut
      MySqlConnection`, comme les fonctions `*_in_tx` du dépôt) : 37-3a (AC-4, 4) lit `find_company_rate_in_tx(…,
      Month, 1er du mois)` dans la transaction de l'écriture ; 37-5 lit les cours **avant** sa transaction, sur une
      connexion du pool (37-5 AC-2 : aucune lecture de cours ni récupération sous le verrou de l'exercice) — aucun
      second accès. Écritures : `copy_fetched_in_tx` (AC-3 bis), `set_manual_rate` (`PUT`), `reset_rate` (↑),
      `acknowledge_corrections` — en transaction avec leur audit, version contrôlée. Une récupération d'un mois ou
      d'un jour déjà présent qui rend une **autre valeur** remplace la valeur publiée (règle (c) de l'AC-3), la
      journalise (`tracing::warn!`) et la compte (`corrected`) ; elle ne touche aucune ligne de `exchange_rates` — les
      lignes d'écriture `afc` déjà passées gardent leur cours ; inchangées, leur `PUT` passe, mais une ligne `afc`
      nouvelle ou modifiée à un autre cours que le cours retenu est refusée `FX_RATE_NOT_AFC` (37-3a, A17) : l'écart
      n'est jamais muet (rouge, avertissement par société, audit du ↑).
- [ ] `kesh-api` : trait `RateSource`, `HttpRateSource`, source factice et endpoint `_test` (AC-3), boucle (AC-4,
      `main.rs`), routes (AC-5, `lib.rs` : `GET` dans `authenticated_routes` ; `POST /fetch`, `PUT /exchange-rates/rates`,
      `POST /exchange-rates/rates/reset`, `GET /exchange-rates/notices`, `POST /exchange-rates/notices/acknowledge`,
      `PUT /fiscal-years/{id}/rate-kinds` dans `comptable_routes`), variantes `AppError` (dont `FX_RATE_INVALID`), audit
      (`audit_labels.rs` : six actions — `exchange_rates.fetched`, `exchange_rate.created`, `exchange_rate.updated`,
      `exchange_rate.reset`, `exchange_rates.corrections_acknowledged`, `fiscal_year.rate_kinds_updated` — et le type
      d'entité `exchange_rate`), registre des routes (AC-5 : +5 routes, total +6, point (vi) +5) et phrase du manuel ;
      appel de la copie (AC-3 bis) dans `fiscal_years::create`,
      `accounts::create` et `accounts::update` de 37-1, avec `copied` à leur audit (aucun dans
      `companies::update_currency_settings`, vide par construction) ; copie limitée aux devises des comptes de la
      société (A24), paramètre `currencies`, `hasAccount` des lignes du tableau.
- [ ] CR-2, CR-3 : tâche automatique au calendrier (AC-4 : `unites_utiles` — exercices de durée quelconque, S4 —,
      `prochain_reveil` — quotidien, puis pause jusqu'au plus tôt de « obtention + 24 jours » et du 25 du mois obtenu, jamais avant le 1er de ce mois, S1 —, démarrage, connexion,
      revérification du mois courant à chaque tour, A10), filtrage strict et tests hostiles (AC-2, R5), routes `GET /fiscal-years/{id}/exchange-rates` et
      `PUT /fiscal-years/{id}/rate-kinds` (AC-5 ; `GET` dans `authenticated_routes`, `PUT` dans `comptable_routes`),
      audit `fiscal_year.rate_kinds_updated`, registre des routes (+1 de plus).
- [ ] CR-3 (S6), **récrite par CR-5 (U1, U3, U4)** : colonne `first_fetched_at` (et `value_changed_at`) de
      `_kesh_fetched_exchange_rates` (AC-1) ; règle d'écriture de la valeur publiée (AC-3) ; copie (AC-3 bis) ; colonnes
      `source`, `acknowledged_rate`, `acknowledged_at` d'`exchange_rates` et leurs `CHECK` (AC-1) ; routes `GET
      /exchange-rates/notices`, `POST /exchange-rates/notices/acknowledge` (par société) et `POST
      /exchange-rates/rates/reset` (↑) ; `applyToNextOpening` de `PUT /rate-kinds` (U4) ; `Cell` à une valeur
      (`corrected`, `acknowledged`, `published`) ; frontend : mentions et ↑ de la case, « AFC corrigé » en rouge, dialogue
      U3 (spécifié, **non rendu** tant que l'option « toutes les sociétés » est masquée : ↑ direct, A23), dialogue U4, bouton « Revérifier les cours publiés », bandeau de
      connexion et son acquittement (AC-6) ; tests (dépôt, API, Vitest, E2E). Les colonnes `republished_*`,
      `notice_acknowledged_at`, la route `…/republications/adopt` et l'action `exchange_rates.republication_adopted` de
      CR-3 ne se font **pas**.
- [ ] Frontend : page `settings/exchange-rates` (tableau annuel par exercice, choix des genres, champs éditables, « AFC ✓ » / « AFC : … » et ↑), carte
      des paramètres, types, API (fonctions `lookup`, `fetch`, `putRate`, `resetRate`, `notices`, `acknowledge` ; la
      forme de `lookup` et de `Cell` est celle de l'AC-5), codes d'erreur, Vitest (AC-6).
- [ ] Tests : parseur (fixture : unités 1/100/1000/10000, devise écartée, mois refusé) ; client contre un serveur HTTP
      local (délai, 500, redirection non suivie, XML tronqué, corps trop gros, mois différent → rien d'écrit) ;
      `un_tour` sans exercice multi-devises ouvert : aucun appel sortant (R2) ; `un_tour` contre une source injoignable rend sans paniquer, écrit `error` et
      `triggered_by = 'auto'`, ne touche aucun cours ; `lookup` n'écrit rien et ne sort pas ; `POST /fetch` refusé au rôle
      Consultation, audité, `triggered_by = 'manual'`, 409 `MULTI_CURRENCY_DISABLED` en mono-devise (les quatre routes),
      400 hors bornes de mois, mémoire de 15 minutes (un mois `not_published` n'est pas redemandé à la source : un
      seul appel sortant, aucune ligne d'audit, une seule écriture de `_kesh_exchange_rate_fetch`), `malformed` / `month_mismatch` / `no_valid_rate` → 503 sans rien écrire, `lookup` `CHF` → 400,
      `aujourdhui_zurich` aux bords du mois (31 → 1er), devise aux bornes hors `DECIMAL` écartée sans annuler les autres,
      `last_status` d'un tour à plusieurs unités (le pire) ; aucune trace d'audit en automatique ; deux `POST /fetch` simultanés
      sérialisés (le second attend la fin du premier, AC-3 ; un seul appel sortant à la fois) ; `POST /_test/seed` et `/reset` vident les deux tables et
      la mémoire (AC-7) ; source factice : XML par défaut au mois demandé ; un `rate_per_unit` changé au refetch est
      compté (`corrected`) ; un `POST /fetch` qui a attendu le verrou derrière une récupération réussie du même mois ne rappelle pas
      la source ; un succès efface l'échec mémorisé du mois ; Vitest : un 503 de `POST /fetch` n'allume pas le bandeau dégradé ; E2E (écran, récupération servie par
      la source factice du mode test).
- [ ] Tests CR-1 (D9, D10) : parseur journalier (fixture, cinq décimales, `land_en` ignoré, racine vide) ; `fetch_day`
      contre le serveur local : `<datum>` antérieur de deux jours (dimanche) accepté et stocké sous la date demandée
      avec `source_date`, `<datum>` postérieur ou antérieur de plus de 7 jours → `date_mismatch`, rien d'écrit ;
      `POST /fetch { date }` : aujourd'hui et demain → 400, audité avec le genre ; la tâche automatique ne demande que
      les jours d'ouverture et de clôture des exercices multi-devises ouverts dont le genre est `day`, passés (AC-4) ;
      cours effectif (U1, CR-5) : ligne `manual` seule, ligne `afc` copiée, ligne `afc` dont la valeur publiée a été
      corrigée depuis (le cours retenu reste l'ancien), aucune ligne mais valeur publiée (404, puis 200 après `POST
      /fetch` qui copie), aucune (404, `details.date` pour `day`) ;
      `PUT …/rates` : création, remplacement d'une ligne `afc` (devient `manual`, `acknowledged_*` à `NULL`), version
      absente sur ligne existante et version périmée → 409, `rate` 0, négatif, 11 décimales, > 10⁶ → `FX_RATE_INVALID`,
      `rateDate` du 15 en `month` → 400, `CHF` → 400, Consultation → 403, mono-devise → 409, audit `created`/`updated`
      avec avant et après ; `POST …/rates/reset` : ligne `manual` → `afc` à la valeur publiée, ligne `afc` corrigée →
      nouvelle valeur, case vide → ligne `afc` créée, ligne `manual` sans valeur publiée → supprimée, ligne `afc` sans
      valeur publiée → 404, `publishedRate` périmé → 409, version périmée → 409, audit `exchange_rate.reset` ; une
      récupération ne modifie aucune ligne de `exchange_rates` (AC-3) ; société B ne voit pas le cours de A ;
      aller-retour `.keshbackup` des cours retenus (sources comprises) ; remise à zéro les vide et laisse les valeurs
      publiées ;
      E2E : saisir un cours, le voir proposé, ↑ le retire ; `xmldaily` 404 JSON (relevé du 2026-11-14) →
      `NotPublished`, `RATE_NOT_PUBLISHED`, mémorisé, rien d'écrit ; 404 de `xmlavgmonth`, 500 de l'une ou l'autre →
      `RATE_SOURCE_UNAVAILABLE` ;
      `<land_en/>` vide et `<gueltigkeit>` d'une seule date acceptés ; `POST /fetch { date: "2000-01-01" }` servi
      `<datum>31.12.1999</datum>` → 200, `sourceDate` 1999-12-31 ; `lookup` `kind` hors `month|day` → 400, forme `day`
      sans cours → 404 avec `details.date` ; `GET /exchange-rates` d'une devise à cours saisi seul (`published` `null`) ;
      cours saisi égal à la valeur publiée conservé (`source: "manual"`) ; deux `PUT` de création simultanés → un 200 et
      un 409 ; `reset` sur une ligne existante sans `version` → 409, avec `version` sur une case vide → 400 ;
      `registry_entries_are_within_import_window` vert avec la table créée (R1).
- [ ] CR-5 (U4) : `applyToNextOpening` de `PUT /fiscal-years/{id}/rate-kinds` (AC-5 : exercice suivant lu sous
      `FOR UPDATE` après l'exercice réglé, ouvert seulement, audit unique, aucune écriture modifiée) ; dialogue de
      l'écran (AC-6) ; tests.
- [ ] CR-4 (T4, T2) : fonction pure unique `openingDate`/`closingDate` dans `kesh-core` (genres `day` et `month`,
      début en cours de mois, bornes de l'AC-5), partagée par le `GET` du tableau et `unites_utiles` (AC-4, AC-5), et
      son jumeau TypeScript de 37-3b, gardés par la fixture commune `fx_opening_dates.json` (créée ici, lue par le
      test Rust et par le test Vitest de 37-3b) ; case d'Ouverture en genre `month` (propre, ou liée à la
      première colonne), libellé « cours mensuel du dernier mois de l'exercice précédent » et aide (AC-6, quatre
      locales) ; notices sur `openingDate` ; tests des exercices au 1er janvier, 1er avril, 1er juillet, 15.03, six et
      dix-huit mois (serveur et Vitest).
- [ ] Manuels (`.tex` seulement), section *Note de version* de la fiche (pas d'écriture au `CHANGELOG.md`), `Cargo.lock` (régénéré après chaque reprise de `main`).

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Added** — cours mensuels de l'AFC récupérés automatiquement pour les exercices multi-devises (accès sortant HTTPS, mandataire par `HTTPS_PROXY`) ; cours de la société et d'une ligne ; erreur `FX_RATE_INVALID`.
- **Sauvegarde** — un `.keshbackup` 0.17.6 ou antérieur se réimporte (mono-devise) ; cours saisis vides.
- **Compteurs** : recomptés depuis la source après chaque reprise de `main` (`LIB_ROUTES`, audit des migrations, `Cargo.lock`).

## Notes de développement

- Modèle de boucle : `crates/kesh-api/src/auto_backup.rs` (`spawn` :788, `un_tour` :803, `CADENCE` :58), lancement
  `main.rs:467`, garde `auto_backup::tests::lancement_par_main_seulement` (:1204, aide `code()` :1190).
- `reqwest` est déjà au `Cargo.lock` (0.12.28, dev-dependency de `kesh-api`, défauts + `json`, `multipart`,
  `cookies`) ; `rustls`
  0.23.46 (ring) y est aussi.
- Calendrier de la source (R4) : le cours du mois M est publié le 25 de M−1 (ou le jour ouvrable suivant), applicable
  dès le 1er de M ; le relevé du 2026-11-14 (sans paramètre, `2026-11`) s'y accorde pour le mois courant, mais
  `j=2026&m=12` était déjà publié le 14.11.2026 (validation CR-2 P2, *Source*) : date de publication à confirmer par
  Beat ; la tâche vérifie chaque jour tant que le mois courant ou le mois suivant manque, puis se met en pause (S1,
  CR-3 ; sûr dans les deux lectures), `POST /fetch` accepte le mois suivant à toute date ; chaque tour revérifie en
  plus le mois courant (A10, un appel de plus, sans réveil de plus).
- Pas de production des cours dans le dépôt : les fixtures sont synthétiques (C-37-1, conditions SIX).
- Cours journaliers relevés le 2026-11-14 (deux `GET`, `d=20241231` et `d=20240630`) : voir *Cours journaliers*.
- Fenêtre d'import : `parse_and_verify_avec_inventaire` (`admin_backup/import.rs:187`) compare le manifeste à
  `TABLES_TO_TRUNCATE` dans les deux sens ; précédent de fermeture : Story 32-5a (`mcp_keys`, commentaire de
  `POST_RESTORE_BACKFILLS`, `post_restore.rs:211-217`). Depuis CR-2 (R1), la liste close des absences admises de 37-1
  évite cette fermeture pour les tables de l'Epic 37.
- Pas d'audit automatique : `audit_log.user_id` est `NOT NULL` (C-37-6).

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 8 MEDIUM, 9 LOW ; remédiés ; alignement de 37-3b/37-5 sur
  POST /fetch et du tableau de l'epic portés par l'orchestrateur.
- 2026-11-14 — validation P2 (Sonnet) : 0 CRITICAL, 0 HIGH, 8 MEDIUM, 8 LOW ; remédiés dans la fiche sauf P2-14, P2-15
  et P2-16 (inter-fiches : 37-1 et 37-5, 37-3b, fichier d'epic).
- 2026-11-14 — validation P3 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 9 LOW (plus 2 LOW inter-fiches) ; remédiés dans la
  fiche sauf 37-2-P3-IF1 (37-3a, forme des `details` de `RATE_UNAVAILABLE`) et 37-2-P3-IF2 (37-3b, appel de
  `POST /fetch` par la fonction d'API de 37-2). Choix de conception de cette passe, à confirmer par la P4 : boucle
  non lancée en mode test (P3-1), verrou pris par mois (P3-3), option de `CallOptions` contre le bandeau dégradé
  (P3-2), audit de `POST /fetch` sur succès seulement (P3-13).
- 2026-11-14 — validation P4 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW (plus 3 LOW inter-fiches) ; remédiés dans la
  fiche sauf 37-2-P4-IF1 (37-3a : `DbError::RateUnavailable` créée par 37-2, nom `find_rate_in_tx`), 37-2-P4-IF2
  (37-3b : `timeoutMs` 90 s et non 60 s) et 37-2-P4-IF3 (37-1 : décomptes du registre des routes en deltas). Choix de
  cette passe, à confirmer : propriétaire unique de `RATE_UNAVAILABLE` = 37-2 (P4-1), `timeoutMs: 90_000` (P4-4),
  formes des réponses `GET` fixées (P4-7).
- 2026-11-14 — validation P5 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW ; remédiés dans la fiche sauf aucun
  (aucun finding inter-fiches). MEDIUM P5-1 : l'endpoint `_test` classé `SansEcritureAuJournal` et non `Exemptee`
  (`sans_ecriture` +2, `exemptees` inchangé). LOW : borne d'attente du verrou et récupération attendue non rejouée
  (P5-2), mémoire effacée sur succès (P5-3), `POST /fetch` dans un `tokio::spawn` (P5-4), DDL de
  `_kesh_exchange_rate_fetch` complété (P5-5), `reset_demo`, MCP et `from_current_user` nommés (P5-6), `CHF` → 400
  `Validation` et les deux `errors.rs` distingués (P5-7).
- 2026-11-14 — validation P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés dans la fiche. P6-1 :
  `fetched_at` de la récupération attendue défini (pris avant l'appel sortant) et le raccourci sans audit ni écriture
  d'état ; P6-2 : `from_current_user` nommé comme méthode du trait `AuditActor` ; P6-3 : le précédent
  `/password-reset-token` écrit un jeton en base, la fiche dit donc « rien au journal d'audit ». Inter-fiches, non
  appliqué : l'entrée P5 du Change Log de 37-1 (IF-1) parle encore de `exemptees` +1 (historique ; l'AC-5 de 37-2 fait foi).
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : cours journaliers OFDF (`xmldaily`, relevé en deux
  `GET` : espace de noms propre, cinq décimales, un jour sans publication rend la publication antérieure — `<datum>`
  lu, `date_mismatch` au-delà de 7 jours) ; `_kesh_exchange_rates` gagne `kind` et `rate_date` (PK `(currency, kind,
  rate_date)`) et `source_date` ; table applicative neuve `exchange_rate_overrides` (cours saisis, sauvegardée,
  vidée à la remise à zéro) qui **referme la fenêtre d'import** (registre de rejeu vidé, `20261106000001` hors
  fenêtre, sauvegarde 0.17.6 refusée) ; cours effectif = saisi ?? AFC (`find_effective_rate_in_tx`), `lookup` et
  `GET` rendent `effectiveRate`, `afcRate`, `overridden` (fin de `ratePerUnit`) ; `POST /fetch { date }` ; `PUT` et
  `DELETE /exchange-rates/overrides` (auditées, version, `FX_RATE_INVALID` et bornes d'un cours seul créés ici) ;
  registre des routes +3 routes / +4 au total ; écran à cours éditables, « AFC : … » et ↑, vue jour ; AC touchés :
  Story, Source, AC-1, AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, Tâches, Notes ; « cours moyens » retiré.
- 2026-11-14 — validation CR-1 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 4 LOW ; remédiés dans la fiche (aucun
  finding inter-fiches). MEDIUM : `xmldaily` rend un **404 JSON** sans publication (relevé `d=20301231`), non une racine
  vide — traité `NotPublished` (CR1-P1-1) ; le test `full_import_report_mirrors_the_production_registry`
  (`admin_full_import_e2e.rs:1595`) rougit avec le registre vide, absent des conséquences de la fermeture de fenêtre
  (CR1-P1-2) ; la phase 5 de `recette-montee.sh` exige un 200 (:651) et ne peut « attendre un refus » : omise à cette
  release, script inchangé (CR1-P1-3) ; deux créations simultanées d'un cours saisi → 409 et non 500 (CR1-P1-4). LOW :
  forme `day` de `RATE_UNAVAILABLE` sans date dans `DbError` (CR1-P1-5), `fetchedAt` `null` d'un cours saisi seul
  (CR1-P1-6), champ vidé, bouton jour désactivé, date par défaut sans exercice (CR1-P1-7), `version` du `DELETE`
  obligatoire (CR1-P1-8). Non appliqué, à trancher par Beat : un cours saisi **égal** au cours AFC est marqué « saisi »
  sans ↑ (AC-6) et ne peut donc plus être retiré depuis l'écran ni suivre une republication de l'AFC — D9 dit « un
  bouton ↑ qui reprend le cours AFC » ; proposition : ↑ pour tout cours saisi existant. Axes exercés : citations de code
  (backup.rs, post_restore.rs, import.rs, audit_labels.rs, audit_route_registry.rs, mail/mod.rs, lib.rs,
  auto_backup.rs, api-client.ts, Cargo.lock, Dockerfile, exports/global.rs, recette-montee.sh), sites de D9/D10
  (sauvegarde, restauration, fenêtre, remise à zéro, export CSV, audit, RBAC, i18n, manuels, MCP), source journalière
  (3 GET : `20000103`, `20301231`, `20000101`), provenance `afc`/`manual`, tests. Non exercés : compilation, gate,
  rendu de l'écran.
- 2026-11-14 — validation CR-1 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; remédiés dans la fiche sauf aucun
  (aucun finding inter-fiches). MEDIUM : `full_import_of_a_0_17_5_backup_keeps_the_numbers` (`admin_full_import_e2e.rs`
  :5638, appel d'`entree_numero_interne` :5669, `.expect` qui panique sur le registre vide) rougit aussi, absent des
  conséquences de la fermeture de fenêtre : les deux tests passent à `replay_retired` (CR1-P2-1) ; arbitrage de
  l'orchestrateur : ↑ pour **tout** cours saisi existant, égal au cours AFC ou sans cours AFC compris (AC-6, Vitest)
  (CR1-P2-2) ; arbitrage : la fonction des bornes d'un cours seul rend une erreur de type propre (`RateBoundsError`),
  traduite par chaque appelant — aligne l'AC-2 sur 37-3a AC-3 (CR1-P2-3). LOW : le 404 de `xmldaily` passe par une
  variante propre de `SourceError`, un 404 de `xmlavgmonth` reste `http_status:404` (CR1-P2-4) ; « autre 404 » du test
  précisé (CR1-P2-5). Axes exercés : diff de la révision et de la remédiation P1 ; citations relues au code
  (`admin_full_import_e2e.rs` :954, :972, :1561-1600, :5475, :5551, :5563, :5638 ; `b723e3d9` :1580-1600 ;
  `post_restore.rs` :205-232, :271, :1135-1145, :1210-1230, boucles des tests sur le registre ; `post_restore_class_a.rs`
  :75-100 ; `recette-montee.sh` :640-660 ; `api-client.ts` :42, :61, :388-425 ; `kesh-db/src/errors.rs` :675-679, :1721 ;
  `backup.rs` :46, :137-160) ; points de contact 37-3a AC-3 et 37-5 AC-4 (bornes) et 37-3b (↑ de la ligne, distinct) ;
  source journalière, un GET : `d=20240401` (lundi de Pâques) rend `<datum>28.03.2024` (jeudi saint, quatre jours :
  la borne de 7 jours tient sur un pont de Pâques). Non exercés : compilation, gate, rendu de l'écran, i18n et manuels
  au-delà de la fiche (rien de neuf depuis la P1).
- 2026-11-14 — validation CR-1 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche (aucun
  finding inter-fiches). LOW : phrase des bornes d'un cours seul redondante après l'ajout de CR1-P2-3 (« Elle sert ici au
  `PUT` … » répétait « le `PUT` … le parseur ») fusionnée (CR1-P3-1). Axes exercés : diff 796ae5a9..272a5289 de la fiche ;
  citations relues (`admin_full_import_e2e.rs` :972, :5551, :5563, :5619, :5638, :5669) ; cohérence avec D9 (epic), 37-3a
  AC-3 (`RateBoundsError`, `FX_VALUE_OUT_OF_RANGE { line }`), 37-5 AC-4 (`FX_RATE_INVALID`) et AC-6 (↑ propre à 37-2),
  37-3b (↑ de la ligne, distinct), `last_error` `http_status:<n>`, tests de l'AC-3. Non exercés : compilation, gate, rendu
  de l'écran, rien hors du diff.
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R1** — la fermeture de la fenêtre d'import écrite par
  CR-1 est **défaite** : `exchange_rate_overrides` reste applicative et sauvegardée, mais s'inscrit à la liste close
  des absences admises de 37-1 (AC-6) — une sauvegarde 0.17.6 se réimporte, cours saisis vides ; registre de rejeu
  (`20261106000001`), compteurs de `post_restore.rs`, `admin_full_import_e2e.rs` (`full_import_report_mirrors_…`,
  `entree_numero_interne`, tests 0.17.4/0.17.5) et phase 5 de `recette-montee.sh` **non touchés** ; manuel et CHANGELOG
  ne disent plus le refus. **R2** — récupération automatique limitée aux exercices multi-devises ouverts (mois et jours
  d'ouverture/clôture de genre `day`). **R4** — calendrier : démarrage, à partir du 25 toutes les 6 h jusqu'au mois
  suivant puis sommeil jusqu'au 25 suivant, connexion (tâche détachée, 15 minutes) ; plus de boucle permanente de 6 h ;
  borne du `POST /fetch` portée au mois suivant à partir du 25 ; terminologie « moyenne du 25 au 24, publiée
  d'avance ». **R5** — filtrage strict (taille, structure, profondeur, DOCTYPE et entités refusés, grammaires des
  nombres, codes ISO de 37-1, mois et dates) et tests de réponses hostiles. **R6** — écran en tableau annuel par
  exercice (mois dans l'ordre de l'exercice, Ouverture, Clôture, « AFC ✓ » / « AFC : … » et ↑, l'écran ne crée jamais
  de cours saisi égal à l'AFC), routes `GET /fiscal-years/{id}/exchange-rates` et `PUT /fiscal-years/{id}/rate-kinds` ;
  **décision** : colonnes `opening_rate_kind`/`closing_rate_kind` dans la migration de **37-1** (AC-1 de 37-1) ;
  tests d'exercices au 1er juillet et au 1er avril. Registre des routes : +4 routes, total +5. Prémisses relues au
  code : `import.rs:250-253`, `backup.rs:796-803`, `post_restore.rs:939`, `:988`, `routes/auth.rs:235`,
  `kesh-import/src/camt053/mod.rs:244`, `:375` (`Event::GeneralRef` de quick-xml 0.42, `Cargo.lock`),
  `audit_route_registry.rs:537-539`. AC touchés : en-tête, Story, Source (calendrier, terminologie), AC-1, AC-2, AC-3,
  AC-4 (réécrit), AC-5, AC-6 (réécrit), AC-7, AC-8, Tâches, Notes. Fiche à revalider (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche (aucun finding
  inter-fiches). MEDIUM : champ d'`AppState` de l'horodatage du déclenchement à la connexion absent de la liste des
  champs neufs (CR2-P1-1, `rate_login_trigger`) ; deltas du registre des routes encore écrits « +3 / +4 » dans le second
  test (`sans_ecriture`, `tout.len()`) alors que CR-2 donne +5 : valeurs alignées (CR2-P1-2) ; ordre des refus du
  `PUT /rate-kinds` contraire à l'ordre général de l'AC-5 (400 avant 409) : aligné (CR2-P1-3). LOW : `<zeit>` n'est pas
  « vers 04:15 » (07:00:05 le 13.11.2026) et une date future n'est jamais refusée par la source (relevé : `d=20261114` et
  `d=20261115` rendent `<datum>13.11.2026</datum>`), d'où la garde « strictement antérieur » (CR2-P1-4) ; exercice clos sur
  l'écran, récupération offerte, champs et genres en lecture seule (A7, CR2-P1-5) ; jour de clôture du 31.12 non
  récupéré par la tâche avant le prochain démarrage, connexion ou 25 (CR2-P1-6). Axes exercés : citations relues au code
  (`import.rs:250-253`, `backup.rs:46`, `:92-99`, `:137-146`, `:796-803`, `post_restore.rs:988`, `auth.rs:235`,
  `auto_backup.rs:788`, `main.rs:467`, `audit_labels.rs` ordre des entités, `audit_route_registry.rs:537-539`,
  `Cargo.lock` quick-xml 0.42.0, `FISCAL_YEAR_CLOSED` 400, `fiscal_years.status` `Open`), accord avec 37-1 (AC-1, AC-3 bis,
  AC-6, nom de migration) et 37-5, R1–R10 et A1–A8, source journalière (2 GET : `20261114`, `20261115`). Non exercés :
  compilation, gate, rendu de l'écran, `resolve_xml_entity` de quick-xml 0.42 (sources absentes du poste), 37-3a/37-3b
  au-delà des renvois, i18n et manuels au-delà de la fiche. Cycle CR-2 : passe suivante à lancer.
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  37-2-CR2-P2-IF1 (fichier d'epic R4 et 37-3b/37-5 : borne « mois suivant à toute date ») et 37-2-CR2-P2-IF2 (37-1 AC-3
  bis : rejet du corps par l'extracteur avant le 404). MEDIUM : R4 contredit par un relevé — `xmlavgmonth?j=2026&m=12`
  rend un mois complet dès le 14.11.2026, `j=2027&m=1` la racine vide : la borne du `POST /fetch` passe au mois suivant
  à toute date, la tâche garde le 25, la définition « du 25 au 24, publiée le 25 » est à confirmer par Beat avant d'aller
  au manuel (CR2-P2-1) ; ordre des refus du `PUT /rate-kinds` posé par CR2-P1-3 (404 → 409 → 400 corps) irréalisable
  avec l'extracteur de corps (`extractors.rs:18`, rejet avant le handler) et attribué à tort à « l'ordre général » :
  corps (400) → 404 → 409 → `FISCAL_YEAR_CLOSED`, écart des routes à corps JSON écrit à l'ordre général (CR2-P2-2) ;
  « Récupérer les cours manquants » appelait `POST /fetch` par case — un appel et une ligne d'audit par devise pour un
  même mois — et ignorait les cases à cours saisi sans cours AFC : un appel par colonne (CR2-P2-3). LOW : la tâche
  dormait du 1er au 24 avec le mois courant manquant après une fenêtre du 25 en échec — condition (b) sur le dernier
  mois publié (CR2-P2-4) ; lecture seule de l'exercice clos propre à l'écran, case du jour de clôture partagée avec
  l'ouverture de l'exercice suivant (CR2-P2-5) ; le contrôle « unité manquante » de la connexion passe dans la tâche
  détachée (CR2-P2-6). Axes exercés : diff `b36599d0..HEAD` (entrées CR-2 P1) ; citations relues au code
  (`api-client.ts` :42, :61, :388-423 ; `extractors.rs` :1-60 ; `routes/fiscal_years.rs` :286-330 ; `errors.rs`
  `FiscalYearClosed` ; `backup.rs` :46-118, :785-806 ; `import.rs` :245-256 ; `post_restore.rs` :936-995 ;
  `audit_labels.rs` entités et actions `fiscal_year.*` ; `audit_route_registry.rs` :530-545 ; `exports/global.rs` :116,
  :673 ; `test_endpoints.rs` :189, :325, :349 ; `admin-manual.tex` :3217 ; `auth.rs` :235) ; accord avec 37-1 (AC-1,
  AC-3 bis, AC-6), 37-3b et 37-5 (borne du mois, `closingRateKind`) ; verrous et mémoire, calendrier, exercice clos (A7),
  R1–R6 ; source, trois GET (`xmlavgmonth` `j=2026&m=12` et `j=2027&m=1`, en-tête `Date`). Non exercés : compilation,
  gate, rendu de l'écran, `resolve_xml_entity` de quick-xml 0.42, i18n et manuels au-delà de la fiche ; `FX_MULTIPLE_CURRENCIES`,
  assistant *Conversion* et archivage sous `force` sans point de contact dans cette fiche hors du tableau (lignes des
  comptes archivés). Cycle CR-2 : passe suivante à lancer (des MEDIUM ont été remédiés).
- 2026-11-14 — révision R4 amendé (orchestrateur, avant CR-2 P3) : tâche automatique alignée sur « une vérification par
  jour dès le 1er du mois tant que le mois courant ou le mois suivant manque, puis plus rien jusqu'au mois suivant »
  (AC-4 : unités utiles au mois suivant, calendrier, `prochain_reveil(now, manque_mois_utile)`, tests ; Source, AC-8,
  Tâches, Notes). Plus de fenêtre « à partir du 25 » ni de cadence de 6 h. À signaler hors fiche : 37-3b (l. 188-193,
  549) parle encore du « dernier mois publié » et du 25.
- 2026-11-14 — validation CR-2 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; deux remédiés, un signalé.
  37-2-CR2-P4-1 (AC-4, parenthèse du jour de clôture) : « au plus tard le 1er du mois suivant » reprenait l'ancien
  « au plus tard le 25 », alors que le réveil (b) du 1er janvier à 00:00 récupère déjà le 31.12 (strictement antérieur,
  « chaque tour traite toutes les unités ») : précisé. 37-2-CR2-P4-2 (AC-4, journal) : le mois suivant étant vérifié
  chaque jour dès le 1er, son `not_published` devient l'état ordinaire de la plus grande partie du mois — un
  `tracing::warn!` quotidien serait du bruit : `info!` pour ce cas. 37-2-CR2-P4-3 (non appliqué, à arbitrer) : le mois
  suivant étant désormais une unité utile dès le 1er, chaque connexion espacée de plus de 15 minutes lance un tour qui
  le redemande à la source et réécrit `not_published` dans `_kesh_exchange_rate_fetch` — conforme à R4 amendé
  (« plus le démarrage et la connexion »), mais en deçà de son intention « une vérification par jour » ; option :
  exclure le mois suivant du jugement de la connexion. Axes exercés : diff `cb15f2cd..dab5975d` de la fiche, entrée
  « révision R4 amendé » ; R4 et R4 amendé du fichier d'epic ; cohérence interne (Source, AC-4 unités utiles,
  calendrier, `prochain_reveil`, connexion, fuseau, AC-5 bornes, AC-6 bouton, AC-8, Tâches, tests, Notes) ; points de
  contact 37-3b (l. 188-195 : borne « mois suivant à toute date », tâche quotidienne — alignés) et 37-5 (l. 62, 359) ;
  citations relues (`auto_backup.rs:788`, `auth.rs:235`, `chrono-tz` au `Cargo.lock`) ; résidus « 25 », « 6 h »,
  « dernier mois publié » grepés (restent seulement la source ESTV, l'historique du Change Log et le titre d'AC-4,
  légitimes). Non exercés : compilation, gate, reste de la fiche hors remédiation, 37-1 et 37-3a (sans point de contact
  avec R4 amendé). Cycle CR-2 : aucune passe de plus exigée pour 37-2 (rien au-dessus de LOW) ; remédiation hors code.
- 2026-11-14 — arbitrage de l'orchestrateur sur CR2-P4-3 : le déclenchement à la connexion ne compte pas le mois
  suivant parmi les unités manquantes ; seule la tâche quotidienne le redemande. Test : connexion avec seul le mois
  suivant manquant → aucun tour. Relu par une passe ciblée.
- 2026-11-14 — validation CR-2 P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW remédiés, 1 signalé. 37-2-CR2-P5-1 (AC-4, parenthèse du jour de clôture) : « la prochaine connexion — qui lance un tour pour toute unité manquante » contredisait l'arbitrage (le mois suivant seul ne lance rien) : précisé. 37-2-CR2-P5-2 (AC-4, connexion) : le test de l'arbitrage (connexion, seul le mois suivant manquant → aucun tour) n'était qu'au Change Log : porté à l'AC ; la formule « que seule la tâche quotidienne redemande » est reformulée (un tour lancé par une autre unité manquante traite aussi le mois suivant, « toutes les unités utiles »). 37-2-CR2-P5-3 (signalé, hors fiche) : le fichier d'epic, R4 amendé, dit « plus le démarrage et la connexion » sans l'exception du mois suivant ; 37-3b et 37-5 ne parlent pas de la connexion (aucun écart). Cohérence vérifiée avec `prochain_reveil`, la mémoire des 15 minutes, l'`info!` du `not_published`. Axes non exercés : compilation, reste de la fiche hors remédiation.
- 2026-11-14 — validation CR-3 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW ; remédiés dans la fiche (aucun
  finding inter-fiches). 37-2-CR3-P1-1 (MEDIUM) : A10 (revérification du mois courant par chaque tour) contredisait
  « la tâche ne redemande jamais une unité présente » (AC-4, AC-6 bouton « Revérifier », Notes) : réécrits. 37-2-CR3-P1-2
  (MEDIUM) : A10 appliqué à l'AC-4 avec ses bords — ajoutée aux unités manquantes hors plafond de 36, sans déclencher
  de tour ni de réveil (`prochain_reveil` inchangé, rien ne manque → aucun appel), sous `rate_fetch_lock` et la mémoire
  de 15 minutes, comptée dans le pire statut, valable pour la connexion aussi, borne du trafic écrite ; tests de
  `un_tour` ajoutés, manuel d'administration (AC-8) et Tâches alignés. 37-2-CR3-P1-3 (LOW) : règle d'écriture (b) de
  l'AC-3 muette sur `source_date` d'un jour dont le `<datum>` change à valeur égale : gardée. Axes exercés : diff
  `294889cc..b6f88c24` de la fiche ; citations relues (`backup.rs:46`, `exports/global.rs`, `audit_labels.rs:24`,
  `auto_backup.rs:58,788,803`, `main.rs:467`, `+layout.svelte:373`, `chk_fiscal_years_dates`, `chrono-tz` absent des
  `Cargo.toml`) ; cohérence S1/S4/S6 (colonnes, `CHECK`, adoption, acquittement, deltas du registre : 6 routes
  mutantes + `_test` = 7) ; sites S1–S6 de la fiche (sauvegarde, `TABLES_TO_TRUNCATE`, export, audit, RBAC, i18n,
  manuels, MCP). Non exercés : source journalière (aucun GET), compilation, gate, rendu de l'écran, 37-1/37-3a/37-3b/37-5
  au-delà des renvois. Cycle CR-3 : A10 est une révision nouvelle, à relire par la passe suivante.
- 2026-11-14 — validation CR-3 P2 (Opus) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW ; remédiés dans la fiche (aucun
  finding inter-fiches). 37-2-CR3-P2-1 (MEDIUM, AC-5 adoption, AC-6 cours republié, Tâches) : « un `PUT` qui les
  reprend tel quel est refusé `FX_RATE_NOT_AFC` » contredisait l'arbitrage A17 (37-3a AC-4, 4 : seules les lignes
  `afc` nouvelles ou modifiées sont contrôlées) : réécrit (inchangée → passe ; nouvelle ou modifiée à l'ancien cours →
  refus). 37-2-CR3-P2-2 (LOW, AC-2 dates) : borne d'année 2000–2099 du `<datum>` contre la borne 2000-01-01 du
  `POST /fetch` — `xmldaily?d=20000101` rend `<datum>31.12.1999</datum>` (un GET, 2026-11-14) : borne basse à 1999,
  test ajouté. 37-2-CR3-P2-3 (LOW, AC-3) : réponse du raccourci sans `republished` : 0, la relecture montre le rouge.
  37-2-CR3-P2-4 (LOW, AC-3 (b)) : la remédiation CR3-P1-3 (`source_date` gardée) n'avait pas de test : ajouté.
  37-2-CR3-P2-5 (LOW, AC-3 (c)) : une republication d'un jour ne garde pas son `<datum>` ; l'adoption laisse
  `source_date` : limite écrite. 37-2-CR3-P2-6 (LOW, AC-4 A10 (iii)) : « démarrage » donné comme voie de découverte
  pendant la pause sans la condition d'une unité manquante : précisé. 37-2-CR3-P2-7 (LOW, Tâches) : « trois actions »
  d'audit pour quatre (`exchange_rates.fetched` + trois `exchange_rate_override.*`) : corrigé. Décompte : 1 MEDIUM,
  6 LOW (P2-1 à P2-7). Axes exercés : diff `b6f88c24..d9c0018e` de la fiche (A10, `source_date`) ligne à ligne contre
  l'AC-1 (pire statut), l'AC-3 (verrou, raccourci, mémoire, règle S6), l'AC-4 (calendrier, connexion, plafond),
  l'AC-6 (bouton « Revérifier »), l'AC-8, les Notes ; A17 contre 37-3a AC-4 (4) ; renvois du fichier d'epic (S1, S6,
  A9–A11, A16, A17) ; citations relues (`routes/auth.rs:235`, `+layout.svelte:373`, `BackupScheduleBanner.svelte`,
  `audit_labels.rs:38-39`, `backup.rs:46,136,949`, `post_restore.rs:939,988`) ; grep des autres fiches 37-* sur
  « redemande » / « revérifi » (aucun écart) ; source journalière : un GET (`d=20000101`). Non exercés : compilation,
  gate, rendu de l'écran, deux GET restants non faits (week-end et date future déjà relevés), 37-1/37-4/37-5 au-delà
  des renvois.
- 2026-11-14 — validation CR-3 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; rien à remédier, aucun
  finding inter-fiches. Axes exercés : diff `d9c0018e..040185ac` de la fiche, phrase par phrase — borne d'année 1999
  du `<datum>` (contre le relevé `d=20000101` du § Source, la borne 2000-01-01 de l'AC-5, le `CHECK` de
  `source_date`, le contrôle `date_mismatch`) ; réponse du raccourci avec `republished` (forme `{kind, rateDate,
  sourceDate, count, rejected, republished}` de l'AC-5) ; `source_date` gardée et limite de l'adoption (AC-5, adoption
  sans écriture de `source_date`) ; test (b) à `<datum>` différent ; réécriture A17 aux trois sites (AC-5, AC-6,
  Tâches) contre 37-3a AC-4 (4) et le fichier d'epic (A17) ; compte des actions d'audit (4 + 3 = 4 de l'AC-5 sur
  `exchange_rate_override.*` et `fetched`, republication ×2, `fiscal_year.rate_kinds_updated`) ; condition « unité
  manquante » de l'A10 (iii) contre l'AC-4 (démarrage, connexion, seul mois suivant). Non exercés : compilation, gate,
  rendu de l'écran, code au-delà des renvois, 37-1/37-4/37-5.
- 2026-11-14 — révision CR-3 (revue de Beat S1–S6 ; Opus) : **S1** — calendrier de la tâche automatique (AC-4,
  remplace R4 amendé) : quotidien tant que le mois courant ou le mois suivant manque, puis, le mois suivant `T` obtenu,
  pause jusqu'au plus tôt de « obtention + 24 jours » et du 25 de `T`, jamais avant le 1er de `T` ; `prochain_reveil`
  et ses tests réécrits (14.11 → 08.12, 25.11 → 19.12, 02.11 → 01.12, propriété « ≤ 25 de `T` », changement d'heure) ;
  date d'obtention lue sur la colonne neuve `first_fetched_at` (jamais avancée) ; connexion inchangée (CR2-P4-3).
  **S4** — tableau et unités utiles sans constante de douze mois : une colonne par mois du premier au dernier,
  tests d'un exercice de six et de dix-huit mois et d'un exercice commencé le 15 (AC-4, AC-5, AC-6). **S6** — cours
  AFC republié : colonnes `republished_rate`, `republished_unit`, `republished_published_rate`, `republished_at`,
  `notice_acknowledged_at` (et trois `CHECK`) dans le `CREATE TABLE` de `_kesh_exchange_rates` (AC-1) ; règle
  d'écriture de la récupération qui ne remplace plus le cours en vigueur (AC-3 ; `changed` devient `republished`) ;
  routes `GET /exchange-rates/notices`, `POST …/notices/acknowledge`, `POST …/republications/adopt` (AC-5, auditées,
  globales à l'installation) ; `republished` dans `Cell` ; écran : rouge et ↑ d'adoption, bouton « Revérifier les
  cours publiés », bandeau de connexion à acquitter (AC-6) ; `FX_RATE_NOT_AFC` et cours effectif sur le cours **en
  vigueur** (AC-5, AC-9) ; registre des routes : la 37-2 compte **+6 routes, +6 tracées, total +7, `sans_ecriture`
  +7, `tout.len()` +7**, en deltas. Manuel (AC-8). Prémisses relues au code : `chk_fiscal_years_dates`
  (`20260404000001_initial_schema.sql:51`) ; `BackupScheduleBanner.svelte` (lecture au montage puis 30 min,
  `role="status"`) et son montage `routes/(app)/+layout.svelte:373` ; aucune notion d'acquittement existante dans
  `crates/` ni `frontend/src` (grep `acknowledg`). Choix de l'orchestrateur, à confirmer : acquittement et adoption
  globaux à l'installation (la table est globale) ; détection d'une republication par un `POST /fetch` d'un cours
  présent seulement (bouton « Revérifier les cours publiés ») ; colonne `first_fetched_at` ajoutée pour S1. AC touchés :
  en-tête, Source, AC-1, AC-3, AC-4, AC-5, AC-6, AC-8, AC-9, Tâches, Notes. Fiche à revalider (cycle CR-3).
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T4** — cours d'ouverture = cours de clôture de
  l'exercice précédent : en genre `month`, `openingDate` = 1er du mois de la **veille** de `startDate` (dernier mois de
  l'exercice précédent), non plus le 1er du premier mois ; fonction pure unique pour `openingDate`/`closingDate`,
  partagée avec `unites_utiles` et le cours proposé de 37-3b (AC-5) ; `unites_utiles` ajoute ce mois hors des mois de
  l'exercice (AC-4) ; notices filtrées sur `openingDate` (AC-5) ; écran : libellé « cours mensuel du dernier mois de
  l'exercice précédent (<mois>) » et aide, case d'Ouverture propre en genre `month`, partagée avec la Clôture de
  l'exercice précédent, colonne récupérable (AC-6). Le défaut du genre d'ouverture d'un exercice qui suit un exercice
  Kesh (genre de clôture de celui-ci) est posé à la création, en 37-1 AC-3 bis ; `PUT /rate-kinds` ne le propage pas.
  **T2** — tests : exercice civil au 1er janvier explicite (plus seulement « témoin »), 1er avril, 1er juillet, six et
  dix-huit mois, avec les mois d'Ouverture `month` corrigés (décembre, mars, juin ; juin 2023 pour dix-huit mois) et
  deux exercices consécutifs à case partagée (AC-4, AC-6). T1, T3, T5, T6 sans point de contact. AC touchés :
  en-tête, AC-4, AC-5, AC-6. Fiche à revalider (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW ; remédiés dans la fiche sauf
  37-2-CR4-P1-IF1 (37-3b : jumeau TS du calcul d'ouverture, test de parité ; la fiche 37-3b dit « partagée avec
  l'écran », qui lit `openingDate` du serveur). MEDIUM : 37-2-CR4-P1-1 (AC-4, AC-5, AC-6) — « le mois d'Ouverture n'est
  pas l'un des `months` / case propre » est faux pour un exercice qui commence en cours de mois (15.03.2024 : la veille
  est le 14.03, même mois que le premier de l'exercice ; la fiche cite elle-même cet exercice) : liée à la première
  colonne, pas de doublon dans `unites_utiles`, test ; 37-2-CR4-P1-2 (AC-5) — « une seule fonction » pour le `GET`,
  `unites_utiles` et le cours proposé de 37-3b, irréalisable : Rust contre TypeScript : fonction de `kesh-core` +
  jumeau TS et test de parité. LOW : P1-3 (AC-1) reste de `journal_entry_lines.currency`, colonne retirée par T3 ;
  P1-4 (AC-6) partage Clôture/Ouverture valable seulement si l'exercice précédent finit la veille (trou : rompu) ;
  P1-5 (AC-4) unités avant les bornes 2000-01 / 2000-01-01 (veille d'un exercice au 01.01.2000) redemandées sans fin :
  exclues (relevé : `xmlavgmonth?j=1999&m=12` et `j=2000&m=1` rendent une racine de 308 octets, soit vide) ; P1-6
  (AC-8, Tâches) manuel muet sur T4 et aucune tâche CR-4 : ajoutés. Axes exercés : diff `2c0f5afc..bd7d0ec7` de la
  fiche ; D9, D10, R1–R10, S1–S6, T1–T6, A18–A22 ; accord avec 37-1 AC-3 bis (genre d'ouverture par défaut) et 37-3b
  (cours proposé) ; T3 : grep `journal_entry_lines` (un reste) ; sites sauvegarde, `TABLES_TO_TRUNCATE`, export CSV,
  audit, RBAC, i18n, manuels, E2E de la fiche ; source : deux GET. Non exercés : compilation, gate, rendu de l'écran,
  code au-delà des renvois relus (la révision ne cite aucune ligne de code neuve), 37-3a/37-4/37-5 au-delà des renvois.
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; tous remédiés dans la fiche.
  MEDIUM : 37-2-CR4-P2-1 (AC-4) — « douze unités distinctes de mois de mars 2024 à mars 2025 » contredit « treize mois »
  de la même phrase et l'AC-5 (treize colonnes) : treize ; 37-2-CR4-P2-2 (AC-5, Tâches) — « test de parité … des deux
  côtés » sans support ni auteur, cas sans le 29.02 que demande 37-3b AC-1 : fixture JSON commune
  `crates/kesh-core/tests/fixtures/fx_opening_dates.json` (modèle `fx_rounding.json`), lue par le test Rust et le test
  Vitest, cas réunis des deux fiches (29.02 et 01.01.2000 compris) ; renvoi « 37-3a/37-3b AC-1 » ramené à 37-3b AC-1
  (37-3a AC-1 est le modèle, sans cours proposé). LOW : P2-3 (AC-6) Ouverture propre « même case que la colonne de ce
  mois dans le tableau de l'exercice précédent » faux s'il y a un trou : « de l'exercice qui le contient, s'il en est
  un » ; P2-4 (AC-6) partage Clôture/Ouverture en genre `day` resté sans la condition « commence le lendemain » posée
  au genre `month` en P1 : condition portée aux deux ; P2-5 (AC-6, bouton) Ouverture liée à la première colonne
  récupérée une seule fois, mois < 2000-01 et jour < 2000-01-01 non récupérables (bornes de l'AC-5). Inter-fiches
  (non appliqué) : 37-3b AC-1 dit « même fixture si 37-2 la fournit, sinon vecteurs recopiés » — 37-2 la fournit
  désormais : nommer `fx_opening_dates.json` et retirer l'alternative. Axes exercés : diff `bd7d0ec7..a8f89e04` de la
  fiche ; entrée CR-4 P1 ; T2, T3, T4, C-37-3 et C-37-9 amendés ; 37-1 AC-1 (`BINARY` des `CHECK`, sens des genres) ;
  37-3a AC-1 (aucune colonne de devise, `fx_rounding.json`) ; 37-3b AC-1 (cours proposé, vecteurs) ; dénombrement des
  mois du 15.03. Non exercés : code (la remédiation ne cite aucune ligne neuve), compilation, gate, rendu de l'écran,
  37-4/37-5.
- 2026-11-14 — validation CR-4 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche. LOW : 37-2-CR4-P3-1 (AC-5) la fixture porte `closingDate` mais le jumeau TS de 37-3b ne calcule que `openingDate` : « lue par le test Vitest » laissait croire à un contrôle des deux champs ; précisé. Axes exercés : diff `a8f89e04..2f792399` phrase par phrase ; « treize unités » (15.03) contre AC-5 et Tâches ; fixture `fx_opening_dates.json` contre 37-3b AC-1 (nom, cas 29.02, aucune copie) et 37-3a (`fx_rounding.json`, répertoire créé par 37-3a, sans conflit) ; condition « commence le lendemain » (genres `day` et `month`) ; Ouverture propre « exercice qui le contient » ; bouton de récupération (bornes 2000-01, Ouverture liée comptée une fois) contre AC-4/AC-5 et la ligne 711 (raison au survol) ; item CR-4 des Tâches. Non exercés : code (aucune ligne neuve citée), compilation, gate, rendu de l'écran, 37-4/37-5. Aucun finding inter-fiches ; la remédiation P2 est cohérente avec 37-3b (alignée en P2).
- 2026-11-14 — révision CR-5 (revue 3 de Beat U1–U6 ; Opus) : **U1** — `exchange_rate_overrides` devient
  `exchange_rates` (applicative, par société, sauvegardée : `source` `afc|manual` extensible, `acknowledged_rate` /
  `acknowledged_at`, `updated_by` `NULL` pour une copie de Kesh, `UNIQUE (company_id, currency, kind, rate_date)`) ;
  `_kesh_exchange_rates` devient `_kesh_fetched_exchange_rates` (système, PK `(source, currency, kind, rate_date)`,
  dernière valeur publiée, `first_fetched_at`, `value_changed_at` ; plus de `republished_*` ni
  `notice_acknowledged_at`) (AC-1) ; règle d'écriture : une correction de l'AFC **remplace** la valeur publiée et ne
  touche jamais `exchange_rates` (AC-3) ; **AC-3 bis neuve** — copie automatique (`afc`, insertion seulement) à
  chaque récupération pour chaque société dont un exercice multi-devises ouvert a l'unité parmi ses unités utiles,
  toujours pour la société d'un `POST /fetch`, et à l'activation, à la création d'un exercice multi-devises, au passage
  en multi-devises et au changement de genre (appels ajoutés dans les routes de 37-1) ; « manque » jugé sur la valeur
  publiée ; A10 : la revérification met à jour la valeur publiée, l'écart est détecté, `exchange_rates` n'est pas
  touchée (AC-4) ; cours effectif = la ligne de la société, `find_company_rate_in_tx` (ex-`find_effective_rate_in_tx`)
  et `find_fetched_rate_in_tx` (ex-`find_rate_in_tx`, affichage seulement) ; `lookup` : `source` et `publishedRate`
  remplacent `afcRate` et `overridden` ; `Cell` à une valeur (`corrected`, `acknowledged`, `published`) ; `PUT
  /exchange-rates/rates` (ex-`…/overrides`) ; `POST /exchange-rates/rates/reset` (↑, remplace le `DELETE` et
  l'adoption) ; notices et acquittement **par société** (`exchange_rates.corrections_acknowledged`, entité `company`) ;
  actions d'audit et type d'entité `exchange_rate` renommés ; **registre des routes** : `DELETE …/overrides` et
  `POST …/republications/adopt` retirées, `POST …/rates/reset` ajoutée — la 37-2 compte **+5 routes, +5 tracées,
  total +6, `sans_ecriture` +6, `tout.len()` +6**, point (vi) **+5** (rejeu des cinq routes, cycle avec la copie
  de 37-1) (AC-5) ; écran : une seule valeur par case, mentions « AFC ✓ », « AFC corrigé : x » en rouge, « AFC : x »,
  « AFC ✓ · saisi » (A2 approuvé), ↑ (AC-6) ; restauration : les cours retenus et leur source reviennent avec la
  sauvegarde, plus de « cours perdus » (AC-7) ; R1 appliquée à `exchange_rates` sous son nom neuf (AC-1). **U3** —
  dialogue du ↑ d'une correction, « pour cette société » ; option « toutes les sociétés » spécifiée (`scope:
  managedCompanies`, lignes `afc` des seules sociétés gérées à exercice multi-devises ouvert) et masquée : prémisse
  relue au code — `users.company_id BIGINT NOT NULL` (`20260419000002_users_company_id.sql:23`),
  `CurrentUser.company_id: i64` (`middleware/auth.rs:41`), aucune table de rattachement (grep des migrations) ;
  issue à ouvrir (AC-5, AC-6). **U4** — `applyToNextOpening` sur `PUT /rate-kinds` (exercice suivant = celui qui
  commence le lendemain, ouvert seulement ; aucune écriture modifiée parce que les soldes sont cumulés et que les
  soldes d'ouverture ne se saisissent que sur le premier exercice — relu : `find_first_by_company`,
  `routes/opening_balances.rs:240`, `:379` ; modèle `NEXT_AFTER_SQL`, `repositories/fiscal_years.rs:207`), dialogue à
  case cochée par défaut (AC-5, AC-6) ; remplace A22. **U5** — A2, A7, A9, A10, A16 portés comme AC ici (A10 et A2
  « à confirmer » levés) ; A11 retiré. **U6** — sans point de contact (la fiche ne dit pas débit/crédit « en CHF »).
  Calendrier S1 inchangé. AC touchés : en-tête, Story, AC-1, AC-3, AC-3 bis, AC-4, AC-5, AC-6, AC-7, AC-8, AC-9,
  Tâches. Fiche à revalider (cycle CR-5).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; remédiés dans la fiche sauf
  37-2-CR5-P1-IF1 (inter-fiches : 37-1 AC-4). MEDIUM : 37-2-CR5-P1-1 (AC-1) `source` portait un `CHECK` d'énumération
  (`IN ('afc','manual')` et `IN ('afc')`) contraire à A28 — passés au format seul (`REGEXP '^[a-z]{2,12}$'`), valeurs
  contrôlées par le code ; 37-2-CR5-P1-2 (AC-3 bis, AC-1, AC-6, Tâches) A24 non appliqué : copie de toutes les devises
  publiées (≈ 1000 lignes par exercice) — limitée aux devises des comptes de la société, nouveau déclencheur « un
  compte reçoit une devise » (`accounts::create`, `accounts::update`), volume recalculé, tests ; conséquence : le
  bouton « Récupérer les cours manquants » relançait un appel à chaque clic pour une devise ajoutée à l'écran sans
  compte (`effectiveRate` toujours `null`) — champ `hasAccount` des lignes du tableau et critère restreint ;
  37-2-CR5-P1-3 (AC-3 bis) A23 non appliqué : le ↑ d'une correction ouvrait un dialogue dont la seule option
  utile est masquée — ↑ direct, dialogue spécifié sous condition unique (AC-6, Vitest, E2E, captures, Tâches, AC-8,
  en-tête). LOW : 37-2-CR5-P1-4 la copie était dite « lue sans verrou » alors que `INSERT … SELECT` pose des verrous
  partagés sous REPEATABLE READ (cycle couvert par `retry_on_deadlock`, écrit) ; 37-2-CR5-P1-5 en-tête : nom
  `exchange_rate_overrides` resté au présent ; 37-2-CR5-P1-6 colonne de comparaison des notices précisée
  (`f.rate_per_unit`). A27 écrit (AC-3 bis) ; A25 et A26 déjà portés. Inter-fiches non appliqué, texte proposé :
  37-2-CR5-P1-IF1 — 37-1 AC-4, ajouter : « Quand une devise est donnée à un compte (création, ou modification qui change
  la devise), la copie des cours déjà récupérés de cette devise pour les exercices multi-devises ouverts (37-2 AC-3
  bis, appel ajouté par 37-2 dans `accounts::create` et `accounts::update`, sous leur `retry_app_on_deadlock`) est faite
  dans la même transaction ; `copied` va au détail de l'audit du compte. » Axes exercés : diff
  `0ae43432..d59a1de9` de la fiche ; U1 (grep des anciens noms dans les six fiches : restes seulement dans des entrées
  de Change Log datées et des mentions « ex- »), AC-1 (tables, `CHECK`, TABLES_TO_TRUNCATE/TABLES_SYSTEME/R1),
  AC-3, AC-3 bis, AC-5 (U3, U4), AC-6, AC-7, A23–A28 ; citations relues au code (`users_company_id.sql:23`,
  `middleware/auth.rs:41`, `NEXT_AFTER_SQL` `fiscal_years.rs:207`, `opening_balances.rs:240` et `:379`,
  `audit_labels.rs` `ENTITY_TYPES` et tri, `backup.rs` `mcp_server_settings` :99 et `TABLES_SYSTEME` :949,
  `audit_route_registry.rs` (vi) :155/:302, `exports/global.rs:116`). Non exercés : les trois `GET` de la source
  journalière (aucune prémisse de source touchée par CR-5), compilation, gate, rendu de l'écran, MCP, manuels au-delà de
  la fiche, 37-3a/37-3b/37-4/37-5 (sauf renvois).
- 2026-11-14 — validation CR-5 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; tous remédiés dans la fiche,
  aucun inter-fiches. MEDIUM : 37-2-CR5-P2-1 (AC-3 bis, AC-5 rejeu, AC-4 fetch, Tâches) alignement 37-1 CR5-P1-2 :
  la copie à l'activation est vide par construction (aucun compte en devise tant que la société est mono-devise) —
  puce « à l'activation » et `companies::update_currency_settings` retirés des appelants (« quatre derniers
  appels »), test « activation → aucune ligne copiée », A27 réécrit (la réactivation ne copie rien), cycles « avec
  une activation » retirés (AC-4, AC-5), renvoi « à ajouter à 37-1 AC-4 » remplacé (37-1 AC-4 le porte) ;
  37-2-CR5-P2-2 (AC-3 bis, tests) tests délégués par 37-1 AC-4 absents (devise changée USD→EUR sur un compte sans
  ligne, `PUT` no-op, aucun exercice multi-devises ouvert) : ajoutés ; 37-2-CR5-P2-3 (AC-1) la justification d'A28
  disait l'élargissement d'un `CHECK` d'énumération « jugé breaking par P1 et P3 » — P3 ne liste ni `DROP CONSTRAINT`
  ni `ADD CONSTRAINT`, et le format seul n'ôte pas le risque P1 (un binaire antérieur rend une erreur interne sur une
  source inconnue, `RateOrigin`) : réécrit, le jugement P1 de la release qui écrit une source neuve reste dû
  (question à Beat : A28 promettait « pas de migration breaking »). LOW : 37-2-CR5-P2-4 le verrouillage de la copie
  décrivait le cycle par `companies` seul et nommait `retry_on_deadlock` pour les routes de comptes — le filtre A24
  lit `accounts` en partage (cycle avec une route de compte qui tient le compte `FOR UPDATE`), et ces routes passent
  par `crate::retry::retry_app_on_deadlock` (`kesh-api/src/retry.rs:40`, `routes/accounts.rs:337`) : écrit (AC-3 bis,
  AC-4) ; 37-2-CR5-P2-5 signature `copy_fetched_in_tx` sans le paramètre `currencies` qu'attend 37-1 AC-4 : ajouté ;
  37-2-CR5-P2-6 en-tête : copie « de chaque cours récupéré utile » sans la restriction A24 : précisée. Axes exercés :
  diff `d59a1de9..3b058509` de la fiche, entrée CR-5 P1 et alignement de l'orchestrateur ; points de contact avec
  37-1 (AC-3 activation, AC-4 copie et verrous, Tâches CR-5, Change Log CR5-P1-1/2) ; registre A23–A29 ; code lu :
  `retry.rs` des deux crates, `routes/accounts.rs:267`, `:337` ; P3 de `CLAUDE.md`. Non exercés : 37-3a, 37-3b, 37-4,
  37-5 (aucune phrase du diff n'y renvoie hors 37-3b/37-5 pour le filet `lookup`, inchangé), rendu de l'écran,
  compilation, gate, sémantique exacte des verrous InnoDB d'un `IN (SELECT …)` (raisonnée, non exécutée).
- 2026-11-14 — validation CR-5 P3 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW ; remédié dans la fiche, aucun
  inter-fiches. MEDIUM : 37-2-CR5-P3-1 (AC-1, AC-3 bis) alignement A30 : la lecture d'une `source` inconnue devient
  tolérante (`RateOrigin::Other(String)`, affichée telle quelle) ; le passage « erreur interne, jamais ignorée » et le
  jugement P1 « à décider alors » ajoutés en P2 sont réécrits (plus de relèvement de `min_required` à prévoir pour une
  source neuve) ; signature `CompanyRate.source` complétée ; test `source = 'ecb'` ajouté. Le reste du diff P2
  (activation retirée, `currencies`, verrous, `retry_app_on_deadlock` `kesh-api/src/retry.rs:40`, `retry_on_deadlock`
  `kesh-db/src/retry.rs:151`) relu : cohérent avec 37-1 AC-3/AC-4. Non exercés : 37-3a, 37-3b, 37-4, 37-5 au-delà des
  renvois, compilation, gate, rendu.
- 2026-11-14 — validation CR-5 P4 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW ; remédiés dans la fiche ; trois
  points inter-fiches signalés (37-3a, 37-3b, 37-5). MEDIUM : 37-2-CR5-P4-1 (AC-1) la lecture tolérante ne disait pas
  ce que fait le code d'une ligne `Other` lue — écrit : traitée partout comme `manual` hors l'affichage et l'audit de
  sa source (`FX_RATE_NOT_AFC` `expectedRate` `null`, `PUT` → `manual`, `reset` → `afc`, ni acquittement ni reprise),
  et la reprise « sociétés gérées » exclut toute source autre que `afc` ; -2 (AC-5) contrat de `lookup` encore borné à
  `"afc"|"manual"` — ouvert à la valeur lue telle quelle ; test `ecb` étendu à `lookup`/grille, `PUT` et `reset`.
  LOW : -3 « un binaire antérieur n'aurait pas cette tolérance » — il ne connaît pas la table ; reformulé, avec le cas
  d'un `.keshbackup` futur importé ici, et la réserve que la release qui écrit une source neuve vérifie que le
  traitement `manual` lui reste juste. Vérifié : A28 (format seul `^[a-z]{2,12}$`, aucune énumération) intact ; P1–P3
  de `CLAUDE.md` : aucune opération P3, aucun binaire antérieur ne lit la table. Non exercés : compilation, gate, rendu.
- 2026-11-14 — validation CR-5 P5 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW ; remédiés dans la fiche.
  MEDIUM : 37-2-CR5-P5-1 (AC-1, AC-5) le ↑ (`reset`) sans valeur publiée ne disait rien d'une ligne `Other` (« ligne
  `manual` → supprimée ; ligne `afc` → 404 ») — elle est supprimée comme une ligne `manual`, écrit à l'AC-1 et à
  l'AC-5, test ajouté ; -2 (AC-6) les mentions de la grille n'énuméraient que `afc` et `manual` — ligne d'une autre
  source : comme `manual`, mêmes ↑, source brute entre parenthèses ; `reset` aussi pour une valeur entrée égale ;
  Vitest. LOW : -3 (AC-5, grille) lignes « devises ayant une ligne `manual` » étendues aux autres sources, test ; -4
  (AC-1) l'import d'un `.keshbackup` futur était dit inconditionnel — conditionné à l'inventaire
  (`admin_backup/import.rs:187`) et à la version minimale du manifeste (`routes/admin.rs:235`), lus. Sites non résolus
  de A30 inventoriés dans les six fiches : copie (insertion seule), `PUT` (→ `manual`), notices et acquittement
  (`source = 'afc'`), `corrected` (`afc` seul), qualification d'une ligne d'écriture (AC-9, `afc` si égal à une ligne
  `afc`, `manual` sinon) traitent déjà une autre source comme `manual` ; restaient le `reset` sans valeur publiée, la
  grille (lignes et mentions), corrigés ici, et des sites de 37-3a, 37-3b, 37-5 (leurs Change Logs). 37-1 (copie
  `source = 'afc'`) et 37-4 (aucune lecture de `exchange_rates.source`) sans site. Non exercés : compilation, gate,
  rendu, reste de la fiche hors diff.
- 2026-11-14 — validation CR-5 P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche. LOW
  37-2-CR5-P6-1 (AC-1) la garde d'import citait `parse_and_verify_avec_inventaire` (`import.rs:187`, signature `pub(crate)`
  d'un test) — la production passe par `parse_and_verify` (`import.rs:175-180`), qui lui transmet `TABLES_TO_TRUNCATE` ;
  `check_import_version_compat` lu à `routes/admin.rs:234-235`. Sites de A30 grepés (`manual`/`afc`) dans les quatre fiches,
  37-1 et 37-4 : aucun site restant qui traite une autre source autrement que `manual` ; `rateSource` reçu en entrée
  (`afc|manual`, 37-3a) est une entrée client, non une lecture de source. Non exercés : reste des fiches hors diff,
  compilation, rendu.
- 2026-11-15 — exigences FR/NFR ajoutées (B26 ; orchestrateur, Sonnet) : 9 FR, 6 NFR.
- 2026-11-15 — validation ciblée des exigences (Opus) : 9 MEDIUM, 7 LOW, tous corrigés. MEDIUM : FR-3/FR-4 (une
  récupération ne modifie jamais `exchange_rates` ; copie par insertion seule, devises des comptes, A24) ; FR-6
  (contrat de l'API réduit à la fonction de lecture : routes, codes, jamais 500 ni 502, `MULTI_CURRENCY_DISABLED`) ;
  FR-8 (réimport d'une sauvegarde sans la table, R1 ; remise à zéro) ; NFR-5 (audit réduit à la saisie et au ↑) ;
  `PUT …/rates` et ↑ sans exigence → **FR-37-2-10** ; `PUT /rate-kinds` et U4 → **FR-37-2-11** ; corrections de l'AFC,
  notices et acquittement → **FR-37-2-12** ; RBAC → **NFR-37-2-7** ; filtrage de la source hostile (R5) et client
  HTTP → **NFR-37-2-8**. LOW : FR-1 (`_kesh_exchange_rate_fetch` omise), FR-2 (`parse_daily_rates`, fonction des
  bornes), FR-5 (calendrier S1, connexion, A10), FR-7 (écran), NFR-3 (bornes), NFR-4 (renvoi AC-4), NFR-6 (manuel
  d'administration) ; FR-9 précisé. Décompte : 12 FR, 8 NFR (+3, +2).
- 2026-11-15 — validation ciblée P2 des exigences (Sonnet) : 1 MEDIUM, 6 LOW, tous corrigés. MEDIUM : FR-5 (la pause n'est pas « d'au plus 24 jours » : réveil au plus tôt de obtention + 24 jours et du 25, **sans reprendre avant le 1er** du mois obtenu — obtenu le 02.11, réveil le 01.12, AC-4). LOW : FR-5 (revérification hors plafond, sans tour ni réveil), FR-4 (course de deux copies), FR-8 (export CSV, AC-1), FR-10 (portée société seule, ligne `afc` sans valeur publiée), FR-7 (Consultation, exercice clos), NFR-4 (endpoint `_test`). Constat hors périmètre, non corrigé : AC-8 écrit au manuel « pause d'au plus 24 jours » (même inexactitude que l'ancien FR-5) — à relire à la prochaine passe sur les AC. Décompte : 12 FR, 8 NFR (inchangé).
- 2026-11-15 — alignement de l'orchestrateur (constat hors périmètre de la validation P2 des exigences) : AC-8 (manuel,
  l. 1067) et la tâche du calendrier (l. 1128) disaient « pause d'au plus 24 jours » ; réécrits sur la règle de l'AC-4
  (réveil au plus tôt de « obtention + 24 jours » et du 25 du mois obtenu, jamais avant le 1er de ce mois).
- 2026-11-15 — validation ciblée P3 des exigences (Opus) : 0 MEDIUM, 1 LOW, corrigé. LOW : FR-5 (« jamais le 25 »
  se lisait « jamais un 25 » : récrit « le réveil ne passe jamais le 25 » ; sommeil jusqu'au 1er du mois suivant quand
  le mois suivant n'est couvert par aucun exercice multi-devises ouvert, AC-4, ajouté). Correctifs P2 relus contre
  l'AC entier (FR-4 AC-3 bis, FR-5 AC-4, FR-7 AC-6, FR-8 AC-1, FR-10 AC-5, NFR-4 AC-3) : justes. Constat hors
  périmètre, non corrigé : l'alignement d'AC-8 (l. 1067-1068) laisse « jamais avant le 1er de ce mois jamais au-delà
  du 25 (S1) », redite de l'ancien texte sans ponctuation — à relire à la prochaine passe sur les AC. Récapitulatif de
  l'epic concordant. Décompte : 12 FR, 8 NFR (inchangé).
- 2026-11-16 — révision CR-7 (A34 multi-devises sans retour, Opus) : copie des cours retirée au passage d'un exercice
  en multi-devises (`set_currency_mode` n'existe plus ; AC-3 bis, FR-4, Tâches, tests) ; paragraphe A27 et test
  « désactivation puis réactivation » retirés ; cycles de rejeu réécrits contre l'activation de 37-1 (`companies` puis
  exercices) et les routes d'exercice qui copient ; point (vi) de 37-1 « vingt-sept ». Décompte : 12 FR, 8 NFR
  (inchangé).
- 2026-11-16 — validation CR-7 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW. MEDIUM 37-2-CR7-P1-1 : le rejeu de
  l'AC-5 attribuait aux cinq routes un cycle avec l'activation de 37-1 ; seule `PUT /rate-kinds` (exercice `FOR UPDATE`
  puis `companies` en partage) en ferme un avec elle, les quatre autres le font entre elles et avec la
  récupération — récrit. LOW -P1-2 : AC-3 bis (copie), cycle « avec une route d'exercice qui tient `companies` `FOR
  UPDATE` » : reste de la bascule, aucune route d'exercice ne la tient plus — remplacé par les routes qui copient.
  LOW -P1-3 : trois renvois « au passage en multi-devises » (AC-1 réimport, AC-7 remise à zéro, U4 de l'AC-5, « lu quand
  il passera en multi-devises ») — remplacés. LOW -P1-4 : aucun test de l'exercice mono-devise (clos avant l'activation,
  rouvert) d'une société multi-devises (récupération, copie, grille, `PUT /rate-kinds`, sélecteur) — ajouté aux tests
  de l'AC-3 bis (cas D). Récapitulatif de l'epic concordant, non modifié. Axes exercés : résidus (grep des neuf termes
  : aucun hors Change Logs datés), AC-3 bis, AC-4 (unités utiles), AC-5 (ordre des refus, rejeu, registre « vingt-sept »),
  AC-7, D-CR7-1 (aucune dépendance de 37-2 au verrou de `companies` des écritures). Non exercés : AC-6 au-delà du
  sélecteur, code, i18n, manuels. Décompte : 12 FR, 8 NFR (inchangé).
- 2026-11-16 — validation CR-7 P2 (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW ; remédiés. LOW 37-2-CR7-P2-1
  (décision de l'orchestrateur) : le 409 `MULTI_CURRENCY_DISABLED` de `PUT /rate-kinds` (AC-5) jugeait « société ou
  exercice mono-devise » — aligné sur l'exercice seul (une société mono-devise n'a que des exercices mono-devise).
  LOW -P2-2 (D-CR7-2) : U4 de l'AC-5 renvoyait à « la limite de 37-1, Révision CR-7 », retirée — récrit : l'exercice
  suivant d'un exercice multi-devises l'est aussi (aucun exercice ne se crée avant un exercice mono-devise, 37-1 AC-3
  bis). LOW -P2-3 (résidu de la bascule) : AC-3 bis, « la route tient déjà `companies` » — faux pour `PUT /rate-kinds`
  (exercice `FOR UPDATE` puis `companies` par la clé étrangère, AC-5 *Rejeu*) ; récrit (routes de compte et
  `fiscal_years::create`, qui lit désormais `companies` en partage en tête, 37-1 AC-3 bis, la tiennent ; `PUT
  /rate-kinds` la prend après l'exercice). Axes exercés : diff P1 (`882d865b..e99364d2`, AC-3 bis, AC-5 rejeu, AC-1,
  AC-7), résidus du modèle réversible (grep : bascule, désactivation, réactivation, « au passage », limite, société
  mono-devise — hors Change Logs, aucun reste), D-CR7-1 (aucune dépendance), D-CR7-2 (renvois), branche du 409 (AC-5,
  FR-11, `GET` du tableau, ordre des refus des routes de cours), KISS (aucun mécanisme propre à la réversibilité ;
  lecture du mode sous le `FOR UPDATE` de l'exercice gardée : verrou déjà pris). Non exercés : AC-4 au-delà des
  unités utiles, AC-6 au-delà du sélecteur, code, i18n, manuels. Décompte : 12 FR, 8 NFR (inchangé).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : PDF des manuels non commités, `.tex` seulement (3 remplacements) ; CHANGELOG remplacé par la section *Note de version* (1 remplacement, section ajoutée) ; compteurs « recomptés depuis la source après chaque reprise de `main` » (3 ajouts). Fond des AC inchangé.
