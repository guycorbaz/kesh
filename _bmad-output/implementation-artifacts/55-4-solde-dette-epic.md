# Story 55-4 : solde de la dette de l'epic

Status: done

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
- Frontend et catalogues i18n (`messages.ftl`, avec leurs gardes Rust ciblées de `kesh-i18n` et de `textes_coherents`),
  **sauf** une dette née de la 55-3a dans le code serveur (`kesh-report`, `kesh-api` ; l'aperçu et la marque,
  `kesh-qrbill`, sont sortis de l'epic le 2026-11-26) — les 55-3a et 55-3b restent dans l'epic (Q1 acceptée par le
  mainteneur le 2026-11-25) ; une 55-4 qui touche le serveur passe le gate complet et l'E2E complet à sa clôture ; une
  55-4 qui touche `messages.ftl` ou le manuel joue à sa clôture `cargo nextest run -p kesh-i18n` et
  `cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` (fichier d'epic, § *Dérogation*).
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

- [x] **T0** — Liste des candidates (AC-1). **Prérequis** : le mainteneur ou son agent a ouvert les issues des sections
      *Issues à ouvrir* et transmis leurs numéros ; une candidate sans numéro peut être retenue, son numéro reporté avant T1.
- [x] **T1** — Corrections et preuves (AC-2).
- [x] **T2** — *(joué par le coordinateur en fin d'epic, 2026-11-27, à `ff0c56a40` : voir le Change Log)* Gates de fin d'epic (fichier d'epic, § *Gates*, *Dérogation*, Q6 tranchée par le mainteneur le
      2026-11-25), **après le dernier commit de code** des corrections, **avant la PR vers `main`** :
      `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, les quatre commandes
      frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`), `scripts/gate-ci.sh kesh_ci_s55_4
      kesh-gate-logs/s55-4/gate-ci.log` (backend comme la CI ; l'argument est une base jetable `kesh_ci_<suffixe>`, non une
      référence git, `scripts/gate-ci.sh:8-10` ; script prêt : `kesh-gate-logs/s55-dev/fin/gate-ci.sh`, sur une copie
      `git archive` de la tête de la branche de l'epic, après la dernière reprise) et E2E complet ; « vert » = seulement les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par
      fichier ; journaux `kesh-gate-logs/s55-4/`. **Si l'E2E rougit**, l'epic n'entre pas dans `main` : la story fautive
      est trouvée (spec rouge rapportée à la story qui a touché son écran ou sa route) et corrigée, puis T2 rejoué. Le
      correctif est porté par la fiche de la story fautive (Change Log, Dev Agent Record), hors du plafond de la 55-4 ; il
      passe une passe de revue ciblée (à la demande du mainteneur, comme toute revue de cet epic) ; story serveur : gate
      complet rejoué. De même si `gate-ci.sh` rougit (il s'arrête au premier binaire en échec : rejoué jusqu'au bout).
- [x] **T3** — Clôture : numéros des issues retenues reportés au fichier d'epic (tableau *Issues*) et au bloc de
      l'Epic 55 de `sprint-status.yaml` ; décompte des issues ouvertes et des bugs à la préparation du tag, demandé au
      mainteneur (pour la rétrospective).

## Note de version

- Dans les Rapports, si les comptes de TVA désignés n'ont pas pu être lus, leur ligne sous le décompte TVA reparaît au
  prochain affichage du décompte (retour sur l'onglet, nouveau décompte), sans recharger la page (refs #1603).

Chaque correction prend une ligne dans la note de version **de cette fiche**, rédigée pour l'utilisateur (aucun numéro
d'epic), avec `refs #N` par ligne ; la note de l'epic est assemblée à la release depuis les notes des fiches.

## Dev Agent Record

### Agent Model Used

Claude Opus (agent de développement de la story, sans revue de code).

### Completion Notes List

- **T0** : Change Log du 2026-11-26 — 14 candidates (L1 à L14 de `kesh-gate-logs/s55-dev/issues-a-ouvrir.md`), six
  exclues (L3, L5, L6, L7, L10, L13), **deux retenues** (L8, L2), six non retenues laissées en issues ; R-1 et V8-2 à V8-5
  traités (T2 et périmètre récrits, V8-5 annoté). Aucune issue n'est encore ouverte : pour les deux retenues, corrigées
  ici, il n'y a pas d'issue à ouvrir ; les autres gardent « numéro à reporter ».
- **L8 — comportement faux** : `+page.svelte`, effet `vatSettingsRequested` — le `catch` remet `vatSettingsRequested` à
  faux ; la lecture se refait au prochain affichage du décompte, jamais en boucle (variable non réactive, remise hors de
  l'effet). **Rouge avant** : test « Story 55-4 (L8) … » de `reports-page-55-2.test.ts` en échec, `Unable to find an element
  by: [data-testid="vat-accounts"]` (`kesh-gate-logs/s55-dev/55-4/vitest-avant-correctif.log`) ; **vert après**
  (`vitest-apres-correctif.log`, 93/93 sur les deux fichiers). Le test vérifie aussi qu'après une lecture réussie, aucune
  lecture de plus n'a lieu (pas de boucle).
- **L2 — test manquant** : test « Story 55-4 (L2) … » de `reports-page-url.test.ts` (grand livre de tous les comptes, « Du »
  tapé sans *Générer*, aller-retour par l'onglet Bilan : la date tapée reste, 0 requête). **Mutation** `+page.svelte:1088`
  (`lastHandledKey = search;` retiré de `writeUrl`), copie non mutée verte d'abord (`vitest-apres-correctif.log`) :
  **tuée**, `AssertionError: expected '2026-01-01' to be '2026-03-01'` sur le test neuf
  (`mutation-L2-cle-ecrite.log`) ; fichier restauré par `git checkout`. ⚠️ **Constat** : la même mutation fait aussi
  rougir deux tests existants, « niveau changé sur le Bilan … redemandé à la fin » (`expected 2 to be 3`) et « V5-5 (a) — clic,
  contrôles différents de l'affiché … » (`expected "vi.fn()" to be called 2 times, but got 1 times`) : la fiche 55-1a la
  disait non tuée à tort (elle ne l'avait pas jouée, « sans objet sous D16 ») ; le test neuf est le premier qui la nomme et
  qui vise sa conséquence directe (un champ remis à la clé de l'adresse).
- **Symptôme grepé** (« retentative », « reboucherait », lecture des comptes de TVA) sur le code, les quatre locales, le
  manuel et les fiches de l'epic : seul le commentaire corrigé le portait ; le manuel (`user-manual.tex:4436-4442`) ne
  décrit pas le cas d'échec, inchangé. Aucun texte visible, aucune clé i18n, aucun `messages.ftl` ni manuel modifié
  (d'où ni `nextest -p kesh-i18n`, ni `textes_coherents`, ni `make user`). Aucun serveur touché.
- **AC-6** : aucun contrôle neuf ni changé (la ligne existante reparaît) : pas de capture. Les specs E2E jouées ont
  écrit leurs captures dans `kesh-gate-logs/s55-dev/55-4/e2e/captures/`.
- **AC-7** : **correction non revue, revue à la demande du mainteneur** (aucune revue de code dans cet epic tant qu'il n'a
  pas donné son accord).
- **Gates** (au dernier commit de code `647b5b039`) : `clippy.sh 55-4/final` fmt rc=0, clippy rc=0 ; `front.sh 55-4/final`
  check rc=0, lint-i18n-ownership rc=0, Vitest **5587 passés (318 fichiers)**, build rc=0 ; **E2E partiel (3 specs)**,
  `e2e.sh 55-4/e2e` : `reports-navigation.spec.ts`, `reports-entry-documents.spec.ts` (demandées par le coordinateur) et
  `reports.spec.ts` (écran des Rapports touché) — **33 passés**, 0 échec. Gate ciblé, E2E partiel (3 specs) ; **E2E complet
  et gate-ci en fin d'epic** (T2, joué par le coordinateur).
- **Tests ajoutés** (de `6182b32a5` à `647b5b039`) : 2 Vitest.

### Issues à ouvrir

Aucune issue née de la 55-4. Les candidates non retenues ou exclues restent à ouvrir telles qu'écrites par leur story
(colonne « Suite (55-4) » de `kesh-gate-logs/s55-dev/issues-a-ouvrir.md`) : L1, L3, L4, L5, L6, L7, L9, L10, L11, L12,
L13, L14 ; L2 et L8 sont corrigées ici (pas d'issue). Décompte des issues ouvertes et des bugs à la préparation du tag :
à demander au mainteneur (pour la rétrospective).

### File List

- `frontend/src/routes/(app)/reports/+page.svelte` (effet des réglages de TVA)
- `frontend/src/routes/(app)/reports/reports-page-55-2.test.ts` (test L8)
- `frontend/src/routes/(app)/reports/reports-page-url.test.ts` (test L2)
- `_bmad-output/implementation-artifacts/55-4-solde-dette-epic.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml`

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
- 2026-11-26 — **Décisions du mainteneur du 2026-11-26 versées** : périmètre serveur ramené à `kesh-report` et
  `kesh-api` (aperçu sorti de l'epic) ; resserrement de la dérogation Q6 (`nextest -p kesh-i18n`, `binary(textes_coherents)`
  pour une 55-4 qui touche `messages.ftl` ou le manuel). Aucune passe (plafond de huit atteint) : recontrôle au **T0**,
  ligne lue citée.
- 2026-11-26 — **T0 (développement)**, tête de départ `6182b32a5` (55-1a à 55-3b en `review`, **sans revue de code** : le
  mainteneur doit d'abord donner son accord). Source des candidates : les sections *Issues à ouvrir* des Dev Agent Records
  des fiches 55-1a à 55-3b, réunies dans `kesh-gate-logs/s55-dev/issues-a-ouvrir.md` (lignes L1 à L14 du tableau). La revue
  du code d'intégration n'a pas eu lieu avant ce T0 (aucun journal `kesh-gate-logs/s55-integration/`) : ses points iront en
  issues à leur nom, hors plafond (V7-1, périmètre « si elle a eu lieu avant le T0 », ligne 49 de cette fiche). Aucune de
  ces issues n'est encore ouverte : **numéro à reporter** pour chacune (AC-1 le permet, V7-2).
  - **Candidates** (story d'origine, site) :
    L1 (55-1a, `routes/(app)/reports/+page.svelte`, avis d'erreur) pas de bouton « Réessayer » ;
    L2 (55-1a, `reports-page-url.test.ts`) mutation « clé écrite non enregistrée » non tuée ;
    L3 (55-1b, `+page.svelte` `resyncActiveGroupedTab`) ;
    L4 (55-1b, `+page.svelte` `writeUrlAtResponse`, `goto` asynchrone) ;
    L5 (55-1a, rapports par compte analytique hors de l'adresse) ;
    L6 (55-1a, `(app)/+layout.ts:10-11`, `login/+page.svelte:74`, la connexion ne ramène pas à l'adresse demandée) ;
    L7 (55-2, `MirrorOfNote`) ; L8 (55-2, `+page.svelte` effet `vatSettingsRequested`, réglages TVA non relus après un
    échec) ; L9 (55-2, recette D17 des écrans d'authentification) ; L10 (55-3a, `kesh-db/src/repositories/
    imported_supplier_invoices.rs:409-433`, `LIMIT 1`) ; L11 (55-3a, section reportée de sa fiche) ; L12 (55-3b, captures
    du manuel) ; L13 (55-3b, `ui/dialog/dialog-content.svelte:42`, « Close » en dur) ; L14 (55-3b, lecture en `Blob` des
    PDF de facture et d'avoir écrite deux fois).
  - **Exclusions** : L3 et L7, code de l'Epic 46 (`resyncActiveGroupedTab`, règle de la 46-2) et de l'Epic 47
    (`MirrorOfNote.svelte`) — issue à leur epic ; L13, antérieure au kickoff : `git log -- frontend/src/lib/components/ui/
    dialog/dialog-content.svelte` ne rend qu'un commit, `3d919d736` du 2026-04-06 (Story 1.9), ligne lue
    `dialog-content.svelte:42` `<span class="sr-only">Close</span>` ; L10, antérieure au kickoff et hors du périmètre
    serveur (`kesh-db`, et non `kesh-report`/`kesh-api`) : le dernier commit du fichier est `4d39f3774` (v0.17.6,
    2026-11-12) ; L6, site antérieur à l'epic (`(app)/+layout.ts`, `login/+page.svelte`, non écrits par une story 55-x),
    issue prévue par la fiche 55-1a ; L5, réduction de périmètre décidée à la validation de la 55-1a (rapports par compte
    analytique hors de l'URL, C-55-1-7), évolution et non dette.
  - **Restent candidates** : L1, L2, L4, L8, L9, L11, L12, L14 — toutes LOW, aucune `bug` (types proposés :
    `enhancement`, `technical-debt`, `documentation`).
  - **Retenues (2, le plafond)** — motif : à priorité égale (LOW, aucune `bug`), l'effet visible d'abord, puis la
    protection d'un livrable `bug` P3 de l'epic :
    1. **L8** — les comptes de TVA désignés ne sont pas relus après un échec : la ligne des comptes disparaît jusqu'au
       rechargement de la page, sans signal. **Seule candidate restante à effet visible** sur un écran de l'epic, hors L1.
       Ligne lue : `+page.svelte:187-191`
       `if (activeTab !== 'vat' || vatReport === null || hiddenByMenu || vatSettingsRequested) return;
       vatSettingsRequested = true; getInvoiceSettings().then(...).catch(() => (vatSettings = null));`. Correctif : la
       lecture en échec se refait au prochain affichage du décompte (retour sur l'onglet, nouveau décompte), jamais en
       boucle — `vatSettingsRequested` n'est pas un `$state`, sa remise à faux dans le `catch`, hors de l'effet, ne le
       relance pas.
    2. **L2** — mutation « clé écrite non enregistrée » de la 55-1a sans test qui la tue. Elle protège le mécanisme du
       livrable b) (#1604, `bug` P3 : Retour du navigateur, état des Rapports dans l'URL). Ligne lue : `+page.svelte:1081-1085`
       `function writeUrl(search: string, replace: boolean): void { lastHandledKey = search; if (page.url.search
       .replace(/^\?/, '') === search) return; void goto(...)`. La fiche 55-1a la disait sans objet sous D16 (« réappliquer
       sa propre adresse redonne le même état ») : faux pour le grand livre de **tous les comptes**, qui garde *Générer* et
       dont un clic d'onglet écrit la clé **affichée**, non celle des champs (`urlForTab`, `+page.svelte:560-563`) — la
       réappliquer remet les dates tapées à celles du rapport affiché. Test neuf, mutation nommée au T1.
  - **Non retenues, laissées en issues** (« reste ouverte », T3) : L1 (choix R-25 en attente du mainteneur : le trancher
    ici préempterait sa décision), L4 (fenêtre de quelques microtâches, non reproduite), L9 (exige une installation dont
    l'envoi de courriels est configuré), L11 (texte d'une fiche, sans effet), L12 (à la release), L14 (refactorisation sans
    effet visible).
  - **R-1 (V8-1, MEDIUM)** — ligne lue : `scripts/gate-ci.sh:8-10` « Usage : scripts/gate-ci.sh <base> [journal] /
    <base> kesh_ci_<suffixe> (a-z, 0-9, _) : recréée à neuf » et `:48-51` `BASE=${1:-}` …
    `[[ "$BASE" =~ ^kesh_ci_[a-z0-9_]+$ ]] || refus … 2`. Décision : T2 récrit, `scripts/gate-ci.sh kesh_ci_s55_4
    kesh-gate-logs/s55-4/gate-ci.log` par le script prêt `kesh-gate-logs/s55-dev/fin/gate-ci.sh` (copie `git archive` de
    la tête, base `kesh_ci_s55_4`) ; parenthèse « ou la base de la PR » retirée (fichier d'epic déjà corrigé, § *Dérogation*
    « `<base>` est une base jetable `kesh_ci_<suffixe>`, non une référence git »).
  - **V8-2 (LOW)** — ligne lue : `grep -n "npm\|clippy" scripts/gate-ci.sh` ne rend aucune commande (le script ne joue que
    `cargo test`, `:117` « ▶ cargo test, comme la CI »). Décision : T2 nomme `fmt`, `clippy`, les quatre commandes frontend,
    `gate-ci.sh` et l'E2E complet.
  - **V8-3 (LOW)** — ligne lue : T2, « la story fautive est trouvée … et corrigée, puis T2 rejoué ». Décision : T2 précise
    que le correctif est porté par la fiche fautive, hors plafond, passe de revue ciblée (à la demande du mainteneur), gate
    complet rejoué pour une story serveur, de même si `gate-ci.sh` rougit.
  - **V8-4 (LOW)** — ligne lue : périmètre, « Frontend seulement, **sauf** une dette née de la 55-3a côté serveur ».
    Décision : périmètre récrit « Frontend et catalogues i18n (`messages.ftl`, gardes Rust ciblées) … sauf une dette née
    de la 55-3a dans le code serveur ».
  - **V8-5 (LOW)** — ligne lue : Change Log du 2026-11-25, « Q1 (55-3a et 55-3b dans l'epic ; dette serveur admise) ».
    Décision : l'entrée datée n'est pas réécrite (relevé daté) ; **annotation** : « dette serveur admise » est une
    conséquence tirée de Q1 (les 55-3a et 55-3b dans l'epic, d'où une dette serveur possible, gate de la *Dérogation* pour
    une story serveur), non une décision du mainteneur.
  - **AC-7 (revue)** : aucune revue de code dans cet epic tant que le mainteneur n'a pas donné son accord ; les corrections
    de T1 sont des **corrections non revues, revue à la demande du mainteneur**.
- 2026-11-26 — **T1 et T3** : L8 corrigée (rouge avant, vert après), L2 couverte (mutation tuée) ; gates de la story verts
  (fmt, clippy, front complet, E2E partiel 3 specs, 33 passés) ; statut `review`, T2 laissé au coordinateur (fin d'epic).
  Correction non revue, revue à la demande du mainteneur.
- 2026-11-27 — **T2, gates de fin d'epic** (coordinateur), à la tête `ff0c56a40` (dernier commit de code `647b5b039`,
  dont fmt, clippy et le front complet sont verts à la 55-4) :
  - **E2E complet** (`LANG=fr_CH.UTF-8`, `kesh-gate-logs/s55-dev/fin/e2e-complet/e2e.log`, 48,2 min) : **542 passés,
    7 échecs, 20 ignorés** ; les sept échecs sont exactement ceux de la KF-029 (#97) : `mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`. `opening-balances.spec.ts:156` passé.
  - **`scripts/gate-ci.sh kesh_ci_s55_4`** sur une copie `git archive` de `ff0c56a40`, cible
    `kesh-targets/gate-ci-e55` (`kesh-gate-logs/s55-dev/fin/gate-ci-1.log`, 5467 s) : **6832 passés, 0 échec, 21
    ignorés**, code 0. Joués l'un après l'autre (disque : la cible neuve de gate-ci n'aurait pas tenu au-dessus du
    plancher de 40 Go avec celle de la story).
  - Journaux sous `kesh-gate-logs/s55-dev/fin/` (et non `s55-4/`, comme le reste de l'epic).
- **2026-11-26 — revue de code, close (une passe).** Tendance : P1 complète Sonnet **0/0/0/1**, relue par
  l'orchestrateur (correctif de la relecture des réglages TVA, `647b5b039`, lu au diff). P1-1 (LOW, hors production) :
  la candidate L2 (« clé écrite non enregistrée ») était déjà tuée par deux tests existants ; la ligne 2 de
  `kesh-gate-logs/s55-dev/issues-a-ouvrir.md` est donc sans objet (annotée dans la liste des revues) ; le test neuf
  reste. Aucun reclassement.
  Prompts versionnés à côté de la fiche (`<clé>-code-review-prompt-pN.md`, consignes communes `55-code-review-brief-commun.md`) ; rapports hors dépôt, `kesh-gate-logs/s55-revue/<clé>/revue-pN-rapport.md`. Base : `epic/55-navigation` après la fusion de la PR de l'Epic 46 (`c8fe4d0ac`, revue d'intégration P1 Opus 0/0/0/2, LOW corrigés `f16463f91`). LOW non corrigés : issues à ouvrir, `kesh-gate-logs/s55-revue/issues-a-ouvrir.md` (ligne 2 de la liste du développement sans objet). Gates de fin d'epic (gate-ci et E2E complet) au dernier commit de code de l'epic : voir le fichier d'epic.
