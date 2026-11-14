# Epic 35 — Comptabilité multi-devises

**Kickoff : 2026-11-14.** Demande de Beat (contributeur, comptabilité d'une SA tenue jusqu'ici dans un logiciel
multi-devises : comptes PayPal, Wise et débiteurs en USD et en EUR, ventes en USD, réévaluation au 31.12 sur le compte
6840 *Différences de change*). L'epic est développé **sur une branche unique** (`epic/35-multi-devises`) et proposé à
Guy en **une seule PR** à la fin, après une recette sur l'exercice 2024 de Beat importé par l'API.

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
- **D3 — Le cours par défaut est le cours mensuel moyen officiel de l'AFC** (ESTV, publié par l'OFDF/BAZG) du mois de la
  date de l'écriture ; il est **modifiable ligne par ligne** (saisie manuelle, p. ex. pour une conversion de devises).
- **D4 — Les cours mensuels se récupèrent automatiquement** sur le site officiel, et **toutes les devises publiées**
  sont stockées dans la table des cours mensuels (précision de Beat, 2026-11-14) — USD et EUR pour Beat, mais
  l'ensemble est universel.
- **D5 — Réévaluation de fin d'exercice** : au **cours mensuel de décembre** (le mois de clôture de l'exercice), dans
  les deux sens (gain et perte), contre un compte de différences de change choisi par l'utilisateur. Fondement : les
  devises ont un cours de marché observable (CO 960b).
- **D6 — Conversion de devises** (EUR → USD) : les deux montants en devise sont saisis avec leurs cours ; l'écart en CHF
  est **proposé** comme ligne de différence de change, jamais forcé en silence.
- **D7 — Périmètre** : (a) réglage, devise des comptes, table des cours et récupération ; (b) écritures en devise ;
  (c) grand livre, balance, bilan et compte de résultat dans les deux devises ; (d) réévaluation. **Hors périmètre** :
  factures, factures fournisseur, imports bancaires (CAMT, CSV) en devise — l'API suffit pour l'instant.

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
- Relevé pour la recette : décembre 2024, USD 0.8846, EUR 0.9470. Les livres 2024 de Beat ont été réévalués à
  USD 0.91193 et EUR 0.94947 (autre source) : l'écart sera chiffré à la recette.

## Stories

| Story | Objet | Migration |
|---|---|---|
| 35-1 | réglage multi-devises de la société, devise des comptes (création, modification, verrou dès la première écriture) | oui, non breaking |
| 35-2 | table des cours mensuels, récupération OFDF/AFC (à la demande et mensuelle automatique), écran *Cours de change* (le cours manuel vit sur la ligne, C-35-1) | oui, non breaking |
| 35-3a | écritures en devise : modèle de ligne, validation, contrôle central, API, MCP, exports ; garde des chemins d'écriture automatiques | oui, non breaking |
| 35-3b | écritures en devise : soldes d'ouverture, formulaire (cours proposé, ligne d'écart), affichage, sélecteurs, E2E, manuel | non |
| 35-4 | rapports dans les deux devises : grand livre, balance, bilan, compte de résultat, exports | non attendue |
| 35-5 | réévaluation de fin d'exercice (le report à nouveau se fait par le cumul depuis l'origine, aucune écriture de report) | non attendue |

**Ordre** : 35-1 → 35-2 → 35-3a → 35-3b → 35-4 → 35-5 (35-4 peut suivre 35-3a). La 35-3 a été découpée à la
rédaction (règle de splitting préventif : plus de cinq modules). Une story qui découvre qu'il lui faut une migration non prévue le dit
avant de l'écrire. Toutes les migrations attendues sont **non breaking** (`CREATE TABLE`, `ADD COLUMN` nullable ou à
défaut) : `kesh_version_min_required` reste à 0.17.6.

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
objection bloquante restante. Fait vérifié au code qui a tranché C-35-2 : l'import `.keshbackup` s'exécute avec
`SET FOREIGN_KEY_CHECKS = 0` (`crates/kesh-db/src/backup.rs:689-736`) ; une colonne ajoutée avec défaut ou `NULL` ne
ferme pas la fenêtre d'import, une table applicative neuve la ferme.

