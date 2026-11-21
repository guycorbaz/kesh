# Correctif : cours de l'AFC des monnaies retirées (lev bulgare, kuna…) écartés comme invalides

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/cours-afc-bgn`, partie de `main` à `70be12a6` (squash de l'Epic 37,
PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` et va à la prochaine release planifiée.
**Aucune migration.** Relevé par le responsable de l'epic à la recette d'un exercice réel 2024
(`data-not-committed/2024-qesh/controle/defauts-kesh.md`, point 4, « BGN », P4).*

## Intention

À la récupération des cours de l'AFC (Story 37-2), chaque mois de 2024 rendait `rejected: 1` et l'état
`invalid_rates:1` — l'écran *Cours de change* affichait « réussie (1 cours écartés comme invalides) » — et le journal
du serveur disait `ecartees=["bgn:code_invalid"]` (les cours journaliers du 31.12.2023 et du 31.12.2024 :
`["bgn:code_invalid", "hrk:code_invalid"]`, `run-8888/kesh.log` de la recette).

**Prémisse vérifiée au code.** `lire_code` (`crates/kesh-core/src/exchange_rates/parse.rs`) validait le code d'une
devise par `CurrencyCode::parse`, c'est-à-dire contre `ISO_4217_CURRENCIES` (`crates/kesh-core/src/currency.rs`), la
liste des monnaies **actives** de la Story 37-1, dont la doc exclut expressément « les codes retirés » (et dont le test
exclut `HRK`). `BGN` n'y figure pas : le lev a été remplacé par l'euro le 01.01.2026, et la liste, écrite en novembre
2026, ne le porte plus. L'AFC, elle, publie le lev pour toutes les périodes où il avait cours. Le cours publié était
donc écarté comme illisible et perdu. La même liste servait aussi aux routes des cours (`exiger_devise_connue`,
`routes/exchange_rates.rs`) : `GET /exchange-rates?currency=BGN` rendait `400 CURRENCY_UNKNOWN`.

**Relevé des codes publiés.** Aucune fixture enregistrée ne porte de vrais flux (les fixtures de la 37-2 sont
synthétiques, C-37-1 : aucun cours officiel au dépôt). Les flux ont donc été relevés une fois, **hors des tests**, le
2026-11-18 (`kesh-gate-logs/fix-cours-afc-bgn/releve-afc/`, 239 réponses ; cours mensuels de janvier, mars, juin,
septembre et décembre de 2000 à 2026, cours journaliers de chaque fin de trimestre de 2000 à 2025, plus quelques
dates). 93 codes distincts publiés ; **21 absents de la liste active** (`releve-plages.txt`, première et dernière
publication relevées) :

| Codes | Monnaie remplacée par | Publiés (relevé) |
|---|---|---|
| ATS BEF DEM ESP FIM FRF IEP ITL NLG PTE GRD | l'euro (1999 ; GRD 2001) | 2000 → fin 2001 |
| SIT | l'euro (2007) | 2002 → fin 2006 |
| CYP MTL | l'euro (2008) | 2007 → janvier 2008 |
| SKK | l'euro (2009) | 2002 → janvier 2009 |
| EEK | l'euro (2011) | 2002 → fin 2010 |
| LVL | l'euro (2014) | 2007 → janvier 2014 |
| LTL | l'euro (2015) | 2007 → janvier 2015 |
| HRK | l'euro (2023) | 2002 → septembre 2025 (cours journaliers ; aucun cours mensuel relevé) |
| BGN | l'euro (2026) | 2010 → mars 2026 (cours mensuels) |
| VEF | le bolívar souverain (VES, 2018) | 2008 → fin 2018 |

Les flux de 2026 après mars (mensuels 2026-10, 2026-12 ; journalier du 13.11.2026) ne portent plus aucun code inconnu.

## Décision

1. **Une seconde liste, close et relevée** : `WITHDRAWN_CURRENCIES` (`kesh-core`), les 21 codes ci-dessus, triée,
   disjointe de la liste active (test). Un code que l'AFC publierait demain et qu'aucune liste ne porte reste écarté
   (`code_invalid` au journal) : c'est le signal pour l'ajouter.
2. **Admise pour un cours de change, jamais pour un compte.** `CurrencyCode::parse_rate_currency` (même
   normalisation que `parse`) et `is_rate_currency` (code lu tel quel) admettent les monnaies actives et retirées ;
   `CurrencyCode::parse` et `normalize_foreign_currency`, qui valident la devise d'un compte, sont inchangés.
   *Pourquoi ne pas ouvrir de compte en BGN* : la demande l'exclut sauf choix motivé, et ouvrir un compte dans une
   monnaie retirée change l'écran du plan comptable (liste proposée, avertissement), le manuel et les règles de
   réévaluation — une fonctionnalité, non un correctif. Le besoin réel (« une société qui a tenu des BGN en 2024–2025
   ne peut pas reprendre ses livres ») reste ouvert : **à tracer en issue** (ci-dessous). Les cours, eux, sont
   désormais là quand ce besoin sera traité.
