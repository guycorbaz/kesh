# Prompt — revue de code P3 **ciblée**, Story 28-4b (remédiation de la passe ciblée P2)

*Versionné le 2026-10-14. **Une seule lentille** (Sonnet), contexte frais, lecture seule, braquée sur **la seule
remédiation** de la passe P2 (§ *La passe ciblée* du `CLAUDE.md`).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-4b`, branche `story/28-4b`.
**Le diff à revoir** : `git diff 54128d35 b6005307 -- . ':(exclude)_bmad-output'` — `54128d35` est le dernier commit
d'avant cette remédiation ; `793b9704` (production) et `62453d21` (un test) portent le code, `47db4a21` le manuel,
`b6005307` le PDF. ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Contexte : le rapport de la passe P2
(`/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p2-ciblee.md` : M1, L1, L2, L3), les choix **C-28-4-51** et
**C-28-4-52** (et l'annotation de C-28-4-48) au registre `epic-28-choix-autonomes.md`, la campagne
`kesh-gate-logs/28-4b-mutations-p2.log` (5 / 5 tuées), la fiche
`_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (Change Log, ligne « Revue P2 ciblée »).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/`.

## La lentille : la classe est-elle fermée, et qu'a-t-on cassé ?

1. **`closeOrphanMode`** (`frontend/src/routes/(app)/bank-accounts/+page.svelte`). L'inventaire des modes à cible
   est-il **complet** (type `Mode` : chaque variante qui porte un `id`) ? La garde est-elle appelée **à un seul
   endroit**, et chaque chemin qui modifie `bankAccounts` passe-t-il par `reload()` (`grep -n "bankAccounts ="`) ?
   `handleJournalUpdated` remplace une ligne sans recharger : peut-il laisser un mode orphelin ? Un `reload()` en
   échec (`baResult` rejeté) laisse-t-il le mode tel quel, et est-ce juste ?
2. **Ordre des gestes réussis** : l'archivage, la clôture et la réouverture referment leur mode **avant** de
   recharger. La création et la modification complète rechargent **puis** referment : une modification qui rend le
   compte invisible (aucun cas ?) déclencherait-elle le message à tort ? Un `closeForm()` placé avant `reload()` dans
   `confirmClose` efface-t-il un état dont le toast ou la suite a besoin ?
3. **Le message neutre** `bank-accounts-notice-stale` : quand s'affiche-t-il, quand s'efface-t-il (remis à `false` à
   chaque relecture réussie) ? Peut-il rester affiché à tort, ou ne jamais s'afficher (rendu sous `{#if loading}` /
   `{:else if loadError}`) ? Les quatre traductions disent-elles la même chose, l'allemand sans « geschlossen » (G11) ?
4. **L1 — la bascule ne referme plus rien** : un formulaire `create` est conservé — mais `{#if loading}` démonte et
   remonte le formulaire : la saisie survit-elle par l'état (`formBankName`…), et le test le prouve-t-il ? Un
   `edit-journal` (composant `BankAccountJournalLinkForm`, état interne) survit-il au démontage, ou perd-il sa saisie ?
5. **L2 — trois messages** : les quatre combinaisons `includeClosed` / `includeArchived` donnent-elles le bon texte ?
   Les guillemets des clés (« … » en fr, «…» en de, “…” en en) sont-ils cohérents avec les catalogues voisins ?
   `sitesTotal` 2107 (page 70 → 73) recompté ?
6. **Les tests et les mutations** : chaque test neuf rougit-il sous la mutation qui le vise (journal) ? Cherche une
   **mutation plausible qui resterait verte** — par exemple : la garde qui referme aussi `create` ; le message posé
   sans fermer le mode ; `staleNotice` jamais remis à `false`.
7. ⛔ **Manuel, PDF compris** (diff `47db4a21`, PDF `b6005307`) : les deux phrases neuves (action refermée quand son
   compte quitte la liste ; liste vide qui dit lesquels sont masqués) disent-elles vrai au code ? Contrôle le PDF
   **aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p3-ciblee-user-manual-flat.txt` (⚠️ ligature « ﬀ », apostrophe
   « ’ »).
8. **L3 de la P2** (fenêtre de course théorique du test B-2) : est-elle bien écrite comme angle mort (Dev Agent
   Record) ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4b-review-p3-ciblee.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée, et
pour chacun : vient-il **de cette remédiation** ou d'avant ? Pour tout finding affirmant qu'un code ou un texte est
absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport et du texte aplati du PDF, dans `/home/gcorbaz/devel/kesh-gate-logs/`
(pas de fichier temporaire, même supprimé ensuite) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, ⛔ **`git checkout`** (sous aucune forme, pas même `git checkout -- <fichier>` ni `git
checkout <sha>`), `git commit`/`add`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`, `cargo` (aucune
sous-commande), `npm`, `npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même les scripts de mutations), `perl`,
`ruby`, `php`, `sed -i`, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl`, `wget`,
`gh` en écriture, `pkill`/`kill`, aucune requête SQL. Pour lire une version passée : `git show <sha>:<chemin>`.
Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
