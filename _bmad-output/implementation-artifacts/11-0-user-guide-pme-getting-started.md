# Story 11.0: Enrichissement du manuel utilisateur PME Kesh (Epic 11 prep)

Status: backlog

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## ⚠️ Correction de cap majeure (2026-06-13)

**Cette story a été re-scopée.** La version initiale (2026-05-31) demandait de **créer ex nihilo** un guide PME (`pme-guide.md` ou `.tex`) en partant du principe que la documentation utilisateur PME n'existait pas (dette catégorie A).

**Cette prémisse était fausse.** Le manuel utilisateur LaTeX `docs/manual/fr/user-manual.tex` (1043 lignes) existe depuis le **2026-05-20** (PR #102, commit `47cb02f`) — soit **11 jours avant** la création de la story 11-0. La spec initiale ne mentionnait que `admin-manual.tex` comme « existant, ne pas dupliquer » et avait **oublié `user-manual.tex`**. Ce manuel a même été maintenu à jour pour v0.2.0 (section recovery ajoutée via Epic 17-4, commit `0d3375c`).

**Constat ground-truth (vérifié 2026-06-13)** : `user-manual.tex` couvre déjà ~85 % de l'intention de la story (les 7 axes de contenu, glossaire, framing PME, conformité OLICo, et même du hors-scope bonus : multi-tenant fiduciaires, avoirs, FAQ). Créer un nouveau `pme-guide.tex` serait une **duplication** (violation DRY + double maintenance).

**Décision (Guy, 2026-06-13)** : rescoper 11-0 en **story d'enrichissement** ciblée sur les écarts réels du manuel existant. Pas de nouveau fichier. La dette catégorie A « doc utilisateur PME » est considérée **essentiellement soldée par `user-manual.tex`** ; cette story ferme les derniers écarts.

## Story

As a **fiduciaire ou comptable PME suisse qui découvre Kesh**,
I want **un manuel utilisateur complet, illustré de captures d'écran et enrichi d'exemples PME concrets**,
so that je puisse adopter Kesh sans formation externe ni essai-erreur, et qu'un nouveau collaborateur puisse se former en autonomie.

## Scope

**Severity : dette catégorie A héritée Epic 9.5 retro action item #3** — **essentiellement soldée** par `docs/manual/fr/user-manual.tex` (PR #102). Cette story ferme les écarts résiduels avant kickoff Epic 11 TVA.

**Cible unique d'édition : `docs/manual/fr/user-manual.tex`** (enrichir l'existant, ne PAS créer de nouveau fichier guide).

### Dans le scope — les 5 écarts réels (gap analysis 2026-06-13)

1. **Screenshots (écart principal)** — le manuel contient actuellement **0 `\includegraphics`**. Ajouter ~20-25 captures d'écran annotées illustrant les écrans clés (onboarding, dashboard, plan comptable, saisie d'écriture, facture/QR Bill, échéancier, import bancaire, réconciliation, rapports, exports). **Approche texte-d'abord** : insérer d'abord les environnements `figure` avec `\includegraphics` pointant vers des fichiers placeholders + `\caption` + commentaire `% TODO capture: <description>` ; les PNG réels seront fournis par Guy depuis sa prod NAS en passe d'itération séparée (cf. T4).

2. **Exemples sectoriels PME** — le manuel est actuellement générique. Enrichir les sections « Plan comptable » et « Facturation » avec des exemples concrets par type de PME suisse (commerce de détail, cabinet médical, agence web, artisan, association). Pas d'« entreprise X / facture Y » abstrait.

3. **Glossaire** — actuellement ~12 termes (`docs/manual/fr/user-manual.tex` §Glossaire comptable, l. 977). Compléter à ~30 termes comptables suisses (ajouter : actif, passif, charge, produit, grand-livre, journal, lettrage/réconciliation, amortissement, provision, capitaux propres, liquidités, créance, dette, TVA déductible/collectée, exercice de clôture, report à nouveau, audit-trail, etc.).

4. **Référence README** — `README.md` ne référence actuellement pas le manuel utilisateur. Ajouter une entrée dans la section Documentation (« Manuel utilisateur FR : `docs/manual/fr/user-manual.pdf` »).

