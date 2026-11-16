# Prompt — passe ciblée P2 de la Story 36-3b (#1171) : une lentille, sur la seule remédiation de la revue P1

Tu relis **la remédiation de la revue P1** de la Story 36-3b de Kesh (comptabilité suisse, SvelteKit 5 runes), release
**0.17.8** : le prix unitaire d'une ligne de facture client se lit comme le montant d'une écriture (« 1.250 » refusé comme
ambigu, « 1'250 » envoyé `1250`). La revue P1 (trois lentilles) a rendu 0 / 0 / 0 / 12 LOW ; l'orchestrateur a fait
corriger les LOW dans la story (règle « Tarir la source »). **Ta passe ne relit que ce correctif.** Modèle : Haiku permis
(passe ciblée de fin de boucle), sinon Sonnet ou Opus. Contexte frais. Langue : français.

Worktree `/home/gcorbaz/devel/kesh-36-3b`, branche `story/36-3b-facture-prix-unitaire-ambigu`. **Le diff à relire, aplati,
en un seul** : `git -C /home/gcorbaz/devel/kesh-36-3b diff 197ba366e e7aaa3efa` — `197ba366e` = fiche en `review` après le
développement ; `6cb932bd6`, `b06fe15f2`, `e7aaa3efa` = les trois commits de code de la remédiation (`e7aaa3efa` = **dernier
commit de code**) ; le commit suivant ne porte que la fiche, le registre et ce prompt. Fichiers du diff de code :
`frontend/src/lib/components/invoices/InvoiceForm.svelte`, `frontend/src/lib/components/invoices/InvoiceForm.test.ts`,
`docs/manual/fr/user-manual.tex`.

À lire pour le contexte : les rapports de la P1 `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/review-p1-{B,E,A}.md` (B1-B4,
E1-E4, A1-A4) ; la fiche `_bmad-output/implementation-artifacts/36-3b-facture-prix-unitaire-ambigu.md`, **Change Log**
(entrée « Revue de code P1 », sort de chaque LOW) et **Dev Agent Record** (« Remédiation de la revue P1 ») ; le registre
`e36-choix-autonomes-36-3b.md` (C-36-3b-2, -3, -7 annotés). Journaux (lecture seule) : `mut/r1/` et `mut/r1-final/`
(`resume.log`, `M*.log`, `R1-M*.log`), `r1-cible-*.log`, `test-fast-ci-r1.log`, `front-*-r1.log`, `e2e-cible-r1.log`.

⛔ Rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se déclare.
- Aucun `rm`, `pkill`, `killall`, `kill` ; aucun `git checkout`/`switch`/`restore`/`stash`, aucun `git` qui écrit.
- **Aucun interpréteur ni rien qui compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `jq`, ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du
  dépôt ou de `kesh-gate-logs` (`mut/mut-36-3b.py`, `e2e-36-3b.sh`, `captures-36-3b.cjs`) ; ni `make`, `latexmk`, `cargo`,
  `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`. Aucune commande `gh`, aucun réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`. ⚠️ `grep` est une fonction vers `ugrep` :
  `/usr/bin/grep` ou `git grep`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée ; tout chemin cité se
  vérifie (`ls` ou `git cat-file -e`).

## La lentille unique : chasseur de régression du correctif

1. **`readLineUnitPrice(i, raw)`** (`InvoiceForm.svelte`) : rend-il exactement ce que rendaient les branches qu'il remplace
   (`validateClient` d'avant : ambigu, illisible, vide → « valeurs numériques invalides » ; charge `payload`) ? Le
   `new Big(lecturePrix.payload)` vaut-il l'ancien `lecturePrix.value` ? La boucle de la charge (après
   `await getInvoiceSettings()`) : `submitting` remis à faux, aucune requête, le bandeau nomme la **bonne** ligne ;
   `stripUiKey(lines[i])` ; plus aucune trace du repli `'Prix unitaire invalide'`
   (`/usr/bin/grep -nF "Prix unitaire invalide" frontend/src/lib/components/invoices/InvoiceForm.svelte`). Le type
   discriminé passe-t-il `npm run check` (`front-check-r1.log`) ? `unitPricePayload` reste-t-il employé (totaux) ?
2. **Les témoins neufs** : « Ligne 2 » (ambigu, illisible), ordre description → prix → quantité, attente des réglages
   (« 12'34 », « 1'250 »). Chacun peut-il rougir (mutations `R1-M15`, `R1-M16`, `R1-M10`, `R1-M17` au journal) ? Le
   sélecteur de la description (`tbody tr input[type="text"]:not([inputmode])`) vise-t-il bien la description et non le
   champ de compte ?
3. **A1** : les trois tests d'AC4 en `expect.soft` ; `ambigus()` et `prixEnvoyes()` (désormais `[]` sans charge) — une
   assertion devenue vacante ? `prixEnvoyes()` modifié change-t-il le sens d'un test existant qui supposait une charge
   (comparer `git diff 197ba366e e7aaa3efa -- frontend/src/lib/components/invoices/InvoiceForm.test.ts`) ? Les journaux
   `mut/r1-final/M3.log`, `M4.log`, `M5.log` nomment-ils « forme rouverte », « message sous le champ », « bandeau »,
   « charge partie » et « charge envoyée » ?
4. **Le manuel** (`docs/manual/fr/user-manual.tex:1158`) : la phrase récrite sur la réouverture est-elle vraie au code
   (`frontend/src/lib/features/invoices/unit-price.ts`, `unitPriceFieldValue`) pour `0.1250`, `1000.1230`, `100.0000`,
   `1.2340` ? La note de version de la fiche dit-elle la même chose ? PDF non régénéré (à l'intégration) : vérifie
   `git diff --stat 197ba366e e7aaa3efa -- docs/`.
5. **Rien d'autre** : `git diff --stat 197ba366e e7aaa3efa` ne nomme que les trois fichiers ci-dessus.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e36-3b/review-p2-ciblee.md` : findings numérotés (`P2-1`…), sévérité (CRITICAL / HIGH /
MEDIUM / LOW), lieu `fichier:ligne`, ligne lue copiée, preuve, remède ; **la liste des axes exercés ET non exercés** ; le
décompte final recompté depuis la liste.
