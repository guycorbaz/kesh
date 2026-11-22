# Story 38-12b : bouton « Télécharger depuis PayPal », manuels, E2E

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`. Epic 38, **phase 2**. **Dépend de** : 38-12a (routes), 38-11d
(la page PayPal `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte`, son import CSV et son compte rendu,
38-11d AC-3). Scindée de 38-12a (voir sa fiche).*

## Amendement du 22.11 (développement, prémisses relues au code de la 38-12a)

*Relu au code de `epic/38-paypal` (`8b8cd338`, qui porte la 38-12a et la 38-11d) avant la première ligne. Ce qui
suit remplace, là où il les contredit, les AC et la stratégie de tests ci-dessous.*

1. **Page commune des prestataires** : `ProviderReviewPage` (40-7d) n'est pas sur cette branche ; la page PayPal de la
   38-11d est l'écran. Le **dialogue générique** va à `frontend/src/lib/features/provider-download/` (route
   `<prestataire>-downloads`, nom du prestataire et nom de l'étape en paramètres, compte rendu fourni par l'appelant
   en *snippet*) ; le **compte rendu de l'import CSV** est **extrait** de la page en un composant
   `frontend/src/lib/features/paypal/PaypalImportReport.svelte`, que la page (import CSV) et le dialogue (téléchargement)
   affichent tous deux (AC-3 « même composant »). Rien n'est recopié.
2. **AC-1, `providerApi`** : la réponse de la **liste** `GET /api/v1/bank-accounts` (celle que lit la page PayPal) porte
   le champ ; les réponses d'un seul compte (`POST`, `PUT`, `PATCH`, clôture, réouverture, archivage) ne changent pas
   (leur forme est le `BankAccount` de `kesh-db`, partagé). `configured` suit `ProviderApiConfig::series_for` (même
   règle que la route : sans casse, comparaison `to_lowercase`) ; `mode` est **générique** : `live` quand l'hôte est
   celui par défaut du prestataire, `sandbox` quand un segment de son nom d'hôte est `sandbox`
   (`api-m.sandbox.paypal.com`), **absent** sinon (serveur simulé du bouclage).
3. **AC-3, messages d'échec** : le `message` d'une tâche `failed` est rendu par le serveur ; le dialogue traduit lui-même
   les codes `PROVIDER_*` qu'il connaît (clés `provider-download-error-*`, avec fenêtre et page, devises, lignes), et
   affiche le `message` du serveur pour tout autre code (`INTERNAL_ERROR`, `BANK_ACCOUNT_CLOSED`,
   `RECONCILIATION_ACCOUNT_LOCKED`…), puis « rien n'a été enregistré, relancez ».
4. **AC-6, dates** : la 38-12a a constaté aux appels réels que l'API rend ses dates en **UTC** (et non dans le fuseau
   du compte) ; elle les **convertit à l'heure de Zurich**. Le manuel dit donc : dates téléchargées à l'heure suisse,
   dates du CSV telles que l'export les écrit (fuseau du compte) ; une transaction proche de minuit peut tomber un autre
   jour selon la source, jamais en double (identifiant). S'ajoute la limite G4 de la 38-12a : la **première heure** du
   jour le plus ancien admis (le plancher : il y a trois ans, plus une heure) n'est pas lue.
5. **Stratégie de tests** (règle des gates de la nuit du 22.11) : fmt, clippy, nextest complet, frontend complet ; E2E
   **seulement** de la spec ajoutée (`paypal-download.spec.ts`) ; ni E2E complet ni `gate-ci.sh` (à la fin de l'epic).
6. **E2E** : `with-company` a son exercice à l'année du jour ? À défaut la spec crée celui de l'année courante ; la
   période de deux mois est prise **dans l'année courante** (fin = aujourd'hui), le serveur simulé ne rend que les
   transactions de la fenêtre demandée, ralentit ses pages et répond une fois `429` avec `Retry-After` pour que la
   progression et la pause se voient.

## À reprendre (revue P3 de la 38-11d)

- **Manuel et `docs/api-external.md`** : le manuel utilisateur (`user-manual.tex`, chapitre PayPal) écrit « Pas de téléchargement automatique depuis PayPal : le fichier s'exporte et s'importe à la main », juste à l'écran avant cette story, faux pour un script depuis la 38-12a (`POST /api/v1/paypal-downloads`, `GET /api/v1/paypal-downloads/{job_id}`, identifiants `KESH_PAYPAL_<n>_*`). La 38-12b réécrit cette phrase (et le PDF) et documente ces routes à `docs/api-external.md`, qui n'en dit rien aujourd'hui.

## Story

En tant que **Comptable**, je veux, depuis la page PayPal où j'importe déjà le CSV, choisir une période et lancer le
téléchargement, voir où il en est, lire son compte rendu et retrouver les lignes dans la revue ; en tant
qu'**Administrateur**, je veux savoir au manuel comment créer l'application PayPal en lecture seule et poser ses
identifiants.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-12b-1` | Un bouton « Télécharger depuis PayPal » n'apparaît que si le compte PayPal choisi a ses identifiants dans l'environnement. | AC-1 |
| `FR-38-12b-2` | La période se choisit ; la progression (« fenêtre 5/13 ») et les pauses se voient ; le délai de trois heures se dit. | AC-2, AC-3 |
| `FR-38-12b-3` | Le manuel d'administration dit les variables et la création de l'application ; le manuel utilisateur le téléchargement. | AC-5, AC-6 |
| `NFR-38-12b-1` | L'E2E exerce le parcours contre un serveur PayPal simulé ; aucune donnée réelle. | AC-7 |

## Critères d'acceptation

*Socle commun S7 et S8* (`epic-39-wise.md`, `epic-40-stripe.md`) : cette fiche écrit le **dialogue générique**,
paramétré par le prestataire (route `<prestataire>-downloads`, libellés), et le **serveur simulé commun** de l'E2E ;
Wise (39-12b) et Stripe (40-8b) les réutilisent.

