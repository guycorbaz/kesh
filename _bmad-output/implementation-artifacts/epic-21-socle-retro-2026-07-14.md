# Rétrospective de jalon — Epic 21, socle backend (21-1 / 21-2 / 21-3)

**Date** : 2026-07-14
**Portée** : rétrospective de **jalon** couvrant le **socle backend** de l'Epic 21 « Échéances & relances débiteurs ». **L'Epic 21 n'est PAS clos** — restent 21-4 (frontend réglages + templates multi-niveau), 21-5a/21-5b (éligibilité + envoi), 21-6 (UI relances), 21-7 (balance âgée), 21-8 (doc + E2E). Cette rétro fait le point sur la phase backend avant d'entamer la phase visible-utilisateur.
**Branche** : `story/21-1-echeances-relances` (poussée, non mergée — clôture au bout de l'epic).

## 1. Objectif & résultat

Poser les fondations backend du cycle de suivi des débiteurs : échéances fiables, montants dus corrects, et configuration des rappels. **3 stories done** (21-2 splittée en 2) :

- **21-1 — Conditions de paiement structurées** (#245 fermé) : délai en jours sur le contact → échéance + libellé 4 langues pré-calculés, validation `due_date >= date`.
- **21-2a / 21-2b — Montant TTC canonique** (#246 fermé) : helper `invoice_total_ttc` + équivalent SQL, appliqué à QR-bill / PDF / `{amount}` e-mail / échéancier / **rapprochement bancaire** (le matching TTC était cassé dès TVA > 0).
- **21-3 — Socle config rappels** (#231 partiel) : tables `dunning_levels` + `company_dunning_settings`, seed lazy, type e-mail `invoice_reminder` + templates par niveau (cascade), 16 défauts FR/DE/IT/EN.

**Gate final** : 98 suites / 1824 tests / 0 échec, clippy 0, fmt 0. **Bug de fond #249** (« Marquer payée » cassé, 422) corrigé au passage.

## 2. Découpage & exécution

| Story | Validate | Dev | Review | Résultat |
|---|---|---|---|---|
| 21-1 | 2 passes (Sonnet→Haiku) | 1 run Fable | 3 passes (Sonnet×3→Haiku→Opus) | done, #245 |
| 21-2 | **4 passes SANS convergence → SPLIT** | — | — | splittée |
| 21-2a | (figée par split) | Fable | 1 passe Sonnet×3 | done |
| 21-2b | (figée par split) | Fable | 1 passe (panel Sonnet/Haiku/Opus) | done, #246 |
| 21-3 | 4 passes (Sonnet→Haiku→Opus→Sonnet), convergé pile à P4 | T1→T9 Opus | 2 passes (panel P1 → Sonnet P2) | done |

## 3. Ce qui a bien marché (Insights)

- **I1 — La règle de splitting préventif a prouvé sa valeur (critère 2).** 21-2 a divergé en validate (4 passes, trend >LOW 4→5→3→3) → split en 21-2a (primitives+surfaces) / 21-2b (réconciliation). Les deux sous-stories ont convergé en **1 passe** de review chacune. 2e application réelle après le précédent Story 7-1 documenté.
- **I2 — Le grep ground-truth fonctionne dans les DEUX sens sur Haiku.** Il a réfuté 0 hallucination (Haiku propre cette fois) ET **confirmé de vrais bugs Haiku** : `level_number` manquant sur `EffectiveEmailTemplate` (raté par Sonnet en Pass 1 de validate). La discipline grep reste le filet.
- **I3 — Panel parallèle de reviewers + rotation LLM séquentielle.** Lancer 3 reviewers en parallèle sur la 1re passe (lentilles distinctes) accélère sans sacrifier la rotation : les passes de remédiation suivantes utilisent un LLM différent, contexte frais.
- **I4 — Cartographie par agents Explore parallèles AVANT la spec.** Pour 21-3, 4 agents Explore ont cartographié en parallèle les patrons (vat_rates / company_invoice_settings / email_templates / conventions migration-export-backup). Résultat : une spec ultra-dense en `fichier:ligne` exacts → dev fluide, pièges anticipés.
- **I5 — Investigation « accélérer les tests » honnête.** Mesures empiriques (32t contre-productif, 6t stable, migration-bound) plutôt que promesses. Livré nextest (1,40×) + CR #251 pour le vrai levier (squash schéma de test).

## 4. Difficultés & apprentissages

- **L1 — Un bump `min_required` DOIT s'accompagner du bump de version Cargo. (À CODIFIER)** La migration breaking de 21-3 a bumpé `kesh_version_min_required='0.7.0'`, mais le binaire restait à Cargo 0.6.0 → `check_downgrade_protection` **refusait le boot ET l'import backup** (le binaire est « plus ancien » que sa propre DB). **Détecté seulement au gate runtime** — les 4 passes de validate statiques l'ont validé (« 0.7.0 correct ») sans l'exercer. Fix : workspace bumpé 0.6.0 → 0.7.0. Les deux bumps sont **les deux moitiés de la même action de version**.
- **L2 — La validate statique ne remplace jamais le gate runtime.** Les 2 findings les plus coûteux du socle (L1 version, H1 seed) n'étaient détectables qu'à l'exécution. Le gate workspace complet est le vrai filet ; ne jamais marquer « done » sans lui.
- **L3 — Bug H1 : un seed lazy ne doit poser que ce que le DEFAULT DB ne couvre pas.** `ensure_seeded_in_tx` réécrivait `grace_period_days=5` inconditionnellement → écrasait une grâce personnalisée par un `update()` antérieur (PUT avant 1er GET), sans audit. Hérité de la spec (AC 9), rattrapé par la review **correctness** (pas par validate — comportement runtime). Fix : le seed ne pose que `seeded_at` + version.
- **L4 — Pièges MariaDB/sqlx à connaître** : (a) `sqlx::migrate!()` ne détecte pas les nouveaux `.sql` → `touch` du fichier de la macro ; (b) erreur 1553 « cannot drop index needed by FK » → créer le nouvel UNIQUE (couvrant le préfixe FK) AVANT de dropper l'ancien.
- **L5 — Coût du dev en un seul enchaînement.** 21-3 (spec 4 agents + validate 4 passes + dev T1→T9 + review 2 passes) a été mené d'un trait sur une session très longue. Le découpage en commits par task (T1, T2-T7, T8, T9) a permis des checkpoints propres — indispensable pour un dev de cette taille.

## 5. Triage dette technique (politique zero-carry-forward)

- **Catégorie A (vraie dette)** : **aucune non-adressée**. #249 corrigé, H1 corrigé, tous les compteurs transverses à jour.
- **Catégorie B (limitations documentées + remédiation tracée GitHub)** :
  - **#250** — encaissement manuel : capturer mode/compte (caisse/virement/carte/TWINT) + passer l'écriture de règlement (aujourd'hui mark-paid ne pose que `paid_at`).
  - **#247 / #248** — UX sidebar (repli, regroupement pages admin en onglets).
  - **#251** — deep-fix lenteur tests (squash schéma de test + durabilité MariaDB).
  - **#252** — mise à jour dépendances (Rust + npm), planifiée **après clôture Epic 21**.
- **Catégorie C (décisions design intentionnelles)** : routes email-templates opèrent niveau 0 (segment niveau = 21-4) ; export souveraineté `dunning_levels` à 0 row (seed lazy, pas d'effet de bord métier).
- **KF-038 (#228)** — flake réconciliation sous contention parallèle : inchangé, tracé, contourné par le gate série.

## 6. Action items

- [x] **Codifier L1 dans CLAUDE.md** (Migration breaking policy) : « un bump `min_required` va toujours de pair avec le bump de version Cargo du workspace ; à vérifier au **runtime** (boot + import), pas seulement en validate statique. » — fait dans le même commit que cette rétro.
- [ ] **À la clôture de l'Epic 21** : ressortir #252 (deps) pour arbitrage (Epic dédié « Technical Debt / Dependencies » ou CR par lot).
- [ ] **Phase suivante = 21-4** (frontend réglages rappels + templates multi-niveau) : consomme `list_effective_for_company(max_reminder_level)` + les routes dunning. Prévoir le segment de niveau dans les routes email-templates (aujourd'hui niveau 0).
- [ ] **21-5b** (envoi) consommera `build_reminder_vars` (les variables `reminderLevel`/`reminderFee`/`totalDue`/`daysOverdue` sont déjà **déclarées** dans `allowed_variables`).

## 7. Métriques

- **Stories** : 4 done (21-1, 21-2a, 21-2b, 21-3 ; 21-2 splittée).
- **Issues fermées** : #245, #246, #249. **#231** partiel (socle).
- **CR/issues créés** : #247, #248, #250, #251, #252 (+ #246 créé puis fermé).
- **Commits** : ~34 au-dessus de `main` (branche epic-21).
- **Gate final** : 98 suites / 1824 tests / 0 échec ; clippy 0 ; fmt 0.
- **Passes adversariales** : 21-1 (2 validate + 3 review), 21-2 (4 validate → split), 21-2a/b (1 review chacune), 21-3 (4 validate + 2 review). Toutes convergées à 0 > LOW.
- **Version** : workspace **0.6.0 → 0.7.0** (forcé par le 1er bump `min_required` du repo).
- **Infra** : tooling nextest « gate rapide » (`.config/nextest.toml` + `scripts/test-fast.sh`, 1,40×).
