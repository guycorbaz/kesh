# Prompt — validation P4, Story 32-1 (défauts courts : 32-1a, 32-1b, 32-1c, 32-1d) — passe ciblée

*Versionné le 2026-10-24. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille** — chasseur de
régressions sur la spécification —, en contexte frais, **Haiku**, braquée sur **le seul diff de la remédiation P3**.
Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-1`, branche `story/32-1`. Fiche :
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-38) ;
`_bmad-output/implementation-artifacts/sprint-status.yaml` (quatre clés `32-1*`). Rapports P3 :
`/home/gcorbaz/devel/kesh-gate-logs/321-validate-p3-R.md`, `321-validate-p3-F.md`.

**Le diff à relire, aplati, en une seule commande** (n'en lis pas d'autre : pas de séquence de commits) :

```sh
git -C /home/gcorbaz/devel/kesh-32-1 diff 8d4c29df 7e4b7b5f -- _bmad-output/implementation-artifacts/32-1-defauts-courts.md _bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md _bmad-output/implementation-artifacts/sprint-status.yaml
```

Les numéros de ligne d'un hunk sont ceux du fichier **après** `7e4b7b5f`. Pour lire le texte entier d'un passage,
lis le fichier au commit (`git show 7e4b7b5f:<chemin>`, ou le fichier du worktree, identique).

**Ce que la fiche prescrit au développement n'existe pas encore : son absence dans le code n'est pas un défaut.** Ce qui
se juge, c'est la vérité de ce que le diff affirme du code **actuel**, et ce que le diff a cassé ailleurs.

## Ce que la remédiation P3 a fait (à vérifier, non à croire)

- **Retrait de l'AC-a11** (décision de l'orchestrateur, C-32-1-38) : E-9 (3) — les helpers lexicaux `code()`,
  `argument_de`, `corps()` du `mod tests` de `crates/kesh-api/src/auto_backup.rs` — n'est plus corrigé ; renvoyé à #727.
  L'AC-a10 est **élargi** à des doc-comments qui écrivent la limite de ces helpers. M-a7a/M-a7b sorties ; huit mutations
  en A. Le réemploi de `crates/kesh-db/tests/lexique/mod.rs` est abandonné. Un **commentaire P3 sur #727** remplace celui
  de la P2.
- **#711** : se ferme toujours par la dernière des 32-1a et 32-1b, son corps de PR écrivant « E-9 (3) suivi par #727 ».
- **Motif du découpage** corrigé (C-32-1-37 amendé) ; grain métier de la 32-1a écrit (**6**).
- **LOW** : R2 (`i18n-keys.test.ts`, `sitesTotal`), R3 (`admin_pat_denied_e2e.rs:124`), R5 (phrase de l'espace disque),
  F3 (C-32-1-26), F4 (`CHANGELOG.md` commun), F5 (commande de l'AC-c9), F6 (`sprint-status.yaml`), F7 (`-p kesh-i18n`
  à l'AC-d5), F8 (`./api-error`), F9 (G22), F10 (200 illisible).

## La lentille — chasseur de régressions sur la spécification

**Le motif mesuré du dépôt : la sévérité se déplace vers ce qu'on vient d'écrire.** Pour chaque ligne **ajoutée** du
diff, deux questions, et une preuve pour chacune.

**A. L'affirmation est-elle vraie au code actuel ?** Ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie** dans
ton rapport. Au minimum, chacune de celles-ci :

1. **AC-a10 élargi** : les doc-comments de `code()` (`auto_backup.rs:1119-1120`), d'`argument_de` (`:1157-1158`), de
   `corps()` (`:1181`) sont-ils à ces lignes ? Le comportement que la fiche leur attribue est-il exact, à la lecture du
   corps des trois fonctions (`:1121-1128`, `:1159-1179`, `:1182-1188`) — lignes entières `//` seulement, troncature à
   la ligne `#[cfg(test)]`, parenthèses des chaînes et des littéraux comptées, coupe au premier `\n}` ? La « conséquence
   muette » est-elle juste (un jeton **exigé** lu dans un commentaire ou une chaîne satisfait la garde) ? L'AC dit-il bien
   « commentaires seuls », et rien d'autre de l'AC-a10 n'a-t-il été perdu (le paragraphe E-9 (2) de `gardes_possedes`) ?
