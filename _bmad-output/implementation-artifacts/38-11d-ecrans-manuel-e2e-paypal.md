# Story 38-11d : écrans PayPal, manuel utilisateur, API externe, E2E

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, après la fusion de `origin/main` (commit `43ceb541`). Epic
38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11b, 38-11c (routes). Reprend, réduits,
l'ancienne 38-9a (réglages d'un compte prestataire) et 38-9b (règles étendues, propositions, « à examiner »).*

## Story

En tant que **Comptable**, je veux régler mes comptes PayPal depuis l'écran des comptes bancaires, importer le CSV
d'un clic, voir les propositions, les écarts et les lignes à examiner sur une page, et accepter la sélection ; en
tant qu'**auteur d'un script**, je veux la même chose par l'API, documentée.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11d-1` | Réglages PayPal au formulaire du compte bancaire (compte PayPal, compte de frais), IBAN facultatif, compte du plan en devise admis. | AC-1 |
| `FR-38-11d-2` | Critères « Type PayPal » et « E-mail exact », sens facultatif, au formulaire et à la liste des règles. | AC-2 |
| `FR-38-11d-3` | Page PayPal : compte PayPal, exercice, bouton d'import et compte rendu, propositions à cocher, écarts, à examiner. | AC-3 à AC-5 |
| `FR-38-11d-4` | Manuel utilisateur (`.tex`) et `docs/api-external.md` décrivent ce que fait le code, limites comprises. | AC-6, AC-7 |
| `NFR-38-11d-1` | Un E2E traverse l'import et l'acceptation par l'interface ; captures d'écran de chaque état (recette visuelle). | AC-8 |

## Critères d'acceptation

1. **AC-1 — Comptes bancaires** (`frontend/src/routes/(app)/bank-accounts/+page.svelte`,
   `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte`, `bank-accounts.api.ts`). Case
   « Compte PayPal » au formulaire de création et de modification d'un compte de type *Compte bancaire* ; cochée :
   champ « Compte PayPal » (libellé, suggestions = libellés déjà employés, p. ex. « PayPal www »), sélecteur « Compte
   de frais » (comptes de charge imputables, CHF ou de la devise du compte lié), IBAN **facultatif**, QR-IBAN masqué,
   « compte principal » masqué ; le sélecteur du compte du plan **offre les comptes en devise** (il les écarte
   aujourd'hui, C-37-5). Corps `paypal: { account, feeAccountId }` ; décochée sur un compte existant → `paypal: null`.
   Refus de 38-11b affichés par leur message traduit (dont `PAYPAL_CURRENCY_DUPLICATE`, `PAYPAL_ACCOUNT_IN_USE`). La
   liste des comptes montre « PayPal · <compte> · <devise> » sous le nom. Vitest : corps envoyé (case cochée,
   décochée, inchangée), champs masqués, comptes en devise offerts seulement case cochée.
2. **AC-2 — Règles** (`frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`, `RulesList.svelte`,
   `rules.types.ts`, `rules.api.ts`). `MatchType` gagne `provider_type` (« Type PayPal (exact) ») et
   `counterparty_email_exact` (« E-mail exact ») ; sélecteur « Sens » (« Entrées et sorties », « Entrées »,
   « Sorties » → `null`, `credit`, `debit`) à la création et à la modification ; la liste affiche le sens s'il est
   posé ; un type inconnu s'affiche brut (comportement actuel). Vitest : libellés, corps envoyé, `PATCH` sans sens
   inchangé.
3. **AC-3 — Page PayPal** `frontend/src/routes/(app)/reconciliation/paypal/+page.svelte` (feature neuve
   `frontend/src/lib/features/paypal/` : `paypal.api.ts`, `paypal.types.ts`), entrée « PayPal » dans le menu du
   rapprochement, visible dès qu'un compte bancaire porte des réglages PayPal. En tête : sélecteurs « Compte PayPal »
   (libellés distincts des comptes) et « Exercice » (exercice courant par défaut). Bouton **« Importer le CSV
   PayPal »** : choix du fichier, envoi à `POST /api/v1/paypal-imports`, puis **compte rendu** affiché jusqu'au
   prochain import (par devise : insérées, doublons ; écartées : non terminées avec leur liste, `Memo`, paires
   neutres) ; refus traduits (`PAYPAL_CURRENCY_ACCOUNT_MISSING` nomme les devises et renvoie aux comptes bancaires).
   La revue (`GET /api/v1/paypal/review`) se recharge après l'import et après l'acceptation.
4. **AC-4 — Propositions.** Tableau : date, genre (« Vente rattachée », « Frais », « Règle <libellé> »,
   « Conversion », « Retrait »), montant et devise, nom, TXN, ce qui sera fait (écriture rattachée n° … ; compte de
   frais ; contrepartie ; deux écritures au compte de conversion ; compte de destination). Une case par ligne,
   « Tout cocher » ; le retrait sans compte de destination porte un sélecteur (comptes d'actif CHF imputables) et
   reste décoché tant qu'il est vide. **« Accepter la sélection »** → `POST /api/v1/paypal/accept` ; compte rendu :
   acceptées, et chaque refus avec son message (`failed-proposal-label.ts` étendu aux codes neufs et à
   `RATE_UNAVAILABLE`, `FX_ACCOUNT_INVALID`). Lot de plus de 500 propositions : envoyé en paquets de 500, l'écran le
   dit.
5. **AC-5 — Écarts et à examiner.** Deux tableaux en lecture : **Écarts** — « Encaissement sans vente »,
   « Vente sans encaissement » (lien vers l'écriture), « Montant ou devise différents » (attendu et trouvé) ;
   **À examiner** — motif traduit (« Plusieurs ventes portent cette TXN », « Compte de frais non réglé », « Conversion
   sans sa paire », « Aucune règle »), type PayPal, nom, e-mail ; action **« Rattacher à une écriture »** qui ouvre le
   rattachement existant (`LinkSettlementModal.svelte`, mode écriture, routes `entry-candidates` / `link-entry`,
   désormais en devise, 38-11c AC-1) ; lien « Écriture manuelle » vers la saisie. Export CSV des deux tableaux
   (fonction d'export du dépôt si elle existe, sinon un bouton qui télécharge le CSV construit à l'écran). Libellés
   aux quatre locales (`crates/kesh-i18n/locales/*`, propriété des clés selon `docs/i18n-key-ownership-pattern.md`) ;
   `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` **recompté** depuis la source.
6. **AC-6 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, **le `.tex` seul** : les PDF se régénèrent à la
   release). Sous-section « Comptes PayPal » dans la section *Import bancaire* (`\section{Import bancaire}`, `:2995`)
   ou après *Comptabilité multi-devises* (`:1063`), avec renvois : un compte bancaire par compte PayPal et par devise,
   lié au compte du plan de la devise, sans IBAN ; compte de frais (CHF ou devise) ; exporter de PayPal le rapport
   « Balance affecting transactions » en CSV, colonnes en anglais ; ce que fait l'import (filtres, frais en ligne à
   part, doublons, paires neutres) ; les ventes viennent des boutiques avec « TXN <id> » dans le libellé ; le
   rapprochement et les trois écarts ; les règles « Type PayPal », « E-mail exact », le sens et la **limite** d'une
   règle par type et valeur ; la conversion en deux écritures au même CHF par le compte de conversion ; le retrait ;
   les **limites** : date de l'export sans conversion de fuseau (fin d'année), remboursements et litiges à la main,
   aucun code TVA, pas de téléchargement automatique (phase 2). Les autres langues : « à traduire » noté (manuels
   DE/IT/EN, règle du dépôt). Le PDF est contrôlé par le prompt de revue (`pdftotext … | tr '\n' ' '`) **après**
   régénération locale, sans le commiter.
