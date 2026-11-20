# Story 38-12b : bouton « Télécharger depuis PayPal », manuels, E2E

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`. Epic 38, **phase 2**. **Dépend de** : 38-12a (routes), 38-11d
(l'écran des comptes bancaires PayPal, la page de rapprochement). Scindée de 38-12a (voir sa fiche).*

## Story

En tant que **Comptable**, je veux, depuis mon compte bancaire PayPal, choisir une période et lancer le téléchargement,
voir où il en est, lire son compte rendu et passer au rapprochement ; en tant qu'**Administrateur**, je veux savoir au
manuel comment créer l'application PayPal en lecture seule et poser ses identifiants.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-12b-1` | Un bouton « Télécharger depuis PayPal » n'apparaît que si le compte PayPal a ses identifiants dans l'environnement. | AC-1 |
| `FR-38-12b-2` | La période se choisit ; la progression (« fenêtre 5/12 ») et les pauses se voient ; le délai de trois heures se dit. | AC-2, AC-3 |
| `FR-38-12b-3` | Le manuel d'administration dit les variables et la création de l'application ; le manuel utilisateur le téléchargement. | AC-5, AC-6 |
| `NFR-38-12b-1` | L'E2E exerce le parcours contre un serveur PayPal simulé ; aucune donnée réelle. | AC-7 |

## Critères d'acceptation

1. **AC-1 — Indicateur et bouton.** La réponse des comptes bancaires (38-11b) gagne `paypal.apiConfigured: boolean`
   (vrai si une paire existe au libellé du compte ; la valeur des secrets n'y paraît jamais ;
   `crates/kesh-api/src/routes/bank_accounts.rs`). Écran des comptes bancaires de 38-11d : bouton **« Télécharger depuis
   PayPal »** sur un compte PayPal, visible de Comptable et plus quand `apiConfigured` ; sinon une ligne d'aide « non
   configuré : voir le manuel d'administration » (Administrateur seulement), et le bouton « Importer un CSV » de 38-11d
   reste seul.
2. **AC-2 — Fenêtre de dialogue.** Période (date de début, date de fin ; défaut : début de l'exercice ouvert à
   aujourd'hui), refus du serveur affichés par code (`PAYPAL_PERIOD_INVALID`, `PAYPAL_PERIOD_TOO_OLD`,
   `PAYPAL_DOWNLOAD_RUNNING`, `PAYPAL_API_NOT_CONFIGURED`) ; texte permanent : « PayPal peut mettre jusqu'à trois
   heures à publier une transaction ; l'historique remonte à trois ans ; le téléchargement se fait par tranches de 31
   jours ». Un seul téléchargement par compte PayPal à la fois.
3. **AC-3 — Progression et résultat.** Après le 202, le dialogue interroge `GET /api/v1/paypal-downloads/{jobId}` toutes
   les secondes : barre et texte « fenêtre `done`/`total` », message « PayPal demande de patienter, reprise dans `n` s »
   quand `pausedUntilSeconds` est non nul ; à `done`, le compte rendu de 38-11d (insérées, doublons, écartées par motif,
   par devise) et le lien vers la page de rapprochement ; à `failed`, le message du serveur avec la fenêtre et la page et
   la mention « rien n'a été enregistré, relancez » ; fermer le dialogue n'arrête pas la tâche (l'écran affiche « un
   téléchargement est en cours » sur 409). Aucun secret n'est affiché.
4. **AC-4 — i18n.** Libellés, messages d'erreur et textes des AC-1 à AC-3 aux quatre locales (`frontend/src/lib/i18n/`
   et `crates/kesh-i18n/locales/*`) ; `sitesTotal` de `i18n-keys.test.ts` recompté depuis la source ; garde
   d'appartenance des clés (`npm run lint-i18n-ownership`) verte.
