# Story 9.5-4: Recherche réglementaire Swiss CO Art. 957a / 958f — conservation 10 ans + intégrité

Status: ready-for-dev

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

Une des 3 options exclusives, formulée explicitement dans le document de recherche §"Verdict" et propagée dans 9-2a/9-2b §L6 :

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

1. **Given** un workspace Kesh à jour avec `main` `35344c9` + branche `chore/epic-9-5-planning` checkée (HEAD `bceb112` post-9-5-1d code-review done), **When** la story démarre, **Then** prérequis confirmés : pas de cargo build / npm install nécessaire (research-only). Outils à confirmer disponibles : accès web (WebFetch / WebSearch ou subagent equivalent), accès au site `https://www.fedlex.admin.ch` (texte officiel CO consolidé), accès à `https://www.eitsa.ch` ou équivalent (jurisprudence + commentaires PME), accès à `https://www.afc.admin.ch` (AFC instructions PME).

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

6. **Given** le document produit + le verdict (a/b/c) statué, **When** la décision est prise (single-pass orchestré OU élicitation Guy via question explicite si ambiguïté reste), **Then** :
   - **Si (a)** : `_bmad-output/implementation-artifacts/9-2b-export-global-zip.md` §L6 mis à jour avec verdict explicite + référence au document de recherche. Pas de nouvelle story Epic 14 créée.
   - **Si (b)** : §L6 9-2b mis à jour avec verdict + référence au document. Une nouvelle issue GitHub `enhancement` créée avec titre `[Epic 14] Swiss CO 958f signature électronique qualifiée (option b retenue 9-5-4)` + label `v0.2-milestone` + label `enhancement` + label `legal-compliance`. **Note** : la story Epic 14 elle-même sera créée plus tard (au kickoff Epic 14) — pour 9-5-4 il suffit que l'issue de traçage existe.
   - **Si (c)** : §L6 9-2b mis à jour avec verdict bloquant + référence au document. Une nouvelle story `9-5-bis-swiss-co-958f-compliance.md` créée comme placeholder (status `backlog`) dans `_bmad-output/implementation-artifacts/` + entry ajoutée dans `sprint-status.yaml` + Critère d'arrêt Epic 9.5 mis à jour. **Note** : le contenu de la story 9-5-bis est intentionnellement laissé vide (juste backlog + status backlog) — sera élaboré séparément si l'option (c) est retenue.

7. **And** si applicable selon recherche, mise à jour `9-2a-export-pdf-csv.md` (e.g. conformité format légal AFC des PDF rapports Bilan/PnL — la recherche peut conclure que les PDF Kesh sont conformes Art. 957a OU partiellement seulement, dans quel cas L13/L14/L15 9-2a sont à compléter).

8. **And** `_bmad-output/planning-artifacts/epic-9-5.md` §"Critères d'arrêt Epic 9.5" item « Document `research-swiss-co-958f.md` produit + décision formelle (a/b/c) appliquée à 9-2a/9-2b si applicable » coché `[x]` avec référence au commit de fermeture story.

### Closure GitHub + sprint-status