- **C-35-1 — Table des cours.** Table **système** globale `_kesh_exchange_rates` (`currency CHAR(3)`, `month DATE`
  au 1er du mois, `rate_per_unit DECIMAL(20,10)`, `published_unit INT`, `published_rate DECIMAL(19,6)`,
  `fetched_at`), PK `(currency, month)`, `CHECK DAY(month) = 1`, source AFC seulement ; état de la récupération dans
  `_kesh_exchange_rate_fetch` (dernier passage, statut, erreur). Non sauvegardée (préfixe `_kesh_`), re-récupérable.
  **Aucun cours manuel mensuel en table** : un cours manuel vit sur la ligne (`rate_source = 'manual'`), audité avec
  l'écriture. Sans cours disponible : erreur `RATE_UNAVAILABLE {currency, month}`, **jamais** de repli à 1.0.
- **C-35-2 — Réglage.** Colonnes sur `companies` : `multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE`,
  `fx_gain_account_id BIGINT NULL`, `fx_loss_account_id BIGINT NULL`, **sans clé étrangère SQL** (cycle
  `companies` ↔ `accounts`, réinitialisation de la société). Les deux peuvent désigner le même compte. Gardes du dépôt
  qui remplacent la FK : (1) à l'écriture du réglage, compte de la société, imputable, actif, de type charge ou produit,
  **en CHF** (jamais un compte en devise), verrou `FOR UPDATE` ; (2) archivage, changement de type ou de devise d'un
  compte désigné refusés (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) ; (3) **revalidation à l'usage** (ligne d'écart,
  réévaluation) : un identifiant orphelin, d'une autre société ou devenu invalide → `FX_ACCOUNT_INVALID`, jamais une
  écriture ; (4) la réinitialisation de la société remet les trois colonnes à leur défaut. Une sauvegarde 0.17.6 se
  réimporte (colonnes à leur défaut).
- **C-35-3 — Ligne d'écriture.** Colonnes `NULL` sur `journal_entry_lines` : `currency CHAR(3)`,
  `amount_currency DECIMAL(19,4)` (≥ 0, même côté que `debit`/`credit`), `exchange_rate DECIMAL(20,10)` (**par
  unité** : 1 JPY, jamais 100 JPY), `rate_source VARCHAR(12) CHECK IN ('afc','manual','revaluation')` — les trois
  valeurs posées d'emblée, une valeur ajoutée plus tard serait breaking. `CHECK` : tout-ou-rien sur les quatre colonnes ;
  `revaluation` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ; `exchange_rate` toujours non `NULL` quand la
  ligne est en devise (objection bloquante de Mary, tour 2 : sans le cours, l'écart ne se recalcule pas depuis
  l'écriture). `debit`/`credit` restent **en CHF** et seuls lus par les rapports (CO 957a al. 4). Égalité « devise de
  la ligne = devise du compte » contrôlée dans le dépôt. Rust : `foreign: Option<ForeignAmount>` sur
  `NewJournalEntryLine` — l'échec de compilation inventorie les sites de construction. Montant CHF arrondi à zéro →
  `FX_LINE_ROUNDS_TO_ZERO` (400, jamais la 500 du `CHECK` existant).
- **C-35-4 — Arrondi et équilibre.** CHF = round(montant × cours, 0.01), `MidpointAwayFromZero`
  (`Money::round_to_centimes`). Le serveur reste strict : déséquilibre → refus, écart dans `details`. L'interface
  **propose** une ligne d'écart préremplie sur le compte de gain ou de perte ; au-delà de n × 0.005 CHF (n lignes en
  devise), elle dit que ce n'est plus un arrondi mais propose quand même (une conversion porte un écart réel, réalisé).
- **C-35-5 — Chemins d'écriture.** Contrôle **central** dans le dépôt `journal_entries` : ligne sans devise sur un
  compte en devise → `ACCOUNT_REQUIRES_CURRENCY` (`FailedProposal` dans un lot). Les sélecteurs des factures,
  règlements et de la réconciliation n'offrent pas les comptes en devise. **Exception étendue** : les soldes
  d'ouverture et leur complément acceptent montant en devise et cours. Devise d'un compte **verrouillée dès sa
  première ligne** ; retour au mono-devise refusé tant qu'un compte en devise existe.