1. **AC-1 — Indicateur et bouton** (S7). La réponse des comptes bancaires (`crates/kesh-api/src/routes/bank_accounts.rs`)
   gagne, pour un compte de prestataire, le champ **générique** `providerApi: { configured: boolean, mode?: "live" |
   "sandbox" }` (`configured` vrai si une série existe au libellé du compte, selon le prestataire du
   compte, `provider_env` de 38-12a ; `mode` pour PayPal selon `KESH_PAYPAL_BASE_URL` ; un champ propre à un autre
   prestataire s'ajoute par sa fiche, le socle n'en porte aucun ; jamais un secret ; `null` pour un compte sans prestataire). **Page PayPal de 38-11d**
   (`reconciliation/paypal/+page.svelte`, feature `frontend/src/lib/features/paypal/`) : à côté de « Importer le CSV
   PayPal », bouton **« Télécharger depuis PayPal »** pour le compte PayPal choisi au sélecteur, visible de Comptable et
   plus quand `providerApi.configured` ; en mode `sandbox`, la mention « bac à sable » ; sinon une ligne d'aide « téléchargement non configuré : voir le manuel
   d'administration » (Administrateur seulement), et l'import CSV reste seul.
2. **AC-2 — Fenêtre de dialogue** (composant générique, `frontend/src/lib/features/provider-download/`). Période (date de début, date de fin ; défaut : l'exercice choisi au sélecteur de la
   page, fin ramenée à aujourd'hui), refus du serveur affichés par code (`PROVIDER_PERIOD_INVALID`
   selon `details.reason`, `PROVIDER_API_NOT_CONFIGURED` ; `API_KEY_OPERATION_FORBIDDEN` n'est pas atteignable depuis
   l'écran) ; sur 409 `PROVIDER_DOWNLOAD_RUNNING`, le dialogue **reprend le suivi** de la tâche en cours par le `jobId`
   de `details` (38-12a AC-8) au lieu d'un message d'erreur. Texte permanent : « PayPal peut mettre jusqu'à trois heures à
   publier une transaction ; l'historique remonte à trois ans ; le téléchargement se fait par tranches de 30 jours ».
3. **AC-3 — Progression et résultat.** Après le 202, le dialogue interroge `GET /api/v1/paypal-downloads/{jobId}` toutes
   les secondes : barre et texte « fenêtre `steps.done`/`steps.total` » (le nom de l'étape
   est un paramètre du prestataire ; `total` nul → compteur sans barre), message « PayPal demande de patienter, reprise dans `n` s »
   quand `pausedUntilSeconds` est non nul ; à `done`, le **compte rendu de l'import CSV de 38-11d** (même composant :
   insérées, doublons, écartées par motif, par devise), « données de PayPal à jour jusqu'à … » (`result.lastRefreshed`), et la
   revue de la page se recharge ; à `failed`, le message traduit du code `PROVIDER_*` (ceux de 38-12a AC-6 et AC-8,
   dont `PROVIDER_CURRENCY_ACCOUNT_MISSING` qui nomme les devises comme à l'import CSV, et `PROVIDER_RESPONSE_INVALID`
   qui liste les lignes de `error.details.lines`) avec la fenêtre et la page de `error.details`, et la mention « rien n'a été enregistré, relancez » ; fermer le dialogue
   n'arrête pas la tâche, et le rouvrir la retrouve (409, AC-2). Le suivi **s'arrête** à la fermeture du dialogue et au démontage
   de la page (aucune interrogation orpheline) ; un `GET` en 404 (tâche purgée ou serveur redémarré) ou une erreur réseau
   répétée affiche « suivi perdu : relancez la même période, rien n'est doublé » (38-12a AC-7). Aucun secret n'est affiché.
4. **AC-4 — i18n.** Libellés, messages d'erreur et textes des AC-1 à AC-3 aux quatre locales
   (`crates/kesh-i18n/locales/*/messages.ftl`, clés au préfixe `paypal-` de la page et `provider-download-` du dialogue générique, `docs/i18n-key-ownership-pattern.md`) ;
   `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté depuis la source ; garde d'appartenance des clés
   (`npm run lint-i18n-ownership`) verte.
5. **AC-5 — Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, `.tex` seul, aucun PDF commité). Section
   « Télécharger les transactions PayPal par l'API » : (a) les variables `KESH_PAYPAL_<n>_ACCOUNT`, `_CLIENT_ID`,
   `_SECRET` (`n` de 1 à 9 ; le libellé **identique** à celui du compte PayPal de Kesh), `KESH_PAYPAL_BASE_URL` (réel par
   défaut, bac à sable pour un essai), `KESH_PAYPAL_PARALLEL` (1 par défaut, 6 au plus ; ce que le parallélisme risque : une
   limitation de débit plus vite atteinte) ; (b) **créer l'application** dans l'espace développeurs de PayPal,
   application « Live », **en ne cochant que la fonction « Transaction Search »** (aucune fonction de paiement :
   l'application ne peut que lire), recopier l'identifiant client et le secret dans l'environnement du serveur ; (c) ce
   que l'application ne fait jamais : écrire chez PayPal ; (d) les secrets ne sont **ni en base ni dans la
   sauvegarde** : après une restauration sur un autre serveur, on les repose ; ils appartiennent à l'installation ; (e)
   la rotation (changer le secret chez PayPal, puis dans l'environnement, redémarrer) ; (f) les refus de démarrage et
   leurs messages ; (f-bis) **où les poser** : au `.env` d'une installation par `docker-compose` (les fichiers `docker-compose.yml` et
   `docker-compose.prod.yml` transmettent les séries 1 à 3 ; au-delà, ajouter les lignes `KESH_PAYPAL_<n>_…` au fichier
   de composition), et les deux **tableaux de variables** du manuel : un paragraphe-tableau neuf au § des variables
   d'environnement (`sec:env-vars`, à la suite de celui des e-mails sortants, patron `:840-862`) et le tableau de la
   section PayPal écrite ici (patron de `sec:recovery-smtp`, `:1155`) — **jamais** le tableau SMTP lui-même ; la forme
   `KESH\_PAYPAL\_<n>\_…` y est celle qu'admet la garde `configuration_transmise.rs` en (F) (famille indexée de 38-12a
   AC-1) ; une variable laissée vide vaut absente ; (g) au paragraphe des **accès sortants** (`sec:acces-sortant-cours`), le nouvel accès HTTPS (port
   443) à l'hôte de PayPal, sans redirection, seulement quand un téléchargement est demandé. Les points **non
   confirmés** de la fiche 38-12a (nom exact de la fonction dans l'interface de PayPal, conditions d'accès du compte,
   hôte réel tant qu'il n'est pas relevé) sont écrits **comme à vérifier**, jamais comme acquis. Le PDF se contrôle à
   la régénération de la release (`pdftotext … | tr '\n' ' '`). *(Les deux phrases du registre des routes et les assertions de `audit_route_registry` sont mises à jour par
   38-12a AC-8.)*
6. **AC-6 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section PayPal de 38-11d) : le téléchargement, la
   période, le délai de trois heures et la date « à jour jusqu'à », la progression et la pause, ce qu'un échec veut dire
   (rien d'enregistré), le mélange avec le CSV (aucun doublon), la limite des dates (fuseau du compte PayPal, sans
   conversion, comme pour le CSV). `docs/api-external.md` : les deux routes, **réservées aux sessions** (clés d'API
   refusées en `403 API_KEY_OPERATION_FORBIDDEN`), avec leurs codes ; les **quatre sites** qui comptent les opérations fermées aux clés suivent : les encarts de fermeture du § 4, la liste des
   ressources (« à quatre exceptions près, toutes quatre fermées aux clés » : le nombre se **recompte** à la reprise de `main` ; les
   deux routes y forment **une** exception, comme `/api/v1/audit-log*`, soit « cinq » si rien d'autre n'a changé), le tableau
   des fermetures et la ligne `API_KEY_OPERATION_FORBIDDEN` du tableau des codes d'erreur ; les codes `PROVIDER_*` des deux
   routes entrent à ce dernier tableau.
7. **AC-7 — E2E.** `frontend/tests/e2e/paypal-download.spec.ts`. La configuration se lit au démarrage du backend : les
   deux cas se jouent donc dans **le même** démarrage, par deux comptes PayPal créés par la spec — (a) un compte dont le
   libellé n'a **pas** de paire : le bouton est absent ; (b) un compte au libellé de `KESH_PAYPAL_1_ACCOUNT`, le backend
   d'E2E démarré avec `KESH_PAYPAL_1_*` et `KESH_PAYPAL_BASE_URL=http://127.0.0.1:<port>` (`http` admis
   vers le seul bouclage, 38-12a AC-1) vers le **serveur de prestataire simulé commun** (S8 ;
   `frontend/tests/e2e/support/provider-mock.ts`, qui joue PayPal ici, Wise et Stripe ensuite) que la spec lance sur le
   port lu de `KESH_E2E_PROVIDER_MOCK_PORT` (ajout au montage de `docs/testing.md` ; le bloc de montage de `CLAUDE.md` n'est **pas**
   touché) : période de deux mois, progression visible, compte rendu, puis la revue de la page affiche les lignes ;
   relancer la même période → « 0 insérée ». Le serveur simulé ne sert que des transactions **en CHF** et le compte PayPal créé par la spec est en CHF avec son compte lié (sinon `PROVIDER_CURRENCY_ACCOUNT_MISSING`). Sans la variable du port, (b) est **ignoré avec son motif écrit** (la ligne
   s'ajoute aux ignorés connus de `docs/testing.md`, jamais comptée verte). Vitest : le dialogue (états `running`,
   pause, `done`, `failed`, refus par code, reprise sur 409) et l'indicateur `providerApi`.

