# Validation P5 (passe ciblée) — fiche 53-1a « La saisie ne passe pas deux fois » (#1355, P1)

**Modèle : Sonnet. Passe ciblée, une seule lentille, contexte frais.** (`CLAUDE.md` § *La passe ciblée* : la P4 (Haiku) a rendu trois MEDIUM et deux LOW, nés des correctifs de la P3, tous remédiés par **un seul commit**, qui ne touche que des fiches ; deux passes ciblées de suite changent de modèle, Haiku → Sonnet.)

## Ce que tu relis — et rien d'autre

Le **seul commit de remédiation de la P4**, sur la branche `story/53-1-ecriture-unique` (worktree `/home/gcorbaz/devel/kesh-53-1`). L'orchestrateur te fournit **en entrée** le diff aplati, produit par :

```sh
git -C /home/gcorbaz/devel/kesh-53-1 diff 060f239e8 HEAD -- \
  _bmad-output/implementation-artifacts/53-1a-saisie-pas-deux-fois.md \
  _bmad-output/implementation-artifacts/53-1b-conversion-pas-deux-fois.md
```

(`060f239e8` est la tête lue par la P4 ; `HEAD` est le commit de remédiation.) Rapport de la P4, pour savoir ce que chaque ajout prétend corriger : `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-1a-p4.md` (V4-1 à V4-5). Ne refais pas la validation de toute la fiche.

## La lentille unique : « ce qui vient d'être écrit est-il vrai au code ? »

Pour **chaque ligne ajoutée** du diff (`+`) :
1. Chaque citation `fichier:ligne`, chaque fait affirmé, chaque commande neuve : est-il exact au code de la branche ? Relis la ligne citée. En particulier : la séparation J2a / J2b et ce que `canWriteEntry` permet et interdit, contre `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte` (`:240-258` `track`, `:245`, `:274-279` `refreshRates`, `:286-294` l'effet de la date, `:298-313` `selectAccount`, `:346`, `:355`, `:479-487` `fetchMissingMonth`, `:625-636` `handleSubmit` et l'attente, `:659-664` le corps) — l'affirmation « aucune recherche ne peut naître ni se résoudre après J2a » est-elle vraie (cherche toute autre source d'appel de `track` ou de `refreshRates`, et tout `$effect`) ; le précédent `UpdateSupplierInvoiceRequest` (`crates/kesh-api/src/routes/supplier_invoices.rs:656-662`) ; la 48-2a et `reassign_entry_lines` (fiche `48-2a-reimputation-ecriture-serveur.md` sur la branche **locale** `story/48-2-reimputation-ecriture-manuelle`, `git show story/48-2-reimputation-ecriture-manuelle:_bmad-output/implementation-artifacts/48-2a-reimputation-ecriture-serveur.md`, l. 200-216 et 347-350) et `project_reassignment.rs:279-281` ; les quatre libellés de `journal-entry-form-submit` (`crates/kesh-i18n/locales/*/messages.ftl:317`) ; l'origine de M14, M15, M21 (`git show 2b3067d28:_bmad-output/implementation-artifacts/53-1-ecriture-pas-deux-fois.md`) et les M1, M3, M5 de la 53-1b ; la ligne ajoutée à la 53-1b sur `ConversionAssistant.svelte:251`, `:255`, `:256-259`.
2. La remédiation corrige-t-elle vraiment le finding de la P3 qu'elle vise, sans en créer un neuf (contradiction avec une autre section de la fiche, AC sans test, mutation intuable, tâche qui manque) ?
3. La frontière 53-1a / 53-1b reste-t-elle nette (la lecture de `details.entry` à la 53-1b seule ; aucune autre trace de la conversion dans la 53-1a hors renvois et pièces communes) ? La table J est-elle cohérente après la séparation (toute entrée et toute sortie menant à J2a ou J2b, plus aucun « J2 » nu ; AC-11, mutations M25 à M27) ?
4. Le manuel : si une ligne ajoutée parle du manuel (« Valider »), vérifie-la contre le `.tex` **et** le PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et `admin-manual.pdf`), **vers ta seule sortie standard**.

## Garde-fou obligatoire pour tout CRITICAL ou HIGH (gardé, quel que soit le modèle)

Avant de rendre un finding CRITICAL ou HIGH qui affirme l'**absence** d'un texte ou d'un code, ou la **présence** d'un anti-pattern, vérifie-le par `grep -nF "<chaîne exacte>" <fichier>` (`-F` obligatoire ; motif sur plusieurs lignes : une ligne discriminante, ou `grep -nFA <N>`) et **cite la sortie** dans le finding. Sans cette sortie, le finding est irrecevable. Le diff peut mêler des lignes de contexte : juge sur le fichier à `HEAD`, non sur le diff seul.

## Interdits

- **N'écris aucun fichier, nulle part**, et n'exécute **aucune commande qui écrit** : pas d'`Edit`/`Write`, **aucune redirection `>` ni `>>`, pas de `tee`**, pas de `git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git restore`, `git mv`, `git push`, pas de `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh` ni aucun script du dépôt, pas de `cargo`, pas de `npm`, pas de `python3`, pas de `docker` ni `docker compose`, pas de `sqlx`, pas de `mariadb`/`mysql`, pas de `curl`, pas de `make`.
- **Jamais `pkill`, `killall` ni `kill`** (ni par motif, ni par PID).
- Aucune commande `gh`.
- Commandes permises : `git diff`, `git show`, `git log`, `git grep`, `grep` (dont `grep -r` : le pathspec `git grep … -- 'crates/*/tests'` ne désigne rien dans ce dépôt), `sed -n`, `awk` (lecture), `cat`, `head`, `tail`, `wc`, `ls`, `pdftotext … -` vers la sortie standard.

## Rapport attendu

En français. Pour chaque finding : identifiant `V5-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), ligne du diff visée, constat **avec la ligne de code ou de fiche lue citée** (et la sortie `grep -nF` pour un CRITICAL ou un HIGH), correctif proposé. Puis le sort de V4-1 à V4-5 (remédié / partiellement / non). Puis le décompte par sévérité, et **la liste des axes exercés et non exercés** (les quatre points ci-dessus ; un axe non exercé dit pourquoi). Un « 0 finding » sans cette liste ne compte pas comme passe. L'orchestrateur range le rapport sous `/home/gcorbaz/devel/kesh-gate-logs/e531/validation-1a-p5.md`.
