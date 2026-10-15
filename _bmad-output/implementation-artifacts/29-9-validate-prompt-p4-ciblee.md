# Prompt — validation P4 **ciblée** de la spec, Story 29-9 (amendement #678)

*Versionné le 2026-10-14. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de l'amendement #678** (`CLAUDE.md`, § *La passe ciblée*). La validation des trois premiers correctifs est
close (P1 R 0/0/4/5 F 0/0/5/6 → P2 R 0/0/1/8 F 0/0/2/7 → P3 ciblée 0/0/0/1) ; l'orchestrateur a ensuite fait entrer
un quatrième correctif, [#678] (P4) : c'est ce texte neuf, et lui seul, que cette passe relit.*

Dépôt `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9`. La fiche est une spec : rien n'est implémenté. Le code
au sol de la branche est celui de `fcd2ae75` ; **l'AC11 s'écrit sur le composant tel que la Story 29-6 le laisse**,
non mergée : lis-le **à son commit figé**, `d71c506b`, dans le worktree de la 29-6, par
`git -C /home/gcorbaz/devel/kesh-29-6 show d71c506b:<chemin>` (jamais l'arbre de travail de ce worktree, qui bouge).
Fichiers de la 29-6 utiles : `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`,
`frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`,
`frontend/tests/e2e/reconciliation-reject-unmatched.spec.ts`, `crates/kesh-i18n/locales/*/messages.ftl`,
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes.md` (C-29-6-18, C-29-6-25). L'issue : `gh issue view
678` (lecture).

**Périmètre : `git diff a844244e b63861aa -- ':!*29-9-validate-prompt-p4-ciblee.md'`** (amendement #678 : `156a6dea`,
puis la phrase du manuel citée à l'AC11 dans `b63861aa`), **diff unique, aplati** — ne lis pas les commits antérieurs, sauf pour
vérifier qu'un texte validé n'a pas été réécrit. Fichiers touchés : la fiche
`_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md` (statut, énoncé, constat, prémisses, **AC11**
neuf, renvois datés dans AC8/AC10, **T10**, T6/T7/T9, dérogation, zones partagées, exclusions, propagation,
références, Change Log), le registre `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-9.md`
(**C-29-9-19 à 23** ajoutés), `sprint-status.yaml` (ligne `29-9-…` et en-tête), le fichier d'epic
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`.

⚠️ **Toute vérification de présence ou d'absence se fait par `grep -nF "<chaîne>" <fichier>` sur le fichier FINAL**
(l'arbre de travail de `kesh-29-9`, égal à `b63861aa` pour ces fichiers ; pour la 29-6, `git -C … show d71c506b:<chemin>
| grep -nF "<chaîne>"`), jamais en lisant les numéros de ligne du diff. Un finding qui affirme qu'un texte « manque »
ou qu'une ligne citée est fausse sans sortie `grep -nF` copiée ne compte pas. **Aucun fichier temporaire** : tout
passe par des tubes vers la sortie standard.

## La lentille — chasseur de défauts de l'amendement #678

Pour chaque axe, vérifie contre le code (sol `fcd2ae75` et 29-6 à `d71c506b`) :

1. **Prémisses de #678 (constat, prémisses)** : les deux cas sont-ils vrais au code — aucune case sans `aria-label`,
   `<label>` ni `aria-labelledby` ; aucune région `aria-live`/`role="status"` dans le composant, au sol comme en 29-6 ?
   Les numéros cités (`:319-327` au sol ; `:316-323`, `:294-324`, `:356-363`, `:357-363`, `:365-367`, `:193-195`,
   `:310`/`:347` en 29-6 ; test `:864-987`, `:965-986`, `:977-978`, `:898`, `:939`, `:943`, `:960`, `:962` ; E2E
   `:160-169` ; mock i18n `:50-55` ; `fr-CH:1262-1316`) désignent-ils ce que la fiche dit ? « Aucun test ne sélectionne
   une case par rôle et nom » est-il vrai ?
2. **Libellé de la case (AC11, C-29-9-20)** : les deux clés et leurs gabarits fr-CH sont-ils cohérents entre l'AC11,
   V19, V20, l'E2E et le registre ? La règle de choix (`counterpartyName ?` : vide ou nul → sans contrepartie) dit-elle
   la même chose que la cellule (`?? ''`) — et une contrepartie `''` est-elle traitée sans contradiction ? Les
   variables sont-elles les mêmes partout, et les gardes i18n nommées (`i18n-entrees-a-variables.test.ts`,
   `i18n-repli-divergent-actif.test.ts`, `lint-i18n-ownership.js`) les accepteraient-elles (lis-les au sol) ? Le
   mock i18n du fichier de test interpole-t-il bien `{ $x }`, si bien que V19/V20 peuvent comparer à la valeur fr-CH ?
3. **Région d'annonce et contrat de la 29-6 (AC11, C-29-9-21)** — l'axe le plus cher : la région englobante laisse-t-elle
   **réellement intactes** les assertions de présence et d'absence citées (Vitest et E2E de la 29-6) ? Le test de
   position est-il le **seul** test de la 29-6 qui casserait — cherche toute autre assertion qui lit `parentElement`,
   `lastElementChild`, `children`, `nextElementSibling` ou l'ordre des enfants de la rangée des boutons, ou qui compte
   les éléments de cette rangée, dans le test Vitest **et** dans les specs E2E de la réconciliation
   (`frontend/tests/e2e/reconciliation-*.spec.ts`, sol et 29-6). La règle « dernier enfant de la rangée » (C-29-6-18)
   tient-elle avec la région ? Les écartées (`<p>` toujours rendu, `data-testid` conditionnel, région sur la rangée)
   sont-elles écartées pour des raisons vraies ?
4. **Tests et mutations (V19-V22, M22-M27)** : chaque test est-il **rouge au sol** (composant de la 29-6, sans
   correctif) pour la bonne raison ? Chaque mutation est-elle tuée par le test indiqué — en particulier M27 (`{#key}`
   qui recrée le nœud) par l'identité de nœud de V21, et M24 par l'absence de la région au premier rendu ? Une mutation
   plausible reste-t-elle survivante (par exemple `aria-live="off"` en gardant `role="status"`, ou la région hors de la
   rangée) ?
5. **E2E (C-29-9-22)** : la spec `reconciliation-show-older.spec.ts` de l'AC9 rend-elle ce volet possible — ses
   transactions sont-elles **toutes** sans proposition (lis l'AC9 et le seed `with-company` :
   `frontend/tests/e2e/` et le code du mode test côté `crates/kesh-api` pour savoir s'il crée des factures) ? Le nom
   accessible construit des **textes des cellules** égale-t-il celui du gabarit rempli des chaînes de l'API (la cellule
   montant affiche `{amount} {currency}` ; la cellule contrepartie `?? ''`) ? Le volet perturbe-t-il la suite de la
   spec (sélection vidée au rechargement, C-29-9-5) ?
6. **Manuel (axe obligatoire)** : « aucun texte visible ne change, manuel inchangé » est-il vrai ? Le manuel FR
   décrit-il quelque part l'indication de la 29-6 ou les cases (`git -C /home/gcorbaz/devel/kesh-29-6 show
   d71c506b:docs/manual/fr/user-manual.tex | grep -nE "cocher|coche|la page le dit"` ; la fiche cite `:2086`) d'une
   façon que l'AC11 rendrait fausse ? Contrôle le **PDF** de la 29-6, aplati : `git -C /home/gcorbaz/devel/kesh-29-6
   show d71c506b:docs/manual/fr/user-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' ' | grep -o "Aucune
   correspondance » et porte une case.\{0,200\}"` (si le motif ne rend rien, élargis-le : les guillemets du PDF
   peuvent différer, ce qu'il faut alors dire). La puce de CHANGELOG prescrite est-elle juste ?
7. **Zones partagées et ordre (C-29-9-23, § *Zones partagées*)** : l'AC11 réécrit-il quoi que ce soit de la 29-6 au
   lieu de l'envelopper ou d'y ajouter (case, `toggle`, `selectionHasUnmatched`, `<p>`, clé, repli) ? T10 « après T4,
   avant T5 » est-il cohérent avec T5 (`sitesTotal`), T6 (CHANGELOG), T7 (E2E) ? La dépendance à la 29-6 mergée
   est-elle écrite sans contradiction avec C-29-9-11 ?
8. **Dérogation au découpage (C-29-9-19, § *Dérogation*)** : le recompte (cinq / sept, aucun module neuf) est-il
   juste au regard des fichiers que l'AC11 touche ? La règle de `CLAUDE.md` (« plus de 5 modules ») est-elle citée
   juste ?
9. **Texte validé non réécrit, recomptes, historique** : le diff modifie-t-il **sur le fond** un AC1-AC10, une tâche
   T0-T9 ou une entrée datée (Change Log P1-P3, entrées C-29-9-1 à 18) au lieu d'y ajouter un renvoi daté ? Recompte à
   la source : 11 AC (`grep -c "^\*\*AC[0-9]"`), 11 tâches (`grep -c "^- \[ \] \*\*T"`), V1-V22
   (`grep -oE "\bV[0-9]{1,2}\b" | sort -uV`), M1-M27 (`grep -c "^| M[0-9]"`), C-29-9-1 à 23 (`grep -c "^### C-29-9-"`)
   — l'entrée « Amendement #678 » du Change Log dit-elle ces nombres ? La consigne `closes #649 #655 #646 #678` est-elle
   au statut et à T9, et **aucun** message de commit de la branche ne porte-t-il un mot-clé de fermeture
   (`git log main..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?)\b"`) ? La clé
   `29-9-trois-petits-correctifs` est-elle restée partout, le titre seul disant « quatre » ? `sprint-status.yaml` et
   le fichier d'epic mentionnent-ils #678 ?

## Ce que tu rends

Findings numérotés `P4-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne` du fichier final,
**preuve** (commande et sortie copiée, `grep -nF` pour toute présence ou absence), origine (amendement #678, ou texte
antérieur), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (les neuf ci-dessus, un par un, avec
ce qui a été fait ou pourquoi pas) — un « 0 finding » sans elle ne compte pas comme passe, et un axe déclaré exercé
sans commande copiée compte comme non exercé.
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p4-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p4-ciblee.md` — **ni fichier
temporaire**, ni dans `/tmp`, ni dans un worktree. Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`,
sous **aucune** forme et dans **aucun** worktree, `cargo`, `npm`, `npx`, `node`, `docker`, `mysql`/`mariadb` et
toute requête SQL, `gh` en écriture, `sed -i`, redirections `>`/`>>`/`tee` hors du rapport, et **tout interpréteur**
(`python`, `python3`, `perl`, `ruby`, `node`, `deno`, `bash`/`sh` sur un script ou avec `-c`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `sort`, `git log`/`show`/`diff`/`blame` (dont
`git -C /home/gcorbaz/devel/kesh-29-6 show d71c506b:<chemin>`), `gh api` et `gh issue view` en lecture, `pdftotext`
vers la sortie standard.

[#678]: https://github.com/guycorbaz/kesh/issues/678
