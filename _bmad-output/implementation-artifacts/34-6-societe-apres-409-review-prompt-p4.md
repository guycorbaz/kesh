# Revue de code P4 — Story 34-6 (#953) : après un 409, l'édition de la société montre l'écriture concurrente

Passe **complète**, trois lentilles **Sonnet** en contexte frais, chacune par un agent distinct. Worktree
`/home/gcorbaz/devel/kesh-e346`, branche `story/34-6-societe-apres-409`, base `665de151`. Périmètre :
`git diff 665de151..HEAD`, aplati. **Regard appuyé sur la remédiation de la P3** : `git diff e2590009..HEAD` (rouges
`8444ca86`, correctif `89397483`, puis registre et fiche).

## Ce que chaque lentille lit d'abord

- La fiche `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md` : Conception, AC-1 à AC-8, Dev Agent Record et
  Change Log, dont les entrées des revues P1, P2 et P3 (sort de chaque finding, gates différés par `PAUSE-HEAVY`).
- Les registres `e34-choix-autonomes-e343.md` (C-e343-6, 7, 14) et `e34-choix-autonomes-e346.md`, de C-e346-1 à C-e346-11.
  Trois sont à lire de près :
  - **C-e346-4**, renversé en P3 : l'avis se vide au départ de l'envoi ;
  - **C-e346-10**, la monotonie ;
  - **C-e346-11**, la séquence de départ à version égale, avec sa limite assumée.
- Les rapports P3 : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p3-{B,E,A}.md`.
- Le `CLAUDE.md` du worktree, § *Review Iteration Rule*.

## Axe dédié : les courses et le verrou de période

Toutes les lentilles l'exercent. Deux questions :
- `data`, ou l'affichage du verrou de période, peuvent-ils encore reculer ?
- Un bloc ouvert peut-il encore envoyer une version plus récente que les valeurs qu'il a montrées ?

À examiner dans `frontend/src/routes/(app)/settings/+page.svelte` :
- `nextSeq`, `requestSeq`, `dataSeq` et `adoptCompany` (versions inférieure, égale, supérieure ; valeur de retour) ;
- `reloadAfterConflict`, qui juge contre `data` quand la relecture est refusée ;
- les trois `save*` : séquence prise avant l'envoi, avis vidé au départ ;
- `submitLock` et `submitUnlock` ;
- `onMount`, qui écrit `data` sans séquence (`dataSeq = 0`).

Les cas à énumérer, et la question à poser pour chacun :
- **Tous les sites** qui écrivent `data`, une `*Version` ou `dataSeq` (`/usr/bin/grep -nE "\bdata = |Version = |dataSeq"`).
  Pour chacun, chaque ordre de réponses possible : deux succès, un succès et un verrou, une relecture et un verrou, un
  verrou et un déverrouillage, deux relectures, une relecture qui échoue.
- **Le serveur réel** : `lock_books` et `unlock_books` ne changent pas la version (`crates/kesh-db/src/repositories/companies.rs`).
  Vérifier que les tests le modélisent ainsi, sauf celui qui dit modéliser l'écriture d'un autre.
- **La limite assumée de C-e346-11** : la séquence ordonne les départs, non le traitement au serveur. Est-elle bien
  bornée à l'affichage ? Un ordre la ferait-il porter sur une version envoyée ?
- **`reloadAfterConflict` juge contre `data.company` quand la relecture est refusée** : la copie et la version du bloc
  restent-elles cohérentes ? Le bloc peut-il alors rester ouvert sans rien montrer, puis écraser ?
- **Avis vidé au départ de l'envoi** : la région vive se vide puis se remplit. Un lecteur d'écran l'annonce-t-il ? Le dire
  au code, sans lecteur réel.

## Autres points de la remédiation P3

- `vi.resetAllMocks()` dans les six `beforeEach` de `settings-page.test.ts` : un test dépend-il encore d'une implémentation
  posée par un autre ? Les défauts nécessaires sont-ils reposés ?
- `PENDING_FIELD` (`read-only:bg-input/50 read-only:cursor-progress`) sur les huit champs : la classe `bg-input` existe-t-elle
  dans le thème, et la fusion `cn` avec les classes du composant `Input` la garde-t-elle ?
- Les tests neufs :
  - « verrouillage à version égale » ;
  - « une relecture partie avant le verrou… » ;
  - « une relecture plus ancienne ne ferme pas… » ;
  - « l'avis est effacé au départ du nouvel envoi » ;
  - le test du verrou récrit (`:898`).

## Ce qui a été différé par `PAUSE-HEAVY`, et à confronter aux journaux s'ils existent à l'heure de la passe

- Les mutations de la P3, dont `<` → `<=` d'`adoptCompany` et la garde de séquence retirée : `/home/gcorbaz/devel/kesh-gate-logs/e346/mut/`.
- Vitest complet, `check`, `build` et l'E2E complet au dernier commit de code : `p3-*.log`, `e2e-r6.log`.
- Les captures, dans `/home/gcorbaz/devel/kesh-gate-logs/e346/captures/`.

Si ces journaux n'existent pas encore, le dire dans les axes non exercés. Ne pas conclure sans eux.

## Axes déclarés non exercés en P3, à exercer ici

- Le rendu à largeur de téléphone (au gabarit).
- Les journaux des mutations M2e, M2c, M3c, M8, M9, M10e et M11.
- Les traductions de-CH, it-CH et en-CH, inchangées depuis la P2.
- `company-conflict.test.ts`, cas par cas.
- Le PDF du manuel : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `awk`. Reste-t-il vrai
  après C-e346-4 renversé et C-e346-11 ?

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p4-B.md`.
- **E — Edge Case Hunter** : chaque branche, chaque ordre de réponses. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p4-E.md`.
- **A — Acceptance Auditor** : AC par AC ; le sort de chaque finding P3 (traité avec test et mutation, ou issue avec son
  numéro) ; les décomptes recomptés depuis la source, avec leur périmètre. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p4-A.md`.

## Règles communes

- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `npm`, ni `npx`, ni `cargo`, ni `make`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `git checkout|switch|reset|commit|stash`. Pas de
  redirection, pas de fichier temporaire : le seul fichier écrit est le rapport, par l'outil Write.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `git grep -nF` ou `/usr/bin/grep -nF` copiée.
- Sévérités CRITICAL, HIGH, MEDIUM, LOW, chacune avec fichier:ligne et preuve.
- **Le rapport déclare les axes exercés ET non exercés.** Un « 0 finding » sans cette liste ne compte pas.
- Aucune donnée réelle dans le rapport.
