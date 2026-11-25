# Story 55-4 : solde de la dette de l'epic

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md`. Release : celle de l'epic, **avant le tag**.
Règle : `CLAUDE.md` § *Priorités des défauts*, les puces « Un LOW de revue qui touche la production peut se corriger dans
la story » et « Une story de solde à la fin de chaque epic » (décisions de Guy du 2026-11-14).

## ⚠️ Points renvoyés au développement (validation P8, dernière passe)

- **R-1 (V8-1, MEDIUM) — Argument de `gate-ci.sh`.** `scripts/gate-ci.sh origin/main` est refusé (code 2) : l'argument est
  une base jetable `kesh_ci_<suffixe>` (`scripts/gate-ci.sh:8-10`). Défaut : `scripts/gate-ci.sh kesh_ci_s55_4
  kesh-gate-logs/s55-4/gate-ci.log`, au dernier commit de code de la branche de l'epic, après la dernière reprise ; la
  parenthèse « ou la base de la PR » retirée (fichier d'epic déjà corrigé).
- **LOW** (`kesh-gate-logs/s55-decisions/55-4-validation-p8.md`) : V8-2 (T2 complet : `fmt`, `clippy`, les quatre commandes
  frontend, `gate-ci.sh`, E2E complet) ; V8-3 (correctif d'une story fautive : porté par sa fiche, hors plafond, passe de
  revue ciblée, gate complet rejoué pour une story serveur, de même si `gate-ci.sh` rougit) ; V8-4 (catalogues i18n : dans
  le périmètre écran, avec leurs gardes Rust ciblées) ; V8-5 (« dette serveur admise » : conséquence tirée de Q1, non
  décision).

## Story

En tant que mainteneur,
je veux que la dette laissée par les revues des stories 55-1a à 55-3b soit soldée avant la release,
afin que l'epic ne fasse pas monter le compte d'issues ouvertes par sa propre dette.

## Périmètre

- **La dette des stories de l'epic** : les points des revues (validation et code) des stories 55-1a à 55-3b, et de la
  revue du code d'intégration, **qui siègent dans du code ou un texte écrit par une story 55-x** et qui n'ont pas été
  corrigés dans leur story. Un défaut trouvé dans le code de l'Epic 46 ou 47 reste à son epic (issue à son nom).
- **Exclus** : les issues a), b) et c) de l'epic (livrables fermés par la PR de l'epic) ; toute issue antérieure au
  kickoff ; un LOW corrigé sur place par la boucle de revue de sa story (il ne compte pas dans le plafond, `CLAUDE.md`
  § *Priorités des défauts*).
- **Plafond : au plus 2 issues** (fixé au kickoff, fichier d'epic). Au-delà, les suivantes restent ouvertes, nommées à la
  rétrospective avec leur raison. Un correctif qui toucherait plus de cinq modules n'est pas pris (par analogie avec
  la règle de splitting préventif) et reste ouvert.
- Frontend seulement, **sauf** une dette née de la 55-3a côté serveur (`kesh-report`, `kesh-api`, `kesh-qrbill`,
  `kesh-i18n`) — les 55-3a et 55-3b restent dans l'epic (Q1 acceptée par le mainteneur le 2026-11-25) ; une 55-4 qui touche
  le serveur passe le gate complet et l'E2E complet à sa clôture (fichier d'epic, § *Dérogation*).
- Aucune migration ; aucun changement de contrat de l'API **par la 55-4**.

## D'où vient la liste (l'agent de l'epic n'a pas `gh`)

- **Source** : les Change Logs et Dev Agent Records des fiches 55-1a à 55-3b, et le compte rendu de la revue du code
  d'intégration (journal `kesh-gate-logs/s55-integration/`, fichier d'epic § *Gates*) **si elle a eu lieu avant le T0** ; sinon, ses
  points iront en issues à leur nom, hors du plafond de cette story.
  Chaque issue ouverte porte dans son corps « Epic 55, story 55-x » (ou « Epic 55, revue d'intégration »), ce qui permet le
  recoupement. Chaque story y écrit, en section *Issues à ouvrir*, les points non corrigés (sévérité, site, texte de
  l'issue) ; le mainteneur ou son agent les ouvre et l'agent de l'epic reporte les numéros dans la fiche.
- **Recoupement** sur GitHub, par le mainteneur ou son agent : `gh issue list --state open --search '"Epic 55" in:body'`.

## Critères d'acceptation

1. **AC-1 — Liste au départ.** Au T0, le Change Log nomme chaque candidate (numéro, titre, story d'origine, site) tirée de
   la source ci-dessus, les exclusions appliquées, et **au plus deux** retenues, avec le motif du choix (priorité, puis
   effet visible). Zéro ou une candidate : la story le dit, ne corrige que ce qu'il y a, et garde T0, T2 et T3. Le numéro peut manquer au T0 ; il est reporté avant T1.
2. **AC-2 — Preuve, selon la nature du correctif.**
   - **Comportement** faux : test (Vitest, Rust ou E2E) rouge avant le correctif, vert après ; journal
     `kesh-gate-logs/s55-4/`.
   - **Test manquant** : une mutation du code de production, nommée au Change Log, que le test neuf tue (journal).
   - **Texte, commentaire, manuel** : `grep` du symptôme **par sa valeur** avant et après (`CLAUDE.md` § *Propagation
     post-patch*), plus le recompte des clés i18n s'il y a lieu.
   Dans tous les cas, le symptôme est grepé sur tout le dépôt et les autres sites traités dans le même correctif.
3. **AC-3 — Fermeture par la PR.** Le mot-clé de fermeture ne s'écrit que dans le corps de la PR de l'epic, jamais dans
   un commit (`CLAUDE.md` § *Issue Tracking Rule*) ; commits en `refs #N`.
