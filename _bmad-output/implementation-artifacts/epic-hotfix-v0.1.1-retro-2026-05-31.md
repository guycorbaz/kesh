# Rétrospective Epic Hotfix v0.1.1 — Onboarding fresh-install + admin recovery + logs fichier + port 80

**Date rétro** : 2026-05-31
**Statut Epic** : ✅ Done (4/4 stories effectives, v011-3 superseded par v011-5)
**Facilitator** : Claude Opus 4.7
**Participant** : Guy (Project Lead)
**Format** : Document factuel direct (party-mode roleplay skippé, cohérent préférence Epic 9.5 retro)

---

## 📊 Métriques Epic Hotfix v0.1.1

### Delivery

| Indicateur | Valeur |
|---|---|
| Stories planifiées | 4 (v011-1 → v011-4, post révision 2026-05-30) |
| Stories effectives | 5 — v011-3 SUPERSEDED, absorbé dans v011-5 (ajoutée post-release v0.1.1) |
| Stories `done` | 4/4 (100%) — v011-1, v011-2, v011-4, v011-5 |
| Stories `superseded` | 1 (v011-3 → v011-5 design unifié 2026-05-30) |
| Période exécution | 2026-05-28 → 2026-05-31 |
| Branches | 4 branches stories distinctes (v011-1, v011-2, v011-4, v011-5) + branches release-prep |
| **Release intermédiaire** | **v0.1.1 publiée 2026-05-29** (Docker Hub `gcorbaz/kesh:0.1.1` + `latest` + GitHub Release tag) — embarque v011-1 + v011-2. v011-4 + v011-5 ciblent **v0.1.2** (release pending tag). |

### PRs

| Story | PR | Squash commit | Merge date |
|---|---|---|---|
| v011-1 (logs fichier) | #123 | `f33cf5f` | 2026-05-28 |
| v011-2 (catch-22 onboarding) | #128 | `40bbfa3` | 2026-05-30 |
| v011-2 release-prep | #129 (CHANGELOG date + README) + #130 (Cargo bump 0.1.0→0.1.1 pour smoke /health) | — | 2026-05-29 |
| v011-3 (break-glass) | SUPERSEDED — pas de PR | N/A | N/A |
| v011-4 (port 80) | #132 | `4c9558b` | 2026-05-30 |
| v011-5 (onboarding self-service) | **#134 OPEN, CI verte** | (squash pending) | en cours |

### Cycle qualité review

| Story | Spec validate | Code-review | Patches | Notes |
|---|---|---|---|---|
| v011-1 logs fichier | 2 (Sonnet+Haiku) | 1 (Sonnet) | ~3 LOW | 0 > LOW Pass 2. CI verte. Story-zéro infrastructure (décision H9). |
| v011-2 catch-22 | 2 (Sonnet+Haiku) | 2 (Sonnet+Haiku) | 4 + 0 | Auditor AC 18/18 ×2 passes confirmé. Full workspace 1308 tests verts. KFs #124-127 créées pendant exec. |
| v011-4 port 80 | 2 spec + 2 code | ~10 | 4 passes total | Cycle bref — scope contenu config + Docker + doc + smoke release.yml. |
| v011-5 onboarding self-service | **4 passes spec converged** (Sonnet→Haiku→Opus→Sonnet, 40 patches) | 2 passes code (Sonnet+Haiku) | **40 + 11** | Cycle complet CLAUDE.md. Pass 3 Opus catch 3 HIGH architectural cross-fichiers + 8 patches. Pass 2 Haiku 2 CRITICAL spec + 1 HIGH code réfutés grep ground-truth. CYCLE CONVERGÉ. |
| **Total Epic** | ~10 passes validate | ~6 passes review | **~70+ patches cumulés** | Trend identique Epic 9.5 (75+ patches sur 7 stories). |

### Quality / NFR

- **Tests** : 0 régression sur baselines projet. v011-2 full workspace 1308 tests verts. v011-5 ajoute 199 lib + 6 setup_admin_e2e + 5 SetupForm Vitest = 210 nouveaux tests, 267 Vitest total.
- **CI** : verte sur les 4 PRs mergées + PR #134 verte (Backend 13m22s + Docker 4m52s + Frontend 1m24s).
- **Release v0.1.1** : Docker Hub publiée 2026-05-29. **3 RCs** avant tag final — instabilité release process (cf. C2 ci-dessous).
- **Manuels FR** : `admin-manual.tex` mis à jour par v011-4 (section port) + v011-5 (section premier démarrage + recovery 7 étapes + matrice 6 cas). PDFs régénérés via `latexmk -xelatex`.

---

## ✅ Follow-through Epic 9.5 retro (action items)

