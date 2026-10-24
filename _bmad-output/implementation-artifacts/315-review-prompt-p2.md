# Prompt — revue de code P2 de la Story 31-5, passe CIBLÉE, une lentille (regression hunter), Haiku

Tu relis **la seule remédiation de la revue de code P1** de la **Story 31-5** de Kesh (comptabilité suisse) — quatre
défauts courts avant la v0.15.0 : #707 (comptes de TVA de la démonstration), #589 (infobulle *Réouvrir*), #504
(communication de la QR-facture dans la langue du document), #469 (refus du journal d'audit traduits). Worktree
`/home/gcorbaz/devel/kesh-31-5`, branche `story/31-5`. Langue du rapport : français. Modèle : **Haiku**.

**Le diff à relire, aplati** (un seul diff, pas la suite des commits) :

    git -C /home/gcorbaz/devel/kesh-31-5 diff adff757c 50a79a7e

`50a79a7e` est le commit de la remédiation P1. Il est **documentaire** : aucune ligne de `crates/`, `frontend/src/` ni
des tests ne doit y changer — vérifie-le (`git diff --stat adff757c 50a79a7e`). Le commit qui suit (`315-review-prompt-p2.md`,
ce prompt) n'est pas à relire.

Contexte à lire, en lecture seule : les rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/315-review-p1-{A,B,E}.md`
(A1 MEDIUM ; A2-A4, B1-B3, E1-E7 LOW), la liste des LOW laissés sans correctif
`/home/gcorbaz/devel/kesh-gate-logs/315-low-residuels.md`, et le journal du gate ciblé
`/home/gcorbaz/devel/kesh-gate-logs/315-review-p1-gate.log`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, aucune
  redirection vers un fichier (`>`, `>>`, `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la
  sortie standard seulement.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/mem-guard.sh`, ni les scripts `315-*.sh` de `kesh-gate-logs`, ni `make`, ni `latexmk`, ni `cargo` (aucune
  sous-commande), ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
  Aucune commande `gh` qui écrit ; `gh issue view 707|589|504|469` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée. Ne te fie pas
  aux numéros de ligne d'un hunk : relis le fichier final.

## Lentille unique — regression hunter

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche ce que **cette remédiation** a cassé, faussé ou laissé à
moitié :

1. **Manuel** `docs/manual/fr/user-manual.tex` (l. 3262, FAQ « Comment changer la langue d'une facture ? ») **et son PDF**
   `docs/manual/fr/user-manual.pdf`, contrôlé **aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) :
   la phrase neuve est-elle au PDF (attention : un saut de page la coupe entre « de sa » et « langue à son premier
   PDF », avec un pied de page entre les deux) ? Zéro `??` ? Dit-elle **vrai au code** — un PDF figé est servi tel quel
   (`crates/kesh-api/src/routes/issued_invoice_pdf.rs`, `get_or_freeze`, `serve_frozen`), une facture non figée est
   rendue par le code courant à son premier PDF, les rappels (`PdfDocument::Reminder`, `invoice_email.rs`) sont rendus
   à chaque envoi dans la langue du client (`resolve_language`) ? Le refigeage d'un administrateur (`refreeze`) est-il
   correctement couvert par « tant que ce PDF n'est pas refigé » ? Contredit-elle une autre phrase du manuel (§ *Le PDF
   émis est figé*, l. ~1148-1165 ; § des rappels) ? La phrase « suit elle aussi la langue de la facture » est-elle restée ?
   `admin-manual.pdf` et `marketing-brochure.pdf` sont-ils hors du diff ?
2. **CHANGELOG** `CHANGELOG.md` l. 64 : même contrôle de vérité ; cohérent avec le manuel, mot pour mot sur le fond ?
3. **Fiche** `_bmad-output/implementation-artifacts/31-5-defauts-courts.md` : les annotations datées (AC10, premier run
   E2E, *Issues à ouvrir*, M1 sous le tableau des mutations) annotent-elles **sans réécrire** les relevés datés ?
   L'annotation A1 dit-elle juste sur `docs/testing.md` l. 244-250 et sur `docs/agents-orchestration.md` § 4, point 2
   (commit `adff757c`) ? L'explication de M1 tient-elle contre `/home/gcorbaz/devel/kesh-gate-logs/315-mutations.log`,
   `315-mut-M1.log` et `315-mut-run.sh` (horodatages : `ls -la --time-style=full-iso`) ? Le Change Log « Revue de code
   P1 » : chiffres par lentille égaux aux rapports, numéros de ligne cités justes (`issued_invoice_pdf.rs` l. 95-105,
   `invoice_email.rs:566`, `:1266`), gate déclaré égal au journal `315-review-p1-gate.log` (47 exécutés, 17 + 30) ?
   File List : `docs/agents-orchestration.md` et `315-review-prompt-p2.md` présents ?
4. **Registre** `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md`, C-31-5-26 : contexte, retenu,
   écartée exacts ?
5. **`315-low-residuels.md`** : chaque fichier:ligne pointe-t-il bien le constat (relis les fichiers) ? Un LOW de P1
   manque-t-il à la fois à la liste et au sort du Change Log ?
6. **`sprint-status.yaml`** : la ligne `31-5-defauts-courts` et l'en-tête (199) disent-ils l'état ? Statut toujours `review` ?
7. **Messages de commit** : `git log adff757c..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit ne rien rendre ; chaque commit porte ses `refs`.
8. **Propagation** : la formule fautive « émise avant la (version) 0.15.0 » survit-elle ailleurs (manuels, `website/`,
   `README.md`, catalogues, doc-comments) ? `git grep -nF "émise avant"`.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/315-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `P2-1…`, chacun avec sévérité, fichier:ligne, **preuve copiée**, correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