- **C-35-6 — Audit et réseau.** La récupération automatique n'est pas auditée (`tracing` + `_kesh_exchange_rate_fetch`,
  affiché à l'écran) ; la récupération à la demande est auditée sous l'utilisateur. `reqwest` passe en dépendance de
  production, **rustls**, URL fixe (non configurable), délai et taille de réponse bornés ; un échec ne bloque ni le
  démarrage ni la saisie.
- **C-35-7 — Réévaluation.** Une OD au dernier jour de l'exercice ; comptes **de bilan** en devise présélectionnés,
  décochables (postes non monétaires : stocks, immobilisations) ; cours = cours mensuel du **mois de clôture de
  l'exercice** (décembre pour un exercice civil, décision D5), modifiable par devise dans l'aperçu (devise non
  publiée) ; écart = round(solde devise × cours) − solde CHF, **cumulés depuis l'origine** jusqu'à la date ; rejouable
  (seul le résidu passe) ; pas d'extourne ; refusée sur un exercice clos ou des livres verrouillés ; gains **et**
  pertes (CO 960b), écrit au manuel comme un choix assumé. Ligne du compte réévalué : `rate_source = 'revaluation'`,
  `amount_currency = 0`, `exchange_rate` = cours retenu ; le libellé isole les écarts **non réalisés**.
- **C-35-9 — Ajouts de la validation P1 des fiches (2026-11-14).**
  - `MULTI_CURRENCY_DISABLED` est un **409** partout.
  - **Verrou des deux côtés** (35-1) : un compte qui porte un rôle, est désigné par une configuration (facturation,
    articles, fournisseurs, règles de réconciliation) ou lié à un compte bancaire ne reçoit pas de devise
    (`ACCOUNT_CURRENCY_IN_USE`) ; lier, désigner ou poser un rôle sur un compte en devise est refusé
    (`ACCOUNT_IN_FOREIGN_CURRENCY`). Le contrôle central de 35-3a reste le filet.
  - **C-35-2 (3) amendé** : la revalidation serveur des comptes de différences de change se fait à la réévaluation
    (35-5) ; la ligne d'écart de 35-3b est une ligne CHF ordinaire, contrôlée à l'écran et par les gardes ordinaires.
  - **Saisie manuelle « cours ou CHF »** (35-3b) : l'utilisateur saisit le cours ou le CHF réel ; le cours se déduit à
    10 décimales de sorte que round(montant × cours, 2) = CHF. Cours proposé pour un solde d'ouverture : celui du mois
    **précédant** la date d'ouverture (mois de clôture de l'exercice antérieur).
  - **Provenance contrôlée** (35-3a) : une ligne `afc` doit porter le cours de la table pour son mois
    (`FX_RATE_NOT_AFC`) ; `CHECK chk_jel_fx_chf` en filet de la cohérence CHF ↔ devise.
  - **Lecture et récupération séparées** (35-2) : `GET /lookup` ne sort jamais sur le réseau ; la récupération est
    `POST /exchange-rates/fetch` (Comptable ou Administrateur), appelée par les écrans sur un cours manquant.
  - **Réglage multi-devises : Administrateur seul** (précédent des réglages de la société) ; une clé d'API ne
    l'active pas.
  - Le cours du mois courant est publié en début de mois (relevé du 2026-11-14).
  - **Sorties MCP** : les champs en devise sont présents à `null` pour une ligne ou un compte CHF (la macro `sortie!`
    n'est pas étendue) ; le JSON des routes HTTP, lui, les omet (`skip_serializing_if`) pour rester identique en
    mono-devise.
  - **PUT d'un compte** : `currency` absent = devise inchangée, `null` = CHF (compatibilité des clients mono-devise ;
    renversement de la P1 par la P2 de 35-1, confirmé par la P3).
- **C-35-8 (complément)** : lettrage d'un compte en devise (débiteur soldé en devise avec écart de change) — limite,
  issue à ouvrir avec la PR.
- **C-35-8 — Tracé pour plus tard** (issues à ouvrir avec la PR) : option de neutralisation des gains de change non
  réalisés par une provision (prudence, CO 960 al. 2) ; factures, factures fournisseur et imports bancaires en devise.