3. **Pas de contrôle de date.** L'AFC publie une monnaie retirée encore des mois après son remplacement (HRK jusqu'en
   septembre 2025, BGN jusqu'en mars 2026, VEF jusqu'à fin 2018). Borner l'admission à la « période en vigueur »
   aurait exigé une date de fin par code — ambiguë (l'euro de 1999 et les monnaies nationales jusqu'à fin 2001, la
   double circulation du lev en janvier 2026, la parité fixe de la kuna) — et aurait laissé l'écart à chaque cours
   journalier de 2023 à 2025 (HRK). Un cours publié d'une monnaie retirée n'est **copié** pour une société que si un
   de ses comptes porte cette devise (`copy_fetched_in_tx`), ce que le point 2 rend impossible : l'admettre sans
   borne de date ne fausse aucun cours retenu. L'AFC fait foi sur ce qu'elle publie.
4. **Les routes des cours suivent** : `exiger_devise_connue` (`GET /exchange-rates`, `GET /exchange-rates/lookup`,
   `PUT /exchange-rates/rates`, `POST /exchange-rates/rates/reset`, `POST /exchange-rates/notices/acknowledge`)
   admet toute devise de cours. Ainsi, tout ce que `POST /fetch` écrit se lit et se filtre, et la ligne BGN que montre
   « Toutes les devises publiées » se saisit comme les autres (sinon un `400 CURRENCY_UNKNOWN` à la saisie d'une case
   affichée). L'écran ne propose pas les monnaies retirées à l'ajout (`FOREIGN_CURRENCIES`, inchangée).

## Critères d'acceptation

1. **AC-1** — Un cours publié par l'AFC dans une monnaie retirée de la liste (BGN, HRK, monnaies d'avant l'euro, VEF)
   est retenu sous son code, mensuel comme journalier ; `rejected` ne le compte plus, l'état ne dit plus
   `invalid_rates`.
2. **AC-2** — Aucun des 93 codes relevés n'est écarté (test sur la liste des codes, sans aucun cours officiel).
3. **AC-3** — Un code retiré que l'AFC ne publie pas (CUC, SLL, ZWL…), un métal ou une unité de compte restent
   écartés (`code_invalid`) ; la règle « CHF écarté » est inchangée.
4. **AC-4** — Un compte ne s'ouvre ni ne se modifie dans une monnaie retirée (`400 CURRENCY_UNKNOWN`).
5. **AC-5** — `GET /exchange-rates?currency=BGN` rend la valeur publiée ; `PUT /exchange-rates/rates` en BGN est
   admis.
6. **AC-6** — Aucune migration, aucun texte i18n changé ; manuel utilisateur (`.tex`) et `docs/api-external.md` à jour.
7. **AC-7** — Captures avant/après de l'écran *Cours de change* (`kesh-gate-logs/fix-cours-afc-bgn/captures/`).

## Hors périmètre (à tracer en issues par l'orchestrateur)

- **Comptes dans une monnaie retirée**, pour reprendre des livres tenus en BGN (ou HRK…) avant le passage à l'euro
  (enhancement) : choix de la monnaie à l'écran (liste distincte, avertissement), règles de réévaluation et de clôture
  pour un exercice postérieur au retrait.
- **Une monnaie que l'AFC ajouterait** à ses publications et qu'aucune liste ne porte reste écartée sans autre signal
  que le journal du serveur et « n cours écartés comme invalides » : un relevé périodique, ou une alerte qui nomme le
  code, serait à décider.

## Tâches

- [x] `WITHDRAWN_CURRENCIES`, `is_rate_currency`, `CurrencyCode::parse_rate_currency` (normalisation partagée,
  `parse_against`) ; doc du module et du type.
- [x] `lire_code` du parseur AFC sur `parse_rate_currency` ; doc de `RejectReason::CodeInvalid`.
- [x] `exiger_devise_connue` des routes des cours sur `is_rate_currency`.
- [x] Tests unitaires (`currency.rs`, `parse/tests.rs`) et d'intégration (`exchange_rates_fetch_e2e.rs`,
  `account_currency_e2e.rs`) ; mutations.
- [x] Manuel utilisateur (`.tex`), `docs/api-external.md`, JSDoc de `iso4217.ts`.
- [x] Captures avant/après.

## Change Log

