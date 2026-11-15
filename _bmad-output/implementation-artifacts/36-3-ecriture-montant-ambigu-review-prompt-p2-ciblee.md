# Prompt — passe ciblée P2 de la Story 36-3 (#1051) : une lentille, sur la seule remédiation de la revue P1

Tu relis **une seule chose** : le correctif que la revue P1 de la Story 36-3 de Kesh (comptabilité suisse, SvelteKit 5
runes, catalogues Fluent ; release 0.17.8) a fait écrire. La revue P1 (trois lentilles, `kesh-gate-logs/e36-3/review-p1-{B,E,A}.md`)
n'a rien rendu au-dessus de LOW ; l'orchestrateur a décidé de corriger les LOW dans la story (règle « Tarir la source ») et
de relire le correctif par cette passe. Le motif : *la sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`,
§ *La passe ciblée*).

Worktree `/home/gcorbaz/devel/kesh-36-3`. **Le diff à relire** :
`git -C /home/gcorbaz/devel/kesh-36-3 diff 6aca96626 b5d4ab689` (`6aca96626` = état relu par la revue P1, comptes rendus
compris ; `b5d4ab689` = le commit de la remédiation, seul commit de code depuis). Les commits suivants ne portent que la
fiche, le registre des choix et ce prompt. Langue : français. Modèle : au choix de l'orchestrateur (une passe ciblée
peut aller à Haiku). Contexte frais.

À lire pour le contexte, sans relire la story entière : les trois rapports P1 ci-dessus ; dans la fiche
`_bmad-output/implementation-artifacts/36-3-ecriture-montant-ambigu.md`, D2, D4, D5 et leurs annotations du 2026-11-14,
le tableau des mutations et son annotation, l'entrée du Change Log « Remédiation de la revue P1 » ; le registre
`e36-choix-autonomes-36-3.md`, C-36-3-12. Journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3/` :
`vitest-cible-r1.log`, `mut/resume.log` (entrées `HEAD b5d4ab689` : M12, M13, M14), `front-{check,test-unit,build,lint}-r1.log`,
`test-fast-ci-r1.log` (chaque journal porte son commit en première ligne). **Captures** : `captures/05-assistant-ht-12-34-illisible.png`,
`captures/06-assistant-ht-0.004-sous-le-centime.png` (et 01 à 04, reprises au même moment) — regarde-les si ton outil lit
les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, `killall`, `kill`. **Aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `stash`…).
- **Rien qui exécute ou compile** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du
  dépôt ou de `kesh-gate-logs` (dont `mut/mut-36-3.py`, `captures-36-3.cjs`, `e2e-36-3.sh`) ; ni `make`, `latexmk`, `cargo`,
  `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `cat -A`, `od -c`, `ls`, `find` (sans `-exec`/`-delete`), `wc`. ⚠️ `grep` est une fonction vers
  `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée au rapport. Tout chemin
  cité se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`).

## La lentille unique — « régression du correctif »

Pour chaque morceau du diff, la question n'est pas « le LOW est-il traité ? » mais « **ce correctif a-t-il cassé ou
affaibli quelque chose** ? ». Axes obligatoires :

1. **`htAmountProblem` et l'assistant** (`vat-purchase.ts`, `VatPurchaseAssistant.svelte`) : chaque `kind` de
   `readHtAmount` donne-t-il le bon message, et un seul ? Le message `below-centime` peut-il apparaître pour un HT vide,
   ambigu ou illisible ? Le bouton *Insérer* et la bordure restent-ils gouvernés par `usable` comme avant ? Un `$derived`
   périmé ? Les replis littéraux des trois messages égaux au fr-CH (apostrophe droite échappée `\'`) ?
2. **La clé neuve** `vat-purchase-ht-below-centime` : présente dans les **quatre** `messages.ftl`, à côté de
   `vat-purchase-ht` (jamais en fin de fichier) ; les traductions disent-elles la même chose (arrondi au centime, ligne de
   charge nulle) ? Entrées du lint `lint-i18n-ownership.js` ; `sitesTotal` 3038 → 3040 (`i18n-keys.test.ts`), et le
   commentaire qui l'explique dit-il vrai ?
3. **Les tests ajoutés** (`balance.test.ts`, `vat-purchase.test.ts`, `JournalEntryForm.create.test.ts`) : chaque cas
   attend-il la bonne issue au code (déroule la regex `JOURNAL_AMOUNT_RE` de `balance.ts` à la main pour `'1 234'`,
   `'1ʼ234'`, `'5\n'`, `'1000.000'`, `"12'345'678'901'234"`, `'0001.234'`, `'1.000'`) ? Un test qui passerait à vide
   (assertion sur un mock jamais appelé, format `%j` qui masquerait un cas) ? Le test « même ligne » écrit-il bien débit et
   crédit **de la même ligne** (indices des champs) ?
4. **Les mutations M12 à M14** (`mut/mut-36-3.py`, `mut/resume.log`) : chaque mutation porte-t-elle sur ce qu'elle dit
   (motifs du script contre le code) ? M13 retire-t-elle bien les **trois** verrous, et le test qui la tue est-il le bon ?
5. **Le manuel** (`docs/manual/fr/user-manual.tex`, item **Montant**) : la phrase corrigée est-elle vraie au code aux deux
   bouts (« 0.500 » passe, « 0001.234 » refusé) et pour l'assistant (« mêmes messages », HT arrondi à 0.00) ? Le PDF n'est
   **pas** régénéré dans la branche (C-36-3-6) : dis seulement la sonde à jouer à l'intégration,
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c "zéros de tête ne comptent pas"`.
6. **Les commentaires de B2** : `git grep -nE "max-decimals|Maximum 4 d" -- . ':!_bmad-output'` doit être vide ; les
   commentaires reformulés disent-ils encore vrai ?
7. **Hors du diff, ce qu'il touche** : `htAmountProblem` est-il appelé ailleurs ? `vat-purchase.ts` reste-t-il importé par
   `supplier-invoice-form.ts:11` sans effet ?

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e36-3/review-p2-ciblee.md`. Findings numérotés `P2-1`, `P2-2`… avec
sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes
exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
