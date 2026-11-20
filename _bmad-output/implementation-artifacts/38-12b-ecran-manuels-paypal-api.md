# Story 38-12b : bouton « Télécharger depuis PayPal », manuels, E2E

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`. Epic 38, **phase 2**. **Dépend de** : 38-12a (routes), 38-11d
(la page PayPal `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte`, son import CSV et son compte rendu,
38-11d AC-3). Scindée de 38-12a (voir sa fiche).*

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
   la régénération de la release (`pdftotext … | tr '\n' ' '`). *(La phrase du registre des routes est mise à jour par
   38-12a.)*
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

- [ ] T1 — `providerApi` à la réponse des comptes (AC-1).
- [ ] T2 — Bouton, dialogue, interrogation, i18n (AC-1 à AC-4).
- [ ] T3 — Manuels et `docs/api-external.md` (AC-5, AC-6).
- [ ] T4 — E2E, serveur simulé, `docs/testing.md` (AC-7).

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