9. **Given** Phase recherche + Phase mise à jour cross-stories complétées, **When** la story est marquée done, **Then** :
   - **Commit closure unique** : `docs(9-5-4): close Swiss CO 958f research with verdict (a|b|c) (closes #N if applicable)` — body contient résumé verdict (option retenue) + liste fichiers modifiés + référence au document de recherche.
   - **GitHub Issue** : si une issue 9-5-4 a été créée préventivement (e.g. tracking research action #3 retro Epic 8), la fermer via le commit. **Si pas d'issue existante** (cas probable — l'action #3 retro Epic 8 n'a pas été convertie en issue GitHub formelle), **ne pas créer d'issue uniquement pour la fermer** — c'est la story file + commit qui font foi (cohérent §Issue Tracking Rule CLAUDE.md, l'issue n'est pas obligatoire pour les research stories internes).
   - `sprint-status.yaml` : entrée `9-5-4-swiss-co-research` mise à jour `backlog → ready-for-dev → in-progress → review → done`. `last_updated` field rafraîchi.
   - Critère d'arrêt Epic 9.5 « 4/4 stories avec status `done` » → en réalité c'est 8/8 (9-5-1a/b/c/d + 9-5-2/3/4 + epic-9-5-retrospective) = 7/8 après 9-5-4 done (manque encore rétrospective).

### Test Locally First — exemption documentée

10. **Given** la story 9-5-4 est **research-only sans modification de code source** (0 fichier `.rs`, `.svelte`, `.ts` modifié — uniquement `.md`), **When** la story est en review/done, **Then** la règle CLAUDE.md `Test Locally First` est **exempte** pour cette story (cf. §"Quand sauter" dans CLAUDE.md — commits doc-only ne nécessitent pas la batterie complète). **Vérification routine seulement** : `npm run lint-i18n-ownership` resté `PASS` si une mention i18n est faite (peu probable pour ce document research), sinon aucun check.

11. **And** **0 régression introduite** par les modifications cross-stories sur 9-2a / 9-2b — vérification ground-truth : les fichiers `_bmad-output/implementation-artifacts/9-2a-*.md` + `_bmad-output/implementation-artifacts/9-2b-*.md` sont des story files documentation, leur édition ne casse rien d'exécutable.

## Tasks / Subtasks

- [ ] **T1** Pré-flight + bibliographie initiale (AC: #1, #2)
  - [ ] T1.1 Confirmer accès web (WebFetch ou WebSearch disponible dans la session) pour récupérer texte CO Art. 957a + 958f + OLICo + LSCSE.
  - [ ] T1.2 Constituer bibliographie minimale (5 primaires + 3 secondaires) — URLs + date d'accès notées.
  - [ ] T1.3 Brancher `chore/epic-9-5-planning` confirmé checkout (déjà sur cette branche post-9-5-1d done, cohérent `feedback_avoid_parallel_prs` qui bundle Epic 9.5 sur une PR unique).

- [ ] **T2** Squelette document `research-swiss-co-958f.md` (AC: #3)
  - [ ] T2.1 Créer `_bmad-output/planning-artifacts/research-swiss-co-958f.md` avec sommaire imposé AC #3 (Préambule, Art. 957a, Art. 958f, OLICo, ECH-0058, LSCSE, État Kesh, Gap analysis, Verdict, Recommandations).
  - [ ] T2.2 Préambule complet : disclaimer non-juridique + scope PME < Art. 727 + date d'analyse + bibliographie initiale (T1.2).

- [ ] **T3** Recherche Art. 957a CO — Tenue de la comptabilité (AC: #3, #5)
  - [ ] T3.1 Récupérer texte officiel Art. 957a CO consolidé via fedlex.admin.ch (alinéas 1 à 4 typiquement — vérifier numérotation actuelle).
  - [ ] T3.2 Synthétiser exigences applicables PME : tenue régulière + langue + monnaie + formats acceptés (livre journal + grand livre + comptes annuels).
  - [ ] T3.3 Mapper sur Kesh : journal_entries.rs + chart_of_accounts.rs + rapports Story 9-1 (Bilan, Pertes & Profits, Balance, Journal). Citer fichiers source précis.

- [ ] **T4** Recherche Art. 958f CO — Conservation 10 ans (AC: #3, #5)
  - [ ] T4.1 Récupérer texte officiel Art. 958f CO consolidé (alinéas 1 à 3 typiquement).
  - [ ] T4.2 Synthétiser exigences : durée 10 ans, intégrité, lisibilité durable, signature électronique qualifiée si support modifiable (§al. 3).
  - [ ] T4.3 Mapper sur Kesh : audit_log immutable + SHA-256 metadata.json + export ZIP Story 9-2b. Identifier l'écart précis avec signature qualifiée si applicable.

- [ ] **T5** Recherche OLICo + ECH-0058 (AC: #3, #5)
  - [ ] T5.1 Récupérer OLICo (RS 221.431) consolidé. Identifier sections sur supports modifiables + formats acceptés (PDF/A, XML, CSV signés).
  - [ ] T5.2 ECH-0058 standard archivage électronique : applicabilité PME (généralement non-obligatoire) + bonne pratique.
  - [ ] T5.3 Synthétiser exigences techniques : intégrité (hash + signature OU log immutable + procédure de vérification) — c'est ici que le débat « audit-trail-only suffit-il ? » se cristallise.

- [ ] **T6** Recherche LSCSE signature électronique qualifiée (AC: #3, #5)
  - [ ] T6.1 LSCSE RS 943.03 : QES (qualifiée), AES (avancée), SES (simple) — distinctions et exigences fournisseurs.
  - [ ] T6.2 Coût ordre-de-grandeur QES pour une PME (Swisscom Trust Service, QuoVadis, SwissSign — typiquement CHF 200-500/an certificat + plateforme).
  - [ ] T6.3 Applicabilité aux exports comptables Kesh : QES nécessaire ? Recommandée ? Optionnelle ? Conclusion documentée avec sources.

- [ ] **T7** Section « État de l'art Kesh » + Gap analysis (AC: #3)
  - [ ] T7.1 Synthèse précise de l'implémentation actuelle Kesh (audit_log, SHA-256, ZIP, PDF/CSV exports) — référencer story files + fichiers source.
  - [ ] T7.2 Tableau Gap analysis (Markdown) ligne par ligne — Exigence légale | État Kesh | Verdict conforme/partiel/non | Référence.
  - [ ] T7.3 Identifier les 2-3 écarts majeurs candidats à remédiation (typiquement : absence QES + horodatage tiers signé).

- [ ] **T8** Verdict + Recommandations (AC: #3, #6)
  - [ ] T8.1 Synthétiser la recherche en verdict (a / b / c) avec justification 3-5 paragraphes ancrés dans T7.2 gap analysis. Probabilité a priori (cf. epic-9-5.md Q2) : **option (b) la plus probable** (PME audit-trail SHA-256 généralement accepté).
  - [ ] T8.2 Recommandations actionables numérotées (0 à N items).
  - [ ] T8.3 **Checkpoint élicitation Guy** : si le verdict est marginalement entre (a) et (b), OU (b) et (c), poser une question explicite à Guy via `AskUserQuestion` ou texte « Verdict statué (X), confirmer avant de propager ? ». Pas de question si le verdict est net (cas attendu).

- [ ] **T9** Mise à jour cross-stories (AC: #6, #7, #8)
  - [ ] T9.1 Mettre à jour `9-2b-export-global-zip.md` §L6 avec verdict + référence document.
  - [ ] T9.2 Si applicable selon T8.1 verdict : mise à jour `9-2a-export-pdf-csv.md` (probablement non — les PDF Kesh sont conformes Art. 957a a priori, à vérifier T3.3).
  - [ ] T9.3 Mettre à jour `epic-9-5.md` §Critères d'arrêt Epic 9.5 — cocher l'item correspondant.
  - [ ] T9.4 Si verdict (b) : créer GitHub Issue `[Epic 14] Swiss CO 958f signature électronique qualifiée` avec labels `enhancement` + `v0.2-milestone` + `legal-compliance` (via `gh issue create`).
  - [ ] T9.5 Si verdict (c) : créer placeholder story `_bmad-output/implementation-artifacts/9-5-bis-swiss-co-958f-compliance.md` (frontmatter `Status: backlog` + 1 ligne scope, à élaborer hors story 9-5-4) + entrée sprint-status.yaml. **Probabilité faible**.

- [ ] **T10** Commit closure + sprint-status (AC: #9)
  - [ ] T10.1 Commit unique `docs(9-5-4): close Swiss CO 958f research with verdict (a|b|c)` — body avec résumé verdict + fichiers modifiés.
  - [ ] T10.2 `sprint-status.yaml` : `9-5-4-swiss-co-research: backlog → done` + `last_updated` field rafraîchi + comment story-history.
  - [ ] T10.3 **Test Locally First exempt** (research-only, cf. AC #10) — vérification routine : `git status` clean post-commit.

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
- **T8 (Verdict + Recommandations)** : 20-40 min + checkpoint Guy.
- **T9 (cross-stories updates)** : 15-30 min selon verdict.
- **T10 (commit + sprint-status)** : 10 min.
- **Total** : ~3-5h, **research subagent parallélisable T3+T4+T5 pour économiser ~1h**.

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

À déterminer au lancement `bmad-dev-story 9-5-4` (recommandé Opus 4.7 ou Sonnet 4.6, **différent de la passe validate** suivant la règle CLAUDE.md `Review Iteration Rule`).

### Debug Log References

À compléter lors du dev-story.

### Completion Notes List

À compléter lors du dev-story.

### File List

À compléter lors du dev-story.

## Change Log

À compléter lors des passes spec validate + dev-story + code-review.
