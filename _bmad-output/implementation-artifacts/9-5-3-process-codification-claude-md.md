# Story 9.5-3: Process codification CLAUDE.md

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want codifier dans `CLAUDE.md` les trois patterns de process découverts ou validés lors des Epics 7/8/9 mais pas encore documentés formellement (Haiku grep ground-truth obligatoire, AcceptedProposal/FailedProposal batch pattern strict, zero tech debt carry-forward policy),
so that ces règles deviennent appliquées systématiquement par tout futur cycle review/dev/retro et résistent au turnover développeurs ou agents LLM.

## Scope

Édite **`CLAUDE.md` à la racine du repo uniquement**. Ajoute **3 nouvelles sections** documentaires (et 0 nouvelle règle de code).

1. **Sous-section « Haiku 4.5 grep ground-truth obligatoire »** insérée sous la section `## Review Iteration Rule` existante, juste après le bloc `**Boucle automatique** :` (avant la sous-section `### Règle de splitting préventif` ligne 137). Cible : la règle déjà connue (memory `feedback_haiku_review_diff_combined`) est promue de user-memory à règle projet, et l'orchestrateur qui lance Haiku peut la citer textuellement.

2. **Sous-section « AcceptedProposal / FailedProposal batch pattern strict »** insérée comme nouvelle sous-section `### Pattern batch — FailedProposal per-proposal` toujours à l'intérieur de `## Review Iteration Rule` (juste après la nouvelle sous-section Haiku ci-dessus), OU comme nouvelle section H2 top-level `## Pattern batch — FailedProposal per-proposal` après `## Review Iteration Rule`. **Décision §placement-batch-pattern** ci-dessous = sous-section H3 à l'intérieur de `## Review Iteration Rule` (cohérent avec « règles process » thématique). Cible : tout endpoint `accept_batch`-style (Epic 8 `accept_one_invoice/split/rule`, Epic 11 pain.001 paiements batch, Epic 12 si applicable) sait que **aucune escalade `AppError` globale** n'est autorisée pour des erreurs per-proposal.

3. **Nouvelle section top-level `## Tech debt management — zero carry-forward policy`** insérée **après** `## Review Iteration Rule` et **avant** `## Issue Tracking Rule` (ligne 152). Cible : la rétrospective de chaque epic doit catégoriser (A/B/C) tous les items résiduels, et la catégorie A bloque le kickoff de l'Epic N+1.

**Hors scope** :