2. **Inventaire, *Gardes lexicales d'`auto_backup.rs`*** : `main.rs:383` et `auto_backup.rs:669` portent-ils les jetons
   cités ? Les cinq fichiers lus (`main.rs`, `lib.rs`, `routes/admin.rs`, `routes/backup_schedule.rs`, `auto_backup.rs`)
   n'ont-ils de `/*` que les trois de `lib.rs` (l. 1179, 1186, 1196) ?
3. **Inventaire, copies** : `crates/kesh-api/tests/admin_pat_denied_e2e.rs` l. 115-124 (doc-comment et
   `strip_line_comments`) et l. 193 (`lib_rs_has_no_double_slash_inside_literals`) ?
4. **Commentaire P3 sur #727** (*Commentaires à verser*) : chaque site et chaque fait — `code()` l. 1121, `argument_de`
   l. 1159, `corps()` l. 1182 ; `kesh-db/src/backup.rs:1950-1951`, `repositories/closing_checks.rs:922`, `:928`,
   `tests/test_schema_guard.rs:818-827` ; `bank_account_closed_inventory.rs:61` (`#[path]`) et `letterings_lexical.rs`
   (`mod lexique`) ; `decouper` aux blocs **non** imbriqués (lis son bras `/*` dans `lexique/mod.rs`) ; le « piège » (une
   déclaration `#[cfg(test)]` en tête tronque `code()`) est-il exact au regard de `code()` et de `gardes_possedes`
   (`auto_backup.rs:1202-1267`) ?
