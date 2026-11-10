# Revue de code P3 — Story 34-6 (#953) : après un 409, l'édition de la société montre l'écriture concurrente

Passe **complète**, trois lentilles **Opus** en contexte frais, chacune par un agent distinct. Worktree
`/home/gcorbaz/devel/kesh-e346`, branche `story/34-6-societe-apres-409`, base `665de151`. Périmètre :
`git diff 665de151..HEAD`, aplati. **Regard appuyé sur la remédiation de la P2** : `git diff 1f054dea..HEAD` (rouges
`fd354737`, correctif `8ee97e5e`, puis registre et fiche).

## Ce que chaque lentille lit d'abord

- La fiche `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md` (Conception, AC-1 à AC-8), son Dev Agent
  Record et son Change Log, dont les entrées « revue de code P1 » et « revue de code P2 » (sort de chaque finding).
- Les registres `e34-choix-autonomes-e343.md` (C-e343-6, 7, 14) et `e34-choix-autonomes-e346.md` (C-e346-1 à 10 ;
  **C-e346-8** version par édition ouverte, **C-e346-9** lecture seule pendant l'envoi, **C-e346-10** monotonie).
- Les rapports P1 et P2 : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p{1,2}-{B,E,A}.md`.
- Le `CLAUDE.md` du worktree, § *Review Iteration Rule*.

## Axe dédié : la monotonie et les courses

Toutes les lentilles l'exercent, chacune à sa manière. La question : **un bloc ouvert peut-il encore envoyer une version
plus récente que les valeurs qu'il a montrées, et `data` ou une version peuvent-ils encore reculer ?**

- `frontend/src/routes/(app)/settings/+page.svelte` : `adoptCompany` (garde `company.version < data.company.version`),
  `syncOpenBlocks` (garde `v > *Version`), `reloadAfterConflict`, les trois succès, `submitLock`/`submitUnlock`,
  `onMount`, et les `*Version = r.version` du bloc en 409.
- Énumérer **tous** les sites qui écrivent `data` ou une `*Version` (`/usr/bin/grep -nE "\bdata = |Version = "`), et pour
  chacun l'ordre des réponses possible : deux succès dont les réponses se croisent, une relecture en vol pendant un
  succès, une relecture en vol pendant une autre relecture, un verrou de période pendant une relecture, une relecture qui
  échoue.
- La garde de `syncOpenBlocks` est déclarée redondante (C-e346-10, M14 équivalente) : vérifier cette prémisse au code.
- Le bloc en 409 prend `r.version` même quand `data` est plus récente : vérifier que le pire est bien un 409 de plus.
- À version égale, `adoptCompany` remplace (le verrou ne change pas la version) : vérifier au dépôt (`lock_books`) et
  chercher une autre écriture de la société qui ne changerait pas la version.

## Autres points de la remédiation P2

- `readonly` au lieu de `disabled` sur les huit champs (B2) : le composant `Input` le transmet-il ?
- La région vive (`role="status"`, `aria-live="polite"`) montée avant l'avis (B3) ; l'avis du bloc e-mail déplacé sous la
  ligne (B5) — capture `e346-email-vide-ailleurs.png` refaite.
- Tests neufs : « une relecture plus ancienne… », « adresse ouverte… (A2-4) », « le verrou de période… (E2-3) »,
  « adresse relue absente (E2-2) », « la région vive… ». Mutations M13 à M19 :
  `/home/gcorbaz/devel/kesh-gate-logs/e346/mut/`. Le rouge de M17 sur le test de la région vive vient d'une valeur
  `mockResolvedValueOnce` laissée par le test qui a échoué avant lui (pollution sous mutation seulement) : juger si
  c'est un défaut d'hygiène des tests.

## Axes déclarés non exercés en P2

- **Exécution** : les lentilles restent en lecture seule ; lire les journaux `p2-*.log`, `e2e-r5.log`, `rouges.log`.
- Le **rendu à largeur de téléphone**, et **un lecteur d'écran réel** : non exercés jusqu'ici. Les juger au code et aux
  captures, en le déclarant.
- Le **PDF du manuel** (B ne l'avait pas ouvert en P2). Le manuel n'a pas changé à la P2 : vérifier qu'il reste vrai
  pour la monotonie. PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `awk`.
- Les **corps** des tests de `settings-page.test.ts` (B n'en avait relu que les titres).
- Le **gabarit complet** de `+page.svelte`, hors hunks.
- Les journaux des mutations M2e, M2c, M3c, M8, M9, M10e, M11.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p3-B.md`.
- **E — Edge Case Hunter** : chaque branche, chaque ordre de réponses. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p3-E.md`.
- **A — Acceptance Auditor** : AC par AC, sort de chaque finding P2 (traité avec test et mutation, ou issue avec numéro),
  décomptes recomptés depuis la source avec leur périmètre. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p3-A.md`.

## Règles communes

- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `npm`, ni `npx`, ni `cargo`, ni `make`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `git checkout|switch|reset|commit|stash`. Pas de
  redirection, pas de fichier temporaire : le seul fichier écrit est le rapport, par l'outil Write.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `git grep -nF` ou `/usr/bin/grep -nF` copiée.
- Sévérités CRITICAL / HIGH / MEDIUM / LOW, avec fichier:ligne et preuve.
- **Le rapport déclare les axes exercés ET non exercés.** Un « 0 finding » sans cette liste ne compte pas.
- Aucune donnée réelle dans le rapport.