## Tâches

- [x] T1 — `providerApi` à la réponse des comptes (AC-1).
- [x] T2 — Bouton, dialogue, interrogation, i18n (AC-1 à AC-4).
- [x] T3 — Manuels et `docs/api-external.md` (AC-5, AC-6).
- [x] T4 — E2E, serveur simulé, `docs/testing.md` (AC-7).

## Stratégie de tests

Vitest du dialogue ; `npm run check`, `lint-i18n-ownership`, `build` ; E2E **complet au dernier commit de code** ;
gate backend complet (la réponse des comptes bancaires change). Captures d'écran de chaque état du dialogue (thème
clair), jointes au prompt de la revue.

## Modules touchés (3)

`frontend` (page PayPal, E2E), `kesh-i18n` (+ `kesh-api` : le seul champ `providerApi`), manuels et `docs/`.

## Fichiers communs

`frontend/src/routes/(app)/reconciliation/paypal/` et `frontend/src/lib/features/paypal/` (38-11d),
`frontend/src/lib/features/provider-download/` et le serveur simulé commun (réutilisés par 39-12b et 40-8b), catalogues i18n,
`i18n-keys.test.ts`, manuels `.tex` (jamais le PDF dans la branche), `docs/api-external.md`, `docs/testing.md`.

## Note de version

*Ajouté* — Bouton « Télécharger depuis PayPal » sur la page PayPal, pour un compte configuré, avec la période, la
progression et le compte rendu ; le manuel d'administration décrit la création de l'application PayPal en lecture seule.
*Changé (API, additif)* — chaque compte de `GET /api/v1/bank-accounts` porte `providerApi` (`{ configured, mode? }`
pour un compte de prestataire, `null` sinon) ; le refus `409 PROVIDER_DOWNLOAD_RUNNING` porte aussi la période de la
tâche en cours (`details.from`, `details.to`), que le dialogue annonce quand elle diffère de la période demandée.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
- 2026-11-20 — validation P1 de la phase 2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 0 LOW
  pour cette fiche (2 H, 10 M, 8 L avec 38-12a), remédiés (rapport `kesh-gate-logs/epic-38-kiss/validation/38-12-P1.md`).
  Principaux : le bouton va sur la page PayPal de 38-11d, où vivent l'import CSV et son compte rendu, non sur l'écran
  des comptes bancaires ; E2E (a) et (b) dans un même démarrage par deux libellés ; chemin i18n réel
  (`crates/kesh-i18n/locales`, `frontend/src/lib/shared/i18n-keys.test.ts`) ; reprise du suivi sur 409 ; accès sortant
  et liste des exceptions de l'API externe aux manuels ; tranches de 30 jours.
- 2026-11-20 — validation P1 de la phase 2, complément (Claude Opus 5.5, même passe, sur message du coordinateur) :
  alignement sur le socle commun S7 et S8 (`epic-39-wise.md`, `epic-40-stripe.md`) — `providerApi { configured, mode?,
  scaKeyConfigured? }` au lieu de `paypal.apiConfigured`, dialogue générique `provider-download`, état `steps`, codes
  `PROVIDER_*`, serveur simulé commun sur `KESH_E2E_PROVIDER_MOCK_PORT`.
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 2 LOW pour cette fiche
  (voir 38-12a et `kesh-gate-logs/epic-38-kiss/validation/38-12-P2.md`), remédiés : variables à poser au `.env` par les
  fichiers `docker-compose` et aux deux tableaux de variables du manuel ; `error.details` (décision 3) ; quatre sites de
  `docs/api-external.md` pour les fermetures aux clés (« six » exceptions) ; arrêt du suivi et suivi perdu ; mock en CHF.
