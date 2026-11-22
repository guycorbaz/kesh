# Story 38-13 : solde de la dette de l'Epic 38

Status: ready-for-dev

*Fiche rédigée le 2026-11-22, branche `story/38-13` partie d'`epic/38-paypal` (`38f616561`). Story de solde
(`CLAUDE.md` § *Priorités des défauts*, « Tarir la source, puis solder ») : elle ne prend que la dette des stories de
l'Epic 38, dans un plafond d'issues.*

## Plafond

**Deux points.** Le fichier d'epic (`_bmad-output/planning-artifacts/epic-38-paypal.md`, l. 110) ne fixe aucun plafond :
il écrit « story de solde non prévue (la dette tracée en issues à la PR) ». À défaut, le plafond est **2**, par
décision de l'orchestrateur du 22.11 ; il est écrit ici et sera rappelé à la PR.

## Story

En tant que responsable de l'epic,
je veux solder, avant la PR de l'Epic 38, les deux points de dette les plus sûrs que ses stories ont laissés,
afin que la PR porte deux issues de moins à ouvrir, sans risque pour les livres.

## Dette candidate et choix

Sources : les entrées « (38-…) » et « (intégration 38/40) » de `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` qui
portent sur du code présent dans `epic/38-paypal` (la page commune `ProviderReviewPage` de l'Epic 40 n'y est pas), et
les LOW gardés aux Dev Agent Records des stories 38-11a à 38-12b.

| Point | Source | Label | Retenu |
|---|---|---|---|
| `series_for` compare les libellés sans ignorer les accents | (38-12b) | bug, P3 | **oui (A)** |
| Six clés `provider-gap-*` / `provider-review-*` sans site | (intégration 38/40) | technical-debt, P4 | **oui (B)** |
| Exercice par défaut `todayLocal()`, période `todayInZurich()` | (intégration 38/40) | bug, P4 | non (plafond) |
| Compte rendu CSV encore affiché après un téléchargement | (intégration 38/40) | bug, P4 | non (plafond ; constaté sur la page commune) |
| `providerApi` absent des réponses d'un seul compte | (38-12b) | enhancement, P4 | non (contrat de l'API) |
| `ACCOUNT_CURRENCY_IN_USE` et la désignation par une règle | (38-11c2 P3 L1) | bug, P3 | non (règle métier, manuel) |
| Devise absente du sélecteur de compte d'une règle | (38-11c2 P4 L5) | enhancement, P4 | non (plafond) |
| 38-12a F7, F8, F12 ; LOW des Dev Agent Records | fiches | technical-debt | non (plafond) |

Choix : les deux points les plus faciles et sans risque — A est un correctif local d'une fonction pure, testé par
tests unitaires ; B retire des textes qu'aucun code ne lit. Les autres restent à ouvrir en issues avec la PR.

## Acceptance Criteria

**AC-A1 — appariement d'un libellé sans accents.** `ProviderApiConfig::series_for` apparie le libellé d'un compte et
celui d'une série sans tenir compte de la casse **ni des accents** (« PayPal Zürich » au compte, « PayPal Zurich » à la
série, et l'inverse ; formes composée et décomposée d'une même lettre), comme la base, dont la collation
`utf8mb4_unicode_ci` compare `bank_accounts.provider_account` ainsi. Les blancs de bord restent ignorés.

**AC-A2 — doublon de libellé.** À la lecture de l'environnement, deux séries dont les libellés ne diffèrent que par la
casse ou les accents sont refusées (« libellé déjà porté par une autre série », la variable `KESH_<P>_<n>_ACCOUNT` de
la seconde nommée), comme aujourd'hui pour la casse seule : sinon un compte de Kesh s'apparierait à deux séries.

**AC-A3 — une seule règle.** Les deux comparaisons passent par une seule fonction documentée (DRY) ; `provider_api_status`
(qui lit `series_for`) suit sans changement.

**AC-A4 — documentation.** Le manuel d'administration FR (`docs/manual/fr/admin-manual.tex`, `KESH_PAYPAL_<n>_ACCOUNT`,
et la liste des refus « libellé déjà porté par une autre série ») dit « ni la casse ni les accents ». Pas de PDF.

**AC-B1 — clés orphelines retirées.** Les six clés `provider-gap-receipt-without-sale`,
`provider-gap-sale-without-receipt`, `provider-gap-amount-or-currency-mismatch`, `provider-review-conversion-unpaired`,
`provider-review-withdrawal-foreign-currency`, `provider-review-no-rule` sont retirées des quatre catalogues
(`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`). Les deux clés du même bloc que lisent les Epics 39 et 40
(`provider-review-ambiguous-sale`, `provider-review-fee-account-missing`, relevé par `git grep` sur `epic/39-wise` et
`epic/40-stripe`) restent, pour ne rien casser à la reprise d'`epic/38-paypal` par ces branches.