Les 5 action items critical path Epic 9.5 → Epic 10 ont été traités :

| # | Action Epic 9.5 retro | Statut | Évidence |
|---|---|---|---|
| 1 | GitHub Milestone v0.2 + labellisation systématique items catégorie B | ⚠️ **Partiellement** — label `v0.2-milestone` existe et utilisé sur Issues #98 (Epic 14) et #133 (TOCTOU L1 v011-5). Mais milestone GitHub `v0.2` formel **pas créé** (à formaliser pré-Epic 11). Items 9-2a L13-L15, 9-2b L1-L18 **non systématiquement labellisés**. |
| 2 | Audit cohérence planning artifacts | ✅ Done partiel — README roadmap v0.1.x → v0.1.2 mis à jour commit `95d6138`. Sprint-status.yaml maintenu cohérent. |
| 3 | Documentation utilisateur PME Kesh | ❌ **NON ADRESSÉ** — guide utilisateur PME consolidant OLICo Art. 9+10 + onboarding + self-hosting non créé. **À carry-forward Epic 11 prep ou Epic dédié documentation** (le manuel admin couvre l'ops, pas l'utilisateur final fiduciaire/PME). |
| 4 | Pre-flight Epic 10 Déploiement | ✅ Done — Epic 10 livré entièrement (5 stories done : 10-1 hardening + 10-2 migrations + 10-3 résilience + 10-4 manuel + 10-5 httpOnly tokens). |
| 5 | PR Epic 9.5 push final | ✅ Done — PR #99 mergée 2026-05-20. |

