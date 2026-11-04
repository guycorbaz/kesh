# Validation P5 — fiche v0172-annulees (Sonnet, passe ciblée, une lentille, lecture seule)

Tu es la lentille de la validation **P5, ciblée**, de la fiche BMAD **v0172-annulees** de Kesh (release 0.17.2, #868 :
repasser en brouillon une facture fournisseur annulée). Modèle : **Sonnet**. Contexte frais. Travaille en français soigné.

Historique : P1 (Sonnet) 0/0/7/8, P2 (Opus) 0/1/4/8, P3 (Sonnet) 0/0/2/5, P4 (Opus, ciblée) 0/0/3/4 ; chacune remédiée.
Cette passe ne relit **pas** la fiche entière : elle relit **la seule remédiation de la P4**, parce que c'est là que naissent les défauts suivants
(CLAUDE.md, § *La passe ciblée*).

## Périmètre

- Le diff : `git -C /home/gcorbaz/devel/kesh-0172a diff d1707de9 HEAD -- _bmad-output/implementation-artifacts/` (en flux).
- Le rapport de la P4 : `/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p4.md` (V4-1 à V4-7), à lire **après** le diff.
- Pour vérifier une prémisse, le code et les documents à `d7ec2ea2`, par `git show d7ec2ea2:<chemin>` et `git grep … d7ec2ea2`,
  **en flux**. Pour situer une ligne du diff, la fiche et le registre à la tête par l'outil de lecture.

## La lentille — trois questions, sur chaque hunk du diff

1. **Le correctif ferme-t-il le finding ?** Pour V4-1 à V4-7, un par un, avec la ligne lue qui le prouve. Les trois
   MEDIUM sont désormais fermés par une **commande** dont la sortie est copiée dans la fiche : **rejoue chacune à
   `d7ec2ea2`, en flux, et compare sa sortie à celle copiée**.
   - V4-1 : la commande des littéraux de `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` ; le tri « dépôt »
     (`:267`, `:784`) / « sources synthétiques » est-il juste (lis les lignes `:850-1025`) ? préambule `:74-266`,
     `withLabel` `:260-265` exacts ? la tâche 5 recompte-t-elle `:784` ?
   - V4-2 : la commande sur `docs/api-external.md` rend-elle les neuf lignes annoncées ? le tri (changent / restent) et
     C-v0172a-26 sont-ils cohérents avec le texte de chaque ligne ?
   - V4-3 : la commande d'apostrophes (avec `K` d'AC-15 plus `supplier-invoices-unvalidate-confirm-body`) rend-elle les
     19 lignes copiées ? la règle par clé est-elle sans contradiction, et chaque clé neuve rattachée à une sœur réelle ?
   - V4-4 : la répartition des montages est-elle close (chaque test de la fiche classé une fois) et juste (P2, P3 :
     `A` clos sans clore `C`) ?
   - V4-5 à V4-7 : la phrase sur la page restée ouverte, le texte cible de la brochure, les traductions du libellé
     d'audit (de, it, en) cohérentes avec le vocabulaire d'AC-15 ?
2. **Le correctif introduit-il un défaut neuf ?** Contradiction avec une autre section de la fiche ou du registre, prémisse
   fausse, numéro de ligne faux, test qui ne pourrait plus rougir, décompte faux.
3. **La propagation est-elle faite ?** Greppe les **valeurs** corrigées dans la fiche et le registre à la tête : « par famille »,
   « même correction », « le jour même », `:255-265`, « restent les lignes des factures clientes », « dans les quatre
   locales », `C-v0172a-1 à`. Les entrées **passées** du Change Log et les choix d'origine du registre gardent
   leurs chiffres (annotations datées seulement) : ne les signale pas comme résidus.

## Sévérités

CRITICAL (le geste corromprait les livres ou ne peut pas aboutir), HIGH (AC faux ou intestable), MEDIUM (prémisse fausse,
contrôle qui ne décide pas, inventaire incomplet, test qui ne tuerait pas sa mutation), LOW (rédaction).

## Interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : pas de `cargo`, `npm`, `make`, `latexmk`, `docker`, `curl`, `sqlx`,
  `mariadb` ; **pas de `scripts/prepare-release.sh`** ni d'aucun script de `scripts/`.
- Pas de `git checkout`/`switch` (sous aucune forme), `git reset`, `git stash`, `git commit`, `git push`, `git tag`,
  `rebase` ; aucune commande `gh` qui écrit ; pas de `claude`, `pkill`, `killall` ; pas de `sed -i`, `perl -pi`.
- ⛔ **Aucune redirection** — ni `>`, ni `>>`, ni `2>`, ni `2>/dev/null`, ni `&>` : **`/dev/null` compris** —, ni `tee`,
  ni `-o` : toute sortie se lit en flux. **Aucun fichier temporaire**, nulle part : ni `/tmp`, ni le scratchpad de la
  session, ni un répertoire de travail. (Incident de la P2 : un `git show … > …/scratchpad/…` a écrit hors du dépôt.)
- Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.
- **Tu n'écris qu'un seul fichier** : ton rapport, par l'outil d'écriture, jamais par le shell.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p5.md` (chemin absolu), en français :

1. En-tête : modèle, date, commits lus (`d1707de9`, tête), fichiers lus.
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
3. Findings `V5-n` : sévérité, question (1, 2 ou 3), citation du diff, preuve (ligne lue copiée), correctif proposé.
4. Tableau V4-1 à V4-7 : fermé ? défaut neuf ?
5. **Axes exercés et non exercés** : les trois questions, et pour la question 1 chacun des sept findings (V4-1 à V4-7), avec ce que tu as
   réellement lu ou exécuté. Un « 0 finding » sans cette liste ne compte pas comme passe.