- 2026-11-18 — Développé, base `70be12a6` : `f4374778` (code), `102758bf` (tests, dernier commit de code).
  Reproduction avant correctif sur le binaire de `70be12a6` (`captures-avant.log`) : `rejected: 1`, état
  `invalid_rates:1`, `GET /exchange-rates?currency=BGN` → `400 CURRENCY_UNKNOWN`, ligne BGN absente de « Toutes les
  devises publiées ». Tests ajoutés, de `70be12a6` à `102758bf` : `currency.rs` 6 → 8 `#[test]`, `parse/tests.rs`
  26 → 29, `exchange_rates_fetch_e2e.rs` 25 → 26 `#[sqlx::test]`, plus deux codes (`BGN`, `HRK`) à la boucle des refus
  de `account_currency_e2e.rs` et deux doctests (`is_rate_currency`, `parse_rate_currency`). Mutations
  (`mutations.log`, `mutations-core.log`) : **M1** `lire_code` revenu à `CurrencyCode::parse` → rouges
  `aucun_code_publie_par_l_afc_n_est_ecarte`, `lev_bulgare_retenu_peso_convertible_ecarte`,
  `kuna_et_lev_retenus_au_cours_journalier`, `fetch_retient_les_monnaies_retirees_publiees` ; **M2** `parse` (devise
  d'un compte) sur les deux listes → rouges `liste_triee_sans_doublon_ni_code_exclu`,
  `monnaie_retiree_admise_pour_un_cours_jamais_pour_un_compte`,
  `chf_est_normalise_et_une_devise_refusee_en_mono_devise` ; **M3** `exiger_devise_connue` revenu à la seule liste
  active → rouge `fetch_retient_les_monnaies_retirees_publiees`. Gate ciblé (`cible-nextest-2.log`) : kesh-core et
  les trois binaires des cours et des comptes, 478/478 ; clippy du workspace vert (`cible-clippy.log`).
- 2026-11-18 — Gates au dernier commit de code `102758bf` (journaux `kesh-gate-logs/fix-cours-afc-bgn/`) :
  `scripts/test-fast.sh --ci` (base remise à zéro ; `KESH_NO_MEMGUARD=1`, la session n'ayant pas de bus systemd
  utilisateur — premier essai en échec sur `mem-guard`, avant tout test) : **5953 passés, 13 ignorés**, un flaky
  rejoué vert hors sujet (`admin_auto_backup_e2e disque_sature_pendant_l_ecriture`) (`gate-backend.log`) ; frontend
  `check`, `lint-i18n-ownership`, `test:unit` **4353 tests (271 fichiers)**, `build` verts ; E2E complet
  (`LANG=fr_CH.UTF-8`) **455 passés, 10 échecs, 18 ignorés**, les dix attendus : sept KF-029 (#97),
  `backup-restore-new-install.spec.ts:262`, `:381`, `opening-balances.spec.ts:156` (`e2e.log`) ; captures après
  (`captures-apres.log`) : `rejected: 0`, état sans erreur, `GET ?currency=BGN` → 200, ligne BGN au tableau, compte
  BGN toujours `400 CURRENCY_UNKNOWN`. `scripts/gate-ci.sh` (copie `git archive` de `6ae0116b`, dont le code est celui de `102758bf`, `KESH_NO_MEMGUARD=1`) :
  **5972 passés, 0 échoué, 17 ignorés** (`gate-ci.log`), cible de compilation supprimée ensuite.
- 2026-11-18 — revue P1 (Sonnet, lecture seule) : **0 au-dessus de LOW**, 2 LOW. P1-1 (« BGN publié jusqu'en 2025 » au lieu de mars 2026, `docs/api-external.md`, manuel, notes de version) corrigé. P1-2 (AC-4 : aucun test d'un `PUT /accounts/{id}` vers BGN ou HRK, seul le `POST` l'est ; même `normalize_foreign_currency`, risque nul au code) : test à ajouter dès que la station plafonne de nouveau ses gates (mem-guard indisponible depuis la saturation mémoire du 18.11), passe ciblée ensuite.
- 2026-11-18 — P1-2 corrigé, `49e75a7e` (test seul, aucune ligne de production) :
  `monnaie_retiree_refusee_a_la_modification_d_un_compte` (`account_currency_e2e.rs`, 8 → 9 `#[sqlx::test]` de
  `2ae7749e` à `49e75a7e`) — `PUT /accounts/{id}` vers `BGN`, `HRK`, `bgn`, sur un compte en CHF et un compte en USD
  sans écriture, en multi-devises : `400 CURRENCY_UNKNOWN`, `details.value` tel que reçu, devise et version inchangées
  en base ; sonde positive vers EUR (200). Mutation **M4** (`normalize_foreign_currency` sur `parse_rate_currency`) →
  rouges `monnaie_retiree_refusee_a_la_modification_d_un_compte` et
  `chf_est_normalise_et_une_devise_refusee_en_mono_devise` (`p2-cible.log`). **Gate ciblé seulement**, par décision du
  responsable de l'epic (`heavy.sh`, sans `KESH_NO_MEMGUARD`) : `cargo fmt --check` et `clippy --workspace --all-targets -D warnings`
  verts ; nextest de kesh-core, `account_currency_e2e`, `exchange_rates_fetch_e2e`, `exchange_rates_api_e2e` (base
  remise à zéro) **479/479** (`p2-nextest.log`). Le gate complet et l'E2E complet de ce commit se feront **groupés, sur
  une branche d'intégration** ; ceux de `102758bf` (ci-dessus) ne couvrent pas `49e75a7e`. Passe ciblée P2 à lancer
  (`kesh-gate-logs/fix-cours-afc-bgn/revue-p2-ciblee-prompt.md`).
- 2026-11-18 — passe P2 ciblée (Haiku, lecture seule) sur `49e75a7e` : 0 CRITICAL/HIGH, 1 MEDIUM, 1 LOW. **P2-1 (MEDIUM, procédure)** : `CLAUDE.md` exige le gate complet et l'E2E complet au dernier commit de code ; seul le gate ciblé a tourné à `49e75a7e`. Décision du responsable de l'epic (2026-11-18) : pour les correctifs courts, gate ciblé par correctif, puis **gate complet groupé** (test-fast `--ci`, E2E complet, `gate-ci.sh`) sur une branche d'intégration réunissant plusieurs correctifs, **avant tout push** ; la boucle de ce correctif se clôt sur ce gate groupé, rapporté ici. **P2-2 (LOW)** : le gate ciblé (`p2-cible.log`) a tourné sur l'arbre de travail de `2ae7749e` augmenté du test, avant le commit `49e75a7e` ; contenu identique au commit (arbre propre à HEAD, 9 tests dans le binaire).
- 2026-11-18 — **Reprise de `main` après la v0.19.0, puis gate complet groupé.** `origin/main` (`773df80d`, tag
  `v0.19.0`) repris par fusion (`594b8f17`), sans rebase ; conflits résolus en gardant les deux intentions (`sprint-status.yaml` : les deux blocs ; `docs/api-external.md` : la mention « Depuis la v0.19.0 » de `main` gardée, la ligne `CURRENCY_UNKNOWN` du correctif récrite dessus).
  Gate ciblé après la reprise : `fmt`, `clippy` du workspace, nextest de `kesh-core`, de `account_currency_e2e`, `exchange_rates_fetch_e2e` et de la bibliothèque `kesh-api` **1184 passés, 5 ignorés**, Vitest `src/lib/shared/utils` **217 tests (20 fichiers)**, garde i18n verte (`kesh-gate-logs/fix-cours-afc-bgn/reprise-v0190-*.log`). Puis `origin/main` `37c2f875` (documentation seule : `CLAUDE.md`, bilan de la
  0.19.0, `sprint-status.yaml`) repris de même. **Gate complet groupé** sur la branche jetable `integ/fixes-0.19.0`
  (`773df80d` + les quatre correctifs fusionnés dans l'ordre PDF, monnaies retirées, résultat reporté, parent du
  compte ; seul conflit, `sprint-status.yaml`), **au commit `5fc6fb1b`**, base `kesh_integ` remise à zéro :
  `scripts/test-fast.sh --ci` **6165 passés, 14 ignorés** ; frontend `check` 0 erreur (29 avertissements
  préexistants), `lint-i18n-ownership`, `test:unit` **4627 tests (278 fichiers)**, `build` ; **E2E complet**
  (`LANG=fr_CH.UTF-8`, port 3382) **478 passés, 10 échecs, 18 ignorés** — les dix attendus, jugés un à un : sept KF-029
  (#97 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`), `backup-restore-new-install.spec.ts:262`, `:381`, `opening-balances.spec.ts:156` (`docs/testing.md`
  inchangé par la 0.19.0) ; `scripts/gate-ci.sh` (copie `git archive`) **6184 passés, 0 échoué, 18 ignorés**.
  Journaux : `kesh-gate-logs/integ-fixes-0.19.0/` (`gate-backend.log`, `front-*.log`, `e2e.log`, `gate-ci.log`) ; un
  premier essai, arrêté par le disque plein de la station, est gardé dans `avorte-disque-plein/`. Le commit de
  documentation `37c2f875` n'a pas été rejoué au gate (aucun code).

## Notes de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Fixed** — Cours de change : les cours que l'AFC publie dans une monnaie aujourd'hui retirée — le lev bulgare (BGN)
  jusqu'en mars 2026, la kuna croate (HRK), les monnaies remplacées par l'euro, le bolívar fort (VEF) — sont récupérés
  comme les autres ; ils étaient écartés comme invalides (« 1 cours écartés comme invalides » à chaque mois de 2024).
  Ils se lisent et se saisissent à l'écran *Cours de change* (« Toutes les devises publiées ») et par l'API ; un
  compte ne s'ouvre toujours que dans une monnaie en cours.
