# Prompt — revue de code P2 **ciblée** de la Story 34-1 (#966), une lentille, Haiku

Tu relis **un seul commit** : la remédiation de la revue de code P1 de la **Story 34-1** de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit 5 runes, MariaDB 10.11), release 0.17.6. La P1 (Sonnet, trois lentilles) a rendu 0 CRITICAL, 0 HIGH, 0 MEDIUM, 17 LOW ;
la passe ciblée cherche ce que **la remédiation** a pu casser — *la sévérité se déplace vers ce qu'on vient d'écrire*.

Worktree `/home/gcorbaz/devel/kesh-e3401`, branche `story/34-1-assistant-compte-bancaire`. Le commit :
`git -C /home/gcorbaz/devel/kesh-e3401 show 7368944b` (diff unique, déjà aplati). Lis aussi l'entrée « revue de code P1 » du
Change Log de la fiche `_bmad-output/implementation-artifacts/34-1-assistant-compte-bancaire.md` (ce que le commit prétend faire)
et, pour le contexte, les trois rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/e3401-review-p1-{B,E,A}.md`. Langue : français.
Contexte frais.

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e3401-review-p2-ciblee.md` (le **seul** fichier que tu écris).

## Interdits absolus

- **Lecture seule.** Aucun fichier hormis ton rapport ; **aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`, `tee`,
  `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit.
- **Rien qui compile ou exécute** : ni `cargo`, `npm`/`npx`/`node`, `python3`, `perl`, `bash -c`, `make`, `latexmk`, `docker`,
  `mariadb`, `curl`, `playwright`, `vitest`, `claude` ; ni **`scripts/prepare-release.sh`**, ni aucun script du dépôt ou de
  `kesh-gate-logs`.
- **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `/usr/bin/grep` (`grep` est ici une fonction vers `ugrep`), `sed -n`,
  `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport — en particulier
  « le patch X n'est pas appliqué » : vérifie-le au fichier, non au diff.

## Les axes

1. **L'écran** (`frontend/src/routes/onboarding/+page.svelte`) : `bankLedgerLoading` (posé avant l'`await`, rendu dans `finally`
   — un chemin le laisse-t-il à `true` et bloque-t-il l'étape pour de bon ?) ; `bankLedgerTouched` (posé par `onchange` : est-ce
   que la présélection elle-même, ou un `bind:value` programmatique, le déclenche ?) ; le garde de `submitBankAccount` et le bouton
   désactivé — le saut (*Configurer plus tard*) reste-t-il possible pendant le chargement ? `bankLedgerFailed` /
   `bankLedgerMotif` : la comparaison au texte de la clé évite-t-elle un doublon ; la garde `onboarding-catch-guard.test.ts`
   reste-t-elle satisfaite (le `catch` appelle toujours `errorMessageOf`) ? le compteur `sitesTotal` d'`i18n-keys.test.ts` change-t-il
   (la clé est maintenant lue dans une fonction appelée deux fois) ?
2. **`linkableLedgerAccounts`** (`format.ts`) : le tri `localeCompare` sur une copie (`filter` rend un tableau neuf : aucune
   mutation de la liste reçue ?) ; ses trois appelants (assistant, page *Comptes bancaires*, formulaire *Lier*) ; `withCurrentAccount`
   place le compte courant en tête — inchangé ?
3. **Les tests neufs** (`onboarding-bank-step.test.ts`, `onboarding_bank_account_e2e.rs`) : peuvent-ils passer **à vide** ?
   (`listeEnAttente` : la promesse est-elle bien celle que lit l'écran ; `soumettre()` pendant le chargement suivi d'un seul
   `setTimeout(0)` suffit-il à prouver l'absence d'appel ; le cas (f) de `bank_step_replays` : versions 5 à 8 justes ; le test
   « archivé » : 1109 a-t-il un identifiant plus grand que 1100 au plan PME, `active = FALSE` libère-t-il bien l'index unique de
   `singleton_role` avant de poser le rôle sur 1109 ?)
4. **Le manuel et le CHANGELOG** : la phrase « sur un compte déjà créé » est-elle vraie au code (l'`INSERT` écrit `NULL` pour un
   champ absent) ; « une description vide ou faite d'espaces vaut `null` » (`normalize_description`) ; le PDF
   (`docs/manual/fr/user-manual.pdf`), s'il a été régénéré après `7368944b`, aplati (`-oF "déjà créé"`, sous `timeout 60`).
5. **Les mutations M8 à M15** (`/home/gcorbaz/devel/kesh-gate-logs/e3401/mut/M8.log` à `M15.log`, et le script `mut-e3401.py` à
   **lire**) si elles ont tourné : chacune est-elle tuée par le test attendu ? M10 déclarée équivalente : l'est-elle ?
6. **Ce que la remédiation n'a pas touché mais aurait dû** : greppe les symptômes corrigés (`inchangés`, `.sort(`, `Non configuré`)
   ailleurs dans le dépôt.

## Rapport

Pour chaque finding : identifiant (`C-1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier:ligne, preuve copiée, correctif. Puis la
**liste des axes exercés et non exercés** (un « 0 finding » sans elle ne compte pas), et tout fichier créé par erreur.