**Verdict follow-through** : 3/5 done, 1/5 partiel, **1/5 non-adressé (Action #3 doc utilisateur PME)**. Cette action est carry-forward vers Epic 11 prep — c'est une **dette catégorie A** (action retrospective non-complétée d'un Epic antérieur, cf. CLAUDE.md §"Tech debt management — zero carry-forward policy"). À adresser AVANT kickoff Epic 11.

---

## 🎯 Successes & Strengths

### S1. **Hotfix release v0.1.1 publié rapidement (3 jours)**

Bug bloquant #120 catch-22 onboarding découvert 2026-05-27 lors du 1er déploiement prod Guy → release v0.1.1 publiée Docker Hub 2026-05-29 (2 jours calendaires). Réactivité conforme à la sévérité « catégorie A bloquante ». Pattern « découverte hors fenêtre rétrospective → release hotfix immédiate » validé empiriquement.

### S2. **Cycle Sonnet→Haiku→Opus→Sonnet à nouveau validé sur v011-5**

Story v011-5 (la plus complexe de l'epic — 29 ACs, ~12-15 modules) a converge via le cycle complet 4 passes spec validate :
- Pass 1 Sonnet : 28 findings raw → 22 patches structurels.
- Pass 2 Haiku : 20 findings → 8 patches + **2 CRITICAL hallucinations réfutées grep ground-truth**.
- **Pass 3 Opus : 8 findings → 3 HIGH cross-fichiers** ratés par Sonnet+Haiku (NewAuditLogEntry schema erroné, 30 sites `from_fields_for_test`, 28 sites `AppState`). Sans Pass 3 Opus, compilation aurait échoué au dev-story T1.
- Pass 4 Sonnet : 2 findings → CONVERGÉ.

Pattern Epic 9.5 Insight I1 (« Opus catches subtle stuff ») reproduit avec valeur tangible (compile errors évités).

### S3. **Innovation pattern : double-usage env vars + no-op-on-same-hash**

v011-5 introduit un pattern original (Guy 2026-05-30) : `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` deviennent **variable double-usage** dont le comportement au boot dépend de l'état DB (matrice 6 cas). Le **no-op sur hash identique** élimine le piège classique « reset à chaque reboot tant que les vars traînent ». Pattern à mémoriser pour futurs flux de récupération offline / break-glass.

### S4. **Discipline Haiku grep ground-truth empiriquement validée v3**

3e validation consécutive (Epic 9.5, puis 2 fois Epic Hotfix v0.1.1) du retour d'expérience `feedback_haiku_review_diff_combined` :
- v011-5 spec validate Pass 2 Haiku : **2 CRITICAL réfutés grep** (méta-spec vs code source divergence).
- v011-5 code-review Pass 2 Haiku : **1 HIGH BH2-1 réfuté grep + Read code flow** (Haiku affirmait « 200 + cookies retournés si `refresh_tokens::create` fail » — faux, `await?` propage Err).
- Pattern systématique : tous CRITICAL/HIGH Haiku passés au grep ground-truth avant patch. 3 hallucinations cumulées dismissed cette epic.

### S5. **CHANGELOG « ⚠️ Action requise upgrade » : pattern utilisateur-friendly**

v011-5 introduit un pattern CHANGELOG section dédiée `### ⚠️ Action requise — upgrade v0.1.1 → v0.1.2` pour signaler aux utilisateurs existants une action manuelle à effectuer avant upgrade (retirer `KESH_ADMIN_PASSWORD` du `.env` s'ils ont changé leur mdp via UI). Wording explicite Q1 Option B (« doc agressive »). **À codifier comme pattern** pour toute release qui introduit un comportement potentiellement piégeux pour les utilisateurs existants.

### S6. **README sync pré-push (CLAUDE.md compliance)**

Le commit `95d6138` met à jour README roadmap (`v0.1.x` → `v0.1.2 (hotfix) 🚧`) **avant le push de PR #134**, conforme à la règle CLAUDE.md « Synchroniser TOUTES les docs avant tout push / création de release ». Pattern de plus en plus systématique.

### S7. **Zero régression sur baselines existantes**

Aucune des 4 stories n'a introduit de régression sur les tests pré-existants. Trend continu depuis Epic 8 — la discipline `cargo test --workspace -j1 -- --test-threads=1` + `npm run check` + Playwright avant chaque PR paie.

### S8. **Splitting préventif v011-2 → v011-5 (refactor mid-epic)**

Décision Guy 2026-05-30 de **superseder v011-3 break-glass** par un design unifié dans v011-5 (onboarding self-service + recovery via les mêmes vars `KESH_ADMIN_*`). Refactor mid-epic propre : `epic-hotfix-v0.1.1.md` planning doc met à jour la story v011-3 avec un disclaimer « SUPERSEDED » + section v011-5 ajoutée. Aucune confusion downstream — sprint-status reflète l'état correct.

---

## ⚠️ Challenges & Growth areas

### C1. **v011-5 dev-story : 2 patches imprévus en cours (heuristics qui ratent)**

Story v011-5 dev-story Opus 4.7 single-pass a découvert 2 bugs en cours d'implémentation que **les 4 passes spec validate n'avaient pas anticipés** :

1. **Cas 2 partial state** (company existe + user 0) : ma refactor cas 2 créait toujours une stub, même quand `company_count > 0`. Test `bootstrap_creates_admin_on_existing_company` échoué (a count=2 expected 1). Fix : branchement `created_stub_this_boot: bool` pour préserver le comportement v011-2 et éviter de créer un 2e stub.

2. **`refresh_tokens.revoked_reason = "admin_break_glass_reset"`** : la spec validate (4 passes) n'a pas catché que cette valeur viole la DB CHECK constraint `chk_refresh_tokens_revoked_reason` qui restreint à `{logout, rotation, password_change, admin_disable, theft_detected}`. Test échoué « 0 tokens revoked ». Fix : mapping `"password_change"` (le motif détaillé reste dans `audit_log.action`, VARCHAR libre).

**Lesson** : la spec validate (même 4 passes Opus inclus) **ne vérifie pas systématiquement** les contraintes DB CHECK whitelist sur les valeurs utilisées par le code spec. À ajouter comme item de check de spec validate : « pour chaque valeur littérale string assignée à une colonne DB, vérifier les CHECK constraints existantes ». Action item ci-dessous.

### C2. **Release v0.1.1 : 3 RCs avant tag final**

La release v0.1.1 publiée 2026-05-29 a nécessité **3 release candidates** avant le tag final stable (PRs #129 CHANGELOG date + #130 Cargo bump 0.1.0→0.1.1 pour smoke /health). **Cause racine** : le smoke test post-build `release.yml` lit la version via `env!("CARGO_PKG_VERSION")` au compile-time, qui doit matcher le tag git. **Sans le Cargo bump manuel**, le smoke test échoue sur tag v0.1.1 alors que le code shippe encore 0.1.0.

**Lesson** : la release process actuelle exige 2 commits manuels avant tag (bump Cargo + CHANGELOG date). Coût marginal acceptable v0.1.x mais devient un footgun à l'usage. **Action item Epic 11 prep** : automatiser ces 2 bumps via un script `scripts/prepare-release.sh <version>` qui patch `Cargo.toml` + `CHANGELOG.md` + commit-message standardisé.

### C3. **Action item Epic 9.5 retro #3 (doc utilisateur PME) non-adressé**

Action item #3 hérité Epic 9.5 retro (« Documentation utilisateur PME Kesh — guide consolidant OLICo Art. 9+10 + onboarding + self-hosting docker-compose ») reste **non-adressé** post-Epic Hotfix v0.1.1. C'est une **dette catégorie A** (action retrospective non-complétée) qui aurait dû être adressée avant ou pendant cet epic. **Carry-forward strict** : à placer en critical path Epic 11 prep.

### C4. **Manuel admin dérive partiellement adressée (Issue #127 / KF-035)**

v011-5 a réécrit la section « Premier démarrage » du manuel admin (setup-UI + recovery) qui adresse **partiellement** Issue #127 (KF-035 « dérive plus large — RBAC 5 rôles fictifs vs 2 réels code, kesh-cli inexistant utilisé partout »). Le commentaire de merge prévu est « v011-5 adresse partiellement (section Premier démarrage). RBAC fictif + kesh-cli inexistant restent ouverts ». **Issue #127 reste OPEN** post-merge PR #134.

**Lesson** : les manuels écrits par LLM (Opus à l'origine) ont une tendance à inventer des sous-commandes CLI et des rôles RBAC. À chaque modification du manuel, vérifier que les **commandes/composants mentionnés existent** dans le code source. Action item ci-dessous.

### C5. **KFs #124-127 créées par v011-2 sans plan de remédiation immédiat**

v011-2 a découvert 4 KFs en cours d'exécution (cohérent CLAUDE.md « hors fenêtre rétrospective ») : #124 E2E onboarding-path-b rot, #125 couverture backend handlers, #126 couverture frontend .ts, #127 KF-035 manuel admin dérive. Toutes labellisées `known-failure` + `technical-debt` + `triage`. **Aucune n'a `v0.2-milestone` ni story de remédiation planifiée** au moment de la rétro — elles sont en catégorie A par défaut (cf. CLAUDE.md « KFs ouvertes GitHub Issues = candidates catégorie A par défaut sauf labelling explicite v0.2-milestone »).

**Action obligatoire pré-Epic 11** : trier ces 4 KFs (#124-127) selon politique zero tech debt carry-forward :
- (a) fix immédiat dans une story dédiée Epic 11 prep
- (b) reclassement catégorie B avec label `v0.2-milestone` et story de remédiation Epic 11 ou Epic dédié closure
- (c) reclassement catégorie B avec justification de différement (rare — réservé aux KFs de scope > 1 epic).

### C6. **Pas de E2E Playwright `setup.spec.ts` exécuté pendant la review**

Le test Playwright `setup.spec.ts` créé par v011-5 (4 tests) **n'a pas été exécuté** pendant le code-review (la CI Kesh ne lance pas Playwright). C'est conforme CLAUDE.md « Pas exécuté par la CI principale ». Mais le quality gate de la PR n'inclut pas la validation manuelle E2E. **Soit l'exécuter en local avant tag release v0.1.2**, soit accepter le risque que le flow `/setup` E2E ne soit validé qu'en prod sur le 1er deploy fresh.

### C7. **GitHub Milestone v0.2 toujours non formalisé**

Action item Epic 9.5 retro #1 (créer GitHub Milestone v0.2 + labellisation systématique items catégorie B) reste **partiellement done** : label `v0.2-milestone` existe et utilisé sur #98 + #133 (et probablement #122 — à vérifier). Mais milestone GitHub formel non créé, et la labellisation systématique de tous les items L1-L18 9-2a/9-2b n'a pas eu lieu. **Action item carry-forward Epic 11 prep**.

---

## 💡 Key insights (méthodologiques)

### I1. **Hotfix workflow BMAD complet validé sur 5 stories en 4 jours**

Epic Hotfix v0.1.1 a appliqué le full workflow BMAD (`create-story` + `validate` + `dev-story` + `code-review`) sur 5 stories distinctes en 4 jours calendaires (2026-05-28 → 2026-05-31) **incluant une release intermédiaire** (v0.1.1 publiée 2026-05-29). Le workflow scale aux hotfix releases — pas besoin de raccourci « hot path » qui sacrifie la qualité review.

### I2. **Patterns DB CHECK constraint comme « land mines » spec validate**

Les contraintes DB CHECK whitelist (e.g. `chk_refresh_tokens_revoked_reason`) sont **systématiquement ratées par les 4 passes spec validate**, même quand Opus est dans la rotation. La spec génère une valeur (e.g. `"admin_break_glass_reset"`) et personne ne fait le `grep -rnF "<value>" crates/kesh-db/migrations/` pour vérifier si elle est dans le whitelist. **Codification CLAUDE.md possible** : ajouter une check `grep CHECK constraint compatibility` à la check-list spec validate pour toute story qui INSERT/UPDATE des colonnes avec CHECK constraint (cf. action item).

### I3. **Refactor mid-epic via story `superseded` propre**

Pattern « v011-3 SUPERSEDED par v011-5 » montre comment refactor mid-epic sans laisser de zombies. Marqueurs explicites dans :
- Planning doc `epic-hotfix-v0.1.1.md` (section v011-3 + disclaimer + section v011-5 ajoutée).
- Sprint-status v011-3 status = `superseded` (nouveau statut non-officiel mais clair par contexte).
- v011-5 story file mentionne explicitement « absorbe v011-3 ».
- PR #134 description mentionne « closes #121 (absorbe v011-3 SUPERSEDED) ».

**À codifier** : pattern `superseded` comme statut sprint-status officiel.

### I4. **Spec validate Pass 3 Opus catch architectural cross-fichiers — pattern récurrent**

3e validation empirique du pattern « Opus catches architectural » :
- Story 10-5 Pass 3 Opus : 16 patches sur 5 axes.
- Story 9-5-1c Pass 3 Opus : `focus-visible` MEDIUM a11y subtile.
- **Story v011-5 Pass 3 Opus : 3 HIGH cross-fichiers** (NewAuditLogEntry schema, 30 sites `from_fields_for_test`, 28 sites `AppState`) — sans ces patches, compilation aurait échoué au dev-story.

Pattern reproductible : **Pass 3 Opus est rarement « inutile »**. La 4ème passe Sonnet sert de convergence check final.

### I5. **Release intermédiaire pendant un epic = pattern viable**

Epic Hotfix v0.1.1 prouve qu'**une release peut être taggée AU MILIEU d'un epic** (v0.1.1 publié 2026-05-29 entre v011-2 done et v011-4/v011-5). Ce n'est pas une mauvaise pratique — l'epic groupe **logiquement** les 5 stories liées au déploiement initial, mais physiquement elles peuvent shipper en 2 vagues (v0.1.1 critique + v0.1.2 UX). Pattern à conserver pour futurs épics multi-vagues.

---

## 🚀 Epic 11 preview & dépendances

**Epic 11 = TVA Suisse** — 2 stories planifiées (cf. sprint-status.yaml) :
- 11-1 Configuration taux TVA (admin saisit les taux applicables)
- 11-2 Calcul TVA + rapport par période (FR67-FR71)

**Référence pré-existante** : `_bmad-output/planning-artifacts/research-swiss-co-958f.md` (créé Epic 9.5 Story 9-5-4) couvre déjà LSCSE / Art. 70 LTVA factures électroniques. Anticipation = couverte.

**Dépendances Epic Hotfix v0.1.1 → Epic 11** :

| Item Epic Hotfix | Disponibilité Epic 11 |
|---|---|
| v0.1.1 publié Docker Hub + déployé prod Guy | ⚠️ **CONDITION SINE QUA NON** v0.1.2 doit aussi être taggé + déployé sur prod NAS Guy AVANT kickoff Epic 11 (cf. epic-hotfix planning §"Dépendances aval Epic 11"). |
| Recovery break-glass disponible | ✅ v0.1.2 livre. |
| Pattern double-usage env vars | ✅ Mémoire `feedback_*` à créer post-PR merge. |
| Pattern « ⚠️ Action requise » CHANGELOG | ✅ Disponible — sera réutilisé Epic 11 si TVA introduit migration breaking sur factures existantes. |
| README roadmap v0.1.2 | ✅ Sync fait commit `95d6138`. |

**Risques Epic 11 (TVA Suisse)** :
- TVA introduit potentiellement **migrations breaking** sur `invoice_lines` (ajout colonnes `vat_rate_id`, `vat_amount`, etc.) — appliquer CLAUDE.md migration breaking policy P1-P5 (bump `kesh_version_min_required`).
- Rapport TVA par période = nouveau type de rapport, peut nécessiter extension `kesh-report` crate (cf. patterns Story 9-1).
- Bornes périodes TVA (trimestrielles standard suisse, mais aussi annuelles, semestrielles) = checkpoint élicitation Guy obligatoire.
- Conformité LTVA : un export TVA officiel (e.g. format e-Government CH) pourrait être demandé v0.2.

**Risques Epic 12 (Avoirs & Paiements pain.001)** :
- Pattern `accept_batch` du POST `/api/v1/payments/batch` — à codifier comme pattern projet (cf. CLAUDE.md §"Pattern batch — FailedProposal per-proposal" déjà existant Epic 9.5 Story 9-5-3).

**Action items carry-forward Epic 11 prep** (cf. § ci-dessous).

---

## ✅ Action items Epic Hotfix v0.1.1 → Epic 11

### CRITICAL PATH (avant kickoff Epic 11)

| # | Action | Owner | Critère succès |
|---|---|---|---|
| **1** | **Merger PR #134 v011-5** sur main | Guy | Squash merge OK, CI verte (déjà confirmée). Closes #121, #122 (recovery v0.2+) reste open. |
| **2** | **Carry-forward Epic 9.5 retro #3** : créer guide utilisateur PME Kesh consolidant conformité OLICo + onboarding + self-hosting. **Dette catégorie A non-complétée**. | Guy | Document `docs/manual/fr/user-guide-pme.tex` (ou format équivalent) créé, PDF généré. Couvre wizard onboarding + flow comptable basique + restoration backup. |
| **3** | **Tag release v0.1.2** + publish Docker Hub `gcorbaz/kesh:0.1.2` + `latest` re-pointé + GitHub Release. Inclut v011-4 (port 80) + v011-5 (onboarding self-service + recovery). | Guy | Docker Hub image téléchargeable, GitHub Release page créée, CHANGELOG section `[0.1.2]` date renseignée. |
| **4** | **Sync website pré-tag v0.1.2** : `website/index.html` + `website/roadmap.html` mentionnent v0.1.2 disponible + features onboarding self-service + recovery break-glass (defer item AUD1-4 de Pass 1 code-review v011-5). | Guy | `git diff main -- website/` non-vide, claims marketing alignés post-merge. |
| **5** | **Triage KFs #124-127** (créées par v011-2) selon politique zero tech debt carry-forward : (a) fix immédiat story dédiée Epic 11 prep, (b) reclassement cat B avec `v0.2-milestone` + story remédiation planifiée, (c) reclassement cat B avec justification. | Guy + Claude | 4 KFs ont soit (a) une story fix planifiée Epic 11 prep, soit (b) label `v0.2-milestone` + commentaire de plan de remédiation, soit (c) justification cat B documentée. |
| **6** | **Action B1 héritée Epic 9.5** finalisée : créer GitHub Milestone `v0.2` formel + labellisation systématique items catégorie B (9-2a L13-L15, 9-2b L1-L18, KF #76, dettes Pass deferred D1-D5 Epic 10, etc.) avec `v0.2-milestone`. | Guy | Milestone `v0.2` créé sur GitHub, ~20 items labellisés `v0.2-milestone`, backlog v0.2 explicite. |
| **7** | **Validation install fresh + recovery sur prod NAS Guy** : tester `docker compose up -d` from scratch sur image `gcorbaz/kesh:0.1.2` → écran setup → admin créé → wizard complet. Puis tester recovery break-glass (modifier `.env`, restart, login avec nouveau mdp). | Guy | Cycle complet validé sur NAS Synology réel. Critère « install < 15 min » FR1 vérifié. |
| **8** | **Exécuter Playwright `setup.spec.ts`** en local (backend up + preset `setup-required` seedé) AVANT tag v0.1.2. 4 tests à valider : redirect /setup, validation mismatch, happy path cookies, 410 reload. | Guy + Claude | 4 tests passés en local, sinon corrections appliquées avant tag. |

### Cleanup parallèle Epic 11 (non-bloquant)

| # | Action | Notes |
|---|---|---|
| 9 | **Issue #127 (KF-035 manuel admin dérive)** : compléter le triage. v011-5 a adressé la section « Premier démarrage ». Reste à corriger RBAC fictif 5 rôles + kesh-cli inexistant utilisé partout. Issue peut rester OPEN ou être splittée en plusieurs issues focalisées. | Hors critical path Epic 11 — peut être cleanup parallèle. |
| 10 | **Mémoire `feedback_db_check_constraint_whitelist_validation`** à créer : codifier la lesson v011-5 (la spec validate ne vérifie pas les CHECK constraints DB sur les valeurs littérales utilisées). Pattern à ajouter au check-list spec validate. | Mémoire courte. |
| 11 | **Script `scripts/prepare-release.sh <version>`** : automatise bump `Cargo.toml` + `CHANGELOG.md` date + commit standardisé. Évite les 2 RCs manuels observés v0.1.1. | Story dédiée Epic 11 prep ou hors-story chore. |
| 12 | **Codifier statut `superseded`** dans sprint-status STATUS DEFINITIONS comme statut officiel (pattern v011-3 → v011-5). | Mémoire courte ou CLAUDE.md mineur. |

### Process (à conserver Epic 11+)

- **Cycle Sonnet → Haiku → Opus → Sonnet review** : maintenir (3 validations consécutives Epic 9.5, Epic 10, Epic Hotfix v0.1.1).
- **Splitting préventif > 5 modules** : maintenir.
- **Test Locally First** + exemption doc-only : maintenir.
- **Discipline Haiku grep ground-truth** pour CRITICAL/HIGH findings : maintenir (3 hallucinations dismissed cette epic).
- **Single PR par story** + retro+sprint-status closure groupés dans dernière PR : maintenir (`feedback_avoid_parallel_prs`).
- **Issue de traçage GitHub Epic future** comme catégorie B suffisant : maintenir.
- **README sync pré-push** : maintenir.

---

## 🔍 Readiness assessment Epic Hotfix v0.1.1

| Dimension | Status | Notes |
|---|---|---|
| Testing & Quality | ✅ | 0 régression baselines projet. ~70+ patches cumulés. CI verte sur 4 PRs mergées + PR #134. v011-2 full workspace 1308 tests verts. v011-5 ajoute 199 lib + 6 setup_admin_e2e + 5 SetupForm = 210 tests. |
| Deployment | ⚠️ → ✅ | v0.1.1 publié Docker Hub 2026-05-29. v0.1.2 release pending tag (post-merge PR #134 + action items #3 #4 #8). **Pas un blocker clôture epic** — c'est la prochaine étape immédiate. |
| Stakeholder Acceptance | ✅ | Single stakeholder Guy. v0.1.1 testé sur prod NAS (a révélé #120 catch-22 d'où l'epic, puis fix vérifié). v0.1.2 validation prod = action item #7 critical path. |
| Technical Health | ⚠️ → ✅ | 4 KFs ouvertes (#124-127) créées par v011-2 — **action item #5 critical path** pour triage. Pas de blockers techniques résiduels. Codebase stable, patterns réutilisables. |
| Unresolved Blockers | ⚠️ → ✅ | Action item Epic 9.5 retro #3 (doc utilisateur PME) **non-adressé** = dette catégorie A héritée. Action item #6 GitHub Milestone v0.2 = partiel. **Items #2, #5, #6 critical path Epic 11 prep**. |
| Documentation | ⚠️ | Manuel admin FR à jour (v011-4 port + v011-5 onboarding/recovery). README sync. **Manquant** : guide utilisateur PME (action #2 critical path). Website non synchronisé (action #4 critical path). |

**Verdict** : Epic Hotfix v0.1.1 fully done sur les 4 stories effectives. **Epic 11 gated par 8 action items critical path** dont 3 dette héritée Epic 9.5 retro (carry-forward strict). Pattern « zero tech debt carry-forward catégorie A » légèrement entamé (action #3 non-complétée 2 epics consécutifs) — à reprendre en main strictement avant Epic 11.

---

## 🎉 Team performance Epic Hotfix v0.1.1

**Claude (Developer + Reviewer)** : Epic Hotfix v0.1.1 a livré 4 stories en 4 jours calendaires (2026-05-28 → 2026-05-31) avec une release intermédiaire (v0.1.1 publiée 2026-05-29) et ~70+ patches review cumulés. Cycles review multi-LLM appliqués systématiquement (jusqu'à 4 passes Sonnet→Haiku→Opus→Sonnet pour v011-5). Discipline grep ground-truth Haiku validée 3 fois (3 hallucinations CRITICAL/HIGH dismissed). Pattern « double-usage env vars no-op-si-hash-identique » innové sur v011-5 (Guy 2026-05-30).

**Guy (Project Lead)** : décision « pas de workaround SQL en prod, attendre v0.1.1 propre » (2026-05-27) montre une discipline qualité forte sous pression d'un bug bloquant en prod. Refactor mid-epic v011-3 → v011-5 (2026-05-30) montre une capacité à reconnaître qu'un design intermédiaire (`KESH_ADMIN_RESET=true` flag explicite) était inférieur au design unifié final (double-usage automatique no-op). Pattern « reconnaître mid-epic qu'un design est sous-optimal et pivoter » à conserver.

---

## 📝 Commitments

**Action items total** : 8 critical path (3 hérités Epic 9.5 partiels/non-adressés + 5 nouveaux Epic 11 prep) + 4 cleanup parallèle + maintien process Epic Hotfix v0.1.1+

**Next steps immédiats** :

1. **Attendre review/merge PR #134** (CI verte, prêt). Closes #121.
2. **Triage KFs #124-127** + finaliser GitHub Milestone v0.2 (action items #5 #6).
3. **Guide utilisateur PME Kesh** (action item #2 — dette héritée Epic 9.5).
4. **Sync website v0.1.2** (action item #4).
5. **Tag release v0.1.2** + Docker Hub publish (action item #3).
6. **Validation install fresh + recovery sur prod NAS Guy** (action item #7).
7. **Playwright `setup.spec.ts` local** (action item #8).
8. **Kickoff Epic 11 TVA Suisse** quand TOUS les action items critical path sont done.

**Epic Hotfix v0.1.1 closed — Epic 11 TVA Suisse gated par les 8 action items critical path.**

---

## Fichiers livrés Epic Hotfix v0.1.1

### Code

- `crates/kesh-api/src/auth/bootstrap.rs` — refactor matrice 6 cas (v011-2 + v011-5), `Result<i64, AppError>` retour `user_count`.
- `crates/kesh-api/src/config.rs` — port défaut 80 (v011-4), `admin_username/password: Option<String>` (v011-5), `KESH_LOG_FILE_*` (v011-1), `LogConfig` struct.
- `crates/kesh-api/src/lib.rs` — `AppState::users_exist: Arc<AtomicBool>` + `new_for_tests` constructeur (v011-5), mount `/api/v1/setup/admin`.
- `crates/kesh-api/src/main.rs` — capture user_count + init `users_exist` + bootstrap subscriber tracing.
- `crates/kesh-api/src/middleware/auth.rs` — 423 Locked gate (v011-5).
- `crates/kesh-api/src/errors.rs` — variants `SetupRequired` (423) + `SetupAlreadyComplete` (410) + i18n.
- `crates/kesh-api/src/routes/auth.rs` — `build_auth_cookies` pub(crate) (v011-5).
- `crates/kesh-api/src/routes/setup.rs` — NEW handler `POST /api/v1/setup/admin` (v011-5).
- `crates/kesh-api/src/routes/test_endpoints.rs` — preset `SetupRequired` + sync `users_exist` post-seed.
- `crates/kesh-api/src/logging.rs` — NEW (v011-1) `init_tracing` + file layer.
- `crates/kesh-db/src/test_fixtures.rs` — helper `seed_stub_company_only` (v011-5).
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — +14 clés `setup-*` + `error-setup-*` (v011-5).
- Frontend : `frontend/src/lib/app/stores/auth.svelte.ts`, `api-client.ts`, `routes/+layout.ts`, `routes/setup/+page.svelte` + `+layout.ts`, `lib/features/setup/SetupForm.svelte` + `setup.api.ts` + `SetupForm.test.ts` (v011-5).
- 28 fichiers tests d'intégration patched avec `users_exist` field addition (v011-5).

### Tests

- `crates/kesh-api/tests/setup_admin_e2e.rs` — NEW 8 tests E2E (v011-5).
- `frontend/tests/e2e/setup.spec.ts` — NEW 4 tests Playwright (v011-5).
- `frontend/src/lib/features/setup/SetupForm.test.ts` — NEW 5 tests Vitest (v011-5).

### Infrastructure

- `Dockerfile` — `EXPOSE 80` (v011-4).
- `docker-compose.{prod,dev,base}.yml` — port 80 mappings, healthcheck `http://localhost/health`, mount `./log:/var/log/kesh` (v011-1 + v011-4).
- `.github/workflows/release.yml` — smoke test port 80 (v011-4).
- `.gitignore` — `log/` (v011-1).

### Documentation

- `docs/manual/fr/admin-manual.tex` + PDF régénéré — sections « Configuration des logs fichier » (v011-1), « Premier démarrage » réécrite + « J'ai oublié mon mot de passe » + matrice 6 cas (v011-5), « Changer le port d'écoute » (v011-4).
- `.env.example` — sections « Logs fichier » (v011-1), « Compte admin initial » triple-usage (v011-5), « Port » override (v011-4).
- `CHANGELOG.md` — sections `[0.1.1]` (v011-1 + v011-2) + `[0.1.2]` avec `### ⚠️ Action requise` (v011-5) + section Modifié port 80 (v011-4).
- `README.md` — roadmap v0.1.x → v0.1.2 (commit `95d6138`).
- `_bmad-output/planning-artifacts/epic-hotfix-v0.1.1.md` — story v011-3 SUPERSEDED + section v011-5 ajoutée.

### GitHub

- **Issues fermées (4)** : #118 port 80, #119 logs, #120 catch-22, #121 break-glass (closes au merge PR #134).
- **Issues créées (5)** : #122 recovery prod-grade v0.2+, #124-127 KFs par v011-2, #133 TOCTOU L1 v0.2-milestone par v011-5.
- **Releases publiées** : v0.1.1 (Docker Hub + GitHub Release 2026-05-29), v0.1.2 en attente tag.

### Sprint tracking

- `_bmad-output/implementation-artifacts/sprint-status.yaml` — 4 entries v011-1/2/4/5 = `done`, v011-3 = `superseded`, `epic-hotfix-v0.1.1-retrospective` = `optional → done` (post-commit retro).

---

**Epic Hotfix v0.1.1 « Onboarding fresh-install + admin recovery + logs fichier + port 80 » formellement closed 2026-05-31.**

**Politique zero tech debt carry-forward catégorie A : entamée par dette héritée Epic 9.5 retro #3 (doc utilisateur PME). Carry-forward strict Epic 11 prep critical path. Epic 11 TVA Suisse gated par 8 action items critical path.**
