# Story v0172-correctifs — revue de code P2, passe CIBLÉE (une lentille — modèle Sonnet)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`,
ni `latexmk`, ni `python3`, ni `node`, ni `docker`, ni `curl`, ni `claude`, ni aucun script du dépôt
(`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`… compris), ni `sed -i`, ni `rm`, ni
`pkill`, ni `killall` ; **aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire ; aucun `git checkout`,
`switch`, `reset`, `stash`, `rebase`. Permis : `git log`, `git show`, `git diff`, `git grep`, `/usr/bin/grep`, `sed -n`,
`cat`, `ls`, `wc`, `pdftotext f.pdf -` vers la sortie standard seulement. Le **seul** fichier que tu écris est ton rapport.
⛔ Aucune donnée réelle (ni `~/kesh/`, ni le NAS).

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

Langue : français soigné. Contexte frais.

## Ce que tu relis — la seule remédiation de la revue P1

- Worktree `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`.
- Le diff : `git -C /home/gcorbaz/devel/kesh-0172b diff afcc24cf..HEAD` (code `59025b3e`, puis CHANGELOG, manuel `.tex`
  et PDF, fiche, registre). Rien d'autre n'est dans ton périmètre.
- Ce que la remédiation prétend faire : l'entrée « Revue de code P1 (Opus) » du Change Log de
  `_bmad-output/implementation-artifacts/v0172-correctifs.md`, et C-v0172b-35, C-v0172b-36 du registre
  `v0172-choix-autonomes.md`. Les findings d'origine : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-review-p1-{B,E,A}.md`
  (B-2, B-3, B-4, E-4, E-5, A-1, A-9).
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/v0172b/` — `rouges.log` (§ « Remédiation de la revue P1 »),
  `mut/MB3*.log`, `gate2-*.log`, `gate-e2e-r6*.log`.

## Ta lentille : chasseur de régressions de la remédiation

1. **Liste vide** (`supplier-invoices/+page.svelte`) : les trois branches (`statusFilter`, sans case, avec case) ; le cas
   `statusFilter` **et** `?cancelled=1` ensemble ; les quatre locales des deux clés neuves ; le libellé du statut interpolé.
2. **Lien « Afficher toutes les factures »** (`?cancelled=1`) : la case est-elle bien cochée à l'arrivée, `offset` absent ;
   le manuel (`user-manual.tex`, § de la liste fournisseur) dit-il vrai ; aucun autre test ou E2E ne supposait `/supplier-invoices`.
3. **Avis « à vérifier »** : repli Svelte et quatre `messages.ftl` concordent ; le manuel (« Les dates », « Créer la facture
   d'un document ») ne contredit plus l'avis.
4. **A-9** : la phrase neuve du manuel cite-t-elle le message réel du catalogue (`reconciliation-errors-account-locked`,
   fr-CH) ? Le comportement décrit (attente, cinq secondes, rien d'enregistré) est-il vrai au code
   (`bank_imports.rs`, `create_under_account_lock`, `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS`) ?
5. **MB3 bis équivalente** (C-v0172b-36) : l'argument tient-il au CSS de `+layout.svelte` ? Une situation (bandeau, page
   courte ou longue, fenêtre basse) où `h-[…]` et `max-h-[…]` se distinguent à l'écran ?
6. **CHANGELOG** : les puces #840, #870, #885 complétées disent vrai ; aucune puce #876.
7. **Gardes i18n** : 2806 → 2808 sites, 395 → 397 clés — recompte aux deux bornes (`afcc24cf` → tête).
8. **Le PDF** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (apostrophes `’`, césures) ; les
   phrases neuves présentes, « ce n'est pas celle du document » absent.
9. **Commits** : `git log afcc24cf..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide.

## Le rapport

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-review-p2.md`. Pour chaque finding : sévérité, lieu
(`fichier:ligne`), ligne lue copiée, défaut, remède. Toute affirmation de présence ou d'absence s'appuie sur une sortie
`/usr/bin/grep -nF` ou `git grep` copiée. Termine par les **axes exercés** et **non exercés** (avec la raison), et le décompte
CRITICAL n · HIGH n · MEDIUM n · LOW n. Un « 0 finding » sans la liste des axes exercés ne compte pas.
