# Revue de code P2 — Story 34-6 (#953) : après un 409, l'édition de la société montre l'écriture concurrente

Passe **complète**, trois lentilles en contexte frais, chacune par un agent distinct, Sonnet ou Opus (la P1 était Opus :
préférer Sonnet). Worktree `/home/gcorbaz/devel/kesh-e346`, branche `story/34-6-societe-apres-409`, base `665de151`.
Périmètre : `git diff 665de151..HEAD`, aplati en un seul diff, avec **un regard appuyé sur la remédiation de la P1** :
`git diff e549552f..HEAD` (commits `c8139a8c` rouges, `499e79d8` correctif, puis manuel, registre et fiche).

## Ce que chaque lentille lit d'abord

- La fiche `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md` (Conception, AC-1 à AC-8, mutations), son
  Dev Agent Record et son Change Log, dont l'entrée « revue de code P1 » (sort de chaque finding).
- Les registres `e34-choix-autonomes-e343.md` (C-e343-6, 7, 14) et `e34-choix-autonomes-e346.md` (C-e346-1 à 9, dont
  **C-e346-8**, la version par édition ouverte, et **C-e346-9**, les champs inactifs pendant l'envoi).
- Les rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p1-{B,E,A}.md`.
- Le `CLAUDE.md` du worktree, § *Review Iteration Rule*.

## La remédiation à relire en priorité

- `frontend/src/routes/(app)/settings/+page.svelte` : `emailVersion`, `contactVersion`, `addressVersion` ; `syncOpenBlocks`
  (appelée après chaque succès et dans `reloadAfterConflict`) ; `r.version` ; `emptyAddress()` ; `disabled={*Submitting}`
  sur les huit champs.
- `frontend/src/lib/features/settings/company-conflict.ts` : `sameBlockValues`.
- `frontend/src/lib/features/settings/CompanyConflictNotice.svelte` : testids en kebab-case (`slug`).
- Tests neufs de `settings-page.test.ts` (« deux éditions ouvertes », « copie et avis, par bloc », AC-3 des coordonnées,
  testid du NPA) et de `company-conflict.test.ts` (`sameBlockValues`).
- Mutations M7 à M12, M2e, M2c, M3c, M10e, M10c : `/home/gcorbaz/devel/kesh-gate-logs/e346/mut/`.

Questions à se poser : un bloc ouvert peut-il encore envoyer une version plus récente que les valeurs qu'il a montrées
(ordre des appels, `data` remplacé par `submitLock`/`submitUnlock`, `onMount`, relecture d'un bloc dont la version
relue est plus ANCIENNE que celle qu'un autre a déjà adoptée) ? `syncOpenBlocks` peut-il faire régresser une version ? Le
bloc qui vient de réussir est-il fermé avant d'être synchronisé ? Un conflit parasite subsiste-t-il ?

## Axes déclarés non exercés en P1, à exercer ici

- **Exécution** : les trois lentilles P1 étaient en lecture seule. Ici encore, lecture seule — mais le Dev Agent Record
  donne les journaux (`kesh-gate-logs/e346/p1-*.log`, `e2e-r4.log`) : les lire et confronter les chiffres.
- **Captures** : `e346-coordonnees-autre-bloc.png`, `e346-email-vide-ailleurs.png` (non ouvertes par B et E en P1) et la
  nouvelle `e346-deux-blocs-ouverts.png`, dans `/home/gcorbaz/devel/kesh-gate-logs/e346/captures/`.
- **La course entre une relecture en vol et le succès d'un autre bloc** (B, non poussée en P1).
- **Le rendu à largeur de téléphone** et **l'accessibilité** de l'avis (`role="status"`) (E).
- **La qualité des traductions** de-CH, it-CH, en-CH (A).
- **`company-conflict.test.ts` cas par cas** (B).

## Manuel

`docs/manual/fr/user-manual.tex` et **`docs/manual/fr/user-manual.pdf`**, § *Vos coordonnées sur la facture* : la
citation « Repris de l'enregistrement entre-temps » (A-4), la condition du message « rechargées » (A-5) et la phrase sur
les blocs ouverts ensemble (B-1). PDF aplati avant tout `grep` ; la commande de la P1 tronquait sur les caractères
multioctets, préférer `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | awk '{i=index($0,"entre-temps ---"); print substr($0,i-40,1600)}'`.

## Lentille B — Blind Hunter

Diff seul, sans la fiche. États Svelte, ordre des affectations, versions, chemins d'erreur, typage, testids. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e346-review-p2-B.md`.

## Lentille E — Edge Case Hunter

Chaque branche de `syncOpenBlocks` et des trois `save*` avec deux ou trois éditions ouvertes ; relecture qui échoue
pendant qu'un autre bloc est ouvert ; succès d'un bloc pendant la relecture d'un autre ; adresse `null` relue alors que
l'adresse est ouverte. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p2-E.md`.

## Lentille A — Acceptance Auditor

AC par AC, puis le sort de chaque finding P1 tel que le Change Log le déclare (traité, avec son test et sa mutation ; ou
issue demandée, avec sa raison) ; décomptes recomptés depuis la source. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e346-review-p2-A.md`.

## Règles communes

- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `npm`, ni `npx`, ni `cargo`, ni `make`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `git checkout|switch|reset|commit|stash`. Pas de
  redirection, pas de fichier temporaire : le seul fichier écrit est le rapport, par l'outil Write.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `git grep -nF` / `/usr/bin/grep -nF` copiée.
- Sévérités CRITICAL / HIGH / MEDIUM / LOW, avec fichier:ligne et preuve.
- **Le rapport déclare les axes exercés ET non exercés.** Un « 0 finding » sans cette liste ne compte pas.
- Aucune donnée réelle dans le rapport.
