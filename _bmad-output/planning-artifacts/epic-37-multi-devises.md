# Epic 37 — Comptabilité multi-devises

**Kickoff : 2026-11-14.** Demande de Beat (contributeur, comptabilité d'une SA tenue jusqu'ici dans un logiciel
multi-devises : comptes PayPal, Wise et débiteurs en USD et en EUR, ventes en USD, réévaluation au 31.12 sur le compte
6840 *Différences de change*). L'epic est développé **sur une branche unique** (`epic/37-multi-devises`) et proposé à
Guy en **une seule PR** à la fin, après une recette sur l'exercice 2024 de Beat importé par l'API — **par l'API
seule** (CR-6 : sessions en `Authorization: Bearer`, activation comprise ; test de bout en bout de 37-5 AC-7, V7 ; la
réévaluation des livres d'origine se repasse par `POST /fiscal-years/{id}/fx-revaluation` avec `rates`, USD 0.91193
et EUR 0.94947 au 31.12.2024, non par des écritures importées, V8).

**Numéro : 37** (renuméroté depuis 35 le 2026-11-14, sur indication de Beat : les numéros 35 et 36 sont déjà réservés ; 38 est réservé par Beat aux comptes PayPal, 39 aux comptes Wise). Les comptes rendus de passe, prompts et Change Logs antérieurs à cette date écrivaient « 35 » : renumérotés avec les fichiers ; la page de revue `epic-35-revue.html`, en cours de lecture par Guy, garde l'ancien numéro (35-N = 37-N).

**Cible : la release qui suivra la v0.17.6** (numéro, CHANGELOG daté, macros des manuels et tag : à la préparation de
la release, par Guy — hors de cette branche).

⚠️ **Pas d'issue GitHub à ce stade** : Beat n'a qu'un accès en lecture au dépôt de Guy. Le détail vit dans cet epic et
dans les fiches ; l'issue et la PR s'ouvrent à la fin, une fois la recette faite (décision de Beat, 2026-11-14).

## Décisions de Beat (2026-11-14)

- **D1 — Réglage de la société.** La comptabilité est **mono-devise** (comportement actuel, inchangé) ou
  **multi-devises**. La **devise principale ne change pas** (CHF) ; en mode multi-devises, les autres devises sont
  **sélectionnables** dans la liste ISO 4217. En mono-devise, rien de ce qui suit n'apparaît.
- **D2 — La devise est portée par le compte**, comme dans les livres de Beat (le compte de ventes `3132-USD-EX` est en
  USD comme le compte PayPal USD). Toute ligne d'écriture sur un compte en devise porte **le montant en devise, le cours
  et le montant en CHF** ; toute ligne a une valeur en CHF, et **l'écriture s'équilibre en CHF**.
- **D3 — Le cours par défaut est le cours mensuel officiel de l'AFC** (fixé d'avance pour le mois, voir *Terminologie*) (ESTV, publié par l'OFDF/BAZG) du mois de la
  date de l'écriture ; il est **modifiable ligne par ligne** (saisie manuelle, p. ex. pour une conversion de devises).
- **D4 — Les cours mensuels se récupèrent automatiquement** sur le site officiel, et **toutes les devises publiées**
  sont stockées dans la table des cours mensuels (précision de Beat, 2026-11-14) — USD et EUR pour Beat, mais
  l'ensemble est universel.
- **D5 — Réévaluation de fin d'exercice** *(cours corrigé par D10)* : au **cours mensuel de décembre** (le mois de clôture de l'exercice), dans
  les deux sens (gain et perte), contre un compte de différences de change choisi par l'utilisateur. Fondement : les
  devises ont un cours de marché observable (CO 960b).
- **D6 — Conversion de devises** (EUR → USD) : les deux montants en devise sont saisis avec leurs cours ; l'écart en CHF
  est **proposé** comme ligne de différence de change, jamais forcé en silence.
- **D7 — Périmètre** : (a) réglage, devise des comptes, table des cours et récupération ; (b) écritures en devise ;
  (c) grand livre, balance, bilan et compte de résultat dans les deux devises ; (d) réévaluation. **Hors périmètre** :
  factures, factures fournisseur, imports bancaires (CAMT, CSV) en devise — l'API suffit pour l'instant.

- **D8 — Comptes bancaires en devise (2026-11-14).** Tant que les imports en devise n'existent pas, PayPal, Wise et
  les autres comptes en USD ou EUR sont des **comptes du plan** en devise, et non des comptes bancaires Kesh (un compte
  lié à un compte bancaire ne peut porter de devise, C-37-9). Beat : « plain ledger accounts for PayPal/Wise is fine if
  I can change type later to bank accounts ». **Engagement pour l'epic des imports en devise** : permettre de lier un
  compte bancaire à un compte du plan **déjà en devise**, en gardant numéro, historique et soldes (à tracer en issue
  avec la PR).

- **D9 — Table des cours modifiable (Beat, 2026-11-14, après validation des fiches : change de périmètre).** La table
  des cours n'est pas en lecture seule : la société peut **saisir son propre cours** pour une devise et un mois (ou un
  jour, D10). Un cours saisi **n'est jamais remplacé** par une récupération AFC. À l'écran, sous le champ, le **cours
  AFC** s'affiche avec un bouton **↑** qui reprend le cours AFC (la saisie est alors effacée). Cours effectif = cours
  saisi s'il existe, sinon cours AFC. Une ligne d'écriture prend le cours effectif ; elle est `afc` si ce cours égale
  le cours AFC, sinon `manual`.
- **D10 — Réévaluation au cours du jour de clôture (corrige D5).** Vérifié le 2026-11-14 : les cours des livres 2024 de
  Beat (USD 0.91193, EUR 0.94947) sont **exactement** les cours **journaliers** OFDF du 31.12.2024
  (`https://www.backend-rates.bazg.admin.ch/api/xmldaily?d=20241231`, cinq décimales, validité 01–03.01.2025),
  importés par son ancien logiciel. Le cours par défaut de la réévaluation est donc le **cours journalier de la date
  de clôture**, récupéré comme les cours mensuels et modifiable (D9) ; à défaut, le cours mensuel du mois de clôture,
  signalé à l'écran. C'est aussi la pratique (cours de clôture, CO 960).
- **Terminologie** (Beat) : les cours mensuels AFC ne sont **pas des moyennes** ; ils sont **fixés d'avance** pour le
  mois (publiés en début de mois). Les textes disent « cours mensuel AFC », jamais « cours moyen ».

## Revue de Beat — CR-2 (2026-11-14, première revue de la page `epic-37-revue.html`)

Réponses de Beat à la revue et aux quatre questions qui ont suivi ; elles **priment** sur D1–D10 et sur le registre
là où elles divergent.

- **R1 — Sauvegardes antérieures réimportables (exception).** Une sauvegarde d'une version antérieure à l'Epic 37 se
  réimporte : elle est forcément mono-devise. L'import tolère l'**absence des seules tables et colonnes ajoutées par
  l'Epic 37** (liste close, déclarée dans le code), traitées comme vides ou à leur défaut. Cela **rouvre la fenêtre
  d'import** fermée par CR-1 (le registre de rejeu redevient tel quel).
- **R2 — Récupération AFC ciblée.** Kesh ne récupère que les cours des **exercices en multi-devises** (R3).
- **R3 — Mode par exercice.** Le réglage de la société est le **défaut** des exercices créés (l'exercice initial
  compris). Chaque exercice peut être passé en mono-devise ou en multi-devises. Passer un exercice en mono-devise est
  **refusé** tant qu'un compte en devise n'est pas à **0** (devise et CHF) au début de l'exercice ou porte une ligne
  dans l'exercice. Dans un exercice mono-devise, les comptes en devise ne sont pas utilisables ; Kesh propose de les
  **archiver**, après avertissement (API : confirmation explicite, `force`). Pas de conversion d'un compte en CHF.
  *(CR-7, A34 : remplacé — le multi-devises ne se désactive plus ; les exercices ouverts à l'activation et ceux créés
  ensuite sont multi-devises, les exercices clos à l'activation restent mono-devise ; plus de bascule, d'avertissement
  ni de `force`.)*
- **R4 — Calendrier de récupération.** Source ESTV (*MWST Monatsmittelkurse*, lue le 2026-11-14) : le cours du mois M
  est la moyenne des cours journaliers du 25 de M−2 au 24 de M−1, **publié le 25 de M−1** (ou le jour ouvrable
  suivant), applicable dès le 1er de M ; l'heure n'est pas publiée. Kesh vérifie toutes les 6 h **à partir du 25**
  jusqu'à obtenir le mois suivant, puis s'arrête jusqu'au 25 suivant ; et au **démarrage** comme à la **connexion**
  s'il manque un mois utile. Pas de boucle permanente toutes les 6 h.
