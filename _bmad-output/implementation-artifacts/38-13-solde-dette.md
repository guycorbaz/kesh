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

**AC-A3 — une seule règle, trois sites.** Les **trois** comparaisons de ce libellé côté serveur passent par une seule
fonction documentée (DRY), `pub(crate)` dans `provider_env.rs` : `series_for`, la garde de doublon de la lecture
(`:459-467`), et la **clé de tâche** de `Downloads::start` (`crates/kesh-api/src/provider_download.rs:614`, comparée
`:622`), qui refuse un second téléchargement du même compte (`409 PROVIDER_DOWNLOAD_RUNNING`) : sans elle, « PayPal
Zürich » et « paypal zurich » s'apparieraient à la même série avec deux clés de tâche, et deux téléchargements du même
compte tourneraient ensemble (validation P1, M1). Un test de `start` à accents près. `provider_api_status` (qui lit
`series_for`) suit sans changement. La règle est celle du frontend, `accountLabelKey`
(`frontend/src/lib/features/bank-accounts/provider.ts:44-47`, NFD, `\p{M}`, minuscules), plus le rognage.

**AC-A4 — documentation.** Disent « sans casse ni accents » pour ce libellé : le manuel d'administration FR
(`docs/manual/fr/admin-manual.tex:1145`, `:1183`), `docs/api-external.md:746`, `:1044`, `:1369`, et les doc-comments
`provider_env.rs:197`, `:735`, `provider_download.rs:602-605`. Le manuel utilisateur (`user-manual.tex:3407`) le dit
déjà ; `.env.example` n'a rien à changer ; pas de manuel DE, IT, EN à suivre. Pas de PDF.

**AC-B1 — clés orphelines retirées.** Les six clés `provider-gap-receipt-without-sale`,
`provider-gap-sale-without-receipt`, `provider-gap-amount-or-currency-mismatch`, `provider-review-conversion-unpaired`,
`provider-review-withdrawal-foreign-currency`, `provider-review-no-rule` sont retirées des quatre catalogues
(`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`). Les deux clés du même bloc que lisent les Epics 39 et 40
(`provider-review-ambiguous-sale`, `provider-review-fee-account-missing`, relevé par `git grep` sur `epic/39-wise` et
`epic/40-stripe`) restent, pour ne rien casser à la reprise d'`epic/38-paypal` par ces branches.

**AC-B2 — aucun lecteur.** Avant le retrait, `git grep` de chaque clé hors des `.ftl` (code Rust, frontend, docs,
manuels, tests) sur `epic/38-paypal`, `epic/39-wise` et `epic/40-stripe` ne rend rien ; les gardes de parité des
catalogues restent vertes.

**AC-B3 — consigne de fusion.** `epic/39-wise` et `epic/40-stripe` contiennent déjà `epic/38-paypal` et ajoutent des
clés jointives du bloc retiré : la reprise de cette story y donnera un conflit sur les quatre catalogues. Consigne,
écrite au Dev Agent Record et au texte de PR : **les six clés restent retirées** (les autres lignes des deux côtés
gardées), contrôle `git grep -c '<clé>' -- crates/kesh-i18n/locales` vide sur la branche fusionnée (validation P1, M2).
La consigne va aussi au rapport rendu à l'orchestrateur, qui la porte à `consignes-correctifs.md` (fichier commun que
cette story n'écrit pas), où la lira l'agent qui fusionnera (validation P2, L2).

**AC-C — trace.** Les deux entrées de `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md` sont barrées, avec le commit et
la mention « corrigé par la 38-13, ne pas ouvrir » ; rien d'autre n'y est retiré.

## Tasks / Subtasks