- Édition de `docs/` ou de `_bmad-output/planning-artifacts/architecture.md` — les 3 patterns sont des règles **process projet**, pas des décisions architecture (cf. distinction CLAUDE.md = règles méta workflow, architecture.md = décisions techniques durables).
- Suppression des memories user-level concernées (`feedback_haiku_review_diff_combined`, `feedback_zero_tech_debt_carryforward`) — la promotion vers CLAUDE.md project-level **ne supprime pas** les memories. Elles restent en place pour traçabilité historique des décisions (cohérent §Q4 epic-9-5.md).
- Ajout d'une 4ème règle « rotation LLM Sonnet → Haiku → Opus → Sonnet ». Le pattern est **déjà** codifié dans la section existante `## Review Iteration Rule` (ligne 121 : « cycle Opus → Sonnet → Haiku → Opus »). Ne pas dupliquer ni reformuler — risque dérive. Si l'ordre canonique a évolué (Insight I1 retro Epic 9 mentionne Sonnet → Haiku → Opus → Sonnet), la **mise à jour de l'ordre est in-scope** mais conserve une **seule** mention canonique.
- Modification du `README.md` (la « Feuille de route » n'est pas impactée — Epic 9.5 est interne process, pas une feature utilisateur).
- Ajout de nouveaux hooks `pre-commit` ou scripts d'automatisation des 3 règles. La codification est documentaire ; l'application reste manuelle (orchestrateur LLM ou dev humain).
- Toute modification de `cargo`/`npm` packages ou de tests. **Story 100% documentation.**

## Acceptance Criteria

### Section 1 — Haiku grep ground-truth

1. **Given** `CLAUDE.md` post-Story 9.5-3, **When** la section `## Review Iteration Rule` est lue, **Then** elle contient une sous-section nommée explicitement `### Haiku-specific guardrails — grep ground-truth obligatoire` (titre déterministe pour faciliter renvoi textuel par d'autres docs / agents).

2. **Given** cette sous-section, **When** elle est lue, **Then** elle contient au minimum :
   - Description du symptôme : Haiku 4.5 reviewers (`BlindHunter` / `EdgeCaseHunter`) peuvent affirmer **CRITICAL/HIGH** « REGRESSION-P1 — patch X n'a pas été appliqué » sur un diff combiné multi-commit, alors que le patch est présent.
   - Cause root : Haiku traite mal l'indexation des line numbers d'un diff `git show A B` quand le 2e commit `B` re-touche des hunks du 1er `A` ; les line numbers du 2e hunk correspondent au file post-A, et Haiku peut chercher la ligne X et y voir le contenu de A, ratant le patch de B.
   - Règle d'application : **pour tout finding `CRITICAL` ou `HIGH` affirmant l'absence de code attendu**, l'orchestrateur **DOIT** lancer `grep -n "<pattern issu du patch>" <file>` avant de traiter comme réel. Si présent → dismiss comme faux-positif. Documenter le dismiss dans le Change Log de la passe.
   - Mitigation préférée : à partir de la Pass 2 d'un cycle review, passer à Haiku un **diff unique** (le commit final `HEAD vs main`) plutôt qu'une séquence de commits multi-patches.
   - Scope du bug : spécifique Haiku 4.5. Sonnet 4.6 et Opus 4.7 ne reproduisent pas l'erreur d'indexation (validé Epic 9 et antérieur).
   - Référence traçabilité : `(cf. memory feedback_haiku_review_diff_combined, validé empiriquement Stories 8-5a-bis Pass 2 Haiku 2026-05-12 [BH2-1 missing scale() validation ligne 2120 + BH2-2 missing != bank_ledger guard ligne 2172] et 9-2b Pass 2 Haiku 2026-05-15 [route.continue() sans await ligne 66 + resolveDownload! sans validation ligne 201] — 4 hallucinations CRITICAL/HIGH réfutées par grep ground-truth)`.

3. **And** cette sous-section est placée **à l'intérieur** de la section H2 `## Review Iteration Rule` (sous-titre H3 `### Haiku-specific guardrails — grep ground-truth obligatoire`), **après** le bloc `**Boucle automatique** :` et **avant** la sous-section existante `### Règle de splitting préventif` (ligne 137 actuelle). Ordre logique : règle générale → cas spécifique Haiku → règle splitting préventif.

4. **And** la mention du cycle de rotation LLM existante (`Opus → Sonnet → Haiku → Opus` actuelle dans `## Review Iteration Rule`) **est harmonisée** vers `Sonnet → Haiku → Opus → Sonnet` (validé empiriquement Epic 9 retro Insight I1, 3 cycles complets — l'ordre actuel est une intuition initiale non-validée par observation systématique). Patcher la ligne en une **seule occurrence canonique** pour éviter toute duplication ou ambiguïté entre la section Review Iteration Rule pré-existante et la nouvelle sous-section Haiku. **Décision §rotation-order-update tranchée Pass 1 spec validate Sonnet 4.6** (cf. §Décisions et Change Log).

### Section 2 — AcceptedProposal/FailedProposal batch pattern

5. **Given** `CLAUDE.md` post-Story 9.5-3, **When** la section `## Review Iteration Rule` est lue, **Then** elle contient une sous-section nommée explicitement `### Pattern batch — FailedProposal per-proposal` (titre déterministe).

6. **Given** cette sous-section, **When** elle est lue, **Then** elle contient au minimum :
   - Énoncé de la règle : pour tout endpoint type `accept_batch` (qui traite N proposals/operations en une seule requête et retourne `{ accepted: [...], failed: [...] }`), **aucune erreur per-proposal ne doit escalader en `AppError` global** retournée comme HTTP error code (4xx/5xx). Chaque erreur d'une proposal individuelle est encapsulée en `FailedProposal` (signature détaillée bloc « Champs obligatoires » ci-dessous — identifiant business de la proposition + `error_code` + `details` optionnels) dans le `failed[]` du response body, avec HTTP `200 OK` au niveau de la requête (succès partiel reste un succès HTTP).
   - Exceptions explicites : les `AppError` global sont autorisées **uniquement** pour les erreurs qui invalident la requête entière en amont du traitement per-proposal — exemples : `401 Unauthorized` (auth middleware), `403 Forbidden` (RBAC global), `400 Bad Request` (body parse fail / schéma JSON invalide), `500 Internal Server Error` (DB pool fermé, panic, etc.). Toute erreur **qui dépend de la proposal individuelle** (validation amount > 0, FK manquant, race condition optimistic lock, currency mismatch, business rule) → `FailedProposal`.
   - Exemple canonique référencé : `accept_one_invoice` (Story 8-4), `accept_one_split` (Story 8-5a-bis), `accept_one_rule` (Story 8-5b). Cf. retro Epic 8 Insight I2 « accept_one_X strict (FailedProposal per-proposal) inviolable » + Story 8-5b Pass 4 ECH4-1 correction (`BANK_ACCOUNT_NOT_CONFIGURED` `200 + failed[]` au lieu de `412` AppError).
   - Garde-fou défensif : dans un `match` exhaustif sur les variants de `Rule` / `ProposalType`, **NE PAS** utiliser `unreachable!()` aux sites variants (piège pour futur refactor + risque crash Tokio task). Préférer `tracing::error + AppError::Internal` (cohérent Story 8-5b code-review Pass 1 patch).
   - Champs obligatoires de `FailedProposal` (signature canonique Epic 8 ground-truth — `crates/kesh-api/src/routes/reconciliation.rs:152-156` à l'heure de la spec) :
     - **identifiant business de la proposition** (e.g. `bank_transaction_id: i64` pour Epic 8 réconciliation ; pour Epic 11 pain.001 ce sera probablement `payment_id` ou équivalent, à adapter selon le type de proposition du batch). **Anti-pattern explicite** : NE PAS utiliser un index positionnel `proposal_index: usize` — fragile à toute réorganisation du batch par le client.
     - `error_code: String` (valeur recommandée = constante canonique e.g. `"BANK_ACCOUNT_NOT_CONFIGURED".to_string()`, **jamais** interpolation `format!("error: {}", e)`. Pour contexte dynamique, utiliser `details`).
     - `details: Option<serde_json::Value>` (JSON object additionnel pour contexte spécifique au code, e.g. `{ "bankAccountId": 17 }`).
   - Réutilisation prévue : Epic 11 (pain.001 paiements batch — un fichier XML contient N transactions, chaque transaction peut échouer indépendamment) + tout endpoint futur qui retournera un summary `{ accepted, failed }`. **Note** : CAMT.053 (Epic 12) est de l'**import** raw (parser → INSERT bank_transactions sans décision utilisateur per-transaction), donc ne suit PAS ce pattern ; la réconciliation post-CAMT.053 utilise déjà l'API Epic 8 `accept_batch` qui implémente ce pattern.
   - Référence traçabilité : `(pattern hérité Epic 8 — cf. retro Epic 8 Insight I2 + Story 8-5b Pass 4 ECH4-1)`.

7. **And** cette sous-section est placée **à l'intérieur** de la section H2 `## Review Iteration Rule` (sous-titre H3 `### Pattern batch — FailedProposal per-proposal`), **immédiatement après** la sous-section Haiku (AC #3) et **avant** la sous-section existante `### Règle de splitting préventif`. Ordre logique : règles review (Haiku, splitting) → règle architecture batch.

   **Note placement alternative considérée** : section H2 top-level dédiée `## Pattern batch — FailedProposal per-proposal`. **Rejetée** car (a) le pattern a été détecté en review (Pass 4 ECH4-1 Story 8-5b), (b) éviter de fragmenter CLAUDE.md en trop de sections H2 (déjà 8 sections), (c) sous-section sous Review Iteration Rule reste discoverable via grep `FailedProposal`.

7bis. **And** un **renvoi cross-section** est ajouté dans `## Code Quality Rules` (après le bullet `**E2E Testing**` actuel) sous forme d'un nouveau bullet `- **Batch API conventions** — Pour les endpoints batch (style \`accept_batch\` qui retournent \`{ accepted, failed }\`), cf. §"Pattern batch — FailedProposal per-proposal" sous §"Review Iteration Rule".` Justification : un futur agent LLM cherchant des conventions HTTP API ne fouillera pas spontanément `## Review Iteration Rule` — le renvoi pallie le déficit de discoverability sans dupliquer le contenu (Pass 1 HIGH-02 patch minimal vs section H2 top-level dédiée).

### Section 3 — Zero tech debt carry-forward

8. **Given** `CLAUDE.md` post-Story 9.5-3, **When** le fichier est lu, **Then** une nouvelle section H2 top-level `## Tech debt management — zero carry-forward policy` existe (titre déterministe).

9. **And** cette section est placée **après** la section H2 `## Review Iteration Rule` (qui se termine par sa sous-section `### Règle de splitting préventif`) et **avant** la section H2 `## Issue Tracking Rule`. Ordre logique : règles review/process → règle gestion dette → règle tracking issues GitHub. **Note** : aucune référence aux numéros de ligne de CLAUDE.md dans les ACs (fragiles à toute insertion antérieure dans le fichier) — utiliser les titres canoniques de section comme ancres stables.

10. **And** cette section contient au minimum :
    - Énoncé de la politique : pas de cumul de dette technique inter-epic. À chaque rétrospective d'epic, **toutes les vraies dettes (catégorie A ci-dessous) doivent être adressées (fix appliqué OU explicitement reclassées en catégorie B avec justification + story de remédiation planifiée)** avant le kickoff de l'Epic N+1.
    - Triage obligatoire en 3 catégories à chaque rétrospective :
      - **Catégorie A — vraie dette** : bug latent, incohérence non-documentée, action retrospective non-complétée, KF dormante GitHub ouverte (sans label `v0.2-milestone`). **Doit être fixée** avant kickoff Epic suivant.
      - **Catégorie B — limitation v0.2 légitime** : feature/limitation documentée avec scope explicite (`L1`/`L2`/... style dans story file) ET story de remédiation planifiée Epic futur. Acceptable indéfiniment tant que tracée.
      - **Catégorie C — décision design intentionnelle** : pattern volontaire (e.g. tables exclues du global export pour raison sécurité, INNER JOIN FK garante, audit-trail-only acceptée v0.1). **Pas une dette.**
    - Critère « critical path » : les items catégorie A passent du « cleanup parallèle optionnel » au « bloquant kickoff Epic suivant » dans la section Action Items de chaque rétro.
    - Pattern « Epic dédié cleanup » : si le volume d'items catégorie A est élevé (seuil indicatif > 8 items ou couvre > 2 axes — KFs + code consistency + process), créer un **Epic dédié `« Technical Debt Closure »`** plutôt que faire un méga-Epic suivant qui mélange feature + dette. Précédents : **Epic 7 historique** (KF-001..007 fermées pré-Epic 8), **Epic 9.5 actuel** (~13 items A post-Epic 9, 4 stories).
    - Distinction critique au triage : limitations documentées (`L1-L18` style dans story files) = catégorie **B** si scope explicite **+** story remédiation planifiée. Sinon → catégorie **A** (dette implicite). Mémo : KFs ouvertes GitHub Issues = candidates catégorie **A** par défaut sauf labelling `v0.2-milestone` (cohérent §"Issue Tracking Rule") — le label `v0.2-milestone` **tient lieu de planification implicite** au cycle v0.2 et qualifie la KF en catégorie **B** sans qu'une story Epic spécifique soit déjà créée (la création de la story se fait au plus tard au kickoff Epic 13+ qui consommera le backlog v0.2).
    - Référence traçabilité : `(politique formalisée 2026-05-17 rétro Epic 9 — cf. memory feedback_zero_tech_debt_carryforward + pattern Epic 7 historique)`.

### Cohérence globale + non-régression

11. **Given** `CLAUDE.md` post-Story 9.5-3, **When** parcouru de bout en bout, **Then** **toutes les sections existantes pré-Story** sont conservées intactes dans leur contenu de fond (Project Overview, Communication, Repository Structure, BMAD Architecture, Key Patterns, Code Quality Rules body, Test Locally First, Review Iteration Rule body, Issue Tracking Rule, Règle de commit et push). Diff **de CLAUDE.md** = **insertion-only** pour les 3 nouvelles sous-sections/sections (T5.1 Haiku + T5.2 FailedProposal + T5.3 Tech debt) + 1 nouveau bullet renvoi cross-section dans `## Code Quality Rules` (T5.5) + **1 modification ligne sur place** pour l'harmonisation de l'ordre rotation LLM dans `## Review Iteration Rule` body (T5.4 — `Opus → Sonnet → Haiku → Opus` → `Sonnet → Haiku → Opus → Sonnet`, AC #4 tranchée Pass 1). Aucune autre modification de contenu existant.

12. **And** **aucun référence cassée** : tous les liens internes existants (e.g. mentions « cf. §Test Locally First », « cf. memory ... ») restent valides. Si une nouvelle référence interne est ajoutée (e.g. « cf. §Tech debt management »), elle pointe vers une section qui existe maintenant.

13. **And** les **3 nouvelles sections respectent le style éditorial existant** de CLAUDE.md :
    - Français (cohérent §Communication : « The user (Guy) works in French »).
    - Format Markdown idem (titres `##` / `###`, emphasis `**...**`, listes `-`, blocks code triple-backtick si exemple code utile).
    - Pas d'emoji (cohérent style sobre CLAUDE.md).
    - Renvois textuels au style existant CLAUDE.md (noms de fonctions, stories, memories, sections) — les paths Rust absolus type `crates/kesh-api/...` sont **évités** dans CLAUDE.md (référence cassable au refactor code), conservés uniquement dans les story files de référence et les memories.

14. **And** **aucun lien externe cassé** : si une URL est ajoutée (e.g. lien GitHub Issues), elle est testée valide ou utilise pattern relatif `(`[guycorbaz/kesh/issues](https://github.com/guycorbaz/kesh/issues)`)` déjà présent CLAUDE.md.

15. **Given** `git diff CLAUDE.md` post-implémentation, **When** lu, **Then** lignes ajoutées **cible ~120-150 lignes** (3 sections × ~40-50 lignes chacune en moyenne), **hard cap ~180 lignes** (= somme des bornes hautes Tx T2.7 60 + T3.9 70 + T4.8 50 ajustée pour la cible — la borne haute T4.8 originale 80 est ramenée à 50 pour aligner avec ce cap). Si > 200 lignes : alerte verbosité, refactoriser en fichiers de référence sous `docs/` (anti-pattern à éviter — CLAUDE.md doit rester lisible bout en bout).

16. **And** **aucun test ni code Rust/TS/Svelte n'est modifié**. `git diff --stat` doit montrer **uniquement** `CLAUDE.md | XX +/-` (plus éventuellement `sprint-status.yaml` pour le marquage `done` et le story file `9-5-3-...md` pour le Change Log final). Story documentation-only stricte.

## Tasks / Subtasks

- [x] **T1** Pre-flight : lire `CLAUDE.md` actuel et identifier ancres d'insertion exactes (AC: #3, #7, #9)
  - [x] T1.1 `Read /home/gcorbaz/Synology/devel/kesh/CLAUDE.md` ligne 111-152 pour confirmer structure section `## Review Iteration Rule` + position `### Règle de splitting préventif` (ligne 137) + position `## Issue Tracking Rule` (ligne 152).
  - [x] T1.2 Identifier le **point d'insertion 1** : **après** le bloc `**Exception** : si un finding MEDIUM+ est explicitement reclassé...` (dernière ligne de contenu de la section H2 `## Review Iteration Rule` avant la sous-section H3 `### Règle de splitting préventif` — ligne 135 actuelle de CLAUDE.md à l'heure de la spec, mais l'ancre est le **texte** de la ligne Exception, pas le numéro) et **avant** la ligne vide précédant `### Règle de splitting préventif`. C'est le seul point propre qui ne fragmente pas le bloc logique « Boucle automatique → Cette règle s'applique à → Exception » qui forme une unité cohérente.
  - [x] T1.3 Identifier le **point d'insertion 2** : entre la nouvelle sous-section Haiku (T2) et la sous-section existante `### Règle de splitting préventif`. Logiquement : ce sera juste après la nouvelle sous-section H3 Haiku écrite en T2.
  - [x] T1.4 Identifier le **point d'insertion 3** : exactement entre la fin de la sous-section `### Règle de splitting préventif` (avant ligne 152 actuelle `## Issue Tracking Rule`). Note : la dérogation paragraphe en fin de §splitting préventif (ligne 150) est la dernière ligne avant la nouvelle section H2.

- [x] **T2** Rédiger sous-section H3 `### Haiku-specific guardrails — grep ground-truth obligatoire` (AC: #1-#4)
  - [x] T2.1 Titre H3 exact : `### Haiku-specific guardrails — grep ground-truth obligatoire`.
  - [x] T2.2 Paragraphe d'intro : énoncé symptôme (CRITICAL/HIGH affirmation fausse), cause root (indexation diff multi-commit), 2 cas concrets Story 9-2b Pass 2 (BH2-1 + BH2-2) référencés en exemples sans recopier le texte verbatim de la memory.
  - [x] T2.3 Bloc règle « **Règle d'application** » avec mots-clés MAJUSCULES `DOIT` cohérents avec style CLAUDE.md (cf. ligne 75-79).
  - [x] T2.4 Bloc « **Mitigation préférée** » : à partir Pass 2 d'un cycle, passer diff unique.
  - [x] T2.5 Bloc « **Scope du bug** » : spécifique Haiku 4.5, Sonnet 4.6 / Opus 4.7 indemnes.
  - [x] T2.6 Ligne de référence finale italique : `(cf. memory feedback_haiku_review_diff_combined, validé empiriquement Stories 8-5a-bis Pass 2 Haiku 2026-05-12 + 9-2b Pass 2 Haiku 2026-05-15 — 4 hallucinations CRITICAL/HIGH réfutées par grep ground-truth)`.
  - [x] T2.7 Vérifier longueur sous-section : entre 25 et 60 lignes Markdown (cohérent §existant `### Quand sauter` ~15 lignes, `### Règle de splitting préventif` ~25 lignes).

- [x] **T3** Rédiger sous-section H3 `### Pattern batch — FailedProposal per-proposal` (AC: #5-#7)
  - [x] T3.1 Titre H3 exact : `### Pattern batch — FailedProposal per-proposal`.
  - [x] T3.2 Paragraphe énoncé de la règle (HTTP 200 OK + `failed[]` per-proposal, jamais AppError global).
  - [x] T3.3 Bloc « **Exceptions explicites** » : 401 / 403 / 400 body parse / 500 panic restent en AppError global (erreurs amont).
  - [x] T3.4 Bloc « **Champs obligatoires `FailedProposal`** » (signature ground-truth `crates/kesh-api/src/routes/reconciliation.rs:152-156`) : identifiant business de la proposition (e.g. `bank_transaction_id: i64` Epic 8, à adapter `payment_id` Epic 11 etc.) + `error_code: String` (valeur = constante canonique recommandée, pas interpolation `format!`) + `details: Option<serde_json::Value>`. **Anti-pattern explicite** dans le texte CLAUDE.md : NE PAS utiliser un index positionnel `proposal_index: usize` (fragile à toute réorganisation batch).
  - [x] T3.5 Bloc « **Garde-fou `match` exhaustif** » : pas de `unreachable!()` aux sites variants Rule/ProposalType, préférer `tracing::error + AppError::Internal` (cohérent Story 8-5b Pass 1 patch).
  - [x] T3.6 Bloc « **Référence canonique** » : `accept_one_invoice` (Story 8-4), `accept_one_split` (Story 8-5a-bis), `accept_one_rule` (Story 8-5b) — implémentations historiques sous `crates/kesh-api/src/routes/reconciliation.rs` au moment de la spec. **Dans la section CLAUDE.md cible, NE PAS inclure le path Rust complet** (référence cassée silencieusement si le fichier est refactoré/splitté) ; conserver uniquement les noms de fonctions + numéro de story de référence. Le path est utile uniquement dans cette spec pour le dev agent qui implémentera la section, pas dans le texte CLAUDE.md durable.
  - [x] T3.7 Bloc « **Réutilisation prévue** » : Epic 11 pain.001 batch + tout endpoint futur retournant `{ accepted, failed }`. **NE PAS** citer Epic 12 CAMT.053 (import raw, ne suit pas le pattern accept_batch ; sa réconciliation post-import utilise déjà l'API Epic 8 accept_batch conformément).
  - [x] T3.8 Ligne de référence finale italique : `(pattern hérité Epic 8 — cf. _bmad-output/implementation-artifacts/epic-8-retro-2026-05-14.md Insight I2 + Story 8-5b Pass 4 ECH4-1 correction)`.
  - [x] T3.9 Vérifier longueur sous-section : entre 30 et 70 lignes Markdown.

- [x] **T4** Rédiger section H2 `## Tech debt management — zero carry-forward policy` (AC: #8-#10)
  - [x] T4.1 Titre H2 exact : `## Tech debt management — zero carry-forward policy`.
  - [x] T4.2 Paragraphe énoncé politique : pas de cumul inter-epic, dettes A fixées avant kickoff Epic N+1.
  - [x] T4.3 Bloc « **Triage obligatoire — 3 catégories** » en liste à puces avec définitions A / B / C complètes (cf. AC #10).
  - [x] T4.4 Bloc « **Critical path** » : items A passent de « cleanup parallèle optionnel » à « bloquant kickoff » dans la section Action Items de chaque rétro.
  - [x] T4.5 Bloc « **Pattern Epic dédié cleanup** » : seuil indicatif > 8 items A ou > 2 axes → Epic dédié type « Technical Debt Closure » (précédents Epic 7 + Epic 9.5).
  - [x] T4.6 Bloc « **Distinction A vs B au triage** » : limitations `L1-L18` style = B **si** scope explicite + story remédiation. Sinon = A. KFs GitHub par défaut = A sauf label `v0.2-milestone`.
  - [x] T4.7 Ligne de référence finale italique : `(politique formalisée 2026-05-17 rétro Epic 9 — cf. memory feedback_zero_tech_debt_carryforward + pattern Epic 7 historique « Technical Debt Closure »)`.
  - [x] T4.8 Vérifier longueur section : entre 40 et **50 lignes Markdown** (ramené depuis 80 pour aligner avec AC #15 hard cap ~180 lignes total ; section H2 top-level, cf. ~63 lignes `## Test Locally First`, ~40 lignes `## Issue Tracking Rule`).

- [x] **T5** Appliquer édits via Edit tool (3 insertions) — préserver intégralement le contenu pré-existant (AC: #11-#16)
  - [x] T5.1 Insertion 1 (sous-section Haiku) via Edit avec `old_string` ancré sur **les 2-3 dernières lignes du bloc Exception** (la ligne `**Exception** : si un finding MEDIUM+ est explicitement reclassé...` + ligne vide qui la suit + ligne `### Règle de splitting préventif`). `new_string` = même bloc Exception + ligne vide + nouvelle sous-section H3 Haiku complète + ligne vide + `### Règle de splitting préventif`. **NE PAS** ancrer sur le bloc `**Boucle automatique** :` — cela fragmenterait le bloc logique « Boucle automatique → Cette règle s'applique à → Exception » qui est cohérent et doit rester contigu.
  - [x] T5.2 Insertion 2 (sous-section FailedProposal batch) via Edit : `old_string` ancré sur la **dernière ligne** de la nouvelle sous-section Haiku (juste insérée) + 1ère ligne `### Règle de splitting préventif`. `new_string` = bloc Haiku final + ligne blanche + nouvelle sous-section H3 FailedProposal + ligne blanche + `### Règle de splitting préventif`.
  - [x] T5.3 Insertion 3 (section H2 Tech debt) via Edit : `old_string` ancré sur **dernière ligne** de `### Règle de splitting préventif` (la ligne dérogation actuelle ligne 150) + ligne blanche + `## Issue Tracking Rule`. `new_string` = dernière ligne dérogation + ligne blanche + nouvelle section H2 `## Tech debt management — zero carry-forward policy` complète + ligne blanche + `## Issue Tracking Rule`.
  - [x] T5.4 Patch ordre rotation LLM (AC #4 décision tranchée Pass 1) : `grep -n "Opus → Sonnet → Haiku → Opus" CLAUDE.md` → identifier l'unique occurrence dans `## Review Iteration Rule`, patcher en `Sonnet → Haiku → Opus → Sonnet` (un seul Edit, ancré sur la ligne complète). Validé empiriquement Epic 9 Insight I1 (3 cycles complets). Documenter le patch en Change Log.
  - [x] T5.5 Insertion 4 (renvoi cross-section Code Quality Rules) via Edit (AC #7bis Pass 1 patch HIGH-02) : `old_string` ancré sur le bullet `**E2E Testing**` actuel de `## Code Quality Rules` + ligne blanche qui le suit + 1ère ligne `## Test Locally First`. `new_string` = bullet E2E Testing inchangé + ligne blanche + nouveau bullet `- **Batch API conventions** — Pour les endpoints batch (style \`accept_batch\` qui retournent \`{ accepted, failed }\`), cf. §"Pattern batch — FailedProposal per-proposal" sous §"Review Iteration Rule".` + ligne blanche + `## Test Locally First`.

- [x] **T6** Validation manuelle post-édition (AC: #11-#16)
  - [x] T6.1 `Read CLAUDE.md` complet bout en bout — vérifier diff = insertion-only (sauf AC #4 ligne 121 si harmonisation).
  - [x] T6.2 `grep -n "^## " CLAUDE.md` → assert nouvelle section H2 `## Tech debt management — zero carry-forward policy` présente entre `## Review Iteration Rule` et `## Issue Tracking Rule`.
  - [x] T6.3 `grep -n "^### " CLAUDE.md` → assert 2 nouvelles sous-sections H3 `### Haiku-specific guardrails` + `### Pattern batch — FailedProposal per-proposal` présentes.
  - [x] T6.3bis `grep "Batch API conventions" CLAUDE.md` → assert 1 match dans la section `## Code Quality Rules` (renvoi cross-section T5.5 ajouté Pass 1 spec validate HIGH-02 patch).
  - [x] T6.4 `wc -l CLAUDE.md` avant vs après : delta lignes attendu ~80 à ~180 (cohérent AC #15).
  - [x] T6.5 Spot-check style : pas d'emoji, FR cohérent, blocks code triple-backtick si exemples (probable T3 pour `FailedProposal { ... }` struct sketch).
  - [x] T6.6 `markdownlint CLAUDE.md` si dispo (sinon skip — pas dans toolchain projet) — sinon vérifier rendu manuel sur titres H2/H3/H4 + tables/lists.

- [x] **T7** Pas de tests automatisés (story documentation-only) — note exception explicite (AC: #16)
  - [x] T7.1 Documenter dans le Change Log final : « Story 9-5-3 est documentation-only — aucun test ajouté ni modifié. `cargo test --workspace` + `npm run test:unit` baselines préservées par construction (zéro édit code). »
  - [x] T7.2 **Pas de skip de la règle « Test Locally First »** : la règle CLAUDE.md §"Quand sauter" exempt explicitement les commits doc-only (`Cette règle ne s'applique pas aux commits doc-only`) — donc cette story tombe dans l'exception. Pas de `cargo test --workspace` ni `npm run test:e2e` requis. Reste à exécuter localement uniquement si la modif a touché autre chose que `CLAUDE.md` (ce qui ne devrait pas être le cas).

- [x] **T8** Mise à jour `sprint-status.yaml` (AC: post-validation)
  - [x] T8.1 Cette étape est gérée par le workflow `bmad-create-story` automatiquement (status `backlog → ready-for-dev`). Pas d'action dev manuelle requise.
  - [x] T8.2 Au moment `dev-story` complétée (Status `review → done`), mettre à jour `sprint-status.yaml` `9-5-3-process-codification-claude-md` : `ready-for-dev → in-progress → review → done`.

### Review Findings

Pass 1 code-review 2026-05-18, 3 reviewers parallèles Sonnet 4.6 (Blind Hunter + Edge Case Hunter + Acceptance Auditor, contextes frais isolés). 22 findings (0C + 6H + 8M + 8L) → 10 patch + 1 defer + 11 dismiss.

- [x] [Review][Patch] E11 — Contradiction sémantique `AppError::Internal` (HIGH, FailedProposal pattern) : préconisé garde-fou `unreachable!()` mais listé comme exception globale interdite per-proposal — clarifier que le cas variant manquant est une dégradation d'urgence acceptable [CLAUDE.md:174]
- [x] [Review][Patch] E1 — Pattern grep avec métacaractères regex (HIGH, Haiku guardrail) : ajouter `-F` (fixed-string) ou échappement explicite pour patterns contenant `.`, `*`, `[`, etc. [CLAUDE.md:144]
- [x] [Review][Patch] E2 — Pattern multi-ligne non couvert (HIGH, Haiku guardrail) : préciser que `grep -n` opère ligne par ligne → choisir une ligne représentative unique [CLAUDE.md:144]
- [x] [Review][Patch] E12 — Dette catégorie A découverte en cours d'Epic N+1 (HIGH, Tech debt) : règle silencieuse sur le triage hors fenêtre rétrospective — ajouter clarification (créer story de fix dans Epic en cours OU bloquer kickoff Epic N+2 selon sévérité) [CLAUDE.md:209]
- [x] [Review][Patch] E13 — Conflit labels v0.2-milestone vs gate v0.1 explicite (HIGH, Tech debt) : clarifier que gate v0.1 explicite > label v0.2-milestone → catégorie A [CLAUDE.md:204]
- [x] [Review][Patch] E3 — Règle Haiku grep couvre uniquement « absence d'un fix », pas « présence d'un bug affirmée » (MEDIUM) : étendre la règle d'application à présence [CLAUDE.md:144]
- [x] [Review][Patch] E4 — Finding architectural sans pattern grepable (MEDIUM, Haiku guardrail) : préciser « si pattern grepable ; sinon vérification manuelle/Read direct » [CLAUDE.md:144]
- [x] [Review][Patch] E15 — Item A résistant à 3+ tentatives de fix (MEDIUM, Tech debt) : ajouter soupape « exceptionnellement reclassement en B avec justification résistance constatée + story remédiation Epic+2 » [CLAUDE.md:208]
- [x] [Review][Patch] F1 — Définition Catégorie B + exception v0.2-milestone contradiction apparente (LOW, Tech debt) : reformuler définition pour intégrer alternative au lieu de poser exception immédiate [CLAUDE.md:204]
- [x] [Review][Patch] F6 — « Cause root » → « Cause racine » (LOW, Haiku guardrail FR cohérence) [CLAUDE.md:142]
- [x] [Review][Defer] E14 — Story remédiation B bloquée/annulée — mécanisme de réévaluation périodique [CLAUDE.md:204] — deferred, processus operational hors scope CLAUDE.md (suivi backlog v0.2)

**Dismissed (11)** : F2 (renvoi Markdown navigabilité, convention maison OK), F3 (asymétrie format BH identifiants, nit), F4 (déjà couvert E11 plus précis), F5 (observation informative non-actionnable), E5 (mitigation diff aplati non-actionnable, complexifier serait pire), E10 (ironie rotation Sonnet→Haiku, grep ground-truth EST le filet par design), E16 (seuil OU déjà marqué « indicatif »), E6/E7/E8/E9 (edge cases batch 3-buckets/doublons/vide/100%fail — scope Epic 11 spec future, pas CLAUDE.md durable).

### Review Findings Pass 2

Pass 2 code-review 2026-05-18, 2 reviewers parallèles Haiku 4.5 (Blind Hunter + Edge Case Hunter, contextes frais isolés). Diff cible : commit `be1dfce` (Pass 1 patches uniquement, diff unique respectant la règle Haiku indexing codifiée). 8 findings (0C + 0H + 6M + 2L) → 6 patch + 1 defer + 1 dismiss.

- [x] [Review][Patch] P2-F1 — Paramètre `<N>` non défini dans `grep -nFA <N>` (MEDIUM, §Haiku guardrails) : ajouter recommandation par défaut « N typiquement 3-5, ajuster selon longueur estimée du bloc patché » [CLAUDE.md:147]
- [x] [Review][Patch] P2-F2 — Critère « pattern architectural » subjectif (MEDIUM, §Haiku guardrails) : ajouter contre-exemple textuel grepable (ordre `app.use()`, séquence d'appels intra-fonction) pour réserver « architectural » aux flux cross-fonction/cross-fichier [CLAUDE.md:148]
- [x] [Review][Patch] P2-F3 — Change Log location implicite (LOW, §Haiku guardrails) : préciser « Change Log de la story (section `### Pass N review`) avec extrait du code lu » [CLAUDE.md:148]
- [x] [Review][Patch] P2-F5 — Bug structurel vs validation métier sans exemples (MEDIUM, §Pattern batch) : ajouter exemples concrets de variant manquant (refactor incomplet : `Rule::NewType` oublié, signature modifiée, type Send par erreur) vs erreur business (amount négatif, currency manquante legacy). Critère décidable explicite : « est-ce que le code compile ? si oui mais match incomplet à cause d'un refactor → bug structurel → AppError::Internal » [CLAUDE.md:178]
- [x] [Review][Patch] P2-F7 — « 3+ tentatives » non-défini (MEDIUM, §Tech debt soupape) : définir 1 tentative = 1 cycle complet `bmad-dev-story` → `bmad-code-review` où fix échoue ou introduit régression [CLAUDE.md:222]
- [x] [Review][Patch] P2-F8 — Contradiction zero carry-forward vs soupape Epic+2 (MEDIUM, §Tech debt) : expliciter que la soupape EST l'exception codifiée à la règle, pas une contradiction. L'item reste tracé (story Epic+2 + label + suivi rétro), distingué d'un report silencieux. Cas spécial Epic N+1 = Debt Closure dédié : soupape Epic+2 reste applicable [CLAUDE.md:227]
- [x] [Review][Defer] P2-F6 — Workflow Project Lead indisponible (MEDIUM, §Tech debt) [CLAUDE.md:220] — deferred, processus opérationnel rare hors scope CLAUDE.md durable. Ajouté à deferred-work.md.

**Dismissed (1)** : P2-F4 — ID business post-persist edge case (MEDIUM) : hypothétique pour Epic non-encore-défini, prématuré ; CAMT.053 est explicitement listé NON-pattern, et pain.001 Epic 11 fournit ID interne client. Pas pertinent v0.1.

## Dev Notes

### Pattern de référence : édit purement documentation

Cette story est **documentation-only** stricte. Aucun code Rust/TS/Svelte n'est touché. Aucun test n'est ajouté ni modifié. Le pattern de référence le plus proche dans l'historique projet est probablement aucun — les stories Epic 7-9 ont toutes touché du code. C'est une **première**, justifiée par la nature de l'Epic 9.5 (Technical Debt Closure, cleanup process).

**Conséquence pratique pour le dev agent** :

- `bmad-dev-story` peut converger en 1 seule passe avec un seul commit `docs(claude): codify 3 patterns (Haiku grep / FailedProposal batch / zero tech debt)`.
- `bmad-code-review` post-dev a peu de surface : cohérence rédactionnelle FR, placement sections correctes, références traçabilité valides (memories existent encore — cf. T6.1). Pas de logique métier à vérifier.
- Le cycle review multi-pass (Sonnet → Haiku → Opus → Sonnet) reste applicable mais convergera vite (effort attendu : 1-2 passes max).

### Pourquoi codifier dans CLAUDE.md plutôt que ailleurs

- **CLAUDE.md** = règles **process projet appliquées par tout agent LLM** (Claude Code, cycle review, dev-story). Chargé automatiquement en contexte à chaque session.
- **architecture.md** = décisions techniques durables (choix lib, schémas DB, contrats API). Pas le bon endroit pour règles de workflow review.
- **docs/** = documentation utilisateur ou archives historiques (cf. `docs/change_request.md` archivé). Pas chargé automatiquement en contexte LLM.
- **Memories user-level** = trace historique des décisions. Persistantes mais user-scoped (pas project-scoped). Ne survivent pas si un autre dev clone le repo.

CLAUDE.md est l'unique emplacement qui combine **(a) chargement automatique en contexte LLM** + **(b) versionning Git + accessibilité tout dev humain ou agent** + **(c) source de vérité projet (vs user-level memories)**.

### Choix éditoriaux tranchés Pass 1 spec validate

**Décision §placement-batch-pattern** (AC #7 + #7bis — **tranchée Pass 1**) : sous-section H3 à l'intérieur de `## Review Iteration Rule` **avec** ajout d'un renvoi cross-section depuis `## Code Quality Rules` (T5.5). Justifications :

1. Le pattern a été détecté en review (Story 8-5b Pass 4 ECH4-1) → contextuellement lié au workflow review.
2. Évite la fragmentation de CLAUDE.md en trop de sections H2 (déjà 8 actuelles).
3. Reste discoverable via grep `FailedProposal`.
4. **Le renvoi depuis `## Code Quality Rules` pallie le déficit de discoverability sémantique** (Pass 1 HIGH-02) : un agent cherchant les conventions HTTP API ne fouillera pas spontanément Review Iteration Rule, mais Code Quality Rules est l'emplacement attendu. Le renvoi est cheap (1 bullet) et évite d'avoir une section H2 dédiée.

**Décision §rotation-order-update** (AC #4 — **tranchée Pass 1**) : harmoniser vers l'ordre `Sonnet → Haiku → Opus → Sonnet` (validé empiriquement Epic 9 retro Insight I1, 3 cycles complets). L'ordre antérieur `Opus → Sonnet → Haiku → Opus` actuellement dans CLAUDE.md `## Review Iteration Rule` reflète une intuition initiale non-validée empiriquement. Patcher l'unique occurrence (T5.4) pour éviter ambiguïté et duplication avec la nouvelle sous-section Haiku.

### Pattern strict insertion-only

Pour minimiser le risque de régression rédactionnelle :

- **Utiliser exclusivement le tool `Edit`** (pas `Write` complet) avec `old_string` ancré sur 2-3 lignes contextuelles existantes pour disambiguation.
- **Une insertion par Edit** (pas batched). Si l'Edit échoue (non-unique match), reformuler l'ancre avec plus de contexte.
- **Vérifier après chaque insertion** : `Read CLAUDE.md offset=<ligne autour>` pour confirmer placement.
- **Pas de `replace_all=true`** — toutes les insertions sont uniques par construction (ancres ciblées sur lignes spécifiques).

### Risques identifiés

| # | Risque | Mitigation |
|---|---|---|
| R1 | Dérive éditoriale style (anglais accidentel, emoji, MAJUSCULES erratiques) | Spot-check T6.5 + comparer à sections existantes (style sobre, FR, MAJUSCULES limitées à mots-clés impératifs `DOIT`, pas d'emoji) |
| R2 | Référence externe cassée (memory renommée, file path obsolète) | T6.1 + vérifier `ls /home/gcorbaz/.claude/projects/.../memory/` post-édition que les 2 memories citées existent encore (`feedback_haiku_review_diff_combined.md`, `feedback_zero_tech_debt_carryforward.md`) |
| R3 | Insertion 2 (FailedProposal batch) ratée car ancre Haiku non-unique (deux insertions séquentielles) | Lire après T5.1, re-construire ancre T5.2 avec contexte frais (dernières 2-3 lignes Haiku qui viennent d'être insérées) |
| R4 | Décision §rotation-order-update divergente avec Guy → Change Request | **Tranchée Pass 1 Sonnet 4.6** : harmonisation `Sonnet → Haiku → Opus → Sonnet` (T5.4). Risque résiduel : si Guy préfère un autre ordre au moment dev-story, revert T5.4 et conserver l'ordre actuel CLAUDE.md `Opus → Sonnet → Haiku → Opus` (1 ligne diff = trivial à revert). |
| R5 | Verbosité explosive (> 200 lignes ajoutées) → CLAUDE.md devient illisible | T6.4 wc -l + AC #15 cap 180 lignes. Si dépassé : refactoriser en `docs/process/` linked depuis CLAUDE.md (anti-pattern à éviter) |
| R6 | KF #91 (DropdownMenu a11y) résolu en Story 9-5-1 entre temps → mention obsolète dans Tech debt section | T4.6 mentionne KFs en général (label `v0.2-milestone`), pas KF #91 spécifique. Aucune dépendance temporelle inter-stories. |

### Project Structure Notes

- Fichier édité : `/home/gcorbaz/Synology/devel/kesh/CLAUDE.md` (racine repo, **un seul fichier**).
- Pas de création de fichier nouveau.
- Pas de suppression de fichier ni de section existante.
- Le story file lui-même `/home/gcorbaz/Synology/devel/kesh/_bmad-output/implementation-artifacts/9-5-3-process-codification-claude-md.md` (créé par `bmad-create-story`).
- Sprint-status `/home/gcorbaz/Synology/devel/kesh/_bmad-output/implementation-artifacts/sprint-status.yaml` mis à jour par le workflow.

### Testing standards summary

- **Pas de tests** ajoutés (story documentation-only).
- **Baselines préservées par construction** : aucun fichier `.rs` / `.ts` / `.svelte` édité → `cargo test --workspace` + `npm run test:unit` + `npm run test:e2e` doivent rendre **strictement identiques** à pré-Story 9-5-3 (sanity-check optionnel post-implementation : 1 cargo test run pour confirmer).
- **Test Locally First** §"Quand sauter" : commits doc-only sont explicitement exemptés → pas de check CI local requis. Pré-push : juste vérifier que le diff `git diff CLAUDE.md` reflète bien insertion-only (sauf AC #4 si harmonisation).

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-3](_bmad-output/planning-artifacts/epic-9-5.md) — spec parent epic
- [Source: _bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md#C1](_bmad-output/implementation-artifacts/epic-9-retro-2026-05-17.md) — challenge C1 « Memory feedback_haiku_review_diff_combined validée 2× Epic 9 »
- [Source: _bmad-output/implementation-artifacts/epic-8-retro-2026-05-14.md#I2](_bmad-output/implementation-artifacts/epic-8-retro-2026-05-14.md) — insight I2 « Pattern accept_one_X strict (FailedProposal per-proposal) inviolable »
- [Source: _bmad-output/implementation-artifacts/epic-8-retro-2026-05-14.md#PROCESS](_bmad-output/implementation-artifacts/epic-8-retro-2026-05-14.md) — Action item #6 retro Epic 8 marquée non-codifiée (la dette que cette story résout)
- [Source: _bmad-output/implementation-artifacts/8-5b-reconciliation-rules-engine.md] — ground-truth canonique du pattern `FailedProposal` (route `accept_one_rule` Pass 4 ECH4-1)
- [Source: CLAUDE.md#Review-Iteration-Rule] — section H2 cible pour insertions 1 + 2 (sous-sections H3)
- [Source: CLAUDE.md#Issue-Tracking-Rule] — section H2 immédiatement après la nouvelle insertion 3 (Tech debt management)
- [Source: feedback_haiku_review_diff_combined.md] — memory à promouvoir (NE PAS supprimer post-codification, cohérent AC §Hors scope)
- [Source: feedback_zero_tech_debt_carryforward.md] — memory à promouvoir (idem)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.7 (1M context) — dev-story single-pass (story documentation-only, faible surface).

### Debug Log References

Aucun debug nécessaire — édits Markdown ciblés via Edit tool avec ancres uniques pré-vérifiées (T1).

### Completion Notes List

- **CLAUDE.md** : 252 → 324 lignes (+72 lignes ajoutées). En-dessous de la cible 120-150 (AC #15) car rédaction concise privilégiée. Hard cap 180 respecté.
- **5 édits appliqués dans l'ordre** :
  1. T5.4 — Patch rotation order ligne 121 : `Opus → Sonnet → Haiku → Opus` → `Sonnet → Haiku → Opus → Sonnet` (+ mention « validé empiriquement Epic 9 retrospective Insight I1 sur 3 cycles complets »).
  2. T5.5 — Bullet renvoi cross-section `## Code Quality Rules` après `**E2E Testing**` : « **Batch API conventions** — Pour les endpoints batch... cf. §Pattern batch... » (1 bullet ajouté).
  3. T5.3 — Section H2 `## Tech debt management — zero carry-forward policy` insérée entre `### Règle de splitting préventif` et `## Issue Tracking Rule` (27 lignes, 4 sous-sections H3).
  4. T5.1+T5.2 combinés — 2 sous-sections H3 (`### Haiku-specific guardrails — grep ground-truth obligatoire` + `### Pattern batch — FailedProposal per-proposal`) insérées entre `**Exception** : si un finding MEDIUM+` et `### Règle de splitting préventif` (44 lignes, 17 + 27).
- **Validation T6 OK** : 4 grep checks réussis (Batch API conventions = 1, Sonnet → Haiku → Opus → Sonnet = 1, Opus → Sonnet → Haiku → Opus = 0 confirmant patch, sections H2/H3 en place). Pas d'emoji ajouté (1 emoji pré-existant ligne 254 hors scope). Les 2 memories citées toujours présentes.
- **Test Locally First exemption appliquée** (T7.2) : commit doc-only, pas de `cargo test --workspace` ni `npm run test:e2e` exécutés (cohérent §"Quand sauter" CLAUDE.md, exemption explicite).
- **Décisions tranchées en spec validate respectées** :
  - §rotation-order-update : harmonisation `Sonnet → Haiku → Opus → Sonnet` appliquée (T5.4).
  - §placement-batch-pattern : H3 sous Review Iteration Rule + renvoi cross-section depuis Code Quality Rules (T5.5).
- **Aucune régression introduite** : édits insertion-only sauf 1 ligne sur place (rotation order) + 1 bullet ajouté (renvoi). Toutes les sections pré-existantes intactes dans leur contenu de fond.

### File List

- `CLAUDE.md` — modifié (3 sections insérées + 1 bullet renvoi cross-section + 1 ligne rotation harmonisée sur place ; 252 → 324 lignes, +72).
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — modifié (status `9-5-3` : ready-for-dev → in-progress → review).
- `_bmad-output/implementation-artifacts/9-5-3-process-codification-claude-md.md` — modifié (cocher T1-T8, Dev Agent Record peuplé, Change Log dev-story ajouté, Status → review).

## Change Log

### Pass 1 spec validate — 2026-05-18, Sonnet 4.6 (subagent contexte frais)

**Verdict trend** : 1 CRITICAL + 2 HIGH + 3 MEDIUM + 3 LOW = 9 findings (Convergence : NON).

**Patches appliqués (9/9 findings)** :

1. **CRITICAL-01** — `FailedProposal` ground-truth : la spec décrivait `proposal_index: usize` + `error_code: &'static str`. Le code réel (`crates/kesh-api/src/routes/reconciliation.rs:152-156` grep ground-truth Pass 1) utilise `bank_transaction_id: i64` + `error_code: String` + `details: Option<serde_json::Value>`. Patch : AC #6 + T3.4 réécrits avec signature canonique + anti-pattern explicite « NE PAS utiliser index positionnel ».
2. **HIGH-01** — Ancre d'insertion 1 (sous-section Haiku) ambiguë entre 2 positions. Patch : T1.2 + T5.1 précisent l'ancre = bloc `**Exception** : si un finding MEDIUM+...` (dernière ligne contenu Review Iteration Rule avant H3 Splitting), pas le bloc Boucle automatique qui fragmenterait « Boucle → Cette règle s'applique → Exception » bloc logique.
3. **HIGH-02** — Placement H3 `### Pattern batch — FailedProposal per-proposal` sous `## Review Iteration Rule` crée déficit discoverability sémantique (un agent cherchant conventions HTTP API ne fouille pas Review Iteration Rule). Patch : ajout AC #7bis + T5.5 → renvoi cross-section depuis `## Code Quality Rules` (1 bullet « Batch API conventions »).
4. **MEDIUM-01** — `error_code: &'static str` corrigé en `error_code: String` (cohérent ground-truth + serde sans lifetime).
5. **MEDIUM-02** — AC #4 conditionnel (`soit conservée, soit harmonisée`) tranché définitivement en faveur de l'harmonisation `Sonnet → Haiku → Opus → Sonnet` (validé Epic 9 Insight I1).
6. **MEDIUM-03** — Path `crates/kesh-api/src/routes/reconciliation.rs` dans T3.6 maintenu pour le dev agent, mais explicitement exclu du texte CLAUDE.md cible (risque de référence cassée si refactoré).
7. **LOW-01** — Référence à « ligne 111 / 152 actuelle » remplacée par « titre canonique de section » dans AC #9 (références aux numéros de ligne fragiles).
8. **LOW-02** — Référence traçabilité enrichie : Stories 8-5a-bis Pass 2 + 9-2b Pass 2 (4 hallucinations Haiku réfutées au total, pas 2).
9. **LOW-03** — AC #13 style absolu paths `crates/...` relâché : style CLAUDE.md existant n'utilise pas ce pattern, on n'invente pas une règle inexistante.

**Décisions ouvertes tranchées Pass 1** :
- §rotation-order-update : **harmoniser** vers `Sonnet → Haiku → Opus → Sonnet` (T5.4).
- §placement-batch-pattern : **H3 sous Review Iteration Rule** + renvoi cross-section depuis Code Quality Rules (AC #7bis + T5.5).

**Trend cumulé** : Pass 1 : 1C + 2H + 3M + 3L → patches appliqués → Pass 2 Haiku 4.5 attendue (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`).

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé, contexte frais — story créée par Opus 4.7 dans la session orchestratrice).

### Pass 2 spec validate — 2026-05-18, Haiku 4.5 (subagent contexte frais)

**Verdict trend** : 1 CRITICAL + 0 HIGH + 2 MEDIUM + 2 LOW = 5 findings (Convergence : NON — présence CRITICAL régression Pass 1).

**Discipline grep ground-truth obligatoire** appliquée par Haiku conformément à la mémoire `feedback_haiku_review_diff_combined` (exactement la règle que la story codifie). Le CRITICAL trouvé est confirmé par grep, pas une hallucination.

**Patches appliqués (2/5 findings — 3 non-blockers)** :

1. **CRITICAL-01 (régression Pass 1)** — AC #6 ligne 54 (énoncé de la règle) disait encore `FailedProposal { proposal_index, error_code, details }` alors que le bloc « Champs obligatoires » 6 lignes plus bas l'avait corrigé en `bank_transaction_id` + anti-pattern explicite. Régression introduite par Pass 1 (patch incomplet cross-bloc). **Patch Pass 2** : ligne 54 réécrite pour omettre les noms de champs et renvoyer au bloc « Champs obligatoires » canonique (« signature détaillée bloc Champs obligatoires ci-dessous »).
2. **MEDIUM-02** — AC #11 ambiguïté « diff insertion-only sauf AC #4 ». **Patch Pass 2** : AC #11 réécrit pour expliciter « diff **de CLAUDE.md** » + énumération des modifs attendues (3 nouvelles sections insertion-only + 1 bullet renvoi cross-section T5.5 + 1 modification ligne sur place T5.4 harmonisation rotation).

**Findings reclassés sans patch (3/5)** :

3. **MEDIUM-01** — Doctrine rotation `Opus → Sonnet → Haiku → Opus` mentionnée dans le story file lui-même (lignes 27 + 48) vs nouvelle doctrine `Sonnet → Haiku → Opus → Sonnet` tranchée Pass 1 (ligne 199). Reclassé **non-blocker** : la spec est un document historique qui décrit l'état CLAUDE.md pré-édit puis prescrit le changement (cohérence narrative correcte). Aucune ambiguïté pour le dev agent. **Aucun patch.**
4. **LOW-01** — Style descriptif vs imperatif ligne 40. Haiku reclasse lui-même en non-finding. **Aucun patch.**
5. **LOW-02** — Spéculation Epic 11 `payment_id`. Pédagogique, utile pour montrer généricité du pattern. **Aucun patch.**

**Régressions Pass 1 introduites** : OUI (CRITICAL-01 — patch Sonnet 4.6 incomplet cross-bloc AC #6 vs T3.4). Détectée et fixée Pass 2 Haiku grâce à la discipline grep ground-truth.

**Trend cumulé** : Pass 1 : 1C + 2H + 3M + 3L → Pass 2 : 1C + 0H + 2M + 2L → 2 patches appliqués (CRITICAL-01 + MEDIUM-02) → Pass 3 Opus 4.7 attendue (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`).

**Modèle Pass 2** : Haiku 4.5 (subagent isolé, contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée).

### Pass 3 spec validate — 2026-05-18, Opus 4.7 (subagent contexte frais, anti-rubber-stamp)

**Verdict trend** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 4 LOW = 4 findings (**Convergence : OUI** — critère d'arrêt CLAUDE.md atteint).

**Discipline ground-truth obligatoire** appliquée pour vérification de l'inspection réelle (anti-rubber-stamp Opus historique cf. retro Epic 9 Insight I1). Opus a re-vérifié :
- Signature `FailedProposal` ground-truth `crates/kesh-api/src/routes/reconciliation.rs:152-156` ✓
- Ancres CLAUDE.md insertion 1/2/3 (lignes 135 Exception, 137 Splitting, 152 Issue Tracking) ✓
- Tous les Tx (T5.1 → T5.5) existent dans Tasks/Subtasks ✓
- Pattern `unreachable!() → tracing::error + AppError::Internal` ground-truth Story 8-5b Pass 1 patch ✓
- Régressions Pass 2 (AC #6 ligne 54 + AC #11) **non régressives** ✓
- Memory `feedback_haiku_review_diff_combined` cohérente avec AC #2 ✓

**Patches appliqués (4/4 LOW polish — non-blockers convergence)** :

1. **LOW-01** — R4 table Risques anachronique : « Pass 1 Sonnet 4.6 tranchera » au futur → réécrit en passé « Tranchée Pass 1 Sonnet 4.6 » + ajout risque résiduel revert trivial si Guy diverge.
2. **LOW-02** — AC #6 « Réutilisation prévue » + T3.7 : suppression de la mention Epic 12 CAMT.053 (import raw, ne suit pas accept_batch pattern). Reformulé : « Epic 11 + tout endpoint futur retournant `{ accepted, failed }` » + note explicative CAMT.053 non-applicable.
3. **LOW-03** — AC #15 cap lignes + T4.8 borne haute Tech debt : alignement math 60 + 70 + 50 = 180 hard cap (au lieu de 60 + 70 + 80 = 210 max théorique). Cible précisée ~120-150.
4. **LOW-04** — AC #10 edge case classification KF labellée `v0.2-milestone` sans story Epic créée : précisé que le label tient lieu de planification implicite au cycle v0.2 → qualifie en catégorie B.

**Régressions Pass 2 introduites** : NON (vérifié par Opus inspection profonde).

**Trend cumulé final** : Pass 1 : 1C + 2H + 3M + 3L (9 patches) → Pass 2 : 1C + 0H + 2M + 2L (2 patches + 3 reclassés) → Pass 3 : 0C + 0H + 0M + 4L (4 patches polish) → **Convergence atteinte après 3 passes** (sous la limite CLAUDE.md de 8 passes max).

**Modèles cycle** : Sonnet 4.6 → Haiku 4.5 → Opus 4.7 (rotation CLAUDE.md respectée, chaque pass LLM différent + contexte frais).

**Story status final** : `ready-for-dev` confirmé définitif. Prête pour `bmad-dev-story 9-5-3`.

### Dev-story — 2026-05-18, Opus 4.7 (single-pass)

**Implémentation** : 5 édits appliqués sur `CLAUDE.md` en séquence — T5.4 (rotation order patch) → T5.5 (bullet renvoi) → T5.3 (H2 Tech debt) → T5.1+T5.2 combinés (2 sous-sections H3 Haiku + FailedProposal). Ordre choisi pour minimiser le risque d'ancres décalées par les edits précédents (chaque édit ciblait une ancre unique pré-vérifiée).

**Métriques** :
- CLAUDE.md : 252 → 324 lignes (+72 ajoutées). Sous la cible 120-150 lignes (AC #15) — rédaction concise privilégiée à la verbosité.
- Sections ajoutées : `### Haiku-specific guardrails — grep ground-truth obligatoire` (17 lignes), `### Pattern batch — FailedProposal per-proposal` (27 lignes), `## Tech debt management — zero carry-forward policy` (27 lignes avec ses 4 sous-H3 Triage / Critical path / Pattern Epic dédié cleanup / Distinction au triage).
- Modification sur place : ligne 121 rotation order (1 ligne diff).
- Bullet renvoi : 1 ligne dans `## Code Quality Rules`.

**Validation T6** : tous les grep checks passent (`Batch API conventions` = 1, nouveau `Sonnet → Haiku → Opus → Sonnet` = 1, ancien `Opus → Sonnet → Haiku → Opus` = 0, sections H2 + H3 toutes en place). Pas d'emoji ajouté. Memories `feedback_haiku_review_diff_combined` + `feedback_zero_tech_debt_carryforward` toujours présentes (cohérent AC §Hors scope — promotion vers CLAUDE.md ne supprime pas les memories user-level).

**Test Locally First** : exemption §"Quand sauter" appliquée — story doc-only stricte (`CLAUDE.md` + `sprint-status.yaml` + story file uniquement, zéro édit de code Rust/TS/Svelte). `cargo test --workspace` + `npm run test:unit` + `npm run test:e2e` baselines préservées par construction.

**Story status** : `in-progress` → `review`. Prête pour `bmad-code-review 9-5-3` (Sonnet 4.6 ou Haiku 4.5 recommandé — Opus déjà utilisé pour dev-story, rotation CLAUDE.md cohérence).

**Modèle dev-story** : Claude Opus 4.7 (1M context, session orchestratrice — pas de subagent isolation nécessaire pour une story doc-only à faible surface).

### Pass 1 code-review — 2026-05-18, Sonnet 4.6 × 3 reviewers parallèles

**Setup** : 3 subagents Sonnet 4.6 parallèles (Blind Hunter + Edge Case Hunter + Acceptance Auditor), contextes frais isolés. Diff cible : commit `7456bdc` (dev unique). Discipline « diff unique » respectée (cf. règle codifiée Haiku-specific guardrails — un seul commit, pas une séquence multi-commit). Spec context fourni à Acceptance Auditor.

**Verdict trend** : 0 CRITICAL + 6 HIGH + 8 MEDIUM + 8 LOW = 22 findings (Convergence Pass 1 : NON).

**Triage** : 10 patch + 1 defer + 11 dismiss.

**Patches appliqués (10/10 ; sur CLAUDE.md, 324 → 342 lignes, +18)** :

1. **E11 (HIGH)** — Contradiction sémantique `AppError::Internal` clarifiée : variant manquant = bug structurel runtime = cas d'usage de l'exception globale ligne 500, pas une violation du pattern per-proposal. Le pattern `FailedProposal` reste inviolable pour erreurs métier ordinaires.
2. **E1 (HIGH)** — Pattern grep `-nF` (fixed-string) obligatoire pour éviter faux-positifs sur métacaractères regex (`.`, `*`, `[`, etc.).
3. **E2 (HIGH)** — Pattern multi-ligne : préciser que `grep -n` est line-by-line → choisir ligne représentative discriminante ou utiliser `grep -nFA <N>`.
4. **E3 (MEDIUM)** — Règle étendue à « présence d'un anti-pattern non-corrigé » (pas seulement absence d'un fix).
5. **E4 (MEDIUM)** — Finding architectural sans pattern grepable : vérification manuelle par `Read` direct, documentée dans Change Log.
6. **E12 (HIGH)** — Triage hors fenêtre rétrospective : si dette A découverte en cours d'Epic N+1, arbitrage Project Lead selon sévérité (critique = fix immédiat dans Epic en cours ; non-critique = report kickoff Epic N+2).
7. **E13 (HIGH)** — Conflit labels v0.2-milestone vs gate v0.1 explicite : gate v0.1 prime → reste en catégorie A jusqu'à fix ou levée explicite du gate.
8. **E15 (MEDIUM)** — Soupape « item A résistant 3+ tentatives » : reclassement exceptionnel en B avec justification + Epic+2 + label GitHub + suivi rétro spécifique.
9. **F1 (LOW)** — Définition Catégorie B refactorée : alternative « story dédiée OU label v0.2-milestone » intégrée dans la définition (vs exception apparente posée séparément).
10. **F6 (LOW)** — « Cause root » → « Cause racine » (FR cohérence).

**Defer (1)** : E14 (MEDIUM) — mécanisme de réévaluation périodique des stories de remédiation B bloquées/annulées. Ajouté à `_bmad-output/implementation-artifacts/deferred-work.md` (« Deferred from: code review of 9-5-3-process-codification-claude-md (2026-05-18) »). Processus operational hors scope CLAUDE.md durable.

**Dismiss (11)** : F2/F3/F4/F5 (4 LOW Blind nits ou observations non-actionnables), E5/E10/E16 (3 méta-observations valides mais complexifier le texte serait pire), E6/E7/E8/E9 (4 edge cases batch — endpoint 3-buckets, doublons identifiant, batch vide, batch 100% fail — relèvent de la spec Epic 11 future, pas de la règle CLAUDE.md durable).

**Trend cumulé** : Pass 1 (Sonnet × 3 reviewers) : 0C + 6H + 8M + 8L → 10 patches + 1 defer + 11 dismiss → Pass 2 Haiku 4.5 attendue (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`, LLM différent passe précédente respectée).

**Modèles Pass 1** : 3 × Sonnet 4.6 subagents isolés contextes frais (Blind Hunter sans context, Edge Case Hunter avec project access, Acceptance Auditor avec spec + project).

### Pass 2 code-review — 2026-05-18, Haiku 4.5 × 2 reviewers parallèles

**Setup** : 2 subagents Haiku 4.5 parallèles (Blind Hunter + Edge Case Hunter, contextes frais isolés). Acceptance Auditor non-relancé (Pass 1 a confirmé 16/16 ACs et les patches Pass 1 sont des clarifications intra-AC). Diff cible : commit `be1dfce` (Pass 1 patches uniquement, **diff unique** respectant la règle Haiku indexing codifiée — test du pudding réussi).

**Discipline grep ground-truth obligatoire** appliquée par les 2 Haiku conformément à la règle codifiée. Blind Hunter a documenté 10 vérifications grep. Aucune hallucination détectée.

**Verdict trend** : 0 CRITICAL + 0 HIGH + 6 MEDIUM + 2 LOW = 8 findings (Convergence Pass 2 : NON — 6 MEDIUM résiduels).

**Triage Pass 2** : 6 patch + 1 defer + 1 dismiss.

**Patches appliqués Pass 2 (6/6 ; CLAUDE.md 342 → 344 lignes, +2 net après reformulations)** :

1. **P2-F1 (MEDIUM)** — Paramètre `<N>` non défini dans `grep -nFA <N>` : ajouté « N typiquement 3-5, ajuster selon longueur estimée du bloc patché ».
2. **P2-F2 (MEDIUM)** — Critère « pattern architectural » subjectif : ajouté contre-exemple textuel grepable (`app.use(...)` middleware order). Réservé « architectural » aux flux cross-fonction/cross-fichier sans pattern unique discriminant.
3. **P2-F3 (LOW)** — Change Log location précisée : « Change Log de la story (section `### Pass N review`) avec extrait du code lu ».
4. **P2-F5 (MEDIUM)** — Bug structurel vs validation métier : ajouté exemples concrets de variant manquant (`Rule::NewType` oublié dans match post-refactor, signature modifiée, type Send par erreur) vs erreur business (amount négatif, currency legacy). Critère décidable explicite : « est-ce que le code compile ? si oui mais match incomplet à cause d'un refactor → bug structurel → AppError::Internal ».
5. **P2-F7 (MEDIUM)** — « 3+ tentatives » défini formellement : 1 tentative = 1 cycle complet `bmad-dev-story` → `bmad-code-review` où le fix échoue à résoudre l'item OU introduit régression sur autres baselines.
6. **P2-F8 (MEDIUM)** — Contradiction zero carry-forward vs soupape Epic+2 explicitée : « soupape EST l'exception codifiée à la règle, pas une contradiction. L'item reste tracé (story Epic+2 + label + suivi rétro), distingué d'un report silencieux. Cas spécial Epic N+1 = Debt Closure : soupape Epic+2 reste applicable ».

**Defer Pass 2 (1)** : P2-F6 (MEDIUM) — workflow Project Lead indisponible. Ajouté à `deferred-work.md` (processus opérationnel rare hors scope CLAUDE.md durable).

**Dismiss Pass 2 (1)** : P2-F4 (MEDIUM) — ID business post-persist edge case hypothétique pour Epic non-encore-défini, prématuré.

**Trend cumulé** : Pass 1 (Sonnet × 3) 0C+6H+8M+8L → 10 patches → Pass 2 (Haiku × 2) 0C+0H+6M+2L → 6 patches → Pass 3 Opus 4.7 attendue (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`, LLM différent passe précédente respectée).

**Modèles Pass 2** : 2 × Haiku 4.5 subagents isolés contextes frais. **Discipline grep ground-truth Haiku** respectée (codifiée par cette story elle-même — test du pudding réussi).