- **R4 amendé (orchestrateur, 2026-11-14, à confirmer par Beat).** Relevé contraire à la page ESTV : le
  14.11.2026 (en-tête `Date` du serveur), `xmlavgmonth?j=2026&m=12` rend **déjà** décembre 2026 complet (USD 0.8320,
  EUR 0.9295) ; `j=2027&m=1` est vide. Le mois suivant paraît donc **avant le 25**, et la définition « du 25 au 24 »
  n'est pas vérifiable en l'état. Calendrier retenu, sûr dans les deux lectures : **une vérification par jour**
  (dès le 1er du mois) tant que le mois courant ou le mois suivant manque pour un exercice multi-devises, puis
  plus rien jusqu'au mois suivant ; plus le démarrage et la connexion (la connexion ne déclenche rien quand seul le
  mois suivant manque : arbitrage CR2-P4-3). `POST /fetch` admet le mois suivant à toute
  date (non publié → `404 RATE_NOT_PUBLISHED`). Le manuel ne décrit la méthode de calcul qu'après confirmation.
- **R5 — Données reprises filtrées.** Tout ce qui vient de la source est validé strictement (taille, structure,
  refus de DOCTYPE et d'entités, codes, nombres, bornes, mois et dates), avec des tests de réponses hostiles.
- **R6 — Écran des cours en tableau annuel.** Une ligne par devise, une colonne par mois **de l'exercice** (dans
  l'ordre de l'exercice), plus **Ouverture** et **Clôture**. Chaque case : cours effectif modifiable ; dessous
  « AFC ✓ » s'il est égal au cours AFC, sinon « AFC : 0.8846 » et ↑. Par exercice, deux choix : **cours d'ouverture**
  (cours journalier de la veille de l'ouverture — défaut — ou cours mensuel du premier mois) et **cours de clôture**
  (cours journalier de la date de clôture — défaut — ou cours mensuel du dernier mois). Testé pour des exercices
  commençant le **1er juillet** et le **1er avril**.
- **R7 — Conversion par un compte de transfert en CHF.** Une écriture ne porte de lignes que dans **une seule devise
  étrangère** (plus des lignes CHF). Une conversion EUR → USD passe par un **compte de conversion** en CHF, désigné à
  l'activation du multi-devises : écriture 1, EUR → compte de conversion ; écriture 2, compte de conversion → USD, avec
  la **différence de change** dans l'écriture 2 pour que le compte de conversion revienne à 0 ; des lignes de frais
  sont possibles (écritures à plusieurs lignes). Un assistant *Conversion* crée les deux écritures. Fondement :
  aucune interdiction expresse trouvée dans le CO (957a impose la tenue en CHF) ; c'est la pratique fiduciaire et
  l'avis du réviseur de Beat ; Kesh l'impose pour la traçabilité (chaque écriture se rapproche d'un seul relevé).
- **R8 — « Ajouter la différence de change »** ajoute une **ligne** à l'écriture en cours ; il ne crée pas d'écriture.
- **R9 — Réévaluation : une écriture par compte réévalué** (comme les livres 2024 : « Gain de change PAYPAL ONLINE USD
  0.91193 »), passées ensemble dans une transaction ; le bouton dit « Passer les écritures ».
- **R10 — Constante du début d'exercice.** L'assistant crée l'exercice initial au 1er janvier
  (`routes/onboarding.rs:1045`) : la date devient une constante nommée, avec un commentaire « ne pas modifier sans … ».
  Kesh accepte déjà des exercices à dates libres : rien d'autre à implémenter pour cela.

### CR-2 — amendements du registre et arbitrages (2026-11-14)

- **R1** : la fenêtre d'import **ne se referme pas** (annule l'avertissement de CR-1). Les tables et colonnes de
  l'Epic 37 forment une liste close d'absences admises (`kesh-db::backup`, 37-1 AC-6) ; registre de rejeu inchangé ;
  phase 5 de `recette-montee.sh` jouée ; une sauvegarde 0.17.6 se réimporte en mono-devise.
- **C-37-2 amendé (R7)** : + `fx_transit_account_id`, compte **de bilan** en CHF, mêmes gardes sauf le type,
  obligatoire pour activer ; `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` porte `details.designation` (`gain|loss|transit`).
- **C-37-10 (R3, R6)** : `fiscal_years.multi_currency`, `opening_rate_kind`, `closing_rate_kind` (migration de
  37-1) ; défaut tiré de la société à la création ; activation de la société → exercices **ouverts** en
  multi-devises ; `MULTI_CURRENCY_DISABLED` jugé sur l'exercice de l'écriture. *(CR-7 : désactivation, bascule
  `PUT /fiscal-years/{id}/currency-mode`, `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`,
  `force` et `laterFiscalYear` retirés ; refus de désactivation 409 `MULTI_CURRENCY_IRREVERSIBLE`.)*
- **C-37-3 complété (R7)** : une seule devise étrangère par écriture (`FX_MULTIPLE_CURRENCIES`, 400), sauf ouverture,
  complément et contre-passation ; conversion par `POST /journal-entries/conversions` (deux écritures, une transaction).
- **C-37-6 amendé (R2, R4, R5)**, **C-37-7 amendé (R9, R6)**, **C-37-9 amendé (R6 : cours d'ouverture selon
  `opening_rate_kind`)**, terminologie R4 : repris des fiches révisées.
- **Arbitrages de l'orchestrateur sur les questions ouvertes de la révision CR-2** (soumis à Beat dans la page de
  revue 2) :
  - A1 — commentaire de la constante (R10) : « Ne pas modifier : l'exercice initial créé par l'assistant commence le
    1er janvier ; un exercice à d'autres dates se crée dans *Exercices* » ;
  - A2 — un cours saisi égal au cours AFC s'affiche « AFC ✓ · saisi » avec ↑ (l'écran n'en crée jamais) ;
  - A3 — l'écriture d'ouverture et son complément sont exemptés de la règle d'une seule devise ;
  - A4 — une ligne en devise datée avant un exercice mono-devise est refusée ; repasser cet exercice en
    multi-devises pour saisir ; *(retiré par CR-7)*
  - A5 — `force` archive les comptes en devise sans ligne après la fin de l'exercice ; pas de bascule sans archivage ;
    l'activation passe tous les exercices ouverts en multi-devises ; *(CR-7 : seule la dernière clause reste)*
  - A6 — compte de conversion : compte de bilan non archivé, à créer (aucun plan livré n'en a ; les fiches disent « p. ex. 1090 », le plan de Beat le numérote 1091) ;
  - A7 — la récupération automatique ignore les exercices clos ; leurs cours se récupèrent à l'écran ;
  - A8 — la conversion passe par une route serveur (deux écritures dans une transaction).
- **Tableau des stories** : 37-2 « ne referme pas la fenêtre d'import (R1) ».
- **C-37-2 (3) complété (CR-2 P1)** : la route de conversion (`POST /journal-entries/conversions`, 37-3b) revalide
  aussi le compte de conversion à l'usage (`FX_ACCOUNT_INVALID`), comme la réévaluation ; la ligne d'écart du
  formulaire reste une ligne CHF ordinaire.

## Revue de Beat — CR-3 (2026-11-14, avant la revue 2 de la page)

Prime sur CR-2 et sur ses arbitrages là où ils divergent.

- **S1 — Calendrier de récupération (remplace R4 amendé).** Vérification **une fois par jour** jusqu'à l'arrivée du
  **premier cours du mois suivant** ; puis **pause** de 24 jours, **au plus tard jusqu'au 25** du mois, où la
  vérification quotidienne reprend pour le mois d'après. Plus le démarrage et la connexion (sans redemander le
  mois suivant seul, CR2-P4-3).
- **S2 — Comptes de différence de change par compte en devise.** Le réglage de la société reste le **défaut**. Sur
  chaque compte en devise, l'utilisateur choisit : *comptes par défaut* (suivent automatiquement le réglage de la
  société quand il change) ou **comptes propres** (gains, pertes) — p. ex. un sous-compte « frais de change cachés
  PayPal », un autre pour Stripe, pour lire séparément les frais de conversion. Toute ligne d'écart, réévaluation ou
  conversion qui touche ce compte prend ses comptes propres. Lien avec la comptabilité analytique (projets) :
  **hors périmètre**, tracé pour plus tard.
