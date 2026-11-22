# Story 38-13 : solde de la dette de l'Epic 38

Status: review

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

- [x] A (AC-A1 à AC-A4)
  - [x] fonction `pub(crate)` de clé de libellé dans `crates/kesh-api/src/provider_env.rs`, dans cet ordre : rognage,
        NFD, retrait des marques combinantes (`unicode_normalization::char::is_combining_mark`), minuscules — ordre écrit
        dans son doc-comment ; employée par les **trois** sites : `series_for`, la garde de doublon (`:459-467`),
        `Downloads::start` (`provider_download.rs:614`)
  - [x] tests unitaires : `series_for` apparié dans les deux sens et en forme décomposée ; doublon refusé à accents
        près ; libellé réellement différent non apparié ; `start` rend `Err(RunningJob)` pour la variante accentuée
        d'un libellé dont une tâche tourne
  - [x] mutations hors dépôt, une par appel de `heavy.sh` (copie `git archive` à sa propre cible
        `kesh-targets/38-13-mut`, copie non mutée jouée verte d'abord, « tuée » seulement par `panicked at` et le nom
        du test) : (1) la fonction ramenée à `trim().to_lowercase()` ; (2) retrait des marques supprimé ; (3) `start`
        sur sa propre `to_lowercase()` ; (4) la garde de doublon sur sa propre `to_lowercase()`
  - [x] documentation de l'AC-A4 : manuel FR `.tex` (`:1145`, `:1183`), `docs/api-external.md` (`:746`, `:1044`,
        `:1369`), doc-comments `provider_env.rs:197`, `:735`, `provider_download.rs:602-605` ; grep du symptôme
        (« sans casse » sur ce libellé) dans le dépôt
- [x] B (AC-B1, AC-B2)
  - [x] grep de chaque clé sur les trois branches ; retrait des six clés des quatre catalogues ; consigne AC-B3
- [x] C : barrer les deux entrées ; gates ; prompt de revue P1

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

Opus (développement) ; validation P1 Sonnet, P2 Opus, P3 ciblée Sonnet.

### Completion Notes List

- **Plafond** : 2, faute de plafond fixé au kickoff (fichier d'epic, l. 110). Deux points corrigés, aucun autre.
- **B** (`3c97735cc`) : les six clés retirées des quatre catalogues (24 lignes) ; `git grep` de chacune ne rend plus
  que cette fiche. `provider-review-ambiguous-sale` et `provider-review-fee-account-missing` gardées (lues sur
  `epic/39-wise` et `epic/40-stripe`).
- **⚠️ Consigne de fusion (AC-B3)** : à la reprise d'`epic/38-paypal` par `epic/40-stripe` puis `epic/39-wise`, le
  conflit des quatre `.ftl` se résout en gardant les lignes des deux côtés **sauf les six clés, qui restent retirées** ;
  contrôle : `git grep -c -E 'provider-gap-|provider-review-(conversion-unpaired|withdrawal-foreign-currency|no-rule)' -- crates/kesh-i18n/locales`
  ne rend rien.
- **A** (`0d6218c26`, mise en forme `98f1fb40d`) : `provider_env::account_label_key` (`pub(crate)`, rognage, NFD,
  retrait des marques combinantes, minuscules), employée par `series_for`, la garde de libellé en double de `read`
  et la clé de tâche de `Downloads::start`. Doc-comments, manuel d'administration FR (`.tex`, deux sites),
  `docs/api-external.md` (trois sites) et le doc-comment `routes/bank_accounts.rs:751` disent « sans casse ni
  accents ». Le grep de « sans casse » sur ces fichiers ne rend plus que des comparaisons d'autres valeurs.
- **Tests** (périmètre `epic/38-paypal` → `HEAD`) : `#[test]` de `provider_env.rs` 9 → 11
  (`le_libelle_s_apparie_sans_accents`, `libelles_egaux_aux_accents_pres_refuses`) ; `provider_download.rs` 15 → 15,
  `une_tache_par_compte_tant_qu_elle_tourne` étendu d'un refus à accents près (`BÕUTIQUE`).
- **Mutations** (`kesh-gate-logs/s38-13/mutation.sh`, `mutations.console`, `mutation-*.log` ; copie `git archive`,
  cible propre supprimée, une par appel de `heavy.sh`) : copie non mutée verte (32 tests `provider_`) ; (1) fonction
  ramenée à `trim().to_lowercase()` → 3 tests rouges ; (2) marques non retirées → 3 rouges ; (3) `start` sur
  `to_lowercase()` → `une_tache_par_compte_tant_qu_elle_tourne` (`provider_download.rs:1205`) ; (4) garde de doublon sur
  `to_lowercase()` → `libelles_egaux_aux_accents_pres_refuses` (`provider_env.rs:562`). Les quatre tuées par
  `panicked at` d'un test nommé.
- **Gates au dernier commit de code `98f1fb40d`** : backend `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`,
  nextest) vert, **6423 exécutés, 6423 passés, 15 ignorés** (`gate-backend-3.log`, base `kesh_s3813` remise à zéro
  sur dev4, `KESH_NO_MEMGUARD=1` : bus systemd utilisateur refusé) ; frontend `check`, `lint-i18n-ownership`, Vitest
  **4995 tests (290 fichiers)**, `build` verts (`front-1-*.log`). Gate 1 rouge sur mem-guard (bus refusé), gate 2
  rouge sur `cargo fmt` seul (test reformaté, `98f1fb40d`). **Aucun E2E** : aucune spec ajoutée (règle de la nuit du
  22.11), aucun écran changé ; E2E complet et `gate-ci.sh` à la fin de l'epic.
