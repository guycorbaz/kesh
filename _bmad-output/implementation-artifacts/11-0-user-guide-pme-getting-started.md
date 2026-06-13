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
- **Format** : PNG, stockage **`docs/manual/fr/screenshots/`** (langue-spécifique : les captures DE/IT/EN auront leur propre dossier ; `docs/manual/shared/` reste réservé aux assets cross-manuels comme le logo et le style). Ajouter `\graphicspath{{./screenshots/}}` dans le préambule de `user-manual.tex` (après le chargement de `../shared/kesh-style`). `graphicx` est déjà chargé par `kesh-style.sty` (l.28).
- **Anonymisation** : données factices (Pierre Exemple, ABC Sàrl, IBAN CH00…) — PAS de données perso Guy.
- **Quantité** : ~20-25 (3 onboarding, 3 plan compta, 5 facture, 5 banking, 3 rapports, 5 réconciliation).
- **Alt-text LaTeX** : `\caption` descriptif + commentaire structurel ; pas d'alt-text HTML natif en LaTeX, le caption joue ce rôle.

## Acceptance Criteria

### Screenshots (AC #1-3)

- [ ] **AC #1** ~20-25 environnements `figure` ajoutés dans `user-manual.tex` aux emplacements pertinents, avec `\includegraphics` (placeholders initialement), `\caption` descriptif et commentaire `% TODO capture: <description>`.
- [ ] **AC #2** `\graphicspath{{./screenshots/}}` ajouté au préambule de `user-manual.tex` ; dossier `docs/manual/fr/screenshots/` créé avec convention de nommage `<feature-slug>-<N>.png`. Un placeholder neutre committé (cf. AC#9 / Risques) garantit la compilation tant que les vraies captures ne sont pas fournies.
- [ ] **AC #3** Données factices anonymisées prévues dans les captions (Pierre Exemple, ABC Sàrl, etc.). PAS de données perso Guy.

### Exemples sectoriels PME (AC #4)

- [ ] **AC #4** Sections « Plan comptable » et « Facturation » enrichies d'au moins 3 exemples PME sectoriels concrets (commerce de détail, services/agence, artisan ou cabinet médical, association). Exemples intégrés au fil du texte, pas en bloc isolé.

### Glossaire (AC #5)

- [ ] **AC #5** Glossaire comptable (§Annexes l. 977) complété à ~30 termes (ajout : actif, passif, charge, produit, grand-livre, journal, lettrage, amortissement, provision, capitaux propres, créance, dette, report à nouveau, audit-trail, etc.). Glossaire **(ré)ordonné** alphabétiquement sur l'ensemble — le glossaire actuel n'est PAS strictement trié (`Échéance`/`Exercice`/`Écriture` l.983-985 sont dans le désordre) : corriger l'ordre en passant. Convention : accents ignorés (É=E), insensible à la casse. Pas de doublon avec les ~12 termes déjà présents.

### Rafraîchissement v0.2.0 (AC #6-7)