- **S3 — Avertissement d'archivage** : il **liste** les comptes en devise qui seraient archivés.
- **S4 — Exercice : pas de constante.** Kesh accepte déjà des exercices à dates libres, sans limite de durée (seule
  règle : fin après début). R10 et l'arbitrage A1 sont **retirés** : l'assistant reste tel quel ; les dates viennent de
  l'exercice. Le tableau des cours a **une colonne par mois de l'exercice, du premier au dernier** (un exercice
  raccourci ou allongé lors d'un changement de date de clôture donne moins ou plus de 12 colonnes), plus Ouverture et
  Clôture.
- **S5 — TVA sur un achat en devise.** L'assistant TVA de la saisie (*VatPurchaseAssistant*) calcule la TVA sur le
  montant en devise et passe la ligne de TVA en **CHF** au même cours que la ligne d'achat (les comptes de TVA sont en
  CHF) ; l'écriture reste à une seule devise étrangère. Le décompte TVA (en CHF) n'est pas modifié.
- **S6 — Cours AFC republié.** Si une récupération rend, pour un cours déjà connu, une **valeur différente**
  (correction de l'AFC), l'ancienne valeur **reste** le cours AFC en vigueur ; la nouvelle est gardée comme **cours AFC
  publié à nouveau**, affichée **en rouge** sous la case avec ↑ pour l'adopter. Un **avertissement** s'affiche à la
  connexion tant qu'il n'est pas **acquitté**. Les lignes déjà passées ne changent jamais.
- **Page de revue** : ajouter à la fin un **schéma des tables et relations** neuves (SVG intégré), la rendre
  autonome (envoyable à Guy) et ajouter un bouton **Enregistrer** qui télécharge la page avec les réponses.

### CR-3 — amendements du registre et arbitrages (2026-11-14)

- **C-37-2 complété (S2)** : `accounts.fx_gain_account_id`, `fx_loss_account_id` (BIGINT NULL, sans FK ; NULL = réglage
  de la société, lu à l'usage) ; CHECK « les deux ou aucun » et « seulement sur un compte en devise » ; mêmes gardes
  que la société et gardes inverses (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` + `holderAccountId`) ; résolution unique
  `fx_accounts_for` + revalidation `validate_fx_accounts_in_tx`, miroir TS `fxAccountsFor`. Le compte de conversion
  reste un réglage de la société. Analytique : hors périmètre (C-37-8).
- **C-37-6 amendé (S1)** : quotidien tant que le mois courant ou suivant manque ; mois suivant T obtenu → pause
  jusqu'à min(obtention + 24 j, 25 de T), jamais avant le 1er de T ; `first_fetched_at` sur `_kesh_exchange_rates`.
- **C-37-1 amendé (S6)** : `_kesh_exchange_rates` gagne `first_fetched_at` et les colonnes `republished_*`,
  `notice_acknowledged_at` ; une récupération ne remplace jamais le cours en vigueur ; adoption et acquittement
  audités, globaux à l'installation ; bandeau à la connexion ; cours effectif et `FX_RATE_NOT_AFC` sur le cours en
  vigueur.
- **C-37-7 amendé (S2, S4)** : contrepartie de chaque écriture de réévaluation = comptes propres du compte réévalué,
  sinon ceux de la société ; mois et jour de clôture = ceux de `end_date`, toute durée d'exercice.
- **R10 et A1 retirés (S4)** ; **S3** : liste complète `{id, number, name, currency}` ; **S5** : assistant TVA en devise.
- Registre des routes (deltas) : epic +11 routes `LIB_ROUTES`, total du registre +12.
- **Arbitrages de l'orchestrateur sur les questions de la révision CR-3** (soumis à Beat, page de revue 2) :
  - A9 — S1 : pause jusqu'à min(obtention + 24 j, 25 du mois T), jamais avant le 1er de T (ex. décembre obtenu le
    14.11 → reprise le 08.12) ;
  - A10 — S6 : chaque tour automatique **revérifie aussi le mois courant** (un appel de plus), pour découvrir seul une
    correction de l'AFC ; le bouton « Revérifier les cours publiés » reste ;
  - A11 — S6 : adoption et acquittement valent pour toute l'installation ; les notices affichées sont filtrées sur les
    devises et les exercices multi-devises de la société ;
  - A12 — S2 : écriture touchant deux comptes en devise aux comptes de différence distincts → la ligne d'écart va sur
    ceux du premier compte de bilan en devise, avec un avis ;
  - A13 — S5 : TVA CHF = TVA en devise × même cours, arrondie ; un écart d'un centime est laissé au bouton d'écart ;
  - A14 — S5 : l'assistant TVA reste à la création d'écriture ;
  - A15 — S2 : les comptes propres couvrent gains et pertes ; le compte de conversion reste un réglage de la société ;
  - A16 — S6 : `first_fetched_at` est une colonne distincte de `fetched_at`.
  - A17 — S6 : au `PUT` d'une écriture, seules les lignes `afc` nouvelles ou modifiées sont contrôlées contre le cours
    AFC en vigueur ; une ligne inchangée ne l'est jamais (37-3a AC-4, 37-3b AC-2).

## Revue 2 de Beat — CR-4 (2026-11-14)

Prime sur tout ce qui précède là où il diverge.

- **T1 — Assistant de démarrage.** Pas de constante (déjà retirée par S4). L'assistant **demande la date de début du
  premier exercice** (défaut : 1er janvier de l'année en cours ; fin = veille du même jour un an plus tard,
  modifiable dans *Exercices*) ; le multi-devises se règle **plus tard**, dans les paramètres. Remplace l'ancien A1.
- **T2 — Tests d'exercices** : commençant le **1er janvier**, le 1er avril et le 1er juillet (plus les durées de 6 et
  18 mois de S4).
- **T3 — Devise portée par le compte seul (DB DRY).** La ligne d'écriture **ne stocke pas** la devise : elle se lit
  sur le compte, dont la devise est verrouillée dès la première ligne (37-1). La ligne garde `amount_currency`,
  `exchange_rate`, `rate_source`. Cas admis, par les comptes de l'écriture : **CHF–CHF** ; **devise–CHF** (et l'inverse),
  avec cours ; **devise–même devise** (p. ex. EUR–EUR : chaque ligne a sa valeur CHF au cours AFC du mois ou au cours
  fixé dans l'écriture). Refusé : **deux devises étrangères différentes** (EUR–USD) → passer par le compte de
  conversion (R7). L'API peut recevoir `foreign.currency` ; s'il est présent, il doit égaler la devise du compte.
  Les réponses donnent la devise lue sur le compte.
- **T4 — Cours d'ouverture = cours de clôture précédent.** Pour que les valeurs CHF concordent d'un exercice à
  l'autre, le cours d'ouverture par défaut est **celui de la clôture de l'exercice précédent** : genre « jour »
  → cours journalier de la veille de l'ouverture (= jour de clôture précédent) ; genre « mois » → cours mensuel
  **du dernier mois de l'exercice précédent** (et non plus du premier mois du nouvel exercice). Par défaut, un exercice
  qui suit un exercice Kesh reprend le genre de clôture de celui-ci. Rappel : dans Kesh, les soldes de bilan sont
  cumulés d'un exercice à l'autre (aucune écriture de report) ; le cours d'ouverture ne sert qu'aux soldes
  d'ouverture d'une société neuve et à l'affichage.
- **T5 — Montant en devise « pour information » sur une écriture CHF : rejeté** (Beat : « feature-creep ») ; à tracer
  comme idée (C-37-8), éventuellement pour la réconciliation ou la saisie automatique de factures.
- **T6 — A6 approuvé** (compte de conversion à créer).

### CR-4 — amendements du registre et arbitrages (2026-11-14)

- **C-37-3 amendé (T3)** : `journal_entry_lines` gagne **trois** colonnes NULL : `amount_currency DECIMAL(19,4)`,
  `exchange_rate DECIMAL(20,10)`, `rate_source VARCHAR(12)` ; **aucune colonne de devise** : la devise d'une ligne est
  celle de son compte (`accounts.currency`, lue par jointure), verrouillée dès la première ligne du compte. CHECK
  tout-ou-rien sur les trois colonnes, `chk_jel_fx_rate_source`, `_amount`, `_rate`, `_chf`. Une seule devise
  étrangère par écriture, jugée sur les devises des comptes. `foreign.currency` facultatif à l'API, contrôlé s'il est
  présent (`FX_CURRENCY_MISMATCH`) ; code neuf `FX_ACCOUNT_IN_CHF` (400) pour une ligne `foreign` sur un compte CHF.
  Liste close R1 : `journal_entry_lines` × 3.
- **C-37-9 amendé (T4)** : cours d'ouverture = cours de clôture de l'exercice précédent — `day` : veille de
  l'ouverture ; `month` : mois de la veille de l'ouverture (dernier mois de l'exercice précédent). Un exercice qui
  suit un exercice Kesh prend à sa création le `closing_rate_kind` de celui-ci comme `opening_rate_kind`.
- **C-37-10 complété (T1)** : l'assistant demande la date de **début et de fin** du premier exercice (corps facultatif
  de `finalize` ; sans corps, 1er janvier–31 décembre comme aujourd'hui) — les dates d'un exercice ne se modifient
  pas après sa création (`PUT /fiscal-years/{id}` ne change que le nom, `routes/fiscal_years.rs:86-90`).
- **C-37-8 (T5)** : montant en devise « pour information » sur une écriture CHF — rejeté, gardé comme idée.
- **Arbitrages de l'orchestrateur sur les questions de la révision CR-4** (soumis à Beat) :
  - A18 — la date de fin du premier exercice se saisit dans l'assistant (défaut : veille du même jour un an plus
    tard), puisqu'elle n'est plus modifiable ensuite ;
  - A19 — nom par défaut « Exercice AAAA », ou « Exercice AAAA/AAAA » s'il chevauche deux années (renommable) ;
  - A20 — `FX_ACCOUNT_IN_CHF` reste un code distinct de `FX_CURRENCY_MISMATCH` (messages plus clairs) ;
  - A21 — les écrans envoient toujours la devise (garde contre un compte modifié entre-temps) ; un client API qui
    l'omet renonce à ce contrôle, la devise du compte faisant foi ;
  - A22 — changer plus tard le genre de clôture d'un exercice ne modifie pas le genre d'ouverture de l'exercice
    suivant déjà créé (chacun se règle dans son tableau des cours).

## Revue 3 de Beat — CR-5 (2026-11-14)

Prime sur tout ce qui précède là où il diverge. Beat approuve A2–A22, **sauf A11 et A22** (remplacés par U3 et U4).

- **U1 — Tables des cours renommées et réparties (remplace C-37-1 et l'S6 tel qu'écrit).**
  - **`exchange_rates`** (table **applicative**, **par société**, sauvegardée ; ex-`exchange_rate_overrides`) : une ligne
    par (société, devise, genre `month|day`, date) avec le **cours retenu par la société** et sa **`source`**
    (`afc`, `manual`, et plus tard d'autres sources, p. ex. `ecb`). Elle reçoit **automatiquement** une copie de
    chaque cours récupéré utile à un exercice multi-devises de la société (`source = 'afc'`) ; un cours saisi
    remplace la ligne (`source = 'manual'`). La source d'un cours reste connue **pour toujours**, même pour les
    exercices clos, même quand la source en ligne ne l'offre plus, même après une restauration.
  - **`_kesh_fetched_exchange_rates`** (table **système**, non sauvegardée ; ex-`_kesh_exchange_rates`) : les cours
    **tels que publiés** par la source, une ligne par (`source`, devise, genre, date), la **dernière** valeur publiée.
    Une republication (correction de l'AFC) met à jour cette ligne ; elle ne touche **jamais** `exchange_rates`.
  - **Une seule valeur à l'écran** (corrige la maquette, U1 de Beat) : la case montre le cours de `exchange_rates` ;
    dessous, « AFC ✓ » s'il égale la valeur publiée ; sinon, si la ligne est `afc`, « AFC corrigé : x » **en rouge**
    avec ↑ ; si la ligne est `manual`, « AFC : x » avec ↑. Avertissement à la connexion tant qu'une ligne `afc` diffère
    de la valeur publiée et n'est pas acquittée (acquittement **par société**, mémorisé sur la ligne avec la valeur
    acquittée : une correction suivante avertit de nouveau).
  - Les colonnes `republished_*` et la notion d'adoption globale disparaissent. `FX_RATE_NOT_AFC` compare au cours de
    `exchange_rates` du mois de l'écriture quand sa source est `afc`.
- **U2 — Changer le compte d'une ligne** en éditant une écriture : la devise de la ligne **suit le compte** (elle
  n'est verrouillée qu'au compte). Passer d'un compte USD à un compte EUR réinitialise montant et cours ; vers un compte
  CHF, la ligne redevient CHF.
- **U3 — Corriger un cours AFC (↑) : par société (remplace A11).** Le ↑ ouvre un dialogue : (•) « Corriger pour
  [nom de la société] » (défaut) ; ( ) « Corriger pour toutes les sociétés sous votre gestion (exercices ouverts
  seulement) ». **Constat au code** : un utilisateur Kesh n'appartient qu'à une société (`users.company_id`) ; la
  seconde option n'a donc aujourd'hui aucun objet. Elle est **spécifiée mais masquée** tant que Kesh ne gère pas
  plusieurs sociétés par utilisateur ; à tracer en issue (C-37-8).
- **U4 — Changer le genre de clôture d'un exercice (remplace A22).** Si l'exercice suivant existe, un **dialogue**
  demande quoi faire : appliquer aussi le genre correspondant au **cours d'ouverture** de l'exercice suivant (case
  cochée par défaut) ; Kesh rappelle qu'aucune écriture d'ouverture n'existe entre deux exercices Kesh (soldes
  cumulés) et n'en modifie donc aucune. Si l'exercice suivant a des soldes d'ouverture saisis (premier exercice Kesh
  suivi d'aucun autre : cas impossible ici), sans objet.
- **U5 — Les arbitrages deviennent des AC** de leur story (A2–A22 hors A11, A22 ; U3 et U4). La page de revue les
  montre dans les stories, en bleu seulement s'ils ont changé.
- **U6 — Vocabulaire** : « débit et crédit restent en **devise principale** » (aujourd'hui CHF), jamais « en CHF » seul.
  La page de revue ajoute un **schéma des réglages** (société, exercice, compte).

### CR-5 — amendements du registre et arbitrages (2026-11-14)

- **C-37-1 amendé (U1)** : `exchange_rates` (applicative, par société, sauvegardée ; `rate`, `source`,
  `acknowledged_rate`/`acknowledged_at`, `version`, `updated_by` NULL pour une copie ; UNIQUE (company_id, currency,
  kind, rate_date)) et `_kesh_fetched_exchange_rates` (système ; PK (source, currency, kind, rate_date) ; dernière
  valeur publiée, `first_fetched_at`, `fetched_at`, `value_changed_at` ; plus de `republished_*`). Copie automatique
  en insertion seule (`copy_fetched_in_tx`, 37-2 AC-3 bis) ; cours effectif = ligne de la société
  (`find_company_rate_in_tx`) ; R1 s'applique à `exchange_rates`.
- **C-37-6 complété** : copie non auditée (`updated_by` NULL, nombre au détail de l'action déclenchante) ; les routes
  mutantes de 37-2 sous `retry_on_deadlock`.
- **C-37-9 complété** : `lookup` → `{ currency, kind, rateDate, effectiveRate, source, publishedRate, publishedUnit,
  sourceDate }` ; `FX_RATE_NOT_AFC` contre la ligne `afc` de la société ; notices et acquittement par société.
- **A11, A22 retirés** (U3, U4). Routes 37-2 en deltas : +5 routes, +5 tracées, total +6.
- **Arbitrages de l'orchestrateur sur les questions de la révision CR-5** (soumis à Beat) :
  - A23 — tant que la seconde option de U3 est masquée, le ↑ s'applique **directement** (sans dialogue) ; le dialogue
    n'apparaît que lorsque l'option « toutes les sociétés » devient disponible ; le ↑ d'un cours `manual` n'a pas de
    dialogue ;
  - A24 — la copie dans `exchange_rates` ne porte que sur les **devises des comptes de la société** (pas les 73
    devises) ; donner une devise à un compte déclenche la copie des cours déjà récupérés pour cette devise ;
  - A25 — `applyToNextOpening` absent de la requête = `false` ; l'écran l'envoie coché par défaut ;
  - A26 — `value_changed_at` est gardée (affiche « AFC corrigé le … ») ;
  - A27 — désactiver le multi-devises ne vide pas `exchange_rates` ; *(sans objet depuis CR-7)*
  - A28 — `source` n'a **pas** de CHECK d'énumération (format seul ; valeurs contrôlées par le code), pour qu'une
    source ajoutée plus tard (`ecb`) ne demande pas de migration breaking (P1) ;
  - A29 — 37-4 : « totaux en devise principale », non « en CHF ».
  - A30 — complète A28 : la lecture d'une `source` inconnue est **tolérante** (`RateOrigin::Other(String)`, affichée
    telle quelle, jamais d'erreur interne), dès cette version ; une source ajoutée plus tard (`ecb`) ne demande donc ni
    relèvement de `kesh_version_min_required` ni migration breaking (P1).

## CR-6 — couverture API et MCP (2026-11-14)

Audit de couverture (`implementation-artifacts/37-couverture-api-mcp.md`) : toutes les fonctions ont une route ; les
trous sont des calculs laissés au client Svelte et des contrats non écrits (aucun HIGH). Correctifs retenus, appliqués
aux fiches :

- **V1** (37-3a AC-2, AC-3, AC-4, AC-6 ; 37-3b AC-1, AC-2) — ligne en devise par `foreign { amount, chf }` sans `rate` :
  cours déduit par **une** fonction pure de `kesh-core` (`rate_from_chf`), dont `rateFromChf` de l'écran est le jumeau
  tenu par la fixture partagée `fx_rate_from_chf.json` ; `rateSource` absent → qualifié `afc`/`manual` par le dépôt
  (A30 compris, provenance gardée au `PUT` d'une ligne inchangée).
- **V2** (37-3a AC-6 ; 37-3b AC-1) — `debit`/`credit` facultatifs sur une ligne en devise, posés depuis `foreign` et
  `side` ; présents, contrôle `FX_AMOUNT_MISMATCH` inchangé.
- **V3** (37-3b AC-2 bis ; 37-3a AC-5 ; 37-1 AC-4 bis) — route de conversion : ligne de différence calculée par le
  serveur (`fx_accounts_for` puis `validate_fx_accounts_in_tx`) **seulement sur demande** (`completeDifference: true`,
  A31) ; portée par le corps, sa paire est contrôlée (400) ; sans l'une ni l'autre, `ENTRY_UNBALANCED`. D6 et C-37-4
  tiennent : l'écran la montre toujours à l'aperçu et l'envoie ; la réponse 201 la porte.
- **V4** (37-3a AC-6) — ligne d'écart d'un client de l'API : comptes du compte en devise, sinon de la société
  (`AccountResponse`, `CompanyJson`).
- **V5** (37-3b AC-2) — l'assistant TVA reste un calcul de l'écran ; règle d'arrondi écrite pour un client de l'API.
- **V6** (37-1 AC-3 ; ce fichier) — le multi-devises s'active par une session (`POST /auth/login`, Bearer), jamais par
  une clé d'API ; test d'API en Bearer.
- **V7** (37-5 AC-7) — test d'intégration HTTP de bout en bout de la recette, en Bearer, de l'activation aux rapports.
- **V8** (37-5 AC-2, AC-8 ; 37-3a AC-6 ; ce fichier) — livres importés : la réévaluation se repasse par
  `POST /fiscal-years/{id}/fx-revaluation` avec `rates` (USD 0.91193, EUR 0.94947 au 31.12.2024), non par des
  écritures `revaluation`.
- **V9** (37-3b AC-1) — cours d'ouverture d'un client de l'API : case `opening` de
  `GET /fiscal-years/{id}/exchange-rates`.
- **V10** (37-1 AC-2) — `kesh_exercices` gagne `multi_devises`, `genre_ouverture`, `genre_cloture` (test, mesure de
  taille).
- **V11** (37-1 AC-8) — aucun outil MCP n'expose la société (existant) : hors périmètre, dit.
- **V12** (37-5 AC-5) — MCP : aucun outil pour l'aperçu ; les écritures passées se lisent par `kesh_ecritures`
  (source `revaluation`).
- **V13** (37-4 AC-7) — `kesh_postes_ouverts` inscrit aux sites « CHF seulement », renvoi à 37-3a AC-9.

Arbitrages de l'orchestrateur pendant la validation CR-6 — **approuvés par Beat le 2026-11-14** :

- **A31** (37-3b AC-2 bis, V3) — la ligne de différence d'une conversion n'est **jamais** ajoutée d'office : le serveur
  la calcule seulement si le corps le demande (`completeDifference: true`). Sans ce champ, une écriture déséquilibrée
  rend `ENTRY_UNBALANCED` comme aujourd'hui. Raison : une ligne calculée absorbe en silence toute erreur de montant,
  de cours ou de frais du client en gain ou perte de change — erreur muette, comme le faux rattachement (`CLAUDE.md`,
  *Un appariement automatique propose, il ne crée jamais*). L'écran envoie la ligne qu'il montre à l'aperçu, sans le champ.
- **A32** (37-3a AC-4, 4 ; 37-3b AC-2) — une ligne au **cours déduit** (`{ amount, chf }` sans `rate`), sans
  provenance ou `afc`, prend le cours retenu `afc` de la société pour le mois si `round(montant × cours retenu, 2)`
  égale son CHF : cours exact, provenance `afc`. Sinon cours de `rate_from_chf`, `manual`. Raison : CHF ÷ montant
  ne redonne presque jamais le cours AFC à la décimale près (1125.83 ÷ 1234.56 ≈ 0.9119281 pour 0.91193), si bien que
  des livres tenus au cours AFC (import 2024) partiraient `manual`. L'écran applique la même règle à un CHF saisi.
- **A33** (37-3b AC-2 bis, V3 ; **approuvé par Beat le 2026-11-15**) — frais de conversion sur un compte de charge en
  devise : l'écriture à compléter (écriture 2) admet, en plus de **son** compte en devise au débit net, des lignes de
  **frais** au **débit** de comptes de charge (`account_type = Expense`, `crates/kesh-db/src/entities/account.rs:16`)
  de la **même devise** (p. ex. 6911 *Frais PayPal USD*). Ces lignes n'entrent pas dans la règle « un seul compte en
  devise » (le compte en devise est le seul compte en devise **hors** comptes de charge ; c'est lui qui donne la paire
  `fx_accounts_for`) ; la ligne de différence se calcule **après** elles (Σ débit, frais compris, contre Σ crédit) :
  le frais ne va jamais dans la différence de change. Restent refusés, quand la forme est examinée (écriture complétée
  ou portant une ligne de différence) : une ligne **au crédit** d'un compte de charge en devise → 400 `Validation`
  (`details: { entry, line }`) ; un compte (de charge ou non) d'une **autre** devise → 400 `FX_MULTIPLE_CURRENCIES`
  (`details: { entry, currencies }`), le code que le dépôt rendrait (37-3a AC-4, 2 bis), anticipé au rang de la
  forme. Un frais en CHF (compte de charge sans devise) reste une ligne CHF ordinaire, comptée avant la différence
  comme les autres. Raison : B3/B27 de l'Epic 38 — chaque sous-compte de frais se rapproche du relevé de son prestataire ; un
  frais versé à la différence de change ne s'y rapprocherait plus.

Aucune migration ni route neuve ; contrat d'API étendu sans rupture pour un corps complet (un corps sans `debit` sur
une ligne CHF rend 400 au lieu de 422, déclaré). Les fiches 37-1, 37-3a, 37-3b, 37-4 et 37-5 sont à revalider (cycle
CR-6) ; 37-2 n'est pas touchée.

## Source des cours — établie le 2026-11-14

- Page ESTV *MWST Monatsmittelkurse* : « XML-File » → `https://www.backend-rates.bazg.admin.ch/api/xmlavgmonth`.
- Mois donné : `xmlavgmonth?j=<année>&m=<mois>` (paramètres relevés sur la page d'archive ESTV `/de/dezember-2024`).
- Format : `<monatsmittelkurs><monat>2024-12</monat><devise code="usd"><waehrung>1 USD</waehrung><kurs>0.8846</kurs>…`
  — 73 devises pour 2024-12 ; l'unité est un entier positif (**1, 100, 1000 et 10000** relevés en 2024-12 ;
  `100 JPY` → 0.5751) ; le cours est en CHF.
- Mois non encore publié : élément **vide** (`<monatsmittelkurs …/>`), relevé pour `j=2030&m=1` — à traiter comme
  « pas encore de cours », jamais comme une erreur fatale.
- Aucune variante JSON trouvée (les chemins `/api/json…` rendent 403).
- Conditions d'utilisation : les données viennent de SIX, mises à disposition par l'OFDF « sans engagement » ;
  redistribution par des tiers interdite. Kesh les **récupère chez l'utilisateur** pour son propre usage et ne les
  redistribue pas (aucun cours n'est versionné dans le dépôt ni livré dans l'image).
- Relevé pour la recette : décembre 2024, USD 0.8846, EUR 0.9470 (cours mensuels). Les livres 2024 de Beat ont été
  réévalués à USD 0.91193 et EUR 0.94947 : ce sont les **cours journaliers du 31.12.2024** (D10).
- **Cours journaliers** : `xmldaily?d=AAAAMMJJ` (racine `<wechselkurse>`, `<datum>`, `<gueltigkeit>`, cours à cinq
  décimales, mêmes unités).

## Stories

| Story | Objet | Migration |
|---|---|---|
| 37-1 | réglage multi-devises de la société, devise des comptes (création, modification, verrou dès la première écriture) | oui, non breaking |
| 37-2 | table des cours AFC (mensuels et journaliers) et des cours saisis, récupération OFDF/AFC, écran *Cours de change* éditable | oui, non breaking ; ne referme pas la fenêtre d'import (R1) |
| 37-3a | écritures en devise : modèle de ligne, validation, contrôle central, API, MCP, exports ; garde des chemins d'écriture automatiques | oui, non breaking |
| 37-3b | écritures en devise : soldes d'ouverture, formulaire (cours proposé, ligne d'écart), affichage, sélecteurs, E2E, manuel | non |
| 37-4 | rapports dans les deux devises : grand livre, balance, bilan, compte de résultat, exports | non attendue |
| 37-5 | réévaluation de fin d'exercice (le report à nouveau se fait par le cumul depuis l'origine, aucune écriture de report) | non attendue |

**Ordre** : 37-1 → 37-2 → 37-3a → 37-3b → 37-4 → 37-5 (37-4 peut suivre 37-3a). La 37-3 a été découpée à la
rédaction (règle de splitting préventif : plus de cinq modules). Une story qui découvre qu'il lui faut une migration non prévue le dit
avant de l'écrire. Toutes les migrations attendues sont **non breaking** (`CREATE TABLE`, `ADD COLUMN` nullable ou à
défaut) : `kesh_version_min_required` reste à 0.17.6.

