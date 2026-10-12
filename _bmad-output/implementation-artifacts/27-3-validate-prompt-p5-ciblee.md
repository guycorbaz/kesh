# Prompt — validation P5 ciblée, Story 27-3 (une lentille, Haiku)

*Versionné le 2026-10-12, par l'orchestrateur.*

Worktree `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu`. **Le seul diff à relire** :
`git show 6d9c5509` (remédiation de la validation P4 ciblée). Fiche :
`_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` ; registre `epic-27-choix-autonomes-27-3.md`.
Code de référence : `frontend/src/lib/features/bank-import/BankImportUpload.svelte` (ou le chemin réel du composant,
à trouver par `git ls-files | grep BankImportUpload`).

## Axes (chacun déclaré exercé ou non)

1. **AC2 réécrit** : la règle « `isConfirming` d'abord, puis `previewReason` » est-elle cohérente avec AC3, AC9 (branche
   `409` : `isConfirming = false` **puis** `runPreview(…, 'stale')`), T3 et C-27-3-41 ? Une autre phrase de la fiche dit-
   elle encore que le texte se lit dans `previewReason` seul ? (`grep -nF "previewReason"`)
2. **V9 et M27** : V9 « après un recalcul » est-il réalisable avec les états décrits ? M27 est-elle tuée par V9 et par
   lui seul ? M27 est-elle bien listée là où la fiche énumère les mutations (AC14, T8, tableau, décomptes) ?
3. **Décomptes** : 29 mutations, 45 choix (`grep -c "^### C-27-3-" <registre>`), et tout résidu « 28 mutations »,
   « M1-M26 », « 44 choix », « 1 à 44 » hors des Change Logs historiques (`grep -nE`).
4. **Réfutation de P4-2** : la fiche (l. 279 et 281) prescrit-elle bien le retrait des deux « avant de déposer » ?

⚠️ Erreur de catégorie connue : un livrable que la fiche **prescrit** mais que le code n'a **pas encore** n'est pas un
défaut de la fiche. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée, prise sur le
fichier final.

## Interdits

Lecture seule. Tu n'écris que `/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p5-ciblee.md`. Aucune commande qui
compile, exécute ou écrit : ni `scripts/*`, ni `cargo`, `npm`, `npx`, `node`, `python`, `python3`, `perl`, `awk` avec un
programme, `bash -c`, `sh -c`, `make`, `docker`, `git` en écriture, `gh` en écriture, ni SQL ni HTTP. Rien sous
`~/kesh-test/`.

## Rapport

Findings avec sévérité, `fichier:ligne`, preuve, correction proposée ; **liste des axes exercés et non exercés**.