- [ ] A (AC-A1 à AC-A4)
  - [ ] fonction `pub(crate)` de clé de libellé dans `crates/kesh-api/src/provider_env.rs`, dans cet ordre : rognage,
        NFD, retrait des marques combinantes (`unicode_normalization::char::is_combining_mark`), minuscules — ordre écrit
        dans son doc-comment ; employée par les **trois** sites : `series_for`, la garde de doublon (`:459-467`),
        `Downloads::start` (`provider_download.rs:614`)
  - [ ] tests unitaires : `series_for` apparié dans les deux sens et en forme décomposée ; doublon refusé à accents
        près ; libellé réellement différent non apparié ; `start` rend `Err(RunningJob)` pour la variante accentuée
        d'un libellé dont une tâche tourne
  - [ ] mutations hors dépôt, une par appel de `heavy.sh` (copie `git archive` à sa propre cible
        `kesh-targets/38-13-mut`, copie non mutée jouée verte d'abord, « tuée » seulement par `panicked at` et le nom
        du test) : (1) la fonction ramenée à `trim().to_lowercase()` ; (2) retrait des marques supprimé ; (3) `start`
        sur sa propre `to_lowercase()` ; (4) la garde de doublon sur sa propre `to_lowercase()`
  - [ ] documentation de l'AC-A4 : manuel FR `.tex` (`:1145`, `:1183`), `docs/api-external.md` (`:746`, `:1044`,
        `:1369`), doc-comments `provider_env.rs:197`, `:735`, `provider_download.rs:602-605` ; grep du symptôme
        (« sans casse » sur ce libellé) dans le dépôt
- [ ] B (AC-B1, AC-B2)
  - [ ] grep de chaque clé sur les trois branches ; retrait des six clés des quatre catalogues ; consigne AC-B3
- [ ] C : barrer les deux entrées ; gates ; prompt de revue P1

## Dev Notes

- **Prémisse A, relue au code** : `series_for` (`provider_env.rs:199-204`), la garde de doublon (`:459-467`) et la clé
  de tâche (`provider_download.rs:614`) comparent par `to_lowercase()` ; `bank_accounts` est en `utf8mb4_unicode_ci` (migration de création, l. 17), et les
  recherches par libellé (`bank_accounts.rs:1649`, `:1694`) comparent `provider_account = ?` sous cette collation, donc
  sans casse ni accents. Conséquence : un compte « PayPal Zürich » et une série « PayPal Zurich » ne s'apparient pas à
  l'écran (`providerApi.configured` faux) ni à la route `POST /paypal-downloads` (`paypal_downloads.rs:101`).
- **Limite de l'approximation** : NFD + retrait des marques couvre les lettres accentuées ; la collation égale aussi
  `ß` et `ss` et quelques ligatures, que la fonction ne replie pas. À écrire dans son doc-comment, sans plus.
- **Faux DRY à écarter** : `strip_diacritics_char` (`crates/kesh-api/src/util.rs:96-108`) ne replie qu'une petite table
  de minuscules précomposées ; ne pas la réutiliser. `is_combining_mark` couvre `General_Category=Mark`, comme le
  `\p{M}` du frontend.
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
  les libellés ne diffèrent que par les accents sont refusées au démarrage ; un téléchargement déjà en cours pour un
  compte est reconnu sous ce libellé écrit sans ses accents.
- **Changed** — Démarrage : deux séries `KESH_PAYPAL_<n>_ACCOUNT` dont les libellés ne diffèrent que par les accents
  sont désormais refusées (« libellé déjà porté par une autre série »).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-22 : fiche rédigée (plafond 2, faute de plafond au kickoff).
- 2026-11-22, validation P1 (Sonnet, contexte séparé ; prompt et rapport `kesh-gate-logs/s38-13/validation-p1-*.md`) :
  0 C, 0 H, 2 M, 5 L. M1 (troisième site, la clé de tâche, `provider_download.rs:614`, relu) → AC-A3 ; M2 (conflit de
  fusion qui ressusciterait les clés) → AC-B3 ; L1 → AC-A4 ; L2 → AC-A3 ; L3 (ligne 467) ; L4 → tâche de mutation ;
  L5 → ligne « Changed ». Passe P2 à lancer.
- 2026-11-22, validation P2 (Opus, contexte séparé ; `kesh-gate-logs/s38-13/validation-p2-*.md`) : 0 C, 0 H, 1 M, 4 L.
  M1 (Tasks restées d'avant la P1 : fonction privée, pas de test de `start`, une seule mutation) → Tasks récrites,
  quatre mutations nommées ; L1 (`Downloads`, `:602-605`) ; L2 → AC-B3 (rapport à l'orchestrateur) ; L3, L4 → Dev
  Notes et Tasks. Passe P3 à lancer (dernière permise).