## Exigences (FR/NFR) par story

*(B26, 2026-11-15 : chaque fiche liste ses exigences, tirées de ses critères d'acceptation ; le texte complet est dans la fiche. Guy dira s'il les veut en issues GitHub ou dans les fiches.)*

**37-1** — FR-37-1-1 migration des réglages (AC-1) ; -2 lecture du réglage, des exercices et de `kesh_exercices` (AC-2) ; -3 `PUT currency-settings`, gardes des comptes, activation irréversible (AC-3) ; -4 mode d'un exercice posé à la création et à l'activation, jamais changé ensuite (AC-3 bis) ; -5 devise d'un compte, ses refus et son verrou (AC-4) ; -6 comptes de différence propres (AC-4 bis) ; -7 réinitialisation de la société (AC-5) ; -8 réimport d'une sauvegarde 0.17.6, aller-retour et CSV (AC-6) ; -9 écrans *Paramètres → Comptabilité* et *Plan comptable* (AC-7) ; -10 devise dans `kesh_plan_comptable` (AC-8) ; -11 dates du premier exercice (AC-11) ; -12 gardes des comptes de différence désignés (AC-4, AC-4 bis).
NFR-37-1-1 non breaking (AC-1) ; -2 RBAC du réglage, session et clé d'API (AC-3) ; -3 pas de régression mono-devise, JSON identique (AC-4, AC-4 bis) ; -4 sauvegarde et restauration (AC-5, AC-6) ; -5 tailles MCP (AC-2, AC-8) ; -6 quatre locales (AC-9) ; -7 manuels et CHANGELOG (AC-10) ; -8 audit (AC-3, AC-4 bis) ; -9 verrous et rejeu (AC-3 à AC-4 bis).

