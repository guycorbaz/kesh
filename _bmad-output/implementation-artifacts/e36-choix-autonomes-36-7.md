# Epic 36 — registre des choix autonomes de la story 36-7 (`36-7-tests-desancres-2026`, #1125)

Choix faits à la rédaction de la fiche (2026-11-15, Opus 5.5, worktree `kesh-36-7`, code lu sur `0bf3c3b4b`), dans le cadre
de C-e360-13. Chaque choix : contexte, option retenue, options écartées, réversibilité.

### C-36-7-1 — L'inventaire de la fiche se fait par exécution, à la tête, avant toute écriture

- **Contexte** : #1125 nomme des sites relevés à la 35-5a, sur un commit antérieur ; les 35-5b, 35-5c et la phase 1 de
  l'Epic 36 ont ajouté des tests depuis. `e35-dev-commun.md` § *Inventaires* demande un détecteur par exécution quand il
  existe.
- **Retenu** : nextest du workspace sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z` (voie 2, base `kesh_e367`) et Vitest
  sous un préchargement Node qui décale `Date`, joués par l'agent de la fiche ; scripts et journaux dans
  `kesh-gate-logs/e36-7/`. Résultat : 13 rouges Rust (les mêmes qu'à la 35-5a), 0 Vitest.
- **Écartées** : reprendre la liste de l'issue ; inventaire par lecture des 175 fichiers de tests Rust qui portent « 2026 »
  (2762 lignes).
- **Réversible** : oui (relevé seulement, aucune écriture au dépôt).

### C-36-7-2 — L'E2E en 2027 sous libfaketime, dont l'installation est demandée à Guy

- **Contexte** : le critère de l'epic exige l'E2E vert « sous `KESH_TEST_INSTANT` », variable que ni le binaire servi
  (`docs/testing.md:119`), ni le runner (`api-fixtures.ts:393`), ni le navigateur ne lisent. La bibliothèque n'est pas
  installée (`apt-cache policy faketime` : « Installé : (aucun) »).
- **Retenu** : backend et runner lancés sous `faketime -f '@2027-01-04 11:00:00'`, `FAKETIME_DONT_FAKE_MONOTONIC=1` ;
  sonde à trois étages (serveur, runner, navigateur) ; MariaDB à l'heure réelle, angle mort écrit. Le paquet `faketime`
  s'installe par Guy (geste système). Sans lui, l'agent s'arrête sur AC4 et AC5 et le dit.
- **Écartées** : faire lire la variable au binaire en mode test (code de production, contraire à la garde) ; `page.clock`
  (navigateur seul) ; conclure sur la seule lecture des specs ; construire libfaketime depuis ses sources hors dépôt
  (téléchargement de code externe, à décider par l'orchestrateur si Guy n'installe pas le paquet).
- **Réversible** : oui (outillage de la station, aucun fichier du dépôt).

### C-36-7-3 — Les trois sites à borne de 2026 antérieure entrent au périmètre

- **Contexte** : `admin_full_import_e2e:2007`, `:2281` et `services_rollout:207` rougissent sous un instant antérieur à
  leur borne, jamais à l'heure réelle ; #1125 les porte en commentaire.
- **Retenu** : les corriger dans la story (D2), pour que #1125 puisse se fermer et que la campagne d'instants n'ait plus de
  rouge attendu.
- **Écartée** : une issue à part — même classe, même fichier pour deux d'entre eux.
- **Réversible** : oui.

### C-36-7-4 — Aucun outillage versionné : les recettes vont dans `docs/testing.md`

- **Contexte** : Guy veut des stories courtes (2026-11-15). Un script de campagne de fin d'année ou un setup Vitest lisant
  `KESH_TEST_INSTANT` serait une fonction neuve.
- **Retenu** : recettes écrites dans `docs/testing.md` ; préchargement Vitest et scripts hors dépôt.
- **Écartée** : `scripts/test-2027.sh`, setup Vitest versionné — proposables en issue.
- **Réversible** : oui.

### C-36-7-5 — Le placeholder `CLI-2026-00042` n'est pas un défaut

- **Contexte** : `frontend/src/routes/(app)/contacts/+page.svelte:1067` porte une année dans un exemple de saisie.
- **Retenu** : laissé tel quel, sans issue : c'est un exemple de forme, non une date affichée comme actuelle — à la
  différence de l'aperçu de numérotation de `settings/invoicing/+page.svelte:159`, pour lequel une issue est demandée.
- **Écartée** : une issue P4 de plus.
- **Réversible** : oui.