5. **Rafraîchissement section « Limites actuelles »** — `user-manual.tex` l. 1010-1019 est périmée (datée v0.1, marqueurs incohérents type « Epic 11 v0.1 »). Mettre à jour selon l'état v0.2.0 réel (cf. ground-truth ci-dessous) : recovery livré, export/import `.keshbackup` livré, PAT livré, httpOnly tokens livrés ; TVA reste à venir (Epic 11), avoirs/pain.001 à venir (Epic 12), QES à venir (Epic 14).

### Validation finale

6. **Régénération PDF** — recompiler `user-manual.pdf` via `latexmk -xelatex` après enrichissement, et le committer (convention projet : versionner les PDFs, cf. PR #102).

7. **Sign-off Guy** — Guy relit le manuel enrichi et confirme qu'il reflète son expérience d'utilisation réelle Kesh v0.2.0 sur sa prod NAS. Cycle d'itération 1-2 passes si nécessaire.

### Hors scope

- **Création d'un nouveau fichier guide** (`pme-guide.md` / `.tex`) — explicitement abandonné (duplication de `user-manual.tex`).
- **TVA** (Epic 11) : chapitre additionnel post-Epic 11, pas pré-écrit ici.
- **Avoirs / pain.001 / paiements** (Epic 12) : idem chapitre additionnel post-Epic.
- **Manuel administrateur** (`docs/manual/fr/admin-manual.tex`) : déjà couvert, référencer mais ne pas dupliquer.
- **Traductions DE/IT/EN** (`docs/manual/{de,en,it}/`) : v0.2+ une fois le FR stable. Actuellement README placeholders.
- **Réécriture du contenu déjà correct** des 7 axes : on enrichit (screenshots + exemples + glossaire), on ne réécrit pas ce qui est déjà juste.

## Contexte technique (ground-truth v0.2.0, vérifié 2026-06-13)

### État du manuel existant `user-manual.tex`

Sections déjà présentes (l. = ligne dans `docs/manual/fr/user-manual.tex`) :

| Section | Ligne | Couvre l'axe 11-0 |
|---------|-------|-------------------|
| Bienvenue dans Kesh | 31 | Concepts (partiel) |
| Premiers pas (compte, connexion, **recovery**, langue) | 55 | Onboarding (partiel) |
| Onboarding (Chemin A/B) | 133 | Axe 2 |
| Tableau de bord | 172 | — |
| Gestion de votre compte | 192 | — |
| Plan comptable (visu, ajout, modif, suppr, import custom) | 222 | Axe 3 |
| Écritures comptables (partie double, exemple, validation) | 277 | Axe 1 |
| Exercices comptables | 363 | Concepts |
| Carnet d'adresses (contacts, catalogue, conditions paiement) | 403 | Axe 4 |
| Facturation QR Bill (création, validation, PDF, échéancier, **avoirs**) | 463 | Axe 4 |
| Import bancaire (CAMT.053, CSV, doublons, rejet partiel) | 538 | Axe 5 |
| Réconciliation (auto, manuel, split, règles) | 620 | Axe 5 |
| Rapports comptables (bilan, résultat, balance, journal) | 709 | Axe 6 |
| Exports de souveraineté (ZIP, intégrité, OLICo 10 ans) | 755 | Axe 6 + 7 |
| Multi-tenant fiduciaires | 820 | bonus |
| Notifications et audit (audit-trail) | 839 | Axe 7 |
| FAQ et dépannage | 873 | bonus |
| Annexes (Glossaire, raccourcis, limites, liens) | 975 | Axe 2 / divers |

### Features v0.2.0 (post-Epic 17) — à refléter dans la section « Limites » et vérifier dans le corps

- **Recovery mot de passe self-service** : `/forgot-password` + `/reset-password`, gated `KESH_FEATURE_FORGOT_PASSWORD`, SMTP (crate `lettre`), token TTL 30 min, anti-énumération. **Déjà documenté** §Premiers pas l. 81.
- **SMTP intégré** : utilisé **uniquement** pour le recovery. **L'envoi de facture reste manuel** (téléchargement PDF `GET /api/v1/invoices/{id}/pdf` + client mail externe) — à vérifier que le manuel ne sur-promet pas un envoi email intégré.
- **Export/import complet d'installation** `.keshbackup` : `GET /api/v1/admin/full-export` + `POST /api/v1/admin/full-import`, pages `/admin/backup` + `/admin/restore`, intégrité SHA-256, backup auto pré-import, anti-PAT. **Distinct** de l'export global de souveraineté par société (`GET /api/v1/exports/global.zip`, §Exports de souveraineté l. 755). **Écart possible** : vérifier si l'export/import complet d'installation est documenté ; sinon ajouter une sous-section (probablement plutôt dans `admin-manual.tex` car réservé admin — à trancher en validate).
- **PAT / clés API** : `/settings/api-keys`, scopes `read`/`read-write`, secret one-time. **Écart possible** : documentation utilisateur des clés API (renvoi vers `docs/api-external.md`).
- **httpOnly tokens** (KF-002 Story 10-5) : auth par cookies httpOnly, `KESH_COOKIE_SECURE`. Détail infra → plutôt `admin-manual.tex`.
- **Onboarding 8 étapes** + `/setup/admin` initial. Vérifier que §Onboarding (l. 133) est aligné (8 étapes réelles : langue UI, mode UI, path A/B, type org, langue comptable, coordonnées, compte bancaire, finalisation).

### Référence documents ground-truth

- `_bmad-output/planning-artifacts/research-swiss-co-958f.md` — Swiss CO Art. 957a/958f + OLICo Art. 6-13 (déjà référencé §Exports de souveraineté).
- `docs/manual/fr/admin-manual.tex` (1887 l.) — install/config/sécurité infra (référencer, ne pas dupliquer).
- `docs/manual/shared/kesh-style.sty` — style LaTeX partagé (macros `\keshkey`, `\keshpath`, `\keshtableheader`, `\keshversionnote`) à réutiliser pour cohérence.
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — clés `tooltip-*` (définitions inline) à référencer plutôt que dupliquer.

### Screenshots — production (inchangé vs spec initiale)

- **Approche** : texte-d'abord. Insérer `figure` + placeholder + `% TODO capture`. Guy fournit les PNG depuis sa prod NAS v0.2.0 en passe séparée.
- **Outil** : flameshot / screenshot natif. **Résolution** : crop sur contenu pertinent.
- **Format** : PNG, stockage `docs/manual/shared/screenshots/` ou `docs/manual/fr/screenshots/` (à trancher validate — cohérence `\graphicspath`).
- **Anonymisation** : données factices (Pierre Exemple, ABC Sàrl, IBAN CH00…) — PAS de données perso Guy.
- **Quantité** : ~20-25 (3 onboarding, 3 plan compta, 5 facture, 5 banking, 3 rapports, 5 réconciliation).
- **Alt-text LaTeX** : `\caption` descriptif + commentaire structurel ; pas d'alt-text HTML natif en LaTeX, le caption joue ce rôle.

## Acceptance Criteria

### Screenshots (AC #1-3)

- [ ] **AC #1** ~20-25 environnements `figure` ajoutés dans `user-manual.tex` aux emplacements pertinents, avec `\includegraphics` (placeholders initialement), `\caption` descriptif et commentaire `% TODO capture: <description>`.
- [ ] **AC #2** `\graphicspath` configuré ; dossier screenshots créé avec convention de nommage `<feature-slug>-<N>.png`.
- [ ] **AC #3** Données factices anonymisées prévues dans les captions (Pierre Exemple, ABC Sàrl, etc.). PAS de données perso Guy.

### Exemples sectoriels PME (AC #4)

- [ ] **AC #4** Sections « Plan comptable » et « Facturation » enrichies d'au moins 3 exemples PME sectoriels concrets (commerce de détail, services/agence, artisan ou cabinet médical, association). Exemples intégrés au fil du texte, pas en bloc isolé.

### Glossaire (AC #5)

- [ ] **AC #5** Glossaire comptable (§Annexes l. 977) complété à ~30 termes (ajout : actif, passif, charge, produit, grand-livre, journal, lettrage, amortissement, provision, capitaux propres, créance, dette, report à nouveau, audit-trail, etc.). Tri alphabétique conservé.

### Rafraîchissement v0.2.0 (AC #6-7)

- [ ] **AC #6** Section « Limites actuelles » (l. 1010-1019) rafraîchie v0.2.0 : retirer ce qui est livré (recovery, export/import installation, PAT), conserver ce qui reste à venir (TVA Epic 11, avoirs/pain.001 Epic 12, QES Epic 14, multi-devises v0.2, PDF/A v0.2). Marqueurs de version cohérents.
- [ ] **AC #7** Corps du manuel vérifié contre le ground-truth v0.2.0 : (a) onboarding aligné sur 8 étapes réelles, (b) aucune sur-promesse d'envoi de facture par email intégré (rester sur « PDF + client mail externe »), (c) décision validate sur où documenter export/import `.keshbackup` (user vs admin manual) et clés API.

### Intégration projet (AC #8-9)

- [ ] **AC #8** Référence ajoutée dans `README.md` section Documentation : « Manuel utilisateur FR : `docs/manual/fr/user-manual.pdf` ».
- [ ] **AC #9** PDF régénéré (`latexmk -xelatex` dans `docs/manual/fr/`) et committé. Pas d'erreur de compilation LaTeX.

### Validation Guy (AC #10)

- [ ] **AC #10** Guy relit le manuel enrichi et confirme qu'il reflète son expérience réelle Kesh v0.2.0. Identifie sections manquantes/ambiguës/incorrectes. Itération 1-2 passes si nécessaire. Sign-off : « ce manuel reflète mon usage réel Kesh v0.2.0 ».

## Tasks / Subtasks

- [ ] **T1 — Gap audit fin du corps existant** (AC #7)
  - [ ] T1.1 Relire §Onboarding (l. 133) et aligner sur les 8 étapes réelles si écart.
  - [ ] T1.2 Vérifier §Facturation (l. 463) : pas de sur-promesse envoi email intégré.
  - [ ] T1.3 Trancher (validate) où documenter export/import `.keshbackup` (user vs admin) et clés API.
- [ ] **T2 — Exemples sectoriels PME** (AC #4)
  - [ ] T2.1 Plan comptable : exemples par secteur (commerce, services, artisan).
  - [ ] T2.2 Facturation : exemple concret de facture PME (avec QR Bill) par secteur.
- [ ] **T3 — Glossaire** (AC #5)
  - [ ] T3.1 Compléter à ~30 termes, tri alphabétique.
- [ ] **T4 — Screenshots (placeholders texte-d'abord)** (AC #1-3)
  - [ ] T4.1 Configurer `\graphicspath` + créer dossier screenshots + `.gitkeep`.
  - [ ] T4.2 Insérer ~20-25 `figure` placeholders + captions + `% TODO capture`.
  - [ ] T4.3 (passe ultérieure Guy) Remplacer placeholders par PNG réels prod NAS anonymisés.
- [ ] **T5 — Rafraîchissement « Limites actuelles »** (AC #6)
  - [ ] T5.1 Mettre à jour l. 1010-1019 selon ground-truth v0.2.0.
- [ ] **T6 — Intégration + build** (AC #8-9)
  - [ ] T6.1 Référence README section Documentation.
  - [ ] T6.2 Régénérer `user-manual.pdf` (`latexmk -xelatex`), vérifier compilation, committer.
- [ ] **T7 — Validation Guy** (AC #10)
  - [ ] T7.1 Guy relit le manuel enrichi (PDF).
  - [ ] T7.2 Itération 1-2 passes correction.
  - [ ] T7.3 Sign-off Guy.

## Dev Notes

### Pré-requis avant kickoff

1. **Déploiement Kesh v0.2.0 sur prod NAS Guy** — ✅ fait (v0.2.0 publié Docker Hub 2026-06-12, NAS à jour). Permet les captures dans des conditions réelles.
2. **Usage réel par Guy** pour retour d'expérience direct sur les frictions à documenter.

### Workflow review — exigeant comme une story code

Cohérent Epic 9.5 retro lesson C1 : « les stories doc-only ne sont PAS triviales ». Cycle attendu :
- Spec validate : 2-3 passes (Sonnet → Haiku → optionnel Opus). **Réduit vs spec initiale** car le scope est désormais un enrichissement ciblé, pas une création complète.
- Dev-story : Sonnet 4.6 ou Opus selon complexité (édition LaTeX ciblée).
- Code-review : 2 passes (Sonnet → Haiku) sur : accuracy ground-truth v0.2.0, compilation LaTeX verte, cohérence glossaire, qualité captions, exemples sectoriels crédibles.

**Différence vs story code** : pas de cargo/clippy/tests. La review porte sur accuracy / completeness / coherence / pedagogy + **compilation LaTeX** (`latexmk -xelatex` doit passer).

### Test Locally First (adapté doc LaTeX)

- **N/A code** : pas de cargo/npm.
- **Compilation LaTeX** : `cd docs/manual/fr && latexmk -xelatex user-manual.tex` doit compiler sans erreur (warnings de placeholder image acceptables jusqu'à ce que les PNG soient fournis — prévoir un placeholder neutre committé pour que le PDF compile entre-temps).
- **Liens** : vérifier les `\href` (liens externes) et `\ref`/`\hyperref` internes.

### Risques

- **Placeholders d'images cassant la compilation** : committer un PNG placeholder neutre (`docs/manual/shared/screenshots/_placeholder.png`) référencé par défaut, pour que `latexmk` compile tant que les vraies captures ne sont pas là. Sinon `\includegraphics` d'un fichier absent fait échouer la compilation.
- **Évolution Kesh pendant rédaction** : v0.2.0 stable, pas de hotfix prévu → risque faible. Freezer si Epic 11 démarre en parallèle.
- **Périmètre dérivant** : ne PAS réécrire le contenu déjà correct. Enrichissement ciblé uniquement (5 écarts).

### Référence

- `docs/manual/fr/user-manual.tex` (1043 l., cible d'édition).
- `docs/manual/fr/admin-manual.tex` (1887 l., référence style + frontière admin/user).
- `docs/manual/shared/kesh-style.sty` (macros partagées).
- `docs/manual/Makefile` (build).
- `_bmad-output/planning-artifacts/research-swiss-co-958f.md` (compliance).
- Action item #3 Epic 9.5 retro 2026-05-20 (dette héritée, essentiellement soldée par PR #102).

## Dev Agent Record

### Agent Model Used

_(à remplir au dev-story)_

### Debug Log References

_(à remplir au dev-story)_

### Completion Notes List

_(à remplir au dev-story)_

### File List

_(à remplir au dev-story)_

## Change Log

### Create-story (2026-05-31)

Story 11-0 créée comme prérequis Epic 11 prep (carry-forward action item #3 Epic 9.5 retro + #2 Hotfix v0.1.1 retro — dette catégorie A). Demandait la **création** d'un guide PME complet. **Décisions ouvertes** : format (markdown/LaTeX/hybride), 1 vs 2 guides, périmètre screenshots.

### Re-scope — correction de cap (2026-06-13)

**Découverte** : `docs/manual/fr/user-manual.tex` (1043 l., PR #102 du 2026-05-20) existait déjà et couvre ~85 % de l'intention de la story — la spec initiale l'avait oublié (ne mentionnait que `admin-manual.tex`). La prémisse « doc PME inexistante » était fausse.

**Décision Guy** : rescoper en **story d'enrichissement** du manuel existant (PAS de nouveau fichier `pme-guide`). Dette cat A « doc utilisateur PME » considérée essentiellement soldée par PR #102 ; cette story ferme les 5 écarts résiduels :
1. Screenshots (0 → ~20-25, approche texte-d'abord).
2. Exemples sectoriels PME.
3. Glossaire (~12 → ~30 termes).
4. Référence README.
5. Rafraîchissement section « Limites actuelles » v0.1 → v0.2.0.

**Décisions tranchées (Guy, 2026-06-13)** :
- Format : LaTeX (cible = `user-manual.tex` existant, donc déjà LaTeX).
- Screenshots : texte-d'abord, captures NAS en passe séparée.
- `getting-started.md` : garder la version de `main` (94 l.) ; version branche `chore/post-v0.1.2-prep` (264 l.) abandonnée.

ACs réduits de 18 → 10, recentrés sur les écarts. Branche : `story/11-0-user-guide-pme` (fraîche depuis `main` v0.2.0).

Status : `backlog`. Prochaine étape : `bmad-create-story validate 11-0-user-guide-pme-getting-started` (2-3 passes attendues vu le scope réduit).