**37-2** — FR-37-2-1 tables des cours (AC-1) ; -2 analyse XML, mensuelle et journalière, arrondi à 10 décimales, bornes d'un cours (AC-2) ; -3 `RateSource` dans `AppState`, règle d'écriture, `exchange_rates` jamais modifiée par une récupération (AC-3) ; -4 copie des cours publiés, insertion seule (AC-3 bis) ; -5 récupération des exercices multi-devises ouverts, calendrier (AC-4) ; -6 cours effectif unique, routes de lecture et de récupération (AC-5) ; -7 écran *Cours de change* (AC-6) ; -8 sauvegarde et restauration des cours, réimport sans la table, export CSV (AC-1, AC-7) ; -9 cours de la société et cours d'une ligne (AC-9) ; -10 cours saisi et ↑ (AC-5) ; -11 genres d'ouverture et de clôture, exercice suivant (AC-5, AC-6) ; -12 corrections de l'AFC, notices et acquittement (AC-5, AC-6).
NFR-37-2-1 non breaking (AC-1) ; -2 arrondi déterministe (AC-2) ; -3 appels sortants bornés (AC-3, AC-4, AC-5) ; -4 aucun réseau en test et en CI (AC-3, AC-4, AC-7) ; -5 audit et registre (AC-4, AC-5, AC-9) ; -6 quatre locales et manuels (AC-8) ; -7 RBAC (AC-5) ; -8 source tenue pour hostile (AC-1, AC-2, AC-3).

