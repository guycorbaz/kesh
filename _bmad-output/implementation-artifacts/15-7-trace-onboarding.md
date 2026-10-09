# Story 15.7 : La piste de contrôle de l'onboarding

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni inventaire.** Elle ne garde
que les pointeurs vers ses sous-stories et l'historique des passes qui ont conduit aux découpages.
*(La définition du statut `split` l'impose ; précédents : 15-1, 15-5, 17-2.)* La version complète
d'avant découpage se lit au commit `cabda169`.

## Les sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-7a** | `15-7a-trace-installation-production.md` | **découpée à la P2** (C-15-7-19) en 15-7a1 + 15-7a2 — corps vidé, pointeurs seuls | — |
| **15-7a1** | `15-7a1-socle-transactions-onboarding.md` | Le **socle** `kesh-db`, sans changement de comportement ni d'audit : variantes `_in_tx` (`accounts`, `bank_accounts`, `company_invoice_settings` — fin du MIRROR —, `vat_rates` ligne à ligne), `companies::clear_stub_in_tx`, `onboarding::lock_state_in_tx` | `refs #434` |
| **15-7a2** | `15-7a2-trace-installation-production.md` | Les **neuf routes de production** (`language`, `mode`, `start-production`, `org-type`, `accounting-language`, `coordinates`, `bank-account`, `skip-bank`, `finalize`) : une transaction par route, `lock_state_at_step`, `installation.step_completed` par un helper unique, entrées de domaine (dont `account.chart_loaded` agrégée et `company_invoice_settings.created`), quatre actions et leurs libellés, registre (94 → 103 tracées), manuel | `refs #434` |
| **15-7b** | `15-7b-trace-demo-et-remise-a-zero.md` | **découpée à la P4** (C-15-7-31) en 15-7b1 (Volet A) + 15-7b2 (Volet B) — corps vidé, pointeurs seuls | — |
| **15-7b1** | `15-7b1-trace-demonstration.md` | **`seed-demo`** : `installation.demo_seeded` puis l'étape dans la dernière transaction du peuplement, acteur threadé, dernière transaction rejouée sur interblocage ; une action, registre (104 → 105, recompté au T0 de la 15-7b1), manuel — dont le § *Chemin A* qui décrivait une démonstration inexistante (#544) | `refs #434` ; `closes #544` |
| **15-7b2** | `15-7b2-remise-a-zero.md` | **`reset`** : remise à zéro en **une** transaction rejouée sur interblocage, connexion fermée à la libération, trois gardes sous son verrou, `installation.reset` en dernier ; **identité de la société préservée** ; **une règle et une fonction pour les principaux orphelins** (`reattach_orphan_principals_in_tx` : utilisateurs rattachés, clés d'API actives orphelines **révoquées puis** repointées — C-15-7-45), y compris sur une base déjà sans société (la **cause** de #528) ; vidage **dérivé de la liste canonique**, partition gardée sur le schéma par trois règles (#279) ; une action, registre (105 → 106, recompté au T0 de la 15-7b1), manuel — dont la sortie de la démonstration, qui exige `KESH_PRODUCTION_RESET` (C-15-7-48) ; **bootstrap** : une société provisoire seulement sur base sans société (la **cause** de #542) | `closes #434`, `closes #279` ; `refs #528`, `refs #542`, `refs #540`, `refs #538`, `refs #534` |
| **15-7b3** | `15-7b3-reparation-des-installations-atteintes.md` | **Réparation des installations déjà atteintes par #528 et #542**, née de la P3 de la 15-7b2 (C-15-7-40) : `repair_installation_in_tx`, au **démarrage** (non bloquante en cas d'échec, C-15-7-47) et dans la transaction de **restauration** d'une sauvegarde ; principaux orphelins par la fonction de la 15-7b2 (clés révoquées quel que soit le nombre de sociétés, C-15-7-45) ; sociétés provisoires superflues supprimées au démarrage (C-15-7-46) ; entrée `installation.repaired` ; une action, manuels, `docs/api-external.md`, CHANGELOG | `closes #528`, `closes #542` ; `refs #546` |

⚠️ **L'ordre n'est pas indifférent** : 15-7a1 → 15-7a2 → 15-7b1 → 15-7b2 → 15-7b3, chacune après le merge de la
précédente ; la 15-7b3 vise la même release (v0.13.0), sans que ce soit une contrainte de sûreté (C-15-7-44). ⚠️ **La 15-7a2, la 15-7b1 et la 15-7b2 partent dans la même release** (C-15-7-25, C-15-7-31) : entre les deux,
une remise à zéro à l'étape ≤ 2 effacerait sans trace les entrées neuves de la 15-7a2.

### Décompte de modules (signal D5 déclaré au Project Lead)

| fiche | modules de code, granularité C-15-7-8 | seuil |
|---|---|---|
| 15-7a1 | 6 — six repositories `kesh-db` ; aucun appel de `finalize` à adapter (C-15-7-30) | franchi ; **story-zéro** (quatre extractions, deux ajouts ponctuels), signal déclaré |
| 15-7a2 | 9 touchés — 4 de code (`routes/onboarding`, `audit_labels`, `kesh-i18n`, `repositories/onboarding`) + 5 de commentaires seuls (`repositories/{accounts, vat_rates, fiscal_years}`, `routes/{profile, companies}`) | franchi au sens de la règle (F3-5) ; signal déclaré, non découpée (aucune ligne exécutable hors des quatre) |
| 15-7b1 | 7 touchés — 4 de code (`kesh-seed`, `routes/onboarding`, `audit_labels`, `kesh-i18n`) + 3 de commentaires seuls (`repositories/{vat_rates, company_invoice_settings, fiscal_years}`) | franchi au sens de la règle ; signal déclaré, non découpée (aucune ligne exécutable hors des quatre) |
| 15-7b2 | 8 — `companies`, `onboarding`, `backup` (`kesh-db`), `kesh-seed`, `routes/onboarding`, `auth/bootstrap`, `audit_labels`, `kesh-i18n` (+ un test de composant frontend) | franchi ; née du découpage de la 15-7b (C-15-7-31) ; dérogation écrite, **coupe prédéclarée** (story-zéro `kesh-db`) — déclenchée à la P3, faite **selon la cause** (C-15-7-40) : la réparation sort en 15-7b3 ; huit modules restent |
| 15-7b3 | 5 — `repositories/companies` (`kesh-db` : réparation, sociétés superflues), `auth/bootstrap`, `routes/admin`, `audit_labels`, `kesh-i18n` ; la règle des principaux est **appelée** (15-7b2) | non franchi (« plus de cinq ») ; recompté après la P1 (#542 ajoutée, C-15-7-46) et après la P2 (références lues dans `information_schema`, dans `companies` — C-15-7-50) |

### Attribution des doc-comments rendus faux (grep élargi de la P2)

Grep d'union exécuté le 2026-10-08 sur `crates` et `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (motifs des
AC 8 de la 15-7a1, 12 de la 15-7a2 et 10 de la 15-7b — réparti à la P4 entre 15-7b1 et 15-7b2). Chaque site a **une** fiche, sauf celui que
signale la note sous le tableau ; « légitime » = reste vrai, à laisser.

| site | fiche |
|---|---|
| `kesh-db/src/repositories/company_invoice_settings.rs:522-524`, `:558`, `:694` (duplication, `MIRROR` d'`insert_with_defaults`) | 15-7a1 |
| `company_invoice_settings.rs:11-17`, `:69`, `:101` ; `company_dunning_settings.rs:35`, `:46` (`MIRROR` d'autres fonctions) | légitime |
| `kesh-api/src/routes/onboarding.rs:523-527` (`TODO(L65`), `:999-1003` (`is_stub = FALSE` inconditionnel) | 15-7a2 |
| `kesh-api/src/routes/companies.rs:233` | 15-7a2 |
| `kesh-db/src/repositories/accounts.rs:8`, `:932-934` ; `vat_rates.rs:284` ; `fiscal_years.rs:200-203` (référence à `bulk_create_from_chart`) | 15-7a2 |
| `kesh-db/src/repositories/onboarding.rs:76-83` ; `kesh-api/src/routes/profile.rs:47-49` | 15-7a2 |
| `docs/MULTI-TENANT-SCOPING-PATTERNS.md:298-302` (liste ordonnée), `:312` (« company only ») | 15-7a2 |
| `kesh-seed/src/lib.rs:70`, `:80-93`, `:155-156`, `:167` | 15-7b1 |
| `kesh-seed/src/lib.rs:228-233` (doc de `reset_demo`) | 15-7b2 |
| `kesh-db/src/repositories/audit_log.rs:16-23` | 15-7b2 |
| `kesh-api/src/lib.rs:302-311` (route `reset`, « second des trois chemins ») | 15-7b2 |
| `kesh-api/src/routes/onboarding.rs:204-216` (handler `seed_demo`) | 15-7b1 |
| `kesh-api/src/routes/onboarding.rs:62`, `:225-231`, `:232-239`, `:281-293`, `:866-869` ; `kesh-api/src/errors.rs:282` (R5-2 de la P5 de la 15-7b2) | 15-7b2 |
| `kesh-db/src/entities/company.rs:174-179` | 15-7b2 |
| `kesh-db/src/repositories/vat_rates.rs:323-324` ; `company_invoice_settings.rs:516`, `:588-598` (dont la raison d'être du rejeu) | 15-7b1 |
| `kesh-db/src/repositories/onboarding.rs:132-135` ; `accounts.rs:1020-1021` | 15-7b2 |
| `kesh-db/src/repositories/fiscal_years.rs:199-204` (`create_for_seed`) | 15-7b1 |
| `kesh-api/src/helpers.rs:55-58` (« FK RESTRICT » de `users.company_id`, faux : `ON DELETE CASCADE`) ; `kesh-api/src/auth/bootstrap.rs:72-73` (cas 1) ; `.env.example:273-279`, `docker-compose.dev.yml:28-29` (`KESH_PRODUCTION_RESET` ; la sortie de la démonstration l'exige, C-15-7-48) | 15-7b2 |
| `kesh-api/src/auth/bootstrap.rs:31-37` (constantes descendues) ; doc d'`ensure_admin_user` (part #542 : le stub n'est inséré que sur une base sans société) ; `frontend/src/routes/onboarding/+layout.svelte:17-19` (« placeholder par le bootstrap ») | 15-7b2 |
| doc d'`ensure_admin_user` (part #528 et #542 : réparation au démarrage, son acteur, non bloquante) ; matrice `//!` du module `auth/bootstrap.rs:1-17` (cas 3 « no-op », « tolérant aux race conditions ») ; `kesh-api/src/routes/admin.rs`, `run_backup_and_restore` (étape « 5-quater ») | 15-7b3 |
| `docs/MULTI-TENANT-SCOPING-PATTERNS.md:314` (ligne `seed_demo`), `:320-330` (part `seed_demo`, renvoi à #538 et non à #43) | 15-7b1 |
| `docs/MULTI-TENANT-SCOPING-PATTERNS.md:313` (ligne `reset`, ordre complet), `:320-330` (part remise à zéro), liste d'exceptions `:330-336` | 15-7b2 |
| `auth/bootstrap.rs:128` ; `routes/onboarding.rs:597`, `:608` ; `retry.rs:7` ; `reconciliation.rs:3831` ; `test_fixtures.rs:176` ; `journal_entries.rs:1189-1190` | légitime |

⚠️ `fiscal_years.rs:199-204` est partagé : la 15-7a2 corrige la référence à `bulk_create_from_chart`,
la 15-7b1 la phrase sur la démonstration — deux retouches successives d'un même doc-comment.

## Pourquoi le découpage

Les deux lentilles de la passe P1 (R7, F-5) ont relevé que le décompte « cinq modules, juste au
seuil » de C-15-7-1 n'était reproductible sous **aucune** granularité : il comptait `kesh-api` deux
fois et `kesh-db` une fois pour cinq repositories. Par crate on en trouve quatre ; à la granularité
des exemples de la règle (`kesh-api/routes/invoices`, `kesh-core/accounting`), neuf — routes
d'onboarding, libellés d'audit, cinq repositories, `kesh-seed`, `kesh-i18n`. L'orchestrateur a retenu
la granularité des exemples, qui franchit le seuil, et appliqué le découpage que C-15-7-1 nommait
déjà comme son retour arrière (choix **C-15-7-8**). La 15-7b prend la partie au risque le plus
élevé — la refonte transactionnelle de la remise à zéro —, élargie à #528 et #279 par décision de
l'orchestrateur. Les remédiations de la passe P1 ont été appliquées **dans les deux fiches filles**,
pas ici.

**Second découpage, à la passe P2** (R2-3, F-1) : appliquée à la seule 15-7a, la même granularité
donnait encore **neuf** modules, sans section de dérogation — la seule dérogation que le `CLAUDE.md`
codifie, le cycle de dépendance Cargo, est absente ici. L'orchestrateur a appliqué le patron
« story-zéro + rollout » (choix **C-15-7-19**) : 15-7a1 pose les extractions mécaniques sans changer
de comportement, 15-7a2 porte les routes, l'audit et la documentation.

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C-15-7-1 | ~~pas de découpage~~ — **renversé** par C-15-7-8 | — |
| C-15-7-2 | chaque étape franchie s'inscrit (`installation.step_completed`) | 15-7a2 (pose), 15-7b1, 15-7b2 (emploient) |
| C-15-7-3 | plan comptable en une entrée agrégée | 15-7a2 |
| C-15-7-4 | démonstration en une entrée de synthèse | 15-7b1 |
| C-15-7-5 | la remise à zéro efface la piste et y inscrit son geste | 15-7b2 |
| C-15-7-6 | une transaction par route, variantes `_in_tx` | 15-7a1 (variantes), 15-7a2, 15-7b1 et 15-7b2 (transactions) |
| C-15-7-7 | finalisation : ce qui a été inséré | 15-7a1 (rendus), 15-7a2 (entrées) |
| C-15-7-8 | découpage 15-7a / 15-7b | toutes |
| C-15-7-9 | #528 : identité de la société préservée, utilisateurs et clés rattachés | 15-7b2 |
| C-15-7-10 | #279 : vidage dérivé de `TABLES_TO_TRUNCATE` | 15-7b2 |
| C-15-7-11 | les trois gardes de la remise à zéro sous le verrou de l'effacement | 15-7b2 |
| C-15-7-12 | ordre des verrous Pattern 5, étape revérifiée sous verrou | 15-7a2 (et 15-7b1 pour `seed_demo`) ; **amendé** par C-15-7-22 pour `reset_demo` |
| C-15-7-13 | atomicité prouvée par un déclencheur SQL de test | 15-7a2, 15-7b1, 15-7b2 |
| C-15-7-14 | dernière transaction de `seed_demo`, boucle de retry conservée autour | 15-7b1 |
| C-15-7-15 | helpers société sur `companies::update_in_tx` ; helper d'étape dans `kesh-db` | 15-7a1 (fin du MIRROR), 15-7a2 |
| C-15-7-16 | `onboarding_state` remis à zéro en place | 15-7b2 |
| C-15-7-17 | taux TVA insérés ligne à ligne | 15-7a1 |
| C-15-7-18 | ordre des entrées : domaine puis étape | 15-7a2, 15-7b1, 15-7b2 |
| C-15-7-19 | découpage de la 15-7a en 15-7a1 (socle) / 15-7a2 (routes et audit) ; décomptes de modules déclarés | toutes |
| C-15-7-20 | emplacement des helpers : `lock_state_in_tx` dans `kesh-db` (15-7a1), `lock_state_at_step` et `record_step_completed_in_tx` à la 15-7a2 | 15-7a1, 15-7a2 |
| C-15-7-21 | `is_stub` : `clear_stub_in_tx` à SQL fixé, `company.updated` si version changée **ou** stub levé, relecture | 15-7a1, 15-7a2 |
| C-15-7-22 | ordre des verrous de la remise à zéro : exception au Pattern 5, rejeu `retry_with` sur interblocage | 15-7b2 |
| C-15-7-23 | #528 sur une base sans société : `insert_stub` et `attach_all_principals_in_tx` uniques — le second **remplacé** par `reattach_orphan_principals_in_tx` (C-15-7-45) | 15-7b2 |
| C-15-7-24 | partition des tables gardée sur le schéma, `RESET_*` dans `kesh_db::backup` ; **amendé** par C-15-7-28 (chaîne sans traverser une table préservée) | 15-7b2 |
| C-15-7-25 | contrôles FK rétablis avant le commit ; 15-7a2 et 15-7b dans la même release | 15-7a2, 15-7b1, 15-7b2 |
| C-15-7-26 | test 8 de la 15-7a1 sur le parcours de production ; enveloppes pool branche par branche — sa séquence `["fiscal_year.created"]` et son parcours sans `finalize` sont **corrigés** par C-15-7-30 | 15-7a1 (15-7a2 remplace la séquence) |
| C-15-7-27 | prédicat de rejeu de la remise à zéro gardé par le type `ResetAttemptError`, testé sur un vrai 1213 — type et prédicat **renommés** `SeedAttemptError` / `is_seed_retryable` et partagés avec la 15-7b1 par C-15-7-38 | 15-7b2 |
| C-15-7-28 | « rejoindre `companies` » sans traverser une table préservée ; valeurs du stub ; dérogation au découpage | 15-7b2 (la dérogation, réfutée à la P4, a conduit au découpage C-15-7-31) |
| C-15-7-29 | test 10 conservé avec mutation ; test à pool d'une connexion ; `bank_accounts` avant les réglages au Pattern 5 | 15-7a2 |
| C-15-7-30 | 15-7a1 : test 8 complété par `finalize`, séquence `["user.created", "fiscal_year.created"]`, SQL littéral du verrou (`LOCK_SQL`), six modules ; correction de fait à la 15-7a2 | 15-7a1, 15-7a2 |
| C-15-7-31 | découpage de la 15-7b en 15-7b1 (Volet A) / 15-7b2 (Volet B) ; même release pour 15-7a2, 15-7b1, 15-7b2 | 15-7b1, 15-7b2 |
| C-15-7-32 | remise à zéro : connexion fermée à la libération (`close_on_drop`), règle 3 de la partition, colonnes du stub gardées par `information_schema.COLUMNS` | 15-7b2 |
| C-15-7-33 | `pub const LOCK_SQL`, fichier `accounts_repository.rs` neuf, verrou d'état appelé en premier | 15-7a1 |
| C-15-7-34 | #542 fermée par la 15-7b2 : bootstrap gardé par `company_count == 0`, sociétés provisoires superflues supprimées par la remise à zéro — réparation des installations touchées **révisée** par C-15-7-46 (au démarrage, 15-7b3) | 15-7b2 |
| C-15-7-35 | `seed_demo` : `ui_mode` relu sous verrou, déclencheur sélectif, résidu et rejeu impossible assertés, boucle de retry non testée assumée | 15-7b1 |
| C-15-7-36 | `reset_cleared_tables()` en fonction, correspondance `SeedError` → `AppError`, test 7b des gardes sous verrou — attente du test 7b **amendée** par C-15-7-38 (`attendre_une_requete_en_cours`) | 15-7b2 |
| C-15-7-37 | `seed_demo` : dernière transaction rejouée sur 1213 (`SeedAttemptError`, `is_seed_retryable`), boucle `InactiveOrInvalidAccounts` inchangée, ordre de `finalize` ; manuel du § *Chemin A* (#544) | 15-7b1 |
| C-15-7-38 | #528 réparée au démarrage (`repair_orphan_principals`, `installation.principals_reattached`) ; un type d'essai et un prédicat pour les deux flux, `AlreadyFinalized` retirée ; test 7b sur `PROCESSLIST` | 15-7b2 (15-7b1 pour le type) |
| C-15-7-39 | clause de coupe de la 15-7b2 : sévérité maximale de la passe précédente ; non déclenchée à la P2 | 15-7b2 |
| C-15-7-40 | coupe de la 15-7b2 à la P3, selon la cause : prévention (15-7b2, `refs #528`) / réparation des installations atteintes (15-7b3, `closes #528`) | 15-7b2, 15-7b3 |
| C-15-7-41 | clés d'API orphelines **révoquées** (`revoke_in_tx`), non repointées, ids au journal ; libellés de `installation.principals_reattached` — **révisé** par C-15-7-45 (révoquées **puis** repointées, sur tous les chemins) et C-15-7-47 (action `installation.repaired`) | 15-7b3 |
| C-15-7-42 | LOW de la P3 de la 15-7b2 ; pas de rejeu de la réparation (démarrage : angle mort assumé ; restauration : route hors `retry_with`) ; acteur du démarrage | 15-7b2, 15-7b3 |
| C-15-7-43 | 15-7b1 : validation close (P3 ciblée Haiku à 0) | 15-7b1 |
| C-15-7-44 | 15-7b3 : fonction `kesh-db` partagée (`repair_orphan_principals_in_tx`, `RepairTrigger`), `attach_users_in_tx` extrait ; statut `backlog` jusqu'à validation ; même release visée — nom et extraction **révisés** par C-15-7-45 et C-15-7-47 | 15-7b3 |
| C-15-7-45 | une règle et une fonction pour les principaux orphelins (`reattach_orphan_principals_in_tx`, posée par la 15-7b2) : clés actives orphelines **révoquées puis** repointées sur tous les chemins ; **révise** C-15-7-41 ; signal D5 déclaré, pas de coupe | 15-7b2 (pose), 15-7b3 (appelle) |
| C-15-7-46 | #542 : la réparation des installations déjà touchées passe à la 15-7b3, au démarrage (la 15-7b2 garde la prévention, `refs #542`) ; **révise** C-15-7-34 | 15-7b2, 15-7b3 |
| C-15-7-47 | 15-7b3 : réparation au démarrage **non bloquante** ; `repair_installation_in_tx`, action `installation.repaired`, libellé neutre ; base sans utilisateur | 15-7b3 |
| C-15-7-48 | sortir de la démonstration exige `KESH_PRODUCTION_RESET` : les manuels le disent, l'impasse est écrite comme choix existant (#534) ; renvoi de la 15-7b1 corrigé sans rouvrir sa validation | 15-7b2, 15-7b1 |
| C-15-7-49 | 15-7b3 : détail des clés révoquées (pas de préfixe au schéma), lignes orphelines de #279 laissées (#546), sessions après un import ; LOW des P4 (15-7b2) et P1 (15-7b3) | 15-7b2, 15-7b3 |
| C-15-7-50 | 15-7b3, P2 : société provisoire supprimée seulement si **aucune ligne d'aucune table** ne la désigne (références lues dans `information_schema`), une suppression par `SAVEPOINT` ; note d'exploitation au § *Procédure de mise à jour* et au § *Dépannage* ; réparation avant la lecture des compteurs ; LOW | 15-7b3 |
| C-15-7-53 | 15-7b3, P3 : mutation « exclure une table des références » portée sur une table en CASCADE (`bank_profiles`) — garde et `SAVEPOINT` se recouvrent en RESTRICT, écrit ; service `kesh-api` au manuel (cinq occurrences fausses corrigées au passage) ; archive sans société sur installation onboardée : limite assumée ; LOW | 15-7b3 |

**Troisième découpage, à la passe P4 de la 15-7b** (R4-2, F-1) : la dérogation que la 15-7b avait
écrite à la P3 (C-15-7-28) reposait sur un constat faux — trois des quatre MEDIUM de la P3 étaient nés
des correctifs de la P2 —, et la P4 a trouvé un MEDIUM né de la remédiation P3. La clause de la fiche
s'appliquait « sans nouvel arbitrage » : coupe Volet A (15-7b1, la démonstration) / Volet B (15-7b2, la
remise à zéro), choix **C-15-7-31**.

## Change Log

- 2026-10-08 — Spécification initiale (`bmad-create-story`, en autonomie), commit `cabda169`. Statut
  `ready-for-dev`. Choix C-15-7-1 à C-15-7-7 consignés.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-7-validate-prompt-p1.md` ; deux
  lentilles Sonnet en contexte frais : R auditeur d'acceptation, F full-scope adversary ; rapports
  `target/gate-logs/15-7-p1-{R,F}.md`, non versionnés). Bruts, **recomptés depuis les rapports** :
  R **1 HIGH, 6 MEDIUM, 5 LOW** ; F **8 MEDIUM, 6 LOW** (son bilan déclare 7 MEDIUM : F-9, défaut
  préexistant, y est compté à part). Après fusion des doublons : **1 HIGH, 8 MEDIUM, 10 LOW
  distincts** :

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R1 = F-2 | HIGH | R, F | le test d'atomicité (« version désynchronisée avant l'appel ») ne peut pas échouer : l'état est relu dans l'appel | déclencheur SQL de test (C-15-7-13) : 15-7a test 9, 15-7b tests 2 et 8 |
  | R2 = F-4 | MEDIUM | R, F | compteurs de `demo_seeded` inobtenables par les enveloppes pool ; boucle de retry ; AC 9 contre AC 4 | 15-7b AC 1 (C-15-7-14) |
  | R3 = F-3 | MEDIUM | R, F | ordre des verrous inversé par la fusion ; étape lue hors verrou ; `existing == 0` hors transaction | 15-7a AC 8 (C-15-7-12) |
  | R4 = F-7 | MEDIUM | R, F | AC sans test (AC 10, refus `step >= 7` en profondeur, no-op hors `org-type`, préexistants de `finalize`, `mode`, `skip-bank`, branches bancaires, jeton sur quelle route) | 15-7a tests 2-12, 15-7b tests 3-11 |
  | R5 = F-6 | MEDIUM | R, F | AC 13 : grep trop étroit (`TODO(L65`, « ne génère PAS », « Pas d'audit log », Pattern 5) | 15-7a AC 12, 15-7b AC 10 |
  | R6 | MEDIUM | R | ordre des entrées non spécifié ; forme de `company.created` absente | 15-7a AC 1, AC 2 (C-15-7-18) |
  | R7 = F-5 | MEDIUM | R, F | décompte de la règle de découpage incohérent | **découpage (C-15-7-8)** |
  | F-1 | MEDIUM | F | une seule des trois gardes de la remise à zéro rejouée sous verrou | 15-7b AC 2 (C-15-7-11) |
  | F-9 | MEDIUM (préexistant) | F (R confirme) | `users.company_id` désigne une société effacée après une remise à zéro | issue **#528** ouverte par l'orchestrateur, **fermée par la 15-7b** (C-15-7-9) ; entraîne #279 (C-15-7-10) |
  | R8 | LOW | R | « onze routes atteignables par jeton » faux pour `reset` ; `finalize` extrait déjà `CurrentUser` | 15-7a inventaire § 1, 15-7b inventaire § 1 |
  | R9 | LOW | R | `rollback` internes à retirer des variantes `_in_tx` | 15-7a AC 8 |
  | R10 (+ F-10) | LOW | R, F | rollback explicite, `FOREIGN_KEY_CHECKS` non rétabli, hypothèse d'auto-incrément ; instantané vs `DELETE` | 15-7b AC 2, 5, 6 |
  | R11 | LOW | R | `UPDATE is_stub` hors transaction dans `seed-demo` | 15-7b AC 1 |
  | R12 | LOW | R | dénominateur du manuel (105 → 112) | 15-7a AC 13, 15-7b AC 11 |
  | F-8 | LOW | F | « 409 » au lieu de 400 | 15-7b AC 2 |
  | F-11 | LOW | F | #279 absente ; `tables_cleared` doit dire vrai | 15-7b AC 3, 5 (#279 fermée) |
  | F-12 | LOW | F | DRY : helper d'étape, `update_in_tx`, MIRROR | 15-7a AC 3, 5, 6 (C-15-7-15) |
  | F-13 | LOW | F | `serde_json` dans `kesh-seed`, clés i18n, notation, taux, société du bootstrap | 15-7a AC 1, 5, 11 ; 15-7b T2 |
  | F-14 | LOW | F | manuel : énumération `user-manual.tex:2013-2016` | 15-7a AC 13 |

  **Décisions de l'orchestrateur** (consignées C-15-7-8 à C-15-7-18) : découpage ; la 15-7b ferme
  #528 et #279 ; gardes de la remise à zéro sous le verrou de l'effacement ; Pattern 5 partout ;
  atomicité par déclencheur de test ; signature de `seed_demo` et boucle de retry tranchées.
  **Trouvés pendant la remédiation** : (a) préserver l'identité de la société sans vider toutes ses
  tables rattacherait les données de démonstration à la société remise à zéro, et `finalize`
  trouverait des réglages pointant des comptes effacés — d'où #279 fermée avec #528, et non à part ;
  (b) la réponse de `GET /api/v1/companies/current` ne porte pas `is_stub` (le test #528 lit la
  base) ; (c) `OrgType` s'écrit `Independant` en base.
  Recompte : 15-7a **14 AC, 8 tâches, 12 tests** ; 15-7b **12 AC, 9 tâches, 11 tests**.
- 2026-10-08 — **Passe de validation P2**, menée **sur les deux fiches filles** (prompts versionnés
  `15-7a-validate-prompt-p2.md`, `15-7b-validate-prompt-p2.md` ; lentilles R regression hunter et F
  full-scope adversary en contexte frais ; rapports `target/gate-logs/15-7{a,b}-p2-{R,F}.md`, non
  versionnés). Bruts, recomptés depuis les rapports : 15-7a — R 3 MEDIUM, 8 LOW ; F 5 MEDIUM, 7 LOW ;
  15-7b — R 3 MEDIUM, 10 LOW ; F 5 MEDIUM, 8 LOW. Après fusion des doublons : **15-7a 5 MEDIUM, 15 LOW
  distincts ; 15-7b 6 MEDIUM, 16 LOW distincts** ; aucun CRITICAL ni HIGH (P1 : 1 HIGH) — la sévérité
  décroît. Tableaux par finding aux Change Logs de la 15-7a2 et de la 15-7b.
  **Décisions de l'orchestrateur** (consignées C-15-7-19 à C-15-7-25) : découpage de la 15-7a en
  15-7a1 / 15-7a2 ; règle composée pour `is_stub` ; tests rendus exécutables et mordants (montage par
  route, pool à une connexion, plan préchargé, `raw_sql`) ; `reset_demo` inscrite à la liste
  d'exceptions du Pattern 5 et rejouée sur interblocage ; helper unique de rattachement des
  utilisateurs et clés, stub inséré sur une base sans société ; partition gardée sur le schéma ;
  attribution de chaque doc-comment à une fiche (tableau ci-dessus).
  **Signaux D5 déclarés** : 15-7a1 (sept modules, rollout mécanique) et 15-7b (huit modules, non
  découpée). **À signaler hors fiches** : la cause probable de l'échec attendu `onboarding.spec.ts:57`
  (#97 : `KESH_PRODUCTION_RESET` absent du montage E2E) ; le libellé du dialogue de confirmation de la
  remise à zéro (CR) ; la fenêtre `seed-demo` / `start-production` (#43) ; la divergence de no-op de
  `installation.ui_mode_changed` entre l'onboarding et `/profile/mode`.
  Recompte : 15-7a1 **8 AC, 5 tâches, 8 tests** ; 15-7a2 **14 AC, 8 tâches, 12 tests** ; 15-7b
  **12 AC, 9 tâches, 16 tests**.
- 2026-10-08 — **Passes de validation : 15-7a1 P1, 15-7a2 P3 (close), 15-7b P3** (prompts versionnés
  `15-7a1-validate-prompt-p1.md`, `15-7a2-validate-prompt-p3.md`, `15-7b-validate-prompt-p3.md` ;
  lentilles R et F en contexte frais, Sonnet ; rapports `target/gate-logs/15-7a1-p1-{R,F}.md`,
  `15-7a2-p3-{R,F}.md`, `15-7b-p3-{R,F}.md`, non versionnés). Bruts recomptés depuis les rapports :
  15-7a1 — R 2 MEDIUM, 5 LOW ; F 1 MEDIUM, 4 LOW ⇒ **2 MEDIUM, 6 LOW distincts** ; 15-7a2 — R 9 LOW ;
  F 10 LOW ⇒ **0 au-dessus de LOW, 17 LOW distincts : validation convergée** ; 15-7b — R 3 MEDIUM,
  6 LOW ; F 4 MEDIUM, 4 LOW ⇒ **4 MEDIUM, 9 LOW distincts**. Détail aux Change Logs des trois fiches.
  Choix C-15-7-26 à C-15-7-29. Propagation : grep des symptômes (`trois copies`, `address_*`,
  `is_deadlock_error`, `atteint companies`, `deux familles`, `Huit enchaînent`, `295-302`, `90-91`,
  `999-1000`, `594-598`, `sous le seuil`) sur les trois fiches, l'index, le registre et le
  sprint-status. **À signaler hors fiches** (en plus de la P2) : `DemoBanner` rend tout 403 par
  « Seul un administrateur… » (même CR que le dialogue de confirmation) ; la fenêtre #43 élargie par
  la préservation de la société (commentaire sur #43). Recompte : 15-7a1 **8 AC, 5 tâches, 8 tests** ;
  15-7a2 **14 AC, 8 tâches, 13 tests** ; 15-7b **12 AC, 9 tâches, 17 tests**.
- 2026-10-08 — **Passes de validation : 15-7a1 P2, 15-7b P4** (prompts versionnés
  `15-7a1-validate-prompt-p2.md`, `15-7b-validate-prompt-p4.md` ; lentilles R et F en contexte frais,
  Opus ; rapports `target/gate-logs/15-7a1-p2-{R,F}.md`, `15-7b-p4-{R,F}.md`, non versionnés). Bruts
  recomptés depuis les rapports : 15-7a1 — R 1 HIGH, 2 MEDIUM, 5 LOW ; F 3 MEDIUM, 8 LOW ⇒ **1 HIGH,
  2 MEDIUM, 10 LOW distincts** ; 15-7b — R 3 MEDIUM, 6 LOW ; F 4 MEDIUM, 7 LOW ⇒ **6 MEDIUM, 11 LOW
  distincts**. Détail aux Change Logs de la 15-7a1 et de la 15-7b2.
  **Signaux D5 constatés, et suivis** : 15-7a1 — R-1 (HIGH) recycle le défaut du test 8 de la P1 ;
  corrigé sans découpage, la fiche étant déjà la story-zéro d'un découpage (C-15-7-30). 15-7b — R4-2 et
  F-1 : **découpée** en 15-7b1 + 15-7b2 (C-15-7-31) ; remédiations dans les fiches filles (C-15-7-32).
  Correction de fait à la 15-7a2 (validation non rouverte) : `user.created` en tête des séquences
  montées par le bootstrap. Propagation : grep des symptômes (`fiscal_year.created"]`, `SELECT_SQL`,
  `adaptation mécanique`, `sept modules`, `#43`, `detach`, `au montage`, `35-36`, `deux règles`) sur
  les fiches, l'index, le registre et le sprint-status. **À signaler hors fiches** : `user-manual.tex:780`
  et `:1772` annoncent 2.5 % et 3.7 % (le seed pose 2.60 et 3.80) ; l'atomicité des quatre premières
  validations de `seed_demo` n'est portée par aucune issue (#538 à compléter ?) ; le bootstrap sans
  variable d'admin insère un stub à chaque démarrage. Recompte : 15-7a1 **8 AC, 5 tâches, 8 tests** ;
  15-7a2 **14 AC, 8 tâches, 13 tests** ; 15-7b1 **7 AC, 7 tâches, 4 tests** ; 15-7b2 **11 AC,
  8 tâches, 13 tests**.
- 2026-10-08 — **Passes de validation : 15-7a1 P3 (close), 15-7b1 P1, 15-7b2 P1** (prompts versionnés
  `15-7a1-validate-prompt-p3.md`, `15-7b1-validate-prompt-p1.md`, `15-7b2-validate-prompt-p1.md` ;
  lentilles R et F en contexte frais ; rapports `target/gate-logs/15-7a1-p3-{R,F}.md`,
  `15-7b1-p1-{R,F}.md`, `15-7b2-p1-{R,F}.md`, non versionnés). Bruts recomptés depuis les rapports :
  15-7a1 — R 1 MEDIUM, 6 LOW ; F 1 MEDIUM, 4 LOW ⇒ **1 MEDIUM, 8 LOW distincts** ; 15-7b1 — R 2 MEDIUM,
  6 LOW ; F 3 MEDIUM, 3 LOW ⇒ **4 MEDIUM, 6 LOW distincts** ; 15-7b2 — R 1 HIGH, 2 MEDIUM, 6 LOW ;
  F 3 MEDIUM, 5 LOW ⇒ **1 HIGH, 4 MEDIUM, 11 LOW distincts**. Détail aux Change Logs des trois fiches.
  **15-7a1 : validation close** (remédiation de fiche seule, aucun changement de conception).
  **#542 fermée par la 15-7b2** (décision de l'orchestrateur) : garde du bootstrap et réparation des
  installations touchées par la remise à zéro ; `closes #542` ajouté aux deux fiches index. **Signal D5
  déclaré** pour la 15-7b2 (HIGH après des MEDIUM), sans recyclage : coupe prédéclarée non appliquée.
  Choix C-15-7-33 à C-15-7-36. Recompte : 15-7a1 **8 AC, 5 tâches, 8 tests** ; 15-7a2 **14 AC,
  8 tâches, 13 tests** ; 15-7b1 **7 AC, 7 tâches, 4 tests** ; 15-7b2 **12 AC, 8 tâches, 17 tests**.
- 2026-10-08 — **Passes de validation : 15-7b1 P2, 15-7b2 P2** (commit `ffcdcf8d` ; détail aux Change
  Logs des deux fiches ; entrée d'index omise à ce commit, rattrapée ici). 15-7b1 — **2 MEDIUM, 9 LOW
  distincts** (C-15-7-37, ferme #544) ; 15-7b2 — **4 MEDIUM, 6 LOW distincts** (réparation de #528 au
  démarrage, C-15-7-38 ; clause de coupe précisée, C-15-7-39).
- 2026-10-08 — **Passes de validation : 15-7b1 P3 ciblée (close), 15-7b2 P3** (prompts versionnés
  `15-7b1-validate-prompt-p3-ciblee.md`, `15-7b2-validate-prompt-p3.md` ; rapports
  `target/gate-logs/15-7b1-p3-ciblee.md`, `15-7b2-p3-{R,F}.md`, non versionnés). 15-7b1 — une lentille
  Haiku ciblée sur `ffcdcf8d` : **0** ; axe non exercé : le PDF, sans objet (`ffcdcf8d` ne touche pas
  `docs/manual/`) — **validation close** (C-15-7-43). 15-7b2 — deux lentilles Sonnet : R **8 LOW**, F
  **2 MEDIUM, 5 LOW** ⇒ **2 MEDIUM, 13 LOW distincts**. **Signal D5 constaté et suivi** : les deux MEDIUM
  (F3-1 clés réactivées, F3-2 restauration) sont nés de la remédiation P2 ⇒ **coupe selon la cause**
  (C-15-7-40) : la réparation des installations déjà atteintes sort en **15-7b3** (`closes #528`), avec
  F3-1 tranché (clés révoquées, C-15-7-41) et F3-2 couvert (réparation à la restauration) ; la 15-7b2
  garde la prévention (`refs #528`). LOW appliqués aux deux fiches (C-15-7-42) ; R3-4 et R3-5 au tableau
  d'attribution ci-dessus. Recompte : 15-7b1 **7 AC, 7 tâches, 4 tests** ; 15-7b2 **12 AC, 8 tâches,
  17 tests** ; 15-7b3 **6 AC, 7 tâches, 3 tests**.
- 2026-10-08 — **Passes de validation : 15-7b2 P4, 15-7b3 P1** (prompts versionnés
  `15-7b2-validate-prompt-p4.md`, `15-7b3-validate-prompt-p1.md` ; quatre lentilles **Opus** ; rapports
  `target/gate-logs/15-7b2-p4-{R,F}.md`, `15-7b3-p1-{R,F}.md`, non versionnés). 15-7b2 — **3 MEDIUM,
  7 LOW distincts** ; 15-7b3 — **1 HIGH, 6 MEDIUM, 12 LOW distincts**. Le HIGH (F1) et le MEDIUM
  R4-1/F4-1 sont **le même défaut** vu des deux fiches : la règle C-15-7-41 appliquée au seul démarrage,
  la remise à zéro et `language` réveillant les clés orphelines ⇒ **une règle, une fonction**
  (`reattach_orphan_principals_in_tx`, posée par la 15-7b2, appelée par la 15-7b3 ; clés révoquées puis
  repointées — C-15-7-45). La réparation de #542 passe de la remise à zéro au démarrage (15-7b3,
  `closes #542` ; 15-7b2 `refs #542` — C-15-7-46). Sortie de la démonstration dite aux manuels
  (C-15-7-48 ; report dans la 15-7b1, sans rouvrir sa validation). **Signal D5 déclaré au Project
  Lead**, pas de coupe (résidu de propagation et ligne de manuel, hors de l'axe prédéclaré). Règle de
  découpage : 15-7b2 toujours huit modules (dérogation écrite) ; 15-7b3 recomptée à **cinq** (non
  franchie). Recompte : 15-7b2 **12 AC, 8 tâches, 17 tests** ; 15-7b3 **6 AC, 7 tâches, 5 tests**.
- 2026-10-08 — **Passe de validation P2 de la 15-7b3** (prompt `15-7b3-validate-prompt-p2.md` ; deux
  lentilles **Sonnet**) : **4 MEDIUM, 11 LOW distincts**, remédiés (C-15-7-50) — sociétés provisoires
  supprimées seulement sans aucune référence (`information_schema`, `SAVEPOINT`), note d'exploitation
  déplacée vers la procédure de mise à jour et le dépannage du manuel, manuel utilisateur `:297`. 15-7b2
  non modifiée. 15-7b3 toujours **cinq** modules. Recompte : 15-7b3 **6 AC, 7 tâches, 6 tests,
  13 mutations**. P3 complète à lancer.
- 2026-10-08 — **Passe de validation P3 de la 15-7b3** (prompt `15-7b3-validate-prompt-p3.md` ; deux
  lentilles **Opus**) : **2 MEDIUM, 13 LOW distincts**, remédiés (C-15-7-53) — la mutation qui exclut une
  table des références porte sur `bank_profiles` (CASCADE), la variante `api_keys` (RESTRICT) étant
  muette sous le `SAVEPOINT` ; service `kesh-api` dans le dépannage, et lignes fausses préexistantes du
  manuel corrigées au passage (`:1905`, `:1906`, `:2058`, `:2063`) ; limite « archive sans société »
  écrite ; § *Rollback* du manuel. **Signal D5 déclaré** (recyclage d'une remédiation, traité localement,
  pas de coupe). 15-7b2 non modifiée. 15-7b3 toujours **cinq** modules. Recompte : 15-7b3 **6 AC,
  7 tâches, 6 tests, 14 mutations**. P4 ciblée à lancer.
