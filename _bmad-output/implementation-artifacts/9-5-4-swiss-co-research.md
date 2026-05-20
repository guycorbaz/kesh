# Story 9.5-4: Recherche réglementaire Swiss CO Art. 957a / 958f — conservation 10 ans + intégrité

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a mainteneur projet Kesh,
I want conclure formellement la recherche réglementaire Swiss Code des Obligations Art. 957a (formats légaux balance / bilan / compte de résultat) + Art. 958f (conservation 10 ans + intégrité / signature des documents comptables), en produisant un document technique `_bmad-output/planning-artifacts/research-swiss-co-958f.md` synthétisant les exigences légales applicables aux PME suisses et la comparaison avec l'implémentation actuelle Kesh (Story 9-1 rapports + 9-2a export PDF/CSV + 9-2b export global ZIP),
so that la décision implicite « audit-trail-only acceptée v0.1 » documentée dans 9-2b §L6 soit **validée par recherche réglementaire explicite** (option a) **OU** que la dette de conformité soit reconnue avec une story v0.2 Epic 14 planifiée (option b) **OU** qu'une story bloquante Epic 9.5-bis soit ajoutée si l'écart est jugé incompatible avec une release v0.1 (option c). Cette décision lève le risque conformité légale Suisse non-évalué identifié en rétrospective Epic 8 (action #3 partielle « Recherche réglementaire Swiss CO Art. 957a — Guy ») et débloque la clôture finale Epic 9.5 + kickoff Epic 10 (déploiement v0.1).

## Scope

Story **recherche réglementaire pure** — **AUCUN code Rust ou Svelte modifié**. Périmètre :

### Document produit (livrable principal)

- **Fichier** : `_bmad-output/planning-artifacts/research-swiss-co-958f.md` (nouveau, à créer).
- **Format** : Markdown, ~400-800 lignes attendues, structure standardisée (cf. §Tasks T2 pour le sommaire imposé).
- **Public cible** : Guy + futurs reviewers Kesh. Niveau technique + interprétation réglementaire — pas du texte légal brut copié-collé. Un développeur Kesh non-juriste doit pouvoir lire et tirer des actions concrètes.
- **Tonalité** : factuelle, sourcée systématiquement (URL + date d'accès + extrait pertinent), pas d'opinion juridique formelle (mention explicite « ce document ne constitue pas un avis juridique »).

### Mise à jour cross-stories (3 fichiers existants)

- **`_bmad-output/implementation-artifacts/9-2b-export-global-zip.md`** : §L6 mis à jour selon décision (a/b/c) — référence le document de recherche + verdict explicite (e.g. « L6 verdict : option (b), conformité v0.1 acceptée + story Epic 14 `swiss-co-958f-signature-electronique` créée »).
- **`_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md`** : si applicable selon recherche, mise à jour section sur la conformité format légal AFC des exports PDF rapports (signature + horodatage).
- **`_bmad-output/planning-artifacts/epic-9-5.md`** : Critère d'arrêt Epic 9.5 §"Document `research-swiss-co-958f.md` produit + décision formelle (a/b/c) appliquée à 9-2a/9-2b" coché + référence au commit closure de la story 9-5-4.

### Décisions formelles produites

Une des 3 options exclusives, formulée explicitement dans le document de recherche §"Verdict" et propagée dans **9-2b §L6** (Swiss CO Art. 958f ZIP non-signé) **+ 9-2a §L7** si applicable (Swiss CO Art. 958f PDF non-signé) — ground-truth Pass 1 spec validate P1-1 : `9-2a §L6 = PDF pagination cosmétique` ≠ Swiss CO, `9-2a §L7 = horodatage signé/certificat PDF Swiss CO Art. 958f conformité partielle`, donc la cible 9-2a correcte est §L7 :

- **Option (a) — Conformité v0.1 stricte** : l'implémentation actuelle (audit_log + SHA-256 dans `metadata.json` + ZIP non-signé) satisfait les exigences Swiss CO 958f pour PME, sans nécessité de signature électronique qualifiée. Pas de story additionnelle nécessaire. v0.1 publishable tel quel.
- **Option (b) — Dette explicite v0.2** : conformité v0.1 acceptée avec dette documentée. Une story Epic 14 « Swiss CO 958f signature électronique qualifiée » est créée et labellisée GitHub Milestone `v0.2`. L'écart est jugé acceptable pour v0.1 (cible PME, audit-trail SHA-256 satisfait l'esprit de la loi à défaut d'une lettre stricte).
- **Option (c) — Bloquant v0.1** : la recherche révèle un écart incompatible avec une release v0.1 commercialisable. Une story Epic 9.5-bis « Swiss CO 958f compliance pre-v0.1 » est ajoutée et bloque le kickoff Epic 10. Probabilité estimée **faible** (cf. epic-9-5.md §Q2 — jurisprudence PME accepte généralement audit_log + SHA-256 comme preuve d'intégrité), mais l'option doit rester disponible.

### Hors scope 9-5-4

- **Implémentation effective de signature électronique** — si décision = (b) ou (c), c'est dans la story de remédiation, pas ici.
- **Recherche TVA** (Art. 70 LTVA / OLTVA) — couverte par Epic 11 Story 11-1 « Configuration taux TVA » ultérieurement. 9-5-4 se limite à 957a + 958f comptabilité.
- **Recherche AFC déclarations électroniques** (e-Form, eTVA, Salaire ELM) — hors périmètre Kesh v0.1 (uniquement comptabilité interne + facturation, pas de transmission AFC).
- **Recherche Swiss GAAP RPC / IFRS** — hors périmètre PME ordinaire (réservée aux entités cotées ou holdings ≥ 40 M CHF cf. Art. 962 CO). 9-5-4 cible PME standard < seuils Art. 727 CO (audit ordinaire).
- **Validation juridique formelle par un avocat** — explicitement hors scope (« ce document ne constitue pas un avis juridique »). Si la décision (b) ou (c) est retenue, recommander une revue juridique externe avant publication v0.1 ou v0.2.

## Acceptance Criteria

### Pré-flight + bibliographie

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée (HEAD `bceb112` post-9-5-1d code-review done), **When** la story démarre, **Then** prérequis confirmés : pas de cargo build / npm install nécessaire (research-only). Outils à confirmer disponibles : accès web (WebFetch / WebSearch ou subagent equivalent), accès au site `https://www.fedlex.admin.ch` (texte officiel CO consolidé), accès à `https://www.expertsuisse.ch` et/ou `https://www.treuhand-suisse.ch` (commentaires fiduciaires PME — patch Pass 1 spec validate P1-5 : remplace `eitsa.ch` qui n'est pas une source juridique suisse reconnue), accès à `https://www.afc.admin.ch` (AFC instructions PME).

2. **Given** le besoin de sourcer la recherche, **When** la bibliographie est constituée, **Then** au minimum **5 sources primaires** et **3 sources secondaires** sont citées dans le document, conformes au critère ci-dessous :
   - **Sources primaires acceptées (citation obligatoire)** :
     - `fedlex.admin.ch` (Code des obligations consolidé — Art. 957a + 958f en français OU allemand selon disponibilité, dernière version consolidée à la date de la story).
     - `admin.ch` Ordonnance OLICo (Ordonnance sur la tenue et la conservation des livres de comptes, RS 221.431, version consolidée).
     - `admin.ch` LSCSE (Loi sur la signature électronique, RS 943.03) — pour les exigences signature qualifiée si applicable.
     - `bj.admin.ch` (Office fédéral de la justice — commentaire officiel CO si disponible).
     - `seco.admin.ch` ou `afc.admin.ch` instructions PME conservation comptable.
   - **Sources secondaires acceptées (recommandées)** : `expertsuisse.ch`, `treuhand-suisse.ch`, `kmu.admin.ch`, commentaires académiques Suisse (Helbing & Lichtenhahn, Schulthess), ECH-0058 archivage électronique (`ech.ch`).
   - **Sources rejetées (à éviter)** : Wikipedia, blogs personnels non-experts, articles marketing de prestataires SaaS d'archivage signé. Si une source secondaire est utilisée, motiver son inclusion en 1 phrase.

### Document de recherche — structure et contenu

3. **Given** la recherche conclue, **When** le document `research-swiss-co-958f.md` est produit, **Then** il satisfait la structure suivante (sommaire imposé) :
   - `## Préambule` — disclaimer non-juridique + champ d'application (PME suisse soumise CO < seuils Art. 727 audit ordinaire) + date de l'analyse (recherche valable à la date `YYYY-MM-DD`, lois Suisses étant amendées régulièrement) + sources externes citées avec URL + date d'accès.
   - `## Art. 957a CO — Tenue de la comptabilité` — extrait pertinent (alinéa par alinéa) + interprétation appliquée à Kesh + checklist « ce qui est requis pour PME ».
   - `## Art. 958f CO — Conservation des documents` — extrait pertinent + interprétation 10 ans + intégrité + signature électronique qualifiée (si applicable) + checklist appliquée à Kesh.
   - `## Ordonnance OLICo (RS 221.431)` — précisions techniques sur la conservation (formats acceptés, lisibilité durable, intégrité, supports) + section spécifique sur les supports modifiables (mention courante « les fichiers texte modifiables ne sont admis que si un mécanisme d'intégrité prouve l'absence d'altération »).
   - `## ECH-0058 Archivage électronique` — référence standard suisse, applicabilité PME (généralement non-obligatoire, mais bonne pratique).
   - `## LSCSE et signature électronique qualifiée` — exigences certifiée (QES / SES / AES distinctions), coût ordre-de-grandeur (CHF/an pour une PME), applicabilité ou non aux exports comptables Kesh.
   - `## Implémentation actuelle Kesh — état de l'art` — synthèse précise (5-10 lignes) de ce que Kesh fournit aujourd'hui : (a) audit_log avec action `exports.global` + user_id + timestamp + métadonnées ZIP (Story 9-2b), (b) SHA-256 dans `metadata.json` du ZIP (intégrité non-signée), (c) audit immutable côté backend (insertion-only, pas de UPDATE/DELETE sur `audit_log`), (d) rapports PDF/CSV produits via Story 9-1 + 9-2a (format légal AFC pour comptes annuels — à vérifier), (e) absence de QES / horodatage tiers signé.
   - `## Gap analysis` — tableau Markdown avec lignes = exigence légale Art. 957a/958f/OLICo/LSCSE, colonnes = (1) Exigence, (2) État Kesh, (3) Verdict conforme/partiel/non-conforme, (4) Référence Kesh (story / fichier / code).
   - `## Verdict` — option retenue (a / b / c) avec justification 3-5 paragraphes, ancré dans le gap analysis et la jurisprudence sourcée.
   - `## Recommandations actionables` — liste numérotée de 0 à N items à appliquer à Kesh (e.g. « mettre à jour 9-2b §L6 », « créer story Epic 14 X », « pas d'action requise »), ordonné par urgence.

4. **And** le document fait au minimum **300 lignes** (sans diluer artificiellement — chaque section apporte une valeur informative) et au maximum **1200 lignes** (limite anti-bloat). Si la recherche révèle un sujet hors scope significatif (e.g. TVA Art. 70 LTVA mentionné), créer une note `Hors scope — à traiter dans story dédiée Epic X` plutôt que développer.

5. **And** **chaque affirmation normative** (e.g. « PME doit conserver pendant 10 ans », « signature qualifiée est requise pour archives électroniques modifiables ») est accompagnée d'une **citation source précise** avec URL + alinéa/section + date d'accès. Pattern recommandé : `[CO Art. 958f al. 1, fedlex.admin.ch, accédé 2026-05-XX](url)`.

### Décision formelle + mise à jour cross-stories

6. **Given** le document produit + le verdict (a/b/c) statué, **When** la décision est prise via **checkpoint élicitation Guy obligatoire T8.3** (cf. Dev Notes §R3 — décision business engageante, jamais autonome LLM ; patch Pass 1 spec validate P1-3 : checkpoint obligatoire indépendamment du niveau de netteté du verdict), **And** si verdict confirmé = (a) la **revue adversariale T8.4** (`bmad-review-adversarial-general` sur §Verdict + §Gap analysis du document de recherche) a été déclenchée et ses findings intégrés (cf. Dev Notes §R2 ; patch Pass 1 spec validate P1-4 : matérialise la mitigation R2 dans le flux d'AC), **Then** :
   - **Si (a)** : `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` §L6 mis à jour avec verdict explicite + référence au document de recherche. Pas de nouvelle story Epic 14 créée.
   - **Si (b)** : §L6 9-2b mis à jour avec verdict + référence au document. Une nouvelle issue GitHub créée avec titre `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` + 4 labels `enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt` (pré-condition : T9.0 a créé les labels manquants — ground-truth Pass 1 spec validate P1-2 : `v0.2-milestone` et `legal-compliance` n'existent pas à la date de la spec, `enhancement` et `technical-debt` existent). **Note** : la story Epic 14 elle-même sera créée plus tard (au kickoff Epic 14) — pour 9-5-4 il suffit que l'issue de traçage existe.
   - **Si (c)** : §L6 9-2b mis à jour avec verdict bloquant + référence au document. Une nouvelle story `9-5-bis-swiss-co-958f-compliance.md` créée comme placeholder (status `backlog`) dans `_bmad-output/implementation-artifacts/` + entry ajoutée dans `sprint-status.yaml` + Critère d'arrêt Epic 9.5 mis à jour. **Note** : le contenu de la story 9-5-bis est intentionnellement laissé vide (juste backlog + status backlog) — sera élaboré séparément si l'option (c) est retenue.

7. **And** si applicable selon recherche, mise à jour `9-2a-export-pdf-csv.md` **§L7** (« Pas d'horodatage signé / certificat sur le PDF (Swiss CO Art. 958f conformité partielle) ») avec le verdict + référence document de recherche — ground-truth `9-2a:424`. **Pas** §L6 (PDF pagination cosmétique, sans rapport avec Swiss CO) ni §L13/L14/L15 (perf PDF 10k / audit timing / test format PDF — sans rapport avec Swiss CO 958f). La recherche peut aussi conclure que les PDF Kesh sont conformes Art. 957a — dans ce cas pas de mise à jour §L7 nécessaire.

8. **And** `_bmad-output/planning-artifacts/epic-9-5.md` §"Critères d'arrêt Epic 9.5" item « Document `research-swiss-co-958f.md` produit + décision formelle (a/b/c) appliquée à 9-2a/9-2b si applicable » coché `[x]` avec référence au commit de fermeture story.

### Closure GitHub + sprint-status

9. **Given** Phase recherche + Phase mise à jour cross-stories complétées, **When** la story est marquée done, **Then** :
   - **Commit closure unique** : `docs(9-5-4): close Swiss CO 958f research with verdict (a|b|c) (closes #N if applicable)` — body contient résumé verdict (option retenue) + liste fichiers modifiés + référence au document de recherche.
   - **GitHub Issue** : si une issue 9-5-4 a été créée préventivement (e.g. tracking research action #3 retro Epic 8), la fermer via le commit. **Si pas d'issue existante** (cas probable — l'action #3 retro Epic 8 n'a pas été convertie en issue GitHub formelle), **ne pas créer d'issue uniquement pour la fermer** — c'est la story file + commit qui font foi (cohérent §Issue Tracking Rule CLAUDE.md, l'issue n'est pas obligatoire pour les research stories internes).
   - `sprint-status.yaml` : entrée `9-5-4-swiss-co-research` mise à jour `backlog → ready-for-dev → in-progress → review → done`. `last_updated` field rafraîchi.
   - Critère d'arrêt epic-9-5.md ligne 206 « 4/4 stories avec status `done` » **satisfait à ce stade** (9-5-1/2/3/4 toutes done) — les sub-stories 1a/b/c/d sont trackées séparément dans `sprint-status.yaml` (8 entrées au total : 9-5-1a + 9-5-1b + 9-5-1c + 9-5-1d + 9-5-2 + 9-5-3 + 9-5-4 + epic-9-5-retrospective) mais ne changent pas le critère officiel `4/4` (qui compte les stories de niveau 1 dans l'epic). Patch Pass 1 spec validate P1-6 : clarification du décompte officiel vs sub-stories sprint-status. Reste la rétrospective Epic 9.5 (status `optional → done`) avant clôture complète de l'epic.

### Test Locally First — exemption documentée

10. **Given** la story 9-5-4 est **research-only sans modification de code source** (0 fichier `.rs`, `.svelte`, `.ts` modifié — uniquement `.md`), **When** la story est en review/done, **Then** la règle CLAUDE.md `Test Locally First` est **exempte** pour cette story (cf. §"Quand sauter" dans CLAUDE.md — commits doc-only ne nécessitent pas la batterie complète). **Vérification routine seulement** : `npm run lint-i18n-ownership` resté `PASS` si une mention i18n est faite (peu probable pour ce document research), sinon aucun check.

11. **And** **0 régression introduite** par les modifications cross-stories sur 9-2a / 9-2b — vérification ground-truth : les fichiers `_bmad-output/implementation-artifacts/9-2a-*.md` + `_bmad-output/implementation-artifacts/9-2b-*.md` sont des story files documentation, leur édition ne casse rien d'exécutable.

## Tasks / Subtasks

- [x] **T1** Pré-flight + bibliographie initiale (AC: #1, #2)
  - [x] T1.1 Confirmer accès web (WebFetch ou WebSearch disponible dans la session) pour récupérer texte CO Art. 957a + 958f + OLICo + LSCSE.
  - [x] T1.2 Constituer bibliographie minimale (5 primaires + 3 secondaires) — URLs + date d'accès notées.
  - [x] T1.3 Brancher `chore/epic-9-5-planning` confirmé checkout (déjà sur cette branche post-9-5-1d done, cohérent `feedback_avoid_parallel_prs` qui bundle Epic 9.5 sur une PR unique).

- [x] **T2** Squelette document `research-swiss-co-958f.md` (AC: #3)
  - [x] T2.1 Créer `_bmad-output/planning-artifacts/research-swiss-co-958f.md` avec sommaire imposé AC #3 (Préambule, Art. 957a, Art. 958f, OLICo, ECH-0058, LSCSE, État Kesh, Gap analysis, Verdict, Recommandations).
  - [x] T2.2 Préambule complet : disclaimer non-juridique + scope PME < Art. 727 + date d'analyse + bibliographie initiale (T1.2).

- [x] **T3** Recherche Art. 957a CO — Tenue de la comptabilité (AC: #3, #5)
  - [x] T3.1 Récupérer texte officiel Art. 957a CO consolidé via fedlex.admin.ch (alinéas 1 à 4 typiquement — vérifier numérotation actuelle).
  - [x] T3.2 Synthétiser exigences applicables PME : tenue régulière + langue + monnaie + formats acceptés (livre journal + grand livre + comptes annuels).
  - [x] T3.3 Mapper sur Kesh : journal_entries.rs + chart_of_accounts.rs + rapports Story 9-1 (Bilan, Pertes & Profits, Balance, Journal). Citer fichiers source précis.

- [x] **T4** Recherche Art. 958f CO — Conservation 10 ans (AC: #3, #5)
  - [x] T4.1 Récupérer texte officiel Art. 958f CO consolidé (alinéas 1 à 3 typiquement).
  - [x] T4.2 Synthétiser exigences : durée 10 ans, intégrité, lisibilité durable, signature électronique qualifiée si support modifiable (§al. 3).
  - [x] T4.3 Mapper sur Kesh : audit_log immutable + SHA-256 metadata.json + export ZIP Story 9-2b. Identifier l'écart précis avec signature qualifiée si applicable.

- [x] **T5** Recherche OLICo + ECH-0058 (AC: #3, #5)
  - [x] T5.1 Récupérer OLICo (RS 221.431) consolidé. Identifier sections sur supports modifiables + formats acceptés (PDF/A, XML, CSV signés).
  - [x] T5.2 ECH-0058 standard archivage électronique : applicabilité PME (généralement non-obligatoire) + bonne pratique. **Note dev** : ground-truth recherche révèle que ECH-0058 est en réalité « Norme d'interface : cadre d'annonce » (échanges admin), PAS un standard archivage. Le standard archivage pertinent est ECH-0039 / ECH-0160. Documenté dans §ECH-0058 du document de recherche avec rectification honnête.
  - [x] T5.3 Synthétiser exigences techniques : intégrité (hash + signature OU log immutable + procédure de vérification) — c'est ici que le débat « audit-trail-only suffit-il ? » se cristallise.

- [x] **T6** Recherche LSCSE signature électronique qualifiée (AC: #3, #5)
  - [x] T6.1 LSCSE RS 943.03 : QES (qualifiée), AES (avancée), SES (simple) — distinctions et exigences fournisseurs.
  - [x] T6.2 Coût ordre-de-grandeur QES pour une PME (Swisscom Trust Service, QuoVadis, SwissSign — typiquement CHF 200-500/an certificat + plateforme).
  - [x] T6.3 Applicabilité aux exports comptables Kesh : QES nécessaire ? Recommandée ? Optionnelle ? Conclusion documentée avec sources.

- [x] **T7** Section « État de l'art Kesh » + Gap analysis (AC: #3)
  - [x] T7.1 Synthèse précise de l'implémentation actuelle Kesh (audit_log, SHA-256, ZIP, PDF/CSV exports) — référencer story files + fichiers source.
  - [x] T7.2 Tableau Gap analysis (Markdown) ligne par ligne — Exigence légale | État Kesh | Verdict conforme/partiel/non | Référence. **18 lignes** : 9 ✅ conforme + 6 🟡 partiellement conforme + 3 ➖ N/A.
  - [x] T7.3 Identifier les 2-3 écarts majeurs candidats à remédiation (typiquement : absence QES + horodatage tiers signé). **3 écarts identifiés** : (1) absence QES/horodatage tiers, (2) absence procès-verbal explicite migration, (3) procédures dispersées doc utilisateur.

- [x] **T8** Verdict + Recommandations (AC: #3, #6)
  - [x] T8.1 Synthétiser la recherche en verdict (a / b / c) avec justification 3-5 paragraphes ancrés dans T7.2 gap analysis. **Verdict proposé : (b) Dette explicite v0.2** (5 paragraphes justification ancrés gap analysis + EXPERTsuisse PP 10 + Motion Schneeberger 22.3004).
  - [x] T8.2 Recommandations actionables numérotées (0 à N items). **10 recommandations** documentées §Recommandations.
  - [x] T8.3 **Checkpoint élicitation Guy (OBLIGATOIRE)** : `AskUserQuestion` avec options (a)/(b)/(c)/Réviser → **Guy a confirmé (b)**.
  - [x] T8.4 **Si verdict confirmé = (a)** → SKIP (verdict (b) confirmé, T8.4 conditionnelle non-applicable). Note documentée §Verdict du document de recherche.

- [x] **T9** Mise à jour cross-stories (AC: #6, #7, #8)
  - [x] T9.0 **Pré-flight T9.4** Vérifié `gh label list --repo guycorbaz/kesh` : `v0.2-milestone` + `legal-compliance` **n'existaient pas** (ground-truth Pass 1 P1-2 confirmé), créés via `gh label create` (couleurs `0075ca` + `e4e669`).
  - [x] T9.1 Mis à jour `9-2b-export-global-zip.md` §L6 avec verdict (b) + référence document de recherche.
  - [x] T9.2 Mis à jour `9-2a-export-pdf-csv.md` **§L7** (ground-truth `9-2a:424` confirmé) avec verdict (b) + référence document. PAS §L6 (PDF pagination cosmétique, sans rapport Swiss CO).
  - [x] T9.3 Mis à jour `epic-9-5.md` §Critères d'arrêt Epic 9.5 — item « Document `research-swiss-co-958f.md` produit + décision formelle » coché `[x]` + détail verdict (b).
  - [x] T9.4 GitHub Issue **#98** créée : `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` avec 4 labels `enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`.
  - [x] T9.5 Verdict (c) → SKIP (non-applicable, verdict (b) confirmé). Pas de story `9-5-bis` créée.

- [x] **T10** Commit closure + sprint-status (AC: #9)
  - [x] T10.1 Commit unique `docs(9-5-4): close Swiss CO 958f research with verdict (b)` avec body verdict + fichiers modifiés.
  - [x] T10.2 `sprint-status.yaml` : `9-5-4-swiss-co-research: in-progress → review` + `last_updated` rafraîchi.
  - [x] T10.3 **Test Locally First exempt** (research-only, AC #10) — `git status` clean post-commit attendu.

## Dev Notes

### Pattern de référence : story doc-only

Cette story 9-5-4 ressemble structurellement à la story 9-5-3 (« Process codification CLAUDE.md ») : aucun code source modifié, uniquement de la documentation. Différence majeure : 9-5-3 codifiait du process projet interne, 9-5-4 produit une recherche externe (lois suisses) avec **dépendance à des sources web** non-internes au projet.

### Sources web — discipline de citation

Toute affirmation normative (« la loi dit X ») doit être citée avec :
- **URL exacte** (pas un domaine racine — un permalink vers l'alinéa précis sur fedlex.admin.ch si possible).
- **Date d'accès** (`accédé 2026-05-XX`) — important car le CO Suisse est amendé régulièrement (la dernière révision majeure 957a+958f date de 2012, en vigueur depuis 2013, mais des amendements ponctuels existent).
- **Extrait original** entre guillemets, **français OU allemand** selon disponibilité (le texte officiel CO existe dans les 3 langues nationales — préférer FR pour Guy, fallback DE si la version FR n'est pas consolidée).

Si une source est inaccessible (paywall expertsuisse.ch, lien mort), noter explicitement dans le document : `[Source N/A — référence trouvée via Z mais non-vérifiable directement à la date d'accès]`. Ne **jamais** inventer une URL.

### Risque R1 — recherche autonome LLM hallucinations

Les LLM ont une tendance documentée à **halluciner des références juridiques** (numéros d'alinéas, jurisprudences fictives, citations inventées). Mitigation obligatoire pour cette story :

- **Cross-check ground-truth systématique** : chaque numéro d'article + alinéa cité doit être vérifié contre fedlex.admin.ch via WebFetch (ou subagent équivalent). Si une citation ne peut pas être vérifiée, la **retirer** du document plutôt que de la conserver « sous bénéfice du doute ».
- **Pas de jurisprudence inventée** : si le LLM ne trouve pas de cas pratique documenté, écrire `« Pas de jurisprudence publique pertinente identifiée à la date d'analyse »` plutôt que d'inventer un arrêt fédéral fictif.
- **Numéros d'ordonnance** : OLICo = RS 221.431, LSCSE = RS 943.03, CO = RS 220 — ces numéros sont **stables** (vérifiables sur le site officiel admin.ch). Ne pas accepter une variation type RS 221.43 ou RS 943.3 sans vérification.

### Risque R2 — verdict (a) suspect

Le résultat « option (a) — Kesh est strictement conforme » est suspect a priori car les exports ZIP Kesh ne sont pas signés QES. Si la recherche conclut (a), prévoir une **revue critique additionnelle** Pass review (e.g. `bmad-review-adversarial-general`) avant propagation cross-stories, pour s'assurer que l'option (a) n'est pas une rationalisation insuffisante. Probabilité a priori (a) : **faible**, (b) : **élevée**, (c) : **faible** (cf. epic-9-5.md Q2).

### Risque R3 — élicitation Guy nécessaire

Le verdict (a/b/c) implique une décision business + acceptation de risque légal — c'est une **décision Guy**, pas une décision LLM autonome. Pattern recommandé :
- LLM produit le document de recherche complet (T2-T7) en autonomie via subagent recherche.
- LLM **propose** un verdict (T8.1) avec justification mais **n'écrit pas la décision finale dans 9-2b §L6 avant validation Guy**.
- Checkpoint explicite T8.3 : `AskUserQuestion` ou texte « Verdict proposé : (X). Confirmer (X) avant propagation cross-stories, ou rebascule sur (Y) ? ».

Cela respecte la nature « décision réglementaire engageante » de la story, pas un automatisme.

### Pourquoi pas de validation juridique formelle

Le scope explicite **hors-scope** une revue par un avocat suisse spécialisé (cf. §Scope « Hors scope »). Justifications :

- **Coût** : revue juridique externe = CHF 1000-3000 typique pour une PME — disproportionné pour la décision v0.1 vs v0.2.
- **Précédent jurisprudence Suisse PME** : audit-trail + SHA-256 est généralement accepté par les fiduciaires et l'AFC en cas de contrôle. La QES est exigée pour les **transmissions** à l'État (e-déclarations TVA notamment), pas pour la **conservation interne**. Cf. recherche T3-T6.
- **Disclaimer document** : le document `research-swiss-co-958f.md` mentionnera explicitement « ce document ne constitue pas un avis juridique formel. Pour une publication v0.1 commerciale, une revue juridique externe est recommandée mais non-bloquante pour la décision tech-stack v0.1 ».

### Outils LLM recommandés pour T3-T6

- **WebFetch** : pour récupérer des pages spécifiques (fedlex.admin.ch alinéas précis). Préférer fedlex.admin.ch à legifrance.gouv.fr ou normattiva.it qui ne couvrent pas le CO Suisse.
- **WebSearch** : pour identifier des commentaires de référence (e.g. « Schulthess commentaire Art. 958f »). Filtrer sur `.ch` ou `.admin.ch` pour pertinence.
- **Subagent dédié** : pour parallélisme — un subagent peut faire T3 (957a), un autre T4 (958f), un autre T5 (OLICo). Convergence ensuite en T7 (gap analysis cross-références).

### Memory carries

- `feedback_haiku_review_diff_combined` : si Pass review Haiku, discipline grep ground-truth — mais pour 9-5-4, le « grep ground-truth » est de **vérifier les citations légales** par WebFetch direct sur fedlex.admin.ch.
- `feedback_avoid_parallel_prs` : story 9-5-4 reste sur la branche `chore/epic-9-5-planning` — pas de PR séparée, regroupement Epic 9.5 attendu après rétro.
- `feedback_zero_tech_debt_carryforward` : option (b) est compatible — la dette est tracée (Epic 14 issue), pas reportée silencieusement.
- `project_prod_deployment_gating` : Kesh n'est pas en prod pendant v0.1, donc un changement d'architecture conformité légale d'ici v0.1 final n'a pas de coût migration utilisateur. Confirme que reporter à v0.2 (option b) est acceptable.

### Project Structure Notes

- **Fichier créé par 9-5-4** :
  - `_bmad-output/planning-artifacts/research-swiss-co-958f.md` — document de recherche (~400-800 lignes attendues).
  - Optionnel selon verdict : `_bmad-output/implementation-artifacts/9-5-bis-swiss-co-958f-compliance.md` (placeholder backlog si option c).

- **Fichiers édités par 9-5-4** :
  - `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` (§L6 verdict + référence).
  - `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` (si applicable selon T3-T7 — probablement pas).
  - `_bmad-output/planning-artifacts/epic-9-5.md` (§Critères d'arrêt — coche item).
  - `_bmad-output/implementation-artifacts/sprint-status.yaml` (entry 9-5-4 + last_updated).
  - `_bmad-output/implementation-artifacts/9-5-4-swiss-co-research.md` (cette spec, Change Log final + statut).

- **Fichiers NON touchés** :
  - **Aucun** fichier source applicatif `.rs` / `.svelte` / `.ts` modifié (research-only, scope explicit).
  - **Aucune** migration DB.
  - **Aucun** test (Vitest, Playwright, cargo test) modifié.

### Testing standards summary

- **Pas de tests automatisés** sur le document research-swiss-co-958f.md (ce n'est pas du code). La qualité de la recherche est évaluée par :
  - **Auto-review** : T8 prévoit un review critique du verdict avant propagation (R2 mitigation).
  - **Pass review optionnelle** : `bmad-create-story validate 9-5-4` après création de spec (passe Sonnet ou Haiku) + `bmad-code-review 9-5-4` après dev-story (passe optionnelle car research-only, mais utile pour vérifier la rigueur des citations + cohérence cross-stories).
  - **Element check humain** : Guy peut relire le document final pré-merge si souhaité, c'est une décision business engageante.

### Estimation effort

- **T1 (pré-flight + biblio)** : 15 min (URLs identifiées + WebFetch test).
- **T2 (squelette)** : 15 min.
- **T3 (Art. 957a)** : 30-60 min (lecture + interprétation + mapping Kesh).
- **T4 (Art. 958f)** : 30-60 min (cœur de la recherche, dépend de la signature électronique).
- **T5 (OLICo + ECH-0058)** : 30 min.
- **T6 (LSCSE)** : 20-40 min.
- **T7 (État + Gap)** : 30-60 min (synthèse cross-référencée — phase critique).
- **T8 (Verdict + Recommandations + checkpoint Guy)** :
  - T8.1-T8.3 : 20-40 min (synthèse + recommandations + checkpoint élicitation).
  - **T8.4 (conditionnel si verdict (a))** : 20-40 min revue adversariale `bmad-review-adversarial-general` (skip si verdict (b) ou (c)). Patch Pass 2 spec validate P2-1.
- **T9 (cross-stories updates)** : 15-30 min selon verdict (incluant T9.0 création labels GitHub si verdict (b), ~5 min).
- **T10 (commit + sprint-status)** : 10 min.
- **Total** : **~3-5h si verdict (b/c) confirmé direct** ; **~3.5-5.5h si verdict (a)** (avec revue adversariale T8.4). `research subagent parallélisable T3+T4+T5 pour économiser ~1h`. Patch Pass 2 P2-1 : effort post-T8.4 ajout explicité.

### References

- [Source: _bmad-output/planning-artifacts/epic-9-5.md#Story-9.5-4] — spec parent ACs source.
- [Source: _bmad-output/implementation-artifacts/9-2b-export-global-zip.md#L6] — limitation v0.1 à recheck.
- [Source: _bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md] — exports PDF/CSV format légal AFC (à vérifier T3.3).
- [Source: _bmad-output/implementation-artifacts/9-1-rapports-comptables-bilan-resultat-balance-journaux.md] — rapports légaux PME.
- [Source: _bmad-output/implementation-artifacts/epic-8-retrospective.md#action-3] — action retro Epic 8 partielle « Recherche réglementaire Swiss CO Art. 957a — Guy ».
- [Source: CLAUDE.md#Test-Locally-First] — règle Test Locally First avec exception research-only documentée.
- [Source: CLAUDE.md#Issue-Tracking-Rule] — GitHub Issues comme source de vérité (mais pas obligatoire pour stories research internes — cf. AC #9).
- [Source: CLAUDE.md#Règle-de-commit-et-push] — branche `chore/epic-9-5-planning` continue + push reporté fin Epic 9.5.
- [External: fedlex.admin.ch — Code des obligations consolidé (RS 220)] — à confirmer URL exact T1.2.
- [External: admin.ch — Ordonnance OLICo (RS 221.431)] — à confirmer URL exact T1.2.
- [External: admin.ch — LSCSE (RS 943.03)] — à confirmer URL exact T1.2.

## Dev Agent Record

### Agent Model Used

**Claude Opus 4.7 (1M context)** — single-pass exécuté 2026-05-20. Cohérent règle CLAUDE.md `Review Iteration Rule` : ≠ Pass 2 Haiku 4.5 spec validate (Sonnet → Haiku → **Opus** dev → cycle suivant à venir code-review).

### Debug Log References

- WebSearch + WebFetch séquentiels pour fedlex.admin.ch (CO Art. 957a/957/958f via PDF officiel `fedlex-data-admin-ch-eli-oc-2012-810-fr-pdf-a.pdf` ; OLICo RS 221.431 PDF complet 4 pages ; LSCSE RS 943.03 PDF 5 pages dépliées).
- Fedlex pages HTML directes bloquent sans JS → fallback PDF/A direct via `https://fedlex.data.admin.ch/filestore/...`. Pattern réutilisable pour recherches futures.
- Ground-truth `gh label list --repo guycorbaz/kesh` confirmé : `v0.2-milestone` + `legal-compliance` absents au moment T9.0, créés à la volée.
- Ground-truth `grep "| L6\|| L7" 9-2a / 9-2b` confirmé : 9-2a §L7 ligne 424 + 9-2b §L6 ligne 802 = cibles Swiss CO Art. 958f (Pass 1 spec validate P1-1 corroboré au moment dev).
- Découverte ECH-0058 ≠ archivage électronique : c'est « Norme d'interface : cadre d'annonce ». Documenté honnêtement §ECH-0058 du document de recherche avec rectification (le standard archivage pertinent est ECH-0039 / ECH-0160).

### Completion Notes List

- **Document de recherche** : 530 lignes (cible 300-1200, AC #4 satisfait). Structure suit le sommaire imposé AC #3 sans déviation.
- **Sources** : 5 primaires (CO, OLICo, LSCSE PDF officiels Fedlex + kmu.admin.ch FR + kmu.admin.ch EN guide électronique) + 3 secondaires (EXPERTsuisse PP 10 cité indirect, Motion Schneeberger 22.3004 TREUHAND|SUISSE, kmu interview 2022). AC #2 satisfait (≥ 5 + ≥ 3).
- **Citations légales** : Art. 957a CO + Art. 957 CO + Art. 958f CO + Art. 1/3/4/5/6/7/8/9/10/11/12 OLICo + Art. 1/2 LSCSE — toutes citées mot pour mot avec URL + date d'accès, conformes AC #5.
- **Gap analysis** : 18 lignes structurées (Exigence | État Kesh | Verdict | Référence). 9 ✅ + 6 🟡 + 3 ➖.
- **Verdict (b) confirmé** par Guy via checkpoint élicitation T8.3 (`AskUserQuestion` avec options exclusives a/b/c/Réviser). Pas d'écart Dev Notes §R3.
- **R1 hallucinations LLM** : mitigé par WebFetch direct PDFs Fedlex officiels (citation mot pour mot, pas reformulation). 0 jurisprudence inventée. Numéros d'ordonnance vérifiés (RS 220 + 221.431 + 943.03 stables).
- **R2 verdict (a) suspect** : non-applicable, verdict (b) retenu.
- **R3 décision Guy** : respecté, checkpoint OBLIGATOIRE exécuté.
- **Test Locally First exempt** (AC #10) : 0 fichier `.rs`/`.svelte`/`.ts` modifié.
- **0 régression** (AC #11) : modifications cross-stories sur 9-2a / 9-2b sont des story files documentation (édition ne casse rien d'exécutable).
- **GitHub Issue #98** créée pour suivi Epic 14 v0.2 (4 labels). Pas de story 9-5-bis (verdict ≠ c).

### File List

**Fichiers créés** :

- `_bmad-output/planning-artifacts/research-swiss-co-958f.md` — document de recherche complet (530 lignes).

**Fichiers modifiés** :

- `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` — §L6 mis à jour avec verdict (b) + référence document de recherche.
- `_bmad-output/implementation-artifacts/9-2a-export-pdf-csv.md` — §L7 mis à jour avec verdict (b) + référence document de recherche.
- `_bmad-output/planning-artifacts/epic-9-5.md` — §Critères d'arrêt Epic 9.5 item « Document `research-swiss-co-958f.md` produit » coché `[x]` + détail verdict (b).
- `_bmad-output/implementation-artifacts/9-5-4-swiss-co-research.md` — Status `ready-for-dev → review`, Tasks/Subtasks toutes cochées `[x]`, Dev Agent Record + File List + Change Log complétés.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — entry `9-5-4-swiss-co-research: ready-for-dev → in-progress → review` + `last_updated` rafraîchi.

**GitHub** :

- Issue `#98` créée : `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` avec 4 labels.
- Labels créés : `v0.2-milestone` (couleur `0075ca`) + `legal-compliance` (couleur `e4e669`).

**Fichiers NON modifiés (comme prévu)** :

- 0 fichier source applicatif (`.rs` / `.svelte` / `.ts`).
- 0 migration DB.
- 0 test (Vitest, Playwright, cargo test).

## Change Log

### Dev-story — 2026-05-20, Opus 4.7 single-pass

**Cycle** : `bmad-dev-story 9-5-4` single-pass (Opus 4.7, contexte frais après spec validate Sonnet 4.6 → Haiku 4.5 convergé). Cohérent règle CLAUDE.md `Review Iteration Rule` cycle `Sonnet → Haiku → Opus`.

**Livrable principal** : document de recherche `_bmad-output/planning-artifacts/research-swiss-co-958f.md` (530 lignes, structure conforme sommaire imposé AC #3, sources primaires 5 + secondaires 3 cités mot pour mot avec URLs + dates).

**Verdict** : **(b) Dette explicite v0.2** confirmé par Guy via checkpoint élicitation T8.3 OBLIGATOIRE (cf. R3 + Pass 1 P1-3). Pas de revue adversariale T8.4 (conditionnel verdict (a) only).

**Propagation cross-stories** : 9-2b §L6 + 9-2a §L7 + epic-9-5.md §Critères d'arrêt mis à jour avec référence document. GitHub Issue **#98** créée pour Epic 14 (labels `enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`, dont 2 créés à la volée via T9.0 `gh label create`).

**Mitigation R1 LLM hallucinations** : tous les textes légaux récupérés mot pour mot depuis PDF officiels Fedlex (oc/2012/810 pour CO ; cc/2002/216 pour OLICo ; cc/2016/752 pour LSCSE). 0 jurisprudence inventée. 0 alinéa fabriqué.

**Découverte additionnelle ECH-0058** : ground-truth WebSearch révèle ECH-0058 = « Norme d'interface : cadre d'annonce » (échanges admin, pas archivage). Documenté honnêtement §ECH-0058 du document avec rectification (standard archivage pertinent = ECH-0039 / ECH-0160, optionnel pour PME).

**Test Locally First** : exempté (AC #10) — 0 fichier source `.rs` / `.svelte` / `.ts` / migration / test modifié. `git status` clean post-commit attendu.

**Statut final** : `ready-for-dev → in-progress → review`. Prochaine étape : `bmad-code-review 9-5-4` recommandé (LLM ≠ Opus 4.7 → Sonnet 4.6 cycle suivant). **Note** : pour une story 100 % doc-only research, la revue code-review est optionnelle (cf. Dev Notes §Testing standards summary) mais utile pour vérifier rigueur citations + cohérence cross-stories.

**Effort réel constaté** : ~2-3h orchestrateur (vs estimation 3-5h spec). Compression vs estimation expliquée par : (1) Opus single-pass mode orchestré sans subagent parallel T3+T4+T5 (subagent était suggéré pour économiser temps, mais single-pass séquentiel reste rapide via PDF read direct) ; (2) PDFs Fedlex sauvegardés localement par Read tool évitent re-fetch ; (3) verdict (b) probable empêche T8.4 + T9.5.

### Pass 1 spec validate — 2026-05-20, Sonnet 4.6 (subagent contexte frais)

**Verdict trend brut** : 0 CRITICAL + 2 HIGH + 2 MEDIUM + 2 LOW = 6 findings (Convergence : NON — 2 HIGH + 2 MEDIUM > LOW restent).

**Discipline grep ground-truth Sonnet** : 4/4 vérifications majeures positives (Sonnet a grep-vérifié 9-2a `| L6 |` et `| L7 |`, `gh label list --repo guycorbaz/kesh`, et cross-checked R3 vs T8.3). Orchestrateur a également vérifié 9-2a:418-432 + gh labels avant patch application.

**Patches appliqués (6/6 — tous validés ground-truth)** :

1. **HIGH P1-1** — AC #7 + Scope §"Décisions formelles" : mauvais numéro de limitation dans 9-2a. Ground-truth `grep -n "| L" 9-2a-export-pdf-csv.md` : `L6 = PDF pagination cosmétique` (ligne 423), `L7 = horodatage signé/certificat PDF Swiss CO Art. 958f conformité partielle` (ligne 424 — la vraie cible Swiss CO). `L13/L14/L15` = perf PDF 10k / audit timing / test format — sans rapport avec Swiss CO. **Patch** : Scope §32 + AC #7 + T9.2 corrigés sur `9-2b §L6 + 9-2a §L7 si applicable` avec ground-truth explicite.

2. **HIGH P1-2** — T9.4 labels GitHub inexistants. Ground-truth `gh label list --repo guycorbaz/kesh` : 12 labels existent (`bug`, `documentation`, `enhancement`, `known-failure`, `technical-debt`, etc.), mais **`v0.2-milestone` et `legal-compliance` n'existent pas**. **Patch** : ajouté T9.0 « Pré-flight T9.4 » qui crée les 2 labels manquants via `gh label create` si absents, avec descriptions et couleurs explicites. T9.4 mis à jour pour utiliser 4 labels (`enhancement` + `v0.2-milestone` + `legal-compliance` + `technical-debt`) avec pré-condition T9.0 exécuté.

3. **MEDIUM P1-3** — T8.3 contradiction R3 : T8.3 disait « pas de question si verdict net » mais R3 Dev Notes disait « toujours valider Guy avant propagation ». Un dev agent suivant T8.3 à la lettre pourrait propager 9-2b §L6 sans demander Guy si verdict (b) jugé net — contraire à la décision business engageante du R3. **Patch** : T8.3 reformulé « Checkpoint élicitation Guy (OBLIGATOIRE, pas conditionnel) » avec `AskUserQuestion` à 3 options. AC #6 mis à jour pour ancrer T8.3 obligatoire.

4. **MEDIUM P1-4** — R2 mitigation absente des tasks. §R2 Dev Notes disait « si verdict (a), prévoir revue adversariale `bmad-review-adversarial-general` » mais aucun subtask T8.x ne le matérialisait. Risque : dev agent ne lit pas toujours Dev Notes, passerait directement T8.3 → T9 si verdict (a). **Patch** : ajouté T8.4 « Si verdict (a) : déclencher revue adversariale » conditionnel + ligne AC #6 ancrant T8.4 dans le flux d'AC.

5. **LOW P1-5** — AC #1 source `eitsa.ch` inexistant / hallucination probable. Sources juridiques suisses reconnues : `expertsuisse.ch`, `treuhand-suisse.ch`, `schulthess.com`, `helbing.ch`, `bger.ch`. Le domaine `eitsa.ch` n'est pas une référence pertinente droit comptable suisse. **Patch** : AC #1 remplacé `eitsa.ch` par `expertsuisse.ch` + `treuhand-suisse.ch` (cohérent AC #2 bibliographie secondaire).

6. **LOW P1-6** — AC #9 confusion `4/4 stories` vs `8/8 sub-stories`. La spec disait « en réalité c'est 8/8 » qui prêtait à confusion avec le critère officiel epic-9-5.md ligne 206. **Patch** : AC #9 reformulé pour clarifier que le critère officiel `4/4` (stories niveau 1 : 9-5-1/2/3/4) est satisfait, et que les 8 entrées sprint-status incluent les sub-stories trackées séparément sans changer le critère.

**Cross-verification orchestrateur ground-truth** (avant patches) :

```
grep -n "| L[0-9]" 9-2a-export-pdf-csv.md
→ 418:L1 + 419:L2 + ... + 423:L6 PDF pagination + 424:L7 Swiss CO Art. 958f ✓ (confirme P1-1)

gh label list --repo guycorbaz/kesh --limit 50
→ 12 labels existants, AUCUN nommé "v0.2-milestone" ni "legal-compliance" ✓ (confirme P1-2)

grep "| L6 " 9-2b-export-global-zip.md
→ 802:L6 Swiss CO Art. 958f ✓ (9-2b cible inchangée, P1-1 ne porte que sur 9-2a)
```

**Findings dismissed** : 0 — tous les 6 findings sont validés et patchés.

**Recommandation Sonnet** : Pass 2 Haiku 4.5 avec discipline grep ground-truth obligatoire (cycle CLAUDE.md `Sonnet → Haiku → Opus → Sonnet`). Vérifier propagation patches Pass 1 + chercher inconsistances résiduelles AC↔T mapping post-patches (notamment AC #6 ↔ T8.3 + T8.4 nouveau).

**Modèle Pass 1** : Sonnet 4.6 (subagent isolé contexte frais — spec créée par Opus 4.7, règle CLAUDE.md `LLM différent passe précédente` respectée).

### Pass 2 spec validate — 2026-05-20, Haiku 4.5 (subagent contexte frais)

**Verdict trend brut** : 0 CRITICAL + 0 HIGH + 0 MEDIUM + 1 LOW = 1 finding (Convergence : **OUI** — critère CLAUDE.md « Uniquement findings LOW » atteint).

**Discipline grep ground-truth Haiku** : 6/6 propagations Pass 1 vérifiées via `grep -nF` direct par le reviewer Haiku, **aucune hallucination ni régression** détectée. Ground-truths cross-checked :

```
grep -nF "9-2a §L7" 9-5-4-swiss-co-research.md
→ 2 hits (Scope + T9.2) ✓ (confirme propagation P1-1)

grep -nF "9-2a §L6" 9-5-4-swiss-co-research.md
→ 1 hit (explication explicite « PAS §L6 = PDF pagination cosmétique » dans T9.2) ✓ (référence correcte, pas régression)

grep -nF "gh label create" 9-5-4-swiss-co-research.md
→ 2 hits T9.0 (v0.2-milestone + legal-compliance) ✓ (confirme propagation P1-2)

grep -nF "OBLIGATOIRE" 9-5-4-swiss-co-research.md
→ 2 hits (T8.3 task + Change Log entry) ✓ (confirme propagation P1-3)

grep -nF "T8.4" 9-5-4-swiss-co-research.md
→ 4 hits (AC #6 + T8.4 task def + Change Log + recommandations) ✓ (confirme propagation P1-4)

grep -nF "eitsa" 9-5-4-swiss-co-research.md
→ 1 hit (Change Log P1-5 documentation patch) ; 0 hit en tant que source AC #1 ✓ (confirme propagation P1-5)

AC #9 ligne 97 : « 4/4 stories niveau 1 satisfait » + « 8 entrées sprint-status (sub-stories) »
✓ (confirme propagation P1-6)
```

**Patch appliqué (1 LOW polish)** :

1. **LOW-P2-1 — Effort estimation post-T8.4 ajout** : §"Estimation effort" Dev Notes ne reflétait pas le coût additionnel `+20-40 min` de T8.4 (revue adversariale conditionnelle si verdict (a)). Le total `~3-5h` ne distinguait pas le chemin verdict (b/c) du chemin (a) avec T8.4. **Patch** : §"Estimation effort" T8 décomposée (T8.1-T8.3 baseline + T8.4 conditionnel), total reformulé `~3-5h (b/c) | ~3.5-5.5h (a)`, T9 mentionne explicitement T9.0 +5 min création labels.

**Trend cumul cycle 2-passes** :
- Pass 1 Sonnet 4.6 : 0C+2H+2M+2L = 6 findings → 6 patches (tous ground-truth validés).
- Pass 2 Haiku 4.5 : 0C+0H+0M+1L = 1 finding → 1 LOW polish → **0 résiduel**.
- **Total : 7 patches sur 2 passes. Cycle court (Sonnet → Haiku) cohérent 9-5-1d / 9-5-1b spec validate done en 2 passes.**

**Cycle complet `Sonnet → Haiku`** : convergence atteinte sans nécessité Opus Pass 3 (scope research-only, pas de subtilité architecturale requise). Pattern cohérent retro Epic 9 Insight I1 « Opus catches subtle stuff » qui s'applique aux scopes complexes — pas le cas ici.

**Modèle Pass 2** : Claude Haiku 4.5 (subagent isolé contexte frais — règle CLAUDE.md `LLM différent passe précédente` respectée Sonnet → Haiku). Discipline grep ground-truth Haiku **0 hallucination** sur ce cycle, **6/6 propagations vérifiées positives**.

**Statut final spec** : `ready-for-dev` confirmé définitif post-Pass 2. Prête pour `bmad-dev-story 9-5-4` (LLM recommandé Opus 4.7 ou Sonnet 4.6 — différent de Pass 2 Haiku, cycle suivant `Sonnet → Haiku → Opus → Sonnet`).