**37-3a** — FR-37-3a-1 colonnes et contraintes `chk_jel_fx_*` (AC-1) ; -2 `ForeignAmount`, `RateProvenance` (AC-2) ; -3 validation `kesh-core` (AC-3) ; -4 contrôle central et ordre des refus (AC-4) ; -5 chemins automatiques refusés (AC-5) ; -6 `foreign` à l'API (AC-6) ; -7 `PUT` et contre-passation (AC-7) ; -8 CSV, audit, MCP (AC-8) ; -9 limite du lettrage écrite (AC-9) ; -11 qualification de la provenance, cours déduit (AC-2, AC-4) ; -13 réévaluation non modifiable (AC-7) ; -14 colonnes de pièce refusant un compte en devise (AC-5) (-10 et -12 retirées par CR-7).
NFR-37-3a-1 ligne CHF inchangée, JSON et audit compris (AC-1, AC-6, AC-8) ; -2 jamais de 500 sur validation (AC-3) ; -3 concurrence : verrous des comptes et de `companies`, cours lu sans verrou (AC-4) ; -4 audit des lignes en devise (AC-7, AC-8) ; -5 sauvegarde et CSV à 4 et 10 décimales (AC-1, AC-8) ; -6 refus par proposition dans les lots (AC-5).

**37-3b** — FR-37-3b-1 soldes d'ouverture en devise, aucune lecture des cours avant les verrous (AC-1) ; -2 formulaire en devise (AC-2) ; -3 ligne d'écart ajoutée au formulaire et assistant *Conversion* avec ses gardes de désactivation, frais de l'écriture 2 admis sur un compte de charge de la devise (AC-2, AC-2 bis, A33) ; -4 affichage, écriture de réévaluation non modifiable (AC-3) ; -5 sélecteurs sans compte en devise (AC-4) ; -6 cours proposé et provenance (AC-1, AC-2) ; -7 assistant TVA achat en devise (AC-2) ; -8 route de conversion, ses refus, leur ordre et ses verrous, frais sur compte de charge de la devise hors différence (AC-2 bis, A33) ; -9 contrat des routes d'ouverture, 400 et non 422 ni 500 (AC-1) ; -10 refus FX du serveur affichés sur la ligne fautive (AC-1, AC-2, AC-2 bis).
NFR-37-3b-1 pas de régression mono-devise (AC-1, AC-2, AC-5) ; -2 motifs de blocage en quatre locales (AC-3) ; -3 inventaire clos, garde Vitest (AC-4) ; -4 E2E sans réseau (AC-5) ; -5 manuel et guide d'API (AC-6) ; -6 calculs jumeaux de l'écran et du serveur (AC-1, AC-2).

**37-4** — FR-37-4-1 règle du solde en devise, bornes d'exercice, inclusion, `balances_as_of` pour 37-5 (AC-1 à AC-1 quater) ; -2 grand livre (AC-1 bis, AC-2) ; -3 balance (AC-3) ; -4 bilan et compte de résultat (AC-4) ; -5 CSV et PDF des quatre rapports, disposition du PDF, affichage (AC-5) ; -6 JSON, vues, MCP (AC-6) ; -7 journal étendu au JSON et à l'écran seulement, autres rapports et sites hors rapports en CHF (AC-7).
NFR-37-4-1 CSV identique octet pour octet sans compte en devise (AC-5) ; -2 JSON et écran inchangés en mono-devise (AC-6) ; -3 aucun total de rapport en devise étrangère (AC-2 à AC-5) ; -4 champs MCP à `null`, tailles (AC-6) ; -5 quatre locales (AC-6) ; -6 concordance des quatre rapports, `balances_as_of` en transaction, contrôle `grep` T3 (AC-8) ; -7 manuel et PDF (AC-9).