- 2026-11-20 — validation P3 ciblée sur le commit `dc83cdf6` (Claude Opus 5.5, contexte frais ; voir 38-12a et
  `kesh-gate-logs/epic-38-kiss/validation/38-12-P3-ciblee.md`) : MEDIUM — les « deux tableaux » désignaient le tableau de
  la récupération par e-mail : tableau neuf au § `sec:env-vars` et tableau de la section PayPal ; LOW — « six » exceptions
  → recompte, les deux routes en forment une ; `providerApi.scaKeyConfigured` retiré (socle amendé à la P2 Wise).
- 2026-11-20 — validation P4 ciblée sur le commit `e6da3774` (Claude Sonnet 5.5, contexte frais ; voir 38-12a et
  `kesh-gate-logs/epic-38-kiss/validation/38-12-P4-ciblee.md`) : rien à remédier ici (tableaux § `sec:env-vars` et section
  PayPal, « cinq » exceptions, `scaKeyConfigured` absent : vérifiés au code de `origin/main`) ; la panique d'une tâche
  rend `INTERNAL_ERROR` (38-12a AC-8), ce que l'écran affiche comme tout code non listé.
- 2026-11-20 — validation P5 ciblée sur les commits `977bf9ec` et `452606a3` (modèle : claude-haiku-5-5, contexte frais ; voir 38-12a et `kesh-gate-logs/epic-38-kiss/validation/38-12-P5-ciblee.md`) : rien à remédier ici ; l'AC-5 et le tableau PayPal sont cohérents avec l'AC-8 de 38-12a.
- 2026-11-22 — développement : amendement du 22.11 (page commune absente de la branche, `providerApi` sur la liste, dates en heure de Zurich, gates de la nuit) ; statut `in-progress`.
- 2026-11-22 — développement terminé (T1 à T4), story en `review` ; détail au Dev Agent Record.
- 2026-11-22 — revue de code P1 (passe complète, une lentille ; rapport `kesh-gate-logs/s38-12b/revue-p1-rapport.md`) :
  0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW. Remédiés : **M1** (une autre période relancée pendant un téléchargement suivait
  l'ancien sans le dire) — `Downloads::start` garde la période de la tâche et la rend au refus, le `409` la porte dans
  `details.from`/`to` (code inchangé), le dialogue affiche `provider-download-resumed` quand elle diffère de la
  période demandée ; `provider-download-close-hint` (quatre locales), manuel utilisateur et `docs/api-external.md`
  (§ PayPal et § 10) disent « pour ce compte, quelle que soit la période » ; **L1** (fermeture pendant le `POST` :
  garde de génération prise au lancement) ; **L2** (`done` sans `result` → « suivi perdu » ; `onDone` hors du `try`
  du suivi, appelé une fois même s'il lève) ; **L3** (E2E : période des 60 jours qui précèdent le jour, sans borne au
  1er janvier, exercice retenu couvrant toute la période) ; **L4** (§ 10 : « il y a trois ans, plus une heure »,
  phrase de la fiche alignée). **L5** et **L6** : notés au Dev Agent Record (L5 aussi aux consignes des correctifs,
  pour la reprise du serveur simulé par Wise). Le serveur étant touché, la P2 est **complète**.
- 2026-11-22 — revue de code P2 (passe complète, une lentille ; rapport `kesh-gate-logs/s38-12b/revue-p2-rapport.md`) :
  0 CRITICAL, 0 HIGH, 0 MEDIUM, 7 LOW. Corrigés dans la story (commit `9ca7fe48`), selon la règle « un LOW facile
  qui touche la production se corrige, puis une passe ciblée » : **L1** (fin demandée future : `resumedNotice`
  la ramène au jour de Zurich avant la comparaison, `max` = aujourd'hui sur le champ *Au*) ; **L2** (le test du 409
  lance la première tâche avec une fin future et attend la fin lue) ; **L3** (`starting` remis à faux par le seul
  `POST` de la génération courante) ; **L4** (E2E : le serveur simulé retient les fenêtres 2 et suivantes jusqu'au
  409) ; **L6** (avis au passé à l'issue, `provider-download-resumed-ended` ; « suivi perdu » qui nomme la période
  après une reprise, `provider-download-lost-resumed` ; quatre locales, manuel utilisateur). **L5** et **L7** :
  notés au Dev Agent Record. Une passe **P3 ciblée** sur `9ca7fe48` suit
  (`kesh-gate-logs/s38-12b/revue-p3-prompt.md`).
- 2026-11-22 — revue de code P3 ciblée sur `9ca7fe48` (une lentille ; rapport `kesh-gate-logs/s38-12b/revue-p3-rapport.md`) :
  0 CRITICAL, **1 HIGH**, 0 MEDIUM, 1 LOW — tous deux nés de la remédiation P2. **HIGH-1** : le `finally` de `start()`
  comparait `generation`, que `track()` incrémente aussitôt (`stopPolling()`) : « Télécharger » restait désactivé après
  tout lancement réussi ou 409 repris (« Nouvelle période »). Corrigé (commit `369963704`) par un compteur de
  lancements `launches`, distinct de la génération du suivi : seul le dernier lancement remet `starting` à faux, ce
  qui garde L3. **LOW-1** (corrigé, test seulement) : la retenue du serveur simulé (`MockReply.hold`) appartient au
  serveur lancé, `ProviderMock.release()` la libère, `close()` aussi (plus de promesse de module dans la spec). Une passe
  **P4 ciblée** sur `369963704` suit (`kesh-gate-logs/s38-12b/revue-p4-prompt.md`).
- 2026-11-22 — revue de code P4 ciblée sur `369963704` (une lentille ; rapport `kesh-gate-logs/s38-12b/revue-p4-rapport.md`) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW** — la boucle se clôt. Son observation hors finding (antérieure au commit :
  `server.close()` du serveur simulé attend la fin d'une connexion gardée ouverte, quelques secondes) est appliquée,
  code de test seul : `server.closeAllConnections()` après `server.close()` (`provider-mock.ts`) ;
  `paypal-download.spec.ts` rejouée : **3 passés** (27,6 s, `p4-e2e.console`, `e2e-p4.log`, arbre de travail sur
  `fe396bd3e` + ce seul changement, commité ensuite sans autre modification).
- 2026-11-22 — **synthèse de la revue de code** : P1 (complète, Opus 5.5) 1 MEDIUM, 6 LOW → P2 (complète, Sonnet 5.5)
  0 au-dessus de LOW, 7 LOW, cinq corrigés en production → P3 (ciblée, Sonnet 5.5) 1 HIGH, 1 LOW, nés de la remédiation
  P2 → P4 (ciblée sur la remédiation P3 ; modèle non consigné au rapport) 0 finding. Tendance au-dessus de LOW :
  1 → 0 → 1 → 0 ; la P3 illustre le motif « la sévérité se déplace vers ce qu'on vient d'écrire ». Aucun reclassement
  en dette ; les LOW non corrigés (P1 L5, L6 ; P2 L5, L7) sont au Dev Agent Record, à ouvrir en issue. Story `done`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), worktree `kesh-wt38b`, branche `story/38-12b` partie d'`epic/38-paypal`
(`8b8cd338`), cible `kesh-targets/kesh-wt38b`, `CARGO_INCREMENTAL=0`.

### Prémisses relues au code (avant la première ligne)

Voir « Amendement du 22.11 » : page commune des prestataires absente de la branche (le dialogue générique est
écrit ici, le compte rendu de l'import extrait) ; `providerApi` sur la seule liste ; dates de l'API converties à
l'heure de Zurich par la 38-12a (le manuel le dit) ; G4 de la 38-12a (première heure du jour plancher non lue) au
manuel utilisateur ; gates de la nuit du 22.11.

### Implementation Plan / décisions

- **Backend (T1)** : `provider_env::ProviderApiStatus { configured, mode? }`, `ProviderApiMode`
  (`live` = hôte par défaut du prestataire, `sandbox` = un segment d'hôte `sandbox`, absent sinon),
  `ProviderApiConfig::status_for` (règle de `series_for`, celle de la route), `ProvidersConfig::status_for(provider,
  account)` (prestataire non téléchargé → `configured: false`) ; `routes/bank_accounts.rs` : champ
  `provider_api` de `BankAccountWithBalance` (liste seulement), `null` pour un compte ordinaire.
- **Dialogue générique (T2)** : `frontend/src/lib/features/provider-download/` — `provider-download.api.ts`
  (routes `<prestataire>-downloads`, types `ProviderApiStatus`, `ProviderDownloadView`), `provider-download.ts`
  (règles pures : période proposée, refus du lancement avec reprise sur 409, phrases d'échec par code, progression,
  politique de suivi : 1 s, 404 ou trois erreurs réseau consécutives = suivi perdu), `ProviderDownloadDialog.svelte`
  (générique `R`, libellés et compte rendu fournis par l'appelant en snippet, garde de génération : suivi arrêté à la
  fermeture, au démontage et à tout nouveau lancement). Aucun nom de prestataire dans le module ni ses clés
  `provider-download-*`.
- **Page PayPal (T2)** : bouton « Télécharger depuis PayPal » (Comptable et Administrateur, compte configuré),
  mention « bac à sable », ligne d'aide de l'Administrateur sans identifiants ; période proposée = exercice choisi,
  fin au jour de Zurich (`todayInZurich`, le jour que juge le serveur) ; revue relue à la fin ; compte rendu
  **extrait** en `features/paypal/PaypalImportReport.svelte` (préfixe d'identifiants de test paramétré : celui de la
  38-11d inchangé pour l'import CSV, `paypal-download-report` dans le dialogue) ; code de ligne `STATUS` libellé.
  `paypalApiStatus` (`bank-accounts/provider.ts`) lit l'état d'un libellé sur ses soldes.
- **i18n (AC-4)** : 39 clés aux quatre locales (32 `provider-download-*`, 6 `paypal-download-*`,
  `paypal-line-status`) ; `sitesTotal` 3632 → 3671 (relevé par la garde, recompté au diff hors tests : 42 sites
  ajoutés, 3 retirés) ; candidates des libellés 138 → 140 (`periodRefusalLabel`, `progressLabel`, conformes).
- **Manuels (T3)** : admin — § « Télécharger les transactions PayPal par l'API » (`sec:paypal-api` : création de
  l'application, points « à vérifier » en encadré, variables et défauts, `.env` et séries 1 à 3 des compose, une
  installation une société, secrets hors base et sauvegarde, sessions seules, rotation, refus de démarrage au texte
  de `provider_env::read`), paragraphe-tableau au § `sec:env-vars`, accès sortant `sec:acces-sortant-paypal` ;
  utilisateur — paragraphe « Télécharger depuis PayPal » (`sec:paypal-telecharger`), limites réécrites (dates,
  téléchargement à la main, la phrase « Pas de téléchargement automatique » retirée — L3 de la 38-11d).
  `.tex` seuls ; PDF régénérés dans une copie hors dépôt (`latexmk -xelatex`, rc 0, aucune référence indéfinie),
  texte contrôlé par `pdftotext | tr '\n' ' '` (sections, tableau, accès sortant, paragraphe utilisateur ; la phrase
  retirée absente).
- **`docs/api-external.md` (T3)** : encart du § 4, liste du § 7 « à cinq exceptions près, toutes cinq » (recompté :
  administration, journal d'audit, suppression d'import, mot de passe, `paypal-downloads*`), ligne du § 9, ligne
  `API_KEY_OPERATION_FORBIDDEN` du § 10, codes `PROVIDER_*` du lancement et des tâches, section « Télécharger le relevé
  d'un compte PayPal », `providerApi`.
- **E2E (T4)** : `frontend/tests/e2e/support/provider-mock.ts` (serveur simulé commun : routes déclarées par
  prestataire, requêtes relevées ; jeu PayPal : jeton, recherche bornée à la fenêtre demandée, pages retardées,
  un `429` `Retry-After: 3`) ; `paypal-download.spec.ts` (trois tests : préparation et `providerApi` à l'API ; (a)
  compte sans identifiants ; (b) deux mois — progression, pause, compte rendu, revue, relance « 0 insérée » ; (b)
  ignoré avec motif sans `KESH_E2E_PROVIDER_MOCK_PORT`) ; montage et ignoré connu à `docs/testing.md`.

### Debug Log

- **Défaut trouvé par l'E2E** (premier passage, `e2e-1.log`) : `parseBankAccount` (`bank-accounts.api.ts`)
  recopiait les champs un à un et perdait `providerApi` — les tests de la page, qui simulent `listBankAccounts`, ne
  pouvaient pas le voir. Corrigé (`6712d85b`) avec un test du mappage (`listBankAccounts garde providerApi`).
- Deuxième passage (`e2e-2.log`) : la période proposée part du début de l'exercice qui couvre le jour
  (`with-company` : 2020-2030), non du 1er janvier — attente de la spec corrigée (`f520d426`).
- Gate frontend 1 : `svelte-check` (type du snippet du test du dialogue) et deux gardes à recompter
  (`sitesTotal`, candidates des libellés) — `5b8d3e18`.
- Gate backend complet (`gate-backend-1.log`, arbre `f520d426`) : 6421 exécutés, **6419 passés, 2 échecs** :
  `kesh-i18n loader::tests::la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` (de-CH « schliessen » réservé aux
  panneaux, dans `provider-download-close-hint`) — corrigé (`605f433f`, texte seul) ; et
  `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris` (témoin temporel du transport MCP, sans rapport avec la
  story) — **vert au rejeu** (`cible-2-nextest.log`).

### Completion Notes

- **Tests ajoutés** (périmètre `8b8cd338` → `605f433f`, recomptés) : Rust **2** (`provider_env::tests::
  etat_du_telechargement_d_un_compte`, `paypal_download_e2e::la_liste_des_comptes_dit_le_telechargement_configure`) ;
  Vitest **33** (`provider-download.test.ts` 14, `ProviderDownloadDialog.test.ts` 11, `provider-api.test.ts` 3,
  `paypal-page.test.ts` 19 → 23, `bank-accounts.api.test.ts` 13 → 14) ; Playwright **3** (`paypal-download.spec.ts`).
- **Sondes négatives** (copies hors dépôt, scratchpad `s38-12b-mut/`) : frontend 8 mutations (F1 à F8),
  copie non mutée verte (51 tests), **8 tuées sur 8**, chacune par un test nommé en échec d'assertion
  (`kesh-gate-logs/s38-12b/mutations-front.log`) ; backend 4 mutations (B1 à B4, cible à part supprimée après),
  copie non mutée verte, **4 tuées sur 4**, chacune par la panique d'assertion du test nommé — le script les
  classait « douteuses » (motif du nom de fil sans l'identifiant `(tid)`), annotation à la fin de
  `mutations-back.log`.
- Captures (thème clair) : `kesh-gate-logs/s38-12b/captures/10` à `17` (compte sans identifiants, bouton, période,
  progression, pause, compte rendu, revue après téléchargement, relance sans doublon).
- Aucun appel réel à PayPal ; aucune migration ; aucun PDF ni CHANGELOG commité.

### Gates

- Dernier commit de code `605f433f` (texte de-CH d'un catalogue).
- Backend complet (`scripts/test-fast.sh --ci`, base remise à zéro, `gate-backend-1.log`, arbre `f520d426`) : fmt et
  clippy workspace verts ; nextest **6421 exécutés, 6419 passés, 2 échecs, 15 ignorés** (ci-dessus) ; puis, au
  commit `605f433f`, `package(kesh-i18n) | binary(mcp_transport_e2e)` : **79 sur 79** (`cible-2-nextest.log`).
  Gate ciblé antérieur (`cible-1-*`) : fmt, clippy, nextest `kesh-api` lib + `paypal_download_e2e` +
  `bank_accounts_e2e` + `textes_coherents` : 862 sur 862.
- Frontend au commit `5b8d3e18` (`front-3-*.log`) : `check` 0 erreur, `lint-i18n-ownership` vert, Vitest
  **4980 passés (290 fichiers)**, `build` vert. `605f433f` ne change qu'une valeur de catalogue de-CH (clés
  inchangées) ; non rejoué.
- **E2E partiel (1 spec)** : `paypal-download.spec.ts` seule (spec ajoutée par la story, règle de la nuit du 22.11),
  backend de test à `KESH_PAYPAL_1_*` vers le serveur simulé (`e2e-partiel.sh`, `e2e-3.log`) : **3 passés** à
  `f520d426`. E2E complet et `gate-ci.sh` : à la fin de l'epic (branche d'intégration).

### Points relevés hors périmètre (à ouvrir en issue par l'orchestrateur)

1. Les réponses d'un seul compte bancaire (`POST`, `PUT`, `PATCH`, clôture…) ne portent pas `providerApi` : un écran
   qui remplacerait une ligne de la liste par la réponse d'une mutation la perdrait (aucun ne le fait aujourd'hui).
2. `ProviderApiConfig::series_for` compare les libellés par `to_lowercase`, la base par une collation sans accents :
   « PayPal Zürich » au compte et « PayPal Zurich » à la série ne s'apparient pas (le bouton n'apparaît pas, refus
   `PROVIDER_API_NOT_CONFIGURED`) — à aligner sur la collation, ou à dire au manuel.

### Remédiation de la revue P1

- **M1, prémisse relue** : `Downloads::start` refusait par (prestataire, société, libellé sans casse) sans regarder
  la période, et le `409` ne portait que `{ provider, jobId }` — vérifié au code avant correctif. Correctif :
  `Job.period`, `RunningJob { id, from, to }` rendu par `start` (signature : `period` en plus, avant
  `steps_total`), `AppError::ProviderDownloadRunning { provider, job_id, from, to }` (détails `AAAA-MM-JJ`, fin lue) ;
  frontend : `StartRefusal` porte `running: Period | null` (période mal formée ignorée), `resumedNotice(running,
  requested)`, avis `provider-download-resumed` (`role="status"`) montré hors formulaire jusqu'à l'issue. Une fin
  demandée dans le futur, ramenée au jour du jour par le serveur, peut faire paraître l'avis pour la même période
  (le texte dit alors la période réellement lue, ce qui reste exact) ; la page PayPal ne propose pas de fin future.
  **À l'intégration avec les Epics 39 et 40** : tout appel de `Downloads::start` d'une autre branche (Wise, Stripe)
  doit passer la période lue.
- **L1, L2, L3, L4** : voir le Change Log. L3 : l'E2E prend les 60 jours qui précèdent le jour (trois fenêtres, pause
  sur la deuxième), sans borne au 1er janvier, et retient un exercice couvrant toute la période ; il joue aussi la
  reprise d'une autre période (fermer, rouvrir, lancer `FROM + 1 jour` → avis, capture 18) et vérifie l'absence d'avis
  à la relance de la même période.