- `issues-a-ouvrir.md` : les deux entrées barrées, avec leur commit (copie d'avant :
  `kesh-gate-logs/s38-13/issues-a-ouvrir.avant-38-13.md`).
- Hors périmètre, relevé : le doc-comment `routes/bank_accounts.rs:751` disait la collation « sans casse » (corrigé au
  passage, commentaire seul).

### File List

- `crates/kesh-api/src/provider_env.rs`
- `crates/kesh-api/src/provider_download.rs`
- `crates/kesh-api/src/routes/bank_accounts.rs` (doc-comment)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `docs/manual/fr/admin-manual.tex`
- `docs/api-external.md`
- `_bmad-output/implementation-artifacts/38-13-solde-dette.md`, `sprint-status.yaml`

## Change Log

- 2026-11-22 : fiche rédigée (plafond 2, faute de plafond au kickoff).
- 2026-11-22, validation P1 (Sonnet, contexte séparé ; prompt et rapport `kesh-gate-logs/s38-13/validation-p1-*.md`) :
  0 C, 0 H, 2 M, 5 L. M1 (troisième site, la clé de tâche, `provider_download.rs:614`, relu) → AC-A3 ; M2 (conflit de
  fusion qui ressusciterait les clés) → AC-B3 ; L1 → AC-A4 ; L2 → AC-A3 ; L3 (ligne 467) ; L4 → tâche de mutation ;
  L5 → ligne « Changed ». Passe P2 à lancer.
- 2026-11-22, validation P2 (Opus, contexte séparé ; `kesh-gate-logs/s38-13/validation-p2-*.md`) : 0 C, 0 H, 1 M, 4 L.
  M1 (Tasks restées d'avant la P1 : fonction privée, pas de test de `start`, une seule mutation) → Tasks récrites,
  quatre mutations nommées ; L1 (`Downloads`, `:602-605`) ; L2 → AC-B3 (rapport à l'orchestrateur) ; L3, L4 → Dev
  Notes et Tasks. Passe P3 à lancer (troisième et dernière de la consigne de cette story ; le plafond de `CLAUDE.md`
  reste de huit — annoté sur le LOW L2 de la P3).
- 2026-11-22, validation P3 ciblée sur `a40d126ce` (Sonnet ; `kesh-gate-logs/s38-13/validation-p3-*.md`) : 0 au-dessus
  de LOW, 2 LOW. L1 (la mutation (2) ne meurt que par une paire accentuée / non accentuée) : les tests prévus portent
  « Zürich » / « zurich » et « Gênf » / « Genf » ; L2 annoté ci-dessus. **Validation close** : trend P1 2 M → P2 1 M →
  P3 0, modèles Sonnet, Opus, Sonnet.
- 2026-11-22, développement : B `3c97735cc`, A `0d6218c26` + `98f1fb40d` ; gates verts à `98f1fb40d` (backend 6423,
  frontend 4995), quatre mutations tuées ; story en `review`, prompt de revue P1
  `kesh-gate-logs/s38-13/revue-p1-prompt.md`.