**37-5** — FR-37-5-1 aperçu, sans écriture, `blockers[]` limités à quatre codes, refus de requête et d'état comme la passation (AC-1, AC-4) ; -2 cours retenu par la société selon le genre de clôture (`day` : jour puis mois ; `month` : mois seul), devise sans cours signalée à l'aperçu, réponse par devise (AC-1, AC-2, AC-7) ; -3 passation atomique, une écriture par compte, sens des lignes, serveur recalcule, verrous (AC-2) ; -4 rejouable compte par compte, correction (AC-3) ; -5 gardes en deux temps, ordre des refus, bornes (AC-4) ; -6 action pour un exercice ouvert multi-devises, Comptable et Administrateur, écriture prévue par compte, bouton désactivé tant que `blockers[]` (AC-1, AC-4, AC-5, AC-6).
NFR-37-5-1 RBAC, Consultation 403 (AC-5) ; -2 pas d'outil MCP d'aperçu (AC-5) ; -3 audit `fx_revaluation.posted` et registres (AC-2) ; -4 exactitude CHF = round(devise × cours), cas de signe, recette par l'API seule (AC-1, AC-7) ; -5 aucune récupération de cours par le serveur, aucun réseau en test (AC-1, AC-7) ; -6 manuel (AC-8).

**Total** : 59 FR et 42 NFR (recomptés aux fiches le 2026-11-16 après la révision CR-7 : 12 + 12 + 12 + 10 + 7 + 6 FR,
9 + 8 + 6 + 6 + 7 + 6 NFR ; 37-3a −2 FR). *(Avant CR-7 : 61 FR et 42 NFR, recomptés aux fiches le 2026-11-15 après la validation ciblée P2 ; 60 et 42 après la P1 ; 47 et 37 à la rédaction.)*

## Règles de cet epic

- Méthode du dépôt : `CLAUDE.md` (fiche → validation en boucle → développement → revue en boucle, Sonnet ↔ Opus pour
  les passes complètes, passe ciblée en fin de boucle ; gate local complet avant la PR).
- Les questions de conception sans réponse claire passent par `bmad-party-mode` (5 tours au plus, jusqu'au consensus) ;
  le résultat va au registre ci-dessous.
- **Production** : l'instance de Beat (`kesh-api`, port 8888) tient ses livres ; elle n'est **jamais** écrite par le
  travail de cet epic. Les essais tournent sur des conteneurs et des bases jetables.
- On trace toujours : un défaut voisin non traité se note au registre, pour devenir une issue à l'ouverture de la PR.

## Registre des choix

Architecture tranchée le 2026-11-14 par `bmad-party-mode` (Winston, Amelia, Mary), consensus au **tour 2**, sans
objection bloquante restante. Fait vérifié au code qui a tranché C-37-2 : l'import `.keshbackup` s'exécute avec
`SET FOREIGN_KEY_CHECKS = 0` (`crates/kesh-db/src/backup.rs:689-736`) ; une colonne ajoutée avec défaut ou `NULL` ne
ferme pas la fenêtre d'import, une table applicative neuve la ferme.

- **C-37-1 — Table des cours.** Table **système** globale `_kesh_exchange_rates` (`currency CHAR(3)`, `month DATE`
  au 1er du mois, `rate_per_unit DECIMAL(20,10)`, `published_unit INT`, `published_rate DECIMAL(19,6)`,
  `fetched_at`), PK `(currency, month)`, `CHECK DAY(month) = 1`, source AFC seulement ; état de la récupération dans
  `_kesh_exchange_rate_fetch` (dernier passage, statut, erreur). Non sauvegardée (préfixe `_kesh_`), re-récupérable.
  **Aucun cours manuel mensuel en table** : un cours manuel vit sur la ligne (`rate_source = 'manual'`), audité avec
  l'écriture. Sans cours disponible : erreur `RATE_UNAVAILABLE {currency, month}`, **jamais** de repli à 1.0.
- **C-37-2 — Réglage.** Colonnes sur `companies` : `multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE`,
  `fx_gain_account_id BIGINT NULL`, `fx_loss_account_id BIGINT NULL`, **sans clé étrangère SQL** (cycle
  `companies` ↔ `accounts`, réinitialisation de la société). Les deux peuvent désigner le même compte. Gardes du dépôt
  qui remplacent la FK : (1) à l'écriture du réglage, compte de la société, imputable, actif, de type charge ou produit,
  **en CHF** (jamais un compte en devise), verrou `FOR UPDATE` ; (2) archivage, changement de type ou de devise d'un
  compte désigné refusés (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) ; (3) **revalidation à l'usage** (ligne d'écart,
  réévaluation) : un identifiant orphelin, d'une autre société ou devenu invalide → `FX_ACCOUNT_INVALID`, jamais une
  écriture ; (4) la réinitialisation de la société remet les trois colonnes à leur défaut. Une sauvegarde 0.17.6 se
  réimporte (colonnes à leur défaut).
- **C-37-3 — Ligne d'écriture.** Colonnes `NULL` sur `journal_entry_lines` : `currency CHAR(3)`,
  `amount_currency DECIMAL(19,4)` (≥ 0, même côté que `debit`/`credit`), `exchange_rate DECIMAL(20,10)` (**par
  unité** : 1 JPY, jamais 100 JPY), `rate_source VARCHAR(12) CHECK IN ('afc','manual','revaluation')` — les trois
  valeurs posées d'emblée, une valeur ajoutée plus tard serait breaking. `CHECK` : tout-ou-rien sur les quatre colonnes ;
  `revaluation` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ; `exchange_rate` toujours non `NULL` quand la
  ligne est en devise (objection bloquante de Mary, tour 2 : sans le cours, l'écart ne se recalcule pas depuis
  l'écriture). `debit`/`credit` restent **en CHF** et seuls lus par les rapports (CO 957a al. 4). Égalité « devise de
  la ligne = devise du compte » contrôlée dans le dépôt. Rust : `foreign: Option<ForeignAmount>` sur
  `NewJournalEntryLine` — l'échec de compilation inventorie les sites de construction. Montant CHF arrondi à zéro →
  `FX_LINE_ROUNDS_TO_ZERO` (400, jamais la 500 du `CHECK` existant).
- **C-37-4 — Arrondi et équilibre.** CHF = round(montant × cours, 0.01), `MidpointAwayFromZero`
  (`Money::round_to_centimes`). Le serveur reste strict : déséquilibre → refus, écart dans `details`. L'interface
  **propose** une ligne d'écart préremplie sur le compte de gain ou de perte ; au-delà de n × 0.005 CHF (n lignes en
  devise), elle dit que ce n'est plus un arrondi mais propose quand même (une conversion porte un écart réel, réalisé).
- **C-37-5 — Chemins d'écriture.** Contrôle **central** dans le dépôt `journal_entries` : ligne sans devise sur un
  compte en devise → `ACCOUNT_REQUIRES_CURRENCY` (`FailedProposal` dans un lot). Les sélecteurs des factures,
  règlements et de la réconciliation n'offrent pas les comptes en devise. **Exception étendue** : les soldes
  d'ouverture et leur complément acceptent montant en devise et cours. Devise d'un compte **verrouillée dès sa
  première ligne** ; aucun retour au mono-devise (CR-7, A34).
- **C-37-6 — Audit et réseau.** La récupération automatique n'est pas auditée (`tracing` + `_kesh_exchange_rate_fetch`,
  affiché à l'écran) ; la récupération à la demande est auditée sous l'utilisateur. `reqwest` passe en dépendance de
  production, **rustls**, URL fixe (non configurable), délai et taille de réponse bornés ; un échec ne bloque ni le
  démarrage ni la saisie.
- **C-37-7 — Réévaluation.** Une OD au dernier jour de l'exercice ; comptes **de bilan** en devise présélectionnés,
  décochables (postes non monétaires : stocks, immobilisations) ; cours = cours mensuel du **mois de clôture de
  l'exercice** (décembre pour un exercice civil, décision D5), modifiable par devise dans l'aperçu (devise non
  publiée) ; écart = round(solde devise × cours) − solde CHF, **cumulés depuis l'origine** jusqu'à la date ; rejouable
  (seul le résidu passe) ; pas d'extourne ; refusée sur un exercice clos ou des livres verrouillés ; gains **et**
  pertes (CO 960b), écrit au manuel comme un choix assumé. Ligne du compte réévalué : `rate_source = 'revaluation'`,
  `amount_currency = 0`, `exchange_rate` = cours retenu ; le libellé isole les écarts **non réalisés**.