**AC-B2 — aucun lecteur.** Avant le retrait, `git grep` de chaque clé hors des `.ftl` (code Rust, frontend, docs,
manuels, tests) sur `epic/38-paypal`, `epic/39-wise` et `epic/40-stripe` ne rend rien ; les gardes de parité des
catalogues restent vertes.

**AC-C — trace.** Les deux entrées de `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` sont barrées, avec le commit et
la mention « corrigé par la 38-13, ne pas ouvrir » ; rien d'autre n'y est retiré.

## Tasks / Subtasks

- [ ] A (AC-A1 à AC-A4)
  - [ ] fonction privée de clé de libellé dans `crates/kesh-api/src/provider_env.rs` (rognage, NFD, retrait des
        marques combinantes, minuscules), employée par `series_for` et par la garde de doublon de la lecture
  - [ ] tests unitaires : apparié dans les deux sens, forme décomposée, doublon refusé à accents près ; libellé
        réellement différent toujours non apparié
  - [ ] mutation hors dépôt (retour à `to_lowercase` seul) tuée par une assertion nommée
  - [ ] manuel FR (`.tex`) ; grep du symptôme (« sans casse » sur ce libellé) dans le dépôt
- [ ] B (AC-B1, AC-B2)
  - [ ] grep de chaque clé sur les trois branches ; retrait des six clés des quatre catalogues
- [ ] C : barrer les deux entrées ; gates ; prompt de revue P1

## Dev Notes

- **Prémisse A, relue au code** : `series_for` (`provider_env.rs:199-204`) et la garde de doublon (`:459-466`)
  comparent par `to_lowercase()` ; `bank_accounts` est en `utf8mb4_unicode_ci` (migration de création, l. 17), et les
  recherches par libellé (`bank_accounts.rs:1649`, `:1694`) comparent `provider_account = ?` sous cette collation, donc
  sans casse ni accents. Conséquence : un compte « PayPal Zürich » et une série « PayPal Zurich » ne s'apparient pas à
  l'écran (`providerApi.configured` faux) ni à la route `POST /paypal-downloads` (`paypal_downloads.rs:101`).
- **Limite de l'approximation** : NFD + retrait des marques couvre les lettres accentuées ; la collation égale aussi
  `ß` et `ss` et quelques ligatures, que la fonction ne replie pas. À écrire dans son doc-comment, sans plus.
- Crate déjà présente : `unicode-normalization` (`crates/kesh-api/Cargo.toml:49`) ; `unicode_normalization::char::is_combining_mark`.
- **Prémisse B, relue** : les huit clés du bloc 38-11c2 (`fr-CH/messages.ftl` ~1897-1904) n'ont aucun lecteur sur
  `epic/38-paypal` ; deux sont lues sur `epic/39-wise` et `epic/40-stripe` (page commune) : elles restent. L'entrée
  d'`issues-a-ouvrir.md` parle bien de six.
- Aucune migration, aucun changement de contrat de l'API, aucun écran touché (texte d'interface retiré mais jamais
  affiché) : pas de capture, pas d'E2E (règle des gates de la nuit du 22.11 : E2E limité aux specs ajoutées, aucune
  ici).
- **Gates** (dernier commit de code) : `cargo fmt --check`, clippy workspace `-D warnings`, nextest complet ; frontend
  complet (`check`, `lint-i18n-ownership`, Vitest, `build`), les catalogues étant lus par le frontend. Pas de
  `gate-ci.sh` (fin de l'epic).

### References

- `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` (entrées « (38-12b) `series_for` … » et « (intégration 38/40) Six
  clés … »)
- `crates/kesh-api/src/provider_env.rs` ; `crates/kesh-api/src/routes/paypal_downloads.rs:101`
- `docs/manual/fr/admin-manual.tex:1145`, `:1183`

## Notes de version

- **Fixed** — Téléchargement PayPal : le libellé d'un compte s'apparie à sa série d'identifiants de l'environnement
  sans tenir compte des accents, comme le reste de Kesh (« PayPal Zürich » et « PayPal Zurich ») ; deux séries dont
  les libellés ne diffèrent que par les accents sont refusées au démarrage.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-22 : fiche rédigée (plafond 2, faute de plafond au kickoff).