5. **AC-5 — Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, `.tex` seul, aucun PDF commité). Section
   « Télécharger les transactions PayPal par l'API » : (a) les variables `KESH_PAYPAL_<n>_ACCOUNT`, `_CLIENT_ID`,
   `_SECRET` (le libellé **identique** à celui du compte PayPal de Kesh), `KESH_PAYPAL_BASE_URL`, `KESH_PAYPAL_PARALLEL`
   (1 par défaut, 6 au plus ; ce que le parallélisme risque : une limitation de débit plus vite atteinte) ; (b)
   **créer l'application** dans l'espace développeurs de PayPal, application « Live », **en ne cochant que la fonction
   « Transaction Search »** (aucune fonction de paiement : l'application ne peut que lire), recopier l'identifiant
   client et le secret dans l'environnement du serveur ; (c) ce que l'application ne fait jamais : écrire chez PayPal ;
   (d) les secrets ne sont **ni en base ni dans la sauvegarde** : après une restauration sur un autre serveur, on les
   repose ; (e) la rotation (changer le secret chez PayPal, puis dans l'environnement, redémarrer) ; (f) les refus de
   démarrage et leurs messages. Les points **non confirmés** de la fiche 38-12a (nom exact de la fonction dans l'interface
   de PayPal, conditions d'accès du compte) sont écrits **comme à vérifier**, jamais comme acquis. Le PDF se contrôle à la
   régénération de la release (`pdftotext … | tr '\n' ' '`).
6. **AC-6 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, section PayPal de 38-11d) : le téléchargement, la
   période, le délai de trois heures, la progression et la pause, ce qu'un échec veut dire (rien d'enregistré), le mélange
   avec le CSV (aucun doublon), la limite des dates (fuseau du compte PayPal, sans conversion, comme pour le CSV).
   `docs/api-external.md` : les deux routes, **réservées aux sessions** (clés d'API refusées), avec leurs codes.
7. **AC-7 — E2E.** `frontend/tests/e2e/paypal-download.spec.ts` : (a) sans identifiants dans l'environnement, le bouton
   est absent ; (b) avec un petit serveur PayPal simulé lancé par la spec (`127.0.0.1`, port lu de
   `KESH_E2E_PAYPAL_MOCK_PORT`) et le backend d'E2E démarré avec `KESH_PAYPAL_1_*` et `KESH_PAYPAL_BASE_URL` vers lui
   (ajout au montage de `docs/testing.md` ; le bloc de montage de `CLAUDE.md` n'est **pas** touché) : période de deux
   mois, progression visible, compte rendu, puis le rapprochement affiche les lignes ; relancer la même période → « 0
   insérée ». Sans la variable du port, (b) est **ignoré avec son motif écrit** (la ligne s'ajoute aux ignorés connus de
   `docs/testing.md`, jamais comptée verte). Vitest : le dialogue (états `running`, pause, `done`, `failed`, refus par
   code) et l'indicateur `apiConfigured`.

## Tâches

- [ ] T1 — `apiConfigured` à la réponse des comptes (AC-1).
- [ ] T2 — Bouton, dialogue, interrogation, i18n (AC-1 à AC-4).
- [ ] T3 — Manuels et `docs/api-external.md` (AC-5, AC-6).
- [ ] T4 — E2E, serveur simulé, `docs/testing.md` (AC-7).

## Stratégie de tests

Vitest du dialogue ; `npm run check`, `lint-i18n-ownership`, `build` ; E2E **complet au dernier commit de code** ;
gate backend complet (la réponse des comptes bancaires change). Captures d'écran de chaque état du dialogue (thème
clair), jointes au prompt de la revue.

## Modules touchés (3)

`frontend` (écran, E2E), `kesh-i18n` (+ `kesh-api` : le seul champ `apiConfigured`), manuels et `docs/`.

## Fichiers communs

`frontend/src/routes/(app)/bank-accounts/` (38-11d), catalogues i18n, `i18n-keys.test.ts`, manuels `.tex` (jamais le PDF
dans la branche), `docs/api-external.md`, `docs/testing.md`.

## Note de version

*Ajouté* — Bouton « Télécharger depuis PayPal » sur un compte bancaire PayPal configuré, avec la période, la progression
et le compte rendu ; le manuel d'administration décrit la création de l'application PayPal en lecture seule.

## Change Log

- 2026-11-20 — fiche rédigée (phase 2), non validée.
