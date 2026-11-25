# Story 55-4 : solde de la dette de l'epic

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md`. Release : celle de l'epic, **avant le tag**.
Règle : `CLAUDE.md` § *Priorités des défauts*, les deux puces sous « Tarir la source, puis solder » (un LOW qui touche la
production peut se corriger dans sa story ; une story de solde à la fin de chaque epic, plafonnée par un nombre d'issues
fixé au kickoff).

## Story

En tant que mainteneur,
je veux que la dette laissée par les revues des stories 55-1 à 55-3b soit soldée avant la release,
afin que l'epic ne fasse pas monter le compte d'issues ouvertes par sa propre dette.

## Périmètre

- **La dette des stories de l'epic** : les points des revues (validation et code) des stories 55-1 à 55-3b, et de la
  revue du code d'intégration, **qui siègent dans du code ou un texte écrit par une story 55-x** et qui n'ont pas été
  corrigés dans leur story. Un défaut trouvé dans le code de l'Epic 46 ou 47 reste à son epic (issue à son nom).
- **Exclus** : les issues a), b) et c) de l'epic (livrables fermés par la PR de l'epic) ; toute issue antérieure au
  kickoff ; un LOW corrigé sur place par la boucle de revue de sa story (il ne compte pas dans le plafond, `CLAUDE.md`
  § *Priorités des défauts*).
- **Plafond : au plus 2 issues** (fixé au kickoff, fichier d'epic). Au-delà, les suivantes restent ouvertes, nommées à la
  rétrospective avec leur raison. Un correctif qui toucherait plus de cinq modules n'est pas pris (règle de splitting
  préventif) et reste ouvert.
- Frontend seulement, **sauf**, si les 55-3a et 55-3b sont restées dans l'epic (Q1), une dette née de la 55-3a côté `kesh-report`.
- Aucune migration ; aucun changement de contrat de l'API **par la 55-4**.

## D'où vient la liste (l'agent de l'epic n'a pas `gh`)

- **Source** : les Change Logs et Dev Agent Records des fiches 55-1 à 55-3b, et le compte rendu de la revue du code
  d'intégration. Chaque story y écrit, en section *Issues à ouvrir*, les points non corrigés (sévérité, site, texte de
  l'issue) ; le mainteneur ou son agent les ouvre et l'agent de l'epic reporte les numéros dans la fiche.
- **Recoupement** sur GitHub, par le mainteneur ou son agent : `gh issue list --state open --search '"Epic 55" in:body'`.

## Critères d'acceptation

1. **AC-1 — Liste au départ.** Au T0, le Change Log nomme chaque candidate (numéro, titre, story d'origine, site) tirée de
   la source ci-dessus, les exclusions appliquées, et **au plus deux** retenues, avec le motif du choix (priorité, puis
   effet visible). Zéro ou une candidate : la story le dit et se réduit d'autant.
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

- [ ] **T0** — Liste des candidates (AC-1).
- [ ] **T1** — Corrections et preuves (AC-2).
- [ ] **T2** — Gates de fin d'epic, **après le dernier commit de code** des corrections (§ *Gates* du fichier d'epic) :
      `scripts/gate-ci.sh origin/main` (ou la base de la PR de l'epic si elle diffère, nommée au journal) et E2E
      complet ; « vert » = seulement les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par
      fichier ; journaux `kesh-gate-logs/s55-4/`.
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