4. **AC-4 — Aucune fermeture pour le chiffre.** Aucune issue fermée sans correctif ; un défaut réel non traité reste ouvert.
5. **AC-5 — Textes et manuel** si une correction touche un texte visible : quatre locales, `sitesTotal` de
   `i18n-keys.test.ts` recompté par la garde, `npm run lint-i18n-ownership` vert ; manuel § *Rapports comptables*
   (`docs/manual/fr/user-manual.tex:4101` à `1e23edb55`), `.tex` seul, PDF aplati contrôlé avant le correctif
   (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
6. **AC-6 — Captures** si un contrôle visible change (`CLAUDE.md` § *Recette visuelle*).
7. **AC-7 — Revue.** Les corrections de production passent la boucle de revue de code et se closent sur une passe sans
   finding au-dessus de LOW (`CLAUDE.md` § *Review Iteration Rule*).
## Tâches

- [ ] **T0** — Liste des candidates (AC-1). **Prérequis** : le mainteneur ou son agent a ouvert les issues des sections
      *Issues à ouvrir* et transmis leurs numéros ; une candidate sans numéro peut être retenue, son numéro reporté avant T1.
- [ ] **T1** — Corrections et preuves (AC-2).
- [ ] **T2** — Gates de fin d'epic (fichier d'epic, § *Gates*, *Dérogation*, Q6 tranchée par le mainteneur le
      2026-11-25), **après le dernier commit de code** des corrections, **avant la PR vers `main`** :
      `scripts/gate-ci.sh origin/main` (ou la base de la PR de l'epic si elle diffère, nommée au journal) et E2E
      complet ; « vert » = seulement les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par
      fichier ; journaux `kesh-gate-logs/s55-4/`. **Si l'E2E rougit**, l'epic n'entre pas dans `main` : la story fautive
      est trouvée (spec rouge rapportée à la story qui a touché son écran ou sa route) et corrigée, puis T2 rejoué.
- [ ] **T3** — Clôture : numéros des issues retenues reportés au fichier d'epic (tableau *Issues*) et au bloc de
      l'Epic 55 de `sprint-status.yaml` ; décompte des issues ouvertes et des bugs à la préparation du tag, demandé au
      mainteneur (pour la rétrospective).

## Note de version

Chaque correction prend une ligne dans la note de version **de cette fiche**, rédigée pour l'utilisateur (aucun numéro
d'epic), avec `refs #N` par ligne ; la note de l'epic est assemblée à la release depuis les notes des fiches.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
- 2026-11-25 — Validation P1 (Sonnet) : 0/0/3/6 ; remédiée : source de la liste et exclusions (V1-1), preuve selon la
  nature du correctif (V1-2), gates de fin d'epic écrits au fichier d'epic et précisés en T2 (V1-3), garde-fous et
  suivi (V1-4 à V1-9).
- 2026-11-25 — Validation P2 ciblée (Haiku) : 0/0/2/3 ; remédiée : règle des gates soumise au mainteneur (Q6), la règle de
  `CLAUDE.md` valant d'ici là (V2-1) ; prérequis des numéros au T0 (V2-2) ; puces citées par leur début (V2-3) ; LOW V2-4,
  V2-5.
- 2026-11-25 — Validation P3 ciblée (Sonnet) : 0/0/1/3 ; remédiée : règle des gates de Q6 propagée aux fiches 55-1, 55-2,
  55-3a (V3-1) ; phrase de l'epic (V3-2) ; marque « Epic 55, story 55-x » dans le corps des issues (V3-3) ; section
  *Issues à ouvrir* de la revue d'intégration (V3-4).
- 2026-11-25 — Validation P4 ciblée (Haiku) : 0/0/1/2 ; remédiée : revue d'intégration et son journal « à produire »
  décrits au fichier d'epic (V4-1) ; marque des issues au § *Gates* du fichier d'epic (V4-2) ; numéro au plus tard avant
  T1 (V4-3).
- 2026-11-25 — Validation P5 ciblée (Sonnet) : 0/0/1/3 ; remédiée : revue d'intégration en AC-8 et T0bis, avant T1 (V5-1,
  fichier d'epic aligné) ; interactions 46/55 (V5-2) ; marque (V5-3) ; tolérance du numéro sortie de l'énumération (V5-4).
- 2026-11-25 — Validation P6 ciblée (Haiku) : 0/0/2/2, deux MEDIUM nés de la remédiation de la P5. **Règle de la sixième
  passe** (`CLAUDE.md` § *Review Iteration Rule* : dès la sixième passe, l'orchestrateur ne remédie plus, il retire) :
  **retirés** la puce « Interactions » (V6-1 : la règle d'origine du périmètre — un défaut du code de l'Epic 46 ou 47 reste
  à son epic — vaut seule), l'AC-8 et la tâche T0bis (V6-2) ; la revue d'intégration reste une étape de l'epic (fichier
  d'epic § *Gates*), non de la 55-4, qui n'en prend les points que si elle a eu lieu avant son T0. V6-3, V6-4 : sans objet
  après le retrait, ou à reprendre au T0 (prérequis « souhaité »).
- 2026-11-25 — Validation P7 ciblée (Sonnet) sur le retrait : 0/0/0/2 — **validation terminée**. Tendance : P1 Sonnet
  0/0/3/6 → P2 ciblée Haiku 0/0/2/3 → P3 ciblée Sonnet 0/0/1/3 → P4 ciblée Haiku 0/0/1/2 → P5 ciblée Sonnet 0/0/1/3 → P6
  ciblée Haiku 0/0/2/2 (retrait) → P7 ciblée Sonnet 0/0/0/2. Les deux LOW se reprennent au **T0 du développement**, ligne
  lue citée au Change Log : V7-1 (« si elle a eu lieu avant le T0 » au périmètre, ligne de la revue d'intégration), V7-2
  (« Prérequis (souhaité) … ; à défaut, AC-1 s'applique »).
- 2026-11-25 — **Décisions du mainteneur versées** : Q1 (55-3a et 55-3b dans l'epic ; dette serveur admise) ; Q6 (T2 : gates
  de fin d'epic avant la PR vers `main`, E2E rouge → l'epic n'entre pas dans `main` avant correction de la story fautive).
  Passe P8 ciblée, la dernière permise ; ses points vont au T0 (R-n).
- 2026-11-25 — Validation P8 ciblée (Opus) sur les décisions du mainteneur : 0/0/1/4. **Dernière passe** : renvoi au
  développement (R-1, LOW listés en tête). `ready-for-dev` sous réserve du T0.