- [ ] **AC #6** Section « Limites actuelles » (l. 1010-1019) rafraîchie v0.2.0 : retirer ce qui est livré (recovery, export/import installation, PAT), conserver ce qui reste à venir (TVA Epic 11, avoirs/pain.001 Epic 12, QES Epic 14, multi-devises v0.2, PDF/A v0.2). Marqueurs de version cohérents.
- [ ] **AC #7** Corps du manuel vérifié et **corrigé** contre le ground-truth v0.2.0 :
  - (a) onboarding : §Onboarding (l.133) ne contredit pas le flux réel v0.2.0 (8 écrans UI) ; ne pas réécrire les 7 étapes métier du Chemin B si exactes (cf. T1.1) ;
  - (b) **corriger** la ligne 511 (§Génération du PDF avec QR Bill) qui sur-promet « fonctionnalité d'envoi direct à venir v0.2 » — cette fonctionnalité n'existe pas en v0.2.0 (vérifié : aucune route d'envoi de facture dans `crates/kesh-api/src/routes/`, le mailer ne sert qu'au recovery). Reformuler en « Vous pouvez l'envoyer manuellement par email ou l'imprimer. » ;
  - (c) **corriger** la §Avoirs et notes de crédit (l.529-534) qui décrit une navigation `Factures → Nouvel avoir` inexistante en v0.2.0 (aucune route `credit_note`) — aligner le texte principal sur la `keshnote` l.534 (avoirs = contre-passation manuelle d'écritures), supprimer la description de menu spéculative ;
  - (d) **clés API / export-import `.keshbackup`** : ces fonctions sont déjà documentées dans `admin-manual.tex` (`.keshbackup` l.1298 réservé Admin anti-PAT ; clés PAT l.1458 accessibles Comptable+Admin). Décision : ne PAS dupliquer dans user-manual. Ajouter au plus une brève mention de renvoi dans §Gestion de votre compte (l.192) pour les clés API (« Pour l'accès programmatique par clé API, voir `docs/api-external.md` et le manuel administrateur »), et rien pour `.keshbackup` (purement admin).

### Intégration projet (AC #8-10)

- [ ] **AC #8** Référence aux manuels ajoutée dans `README.md`. **`README.md` n'a pas de section « Documentation »** (sections actuelles : Fonctionnalités, Pile technique, Démarrage rapide, Structure du projet, Architecture, Développement, Tests, Feuille de route, Contribuer, Licence). → Créer une nouvelle section `## Documentation` (placée après `## Structure du projet`) listant : Manuel administrateur (`docs/manual/fr/admin-manual.pdf`), Manuel utilisateur **complet** (`docs/manual/fr/user-manual.pdf`), Guide de **démarrage rapide** (`docs/user-guide/fr/getting-started.md`) — distinguer explicitement les deux pour éviter la confusion (quick-start vs manuel de référence). Mettre à jour la « Table des matières » (l.11) en conséquence.
- [ ] **AC #9** Variables de version LaTeX mises à jour dans `docs/manual/shared/kesh-style.sty` (l.64-66) : `\keshVersion{0.2.0}`, `\keshReleaseDate{2026-06-12}`, `\keshTargetRelease{v0.2}` (actuellement `0.1.0-dev` / `2026-05-20` / `v0.1`, affichées en header/footer/couverture l.179/186/444/464). **Ce fichier est partagé par les 3 manuels** → régénérer **et committer les 3 PDFs** (`admin-manual.pdf`, `user-manual.pdf`, `marketing-brochure.pdf`).
- [ ] **AC #10** PDF(s) régénéré(s) via la commande de build du projet (`make -C docs/manual user` ou les 3 cibles si AC#9 touche la version partagée) sans erreur de compilation LaTeX. Un placeholder neutre `docs/manual/fr/screenshots/_placeholder.png` est committé pour que la compilation passe tant que les vraies captures manquent.

### Effet de bord version partagée (AC #12)

- [ ] **AC #12** Le bump `keshVersion → 0.2.0` (AC#9) estampille « 0.2.0 » sur les 3 manuels via `kesh-style.sty`. **`marketing-brochure.tex` a une feuille de route périmée** (vérifié : l.373 « Déjà livré Epic 1 à 9.5 », l.390 « En préparation (Epic 10) » alors qu'Epic 10 clos, l.398 « Backlog v0.1 (Epic 11 à 15) », l.410-414 « Vision v0.2 : clés PAT » alors que PAT **livré** v0.2.0). Régénérer ce PDF estampillé 0.2.0 publierait un artefact mensonger. → Rafraîchir a minima la feuille de route marketing (l.~373-419) : déplacer recovery + clés PAT + export/import de « Vision v0.2 » vers « Déjà livré v0.2.0 » ; corriger « En préparation (Epic 10) » ; ré-étiqueter « Backlog v0.1 » → « Backlog v0.2 ». **Alternative (au choix de Guy)** : ne pas toucher la brochure dans cette story → alors NE PAS bumper la version dans le `kesh-style.sty` partagé (overrider `\keshVersion` localement dans le préambule de `user-manual.tex` seul via `\renewcommand`), créer une issue GitHub `documentation` pour la sync marketing, et documenter la dérogation. Décision tranchée en dev-story / par Guy.

### Validation Guy (AC #13)

- [ ] **AC #13** Guy relit le manuel enrichi et confirme qu'il reflète son expérience réelle Kesh v0.2.0. Identifie sections manquantes/ambiguës/incorrectes. Itération 1-2 passes si nécessaire. Sign-off : « ce manuel reflète mon usage réel Kesh v0.2.0 ».

## Tasks / Subtasks

- [ ] **T1 — Gap audit + corrections du corps existant** (AC #7)
  - [ ] T1.1 Vérifier que §Onboarding (l. 133) **ne contredit pas** le flux réel v0.2.0 (8 écrans UI : langue UI → mode UI → path A/B → type org → langue comptable → coordonnées → compte bancaire → finalisation). **NE PAS réécrire** la description actuelle des 7 étapes métier du Chemin B si elle reste exacte (elle décrit les champs saisis, pas les écrans — découpage valide et pédagogique). Corriger uniquement si un champ documenté n'existe plus ou si une étape réelle manque. Les « 8 étapes » sont un repère de vérification, pas un format d'écriture imposé.
  - [ ] T1.2 **Corriger** §Génération du PDF (l. 511) : retirer « (fonctionnalité d'envoi direct à venir v0.2) », reformuler en envoi manuel.
  - [ ] T1.3 **Corriger** §Avoirs et notes de crédit (l. 529-534) : supprimer la navigation `Factures → Nouvel avoir` inexistante, aligner sur la `keshnote` (contre-passation manuelle).
  - [ ] T1.4 Clés API : ajouter une brève mention de renvoi dans §Gestion de votre compte (l. 192) vers `docs/api-external.md` + admin-manual. Pas de doc `.keshbackup` dans user-manual (admin-only, déjà admin-manual l.1298).
- [ ] **T2 — Exemples sectoriels PME** (AC #4)
  - [ ] T2.1 Plan comptable : exemples par secteur (commerce, services, artisan).
  - [ ] T2.2 Facturation : exemple concret de facture PME (avec QR Bill) par secteur.
- [ ] **T3 — Glossaire** (AC #5)
  - [ ] T3.1 Compléter à ~30 termes, **réordonner** tout le glossaire alphabétiquement (corriger le désordre existant l.983-985), sans doublon.
- [ ] **T4 — Screenshots (placeholders texte-d'abord)** (AC #1-3)
  - [ ] T4.1 Ajouter `\graphicspath{{./screenshots/}}` au préambule de `user-manual.tex` ; créer `docs/manual/fr/screenshots/` + committer un `_placeholder.png` neutre (garantit la compilation).
  - [ ] T4.2 Insérer ~20-25 `figure` (`\includegraphics{_placeholder}` par défaut) + captions descriptifs + `% TODO capture`.
  - [ ] T4.3 (passe ultérieure Guy) Remplacer placeholders par PNG réels prod NAS anonymisés.
- [ ] **T5 — Rafraîchissement version + « Limites actuelles »** (AC #6, #9)
  - [ ] T5.1 Mettre à jour §Limites actuelles l. 1010-1019 selon ground-truth v0.2.0.
  - [ ] T5.2 Bumper `kesh-style.sty` l.64-66 : `keshVersion 0.2.0`, `keshReleaseDate 2026-06-12`, `keshTargetRelease v0.2`.
- [ ] **T6 — Intégration + build** (AC #8, #9, #10)
  - [ ] T6.1 Référence README section Documentation.
  - [ ] T6.2 Régénérer les PDFs impactés (`make -C docs/manual user` ; + `admin` et `marketing` car version partagée bumpée en T5.2), vérifier compilation, committer les .pdf.
  - [ ] T6.3 (AC#12) Traiter l'effet de bord du bump version sur la brochure marketing : soit rafraîchir sa feuille de route (l.~373-419), soit déroger (override version local `user-manual.tex` seul + issue GitHub + dérogation documentée). Décision Guy/dev-story.
- [ ] **T7 — Validation Guy** (AC #13)
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
- **Compilation LaTeX** : commande canonique projet **`make -C docs/manual user`** (le Makefile utilise `xelatex` 2 passes, cf. `docs/manual/Makefile` l.19/69). `latexmk -xelatex` est équivalent si préféré. Doit compiler sans erreur. Un placeholder neutre committé (`docs/manual/fr/screenshots/_placeholder.png`) évite l'échec `File not found` tant que les vraies captures manquent. Si T5.2 bumpe la version partagée : recompiler aussi `make -C docs/manual admin marketing`.
- **Liens** : vérifier les `\href` (liens externes) et `\ref`/`\hyperref` internes.

### Risques

- **Placeholders d'images cassant la compilation** : committer un PNG placeholder neutre (`docs/manual/fr/screenshots/_placeholder.png`) référencé par défaut, pour que la compilation passe tant que les vraies captures ne sont pas là. Sinon `\includegraphics` d'un fichier absent fait échouer la compilation.
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

### Validate Pass 1 (Sonnet 4.6, 2026-06-13)

6 findings (1 CRITICAL, 2 HIGH, 2 MEDIUM, 1 LOW), tous vérifiés ground-truth (grep/Read sur `user-manual.tex`, `kesh-style.sty`, `admin-manual.tex`, `crates/kesh-api`). Tous patchés :
- **F1 CRITICAL** — chemin screenshots non tranché (risque compilation cassée) → fixé `docs/manual/fr/screenshots/` + `\graphicspath` préambule (AC#2, §Screenshots, T4.1).
- **F2 HIGH** — l.511 sur-promet « envoi direct à venir v0.2 » (fonction inexistante v0.2.0) → AC#7b passe de « vérifier » à « corriger » + T1.2.
- **F3 HIGH** — `keshVersion=0.1.0-dev` affiché header/footer/couverture (partagé 3 manuels) → nouvel AC#9 bump version + régén 3 PDFs, T5.2.
- **F4 MEDIUM** — décision PAT/`.keshbackup` (user vs admin) → tranchée AC#7d : déjà dans admin-manual (l.1298/1458), pas de duplication, simple renvoi clés API §Gestion compte. T1.4.
- **F5 MEDIUM** — §Avoirs l.529-534 décrit UI `Factures → Nouvel avoir` inexistante → AC#7c corrige, T1.3.
- **F6 LOW** — commande build `make -C docs/manual user` (canonique) vs `latexmk` → clarifié §Test Locally First + AC#10/T6.2.

ACs 10 → 11 (ajout version bump + build séparés). Prochaine étape : Pass 2 (Haiku 4.5, contexte frais, diff unique).

### Validate Pass 2 (Haiku 4.5, 2026-06-13)

0 CRITICAL, 0 HIGH, 2 MEDIUM, reste LOW. **Aucune hallucination** (tous les paths/lignes/ACs vérifiés grep/Read confirmés). 2 MEDIUM patchés :
- **M1 MEDIUM** — T7 référençait « AC #10 » alors que la validation Guy est AC #11 (décalage post-Pass1) → corrigé.
- **M2 MEDIUM** — AC#8 supposait une section « Documentation » dans README qui n'existe pas (vérifié `grep '^##'`) → AC#8 précise de créer la section `## Documentation` après `## Structure du projet` + MAJ Table des matières.

Findings résiduels LOW seulement (glossaire « etc. » = latitude dev acceptable). Prochaine étape : Pass 3 (Opus 4.8, contexte frais) pour confirmer convergence.

### Validate Pass 3 (Opus 4.8, 2026-06-13)

1 HIGH, 2 MEDIUM, 2 LOW — findings de **second ordre** dans les angles morts de Sonnet/Haiku, tous vérifiés ground-truth. Tous patchés :
- **F-OPUS-1 HIGH** — le bump version partagé (AC#9) force la régén de `marketing-brochure.pdf`, dont la feuille de route est périmée (PAT/recovery listés « Vision v0.2 » alors que livrés v0.2.0 ; « En préparation Epic 10 » clos ; « Backlog v0.1 » mal étiqueté). Estampiller 0.2.0 dessus = artefact mensonger → nouvel **AC#12** (rafraîchir roadmap marketing OU déroger avec override version local + issue), T6.3.
- **F-OPUS-2 MEDIUM** — AC#5 « tri alphabétique conservé » conservait un tri inexistant (`Échéance`/`Exercice`/`Écriture` l.983-985 désordonnés) → AC#5/T3.1 passent à « (ré)ordonner ».
- **F-OPUS-3 MEDIUM** — T1.1 « aligner sur 8 étapes » = travail fantôme / risque de réécriture régressive (le manuel décrit 7 étapes métier Chemin B, valides ; « 8 étapes » = flux UI, découpage différent) → T1.1 + AC#7a clarifiés « vérifier ne contredit pas, ne pas réécrire si exact ».
- **F-OPUS-4 LOW** — getting-started.md vs user-manual dans README → distinguer « démarrage rapide » vs « manuel complet ».
- **F-OPUS-5 LOW** — heading « Intégration projet (AC #8-9) » → « (AC #8-10) ».

ACs 11 → 13 (ajout AC#12 effet de bord version + décalage validation Guy en #13). Pass 3 = NON convergé (1 HIGH + 2 MEDIUM > LOW). Prochaine étape : Pass 4 (Sonnet 4.6, contexte frais) — cycle Sonnet→Haiku→Opus→Sonnet.

Status : `backlog`.