- **Tests ajoutés par la remédiation** (périmètre `bc7b7520` → `8b941584`, recomptés) : Vitest **9**
  (`provider-download.test.ts` +3 : `resumedNotice` ×2, `startRefusal` période ; `ProviderDownloadDialog.test.ts` +6 :
  409 autre période, 409 même période, fermeture pendant le `POST` ×2, `done` sans `result`, `onDone` qui lève) ;
  Rust **0** test neuf, assertions ajoutées à `une_tache_par_compte_tant_qu_elle_tourne` et
  `refus_tache_unique_et_cles_d_api` ; Playwright **0** neuf, test (b) étendu. `sitesTotal` 3671 → **3672** (relevé
  par la garde, `front-4-test-unit.log` ; au diff : 1 site ajouté).
- **Sondes négatives** : frontend 7 mutations (R1 à R7, copie hors dépôt à `node_modules` réel, non mutée verte,
  34 tests) **7 tuées sur 7** par test nommé en échec d'assertion (`kesh-gate-logs/s38-12b/mutations-front-p1.log`) ;
  serveur 3 mutations (B1 à B3, copie `git archive`, cible à part supprimée), non mutée verte, **3 tuées sur 3** par
  panique d'assertion nommée (`mutations-back-p1.log`).
- **Gates au dernier commit de code `8b941584`** : backend `scripts/test-fast.sh --ci` (base remise à zéro,
  `gate-backend-2.log`) : fmt, clippy workspace verts, nextest **6421 exécutés, 6421 passés, 15 ignorés** (le premier
  passage avait rougi sur `cargo fmt --check`, corrigé par `rustfmt` avant le commit ; lancé sous
  `env -u XDG_RUNTIME_DIR` : le bus systemd utilisateur refusait la connexion, `mem-guard` sans plafond, `heavy.sh`
  bornant la station) ; frontend (`front-5-*.log`) : `check` 0 erreur, `lint-i18n-ownership` vert, Vitest **4989
  passés (290 fichiers)**, `build` vert ; **E2E partiel (1 spec)** `paypal-download.spec.ts` (`e2e-4.log`) : **3
  passés**. E2E complet et `gate-ci.sh` : à la fin de l'epic.
  *(Annoté le 22.11, revue P2, L7 : ce gate backend a tourné sur l'**arbre de travail** de `8b941584` avant son
  commit — `gate-backend-2.console` lit HEAD `bc7b7520`, commit daté de 06:29:19, gate fini à 06:32 après 921 s de
  nextest ; le journal ne prouve donc pas l'identité du contenu. Le gate backend de la remédiation de la revue P2
  tourne, lui, au commit, ci-dessous.)*
