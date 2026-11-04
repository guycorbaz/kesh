# Validation P4 — fiche v0172-annulees (Opus, passe ciblée, une lentille, lecture seule)

Tu es la lentille de la validation **P4, ciblée**, de la fiche BMAD **v0172-annulees** de Kesh (release 0.17.2, #868 :
repasser en brouillon une facture fournisseur annulée). Modèle : **Opus**. Contexte frais. Travaille en français soigné.

Historique : P1 (Sonnet) 0/0/7/8, P2 (Opus) 0/1/4/8, P3 (Sonnet) 0/0/2/5 ; chacune remédiée. Cette passe ne relit **pas**
la fiche entière : elle relit **la seule remédiation de la P3**, parce que c'est là que naissent les défauts suivants
(CLAUDE.md, § *La passe ciblée*).

## Périmètre

- Le diff : `git -C /home/gcorbaz/devel/kesh-0172a diff ea098675 HEAD -- _bmad-output/implementation-artifacts/` (en flux).
- Le rapport de la P3 : `/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p3.md` (V3-1 à V3-7), à lire **après** le diff.
- Pour vérifier une prémisse, le code et les documents à `d7ec2ea2`, par `git show d7ec2ea2:<chemin>` et `git grep … d7ec2ea2`,
  **en flux**. Pour situer une ligne du diff, la fiche et le registre à la tête par l'outil de lecture.

## La lentille — trois questions, sur chaque hunk du diff

1. **Le correctif ferme-t-il le finding ?** Pour V3-1 à V3-7, un par un, avec la ligne lue qui le prouve. En particulier :
   - V3-1 : la 18ᵉ clé (`audit-log-action-supplier-invoice-unvalidated`) est-elle dans la liste `K` de la commande d'AC-15
     **et** dans la liste « valeur modifiée (18) » ? **Rejoue la sonde** à `d7ec2ea2` (la commande d'AC-15, chaque fichier
     lu par `git show d7ec2ea2:crates/kesh-i18n/locales/<loc>/messages.ftl | …`) : obtiens-tu `clés=18 verbe=14` dans les
     quatre locales ? Une autre clé fournisseur porte-t-elle encore le verbe hors de `K` (cherche dans les quatre
     catalogues, famille `audit-log-action-supplier-invoice-*` comprise) ? Un repli en dur du libellé d'audit existe-t-il ?
   - V3-2 : `CANDIDATES_ATTENDUES` (`frontend/src/lib/shared/i18n-libelle-en-dur.test.ts:267`) est-il au tableau et à la
     tâche 5, avec une consigne de recompte exécutable (lis son préambule `:240-266`) ?
   - V3-3 : la topologie des exercices (§ *Topologie des exercices des montages*) est-elle **montable** et rend-elle, pour
     T3a, T3b, T3c, T3c', P4, P5, P6, le refus annoncé ? Déroule chaque cas contre `delete_in_tx`
     (`crates/kesh-db/src/repositories/journal_entries.rs:1822-1980`) et le pré-contrôle (tâche 3.5) ; vérifie aussi que
     l'annulation elle-même est possible dans l'ordre prescrit (états posés **après** l'annulation ; `cancel_in_tx`
     refuse un exercice clos), que la contre-passation datée du jour tombe dans `C`, et que les exercices ne se
     chevauchent pas (`fiscal_years`, lire la migration et `find_open_covering_date`).
   - V3-4 : la règle typographique est-elle exacte (lis les apostrophes réelles des clés citées dans le FTL fr-CH) ?
   - V3-5 : `README.md:42`, `docs/manual/fr/marketing-brochure.tex:449` (et son PDF), `docs/api-external.md:574`, `:961` :
     prémisses exactes ? un autre site hors manuels (`website/`, `CHANGELOG.md`, `docs/`) dit-il encore « dévalide » pour une
     facture fournisseur ?
   - V3-6 : C-v0172a-5 et C-v0172a-20 annotés sans réécriture de leur texte d'origine ?
   - V3-7 : l'argument de C-v0172a-25 tient-il au code (route `DELETE` fermée aux clés, `routes/supplier_invoices.rs` et
     `crates/kesh-api/src/lib.rs` ; écran de la fiche) ?
2. **Le correctif introduit-il un défaut neuf ?** Contradiction avec une autre section de la fiche ou du registre, prémisse
   fausse, numéro de ligne faux, test qui ne pourrait plus rougir, décompte faux.
3. **La propagation est-elle faite ?** Greppe les **valeurs** corrigées dans la fiche et le registre à la tête : `17`
   (dans le sens « nombre de clés »), `clés=17`, `verbe=13`, `dix-sept`, `quinze`, `Exercice 2031`, `withLabel`,
   `CANDIDATES_ATTENDUES`, `C-v0172a-1 à`. Les entrées **passées** du Change Log et les choix d'origine du registre gardent
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

`/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p4.md` (chemin absolu), en français :

1. En-tête : modèle, date, commits lus (`ea098675`, tête), fichiers lus.
2. Décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
3. Findings `V4-n` : sévérité, question (1, 2 ou 3), citation du diff, preuve (ligne lue copiée), correctif proposé.
4. Tableau V3-1 à V3-7 : fermé ? défaut neuf ?
5. **Axes exercés et non exercés** : les trois questions, et pour la question 1 chacun des sept findings, avec ce que tu as
   réellement lu ou exécuté. Un « 0 finding » sans cette liste ne compte pas comme passe.