5. **AC-b2** : `api-error.ts` l. 5-9 (dix-huit fichiers de test) ; `AdminRestorePanel.svelte:20` importe-t-il
   `isApiError` de `api-client` ? `routes/backup_schedule.rs:144-146` : le 200 rend-il l'état après la tentative quel
   que soit son statut ? La limite assumée (le message « pas parvenue … ou s'écrit encore » inexact pour le 200
   illisible) contredit-elle une autre phrase de l'AC-b2, de l'AC-b7 (*Ce que l'écran montre*), du registre (C-32-1-34)
   ou de la valeur fr-CH prescrite ?
6. **AC-b7** : `seuil_d_espace` (`auto_backup.rs:496-500`) part-il de `max(dernière, documents)` ? La phrase neuve
   (« … quand celle-ci est plus grande que les documents. Sinon — en particulier avant la première … ») est-elle vraie
   dans les **trois** cas (dernière ≥ documents, dernière < documents, aucune réussite), y compris documents = 0 ? Le
   contrôle aplati (« un cinquième de la taille des documents ») tient-il encore ? G22 (`textes_coherents.rs` l.
   1599-1663) : les quatre motifs cités (deux exigés, deux refusés) sont-ils exacts, et la phrase prescrite les
   respecte-t-elle ?
7. **AC-b8** : `frontend/src/lib/shared/i18n-keys.test.ts` l. 715 (`sitesTotal: 2332`) et l. 1025 ; la commande de
   recompte citée est-elle celle du fichier ? « +2 » : les deux sites nommés sont-ils les seuls `i18nMsg(` neufs que
   la partie B prescrit (relis AC-b1 à AC-b5) ? La ligne *Modules* de la 32-1b (5 au grain métier) tient-elle avec
   `shared/i18n-keys.test.ts` ?
8. **AC-c9** : la commande est-elle **bien formée** (essaie-la mentalement : `$'…'` de bash dans un argument de `sed`,
   `\xe2\x86\x92` = U+2192, `+` littéral en regex de base, `cmp` sur deux substitutions de processus, `git show … >`
   vers un chemin absolu de `kesh-gate-logs/`) et **discriminante** (rend-elle « identiques » par construction dans un
   cas) ? Le témoin « Conforme au Code des obligations suisse » est-il dans `marketing-brochure.tex` ?
9. **AC-d5** : `parity_between_locales` est-il dans `crates/kesh-i18n/src/loader.rs` (l. 771) ?
10. **Modules de la 32-1a** (**6** au grain métier, **3** au grain des crates) contre la partie A **après** retrait et
    *Fichiers touchés* ; le grain est-il celui qu'emploie la ligne *Modules* de la 32-1b ?

**B. Le diff a-t-il laissé un résidu ou cassé ailleurs ?** Le retrait d'une AC est une écriture neuve.

1. **Résidus du retrait** : `grep -n 'AC-a11\|lexique\|M-a7\|decouper\|en tête\|letterings_lexical\|bank_account_closed_inventory'`
   sur la fiche, le registre et `sprint-status.yaml`. Chaque occurrence doit être soit **historique** (entrée passée du
   Change Log, registre C-32-1-6, 20, 32 avec leur annotation datée, segments datés du `sprint-status`), soit un
   **renvoi** au retrait, soit le texte du commentaire à verser. Une occurrence qui **prescrit** encore au développement
   (AC, tâche, gate, *Fichiers touchés*, Dev Notes, *Risques*, *Pourquoi quatre clés*) est un finding.
2. **Comptes** : « huit » mutations en A (AC-a13) contre la liste des AC ; le Change Log P3 (2 MEDIUM, 11 LOW distincts ;
   R 0/0/1/4, F 0/0/2/8) recompté depuis les en-têtes des deux rapports P3 ; les compteurs du registre (en-tête :
   C-32-1-1 à 38).
3. **#711** : la phrase de fermeture (*Pourquoi quatre clés*, mots-clés de PR ; *Origine* ; C-32-1-38) est-elle
   cohérente partout — « dernière des 32-1a et 32-1b », commentaire sur #727 versé avant, `refs #711` à défaut ? Le bilan
   « −3 + 3 = 0 issue, −1 bug » est-il resté juste ? Aucun mot-clé de fermeture dans le message du commit
   (`git log -1 --format=%B 7e4b7b5f | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`).
4. **Historique** : la remédiation a-t-elle **réécrit** une entrée passée (Change Log P1, P2, Découpage ; registre) au
   lieu de l'annoter ? Une annotation datée dit-elle vrai à la date qu'elle porte ?
5. **Registre** : C-32-1-38 et les amendements de C-32-1-26, 37 et annotations de C-32-1-6, 20, 32, 35 contredisent-ils
   un passage de la fiche ? Une « Écartée » de C-32-1-38 est-elle reprise ailleurs comme « retenue » ?
6. **Manuel** (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : les phrases neuves prescrites au manuel
   d'administration (AC-b7, *Espace disque requis*) seront-elles vraies après le correctif, et contrôlables au PDF aplati
   par la commande des Dev Notes (§ *Contrôle aplati d'un PDF*) ? Vérifie le texte **actuel** du PDF : `pdftotext
   docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<fragment>"` — témoin positif
   d'abord (« trop grande pour être restaurée par l'écran »).

## Ce que tu rends

Findings `P4-1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, fichier et ligne (après `7e4b7b5f`), site du dépôt, **preuve
(commande et sortie copiée)**, correction proposée ; pour chacun, dis s'il vient de la remédiation P3 (ce diff) ou d'avant.
⛔ **Avant d'affirmer qu'un texte manque ou qu'un patch n'a pas été appliqué, vérifie au fichier par `grep -nF`** (règle
*Haiku-specific guardrails* de `CLAUDE.md`) et copie la sortie. ⛔ **La liste des axes exercés ET non exercés** (A1 à
A10, B1 à B6), avec pour chacun ce qui a été lu — sans elle, « 0 finding » ne compte pas.

Rapport écrit **d'un seul jet**, un seul fichier, **par l'outil d'écriture de fichier** (jamais par une redirection du
shell) : `/home/gcorbaz/devel/kesh-gate-logs/321-validate-p4.md`. Dernier message : chemin, bilan par sévérité (recompté
sur ta liste), une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun fichier** hors de ton rapport ; **aucun fichier temporaire** ; **aucune redirection** du shell vers un
fichier (`>`, `>>`, `tee`, `-o`) — la commande de l'AC-c9, qui en contient une, se juge **à la lecture**, elle ne
s'exécute pas. Aucun `git checkout`/`switch` sous aucune forme. **Aucune commande qui écrit, compile ou exécute** :
`scripts/*` (dont **`scripts/prepare-release.sh`**, qui bumpe les versions du workspace — à **lire** seulement), `make`,
`latexmk`, `lualatex`, `xelatex`, `cargo`, `rustup`, `npm`, `npx`, `node`, `docker`, `curl`, **`python3`, `python`,
`perl`, `ruby`**, `mariadb`/`mysql`, `sqlx`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en
écriture (`pr`, `issue create|edit|close|comment`). `pdftotext` vers la sortie standard seulement (`-` comme destination).
`sed` sans `-i`, en filtre seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture de
fichiers, `grep`, `sed -n` et `sed` en filtre de tube, `awk` en lecture, `wc`, `tr`, `od`,
`git log|show|diff|blame|merge-base`, `gh issue view`, `pdftotext … -`.