- **C-37-9 — Ajouts de la validation P1 des fiches (2026-11-14).**
  - `MULTI_CURRENCY_DISABLED` est un **409** partout.
  - **Verrou des deux côtés** (37-1) : un compte qui porte un rôle, est désigné par une configuration (facturation,
    articles, fournisseurs, règles de réconciliation) ou lié à un compte bancaire ne reçoit pas de devise
    (`ACCOUNT_CURRENCY_IN_USE`) ; lier, désigner ou poser un rôle sur un compte en devise est refusé
    (`ACCOUNT_IN_FOREIGN_CURRENCY`). Le contrôle central de 37-3a reste le filet.
  - **C-37-2 (3) amendé** : la revalidation serveur des comptes de différences de change se fait à la réévaluation
    (37-5) ; la ligne d'écart de 37-3b est une ligne CHF ordinaire, contrôlée à l'écran et par les gardes ordinaires.
  - **Saisie manuelle « cours ou CHF »** (37-3b) : l'utilisateur saisit le cours ou le CHF réel ; le cours se déduit à
    10 décimales de sorte que round(montant × cours, 2) = CHF. Cours proposé pour un solde d'ouverture : celui du mois
    **précédant** la date d'ouverture (mois de clôture de l'exercice antérieur).
  - **Provenance contrôlée** (37-3a) : une ligne `afc` doit porter le cours de la table pour son mois
    (`FX_RATE_NOT_AFC`) ; `CHECK chk_jel_fx_chf` en filet de la cohérence CHF ↔ devise.
  - **Lecture et récupération séparées** (37-2) : `GET /lookup` ne sort jamais sur le réseau ; la récupération est
    `POST /exchange-rates/fetch` (Comptable ou Administrateur), appelée par les écrans sur un cours manquant.
  - **Réglage multi-devises : Administrateur seul** (précédent des réglages de la société) ; une clé d'API ne
    l'active pas. *(CR-6, V6 : il s'active par une **session** — `POST /auth/login`, `accessToken` en
    `Authorization: Bearer`, ou le cookie de l'écran —, non « à l'écran » seulement ; `require_not_pat` ne refuse que
    la clé d'API.)*
  - Le cours du mois courant est publié en début de mois (relevé du 2026-11-14).
  - **Sorties MCP** : les champs en devise sont présents à `null` pour une ligne ou un compte CHF (la macro `sortie!`
    n'est pas étendue) ; le JSON des routes HTTP, lui, les omet (`skip_serializing_if`) pour rester identique en
    mono-devise.
  - **PUT d'un compte** : `currency` absent = devise inchangée, `null` = CHF (compatibilité des clients mono-devise ;
    renversement de la P1 par la P2 de 37-1, confirmé par la P3).
- **CR-1 — amendements du registre (2026-11-14, révision des fiches pour D9, D10 et la terminologie).**
  - **C-37-1 amendé** : `_kesh_exchange_rates` ne porte que les **cours AFC bruts**, mensuels et journaliers (`kind`
    'month' | 'day', `rate_date`, `source_date` = `<datum>` lu pour un jour ; PK `(currency, kind, rate_date)`), non
    sauvegardée, re-récupérable. Les **cours saisis** (D9) vont dans la table **applicative** `exchange_rate_overrides`
    (company_id, currency, kind, rate_date, rate, version, updated_by ; UNIQUE (company_id, currency, kind,
    rate_date)), sauvegardée, exportée en CSV, vidée à la remise à zéro. Cours effectif = cours saisi ?? cours AFC
    (`find_effective_rate_in_tx`, seule fonction) ; une récupération ne touche jamais aux cours saisis.
    ⚠️ **La table applicative neuve referme la fenêtre d'import** : un `.keshbackup` 0.17.6 ou antérieur ne se
    réimporte plus dans la nouvelle version (registre de rejeu vidé : `20261106000001` passe « Hors fenêtre »). À
    écrire au CHANGELOG et au manuel d'administration, et **à confirmer avec Guy** (production) ; soumis à Beat dans la
    page de revue.
  - **C-37-7 amendé (D10)** : cours par défaut de la réévaluation = cours effectif journalier de la date de fin
    d'exercice (`xmldaily`, `POST /exchange-rates/fetch { date }`), sinon cours effectif mensuel du mois de fin
    (`rateKind` signalé), sinon `RATE_UNAVAILABLE` ; modifiable par devise. Un jour sans publication (week-end) prend
    la publication antérieure (`xmldaily?d=D` rend la dernière `<datum>` ≤ D, relevé le 2026-11-14).
  - **C-37-9 complété** : une ligne prend le cours effectif ; `rate_source` = `afc` si égal au cours AFC mensuel du
    mois, sinon `manual` ; `FX_RATE_NOT_AFC` compare toujours au cours AFC brut ; `FX_RATE_INVALID` et la fonction des
    bornes d'un cours sont créés par 37-2 ; `lookup` rend `{ currency, kind, rateDate, effectiveRate, afcRate|null,
    overridden, publishedUnit, publishedRate, sourceDate }`.
  - **Compte des passes** : le changement de périmètre ouvre un **nouveau cycle de validation** (CR-1 P1, P2…) pour
    37-2, 37-3a, 37-3b et 37-5, plafonné à huit passes comme tout cycle ; 37-1 (annotation) et 37-4 (inchangée)
    restent validées.
- **C-37-8 (complément 2)** : course préexistante, antérieure à l'Epic 37 — archivage d'un compte parent contre
  création d'un sous-compte (relevée à la validation P5 de 37-1) : issue à ouvrir avec la PR.
- **C-37-8 (complément)** : lettrage d'un compte en devise (débiteur soldé en devise avec écart de change) — limite,
  issue à ouvrir avec la PR.
- **C-37-8 — Tracé pour plus tard** (issues à ouvrir avec la PR) : option de neutralisation des gains de change non
  réalisés par une provision (prudence, CO 960 al. 2) ; factures, factures fournisseur et imports bancaires en devise.

## CR-7 — revue de Guy (2026-11-16), approuvée par Beat

Revue 4 de Guy (page `epic-35-revue.html`, 35-N = 37-N) : tout approuvé, 37-5 sans avis. Décisions retenues :

- **A34 — Multi-devises sans retour (37-1).** Proposé par Guy, approuvé par Beat : une fois activé, le multi-devises
  reste activé (confirmation à l'activation) ; tout exercice ouvert ou créé ensuite est multi-devises. **Retirés** : la
  désactivation de la société, la bascule d'un exercice vers le mono-devise, l'avertissement des comptes en devise
  actifs et l'archivage forcé (`force`), et ce qui n'existait que pour eux. Remplace la décision de Beat d'un mode par
  exercice (révisions CR-1/CR-2).
- **A35 — Sous-comptes.** Beaucoup de comptes attendus (un par solde et par devise, sous-comptes de frais). Kesh gère
  déjà comptes parents et sous-comptes ; la forme du numéro (point ou non) est laissée au plan de l'utilisateur, aucun
  travail ici.
- **A36 — KISS (Beat).** Toute révision de ce cycle retire plutôt qu'elle n'ajoute ; ce qui se simplifie par A34 se
  simplifie.
- Remarque de Guy (Bexio : devise et CHF par ligne, cours calculé) : déjà couverte par V1/A32 ; la devise reste celle
  du compte (T3).
- **Application d'A34 (2026-11-16, Opus ; modèle écrit en tête de 37-1, « Révision CR-7 »).** Le réglage de la
  société devient irréversible (refus 409 `MULTI_CURRENCY_IRREVERSIBLE`, code neuf ; confirmation à l'écran) ;
  `fiscal_years.multi_currency` est gardée, posée à la création et à l'activation seulement — seule trace des exercices
  clos à l'activation, qui restent mono-devise même rouverts ; schéma, liste close des absences admises et fenêtre
  d'import inchangés. **Retirés** : route `PUT /fiscal-years/{id}/currency-mode` (`set_currency_mode`, `force`, audit,
  rejeu), codes `MULTI_CURRENCY_IN_USE`, `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`,
  désactivation, `archive_in_tx`, garde de `reactivate`, écran *Exercices* (colonne, bascule, dialogue), copie des
  cours au passage en multi-devises (37-2), garde `laterFiscalYear` et motif `monoCurrencyFiscalYear` (37-3a, 37-3b,
  37-5), masquage par exercice à l'écran (37-3b), A4, A27, A5 hormis l'activation. Registre des routes : 37-1 +1
  route (au lieu de +2), point (vi) « vingt-sept ». Exigences : 61 → 59 FR (37-3a −2), 42 NFR.
- **Points à faire valider** : (1) un exercice créé après l'activation dans un trou devant un exercice clos avant
  l'activation puis rouvert reste admis (limite écrite en 37-1) ; (2) le verrou de `companies` de 37-3a (pris quand une
  ligne porte `foreign`) pourrait être retiré — une société ne redevient jamais mono-devise et la devise d'un compte,
  lue en partage, suffit —, non appliqué ici (texte de concurrence à reprendre).