7. **AC-7 — `docs/api-external.md`.** Section neuve après *Accepter des propositions de rapprochement* (`:711`) :
   `POST /api/v1/paypal-imports` (multipart, réponse, refus), `GET /api/v1/paypal/review`, `POST
   /api/v1/paypal/accept` (formes des propositions, *Pattern batch*), champs `paypal` des comptes bancaires, critères
   et `direction` des règles, rattachement en devise ; un exemple `curl` d'import et d'acceptation avec une clé
   `read-write` ; le script des ventes des boutiques (R1) écrit ses écritures par `POST /api/v1/journal-entries` avec
   « TXN <id> » dans le libellé et la ligne PayPal en devise.
8. **AC-8 — E2E et recette visuelle.** `frontend/tests/e2e/paypal-import.spec.ts` : société multi-devises préparée
   par l'API (comptes du plan EUR et USD, comptes bancaires PayPal EUR, USD et CHF, compte de frais, cours mensuels,
   compte de conversion, une règle par e-mail, deux écritures de vente « TXN … » dont une de montant faux) ; import de
   la fixture fictive de 38-11a **par l'interface** ; compte rendu attendu ; propositions, un écart de chaque genre ;
   « Tout cocher », « Accepter la sélection » → aucun refus, revue vide de propositions ; une clé API de la sonde
   négative n'est pas nécessaire (le smoke la porte). Captures d'écran de chaque état (formulaire du compte PayPal,
   règle « E-mail exact », compte rendu d'import, propositions, écarts, à examiner, compte rendu d'acceptation),
   thème clair, hors dépôt (`kesh-gate-logs/`), jointes au Dev Agent Record et au prompt de revue. Suite E2E complète
   au dernier commit de code (7 à 9 échecs attendus, jugés fichier par fichier, `docs/testing.md`).

## Tâches

- [ ] T1 — Formulaire et liste des comptes bancaires (AC-1).
- [ ] T2 — Règles (AC-2).
- [ ] T3 — Feature et page PayPal : import, propositions, écarts, à examiner (AC-3 à AC-5) ; i18n, `sitesTotal`.
- [ ] T4 — Manuel utilisateur, `docs/api-external.md` (AC-6, AC-7).
- [ ] T5 — E2E, captures, suite complète (AC-8).

## Stratégie de tests

Vitest pour chaque composant touché (corps envoyés, états, refus traduits) ; `npm run check`,
`npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; E2E de l'AC-8 puis la suite complète au dernier
commit de code ; garde `textes_coherents.rs` verte (textes du manuel).

## Modules touchés (5)

`frontend` (bank-accounts, reconciliation/rules, paypal neuve), `kesh-i18n` (libellés), manuel utilisateur FR,
`docs/api-external.md`, E2E.

## Note de version

*Ajouté* — Écran PayPal : import du CSV d'un clic, propositions à accepter en lot, écarts entre boutiques et PayPal,
lignes à examiner ; réglages PayPal au formulaire du compte bancaire ; critères de règle « Type PayPal » et « E-mail
exact », avec un sens facultatif.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