- **L5 de la revue P2** (non corrigé, théorique) : l'exercice de repli de la préparation E2E
  (`paypal-download.spec.ts`, test 1 : du 1er janvier de l'année de `FROM` au 31 décembre de l'année du jour) peut
  chevaucher un exercice existant qui ne couvre qu'une partie de `FROM..TODAY`, et l'API le refuserait
  (`FY_OVERLAP_KEY`). Avec la graine `with-company` (2020-2030), cela n'arrive qu'en janvier-février 2031, ou plus tôt
  avec une graine découpée en exercices annuels : l'échec du `POST /fiscal-years` le dirait alors (message de l'API
  dans l'assertion de `api()`).
- **L5** (non corrigé, hors périmètre) : le serveur simulé commun apparie la route par chemin exact
  (`r.path === recorded.path`) ; réutilisable tel quel pour Stripe, non pour Wise (identifiants dans le chemin) :
  prévoir un `match(path)` dans `ProviderMockRoute` à la 39-12b. Noté aux consignes des correctifs.
- **L6** (non corrigé, joint au point 2 ci-dessus) : `paypalApiStatus` apparie les libellés sans accents et prend le
  premier solde configuré ; avec deux soldes « Zurich »/« Zürich », le bouton peut paraître pour un libellé que
  `series_for` refusera (`PROVIDER_API_NOT_CONFIGURED`, message juste).

### Remédiation de la revue P2

- **Prémisses relues au code** avant correctif : L1 — `routes/paypal_downloads.rs` garde `effective_to(request.to,
  now)` et le passe à `start`, `resumedNotice` comparait la fin demandée telle quelle, le champ *Au* n'avait pas de
  `max` ; L2 — le seul 409 testé lançait `2020-01-01 → 2020-01-30` sous une horloge au 2020-06-30 ; L3 — le
  `finally` de `start()` remettait `starting` à faux sans condition ; L4 — la spec dépendait des 900 ms par page et
  de la pause de 3 s ; L6 — l'avis restait au futur en `done`, et « Suivi perdu : relancez la même période » ne
  disait pas laquelle après une reprise. Toutes vérifiées.
- **Correctif** (`9ca7fe48`) : `resumedNotice(running, requested, today)` (fin demandée ramenée à `today`) ; dialogue
  : `today` lu à l'ouverture (`todayInZurich`), `max={today}` sur *Au*, `todayInZurich()` passé à la comparaison,
  `finally { if (myGen === generation) starting = false; }`, avis `provider-download-resumed` en `running`,
  `provider-download-resumed-ended` (passé) en `done` et `failed`, caché en `lost`, où
  `provider-download-lost-resumed` nomme la période suivie ; quatre locales ; manuel utilisateur (§ « Fermer la fenêtre
  n'arrête pas… »). Serveur simulé : `MockReply.until` (au plus `HOLD_CAP_MS` = 45 s, sous le délai total de 60 s
  d'une requête du backend), `holdFromWindow` ; spec : `held`/`releaseHold()` libéré une fois l'avis de reprise vu.
  Test serveur : première tâche `2020-06-01 → 2020-07-15`, 409 attendu `2020-06-01 → 2020-06-30` (message nommé
  « 409 : période de la tâche suivie, fin lue »).
- **Tests ajoutés** (périmètre `707d93e8` → `9ca7fe48`, recomptés) : Vitest **4** (`provider-download.test.ts` +1 :
  fin future ramenée ; `ProviderDownloadDialog.test.ts` +3 : 409 sur une fin future et `max`, suivi perdu après une
  reprise, ancien `POST` après réouverture), et une assertion de texte au passé ajoutée au test « 409 d'une AUTRE
  période » ; Rust **0** neuf, `refus_tache_unique_et_cles_d_api` modifié ; Playwright **0** neuf, test (b) rendu
  indépendant du temps. `sitesTotal` 3672 → **3674** (garde verte au gate ; au diff : 2 sites nets).
- **Sondes négatives** : frontend 6 mutations (R1 fin non ramenée, R2 `max` retiré, R3 `finally` sans garde, R4 avis
  au futur à l'issue, R5 suivi perdu sans la période, R6 avis montré en `lost`), copie hors dépôt à `node_modules`
  réel, non mutée verte (38), **6 tuées sur 6** par test nommé en `AssertionError`
  (`kesh-gate-logs/s38-12b/mutations-front-p2.log`) ; serveur : la mutation `(request.from, request.to)` à `paypal_downloads.rs` sur une copie `git archive` à cible propre (supprimée),
  non mutée verte, **tuée** par `panicked at …paypal_download_e2e.rs:1292` et le message « 409 : période de la
  tâche suivie, fin lue » (`mutations-back-p2.log`).
- **Gates au commit `9ca7fe48`** (HEAD lu dans chaque console, cette fois après le commit) : backend
  `scripts/test-fast.sh --ci` sous `heavy.sh` (`KESH_NO_MEMGUARD=1`, base remise à zéro ; `p2-gate-backend.console`,
  `gate-backend-p2.log`) : fmt et clippy workspace verts, nextest **6421 exécutés, 6421 passés, 15 ignorés** ;
  frontend (`p2-front.console`, `front-p2-*.log`) : `check` 0 erreur, `lint-i18n-ownership` vert, Vitest **4993
  passés (290 fichiers)** (= 4989 + 4), `build` vert ; **E2E partiel (1 spec)** `paypal-download.spec.ts`
  (`p2-e2e.console`, `e2e-p2.log`) : **3 passés**. Captures 10 à 18 reprises (la 15 montre l'avis au passé sous le
  compte rendu). E2E complet et `gate-ci.sh` : à la fin, sur la branche d'intégration.

### Remédiation de la revue P3

- **Prémisse relue au code** : `start()` prend `myGen = generation`, puis `track()` commence par `stopPolling()`
  (`generation += 1`) : au `finally`, `myGen !== generation` sur les deux chemins qui suivent une tâche — vérifiée par
  les deux tests neufs, rouges sur la garde de la P2 (mutation R9 ci-dessous).
- **Correctif** (`369963704`) : `let launches = 0`, `const myLaunch = ++launches` dans `start()`,
  `finally { if (myLaunch === launches) starting = false; }` ; `myGen`/`generation` gardent seuls le suivi (aucun
  `track()` d'une génération passée). Serveur simulé : `MockReply.hold` (au lieu de `until`), retenue propre à chaque
  serveur, `ProviderMock.release()`, `close()` qui libère avant `server.close()`, minuteur du plafond annulé ; spec :
  `holdFromWindow: { window: 2 }`, `mock?.release()` là où était `releaseHold()`.
- **Tests ajoutés** (périmètre `d4b4512c` → `369963704`, recomptés) : Vitest **2** (`ProviderDownloadDialog.test.ts` :
  lancement réussi puis « Nouvelle période », 409 repris puis « Nouvelle période » — bouton actif) ; Rust 0 ;
  Playwright 0 (spec adaptée au serveur simulé). `sitesTotal` inchangé (aucun texte touché).
- **Sondes négatives** (copie `git archive 369963704 frontend` hors dépôt, `node_modules` réel par `npm ci`, supprimée ;
  `kesh-gate-logs/s38-12b/mutations-front-p3.log`) : non mutée verte (40) ; **R7** condition du `finally` retirée →
  tuée par « ancien POST qui répond après une réouverture … (revue P2, L3) » (`AssertionError: expected false to be
  true`) ; **R8** remise à faux retirée → tuée par les deux tests neufs (`AssertionError: bouton actif après …`) ;
  **R9** garde de la P2 rétablie (`myGen === generation`) → tuée par les deux tests neufs. Trois sur trois.
- **Gates au commit `369963704`** (HEAD lu dans chaque console ; aucun `.rs` touché, pas de gate Rust) : frontend
  (`p3-front.console`, `front-p3-*.log`) : `check` 0 erreur, `lint-i18n-ownership` vert, Vitest **4995 passés (290
  fichiers)** (= 4993 + 2), `build` vert ; **E2E partiel (1 spec)** `paypal-download.spec.ts` (`p3-e2e.console`,
  `e2e-p3.log`) : **3 passés** (29,2 s). E2E complet et `gate-ci.sh` : à la fin, sur la branche d'intégration.

### File List

Ajoutés : `frontend/src/lib/features/provider-download/{provider-download.api.ts, provider-download.ts,
ProviderDownloadDialog.svelte, provider-download.test.ts, ProviderDownloadDialog.test.ts}`,
`frontend/src/lib/features/paypal/PaypalImportReport.svelte`, `frontend/src/lib/features/bank-accounts/provider-api.test.ts`,
`frontend/tests/e2e/paypal-download.spec.ts`, `frontend/tests/e2e/support/provider-mock.ts`.
Modifiés : `crates/kesh-api/src/provider_env.rs`, `crates/kesh-api/src/routes/bank_accounts.rs`,
`crates/kesh-api/src/{provider_download.rs, errors.rs, routes/paypal_downloads.rs}` (revue P1),
`crates/kesh-api/tests/paypal_download_e2e.rs`, `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`,
`frontend/src/routes/(app)/reconciliation/paypal/{+page.svelte, paypal-page.test.ts}`,
`frontend/src/lib/features/paypal/{paypal.types.ts, paypal-labels.ts}`,
`frontend/src/lib/features/bank-accounts/{bank-accounts.api.ts, bank-accounts.api.test.ts, provider.ts}`,
`frontend/src/lib/shared/{i18n-keys.test.ts, i18n-libelle-en-dur.test.ts}`, `docs/manual/fr/{admin-manual.tex,
user-manual.tex}`, `docs/api-external.md`, `docs/testing.md`,
`_bmad-output/implementation-artifacts/{38-12b-ecran-manuels-paypal-api.md, sprint-status.yaml}`.
